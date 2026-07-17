# Teil 2 — Ein Typ, viele Verhalten

Teil 1 endete auf einem Widerspruch. Wir brauchen einen einzigen `Bytes`-Typ (weil die
API-Grenze es verlangt), aber dieser Typ muss drei verschiedene Arten kennen, Speicher
aufzuräumen — bei einer Konstante nichts tun, bei einer allein-besitzenden Region
freigeben, bei einer geteilten Region zählen-dann-freigeben — und welche gilt, ist
erst zur Laufzeit bekannt. Dieser Teil sucht den Mechanismus, der das möglich macht.

Aber bevor wir etwas Neues erfinden, lohnt ein genauer Blick darauf, wie Rust dieses
Problem *normalerweise* löst. Denn es stellt sich heraus, dass unser Entwurf gar nichts
Neues ist — er ahmt nur nach, was Rust ohnehin tut, eine Ebene tiefer.

## Normalerweise lebt „wie man aufräumt" schon im Typ

Schau auf drei vertraute Typen und beachte, was entscheidet, wie jeder aufgeräumt wird:

```rust
&'static [u8]   // ein konstanter Byte-String, lebt das ganze Programm lang
Vec<u8>         // ein Array von Bytes, das wir besitzen
Arc<[u8]>       // ein geteiltes Array von Bytes, mit counter
```

Wenn ein `Vec<u8>` aus dem Gültigkeitsbereich fällt und zerstört wird, führt der
Compiler den Code aus, der seinen Speicher freigibt. Wird ein `Arc<[u8]>` zerstört,
führt der Compiler den Code aus, der den counter verringert und nur bei 0 freigibt.
Wird ein `&'static [u8]` zerstört, führt der Compiler gar nichts aus — es ist eine
Referenz auf eine Konstante, nichts aufzuräumen.

Das Bemerkenswerte: **Du musst dem Compiler nie sagen, welchen Code er ausführen
soll.** Er schaut auf den *Typ* und weiß es. `Vec` nimmt die Art von `Vec`, `Arc` die
Art von `Arc`. Diese Entscheidung fällt *zur Kompilierzeit*, und sie ist gratis — es
gibt keinen „zur Laufzeit fragen, welche Art das ist"-Schritt, weil der Typ die Frage
vorab beantwortet hat.

Anders gesagt: In Rust ist ein *Typ* nicht nur die Form der Daten. Er *trägt* auch
eine Subroutine mit sich — wie man klont, wie man droppt —, die der Compiler
nachschlägt und einfügt. Man kann sich einen Typ als unsichtbare, statische
Nachschlagetabelle für Verhalten vorstellen.

Genau darum ist Rust schnell: Jede Art des Besitzens ist ihr eigener Typ, jede
Aufräum-Entscheidung ist zur Kompilierzeit erledigt, und das laufende Programm muss
nicht nachdenken.

Und genau das wollte der „drei getrennte Typen"-Ansatz vom Ende von Teil 1 ausnutzen —
den Compiler es für uns tun lassen. Er stirbt nur, weil die API einen einzigen Typ
braucht. Also müssen wir jetzt selbst tun, was der Compiler sonst still tut — aber zur
Laufzeit.

## Der Zusammenstoß: ein Typ hat nur ein `Drop`

Die „ein Typ"-Anforderung stößt frontal auf die Art, wie Rust Verhalten an Typen
bindet. In Rust definiert man mit einem `Drop`-Block, „was passiert, wenn ein Wert
zerstört wird":

```rust
impl Drop for Bytes {
    fn drop(&mut self) {
        // Was schreiben wir HIER hinein?
    }
}
```

Aber dieser Block wird *einmal* für den `Bytes`-Typ geschrieben. Und du kannst keine
Zeile schreiben, die für alle drei Fälle richtig ist:

- Schreib „Speicher freigeben", und es ist richtig für eine allein-besitzende Region,
  aber für eine Konstante gibt es Speicher frei, den wir nicht besitzen — das Programm
  stürzt ab.
- Schreib „nichts tun", und es ist richtig für eine Konstante, aber für eine
  allein-besitzende Region leakt es — der Speicher wird nie zurückgegeben.
