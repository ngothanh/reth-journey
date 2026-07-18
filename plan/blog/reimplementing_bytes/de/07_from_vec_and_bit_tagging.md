# Teil 7 — `from_vec` und der Bit-Packing-Trick: eine 8-Byte-Zelle, zwei Bedeutungen

Teil 6 schrieb `share_*` fertig, aber es gibt noch keine Möglichkeit, eine `Bytes` zu
*erzeugen*, die diesen Weg einschlägt. Der Eingang ist `from_vec` — es nimmt ein
`Vec<u8>` entgegen, *ohne* zu kopieren. Und genau beim Schreiben von `from_vec` stoßen
wir auf das, was Teil 5 in einer Randnotiz aufschob: Das `data` einer `promotable`-`Bytes`
muss *zwei verschiedene Arten* von Pointern halten können — einen buffer-Pointer (noch
nicht hochgestuft) *oder* einen `Shared`-Pointer (hochgestuft) — in denselben 8 Bytes,
und jede spätere Funktion muss unterscheiden können, welche Art gerade gehalten wird.

Dieser Artikel seziert genau diesen Trick bis auf den Grund. Es ist die „kleinteiligste"
Stelle des ganzen Entwurfs, also gehen wir sehr langsam vor, und am Ende geben wir einen
Satz zum Merken, der alles zusammenschnürt.

## `from_vec`: normalisieren und dann aufschieben

```rust
pub fn from_vec(bytes: Vec<u8>) -> Self {
    if bytes.is_empty() {
        return Self::from_static(&[]); // leer → direkt static-repr, keine Allokation
    }
    let boxed: Box<[u8]> = bytes.into_boxed_slice(); // normalisieren: cap == len
    let len = boxed.len();
    let buf = Box::into_raw(boxed) as *mut u8;       // Besitz ÜBERNEHMEN — jetzt sind wir fürs free zuständig
    // ... Bit packen und dann Bytes bauen (unten) ...
}
```

Drei Dinge, jedes mit einem Grund:

**`is_empty` → `from_static(&[])`.** `into_boxed_slice` eines leeren `Vec` liefert einen
*dangling* Pointer, den wir weder bit-packen noch freigeben wollen. Leeres direkt in die
`static`-repr zu schicken (ein ewig-lebender leerer buffer) ist am saubersten — keine
Allokation für 0 Bytes.

**`into_boxed_slice()` — normalisieren auf `cap == len`.** Das ist das Schlüsseldetail,
auf das wir uns später *verlassen, um `cap` gar nicht speichern zu müssen*. Ein `Vec`
kann `cap > len` haben (überschüssiger Platz); `into_boxed_slice` schrumpft auf
`cap == len`. Der Preis: Wenn der `Vec` überschüssigen Platz hat, *reallokiert und
memcpy-t* diese Operation. Ja, und das echte `bytes` macht genau dasselbe — also merke
dir, dass dieses realloc bei einem `Vec` mit übriger capacity passieren kann.

**`Box::into_raw` — Besitz übernehmen.** Vor dieser Zeile würde die `Box` den buffer
selbst freigeben, sobald sie den scope verlässt. Nach `into_raw` verschwindet die `Box`
und *nichts* gibt mehr automatisch frei — **du** hast das free unterschrieben (später
über `free_boxed_slice`/`release_shared`). `buf` ist jetzt die Heap-Adresse des ersten
Bytes. Lässt man `buf` hier fallen, ist es ein Leak.

## Das Problem: eine Zelle, zwei Bedeutungen

Eine `promotable`-`Bytes` braucht ein `data` (wir nennen dieses Feld `ctx`), das
Folgendes hält:

- **vor** dem Hochstufen: einen Pointer auf den rohen **buffer**,
- **nach** dem Hochstufen: einen Pointer auf den **`Shared`**-Block.

Und Teil 5 hat schon geschlossen, warum diese Unterscheidungsmarke *in* `ctx` liegen
*muss*: Die Promotion ändert den Zustand *mitten in der Lebenszeit* über ein
Ein-Wort-CAS auf `ctx` — aber die `vtable` erstarrt bei der Geburt, sie lässt sich nicht
im selben Zug mit-CAS-en. Also brauchen wir eine Möglichkeit, *nur aus `ctx`*, zu
wissen, welche Art es gerade ist.

Die Methode: das **niedrigste Bit** des Pointers als KIND-Flag borgen.

```rust
const KIND_ARC: usize = 0b0; // niedriges Bit = 0 → ctx ist *mut Shared
const KIND_VEC: usize = 0b1; // niedriges Bit = 1 → ctx ist ein buffer-Pointer
const KIND_MASK: usize = 0b1;
```

Warum ist das niedrige Bit *freier Platz zum Borgen*? Wegen der **Ausrichtung
(alignment)**. Ein Wert vom Typ `T` mit alignment `A` liegt immer an einer durch `A`
teilbaren Adresse — ein Vielfaches von 8 endet in Binärdarstellung immer auf `000`. Der
`Shared`-Block enthält einen Pointer + `usize` + `AtomicUsize`, hat also alignment ≥ 8 →
seine Adresse endet **immer auf Bit 0**. Also ist `Shared` *von Natur aus* `KIND_ARC`,
ohne dass man etwas tun muss.

