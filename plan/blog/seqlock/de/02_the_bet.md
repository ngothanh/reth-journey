# Teil 2 — Die Wette: Lass es zerreißen und fang es ab

Teil 1 hat uns in die Enge getrieben. Jeder Lock zwingt entweder den Reader, gemeinsamen
Speicher zu schreiben, oder den Writer zu warten — und der einzige Kandidat, der keines
von beidem tut, per-field-Atomics, ist falsch. Die Randbedingungen verlangten einen
unsichtbaren Reader: einen, der nichts Gemeinsames schreibt, gegen einen Writer, der sich
verhält, als gäbe es keinen Reader.

Wenn der Writer nicht kooperiert, hält nichts einen Reader davon ab, einen halb
geschriebenen Wert zu sehen. Also hör auf, es verhindern zu wollen. **Lass den Writer an
Ort und Stelle überschreiben, lass den Read zerreißen, und gib dem Reader ein Mittel, es
hinterher zu bemerken und noch einmal zu lesen.** Der Reader ist read-only und kann seine
Arbeit gratis wiederholen (die dritte Asymmetrie aus Teil 1), ein vergeudeter Read kostet
also nichts als ein wenig Zeit. Das ist die Wette.

Es reduziert das ganze Problem auf eine einzige Frage:

> Woher weiß ein Reader im Nachhinein, dass er *während* eines Writes gelesen hat?

Alles Weitere folgt daraus, sie zu beantworten.

## Erster Versuch: ein „writing"-Flag

Der naheliegende Detektor ist ein Boolean, das der Writer setzt, während er arbeitet. Der
Reader wartet, bis es wieder frei ist, und liest dann:

![Ein einzelnes writing-Flag: der Reader kann trotzdem einen vollständig zerrissenen Wert lesen](../img/cards/bool_flag.png)

Spiel es durch, und es fällt auseinander. Der Reader prüft `writing`, sieht `false` und
beginnt zu lesen. *Dann* läuft ein Writer — Flag hoch, überschreiben, Flag runter —
vollständig innerhalb des Reads. Der Reader prüft nie erneut; er hat das Tor bereits
passiert. Er geht mit einem Wert davon, der halb alt, halb neu ist — und das Flag stand
`false`, in beiden Momenten, in denen es darauf ankam.

Das tiefere Problem ist nicht die fehlende zweite Prüfung. Es ist, dass **ein Boolean kein
Gedächtnis hat.** „Gerade schreibt niemand" und „jemand hat geschrieben, während du
weggeschaut hast" sind derselbe Wert — `false`. Ein Flag kann dir den aktuellen Zustand
nennen; es kann dir nicht sagen, ob sich der Zustand *geändert* hat, während du beschäftigt
warst. Und „hat er sich geändert, während ich beschäftigt war" ist genau die Frage.

## Was der Detektor wirklich braucht

Der Reader muss den Detektor also *zweimal* abtasten — einmal vor dem Kopieren, einmal
danach — und nur dann schließen „kein Write hat mich überlappt", wenn die beiden
Stichproben übereinstimmen. Damit dieser Vergleich überhaupt etwas bedeutet, muss der
Detektor eine Eigenschaft haben, die dem Boolean fehlt:

> Jedes Mal, wenn der Writer ihn berührt, muss er einen Wert annehmen, den er **noch nie
> zuvor hatte**.

Würde er nur umschalten, könnten zwei Stichproben zufällig übereinstimmen — der Writer
kippte ihn um und wieder zurück, während der Reader kopierte, und der Reader sieht an
beiden Enden denselben Wert und schließt fälschlich, es sei nichts geschehen. Ein Wert, der
sich nie wiederholt, schließt diesen Zufall aus. Der natürliche solche Wert ist ein Zähler,
der nur je aufwärts zählt.

