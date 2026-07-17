# Teil 1 — Die Reise eines Bytes vom Draht ins Programm

Beginnen wir mit einem ganz gewöhnlichen Programm: einer laufenden Ethereum-Node.
Sie öffnet einen Socket, und am anderen Ende schickt die ganze Welt Daten — neue
Blöcke, Transaktionen, Nachrichten zwischen Peers. Die Aufgabe der Node auf unterster
Ebene ist es, diese Bytes einzulesen, ihnen Sinn zu geben und sie entweder zu
speichern oder an einen anderen Peer weiterzureichen.

Klingt einfach, aber schon beim Schritt „Bytes einlesen" steckt ein Problem, um das
sich diese ganze Serie dreht. Wir gehen sehr langsam vor, denn alles Spätere wächst
von hier aus.

## Bytes kommen nicht ordentlich an

Wenn du von einem Socket liest, kommen die Daten nicht als ein sauberer Block bekannter
Länge. Sie kommen in Häppchen — mal 40 Bytes, mal 1500, je nach Netz. Eine vollständige
Nachricht — etwa ein Block-Header — muss vielleicht aus fünf, sechs solcher Lesevorgänge
zusammengesetzt werden. Und du **weißt vorher nicht**, wie lang die Nachricht ist, bis
du fast alles gelesen hast.

Also brauchst du einen Ort zum Auffangen: eine Speicherregion, an die du immer weiter
Bytes anhängst und die **von selbst wächst**, wenn sie voll ist. In Rusts Standard-
bibliothek ist das Nächstliegende `Vec<u8>` — ein wachsendes Array von Bytes. Du
`push`st Bytes hinein, und wenn der Platz ausgeht, fordert es automatisch mehr Speicher
an.

In dieser Serie nennen wir diesen schreibbaren Auffang-Puffer **`BytesMut`** (`Mut`
für *mutable* — veränderbar). Stell ihn dir ungefähr als `Vec<u8>` vor: ein Pointer
auf eine Region im Heap, plus zwei Zahlen — wie viele Bytes bereits geschrieben sind
(`len`) und wie viel die Region gerade fasst (`cap`, kurz für *capacity*).

```
BytesMut fängt einen Block-Header, 7 Bytes geschrieben, Region fasst 1024:

   Pointer ──────────► [ 68 65 61 64 65 72 21 · · · · · · · · ]
                       └─ 7 Bytes geschrieben ┘└─ freier Platz ─┘
   len = 7
   cap = 1024
```

Der springende Punkt bei `BytesMut`: er ist **schreibbar** und er **wächst**. Genau
das brauchst du, solange du noch eine Nachricht vom Socket liest.

## Aber sobald das Lesen fertig ist, steht `BytesMut` im Weg

Nehmen wir an, die Nachricht ist fertig gelesen. Was willst du jetzt damit tun?

- sie einem Decoder geben, um den Inhalt zu verstehen,
- sie in einen Cache legen, um sie später nachzuschlagen,
- sie an einen anderen Peer weiterschicken.

Und meist ist es **alles drei gleichzeitig**: derselbe Block-Header, den der Decoder
zum Parsen hält, den der Cache hält, den die Sende-Warteschlange hält.

Hier wird `BytesMut` lästig, aus zwei Gründen.

Erstens ist er **schreibbar**. Ist die Nachricht einmal fertig, *wollen* wir nicht,
dass jemand sie noch verändert. Wenn Decoder und Cache denselben `BytesMut` halten und
einer ihn versehentlich überschreibt, liest der andere kaputte Daten. Unveränderliche
Daten sind die Voraussetzung für sicheres Teilen: kann niemand schreiben, kollidieren
viele Leser nie.

Zweitens ist es **teuer**, viele Stellen denselben `BytesMut` halten zu lassen. Ein
`BytesMut` besitzt seinen Speicher; um ihn drei Stellen zu geben, ist der einzige
sichere Weg, ihn dreimal zu kopieren. Und Kopieren kostet — wie viel, sehen wir am
Ende dieses Teils.

