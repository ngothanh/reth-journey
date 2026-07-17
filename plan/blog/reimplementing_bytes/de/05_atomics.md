# Teil 5 — `AtomicPtr`: sicher zurückschreiben

Teil 4 schloss auf einem Problem und drei Anforderungen. Das Problem: Beim Klonen einer
allein-besitzenden `Bytes` müssen wir *zurück* ins Original schreiben, um es auf geteilt
hochzustufen, sonst Double-Free. Die drei Anforderungen dafür: Es muss einen Weg zum
`data`-Feld des Originals geben; wir müssen durch eine nur-lesbare Referenz schreiben
können; und es muss sicher sein, wenn mehrere Threads es tun.

Dieser Teil löst alle drei. Und das Schöne: Alle drei, obwohl aus drei verschiedenen
Welten, werden durch eine einzige Typwahl für `data` gelöst — es in einen *atomaren*
Pointer (`AtomicPtr`) zu verwandeln. Das ist der abstrakteste Teil der Serie, also
sezieren wir jede Anforderung einzeln, und mit jeder erscheint das passende
Concurrency-Konzept als Antwort auf ein *konkretes* Problem, das wir wirklich lösen
müssen, nicht als Theorie im Leeren.

## Warum ein atomarer Pointer

Erinnere dich an Teil 4s drei Anforderungen, denn das Bemerkenswerte ist, dass sie aus
drei unzusammenhängenden Welten kommen. Die erste — „es muss einen Weg zu `b1`s
`data`-Feld geben" — dreht sich um *Parameterübergabe*: Bekommt die Funktion eine Kopie
oder eine Referenz. Die zweite — „wir müssen durch eine nur-lesbare Referenz schreiben
können" — dreht sich um die *Ausleihregeln* des Compilers. Die dritte — „es muss sicher
sein, wenn mehrere Threads es tun" — dreht sich um das *Speichermodell* der Hardware.
Drei Belange auf drei verschiedenen Ebenen, keiner weiß vom anderen.

Und doch werden alle drei durch eine einzige Änderung gelöst: den Typ von `data` von
einem normalen Pointer in einen *atomaren* Pointer (`AtomicPtr`) zu ändern.

Erinnere dich aus Teil 2, dass das `data`-Feld gerade den Typ **`*mut ()`** hat — ein
Raw Pointer, 8 Bytes, „Bedeutung aufgeschoben". Dieser Typ *kann* die drei obigen
Anforderungen nicht erfüllen: Ein `*mut ()`, in eine Funktion übergeben, wird *als
Kopie* übergeben (scheitert an der ersten); selbst mit einer Referenz darauf *verbietet*
Rust das Schreiben durch eine nur-lesbare Referenz (scheitert an der zweiten); und ein
`*mut ()` von mehreren Threads gleichzeitig zu lesen/schreiben ist ein Data Race, also
Undefined Behavior (scheitert an der dritten).

Alle drei Fehlschläge werden durch eine einzige Änderung behoben: den Typ von `data` von
`*mut ()` auf **`AtomicPtr<()>`** ändern — dieselben 8 Bytes, dieselbe Rolle „Bedeutung
aufgeschoben", aber nun ein *atomarer* Pointer. Er erlaubt, eine Referenz darauf zu
nehmen (löst die erste). Er erlaubt das Schreiben durch eine nur-lesbare Referenz, dank
einer Eigenschaft namens *interior mutability* (löst die zweite). Und er bietet atomare
Operationen, damit Threads sich nicht gegenseitig zertrampeln (löst die dritte).

Das ist als eigene Lektion mitzunehmen: Fragt jemand „warum ist dieses Feld atomar?",
ist die richtige Antwort *nicht* „wegen Concurrency" im Allgemeinen. Hier sind es drei
konkrete, getrennte Anforderungen, die zufällig durch dasselbe gelöst werden. Zu
erkennen, wann mehrere verschiedene Anforderungen auf einen einzigen Mechanismus
zusammenlaufen — das ist die Hälfte der Systems-Entwurfskunst.

