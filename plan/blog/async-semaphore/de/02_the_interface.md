# Teil 2 — Das Interface, von den Use Cases abgelesen

Das Briefing in Rust zu übersetzen beginnt mit den zwei Operationen, die jeder
einer Semaphore zutraut. Bitte irgendwen, die API zu skizzieren, und du bekommst
ungefähr das:

```rust
impl Semaphore {
    pub fn new(permits: usize) -> Self;
    pub fn acquire(&self);   // ein Permit nehmen, notfalls warten
    pub fn release(&self);   // es zurückgeben
}
```

Sieht vollständig aus. Am Ende dieses Teils wird `release` komplett verschwunden
sein, `acquire` einen Rückgabetyp haben, hinter dessen jedem Stück eine Geschichte
steckt, und drei Methoden werden aufgetaucht sein, von denen diese Skizze nichts
ahnt. Keine dieser Änderungen ist Geschmackssache — jede wird von einem Use Case
aus Teil 1 erzwungen, und dem *Warum* zu folgen ist der schnellste Weg, das
Interface zu verstehen statt es auswendig zu lernen.

## Das Permit gibt sich selbst zurück

Beginnen wir mit dem Connection Pool, denn er zerlegt die Skizze sofort. Ein Pool
vergibt Permits, die für Verbindungen stehen, und der Deal ist streng: Halte das
Permit genau so lange wie die Verbindung, dann gib es zurück. Die Skizze macht das
Zurückgeben zur Aufgabe des Aufrufers — ruf `release`, wenn du fertig bist. Und
jetzt zähle die Arten, auf die ein Aufrufer „fertig" ist: der glückliche Pfad, ja —
aber auch das early return bei einem ungültigen Request, das `?`, das einen Fehler
weiterreicht, die Panic, das Future, das auf halbem Weg gedroppt wird. Vergiss
`release` auf *irgendeinem* davon, und ein Verbindungs-Slot ist für immer weg. Ein
so betriebener Pool scheitert nicht laut; er schrumpft leise, ein vergessener Pfad
nach dem anderen, bis der Dienst auf einem Pool der Größe null wartet.

Das Problem ist nicht Schlamperei — es ist, dass die Skizze das Zurückgeben als
*Pflicht* kodiert, und Pflichten werden vergessen. Rust hat ein besseres Werkzeug:
Mach das Permit zu einem *Wert*, und mach das Zurückgeben zu dem, was der Wert
beim Verschwinden tut.

```rust
let permit = sem.acquire().await;   // permit: SemaphorePermit
run_expensive_thing().await;
// permit verlässt hier den Scope — automatisch zurückgegeben, auf jedem Pfad
```

`SemaphorePermit` gibt sich in seinem Destruktor selbst zurück. Der Fehlerpfad,
der Panic-Pfad, der Dropped-Future-Pfad — sie alle laufen Destruktoren, also geben
sie alle das Permit zurück. Es ist derselbe Zug, den `Mutex::lock` mit seinem
Guard macht, angewandt auf Kapazität statt Daten. Und sieh, was mit der
öffentlichen API passiert ist: **`release` steht nicht mehr drin.** Das Permit zu
droppen *ist* das Release. Eine Operation, die der Aufrufer nicht vergessen kann,
ist eine Operation, die er nicht falsch machen kann.

## Aber etwas wie release schleicht sich zurück

Wenn Permits sich selbst zurückgeben — braucht es je eine release-förmige Methode?
Einmal, für einen anderen Job. Teil 1 enthielt ein Tor, das geschlossen startet:
eine Semaphore mit *null* Permits, an der sich Wartende sammeln, bis irgendein
Ereignis den Fluss öffnet. Öffnen heißt, Permits herbeizuzaubern, die nie
entnommen wurden:

```rust
pub fn add_permits(&self, n: usize);
```

