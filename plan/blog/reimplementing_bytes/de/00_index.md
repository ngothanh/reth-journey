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

## Die fünf Teile

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

## Wie man liest

Der Reihe nach — jeder Teil baut direkt auf dem auf, was der vorherige gerade
aufgestellt hat. Jeder Teil dauert etwa 15 Minuten, ist in sich abgeschlossen,
beginnt dort, wo der letzte aufgehört hat, und schließt mit der Frage, die der
nächste aufgreift.

## Umfang

Die Serie handelt vom *Entwurf*, nicht von Implementierungsdetails. Wir lassen
absichtlich ein paar reine Code-Dinge weg (genaue Funktionssignaturen, ein paar
`Layout`-Feinheiten, einen Bit-Packing-Trick zum Speichersparen). Die ergeben sich
von selbst, sobald man sich hinsetzt und schreibt — wenn man die fünf Teile hat.

*English: [`../en/00_index.md`](../en/00_index.md) · Tiếng Việt:
[`../vi/00_index.md`](../vi/00_index.md) · 简体中文: [`../zh/00_index.md`](../zh/00_index.md)*