- Schreib „counter verringern", und es ist richtig für eine geteilte Region, aber eine
  Konstante und eine allein-besitzende Region haben keinen counter zum Verringern.

Es gibt keine *feste* Code-Zeile, die für alle drei richtig ist, weil die zutreffende
Art erst zur Laufzeit bekannt ist, pro Wert — dieses `b1` ist eine Konstante, jenes
`b2` allein-besitzend, jenes `b3` geteilt. Der Compiler ist am Ende: Er wählt Verhalten
*nach Typ*, und wir haben nur einen Typ.

Also muss die „welches Verhalten"-Entscheidung den Compiler verlassen und zur Laufzeit
wandern. Die Frage ist, wie.

## Die Idee: „wie man aufräumt" in den Wert selbst legen

Kann der Compiler nicht zur Kompilierzeit wählen, dann lass *den Wert die Wahl selbst
tragen*. Und um zur Laufzeit zu wählen, muss das, womit man wählt, *Daten in der
Struktur* sein, nicht der Typ.

Konkret: Wir nehmen genau das, was ein „Typ" unsichtbar trägt — die clone/drop-
Subroutine — und machen daraus ein sichtbares Feld. In Rust ist „eine Subroutine, die
man in einer Variablen speichern kann" ein *Funktionspointer*. Also sammeln wir zwei
Funktionspointer in einer kleinen Tabelle:

```rust
struct Vtable {
    clone: /* Pointer auf fn: "diese Art klonen: dies ausführen" */,
    drop:  /* Pointer auf fn: "diese Art droppen: dies ausführen" */,
}
```

(„Vtable" ist der traditionelle Name für so eine Tabelle — *virtual table*, eine
Tabelle virtueller Funktionen.)

Dann erstellen wir drei dieser Tabellen im Voraus, eine pro Art des Besitzens, und
lassen sie das ganze Programm lang leben:

```rust
static STATIC_VTABLE: Vtable = /* clone, drop im Konstanten-Stil */;
static OWNED_VTABLE:  Vtable = /* clone, drop im Allein-Stil */;
static SHARED_VTABLE: Vtable = /* clone, drop im Geteilt-Stil */;
```

Und jeder `Bytes`-Wert trägt einen Pointer auf eine dieser drei Tabellen:

```rust
struct Bytes {
    /* ... wo die Bytes sind ... */
    vtable: &'static Vtable,   // dieser Pointer entscheidet das Schicksal des Werts
}
```

Jetzt hat `Bytes`' `Drop`-Funktion nur eine Aufgabe: *das `vtable`-Feld lesen und die
`drop`-Funktion darin aufrufen*. Ein Wert, der `STATIC_VTABLE` hält, ruft die
Nichts-tun-Funktion; einer, der `OWNED_VTABLE` hält, die Freigabe-Funktion. Ein
einziger `impl Drop`-Block, drei verschiedene Verhalten, jeweils pro Wert zur Laufzeit
richtig gewählt. Genau, was wir brauchen.

Die knappste Sicht auf das, was gerade passiert ist:

> Die vtable *ist* der Typ, nur von der Kompilierzeit-Ebene herab zu einem
> Laufzeitwert degradiert. Die drei Typen `&'static [u8]`, `Vec<u8>`, `Arc<[u8]>`
> verschwinden nicht — sie werden zu drei *Werten* `STATIC_VTABLE`, `OWNED_VTABLE`,
> `SHARED_VTABLE`, die wir einem Feld zuweisen, vergleichen und aufrufen können, zur
> Laufzeit.

### Du hast eigentlich schon eine vtable benutzt

Wenn du je `&dyn IrgendeinTrait` in Rust geschrieben hast, hast du eine vtable benutzt
— der Compiler hat die Tabelle nur für dich gebaut. Ein `&dyn Trait` ist intern ein
Paar Pointer: einer auf die Daten, einer auf die Tabelle der Methoden des Traits. Rufst
du eine Methode über `dyn` auf, schlägt das Programm die richtige Funktion in dieser
Tabelle nach und ruft sie. Genau diesen Mechanismus bauen wir von Hand.