## Der Satz zum Merken: **VEC immer UNGERADE, ARC immer GERADE**

Alles folgt aus genau dieser einen Zeile. Wenn irgendeine Funktion **auf `ctx` schaut**,
um den Zustand zu dekodieren:

- **`ctx` ungerade (Bit = 1) → VEC** (noch buffer, nicht hochgestuft),
- **`ctx` gerade (Bit = 0) → ARC** (schon ein `Shared`).

- *ARC immer gerade*: `Shared` hat alignment 8 → von Natur aus Bit 0. Gratis.
- *VEC muss ungerade sein*: um **nicht mit ARC zu kollidieren**. Würde ein gerader
  buffer-Pointer direkt in `ctx` gelegt, sähe eine spätere `clone`/`drop`-Funktion Bit 0
  → hielte es für „schon hochgestuft, das ist ein `Shared`" → zwänge den buffer zu einem
  `*mut Shared` und läse dann `ref_count`... also läse sie ein paar deiner Datenbytes,
  die sie für den Zähler hält → Chaos. Also *erzwingen* wir, dass der VEC-Zustand immer
  ungerade herausliest.

## Die Komplikation: ein `u8`-buffer kann gerade *oder* ungerade sein

Das ist die Stelle, die diesen Artikel von einem Lehrbuch-tagged-pointer unterscheidet.
Der buffer ist `u8`, **alignment = 1**, also garantiert seine Adresse **nicht** Bit 0 =
0 — sie kann gerade oder ungerade sein. Aber wir *wollen*, dass sie immer ungerade
herausliest (KIND_VEC). Also:

- **buffer gerade** (Bit 0): Wir müssen das Bit *setzen* (`buf | 1`), um VEC zu
  markieren. Um die echte Adresse zurückzubekommen, müssen wir dieses Bit *löschen*
  (`& !1`). → benutze **`PROMOTABLE_EVEN_VTABLE`**.
- **buffer ungerade** (Bit 1 schon da): Es liest bereits als VEC heraus, KEIN Setzen
  nötig. Aber dieses Bit 1 *ist ein echter Teil der Adresse*, also darf man es beim
  Zurückholen absolut NICHT löschen. Unverändert speichern. → benutze
  **`PROMOTABLE_ODD_VTABLE`**.

Der Rest des `from_vec`-Codes ist genau diese Verzweigung:

```rust
    if buf as usize & KIND_MASK == 0 {
        // EVEN: Bit setzen als VEC-Markierung; später per MASK das Bit entfernen zum recover.
        let ctx = (buf as usize | KIND_VEC) as *mut ();
        Bytes { ptr: NonNull::new_unchecked(buf), len,
                ctx: AtomicPtr::new(ctx), vtable: &PROMOTABLE_EVEN_VTABLE }
    } else {
        // ODD: niedriges Bit ist schon 1 == VEC; Pointer WÖRTLICH speichern, später NICHT masken.
        Bytes { ptr: NonNull::new_unchecked(buf), len,
                ctx: AtomicPtr::new(buf as *mut ()), vtable: &PROMOTABLE_ODD_VTABLE }
    }
```

Beachte, dass `ptr` den **sauberen Pointer** speichert (`buf` ohne Bit-Packing); nur
`ctx` trägt den tag. Dadurch sieht `deref` (Lesen über `ptr`) das Bit nie — der Lesepfad
benutzt immer die echte Adresse.

## Warum *zwei* vtables, nicht eine?

Das ist die beste Frage, und die Antwort berührt eine Wahrheit über Information. Stell
dir vor, du hättest nur eine vtable und nur `ctx`:

```
Fall 1 (buffer gerade 0x1000): Bit setzen → ctx = 0x1001 → recover muss Bit löschen → 0x1000
Fall 2 (buffer ungerade 0x1001): unverändert → ctx = 0x1001 → recover muss halten   → 0x1001
```

**Beide Fälle haben genau dasselbe `ctx` (0x1001), aber die echte buffer-Adresse ist
verschieden (0x1000 vs. 0x1001).** Nur aus `ctx` kannst du *nicht* wissen, welche der
echte buf ist — 1 Bit Information ist verloren. Das Packen des tags ins niedrige Bit ist
**verlustbehaftet bei ungeraden Adressen**.

Also brauchst du **1 zusätzliches Bit** irgendwo, um dir zu merken „ist der Ursprungs-
buffer gerade oder ungerade" — also „muss recover masken oder nicht". Und **der
vtable-Pointer ist genau der Ort, dieses Bit zu speichern**, gratis, weil du ihn ohnehin
mitträgst. `EVEN` = „beim recover masken", `ODD` = „beim recover unverändert lassen".
Eine vtable + nur `ctx` heißt *fehlende Information*, Punkt.

