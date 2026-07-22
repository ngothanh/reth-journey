# Teil 6 — Vom Modell zum Code: `static` und `shared`

Die ersten fünf Teile haben das *Modell* fertig aufgebaut: eine `Bytes` besteht aus
`ptr` + `len` (welche Bytes) und `data` + `vtable` (wer besitzt), mit drei Arten des
Besitzens — `static`, `promotable`, `shared`. Ab hier *setzen wir uns hin und
schreiben*. Und das Angenehme: Zwei der drei Besitz-Arten sind fast trivial
hinzuschreiben. `static` ist die Aufwärmübung, `shared` ist nur an einer einzigen
Stelle schwer richtig zu machen — aber genau diese Stelle ist die wichtigste Lektion
zum Memory Ordering, die Teil 5 *noch nicht* berührt hat.

Wir schreiben die ersten vier vtable-Funktionen: `static_clone`, `static_drop`,
`share_clone`, `share_drop`. Und wir beantworten eine Frage, die Teil 5 aufgeschoben
hat: Das Ordering der Promotion dient dazu, einen `Shared`-Block zu *veröffentlichen*;
das Ordering von `share_drop` dagegen dient dazu, einen geteilten buffer *freizugeben*
— eine völlig andere Gefahr, namens *free-while-read*.

## Landkarte: `ctx` lesen → repr kennen → welche Funktion laufen soll

Der ganze Implementierungsteil dreht sich um eine Bewegung: Jede vtable-Funktion liest
`ctx`, leitet daraus ab, in welchem repr sie gerade ist, und verzweigt. Verankere das im
Kopf, bevor du in den Code gehst:

```
vtable = STATIC       ctx = null                  clone: copy struct   · drop: no-op        (0-mal freigegeben)

vtable = SHARE        ctx = *mut Shared           clone: +refcount     · drop: -refcount    (1-mal freigegeben)

vtable = OWNED        ctx UNGERADE (cap<<1|1)     clone: promote_owned · drop: dealloc(self.ptr, cap)
                      ctx GERADE   (*mut Shared)  clone/drop: über Shared (wie Zeile SHARE)

     einziger Zustandsübergang, einseitig:
        OWNED (cap in ctx) ──(erster clone: promote_owned, CAS)──► OWNED/ARC (Shared in ctx)
```

Teil 6 schreibt die ersten zwei Zeilen (`STATIC`, `SHARE`). **Teil 7 baut die ganze
`OWNED`-Zeile** — die einfachste Variante, die zero-copy/zero-alloc `freeze` erreicht:
`cap` in `ctx` encoden, `promote_owned`, `slice`. **Teil 8** fügt die *alltäglichen*
Anforderungen hinzu (advance an Ort und Stelle, lazy-promote als harte Randbedingung),
zeigt, dass EVEN/ODD von `bytes` *der Preis für advance* ist, und schließt mit dem
**trilemma**. Merke: `vtable` friert bei der Geburt ein; nur das *niedrige Bit von `ctx`*
ändert sich beim Hochstufen — deshalb benutzt ein bereits-hochgestufter OWNED-handle
weiter `OWNED_VTABLE`, verzweigt nur in den Shared-Zweig.

## `static`: die Aufwärmübung

Erinnere dich: Eine `static`-`Bytes` zeigt in Speicher, der ewig lebt (`&'static
[u8]`), also gibt es nichts zu zählen und nichts freizugeben. `data` bleibt null. Seine
zwei Funktionen sind die zwei kürzesten Antworten der ganzen Serie:

```rust
fn static_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    // Kein refcount. Clone baut nur einen handle nach, der auf dieselbe Stelle zeigt.
    unsafe {
        Bytes {
            ptr: NonNull::new_unchecked(ptr as *mut u8),
            len,
            ctx: AtomicPtr::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }
}

fn static_drop(_ctx: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    // Tut nichts. Die Bytes leben ewig, es gibt nichts freizugeben.
}
```

`static_drop` ist *mit Absicht* leer — es ist die Verkörperung der ersten der fünf
mitzunehmenden Fragen: *Wie oft genau wird dieser Block freigegeben?* Für `static`
lautet die Antwort **0**. Eine leere `drop`-Funktion ist nicht unfertig; sie ist
„0-mal" in Code gegossen. Beachte, dass `ctx` hier null ist, also darf es absolut nicht
dereferenziert werden — und zum Glück dereferenziert es keine einzige Zeile.