Der einzige Unterschied zwischen `dyn` und dem, was wir tun:

- Bei `dyn` ist jeder Fall ein *anderer Typ* (ein `u64`, ein `String`...), also weiß
  der Compiler, welche Tabelle er für welchen Typ baut.
- Hier sind alle drei Fälle *bereits derselbe Typ* `Bytes`. Der Compiler hat nichts
  mehr zu unterscheiden, also kann er die Tabelle nicht bauen — wir bauen sie von Hand
  und weisen selbst zu, welche Tabelle zu welchem Wert gehört, im Moment der Erzeugung.

Das gibt uns eine später nutzbare Regel: Wann sollte man eine vtable von Hand
schreiben? Genau dann, wenn *ein* Typ *mehrere* Verhalten braucht, pro Wert zur
Laufzeit gewählt. Bei mehreren Typen → `dyn`. Bei einem Verhalten → eine normale
Funktion. Eine handgeschriebene vtable füllt genau die Lücke „ein Typ, viele Verhalten,
pro Wert".

## Warum die Tabelle genau zwei Slots hat

Das ist die Frage, die jeder, der so einen Typ entwirft, beantworten können sollte,
denn sie ist der Test, ob man den Entwurf *versteht* oder nur abgeschrieben hat. Wir
betrachten beide Richtungen: warum nicht *weniger* als zwei und warum nicht *mehr* als
zwei.

Es gibt eine einfache Entscheidungsregel. Ein Funktions-Slot verdient nur dann einen
Platz in der vtable, wenn sich das Verhalten der Operation *danach ändert, wer den
Speicher besitzt*. Tut eine Operation dasselbe, egal welcher Besitz, ist es sinnlos —
schlimmer, schädlich —, sie in die vtable zu legen, wie wir sehen werden.

Versuche, *alles* aufzulisten, was ein `Bytes` tun kann, und frage bei jedem „hängt es
vom Besitz ab":

- Bytes auslesen (Länge holen, Inhalt holen, vergleichen, ausgeben): Bei allen drei
  Arten lautet die Antwort „schau einfach auf Pointer und Länge und lies". Hängt
  *nicht* vom Besitz ab.
- `clone`: Eine Konstante kopiert die ganze Struktur; eine allein-besitzende Region
  muss etwas Kompliziertes tun (Teil 4); eine geteilte erhöht den counter. Hängt vom
  Besitz *ab*.
- `drop`: Eine Konstante tut nichts; eine allein-besitzende Region gibt frei; eine
  geteilte verringert. Hängt vom Besitz *ab*.

Genau zwei Operationen hängen vom Besitz ab. Also genau zwei Slots: `clone` und `drop`.

Warum nicht *weniger*? Kann man `clone` und `drop` in einen Slot verschmelzen? Nein —
sie sind zwei unabhängige Operationen zu zwei verschiedenen Zeitpunkten (eine beim
Duplizieren eines Handles, eine beim Loslassen eines Handles), und keines Verhalten
folgt aus dem anderen. Streich den `drop`-Slot, und du weißt nicht, wie man freigibt;
streich den `clone`-Slot, und du weißt nicht, wie man dupliziert. Jedes hängt auf seine
Weise vom Besitz ab, also braucht jedes seinen eigenen Slot.

Und die „Bytes lesen"-Hälfte — warum *kein* Slot? Weil sie nicht vom Besitz abhängt.
Pointer und Länge beantworten „welche Bytes" vollständig, bei allen drei Arten
identisch. Ihr einen vtable-Slot zu geben, hieße, bei jedem Lesen einen *indirekten
Funktionsaufruf* zu bezahlen — für eine Operation, die nicht interessiert, wer was
besitzt. Das ist der Keim von Teil 3, aber schon hier sichtbar: Eine Operation, die
nicht vom Besitz abhängt, ist aus der vtable *verboten*, weil ihre Aufnahme den
Hot-Path für nichts zahlen lässt.

Warum nicht *mehr*? Es gibt zwei naheliegende Einwände.