Die nächsten Abschnitte sezieren jede Anforderung. Die erste (eine Referenz nehmen) ist
trivial — nur die Funktionssignatur ändern, um `&data` statt `data` zu übergeben. Die
anderen zwei bergen den Stoff, und jede führt uns zu einem Concurrency-Konzept.

## Anforderung zwei — durch eine nur-lesbare Referenz schreiben: interior mutability

Es gibt eine Regel in Rust, einfach genug, um sie sich einzuprägen: Eine nur-lesbare
Referenz (`&T`) darf *nur gelesen* werden. Um durch sie zu schreiben, muss `T` innen
etwas namens `UnsafeCell` enthalten.

`UnsafeCell` ist das *Einzige* in ganz Rust, das „Daten durch eine nur-lesbare Referenz
verändern" erlaubt. Es ist ein vom Compiler erlaubtes Loch, direkt durch die
Ausleihregeln gebohrt. Jedes andere Werkzeug, mit dem du je „durch eine geteilte
Referenz geschrieben" hast, ist `UnsafeCell` plus eine Disziplin, die es sicher macht:

- `Mutex` ist `UnsafeCell` plus „vor dem Eintritt sperren".
- `RefCell` ist `UnsafeCell` plus „Ausleihen zur Laufzeit zählen, bei Verstoß Panik".
- `Cell` ist `UnsafeCell` plus „nur hinein-/herauskopieren, das Innere nie verleihen".
- Und ein atomarer Pointer ist ein `UnsafeCell`, das einen Pointer hält, plus „nur mit
  den atomaren Instruktionen der CPU lesen/schreiben".

Wenn also `clone` nur eine nur-lesbare Referenz auf `b1` hat, aber in `b1.data`
schreiben muss, *muss* dieses Feld ein `UnsafeCell` enthalten. Ein atomarer Pointer ist
genau das, was wir brauchen: Er öffnet das Schreiben-durch-nur-lesbare-Referenz-Loch
(zweite Anforderung) und regelt zugleich den Multithread-Teil (dritte).

Es gibt hier eine einprägsame Symmetrie. `Arc<T>` gibt dir ebenfalls nur eine
nur-lesbare Referenz auf sein Inneres. Das ist der *Grund*, warum alle `Arc<Mutex<T>>`
schreiben — `Arc` regelt das *Teilen*, `Mutex` das *Schreiben*. `Bytes` steht vor genau
diesem Problem, nur anders gelöst: Beide sind „geteilt und muss schreiben", aber
`Arc<Mutex<Vec>>` nimmt eine Sperre (weil `Vec` groß ist, nicht atomisierbar), während
`Bytes` einen atomaren Pointer nimmt (weil das zu Schreibende genau 8 Bytes ist).

Und hier der innehaltenswerte Punkt: Warum darf `Bytes` eine atomare Operation statt
einer Sperre nehmen? Weil das zu Schützende *genau ein Maschinenwort* ist (8 Bytes auf
einer 64-Bit-Maschine). Das ist eine Hardware-Tatsache, einmal nachschlagen und merken,
kein Herleiten nötig: Eine 64-Bit-CPU hat atomare Lese-, Schreib- und
„compare-and-swap"-Instruktionen für genau 8 Bytes oder weniger. Alles Größere als 8
Bytes kann *keine Instruktion* atomar tun — dann braucht man eine Sperre (eine Sperre
liest/schreibt mehrere Maschinenwörter nacheinander unter dem Schutz eines Flags). Da
`data` nur 8 Bytes ist, ist es selbst „zugleich die Sperre und die Daten", keine
separate `Mutex` daneben. Genau das hält `Bytes` klein, sperrfrei und dennoch
thread-sicher. Wäre das Hochzustufende größer als 8 Bytes, bräche dieser ganze Entwurf,
und man wäre zurück bei einer Sperre.

## Anforderung drei — mehrere Threads stufen gleichzeitig hoch: Send, Sync und CAS

