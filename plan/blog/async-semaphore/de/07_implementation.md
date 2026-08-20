# Teil 7 — Aufschreiben, und ihm trauen

Sechs Teile lang hat das Design still Regeln angesammelt. An einem Ort
versammelt: Ein eingereihter Wartender nimmt nie aus dem Zähler. Ein gedroppter
Wartender gibt zurück, was er besitzt. Ein Permit wird genau einmal konsumiert.
Ein Node wird ausgehängt, bevor sein Speicher verschwindet. Ein freigegebenes
Permit geht zuerst an die Spitze der Schlange, dann in den Zähler. Close lässt
jeden Wartenden genau einmal scheitern.

Nichts auf der Liste ist für sich schwierig. Schwierig ist, wer gehorchen muss:
vier verschiedene Funktionen — `poll`, der Release-Pfad, `Drop` und `close` — die
denselben geteilten Zustand aus verschiedenen Richtungen anfassen. Naiv
geschrieben trägt jede Funktion ihre private Kopie der Regeln, und die Korrektheit
wohnt in den *Absprachen zwischen* ihnen: `Drop` muss wissen, ob Release diesen
Wartenden schon zugeteilt hat; `poll` muss wissen, was `Drop` später tun wird.
Ändere eine Funktion, verifiziere die anderen drei neu. Solcher Code kann jeden
Test bestehen und trotzdem unwartbar sein, weil die Invarianten nur in den Lücken
zwischen den Funktionen existieren, wo kein Leser sie sieht.

Die letzte Idee dieser Serie ist, was man dagegen tut.

## Der Lebenszyklus wird ein Feld

Teil 5 endete damit, dass `granted: bool` knarzte — eine Drei-Wege-Unterscheidung,
gequetscht in einen Boolean. Befördere sie. Das Lebenszyklus-Diagramm wird ein
Typ, und der Typ wohnt im Node:

```rust
enum WaiterState {
    Idle,      // erzeugt, nie eingereiht
    Waiting,   // in der Queue, besitzt seinen Platz in der Reihe
    Granted,   // aus der Queue, ein Permit liegt auf seinen Namen
    Done,      // besitzt nichts — konsumiert oder gecancelt
}

struct Waiter {
    waker: Option<Waker>,
    state: WaiterState,               // ← ersetzt `granted: bool`
    prev:  Option<NonNull<Waiter>>,
    next:  Option<NonNull<Waiter>>,
    _pin:  PhantomPinned,
}
```

Jede Regel aus dem Stapel wird eine *Transition* — ein Pfeil, einer Funktion
gehörend:

| wer | findet den Wartenden in | tut | hinterlässt ihn in |
|---|---|---|---|
| `poll` | Idle, Permit frei | nimmt aus dem Zähler | Done |
| `poll` | Idle, keins frei | reiht sich ein | Waiting |
| Release | Waiting, vorne | teilt Permit zu, weckt | Granted |
| `poll` | Granted | konsumiert sein Permit | Done |
| `Drop` | Waiting | hängt seinen Node aus | Done |
| `Drop` | Granted | gibt sein Permit weiter | Done |
| `Drop` | Idle oder Done | nichts — nichts wird besessen | — |
| `close` | Waiting | weckt mit Fehler | (sein eigenes poll hängt aus) |

