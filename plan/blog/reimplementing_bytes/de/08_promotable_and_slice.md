# Teil 8 — Wenn die Anforderungen wachsen: `advance`, lazy-promote und das trilemma

Teil 7 baute die einfachste Variante — cap-in-ctx — die genau die *aktuellen*
Anforderungen erfüllt: zero-copy, zero-alloc `freeze`, `slice` O(1), lazy-promote. Aber
echte Software steht selten still. Dieser Artikel fügt Anforderungen *eine nach der
anderen* hinzu, sieht, was bricht, und listet **jede Kodierung von `ctx`** samt ihrem
Preis auf. Zum Abschluss ein Unmöglichkeits-Theorem: In einer 4-Wort-Struktur kannst du
nicht alles haben.

## Anforderung A: `advance` an Ort und Stelle

**Was `advance` ist.** `bytes::Buf::advance(n)` — „verschluckt" die ersten `n` Bytes,
indem es den *View-Pointer verschiebt* (`self.ptr += n`, `len -= n`) **an Ort und
Stelle**, auf einem einzeln-besessenen handle, *ohne zu klonen*. Das ist das Messer eines
*verbrauchenden cursors*.

**Wann es nötig ist.** Netzwerk-frames mit einem laufenden Pointer lesen; manche
Streaming-decoder laufen direkt über einen owned buffer. (Beachte: Ethereums RLP braucht
es meist *nicht* — man läuft mit einem cursor über ein geliehenes `&[u8]`, ohne den
Pointer der owned `Bytes` zu bewegen. Das ist der Grund, warum Teil 7 für diese `Bytes`
passt.)

**Warum cap-in-ctx bricht.** Nach `advance(3)` ist `self.ptr = buf + 3 ≠ buf`. Aber
`owned_drop` freet per `dealloc(self.ptr, cap)` = `dealloc(buf + 3, ...)` — es gibt einen
Pointer *mitten* in der Allokation frei → **UB / kaputter Heap**. Die Wurzel: cap-in-ctx
*nimmt an* `self.ptr == buf`, und `advance` bricht genau diese Annahme.

Mit `advance` ist `self.ptr` nicht mehr vertrauenswürdig als `buf`. Wir müssen `buf`
anderswo speichern. Es gibt zwei Wege, jeder mit einem Preis.

## Weg 1 für `advance`: `buf` in `ctx` speichern → daraus entsteht EVEN/ODD

Wenn `self.ptr` nicht vertrauenswürdig ist, steck den *buffer-Pointer* in `ctx`. Aber
jetzt hat `cap` keinen Platz mehr in `ctx` (die Zelle ist mit dem Pointer belegt). Wir
stellen `cap` per **Arithmetik** wieder her: `cap = (ptr - buf) + len` = Abstand vom Boden
bis zum *Ende* der View. Korrekt *nur wenn* die View immer das Ende der Allokation
erreicht — aber `advance` trimmt nur vorne (das View-Ende steht still), also ist die
Arithmetik in Ordnung... **unter der Bedingung, dass `cap == len` bei der Erzeugung gilt.**
`cap == len` erzwingen = `into_boxed_slice` (Vec shrinken) → **zero-copy-aus-Vec geht
verloren** (ein realloc + memcpy, falls der Vec überschüssigen Platz hat).

Dann kommt der Bit-Packing-Trick — weil `ctx` jetzt einen *Pointer* hält, braucht es ein
Bit, um OWNED von ARC zu unterscheiden. Ein `u8`-buffer-Pointer (align 1) hat *kein*
garantiert freies niedriges Bit:

```
Fall gerade   (buf 0x1000): Bit setzen → ctx = 0x1001 → recover muss Bit LÖSCHEN → 0x1000
Fall ungerade (buf 0x1001): unverändert → ctx = 0x1001 → recover muss HALTEN    → 0x1001
```