Es gibt eine zweite Sache, die das zweimalige Abtasten übersieht. Den Zähler vorher und
nachher zu lesen fängt ein Write, das *während* des Kopierens *fertig* wurde — der Zähler
ist vorgerückt. Es fängt kein Write, das bereits *in Arbeit* war, als der Reader eintraf:
Der Zähler könnte die ganze Zeit unverändert auf demselben Wert stehen, während der Payload
doch durchweg unterwegs war. Der Zähler muss also zusätzlich in seinem Wert codieren „gerade
läuft ein Write", und der Reader muss sich weigern, überhaupt mit dem Kopieren zu beginnen,
wenn er das sieht.

Ein einziger Integer kann beide Signale zugleich tragen. Lass den Zähler **gerade sein,
wenn der Wert stabil ist, und ungerade, während ein Write läuft.** Ein einzelnes Inkrement
erledigt beide Aufgaben: Es kippt die Parität um (ungerade verkündet also „schreibe gerade")
und es erzeugt eine nie zuvor gesehene Zahl (zwei gleiche gerade Stichproben beweisen also
„dazwischen ist nichts geschehen"). Der Writer inkrementiert einmal beim Betreten — gerade
zu ungerade — und einmal beim Verlassen — ungerade zu gerade.

## Das Protokoll

![Das Ungerade/Gerade-Protokoll: der Writer klammert den Write mit zwei Inkrementen ein; der Reader tastet den Zähler vorher und nachher ab](../img/cards/protocol.png)

Lies es als Vertrag zwischen den beiden Seiten. Der Writer verspricht: Der Payload wird nur
je berührt, während der Zähler ungerade ist. Der Reader prüft zwei Dinge und vertraut seiner
Kopie nur, wenn beide gelten — der Zähler war **gerade**, als er begann (kein Write in
Arbeit), und es war der **gleiche** gerade Wert, als er fertig war (kein Write hat dazwischen
begonnen und geendet). Alles andere, und er geht in die Schleife zurück und versucht es
erneut.

Geh die beiden gefährlichen Verzahnungen durch und sieh, wie beide gefangen werden:

```
Reader startet, während ein Write in Arbeit ist:
  s1 = seq  →  ungerade  →  Reader kopiert gar nicht erst; er versucht es erneut.  ✓

ein Write beginnt und endet während des Kopierens:
  s1 = seq  →  8 (gerade, ok)
  payload kopieren …      ← Writer läuft hier: 8 → 9 → 10
  s2 = seq  →  10         → s1 ≠ s2 → erneut versuchen.               ✓
```

Beide Löcher mit einem einzigen Zähler geschlossen. Beachte, was der Reader nie tut: Er
schreibt niemals gemeinsamen Speicher. Zwei Loads, eine Kopie, zwei weitere Loads — alles
Reads. Der Writer wartet nie auf einen Reader — er inkrementiert und geht. Es gibt keinen
alten Wert zurückzugewinnen, denn der Writer hat nie einen neuen erzeugt; er hat an Ort und
Stelle überschrieben. Jede Randbedingung aus Teil 1 ist erfüllt, und wir sind dorthin
gelangt, indem wir das Zerreißen umarmt haben, statt es zu bekämpfen.

Du könntest hier aufhören und glauben, du seist fertig. Die Logik ist vollständig, und auf
dem Papier geht jeder Fall auf. Also hier der unangenehme Teil: Genau dieses Protokoll, auf
die naheliegende Weise geschrieben, **zerreißt trotzdem** — nicht weil die Logik falsch ist,
sondern weil die Maschine darunter deine Instruktionen nicht in der Reihenfolge ausführt, in
der du sie geschrieben hast. Das ist Teil 3, und dort wohnen die meisten echten
SeqLock-Bugs tatsächlich.

---

*Weiter: [Teil 3 — Das Memory-Ordering richtig hinbekommen](03_memory_ordering.md) · [Index](00_index.md)*

*English: [`../en/02_the_bet.md`](../en/02_the_bet.md)*
