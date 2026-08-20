# Eine Async-Semaphore entwerfen

Irgendwo in jedem ausgelasteten Async-Programm steckt eine Obergrenze — höchstens N
Datenbankverbindungen, höchstens N schwere Requests, höchstens N Nachrichten
unterwegs — und das Ding, das diese Grenze durchsetzt, ist eine Semaphore. Wer
`tokio::sync::Semaphore` benutzt hat, hatte eine in der Hand. Diese Serie entwirft
eine von Grund auf.

Es ist eine Design-Untersuchung, kein Coding-Tutorial. Wir beginnen bei den
Problemen, für die eine Semaphore existiert, lesen das Interface von den Use Cases
ab und arbeiten uns dann durch die Fragen, die jeder beantworten muss, der eine
baut: Wo lebt das Warten physisch? Wer bekommt ein freigewordenes Permit? Was
passiert, wenn ein Wartender mitten im Warten gecancelt wird? Wo liegen die Records
der Wartenden im Speicher? Jeder Teil endet auf der Frage, die der nächste
beantwortet, und keine Entscheidung fällt vom Himmel — jede wird von einem Use Case
erzwungen oder vom Scheitern der einfacheren Alternative. Die Implementierung ist
absichtlich der letzte Teil: Steht das Design, folgt der Code daraus.

Kein Vorwissen über Async-Interna nötig. Die Maschinerie — `futex`, `Waker`,
`Pin` — wird in dem Moment eingeführt, in dem das Design mit ihr kollidiert.

## Die Teile

**[Teil 1 — Was eine Semaphore ist, und wann man zu ihr greift.](01_what_is_a_semaphore.md)**
Jeder Server hat eine Decke, und der Ärger beginnt, wenn nichts im Code von ihr
weiß. Ein Türsteher mit N Armbändern; warum eine Semaphore keine Mutex ist; vier
Probleme, die sich als dasselbe Problem entpuppen — und die zwei Anforderungen, die
sie nur flüstern: Fairness und Cancellation.

**[Teil 2 — Das Interface, von den Use Cases abgelesen.](02_the_interface.md)**
Die Zwei-Methoden-Skizze, die jeder hinschreiben würde, und wie die Use Cases sie
zerlegen: das Permit, das sich selbst zurückgibt; der Unterschied zwischen
Zurückgeben und Prägen; zwei Fehlertypen, die sich weigern zu lügen; Shutdown als
Teil des Interface — und die leise Entscheidung (ein *benanntes* Future), die sich
erst in Teil 5 auszahlt.

**[Teil 3 — Wo lebt das Warten?](03_where_waiting_lives.md)**
„Warten" ist eine physische Angelegenheit, und der Kernel — der Threads wunderbar
parken kann — hat von deinen Tasks noch nie gehört. Der Vertrag, der den Futex
ersetzt: einmal gepollt, dann Stille, bis der Waker feuert. Was das erzwingt: die
Semaphore erinnert sich selbst an ihre Wartenden, Release muss wecken, und drei
Regeln, die jeder irgendwann auf die harte Tour lernt. Die Überraschung am Ende:
die Userspace-Variante ist *billiger*.

**[Teil 4 — Fairness: Wer bekommt das freie Permit?](04_fairness.md)**
Ein Permit kommt zurück, drei Tasks wollen es. Lässt man sie rennen, kann unter
Last ein einzelner Pechvogel jedes Rennen verlieren — unsichtbar für Tests,
unübersehbar im p99.9. Übergibt man das Permit stattdessen an den vordersten der
Schlange, trägt eine einzige Invariante das ganze Design: *Wartet irgendwer, steht
der Zähler auf null.* Beide Antworten sind korrekt; die Weggabelung ist echt.

**[Teil 5 — Cancellation: Wenn ein Wartender verschwindet.](05_cancellation.md)**
Ein Thread, der zu warten beginnt, hört auch wieder auf zu warten; ein Future kann
einfach aufhören zu existieren. Beim Warten gedroppt, muss es seinen Eintrag
mitnehmen. In der Lücke zwischen *geweckt* und *gelaufen* gedroppt — mit einem
Permit, das es nie konsumieren wird — muss es das Permit zurückgeben, sonst blutet
die Kapazität lautlos aus. Ein Satz deckt jeden Fall ab: Beim Drop gibt ein
Wartender zurück, was er in diesem Moment besitzt.