Die Unterscheidung lohnt Präzision, weil beide Ideen sonst unter dem Wort
„release" verschmieren. Ein Permit droppen *gibt Geliehenes zurück* — Routine,
automatisch, passiert ständig. `add_permits` *prägt neue Kapazität* — bewusst,
selten, verändert, was die Semaphore ist. Sie zu trennen heißt: Die
Routineoperation bleibt unmissbrauchbar, und die seltene sieht an der Aufrufstelle
angemessen ungewöhnlich aus.

## Zwei Arten zu scheitern, und die Ehrlichkeit, es zu sagen

Der Load-Shedding-Use-Case verlangte ein anderes acquire: Warte nicht, sag es mir
*jetzt*. Das ist `try_acquire`, und sein Rückgabetyp ist die Stelle, an der das
Design zu sprechen beginnt:

```rust
pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>, TryAcquireError>;

pub enum TryAcquireError {
    NoPermits,   // gerade beschäftigt — Routine; später erneut oder Last abwerfen
    Closed,      // heruntergefahren — endgültig; hör auf zu versuchen
}
```

Warum ein Enum und nicht schlicht `Option`? Weil die beiden Fehlschläge den
Aufrufer zu Gegenteiligem auffordern. „Gerade keine Permits" ist Dienstag — retry
später, Request abwerfen, Fallback nehmen. „Closed" heißt: Die Semaphore ist
endgültig weg, und eine Retry-Schleife würde ewig drehen. Ein Aufrufer, der die
beiden nicht unterscheiden kann, muss raten — und ein Load-Shedding-Pfad, der
falsch rät, macht aus dem Shutdown einen Retry-Sturm.

Womit die Frage im Raum steht, die dieses Enum eingeschmuggelt hat: closed? Woher
kommt das?

## Shutdown gehört zum Interface

Stell dir den Server aus Teil 1 beim Herunterfahren vor, während vierzig Tasks in
`acquire` geparkt sind. Die Arbeit, die ihnen Permits freigegeben hätte, wird
gerade abgebaut — diese Permits kommen nie. Ohne Hilfe warten vierzig Tasks ewig,
und aus „graceful shutdown" wird ein Hänger, den irgendwann jemand mit `kill -9`
löst.

Jemand muss der Semaphore sagen können: Es ist vorbei — alle raus.

```rust
pub fn close(&self);
```

`close` weckt jeden geparkten Wartenden mit einem Fehler statt einem Permit und
lässt jedes künftige `acquire` sofort scheitern. Eine Methode — aber ihre Existenz
strahlt rückwärts in den Typ von `acquire` selbst: Wenn Warten mit „die Semaphore
hat geschlossen" enden kann, hat `acquire` einen Fehlschlagsmodus, und die
Signatur muss ihn aussprechen:

```rust
pub struct AcquireError;   // bedeutet genau eines: closed

// acquires letztlicher Output:
Result<SemaphorePermit<'_>, AcquireError>
```

Beachte, was `AcquireError` *nicht* ist: Es hat keine `NoPermits`-Variante, und
das ist kein Versehen. `acquire` kann nicht an Permitmangel scheitern — auf
Permits zu warten ist sein ganzer Job. Das Einzige, was das Warten unglücklich
beenden kann, ist die Schließung. Also bekommen `acquire` und `try_acquire`
*verschiedene* Fehlertypen, von denen jeder exakt die Ausgänge auflistet, die
seine Methode produzieren kann — und wer auf einen von beiden matcht, wird nie
gebeten, einen unmöglichen Fall zu behandeln.

## Die Signatur, die sich drei Teile später auszahlt

Eine Entscheidung bleibt, und sie ist unsichtbar, bis man genau hinsieht. Die
natürliche Schreibweise eines asynchronen `acquire` wäre:

```rust
pub async fn acquire(&self) -> Result<SemaphorePermit<'_>, AcquireError>;
```