Rust hat zwei Begriffe für Daten, die eine Thread-Grenze überqueren. Ein Wert ist
„sendbar" (`Send`), wenn er auf einen anderen Thread *verschoben* werden darf. Ein Typ
ist „teilbar" (`Sync`), wenn eine Referenz auf ihn von mehreren Threads *benutzt* werden
darf. Der Compiler leitet beides ab; und weil `Bytes` Raw Pointer hält (die Rust
standardmäßig als nicht-sendbar, nicht-teilbar behandelt, denn es ist pessimistisch bei
Pointern), ist `Bytes` standardmäßig *keines* von beiden.

Aber das Codebase *braucht* sie. Caches werden über mehrere Worker geteilt, Nachrichten
über Channels zwischen Threads gesendet. Ist `Bytes` nicht sendbar und teilbar,
kompiliert der Code nicht einmal — das ist ein echter Fehler:

```
error[E0277]: `*mut ()` cannot be sent between threads safely
```

Also *versprechen* wir dem Compiler, dass `Bytes` sicher zu senden und zu teilen ist.
Das Versprechen ist wahr, weil: Die Payload ist unveränderlich (viele Leser kollidieren
nicht), und der einzige veränderliche Zustand — `data` — ist atomar. Genau das macht
`Arc<[u8]>` sendbar und teilbar, und `Bytes` hat dieselbe Form.

Aber das Versprechen hat seinen Preis. Sobald `Bytes` teilbar ist, können zwei Threads
beide eine Referenz auf `b1` (eine allein-besitzende Region) halten und beide `clone`
rufen. Stufen wir naiv hoch — `data` auslesen, dann den neuen Wert hineinschreiben, als
zwei getrennte Schritte —, passiert dieses Szenario:

```
Thread 1: liest data, sieht "nicht hochgestuft"
Thread 2: liest data, sieht "nicht hochgestuft"     ← dazwischengedrängt
Thread 1: allokiert counter A, schreibt data = A
Thread 2: allokiert counter B, schreibt data = B     ← ÜBERSCHREIBT A
```

Ergebnis: Zwei counter entstehen, einer (A) wird verworfen — ein Leak, oder bei
schiefer Zähl-Logik ein Use-after-Free. Das ist ein klassischer Bug namens *Lost Update*
— Verlust durch Überschreiben — der nicht-atomaren „lesen-dann-schreiben"-Art. Die zwei
getrennten Schritte lassen eine Lücke, in die der andere Thread schlüpft.

Die Abhilfe ist eine Operation, die „prüfen" und „schreiben" in einen untrennbaren
Schritt verschmilzt, genannt *compare-and-swap*, kurz CAS. In Menschensprache:

> „Hey `data`, *wenn* du noch der alte Wert bist (nicht hochgestuft), dann ändere dich
> in den Pointer meines counters — und tu diese zwei Dinge *verschmolzen*, kein Thread
> kann dazwischen. Aber wenn ein anderer dich schon geändert hat, dann ändere dich
> *nicht*, und sag mir, was du jetzt hältst."

Die Hardware garantiert, dass beim Ansturm mehrerer Threads *genau ein* CAS gewinnt. Der
Gewinner installiert seinen counter in `data`; in diesem Augenblick wird `b1` geteilt
(weil `b1` und die `data`-Zelle eins sind). Der Verlierer bekommt das Signal „ein
anderer hat schon geändert", verwirft den counter, den er allokiert hat, und nutzt den
counter des Gewinners. Am Ende gibt es einen counter, und der Speicher wird genau einmal
freigegeben — die Zähl-Denkweise aus Teil 3 geht wieder auf.

Ein zu beachtendes Detail im Weg des Verlierers: Beim Verwerfen des überzähligen
counters muss er es tun, *ohne* die Payload-Freigabe auszulösen — denn die Payload
gehört jetzt dem counter des Gewinners. Er gibt nur die *Hülle* des Extra-Blocks frei
und überspringt dessen Payload-Aufräumung. Vergisst man dieses Detail, wird die Payload
zweimal freigegeben.

