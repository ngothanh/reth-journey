# Teil 7 — Die einfachste Variante: zero-copy, zero-alloc `freeze`

Teil 6 gab uns zwei repr — `static` und `shared` — aber es gibt noch nichts, das eine
`Bytes` *erzeugt*, die einen buffer-auf-dem-Heap besitzt, und die titelgebende
Anforderung der ganzen Serie ist noch nicht erreicht: **`freeze` muss O(1) sein —
zero-copy, zero-allocation.** Dieser Artikel baut die **einfachste Variante**, die genau
diese Anforderung erfüllt, und *nur* diese.

Das ist eine bewusste Wahl: Wir bauen *nicht* auf Vorrat für noch nicht vorhandene
Bedürfnisse (advance an Ort und Stelle, fortgeschrittene lazy-promote-Optimierung). Wir
beginnen mit dem lauffähigen Minimum. Erst Teil 8 fragt „was, wenn wir mehr brauchen?" —
und zeigt, dass jedes zusätzliche Bedürfnis einen Trade-off *erzwingt*.

## Das Ein-Besitzer-Problem

Eine `Bytes`, die gerade aus `from_vec` oder `BytesMut::freeze` kommt, **besitzt einen
buffer, allein**. Sie muss zwei Dinge können:

- **drop** → den buffer freigeben. `dealloc` braucht den *Boden der Allokation* + das
  *`cap`* (um das korrekte `Layout::array::<u8>(cap)` zu rekonstruieren).
- **clone** → auf shared hochstufen (Teil 4): einen `Shared` mit refcount allokieren.

Beide Dinge brauchen Information, aber wir haben nur *eine* Zelle zum Ablegen: `ctx`. Und
`ctx` muss sich vom `Shared`-Pointer (dem bereits-hochgestuften Zustand) unterscheiden
lassen. Was packen wir also in `ctx`?

## Die entscheidende Vereinfachung: `self.ptr` ist bereits der buffer-Boden

Hier fällt alles zusammen. Bei einem besitzenden handle, dessen *View den Boden nie
verlässt*, ist **`self.ptr` genau der buffer-Boden (`buf`)**. Also **braucht** `ctx`
keinen Pointer zu speichern — es speichert genau das, was `drop` *nicht* aus `ptr`/`len`
ableiten kann: das **`cap`**.

(Die Bedingung „View verlässt den Boden nie" gilt, weil der einzige Weg, `ptr` zu
bewegen, `slice` ist, und `slice` *promotet* — siehe Ende des Artikels. Also hat ein
OWNED-handle *immer* `self.ptr == buf`. Das ist das Grund-invariant des ganzen Entwurfs.)

## Kodierung: `cap` in `ctx`

```rust
const OWNED_TAG: usize = 1;
//   ctx UNGERADE (bit 0 = 1) → OWNED: ctx = (cap << 1) | 1;  buf = self.ptr
//   ctx GERADE   (bit 0 = 0) → ARC:   ctx = *mut Shared  (Shared alignment ≥ 8 → immer gerade)
```

Ein niedriges Bit unterscheidet die zwei Zustände. `Shared` auf dem Heap ist immer gerade
(alignment), also *erzwingen* wir, dass OWNED immer ungerade ist, per `(cap << 1) | 1` —
`cap` ist eine Zahl, die wir selbst kontrollieren, links schieben und dann das Bit setzen,
fertig. **Eine einzige `OWNED_VTABLE`.** (Kein „buffer gerade/ungerade", kein EVEN/ODD —
das ist Sache von Teil 8, wenn wir gezwungen sind, einen *Pointer* statt des *cap* zu
speichern.)

## `from_vec` und `from_owned_parts`