**Beide Fälle haben genau dasselbe `ctx` (0x1001), aber `buf` ist verschieden** → das
Packen des tags ins niedrige Bit ist *verlustbehaftet*. Du brauchst **1 zusätzliches
Bit**, um „Ursprung gerade oder ungerade" zu speichern — und der *vtable-Pointer* ist der
Ort dafür: **`EVEN`** („beim recover masken") vs. **`ODD`** („unverändert lassen"). Genau
hier **entstehen die zwei vtables EVEN/ODD — als *Preis dafür, einen Pointer zu
speichern*, also der Preis von `advance`.**

Das ist genau der „aus Vec"-Weg des echten `bytes`. **Trade-off: `advance` + lazy-promote
behalten, aber zero-copy-aus-Vec verlieren (shrink) + EVEN/ODD schultern.**

## Weg 2 für `advance`: refcount von Anfang an

Speichere **sowohl `buf` als auch `cap`** in einem `Shared`-Block auf dem Heap, mit
refcount *ab der Geburt*. `ctx` ist *immer* `*mut Shared`. `self.ptr` ist die View
(advance nach Belieben), `Shared.buf` ist der Boden, `Shared.cap` die Größe. Jede
Operation läuft über `Shared`:

- `advance`: `self.ptr += n`. `slice`: clone (ref++) + verengen. Beide einfach.
- `freeze`: den vorhandenen `Shared` *wiederverwenden* → **0 alloc** — aber nur, wenn
  `Shared` *schon vor dem freeze existiert* → **`BytesMut` muss ab `new()` refcounten**.

**Trade-off: `advance` + zero-alloc-freeze bekommen, aber lazy-promote verlieren** — jeder
buffer auf dem Heap zahlt einen `Shared` + atomic *ab der Geburt*, selbst wenn er nie
geklont wird.

## Anforderung B: lazy-promote als harte Randbedingung

**Was es ist.** Ein einzeln-besessener buffer, der nie geklont wurde, zahlt **kein**
atomic und allokiert **keinen** `Shared`. **Wann es wichtig ist.** RLP-decode gießt
*Millionen* Einweg-blobs; ein atomic + eine alloc *pro blob* ist der größte vermeidbare
Aufwand auf dem hot path. cap-in-ctx (Teil 7) und EVEN/ODD *haben* lazy-promote.
Refcount-von-Anfang-an *nicht*.

## Jede Kodierung von `ctx`, nebeneinander

| Ansatz | `ctx` (nicht-hochgestuft) hält | `buf` aus | `cap` aus | `advance` | zero-copy freeze | lazy-promote | Komplexität |
|---|---|---|---|---|---|---|---|
| **cap-in-ctx** (Teil 7) | `cap` | `self.ptr` | `ctx` | ❌ | ✅ | ✅ | 1 vtable |
| **buf-in-ctx EVEN/ODD** (`bytes`) | buf-Pointer (tagged) | `ctx` (mask) | Arithmetik (`cap==len`) | ✅ | ❌ (shrink) | ✅ | 2 vtables |
| **refcount-von-Anfang-an** | *immer* `*mut Shared` | `Shared` | `Shared` | ✅ | ✅¹ | ❌ | 2 repr, logisch am einfachsten |

¹ zero-copy freeze braucht `BytesMut` refcount-von-Anfang-an.

## Trilemma: warum „alles unterstützen" unmöglich ist

Schau auf die drei Spalten `advance` / zero-copy-freeze / lazy-promote: **keine Zeile
bekommt alle drei.** Das ist keine Implementierungsgrenze — es ist ein Theorem:

> In einer 4-Wort-Struktur bekommst du nur **2 von 3** {lazy-promote, `advance`,
> zero-alloc-freeze mit `cap>len`}.

