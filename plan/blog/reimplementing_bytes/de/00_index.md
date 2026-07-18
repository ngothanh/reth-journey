# `Bytes` neu implementieren: ein Typ, drei Arten, Speicher zu besitzen

Dies ist eine Serie über ein kleines, aber berüchtigt kniffliges Stück Code: einen
*Zero-Copy-Byte-Handle*. Wenn du schon einmal Rusts `bytes`-Crate, Facebooks `IOBuf`
oder Nettys `ByteBuf` benutzt hast, dann steckt genau dieses Ding darin — nur bauen
wir es hier von Grund auf nach, um zu verstehen, warum es so entworfen ist.

Die Serie ist keine Schritt-für-Schritt-Coding-Anleitung. Sie ist eine
*Untersuchung*: Wir beginnen mit einem alltäglichen Bedürfnis (ein Netzwerkprogramm
liest Daten ein und reicht sie weiter), stoßen auf ein Performance-Problem,
probieren die naheliegenden Lösungen aus, sehen sie scheitern, und jedes Mal, wenn
wir gegen eine Wand laufen, zeigt sich ein Stück des echten Entwurfs. Kein Stück
fällt vom Himmel; jede Entscheidung wird von der vorherigen *erzwungen*.

Du musst `Bytes`, `BytesMut` oder `freeze` vorher nicht kennen — Teil 1 baut alles
von null auf.

Die Serie teilt sich in zwei Abschnitte: **Teil 1–5 sind *Entwurf*** (warum `Bytes`
diese Form hat), **Teil 6–8 sind *Implementierung*** (sich hinsetzen und jede
vtable-Funktion, `from_vec` und `slice` korrekt schreiben, samt den Code-Details, die
der Entwurfsteil absichtlich aufschob: die Memory-Ordering-Disziplin des refcounts, den
Bit-Packing-Trick und den Promotion-Wettlauf).

## Der Entwurf — fünf Teile

**[Teil 1 — Die Reise eines Bytes vom Draht ins Programm.](01_the_problem.md)**
Wir setzen die Bühne: Ein Netzwerkprogramm nimmt Daten auf, braucht einen
schreibbaren Puffer (`BytesMut`) und muss ihn dann über eine Operation namens
`freeze` in einen teilbaren, nur-lesbaren Handle (`Bytes`) verwandeln. Wir
entdecken, dass `freeze` sehr langsam sein kann, wenn es kopiert, und stellen eine
Anforderung: `freeze` darf nicht kopieren. Dann probieren wir zwei naheliegende
Entwürfe (`Vec<u8>` und `Arc<[u8]>`) und sehen, wo sie brechen — das legt den
zentralen Widerspruch offen: *ein Typ, drei Arten der Aufräumung.*

**[Teil 2 — Ein Typ, viele Verhalten.](02_vtable.md)**
In Rust lebt „wie Speicher aufgeräumt wird" normalerweise im *Typ*, und der Compiler
erledigt alles. Aber wir haben nur einen Typ und brauchen drei Verhalten. Dieser
Teil zeigt, wie man die Aufräum-Entscheidung vom Compiler herab in *Daten innerhalb
der Struktur* verlegt — eine handgeschriebene Dispatch-Tabelle (eine vtable). Dazu
eine Frage, die jeder, der so einen Typ entwirft, beantworten können muss: warum
diese Tabelle genau *zwei* Slots hat.

**[Teil 3 — „Welche Bytes" von „wer besitzt sie" trennen.](03_split_and_counting.md)**
Der Trick, der diesen Entwurf zugleich flexibel und *schnell* macht: die Felder der
Struktur so anzuordnen, dass das Lesen der Bytes nie auf die Besitz-Information
schauen muss. Dieser Teil führt auch eine einfache Denkweise ein, die das Rückgrat
der schweren Teile bildet — jede Art, Speicher zu besitzen, reduziert sich auf eine
Frage: *wie oft wird dieser Block freigegeben?*

**[Teil 4 — Die Wand: wenn clone alles kaputt macht.](04_promotion.md)**
Das ist der schwerste Teil. Drei der vier Verhalten sind leicht, aber das Klonen
eines allein-besitzenden Handles verursacht ein Double-Free. Der einzige Ausweg —
genannt *Promotion* — erzwingt etwas sehr Ungewöhnliches in Rust: ein Wert muss
*zurück* in einen anderen, bereits existierenden Wert schreiben, mitten in dessen
Lebenszeit.

**[Teil 5 — `AtomicPtr`: sicher zurückschreiben.](05_atomics.md)**
Das Zurückschreiben aus Teil 4 stellt drei unabhängige Anforderungen, und alle drei
werden zufällig durch eine einzige Feldtyp-Wahl gelöst. Dieser Teil geht durch die
drei Concurrency-Konzepte, die die meisten am abstraktesten finden — interior
mutability, CAS und Memory Ordering — aber diesmal hängt jedes an einem konkreten
Problem, das wir wirklich lösen müssen, nicht an bloßer Theorie. Er schließt mit fünf
Fragen, die man in jedes spätere Systems-Problem mitnehmen kann.