Erstens: „einen `slice`-Slot hinzufügen?" — weil ein Teilstück eines `Bytes` zu nehmen
*scheinbar* vom Besitz abhängt (eine Konstante slicen ergibt eine Konstante; eine
allein-besitzende Region slicen kann keine andere allein-besitzende ergeben, denn das
wären zwei Besitzer eines Blocks). Aber `slice` **lässt sich über `clone` ausdrücken**:
ein Teilstück nehmen ist nur „den Handle duplizieren (damit `clone` den Besitzteil
regelt), dann Pointer und Länge auf das Teilstück verengen". `clone` weiß schon, wie
man für alle drei Arten korrekt dupliziert. Ein Slot, der *aus einem anderen Slot
rekonstruierbar* ist, verdient seinen Platz nicht.

Zweitens: „alles in einen Slot kollabieren, der ein Enum zurückgibt, das die Art
angibt, und dann darauf verzweigen?" — also eine `kind()`-Funktion, die die Art
zurückgibt, dann `match`en `clone` und `drop` darauf. Es funktioniert, aber du
dispatcht *zweimal*: einen indirekten Aufruf (`kind` rufen), *dann* eine Verzweigung
(`match`). Während der ganze Reiz eines Funktionspointers ist, dass *ihn aufzurufen
bereits das Dispatchen ist* — ein Schritt. Zudem ist der Enum-Ansatz *geschlossen*:
Eine vierte Art des Besitzens hinzuzufügen hieße, *jedes* `match` im Codebase zu
ändern; mit einer vtable fügst du nur eine neue `static`-Tabelle hinzu und rührst sonst
nichts an.

Zusammengefasst: genau zwei Slots, weil genau zwei Operationen vom Besitz abhängen und
nicht auseinander rekonstruierbar sind. Weniger verliert Fähigkeit; mehr ist entweder
redundant (rekonstruierbar) oder langsam (zweimal dispatchen). Die mitzunehmende Regel:
Ein Slot verdient seinen Platz genau dann, wenn er von verborgenem Zustand abhängt
*und* nicht aus den anderen Slots rekonstruierbar ist.

(Eine praktische Notiz für Neugierige: Die echte `bytes`-Crate hat fünf Slots, nicht
zwei. Die drei zusätzlichen sind *reine Optimierungen* — jede vermeidet eine gemessene
Kopie. Etwa „in einen `Vec` verwandeln" auf einem Puffer, der zufällig allein-besitzend
ist, kann den Speicher direkt übergeben, statt zu kopieren — aber nur, wenn es fragen
kann „bin ich gerade allein-besitzend?", was nur die vtable weiß. Jeder zusätzliche
Slot ließe sich aus `clone` plus `drop` bauen; sie existieren, weil jemand eine
lohnende Kopie *gemessen* hat. Jeder zusätzliche Slot kostet einen indirekten Aufruf,
also musste sich jeder mit einem Benchmark *seinen Sitz verdienen*. Dieser Lernbau
macht zwei — das ist die korrekte Minimalmenge.)

## Der Begleiter der vtable: ein Datenfeld

Die vtable sagt *wie* man klont/droppt. Aber „wie" braucht meist ein begleitendes
*Datum*. Die Freigabe-Funktion einer allein-besitzenden Region muss wissen, wie lang
die allokierte Region ist, um genau so viel zurückzugeben. Die Verringerungs-Funktion
einer geteilten Region muss wissen, an welcher Adresse der counter liegt. Die Funktion
einer Konstante braucht nichts.

Woher kommt dieses Datum? Wir fügen ein weiteres Feld hinzu, vorläufig `data` genannt.
Es muss *drei verschiedene Arten von Information* tragen, je nach vtable:

- bei einer Konstante wird `data` nicht benutzt;
- bei einer allein-besitzenden Region hält `data` die Länge der allokierten Region —
  eine *Zahl*, keine Adresse;
- bei einer geteilten Region hält `data` die *Adresse* des counters — ein echter
  Pointer.