Es gibt einen schönen Namen für CASs Rolle hier: Es ist der *Linearisierungspunkt*.
Obwohl die zwei Threads parallel anstürmen, ist der CAS die Marke, die dieses Chaos in
eine klare Reihenfolge verwandelt — „wer den CAS gewinnt, gilt als zuerst geschehen".
Wann immer du „genau einer von mehreren Wettläufern darf X tun" brauchst, ist CAS das
Werkzeug, und die Gewinner-Marke ist der Linearisierungspunkt.

## Anforderung drei, Fortsetzung — Memory Ordering: „schreiben können" reicht nicht, man muss „die richtige Reihenfolge sehen"

CAS löst nur die Hälfte des Multithreadings: Es garantiert, dass genau ein Thread den
counter *installiert*. Aber es gibt eine zweite Gefahr, subtiler und ganz getrennt — und
hier finden es die meisten am schwersten. Wir stellen erst das Problem, dann sezieren wir
jede einzelne Operation, um zu sehen, welche Synchronisations-„Stärke" jede braucht.

### Das Problem: Schreibvorgänge werden umsortiert

Der Gewinner des Promotion-Wettlaufs tut zwei Dinge, in dieser Reihenfolge *im Code*:
Zuerst initialisiert er den Inhalt des `Shared`-Blocks (schreibt hinein die
Originaladresse des Speichers und die Länge), dann *veröffentlicht* er die Adresse des
`Shared`-Blocks über den CAS, der `data` schreibt.

Das Problem ist, dass Hardware und Compiler gleichermaßen Schreibvorgänge in den Speicher
*umsortieren* dürfen, um schneller zu sein — sie puffern, verschmelzen, ordnen um. Für
einen einzelnen Thread ist das harmlos, weil das Endergebnis richtig aussieht. Aber bei
mehreren Threads kann ein anderer Thread die Schreibvorgänge des Gewinners *in anderer
Reihenfolge* sehen als der Code.

Die konkrete Katastrophe: Ein zweiter Thread ruft `clone`, liest `data`, sieht, dass es
schon die Adresse des `Shared`-Blocks ist, und greift auf diesen Block zu, um den counter
zu erhöhen — also liest er dessen Inhalt. Sieht der zweite Thread *die Adresse*, aber
*noch nicht* den *Inhalt*, den der Gewinner eben initialisiert hat — unter den
Umsortierregeln völlig zulässig —, dann liest er einen `Shared`-Block voller Müll, und
alles danach ist Undefined Behavior. Die Adresse ist dem Inhalt, auf den sie zeigt,
„vorausgeeilt".

Wir brauchen eine Garantie: *Wer die Adresse des `Shared`-Blocks gesehen hat, muss auch
dessen fertigen, initialisierten Inhalt sehen.* Das ist die Aufgabe der *Memory
Orderings* — der „Etiketten", die wir jeder atomaren Operation anheften, um zu sagen,
wie weit sie umsortiert werden darf.

### Vier Stärken, und wie man sie sich vorstellt

Rust hat vier Etiketten, die wir nutzen: `Relaxed`, `Acquire`, `Release` und `AcqRel`.
Die zwei mittleren stellt man sich am leichtesten als „veröffentlichen" und „abonnieren"
vor:

- Ein **Schreiben** der **`Release`**-Art ist eine *Veröffentlichung*: Alles, was ich
  *vor* dieser Operation geschrieben habe, sieht jeder, der den Wert liest, den ich eben
  schrieb.
- Ein **Lesen** der **`Acquire`**-Art ist ein *Abonnement*: Sobald ich den
  veröffentlichten Wert lese, sehe ich auch alles, was der Veröffentlicher *vor* dem
  Veröffentlichen geschrieben hat.
- **`Relaxed`** heißt „mach diese Operation nur atomar, versprich nichts über die
  Reihenfolge relativ zu anderen Schreibvorgängen" — das billigste.