## `shared`: der `Shared`-Block und seine drei Felder

`shared` ist ein selbstgeschriebenes `Arc<[u8]>`. Wir brauchen einen Kontrollblock auf
dem Heap, der den counter hält:

```rust
struct Shared {
    buf: *mut u8,          // URSPRUNGS-Adresse der Allokation — um sie später dem allocator zurückzugeben
    cap: usize,            // Größe der Allokation — mit buf zusammen ergibt sie die "Art, freizugeben"
    ref_count: AtomicUsize,
}
```

Ein Detail, das Teil 4 vorausschickte, wird hier konkret: `Shared.buf` ist die
*Ursprungs*-Adresse der Allokation, **nicht** der Pointer, den der handle gerade hält
(`Bytes.ptr`). Bei einem ungeschnittenen handle sind die zwei gleich; aber nach einem
`slice` zeigt `Bytes.ptr` in die *Mitte* des buffers, während `buf` weiter der Anfang
sein muss — denn dem allocator darf man nur genau den Pointer zurückgeben, den er
ausgegeben hat. Deshalb leben `buf`/`cap` in `Shared`, getrennt von `ptr`/`len` des
handles. (Teil 8 nutzt genau diese Eigenschaft, um `slice` O(1) zu machen.)

## `share_clone`: counter erhöhen, und warum `Relaxed` genügt

```rust
fn share_clone(ctx: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = ctx.load(Ordering::Relaxed) as *mut Shared;
    unsafe { shallow_clone_arc(shared, ptr, len) }
}

unsafe fn shallow_clone_arc(shared: *mut Shared, ptr: *const u8, len: usize) -> Bytes {
    let old = (*shared).ref_count.fetch_add(1, Ordering::Relaxed);
    if old > isize::MAX as usize / 2 {
        abort(); // siehe "Warum abort statt panic" unten
    }
    Bytes {
        ptr: NonNull::new_unchecked(ptr as *mut u8),
        len,
        ctx: AtomicPtr::new(shared as *mut ()),
        vtable: &SHARE_VTABLE,
    }
}
```

Es gibt hier *zwei* atomare Operationen, und beide sind `Relaxed`. Das ist die Stelle,
die Neulinge leicht verwirrt, also gehen wir sie gründlich durch.

**Das `load` des `shared`-Pointers: `Relaxed`.** Erinnere dich an das Prinzip aus Teil
5 — Ordering schützt *nicht den atomaren Wert selbst*, es schützt *den anderen Speicher
rundherum* um diese Operation. Hier ist der `shared`-Pointer eine *stabile* Adresse:
Sie wird bei der Geburt des handles gesetzt und ändert sich nie über die gesamte
Lebenszeit des handles. Wir benutzen dieses Lesen nicht als *Signal* dafür, dass gerade
neuer Speicher veröffentlicht wurde — wir holen nur eine Adresse, die *wir ohnehin schon
besitzen*. Keine happens-before-Kante ist aufzubauen, also ist `Relaxed` das ehrliche
Minimum.

(Zum Vergleich zum Merken: Das Lesen von `data` in `promotable_clone` in Teil 5 muss
`Acquire` sein, weil es dort *vielleicht* ein Signal „gerade fertig hochgestuft, hier
ist ein neues `Shared`" ist — und wir werden gleich den *Inhalt* dieses `Shared`-Blocks
*lesen*. Dasselbe `load`, anderes Ordering, weil das eine „eine bereits besessene
Adresse holen" ist und das andere „gerade veröffentlichten Speicher empfangen".)

**Das `fetch_add`, das den counter erhöht: `Relaxed`.** Den refcount zu erhöhen
*veröffentlicht* niemandem irgendeinen Speicher. Um `clone` überhaupt rufen zu können,
hältst du bereits einen lebendigen handle → die payload und der `Shared`-Block existieren
schon und sind für dich sichtbar. Das Erhöhen ist nur Arithmetik auf einem Zähler; es
gibt nichts zu synchronisieren. Also `Relaxed`.

