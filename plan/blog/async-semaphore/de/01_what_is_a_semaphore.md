# Teil 1 — Was eine Semaphore ist, und wann man zu ihr greift

Irgendwo in jedem ausgelasteten Server steckt eine Zahl, die niemand absichtlich
gewählt hat: die größte Menge teurer Arbeit, die er gleichzeitig stemmen kann,
bevor er umfällt. Vielleicht ist es die Anzahl der Bildkonvertierungen, die
gleichzeitig in den Speicher passen, oder die Verbindungen, die die Datenbank
duldet, oder die In-Flight-Requests, die ein nachgelagerter Dienst akzeptiert,
bevor er in Timeouts läuft. Der Server hat eine Decke. Der Ärger beginnt, wenn
nichts im Code von dieser Decke weiß.

Stell dir einen Endpoint vor, der etwas wirklich Teures tut — ein PDF rendert,
eine schwere Query ausführt — und dabei vielleicht 50 MB und einen CPU-Schub
kostet. Bei zehn Requests in flight ist das ein halbes Gigabyte; kein Problem.
Dann wirst du irgendwo verlinkt, oder ein Client retried in einer Schleife, und
zehntausend kommen in wenigen Sekunden. Sieh dir an, was eine Async-Runtime damit
macht: Sie spawnt zehntausend Tasks und startet alle. Es gibt keinen natürlichen
Gegendruck — nichts in `spawn` sagt „Moment mal." Die Maschine wird nicht anmutig
langsamer. Sie greift nach 500 GB, die sie nicht hat, der Kernel erschießt sie,
und die billigen Endpoints sterben mit dem teuren.

Die Lösung ist kein schnellerer Endpoint. Es ist eine Grenze: *höchstens N davon
laufen gleichzeitig, der Rest wartet, bis er dran ist.* Das Ding, das diese Grenze
durchsetzt, ist eine Semaphore, und diese Serie entwirft eine — eine echte, fair
und cancellation-sicher — für async Rust. Wir beginnen hier, bei ihrem Zweck, denn
jede Designentscheidung der späteren Teile wird von etwas auf dieser Seite
erzwungen.

## Der Türsteher

Das klarste Bild einer Semaphore ist ein Türsteher vor einem Club mit strenger
Brandschutzauflage. Der Raum fasst genau N Leute. Der Türsteher hat N Armbänder.
Du willst rein, du nimmst ein Armband; sind keine mehr da, wartest du draußen;
verlässt jemand den Club und gibt sein Armband zurück, reicht der Türsteher es an
den Nächsten in der Schlange. Der Türsteher zählt nie Köpfe im Raum — die
Armbänder *sind* die Zählung. Sind sie weg, ist der Raum voll, und das ist der
gesamte Durchsetzungsmechanismus.

Übersetzt man die Geschichte zurück, hat man die Definition:

> Eine Semaphore ist ein Zähler von *Permits*, auf den man warten kann. Eines
> nehmen ist `acquire`; es zurückgeben ist Release. Sind keine übrig, schlägt
> `acquire` nicht fehl — es wartet.

Der Zähler-Teil ist trivial; zählen kann jeder Integer. Das Warten ist das ganze
Problem, und es ist der Grund, warum man das nicht aus einem `AtomicUsize` und
einer Subtraktion bauen kann. Ein Atomic kann bis null herunterzählen, aber es hat
keine Ahnung, wie es einen Aufrufer bei null *warten* lässt, keine Ahnung, wie es
ihn weckt, wenn ein Permit zurückkommt, und — wie wir drei Teile später sehen
werden — keine Ahnung, was zu tun ist, wenn ein Wartender aufgibt und geht. Alles
Schwere an einer Semaphore versteckt sich im Wort „warten".

Eine Klarstellung lohnt sich früh, weil die beiden ständig verwechselt werden:
Eine Semaphore ist keine Mutex. Eine Mutex beantwortet *wem gehört das?* — genau
ein Inhaber, der ein Stück Daten davor schützt, von zwei Threads gleichzeitig
angefasst zu werden. Eine Semaphore beantwortet *wie viele gleichzeitig?* — bis zu
N Inhaber, die nichts schützen, sondern Kapazität rationieren. Eine Mutex ist
letztlich eine Semaphore mit zufällig einem Permit — aber wer zu einer
Ein-Permit-Semaphore greift, um ein Feld zu schützen, wollte eine Mutex; und wer
zu einer Mutex greift, um „nur vier davon gleichzeitig" auszudrücken, ist dabei,
eine Semaphore von Hand zu bauen und dabei subtil zu scheitern.

## Dieselbe Form, viermal

Was eine Semaphore eine ganze Serie wert macht, ist nicht die
Türsteher-Geschichte — sondern dass vier scheinbar unverwandte Probleme sich als
die Türsteher-Geschichte in verschiedenen Kostümen entpuppen. Und jedes verlangt
still etwas Bestimmtes vom Design; so kommen wir in Teil 2 zu einem Interface, das
nicht geraten ist.

Das erste ist das vom Anfang: **begrenzte Nebenläufigkeit**, eine Obergrenze für
gleichzeitige schwere Operationen. In der Praxis laufen oft zwei Semaphoren
nebeneinander — eine großzügige Grenze für normale Arbeit und eine engere für die
schwerste Sorte, weil ein Trace oder ein großer Export mehr Speicher kostet als
ein normaler Request und eine kleinere Decke verdient. Schon hier muss das Design
also billig genug sein, dass mehrere Instanzen keine Überlegung wert sind.