Anders gesagt: `BytesMut` ist gut darin, eine Nachricht zu *bauen*, aber schlecht
darin, eine fertig gebaute zu *teilen*.

## `Bytes`: der nur-lesbare, teilbare Handle

Wir brauchen also einen zweiten Typ für die spätere Phase: wenn die Nachricht fertig
ist und nur noch gelesen und herumgereicht werden muss. Wir nennen ihn **`Bytes`**
(ohne `Mut` — nicht veränderbar).

`Bytes` ist ein *nur-lesbarer Handle* auf ein paar Bytes. Er hat die drei
Eigenschaften, die wir brauchen:

- **unveränderlich** — niemand kann durch `Bytes` schreiben, also ist Teilen sicher;
- **billig zu klonen** — eine weitere haltende Stelle hinzuzufügen ist fast gratis,
  ohne den Inhalt zu kopieren;
- **selbst-aufräumend** — wenn die letzte haltende Stelle loslässt, wird der Speicher
  automatisch freigegeben.

Und die Operation, die einen (fertigen) `BytesMut` in einen (teilbaren) `Bytes`
verwandelt, hat einen Namen: **`freeze`** — „einfrieren". Du hörst auf zu schreiben,
frierst den Puffer ein, und von da an ist er nur noch lesbar.

```
   BytesMut  ──freeze──►  Bytes
   (schreibbar,           (nur-lesbar,
    im Bau)                teilbar)
```

Das ganze Leben einer eingehenden Nachricht, aufgezeichnet, ist:

```
Socket ──► BytesMut ──► freeze ──► Bytes ──► Decoder / Cache / weiterreichen
          (auffangen &    (ein-      (herumgereicht)
           wachsen)        frieren)
```

Jetzt haben wir genug Vokabular für die zentrale Frage.

## Was `freeze` intern tut, und warum es sehr langsam sein kann

`freeze` klingt nach einer harmlosen Operation — nur ein „Umetikettieren" eines
Puffers von schreibbar auf nur-lesbar. Aber hier wird die gesamte Performance des
Empfangspfads entschieden, aus einem einfachen Grund: **jede eingehende Nachricht
läuft hindurch.** Eine ausgelastete Node `freeze`t hunderttausende, Millionen Male
pro Sekunde.

Die Frage ist: Muss `freeze`, wenn es läuft, den Pufferinhalt an eine neue Stelle
*kopieren*?

Die naivste Umsetzung **muss**. Sie allokiert eine neue Region, kopiert jedes Byte
aus dem `BytesMut` hinüber und gibt einen `Bytes` zurück, der auf die neue Region
zeigt. Rechnen wir aus, was diese Kopie kostet.

Nehmen wir einen Block-Header-Burst von etwa 1 MiB. Die Kopiergeschwindigkeit
(memcpy) auf moderner Hardware liegt bei etwa 5 GiB/s. Also dauert das Kopieren von
1 MiB:

```
1 MiB ÷ 5 GiB/s ≈ 200 Mikrosekunden
```

200 Mikrosekunden klingen wenig, aber das sind **200 Mikrosekunden, in denen die CPU
stillsteht** und nichts tut, außer Bytes von einer Stelle zur anderen zu schieben —
für *ein* `freeze`. Multipliziert mit den Nachrichten pro Sekunde verbrennt deine
Node einen erheblichen Anteil ihrer Zeit nur damit, Daten *neu zu kopieren*, die sie
gerade erst eingelesen hat.

Für ein System, in dem Durchsatz alles ist, ist eine Kopie pro `freeze` nicht
„unoptimiert" — sie schließt den Entwurf ganz aus dem Rennen aus. Also stellen wir
eine harte Anforderung, und diese ganze Serie ist die Geschichte, was es kostet, sie
einzuhalten:

> `freeze` muss in **konstanter** Zeit laufen, egal wie groß der Puffer. Der
> Speicher, der die Payload hält, **darf sich nicht bewegen**; das Einzige, das
> übergehen darf, ist der *Besitz* daran — von `BytesMut` an `Bytes`.

„Den Speicher nicht bewegen, nur den Besitz übergeben" ist die Idee, die wir im Rest
ausgraben. Aber bevor wir bauen, lohnt die Frage: Warum erreicht der naheliegendste
Ansatz das *nicht*? Denn die Antwort zeigt, wo das eigentliche Problem liegt.

## Versuch #1: `Bytes` als Hülle um `Vec<u8>`

Erste Idee: `freeze` gibt den Puffer einfach direkt als `Vec<u8>` zurück.

Das *vermeidet* zwar die Kopie — `Vec<u8>` und `BytesMut` sind beide ein flacher
Heap-Block, also kann ein `Vec` den Speicher des `BytesMut` „übernehmen", ohne neu zu
kopieren. In Sachen Kopie klappt es also. Aber es bricht an zwei anderen Stellen, und
genau die prägen alles Weitere.

Erstens ist `Vec<u8>` **schreibbar**. Wir sind zurück beim Problem von `BytesMut`:
keine Garantie, dass der Inhalt unveränderlich ist, also nicht sicher teilbar.

Zweitens ist `Vec<u8>` **nicht billig zu klonen**. `Vec::clone()` kopiert den ganzen
Inhalt. Und „viele Stellen halten es" passiert dauernd. Ist jedes „auch-halten" ein
memcpy, dann haben wir die Kopie bloß von `freeze` nach `clone` *verschoben* — nicht
getötet.

Lehre aus Versuch #1: Wir brauchen einen Typ, der zugleich **unveränderlich**,
**billig zu klonen** und **fähig, existierenden Speicher zu übernehmen** ist. Diese
drei Eigenschaften sind der Maßstab für jeden weiteren Kandidaten.

## Versuch #2: `Bytes` als Hülle um `Arc<[u8]>`

Das ist fast jedes Rust-Programmierers zweiter Reflex und zugleich der Punkt, an dem
viele Entwürfe in der echten Welt beginnen.

`Arc` (kurz für *Atomically Reference-Counted*) ist Rusts Standardwerkzeug für „viele
Stellen besitzen gemeinsam ein Datum". Innen hält es einen **counter**, wie viele
Stellen es gerade halten. Jedes `clone` erhöht den counter um eins; jedes Mal, wenn
eine Kopie fallengelassen wird, verringert er sich um eins; erreicht der counter 0,
gibt sich das Datum selbst frei. `Arc<[u8]>` heißt „ein Array von Bytes,
referenzgezählt".

```rust
struct Bytes(Arc<[u8]>);
```

Das gibt uns sofort **zwei von drei** nötigen Eigenschaften:

- **unveränderlich** — `Arc<[u8]>` verleiht nur eine Lese-Sicht, niemand kann
  schreiben;
- **billig zu klonen** — `clone` erhöht nur den counter um eins, ohne den Inhalt zu
  kopieren. Genau das „viele Stellen halten eine Nachricht", das wir brauchen.

Und es **funktioniert**. Der ursprüngliche Ansatz beginnt genau mit diesem Entwurf.
Das Problem taucht erst bei der dritten Eigenschaft auf — `freeze` darf nicht
kopieren — und um zu sehen, warum es bricht, muss man auf das *Speicherlayout*
schauen.

### Warum `Arc<[u8]>` bei `freeze` eine Kopie erzwingt

`Arc<[u8]>` ist ein **einziger** Speicherblock, in dem der counter direkt *vor* den
Bytes **verschmolzen** sitzt:

```
Arc<[u8]>:   [ counter | b0 b1 b2 ... bN ]
             └─ Header ─┘└──── Payload ────┘
                ein einziger Block
```

Der Puffer von `BytesMut` ist nur Payload, **ohne counter** davor:

```
BytesMut:    [ b0 b1 b2 ... bN ]
             └──── Payload ────┘
```

Diese beiden Layouts sind unterschiedlich geformt und passen nie zusammen. Um den
Puffer von `BytesMut` in ein `Arc<[u8]>` zu verwandeln, müsste man den counter
*unmittelbar vor* den aktuellen Pointer setzen. Aber der Speicher direkt davor
**gehört dir nicht** — der Allocator hat ihn nie ausgegeben, und dort zu schreiben
zertrampelt einen anderen Teil des Programms. Man kann keinen Header vor einen bereits
allokierten Block „nachrüsten".

Also ist `Arc::from(vec)` **gezwungen**:

1. den Allocator um einen *neuen* Block der Größe `counter + N` zu bitten;
2. die N Payload-Bytes vom alten Puffer in den neuen Block zu kopieren;
3. den alten Puffer freizugeben.

Schritt 2 ist das memcpy, das wir zu töten geschworen haben. Und das Wichtige: es ist
**kein Code-Bug**, sondern eine unausweichliche Folge der *Form* von `Arc<[u8]>`.
`Arc<[u8]>` kennt nur eine Art des Besitzens — Referenzzählung — und diese verlangt,
dass der counter *innerhalb* desselben Blocks wie die Payload lebt. Ein Typ, dessen
counter mit seiner Payload verschmolzen ist, **kann keine Payload übernehmen**, die
anderswo allokiert wurde.

In einem Satz, das Scharnier der ganzen Serie:

> `Arc<[u8]>` kann keine existierende Speicherregion *übernehmen*. Der einzige Weg,
> Bytes in ein `Arc<[u8]>` zu bekommen, ist, einen frischen Block zu allokieren und
> hineinzukopieren. Aber wir brauchen das Gegenteil: einen Handle, der direkt in den
> Speicher *eines anderen* zeigt (hier den Puffer von `BytesMut`) und die
> Verantwortung übernimmt, ihn aufzuräumen.

## Was zutage tritt, wenn ein Handle in „fremden Speicher" zeigt

Sobald wir die Idee „ein Handle zeigt in einen existierenden Puffer" akzeptieren,
taucht eine neue Frage auf — eine, die `Arc<[u8]>` nie beantworten musste.

`Arc<[u8]>` weiß immer genau, was zu tun ist, wenn eine Kopie fallengelassen wird:
counter verringern, bei 0 freigeben. Immer, ohne Ausnahme — weil es nur eine Art des
Besitzens hat. Aber ein frei zeigender Handle könnte in drei Arten von Speicher mit
drei gegensätzlichen Schicksalen zeigen:

- Er zeigt in eine **Konstante, die in der ausführbaren Datei einkompiliert ist**
  (etwa ein hartkodierter Byte-String im Programm). Diese Region wurde nie allokiert;
  beim Loslassen dürfen wir **nichts** tun — sie freizugeben hieße, Speicher
  freizugeben, den wir nicht besitzen.
- Er zeigt in einen **gerade von `BytesMut` übernommenen Puffer**. Diese Region
  *wurde* allokiert, und genau ein Handle besitzt sie; beim Loslassen müssen wir sie
  freigeben.
- Er zeigt in eine **über mehrere Stellen geteilte Region**. Nun brauchen wir einen
  counter; wer zuletzt loslässt, gibt frei.

Das ist der zentrale Widerspruch des ganzen Problems, zum ersten Mal ausgesprochen:

> Derselbe `Bytes`-Typ, aber drei verschiedene Arten der Aufräumung, und welche gilt,
> ist erst zur Laufzeit bekannt, pro einzelnem Wert.