Und jede Funktion wird ein `match`, das ihre Zeilen abschreibt. `poll`, in voller
Form:

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    // SAFETY: Wir bewegen nie etwas aus `this` heraus (Teil 6s Versprechen)
    let this = unsafe { self.get_unchecked_mut() };
    let mut state = this.semaphore.state.lock().unwrap();

    match this.node.state {
        WaiterState::Idle if state.closed => {
            this.node.state = WaiterState::Done;
            Poll::Ready(Err(AcquireError))
        }
        WaiterState::Idle if state.permits > 0 => {
            state.permits -= 1;
            this.node.state = WaiterState::Done;
            Poll::Ready(Ok(SemaphorePermit { semaphore: this.semaphore }))
        }
        WaiterState::Idle => {
            this.node.waker = Some(cx.waker().clone());
            this.node.state = WaiterState::Waiting;
            let node = NonNull::from(&mut this.node);
            unsafe { state.queue.push_back(node) };      // gepinnt → Adresse ist final
            Poll::Pending
        }
        WaiterState::Waiting if state.closed => {
            unsafe { state.queue.unlink(NonNull::from(&mut this.node)) };
            this.node.state = WaiterState::Done;
            Poll::Ready(Err(AcquireError))
        }
        WaiterState::Waiting => {
            this.node.update_waker(cx.waker());          // Teil 3s Auffrisch-Regel
            Poll::Pending
        }
        WaiterState::Granted => {
            this.node.state = WaiterState::Done;         // ← die Anti-Präge-Zeile
            Poll::Ready(Ok(SemaphorePermit { semaphore: this.semaphore }))
        }
        WaiterState::Done => unreachable!("polled after completion"),
    }
}
```

`Drop` schreibt seine drei Zeilen ab:

```rust
impl Drop for Acquire<'_> {
    fn drop(&mut self) {
        let mut to_wake = WakerList::new();
        {
            let mut state = self.semaphore.state.lock().unwrap();
            match self.node.state {
                WaiterState::Waiting => {
                    // ich besitze einen Platz in der Reihe — nimm meinen Node mit
                    unsafe { state.queue.unlink(NonNull::from(&mut self.node)) };
                }
                WaiterState::Granted => {
                    // ich besitze ein Permit, das ich nie konsumieren werde — gib es weiter
                    release_one(&mut state, &mut to_wake);
                }
                WaiterState::Idle | WaiterState::Done => {}   // ich besitze nichts
            }
        }
        to_wake.wake_all();     // außerhalb des Locks — Teil 3s Regel
    }
}
```

Und Release, geteilt zwischen `add_permits` und dem Granted-Arm oben:

```rust
fn release_one(state: &mut State, to_wake: &mut WakerList) {
    match state.queue.pop_front() {
        Some(mut node) => unsafe {
            let node = node.as_mut();
            node.state = WaiterState::Granted;       // das Permit wandert in den Record
            if let Some(w) = node.waker.take() { to_wake.push(w); }
        },
        None => state.permits += 1,                  // Überlauf nur, wenn niemand wartet
    }
}
```

Lies Teil 5s zwei Albträume direkt aus der Tabelle ab. Der Leak — ein zugeteilter
Wartender, gedroppt im Weck-Lauf-Fenster — ist behandelt, weil der Pfeil
`Drop`-bei-Granted *existiert*. Die doppelte Rückgabe ist unmöglich, weil
Konsumieren den Zustand auf Done bewegt und die Zeile `Drop`-bei-Done *nichts*
tut. Der eine Bug ist ein Pfeil, den es gibt; der andere ein Pfeil, den es nicht
gibt. Das kann jeder auditieren, Zeile für Zeile.

Das Platzierungsdetail, das alles trägt: Der Zustand wohnt im *Node* — nicht in
den privaten Feldern des Futures — weil die Release-Seite Wartende über
Queue-Pointer erreicht und Zustände durch sie lesen und kippen muss. Was beide
Seiten sehen müssen, kommt in den Node; der Node wird nur unter dem Lock berührt;
dieser Satz ist die gesamte Nebenläufigkeitsgeschichte der State Machine.

## Den sicheren Teilen trauen

Die Transitionstabelle hat eine angenehme Eigenschaft: Jeder Pfeil ist ein Test.
Treibe `poll` von Hand — keine Runtime, keine Threads, kein Schlafen — und jede
Zeile wird eine deterministische Behauptung:

```rust
#[test]
fn cancelled_after_grant_returns_the_permit() {   // der Teil-5-Bug, für immer festgenagelt
    let sem = Semaphore::new(0);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(&waker);

    let mut fut = Box::pin(sem.acquire());        // Box::pin BESITZT das Future,
                                                  // drop() unten droppt es also wirklich
    assert!(fut.as_mut().poll(&mut cx).is_pending());  // Idle → Waiting
    sem.add_permits(1);                                // Waiting → Granted
    drop(fut);                                         // gedroppt im Fenster!

    assert_eq!(sem.available_permits(), 1);       // Drop@Granted hat weitergegeben ✓
}
```

Drei Zeilen Handlung, ein Assert, Mikrosekunden Laufzeit — und es schlägt laut
fehl, falls je jemand den Weitergabe-Pfeil bricht. Jedes Verhaltensversprechen des
Designs lässt sich so festnageln, dauerhaft.

## Den unsicheren Teilen trauen

Teil 6s Pointer-Chirurgie hat etwas hinterlassen, das Tests nicht erreichen, und
es verdient eine Feststellung ohne Trost: **`unsafe`-Code kann falsch sein und
jeden Test bestehen, den du schreibst.** Undefined Behavior ist kein falscher
Output, gegen den man asserten kann — es ist eine verletzte Annahme, die der
heutige Compiler auf der heutigen Maschine zufällig in funktionierenden Code
übersetzt. Deine Testsuite führt eine Übersetzung des Programms aus. Die
Verletzung wohnt in den Übersetzungen, die du nicht ausgeführt hast: ein anderes
Optimierungslevel, ein neuerer Compiler, eine andere Plattform.

Also bekommen die unsicheren Teile eine Prüfung, die zum Versprechen passt. Für
Speicherversprechen — jede Dereferenzierung trifft eine lebende Allokation, jeder
Schreibzugriff läuft durch einen Pointer mit Schreibrecht, kein verbotenes
Aliasing — ist das Werkzeug **Miri**:

```
$ cargo +nightly miri test
```

Es führt dieselben deterministischen Tests von oben auf einer abstrakten Maschine
aus, die verfolgt, was echte Ausführung nicht sehen kann, und verwandelt Teil 6s
Pointer-Argument („erst nach dem Pinnen erzeugt, vor der Deallokation zerstört,
nur unter dem Lock berührt") von einem Argument in eine mechanische Prüfung.

Für Ordnungsversprechen — Korrektheit unter jeder Thread-Verzahnung — ist das
Werkzeug **loom**. Unser Design gibt ihm wenig zu finden, und der Grund ist
bemerkenswert: Alles Geteilte sitzt hinter einer `Mutex`, also kollabieren die
Verzahnungen zu „wer hat zuerst den Lock genommen". Das hat die
Ein-Lock-Vereinfachung gekauft. Der Tag, an dem der Zähler aus dem Lock in einen
atomaren Fast-Path wandert — die in Teil 3 markierte tokio-Optimierung — ist der
Tag, an dem loom aufhört, optional zu sein.

## Das Ganze, einmal abgeschritten

Ein Aufrufer schreibt eine Zeile:

```rust
let permit = sem.acquire().await?;
```

und darunter, in der Reihenfolge ihres Auftritts: ein Permit, das sich selbst
zurückgibt, weil Pools sonst lecken (Teil 2). Ein benanntes Future, weil
Cancellation ein `Drop` braucht (Teil 2). Eine Waker-Queue hinter einem Lock, weil
der Kernel keinen Task parken kann (Teil 3). Hand-off an die Spitze dieser Queue,
weil Rennen unter Last aushungert (Teil 4). Ein Lebenszyklus, der weiß, was jeder
Wartende besitzt, weil Futures mitten im Warten verschwinden (Teil 5). Nodes, die
in den Futures selbst wohnen, gepinnt, weil der Hot Path sich den Allokator nicht
leisten kann (Teil 6). Ein Zustandsfeld, das vier Funktionen regiert, geprüft von
handgetriebenen Tests und Miri (dieser Teil).

Reiß irgendeine Schicht heraus, und ein bestimmter Use Case aus Teil 1
zerbricht. Kein Schritt brauchte Genialität — nur die Weigerung, eine Frage zu
überspringen.

Wer dieses Skelett in Produktionsmuskeln sehen will, lese tokios
`batch_semaphore.rs` neben dieser Serie — gebatchte Wakeups, ein atomarer
Fast-Path, `acquire_many` — man erkennt jeden Knochen wieder. Und die nächste
Serie beginnt, wo der Komfort dieser endet: eine Queue mit mehreren Produzenten,
mehreren Konsumenten und keinem Lock, hinter dem man sich verstecken kann — wo
Korrektheit vollständig auf atomaren Orderings ruht, und die erste Frage eine ist,
die diese Serie nie stellen musste: Was heißt „vorher" überhaupt, ohne Lock?

---

*Das war die Serie. · [Index](00_index.md)*

*English: [`../en/07_implementation.md`](../en/07_implementation.md)*