Konkreter Beweis: `advance` bewegt die View vom Boden weg → `buf` *muss* gespeichert
werden. freeze-`cap>len` → das echte `cap` *muss* gespeichert werden. Das sind **zwei
unabhängige Werte**, aber die `ctx`-Zelle hält nur *einen*. Beide halten → ein
`Shared`-Block auf dem Heap ist nötig → damit freeze *nicht* allokiert, muss `Shared`
*vor* dem freeze existieren → `BytesMut` refcount-von-Anfang-an → **lazy-promote geht
verloren.**

Das ganze trilemma läuft auf **eine Frage** hinaus: *Verlässt die View den buffer-Boden,
**bevor** sie hochgestuft ist (also gibt es `advance`)?*
- **Ja** → `buf` muss gespeichert werden → Pointer in `ctx` → EVEN/ODD, und `cap` muss per
  Arithmetik hergeleitet werden (zero-copy-aus-Vec verloren) *oder* refcount (lazy-promote
  verloren).
- **Nein** → `ctx` ist frei → `cap` packen → eine vtable, behält sowohl lazy-promote als
  auch zero-alloc-freeze.

## Fazit: der „richtige" Entwurf = *deine* Anforderungen

Es gibt keine absolut beste Variante. Wähle den Punkt, der zu den echten Anforderungen
passt:

- **`Bytes` für Ethereum/RLP** (dieser Artikel): slice + clone + freeze, *kein* advance
  auf einem owned handle → **cap-in-ctx** (Teil 7). lazy-promote (billiger hot path) +
  zero-alloc-freeze behalten, im Tausch gegen `advance`, das diese Art nicht nutzt. Das
  ist die richtige Wahl.
- **`bytes` als ein `Buf`** (Netzwerk): braucht `advance` → **EVEN/ODD** (shrink-aus-Vec
  in Kauf nehmen) + `BytesMut` refcount-von-Anfang-an für zero-copy freeze. *Deshalb* ist
  das echte `bytes` komplex — es zahlt den Preis für ein breiteres feature set.
- **Am allgemeinsten / am leichtesten zu durchdenken**: **alles refcounten**
  (lazy-promote aufgeben) — zwei repr STATIC + SHARED, kein tag, keine promotion.

Die mitzunehmende Lektion: „`bytes` neu schreiben" heißt *nicht*, es Zeile für Zeile
abzuschreiben. Es heißt, den ganzen **Entwurfsraum** zu verstehen und den richtigen Punkt
für die eigenen Anforderungen zu wählen — und dann begründen zu können, warum. `bytes`
wählt EVEN/ODD, weil es ein `Buf` ist; wir wählen cap-in-ctx, weil diese `Bytes` slice-t
statt advance-t. Beide sind *richtig* — für ihr jeweiliges Problem.

## Nachweis, und Ende der Serie

Bugs in allen drei Entwürfen — vertauschter KIND-Zweig, `shared` vs. `actual`, falsches
ordering, dealloc mit falschem `cap`/`buf` — *kompilieren sauber* und *scheinen* auf einem
Thread *richtig zu laufen*. Pflicht: **`miri`** (`cargo +nightly miri test`, mit
`-Zmiri-strict-provenance` für cap-in-ctx), und ein **Promotion-Wettlauf-Test** (N Threads
`clone`-en denselben handle → stechen in `Err(actual)`; `loom`, um alle interleavings zu
erschöpfen).

Von „ein Byte kommt vom Draht herein" (Teil 1) bis zum trilemma (dieser Artikel) wurde
jedes Stück vom vorherigen *erzwungen*, und das letzte Stück zeigt: Sogar „wie man eine
8-Byte-Zelle kodiert" hat keine absolute Antwort — nur *benannte* Trade-offs, gewählt nach
den Anforderungen. Jetzt kannst du `bytes` nicht nur lesen, sondern es an *jedem* Punkt des
Trade-off-Raums *neu entwerfen* und für deine Wahl argumentieren.

---

*Zurück: [Teil 7](07_from_vec_and_bit_tagging.md) · [Inhalt](00_index.md)*

*English: [`../en/08_promotable_and_slice.md`](../en/08_promotable_and_slice.md)*
