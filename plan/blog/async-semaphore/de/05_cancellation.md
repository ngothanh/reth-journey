# Teil 5 — Cancellation: Wenn ein Wartender verschwindet

Ein Thread, der zu warten beginnt, wird früher oder später aufhören zu warten. Er
mag eine Mikrosekunde blockieren oder eine Stunde, aber das Betriebssystem
verspricht, dass er irgendwann aufwacht und seine nächste Zeile ausführt.
Thread-Code stützt sich überall auf dieses Versprechen, ohne es je zu benennen:
das Aufräumen, die Buchführung, das Nehmen des Permits — alles wohnt „nach dem
Warten", und nach dem Warten kommt immer.

Ein Async-Task verspricht nichts dergleichen. Ein wartender Task kann einfach
aufhören zu existieren — sein Future mitten im Warten gedroppt, nie wieder
gepollt. Und das ist kein exotischer Fehlerfall; es ist ein Feature im täglichen
Einsatz:

```rust
match timeout(Duration::from_millis(100), sem.acquire()).await {
    Ok(permit) => handle(permit).await,
    Err(_)     => return Response::too_busy(),  // acquires Future: gedroppt, mitten im Warten
}
```

Ein `timeout`, das aufgibt; ein `select!`, das seinen Verliererzweig fallen lässt;
ein abgebrochener Handler — jedes davon zerstört ein Future, das in unserer Queue
geparkt war und einen Platz in der Schlange hielt. Die Frage, die diesen ganzen
Teil ordnet: Wenn ein Wartender verschwindet — *was schuldete er, und wem?* Die
Antwort hängt davon ab, *wann* er verschwand.

## Verschwinden beim Warten

Zuerst der mildere Fall: gedroppt, während er eingereiht, aber noch nicht
zugeteilt war. Der Wartende besaß nichts — aber er hat etwas *zurückgelassen*:
seinen Record in der Queue.

Dieser Record darf nicht verwaisen, denn die Release-Seite vertraut der Queue:

```
Queue:  [ A ] → [ B✝ ] → [ C ]        Bs Future wurde gedroppt; sein Record blieb

nächstes Release:  pop_front… erreicht B✝ → granted = true, wake(Bs Waker)
                   → weckt einen Task, den es nicht mehr gibt
                   → das Permit liegt in einem Record, den nie wieder jemand liest
                   → Kapazität −1, lautlos
```

Ein Handler, der in einer Retry-Schleife in Timeouts läuft, kann einen Pool auf
diese Weise in Minuten leerbluten, ohne eine einzige Logzeile. Die erste Regel
schreibt sich also selbst — und das benannte Future aus Teil 2 ist es, was sie
schreibbar macht, denn das compiler-generierte Future eines `async fn` hat keinen
Ort dafür:

```rust
impl Drop for Acquire<'_> {
    fn drop(&mut self) {
        let mut state = self.semaphore.state.lock().unwrap();
        // ich habe noch gewartet → nimm meinen Record mit aus der Queue
        state.queue.remove(my_record);
    }
}
```

Steck ein Detail für Teil 6 ein: Der zu entfernende Record sitzt irgendwo in der
*Mitte* der Queue. Cancellation fragt nie höflich vorne an.

## Verschwinden nach der Zuteilung

Der schwere Fall existiert wegen der Lücke, die Teil 4 immer wieder umkreist hat:
Einen Task zu wecken heißt nicht, ihn auszuführen. Zwischen „dein Waker hat
gefeuert" und „ein Worker pollt dich" liegt ein Fenster, und Cancellation kann
mitten hinein schlagen:

```
t₀   Release:  As Record gepoppt, granted = true, wake(A)

t₁   A ist lauffähig — aber kein Worker hat ihn bisher aufgegriffen     ← das Fenster

t₂   As Timeout läuft zuerst ab.  As Future wird gedroppt.
```

Jetzt mach die Buchführung, langsam, denn sie ist der ganze Bug. Das Permit wurde
in As Record gelegt, also hat es den Zähler nie berührt. A lief nie wieder, also
wurde es nie konsumiert. Es ist nicht im Zähler; kein lebender Task hält es. Es
ist *nirgendwo* — und es kommt nie zurück.

Eine `Semaphore::new(4)`, der das einmal passiert, ist eine `Semaphore::new(3)`
mit altem Namen. Vier unglückliche Timeouts später lässt die Semaphore niemanden
mehr durch, für immer, ohne je einen Fehler zurückgegeben oder ein Wort geloggt zu
haben. In Produktion taucht das Wochen später auf, als „der Dienst nimmt unter
Last auf mysteriöse Weise keinen Traffic mehr an".

Die Regel ist, sobald die Form sichtbar ist, fast selbstverständlich: Ein beim
Zugeteiltsein gedroppter Wartender muss sein Permit über den normalen Release-Pfad
zurückgeben. `Drop` wächst um einen zweiten Zweig:

```rust
impl Drop for Acquire<'_> {
    fn drop(&mut self) {
        let mut state = self.semaphore.state.lock().unwrap();
        if my_record.granted {
            // ein Permit liegt auf meinen Namen, unkonsumiert → gib es weiter:
            // an den nächsten Wartenden, oder in den Zähler, wenn niemand wartet
            release_one(&mut state, &mut to_wake);
        } else {
            // ich habe noch gewartet → nimm meinen Record mit
            state.queue.remove(my_record);
        }
        // (oben gesammelte Waker werden nach dem Lock-Drop gerufen — Teil 3)
    }
}
```

Beachte: Der Granted-Zweig ruft dasselbe `release_one` wie `add_permits` — und
dieses Release kann das Permit einem zweiten Wartenden B zuteilen, der *ebenfalls*
im Weck-Lauf-Fenster sitzt, der *ebenfalls* gedroppt wird, dessen `Drop` erneut
freigibt, an C. Das Permit pflanzt sich durch die Schlange fort, bis es jemanden
erreicht, der tatsächlich läuft, oder die Schlange leer ist und es sich in den
Zähler legt. Jeder Sprung ist derselbe Zug — der sterbende Halter gibt zurück, was
er hält — die Kette braucht keine Sonderbehandlung.

## Der Bug auf der anderen Seite der Münze

Die zwei `Drop`-Zweige machen Leaks unmöglich. Sie erzeugen zugleich eine Falle
auf dem Erfolgspfad, zwei Zeilen groß. Hier ist `poll` beim Konsumieren eines
zugeteilten Permits — zuerst die kaputte Version:

```rust
// KAPUTTES Konsumieren:
if my_record.granted {
    return Poll::Ready(Ok(permit));
    //  das Future wird trotzdem irgendwann gedroppt — jedes Future wird das.
    //  Drop läuft, sieht granted == true… und gibt das Permit NOCHMAL frei.
}
```

Das Permit wird einmal konsumiert und zweimal zurückgegeben. Jetzt verliert die
Semaphore keine Kapazität — sie *prägt* welche: `new(4)` driftet Richtung
`new(5)`. Die Korrektur ist, die beglichene Schuld zu verbuchen:

```rust
// KORREKTES Konsumieren:
if my_record.granted {
    my_record.granted = false;              // ← die eine Zeile zwischen Leak und Prägung
    detach_from_queue_bookkeeping();
    return Poll::Ready(Ok(permit));
}
```

Leak und Prägung sind derselbe Buchungsfehler mit umgekehrtem Vorzeichen, und
beide laufen auf eine Frage zur Drop-Zeit hinaus: *Was besitzt dieser Wartende
jetzt gerade?*

## Drei Regeln, die eine sind

Leg alles nebeneinander, und das Muster schließt sich. Zu jedem Zeitpunkt besitzt
ein Wartender genau eines — erst einen Platz in der Schlange, dann ein Permit,
dann nichts:

```
 (erzeugt) ──enqueue──►  WAITING  ──grant──►  GRANTED  ──consume──►  DONE
                            │                    │
                      hier gedroppt        hier gedroppt
                            │                    │
                            ▼                    ▼
                  gib meinen PLATZ        gib mein PERMIT
                       zurück                 zurück
```

> **Beim Drop gibt ein Wartender zurück, was er in diesem Moment besitzt.** Die
> drei Regeln sind ein Satz, gelesen an drei Punkten einer Zeitleiste.

Das `granted: bool` aus Teil 4 beginnt zu knarzen — es ist in Wahrheit ein
Drei-Zustands-Wert (wartend / zugeteilt / fertig), gequetscht in einen Boolean
plus Kontext. Teil 7 wird es zu einem ehrlichen Enum befördern und jede Funktion
zu einem `match` darüber machen. Aber zuerst schuldet das Design eine letzte
Antwort.

## Die verbleibende Frage ist physisch

Durch diesen ganzen Teil hat „my_record" stille Arbeit geleistet: `Drop` erreicht
*seinen eigenen* Record in der Queue; Release erreicht den *vordersten*; beide
verändern ihn. Jeder Wartende braucht so einen Record — Waker, Zuteilungszustand,
ein Platz in der Reihe — und dieser Record muss an einer echten Adresse im
Speicher liegen. In der Semaphore? Im Future? Die Wahl entscheidet, ob `acquire`
allokiert, und die schnelle Antwort klingt beim ersten Hören, als dürfte sie gar
nicht legal sein. Teil 6.

---

*Weiter: [Teil 6 — Wo die Wartenden wohnen, und wofür Pin da ist](06_memory_and_pin.md) · [Index](00_index.md)*

*English: [`../en/05_cancellation.md`](../en/05_cancellation.md)*
