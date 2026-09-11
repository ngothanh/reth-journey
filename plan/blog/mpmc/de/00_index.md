# Einen lock-freien MPMC-Ring entwerfen

Am schreibenden Ende von allem, was schnell ist — eine Matching Engine, die Orders von
einem Dutzend Feed-Threads entgegennimmt, ein Write-Ahead-Log, das viele Writer in
einen einzigen Commit bündelt —, sitzt eine Queue, in die viele Threads pushen und aus
der viele Threads poppen, millionenfach pro Sekunde. Greif zu einem `Mutex<VecDeque>`,
und jeder dieser Threads serialisiert sich auf einem einzigen Lock-Wort, das zwischen
den Cores hin- und herspringt. Diese Serie entwirft die Queue, die das nicht
tut: einen Ring — beschränkt, allokationsfrei, lock-free —, den sich N Producer und M
Consumer teilen und bei dem nur noch die Übergabe selbst etwas kostet. Vier Teile,
zehn bis fünfzehn Minuten pro Teil.

Es ist eine Design-Untersuchung, kein Tutorial. Jede Entscheidung ist erzwungen — durch
einen Use Case, durch das Scheitern der einfacheren Variante oder durch ein
Verifikationswerkzeug. Kein Lock-free-Hintergrund wird vorausgesetzt; `compare_exchange`,
`Acquire`/`Release`, `UnsafeCell`, `MaybeUninit`, loom, Miri und `CachePadded` werden
dort eingeführt, wo das Design mit ihnen kollidiert.

## Die Teile

**[Teil 1 — Das Problem, und warum die naheliegenden Queues nicht passen.](01_the_problem.md)**
Drei Randbedingungen — beschränkt und allokationsfrei, lock-free, Viele-zu-viele und
nicht-blockierend — und die Tour durch die Queues, von denen jede eine davon verletzt.
Das Lock erledigt, wie sich herausstellt, zwei Jobs — Reihen vergeben und Daten
übergeben —, und die beiden zu trennen ist das ganze Design: ein Array, zwei
Ticket-Zähler und eine Maske.