**Der Overflow-Riegel — und warum `abort` statt `panic`.** Weil `fetch_add` mit
`Relaxed` sehr billig ist, könnte eine krankhafte `mem::forget`-Schleife (oder ein
Clone-Sturm) *theoretisch* den `usize` überlaufen und auf eine kleine Zahl zurückbringen
→ verfrühte Freigabe → use-after-free. Also setzen wir einen Riegel: Übersteigt der
counter die Schwelle, wird hart gestoppt. Gestoppt wird mit `abort`, nicht `panic`, denn
zu dem Zeitpunkt ist die Speichersicherheit schon kaputt — und `panic` *kann von
`catch_unwind` abgefangen werden* und es *unwindet durch die `Drop`s*, und `Drop` fasst
genau diesen nicht mehr vertrauenswürdigen counter an. `abort` ist ein bedingungsloser
Stopp. (Wir prüfen die Schwelle über den *Rückgabewert von `fetch_add`*, nicht über ein
separates `load` — um die TOCTOU-Lücke zwischen „lesen" und „erhöhen" zu vermeiden.)

## `share_drop`: die Gefahr free-while-read

Das ist der wertvolle Teil des ganzen Artikels. `share_drop` verringert den counter,
und wenn ich der Letzte bin, gebe ich den buffer + den `Shared`-Block frei.

```rust
fn share_drop(ctx: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    let shared = ctx.load(Ordering::Relaxed) as *mut Shared;
    unsafe { release_shared(shared) }
}

unsafe fn release_shared(shared: *mut Shared) {
    if (*shared).ref_count.fetch_sub(1, Ordering::Release) != 1 {
        return; // noch nicht der Letzte — fertig
    }
    core::sync::atomic::fence(Ordering::Acquire);
    let cap = (*shared).cap;
    drop(Vec::from_raw_parts((*shared).buf, cap, cap)); // buffer von der URSPRUNGS-Adresse freigeben
    drop(Box::from_raw(shared));                        // den Shared-Block freigeben
}
```

Beachte, dass `release_shared` **`ptr`/`len` des handles nicht braucht** — es gibt die
ganze Allokation von `Shared.buf`/`Shared.cap` frei. Genau das macht `slice` sicher:
Egal wie weit der handle geschnitten wurde, drop gibt immer den korrekten
Ursprungs-Pointer zurück. (Wir benutzen `cap` sowohl für die Länge *als auch* die
capacity von `Vec::from_raw_parts` — wir beschreiben die *Allokation*, nicht die *View*.
`u8` hat keinen destructor, also beeinflusst die Länge nur „wie viele destructor laufen",
aber die Allokation korrekt zu beschreiben ist eine Gewohnheit, die man behalten muss:
An dem Tag, an dem der buffer einen Typ mit `Drop` hält, würde das versehentliche
Benutzen der `len` der View die falsche Anzahl destructor laufen lassen.)

Nun zum Ordering, und warum es sich **völlig unterscheidet** vom Ordering aus Teil 5.

### Das Problem: freigeben, während ein anderer Thread noch liest

Teil 5 sorgte sich um die Gefahr *publish-before-read*: die Adresse des `Shared`-Blocks
veröffentlichen, bevor sein Inhalt erscheinen konnte. Hier ist die Gefahr umgekehrt:
**den buffer freigeben, während ein anderer Thread ihn noch liest** — free-while-read.

Bühne aufbauen: `b1` und `b2` sind zwei handles, die sich denselben buffer teilen, auf
zwei verschiedenen Threads. Thread A liest ein paar Bytes und dropt dann `b2`; Thread B
dropt `b1`. Der counter geht `2 → 1 → 0`. Deine sequenzielle Intuition sagt: „counter
auf 0 heißt, niemand benutzt es mehr → sichere Freigabe". Richtig — *wenn es nur einen
Thread gibt*. Aber über mehrere Threads, auf umsortierender Hardware, **sind „counter
auf 0" und „alle Lesevorgänge sind fertig" NICHT automatisch derselbe Zeitpunkt.** Die
CPU/der Compiler darf Thread As Lesen des buffers *hinter* dessen eigenes Verringern des
counters verschieben.

Sieh, wie es bricht, wenn es *kein* Ordering gibt (angenommen, beide Verringerungen sind
`Relaxed`):

```
Thread A                              Thread B
  fetch_sub → 2→1 (Relaxed)
  ...lesen b2[0] HIERHER VERSCHOBEN      fetch_sub → 1→0, sieht 0
       │                                 free(buf)         ← buffer verschwindet
       └── lesen b2[0] JETZT GERADE ←──────────────────── USE-AFTER-FREE
```

As Lesen wird über dessen eigenes Verringern verschoben, also sieht B den counter auf 0
und gibt frei, *während* As Lesen noch hängt. Es liest aus totem Speicher.

### Die Abhilfe: `Release` beim Verringern, `Acquire`-fence vor der Freigabe