**[Teil 6 — Wo die Wartenden wohnen, und wofür Pin da ist.](06_memory_and_pin.md)**
Der Record eines Wartenden muss an irgendeiner Adresse liegen, und die Wahl
entscheidet, ob `acquire` allokiert. Die schnelle Antwort klingt illegal: im Future
selbst, mit einer Queue, die Pointer durch fremden Speicher fädelt. Tragfähig ist
das unter genau einer Garantie — ein verlinktes Future bewegt sich nie wieder — und
genau diese Garantie ist es, die `Pin` erzwingt: nicht mit einem Laufzeit-Wächter,
sondern indem es `&mut` vorenthält.

**[Teil 7 — Aufschreiben, und ihm trauen.](07_implementation.md)**
Sechs Teile Regeln kollabieren zu einem Vier-Zustands-Lebenszyklus im Record jedes
Wartenden; jede Funktion wird ein `match`, und die zwei schlimmsten Bugs werden zu
einem Pfeil, den es gibt, und einem, den es nicht gibt. Dann die Vertrauensfrage:
handgetriebene Tests für jede Transition, Miri für die Pointer-Versprechen, die der
Compiler auf Treu und Glauben genommen hat — und warum loom hier *noch* nichts zu
finden hat.

## Wie man sie liest

Der Reihe nach — jeder Teil beginnt, wo der vorige aufgehört hat, und endet auf der
Frage, die der nächste beantwortet. Zehn bis fünfzehn Minuten pro Teil. Teil 1–2
sind das Außen; Teil 3–6 die vier Designfragen; Teil 7 die Niederschrift. Schon
nach Teil 2 liest man die Doku von `tokio::sync::Semaphore` anders; das
Design-Fleisch beginnt bei Teil 3.

## Scope

Diese Serie entwirft die Ideen in `tokio::sync::Semaphore` — keinen
Drop-in-Ersatz. Alles bleibt durchgehend hinter einer `Mutex`: der lock-freie
Fast-Path, den Produktionsimplementierungen ergänzen, wird dort markiert, wo er
hingehörte, und bewusst nicht gebaut — er ändert keine der Designfragen und
verdunkelt mehrere. Wo tokios `batch_semaphore` abweicht (gebatchte Wakeups, der
atomare Fast-Path, `acquire_many`), zeigt Teil 7 hin.

## Glossar

- **Permit** — die Einheit, die eine Semaphore vergibt; N Permits heißt N
  gleichzeitige Inhaber.
- **Futex** — der Linux-Syscall, mit dem ein *Thread* schläft, bis ein anderer
  Thread eine Adresse signalisiert; das Park-Primitiv des Kernels.
- **Task / Future** — eine Userspace-Einheit asynchroner Arbeit; die Runtime pollt
  sie, der Kernel weiß nichts von ihr.
- **`poll` / `Pending`** — die Runtime fragt „fertig?"; `Pending` heißt „noch
  nicht — und frag nicht wieder, bis mein Waker feuert."
- **`Waker`** — das Handle, das ein Future hinterlegt, damit später jemand „poll
  mich nochmal" sagen kann; der Userspace-Ersatz für den Futex-Wake.
- **Barging** — freigewordene Permits landen im gemeinsamen Zähler und jeder darf
  zugreifen; geweckte Wartende rennen erneut gegen Neuankömmlinge.
- **Hand-off** — freigewordene Permits werden einem bestimmten Wartenden
  zugewiesen; es gibt nichts, worum man rennen könnte.
- **Cancellation** — ein Future wird vor seiner Vollendung gedroppt; in Async
  Routine (`timeout`, `select!`), kein Fehlerpfad.
- **Intrusive Liste** — eine verkettete Liste, deren Links *in* den Elementen
  leben; keine Allokation pro Knoten.
- **`Pin`** — ein Referenztyp mit dem Versprechen, dass sein Ziel sich nie wieder
  bewegt; erzwungen durch Vorenthalten von `&mut`; die Voraussetzung dafür, in ein
  Future hineinzuzeigen.
- **Miri** — ein Interpreter, der ungültige Pointer-Zugriffe (UB) fängt, die Tests
  ausführen, aber nicht erkennen können.
- **loom** — ein Werkzeug, das einen nebenläufigen Test unter jeder möglichen
  Thread-Verzahnung wiederholt; der Prüfer für lock-freien Code.

*English: [`../en/00_index.md`](../en/00_index.md)*
