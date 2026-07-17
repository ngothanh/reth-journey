# Teil 3 — „Welche Bytes" von „wer besitzt sie" trennen

Teil 2 hinterließ eine Schuld. Wir haben den Mechanismus für einen `Bytes`-Typ, der
drei Aufräumungen trägt, aber die zweite Hälfte der Anforderung nicht beantwortet: Das
Lesen der Bytes muss *schnell* bleiben. Ein `Bytes` hat jetzt vier Felder — Pointer,
Länge, `data`, `vtable` — deutlich mehr als `Arc<[u8]>`, das nur einen Pointer hatte.
Macht dieses „mehr" das Lesen langsamer?

Die Antwort ist nein, und *warum* es nein ist, ist die schönste Idee des ganzen
Entwurfs. Sie ist auch das, was diesen Entwurf einen naheliegenderen Ansatz *schlagen*
lässt, den wir am Ende vergleichen.

## Ein Handle beantwortet zwei völlig unabhängige Fragen

Schau noch einmal auf ein `Bytes` und beachte, dass seine vier Felder sich eigentlich
in zwei Gruppen teilen, die zwei völlig getrennte Fragen beantworten.

Erste Frage: *welche Bytes?* — wo sie sind, wie lang sie sind. Beantwortet durch Pointer
und Länge.

Zweite Frage: *wer besitzt sie?* — was beim Klonen oder Loslassen zu tun ist.
Beantwortet durch `data` und `vtable`.

```rust
struct Bytes {
    ptr:    /* Pointer */,        // ┐ "welche Bytes"
    len:    /* Länge */,          // ┘
    data:   /* 8 extra Bytes */,  // ┐ "wer besitzt"
    vtable: /* Tabellen-Pointer */,// ┘
}
```

Der Kern: Der Byte-Inhalt ist *immer* nur ein roher Lauf von Bytes, egal ob er von
einer Konstante, einer allein-besitzenden oder einer geteilten Region kommt. Es gibt
keine „verborgene Form", die man später entdecken müsste. Also beantworten Pointer und
Länge die erste Frage *vollständig*, und die erste Frage *braucht nie* die Antwort der
zweiten.

Die Gegenrichtung ist nicht symmetrisch: `clone` und `drop` brauchen `data`/`vtable`,
aber sie brauchen *auch* Pointer und Länge (um einen neuen Handle zu bauen, um die
richtige Adresse freizugeben). Also liest die „wer besitzt"-Gruppe beide Gruppen; die
„welche Bytes"-Gruppe aber liest nur ihre eigene. Genau diese Asymmetrie nutzen wir aus.

Ein Bild: Pointer und Länge sind wie *Regalposition und Seitenzahl* eines Buches. `data`
und `vtable` sind wie die *Leihkarte* hinten im Einband: wer dieses Exemplar ausleiht,
was bei der Rückgabe zu tun ist. Die Leihkarte sagt dir nichts darüber, was im Buch
steht — und du liest das ganze Buch, ohne die Karte auch nur einmal anzuschauen.

## Dadurch ist das Lesen gratis

Die direkte Folge: Jede Lese-Operation — Inhalt holen, Länge holen, vergleichen, hashen,
ausgeben — berührt nur Pointer und Länge. Kein `vtable`-Nachschlagen. Keine Verzweigung
nach der Besitz-Art. Kein counter berührt. Den Inhalt holen ist nur „von diesem Pointer
lies so viele Bytes" — eine Zeile, und *genau* das, was auch `Arc<[u8]>` kompiliert.

Das ist der ganze Grund, warum das Lesen billig bleibt. Die zwei neu hinzugefügten
Felder (`data`, `vtable`) kosten auf dem Hot-Path nichts, einfach weil der Hot-Path nur
liest, und beim Lesen sind diese zwei Felder unsichtbar. Der Preis der Flexibilität —
ein Typ trägt drei Aufräumungen — wird *ganz* auf `clone` und `drop` abgeladen, zwei
*kalte* Operationen, die selten laufen; und er *sickert nicht* ins Lesen, die *heiße*
Operation, die ständig läuft.

