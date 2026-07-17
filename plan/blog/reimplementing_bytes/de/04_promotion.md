# Teil 4 — Die Wand: wenn clone alles kaputt macht

Drei der vier Verhalten sind fertig, und sie waren überraschend leicht. Eines bleibt:
das Klonen eines `Bytes`, das eine allein-besitzende Speicherregion besitzt. Klingt
nach nichts Besonderem. Aber hier läuft der ganze Entwurf gegen eine Wand, und diese
Wand ist schwer nicht wegen der Syntax — sie ist schwer, weil sie einer Annahme
widerspricht, der wir beim Rust-Schreiben stillschweigend die ganze Zeit vertrauen.

Wir gehen hier sehr langsam vor, denn wenn du das verstehst, wird alles in Teil 5
(Atomics, CAS, Memory Ordering) zur natürlichen Folge statt zu einem Haufen loser
Konzepte.

## „Allein-besitzend" ist ein Versprechen, keine Beschreibung

Nimm die Freigaben-zählen-Denkweise aus Teil 3 und probier den naivsten clone: Das
Klonen einer allein-besitzenden Region gibt eine weitere allein-besitzende zurück,
genau wie wir eine Konstante kopiert haben.

Sag, `b1` ist eine allein-besitzende `Bytes`, die auf Speicher an Adresse `0xAAAA`
zeigt. Wir klonen sie zu `b2`. Der naive Weg macht `b2` ebenfalls allein-besitzend,
ebenfalls auf `0xAAAA` zeigend — *dieselbe Region*. Jetzt zähl:

```
b2 wird losgelassen → allein-besitzend → gib 0xAAAA frei   ← Mal 1
b1 wird losgelassen → allein-besitzend → gib 0xAAAA frei   ← Mal 2 💥
```

Zahl ist 2. Double-Free. Warum lässt es sich nicht „cleverer" vermeiden?

Weil „allein-besitzend" bedeutet „ich bin der *einzige* Besitzer". Das ist kein
passives Etikett, das einen Zustand beschreibt — es ist ein *Versprechen*, dem die
Freigabe-Funktion *vertraut*, um überhaupt freizugeben. Sie prüft nicht „hält jemand
sonst noch"; sie *nimmt an*, sie sei die Einzige, denn das ist die Klausel des
Allein-besitzend-Etiketts.

Vergleiche mit einer Konstante: Wir kopieren eine Konstante frei, weil „niemand besitzt"
verdoppelt immer noch „niemand besitzt" ist. Aber einen *Einzelbesitzer* zu kopieren,
gibt *zwei Besitzer*, und beide tragen weiter das „einzig"-Etikett, und beide werden
freigeben.

Hier der Kern, und wo viele straucheln: Der Akt des Klonens *macht `b1`s Versprechen
„ich bin der Einzige" falsch* — obwohl wir `b1` nie berührt haben. Dass `b2` bloß ins
Dasein tritt, macht `b1` zum Lügner. Man denkt meist, `clone`s einzige Aufgabe sei „eine
Kopie machen". Aber hier hat `clone` eine zweite, verborgene Aufgabe: *den Zustand des
Originals so zu reparieren, dass es sich nicht mehr selbst widerspricht.* Wer das
Problem verursacht, muss aufräumen.

## Der einzige Ausweg: zu geteilt hochstufen

Es gibt keinen sicheren Weg, ein `clone` zu schreiben, das eine allein-besitzende Region
zurückgibt. Um zu entkommen, muss `b1` *aufhören, der Einzelbesitzer zu sein*. Konkret:
Wir verschieben beide Handles auf die dritte Art des Besitzens — geteilt, mit counter.
Dieser Vorgang heißt *Promotion*, das Hochstufen von allein-besitzend auf geteilt.

Wir allokieren einen kleinen neuen Block für den counter — nennen wir ihn `Shared`-Block
— und wickeln ihn in ein `Arc`, damit `Arc` das atomare Zählen und die
Freigabe-bei-0 übernimmt. Das Wichtige: Dieser `Shared`-Block *zeigt* nur auf die
Payload; die Payload selbst *rührt sich nicht*. Also ist das weiterhin Zero-Copy — wir
kopieren den Byte-Lauf nicht neu, wir allokieren nur einen kleinen Extra-Ort zum Zählen.