(Es sind nicht 4 verschiedene Zweige: Die zwei ARC-Fälle — ob EVEN oder ODD — sind
*genau gleich*, beide lesen `ctx` direkt als `*mut Shared` ohne Maske, weil `Shared`
immer Bit 0 ist. EVEN/ODD unterscheiden sich *nur* im VEC-Zweig.)

Die vollständige Tabelle, zwei verschiedene Zeitpunkte — beim *encode* (`from_vec`
schaut auf `buf`) und beim *decode* (`clone`/`drop` schauen auf `ctx`):

| Ursprungs-buffer | encode (schaut auf `buf`) | decode (schaut auf `ctx`) | vtable |
|---|---|---|---|
| gerade | Bit setzen `\| 1` | `ctx` gerade → **ARC**, `ctx` ungerade → **VEC (masken zum recover)** | EVEN |
| ungerade | unverändert | `ctx` gerade → **ARC**, `ctx` ungerade → **VEC (unverändert lassen)** | ODD |

## Eine praktische Anmerkung: `ODD` läuft fast nie

In der Praxis richtet der System-allocator *großzügig* aus — Rusts `malloc`/allocator
liefert meist einen Pointer mit alignment ≥ 16, sogar für einen `u8`-buffer (der nur
alignment 1 braucht). Also ist `buf` fast immer gerade, und `PROMOTABLE_ODD_VTABLE` ist
auf einem gewöhnlichen allocator fast toter Code. Aber das alignment von `u8`
*garantiert* keine gerade Adresse (ein eigener allocator, eine arena oder eine
Sub-Allokation kann eine ungerade Adresse zurückgeben), also existiert der ODD-Zweig
rein als *Sicherheitsnetz für die Korrektheit*. Um `promotable_odd_*` wirklich zu
durchlaufen, müsstest du absichtlich eine `Bytes` auf einem buffer mit ungerader Adresse
bauen — der gewöhnliche `from_vec`-Weg kommt vielleicht nie dorthin.

## Ein Ausweg: falls dir das Bit-Packing überflüssig vorkommt

Das Gefühl „kleinteilig" ist *richtig*. Und es weist auf etwas hin: Bit-Packing ist ein
Werkzeug für *Allgemeinheit*, nicht Pflicht für eine minimale `Bytes`. Das echte `bytes`
steckt `buf` in `ctx`, weil es `advance`/`split` unterstützt — Operationen, die `ptr`
von `buf` wegbewegen, *ohne* hochzustufen, also muss es den Ursprungs-`buf` anderswo
merken → daraus entstehen tag + EVEN/ODD.

Aber wenn deine `Bytes` das invariant „ein VEC wird nie geslict" hat (Teil 8 baut es —
`slice` stuft immer hoch), dann hat ein VEC-handle *immer* `ptr == buf` und `cap ==
len`. Das heißt, `buf`/`cap` liegen schon in `ptr`/`len`, sie erneut in `ctx` zu packen
ist *überflüssig*. Dann kannst du auf **eine einzige vtable** zusammenführen und VEC/ARC
per null unterscheiden:

```
ctx == null  → VEC (buf aus self.ptr, cap aus self.len holen)
ctx != null  → ARC (ctx ist *mut Shared)
```

`null` kollidiert nie mit einem `Shared`-Pointer, ist also ein absolut sicherer
sentinel, und das ganze EVEN/ODD verschwindet. Das ist eine echte Entwurfsentscheidung:
den tag behalten, um `bytes` 1:1 zu spiegeln und für ein späteres `advance` bereit zu
sein, oder den tag fallen lassen für Schlankheit mit dem aktuellen feature set. Beide
sind richtig — zu wissen, wofür man gerade zahlt, ist das Wichtige.

## Was wir haben, und was Teil 8 macht

`from_vec` fertig: auf eine boxed slice normalisieren (`cap == len`), das free
übernehmen, dann je nach gerade/ungerade das Bit packen, um `EVEN`/`ODD` zu wählen. Der
mitzunehmende Satz: **VEC ungerade, ARC gerade** — und gerade/ungerade des buffers
entscheidet *nur*, wie man *recover-t* (masken oder nicht), gemerkt über die Wahl der
vtable.

Jetzt können wir eine `promotable`-`Bytes` erzeugen, aber die vier `promotable_*`-
Funktionen sind noch leer, und das erste `clone` — die *Promotion*, für die Teil 4 und 5
das ganze Modell aufgebaut haben — ist noch nicht geschrieben. Teil 8 schreibt den Rest:
die vier dispatch-Funktionen, den CAS-Wettlauf mit dem Verlierer-Zweig, und `slice`
O(1) — das zugleich die Promotion nutzt und das invariant „VEC wird nie geslict" *erzwingt*,
das oben versprochen wurde.

---

*Weiter: [Teil 8 — promotable vollständig und `slice`](08_promotable_and_slice.md) ·
[Inhalt](00_index.md)*

*English: [`../en/07_from_vec_and_bit_tagging.md`](../en/07_from_vec_and_bit_tagging.md)*