- Jeder, der dropt, verringert den counter mit **`Release`** → „veröffentliche: alle
  meine buffer-Zugriffe liegen *vor* diesem Verringern, keiner darf dahinter rutschen."
- Der Letzte (das `fetch_sub`, das 1 zurückgibt) führt einen **`fence(Acquire)`** *vor*
  der Freigabe aus → „abonniere: synchronisiere mit *jeder* `Release`-Verringerung der
  anderen Threads, sodass all deren buffer-Zugriffe nun happens-before meiner Freigabe."

Dieses `Release`/`Acquire`-Paar ist genau das, was „counter auf 0" an „alle Leser sind
wirklich fertig" *klebt*. Ohne es ist der counter richtig, aber die Speichersichtbarkeit
falsch.

Ein subtiles Detail, das *einen* fence genügen lässt, um mit *allen* Verringerungen zu
synchronisieren: Jedes `fetch_sub` ist eine Read-Modify-Write-Operation, also liest die
letzte Verringerung einen Wert, der in der *Release-Sequenz* liegt, die von jeder
vorherigen `Release`-Verringerung geführt wird — genau das erlaubt einem einzigen
`fence(Acquire)`, sich mit allen zu paaren.

### Warum ein separates `fence(Acquire)` statt `fetch_sub(AcqRel)`?

Du *könntest* die Verringerung zu `AcqRel` machen und den fence weglassen — auch
richtig. Aber `AcqRel` zwingt `Acquire` auf *jede* Verringerung, auch die
nicht-letzten (die nur returnen, nichts freigeben). Der separate fence lässt **nur den
Letzten** den Preis der `Acquire`-Barriere zahlen; alle anderen verringern nur mit dem
billigeren `Release`. Das ist eine Frage der Performance, nicht von Richtig-oder-Falsch
— und genau der Grund, warum das echte `Arc` so geschrieben ist.

## Zwei Arten von Ordering in der Serie gegenübergestellt

Das ist der mitzunehmende Punkt, denn er trennt zwei Gefahren, die man oft in einen Topf
wirft:

| | Teil 5 (promotion) | Teil 6 (`share_drop`) |
|---|---|---|
| Gefahr | publish-before-read: Pointer veröffentlichen, bevor der Inhalt erscheint | free-while-read: freigeben, während ein anderer noch liest |
| Operation | CAS schreibt `data` = neues `Shared` | `fetch_sub` counter |
| „Veröffentlichen"-Seite | CAS erfolgreich → `Release` | jede Verringerung → `Release` |
| „Empfangen"-Seite | `load`/fehlgeschlagenes CAS → `Acquire` | `fence(Acquire)` des Letzten |

Dasselbe `Release`/`Acquire`-Paar, zwei verschiedene Probleme. Das allgemeine Prinzip
gilt weiter: *Wann immer eine deiner atomaren Operationen von einem anderen Thread als
Signal benutzt wird, um zu entscheiden „jetzt darf ich den gemeinsamen Speicher anfassen
(oder freigeben)", müssen die Speicherzugriffe rund um diese Operation über ein
Release/Acquire-Paar geordnet werden.*

## Was wir haben, und was Teil 7 macht

Vier Funktionen fertig: `static_*` (0-mal freigeben), `share_*` (`Relaxed`-Disziplin
beim Erhöhen, `Release`+`fence(Acquire)` beim Verringern). Der Kernpunkt: Das Ordering
von `share_drop` ist nicht das Ordering aus Teil 5 — es wehrt free-while-read ab, nicht
publish-before-read.

Aber wir können immer noch keine `shared`-`Bytes` *erzeugen*. Nichts ruft bisher
`SHARE_VTABLE`. Das fehlende Stück ist `from_vec` — ein `Vec<u8>` in eine `Bytes`
verwandeln. Und genau beim Schreiben von `from_vec` stoßen wir auf das, was Teil 5
absichtlich in einer Randnotiz aufschob: Wie kann *eine 8-Byte-Zelle* zugleich einen
buffer-Pointer und einen `Shared`-Pointer halten und die zwei Arten unterscheiden? Das
ist der Bit-Packing-Trick, und Teil 7 seziert ihn bis auf den Grund.

---

*Weiter: [Teil 7 — `from_vec` und der Bit-Packing-Trick](07_from_vec_and_bit_tagging.md) ·
[Inhalt](00_index.md)*

*English: [`../en/06_static_and_shared.md`](../en/06_static_and_shared.md)*