Dann verschieben wir *sowohl* `b1` *als auch* `b2` auf geteilt, beide auf diesen
`Shared`-Block zeigend, counter auf 2 gesetzt.

```
        Payload (rührt sich nicht)
           ▲              ▲
           │              │
     b1: geteilt     b2: geteilt
           │              │
           └──► Shared ◄──┘      counter = 2
                (Arc)
```

Jetzt zähl neu: Jeder Handle verringert beim Loslassen den counter um eins. Der Speicher
wird genau einmal freigegeben, wenn der counter 0 erreicht. Die Zahl ist zurück bei 1.

Beachte ein Detail, das später zählt: Der `Shared`-Block muss die *Originaladresse* des
allokierten Speichers merken, nicht unbedingt den Pointer, den der Handle gerade hält —
denn ein Teilstück zu nehmen kann den Pointer eines Handles nach vorn schieben, aber bei
der Rückgabe an den Allocator müssen wir genau den Original-Pointer zurückgeben, den er
ausgab.

## Promotion ist eine Einbahnstraße

Es gibt hier eine Zustandskette:

```
Konstante ────────────────────────────────────  (ändert sich nie)

allein-besitzend ──(erstes clone)──► geteilt ──(weitere clones)──► geteilt ──► ...
                   Promotion
                   ◄─── keine Rückrichtung ───
```

Warum geht *geteilt nie zurück zu allein-besitzend*? Weil, sobald es zwei oder mehr
teilende Handles gibt, kein Handle ohne counter weiß, ob es das letzte ist. Um zu
allein-besitzend zurückzukehren, müsste man den counter fallenlassen — aber das verliert
die Fähigkeit zu zählen, und wenn es noch zwei oder mehr Handles gibt, ist das ein
wartendes Double-Free. Also: einmal geteilt, immer geteilt.