Mal eine Zahl, mal ein Pointer. Kein „ordentlicher" Rust-Typ beschreibt das. Also
deklarieren wir es mit dem formlosesten verfügbaren Typ: **`*mut ()`**. Das ist ein
*Raw Pointer* auf `()` — Rusts „leeren" Typ; anders gesagt ist es das `void*` von C:
genau **8 Bytes** (auf einer 64-Bit-Maschine), ohne Bedeutung, bis jemand es
interpretiert. Lies `data` nicht als „einen Pointer"; lies es als „8 Bytes, Bedeutung
aufgeschoben". Bei einer allein-besitzenden Region stopfen wir die Längenzahl direkt in
diese 8 Bytes (eine Ganzzahl im Pointer-Gewand); bei einer geteilten sind diese 8 Bytes
eine echte Adresse.

`data` und `vtable` reisen immer als Paar: `data` sind 8 an sich bedeutungslose Bytes,
und die `vtable` ist das *Einzige*, das weiß, was diese 8 Bytes diesmal bedeuten. Darum
nimmt — wie du beim Coden sehen wirst — jede vtable-Funktion `data` als erstes Argument:
Wir reichen die 8 bedeutungslosen Bytes an die eine Funktion, die weiß, wie man sie
liest.

## Zusammenfassung: wo der Entwurf steht

Zusammengesetzt sieht unser `Bytes` nach Teil 2 so aus — zum ersten Mal mit echten
Typen, keine Platzhalter mehr:

```rust
struct Bytes {
    ptr:    NonNull<u8>,      // "welche Bytes": zeigt auf den Anfang des Laufs
    len:    usize,            // "welche Bytes": wie lang
    data:   *mut (),          // "wer besitzt": 8 Bytes, Bedeutung von der vtable definiert
    vtable: &'static Vtable,  // "wer besitzt": welches clone/drop-Set benutzen
}

struct Vtable {
    clone: /* Funktionspointer */,   // diese Art klonen: dies ausführen
    drop:  /* Funktionspointer */,   // diese Art droppen: dies ausführen
}
```

Und die drei möglichen `data`/`vtable`-Kombinationen:

| vtable          | was `data` hält            | was `drop` tut |
|-----------------|----------------------------|----------------|
| `STATIC_VTABLE` | (ungenutzt)                | nichts         |
| `OWNED_VTABLE`  | Länge der Region (Zahl)    | freigeben      |
| `SHARED_VTABLE` | Adresse des counters (Ptr) | verringern     |

Behalte diese Tabelle im Kopf; in Teil 5 ändert sie sich um genau *eine* Zeile — den
Typ des `data`-Felds — und wir sehen, warum `*mut ()` nicht genügt. (Der Grund liegt in
`clone` und ist noch nicht sichtbar.)

## Was wir haben, und was Teil 3 als Nächstes löst

Wir haben jetzt die erste Hälfte von Teil 1s Frage gelöst: ein einziger `Bytes`-Typ,
der drei verschiedene Aufräumungen trägt und die richtige pro Wert zur Laufzeit wählt,
dank eines `vtable`-Felds, das auf eine von drei vorgebauten Tabellen zeigt.

Aber es gibt die zweite Hälfte, nicht minder wichtig: *Das Lesen der Bytes muss so
billig bleiben wie bei `Arc<[u8]>`.* Im „zwei Slots"-Abschnitt haben wir es erahnt: Die
Lese-Operationen sind *nicht* in der vtable. Aber „nicht in der vtable" reicht allein
nicht. Wir müssen die Felder der Struktur auch so anordnen, dass das Lesen `data` oder
`vtable` *absolut nie* berührt, nicht einmal eine Verzweigung. Warum diese Anordnung ein
`enum` schlägt und warum sie die zwei neu hinzugefügten Felder auf dem Hot-Path *gratis*
macht, ist Teil 3.

Teil 3 führt auch eine Denkweise ein, die das Rückgrat der zwei schwersten
verbleibenden Teile bildet: Jede Art, Speicher zu besitzen, reduziert sich zutiefst auf
eine Frage — *wie oft wird diese Region freigegeben?*

---

*Weiter: [Teil 3 — „Welche Bytes" von „wer besitzt"](03_split_and_counting.md) ·
[Inhalt](00_index.md)*

*English: [`../en/02_vtable.md`](../en/02_vtable.md)*