## Die Implementierung — drei Teile

**[Teil 6 — Vom Modell zum Code: `static` und `shared`.](06_static_and_shared.md)**
Wir schreiben die ersten vier vtable-Funktionen. `static` ist die Aufwärmübung (eine
leere `drop`-Funktion ist genau „0-mal free"). `shared` ist nur an einer einzigen
Stelle schwer, aber diese Stelle ist die wichtige ordering-Lektion, die der Entwurfsteil
noch nicht berührte: `share_drop` muss *free-while-read* abwehren — den buffer
freigeben, während ein anderer Thread noch liest — mit `Release` beim Verringern des
counters und einem `fence(Acquire)` vor der Freigabe. Wir stellen es dem
*publish*-ordering aus Teil 5 gegenüber, um die zwei verschiedenen Gefahren zu sehen.

**[Teil 7 — `from_vec` und der Bit-Packing-Trick.](07_from_vec_and_bit_tagging.md)**
Wie kann eine 8-Byte-Zelle (`data`) zugleich einen buffer-Pointer und einen
counter-Pointer halten und die zwei Arten unterscheiden? Der Trick borgt das *niedrigste
Bit*, zusammengefasst in einem Satz: **VEC ungerade, ARC gerade**. Weil ein `u8`-buffer
gerade oder ungerade sein kann, brauchen wir *zwei* vtables (`EVEN`/`ODD`) — und dieser
Teil zeigt, warum eine vtable *nicht genug Information* ist, nicht etwa Faulheit.

**[Teil 8 — promotable vollständig, und `slice` O(1).](08_promotable_and_slice.md)**
Alles zusammenfügen: die vier dispatch-Funktionen, die Funktion `promote_vec` mit dem
CAS und dem *Verlierer*-Zweig (die Stelle, wo `actual` sich von `shared` unterscheidet —
ein klassischer Bug), und `slice` als clone-dann-verengen. Der geschlossene Kreis:
`slice` *befolgt* das invariant „VEC wird nie geschnitten" nicht nur — es *erzwingt*
dieses invariant durch Struktur, und genau dieses invariant macht das Wiederherstellen
von `cap` per Arithmetik sicher.

## Wie man liest

Der Reihe nach — jeder Teil baut direkt auf dem auf, was der vorherige gerade
aufgestellt hat. Jeder Teil dauert etwa 15 Minuten, ist in sich abgeschlossen,
beginnt dort, wo der letzte aufgehört hat, und schließt mit der Frage, die der
nächste aufgreift.

## Umfang

Der Entwurfsteil (1–5) handelt vom *warum* und lässt Code-Details absichtlich weg,
damit das Modell klar hervortritt. Der Implementierungsteil (6–8) greift genau diese
Details wieder auf — Funktionssignaturen, die ordering-Disziplin des refcounts, den
Bit-Packing-Trick, den CAS-Wettlauf — und schreibt sie so weit aus, dass du sie
mittippen kannst. Willst du nur den Entwurf *verstehen*, ist Teil 5 ein vollständiger
Abschluss; willst du `Bytes` *neu schreiben*, geh die letzten drei Teile weiter.

## Glossar (schnelles Nachschlagen)

Wir behalten die englischen Terms bei; hier eine einzeilige Bedeutungsübersetzung,
damit du den Artikel nicht zum Nachschlagen verlassen musst:

- **`deref`** — die slice `&[u8]` aus einer `Bytes` holen (über das `Deref`-Trait). Der
  *Lese*-Pfad, billig und ohne den Besitz-Teil anzufassen.
- **refcount** — der Zähler, wie viele handles sich einen buffer teilen; bei 0 wird
  freigegeben.
- **CAS** (*compare-and-swap*) — die atomare Operation „wenn du noch X bist, mach Y
  daraus", ohne dass ein Thread dazwischenrutscht. Das Fundament lock-freier
  Aktualisierungen.
- **`Release` / `Acquire`** — ein Paar von *Memory-Ordering*-Labeln: die eine Seite
  *veröffentlicht*, die andere *abonniert*; sie wirken nur als Paar auf derselben
  Variable.
- **UB** (*undefined behavior*) — undefiniertes Verhalten; ist es einmal eingetreten,
  darf der Compiler *alles* tun, und der Fehler bleibt meist still.
- **`Miri`** — ein Interpreter, der Rust-Code unter einem schwachen Speichermodell
  ausführt, um UB in `unsafe`-Code (use-after-free, double-free, data race) zu *fangen*,
  das `cargo test` übersieht.

*English: [`../en/00_index.md`](../en/00_index.md) · Tiếng Việt:
[`../vi/00_index.md`](../vi/00_index.md) · 简体中文: [`../zh/00_index.md`](../zh/00_index.md)*