**[Teil 2 — Ein Zähler kann nicht „geschrieben" sagen.](02_the_sequence.md)** Der
naheliegende Ring zerreißt in dem Moment, in dem ein Consumer neben einem Producer
läuft, weil `tail` beim *Claim* weiterrückt, vor dem Write. Ein Boolean pro Slot kann
das nicht reparieren — ein Boolean hat kein Gedächtnis, und Slots werden über Runden
hinweg wiederverwendet. Vyukovs Sequenz pro Slot kann es: eine Zahl, die sagt, für
welches Ticket der Slot bereit ist. Ein Drei-Wege-Gate, die `publish`- und
`vacate`-Regeln und der Vertrag darüber, wann `try_push` erneut versucht und wann es
zurückkehrt.

**[Teil 3 — Das Memory Ordering richtig hinbekommen.](03_memory_ordering.md)** Alles
`Relaxed`, und der Ring ist ein Data Race. Die Orderings werden hergeleitet, nicht
geraten: die zwei Stellen finden, an denen Daten den Thread wechseln, und dann bemerken,
dass der payload nur über `seq` veröffentlicht werden kann, nie über den Zähler — ein
`Release` legt einen Boden unter alles, was *davor* kommt, und der Claim kommt vor dem
Write. Boden und Dach, ein Gate nach dem anderen, warum der CAS `Relaxed` bleibt, und
warum dieser Ring — anders als ein SeqLock — keinen Fence braucht. Das Herz der Serie.

**[Teil 4 — Der Beweis: zwei Werkzeuge und ein lügendes Grün.](04_proving_it.md)** Miri
nennt den Race beim Namen; loom, mit dem `Relaxed`-Ring gefüttert, explodiert, statt zu
antworten. Mit den Orderings drin werden beide grün — und das Grün von loom ist weniger
wert, als es aussieht: Es modelliert einen Compare-and-Swap stärker als die Hardware und
winkt eine kaputte Übergabe durch, die über einen läuft — gezeigt, indem man eine
einzige Zeile ändert. Die Arbeitsteilung zwischen den drei Instrumenten, ein `Drop`, das
Miris Leak-Checker absegnet, und der Benchmark: `head` und `tail` mit Padding auf
getrennte cache lines zu legen, bringt die SPSC-Übergabe von ~51 ns auf ~9 ns.

## Wie man sie liest

Der Reihe nach. Jeder Teil beginnt dort, wo der vorige aufgehört hat, und schließt mit
der Frage, die der nächste beantwortet. Nach Teil 2 aufzuhören lässt dich mit einem Ring
zurück, der logisch korrekt ist; die Teile 3 und 4 sind der Ort, an dem er auf die
Hardware und die Verifizierer trifft und an dem die Fehler wohnen, die Tests bestehen.

## Umfang

Ein generischer, beschränkter `MpmcRing<T>` — der Vyukov-Algorithmus, so wie man ihn in
eine Concurrency-Crate legen würde. Gebaut und gemessen auf `aarch64` (Apple M2), denn
ein schwaches Memory-Modell ist der Ort, an dem sich die Fehler aus Teil 3 zeigen; auf
x86 würde sich der Data Race hinter einer stärkeren Hardware-Garantie verstecken. Die
Zahlen in Teil 4 stammen aus den `criterion`-Benches der Crate, und jede Aussage über
Korrektheit ist eine, die loom oder Miri tatsächlich geprüft hat.

## Glossar

- **MPMC-Ring** — ein beschränkter Ringpuffer, in den mehrere Producer pushen und aus
  dem mehrere Consumer poppen, lock-free und allokationsfrei.
- **lock-free** — irgendein Thread kommt immer voran; kein Thread hält ein Lock, auf das
  andere warten. Nicht wait-free: Ein einzelner Thread kann es erneut versuchen müssen.
- **Ticket** — der Wert, den ein Thread von `tail` (Producer) oder `head` (Consumer)
  zieht; `ticket & (capacity − 1)` ist sein Slot.
- **Slot / Zelle** — ein Element des Puffers: ein payload und seine Sequenznummer.
- **`seq`** — der Zähler pro Slot, der sagt, für welches Ticket der Slot bereit ist. Er
  zählt nur hoch: `pos` (Producer darf schreiben), `pos + 1` (Consumer darf lesen),
  `pos + capacity` (der Producer der nächsten Runde darf schreiben).
- **claim vs. publish** — ein Ticket auf dem Zähler ziehen vs. den payload über `seq`
  sichtbar machen. Verschiedene Operationen, verschiedene Atomics.
- **Übergabe** (hand-off) — ein Thread ist mit ein paar Bytes fertig und signalisiert
  es, ein anderer holt sie ab: eine Sendeseite und eine Empfangsseite. Zwei pro Slot:
  A (`publish`, Producer → Consumer) und B (`vacate`, Consumer → Producer der nächsten
  Runde).
- **`compare_exchange` (CAS)** — atomares Read-Modify-Write, das nur gelingt, wenn der
  Wert dem entspricht, was du erwartet hast; so beansprucht genau ein Thread ein Ticket.
- **`Relaxed` / `Acquire` / `Release`** — `Relaxed` ist atomar ohne Ordering; `Release`
  auf einem Store ist ein Boden unter allem, was davor kommt; `Acquire` auf einem Load
  ist ein Dach über allem, was danach kommt; ein `Acquire`-Load, der einen
  `Release`-Store beobachtet, verbindet die beiden.
- **happens-before** — die threadübergreifende Garantie, die ein `Release`/`Acquire`-Paar
  aufbaut; ohne sie sind ein einfacher Write auf einem Thread und ein einfacher Read auf
  einem anderen ein Data Race.
- **False Sharing** — zwei unabhängige Werte auf einer cache line, wobei jeder Write die
  Kopie des anderen Cores invalidiert; behoben durch `CachePadded`.
- **loom** — ein Model Checker, der einen kleinen Test unter jeder Verschränkung
  ausführt; blind für Übergaben, die über einen CAS laufen.
- **Miri** — ein Interpreter, der Rust gegen das Memory-Modell ausführt und UB meldet —
  Data Races, uninitialisierte Reads, Leaks.

*English: [`../en/00_index.md`](../en/00_index.md)*
