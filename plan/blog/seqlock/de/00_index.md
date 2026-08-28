# Ein SeqLock entwerfen

Irgendwo in einem System, das einen gemeinsamen Wert weit häufiger liest, als es
ihn schreibt — ein Blockchain-Node, der zehntausende Male pro Sekunde zwischen zwei
Blöcken fragt „was ist der Chain-Head?", eine Börse, die bei jeder einzelnen Order
den Mark-Preis liest —, steckt ein Primitive, das all diese Leser vorankommen lässt,
ohne je zu blockieren und ohne je ein Byte in den gemeinsamen Speicher zu schreiben.
Es heißt SeqLock, und wer auf Linux `clock_gettime` aufgerufen hat, ohne dass der
Aufruf den Kernel erreicht, hat eines benutzt. Diese Serie entwirft eines von Grund
auf.

Es ist eine Design-Untersuchung, kein Coding-Tutorial. Wir beginnen bei dem Problem,
für dessen Lösung ein SeqLock existiert, sehen jedem Lock, zu dem man normalerweise
greifen würde, dabei zu, wie es an genau einer der Randbedingungen scheitert, und
gehen dann die Wette ein, die das ganze Primitive definiert: Statt den Leser daran
zu hindern, einen halb geschriebenen Wert zu beobachten, lassen wir es geschehen und
bringen den Leser dazu, es zu erkennen. Jede Entscheidung danach ist erzwungen —
durch einen Use Case, oder durch das Scheitern der einfacheren Alternative, oder, in
einem denkwürdigen Fall, durch einen ARM-Prozessor, der deine Instruktionen umordnet
und einen Wert korrumpiert, von dem deine Testsuite steif und fest behauptet, er sei
in Ordnung.

Kein Lock-free-Hintergrund wird vorausgesetzt. Die Maschinerie — `Relaxed`/`Acquire`/`Release`,
fences, `Pod`, Miri, loom — wird in dem Moment eingeführt, in dem das Design mit ihr
kollidiert.

## Die Teile

**[Teil 1 — Das Problem, und warum die naheliegenden Locks nicht passen.](01_the_problem.md)**
Ein Schreiber, viele Leser und ein Wert aus mehreren Feldern, der nicht in ein
einzelnes Maschinenwort passt — es gibt also immer einen Augenblick, in dem der
Speicher halb alt, halb neu ist, und ein Leser, der genau dort landet, bekommt einen
Wert, den es nie gab. Die drei Randbedingungen, die das schwer machen, und dann die
Tour der Fehlschläge: Ein `RwLock` lässt Leser gemeinsam hinein, zwingt aber jeden von
ihnen, einen gemeinsamen Zähler zu *schreiben*, und serialisiert damit Cores, die gar
nicht in Konflikt stehen; `ArcSwap` und RCU umgehen das Zerreißen, ziehen den Leser
aber zurück hinein, sich selbst zur Reclamation anzumelden. Jede korrekte Option bricht
dieselbe Regel, und sie weist auf den einzigen Ausweg — der Leser muss unsichtbar sein.

**[Teil 2 — Die Wette: Lass es zerreißen und fang es ab.](02_the_bet.md)**
Wenn der Schreiber nicht aufgehalten werden kann und der Leser sich nicht anmelden
kann, bleibt ein einziger Zug: den Read zerreißen lassen und dem Leser einen Weg geben,
es hinterher zu bemerken und erneut zu versuchen. Das reduziert alles auf eine einzige
Frage — woher weiß ein Leser, dass er während eines Schreibvorgangs gelesen hat? — und
wir leiten die Antwort auf die harte Tour her, indem wir einem booleschen Flag beim
Scheitern zusehen, weil es keine Geschichte trägt, bis das Einzige übrig bleibt, was
funktioniert: ein Zähler, der gerade ist, wenn der Wert stabil ist, und ungerade,
während er geschrieben wird, vor und nach dem Lesen abgetastet.

**[Teil 3 — Das Memory-Ordering richtig hinbekommen.](03_memory_ordering.md)**
Das Protokoll ist auf dem Papier korrekt und zerreißt trotzdem auf einem echten Apple
M2, vier von fünf Läufen grün — der Fingerabdruck eines Memory-Ordering-Bugs. Der
Zähler baut ein Fenster; noch zwingt nichts die Payload, darin zu bleiben. Wir
reparieren es mit fences, und um sie zu platzieren, brauchen wir die Idee, die alle
immer verkehrt herum verstehen: `Release` und `Acquire` sind Einweg-Gates, von denen
jedes nur eine Seite der Operation bewacht, an die es geheftet ist. Zwei der vier
Fensterkanten kommen mit einem Ordering auf dem Atomic selbst aus; die anderen beiden
brauchen einen eigenständigen fence — und die fences sind, wie sich herausstellt, das,
was zwei Threads sich zu einer happens-before-Beziehung die Hand reichen lässt. Das ist
das Herz der Serie.