Das ist ein überall nutzbares Entwurfsprinzip, nicht nur für `Bytes`: Wenn du Zustand
für Flexibilität hinzufügst, ordne das Layout so an, dass dieser Zustand *außerhalb des
Hot-Paths* sitzt. Muss der Hot-Path den neuen Zustand *anschauen* — sei es nur eine
Verzweigung —, dann ist die Flexibilität mit ihren Kosten an die teuerste Stelle
gesickert.

## Warum nicht einfach ein `enum`

Hier werden viele fragen: Wozu der Aufwand mit vtable und Funktionspointern, wo Rust
doch ein `enum` hat, um „eines von dreien" darzustellen?

```rust
enum Bytes {
    Static { /* ... */ },
    Owned  { /* ... */ },
    Shared { /* ... */ },
}
```

Dieser Weg ist *korrekt*. Er ist sogar *sicherer* (kein `unsafe`-Code). Warum wählt der
echte Entwurf ihn also nicht?

Weil ein `enum` das unterscheidende Tag (die „wer besitzt"-Frage) *zusammen* mit den
Daten (der „welche Bytes"-Frage) legt. Bei jedem Lesen musst du dieses Tag `match`en —
eine Verzweigung —, um Pointer und Länge herauszuholen, *obwohl das Lesen der Bytes
nichts mit dem Tag zu tun hat*. Du bezahlst die „wer besitzt"-Frage bei *jedem* Stellen
der „welche Bytes"-Frage.

Bei unserem flachen Layout — Pointer und Länge immer an derselben festen Position für
alle drei Arten — holt das Lesen sie direkt heraus, ohne Verzweigung. Der
`vtable`-Pointer liegt abseits und wird nur von `clone` und `drop` berührt.

Der Kompromiss hier ist echt und verdient klare Worte: Das flache Layout zu wählen,
heißt, die statische Sicherheit des `enum` aufzugeben (du schreibst `unsafe`-Code und
hältst die Invariante „`data` muss zur `vtable` passen" selbst ein), im Tausch gegen
verzweigungsfreies Lesen. Für einen Typ, dessen Lese-Operationen ständig in heißen
Schleifen aufgerufen werden, lohnt dieser Tausch. Für einen selten gelesenen Typ ist das
`enum` die richtige Wahl. Zu wissen, wo man auf diesem Spektrum steht, ist Teil der
Entwurfskunst — nicht immer gewinnt „schneller".

## Die tragende Denkweise: Freigaben zählen

Nun zur „wer besitzt"-Gruppe, und wir legen das Fundament für die zwei schwersten Teile.
Die drei Arten des Besitzens klingen verschieden, aber sie sind wirklich nur drei
Antworten auf *dieselbe* Frage:

> Wie oft genau wird diese Speicherregion freigegeben (`dealloc`), und von wem?

- Eine Konstante: **0**-mal freigegeben. Sie wurde nie allokiert; man kann nicht
  zurückgeben, was man nie geliehen hat.
- Eine allein-besitzende Region: **1**-mal freigegeben, vom Handle selbst, beim
  Loslassen.
- Eine geteilte Region: **1**-mal freigegeben, vom *letzten* Handle, wenn der counter 0
  erreicht.

Die richtige Zahl ist immer wie oben. Und das macht diese Frage zu einem Werkzeug statt
zu einem Spruch: *Jeder Bug in diesem Entwurf reduziert sich auf das Falschzählen dieser
Zahl.* **0** zählen, wenn es 1 sein sollte, ist ein Leak. **2** zählen, wenn es 1 sein
sollte, ist ein Double-Free oder Use-after-Free. Durch Teil 4 und 5 fragst du, wann
immer du unsicher bist „ist das richtig": *Für genau diese Region, welche Zahl habe ich
gerade erzeugt?*

Aus dieser Denkweise treten zwei wichtige Dinge hervor.

## `Drop` räumt nicht die Struktur auf — es macht eine Allokation rückgängig

Schau noch einmal auf die vier Felder von `Bytes`: ein Pointer, eine Zahl, `data`
(Pointer oder Zahl), `vtable` (eine statische Referenz). *Keines besitzt irgendetwas.*
Löschtest du den `impl Drop for Bytes`-Block ganz, wäre das Loslassen eines `Bytes`
bereits eine perfekte Nulloperation — die Struktur verschwindet vom Stack, ohne fremde
Hilfe.

Wozu also gibt es `Drop`? *Nur* um die Heap-Region zurückzugeben, auf die die Struktur
zeigt. Hier geht die Intuition oft daneben: `Drop` ist nicht dazu da, „den Wert selbst
aufzuräumen" — der Wert löst sich von selbst auf. `Drop` existiert allein, um *eine
frühere Allokation rückgängig zu machen*. Wurde nie allokiert, gibt es nichts
rückgängig zu machen.

Genau darum ist die `drop`-Funktion einer Konstante eine *leere* Funktion, und diese
Leere ist *richtig*. Eine Byte-Konstante wurde nie allokiert, also muss sie 0-mal
freigegeben werden, also tut ihre Aufräum-Funktion nichts. Anders gesagt: keine
Allokation, kein `Drop`. Eine Konstante zu leaken ist *richtig* — sie lebt das ganze
Programm lang, egal was du tust.

## Die stille Falle: die größte Lektion über unsafe-Code

Die Freigabe-Funktion einer allein-besitzenden Region muss genau so viele Bytes
zurückgeben, wie *allokiert* wurden, nicht wie viele *geschrieben* wurden. Erinnere dich
an Teil 1: Ein `BytesMut` könnte 1024 Bytes allokiert, aber erst 7 geschrieben haben.
Beim Freigeben verlangt der Allocator genau den 1024-Byte-Block zurück, den er ausgab —
er matcht auf die *allokierte Größe*, nicht auf den Inhalt.

Was passiert, wenn du versehentlich nach geschriebenen Bytes (7) statt allokierten
(1024) zurückgibst?

Unter Linux und macOS ruft die finale Freigabe letztlich Cs `free(ptr)` auf — das
*einen* Parameter nimmt, den Pointer; es schlägt die Größe aus Metadaten direkt vor dem
Block nach und *verwirft* die Größe, die du übergibst. Folge: Das Programm **stürzt
nicht ab**. Alle Tests bestehen. Läuft zehn Millionen Mal, besteht. Läuft zwei Jahre in
Produktion, besteht.

Aber es ist Undefined Behavior. Der Vertrag der Freigabe verlangt, dass die
Größe-beim-Freigeben gleich der Größe-beim-Allokieren ist. Der Tag, an dem es
hochgeht, ist der Tag, an dem jemand den Standard-Allocator gegen einen anderen tauscht
— etwa `jemalloc` oder `mimalloc` —, die der übergebenen Größe *vertrauen* und sie zur
Bucket-Wahl nutzen. Er gibt den 1024-Byte-Block in den Bucket für 8-Byte-Blöcke zurück;
ein paar tausend Allokationen später schreiben zwei Programmteile in dieselbe Region;
und du hast Heap-Korruption an einer völlig unzusammenhängenden Stelle, unauffindbar.

Das ist die einzuprägendste Lektion über unsafe-Code, und sie ist das Gegenteil der
Intuition:

> Der gefährliche Bug in unsafe-Rust ist nicht der, der abstürzt — es ist der, der
> *korrekt läuft*. Die Intuition aus sicherem Rust — „falsch heißt sofort Panik" — ist
> hier umgekehrt: Der Standard eines Fehlers ist *Stille*.

Das Werkzeug, das ihn fängt, ist `miri` — ein Interpreter, der kein echtes `free`
ausführt, sondern *den Vertrag prüft*: Er merkt sich die Größe bei der Allokation,
vergleicht sie bei der Freigabe und schreit sofort „incorrect layout on deallocation",
an der richtigen Zeile. Auch darum muss jedes doppelt genutzte Feld — wie `data`, mal
eine Zahl, mal ein Pointer — direkt an Ort und Stelle dokumentiert und per Test geprüft
werden: Der Fehler zeigt sich zur Laufzeit nicht von selbst.

## Drei „leichte" Verhalten fallen heraus

Mit der Freigaben-zählen-Denkweise treten drei der vier Verhalten hervor — und das
Schöne: *keines berührt noch Multithreading*. Darum baust du sie zuerst; Teil 4 und 5
sind der schwere Teil.

Bei einer **Konstante**, 0-mal freigegeben. Eine konstante `Bytes` zu erzeugen ist nur,
einen Handle auf den existierenden Byte-String zu bauen, `vtable` als die Konstanten-
Tabelle zu setzen und `data` leer zu lassen. Ihre Aufräum-Funktion ist leer. Fertig.

Bei einer **allein-besitzenden Region** töten wir endlich das memcpy aus Teil 1. Wir
wollen den Speicher 1-mal freigeben, nicht 2. Das Problem: Der alte `BytesMut` *wird*
den Speicher freigeben, wenn er zerstört wird (er hat seine eigene Aufräumung); dann
*wird auch* der neue `Bytes` freigeben. Das sind 2 — ein Double-Free. Um auf 1 zu
kommen, dürfen wir den `BytesMut` seine Aufräumung *nicht* ausführen lassen.

Rust hat dafür ein Werkzeug: `mem::forget`. Der Name klingt nach „eine Variable
löschen", aber es ist wirklich eine Erklärung:

> „Ich habe diesen Speicher an jemand anderen übergeben. Führe meine Aufräumung nicht
> aus."

Das ist die *Definition* einer Zero-Copy-Übergabe: Der Empfänger übernimmt den Speicher
des Senders (keine Kopie), und eine Region bekommt genau einen Aufräumer. Der Puffer
selbst rührt sich nicht; nur die *Verantwortung, ihn freizugeben*, geht von `BytesMut`
an `Bytes`. Also läuft `freeze` so: Pointer / Länge / allokierte-Größe aus dem
`BytesMut` auslesen; `mem::forget` rufen, damit der `BytesMut` nicht mehr aufräumt; dann
eine allein-besitzende `Bytes` bauen, die auf genau diesen Puffer zeigt, mit `data` als
allokierter Größe.

Ein feiner Punkt: `mem::forget` ist *normalerweise ein Leak* — das ist sein
Hauptzweck (und seine Gefahr). Hier leakt es *nicht*, nur weil wir den Pointer
*vorher* ausgelesen und an `Bytes` übergeben haben. `mem::forget` prüft das nicht;
*du* musst sicherstellen, dass jemand übernimmt. Darum ist die Reihenfolge
auslesen-dann-forget zwingend; drehst du sie um, stoppt dich der Compiler (du hättest
`self` schon in `forget` übergeben). Das ist schwer falsch zu machen — genau die Art
Code, die wir mögen.

Und die Freigabe-Funktion der allein-besitzenden Region, wie oben besprochen: den
Speicher nach der aus `data` gelesenen allokierten Größe zurückgeben. Denk an die stille
Falle — allokiert, nicht geschrieben.

## Was wir haben, und die Wand voraus

Nach Teil 3 haben wir eine `Bytes`, die für *zwei der drei* Arten funktioniert, mit
einem gratis Lesepfad und noch ohne Multithreading. Eine Konstante klont durch Kopieren
der Struktur und räumt mit einer leeren Funktion auf. Eine allein-besitzende Region hat
ein `freeze` als konstantzeitige Übergabe — Teil 1s memcpy ist tot, also ist die harte
Anforderung aus Teil 1 erfüllt — und räumt durch Freigabe nach allokierter Größe auf.
Die Freigaben-zählen-Denkweise steht als Diagnosewerkzeug bereit.

Genau ein Verhalten bleibt: das Klonen einer allein-besitzenden Region. Und es macht
alles kaputt. Nimm die Zähl-Denkweise und probier: Klone eine allein-besitzende Region
und lass *beide* Handles allein-besitzend, dann geben beide frei — Zahl ist 2 —
Double-Free.

Warum das unvermeidlich ist und warum sein Ausweg etwas sehr Ungewöhnliches in Rust
erzwingt — zurück in einen bereits existierenden Wert zu schreiben — ist Teil 4, der
schwerste Teil der ganzen Serie.

---

*Weiter: [Teil 4 — Die Wand: wenn clone alles kaputt macht](04_promotion.md) ·
[Inhalt](00_index.md)*

*English: [`../en/03_split_and_counting.md`](../en/03_split_and_counting.md)*