- **`AcqRel`** heißt „sowohl `Acquire` als auch `Release`", für eine Operation, die
  *zugleich liest und schreibt* (wie CAS, das den alten Wert liest und den neuen
  schreibt).

Der Kern: `Release` und `Acquire` wirken nur, wenn sie *als Paar* kommen, auf *derselben
Variable*. Eine Seite veröffentlicht, die andere abonniert; das Paar baut die
Reihenfolge-Verbindung, die zwei Threads verknüpft. Fehlt eine Seite, bricht das Paar,
und die Garantie verschwindet.

### Jede Operation auf `data` sezieren

Nun angewandt auf genau die Stellen, an denen der Code während der Promotion `data`
berührt, und bei jeder gefragt: Welches Etikett braucht sie, und warum.

**Das erste Lesen von `data`, das `clone` eröffnet.** Bevor wir entscheiden, ob eine
Promotion nötig ist, lesen wir `data`, um zu sehen, was es gerade ist. Etikett:
**`Acquire`**. Warum? Weil `data` schon von einem anderen Thread hochgestuft sein kann —
es kann schon die Adresse eines `Shared`-Blocks sein —, in welchem Fall wir direkt in den
„schon geteilt"-Zweig gehen und auf diesen `Shared`-Block *zugreifen*, um den counter zu
erhöhen. Um sicher zuzugreifen, müssen wir seinen initialisierten Inhalt sehen — also muss
dieses Lesen `Acquire` sein, um mit dem `Release` dessen zu paaren, der es hochgestuft
hat.

**Der CAS — das ist die Antwort auf „warum AcqRel".** Etwas, das viele übersehen:
`compare_exchange` trägt *zwei* Ordering-Etiketten, nicht eines — eines für den *Erfolgs*-
Fall, eines für den *Fehlschlag*-Fall. Denn ein CAS hat zwei völlig verschiedene
Ausgänge, und jeder braucht eine andere Garantie.

- *Wenn der CAS fehlschlägt* (ein anderer hat zuerst hochgestuft): Der CAS gibt uns den
  aktuellen Wert von `data` zurück — die Adresse des `Shared`-Blocks des Gewinners. Und
  gleich danach werden wir diese Adresse *benutzen* (den counter des anderen `Shared`-
  Blocks erhöhen). Das heißt, wir werden gleich auf einen `Shared`-Block zugreifen, den
  ein anderer Thread initialisiert hat — also muss der Fehlschlag-Fall **`Acquire`** sein,
  aus genau demselben Grund wie das eröffnende Lesen.
- *Wenn der CAS gelingt* (wir sind der Gewinner): Wir haben eben den `Shared`-Block
  *veröffentlicht*, den *wir selbst* oben gebaut haben. Damit ein anderer Thread später
  diese Adresse liest und auf den `Shared`-Block zugreift, ohne auf Müll zu treffen, muss
  dieses Schreiben **`Release`** sein.

Also braucht der Fehlschlag-Fall `Acquire`, der Erfolgs-Fall `Release`. Und hier der
Schlüssel: Rust verlangt, dass das Etikett des *Erfolgs*-Falls nicht schwächer ist als
das des *Fehlschlag*-Falls. Aber `Release` allein schließt `Acquire` *nicht* ein (sie
sind zwei verschiedene Richtungen — eine regelt die Schreibseite, eine die Leseseite).
Damit der Erfolgs-Fall also `Release` trägt (für seine eigene Veröffentlichung) *und*
stark genug gegenüber dem `Acquire` des Fehlschlag-Falls ist, muss er *beide* tragen —
und dieses „sowohl `Acquire` als auch `Release`"-Etikett ist genau **`AcqRel`**.

Kurz gesagt: `AcqRel` für den CAS ist nicht „auf Nummer sicher" gewählt — es ist das
einzige Etikett, das zwei Dinge zugleich auf derselben Instruktion erfüllt: Der Verlierer
muss den `Shared`-Block des Gewinners *empfangen* (`Acquire`), und der Gewinner muss
seinen `Shared`-Block *veröffentlichen* (`Release`).

