# Teil 8 — promotable vollständig, und `slice` O(1)

Wir haben `from_vec`, das eine `promotable`-`Bytes` erzeugt (Teil 7), und wir haben
verstanden, *warum* Promotion existiert (Teil 4) samt *den Concurrency-Werkzeugen*, die
sie braucht (Teil 5). Dieser letzte Artikel fügt alles zu Code zusammen: die vier
`promotable_*`-Funktionen, die Funktion `promote_vec` mit ihrem CAS-Wettlauf, die
Funktion `slice` O(1), und das stille invariant, das alles stützt.

Das Angenehme: Nach all der Vorbereitung schreiben sich die vier dispatch-Funktionen
fast von selbst. Die ganze Schwierigkeit ballt sich in genau einer Funktion —
`promote_vec` — und genau einem ihrer Zweige, dem *Verlierer*-Zweig.

## Die vier `promotable_*`-Funktionen sind nur dispatch

Jede Funktion tut genau eine Sache: `ctx` lesen, KIND anschauen (nach dem Satz „VEC
ungerade, ARC gerade" aus Teil 7), dann verzweigen. Der ARC-Zweig delegiert an die
`shared`-Helfer aus Teil 6; der VEC-Zweig macht seine eigene Vec-Arbeit.

```rust
fn promotable_even_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let tagged = ctx.load(Ordering::Acquire); // Acquire: gerade könnte jemand promotet & Shared veröffentlicht haben
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe { shallow_clone_arc(tagged as *mut Shared, ptr, len) } // schon promotet → wie share_clone
    } else {
        let buf = (tagged as usize & !KIND_MASK) as *mut u8;          // EVEN: Bit wegmasken
        unsafe { promote_vec(ctx, tagged, buf, ptr, len) }            // erstes clone → promote
    }
}
```

`promotable_odd_clone` ist identisch, nur der VEC-Zweig recover-t ohne Maske:
`let buf = tagged as *mut u8;`. Und die zwei drop-Funktionen sind genauso, nur mit
vertauschten Aufgaben: ARC → `release_shared` (counter verringern), VEC →
`free_boxed_slice` (buffer direkt freigeben, nicht atomar):

```rust
fn promotable_even_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, len: usize) {
    let tagged = *ctx.get_mut(); // &mut = exklusiv → normales Lesen, kein Atomic (siehe Teil 5)
    if tagged as usize & KIND_MASK == KIND_ARC {
        unsafe { release_shared(tagged as *mut Shared) }
    } else {
        let buf = (tagged as usize & !KIND_MASK) as *mut u8;
        unsafe { free_boxed_slice(buf, ptr, len) }
    }
}
```

> **Tödliche Falle:** Am leichtesten *vertauscht* man die KIND-Bedingung. Halte dich fest
> an „VEC ungerade, ARC gerade": Nur der `== KIND_ARC`-Zweig geht den `Shared`-Weg; der
> andere Zweig (VEC) promotet / gibt den buffer frei. Verschreibst du dich zu `==
> KIND_VEC` für den `Shared`-Weg, zwingst du den buffer zu einem `*mut Shared` → stilles
> UB. Genau diese Art Bug ist es, wofür `miri` erschaffen wurde.

Beachte, dass das `load` in clone `Acquire` ist, in drop dagegen ein normales Lesen über
`get_mut` — genau wie Teil 5 begründete: clone teilt eine Referenz (kann rennen), drop
ist exklusiv (rennt nicht).

## `promote_vec`: `Shared` allokieren, CAS, und den Verlierer behandeln

Das ist das Herzstück. Es setzt genau das „zurück ins Original schreiben" aus Teil 4 und
den CAS aus Teil 5 um.

```rust
unsafe fn promote_vec(
    ctx: &AtomicPtr<()>, tagged: *mut (), buf: *mut u8, ptr: *const u8, len: usize,
) -> Bytes {
    // 1. Größe der Allokation wiederherstellen. Siehe "warum diese Arithmetik sicher ist" unten.
    let cap = (ptr as usize - buf as usize) + len;

    // 2. Shared-Block allokieren, ref_count = 2 (Ursprungs-handle + das clone, das wir gleich zurückgeben).
    let shared = Box::into_raw(Box::new(Shared {
        buf, cap, ref_count: AtomicUsize::new(2),
    }));

    // 3. Veröffentlichen: ctx von `tagged` auf `shared` swappen.
    match ctx.compare_exchange(tagged, shared as *mut (), Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8), len,
            ctx: AtomicPtr::new(shared as *mut ()), vtable: &SHARE_VTABLE,
        },
        Err(actual) => {
            // Jemand hat zuerst promotet. Wirf MEIN Shared weg, häng dich an das des Gewinners.
            drop(Box::from_raw(shared));                          // Kontroll-Hülle freigeben, NICHT buf freigeben
            shallow_clone_arc(actual as *mut Shared, ptr, len)   // `actual` benutzen, NICHT `shared`
        }
    }
}
```

Drei Punkte sind zu nennen.

**`ref_count = 2`, nicht 1.** Der CAS veröffentlicht `Shared` für *zwei* handles
zugleich: den Ursprungs-handle (`b1`, dessen `ctx` wir gerade CAS-ten) und das clone, das
wir zurückgeben. Beide zeigen jetzt auf dieses `Shared`, also startet der counter bei 2.
Prüfe es mit der Zähl-Denkweise aus Teil 3: zwei handles → zwei drops → auf 0 → einmal
free. Geht auf.

**Der `Ok`-Zweig — das Schöne an der Promotion.** Der CAS schreibt in das `ctx` des
*Ursprungs-handles `b1`* (wir bekommen `ctx: &AtomicPtr`, was genau `&b1.ctx` ist). Also
*wird `b1` an Ort und Stelle zu geteilt*, obwohl `b1.vtable` weiter `PROMOTABLE_*` ist
(unänderbar — Teil 5). Beim nächsten clone/drop von `b1` liest die `promotable_*`-Funktion
`ctx`, sieht KIND_ARC (Bit 0) und geht selbst den `Shared`-Zweig. Das neue clone trägt
direkt `SHARE_VTABLE`. Zwei „Geschmacksrichtungen" arc-gestützter handles existieren
nebeneinander, beide zählen genau einen counter.

**Der `Err(actual)`-Zweig — `actual` ist ANDERS als `shared`.** Das ist die Stelle, die
Teil 4 „vorsichtig sein beim Wegwerfen des überzähligen counters" nannte, und ein
klassischer Bug. `compare_exchange(expected, new)` heißt: „*wenn* `ctx` noch gleich
`expected` ist, ändere es zu `new`, sonst melde den aktuellen Wert". Bei `Err(actual)`:

- `shared` = *mein* eben allokierter `Shared`-Block (z. B. 0xBBB) — verloren, *nutzlos*.
- `actual` = der Wert, der wirklich in `ctx` steht = der `Shared`-Block *des Gewinners*
  (z. B. 0xAAA) — eine *völlig andere* Adresse, weil jeder Thread einmal `Box::new`
  aufruft → zwei Heap-Regionen.

Also müssen wir (a) mein `shared` wegwerfen — und *richtig* wegwerfen:
`Box::from_raw(shared)` gibt nur die *Kontroll-Hülle* frei, **fasst `buf` nicht an**
(weil `Shared` kein `Drop`-impl hat; `buf` gehört jetzt dem `Shared` des Gewinners); dann
(b) `shallow_clone_arc(actual)`, um den counter des Gewinners zu erhöhen. `shared`
(bereits freigegeben) in Schritt (b) versehentlich zu benutzen ist ein sofortiges
use-after-free, *und* lässt das echte `Shared` fallen → counter schief → double-free.

Prüfe den counter im 3-Thread-Wettlauf: Gewinner A erzeugt `Shared` mit `ref=2` (Ursprung
+ A); B und C verlieren, jeder `shallow_clone_arc(actual)` +1 → auf `4`? Nein — nur einer
von B/C „verliert zuerst", aber beide +1, macht **4**... Moment. Zähle noch mal richtig:
Es gibt nur *einen* Ursprungs-handle und *ein* gewinnendes promote (A). Jeder Thread-clone
erzeugt *einen* neuen handle. 3 Thread-clones → 3 neue handles + 1 Ursprung = 4 handles.
A setzt ref=2 (Ursprung + As handle), B +1 = 3 (plus Bs handle), C +1 = 4 (plus Cs
handle). Genau 4 lebende handles → 4 drops → einmal free. Geht auf.

### Warum die Arithmetik `cap = (ptr - buf) + len` sicher ist

`promote_vec` bekommt `cap` nicht gereicht — es stellt es per Arithmetik wieder her.
`(ptr - buf)` ist der Abstand vom buffer-Boden zum Anfang der View; plus `len` ergibt den
Abstand zum *Ende* der View. Das ist nur dann gleich der Größe der Allokation, **wenn die
View immer das Ende der Allokation erreicht** — also der buffer nie am Ende gekürzt wurde.

Und genau so ist es, dank eines invariant: **ein VEC-handle wird nie geslict.** Weil
`slice` (siehe unten) durch `clone` geht, und ein VEC zu klonen *promotet* es zu ARC.
Also hältst du nie einen bereits-geschnittenen VEC — ein VEC ist immer der unversehrte
buffer, `ptr == buf`, `cap == len`. Deshalb stellt auch `free_boxed_slice` `cap` per genau
dieser Arithmetik wieder her, statt `cap` speichern zu müssen:

```rust
unsafe fn free_boxed_slice(buf: *mut u8, ptr: *const u8, len: usize) {
    let cap = (ptr as usize - buf as usize) + len;
    drop(Vec::from_raw_parts(buf, cap, cap));
}
```

(Umgekehrt *speichert* die `shared`-repr `cap` in `Shared`, weil du *nach* dem Promoten an
beiden Enden frei schneiden darfst, also `cap` nicht mehr aus der View wiederherstellen
kannst. Das eine stellt per Arithmetik wieder her, das andere speichert explizit — diese
Asymmetrie ist genau die Folge des invariant.)

## `slice`: O(1), und es *erzwingt* das invariant

Die ganze `Bytes` ist dafür geboren, dass `slice` billig ist. Der Trick: **clone, dann
die View verengen** — nichts kopieren.

```rust
pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    // ... start, end berechnen, in-Grenzen asserten ...
    if start == end {
        return Bytes::from_static(&[]); // leer → keinen refcount halten
    }
    let mut sub = self.clone(); // backing teilen (counter erhöhen / promoten, falls gerade VEC)
    sub.ptr = unsafe { NonNull::new_unchecked(sub.ptr.as_ptr().add(start)) };
    sub.len = end - start;
    sub
}
```

Das Schöne ist, dass du es *einmal* schreibst und es für *alle drei* repr richtig ist,
weil `clone` den repr-spezifischen Teil schon erledigt:

- **static**: triviales clone (kein counter). Verenge auf eine `'static`-slice → weiter
  static, drop weiter no-op. Keine Allokation.
- **shared**: clone erhöht den counter atomar. View verengen; `Shared.buf`/`cap` ändern
  sich nicht, also gibt drop weiter vom Boden frei. *Das* ist der Grund, warum `Shared`
  `buf`/`cap` getrennt von der View speichert.
- **promotable**: clone **promotet** zu shared, dann wird das verengt.

Genau der letzte Punkt ist die schönste Stelle: **eine promotable-`Bytes` zu slicen
promotet sie** — genau das invariant „VEC wird nie geslict", auf das sich sowohl
`promote_vec` als auch `free_boxed_slice` verlassen, um `cap` per Arithmetik
wiederherzustellen. `slice` *befolgt* das invariant nicht nur, es *erzwingt* das
invariant, durch Struktur: Der einzige Weg zu schneiden führt über clone, und clone
promotet. Ein geschlossener Kreis.

Zwei kleine Sicherheitspunkte: `ptr.add(start)` liegt in-Grenzen, weil wir `start <= end
<= len` asserted haben; und einen kleinen offset zu einem non-null-Pointer zu addieren
kann nicht null ergeben, also ist `new_unchecked` weiter korrekt.

## Fertig. Der Blick aufs ganze Code-Bild

Drei repr, vier-plus Funktionen, ein invariant:

```
static     clone: struct kopieren     drop: no-op            (0-mal free)
shared     clone: fetch_add Relaxed    drop: fetch_sub Release + fence(Acquire)  (1-mal free)
promotable clone: nicht promotet → promote_vec (CAS);  schon → shallow_clone_arc
           drop:  nicht promotet → free_boxed_slice;   schon → release_shared

invariant:  slice ⇒ clone ⇒ (VEC → promote) ⇒ VEC wird nie geschnitten
            ⇒ VEC immer ptr==buf, cap==len ⇒ cap per Arithmetik recovern ist sicher
```

Und der Lesepfad — `deref`, `len`, vergleichen, hashen — berührt weiter nur `ptr` +
`len`, nie `ctx`/`vtable`, also ist er so billig wie `Arc<[u8]>`. Die ganze Maschinerie
aus `ctx`/`vtable`/tag/CAS/ordering kommt *nur* bei `clone` oder `drop` ins Spiel.

## Nachweis: nicht glauben, messen

Die Bugs in diesem Artikel — vertauschtes KIND, `shared` vs. `actual`, falsches ordering
— *kompilieren sauber* und *scheinen* oft auf einem Thread *richtig zu laufen*. Sie
zeigen sich erst bei einem Wettlauf oder wenn ein Werkzeug ins Speichermodell schaut.
Also zwei Pflicht-Dinge:

- **`miri`**: `cargo +nightly miri test` — fängt use-after-free, double-free, Lesen
  uninitialisierten Speichers und data race. Drei der vier obigen Bugs schnappt sich
  `miri` sofort.
- **Promotion-Wettlauf-Test**: Lass N Threads *einen* Ursprungs-handle gleichzeitig
  `clone`-en, um mehrere `promote_vec`-Läufe parallel zu erzwingen und in den
  `Err(actual)`-Zweig zu stechen; wiederhole viele Male. `loom` (falls du weiter gehen
  willst) erschöpft alle möglichen Umsortier-Reihenfolgen.

Erinnere dich an den dritten der drei Schlusssätze aus Teil 5: Der gefährliche Bug in
unsafe ist nicht der, der das Programm abstürzen lässt, sondern der, der *korrekt läuft* —
die Intuition aus sicherem Rust ist umgekehrt, der Standard eines Fehlers ist Stille. Bei
promotable ist diese Stille am dicksten. Nimm immer `miri` mit.

## Ende der Serie

Von „ein Byte kommt vom Draht herein" (Teil 1) bis `promote_vec` mit seinem
Verlierer-Zweig (dieser Artikel) wurde jedes Stück vom vorherigen *erzwungen*: `Arc<[u8]>`
erlaubt kein O(1)-`freeze` → Besitz herab in eine vtable → Lesen vom Besitz trennen →
exklusiv-clone ist double-free → Promotion schreibt zurück → `AtomicPtr` löst drei
Anforderungen → und schließlich alles in Code gegossen mit tagged pointer, CAS und einem
selbst-erzwingenden invariant. Kein Stück fiel vom Himmel.

Jetzt kannst du `bytes` nicht nur *lesen*, du kannst es *neu schreiben* — und für jede
Zeile argumentieren.

---

*Zurück: [Teil 7](07_from_vec_and_bit_tagging.md) · [Inhalt](00_index.md)*

*English: [`../en/08_promotable_and_slice.md`](../en/08_promotable_and_slice.md)*