Das zweite ist **Load Shedding**. Unter einem Burst will man nicht immer auf ein
Permit warten — manchmal ist die richtige Antwort, sofort aufzugeben und „zu
beschäftigt" zurückzumelden, damit der Aufrufer anderswo sein Glück versucht. Das
ist nicht das blockierende `acquire`; es ist eine andere Operation — „gib mir ein
Permit nur, wenn *jetzt gerade* eines frei ist" — und das Design muss sie
anbieten.

Das dritte ist ein **Connection Pool**, und er lehrt die schärfste Lektion. Hier
repräsentiert das Permit nicht bloß eine Verbindung — es *ist* praktisch die
Verbindung: Man hält es genau so lange wie die Verbindung, und danach geht es
zurück. Aber „danach" schließt die hässlichen Pfade ein — einen Fehler, eine
Panic, ein Future, das auf halbem Weg gedroppt wird. Wenn das Zurückgeben etwas
ist, woran der Programmierer denken muss, dann ist jedes early return, das du je
schreibst, ein geleaktes Permit in Wartestellung, und der Pool blutet leer. Das
Permit muss sich *selbst* zurückgeben.

Das vierte erkennen die meisten erst, wenn man es ihnen zeigt: ein **begrenzter
Channel** — eine Queue, die den Produzenten warten lässt, wenn sie voll ist — ist
innen eine Semaphore. Kapazität N ist N Permits; `send` ist `acquire`; ein
Konsument, der ein Element entnimmt, ist Release. Jede Pipeline mit Backpressure,
die du je gebaut hast, hat eine Semaphore in sich versteckt. Und weil `send` auf
dem Hot Path von allem sitzt, verlangt dieser Use Case, dass `acquire` `async`
und wirklich billig ist.

Legt man die vier nebeneinander, schreibt sich das Interface fast von selbst: ein
Zähler, auf den man wartet; ein nicht-blockierender Versuch; ein Permit, das sich
selbst zurückgibt; ein `acquire`, das async und leicht ist.

## Die zwei Dinge, die die Use Cases nur flüstern

Es gibt zwei weitere Anforderungen, und sie sind der Grund, warum diese Serie
sieben Teile hat statt zwei: Ein ruhiger Test löst keine von beiden aus, die
Produktion beide, ständig.

Die erste ist **Fairness**. Zurück zum Türsteher, voller Raum, wartende Menge —
und stell ihn dir ein wenig nachlässig vor: Jedes Mal, wenn ein Armband
zurückkommt, gibt er es nicht dem, der am längsten wartet, sondern dem, der
zufällig am nächsten steht — und das ist an einer belebten Tür immer irgendein
Neuankömmling. Ein Pechvogel kann die ganze Nacht dort stehen, während Leute, die
nach ihm kamen, durchgewunken werden. Eine Semaphore kann genau das tun, und unter
stetiger Last kann ein Wartender unbegrenzt überholt werden. Nichts stürzt ab.
Dein p50 sieht großartig aus. Dein p99.9 ist eine Katastrophe in Zeitlupe, denn er
misst die Leute, die an der Tür feststecken. Für alles mit einem Latenzbudget ist
„manche Aufrufer warten unbegrenzt" kein Ausreißer, sondern ein Ausfall — Fairness
muss ins Design, und Teil 4 handelt davon, was sie kostet.

Die zweite ist **Cancellation**, und sie hat kein Gegenstück im gewöhnlichen
Thread-Code. Ein Thread, der wartet, wird irgendwann aufhören zu warten und seine
nächste Zeile ausführen — das verspricht das OS. Ein Async-Task verspricht nichts
dergleichen: Ein wartender Task kann einfach aufhören zu existieren, sein Future
mitten im Warten gedroppt, weil ein `timeout` gefeuert oder ein `select!` den
anderen Zweig genommen hat. Meistens ist das genau, was man will. Aber für die
Semaphore, die den Platz dieses Wartenden in der Schlange hält — womöglich mit
einem bereits für ihn reservierten Permit — ist ein Wartender, der sich in Luft
auflösen kann, die Quelle des härtesten Bugs im ganzen Ding. In Teil 5 stellen wir
ihn.

## Was wir in der Hand halten

Noch bevor eine Zeile Implementierung existiert, haben die Use Cases uns das
vollständige Briefing diktiert:

```
ein Budget zählen                  new(n)
auf ein freies Permit warten       acquire().await
ein Permit automatisch zurückgeben ein RAII-Permit-Wert
sich weigern zu warten             try_acquire()
herunterfahren ohne zu hängen      close()
nie einen Wartenden verhungern     Fairness            (Teil 4)
das Verschwinden überleben         cancellation-sicher (Teil 5)
```

Im Code sieht das Ziel so aus:

```rust
static PDF_JOBS: Semaphore = Semaphore::new(10);

async fn render_endpoint(req: Request) -> Response {
    let _permit = PDF_JOBS.acquire().await?;  // wartet, wenn schon 10 laufen
    render_pdf(req).await                     // Permit wird währenddessen gehalten
}                                             // Permit hier gedroppt → Nächster wacht auf
```

Das ist das Design, formuliert als Bedürfnisse statt als Code. In Teil 2 wird aus
jeder Zeile eine echte Rust-Signatur — und wir entdecken dabei, dass sich ein paar
dieser Anforderungen stillschweigend widersprechen, und das Interface entscheiden
muss, wer gewinnt.

---

*Weiter: [Teil 2 — Das Interface, von den Use Cases abgelesen](02_the_interface.md) · [Index](00_index.md)*

*English: [`../en/01_what_is_a_semaphore.md`](../en/01_what_is_a_semaphore.md)*