```rust
pub fn from_vec(bytes: Vec<u8>) -> Self {
    if bytes.is_empty() {
        return Self::from_static(&[]); // leer → static, 0 Allokationen (leeres Vec droppt normal)
    }
    // cap des Vec UNVERÄNDERT behalten — KEIN into_boxed_slice, KEIN realloc.
    let mut bytes = core::mem::ManuallyDrop::new(bytes);
    let (buf, len, cap) = (bytes.as_mut_ptr(), bytes.len(), bytes.capacity());
    unsafe { Self::from_owned_parts(NonNull::new_unchecked(buf), len, cap) }
}

pub(crate) unsafe fn from_owned_parts(ptr: NonNull<u8>, len: usize, cap: usize) -> Self {
    if cap == 0 { return Bytes::from_static(&[]); } // z. B. BytesMut::new(0) → ptr dangling
    Bytes {
        ptr, len,                                    // self.ptr = buf
        // cap in ctx gepackt als provenance-lose Adresse (wir lesen nur .addr() zurück, kein deref)
        ctx: AtomicPtr::new(ptr::without_provenance_mut((cap << 1) | OWNED_TAG)),
        vtable: &OWNED_VTABLE,
    }
}
```

Zwei Punkte machen die ganze Schönheit dieser Variante aus:

- **Kein `into_boxed_slice`.** Das echte `bytes` shrinkt den Vec auf `cap == len` (ein
  realloc + memcpy, falls der Vec überschüssigen Platz hat). Wir *nicht* — buffer
  unverändert, `cap` darf > `len` sein. Dadurch ist `BytesMut::freeze` eines buffers mit
  `cap 1024 / len 7` **zero-copy** (Pointer bleibt gleich) *und* `from_owned_parts`
  **allokiert nichts** (nicht mal einen control-block) → **zero allocation**. Genau das
  ist die titelgebende Anforderung, erreicht.
- **`without_provenance_mut` + `.addr()`**: Wir speichern eine *Ganzzahl* in einer
  `AtomicPtr`-Zelle. Weil wir sie nie als Pointer deref-en, ist das die korrekte
  strict-provenance-API — Miri `-Zmiri-strict-provenance` bleibt sauber.

## `owned_clone` / `owned_drop` — nur dispatch

```rust
fn owned_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let raw = ctx.load(Ordering::Acquire); // Acquire: gerade könnte jemand promotet & Shared veröffentlicht haben
    if raw.addr() & OWNED_TAG == 0 {
        unsafe { shallow_clone_arc(raw as *mut Shared, ptr, len) } // schon promotet → wie share_clone
    } else {
        let cap = raw.addr() >> 1;                                 // cap DIREKT gelesen, keine Arithmetik
        unsafe { promote_owned(ctx, raw, ptr, cap, len) }          // erstes clone → promote
    }
}

fn owned_drop(ctx: &mut AtomicPtr<()>, ptr: *const u8, _len: usize) {
    let raw = *ctx.get_mut(); // &mut = exklusiv → normales Lesen, kein Atomic (Teil 5)
    if raw.addr() & OWNED_TAG == 0 {
        unsafe { release_shared(raw as *mut Shared) }
    } else {
        let cap = raw.addr() >> 1;
        unsafe { dealloc(ptr as *mut u8, Layout::array::<u8>(cap).unwrap()) } // buf = self.ptr
    }
}
```

`buf` ist `self.ptr` (keine Maske), `cap` ist `ctx.addr() >> 1` (direkt gelesen).
Verglichen mit EVEN/ODD aus Teil 8 — Pointer maskieren + `cap` per Arithmetik herleiten —
ist das deutlich schlanker.

> **Falle:** Den KIND-Zweig vertauschen. Halte dich fest: `ctx` **gerade = ARC**, `ctx`
> **ungerade = OWNED**. Verwechselst du es, zwingst du eine cap-Zahl zu einem
> `*mut Shared` und deref-st es → stilles UB. `miri` fängt genau diese Art.

## `promote_owned` — `Shared` allokieren, CAS, den Verlierer behandeln

Das Herzstück des Artikels: die Umsetzung des „zurück ins Original schreiben" (Teil 4) +
des CAS (Teil 5).

```rust
unsafe fn promote_owned(
    ctx: &AtomicPtr<()>, tagged: *mut (), ptr: *const u8, cap: usize, len: usize,
) -> Bytes {
    let shared = Box::into_raw(Box::new(Shared {
        buf: ptr as *mut u8, cap, ref_count: AtomicUsize::new(2), // Ursprungs-handle + das clone
    }));
    match ctx.compare_exchange(tagged, shared as *mut (), Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8), len,
            ctx: AtomicPtr::new(shared as *mut ()), vtable: &SHARE_VTABLE,
        },
        Err(actual) => {
            drop(Box::from_raw(shared));                        // Hülle freigeben, NICHT buf freigeben
            shallow_clone_arc(actual as *mut Shared, ptr, len) // `actual` benutzen, NICHT `shared`
        }
    }
}
```