Ein `async fn` kompiliert zu einem Future-Typ, den der Compiler erfindet — anonym
und unantastbar. Man kann kein Trait für einen Typ implementieren, den man nicht
benennen kann, und gleich wird ein Trait enorm wichtig: `Drop`. Teil 1 hat
versprochen, dass ein wartender Task gecancelt werden kann — sein Future mitten im
Warten gedroppt — und Teil 5 wird zeigen, dass die Semaphore in diesem Moment
echte Aufräumarbeit hat. Aufräumen beim Drop heißt eigene `Drop`-Logik *auf dem
Future*. Also muss das Future ein Typ sein, der uns gehört:

```rust
pub fn acquire(&self) -> Acquire<'_>;    // ein benanntes Future…

pub struct Acquire<'a> { /* … */ }        // …für das wir Drop implementieren können
```

Für Aufrufer ändert sich nichts — `sem.acquire().await` liest sich exakt gleich.
Aber jetzt existiert ein Ort für das Cancellation-Aufräumen. Die Lektion trägt
über Semaphoren hinaus: Jedes Async-Primitiv, das auf das *Verschwinden* eines
Wartenden reagieren muss, braucht ein benanntes Future — und wer das erst nach dem
Ausliefern des `async fn` entdeckt, hat kein Refactoring vor sich, sondern einen
API-Bruch.

## Ein Permit, zwei Lebensdauern

Das Permit, wie entworfen, borgt die Semaphore — `SemaphorePermit<'a>` hält
`&'a Semaphore`, damit sein Destruktor weiß, wohin mit dem Permit. Für den Use
Case „diesen Abschnitt begrenzen" perfekt: null Overhead, scope-förmig. Aber gib
ein Permit an einen gespawnten Task, und die Borrow bricht — `tokio::spawn`
verlangt `'static`: Ein gespawnter Task kann die Funktion überleben, die ihn
erzeugt hat, also darf er keine Borrows ihrer Locals tragen.

Der Pool-Use-Case tut genau das — Permits reisen in gespawnten
Verbindungs-Handlern. Also wächst dem Interface eine Owned-Variante:

```rust
pub fn acquire_owned(self: Arc<Semaphore>) -> AcquireOwned;
// liefert OwnedSemaphorePermit — hält ein Arc, ist 'static, überquert spawn frei
```

Geborgt für scope-förmige Grenzen, owned für Permits, die ihren Scope überleben;
der Preis der Owned-Variante ist ein Refcount-Inkrement. Server-Code, der pro
Request spawnt, landet fast überall bei `acquire_owned`.

## Das zusammengesetzte Interface

```rust
impl Semaphore {
    pub fn new(permits: usize) -> Self;
    pub fn available_permits(&self) -> usize;

    pub fn acquire(&self) -> Acquire<'_>;
    pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>, TryAcquireError>;
    pub fn acquire_owned(self: Arc<Self>) -> AcquireOwned;

    pub fn add_permits(&self, n: usize);
    pub fn close(&self);
}

pub struct SemaphorePermit<'a>;                  // RAII: Drop = zurückgeben
pub struct AcquireError;                         // closed — acquires einziger Fehler
pub enum TryAcquireError { NoPermits, Closed }   // busy ist Routine, closed ist final
```

Jede Methode hat jetzt eine Papierspur zurück zu Teil 1. Aber sieh, was die
Signaturen *nicht* sagen: Nichts hier verspricht Fairness, und nichts verspricht,
dass Cancellation sicher ist. Diese beiden leben unterhalb des Interface, als
Eigenschaften der Maschinerie im Inneren — und beide hängen an einer Frage, die
das Interface höflich verweigert: Wenn `acquire` kein Permit zu vergeben hat und
der Aufrufer warten muss — *wo, physisch, findet dieses Warten statt?* Ein Thread
kann anhalten und warten. Ein Task ist kein Thread. Teil 3.

---

*Weiter: [Teil 3 — Wo lebt das Warten?](03_where_waiting_lives.md) · [Index](00_index.md)*

*English: [`../en/02_the_interface.md`](../en/02_the_interface.md)*