**Das Lesen von `data` in `drop` — gar keine Atomics nötig.** Weil `drop` eine
*exklusive* Referenz auf den Wert nimmt (erinnere dich an Teil 4), weiß es sicher, dass
kein anderer Thread ihn noch hält — kein Wettlauf, also genügt ein normales Lesen. Die
exklusive Referenz ist selbst der Beweis für keinen Data Race. Genau darum nimmt `drop`s
Signatur eine exklusive Referenz, `clone`s eine geteilte: nicht Stil, sondern „drop hat
Exklusivität, clone nicht".

**Der Randfall: eine konstante `Bytes`.** `data` ist leer, synchronisiert nichts mit
niemandem — also braucht jede Berührung nur **`Relaxed`**, das billigste Etikett.

### Warum nicht einfach `SeqCst`, um sicherzugehen

`SeqCst` (sequenzielle Konsistenz) ist das stärkste Etikett, das Rust hat — es zwingt
*jede* `SeqCst`-Operation im ganzen Programm in eine einzige globale Reihenfolge, auf die
sich alle Threads einigen. Klingt sicher, und viele greifen „zur Sicherheit" danach. Aber
hier ist es sowohl *übertrieben* als auch *teuer*. Übertrieben, weil wir nur eine
*paarweise* Verbindung zwischen einem Veröffentlicher und einem Empfänger auf einer
einzigen Variable, `data`, brauchen — keine programmweite Einigung auf eine globale
Reihenfolge. Teuer, weil `SeqCst` meist stärkere Memory Fences einfügen muss, was genau
den Pfad verlangsamt, den der ganze Entwurf schnell halten soll. Die minimal nötige
Stärke zu wählen — `Acquire`/`Release`/`AcqRel` wo es zählt, `Relaxed` wo nicht — ist
Teil davon, sperrfreien Code ordentlich zu schreiben.

Das mitzunehmende Prinzip: `Release` und `Acquire` kommen immer als Paar auf derselben
Variable, verknüpfen eine „Veröffentlichung" mit einem „Abonnement"; eine Operation, die
*zugleich liest und schreibt* und wo beide Rollen zählen (wie CAS), braucht `AcqRel`; und
wann immer du einem anderen Thread einen Pointer auf frisch initialisierte Daten
veröffentlichst, brauchst du *immer* dieses Paar — sonst können die Daten „nach" dem
Pointer ankommen.

## Eine knappe Folge: warum das Etikett in „promotable" umbenannt wird

Das letzte Detail, eine direkte Folge davon, dass „Zurückschreiben" auf eine Grenze
stößt.

Beim Hochstufen kann `clone` `b1.data` schreiben (dank des atomaren Pointers), aber
*nicht* `b1.vtable` — weil `vtable` ein normales Feld ist, *ohne* interior mutability, und
`clone` nur eine nur-lesbare Referenz hat. Also nach dem Hochstufen:

```
b1.vtable = weiter die "allein-besitzend"-Tabelle   ← festgeklemmt, unänderbar, lügt jetzt
b1.data   = jetzt die Adresse des Shared-Blocks      ← geändert (per CAS), sagt die Wahrheit
```

`b1` dispatcht für immer über die alte Tabelle, obwohl es tatsächlich geteilt ist. Also
muss diese Tabelle bei ihrer *allerersten Instruktion jedes Aufrufs* (sowohl `clone` als
auch `drop`) sich fragen: Ist mein `data` jetzt eine Längenzahl (nicht hochgestuft) oder
eine `Shared`-Block-Adresse (hochgestuft)? — und entsprechend verzweigen. Genau darum ist
das Etikett *nicht* „allein-besitzend", sondern „promotable" — „dieses *könnte schon*
geteilt geworden sein".

