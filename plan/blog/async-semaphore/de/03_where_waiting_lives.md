# Teil 3 — Wo lebt das Warten?

Wenn ein Task ein Permit verlangt und keines frei ist, wartet er. Der Satz gleitet
leicht vorbei, aber „warten" leistet darin enorme Arbeit. Warten ist eine physische
Angelegenheit: Irgendwo muss irgendeine Maschinerie sich merken, dass genau dieses
Stück Arbeit schläft, es am CPU-Verbrennen hindern, während es schläft, und wissen,
wie man es weckt, wenn es dran ist. Jedes Synchronisationsprimitiv ist darunter
eine Antwort auf die Frage, wer dieses Gedächtnis führt. Die Async-Antwort ist
seltsam genug, dass sich zuerst ein Blick auf die gewöhnliche lohnt.

## Wie ein Thread wartet

In einem Thread-Programm gehört die Maschinerie dem Betriebssystem. Ein Thread,
der warten muss, sagt es dem Kernel — auf Linux über einen Syscall namens
`futex` — und der Kernel nimmt ihn von der CPU:

```rust
// ein Thread-basiertes acquire, im Kern:
fn acquire(&self) {
    loop {
        if try_take_permit() { return; }
        futex_wait(&self.counter);   // Syscall: der Kernel parkt DIESEN THREAD
    }                                // geweckt durch futex_wake eines Releasers
}
```

Der Thread kostet nichts, solange er geparkt ist. Wird ein Permit freigegeben,
ruft die Gegenseite `futex_wake`, der Kernel wählt einen Schläfer, und der
geweckte Thread macht an genau der Zeile weiter, an der er stehen blieb. Die
schweren Teile — sich merken, wer schläft; wählen, wen man weckt — sind das
Problem des Kernels.

Das funktioniert aus einem Grund, den man leicht übersieht: Das, was wartet, und
das, was der Kernel scheduled, sind *dasselbe Objekt*. Ein Thread ist eine
Kernel-Ressource. Natürlich kann der Kernel einen parken.

Und jetzt brich diese Annahme.

## Der Kernel hat von deinen Tasks noch nie gehört

Eine Async-Runtime fährt, sagen wir, acht Worker-Threads und fünfzigtausend
Tasks. Ein Task ist kein Thread — er ist ein Userspace-Wert, eine pausierte State
Machine, die ein Worker kurz pollt und beiseitelegt. Der Kernel scheduled die acht
Worker. Die fünfzigtausend Tasks sind für ihn unsichtbar.

Angenommen, `acquire` ruft tief in irgendeinem Task trotzdem `futex_wait`. Der
Kernel kann keinen *Task* parken; er parkt, was er kennt — den Worker-Thread, auf
dem der Task zufällig lief. Ein wartender Task hat gerade einen ganzen Worker
beschlagnahmt, und die sechstausend anderen Tasks, die diesem Worker zugeteilt
sind, stranden hinter einem Warten, das nichts mit ihnen zu tun hat. Acht wartende
Tasks auf einer Acht-Worker-Runtime frieren das Programm fest: jeder Worker
geparkt, ein Berg lauffähiger Arbeit, niemand mehr da, sie auszuführen.

Selbst ohne den Freeze sind zwei Dinge kaputt. Die Runtime hat `poll` als
gewöhnliche Funktion aufgerufen und erwartet eine Antwort — fertig oder noch
nicht — und ein `poll`, das seinen Thread parkt, antwortet nicht; es kehrt einfach
nie zurück. Und wenn der Kernel den geparkten *Thread* irgendwann weckt: Welcher
*Task* sollte weitermachen? Das hat niemand irgendwo notiert, wo die Runtime es
sehen könnte.

Die Schlussfolgerung ist strukturell: **Das Warten muss eine Ebene nach oben.**
Der Kernel merkt sich wartende Threads; etwas im Userspace muss sich wartende
Tasks merken. Für eine Semaphore ist dieses Etwas die Semaphore selbst.

## Der Vertrag, der es möglich macht

Rusts Async-Modell gibt der Semaphore zwei Werkzeuge — `Poll::Pending` und den
`Waker` — verpackt in eine Regel, von der Anfänger routinemäßig annehmen, sie
könne nicht stimmen:

> Nachdem ein Future `Pending` zurückgegeben hat, wird es nicht wieder gepollt —
> überhaupt nicht — bis sein `Waker` aufgerufen wird.

Kein periodisches Nachschauen, kein Hintergrund-Sweep, kein Timeout als letzte
Rettung. Ein `Pending`-Future, dessen Waker niemand ruft, schläft für immer, und
das ist kein Defekt — es ist dieselbe Disziplin wie beim Futex, eine Ebene höher.
*Schlafe still; werde ausdrücklich geweckt.*

Für die Semaphore diktiert der Vertrag beide Hälften des Designs auf einmal, und
jetzt können wir sie hinschreiben. Der Zustand ist ein Zähler plus eine Queue von
Wakern, gemeinsam bewacht:

```rust
pub struct Semaphore {
    state: Mutex<State>,
}

struct State {
    permits: usize,
    waiters: VecDeque<Waker>,     // ← hier lebt das Warten, physisch
}
```

Die Acquire-Seite ist das benannte Future aus Teil 2, und sein `poll` erledigt die
Prüfung und, falls nötig, die Einschreibung:

```rust
impl Future for Acquire<'_> {
    type Output = Result<SemaphorePermit<'a>, AcquireError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.semaphore.state.lock().unwrap();

        if state.permits > 0 {
            state.permits -= 1;
            return Poll::Ready(Ok(SemaphorePermit { semaphore: self.semaphore }));
        }
        state.waiters.push_back(cx.waker().clone());   // „warten" = den Waker hinterlegen
        Poll::Pending
    }
}
```

Und die Release-Seite muss wecken — ein Permit auf einen Zähler zu legen, den nie
wieder jemand ansieht, hilft niemandem:

```rust
pub fn add_permits(&self, n: usize) {
    let mut state = self.state.lock().unwrap();
    state.permits += n;
    for _ in 0..n {
        match state.waiters.pop_front() {
            Some(waker) => waker.wake(),   // ← ohne diese Zeile: Deadlock
            None => break,
        }
    }
}
```

Das ist eine funktionierende Async-Semaphore in etwa dreißig Zeilen. Sie enthält
außerdem zwei Bugs und einen Deadlock auf Abruf, alle drei unsichtbar in einem
ruhigen Test. Sie zu beheben ist eine Sache von drei Regeln.

## Drei Regeln, die jeder auf die harte Tour lernt

**Prüfen und Einschreiben unter EINEM Lock.** Sieh dir an, was das `poll` oben
richtig macht: Permitprüfung und `push_back` passieren unter einem einzigen
`lock()`. Hier ist eine Version, die fast identisch aussieht und kaputt ist — der
Unterschied ist ein einziges `let`:

```rust
// KAPUTT — Prüfung und Einschreibung laufen unter VERSCHIEDENEN Locks:

let permits = self.semaphore.state.lock().unwrap().permits;
//            └──────────────┬──────────────────┘
//            das liefert einen MutexGuard — der Lock wird nur gehalten,
//            solange der Guard lebt. Aber der Guard landet in keiner
//            Variable, er ist ein TEMPORARY: Rust droppt ihn am Ende
//            dieses Statements, am `;` — und einen MutexGuard droppen
//            heißt ENTSPERREN. Nettoeffekt der Zeile: lock, `permits`
//            kopieren, unlock.

if permits == 0 {
    //  ← der Lock wird hier NICHT gehalten. Ein Release kann in diese
    //    Lücke fallen: Es sieht eine leere Queue, weckt niemanden, und
    //    sein Permit liegt im Zähler — wo wir schon nachgesehen haben
    //    und nie wieder nachsehen werden.

    self.semaphore.state.lock().unwrap()          // eine brandneue Lock-Übernahme
        .waiters.push_back(cx.waker().clone());   // einschreiben — zu spät
    return Poll::Pending;                          // für immer schlafen
}
```

Das ist das klassische *Lost Wakeup*, und in Rust kommt es oft genau durch diese
Tür: Ein `MutexGuard`, der nie an eine Variable gebunden wurde, stirbt am
Semikolon, und der Lock öffnet sich lautlos zwischen zwei Statements, die atomar
gemeint waren. Die korrekte Version unterscheidet sich um eine Bindung:

```rust
let mut state = self.semaphore.state.lock().unwrap();
//  └── der Guard hat jetzt einen NAMEN und lebt bis zum Ende des Scopes —
//      alles darunter passiert in einer durchgehenden Critical Section
if state.permits > 0 { … }
state.waiters.push_back(cx.waker().clone());
Poll::Pending
// `state` droppt hier → unlock, NACH Prüfung und Einschreibung
```

Eine Lock-Übernahme deckt Blick und Einschreibung ab, also kann kein Release
dazwischen landen.

**Den Waker bei jedem Poll auffrischen.** Der Waker, den die Runtime übergibt, ist
nicht garantiert von Poll zu Poll dasselbe Objekt — Kombinatoren wie `select!`
wickeln Waker ein und ersetzen sie; Tasks wandern zwischen Workern. Das
Kleingedruckte des Vertrags: Es zählt der Waker des *jüngsten* Polls. Ein
Wartender, der erneut gepollt wird, während er noch wartet, muss seinen
hinterlegten Waker aktualisieren:

```rust
// beim Re-Poll, noch wartend:
if !stored.will_wake(cx.waker()) {
    *stored = cx.waker().clone();   // will_wake spart nur den Clone, wenn gleich
}
```

Behalte einen veralteten Waker, und der spätere Weckruf weckt womöglich gar
nichts — der Wartende schläft neben dem Permit, das für ihn gedacht war.

**Außerhalb des Locks wecken.** `waker.wake()` führt fremden Code aus — die
Interna der Runtime, manchmal mehr. Manche Executor pollen den geweckten Task
*synchron, innerhalb des `wake()`-Aufrufs*. Ruft dieser Task sofort `acquire`,
nimmt er den Lock der Semaphore — den der Releaser noch hält, während er weckt.
Die Lösung formt `add_permits` um:

```rust
pub fn add_permits(&self, n: usize) {
    let mut to_wake = Vec::new();
    {
        let mut state = self.state.lock().unwrap();
        state.permits += n;
        for _ in 0..n {
            match state.waiters.pop_front() {
                Some(waker) => to_wake.push(waker),   // unter dem Lock einsammeln…
                None => break,
            }
        }
    }                                                 // …Lock hier freigegeben…
    for waker in to_wake {
        waker.wake();                                 // …dann fremden Code ausführen
    }
}
```

## Was der Umzug nach oben kostet

Nichts — das ist die Überraschung. Es liegt nahe, die Userspace-Variante für die
Billigversion der „echten" Kernel-Variante zu halten. Das Gegenteil stimmt:

| | Thread + Futex | Task + Waker |
|---|---|---|
| einschlafen | ein Syscall | Waker hinterlegen, `Pending` zurückgeben |
| jemanden wecken | ein Syscall | ein Funktionsaufruf |
| Kosten im geparkten Zustand | ein ganzer OS-Thread samt Stack | eine pausierte State Machine |

Beide Richtungen des Umlaufs bleiben im Userspace. Eine Markierung, bevor es
weitergeht: Alles hier sitzt hinter einer einzigen `Mutex`.
Produktionsimplementierungen halten den Zähler in einem Atomic, damit ein
unumkämpftes `try_acquire` nie lockt — zum Preis heikler Races zwischen Atomic und
Queue. Diese Optimierung ändert keine der kommenden Fragen, also bleibt der Lock.

## Die Frage, die die Queue als Nächstes stellt

Die Semaphore erinnert sich jetzt an ihre Wartenden. Wenn also ein Permit
zurückkommt und drei Tasks warten, hält die Release-Seite eine Queue und eine
Entscheidung: *Wer bekommt es?*

Der Code oben hat bereits entschieden, ohne zu fragen — `pop_front` einen Waker,
wecken, und den geweckten Task mit allen anderen um das Permit im Zähler rennen
lassen. Das ist simpel, es ist sogar korrekt, und es verbirgt einen Fehlermodus,
den kein funktionaler Test dir zeigt, ein Lasttest in Produktion aber an einem
Nachmittag findet. Diese Weggabelung ist Teil 4.

---

*Weiter: [Teil 4 — Fairness: Wer bekommt das freie Permit?](04_fairness.md) · [Index](00_index.md)*

*English: [`../en/03_where_waiting_lives.md`](../en/03_where_waiting_lives.md)*
