# Teil 4 — Fairness: Wer bekommt das freie Permit?

Ein Permit kommt zurück, und drei Tasks warten darauf. Jemand muss entscheiden,
wer weiterdarf — und jedes Semaphoren-Design entscheidet, auch die Designs, die
nie gemerkt haben, dass sie entscheiden. Der Code aus Teil 3 hat bereits
entschieden, ohne zu fragen. Dieser Teil macht die Entscheidung sichtbar, zeigt
ihren Preis und ersetzt sie.

## Die erste Antwort: Lasst sie rennen

Hier ist der Release-Pfad aus Teil 3, mit unterstrichener Entscheidung:

```rust
state.permits += n;                      // Permit landet im GEMEINSAMEN Zähler
match state.waiters.pop_front() {
    Some(waker) => to_wake.push(waker),  // wecke jemanden — versprich ihm nichts
    None => break,
}
```

Der geweckte Wartende durchläuft `poll` wie jeder andere: Zähler prüfen, Permit
nehmen, falls noch eines da ist. Dieses Design heißt **Barging**, und seine
Tugenden verdienen eine faire Anhörung. Der Zustand ist minimal — ein Zähler und
nackte Waker. Und es ist nachsichtig: Weil der Zähler die einzige Quelle der
Wahrheit ist, ist *schlampiges Wecken harmlos*. Weckt man zu viele, prüfen die
Überzähligen nach, finden nichts und reihen sich wieder ein — verschwenderisch,
nie falsch. Für Workloads ohne Latenzverpflichtungen ist das ein vertretbarer Ort
zum Aufhören.

## Was das Rennen kostet

Der Ärger versteckt sich in einer Lücke, die Teil 3 schon offengelegt hat: *Einen
Task zu wecken heißt nicht, ihn auszuführen.* `wake()` markiert den Task als
lauffähig; ein Worker greift ihn später auf. Zwischen diesen beiden Momenten liegt
das Permit im Zähler — für alle sichtbar. Sieh zu, was ein stetiger Strom von
Neuankömmlingen mit diesem Fenster anstellt:

```
Zähler = 0.  A ist in der Queue geparkt.

Release  →  Zähler = 1, wecke A
                 ▲
                 │    A ist lauffähig… aber läuft noch nicht
Neuling B:       │    poll sieht Zähler == 1  →  nimmt es.  Zähler = 0
A läuft endlich: └──  poll sieht Zähler == 0  →  A reiht sich wieder ein

Release  →  wecke A …  Neuling C schnappt es zuerst  …  A parkt wieder
Release  →  wecke A …  Neuling D schnappt es zuerst  …  A parkt wieder
```

Die Lücke ist strukturell — die Runtime *muss* A erst schedulen, und ein Neuling,
der bereits auf einer CPU sitzt, schlägt einen Wartenden, der es nicht tut, jedes
Mal. Unter stetigem Zustrom kann A dieses Rennen *unbegrenzt* verlieren. Die
Wartenden werden von der Schlange vorbeigedrängelt — daher der Name — und A wird
vom Design selbst ausgehungert.

Zwei Eigenschaften machen das gefährlich. Tests sehen es nicht: Jede einzelne
Operation ist korrekt, und As Verhungern ist eine statistische Eigenschaft von
Contention — „bekommt ein Wartender irgendwann ein Permit?" besteht in jeder
ruhigen Umgebung mit Bravour. Und die Produktion sieht es nur dort, wo man zuletzt
hinschaut: Nichts stürzt ab, p50 glänzt, und p99.9 ist ein Grauen, denn p99.9 ist
genau da, wo die Pechvögel wohnen. Für die Pool- und Latenzbudget-Use-Cases aus
Teil 1 ist „manche Aufrufer warten unter Last unbegrenzt" ein Ausfall mit guten
Manieren.

## Die zweite Antwort: Übergib es

Die Lösung invertiert eine Entscheidung: Wartet irgendwer, berührt ein
freigegebenes Permit **nie den gemeinsamen Zähler**. Es wandert direkt in die
Hände des vordersten Wartenden.

Damit das ausdrückbar wird, muss ein Queue-Eintrag wachsen: Ein nackter `Waker`
kann kein Permit halten. Jeder Wartende bekommt einen kleinen *Record*, geteilt
zwischen Queue und dem Future dieses Wartenden — die Queue erreicht ihn zum
Zuteilen, das Future liest ihn, um es zu erfahren:

```rust
struct Waiter {
    waker: Waker,
    granted: bool,     // ← „ein Permit liegt auf deinen Namen bereit"
}

struct State {
    permits: usize,
    queue: /* Waiter-Records in Ankunftsreihenfolge — physisches Zuhause: Teil 6 */,
}
```

Release wechselt von „Zähler erhöhen" zu „zuteilen, oder erhöhen, wenn niemand da
ist":