`Arc<[u8]>` weicht diesem Widerspruch aus, indem es nur eine Aufräum-Disziplin
unterstützt (und dafür mit dem `freeze`-memcpy zahlt). Wir dürfen nicht ausweichen:
Wir brauchen alle drei *in einem Typ* — für ein kopierfreies `freeze` (die
„allein-besitzend"-Disziplin), ein billiges `clone` (die „geteilt"-Disziplin) und um
nichts für feste Byte-Konstanten zu zahlen (die „static"-Disziplin, für hartkodierte
Byte-Strings wie Genesis-Konstanten oder Precompile-Bytecode).

## Warum man nicht einfach drei getrennte Typen nehmen kann

An dieser Stelle denkst du vielleicht: „Dann mach drei getrennte Typen —
`StaticBytes`, `OwnedBytes`, `SharedBytes`, je einer pro Aufräumung, und lass den
Compiler es regeln."

Was den Besitz angeht, ist das eigentlich der *richtige* Ansatz — und in Teil 2 sehen
wir, dass Rust normalerweise *will*, dass man genau das tut. Er stirbt aus einem ganz
anderen Grund: der **API-Grenze**.

Schau auf eine typische Funktion, die Bytes konsumiert:

```rust
fn decode(data: Bytes) -> Header;
```

Dieses `decode` — und hunderte Funktionen wie es — muss Bytes *unabhängig von ihrer
Quelle* schlucken. Mit drei getrennten Typen:

- müsstest du `decode` dreimal schreiben (oder jede Byte-konsumierende Funktion
  generisch machen — eine Explosion);
- könnte ein `Vec<Bytes>` keine Mischung der drei halten;
- könnte ein Channel, der `Bytes` zwischen Threads sendet, keine Mischung senden;
- müsste eine Struktur mit einem `Bytes`-Feld einen der drei hart festlegen und
  verlöre alle Flexibilität.

Die gesamte Infrastruktur unter `Bytes` setzt implizit **einen** Typ voraus. Die
„ein Typ"-Anforderung stellen wir nicht zum Spaß — sie kommt vom Code, der `Bytes`
*benutzt*.

Und hier die Falle: ein Typ bedeutet eine Aufräum-Funktion (`Drop`), also ein *festes*
Verhalten. Und wir haben gerade bewiesen, dass wir *drei* Verhalten brauchen, pro Wert
zur Laufzeit gewählt. Der Widerspruch zwischen „ein Typ, von der API erzwungen" und
„drei Aufräumungen, von kopierfrei-freeze plus billig-clone plus gratis-Konstanten
erzwungen" — das ist das eigentliche Problem.

## Was also Teil 2 löst

Zusammengefasst haben wir drei Ansätze und drei Gründe, warum sie sterben:

- `Vec<u8>`: erreicht kopierfreies freeze, ist aber nicht unveränderlich und klont
  durch Kopieren.
- `Arc<[u8]>`: unveränderlich und billig zu klonen, aber freeze erzwingt ein memcpy,
  weil der counter mit der Payload verschmolzen ist.
- drei getrennte Typen: erreicht alle drei Aufräumungen, aber die API-Grenze verlangt
  einen Typ.

Auf eine Frage eingedampft, das muss Teil 2 beantworten:

> Wie kann ein einziger `Bytes`-Typ drei verschiedene Aufräum-Verhalten tragen und
> das richtige pro Wert zur Laufzeit wählen — während das Lesen der Bytes so billig
> bleibt wie bei `Arc<[u8]>`?

Der Satz „ein Typ, viele Verhalten, zur Laufzeit gewählt" sollte vertraut klingen: Es
ist genau das Problem, für das es *dynamic dispatch* gibt. Teil 2 zeigt, wie man es
richtig einsetzt, und eine Nebenfrage, die genauso wichtig ist — warum die
Dispatch-Tabelle (die vtable), die wir bauen, genau *zwei* Slots hat, nicht mehr und
nicht weniger.

---

*Weiter: [Teil 2 — Ein Typ, viele Verhalten](02_vtable.md) · [Inhalt](00_index.md)*

*English: [`../en/01_the_problem.md`](../en/01_the_problem.md)*