Etwas tiefer betrachtet, ist das ein allgemeines Gesetz: Die unterscheidende Marke *muss
in der veränderlichen Zelle leben* (`data`, atomar), *nicht im unveränderlichen
vtable-Pointer*. Das ist eine unausweichliche Folge davon, dass sich der Besitz mitten in
der Lebenszeit ändert: Der vtable-Pointer erstarrt im Moment der Geburt des Werts, also
kann er nicht der Ort dynamischen Zustands sein. Wann immer sich der Zustand eines Werts
*nach* seiner Geburt ändert, muss die Marke dieses Zustands im *veränderlichen* Teil
sitzen, nicht im *unveränderlichen* — und hier ist nur `data` veränderlich.

(Eine Randnotiz, *nicht* Teil des Modells: Wie hält `data` sowohl eine Längenzahl als
auch eine `Shared`-Block-Adresse in denselben 8 Bytes, und wie unterscheidet die
„promotable"-Tabelle die zwei? Es gibt einen Trick, der das *niedrigste Bit* borgt: Die
Adresse eines `Shared`-Blocks ist immer gerade — wegen der Ausrichtungsregeln des
Speichers —, also ist ihr niedrigstes Bit immer 0; wir setzen das niedrige Bit auf 1,
wenn wir eine Zahl hineinpacken, und ein Blick auf das niedrige Bit sagt, welche Art
gespeichert ist. Das ist rein eine *Speicheroptimierung* — es durch ein separates Feld
für die Länge zu ersetzen, ist ebenso korrekt, nur ein Maschinenwort größer. Die
`bytes`-Crate packt das Bit, weil sie jedes Byte zählt; eine zum-Lernen-gebaute Version
muss das nicht.)

## Zusammenfassung: der vollständige Entwurf

Hier der ganze Entwurf, Seite an Seite, ein letztes Mal. Verglichen mit der
Zusammenfassung am Ende von Teil 2 hat sich genau *eine* Zeile geändert — der Typ von
`data` —, aber ganz Teil 4 und 5 dienten dazu, diese eine Zeile zu erklären.

```rust
struct Bytes {
    ptr:    NonNull<u8>,      // "welche Bytes": zeigt auf den Anfang des Laufs
    len:    usize,            // "welche Bytes": wie lang
    data:   AtomicPtr<()>,    // "wer besitzt": 8 atomare Bytes; *mut () in T2, jetzt atomar
    vtable: &'static Vtable,  // "wer besitzt": welches clone/drop-Set benutzen
}

struct Vtable {
    clone: unsafe fn(&AtomicPtr<()>, ptr, len) -> Bytes,  // & damit clone das Original reparieren kann
    drop:  unsafe fn(&mut AtomicPtr<()>, ptr, len),       // &mut: exklusiv, nicht-atomar lesen
}
```

Die drei Arten des Besitzens, und was `data` in jeder hält:

| vtable              | was `data` hält                     | was `clone` tut            | was `drop` tut |
|---------------------|-------------------------------------|----------------------------|----------------|
| `STATIC_VTABLE`     | null (ungenutzt)                    | Struktur kopieren          | nichts         |
| `PROMOTABLE_VTABLE` | Länge (Zahl), *oder* eine counter-Adresse nach dem Hochstufen | noch nicht hochgestuft: counter bauen, per CAS auf geteilt; schon: counter erhöhen | noch nicht: freigeben; schon: verringern |
| `SHARED_VTABLE`     | die Adresse des counters (echter Ptr)| counter erhöhen           | verringern     |

Die Daten zu lesen (`deref`, `len`, vergleichen, hashen) berührt nur `ptr` + `len` — nie
`data` oder `vtable` — also ist es so billig wie `Arc<[u8]>`. `data` + `vtable` kommen nur
bei `clone` oder `drop` ins Spiel. Das ist der ganze Entwurf.

Der Weg vom Anfang bis hierher, in einem Bild:

```
Arc<[u8]>            T1: counter mit Payload verschmolzen ⇒ freeze MUSS kopieren
   │
   ▼ O(1)-freeze nötig ⇒ Bytes muss den Speicher ÜBERNEHMEN, nicht kopieren
Bytes{ ptr, len, data, vtable }
   │  T2: "Besitz" vom Typ herab in eine vtable (ein Typ, drei Verhalten)
   │      data: *mut ()  — 8 Bytes, "Bedeutung aufgeschoben"
   │  T3: "welche Bytes" (ptr,len) von "wer besitzt" (data,vtable) trennen ⇒ gratis Lesen
   │      + Denkweise: wie oft wird jede Region freigegeben? (0/1/1)
   │
   ▼ T4: eine allein-besitzende Region klonen = Double-Free
   │      ⇒ Promotion: ZURÜCK ins Original schreiben, um es auf geteilt hochzustufen
   │
   ▼ T5: Zurückschreiben braucht &data + Schreiben-durch-& + Thread-Sicherheit
          ⇒ data: *mut ()  ➜  AtomicPtr<()>
          interior mutability · CAS · Acquire/Release/AcqRel
```

## Fünf Fragen für jedes spätere Problem

Klapp die Serie zu und vergiss vtables, Atomics, CAS. Was mitzunehmen ist — in spätere
Probleme über Write-Ahead-Logs, das Teilen von Knoten in einer Skiplist, das Cachen von
Datenblöcken und in jeden Entwurf, der unsafe, Besitz oder Optimierung berührt — sind
diese fünf Fragen. Sie sind, worauf die ganze Geschichte hinausläuft.

Erstens: *Wie oft genau wird jede Region freigegeben?* 0 ist ein Leak, 2 ist ein
Double-Free, 1 ist richtig. Jeder Bug reduziert sich auf diese Zahl.

Zweitens: *Was unterscheidet sich zwischen den Fällen?* Nur das braucht Dispatch. Was
gleich ist, lass in Ruhe — genau darum ist der Hot-Path gratis.

Drittens: *Berührt der Hot-Path das Dispatchen?* Wenn ja, ist der Entwurf falsch. Das
Lesen muss so billig sein wie `Arc<[u8]>`.

Viertens: *Gibt es ein Schreiben durch eine nur-lesbare Referenz?* Wenn ja, brauchst du
interior mutability. Eine nackte nur-lesbare Referenz ist immer nur-lesbar.

Fünftens: *Gibt es mehrere Threads?* Wenn ja, nimm atomare Operationen statt normalem
Lesen/Schreiben; eine „genau ein Gewinner"-Operation ruft nach CAS; und wann immer du
einem anderen Thread einen Pointer auf Daten veröffentlichst, nimm ein Release/Acquire-
Paar auf derselben Variable.

Und drei Schlusssätze zur Denkweise, über die Serie verstreut. `Drop` räumt nicht die
Struktur auf — die Struktur löst sich von selbst auf; `Drop` macht nur eine Allokation
rückgängig, also keine Allokation, kein `Drop`. Eine vtable ist ein Typ, von der
Kompilierzeit-Ebene zu einem Laufzeitwert degradiert, benutzt, wenn ein Typ mehrere
Verhalten braucht, pro Wert gewählt. Und der gefährliche Bug in unsafe ist nicht der, der
abstürzt, sondern der, der korrekt läuft — die Intuition aus sicherem Rust ist umgekehrt,
der Standard eines Fehlers ist Stille, also nimm immer `miri` mit, um ihn zum Sprechen zu
bringen.

Jetzt hast du genug vom Modell im Kopf, um dich hinzusetzen und `Bytes` von null neu zu
implementieren — die vier Verhalten für drei Besitz-Arten, die `freeze`-Operation und
Promotion — und für jede Wahl zu argumentieren. Die restlichen Code-Details zeigen sich
beim Schreiben, weil du den Grund für die Existenz jedes einzelnen verstehst.

---

*Zurück: [Teil 4](04_promotion.md) · [Inhalt](00_index.md)*

*English: [`../en/05_atomics.md`](../en/05_atomics.md)*