- **`ref_count = 2`**: Der CAS veröffentlicht `Shared` für *zwei* handles — den Ursprung
  `b1` (dessen `ctx` wir gerade CAS-ten) + das zurückgegebene clone. Zwei drops → auf 0 →
  einmal free. Geht auf.
- **`Ok` — das Schöne**: Der CAS schreibt in das `ctx` des *Ursprungs-handles*, also wird
  `b1` *an Ort und Stelle* zu shared, obwohl `b1.vtable` weiter `OWNED_VTABLE` ist; beim
  nächsten Mal liest es `ctx`, sieht das gerade Bit → geht selbst den Shared-Zweig.
- **`Err(actual)` — der klassische Bug**: `actual` = das `Shared` des **Gewinners**
  (anders als das eigene `shared`, weil jeder Thread `Box::new` in einer eigenen
  Heap-Region aufruft). Wir müssen das eigene `shared` wegwerfen (`Box::from_raw` gibt nur
  die *Hülle* frei, fasst `buf` nicht an, weil `Shared` kein `Drop` hat) und uns dann an
  `actual` hängen. `shared` (bereits freigegeben) versehentlich zu benutzen ist ein
  sofortiges use-after-free.

## `slice` — O(1), und es *erzwingt* das invariant

```rust
pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    // ... start, end berechnen, in-Grenzen asserten ...
    if start == end { return Bytes::from_static(&[]); }
    let mut sub = self.clone();  // backing teilen (counter erhöhen / promoten, falls gerade OWNED)
    sub.ptr = unsafe { NonNull::new_unchecked(sub.ptr.as_ptr().add(start)) };
    sub.len = end - start;
    sub
}
```

*Einmal* geschrieben, korrekt für alle drei repr, weil `clone` den repr-spezifischen Teil
schon erledigt. Der Kernpunkt: **eine OWNED-`Bytes` zu slicen `clone`-t sie → ein OWNED zu
klonen *promotet* es zu SHARED.** Also ist das Schnitt-Ergebnis immer SHARED (nutzt
`Shared.buf` als Boden, frei schneidbar), während der ursprüngliche OWNED-handle *nie*
seinen `ptr` bewegt sieht. So wird das invariant `self.ptr == buf` *durch Struktur
erzwungen*: Der einzige Weg, `ptr` zu bewegen, ist `slice`, und `slice` promotet.
`owned_drop`s `dealloc(self.ptr, cap)` trifft dadurch immer den Boden.

## Die einfachste Variante ist fertig

Wir haben eine vollständige, korrekte `Bytes`, die **die titelgebende Anforderung
erreicht**: `freeze` zero-copy + zero-alloc, `slice` O(1), `clone` lazy-promote, Lesen so
billig wie `Arc<[u8]>`. Miri `-Zmiri-strict-provenance` bleibt sauber, ein `freeze`-Test
bestätigt 0 alloc / 0 dealloc.

```
static  ctx = null                 clone: copy      drop: no-op                (free 0)
shared  ctx = *mut Shared          clone: +refcount drop: -refcount+fence      (free 1)
OWNED   ctx = (cap<<1|1) ODER Shared;  buf = self.ptr;  clone: promote/arc  drop: dealloc/arc
```

**Aber** — das ist die Variante für genau die *aktuellen Anforderungen*. Der Alltag
gebiert weitere: *advance an Ort und Stelle* (wann? was kostet es? wie schreibt man es?)
und *lazy-promote als harte Randbedingung*. Teil 8 seziert jede einzeln: Jede neue
Anforderung **erzwingt** eine andere Kodierung, die EVEN/ODD oder refcount-von-Anfang-an
nach sich zieht — und schließlich das **trilemma**, das zeigt, warum „alles unterstützen"
in einer 4-Wort-Struktur unmöglich ist.

---

*Weiter: [Teil 8 — Wenn die Anforderungen wachsen: advance, lazy-promote und das trilemma](08_promotable_and_slice.md) ·
[Inhalt](00_index.md)*

*English: [`../en/07_from_vec_and_bit_tagging.md`](../en/07_from_vec_and_bit_tagging.md)*
