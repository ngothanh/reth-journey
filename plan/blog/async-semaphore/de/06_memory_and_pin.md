# Teil 6 — Wo die Wartenden wohnen, und wofür Pin da ist

Jeder Wartende trägt ein kleines Bündel Fakten: seinen Waker, seinen
Lebenszyklus-Zustand, seinen Platz in der Reihe. Teil 5 nannte es „my record" und
vermied sorgfältig zu sagen, wo es wohnt. Dieses Ausweichen endet hier, denn die
Adresse entscheidet, ob `acquire` allokiert — und Teil 1 hat eine Semaphore in
jeden begrenzten Channel gesteckt, auf den Hot Path jedes `send`. Eine
Heap-Allokation ist dort der falsche Gast: meist schnell, gelegentlich sehr
langsam, immer unvorhersehbar. Latenz-Tails bestehen aus genau so etwas.

Für den Record gibt es zwei mögliche Zuhause.

## Die naheliegende Adresse: in der Semaphore

Lass die Semaphore die Records ihrer Wartenden besitzen — eine wachsende Sammlung
davon, plus eine Liste für die Ankunftsreihenfolge, und jedes Future trägt einen
Index, um seinen eigenen zu finden:

```rust
struct State {
    permits: usize,
    records: Slab<Waiter>,      // Speicher: alle Waiter-Records, heap-basiert
    order:   VecDeque<usize>,   // Ankunftsreihenfolge: Indizes in `records`
}

pub struct Acquire<'a> {
    semaphore: &'a Semaphore,
    key: Option<usize>,         // „mein Record ist records[key]" — None bis zur Einreihung
}
```

Das ist ehrliches, vollständig sicheres Rust, und es funktioniert. Seine Kosten
sind von der leisen Sorte. Das Einreihen kann allokieren — und gelegentlich den
ganzen Backing-Speicher umziehen, genau der unvorhersehbare Ausschlag, den der Hot
Path nicht beherbergen kann. Speicher und Reihenfolge sind zwei Strukturen, die
sich nie widersprechen dürfen, also fasst jede Operation beide an. Und Teil 5s
eingestecktes Detail wird fällig: Cancellation entfernt einen Eintrag aus der
*Mitte*, was gegen `order` einen linearen Scan bedeutet:

```rust
// Drop, gecancelt beim Warten:
state.order.retain(|&k| k != my_key);   // O(n)-Scan, um mich selbst zu entfernen
state.records.remove(my_key);
```

Es gibt auch eine subtilere Schieflage, die auf die Lösung zeigt. Der natürliche
Besitzer des Records ist nicht die Semaphore — es ist der *Wartende*. Ein Record
entsteht, wenn ein Wartender sich einreiht, und stirbt, wenn er geht: identische
Lebensdauern. Ihn in der Semaphore unterzubringen macht sie zur Vermieterin von
Mietern, deren Mietverträge sie nicht versteht.

## Die seltsame Adresse: im Future selbst

Folge dem Besitz. Der Record lebt exakt so lange wie das Warten — und ein Stück
Speicher mit genau dieser Lebensdauer existiert bereits: das `Acquire`-Future. Der
Aufrufer *hält den Speicher des Wartens schon in der Hand*. Also leg den Record
ins Future, und lass die Queue nichts sein als Pointer, die sich durch die
Wartenden fädeln:

```rust
struct Waiter {
    waker: Option<Waker>,
    granted: bool,
    prev: Option<NonNull<Waiter>>,   // ← die Links der Queue leben IM Record
    next: Option<NonNull<Waiter>>,
}

pub struct Acquire<'a> {
    semaphore: &'a Semaphore,
    node: Waiter,                    // ← der Record lebt IM Future
}

struct State {
    permits: usize,
    head: Option<NonNull<Waiter>>,   // die gesamte „Queue": zwei Pointer
    tail: Option<NonNull<Waiter>>,
}
```

Das ist eine *intrusive* Liste — Links im Payload statt in Knoten, die die Liste
drumherum allokiert. Die Buchführung von Option eins verdampft: Allokation pro
Wartendem — keine, der Speicher des Futures existiert sowieso; zu
synchronisierende Strukturen — eine, die Pointerkette *ist* Speicher und
Reihenfolge zugleich; Entfernen aus der Mitte — zwei Pointer-Schreibzugriffe:

```rust
// Drop, gecancelt beim Warten — O(1), kein Scan:
unsafe {
    (*prev.as_ptr()).next = my.next;   // meine Nachbarn zeigen jetzt an mir vorbei
    (*next.as_ptr()).prev = my.prev;
}
```

(Doppelt verkettet genau deshalb: Ein sich aushängender Knoten muss beide Nachbarn
erreichen. Cancellation ist die Operation, die die Datenstruktur auswählt.)

tokios Semaphore macht exakt das. Aber gerade ist etwas Alarmierendes passiert,
und es verdient klare Worte: **Die Semaphore hält jetzt Pointer in das Innere von
Futures, die ihr nicht gehören.**

## Das Problem: Futures bewegen sich

Ein Future ist ein gewöhnlicher Rust-Wert, und gewöhnliche Werte *bewegen sich*.
All das ist sicherer Alltagscode:

```rust
let fut = sem.acquire();          // Acquire auf dem Stack, node darin
let boxed = Box::new(fut);        // MOVE: jedes Byte, node inklusive, auf den Heap kopiert
let fut2 = returns_a_future();    // MOVE: aus dem Frame des Aufgerufenen in unseren
tokio::spawn(async move { … });   // MOVE: der ganze async-Block in den Task
```

Ein Move ist ein memcpy. Die Bytes ziehen um, und niemand benachrichtigt den, der
sich die alte Adresse gemerkt hat. Der Fehler in Zeitlupe:

```
1.  fut.node ist verlinkt:     head ──► &fut.node   (Adresse 0x7ffd_1000, Stack)
2.  fut zieht in eine Box:     nodes Bytes liegen jetzt bei 0x5561_2000 (Heap)
3.  die Queue zeigt weiter auf 0x7ffd_1000 — toten Stack-Speicher
4.  nächstes Release: (*head).granted = true    ← SCHREIBZUGRIFF durch hängenden Pointer
```

Kein Leak, kein Logikfehler — Speicherkorruption, die weit entfernt von ihrer
Ursache auftaucht. Das intrusive Design ist nur unter einer Garantie tragfähig:
**Sobald der Node eines Futures in die Queue verlinkt ist, darf dieses Future sich
nie wieder bewegen.**

Rust hat einen Typ, dessen einziger Daseinszweck diese Garantie ist. Du schreibst
ihn seit Teil 2 in jede `poll`-Signatur, vermutlich ohne ihn je gebraucht zu
haben: `Pin`.

## Was Pin wirklich tut — aus der Nähe betrachtet

Beginne beim Default. Jeder Typ in Rust ist automatisch `Unpin`, was heißt: „Mich
zu pinnen ist bedeutungslos, ich darf mich frei bewegen." Für einen `Unpin`-Typ
ist `Pin` ein No-op-Wrapper — weshalb `poll`-Implementierungen ihm beiläufig
entkommen:

```rust
// Acquire, wie bisher definiert, ist Unpin (alle Felder sind es), also geht das:
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this: &mut Acquire = self.get_mut();   // ✓ kompiliert — Unpin lässt dich raus
    …
}
```

`&mut Acquire` ist exakt die Macht, den Wert zu bewegen (`mem::swap`,
`mem::replace`, schlichte Zuweisung — alle nehmen `&mut`). Solange `get_mut`
funktioniert, existiert die Garantie „bewegt sich nie wieder" nicht. Damit sie
existiert, erklären wir, dass unser Typ nicht entpinnt werden darf — ein
Markerfeld:

```rust
use core::marker::PhantomPinned;

struct Waiter {
    waker: Option<Waker>,
    granted: bool,
    prev: Option<NonNull<Waiter>>,
    next: Option<NonNull<Waiter>>,
    _pin: PhantomPinned,      // ← Waiter ist jetzt !Unpin; Acquire, das ihn enthält, auch
}
```

An den Bytes hat sich nichts geändert — `PhantomPinned` ist null Byte groß.
Geändert hat sich, was der Compiler uns zu schreiben erlaubt. Das `get_mut` von
oben scheitert jetzt:

```
error[E0277]: `PhantomPinned` cannot be unpinned
   --> src/semaphore.rs:88:31
    |
 88 |         let this = self.get_mut();
    |                         ^^^^^^^ within `Acquire<'_>`, the trait `Unpin`
    |                                 is not implemented for `PhantomPinned`
    |
    = note: consider using `Box::pin`
    = note: required because it appears within the type `Waiter`
    = note: required because it appears within the type `Acquire<'_>`