**[Teil 4 — Lesen ohne UB, und ihm vertrauen.](04_trusting_it.md)**
Wir lassen den Leser bewusst Bytes lesen, die der Schreiber gerade ändert. In C ist das
eine Volkstradition mit `volatile`; in Rusts Memory-Modell ist es ein Data Race —
undefiniertes Verhalten — und Miri sagt es laut und deutlich. Die Korrektur macht jeden
Zugriff auf die Payload atomar, Wort für Wort, und verwandelt „Müll lesen" von UB in
einen legalen Read, den der Zähler wegwirft — was die Payload zwingt, `Pod` zu sein,
eine Schranke, die sich als Lizenz erweist, die der Implementierer *unterschreibt*,
statt einer, die der Compiler prüft, und die nicht einmal Alignment abdeckt. Dann
bekommt der Sequenzzähler einen zweiten Job als Lock der Schreiber, und dazu die
Vertrauensfrage: der Test, der *scheitern* muss, Miri für das Race, loom für die
Verschränkungen, und ein Benchmark, der — in Nanosekunden — zeigt, wie ein Lesepfad
flach bleibt, während ein `RwLock` 450× langsamer wird.

## Wie man sie liest

Der Reihe nach — jeder Teil beginnt dort, wo der vorige aufgehört hat, und schließt mit
der Frage, die der nächste beantwortet. Zehn bis fünfzehn Minuten pro Teil. Teil 1
steckt das Problem ab und schließt die Alternativen aus; Teil 2 geht die Kernwette ein;
Teil 3 ist das Memory-Ordering-Herz; Teil 4 ist die Sprache, der Fall mehrerer
Schreiber und der Beweis. Nach Teil 2 aufzuhören gibt dir bereits die ganze Idee; die
Teile 3–4 sind der Ort, an dem sie auf die Hardware und die Sprache trifft und an dem
die meisten echten SeqLock-Bugs wohnen.

## Umfang

Diese Serie entwirft ein generisches, wiederverwendbares `SeqLock<T>` — die Sorte, die
man in eine Concurrency-Crate legt, nicht eine Einmalvariante, fest verdrahtet auf ein
einzelnes Struct. Alles wird auf `aarch64` (Apple M2) gebaut und gemessen, denn das
schwache Memory-Modell ist der Ort, an dem sich die interessanten Fehler zeigen; ein
x86-Lauf würde die Hälfte von Teil 3 verbergen. Die Zahlen in Teil 4 sind echt, aus den
`criterion`-Benches der Crate.

## Glossar

- **SeqLock** — ein Lock, bei dem Leser nie blockieren und nie in den gemeinsamen
  Speicher schreiben; sie lesen optimistisch und versuchen es erneut, falls ein
  Schreibvorgang überlappt hat. Ein einzelner Wert, viele Leser, ein seltener Schreiber.
- **torn read** („zerrissener Read") — einen Wert beobachten, der teils alt, teils neu
  ist, weil ein Schreibvorgang lief; ein Wert, den es als Ganzes nie wirklich gab.
- **payload** — der geschützte Wert selbst (im Gegensatz zum Sequenzzähler, der ihn
  bewacht).
- **Sequenzzähler / seq** — die Ganzzahl, die der Schreiber um jeden Schreibvorgang
  herum hochzählt; gerade = stabil, ungerade = ein Schreibvorgang läuft. Der Leser liest
  sie davor und danach.
- **`Relaxed` / `Acquire` / `Release`** — Memory-Orderings auf einer atomaren Operation.
  `Relaxed` = atomar, aber keine Ordering-Garantien; `Acquire`/`Release` fügen eine
  einseitige Ordnung hinzu und paaren sich über Threads hinweg, um happens-before
  herzustellen.
- **fence** — eine eigenständige Ordering-Barriere (`atomic::fence`), an kein einzelnes
  Atomic geheftet; zweiseitig für die Operationen, die sie regelt, während ein
  Ordering-auf-einer-Operation einseitig ist.
- **`Pod`** („plain old data") — ein Marker-Trait, der verspricht, dass ein Typ nur
  Bytes ist: kein Padding, jedes Bitmuster gültig, definiertes Layout. Erlaubt es, ihn
  gefahrlos als rohe Wörter umzudeuten.
- **MESI / Cache-Kohärenz** — das Protokoll, das die Caches der einzelnen Cores
  konsistent hält; eine cache line, die ein Core schreibt, muss in den anderen
  invalidiert werden — deshalb serialisiert ein gemeinsam geschriebener Zähler Cores,
  die logisch gar nicht in Konflikt stehen.
- **Miri** — ein Interpreter, der Rust gegen das Memory-Modell ausführt und
  undefiniertes Verhalten (Data Races, ungültige Pointer) fängt, das ein normaler Test
  zwar ausführt, aber nicht erkennen kann.
- **loom** — ein Model Checker, der einen kleinen nebenläufigen Test unter jeder
  möglichen Thread-Verschränkung erneut ausführt; der Verifizierer für Lock-free-Code.

*English: [`../en/00_index.md`](../en/00_index.md)*