(Theoretisch könnte man, wenn der counter auf 1 zurückfällt, zu allein-besitzend
*herabstufen*, um die Atomic-Kosten zu vermeiden. Die echte `bytes`-Crate tut das nicht
— die Komplexität lohnt den Gewinn nicht. Das ist eine „Entscheidung, es *nicht* zu
tun", die Beachtung verdient: Manchmal ist guter Entwurf zu wissen, wo man aufhört.)

Das erklärt auch einen Namen. `b1`s Etikett beginnt als „allein-besitzend", aber es
*kann* „geteilt" werden. In Teil 5 sehen wir, dass dieses Etikett sich nicht selbst
reparieren kann, also trägt die vtable einen Namen, der die Möglichkeit des Wandels
spiegelt — *promotable*, „kann hochgestuft werden". Aber der tiefe Grund für den Namen
ist eine technische Einschränkung aus Teil 5; für jetzt genügt: „allein-besitzend" heißt
hier „gerade allein, aber bereit, geteilt zu werden".

## Das Ungewöhnliche: zurück in einen bereits existierenden Wert schreiben

Nun das, was diesen Teil *konzeptionell* schwer macht, nicht code-technisch.

In normalem Rust wird die Art des Besitzens eines Werts *im Moment seiner Geburt
festgelegt*. Ein `Vec`, als `Vec` geboren, bleibt `Vec` bis zum Tod. Ein `Arc`, als
`Arc` geboren. Man „konvertiert" nie einen lebenden Wert von einer Besitz-Art in eine
andere — man erzeugt einen *neuen* Wert und lässt den alten fallen.

Hier ist es völlig anders. `b1` wird allein-besitzend geboren, aber *mitten in seiner
Lebenszeit in geteilt verwandelt*, von einem *anderen Wert* — `b2` — in genau dem
Moment, in dem `b2` erzeugt wird. `b1` ändert sich nicht selbst; es *wird* von `b2`
geändert.

Genau dieses Ungewöhnliche gebiert die ganze restliche Komplexität. Damit `clone`
(laufend im Auftrag, `b2` zu erzeugen) `b1` reparieren kann, muss es drei *unabhängige*
Anforderungen erfüllen.

Die erste Anforderung: Es muss einen *Weg zu* `b1`s Feld geben. Wenn `clone` gerade
läuft, empfängt es `b1`s `data` als eine *Kopie* — 8 herauskopierte Bytes. Dieser Kopie
einen neuen Wert zuzuweisen, lässt das originale `b1` nichts merken. Um `b1` reparieren
zu können, muss `clone` eine *Referenz auf* das echte Feld empfangen, keine Kopie.

Die zweite Anforderung: Es muss durch diesen Weg *schreiben* können. Selbst mit einer
Referenz auf `b1`s Feld hat `clone` nur eine *geteilte, nur-lesbare* Referenz auf `b1`
(weil `clone`s Signatur in Rust `&self` ist). Durch eine nur-lesbare Referenz zu
schreiben, *verbietet* Rust standardmäßig. Es braucht einen besonderen Mechanismus.

Die dritte Anforderung: Es muss *sicher sein, wenn mehrere Threads es gleichzeitig tun*.
`Bytes` wird über Threads sendbar sein müssen (Teil 5 erklärt, warum das zwingend ist).
Dann können zwei Threads beide eine Referenz auf `b1` halten und beide `clone` rufen,
beide versuchend hochzustufen. Naiv gemacht allokieren die zwei Threads zwei counter,
einer wird verworfen — ein Leak oder Schlimmeres.

Diese drei Anforderungen stammen aus drei völlig verschiedenen Welten — eine dreht sich
um Parameterübergabe (Kopie oder Referenz), eine um Rusts Ausleihregeln (Schreiben durch
eine nur-lesbare Referenz), eine um das Multithread-Speichermodell. Sie wissen nichts
voneinander. Und doch zeigen, wie Teil 5 zeigt, alle drei auf *eine* einzige Änderung
des Typs des `data`-Felds.

## Die Signatur muss sich ändern

Konkret müssen die vtable-Funktionen von „`data` als Kopie nehmen" auf „eine Referenz
auf `data` nehmen" wechseln. Und ein kleines, aber interessantes Detail: Die
`clone`-Funktion nimmt eine *geteilte* Referenz (nur-lesbar, da sie `&self` hat),
während die `drop`-Funktion eine *exklusive* Referenz bekommt (`&mut self`, denn wenn
ein Wert zerstört wird, hält ihn sicher kein anderer Thread mehr). Dieser Unterschied —
„clone bekommt geteilt, drop bekommt exklusiv" — klingt gering, aber in Teil 5 hat er
eine sehr konkrete Folge: `drop` liest `data` ohne Atomics, `clone` braucht sie.

## Was wir haben, und was Teil 5 löst

Teil 4 endet hier: Das Klonen einer allein-besitzenden Region ist ein Double-Free, also
ist *Promotion* — beide Handles auf geteilt hochstufen — der einzige Ausweg; wir
allokieren einen `Shared`-Block mit counter, verschieben beide Handles darauf, die
Payload bewegt sich nicht. Promotion ist Einbahn. Und das ungewöhnliche Kernstück:
`clone` muss *zurück* in `b1` schreiben — einen bereits existierenden Wert —, weil der
Akt des Klonens `b1`s Versprechen falsch macht.

Dieses Zurückschreiben stellt drei unabhängige Anforderungen: ein Weg zum Feld, die
Fähigkeit, durch eine nur-lesbare Referenz zu schreiben, und Multithread-Sicherheit.
Teil 5 zeigt, dass alle drei auf einen einzigen Feldtyp zusammenlaufen, und dass jede
Anforderung einem Stück des Concurrency-Puzzles entspricht — interior mutability
(Schreiben durch eine nur-lesbare Referenz), CAS (genau einen Gewinner beim Wettlauf
wählen) und Memory Ordering (garantieren, dass der andere Thread das eben Hochgestufte
*sieht*). Es ist der abstrakteste Teil der Serie, und wir gehen langsam vor.

---

*Weiter: [Teil 5 — `AtomicPtr`: sicher zurückschreiben](05_atomics.md) ·
[Inhalt](00_index.md)*

*English: [`../en/04_promotion.md`](../en/04_promotion.md)*