```

Das ist `Pin`s gesamter Mechanismus, sichtbar in einem Fehler: Es sperrt keinen
Speicher und installiert keinen Laufzeit-Wächter — es **enthält `&mut` vor**.
Jeder sichere Weg, einen Wert zu bewegen, braucht `&mut`; `Pin` weigert sich, für
einen `!Unpin`-Typ eines herzustellen; also kann sicherer Code ein gepinntes
`Acquire` nicht bewegen. Durchgesetzt wird die Garantie so, wie der Borrow-Checker
alles durchsetzt: Das verletzende Programm kompiliert nicht.

In `poll` müssen wir unsere Felder trotzdem anfassen, also nehmen wir die
Notluke — und akzeptieren ihre Bedingung:

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    // SAFETY: Wir nutzen `this`, um Felder zu lesen/schreiben und die Adresse
    // des Nodes zu nehmen. Wir bewegen nie etwas heraus — das ist das
    // Versprechen, um das unsafe uns bittet.
    let this: &mut Acquire = unsafe { self.get_unchecked_mut() };

    let node = NonNull::from(&mut this.node);   // jetzt SPEICHERBAR:
    …                                           // die Adresse eines gepinnten Futures ist final
}
```

## Warum das Timing aufgeht

Die Garantie hat eine Form, die man einmal von Anfang bis Ende sehen sollte, weil
jede Phase eine Aufgabe erfüllt:

```
erzeugt ──── bewegt sich frei ────► gepinnt ──── gepollt, gepollt… ────► gedroppt
             (legal: es wurde              (poll darf den Node               (Drop hängt aus,
              nie gepollt, ist              jetzt in die Queue                BEVOR der
              also nirgends                 verlinken — die                   Speicher geht —
              verlinkt; keine               Adresse ist                       Teil 5 hat das
              Pointer hinein                eingefroren)                      schon geschrieben!)
              existieren)
```

Vor seinem ersten Poll ist ein Future nie gelaufen, kann sich also nirgends
verlinkt haben — keine Pointer hinein existieren, und es zu bewegen ist harmlos.
Genau das erlaubt Rust: `Box::new(fut)`, `spawn(fut)`, alles in Ordnung. Die
Runtime pinnt dann jedes Future *vor seinem ersten Poll* — gespawnte in die
Allokation des Tasks, ge-`await`-ete im Inneren ihres Elternteils — und ab da
erhält `poll` ein `Pin<&mut Self>` als Beweis, dass die Adresse final ist. Pointer,
die nach dem Pinnen entstehen, zeigen auf Speicher, der nicht mehr weglaufen kann.

Und der Ausgang ist schon bewacht: Teil 5s `Drop` hängt den Node aus — aus
Buchführungsgründen — bevor der Speicher des Futures freigegeben wird. Pointer in
das Future entstehen erst nach dem Pinnen und sterben vor der Deallokation. Das
Fenster, in dem sie existieren, ist exakt das Fenster, in dem sie gültig sind.
Jede Klausel dieses Arguments wird von der Sprache erzwungen — außer
Aushängen-beim-Drop, das uns gehört. Und das hatten wir schon geschrieben, bevor
`Pin` je zur Sprache kam.

## Der ehrliche Tausch

Option eins zahlt zur Laufzeit: Allokationen, Scans, zwei Strukturen im Einklang.
Option zwei zahlt in Verpflichtungen: Die Link-Chirurgie ist `unsafe`, das der
Compiler nicht prüft; Records dürfen nur unter dem Lock der Semaphore berührt
werden; der Pinning-Vertrag zieht sich durch alles. Identisches Verhalten; die
zweite ist, was man ausliefert, wenn die Semaphore unter einem Channel sitzt, der
eine Million Nachrichten pro Sekunde bewegt.

Verpflichtungen, die der Compiler nicht prüft, brauchen etwas anderes, das sie
prüft. Das — und das Ganze endlich niederzuschreiben — ist Teil 7.

---

*Weiter: [Teil 7 — Aufschreiben, und ihm trauen](07_implementation.md) · [Index](00_index.md)*

*English: [`../en/06_memory_and_pin.md`](../en/06_memory_and_pin.md)*