```rust
// Hand-off-Release: ein Permit
match state.queue.pop_front() {
    Some(waiter) => {
        waiter.granted = true;                  // das Permit wandert IN den Record…
        to_wake.push(waiter.waker.clone());     // …und sein Besitzer wird geweckt
    }
    None => state.permits += 1,                 // nur in den Zähler, wenn die
}                                               //   Queue leer ist
```

Und `poll` ändert sich für einen bereits eingereihten Wartenden auf eine Weise,
die leicht zu übersehen und essenziell ist:

```rust
// Re-Poll eines eingereihten Wartenden:
if my_record.granted {
    return Poll::Ready(Ok(permit));    // konsumiere, was bereits mir gehört
}
if !my_record.waker.will_wake(cx.waker()) {
    my_record.waker = cx.waker().clone();
}
Poll::Pending
// beachte, was FEHLT: kein `state.permits > 0`. Ein eingereihter Wartender
// liest den Zähler nie — sein Permit kommt über seinen Record oder gar nicht.
```

Jetzt spielt die Weck-Lauf-Lücke keine Rolle mehr. Wie lange A auch braucht, um
gescheduled zu werden — sein Permit wartet in seinem Record; im gemeinsamen Zähler
gibt es nichts, was ein Neuling stehlen könnte. Die Wartezeit ist durch die
Position in der Schlange begrenzt, Punkt.

Das ganze Design komprimiert sich in eine Invariante, die es wert ist,
ausgesprochen zu werden, weil die restlichen Teile auf ihr ruhen:

> **Wartet irgendwer, steht der Zähler auf null.**

Sie gilt automatisch — eingereihte Wartende fangen jedes freigegebene Permit ab,
bevor der Zähler es sieht — und sie erledigt die Durchsetzung nebenbei: Das `poll`
eines Neulings findet `permits == 0` und hat keine Wahl, als sich hinten
anzustellen. Selbst `try_acquire` wird gratis ehrlich: Solange Wartende da sind,
gibt es nichts zu greifen, also kann auch es sich nicht vordrängeln.

## Warum die fehlende Prüfung zählt

Das abwesende `state.permits > 0` im Re-Poll verdient einen eigenen Absatz, denn
es wieder einzubauen ist der naheliegende Fehler. Wann könnte ein eingereihter
Wartender überhaupt einen Zähler über null sehen? Wenn `add_permits` mehr Permits
prägt, als Wartende da sind — der Überschuss läuft in den Zähler über, während
zugeteilte Wartende noch aufwachen. Ein eingereihter Wartender, der sich am Zähler
bedient *und* später `granted == true` in seinem Record findet, hat zwei Permits
für ein Release genommen. Die Semaphore hat Kapazität aus dem Nichts geprägt, und
kein Test, der diese Verzahnung nicht gezielt nachstellt, wird es je bemerken.

Eine Regel also: Wer sich eingereiht hat, nimmt *nur* aus seinem Record.

## Wählen, mit offenen Augen

| | Barging | Hand-off |
|---|---|---|
| Zustand | Zähler + nackte Waker | Zähler + Records pro Wartendem, geordnet |
| Wecken | schlampig ist sicher | muss genau den Zugeteilten wecken |
| schlimmste Wartezeit | unbegrenzt unter Last | begrenzt durch Schlangenposition |
| Cancellation | trivial — Wartende besitzen nichts | schwer — ein zugeteiltes Permit kann verwaisen |
| passt zu | Best-Effort-Drosselung | Pools, Budgets, alles mit Latenz-SLA |

Beide Spalten sind korrekte Semaphoren; die Weggabelung ist echt. Diese Serie
nimmt Hand-off, aus demselben Grund wie tokio — die Use Cases, die unser Interface
geformt haben, sind genau die, die unbegrenzte Wartezeiten nicht schlucken können.
Die Warnung gilt nicht dem Wählen von Barging, sondern dem Wählen *aus Versehen* —
und genau das passiert, wann immer jemand eine Semaphore aus einem Zähler und
einer Condvar improvisiert und nie fragt, wer das freie Permit bekommt. Der p99.9
antwortet irgendwann.

## Die Zeile in der Tabelle mit Zähnen

Eine Zelle oben verdient einen zweiten Blick: Unter Hand-off wurde Cancellation
*schwer*. Ein Barging-Wartender besitzt nichts — verschwindet er mitten im Warten,
geht nichts verloren. Ein Hand-off-Wartender kann verschwinden, *während ein
Permit in seinem Record liegt* — `granted = true`, geweckt, und von einem
`timeout` gedroppt, einen Augenblick bevor ein Worker ihn gepollt hätte.

Was passiert mit diesem Permit? Nichts stürzt ab. Nichts loggt. Das ist das
Problem — und es ist Teil 5.

---

*Weiter: [Teil 5 — Cancellation: Wenn ein Wartender verschwindet](05_cancellation.md) · [Index](00_index.md)*

*English: [`../en/04_fairness.md`](../en/04_fairness.md)*
