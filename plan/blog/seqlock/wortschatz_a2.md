# SeqLock-Wortschatz — Deutsch → Englisch

Jedes A2+ Wort aus der deutschen Serie, mit englischer Bedeutung und einem Beispielsatz aus dem Text — **diesmal mit englischer Übersetzung des Beispiels darunter**. 535 Einträge; T1–T4 = Teil des ersten Vorkommens.

## Substantive — nouns (112)

**die Randbedingung** · -en · `T0` — constraint, boundary condition
> DE: Wir beginnen bei dem Problem, für dessen Lösung ein SeqLock existiert, sehen jedem Lock, zu dem man normalerweise greifen würde, dabei zu, wie es an genau einer der **Randbedingungen** scheitert, und gehen dann die Wette ein, die das ganze Primitive definiert: Statt den Leser daran zu hindern, einen halb geschriebenen Wert zu beobachten, lassen wir es geschehen und bringen den Leser dazu, es zu erkennen.
> EN: We start from the problem that a SeqLock exists to solve, watch each lock you would normally reach for fail at exactly one of the **constraints**, and then make the bet that defines the whole primitive: Instead of preventing the reader from observing a half-written value, we let it happen and get the reader to detect it.

**die Maschinerie** · -n · `T0` — machinery — here, the technical apparatus
> DE: Die **Maschinerie** — `Relaxed`/`Acquire`/`Release`, fences, `Pod`, Miri, loom — wird in dem Moment eingeführt, in dem das Design mit ihr kollidiert.
> EN: The **machinery** — `Relaxed`/`Acquire`/`Release`, fences, `Pod`, Miri, loom — is introduced the moment the design collides with it.

**der Schreiber** · - · `T0` — writer — the thread that updates the value
> DE: Ein **Schreiber**, viele Leser und ein Wert aus mehreren Feldern, der nicht in ein einzelnes Maschinenwort passt — es gibt also immer einen Augenblick, in dem der Speicher halb alt, halb neu ist, und ein Leser, der genau dort landet, bekommt einen Wert, den es nie gab.
> EN: One **writer**, many readers, and a value made of several fields that does not fit into a single machine word — so there is always a moment in which memory is half old, half new, and a reader who lands exactly there gets a value that never existed.

**der Augenblick** · -e · `T0` — instant, moment
> DE: Ein Schreiber, viele Leser und ein Wert aus mehreren Feldern, der nicht in ein einzelnes Maschinenwort passt — es gibt also immer einen **Augenblick**, in dem der Speicher halb alt, halb neu ist, und ein Leser, der genau dort landet, bekommt einen Wert, den es nie gab.
> EN: One writer, many readers, and a value made of several fields that does not fit into a single machine word — so there is always a **moment** in which memory is half old, half new, and a reader who lands exactly there gets a value that never existed.

**das Maschinenwort** · ¨-er · `T0` — machine word — the CPU's native register/word size
> DE: Ein Schreiber, viele Leser und ein Wert aus mehreren Feldern, der nicht in ein einzelnes **Maschinenwort** passt — es gibt also immer einen Augenblick, in dem der Speicher halb alt, halb neu ist, und ein Leser, der genau dort landet, bekommt einen Wert, den es nie gab.
> EN: One writer, many readers, and a value made of several fields that does not fit into a single **machine word** — so there is always a moment in which memory is half old, half new, and a reader who lands exactly there gets a value that never existed.

**der Fehlschlag** · ¨-e · `T0` — failure, setback
> DE: Die drei Randbedingungen, die das schwer machen, und dann die Tour der **Fehlschläge**: Ein `RwLock` lässt Leser gemeinsam hinein, zwingt aber jeden von ihnen, einen gemeinsamen Zähler zu *schreiben*, und serialisiert damit Cores, die gar nicht in Konflikt stehen; `ArcSwap` und RCU umgehen das Zerreißen, ziehen den Leser aber zurück hinein, sich selbst zur Reclamation anzumelden.
> EN: The three constraints that make this hard, and then the tour of **failures**: An `RwLock` lets readers in together, but forces each of them to *write* a shared counter, and thereby serializes cores that are not in conflict at all; `ArcSwap` and RCU sidestep the tearing, but pull the reader back in to register itself for reclamation.

**der Zähler** · - · `T0` — counter
> DE: Die drei Randbedingungen, die das schwer machen, und dann die Tour der Fehlschläge: Ein `RwLock` lässt Leser gemeinsam hinein, zwingt aber jeden von ihnen, einen gemeinsamen **Zähler** zu *schreiben*, und serialisiert damit Cores, die gar nicht in Konflikt stehen; `ArcSwap` und RCU umgehen das Zerreißen, ziehen den Leser aber zurück hinein, sich selbst zur Reclamation anzumelden.
> EN: The three constraints that make this hard, and then the tour of failures: An `RwLock` lets readers in together, but forces each of them to *write* a shared **counter**, and thereby serializes cores that are not in conflict at all; `ArcSwap` and RCU sidestep the tearing, but pull the reader back in to register itself for reclamation.

**der Ausweg** · -e · `T0` — way out, escape
> DE: Jede korrekte Option bricht dieselbe Regel, und sie weist auf den einzigen **Ausweg** — der Leser muss unsichtbar sein.
> EN: Every correct option breaks the same rule, and it points to the only **way out** — the reader must be invisible.

**der Schreibvorgang** · ¨-e · `T0` — write operation
> DE: Das reduziert alles auf eine einzige Frage — woher weiß ein Leser, dass er während eines **Schreibvorgangs** gelesen hat? — und wir leiten die Antwort auf die harte Tour her, indem wir einem booleschen Flag beim Scheitern zusehen, weil es keine Geschichte trägt, bis das Einzige übrig bleibt, was funktioniert: ein Zähler, der gerade ist, wenn der Wert stabil ist, und ungerade, während er geschrieben wird, vor und nach dem Lesen abgetastet.
> EN: This reduces everything to a single question — how does a reader know it read during a **write**? — and we derive the answer the hard way, by watching a boolean flag fail, because it carries no history, until the only thing left that works remains: a counter that is even when the value is stable and odd while it is being written, sampled before and after the read.

**der Fingerabdruck** · ¨-e · `T0` — fingerprint — a tell-tale signature
> DE: Das Protokoll ist auf dem Papier korrekt und zerreißt trotzdem auf einem echten Apple M2, vier von fünf Läufen grün — der **Fingerabdruck** eines Memory-Ordering-Bugs.
> EN: The protocol is correct on paper and still tears on a real Apple M2, four out of five runs green — the **fingerprint** of a memory-ordering bug.

**der Lauf** · ¨-e · `T0` — run — a single execution of the test/program
> DE: Das Protokoll ist auf dem Papier korrekt und zerreißt trotzdem auf einem echten Apple M2, vier von fünf **Läufen** grün — der Fingerabdruck eines Memory-Ordering-Bugs.
> EN: The protocol is correct on paper and still tears on a real Apple M2, four out of five **runs** green — the fingerprint of a memory-ordering bug.

**der Zugriff** · -e · `T0` — access (to memory)
> DE: Die Korrektur macht jeden **Zugriff** auf die Payload atomar, Wort für Wort, und verwandelt „Müll lesen" von UB in einen legalen Read, den der Zähler wegwirft — was die Payload zwingt, `Pod` zu sein, eine Schranke, die sich als Lizenz erweist, die der Implementierer *unterschreibt*, statt einer, die der Compiler prüft, und die nicht einmal Alignment abdeckt.
> EN: The fix makes every **access** to the payload atomic, word by word, and turns "reading garbage" from UB into a legal read that the counter throws away — which forces the payload to be `Pod`, a bound that turns out to be a license the implementer *signs*, rather than one the compiler checks, and that does not even cover alignment.

**die Schranke** · -n · `T0` — bound, barrier, constraint
> DE: Die Korrektur macht jeden Zugriff auf die Payload atomar, Wort für Wort, und verwandelt „Müll lesen" von UB in einen legalen Read, den der Zähler wegwirft — was die Payload zwingt, `Pod` zu sein, eine **Schranke**, die sich als Lizenz erweist, die der Implementierer *unterschreibt*, statt einer, die der Compiler prüft, und die nicht einmal Alignment abdeckt.
> EN: The fix makes every access to the payload atomic, word by word, and turns "reading garbage" from UB into a legal read that the counter throws away — which forces the payload to be `Pod`, a **bound** that turns out to be a license the implementer *signs*, rather than one the compiler checks, and that does not even cover alignment.

**die Verschränkung** · -en · `T0` — interleaving (of threads)
> DE: Dann bekommt der Sequenzzähler einen zweiten Job als Lock der Schreiber, und dazu die Vertrauensfrage: der Test, der *scheitern* muss, Miri für das Race, loom für die **Verschränkungen**, und ein Benchmark, der — in Nanosekunden — zeigt, wie ein Lesepfad flach bleibt, während ein `RwLock` 450× langsamer wird.
> EN: Then the sequence counter gets a second job as the writers' lock, and with it the question of trust: the test that has to *fail*, Miri for the race, loom for the **interleavings**, and a benchmark that — in nanoseconds — shows how a read path stays flat while an `RwLock` becomes 450× slower.

**der Sequenzzähler** · - · `T0` — sequence counter — the seq that guards the payload
> DE: Dann bekommt der **Sequenzzähler** einen zweiten Job als Lock der Schreiber, und dazu die Vertrauensfrage: der Test, der *scheitern* muss, Miri für das Race, loom für die Verschränkungen, und ein Benchmark, der — in Nanosekunden — zeigt, wie ein Lesepfad flach bleibt, während ein `RwLock` 450× langsamer wird.
> EN: Then the **sequence counter** gets a second job as the writers' lock, and with it the question of trust: the test that has to *fail*, Miri for the race, loom for the interleavings, and a benchmark that — in nanoseconds — shows how a read path stays flat while an `RwLock` becomes 450× slower.

**die Ganzzahl** · -en · `T0` — integer (whole number)
> DE: **Sequenzzähler / seq** — die **Ganzzahl**, die der Schreiber um jeden Schreibvorgang herum hochzählt; gerade = stabil, ungerade = ein Schreibvorgang läuft.
> EN: **Sequence counter / seq** — the **integer** that the writer increments around every write; even = stable, odd = a write is in progress.

**der Handelsplatz** · ¨-e · `T1` — trading venue, marketplace
> DE: Ein langsamer Read dort ist ein langsamer Tail für den ganzen **Handelsplatz**.
> EN: A slow read there is a slow tail for the whole **trading venue**.

**der Wert** · -e · `T1` — value — the data value being stored/read
> DE: Ein Reader, der in diesem Fenster landet, bekommt keinen veralteten **Wert**.
> EN: A reader that lands in this window does not get a stale **value**.

**das Register** · - · `T1` — register — a CPU register
> DE: Der Wert passt nicht in ein **Register**
> EN: The value does not fit in a **register**

**die Instruktion** · -en · `T1` — instruction — a CPU/machine instruction
> DE: Es gibt keine CPU-**Instruktion**, die 40 Byte unteilbar schreibt.
> EN: There is no CPU **instruction** that writes 40 bytes indivisibly.

**das Fenster** · - · `T1` — window — here a brief interval of time, not a physical window
> DE: Das heißt, es gibt *immer* ein **Fenster** — wie kurz auch immer —, in dem der Speicher einen Wert hält, der halb der alte Head und halb der neue ist.
> EN: That means there is *always* a **window** — however brief — in which memory holds a value that is half the old head and half the new.

**der Speicher** · - · `T1` — memory, storage
> DE: Das heißt, es gibt *immer* ein Fenster — wie kurz auch immer —, in dem der **Speicher** einen Wert hält, der halb der alte Head und halb der neue ist.
> EN: That means there is *always* a window — however brief — in which **memory** holds a value that is half the old head and half the new.

**das Feld** · -er · `T1` — field — a struct field (not an agricultural field)
> DE: Mach jedes **Feld** zu seinem eigenen Atomic.
> EN: Make each **field** its own atomic.

**die Korrektheit** · kein Pl. · `T1` — correctness
> DE: Wäre **Korrektheit** die einzige Anforderung, wäre dies ein gelöstes und langweiliges Problem — ein Lock drumherum, und Feierabend.
> EN: Were **correctness** the only requirement, this would be a solved and boring problem — a lock around it, and done for the day.

**die Anforderung** · -en · `T1` — requirement
> DE: Wäre Korrektheit die einzige **Anforderung**, wäre dies ein gelöstes und langweiliges Problem — ein Lock drumherum, und Feierabend.
> EN: Were correctness the only **requirement**, this would be a solved and boring problem — a lock around it, and done for the day.

**der Pfad** · -e · `T1` — path — here the code/execution path
> DE: Der Engine-Thread, der den Chain-Head vorrückt, ist der kritische **Pfad** des Nodes; der Oracle-Thread, der den Mark-Price aktualisiert, ebenso.
> EN: The engine thread that advances the chain head is the node's critical **path**; the oracle thread that updates the mark price, likewise.

**der Konflikt** · -e · `T1` — conflict
> DE: Sie haben keinen logischen **Konflikt** — Lesen ist teilbar —, also ist jeder Aufwand, der *nur deshalb entsteht, weil es andere Reader gibt*, reine Verschwendung.
> EN: They have no logical **conflict** — reading is shareable — so any cost that arises *only because other readers exist* is pure waste.

**der Aufwand** · kein Pl. · `T1` — effort, cost, overhead
> DE: Sie haben keinen logischen Konflikt — Lesen ist teilbar —, also ist jeder **Aufwand**, der *nur deshalb entsteht, weil es andere Reader gibt*, reine Verschwendung.
> EN: They have no logical conflict — reading is shareable — so any **cost** that arises *only because other readers exist* is pure waste.

**die Verschwendung** · kein Pl. · `T1` — waste
> DE: Sie haben keinen logischen Konflikt — Lesen ist teilbar —, also ist jeder Aufwand, der *nur deshalb entsteht, weil es andere Reader gibt*, reine **Verschwendung**.
> EN: They have no logical conflict — reading is shareable — so any cost that arises *only because other readers exist* is pure **waste**.

**das Latenzbudget** · -s · `T1` — latency budget — the allowed time cap
> DE: Auf der Exchange lebt er in einem **Latenzbudget** pro Order, gemessen in Mikrosekunden.
> EN: On the exchange it lives within a per-order **latency budget**, measured in microseconds.

**die Obergrenze** · -n · `T1` — upper bound, cap
> DE: Er kann nicht für eine Heap-Allocation pausieren, und er kann nicht ohne **Obergrenze** erneut versuchen.
> EN: It cannot pause for a heap allocation, and it cannot retry without an **upper bound**.

**der Kandidat** · -en · `T1` — candidate — here a candidate solution
> DE: Halte diese drei gegen jeden **Kandidaten** weiter unten; jeder erfüllt die Korrektheit und bricht eines davon.
> EN: Hold these three against every **candidate** below; each satisfies correctness and breaks one of them.

**die Asymmetrie** · -n · `T1` — asymmetry
> DE: Die **Asymmetrie**, die die Mutex-Sichtweise übersieht
> EN: The **asymmetry** that the mutex view overlooks

**die Sichtweise** · -n · `T1` — viewpoint, framing, perspective
> DE: Die Asymmetrie, die die Mutex-**Sichtweise** übersieht
> EN: The asymmetry that the mutex **view** overlooks

**die Beobachtung** · -en · `T1` — observation
> DE: Hier ist die **Beobachtung**, um die sich das ganze Design dreht, und sie ist der Grund, warum dies *nicht* das klassische Mutual-Exclusion-Problem ist.
> EN: Here is the **observation** the whole design turns on, and it is the reason this is *not* the classic mutual-exclusion problem.

**die Partei** · -en · `T1` — party — one participant/actor
> DE: Hier aber modifiziert nur eine **Partei**.
> EN: Here, though, only one **party** modifies.

**die Größenordnung** · -en · `T1` — order of magnitude — 'um Größenordnungen' = by orders of magnitude
> DE: Reads übertreffen Writes um **Größenordnungen**.
> EN: Reads outnumber writes by **orders of magnitude**.

**der Seiteneffekt** · -e · `T1` — side effect
> DE: Kommt ein Read verstümmelt heraus, kostet nochmaliges Lesen nichts — es gibt keinen **Seiteneffekt** zurückzurollen.
> EN: If a read comes out garbled, reading again costs nothing — there is no **side effect** to roll back.

**die Garantie** · -n · `T1` — guarantee
> DE: Ein Mutex bezahlt für eine stärkere **Garantie**, als wir brauchen: Er gewährt *exklusiven Besitz*, um den der Reader hier nie gebeten hat.
> EN: A mutex pays for a stronger **guarantee** than we need: it grants *exclusive ownership*, which the reader here never asked for.

**der Besitz** · kein Pl. · `T1` — possession, ownership
> DE: Ein Mutex bezahlt für eine stärkere Garantie, als wir brauchen: Er gewährt *exklusiven **Besitz***, um den der Reader hier nie gebeten hat.
> EN: A mutex pays for a stronger guarantee than we need: it grants *exclusive **ownership***, which the reader here never asked for.

**die Währung** · -en · `T1` — currency
> DE: Und der Reader bezahlt diese Garantie in der einen **Währung**, die wir uns nicht leisten können — er muss in gemeinsamen Speicher schreiben, um den Lock zu nehmen.
> EN: And the reader pays for that guarantee in the one **currency** we cannot afford — it has to write to shared memory to take the lock.

**der Einwand** · ¨-e · `T1` — objection
> DE: Der naheliegende **Einwand**: Ein Read-Write-Lock *ist* für viele Reader gebaut.
> EN: The obvious **objection**: a read-write lock *is* built for many readers.

**das Versprechen** · - · `T1` — promise
> DE: Nein — denn „lässt sie gemeinsam rein" ist ein **Versprechen**, das das *Interface* gibt und das die *Implementierung* nicht umsonst halten kann.
> EN: No — because "lets them in together" is a **promise** the *interface* makes and one the *implementation* cannot keep for free.

**die Implementierung** · -en · `T1` — implementation
> DE: Nein — denn „lässt sie gemeinsam rein" ist ein Versprechen, das das *Interface* gibt und das die **Implementierung** nicht umsonst halten kann.
> EN: No — because "lets them in together" is a promise the *interface* makes and one the **implementation** cannot keep for free.

**das Eintreten** · kein Pl. · `T1` — entering — 'beim Eintreten' = on the way in (nominalized eintreten)
> DE: Das zu wissen heißt: Jeder Reader erhöht beim **Eintreten** einen gemeinsamen Zähler und verringert ihn beim Verlassen:
> EN: To know that means: every reader increments a shared counter on **entering** and decrements it on leaving:

**das Verlassen** · kein Pl. · `T1` — leaving — 'beim Verlassen' = on the way out (nominalized verlassen)
> DE: Das zu wissen heißt: Jeder Reader erhöht beim Eintreten einen gemeinsamen Zähler und verringert ihn beim **Verlassen**:
> EN: To know that means: every reader increments a shared counter on entering and decrements it on **leaving**:

**die Metadaten** · nur Pl. · `T1` — metadata
> DE: Lesen soll teilbar sein, und hier ist es *alles andere als das* — die Reader serialisieren sich auf **Metadaten**, die der Lock nur braucht, um zu existieren.
> EN: Reading is supposed to be shareable, and here it is *anything but that* — the readers serialize on **metadata** the lock needs only in order to exist.

**die Mischung** · -en · `T1` — mix, mixture
> DE: Ein Pointer ist 8 Byte, also *ist* das Umkippen atomar; ein Reader sieht entweder den ganzen alten oder den ganzen neuen Wert, nie eine **Mischung**.
> EN: A pointer is 8 bytes, so the flip *is* atomic; a reader sees either the whole old or the whole new value, never a **mixture**.

**das Werkzeug** · -e · `T1` — tool
> DE: Genau das tun `ArcSwap` und RCU, und für große oder pointer-reiche Werte ist es das richtige **Werkzeug**.
> EN: That is exactly what `ArcSwap` and RCU do, and for large or pointer-rich values it is the right **tool**.

**die Anwesenheit** · kein Pl. · `T1` — presence
> DE: Der Writer muss wissen, ob irgendein Reader noch den alten Pointer hält — was heißt, der Reader muss erneut *seine **Anwesenheit** ankündigen* (ein Reference Count, eine Epoche, ein Hazard Pointer).
> EN: The writer has to know whether any reader still holds the old pointer — which means the reader has to once again *announce its **presence*** (a reference count, an epoch, a hazard pointer).

**die Epoche** · -n · `T1` — epoch — here epoch-based reclamation
> DE: was heißt, der Reader muss erneut *seine Anwesenheit ankündigen* (ein Reference Count, eine **Epoche**, ein Hazard Pointer).
> EN: which means the reader has to once again *announce its presence* (a reference count, an **epoch**, a hazard pointer).

**der Zustand** · ¨-e · `T1` — state — shared program state
> DE: Wir sind zurück bei Readern, die gemeinsamen **Zustand** schreiben, plus einer Allocation bei jedem Write und einem Reclamation-Problem, das zu verwalten ist.
> EN: We are back to readers writing shared **state**, plus an allocation on every write and a reclamation problem to manage.

**die Konsistenz** · kein Pl. · `T1` — consistency
> DE: keine feldübergreifende **Konsistenz** — das erfundene Paar
> EN: no cross-field **consistency** — the fabricated pair

**die Zeile** · -n · `T1` — line, row (of text / a table)
> DE: Jede **Zeile**, die korrekt ist, zwingt den Reader, in gemeinsamen Speicher zu schreiben oder den Writer warten zu lassen.
> EN: Every **row** that is correct forces the reader to write to shared memory or to make the writer wait.

**der Schluss** · ¨-e · `T1` — conclusion
> DE: Die Constraints deuten auf einen einzigen **Schluss**:
> EN: The constraints point to a single **conclusion**:

**der Kniff** · -e · `T1` — trick, knack — the clever move
> DE: Der **Kniff** also — die ganze Idee eines SeqLock — ist, gar nicht erst zu *versuchen*, es zu verhindern, und stattdessen den Reader das Chaos lesen zu lassen und es dann *bemerken* zu lassen.
> EN: So the **trick** — the whole idea of a SeqLock — is to not even *try* to prevent it, and instead let the reader read the chaos and then *notice* it.

**das Chaos** · kein Pl. · `T1` — chaos, mess
> DE: Der Kniff also — die ganze Idee eines SeqLock — ist, gar nicht erst zu *versuchen*, es zu verhindern, und stattdessen den Reader das **Chaos** lesen zu lassen und es dann *bemerken* zu lassen.
> EN: So the trick — the whole idea of a SeqLock — is to not even *try* to prevent it, and instead let the reader read the **chaos** and then *notice* it.

**die Wette** · -n · `T1` — bet, wager
> DE: *Weiter: [Teil 2 — Die **Wette**: lass es zerreißen und fang es ab](02_the_bet.md) · [Index](00_index.md)*
> EN: *Next: [Part 2 — The **bet**: let it tear and catch it](02_the_bet.md) · [Index](00_index.md)*

**das Mittel** · - · `T2` — means, way, device
> DE: Lass den Writer an Ort und Stelle überschreiben, lass den Read zerreißen, und gib dem Reader ein **Mittel**, es hinterher zu bemerken und noch einmal zu lesen.
> EN: Let the writer overwrite in place, let the read tear, and give the reader a **means** to notice it afterward and read again.

**der Versuch** · -e · `T2` — attempt, try
> DE: ## Erster **Versuch**: ein „writing"-Flag
> EN: ## First **attempt**: a "writing" flag

**der Detektor** · -en · `T2` — detector
> DE: Der naheliegende **Detektor** ist ein Boolean, das der Writer setzt, während er arbeitet.
> EN: The obvious **detector** is a boolean that the writer sets while it works.

**das Gedächtnis** · -se · `T2` — memory — the faculty of remembering (not RAM)
> DE: Es ist, dass ein Boolean kein **Gedächtnis** hat.
> EN: It's that a boolean has no **memory**.

**die Stichprobe** · -n · `T2` — sample, spot-check
> DE: Der Reader muss den Detektor also *zweimal* abtasten — einmal vor dem Kopieren, einmal danach — und nur dann schließen „kein Write hat mich überlappt", wenn die beiden **Stichproben** übereinstimmen.
> EN: So the reader must sample the detector *twice* — once before copying, once after — and only then conclude "no write overlapped me" when the two **samples** agree.

**der Vergleich** · -e · `T2` — comparison
> DE: Damit dieser **Vergleich** überhaupt etwas bedeutet, muss der Detektor eine Eigenschaft haben, die dem Boolean fehlt:
> EN: For this **comparison** to mean anything at all, the detector must have a property the boolean lacks:

**die Eigenschaft** · -en · `T2` — property, characteristic, trait
> DE: Damit dieser Vergleich überhaupt etwas bedeutet, muss der Detektor eine **Eigenschaft** haben, die dem Boolean fehlt:
> EN: For this comparison to mean anything at all, the detector must have a **property** the boolean lacks:

**der Zufall** · ¨-e · `T2` — chance, coincidence
> DE: Ein Wert, der sich nie wiederholt, schließt diesen **Zufall** aus.
> EN: A value that never repeats rules out this **coincidence**.

**die Parität** · -en · `T2` — parity — even/odd status
> DE: Ein einzelnes Inkrement erledigt beide Aufgaben: Es kippt die **Parität** um (ungerade verkündet also „schreibe gerade") und es erzeugt eine nie zuvor gesehene Zahl (zwei gleiche gerade Stichproben beweisen also „dazwischen ist nichts geschehen").
> EN: A single increment does both jobs: it flips the **parity** (so odd announces "currently writing") and it produces a number never before seen (so two equal even samples prove "nothing happened in between").

**das Protokoll** · -e · `T2` — protocol
> DE: ## Das **Protokoll**
> EN: ## The **protocol**

**der Vertrag** · ¨-e · `T2` — contract, agreement
> DE: Lies es als **Vertrag** zwischen den beiden Seiten.
> EN: Read it as a **contract** between the two sides.

**die Schleife** · -n · `T2` — loop
> DE: Alles andere, und er geht in die **Schleife** zurück und versucht es erneut.
> EN: Anything else, and it goes back into the **loop** and tries again.

**die Verzahnung** · -en · `T2` — interleaving, interlocking, meshing
> DE: Geh die beiden gefährlichen **Verzahnungen** durch und sieh, wie beide gefangen werden:
> EN: Walk through the two dangerous **interleavings** and see how both are caught:

**das Loch** · ¨-er · `T2` — hole, gap
> DE: Beide **Löcher** mit einem einzigen Zähler geschlossen.
> EN: Both **holes** closed with a single counter.

**die Weise** · -n — 'auf … Weise' = in a … way · `T2` — way, manner
> DE: Also hier der unangenehme Teil: Genau dieses Protokoll, auf die naheliegende **Weise** geschrieben, zerreißt trotzdem — nicht weil die Logik falsch ist, sondern weil die Maschine darunter deine Instruktionen nicht in der Reihenfolge ausführt, in der du sie geschrieben hast.
> EN: So here's the uncomfortable part: this very protocol, written the obvious **way**, still tears — not because the logic is wrong, but because the machine underneath doesn't execute your instructions in the order you wrote them.

**die Reihenfolge** · -n · `T2` — order, sequence
> DE: Also hier der unangenehme Teil: Genau dieses Protokoll, auf die naheliegende Weise geschrieben, zerreißt trotzdem — nicht weil die Logik falsch ist, sondern weil die Maschine darunter deine Instruktionen nicht in der **Reihenfolge** ausführt, in der du sie geschrieben hast.
> EN: So here's the uncomfortable part: this very protocol, written the obvious way, still tears — not because the logic is wrong, but because the machine underneath doesn't execute your instructions in the **order** you wrote them.

**das Verhältnis** · das; -se · `T3` — ratio; also relationship, proportion
> DE: Genau dieses **Verhältnis** — meistens grün, gelegentlich falsch — ist der Fingerabdruck eines Memory-Ordering-Bugs, und deshalb kannst du einem bestandenen Test hier nicht trauen.
> EN: Exactly this **ratio** — mostly green, occasionally wrong — is the fingerprint of a memory-ordering bug, and that is why you cannot trust a passing test here.

**die Kante** · die; -n · `T3` — edge
> DE: Die zwei Bumps des Writers sind seine **Kanten**; der payload soll zwischen ihnen leben.
> EN: The writer's two bumps are its **edges**; the payload should live between them.

**das Durcheinander** · das; kein Pl. · `T3` — mess, jumble, muddle — here the reordered shuffle of operations
> DE: Ein anderer Thread sieht das **Durcheinander**.
> EN: Another thread sees the **mess**.

**der Fluchtweg** · der; -e · `T3` — escape route
> DE: Vier **Fluchtwege**: der payload driftet über jede der vier Fensterkanten hinaus
> EN: Four **escape routes**: the payload drifts out past each of the four window edges

**das Leck** · das; -s · `T3` — leak
> DE: Jede Kante ist ein eigenes **Leck**, und jede braucht eine eigene Entscheidung.
> EN: Each edge is its own **leak**, and each needs its own decision.

**die Umordnung** · die; -en · `T3` — reordering
> DE: Aber der *Grund*, warum er korrekt ist, und der Grund, warum eine fence das richtige Werkzeug ist, reicht tiefer als „verhindert **Umordnung**“.
> EN: But the *reason* why it is correct, and the reason why a fence is the right tool, reaches deeper than "prevents **reordering**".

**die Behauptung** · die; -en · `T3` — claim, assertion
> DE: Die zu beweisende **Behauptung** ist eine einzige:
> EN: The **claim** to be proven is a single one:

**der Datenkanal** · der; ¨-e · `T3` — data channel
> DE: Das ist auch der Grund, warum ein Ordering-auf-der-Operation nicht genügen würde, selbst dort, wo es typecheckt: Ein Ordering auf einem Atomic verknüpft *dieses Atomic* über Threads hinweg, aber hier ist der **Datenkanal** der **payload**, und das, worüber wir synchronisieren, ist die **seq** — zwei verschiedene Variablen.
> EN: That is also the reason why an ordering-on-the-operation would not suffice, even where it typechecks: an ordering on an atomic links *this atomic* across threads, but here the **data channel** is the **payload**, and what we synchronize over is the **seq** — two different variables.

**das Verbrechen** · das; - · `T3` — crime
> DE: Was bleibt, ist ein subtileres **Verbrechen**, das wir die ganze Zeit begangen haben: Der Reader hat Bytes gelesen, die der Writer gerade aktiv verändert, und in Rusts Memory-Model ist das nicht bloß „Müll lesen“ — es ist Undefined Behaviour.
> EN: What remains is a subtler **crime** that we've been committing the whole time: the reader has read bytes that the writer is actively changing right now, and in Rust's memory model that is not merely "reading garbage" — it is Undefined Behaviour.

**der Haken** · - · `T4` — the catch, the snag (the hidden problem)
> DE: Der **Haken** ist, dass dieser Read in Rusts Speichermodell nicht bloß „Müll liest".
> EN: The **catch** is that this Read, in Rust's memory model, doesn't just "read garbage".

**das Speichermodell** · -e · `T4` — memory model
> DE: Der Haken ist, dass dieser Read in Rusts **Speichermodell** nicht bloß „Müll liest".
> EN: The catch is that this Read, in Rust's **memory model**, doesn't just "read garbage".

**die Annahme** · -n · `T4` — assumption
> DE: Es ist UB: Der Compiler darf annehmen, dass es nie passiert, und unter dieser **Annahme** optimieren — den Read aus der Retry-Schleife herausziehen, den Wert als unverändert beweisen, Zweige löschen, von denen er „weiß", dass sie tot sind.
> EN: It is UB: the compiler may assume that it never happens, and optimize under that **assumption** — pull the Read out of the retry loop, prove the value unchanged, delete branches that it "knows" are dead.

**der Zweig** · -e · `T4` — branch (of code)
> DE: Es ist UB: Der Compiler darf annehmen, dass es nie passiert, und unter dieser Annahme optimieren — den Read aus der Retry-Schleife herausziehen, den Wert als unverändert beweisen, **Zweige** löschen, von denen er „weiß", dass sie tot sind.
> EN: It is UB: the compiler may assume that it never happens, and optimize under that assumption — pull the Read out of the retry loop, prove the value unchanged, delete **branches** that it "knows" are dead.

**die Prämisse** · -n · `T4` — premise
> DE: Der Fehler ist nicht der Müll; der Fehler ist, dass der Compiler nun aus einer falschen **Prämisse** schließt.
> EN: The bug is not the garbage; the bug is that the compiler now reasons from a false **premise**.

**der Müll** · kein Pl. · `T4` — garbage, rubbish
> DE: Der Fehler ist nicht der **Müll**; der Fehler ist, dass der Compiler nun aus einer falschen Prämisse schließt.
> EN: The bug is not the **garbage**; the bug is that the compiler now reasons from a false premise.

**der Reflex** · -e · `T4` — reflex, knee-jerk reaction
> DE: Wer von C kommt, dessen **Reflex** ist `volatile`.
> EN: For anyone coming from C, the **reflex** is `volatile`.

**der Handel** · kein Pl. · `T4` — deal, bargain — einen Handel schließen = to strike a deal
> DE: Im Kernel funktioniert es, weil der Kernel von einem bekannten Compiler mit bekannten Flags übersetzt wird; es ist ein **Handel**, der mit einer bestimmten Implementierung geschlossen wird, nicht mit der Sprache.
> EN: In the kernel it works because the kernel is compiled by a known compiler with known flags; it is a **deal** struck with a particular implementation, not with the language.

**das Missverhältnis** · -se · `T4` — mismatch, disparity
> DE: (Hans Boehm hat ein ganzes Paper über genau dieses **Missverhältnis** geschrieben: seqlocks und Speichermodelle von Sprachen vertragen sich nicht, es sei denn, die Sprache gibt einem ein hinreichend billiges Atomic.)
> EN: (Hans Boehm wrote an entire paper about exactly this **mismatch**: seqlocks and language memory models don't get along, unless the language gives you a sufficiently cheap Atomic.)

**die Legalität** · kein Pl. · `T4` — legality, lawfulness
> DE: Was es hinzufügt, ist **Legalität**: Ein Atomzugriff, der gegen einen anderen Atomzugriff rennt, ist *kein* Data Race, also kein UB.
> EN: What it adds is **legality**: an atomic access that races against another atomic access is *not* a Data Race, hence not UB.

**das Tor** · -e · `T4` — gate — here metaphorical: a check a type must pass
> DE: Die Anforderung, die das erzwingt: `Pod`, und warum es zwei **Tore** sind
> EN: The requirement that enforces this: `Pod`, and why it is two **gates**

**das Bitmuster** · - · `T4` — bit pattern
> DE: Um ein beliebiges `T` als eine Reihe von `usize`-Wörtern umzudeuten, muss `T` tatsächlich schlichte Bytes *sein* — kein Padding, jedes **Bitmuster** gültig (der Leser wird halb geschriebene Mischungen beobachten, bevor er sie verwirft), ein definiertes Layout.
> EN: To reinterpret an arbitrary `T` as a series of `usize` words, `T` must actually *be* plain bytes — no padding, every **bit pattern** valid (the reader will observe half-written mixtures before discarding them), a defined layout.

**die Lizenz** · -en · `T4` — license, permit — here: a permission the implementer grants
> DE: Erstens ist `Pod` eine **Lizenz**, die der Implementierende unterschreibt, kein Fakt, den der Compiler nachprüft — `unsafe impl Pod for Foo {}` ist ein Versprechen, das man gibt und für das man die Verantwortung trägt; macht man es falsch, ist es UB, weshalb der Trait `unsafe` zu implementieren ist.
> EN: First, `Pod` is a **license** that the implementer signs, not a fact that the compiler checks — `unsafe impl Pod for Foo {}` is a promise you make and bear responsibility for; get it wrong and it is UB, which is why the trait is `unsafe` to implement.

**der Missbrauch** · kein Pl. · `T4` — misuse, abuse
> DE: Es macht Korrektheit nicht automatisch; es lokalisiert die Beweispflicht auf eine greppbare Zeile und lässt versehentlichen **Missbrauch** (ein `String`, ein Typ mit Padding) an der Kompilierung scheitern.
> EN: It does not make correctness automatic; it localizes the burden of proof to a greppable line and makes accidental **misuse** (a `String`, a type with padding) fail at compilation.

**die Falle** · -n · `T4` — trap, pitfall
> DE: Zweitens — und das ist die **Falle** — ist `Pod` notwendig, aber **nicht hinreichend**.
> EN: Second — and this is the **trap** — `Pod` is necessary but **not sufficient**.

**das Vielfache** · -n (adj. Dekl.) · `T4` — a multiple (of a number)
> DE: Also sind die Prüfungen auf Größen-**Vielfaches** und Alignment ein *zweites, unabhängiges Tor*, das der Typ passieren muss, separat erzwungen (ein `const`-Assert, das zur Compile-Zeit scheitert, nicht zur Laufzeit).
> EN: So the checks for size **multiple** and alignment are a *second, independent gate* that the type must pass, enforced separately (a `const` Assert that fails at compile time, not at runtime).

**die Laufzeit** · -en · `T4` — runtime (as opposed to compile time)
> DE: Also sind die Prüfungen auf Größen-Vielfaches und Alignment ein *zweites, unabhängiges Tor*, das der Typ passieren muss, separat erzwungen (ein `const`-Assert, das zur Compile-Zeit scheitert, nicht zur **Laufzeit**).
> EN: So the checks for size multiple and alignment are a *second, independent gate* that the type must pass, enforced separately (a `const` Assert that fails at compile time, not at **runtime**).

**die Lüge** · -n · `T4` — lie, falsehood
> DE: Ein einziger Schreiber war eine bequeme **Lüge**
> EN: A single writer was a convenient **lie**

**das Gegenteil** · -e · `T4` — the opposite, the reverse
> DE: Das ist ein Test, dessen Aufgabe es ist, beim Fehler zu *scheitern* — das **Gegenteil** eines Tests, der den Happy Path bestätigt.
> EN: This is a test whose job is to *fail* on the bug — the **opposite** of a test that confirms the happy path.

**die Beseitigung** · -en · `T4` — removal, elimination
> DE: **Miri**, für das Undefined Behaviour, das ein normaler Test ausführen kann, ohne es zu erkennen — das Data Race, dessen **Beseitigung** diesen Teil ausgemacht hat.
> EN: **Miri**, for the Undefined Behaviour that a normal test can execute without detecting it — the Data Race whose **elimination** is what this part has been about.

**die Invariante** · -n · `T4` — invariant (a property that must always hold)
> DE: Es spielt ein kleines Szenario — ein Schreiber, ein Leser; dann zwei Schreiber — unter *jeder* Thread-Verschränkung, die das Speichermodell erlaubt, erneut durch und prüft, dass die **Invarianten** in allen gelten.
> EN: It replays a small scenario — one writer, one reader; then two writers — under *every* thread interleaving that the memory model allows, and checks that the **invariants** hold in all of them.

**der Lohn** · ¨-e · `T4` — reward, payoff — also wage/pay
> DE: Der **Lohn**, in Nanosekunden
> EN: The **payoff**, in nanoseconds

**der Lesepfad** · -e · `T4` — read path (the code path a reader takes)
> DE: All das — das Zerreißen, die fences, die Atomics, die `Pod`-Anforderung — erkauft eine einzige Sache: einen **Lesepfad**, der flach bleibt, während sich Leser häufen, dort wo ein `RwLock` einbricht.
> EN: All of this — the tearing, the fences, the Atomics, the `Pod` requirement — buys a single thing: a **read path** that stays flat while readers pile up, where an `RwLock` collapses.

**die Latenz** · -en · `T4` — latency
> DE: Gemessen auf einem Apple M2, ein 32-Byte-Payload, Leser-**Latenz** mit wachsender Leserzahl:
> EN: Measured on an Apple M2, a 32-byte payload, reader **latency** with growing reader count:

**die Kluft** · ¨-e · `T4` — gap, gulf, chasm
> DE: Diese **Kluft** ist das MESI-Diagramm aus Teil 1, ausgezahlt in Nanosekunden.
> EN: This **gap** is the MESI diagram from Part 1, paid out in nanoseconds.

**die Einschränkung** · -en · `T4` — constraint, restriction, limitation
> DE: Nichts davon ist umsonst, und der Preis ist genau die Menge an **Einschränkungen**, die den Lesepfad gratis gemacht hat:
> EN: None of this is free, and the price is exactly the set of **constraints** that made the read path free:

**die Spitze** · -n · `T4` — tip, top, peak — here: the top (best bid/ask) of an order book
> DE: Wer das annimmt, bekommt das Ding, das man einmal baut und überall dort wiederverwendet, wo ein kleiner Wert weit öfter gelesen als geschrieben wird — der chain head, der mark price, die **Spitze** eines order book.
> EN: Whoever accepts this gets the thing you build once and reuse everywhere a small value is read far more often than written — the chain head, the mark price, the **top** of an order book.

**der Kern** · -e · `T4` — core (CPU core)
> DE: Wer es zurückweist, greift zu `RwLock` und zahlt das 450-Fache, sobald acht **Kerne** zum ersten Mal gleichzeitig lesen.
> EN: Whoever rejects it reaches for `RwLock` and pays 450 times as much, as soon as eight **cores** read simultaneously for the first time.

**der Tausch** · kein Pl. · `T4` — trade, exchange, trade-off
> DE: Bei den read-mostly Problemen, für die er gebaut ist, ist das genau der **Tausch**, den man will.
> EN: For the read-mostly problems it is built for, that is exactly the **trade** you want.

## Verben — verbs (170)

**entwerfen** · entwarf · entworfen · `T0` — to design, draft
> DE: Diese Serie **entwirft** eines von Grund auf.
> EN: This series **designs** one from the ground up.

**scheitern** · (an + Dat.) — sein · `T0` — to fail (at), fall short
> DE: Wir beginnen bei dem Problem, für dessen Lösung ein SeqLock existiert, sehen jedem Lock, zu dem man normalerweise greifen würde, dabei zu, wie es an genau einer der Randbedingungen **scheitert**, und gehen dann die Wette ein, die das ganze Primitive definiert: Statt den Leser daran zu hindern, einen halb geschriebenen Wert zu beobachten, lassen wir es geschehen und bringen den Leser dazu, es zu erkennen.
> EN: We start from the problem that a SeqLock exists to solve, watch each lock you would normally reach for **fail** at exactly one of the constraints, and then make the bet that defines the whole primitive: Instead of preventing the reader from observing a half-written value, we let it happen and get the reader to detect it.

**hindern** · jdn. daran hindern · `T0` — to prevent, keep (someone from doing)
> DE: Statt den Leser daran zu **hindern**, einen halb geschriebenen Wert zu beobachten, lassen wir es geschehen und bringen den Leser dazu, es zu erkennen.
> EN: Instead of **preventing** the reader from observing a half-written value, we let it happen and get the reader to detect it.

**geschehen** · geschah · geschehen — sein · `T0` — to happen, occur
> DE: Statt den Leser daran zu hindern, einen halb geschriebenen Wert zu beobachten, lassen wir es **geschehen** und bringen den Leser dazu, es zu erkennen.
> EN: Instead of preventing the reader from observing a half-written value, we let it **happen** and get the reader to detect it.

**erzwingen** · erzwang · erzwungen · `T0` — to force, compel, bring about by force
> DE: Jede Entscheidung danach ist **erzwungen** — durch einen Use Case, oder durch das Scheitern der einfacheren Alternative, oder, in einem denkwürdigen Fall, durch einen ARM-Prozessor, der deine Instruktionen umordnet und einen Wert korrumpiert, von dem deine Testsuite steif und fest behauptet, er sei in Ordnung.
> EN: Every decision after that is **forced** — by a use case, or by the failure of the simpler alternative, or, in one memorable case, by an ARM processor that reorders your instructions and corrupts a value that your test suite adamantly insists is fine.

**korrumpieren** · `T0` — to corrupt (data)
> DE: Jede Entscheidung danach ist erzwungen — durch einen Use Case, oder durch das Scheitern der einfacheren Alternative, oder, in einem denkwürdigen Fall, durch einen ARM-Prozessor, der deine Instruktionen umordnet und einen Wert **korrumpiert**, von dem deine Testsuite steif und fest behauptet, er sei in Ordnung.
> EN: Every decision after that is forced — by a use case, or by the failure of the simpler alternative, or, in one memorable case, by an ARM processor that reorders your instructions and **corrupts** a value that your test suite adamantly insists is fine.

**umordnen** · (sep.) · `T0` — to reorder, rearrange
> DE: Jede Entscheidung danach ist erzwungen — durch einen Use Case, oder durch das Scheitern der einfacheren Alternative, oder, in einem denkwürdigen Fall, durch einen ARM-Prozessor, der deine Instruktionen **umordnet** und einen Wert korrumpiert, von dem deine Testsuite steif und fest behauptet, er sei in Ordnung.
> EN: Every decision after that is forced — by a use case, or by the failure of the simpler alternative, or, in one memorable case, by an ARM processor that **reorders** your instructions and corrupts a value that your test suite adamantly insists is fine.

**voraussetzen** · (sep.) · `T0` — to presuppose, assume, take as given
> DE: Kein Lock-free-Hintergrund wird **vorausgesetzt**.
> EN: No lock-free background is **assumed**.

**kollidieren** · — sein · `T0` — to collide, clash
> DE: Die Maschinerie — `Relaxed`/`Acquire`/`Release`, fences, `Pod`, Miri, loom — wird in dem Moment eingeführt, in dem das Design mit ihr **kollidiert**.
> EN: The machinery — `Relaxed`/`Acquire`/`Release`, fences, `Pod`, Miri, loom — is introduced the moment the design **collides** with it.

**zwingen** · zwang · gezwungen · `T0` — to force, compel (coerce someone)
> DE: Die drei Randbedingungen, die das schwer machen, und dann die Tour der Fehlschläge: Ein `RwLock` lässt Leser gemeinsam hinein, **zwingt** aber jeden von ihnen, einen gemeinsamen Zähler zu *schreiben*, und serialisiert damit Cores, die gar nicht in Konflikt stehen; `ArcSwap` und RCU umgehen das Zerreißen, ziehen den Leser aber zurück hinein, sich selbst zur Reclamation anzumelden.
> EN: The three constraints that make this hard, and then the tour of failures: An `RwLock` lets readers in together, but **forces** each of them to *write* a shared counter, and thereby serializes cores that are not in conflict at all; `ArcSwap` and RCU sidestep the tearing, but pull the reader back in to register itself for reclamation.

**serialisieren** · `T0` — to serialise — force operations into a sequence
> DE: Die drei Randbedingungen, die das schwer machen, und dann die Tour der Fehlschläge: Ein `RwLock` lässt Leser gemeinsam hinein, zwingt aber jeden von ihnen, einen gemeinsamen Zähler zu *schreiben*, und **serialisiert** damit Cores, die gar nicht in Konflikt stehen; `ArcSwap` und RCU umgehen das Zerreißen, ziehen den Leser aber zurück hinein, sich selbst zur Reclamation anzumelden.
> EN: The three constraints that make this hard, and then the tour of failures: An `RwLock` lets readers in together, but forces each of them to *write* a shared counter, and thereby **serializes** cores that are not in conflict at all; `ArcSwap` and RCU sidestep the tearing, but pull the reader back in to register itself for reclamation.

**umgehen** · umging · umgangen (untrennbar) · `T0` — to circumvent, get around, avoid
> DE: Die drei Randbedingungen, die das schwer machen, und dann die Tour der Fehlschläge: Ein `RwLock` lässt Leser gemeinsam hinein, zwingt aber jeden von ihnen, einen gemeinsamen Zähler zu *schreiben*, und serialisiert damit Cores, die gar nicht in Konflikt stehen; `ArcSwap` und RCU **umgehen** das Zerreißen, ziehen den Leser aber zurück hinein, sich selbst zur Reclamation anzumelden.
> EN: The three constraints that make this hard, and then the tour of failures: An `RwLock` lets readers in together, but forces each of them to *write* a shared counter, and thereby serializes cores that are not in conflict at all; `ArcSwap` and RCU **sidestep** the tearing, but pull the reader back in to register itself for reclamation.

**sich anmelden** · (sep.) · `T0` — to register, sign up, announce oneself
> DE: Die drei Randbedingungen, die das schwer machen, und dann die Tour der Fehlschläge: Ein `RwLock` lässt Leser gemeinsam hinein, zwingt aber jeden von ihnen, einen gemeinsamen Zähler zu *schreiben*, und serialisiert damit Cores, die gar nicht in Konflikt stehen; `ArcSwap` und RCU umgehen das Zerreißen, ziehen den Leser aber zurück hinein, sich selbst zur Reclamation **anzumelden**.
> EN: The three constraints that make this hard, and then the tour of failures: An `RwLock` lets readers in together, but forces each of them to *write* a shared counter, and thereby serializes cores that are not in conflict at all; `ArcSwap` and RCU sidestep the tearing, but pull the reader back in to **register** itself for reclamation.

**weisen auf** · wies · gewiesen (auf + Akk.) · `T0` — to point at, indicate
> DE: Jede korrekte Option bricht dieselbe Regel, und sie **weist auf** den einzigen Ausweg — der Leser muss unsichtbar sein.
> EN: Every correct option breaks the same rule, and it **points to** the only way out — the reader must be invisible.

**aufhalten** · hielt auf · aufgehalten (sep.) · `T0` — to stop, hold up, halt
> DE: Wenn der Schreiber nicht **aufgehalten** werden kann und der Leser sich nicht anmelden kann, bleibt ein einziger Zug: den Read zerreißen lassen und dem Leser einen Weg geben, es hinterher zu bemerken und erneut zu versuchen.
> EN: If the writer cannot be **stopped** and the reader cannot register, a single move remains: let the read tear and give the reader a way to notice it afterwards and try again.

**zerreißen** · zerriss · zerrissen · `T0` — to tear (apart) — here, a torn read
> DE: Wenn der Schreiber nicht aufgehalten werden kann und der Leser sich nicht anmelden kann, bleibt ein einziger Zug: den Read **zerreißen** lassen und dem Leser einen Weg geben, es hinterher zu bemerken und erneut zu versuchen.
> EN: If the writer cannot be stopped and the reader cannot register, a single move remains: let the read **tear** and give the reader a way to notice it afterwards and try again.

**herleiten** · (sep.) · `T0` — to derive, deduce
> DE: Das reduziert alles auf eine einzige Frage — woher weiß ein Leser, dass er während eines Schreibvorgangs gelesen hat? — und wir **leiten** die Antwort auf die harte Tour **her**, indem wir einem booleschen Flag beim Scheitern zusehen, weil es keine Geschichte trägt, bis das Einzige übrig bleibt, was funktioniert: ein Zähler, der gerade ist, wenn der Wert stabil ist, und ungerade, während er geschrieben wird, vor und nach dem Lesen abgetastet.
> EN: This reduces everything to a single question — how does a reader know it read during a write? — and we **derive** the answer the hard way, by watching a boolean flag fail, because it carries no history, until the only thing left that works remains: a counter that is even when the value is stable and odd while it is being written, sampled before and after the read.

**abtasten** · (sep.) · `T0` — to sample, scan (read a value)
> DE: Das reduziert alles auf eine einzige Frage — woher weiß ein Leser, dass er während eines Schreibvorgangs gelesen hat? — und wir leiten die Antwort auf die harte Tour her, indem wir einem booleschen Flag beim Scheitern zusehen, weil es keine Geschichte trägt, bis das Einzige übrig bleibt, was funktioniert: ein Zähler, der gerade ist, wenn der Wert stabil ist, und ungerade, während er geschrieben wird, vor und nach dem Lesen **abgetastet**.
> EN: This reduces everything to a single question — how does a reader know it read during a write? — and we derive the answer the hard way, by watching a boolean flag fail, because it carries no history, until the only thing left that works remains: a counter that is even when the value is stable and odd while it is being written, **sampled** before and after the read.

**bewachen** · `T0` — to guard, watch over
> DE: Wir reparieren es mit fences, und um sie zu platzieren, brauchen wir die Idee, die alle immer verkehrt herum verstehen: `Release` und `Acquire` sind Einweg-Gates, von denen jedes nur eine Seite der Operation **bewacht**, an die es geheftet ist.
> EN: We fix it with fences, and to place them we need the idea that everyone always understands backwards: `Release` and `Acquire` are one-way gates, each of which **guards** only one side of the operation it is pinned to.

**heften** · (an + Akk.) · `T0` — to pin, attach, fasten (to)
> DE: Wir reparieren es mit fences, und um sie zu platzieren, brauchen wir die Idee, die alle immer verkehrt herum verstehen: `Release` und `Acquire` sind Einweg-Gates, von denen jedes nur eine Seite der Operation bewacht, an die es **geheftet** ist.
> EN: We fix it with fences, and to place them we need the idea that everyone always understands backwards: `Release` and `Acquire` are one-way gates, each of which guards only one side of the operation it is **pinned** to.

**auskommen** · kam aus · ausgekommen (sep., mit + Dat.) — sein · `T0` — to make do (with), get by (on)
> DE: Zwei der vier Fensterkanten **kommen** mit einem Ordering auf dem Atomic selbst **aus**; die anderen beiden brauchen einen eigenständigen fence — und die fences sind, wie sich herausstellt, das, was zwei Threads sich zu einer happens-before-Beziehung die Hand reichen lässt.
> EN: Two of the four window edges **get by** with an ordering on the atomic itself; the other two need a standalone fence — and the fences are, as it turns out, what lets two threads reach out their hands to one another into a happens-before relationship.

**sich erweisen als** · erwies · erwiesen · `T0` — to prove/turn out to be
> DE: Die Korrektur macht jeden Zugriff auf die Payload atomar, Wort für Wort, und verwandelt „Müll lesen" von UB in einen legalen Read, den der Zähler wegwirft — was die Payload zwingt, `Pod` zu sein, eine Schranke, die sich als Lizenz **erweist**, die der Implementierer *unterschreibt*, statt einer, die der Compiler prüft, und die nicht einmal Alignment abdeckt.
> EN: The fix makes every access to the payload atomic, word by word, and turns "reading garbage" from UB into a legal read that the counter throws away — which forces the payload to be `Pod`, a bound that **turns out** to be a license the implementer *signs*, rather than one the compiler checks, and that does not even cover alignment.

**ausschließen** · schloss aus · ausgeschlossen (sep.) · `T0` — to rule out, exclude
> DE: Teil 1 steckt das Problem ab und **schließt** die Alternativen **aus**; Teil 2 geht die Kernwette ein; Teil 3 ist das Memory-Ordering-Herz; Teil 4 ist die Sprache, der Fall mehrerer Schreiber und der Beweis.
> EN: Part 1 stakes out the problem and **rules out** the alternatives; Part 2 makes the core bet; Part 3 is the memory-ordering heart; Part 4 is the language, the multiple-writers case, and the proof.

**invalidieren** · `T0` — to invalidate (a cache line)
> DE: **MESI / Cache-Kohärenz** — das Protokoll, das die Caches der einzelnen Cores konsistent hält; eine cache line, die ein Core schreibt, muss in den anderen **invalidiert** werden — deshalb serialisiert ein gemeinsam geschriebener Zähler Cores, die logisch gar nicht in Konflikt stehen.
> EN: **MESI / cache coherence** — the protocol that keeps the individual cores' caches consistent; a cache line that one core writes must be **invalidated** in the others — which is why a jointly written counter serializes cores that are logically not in conflict at all.

**aufrufen** · rief auf · aufgerufen (sep.) · `T0` — to call, invoke (e.g. a function or syscall)
> DE: Es heißt SeqLock, und wer auf Linux `clock_gettime` **aufgerufen** hat, ohne dass der Aufruf den Kernel erreicht, hat eines benutzt.
> EN: It is called SeqLock, and anyone who has **called** `clock_gettime` on Linux without the call reaching the kernel has used one.

**vorankommen** · kam voran · vorangekommen (sep.) — sein · `T0` — to advance, make progress, get ahead
> DE: Irgendwo in einem System, das einen gemeinsamen Wert weit häufiger liest, als es ihn schreibt — ein Blockchain-Node, der zehntausende Male pro Sekunde zwischen zwei Blöcken fragt „was ist der Chain-Head?", eine Börse, die bei jeder einzelnen Order den Mark-Preis liest —, steckt ein Primitive, das all diese Leser **vorankommen** lässt, ohne je zu blockieren und ohne je ein Byte in den gemeinsamen Speicher zu schreiben.
> EN: Somewhere in a system that reads a shared value far more often than it writes it — a blockchain node that asks tens of thousands of times per second between two blocks "what is the chain head?", an exchange that reads the mark price on every single order —, sits a primitive that lets all these readers **make progress** without ever blocking and without ever writing a single byte to shared memory.

**verwandeln** · (in + Akk.) · `T0` — to turn (something) into, transform
> DE: Die Korrektur macht jeden Zugriff auf die Payload atomar, Wort für Wort, und **verwandelt** „Müll lesen" von UB in einen legalen Read, den der Zähler wegwirft — was die Payload zwingt, `Pod` zu sein, eine Schranke, die sich als Lizenz erweist, die der Implementierer *unterschreibt*, statt einer, die der Compiler prüft, und die nicht einmal Alignment abdeckt.
> EN: The fix makes every access to the payload atomic, word by word, and **turns** "reading garbage" from UB into a legal read that the counter throws away — which forces the payload to be `Pod`, a bound that turns out to be a license the implementer *signs*, rather than one the compiler checks, and that does not even cover alignment.

**wegwerfen** · warf weg · weggeworfen (sep.) · `T0` — to throw away, discard
> DE: Die Korrektur macht jeden Zugriff auf die Payload atomar, Wort für Wort, und verwandelt „Müll lesen" von UB in einen legalen Read, den der Zähler **wegwirft** — was die Payload zwingt, `Pod` zu sein, eine Schranke, die sich als Lizenz erweist, die der Implementierer *unterschreibt*, statt einer, die der Compiler prüft, und die nicht einmal Alignment abdeckt.
> EN: The fix makes every access to the payload atomic, word by word, and turns "reading garbage" from UB into a legal read that the counter **throws away** — which forces the payload to be `Pod`, a bound that turns out to be a license the implementer *signs*, rather than one the compiler checks, and that does not even cover alignment.

**unterschreiben** · unterschrieb · unterschrieben · `T0` — to sign (put one's signature to) — here, a promise the implementer vouches for
> DE: Die Korrektur macht jeden Zugriff auf die Payload atomar, Wort für Wort, und verwandelt „Müll lesen" von UB in einen legalen Read, den der Zähler wegwirft — was die Payload zwingt, `Pod` zu sein, eine Schranke, die sich als Lizenz erweist, die der Implementierer **unterschreibt**, statt einer, die der Compiler prüft, und die nicht einmal Alignment abdeckt.
> EN: The fix makes every access to the payload atomic, word by word, and turns "reading garbage" from UB into a legal read that the counter throws away — which forces the payload to be `Pod`, a bound that turns out to be a license the implementer **signs**, rather than one the compiler checks, and that does not even cover alignment.

**verbergen** · verbarg · verborgen · `T0` — to hide, conceal
> DE: Alles wird auf `aarch64` (Apple M2) gebaut und gemessen, denn das schwache Memory-Modell ist der Ort, an dem sich die interessanten Fehler zeigen; ein x86-Lauf würde die Hälfte von Teil 3 **verbergen**.
> EN: Everything is built and measured on `aarch64` (Apple M2), because the weak memory model is where the interesting bugs show up; an x86 run would **hide** half of Part 3.

**überlappen** · `T0` — to overlap
> DE: **SeqLock** — ein Lock, bei dem Leser nie blockieren und nie in den gemeinsamen Speicher schreiben; sie lesen optimistisch und versuchen es erneut, falls ein Schreibvorgang **überlappt** hat.
> EN: **SeqLock** — a lock in which readers never block and never write to shared memory; they read optimistically and try again if a write has **overlapped**.

**umdeuten** · (sep.) · `T0` — to reinterpret, recast (as something else)
> DE: Erlaubt es, ihn gefahrlos als rohe Wörter **umzudeuten**.
> EN: Permits it to be safely **reinterpreted** as raw words.

**hochzählen** · (sep.) · `T0` — to count up, increment
> DE: **Sequenzzähler / seq** — die Ganzzahl, die der Schreiber um jeden Schreibvorgang herum **hochzählt**; gerade = stabil, ungerade = ein Schreibvorgang läuft.
> EN: **Sequence counter / seq** — the integer that the writer **increments** around every write; even = stable, odd = a write is in progress.

**vorrücken** · trennb. · `T1` — to advance, move forward
> DE: Ein Blockchain-Node **rückt** seinen kanonischen Chain-Head etwa alle zwölf Sekunden vor.
> EN: A blockchain node **advances** its canonical chain head roughly every twelve seconds.

**markieren** · `T1` — to tag, mark
> DE: jedes mit `latest` **markierte** `eth_call`
> EN: every `eth_call` **tagged** with `latest`

**validieren** · `T1` — to validate
> DE: jede Transaktion, die der Mempool **validiert**
> EN: every transaction that the mempool **validates**

**aktualisieren** · `T1` — to update
> DE: Ein Oracle-Thread **aktualisiert** `(mark_price, funding_index, timestamp)` einmal pro Tick, und die Risk-Engine liest es bei *jeder einzelnen Order*, um Margin zu berechnen.
> EN: An oracle thread **updates** `(mark_price, funding_index, timestamp)` once per tick, and the risk engine reads it on *every single order* to compute margin.

**berechnen** · `T1` — to compute, calculate
> DE: Ein Oracle-Thread aktualisiert `(mark_price, funding_index, timestamp)` einmal pro Tick, und die Risk-Engine liest es bei *jeder einzelnen Order*, um Margin zu **berechnen**.
> EN: An oracle thread updates `(mark_price, funding_index, timestamp)` once per tick, and the risk engine reads it on *every single order* to **compute** margin.

**bieten** · bot · geboten · `T1` — to offer, provide
> DE: Der größte atomare Store, den die Hardware **bietet**, ist ein Maschinenwort — 8 Byte auf einer 64-Bit-Maschine, 16 mit einem double-width compare-and-swap, wenn man vorsichtig ist.
> EN: The largest atomic store the hardware **offers** is a machine word — 8 bytes on a 64-bit machine, 16 with a double-width compare-and-swap, if you are careful.

**paaren** · `T1` — to pair (up) — here past participle 'gepaart' = paired
> DE: Er bekommt den Hash von Block 1000, **gepaart** mit der Zahl 999:
> EN: It gets the hash of block 1000, **paired** with the number 999:

**weitergeben** · gab weiter · weitergegeben, trennb. · `T1` — to pass on, hand on
> DE: Gibt man den an einen Nutzer **weiter**, ist er falsch; füttert man ihn in einen State-Root-Lookup, wird der Node korrumpiert.
> EN: Hand it **on** to a user and it is wrong; feed it into a state-root lookup and the node gets corrupted.

**füttern** · `T1` — to feed — here feed data into a routine
> DE: Gibt man den an einen Nutzer weiter, ist er falsch; **füttert** man ihn in einen State-Root-Lookup, wird der Node korrumpiert.
> EN: Hand it on to a user and it is wrong; **feed** it into a state-root lookup and the node gets corrupted.

**erfinden** · erfand · erfunden · `T1` — to invent, make up — here participle 'erfunden' = fabricated
> DE: Das hier ist **erfunden**.
> EN: This one is **fabricated**.

**gehören** · (+ Dat.) · `T1` — to belong to — here 'is the domain/property of'
> DE: Diese Grenze **gehört** der Hardware, nicht der Sprache.
> EN: This limit **belongs** to the hardware, not to the language.

**retten** · `T1` — to save, rescue
> DE: Atomic pro Feld **rettet** dich nicht
> EN: Per-field atomics do not **save** you

**packen** · `T1` — to pack — here 'wrap' a value in a type
> DE: `mark_price` ist ein `u64`, **pack** es in ein `AtomicU64`; `funding_index` auch.
> EN: `mark_price` is a `u64`, **pack** it into an `AtomicU64`; `funding_index` too.

**liquidieren** · `T1` — to liquidate — forcibly close a position
> DE: Das *Paar* ist ein Wert, den es nie gegeben hat, und die daraus berechnete Margin ist falsch — falsch genug, um einen Account zu **liquidieren**, der eigentlich gesund war.
> EN: The *pair* is a value that never existed, and the margin computed from it is wrong — wrong enough to **liquidate** an account that was actually healthy.

**fangen** · fing · gefangen · `T1` — to catch
> DE: Das ist echtes Geld, verloren an einen Konsistenz-Bug, den Atomics pro Feld strukturell nicht **fangen** können.
> EN: That is real money, lost to a consistency bug that per-field atomics structurally cannot **catch**.

**veröffentlichen** · `T1` — to publish
> DE: **Veröffentliche** einen Snapshot aus mehreren Feldern so, dass jeder Reader stets einen Snapshot sieht, den es als Ganzes tatsächlich gegeben hat.
> EN: **Publish** a snapshot made of several fields such that every reader always sees a snapshot that, as a whole, actually existed.

**verpacken** · `T1` — to wrap, package — here 'comes wrapped in'
> DE: Interessant wird es, weil Korrektheit in drei Constraints **verpackt** kommt, mit denen der Lock zu kämpfen hat.
> EN: It gets interesting because correctness comes **wrapped** in three constraints that the lock has to struggle with.

**kämpfen** · `T1` — to fight, struggle — 'zu kämpfen haben mit' = to struggle with
> DE: Interessant wird es, weil Korrektheit in drei Constraints verpackt kommt, mit denen der Lock zu **kämpfen** hat.
> EN: It gets interesting because correctness comes wrapped in three constraints that the lock has to **struggle** with.

**bremsen** · `T1` — to slow down, brake
> DE: Reader dürfen den Writer nicht **bremsen**.
> EN: Readers must not **slow down** the writer.

**blockieren** · `T1` — to block
> DE: Kann ein Reader den Writer warten lassen, haben wir einen unwichtigen Thread den wichtigsten **blockieren** lassen.
> EN: If a reader can make the writer wait, we have let an unimportant thread **block** the most important one.

**entstehen** · entstand · entstanden · `T1` — to arise, come about
> DE: Sie haben keinen logischen Konflikt — Lesen ist teilbar —, also ist jeder Aufwand, der *nur deshalb **entsteht**, weil es andere Reader gibt*, reine Verschwendung.
> EN: They have no logical conflict — reading is shareable — so any cost that *arises only because other readers **exist*** is pure waste.

**messen** · maß · gemessen · `T1` — to measure
> DE: Auf der Exchange lebt er in einem Latenzbudget pro Order, **gemessen** in Mikrosekunden.
> EN: On the exchange it lives within a per-order latency budget, **measured** in microseconds.

**pausieren** · `T1` — to pause
> DE: Er kann nicht für eine Heap-Allocation **pausieren**, und er kann nicht ohne Obergrenze erneut versuchen.
> EN: It cannot **pause** for a heap allocation, and it cannot retry without an upper bound.

**erfüllen** · `T1` — to satisfy, fulfill, meet
> DE: Halte diese drei gegen jeden Kandidaten weiter unten; jeder **erfüllt** die Korrektheit und bricht eines davon.
> EN: Hold these three against every candidate below; each **satisfies** correctness and breaks one of them.

**brechen** · brach · gebrochen · `T1` — to break — here violate a constraint
> DE: Halte diese drei gegen jeden Kandidaten weiter unten; jeder erfüllt die Korrektheit und **bricht** eines davon.
> EN: Hold these three against every candidate below; each satisfies correctness and **breaks** one of them.

**übersehen** · übersah · übersehen · `T1` — to overlook, miss
> DE: Die Asymmetrie, die die Mutex-Sichtweise **übersieht**
> EN: The asymmetry that the mutex view **overlooks**

**existieren** · `T1` — to exist
> DE: Ein Mutex **existiert**, um zu lösen: „Viele Parteien *modifizieren* alle, also müssen sie sich abwechseln."
> EN: A mutex **exists** to solve: "Many parties all *modify*, so they have to take turns."

**modifizieren** · `T1` — to modify
> DE: Hier aber **modifiziert** nur eine Partei.
> EN: Here, though, only one party **modifies**.

**sich abwechseln** · trennb. · `T1` — to take turns, alternate
> DE: Ein Mutex existiert, um zu lösen: „Viele Parteien *modifizieren* alle, also müssen sie sich **abwechseln**."
> EN: A mutex exists to solve: "Many parties all *modify*, so they have to **take turns**."

**bestehen** · bestand · bestanden · `T1` — to exist, subsist — 'bestehen zwischen' = exist between
> DE: Der einzige echte Konflikt **besteht** zwischen dem Writer und einem Reader, und er ist auf drei Weisen asymmetrisch:
> EN: The only real conflict **is** between the writer and a reader, and it is asymmetric in three ways:

**übertreffen** · übertraf · übertroffen · `T1` — to exceed, outnumber
> DE: Reads **übertreffen** Writes um Größenordnungen.
> EN: Reads **outnumber** writes by orders of magnitude.

**optimieren** · `T1` — to optimize
> DE: Den Write-Pfad zu **optimieren** heißt, das Falsche zu optimieren.
> EN: To **optimize** the write path is to optimize the wrong thing.

**stillhalten** · hielt still · stillgehalten, trennb. · `T1` — to hold/keep still
> DE: Der Reader braucht den Wert nicht **stillgehalten**.
> EN: The reader does not need the value **held still**.

**anfassen** · trennb. · `T1` — to touch
> DE: Er modifiziert nichts, also braucht er kein „**Fasst** das nicht an, während ich arbeite."
> EN: It modifies nothing, so it needs no "**Don't touch** this while I'm working."

**losziehen** · zog los · losgezogen, trennb. · `T1` — to set off, head off
> DE: Er braucht einen gültigen Snapshot, dann **zieht** er los und rechnet auf diesem Snapshot; dass sich der Wert einen Augenblick später ändert, ist in Ordnung.
> EN: It needs a valid snapshot, then it **goes** off and computes on that snapshot; that the value changes a moment later is fine.

**einfrieren** · fror ein · eingefroren, trennb. · `T1` — to freeze — here participle 'eingefroren' = frozen
> DE: Weil er read-only ist, braucht er *einen* Snapshot, den es einmal gegeben hat — nicht den *neuesten* und keinen **eingefrorenen**.
> EN: Because it is read-only, it needs *a* snapshot that existed at one point — not the *latest* and not a **frozen** one.

**wiederholen** · `T1` — to repeat, redo
> DE: Der Reader kann seine Arbeit **wiederholen**.
> EN: The reader can **repeat** its work.

**herauskommen** · kam heraus · herausgekommen, trennb. · `T1` — to come out
> DE: **Kommt** ein Read verstümmelt heraus, kostet nochmaliges Lesen nichts — es gibt keinen Seiteneffekt zurückzurollen.
> EN: **If** a read comes out garbled, reading again costs nothing — there is no side effect to roll back.

**zurückrollen** · trennb. · `T1` — to roll back, undo
> DE: Kommt ein Read verstümmelt heraus, kostet nochmaliges Lesen nichts — es gibt keinen Seiteneffekt **zurückzurollen**.
> EN: If a read comes out garbled, reading again costs nothing — there is no side effect to **roll back**.

**gewähren** · `T1` — to grant
> DE: Ein Mutex bezahlt für eine stärkere Garantie, als wir brauchen: Er **gewährt** *exklusiven Besitz*, um den der Reader hier nie gebeten hat.
> EN: A mutex pays for a stronger guarantee than we need: it **grants** *exclusive ownership*, which the reader here never asked for.

**bitten um** · bat · gebeten · `T1` — to ask for, request
> DE: Ein Mutex bezahlt für eine stärkere Garantie, als wir brauchen: Er gewährt *exklusiven Besitz*, um den der Reader hier nie **gebeten** hat.
> EN: A mutex pays for a stronger guarantee than we need: it grants *exclusive ownership*, which the reader here never **asked** for.

**sich leisten** · `T1` — to afford
> DE: Und der Reader bezahlt diese Garantie in der einen Währung, die wir uns nicht **leisten** können — er muss in gemeinsamen Speicher schreiben, um den Lock zu nehmen.
> EN: And the reader pays for that guarantee in the one currency we cannot **afford** — it has to write to shared memory to take the lock.

**reinlassen** · ließ rein · reingelassen, trennb. · `T1` — to let in (colloquial for hereinlassen)
> DE: Warum also kein `RwLock`? Er **lässt** Reader doch schon gemeinsam rein
> EN: So why not an `RwLock`? It already **lets** readers in together

**erkennen** · erkannte · erkannt · `T1` — to recognize, tell, discern
> DE: Um Reader gemeinsam reinzulassen, muss der Lock wissen, wie viele Reader gerade drin sind, damit er **erkennt**, wann es sicher ist, einen Writer zuzulassen.
> EN: To let readers in together, the lock has to know how many readers are currently in, so it can **tell** when it is safe to admit a writer.

**zulassen** · ließ zu · zugelassen, trennb. · `T1` — to admit, allow in
> DE: Um Reader gemeinsam reinzulassen, muss der Lock wissen, wie viele Reader gerade drin sind, damit er erkennt, wann es sicher ist, einen Writer **zuzulassen**.
> EN: To let readers in together, the lock has to know how many readers are currently in, so it can tell when it is safe to **admit** a writer.

**erhöhen** · `T1` — to increment, raise
> DE: Das zu wissen heißt: Jeder Reader **erhöht** beim Eintreten einen gemeinsamen Zähler und verringert ihn beim Verlassen:
> EN: To know that means: every reader **increments** a shared counter on entering and decrements it on leaving:

**verringern** · `T1` — to decrement, reduce
> DE: Das zu wissen heißt: Jeder Reader erhöht beim Eintreten einen gemeinsamen Zähler und **verringert** ihn beim Verlassen:
> EN: To know that means: every reader increments a shared counter on entering and **decrements** it on leaving:

**verbringen** · verbrachte · verbracht · `T1` — to spend (time)
> DE: Also **verbringen** zweiunddreißig Reader auf zweiunddreißig Cores, ganz ohne jeden logischen Konflikt, ihre Zeit damit, eine Line zwischen sich hin- und herzuschieben:
> EN: So thirty-two readers on thirty-two cores, with no logical conflict whatsoever, **spend** their time shuffling a line back and forth between themselves:

**sich serialisieren** · `T1` — to serialize — become forced into sequential order
> DE: Lesen soll teilbar sein, und hier ist es *alles andere als das* — die Reader **serialisieren** sich auf Metadaten, die der Lock nur braucht, um zu existieren.
> EN: Reading is supposed to be shareable, and here it is *anything but that* — the readers **serialize** on metadata the lock needs only in order to exist.

**verletzen** · `T1` — to violate, breach
> DE: Schlimmer noch, der Reader blockiert weiterhin den Writer: Solange irgendein Reader die Read-Seite hält, wartet der Writer, was ebenfalls das erste Constraint **verletzt**.
> EN: Worse still, the reader continues to block the writer: as long as any reader holds the read side, the writer waits, which likewise **violates** the first constraint.

**tauschen** · `T1` — to swap, exchange
> DE: Und warum nicht einen Pointer **tauschen**? (`ArcSwap`, RCU)
> EN: And why not **swap** a pointer? (`ArcSwap`, RCU)

**überschreiben** · überschrieb · überschrieben · `T1` — to overwrite
> DE: **Überschreib** nicht an Ort und Stelle — bau den neuen Wert woanders und kipp dann einen einzelnen Pointer darauf um.
> EN: Don't **overwrite** in place — build the new value elsewhere and then flip a single pointer onto it.

**umkippen** · kippte um · umgekippt, trennb. · `T1` — to tip over — here 'flip' a pointer
> DE: Überschreib nicht an Ort und Stelle — bau den neuen Wert woanders und **kipp** dann einen einzelnen Pointer darauf um.
> EN: Don't overwrite in place — build the new value elsewhere and then **flip** a single pointer onto it.

**verschieben** · verschob · verschoben · `T1` — to shift, move, postpone
> DE: Aber es **verschiebt** den schweren Teil, statt ihn zu beseitigen.
> EN: But it **relocates** the hard part instead of eliminating it.

**beseitigen** · `T1` — to eliminate, remove, get rid of
> DE: Aber es verschiebt den schweren Teil, statt ihn zu **beseitigen**.
> EN: But it relocates the hard part instead of **eliminating** it.

**freigeben** · gab frei · freigegeben, trennb. · `T1` — to free, release (memory)
> DE: Wann ist es sicher, **freizugeben**?
> EN: When is it safe to **free** it?

**ankündigen** · trennb. · `T1` — to announce
> DE: Der Writer muss wissen, ob irgendein Reader noch den alten Pointer hält — was heißt, der Reader muss erneut *seine Anwesenheit **ankündigen*** (ein Reference Count, eine Epoche, ein Hazard Pointer).
> EN: The writer has to know whether any reader still holds the old pointer — which means the reader has to once again ***announce** its presence* (a reference count, an epoch, a hazard pointer).

**verwalten** · `T1` — to manage, administer
> DE: Wir sind zurück bei Readern, die gemeinsamen Zustand schreiben, plus einer Allocation bei jedem Write und einem Reclamation-Problem, das zu **verwalten** ist.
> EN: We are back to readers writing shared state, plus an allocation on every write and a reclamation problem to **manage**.

**klingen** · klang · geklungen · `T1` — to sound, seem
> DE: Das **klingt** unmöglich: Wenn der Writer sich nie mit Readern koordiniert, was hält einen Reader davon ab, einen halb geschriebenen Wert zu lesen?
> EN: That **sounds** impossible: if the writer never coordinates with readers, what keeps a reader from reading a half-written value?

**sich koordinieren** · `T1` — to coordinate (mit = with)
> DE: Das klingt unmöglich: Wenn der Writer sich nie mit Readern **koordiniert**, was hält einen Reader davon ab, einen halb geschriebenen Wert zu lesen?
> EN: That sounds impossible: if the writer never **coordinates** with readers, what keeps a reader from reading a half-written value?

**abhalten von** · hielt ab · abgehalten, trennb. · `T1` — to stop/prevent (sb.) from — 'jdn. davon abhalten'
> DE: Das klingt unmöglich: Wenn der Writer sich nie mit Readern koordiniert, was **hält** einen Reader davon ab, einen halb geschriebenen Wert zu lesen?
> EN: That sounds impossible: if the writer never coordinates with readers, what **keeps** a reader from reading a half-written value?

**verhindern** · `T1` — to prevent
> DE: Der Kniff also — die ganze Idee eines SeqLock — ist, gar nicht erst zu *versuchen*, es zu **verhindern**, und stattdessen den Reader das Chaos lesen zu lassen und es dann *bemerken* zu lassen.
> EN: So the trick — the whole idea of a SeqLock — is to not even *try* to **prevent** it, and instead let the reader read the chaos and then *notice* it.

**bemerken** · `T1` — to notice
> DE: Der Kniff also — die ganze Idee eines SeqLock — ist, gar nicht erst zu *versuchen*, es zu verhindern, und stattdessen den Reader das Chaos lesen zu lassen und es dann **bemerken** zu lassen.
> EN: So the trick — the whole idea of a SeqLock — is to not even *try* to prevent it, and instead let the reader read the chaos and then **notice** it.

**abfangen** · fing ab · abgefangen, trennb. · `T1` — to catch, intercept
> DE: *Weiter: [Teil 2 — Die Wette: lass es zerreißen und **fang** es ab](02_the_bet.md) · [Index](00_index.md)*
> EN: *Next: [Part 2 — The bet: let it tear and **catch** it](02_the_bet.md) · [Index](00_index.md)*

**verlangen** · `T2` — to demand, require
> DE: Die Randbedingungen **verlangten** einen unsichtbaren Reader: einen, der nichts Gemeinsames schreibt, gegen einen Writer, der sich verhält, als gäbe es keinen Reader.
> EN: The constraints **demanded** an invisible reader: one that writes nothing shared, against a writer that behaves as if there were no reader.

**sich verhalten** · verhielt · verhalten · `T2` — to behave, act
> DE: Die Randbedingungen verlangten einen unsichtbaren Reader: einen, der nichts Gemeinsames schreibt, gegen einen Writer, der sich **verhält**, als gäbe es keinen Reader.
> EN: The constraints demanded an invisible reader: one that writes nothing shared, against a writer that **behaves** as if there were no reader.

**kooperieren** · `T2` — to cooperate, coordinate
> DE: Wenn der Writer nicht **kooperiert**, hält nichts einen Reader davon ab, einen halb geschriebenen Wert zu sehen.
> EN: If the writer doesn't **cooperate**, nothing stops a reader from seeing a half-written value.

**abhalten** · hielt ab · abgehalten (sep.); jdn von etw abhalten · `T2` — to keep / prevent (sb) from (doing)
> DE: Wenn der Writer nicht kooperiert, **hält nichts einen Reader davon ab**, einen halb geschriebenen Wert zu sehen.
> EN: If the writer doesn't cooperate, **nothing stops a reader** from seeing a half-written value.

**reduzieren** · `T2` — to reduce, boil down
> DE: Es **reduziert** das ganze Problem auf eine einzige Frage:
> EN: It **reduces** the whole problem to a single question:

**durchspielen** · spielte durch · durchgespielt (sep.) · `T2` — to trace / run through, play out
> DE: **Spiel** es **durch**, und es fällt auseinander.
> EN: **Play** it **through**, and it falls apart.

**auseinanderfallen** · fiel auseinander · auseinandergefallen (sep.) · `T2` — to fall apart, collapse
> DE: Spiel es durch, und es **fällt auseinander**.
> EN: Play it through, and it **falls apart**.

**passieren** · (hat) — here: to pass (through), not 'to happen' · `T2` — to pass through, get past
> DE: Der Reader prüft nie erneut; er hat das Tor bereits **passiert**.
> EN: The reader never checks again; it has already **passed** the gate.

**davongehen** · ging davon · davongegangen (sep.) · `T2` — to walk away, leave (with)
> DE: Er **geht** mit einem Wert **davon**, der halb alt, halb neu ist — und das Flag stand `false`, in beiden Momenten, in denen es darauf ankam.
> EN: It **walks away** with a value that is half old, half new — and the flag read `false`, at both moments when it mattered.

**wegschauen** · schaute weg · weggeschaut (sep.) · `T2` — to look away
> DE: „Gerade schreibt niemand" und „jemand hat geschrieben, während du **weggeschaut** hast" sind derselbe Wert — `false`.
> EN: "No one is writing right now" and "someone wrote while you **looked away**" are the same value — `false`.

**kopieren** · `T2` — to copy; das Kopieren = the copying
> DE: Der Zähler muss also zusätzlich in seinem Wert codieren „gerade läuft ein Write", und der Reader muss sich weigern, überhaupt mit dem **Kopieren** zu beginnen, wenn er das sieht.
> EN: So the counter must additionally encode in its value "a write is running right now," and the reader must refuse to even begin **copying** when it sees that.

**schließen** · schloss · geschlossen — here: to conclude / infer · `T2` — to conclude, infer (auf etw. schließen)
> DE: Der Reader muss den Detektor also *zweimal* abtasten — einmal vor dem Kopieren, einmal danach — und nur dann **schließen** „kein Write hat mich überlappt", wenn die beiden Stichproben übereinstimmen.
> EN: So the reader must sample the detector *twice* — once before copying, once after — and only then **conclude** "no write overlapped me" when the two samples agree.

**übereinstimmen** · stimmte überein · übereingestimmt (sep.) · `T2` — to agree, match, coincide
> DE: Der Reader muss den Detektor also *zweimal* abtasten — einmal vor dem Kopieren, einmal danach — und nur dann schließen „kein Write hat mich überlappt", wenn die beiden Stichproben **übereinstimmen**.
> EN: So the reader must sample the detector *twice* — once before copying, once after — and only then conclude "no write overlapped me" when the two samples **agree**.

**berühren** · `T2` — to touch
> DE: Jedes Mal, wenn der Writer ihn **berührt**, muss er einen Wert annehmen, den er noch nie zuvor hatte.
> EN: Every time the writer **touches** it, it must take on a value it has never had before.

**annehmen** · nahm an · angenommen (sep.) — here: to take on / assume (a value) · `T2` — to take on, assume (a value); become
> DE: Jedes Mal, wenn der Writer ihn berührt, muss er einen Wert **annehmen**, den er noch nie zuvor hatte.
> EN: Every time the writer touches it, it must **take on** a value it has never had before.

**umschalten** · schaltete um · umgeschaltet (sep.) · `T2` — to toggle, switch over
> DE: Würde er nur **umschalten**, könnten zwei Stichproben zufällig übereinstimmen — der Writer kippte ihn um und wieder zurück, während der Reader kopierte, und der Reader sieht an beiden Enden denselben Wert und schließt fälschlich, es sei nichts geschehen.
> EN: If it only **toggled**, two samples could agree by chance — the writer flipped it over and back while the reader was copying, and the reader sees the same value at both ends and wrongly concludes nothing happened.

**sich wiederholen** · `T2` — to repeat itself, recur
> DE: Ein Wert, der sich nie **wiederholt**, schließt diesen Zufall aus.
> EN: A value that never **repeats** rules out this coincidence.

**eintreffen** · traf ein · eingetroffen (sep.) · `T2` — to arrive
> DE: Es fängt kein Write, das bereits *in Arbeit* war, als der Reader **eintraf**: Der Zähler könnte die ganze Zeit unverändert auf demselben Wert stehen, während der Payload doch durchweg unterwegs war.
> EN: It catches no write that was already *in progress* when the reader **arrived**: the counter could stand unchanged at the same value the whole time, while the payload was in flight all along.

**codieren** · `T2` — to encode
> DE: Der Zähler muss also zusätzlich in seinem Wert **codieren** „gerade läuft ein Write", und der Reader muss sich weigern, überhaupt mit dem Kopieren zu beginnen, wenn er das sieht.
> EN: So the counter must additionally **encode** in its value "a write is running right now," and the reader must refuse to even begin copying when it sees that.

**sich weigern** · `T2` — to refuse
> DE: Der Zähler muss also zusätzlich in seinem Wert codieren „gerade läuft ein Write", und der Reader muss sich **weigern**, überhaupt mit dem Kopieren zu beginnen, wenn er das sieht.
> EN: So the counter must additionally encode in its value "a write is running right now," and the reader must **refuse** to even begin copying when it sees that.

**erledigen** · `T2` — to do, handle, accomplish
> DE: Ein einzelnes Inkrement **erledigt** beide Aufgaben: Es kippt die Parität um (ungerade verkündet also „schreibe gerade") und es erzeugt eine nie zuvor gesehene Zahl (zwei gleiche gerade Stichproben beweisen also „dazwischen ist nichts geschehen").
> EN: A single increment **does** both jobs: it flips the parity (so odd announces "currently writing") and it produces a number never before seen (so two equal even samples prove "nothing happened in between").

**verkünden** · `T2` — to announce, proclaim
> DE: Ein einzelnes Inkrement erledigt beide Aufgaben: Es kippt die Parität um (ungerade **verkündet** also „schreibe gerade") und es erzeugt eine nie zuvor gesehene Zahl (zwei gleiche gerade Stichproben beweisen also „dazwischen ist nichts geschehen").
> EN: A single increment does both jobs: it flips the parity (so odd **announces** "currently writing") and it produces a number never before seen (so two equal even samples prove "nothing happened in between").

**erzeugen** · `T2` — to produce, generate, create
> DE: Ein einzelnes Inkrement erledigt beide Aufgaben: Es kippt die Parität um (ungerade verkündet also „schreibe gerade") und es **erzeugt** eine nie zuvor gesehene Zahl (zwei gleiche gerade Stichproben beweisen also „dazwischen ist nichts geschehen").
> EN: A single increment does both jobs: it flips the parity (so odd announces "currently writing") and it **produces** a number never before seen (so two equal even samples prove "nothing happened in between").

**beweisen** · bewies · bewiesen · `T2` — to prove
> DE: Ein einzelnes Inkrement erledigt beide Aufgaben: Es kippt die Parität um (ungerade verkündet also „schreibe gerade") und es erzeugt eine nie zuvor gesehene Zahl (zwei gleiche gerade Stichproben **beweisen** also „dazwischen ist nichts geschehen").
> EN: A single increment does both jobs: it flips the parity (so odd announces "currently writing") and it produces a number never before seen (so two equal even samples **prove** "nothing happened in between").

**inkrementieren** · `T2` — to increment
> DE: Der Writer **inkrementiert** einmal beim Betreten — gerade zu ungerade — und einmal beim Verlassen — ungerade zu gerade.
> EN: The writer **increments** once on entry — even to odd — and once on exit — odd to even.

**betreten** · betrat · betreten — hier: das Betreten = entering · `T2` — to enter; das Betreten = entry / the way in
> DE: Der Writer inkrementiert einmal beim **Betreten** — gerade zu ungerade — und einmal beim Verlassen — ungerade zu gerade.
> EN: The writer increments once on **entry** — even to odd — and once on exit — odd to even.

**verlassen** · verließ · verlassen — hier: das Verlassen = leaving · `T2` — to leave; das Verlassen = exit / the way out
> DE: Der Writer inkrementiert einmal beim Betreten — gerade zu ungerade — und einmal beim **Verlassen** — ungerade zu gerade.
> EN: The writer increments once on entry — even to odd — and once on **exit** — odd to even.

**einklammern** · klammerte ein · eingeklammert (sep.) · `T2` — to bracket, put in parentheses; enclose
> DE: ![Das Ungerade/Gerade-Protokoll: der Writer **klammert** den Write mit zwei Inkrementen **ein**; der Reader tastet den Zähler vorher und nachher ab](../img/cards/protocol.png)
> EN: ![The odd/even protocol: the writer **brackets** the write with two increments; the reader samples the counter before and after](../img/cards/protocol.png)

**versprechen** · versprach · versprochen · `T2` — to promise
> DE: Der Writer **verspricht**: Der Payload wird nur je berührt, während der Zähler ungerade ist.
> EN: The writer **promises**: the payload is only ever touched while the counter is odd.

**vertrauen** · (+ Dat.) · `T2` — to trust
> DE: Der Reader prüft zwei Dinge und **vertraut** seiner Kopie nur, wenn beide gelten — der Zähler war gerade, als er begann (kein Write in Arbeit), und es war der gleiche gerade Wert, als er fertig war (kein Write hat dazwischen begonnen und geendet).
> EN: The reader checks two things and **trusts** its copy only if both hold — the counter was even when it began (no write in progress), and it was the same even value when it finished (no write began and ended in between).

**gelten** · galt · gegolten · `T2` — to hold, be valid / true, apply
> DE: Der Reader prüft zwei Dinge und vertraut seiner Kopie nur, wenn beide **gelten** — der Zähler war gerade, als er begann (kein Write in Arbeit), und es war der gleiche gerade Wert, als er fertig war (kein Write hat dazwischen begonnen und geendet).
> EN: The reader checks two things and trusts its copy only if both **hold** — the counter was even when it began (no write in progress), and it was the same even value when it finished (no write began and ended in between).

**durchgehen** · ging durch · durchgegangen (sep.) — hier: to walk/go through · `T2` — to go / walk through, review step by step
> DE: **Geh** die beiden gefährlichen Verzahnungen **durch** und sieh, wie beide gefangen werden:
> EN: **Walk through** the two dangerous interleavings and see how both are caught:

**beachten** · `T2` — to note, heed, pay attention to
> DE: **Beachte**, was der Reader nie tut: Er schreibt niemals gemeinsamen Speicher.
> EN: **Notice** what the reader never does: it never writes shared memory.

**zurückgewinnen** · gewann zurück · zurückgewonnen (sep.) · `T2` — to reclaim, recover, regain
> DE: Es gibt keinen alten Wert **zurückzugewinnen**, denn der Writer hat nie einen neuen erzeugt; er hat an Ort und Stelle überschrieben.
> EN: There is no old value **to recover**, because the writer never produced a new one; it overwrote in place.

**gelangen** · (sein) · `T2` — to reach, get to, arrive at
> DE: Jede Randbedingung aus Teil 1 ist erfüllt, und wir sind dorthin **gelangt**, indem wir das Zerreißen umarmt haben, statt es zu bekämpfen.
> EN: Every constraint from Part 1 is satisfied, and we **got** there by embracing the tearing rather than fighting it.

**umarmen** · `T2` — to embrace, hug
> DE: Jede Randbedingung aus Teil 1 ist erfüllt, und wir sind dorthin gelangt, indem wir das Zerreißen **umarmt** haben, statt es zu bekämpfen.
> EN: Every constraint from Part 1 is satisfied, and we got there by **embracing** the tearing rather than fighting it.

**bekämpfen** · `T2` — to fight, combat, oppose
> DE: Jede Randbedingung aus Teil 1 ist erfüllt, und wir sind dorthin gelangt, indem wir das Zerreißen umarmt haben, statt es zu **bekämpfen**.
> EN: Every constraint from Part 1 is satisfied, and we got there by embracing the tearing rather than **fighting** it.

**aufgehen** · ging auf · aufgegangen (sep.) — hier: die Rechnung geht auf · `T2` — to work out, come out right (like a calculation)
> DE: Die Logik ist vollständig, und auf dem Papier **geht** jeder Fall **auf**.
> EN: The logic is complete, and on paper every case **works out**.

**ausführen** · führte aus · ausgeführt (sep.) · `T2` — to execute, carry out, run
> DE: Also hier der unangenehme Teil: Genau dieses Protokoll, auf die naheliegende Weise geschrieben, zerreißt trotzdem — nicht weil die Logik falsch ist, sondern weil die Maschine darunter deine Instruktionen nicht in der Reihenfolge **ausführt**, in der du sie geschrieben hast.
> EN: So here's the uncomfortable part: this very protocol, written the obvious way, still tears — not because the logic is wrong, but because the machine underneath doesn't **execute** your instructions in the order you wrote them.

**hinbekommen** · bekam hin · hingebekommen (sep.) · `T2` — to manage, pull off, get right
> DE: *Weiter: [Teil 3 — Das Memory-Ordering richtig **hinbekommen**](03_memory_ordering.md) · [Index](00_index.md)*
> EN: *Next: [Part 3 — **Getting** the memory ordering right](03_memory_ordering.md) · [Index](00_index.md)*

**trauen** · (+ Dat.) · `T3` — to trust — takes the dative
> DE: Der Reader liest den Zähler, kopiert den payload, liest den Zähler erneut und **traut** der Kopie nur, wenn beide Reads übereinstimmten und gerade waren.
> EN: The reader reads the counter, copies the payload, reads the counter again, and **trusts** the copy only if both reads matched and were even.

**aufsteigen** · stieg auf · aufgestiegen — trennb. · `T3` — to rise up, ascend — separable; here the copy floats up past a boundary
> DE: ③ Die Kopie des Readers **steigt** über `s1` **auf**, oder ④ sinkt unter `s2` — validiert und dann neu gelesen.
> EN: ③ The reader's copy **rises up** above `s1`, or ④ sinks below `s2` — validated and then re-read.

**sinken** · sank · gesunken · `T3` — to sink, drop
> DE: ③ Die Kopie des Readers steigt über `s1` auf, oder ④ **sinkt** unter `s2` — validiert und dann neu gelesen.
> EN: ③ The reader's copy rises up above `s1`, or ④ **sinks** below `s2` — validated and then re-read.

**festnageln** · trennb. · `T3` — to nail down, pin down — figurative here (fix in place)
> DE: Um den payload **festzunageln**, greifen wir zu `Release` und `Acquire`.
> EN: To **nail down** the payload, we reach for `Release` and `Acquire`.

**überdachen** · `T3` — to roof over, cover — figurative here
> DE: Aber ein `Acquire` **überdacht**, was *nach* `s2` kommt; die Kopie liegt *davor*, ungedeckt.
> EN: But an `Acquire` **roofs over** what comes *after* `s2`; the copy lies *before* it, uncovered.

**aufgreifen** · griff auf · aufgegriffen — trennb. · `T3` — to pick up, take up — separable
> DE: **Greift** die Kopie des Readers auch nur **ein einziges Byte** von Write N **auf**, dann **muss** der `s2`-Read des Readers den Bump auf ungerade von Write N beobachten.
> EN: If the reader's copy **picks up** even **a single byte** of Write N, then the reader's `s2` read **must** observe Write N's bump to odd.

**verknüpfen** · `T3` — to link, tie together
> DE: Das ist auch der Grund, warum ein Ordering-auf-der-Operation nicht genügen würde, selbst dort, wo es typecheckt: Ein Ordering auf einem Atomic **verknüpft** *dieses Atomic* über Threads hinweg, aber hier ist der Datenkanal der **payload**, und das, worüber wir synchronisieren, ist die **seq** — zwei verschiedene Variablen.
> EN: That is also the reason why an ordering-on-the-operation would not suffice, even where it typechecks: an ordering on an atomic **links** *this atomic* across threads, but here the data channel is the **payload**, and what we synchronize over is the **seq** — two different variables.

**begehen** · beging · begangen · `T3` — to commit (a crime, a mistake)
> DE: Was bleibt, ist ein subtileres Verbrechen, das wir die ganze Zeit **begangen** haben: Der Reader hat Bytes gelesen, die der Writer gerade aktiv verändert, und in Rusts Memory-Model ist das nicht bloß „Müll lesen“ — es ist Undefined Behaviour.
> EN: What remains is a subtler crime that we've **committed** the whole time: the reader has read bytes that the writer is actively changing right now, and in Rust's memory model that is not merely "reading garbage" — it is Undefined Behaviour.

**genügen** · (+ Dat.) · `T3` — to suffice, be enough — governs the dative (jemandem/einer Sache genügen)
> DE: Gleiche Seite — ein Ordering auf dem Atomic selbst **genügt**.
> EN: Same side — an ordering on the atomic itself **suffices**.

**verriegeln** · `T3` — to bolt, lock (a door)
> DE: Wir haben gerade eine Tür **verriegelt**, durch die niemand geht, und Fluchtweg ① steht immer noch sperrangelweit offen.
> EN: We just **bolted** a door that no one walks through, and escape route ① still stands wide open.

**beobachten** · `T3` — to observe, watch
> DE: Greift die Kopie des Readers auch nur **ein einziges Byte** von Write N auf, dann **muss** der `s2`-Read des Readers den Bump auf ungerade von Write N **beobachten**.
> EN: If the reader's copy picks up even **a single byte** of Write N, then the reader's `s2` read **must** **observe** Write N's bump to odd.

**verbieten** · verbot · verboten · `T4` — to forbid, prohibit
> DE: Der Read, den die Sprache **verbietet**
> EN: The Read that the language **forbids**

**übersetzen** · `T4` — to compile — literally 'to translate'; here = compile
> DE: Im Kernel funktioniert es, weil der Kernel von einem bekannten Compiler mit bekannten Flags **übersetzt** wird; es ist ein Handel, der mit einer bestimmten Implementierung geschlossen wird, nicht mit der Sprache.
> EN: In the kernel it works because the kernel is **compiled** by a known compiler with known flags; it is a deal struck with a particular implementation, not with the language.

**sich vertragen** · vertrug · vertragen · `T4` — to get along, be compatible
> DE: (Hans Boehm hat ein ganzes Paper über genau dieses Missverhältnis geschrieben: seqlocks und Speichermodelle von Sprachen **vertragen** sich nicht, es sei denn, die Sprache gibt einem ein hinreichend billiges Atomic.)
> EN: (Hans Boehm wrote an entire paper about exactly this mismatch: seqlocks and language memory models don't **get along**, unless the language gives you a sufficiently cheap Atomic.)

**durchlaufen** · durchlief · durchlaufen · `T4` — to traverse, walk/step through
> DE: Hardware hat kein 40-Byte-Atomic, aber ein 8-Byte-Atomic, also **durchlaufen** wir den Wert ein `usize`-Wort nach dem anderen, jedes Wort ein `Relaxed`-Load oder -Store:
> EN: Hardware has no 40-byte Atomic, but an 8-byte Atomic, so we **walk** the value one `usize` word at a time, each word a `Relaxed` Load or Store:

**hinzufügen** · fügte hinzu · hinzugefügt (trennb.) · `T4` — to add
> DE: Was es **hinzufügt**, ist Legalität: Ein Atomzugriff, der gegen einen anderen Atomzugriff rennt, ist *kein* Data Race, also kein UB.
> EN: What it **adds** is legality: an atomic access that races against another atomic access is *not* a Data Race, hence not UB.

**verwerfen** · verwarf · verworfen · `T4` — to discard, reject, throw out
> DE: Um ein beliebiges `T` als eine Reihe von `usize`-Wörtern umzudeuten, muss `T` tatsächlich schlichte Bytes *sein* — kein Padding, jedes Bitmuster gültig (der Leser wird halb geschriebene Mischungen beobachten, bevor er sie **verwirft**), ein definiertes Layout.
> EN: To reinterpret an arbitrary `T` as a series of `usize` words, `T` must actually *be* plain bytes — no padding, every bit pattern valid (the reader will observe half-written mixtures before **discarding** them), a defined layout.

**lokalisieren** · `T4` — to localise, pin down to one place
> DE: Es macht Korrektheit nicht automatisch; es **lokalisiert** die Beweispflicht auf eine greppbare Zeile und lässt versehentlichen Missbrauch (ein `String`, ein Typ mit Padding) an der Kompilierung scheitern.
> EN: It does not make correctness automatic; it **localizes** the burden of proof to a greppable line and makes accidental misuse (a `String`, a type with padding) fail at compilation.

**ausgehen von** · ging aus · ausgegangen (trennb.) · `T4` — to assume, presuppose, proceed on the basis of
> DE: Das Protokoll ging bisher von einem einzigen Schreiber **aus**.
> EN: The protocol has so far **assumed** a single writer.

**verschränken** · `T4` — to interleave, intertwine
> DE: Echter Code hat mehrere — und wenn zwei Threads gleichzeitig `store` aufrufen, erhöhen beide den Zähler und **verschränken** ihre Payload-Writes, und ein Leser kann das Durcheinander akzeptieren.
> EN: Real code has several — and when two threads call `store` simultaneously, both increment the counter and **interleave** their payload writes, and a reader can accept the mess.

**abstürzen** · stürzte ab · abgestürzt (trennb.) · `T4` — to crash (of a program)
> DE: Nichts **stürzt ab** (jeder Zugriff ist jetzt atomar, also ist es kein UB — bloß falsch), aber es ist falsch.
> EN: Nothing **crashes** (every access is now atomic, so it is not UB — just wrong), but it is wrong.

**übernehmen** · übernahm · übernommen · `T4` — to take on, assume (a task/role)
> DE: Der Sequenzzähler **übernimmt** eine zweite Aufgabe: Er wird zum Lock der Schreiber.
> EN: The sequence counter **takes on** a second job: it becomes the writers' lock.

**gelingen** · gelang · gelungen (+ Dat.) · `T4` — to succeed, work out
> DE: Von gerade → ungerade zu erhöhen ist kein blindes Inkrement mehr, sondern ein compare-and-swap, das nur *von einem geraden Wert aus* **gelingt**.
> EN: Incrementing from even → odd is no longer a blind increment, but a compare-and-swap that **succeeds** only *from an even value*.

**zusehen** · sah zu · zugesehen (trennb.) · `T4` — to watch, look on
> DE: Wir haben schon **zugesehen**, wie dieser Code vier von fünf Malen durchläuft, während er falsch ist.
> EN: We have already **watched** this code pass four out of five times while it is wrong.

**benennen** · benannte · benannt · `T4` — to name, identify by name
> DE: Dann ist jeder Load, dessen Wörter sich unterscheiden, von Konstruktion her ein torn read — ein „zerrissener Read" —, und die Assertion **benennt** ihn:
> EN: Then any Load whose words differ is by construction a torn read — a "torn Read" — and the assertion **names** it:

**erkaufen** · `T4` — to buy at a cost, gain at a price
> DE: All das — das Zerreißen, die fences, die Atomics, die `Pod`-Anforderung — **erkauft** eine einzige Sache: einen Lesepfad, der flach bleibt, während sich Leser häufen, dort wo ein `RwLock` einbricht.
> EN: All of this — the tearing, the fences, the Atomics, the `Pod` requirement — **buys** a single thing: a read path that stays flat while readers pile up, where an `RwLock` collapses.

**sich häufen** · `T4` — to pile up, accumulate, multiply
> DE: All das — das Zerreißen, die fences, die Atomics, die `Pod`-Anforderung — erkauft eine einzige Sache: einen Lesepfad, der flach bleibt, während sich Leser **häufen**, dort wo ein `RwLock` einbricht.
> EN: All of this — the tearing, the fences, the Atomics, the `Pod` requirement — buys a single thing: a read path that stays flat while readers **pile up**, where an `RwLock` collapses.

**einbrechen** · brach ein · eingebrochen (trennb.) · `T4` — to collapse, break down, cave in
> DE: All das — das Zerreißen, die fences, die Atomics, die `Pod`-Anforderung — erkauft eine einzige Sache: einen Lesepfad, der flach bleibt, während sich Leser häufen, dort wo ein `RwLock` **einbricht**.
> EN: All of this — the tearing, the fences, the Atomics, the `Pod` requirement — buys a single thing: a read path that stays flat while readers pile up, where an `RwLock` **collapses**.

**klettern** · `T4` — to climb, clamber — here: (of a number) to climb, rise
> DE: Die eigentliche Geschichte ist die Form: Kommen Leser hinzu, bleibt SeqLock flach — 0,75 ns bei einem, ~1,5 ns bei acht — während `RwLock` fast linear auf 680 ns **klettert**, weil jeder Leser diesen gemeinsamen Zähler unaufhörlich schreibt und seine cache line herumspringen lässt.
> EN: The real story is the shape: as readers are added, SeqLock stays flat — 0.75 ns with one, ~1.5 ns with eight — while `RwLock` **climbs** almost linearly to 680 ns, because every reader incessantly writes this shared counter and makes its cache line bounce around.

**auszahlen** · zahlte aus · ausgezahlt (trennb.) · `T4` — to pay out
> DE: Diese Kluft ist das MESI-Diagramm aus Teil 1, **ausgezahlt** in Nanosekunden.
> EN: This gap is the MESI diagram from Part 1, **paid out** in nanoseconds.

**ausleihen** · lieh aus · ausgeliehen (trennb.) · `T4` — to borrow — here: borrow a reference (&T)
> DE: der Leser bekommt eine *Kopie*, nie ein `&T` zum **Ausleihen**;
> EN: the reader gets a *copy*, never a `&T` to **borrow**;

**wiederverwenden** · verwendete wieder · wiederverwendet (trennb.) · `T4` — to reuse
> DE: Wer das annimmt, bekommt das Ding, das man einmal baut und überall dort **wiederverwendet**, wo ein kleiner Wert weit öfter gelesen als geschrieben wird — der chain head, der mark price, die Spitze eines order book.
> EN: Whoever accepts this gets the thing you build once and **reuse** everywhere a small value is read far more often than written — the chain head, the mark price, the top of an order book.

**zurückweisen** · wies zurück · zurückgewiesen (trennb.) · `T4` — to reject, turn down, refuse
> DE: Wer es **zurückweist**, greift zu `RwLock` und zahlt das 450-Fache, sobald acht Kerne zum ersten Mal gleichzeitig lesen.
> EN: Whoever **rejects** it reaches for `RwLock` and pays 450 times as much, as soon as eight cores read simultaneously for the first time.

## Adjektive & Adverbien — adjectives and adverbs (93)

**denkwürdig** · `T0` — memorable, notable
> DE: Jede Entscheidung danach ist erzwungen — durch einen Use Case, oder durch das Scheitern der einfacheren Alternative, oder, in einem **denkwürdigen** Fall, durch einen ARM-Prozessor, der deine Instruktionen umordnet und einen Wert korrumpiert, von dem deine Testsuite steif und fest behauptet, er sei in Ordnung.
> EN: Every decision after that is forced — by a use case, or by the failure of the simpler alternative, or, in one **memorable** case, by an ARM processor that reorders your instructions and corrupts a value that your test suite adamantly insists is fine.

**naheliegend** · `T0` — obvious, the one you'd reach for first
> DE: **[Teil 1 — Das Problem, und warum die naheliegenden Locks nicht passen.](01_the_problem.md)**
> EN: **[Part 1 — The Problem, and why the obvious locks do not fit.](01_the_problem.md)**

**unsichtbar** · `T0` — invisible
> DE: Jede korrekte Option bricht dieselbe Regel, und sie weist auf den einzigen Ausweg — der Leser muss **unsichtbar** sein.
> EN: Every correct option breaks the same rule, and it points to the only way out — the reader must be **invisible**.

**boolesch** · `T0` — boolean
> DE: Das reduziert alles auf eine einzige Frage — woher weiß ein Leser, dass er während eines Schreibvorgangs gelesen hat? — und wir leiten die Antwort auf die harte Tour her, indem wir einem **booleschen** Flag beim Scheitern zusehen, weil es keine Geschichte trägt, bis das Einzige übrig bleibt, was funktioniert: ein Zähler, der gerade ist, wenn der Wert stabil ist, und ungerade, während er geschrieben wird, vor und nach dem Lesen abgetastet.
> EN: This reduces everything to a single question — how does a reader know it read during a write? — and we derive the answer the hard way, by watching a **boolean** flag fail, because it carries no history, until the only thing left that works remains: a counter that is even when the value is stable and odd while it is being written, sampled before and after the read.

**gerade** · `T0` — even (number) — as opposed to odd
> DE: Das reduziert alles auf eine einzige Frage — woher weiß ein Leser, dass er während eines Schreibvorgangs gelesen hat? — und wir leiten die Antwort auf die harte Tour her, indem wir einem booleschen Flag beim Scheitern zusehen, weil es keine Geschichte trägt, bis das Einzige übrig bleibt, was funktioniert: ein Zähler, der **gerade** ist, wenn der Wert stabil ist, und ungerade, während er geschrieben wird, vor und nach dem Lesen abgetastet.
> EN: This reduces everything to a single question — how does a reader know it read during a write? — and we derive the answer the hard way, by watching a boolean flag fail, because it carries no history, until the only thing left that works remains: a counter that is **even** when the value is stable and odd while it is being written, sampled before and after the read.

**ungerade** · `T0` — odd (number)
> DE: Das reduziert alles auf eine einzige Frage — woher weiß ein Leser, dass er während eines Schreibvorgangs gelesen hat? — und wir leiten die Antwort auf die harte Tour her, indem wir einem booleschen Flag beim Scheitern zusehen, weil es keine Geschichte trägt, bis das Einzige übrig bleibt, was funktioniert: ein Zähler, der gerade ist, wenn der Wert stabil ist, und **ungerade**, während er geschrieben wird, vor und nach dem Lesen abgetastet.
> EN: This reduces everything to a single question — how does a reader know it read during a write? — and we derive the answer the hard way, by watching a boolean flag fail, because it carries no history, until the only thing left that works remains: a counter that is even when the value is stable and **odd** while it is being written, sampled before and after the read.

**eigenständig** · `T0` — standalone, independent, self-contained
> DE: Zwei der vier Fensterkanten kommen mit einem Ordering auf dem Atomic selbst aus; die anderen beiden brauchen einen **eigenständigen** fence — und die fences sind, wie sich herausstellt, das, was zwei Threads sich zu einer happens-before-Beziehung die Hand reichen lässt.
> EN: Two of the four window edges get by with an ordering on the atomic itself; the other two need a **standalone** fence — and the fences are, as it turns out, what lets two threads reach out their hands to one another into a happens-before relationship.

**bewusst** · `T0` — deliberate(ly), on purpose
> DE: Wir lassen den Leser **bewusst** Bytes lesen, die der Schreiber gerade ändert.
> EN: We **deliberately** let the reader read bytes that the writer is in the middle of changing.

**undefiniert** · `T0` — undefined — as in undefined behaviour
> DE: In C ist das eine Volkstradition mit `volatile`; in Rusts Memory-Modell ist es ein Data Race — **undefiniertes** Verhalten — und Miri sagt es laut und deutlich.
> EN: In C this is a folk tradition with `volatile`; in Rust's memory model it is a data race — **undefined** behavior — and Miri says so loud and clear.

**atomar** · `T0` — atomic (indivisible operation)
> DE: Die Korrektur macht jeden Zugriff auf die Payload **atomar**, Wort für Wort, und verwandelt „Müll lesen" von UB in einen legalen Read, den der Zähler wegwirft — was die Payload zwingt, `Pod` zu sein, eine Schranke, die sich als Lizenz erweist, die der Implementierer *unterschreibt*, statt einer, die der Compiler prüft, und die nicht einmal Alignment abdeckt.
> EN: The fix makes every access to the payload **atomic**, word by word, and turns "reading garbage" from UB into a legal read that the counter throws away — which forces the payload to be `Pod`, a bound that turns out to be a license the implementer *signs*, rather than one the compiler checks, and that does not even cover alignment.

**nebenläufig** · `T0` — concurrent
> DE: **loom** — ein Model Checker, der einen kleinen **nebenläufigen** Test unter jeder möglichen Thread-Verschränkung erneut ausführt; der Verifizierer für Lock-free-Code.
> EN: **loom** — a model checker that re-runs a small **concurrent** test under every possible thread interleaving; the verifier for lock-free code.

**generisch** · `T0` — generic (over a type parameter)
> DE: Diese Serie entwirft ein **generisches**, wiederverwendbares `SeqLock<T>` — die Sorte, die man in eine Concurrency-Crate legt, nicht eine Einmalvariante, fest verdrahtet auf ein einzelnes Struct.
> EN: This series designs a **generic**, reusable `SeqLock<T>` — the kind you put in a concurrency crate, not a one-off variant hard-wired to a single struct.

**kanonisch** · `T1` — canonical — the one authoritative version
> DE: Ein Blockchain-Node rückt seinen **kanonischen** Chain-Head etwa alle zwölf Sekunden vor.
> EN: A blockchain node advances its **canonical** chain head roughly every twelve seconds.

**ständig** · `T1` — constantly, continually
> DE: ein Wert, selten geschrieben, **ständig** von überall gelesen.
> EN: a value, rarely written, **constantly** read from everywhere.

**unteilbar** · `T1` — indivisible, indivisibly — as one atomic unit
> DE: Es gibt keine CPU-Instruktion, die 40 Byte **unteilbar** schreibt.
> EN: There is no CPU instruction that writes 40 bytes **indivisibly**.

**veraltet** · `T1` — stale, outdated
> DE: Ein Reader, der in diesem Fenster landet, bekommt keinen **veralteten** Wert.
> EN: A reader that lands in this window does not get a **stale** value.

**überlebbar** · `T1` — survivable
> DE: Veraltet wäre **überlebbar** — „ein paar Millisekunden hinterher" ist in Ordnung.
> EN: Stale would be **survivable** — "a few milliseconds behind" is fine.

**dazwischen** · `T1` — in between (adv.)
> DE: Die Risk-Engine liest `mark_price` aus Tick N und, ein paar Nanosekunden später, `funding_index` aus Tick N+1, weil der Writer **dazwischen** beide aktualisiert hat.
> EN: The risk engine reads `mark_price` from tick N and, a few nanoseconds later, `funding_index` from tick N+1, because the writer updated both **in between**.

**strukturell** · `T1` — structurally — by their very structure
> DE: Das ist echtes Geld, verloren an einen Konsistenz-Bug, den Atomics pro Feld **strukturell** nicht fangen können.
> EN: That is real money, lost to a consistency bug that per-field atomics **structurally** cannot catch.

**stets** · `T1` — always, at all times (formal, adv.)
> DE: Veröffentliche einen Snapshot aus mehreren Feldern so, dass jeder Reader **stets** einen Snapshot sieht, den es als Ganzes tatsächlich gegeben hat.
> EN: Publish a snapshot made of several fields such that every reader **always** sees a snapshot that, as a whole, actually existed.

**tatsächlich** · `T1` — actually, in fact (adv.)
> DE: Veröffentliche einen Snapshot aus mehreren Feldern so, dass jeder Reader stets einen Snapshot sieht, den es als Ganzes **tatsächlich** gegeben hat.
> EN: Publish a snapshot made of several fields such that every reader always sees a snapshot that, as a whole, **actually** existed.

**drumherum** · `T1` — around it (colloquial, adv.)
> DE: Wäre Korrektheit die einzige Anforderung, wäre dies ein gelöstes und langweiliges Problem — ein Lock **drumherum**, und Feierabend.
> EN: Were correctness the only requirement, this would be a solved and boring problem — a lock **around it**, and done for the day.

**kritisch** · `T1` — critical — 'critical path'
> DE: Der Engine-Thread, der den Chain-Head vorrückt, ist der **kritische** Pfad des Nodes; der Oracle-Thread, der den Mark-Price aktualisiert, ebenso.
> EN: The engine thread that advances the chain head is the node's **critical** path; the oracle thread that updates the mark price, likewise.

**ebenso** · `T1` — likewise, too (adv.)
> DE: Der Engine-Thread, der den Chain-Head vorrückt, ist der kritische Pfad des Nodes; der Oracle-Thread, der den Mark-Price aktualisiert, **ebenso**.
> EN: The engine thread that advances the chain head is the node's critical path; the oracle thread that updates the mark price, **likewise**.

**unwichtig** · `T1` — unimportant
> DE: Kann ein Reader den Writer warten lassen, haben wir einen **unwichtigen** Thread den wichtigsten blockieren lassen.
> EN: If a reader can make the writer wait, we have let an **unimportant** thread block the most important one.

**gegenseitig** · `T1` — mutually, each other
> DE: Reader dürfen sich nicht **gegenseitig** bremsen.
> EN: Readers must not slow **each other** down.

**logisch** · `T1` — logical
> DE: Sie haben keinen **logischen** Konflikt — Lesen ist teilbar —, also ist jeder Aufwand, der *nur deshalb entsteht, weil es andere Reader gibt*, reine Verschwendung.
> EN: They have no **logical** conflict — reading is shareable — so any cost that arises *only because other readers exist* is pure waste.

**teilbar** · `T1` — divisible, shareable
> DE: Sie haben keinen logischen Konflikt — Lesen ist **teilbar** —, also ist jeder Aufwand, der *nur deshalb entsteht, weil es andere Reader gibt*, reine Verschwendung.
> EN: They have no logical conflict — reading is **shareable** — so any cost that arises *only because other readers exist* is pure waste.

**rein** · `T1` — pure, sheer
> DE: Sie haben keinen logischen Konflikt — Lesen ist teilbar —, also ist jeder Aufwand, der *nur deshalb entsteht, weil es andere Reader gibt*, **reine** Verschwendung.
> EN: They have no logical conflict — reading is shareable — so any cost that arises *only because other readers exist* is **pure** waste.

**beschränkt** · `T1` — bounded, limited
> DE: Der Read-Pfad muss **beschränkt** und allokationsfrei sein.
> EN: The read path must be **bounded** and allocation-free.

**allokationsfrei** · `T1` — allocation-free — performing no heap allocation
> DE: Der Read-Pfad muss beschränkt und **allokationsfrei** sein.
> EN: The read path must be bounded and **allocation-free**.

**erneut** · `T1` — again, anew (adv.)
> DE: Er kann nicht für eine Heap-Allocation pausieren, und er kann nicht ohne Obergrenze **erneut** versuchen.
> EN: It cannot pause for a heap allocation, and it cannot **retry** without an upper bound.

**klassisch** · `T1` — classic, classical
> DE: Hier ist die Beobachtung, um die sich das ganze Design dreht, und sie ist der Grund, warum dies *nicht* das **klassische** Mutual-Exclusion-Problem ist.
> EN: Here is the observation the whole design turns on, and it is the reason this is *not* the **classic** mutual-exclusion problem.

**echt** · `T1` — genuine, real
> DE: Der einzige **echte** Konflikt besteht zwischen dem Writer und einem Reader, und er ist auf drei Weisen asymmetrisch:
> EN: The only **real** conflict is between the writer and a reader, and it is asymmetric in three ways:

**asymmetrisch** · `T1` — asymmetric
> DE: Der einzige echte Konflikt besteht zwischen dem Writer und einem Reader, und er ist auf drei Weisen **asymmetrisch**:
> EN: The only real conflict is between the writer and a reader, and it is **asymmetric** in three ways:

**gültig** · `T1` — valid
> DE: Er braucht einen **gültigen** Snapshot, dann zieht er los und rechnet auf diesem Snapshot; dass sich der Wert einen Augenblick später ändert, ist in Ordnung.
> EN: It needs a **valid** snapshot, then it goes off and computes on that snapshot; that the value changes a moment later is fine.

**neueste** · `T1` — latest, newest (superlative of neu)
> DE: Weil er read-only ist, braucht er *einen* Snapshot, den es einmal gegeben hat — nicht den **neuesten** und keinen *eingefrorenen*.
> EN: Because it is read-only, it needs *a* snapshot that existed at one point — not the **latest** and not a *frozen* one.

**verstümmelt** · `T1` — garbled, mangled
> DE: Kommt ein Read **verstümmelt** heraus, kostet nochmaliges Lesen nichts — es gibt keinen Seiteneffekt zurückzurollen.
> EN: If a read comes out **garbled**, reading again costs nothing — there is no side effect to roll back.

**nochmalig** · `T1` — repeated, renewed — 'doing it once more'
> DE: Kommt ein Read verstümmelt heraus, kostet **nochmaliges** Lesen nichts — es gibt keinen Seiteneffekt zurückzurollen.
> EN: If a read comes out garbled, reading **again** costs nothing — there is no side effect to roll back.

**exklusiv** · `T1` — exclusive
> DE: Ein Mutex bezahlt für eine stärkere Garantie, als wir brauchen: Er gewährt ***exklusiven** Besitz*, um den der Reader hier nie gebeten hat.
> EN: A mutex pays for a stronger guarantee than we need: it grants ***exclusive** ownership*, which the reader here never asked for.

**gemeinsam** · `T1` — shared, common, joint
> DE: Und der Reader bezahlt diese Garantie in der einen Währung, die wir uns nicht leisten können — er muss in **gemeinsamen** Speicher schreiben, um den Lock zu nehmen.
> EN: And the reader pays for that guarantee in the one currency we cannot afford — it has to write to **shared** memory to take the lock.

**gleichzeitig** · `T1` — simultaneously, at the same time (adv.)
> DE: Mehrere Reader halten die Read-Seite **gleichzeitig**.
> EN: Multiple readers hold the read side **simultaneously**.

**erledigt** · `T1` — done, settled, finished
> DE: Ist die Sache damit nicht **erledigt**?
> EN: Isn't that the matter **settled**?

**umsonst** · `T1` — for free; in vain (adv.)
> DE: Nein — denn „lässt sie gemeinsam rein" ist ein Versprechen, das das *Interface* gibt und das die *Implementierung* nicht **umsonst** halten kann.
> EN: No — because "lets them in together" is a promise the *interface* makes and one the *implementation* cannot keep **for free**.

**drin** · `T1` — inside (colloquial for darin, adv.)
> DE: Um Reader gemeinsam reinzulassen, muss der Lock wissen, wie viele Reader gerade **drin** sind, damit er erkennt, wann es sicher ist, einen Writer zuzulassen.
> EN: To let readers in together, the lock has to know how many readers are currently **in**, so it can tell when it is safe to admit a writer.

**physisch** · `T1` — physical, physically
> DE: **Physisch** schon.
> EN: **Physically**, they are.

**weiterhin** · `T1` — still, continuing to (adv.)
> DE: Schlimmer noch, der Reader blockiert **weiterhin** den Writer: Solange irgendein Reader die Read-Seite hält, wartet der Writer, was ebenfalls das erste Constraint verletzt.
> EN: Worse still, the reader **continues** to block the writer: as long as any reader holds the read side, the writer waits, which likewise violates the first constraint.

**irgendein** · `T1` — any, some (or other)
> DE: Schlimmer noch, der Reader blockiert weiterhin den Writer: Solange **irgendein** Reader die Read-Seite hält, wartet der Writer, was ebenfalls das erste Constraint verletzt.
> EN: Worse still, the reader continues to block the writer: as long as **any** reader holds the read side, the writer waits, which likewise violates the first constraint.

**ebenfalls** · `T1` — also, likewise, as well (adv.)
> DE: Schlimmer noch, der Reader blockiert weiterhin den Writer: Solange irgendein Reader die Read-Seite hält, wartet der Writer, was **ebenfalls** das erste Constraint verletzt.
> EN: Worse still, the reader continues to block the writer: as long as any reader holds the read side, the writer waits, which **likewise** violates the first constraint.

**clever** · `T1` — clever
> DE: Es gibt eine **cleverere** Familie, die das Tearing komplett umgeht.
> EN: There is a **cleverer** family that sidesteps tearing entirely.

**woanders** · `T1` — somewhere else, elsewhere (adv.)
> DE: Überschreib nicht an Ort und Stelle — bau den neuen Wert **woanders** und kipp dann einen einzelnen Pointer darauf um.
> EN: Don't overwrite in place — build the new value **elsewhere** and then flip a single pointer onto it.

**manche** · `T1` — some
> DE: Sobald der Writer den Pointer umgekippt hat, lesen **manche** Reader vielleicht noch den alten Wert.
> EN: Once the writer has flipped the pointer, **some** readers may still be reading the old value.

**zugrunde liegend** · `T1` — underlying, root
> DE: Korrekt und oft die Sache wert — aber es bricht dieselben Constraints, aus demselben **zugrunde liegenden** Grund.
> EN: Correct and often worth it — but it breaks the same constraints, for the same **underlying** reason.

**feldübergreifend** · `T1` — cross-field — spanning multiple fields
> DE: keine **feldübergreifende** Konsistenz — das erfundene Paar
> EN: no **cross-field** consistency — the fabricated pair

**überhaupt** · `T1` — at all (adv., intensifier)
> DE: Der Reader muss unsichtbar sein — er darf keinen gemeinsamen Speicher schreiben, und der Writer muss laufen, als gäbe es **überhaupt** keinen Reader.
> EN: The reader must be invisible — it must write no shared memory, and the writer must run as if there were **no** reader at all.

**unmöglich** · `T1` — impossible
> DE: Das klingt **unmöglich**: Wenn der Writer sich nie mit Readern koordiniert, was hält einen Reader davon ab, einen halb geschriebenen Wert zu lesen?
> EN: That sounds **impossible**: if the writer never coordinates with readers, what keeps a reader from reading a half-written value?

**gratis** · `T2` — free of charge, for free — here adverbial
> DE: Der Reader ist read-only und kann seine Arbeit **gratis** wiederholen (die dritte Asymmetrie aus Teil 1), ein vergeudeter Read kostet also nichts als ein wenig Zeit.
> EN: The reader is read-only and can repeat its work **for free** (the third asymmetry from Part 1), so a wasted read costs nothing but a little time.

**vergeudet** · `T2` — wasted, squandered — part. of vergeuden
> DE: Der Reader ist read-only und kann seine Arbeit gratis wiederholen (die dritte Asymmetrie aus Teil 1), ein **vergeudeter** Read kostet also nichts als ein wenig Zeit.
> EN: The reader is read-only and can repeat its work for free (the third asymmetry from Part 1), so a **wasted** read costs nothing but a little time.

**aktuell** · `T2` — current, present — false friend: NOT 'actual/actually'
> DE: Ein Flag kann dir den **aktuellen** Zustand nennen; es kann dir nicht sagen, ob sich der Zustand *geändert* hat, während du beschäftigt warst.
> EN: A flag can tell you the **current** state; it can't tell you whether the state has *changed* while you were busy.

**vollständig** · `T2` — complete(ly), full
> DE: Die Logik ist **vollständig**, und auf dem Papier geht jeder Fall auf.
> EN: The logic is **complete**, and on paper every case works out.

**zerrissen** · `T2` — torn, ripped — part. of zerreißen
> DE: ![Ein einzelnes writing-Flag: der Reader kann trotzdem einen vollständig **zerrissenen** Wert lesen](../img/cards/bool_flag.png)
> EN: ![A single writing flag: the reader can still read a fully **torn** value](../img/cards/bool_flag.png)

**beschäftigt** · `T2` — busy, occupied
> DE: Und „hat er sich geändert, während ich **beschäftigt** war" ist genau die Frage.
> EN: And "did it change while I was **busy**" is exactly the question.

**zufällig** · `T2` — by chance, coincidental(ly), random
> DE: Würde er nur umschalten, könnten zwei Stichproben **zufällig** übereinstimmen — der Writer kippte ihn um und wieder zurück, während der Reader kopierte, und der Reader sieht an beiden Enden denselben Wert und schließt fälschlich, es sei nichts geschehen.
> EN: If it only toggled, two samples could agree **by chance** — the writer flipped it over and back while the reader was copying, and the reader sees the same value at both ends and wrongly concludes nothing happened.

**fälschlich** · `T2` — wrongly, mistakenly, falsely — adverbial
> DE: Würde er nur umschalten, könnten zwei Stichproben zufällig übereinstimmen — der Writer kippte ihn um und wieder zurück, während der Reader kopierte, und der Reader sieht an beiden Enden denselben Wert und schließt **fälschlich**, es sei nichts geschehen.
> EN: If it only toggled, two samples could agree by chance — the writer flipped it over and back while the reader was copying, and the reader sees the same value at both ends and **wrongly** concludes nothing happened.

**unverändert** · `T2` — unchanged
> DE: Es fängt kein Write, das bereits *in Arbeit* war, als der Reader eintraf: Der Zähler könnte die ganze Zeit **unverändert** auf demselben Wert stehen, während der Payload doch durchweg unterwegs war.
> EN: It catches no write that was already *in progress* when the reader arrived: the counter could stand **unchanged** at the same value the whole time, while the payload was in flight all along.

**unterwegs** · `T2` — on the way, in transit — here: 'mid-flight'
> DE: Es fängt kein Write, das bereits *in Arbeit* war, als der Reader eintraf: Der Zähler könnte die ganze Zeit unverändert auf demselben Wert stehen, während der Payload doch durchweg **unterwegs** war.
> EN: It catches no write that was already *in progress* when the reader arrived: the counter could stand unchanged at the same value the whole time, while the payload was **in flight** all along.

**stabil** · `T2` — stable, steady
> DE: Lass den Zähler gerade sein, wenn der Wert **stabil** ist, und ungerade, während ein Write läuft.
> EN: Let the counter be even when the value is **stable**, and odd while a write is running.

**unangenehm** · `T2` — unpleasant, uncomfortable, awkward
> DE: Also hier der **unangenehme** Teil: Genau dieses Protokoll, auf die naheliegende Weise geschrieben, zerreißt trotzdem — nicht weil die Logik falsch ist, sondern weil die Maschine darunter deine Instruktionen nicht in der Reihenfolge ausführt, in der du sie geschrieben hast.
> EN: So here's the **uncomfortable** part: this very protocol, written the obvious way, still tears — not because the logic is wrong, but because the machine underneath doesn't execute your instructions in the order you wrote them.

**wasserdicht** · `T3` — watertight; figuratively airtight, foolproof
> DE: Auf dem Papier **wasserdicht**.
> EN: On paper, **watertight**.

**unbeschränkt** · `T3` — unrestricted, unbounded, unconstrained
> DE: `Relaxed` bedeutet genau das, was es sagt: Die Operation ist atomar, und ihre Reihenfolge relativ zu allem drumherum ist **unbeschränkt**.
> EN: `Relaxed` means exactly what it says: the operation is atomic, and its order relative to everything around it is **unconstrained**.

**einseitig** · `T3` — one-sided, one-way
> DE: Jedes ist ein **einseitiges** Gate, und es bewacht nur eine Seite der Operation, an der es hängt.
> EN: Each is a **one-sided** gate, and it guards only one side of the operation it hangs on.

**unbewacht** · `T3` — unguarded
> DE: Der rote Pfeil in jedem Panel ist die Richtung, die *nicht* bewacht wird — und genau diese **unbewachte** Richtung ist die, die alle vergessen.
> EN: The red arrow in each panel is the direction that is *not* guarded — and exactly this **unguarded** direction is the one everyone forgets.

**verlockend** · `T3` — tempting, enticing
> DE: Der **verlockende** Fix ist, `s2` zu einem `Acquire`-Load zu machen.
> EN: The **tempting** fix is to make `s2` an `Acquire` load.

**ungedeckt** · `T3` — uncovered, unprotected
> DE: Aber ein `Acquire` überdacht, was *nach* `s2` kommt; die Kopie liegt *davor*, **ungedeckt**.
> EN: But an `Acquire` roofs over what comes *after* `s2`; the copy lies *before* it, **uncovered**.

**subtil** · `T3` — subtle
> DE: Was bleibt, ist ein **subtileres** Verbrechen, das wir die ganze Zeit begangen haben: Der Reader hat Bytes gelesen, die der Writer gerade aktiv verändert, und in Rusts Memory-Model ist das nicht bloß „Müll lesen“ — es ist Undefined Behaviour.
> EN: What remains is a **subtler** crime that we've been committing the whole time: the reader has read bytes that the writer is actively changing right now, and in Rust's memory model that is not merely "reading garbage" — it is Undefined Behaviour.

**umgekehrt** · `T3` — reversed, the other way round; conversely
> DE: Gleiches Wort, **umgekehrte** Wirkung — denn eine fence ist eine Wand, die du positionierst, während ein Ordering-auf-einer-Operation ein einseitiges Gate ist, das an diese Operation geklebt ist.
> EN: Same word, **opposite** effect — because a fence is a wall that you position, while an ordering-on-an-operation is a one-sided gate glued to that operation.

**schlicht** · `T4` — plain, simple, unadorned
> DE: Hier ist der natürliche Weg, das Payload herauszukopieren — ein **schlichter** Read durch einen raw pointer:
> EN: Here is the natural way to copy the payload out — a **plain** Read through a raw pointer:

**sorgfältig** · `T4` — careful, meticulous
> DE: Die **sorgfältige** `s1 == s2`-Prüfung kann von einem Compiler wegoptimiert werden, der unter der No-Data-Race-Annahme *bewiesen* hat, dass sie immer wahr ist.
> EN: The **careful** `s1 == s2` check can be optimized away by a compiler that, under the no-data-race assumption, has *proven* that it is always true.

**unumwunden** · `T4` — bluntly, plainly, in no uncertain terms (as adv.)
> DE: Miri — ein Interpreter, der den Code gegen das Speichermodell laufen lässt — sagt es **unumwunden**:
> EN: Miri — an interpreter that runs the code against the memory model — says it **plainly**:

**hinreichend** · `T4` — sufficient(ly), enough — formal register
> DE: (Hans Boehm hat ein ganzes Paper über genau dieses Missverhältnis geschrieben: seqlocks und Speichermodelle von Sprachen vertragen sich nicht, es sei denn, die Sprache gibt einem ein **hinreichend** billiges Atomic.)
> EN: (Hans Boehm wrote an entire paper about exactly this mismatch: seqlocks and language memory models don't get along, unless the language gives you a **sufficiently** cheap Atomic.)

**weithin** · `T4` — widely, far and wide (as adv.)
> DE: `Relaxed` ist der Schlüssel, und es wird **weithin** missverstanden.
> EN: `Relaxed` is the key, and it is **widely** misunderstood.

**beliebig** · `T4` — arbitrary, any (whatsoever)
> DE: Um ein **beliebiges** `T` als eine Reihe von `usize`-Wörtern umzudeuten, muss `T` tatsächlich schlichte Bytes *sein* — kein Padding, jedes Bitmuster gültig (der Leser wird halb geschriebene Mischungen beobachten, bevor er sie verwirft), ein definiertes Layout.
> EN: To reinterpret an **arbitrary** `T` as a series of `usize` words, `T` must actually *be* plain bytes — no padding, every bit pattern valid (the reader will observe half-written mixtures before discarding them), a defined layout.

**versehentlich** · `T4` — accidental(ly), inadvertent(ly)
> DE: Es macht Korrektheit nicht automatisch; es lokalisiert die Beweispflicht auf eine greppbare Zeile und lässt **versehentlichen** Missbrauch (ein `String`, ein Typ mit Padding) an der Kompilierung scheitern.
> EN: It does not make correctness automatic; it localizes the burden of proof to a greppable line and makes **accidental** misuse (a `String`, a type with padding) fail at compilation.

**womöglich** · `T4` — possibly, perhaps (as adv.)
> DE: `u8` ist ein völlig ehrliches `Pod`, und `SeqLock<u8>` geht trotzdem kaputt: Ein Byte ist kein ganzes `usize`-Wort, und das Payload ist **womöglich** nicht wortausgerichtet für den atomaren Load.
> EN: `u8` is a perfectly honest `Pod`, and `SeqLock<u8>` breaks anyway: a byte is not a whole `usize` word, and the payload is **possibly** not word-aligned for the atomic Load.

**wortausgerichtet** · `T4` — word-aligned
> DE: `u8` ist ein völlig ehrliches `Pod`, und `SeqLock<u8>` geht trotzdem kaputt: Ein Byte ist kein ganzes `usize`-Wort, und das Payload ist womöglich nicht **wortausgerichtet** für den atomaren Load.
> EN: `u8` is a perfectly honest `Pod`, and `SeqLock<u8>` breaks anyway: a byte is not a whole `usize` word, and the payload is possibly not **word-aligned** for the atomic Load.

**bequem** · `T4` — convenient, comfortable
> DE: Ein einziger Schreiber war eine **bequeme** Lüge
> EN: A single writer was a **convenient** lie

**belegt** · `T4` — occupied, taken (from belegen)
> DE: Ungerade bedeutet schon „ein Write ist im Gange"; jetzt bedeutet es auch „der Write-Slot ist **belegt**".
> EN: Odd already means "a Write is in progress"; now it also means "the Write slot is **occupied**".

**ahnungslos** · `T4` — oblivious, clueless, unaware
> DE: Schreiber serialisieren sich; Leser bleiben lock-free und **ahnungslos**.
> EN: Writers serialize themselves; readers stay lock-free and **oblivious**.

**bedeutungslos** · `T4` — meaningless, insignificant
> DE: In lock-free Code ist ein grüner Test für sich genommen nahezu **bedeutungslos**; Korrektheit ist eine Eigenschaft *jeder* Verschränkung, und ein Test übt ein paar zufällige aus.
> EN: In lock-free code, a green test on its own is nearly **meaningless**; correctness is a property of *every* interleaving, and a test exercises a few random ones.

**ausschließlich** · `T4` — exclusively, only
> DE: Lass den Schreiber **ausschließlich** `[n, n, n, n]` veröffentlichen — vier identische Wörter.
> EN: Have the writer publish **exclusively** `[n, n, n, n]` — four identical words.

**erschöpfend** · `T4` — exhaustive, thorough
> DE: Wo der torn-read-Test ein paar Schedules abtastet, ist loom über ein beschränktes Modell **erschöpfend**; es ist das Nächste an einem Beweis, dass die fences richtig platziert sind.
> EN: Where the torn-read test samples a few schedules, loom is **exhaustive** over a bounded model; it is the closest thing to a proof that the fences are placed correctly.

**flach** · `T4` — flat, level (here: latency that doesn't rise)
> DE: All das — das Zerreißen, die fences, die Atomics, die `Pod`-Anforderung — erkauft eine einzige Sache: einen Lesepfad, der **flach** bleibt, während sich Leser häufen, dort wo ein `RwLock` einbricht.
> EN: All of this — the tearing, the fences, the Atomics, the `Pod` requirement — buys a single thing: a read path that stays **flat** while readers pile up, where an `RwLock` collapses.

**unaufhörlich** · `T4` — incessantly, ceaselessly (as adv.)
> DE: Die eigentliche Geschichte ist die Form: Kommen Leser hinzu, bleibt SeqLock flach — 0,75 ns bei einem, ~1,5 ns bei acht — während `RwLock` fast linear auf 680 ns klettert, weil jeder Leser diesen gemeinsamen Zähler **unaufhörlich** schreibt und seine cache line herumspringen lässt.
> EN: The real story is the shape: as readers are added, SeqLock stays flat — 0.75 ns with one, ~1.5 ns with eight — while `RwLock` climbs almost linearly to 680 ns, because every reader **incessantly** writes this shared counter and makes its cache line bounce around.

## Redewendungen & feste Wendungen — idioms and set phrases (44)

**von Grund auf** · `T0` — from scratch, from the ground up
> DE: Diese Serie entwirft eines **von Grund auf**.
> EN: This series designs one **from the ground up**.

**eine Wette eingehen** · ging ein · eingegangen (sep.) · `T0` — to make/take a bet
> DE: Wir beginnen bei dem Problem, für dessen Lösung ein SeqLock existiert, sehen jedem Lock, zu dem man normalerweise greifen würde, dabei zu, wie es an genau einer der Randbedingungen scheitert, und gehen dann **die Wette ein**, die das ganze Primitive definiert: Statt den Leser daran zu hindern, einen halb geschriebenen Wert zu beobachten, lassen wir es geschehen und bringen den Leser dazu, es zu erkennen.
> EN: We start from the problem that a SeqLock exists to solve, watch each lock you would normally reach for fail at exactly one of the constraints, and then **make the bet** that defines the whole primitive: Instead of preventing the reader from observing a half-written value, we let it happen and get the reader to detect it.

**steif und fest behaupten** · `T0` — to insist adamantly, swear blind
> DE: Jede Entscheidung danach ist erzwungen — durch einen Use Case, oder durch das Scheitern der einfacheren Alternative, oder, in einem denkwürdigen Fall, durch einen ARM-Prozessor, der deine Instruktionen umordnet und einen Wert korrumpiert, von dem deine Testsuite **steif und fest behauptet**, er sei in Ordnung.
> EN: Every decision after that is forced — by a use case, or by the failure of the simpler alternative, or, in one memorable case, by an ARM processor that reorders your instructions and corrupts a value that your test suite **adamantly insists** is fine.

**in Konflikt stehen** · `T0` — to be in conflict, clash
> DE: Die drei Randbedingungen, die das schwer machen, und dann die Tour der Fehlschläge: Ein `RwLock` lässt Leser gemeinsam hinein, zwingt aber jeden von ihnen, einen gemeinsamen Zähler zu *schreiben*, und serialisiert damit Cores, die gar nicht **in Konflikt stehen**; `ArcSwap` und RCU umgehen das Zerreißen, ziehen den Leser aber zurück hinein, sich selbst zur Reclamation anzumelden.
> EN: The three constraints that make this hard, and then the tour of failures: An `RwLock` lets readers in together, but forces each of them to *write* a shared counter, and thereby serializes cores that are not **in conflict** at all; `ArcSwap` and RCU sidestep the tearing, but pull the reader back in to register itself for reclamation.

**auf die harte Tour** · `T0` — the hard way
> DE: Das reduziert alles auf eine einzige Frage — woher weiß ein Leser, dass er während eines Schreibvorgangs gelesen hat? — und wir leiten die Antwort **auf die harte Tour** her, indem wir einem booleschen Flag beim Scheitern zusehen, weil es keine Geschichte trägt, bis das Einzige übrig bleibt, was funktioniert: ein Zähler, der gerade ist, wenn der Wert stabil ist, und ungerade, während er geschrieben wird, vor und nach dem Lesen abgetastet.
> EN: This reduces everything to a single question — how does a reader know it read during a write? — and we derive the answer **the hard way**, by watching a boolean flag fail, because it carries no history, until the only thing left that works remains: a counter that is even when the value is stable and odd while it is being written, sampled before and after the read.

**verkehrt herum** · `T0` — backwards, the wrong way round
> DE: Wir reparieren es mit fences, und um sie zu platzieren, brauchen wir die Idee, die alle immer **verkehrt herum** verstehen: `Release` und `Acquire` sind Einweg-Gates, von denen jedes nur eine Seite der Operation bewacht, an die es geheftet ist.
> EN: We fix it with fences, and to place them we need the idea that everyone always understands **backwards**: `Release` and `Acquire` are one-way gates, each of which guards only one side of the operation it is pinned to.

**sich die Hand reichen** · `T0` — to shake hands / meet — here, two threads synchronising
> DE: Zwei der vier Fensterkanten kommen mit einem Ordering auf dem Atomic selbst aus; die anderen beiden brauchen einen eigenständigen fence — und die fences sind, wie sich herausstellt, das, was zwei Threads sich zu einer happens-before-Beziehung **die Hand reichen** lässt.
> EN: Two of the four window edges get by with an ordering on the atomic itself; the other two need a standalone fence — and the fences are, as it turns out, what lets two threads **reach out their hands** to one another into a happens-before relationship.

**laut und deutlich** · `T0` — loud and clear
> DE: In C ist das eine Volkstradition mit `volatile`; in Rusts Memory-Modell ist es ein Data Race — undefiniertes Verhalten — und Miri sagt es **laut und deutlich**.
> EN: In C this is a folk tradition with `volatile`; in Rust's memory model it is a data race — undefined behavior — and Miri says so **loud and clear**.

**auf dem Papier** · `T0` — on paper, in theory — as opposed to in practice
> DE: Das Protokoll ist **auf dem Papier** korrekt und zerreißt trotzdem auf einem echten Apple M2, vier von fünf Läufen grün — der Fingerabdruck eines Memory-Ordering-Bugs.
> EN: The protocol is correct **on paper** and still tears on a real Apple M2, four out of five runs green — the fingerprint of a memory-ordering bug.

**außer Reichweite** · `T1` — out of reach
> DE: Vierzig Byte sind **außer Reichweite**.
> EN: Forty bytes are **out of reach**.

**es hat etw. gegeben** · `T1` — there existed sth — 'es gibt' in the past; here 'den es nie gegeben hat' = that never existed
> DE: Er bekommt den Hash von Block 1000, gepaart mit der Zahl 999: einen Wert, den es **nie gegeben hat**.
> EN: It gets the hash of block 1000, paired with the number 999: a value that **never existed**.

**für sich** · `T1` — on its own, in isolation
> DE: Jeder Read war **für sich** atomar und korrekt.
> EN: Each read was atomic and correct **on its own**.

**als Ganzes** · `T1` — as a whole
> DE: Veröffentliche einen Snapshot aus mehreren Feldern so, dass jeder Reader stets einen Snapshot sieht, den es **als Ganzes** tatsächlich gegeben hat.
> EN: Publish a snapshot made of several fields such that every reader always sees a snapshot that, **as a whole**, actually existed.

**und Feierabend** · `T1` — and that's it / and you're done — colloquial 'call it a day'
> DE: Wäre Korrektheit die einzige Anforderung, wäre dies ein gelöstes und langweiliges Problem — ein Lock drumherum, und **Feierabend**.
> EN: Were correctness the only requirement, this would be a solved and boring problem — a lock around it, and **done for the day**.

**sich drehen um** · `T1` — to revolve around, center on
> DE: Hier ist die Beobachtung, um die sich das ganze Design **dreht**, und sie ist der Grund, warum dies *nicht* das klassische Mutual-Exclusion-Problem ist.
> EN: Here is the observation the whole design **turns on**, and it is the reason this is *not* the classic mutual-exclusion problem.

**im Konflikt stehen** · `T1` — to be in conflict
> DE: Logisch stehen diese Reader nicht **im Konflikt**.
> EN: Logically, these readers are not **in conflict**.

**hin- und herschieben** · `T1` — to push back and forth — here 'bouncing' a cache line
> DE: Also verbringen zweiunddreißig Reader auf zweiunddreißig Cores, ganz ohne jeden logischen Konflikt, ihre Zeit damit, eine Line zwischen sich **hin- und herzuschieben**:
> EN: So thirty-two readers on thirty-two cores, with no logical conflict whatsoever, spend their time **shuffling a line back and forth** between themselves:

**alles andere als** · `T1` — anything but, far from
> DE: Lesen soll teilbar sein, und hier ist es ***alles andere als** das* — die Reader serialisieren sich auf Metadaten, die der Lock nur braucht, um zu existieren.
> EN: Reading is supposed to be shareable, and here it is ***anything but** that* — the readers serialize on metadata the lock needs only in order to exist.

**an Ort und Stelle** · `T1` — in place, on the spot
> DE: Überschreib nicht **an Ort und Stelle** — bau den neuen Wert woanders und kipp dann einen einzelnen Pointer darauf um.
> EN: Don't overwrite **in place** — build the new value elsewhere and then flip a single pointer onto it.

**die Sache wert** · `T1` — worth it, worth the trouble
> DE: Korrekt und oft die Sache **wert** — aber es bricht dieselben Constraints, aus demselben zugrunde liegenden Grund.
> EN: Correct and often **worth** it — but it breaks the same constraints, for the same underlying reason.

**gemeinsam haben** · `T1` — to have in common
> DE: Was jeder Fehlschlag **gemeinsam hat**
> EN: What every failure **has in common**

**in eine Reihe stellen** · `T1` — to line up, put in a row
> DE: Stell die Kandidaten **in eine Reihe**:
> EN: Line the candidates **up in a row**:

**keines von beidem** · `T1` — neither of the two
> DE: Die eine Zeile, die **keines von beidem** tut, ist nicht korrekt.
> EN: The one row that does **neither** is not correct.

**deuten auf** · `T1` — to point to, indicate
> DE: Die Constraints **deuten** auf einen einzigen Schluss:
> EN: The constraints **point** to a single conclusion:

**gar nicht erst** · `T1` — not even (bother to), not in the first place
> DE: Der Kniff also — die ganze Idee eines SeqLock — ist, **gar nicht erst** zu *versuchen*, es zu verhindern, und stattdessen den Reader das Chaos lesen zu lassen und es dann *bemerken* zu lassen.
> EN: So the trick — the whole idea of a SeqLock — is to **not even** *try* to prevent it, and instead let the reader read the chaos and then *notice* it.

**in die Enge treiben** · trieb · getrieben · `T2` — to back / drive (sb) into a corner
> DE: Teil 1 hat uns **in die Enge getrieben**.
> EN: Part 1 **cornered** us.

**nichts als** · `T2` — nothing but
> DE: Der Reader ist read-only und kann seine Arbeit gratis wiederholen (die dritte Asymmetrie aus Teil 1), ein vergeudeter Read kostet also **nichts als** ein wenig Zeit.
> EN: The reader is read-only and can repeat its work for free (the third asymmetry from Part 1), so a wasted read costs **nothing but** a little time.

**darauf ankommen** · kam an · angekommen (sep.) · `T2` — to matter, be what counts
> DE: Er geht mit einem Wert davon, der halb alt, halb neu ist — und das Flag stand `false`, in beiden Momenten, in denen es **darauf ankam**.
> EN: It walks away with a value that is half old, half new — and the flag read `false`, at both moments when it **mattered**.

**noch nie zuvor** · `T2` — never before, never ever previously
> DE: Jedes Mal, wenn der Writer ihn berührt, muss er einen Wert annehmen, den er **noch nie zuvor** hatte.
> EN: Every time the writer touches it, it must take on a value it has **never before** had.

**in Arbeit** · `T2` — in progress, being worked on
> DE: Es fängt kein Write, das bereits **in Arbeit** war, als der Reader eintraf: Der Zähler könnte die ganze Zeit unverändert auf demselben Wert stehen, während der Payload doch durchweg unterwegs war.
> EN: It catches no write that was already **in progress** when the reader arrived: the counter could stand unchanged at the same value the whole time, while the payload was in flight all along.

**zu etwas greifen** · griff · gegriffen · `T3` — to reach for / resort to something
> DE: Um den payload festzunageln, **greifen wir zu** `Release` und `Acquire`.
> EN: To nail the payload down, **we reach for** `Release` and `Acquire`.

**sich die Hand geben** · gab · gegeben · `T3` — to shake hands — here: two fences synchronise
> DE: Zwei fences auf zwei Threads **geben sich die Hand** und bauen eine happens-before-Beziehung, und diese Beziehung ist die eigentliche Garantie.
> EN: Two fences on two threads **shake hands** and build a happens-before relationship, and this relationship is the actual guarantee.

**die Brücke schlagen** · schlug · geschlagen · `T3` — to build a bridge, bridge (a gap)
> DE: Nur eine fence **schlägt die Brücke** von der einen zur anderen.
> EN: Only a fence **builds the bridge** from the one to the other.

**ineinander rasten** · `T3` — to click/lock into each other, interlock
> DE: Aber wenn der Reader ein Byte liest, das der Writer *nach* seiner `fence(Release)` gespeichert hat, und der Reader nach dem Read eine `fence(Acquire)` ausführt, **rasten** die beiden fences **ineinander**: Alles vor der fence des Writers happens-before alles nach der fence des Readers.
> EN: But if the reader reads a byte that the writer stored *after* its `fence(Release)`, and the reader executes a `fence(Acquire)` after the read, the two fences **lock into each other**: everything before the writer's fence happens-before everything after the reader's fence.

**sperrangelweit offen** · `T3` — wide open — intensified 'offen' (a gap left completely unguarded)
> DE: Wir haben gerade eine Tür verriegelt, durch die niemand geht, und Fluchtweg ① steht immer noch **sperrangelweit offen**.
> EN: We just bolted a door that no one walks through, and escape route ① still stands **wide open**.

**„läuft doch auf meiner Maschine“** · `T3` — "but it works on my machine" — the classic developer excuse for a bug that only shows up elsewhere
> DE: Das ist der Bug, für den **„läuft doch auf meiner Maschine“** erfunden wurde.)
> EN: This is the bug for which **"but it runs on my machine"** was invented.)

**mit Absicht** · `T4` — on purpose, deliberately
> DE: Aber sieh zurück, was der Leser die ganze Zeit getan hat, **mit Absicht**: das Payload zu lesen, während der Schreiber es gerade aktiv überschreibt.
> EN: But look back at what the reader has been doing the whole time, **on purpose**: reading the payload while the writer is actively overwriting it.

**die Verantwortung tragen** · trug · getragen · `T4` — to bear responsibility, be accountable
> DE: Erstens ist `Pod` eine Lizenz, die der Implementierende unterschreibt, kein Fakt, den der Compiler nachprüft — `unsafe impl Pod for Foo {}` ist ein Versprechen, das man gibt und für das man die **Verantwortung trägt**; macht man es falsch, ist es UB, weshalb der Trait `unsafe` zu implementieren ist.
> EN: First, `Pod` is a license that the implementer signs, not a fact that the compiler checks — `unsafe impl Pod for Foo {}` is a promise you make and **bear responsibility for**; get it wrong and it is UB, which is why the trait is `unsafe` to implement.

**im Gange (sein)** · `T4` — to be in progress, underway, going on
> DE: Ungerade bedeutet schon „ein Write ist **im Gange**"; jetzt bedeutet es auch „der Write-Slot ist belegt".
> EN: Odd already means "a Write is **in progress**"; now it also means "the Write slot is occupied".

**dran sein** · `T4` — to be one's turn (du bist dran = it's your turn)
> DE: Eine Ganzzahl, zwei Bedeutungen, kein zusätzlicher Zustand: Dem Leser sagt ungerade „lies nicht"; einem anderen Schreiber sagt ungerade „warte, bis du **dran** bist".
> EN: One integer, two meanings, no extra state: to the reader, odd says "don't read"; to another writer, odd says "wait until it's your **turn**".

**für sich genommen** · `T4` — on its own, taken by itself
> DE: In lock-free Code ist ein grüner Test **für sich genommen** nahezu bedeutungslos; Korrektheit ist eine Eigenschaft *jeder* Verschränkung, und ein Test übt ein paar zufällige aus.
> EN: In lock-free code, a green test **on its own** is nearly meaningless; correctness is a property of *every* interleaving, and a test exercises a few random ones.

**von Konstruktion her** · `T4` — by construction, by design
> DE: Dann ist jeder Load, dessen Wörter sich unterscheiden, **von Konstruktion her** ein torn read — ein „zerrissener Read" —, und die Assertion benennt ihn:
> EN: Then any Load whose words differ is **by construction** a torn read — a "torn Read" — and the assertion names it:

**das Nächste an** · `T4` — the closest thing to
> DE: Wo der torn-read-Test ein paar Schedules abtastet, ist loom über ein beschränktes Modell erschöpfend; es ist **das Nächste an** einem Beweis, dass die fences richtig platziert sind.
> EN: Where the torn-read test samples a few schedules, loom is exhaustive over a bounded model; it is **the closest thing to** a proof that the fences are placed correctly.

**greifen zu** · griff · gegriffen · `T4` — to reach for, resort to
> DE: Wer es zurückweist, **greift zu** `RwLock` und zahlt das 450-Fache, sobald acht Kerne zum ersten Mal gleichzeitig lesen.
> EN: Whoever rejects it **reaches for** `RwLock` and pays 450 times as much, as soon as eight cores read simultaneously for the first time.

## Textstruktur & Signalwörter — discourse markers (34)

**der Reihe nach** · `T0` — in order, one after another
> DE: **Der Reihe nach** — jeder Teil beginnt dort, wo der vorige aufgehört hat, und schließt mit der Frage, die der nächste beantwortet.
> EN: **In order** — each part begins where the previous one left off and closes with the question the next one answers.

**trotzdem** · `T0` — nonetheless, even so
> DE: Das Protokoll ist auf dem Papier korrekt und zerreißt **trotzdem** auf einem echten Apple M2, vier von fünf Läufen grün — der Fingerabdruck eines Memory-Ordering-Bugs.
> EN: The protocol is correct on paper and **still** tears on a real Apple M2, four out of five runs green — the fingerprint of a memory-ordering bug.

**hinterher** · `T0` — afterward, after the fact
> DE: Wenn der Schreiber nicht aufgehalten werden kann und der Leser sich nicht anmelden kann, bleibt ein einziger Zug: den Read zerreißen lassen und dem Leser einen Weg geben, es **hinterher** zu bemerken und erneut zu versuchen.
> EN: If the writer cannot be stopped and the reader cannot register, a single move remains: let the read tear and give the reader a way to notice it **afterwards** and try again.

**wie kurz auch immer** · `T1` — however brief — concessive aside
> DE: Das heißt, es gibt *immer* ein Fenster — **wie kurz auch immer** —, in dem der Speicher einen Wert hält, der halb der alte Head und halb der neue ist.
> EN: That means there is *always* a window — **however brief** — in which memory holds a value that is half the old head and half the new.

**eigentlich** · `T1` — actually, really — 'as a matter of fact'
> DE: Das *Paar* ist ein Wert, den es nie gegeben hat, und die daraus berechnete Margin ist falsch — falsch genug, um einen Account zu liquidieren, der **eigentlich** gesund war.
> EN: The *pair* is a value that never existed, and the margin computed from it is wrong — wrong enough to liquidate an account that was **actually** healthy.

**deshalb** · `T1` — therefore, for that reason
> DE: Sie haben keinen logischen Konflikt — Lesen ist teilbar —, also ist jeder Aufwand, der *nur **deshalb** entsteht, weil es andere Reader gibt*, reine Verschwendung.
> EN: They have no logical conflict — reading is shareable — so any cost that arises *only **because** other readers exist* is pure waste.

**doch** · `T1` — after all / already — modal particle asserting against an expectation
> DE: Warum also kein `RwLock`? Er lässt Reader **doch** schon gemeinsam rein
> EN: So why not an `RwLock`? It **already** lets readers in together

**schlimmer noch** · `T1` — worse still, worse yet
> DE: **Schlimmer noch**, der Reader blockiert weiterhin den Writer: Solange irgendein Reader die Read-Seite hält, wartet der Writer, was ebenfalls das erste Constraint verletzt.
> EN: **Worse still**, the reader continues to block the writer: as long as any reader holds the read side, the writer waits, which likewise violates the first constraint.

**solange** · `T1` — as long as, while
> DE: Schlimmer noch, der Reader blockiert weiterhin den Writer: **Solange** irgendein Reader die Read-Seite hält, wartet der Writer, was ebenfalls das erste Constraint verletzt.
> EN: Worse still, the reader continues to block the writer: **as long as** any reader holds the read side, the writer waits, which likewise violates the first constraint.

**sobald** · `T1` — as soon as, once
> DE: **Sobald** der Writer den Pointer umgekippt hat, lesen manche Reader vielleicht noch den alten Wert.
> EN: **Once** the writer has flipped the pointer, some readers may still be reading the old value.

**als gäbe es** · `T1` — as if there were — Konjunktiv II hypothetical
> DE: Der Reader muss unsichtbar sein — er darf keinen gemeinsamen Speicher schreiben, und der Writer muss laufen, **als gäbe es** überhaupt keinen Reader.
> EN: The reader must be invisible — it must write no shared memory, and the writer must run **as if there were** no reader at all.

**stattdessen** · `T1` — instead
> DE: Der Kniff also — die ganze Idee eines SeqLock — ist, gar nicht erst zu *versuchen*, es zu verhindern, und **stattdessen** den Reader das Chaos lesen zu lassen und es dann *bemerken* zu lassen.
> EN: So the trick — the whole idea of a SeqLock — is to not even *try* to prevent it, and **instead** let the reader read the chaos and then *notice* it.

**im Nachhinein** · `T2` — in retrospect, after the fact
> DE: Woher weiß ein Reader **im Nachhinein**, dass er *während* eines Writes gelesen hat?
> EN: How does a reader know **in hindsight** that it read *during* a write?

**während** · `T2` — while, during — subordinating conj. of simultaneity
> DE: Der naheliegende Detektor ist ein Boolean, das der Writer setzt, **während** er arbeitet.
> EN: The obvious detector is a boolean that the writer sets **while** it works.

**Alles Weitere** · `T2` — everything else, all further matters
> DE: **Alles Weitere** folgt daraus, sie zu beantworten.
> EN: **Everything else** follows from answering it.

**daraus** · `T2` — from that / it — pronominal adverb; folgt daraus = follows from it
> DE: Alles Weitere folgt **daraus**, sie zu beantworten.
> EN: Everything else follows **from** answering it.

**bereits** · `T2` — already — more formal than 'schon'
> DE: Der Reader prüft nie erneut; er hat das Tor **bereits** passiert.
> EN: The reader never checks again; it has **already** passed the gate.

**innerhalb** · + Gen. · `T2` — within, inside (of)
> DE: *Dann* läuft ein Writer — Flag hoch, überschreiben, Flag runter — vollständig **innerhalb** des Reads.
> EN: *Then* a writer runs — flag up, overwrite, flag down — entirely **within** the read.

**zuvor** · `T2` — before, previously, earlier
> DE: Ein einzelnes Inkrement erledigt beide Aufgaben: Es kippt die Parität um (ungerade verkündet also „schreibe gerade") und es erzeugt eine nie **zuvor** gesehene Zahl (zwei gleiche gerade Stichproben beweisen also „dazwischen ist nichts geschehen").
> EN: A single increment does both jobs: it flips the parity (so odd announces "currently writing") and it produces a number never **before** seen (so two equal even samples prove "nothing happened in between").

**aufwärts** · `T2` — upward, up
> DE: Der natürliche solche Wert ist ein Zähler, der nur je **aufwärts** zählt.
> EN: The natural such value is a counter that only ever counts **upward**.

**nur je** · `T2` — only ever — 'je' as 'at any time'
> DE: Der natürliche solche Wert ist ein Zähler, der **nur je** aufwärts zählt.
> EN: The natural such value is a counter that **only ever** counts upward.

**durchweg** · `T2` — throughout, consistently, without exception
> DE: Es fängt kein Write, das bereits *in Arbeit* war, als der Reader eintraf: Der Zähler könnte die ganze Zeit unverändert auf demselben Wert stehen, während der Payload doch **durchweg** unterwegs war.
> EN: It catches no write that was already *in progress* when the reader arrived: the counter could stand unchanged at the same value the whole time, while the payload was in flight **all along**.

**zusätzlich** · `T2` — additionally, in addition, also
> DE: Der Zähler muss also **zusätzlich** in seinem Wert codieren „gerade läuft ein Write", und der Reader muss sich weigern, überhaupt mit dem Kopieren zu beginnen, wenn er das sieht.
> EN: So the counter must **additionally** encode in its value "a write is running right now," and the reader must refuse to even begin copying when it sees that.

**zugleich** · `T2` — at the same time, simultaneously
> DE: Ein einziger Integer kann beide Signale **zugleich** tragen.
> EN: A single integer can carry both signals **at once**.

**denn** · `T2` — because, for — coordinating causal conjunction (no verb-final clause)
> DE: Es gibt keinen alten Wert zurückzugewinnen, **denn** der Writer hat nie einen neuen erzeugt; er hat an Ort und Stelle überschrieben.
> EN: There is no old value to recover, **because** the writer never produced a new one; it overwrote in place.

**dorthin** · `T2` — there, to that place — directional
> DE: Jede Randbedingung aus Teil 1 ist erfüllt, und wir sind **dorthin** gelangt, indem wir das Zerreißen umarmt haben, statt es zu bekämpfen.
> EN: Every constraint from Part 1 is satisfied, and we got **there** by embracing the tearing rather than fighting it.

**indem** · `T2` — by (doing sth) — subordinating conj. of means
> DE: Jede Randbedingung aus Teil 1 ist erfüllt, und wir sind dorthin gelangt, **indem** wir das Zerreißen umarmt haben, statt es zu bekämpfen.
> EN: Every constraint from Part 1 is satisfied, and we got there **by** embracing the tearing rather than fighting it.

**statt** · + Gen. / statt zu + Inf. · `T2` — instead of
> DE: Jede Randbedingung aus Teil 1 ist erfüllt, und wir sind dorthin gelangt, indem wir das Zerreißen umarmt haben, **statt** es zu bekämpfen.
> EN: Every constraint from Part 1 is satisfied, and we got there by embracing the tearing **rather than** fighting it.

**darunter** · — hier: 'underneath, below' · `T2` — underneath, below it
> DE: Also hier der unangenehme Teil: Genau dieses Protokoll, auf die naheliegende Weise geschrieben, zerreißt trotzdem — nicht weil die Logik falsch ist, sondern weil die Maschine **darunter** deine Instruktionen nicht in der Reihenfolge ausführt, in der du sie geschrieben hast.
> EN: So here's the uncomfortable part: this very protocol, written the obvious way, still tears — not because the logic is wrong, but because the machine **underneath** doesn't execute your instructions in the order you wrote them.

**mithin** · `T3` — hence, therefore, consequently — formal
> DE: Jetzt ist sie über der fence festgenagelt, **mithin** über `s2`.
> EN: Now it is nailed down above the fence, **hence** above `s2`.

**ohnehin** · `T3` — anyway, in any case, already
> DE: Stell an jeder Kante eine Frage: *Liegt das, was ich festhalten muss, auf der Seite, die das Gate dieser Operation **ohnehin** abdeckt, oder auf der Gegenseite?*
> EN: Ask a question at every edge: *Does what I need to hold fast lie on the side that this operation's gate covers **anyway**, or on the opposite side?*

**bloß** · `T3` — merely, just — limiting particle
> DE: Was bleibt, ist ein subtileres Verbrechen, das wir die ganze Zeit begangen haben: Der Reader hat Bytes gelesen, die der Writer gerade aktiv verändert, und in Rusts Memory-Model ist das nicht **bloß** „Müll lesen“ — es ist Undefined Behaviour.
> EN: What remains is a subtler crime that we've been committing the whole time: the reader has read bytes that the writer is actively changing right now, and in Rust's memory model that is not **merely** "reading garbage" — it is Undefined Behaviour.

**es sei denn** · `T4` — unless
> DE: (Hans Boehm hat ein ganzes Paper über genau dieses Missverhältnis geschrieben: seqlocks und Speichermodelle von Sprachen vertragen sich nicht, **es sei denn**, die Sprache gibt einem ein hinreichend billiges Atomic.)
> EN: (Hans Boehm wrote an entire paper about exactly this mismatch: seqlocks and language memory models don't get along, **unless** the language gives you a sufficiently cheap Atomic.)

**sodass** · `T4` — so that (introduces a result)
> DE: `Relaxed` verhindert das Zerreißen nicht; es macht das Zerreißen *legal*, **sodass** der Zähler seine Arbeit tun darf, statt dass der Compiler um ein Rennen herum falsch kompiliert.
> EN: `Relaxed` does not prevent the tearing; it makes the tearing *legal*, **so that** the counter is allowed to do its job, instead of the compiler miscompiling around a race.

## Englische Lehnwörter — English loanwords and the gender German gives them (82)

**die Payload** · die, -s · `T0` — payload — the guarded value the SeqLock protects
> DE: Der Zähler baut ein Fenster; noch zwingt nichts die **Payload**, darin zu bleiben.
> EN: The counter builds a window; nothing yet forces the **payload** to stay inside it.

**der Fence** · der, -s · `T0` — fence — a standalone memory-ordering barrier
> DE: Zwei der vier Fensterkanten kommen mit einem Ordering auf dem Atomic selbst aus; die anderen beiden brauchen einen eigenständigen **fence** — und die fences sind, wie sich herausstellt, das, was zwei Threads sich zu einer happens-before-Beziehung die Hand reichen lässt.
> EN: Two of the four window edges get by with an ordering on the atomic itself; the other two need a standalone **fence** — and the fences are, as it turns out, what lets two threads reach out their hands to one another into a happens-before relationship.

**das Data Race** · das, -s · `T0` — data race — concurrent unsynchronized access; undefined behaviour in Rust
> DE: In C ist das eine Volkstradition mit `volatile`; in Rusts Memory-Modell ist es ein **Data Race** — undefiniertes Verhalten — und Miri sagt es laut und deutlich.
> EN: In C this is a folk tradition with `volatile`; in Rust's memory model it is a **data race** — undefined behavior — and Miri says so loud and clear.

**die Cache line** · die, -s · `T0` — cache line — the unit of cache coherence
> DE: **MESI / Cache-Kohärenz** — das Protokoll, das die Caches der einzelnen Cores konsistent hält; eine **cache line**, die ein Core schreibt, muss in den anderen invalidiert werden — deshalb serialisiert ein gemeinsam geschriebener Zähler Cores, die logisch gar nicht in Konflikt stehen.
> EN: **MESI / cache coherence** — the protocol that keeps the individual cores' caches consistent; a **cache line** that one core writes must be invalidated in the others — which is why a jointly written counter serializes cores that are logically not in conflict at all.

**der Core** · der, -s · `T0` — core — a CPU core
> DE: Die drei Randbedingungen, die das schwer machen, und dann die Tour der Fehlschläge: Ein `RwLock` lässt Leser gemeinsam hinein, zwingt aber jeden von ihnen, einen gemeinsamen Zähler zu *schreiben*, und serialisiert damit **Cores**, die gar nicht in Konflikt stehen; `ArcSwap` und RCU umgehen das Zerreißen, ziehen den Leser aber zurück hinein, sich selbst zur Reclamation anzumelden.
> EN: The three constraints that make this hard, and then the tour of failures: An `RwLock` lets readers in together, but forces each of them to *write* a shared counter, and thereby serializes **cores** that are not in conflict at all; `ArcSwap` and RCU sidestep the tearing, but pull the reader back in to register itself for reclamation.

**der Benchmark** · der, -s · `T0` — benchmark — a performance measurement
> DE: Dann bekommt der Sequenzzähler einen zweiten Job als Lock der Schreiber, und dazu die Vertrauensfrage: der Test, der *scheitern* muss, Miri für das Race, loom für die Verschränkungen, und ein **Benchmark**, der — in Nanosekunden — zeigt, wie ein Lesepfad flach bleibt, während ein `RwLock` 450× langsamer wird.
> EN: Then the sequence counter gets a second job as the writers' lock, and with it the question of trust: the test that has to *fail*, Miri for the race, loom for the interleavings, and a **benchmark** that — in nanoseconds — shows how a read path stays flat while an `RwLock` becomes 450× slower.

**der Pointer** · der, - · `T0` — pointer — a memory pointer
> DE: **Miri** — ein Interpreter, der Rust gegen das Memory-Modell ausführt und undefiniertes Verhalten (Data Races, ungültige **Pointer**) fängt, das ein normaler Test zwar ausführt, aber nicht erkennen kann.
> EN: **Miri** — an interpreter that runs Rust against the memory model and catches undefined behavior (data races, invalid **pointers**) that a normal test does execute but cannot detect.

**das Alignment** · das, kein Pl. · `T0` — alignment — memory alignment of a type
> DE: Die Korrektur macht jeden Zugriff auf die Payload atomar, Wort für Wort, und verwandelt „Müll lesen" von UB in einen legalen Read, den der Zähler wegwirft — was die Payload zwingt, `Pod` zu sein, eine Schranke, die sich als Lizenz erweist, die der Implementierer *unterschreibt*, statt einer, die der Compiler prüft, und die nicht einmal **Alignment** abdeckt.
> EN: The fix makes every access to the payload atomic, word by word, and turns "reading garbage" from UB into a legal read that the counter throws away — which forces the payload to be `Pod`, a bound that turns out to be a license the implementer *signs*, rather than one the compiler checks, and that does not even cover **alignment**.

**der Marker-Trait** · der, -s · `T0` — marker trait — a trait with no methods, only a promise
> DE: **`Pod`** („plain old data") — ein **Marker-Trait**, der verspricht, dass ein Typ nur Bytes ist: kein Padding, jedes Bitmuster gültig, definiertes Layout.
> EN: **`Pod`** ("plain old data") — a **marker trait** that promises a type is only bytes: no padding, every bit pattern valid, defined layout.

**die Reclamation** · die, kein Pl. · `T0` — reclamation — reclaiming memory no longer in use
> DE: Die drei Randbedingungen, die das schwer machen, und dann die Tour der Fehlschläge: Ein `RwLock` lässt Leser gemeinsam hinein, zwingt aber jeden von ihnen, einen gemeinsamen Zähler zu *schreiben*, und serialisiert damit Cores, die gar nicht in Konflikt stehen; `ArcSwap` und RCU umgehen das Zerreißen, ziehen den Leser aber zurück hinein, sich selbst zur **Reclamation** anzumelden.
> EN: The three constraints that make this hard, and then the tour of failures: An `RwLock` lets readers in together, but forces each of them to *write* a shared counter, and thereby serializes cores that are not in conflict at all; `ArcSwap` and RCU sidestep the tearing, but pull the reader back in to register itself for **reclamation**.

**das Flag** · das, -s · `T0` — flag — a boolean status marker
> DE: Das reduziert alles auf eine einzige Frage — woher weiß ein Leser, dass er während eines Schreibvorgangs gelesen hat? — und wir leiten die Antwort auf die harte Tour her, indem wir einem booleschen **Flag** beim Scheitern zusehen, weil es keine Geschichte trägt, bis das Einzige übrig bleibt, was funktioniert: ein Zähler, der gerade ist, wenn der Wert stabil ist, und ungerade, während er geschrieben wird, vor und nach dem Lesen abgetastet.
> EN: This reduces everything to a single question — how does a reader know it read during a write? — and we derive the answer the hard way, by watching a boolean **flag** fail, because it carries no history, until the only thing left that works remains: a counter that is even when the value is stable and odd while it is being written, sampled before and after the read.

**der Model Checker** · der, - · `T0` — model checker — explores every possible interleaving of a concurrent test
> DE: **loom** — ein **Model Checker**, der einen kleinen nebenläufigen Test unter jeder möglichen Thread-Verschränkung erneut ausführt; der Verifizierer für Lock-free-Code.
> EN: **loom** — a **model checker** that re-runs a small concurrent test under every possible thread interleaving; the verifier for lock-free code.

**der Chain-Head** · der; -s · `T1` — chain head — the current tip block of the chain
> DE: Ein Blockchain-Node rückt seinen kanonischen **Chain-Head** etwa alle zwölf Sekunden vor.
> EN: A blockchain node advances its canonical **chain head** roughly every twelve seconds.

**der Node** · der; -s · `T1` — node — a blockchain network participant
> DE: Gibt man den an einen Nutzer weiter, ist er falsch; füttert man ihn in einen State-Root-Lookup, wird der **Node** korrumpiert.
> EN: Hand it to a user and it is wrong; feed it into a state-root lookup and the **node** gets corrupted.

**der Mempool** · der; -s · `T1` — mempool — the pool of pending transactions
> DE: jede Transaktion, die der **Mempool** validiert
> EN: every transaction that the **mempool** validates

**der Peer** · der; -s · `T1` — peer — another node on the network
> DE: jede Antwort an einen **Peer**
> EN: every response to a **peer**

**der Read** · der; -s · `T1` — read — a read operation
> DE: Zehntausende **Reads** pro Sekunde, aus Dutzenden Threads, gegen einen einzigen Write alle zwölf Sekunden.
> EN: Tens of thousands of **reads** per second, from dozens of threads, against a single write every twelve seconds.

**der Write** · der; -s · `T1` — write — a write operation
> DE: Zehntausende Reads pro Sekunde, aus Dutzenden Threads, gegen einen einzigen **Write** alle zwölf Sekunden.
> EN: Tens of thousands of reads per second, from dozens of threads, against a single **write** every twelve seconds.

**der Thread** · der; -s · `T1` — thread — an OS thread of execution
> DE: Zehntausende Reads pro Sekunde, aus Dutzenden **Threads**, gegen einen einzigen Write alle zwölf Sekunden.
> EN: Tens of thousands of reads per second, from dozens of **threads**, against a single write every twelve seconds.

**der Tick** · der; -s · `T1` — tick — one market-data update interval
> DE: Ein Oracle-Thread aktualisiert `(mark_price, funding_index, timestamp)` einmal pro **Tick**, und die Risk-Engine liest es bei *jeder einzelnen Order*, um Margin zu berechnen.
> EN: An oracle thread updates `(mark_price, funding_index, timestamp)` once per **tick**, and the risk engine reads it on *every single order* to compute margin.

**die Order** · die; -s · `T1` — order — a trading order
> DE: Ein Oracle-Thread aktualisiert `(mark_price, funding_index, timestamp)` einmal pro Tick, und die Risk-Engine liest es bei *jeder einzelnen **Order***, um Margin zu berechnen.
> EN: An oracle thread updates `(mark_price, funding_index, timestamp)` once per tick, and the risk engine reads it on *every single **order*** to compute margin.

**die Risk-Engine** · die; -s · `T1` — risk engine — the component that computes trading risk/margin
> DE: Ein Oracle-Thread aktualisiert `(mark_price, funding_index, timestamp)` einmal pro Tick, und die **Risk-Engine** liest es bei *jeder einzelnen Order*, um Margin zu berechnen.
> EN: An oracle thread updates `(mark_price, funding_index, timestamp)` once per tick, and the **risk engine** reads it on *every single order* to compute margin.

**die Margin** · die; -s · `T1` — margin — collateral backing a position
> DE: die Risk-Engine liest es bei *jeder einzelnen Order*, um **Margin** zu berechnen.
> EN: the risk engine reads it on *every single order* to compute **margin**.

**der Tail** · der; -s · `T1` — tail — tail latency, the slow worst-case response
> DE: Ein langsamer Read dort ist ein langsamer **Tail** für den ganzen Handelsplatz.
> EN: A slow read there is a slow **tail** for the whole trading venue.

**der SeqLock** · der; -s · `T1` — SeqLock — a sequence lock
> DE: Das ist die Form, für die ein **SeqLock** gebaut ist:
> EN: That is the shape a **SeqLock** is built for:

**der Hash** · der; -s · `T1` — hash — a fixed-size digest
> DE: Ein Chain-Head ist `(B256, u64)` — ein 32-Byte-**Hash** und eine 8-Byte-Zahl, 40 Byte.
> EN: A chain head is `(B256, u64)` — a 32-byte **hash** and an 8-byte number, 40 bytes.

**das Byte** · das; -/-s · `T1` — byte
> DE: Ein Chain-Head ist `(B256, u64)` — ein 32-Byte-Hash und eine 8-Byte-Zahl, 40 **Byte**.
> EN: A chain head is `(B256, u64)` — a 32-byte hash and an 8-byte number, 40 **bytes**.

**der Store** · der; -s · `T1` — store — a write to memory
> DE: Der größte atomare **Store**, den die Hardware bietet, ist ein Maschinenwort — 8 Byte auf einer 64-Bit-Maschine, 16 mit einem double-width compare-and-swap, wenn man vorsichtig ist.
> EN: The largest atomic **store** the hardware offers is a machine word — 8 bytes on a 64-bit machine, 16 with a double-width compare-and-swap, if you are careful.

**der Reader** · der; - · `T1` — reader — a reading thread / the read side
> DE: Ein **Reader**, der in diesem Fenster landet, bekommt keinen veralteten Wert.
> EN: A **reader** that lands in this window does not get a stale value.

**das Atomic** · das; -s · `T1` — atomic — an atomic variable/type
> DE: Mach jedes Feld zu seinem eigenen **Atomic**.
> EN: Make each field its own **atomic**.

**der Writer** · der; - · `T1` — writer — the writing thread / the write side
> DE: Die Risk-Engine liest `mark_price` aus Tick N und, ein paar Nanosekunden später, `funding_index` aus Tick N+1, weil der **Writer** dazwischen beide aktualisiert hat.
> EN: The risk engine reads `mark_price` from tick N and, a few nanoseconds later, `funding_index` from tick N+1, because the **writer** updated both in between.

**der Account** · der; -s · `T1` — account — a trading account
> DE: Das *Paar* ist ein Wert, den es nie gegeben hat, und die daraus berechnete Margin ist falsch — falsch genug, um einen **Account** zu liquidieren, der eigentlich gesund war.
> EN: The *pair* is a value that never existed, and the margin computed from it is wrong — wrong enough to liquidate an **account** that was actually healthy.

**der Bug** · der; -s · `T1` — bug — a software defect
> DE: Das ist echtes Geld, verloren an einen Konsistenz-**Bug**, den Atomics pro Feld strukturell nicht fangen können.
> EN: That is real money, lost to a consistency **bug** that per-field atomics structurally cannot catch.

**der Snapshot** · der; -s · `T1` — snapshot — a consistent point-in-time copy
> DE: Veröffentliche einen **Snapshot** aus mehreren Feldern so, dass jeder Reader stets einen Snapshot sieht, den es als Ganzes tatsächlich gegeben hat.
> EN: Publish a **snapshot** made of several fields such that every reader always sees a snapshot that, as a whole, actually existed.

**der Lock** · der; -s · `T1` — lock — a synchronization primitive
> DE: Wäre Korrektheit die einzige Anforderung, wäre dies ein gelöstes und langweiliges Problem — ein **Lock** drumherum, und Feierabend.
> EN: Were correctness the only requirement, this would be a solved and boring problem — a **lock** around it, and done for the day.

**das Constraint** · das; -s · `T1` — constraint — a requirement the design must meet
> DE: Interessant wird es, weil Korrektheit in drei **Constraints** verpackt kommt, mit denen der Lock zu kämpfen hat.
> EN: It gets interesting because correctness comes wrapped in three **constraints** that the lock has to struggle with.

**der Mark-Price** · der; -s · `T1` — mark price — the reference price used for margining
> DE: Der Engine-Thread, der den Chain-Head vorrückt, ist der kritische Pfad des Nodes; der Oracle-Thread, der den **Mark-Price** aktualisiert, ebenso.
> EN: The engine thread that advances the chain head is the node's critical path; the oracle thread that updates the **mark price**, likewise.

**die Exchange** · die; -s · `T1` — exchange — a trading exchange
> DE: Auf der **Exchange** lebt er in einem Latenzbudget pro Order, gemessen in Mikrosekunden.
> EN: On the **exchange** it lives within a per-order latency budget, measured in microseconds.

**die Heap-Allocation** · die; -s · `T1` — heap allocation
> DE: Er kann nicht für eine **Heap-Allocation** pausieren, und er kann nicht ohne Obergrenze erneut versuchen.
> EN: It cannot pause for a **heap allocation**, and it cannot retry without an upper bound.

**der Mutex** · der; -e · `T1` — mutex — a mutual-exclusion lock
> DE: Ein **Mutex** existiert, um zu lösen: „Viele Parteien *modifizieren* alle, also müssen sie sich abwechseln."
> EN: A **mutex** exists to solve: "Many parties all *modify*, so they have to take turns."

**read-only** · adj (indekl.) · `T1` — read-only — performing no writes
> DE: Weil er **read-only** ist, braucht er *einen* Snapshot, den es einmal gegeben hat — nicht den *neuesten* und keinen *eingefrorenen*.
> EN: Because it is **read-only**, it needs *a* snapshot that existed at one point — not the *latest* and not a *frozen* one.

**der RwLock** · der; -s · `T1` — RwLock — a read-write lock
> DE: Warum also kein `RwLock`? Er lässt Reader doch schon gemeinsam rein
> EN: So why not an `RwLock`? It already lets readers in together

**das Interface** · das; -s · `T1` — interface — the API contract
> DE: Nein — denn „lässt sie gemeinsam rein" ist ein Versprechen, das das **Interface** gibt und das die *Implementierung* nicht umsonst halten kann.
> EN: No — because "lets them in together" is a promise the **interface** makes and one the *implementation* cannot keep for free.

**das MESI-Protokoll** · das; -e · `T1` — MESI protocol — the cache-coherence protocol
> DE: Dieser Zähler lebt auf einer cache line, und eine cache line, die ein Core schreibt, muss in jedem anderen Core, der sie hält, invalidiert werden — das **MESI-Protokoll**.
> EN: This counter lives on a cache line, and a cache line that one core writes must be invalidated in every other core that holds it — the **MESI protocol**.

**das Tearing** · das; kein Pl. · `T1` — tearing — a torn (partially updated) read or write
> DE: Es gibt eine cleverere Familie, die das **Tearing** komplett umgeht.
> EN: There is a cleverer family that sidesteps **tearing** entirely.

**der Reference Count** · der; -s · `T1` — reference count — a live-references counter
> DE: was heißt, der Reader muss erneut *seine Anwesenheit ankündigen* (ein **Reference Count**, eine Epoche, ein Hazard Pointer).
> EN: which means the reader has to once again *announce its presence* (a **reference count**, an epoch, a hazard pointer).

**der Hazard Pointer** · der; - · `T1` — hazard pointer — a reclamation-safety marker
> DE: was heißt, der Reader muss erneut *seine Anwesenheit ankündigen* (ein Reference Count, eine Epoche, ein **Hazard Pointer**).
> EN: which means the reader has to once again *announce its presence* (a reference count, an epoch, a **hazard pointer**).

**die Allocation** · die; -s · `T1` — allocation — a heap allocation
> DE: Wir sind zurück bei Readern, die gemeinsamen Zustand schreiben, plus einer **Allocation** bei jedem Write und einem Reclamation-Problem, das zu verwalten ist.
> EN: We are back to readers writing shared state, plus an **allocation** on every write and a reclamation problem to manage.

**per-field-Atomics** · die (Pl.) · `T2` — per-field atomics — one atomic per struct field
> DE: Jeder Lock zwingt entweder den Reader, gemeinsamen Speicher zu schreiben, oder den Writer zu warten — und der einzige Kandidat, der keines von beidem tut, **per-field-Atomics**, ist falsch.
> EN: Every lock forces either the reader to write shared memory, or the writer to wait — and the only candidate that does neither, **per-field atomics**, is wrong.

**das Boolean** · das, -s · `T2` — boolean — a true/false value
> DE: Der naheliegende Detektor ist ein **Boolean**, das der Writer setzt, während er arbeitet.
> EN: The obvious detector is a **boolean** that the writer sets while it works.

**der Payload** · der, -s · `T2` — payload — the protected data being read/written
> DE: Es fängt kein Write, das bereits *in Arbeit* war, als der Reader eintraf: Der Zähler könnte die ganze Zeit unverändert auf demselben Wert stehen, während der **Payload** doch durchweg unterwegs war.
> EN: It catches no write that was already *in progress* when the reader arrived: the counter could stand unchanged at the same value the whole time, while the **payload** was in flight all along.

**der Integer** · der, -s · `T2` — integer
> DE: Ein einziger **Integer** kann beide Signale zugleich tragen.
> EN: A single **integer** can carry both signals at once.

**das Inkrement** · das, -e · `T2` — increment
> DE: Ein einzelnes **Inkrement** erledigt beide Aufgaben: Es kippt die Parität um (ungerade verkündet also „schreibe gerade") und es erzeugt eine nie zuvor gesehene Zahl (zwei gleiche gerade Stichproben beweisen also „dazwischen ist nichts geschehen").
> EN: A single **increment** does both jobs: it flips the parity (so odd announces "currently writing") and it produces a number never before seen (so two equal even samples prove "nothing happened in between").

**der Load** · der, -s · `T2` — load — an atomic read/load
> DE: Zwei **Loads**, eine Kopie, zwei weitere Loads — alles Reads.
> EN: Two **loads**, one copy, two more loads — all reads.

**das Memory-Ordering** · das, kein Pl. · `T2` — memory ordering — the ordering of memory operations
> DE: *Weiter: [Teil 3 — Das **Memory-Ordering** richtig hinbekommen](03_memory_ordering.md) · [Index](00_index.md)*
> EN: *Next: [Part 3 — Getting the **memory ordering** right](03_memory_ordering.md) · [Index](00_index.md)*

**Memory Ordering** · das; kein Pl. · `T3` — memory ordering — the rules for how memory operations become visible across threads
> DE: Teil 3 — Das **Memory Ordering** richtig hinbekommen
> EN: Part 3 — Getting the **Memory Ordering** right

**der Bump** · der; -s · `T3` — bump — an increment (of the sequence counter)
> DE: Die zwei **Bumps** des Writers sind seine Kanten; der payload soll zwischen ihnen leben.
> EN: The writer's two **bumps** are its edges; the payload should live between them.

**die Contention** · die; kein Pl. · `T3` — contention — multiple threads competing for the same data
> DE: Lass es auf einem Apple M2 unter echter **Contention** laufen, und es zerreißt — ein Reader bekommt `[54458, 54458, 54459, 54459]`, halb die eine Version und halb die nächste.
> EN: Run it on an Apple M2 under real **contention**, and it tears apart — a reader gets `[54458, 54458, 54459, 54459]`, half the one version and half the next.

**der Compiler** · der; - · `T3` — compiler
> DE: Der **Compiler** darf umordnen; die CPU — auf einem schwachen Memory-Model wie aarch64 — darf zur Laufzeit umordnen, solange die eigene Sicht *dieses einen Threads* konsistent bleibt.
> EN: The **compiler** may reorder; the CPU — on a weak memory model like aarch64 — may reorder at runtime, as long as *that one thread's* own view stays consistent.

**das Gate** · das; -s · `T3` — gate — here a one-way memory-ordering barrier
> DE: Jedes ist ein einseitiges **Gate**, und es bewacht nur eine Seite der Operation, an der es hängt.
> EN: Each is a one-sided **gate**, and it guards only one side of the operation it hangs on.

**die fence** · die; -s · `T3` — fence — a standalone memory barrier; note the German text genders it feminine (die fence)
> DE: Nur eine **fence** schlägt die Brücke von der einen zur anderen.
> EN: Only a **fence** builds the bridge from the one to the other.

**der Handshake** · der; -s · `T3` — handshake — here two fences synchronising
> DE: Was die fences wirklich einbringen: ein **Handshake**
> EN: What the fences really bring in: a **handshake**

**der torn read** · der; -s · `T3` — torn read — a read that mixes bytes from two versions
> DE: Also wird ein **torn read** *garantiert* gefangen — `s1 != s2`, retry.
> EN: So a **torn read** is *guaranteed* to be caught — `s1 != s2`, retry.

**das Undefined Behaviour** · das; kein Pl. · `T3` — undefined behaviour — code the language spec places no constraints on
> DE: Was bleibt, ist ein subtileres Verbrechen, das wir die ganze Zeit begangen haben: Der Reader hat Bytes gelesen, die der Writer gerade aktiv verändert, und in Rusts Memory-Model ist das nicht bloß „Müll lesen“ — es ist **Undefined Behaviour**.
> EN: What remains is a subtler crime that we've been committing the whole time: the reader has read bytes that the writer is actively changing right now, and in Rust's memory model that is not merely "reading garbage" — it is **Undefined Behaviour**.

**das Payload** · das, -s · `T4` — payload — the data value the SeqLock guards
> DE: Hier ist der natürliche Weg, das **Payload** herauszukopieren — ein schlichter Read durch einen raw pointer:
> EN: Here is the natural way to copy the **payload** out — a plain Read through a raw pointer:

**der raw pointer** · der, - · `T4` — raw pointer
> DE: Hier ist der natürliche Weg, das Payload herauszukopieren — ein schlichter Read durch einen **raw pointer**:
> EN: Here is the natural way to copy the payload out — a plain Read through a **raw pointer**:

**der Interpreter** · der, - · `T4` — interpreter (a program that runs code)
> DE: Miri — ein **Interpreter**, der den Code gegen das Speichermodell laufen lässt — sagt es unumwunden:
> EN: Miri — an **interpreter** that runs the code against the memory model — says it plainly:

**der Kernel** · der, - · `T4` — kernel (OS kernel)
> DE: So liest der seqlock des Linux-**Kernels** sein Payload, und dort funktioniert es.
> EN: This is how the Linux **kernel**'s seqlock reads its payload, and there it works.

**das Paper** · das, -s · `T4` — paper (academic paper)
> DE: (Hans Boehm hat ein ganzes **Paper** über genau dieses Missverhältnis geschrieben: seqlocks und Speichermodelle von Sprachen vertragen sich nicht, es sei denn, die Sprache gibt einem ein hinreichend billiges Atomic.)
> EN: (Hans Boehm wrote an entire **paper** about exactly this mismatch: seqlocks and language memory models don't get along, unless the language gives you a sufficiently cheap Atomic.)

**das Padding** · das, kein Pl. · `T4` — padding (unused bytes inserted for alignment)
> DE: Um ein beliebiges `T` als eine Reihe von `usize`-Wörtern umzudeuten, muss `T` tatsächlich schlichte Bytes *sein* — kein **Padding**, jedes Bitmuster gültig (der Leser wird halb geschriebene Mischungen beobachten, bevor er sie verwirft), ein definiertes Layout.
> EN: To reinterpret an arbitrary `T` as a series of `usize` words, `T` must actually *be* plain bytes — no **padding**, every bit pattern valid (the reader will observe half-written mixtures before discarding them), a defined layout.

**das Layout** · das, -s · `T4` — layout (in-memory layout of a type)
> DE: Um ein beliebiges `T` als eine Reihe von `usize`-Wörtern umzudeuten, muss `T` tatsächlich schlichte Bytes *sein* — kein Padding, jedes Bitmuster gültig (der Leser wird halb geschriebene Mischungen beobachten, bevor er sie verwirft), ein definiertes **Layout**.
> EN: To reinterpret an arbitrary `T` as a series of `usize` words, `T` must actually *be* plain bytes — no padding, every bit pattern valid (the reader will observe half-written mixtures before discarding them), a defined **layout**.

**der Trait** · der, -s · `T4` — trait (a Rust interface/capability marker)
> DE: Das ist der `Pod`-**Trait**:
> EN: That is the `Pod` **trait**:

**der Assert** · der, -s · `T4` — assert (a compile-time/runtime assertion)
> DE: Also sind die Prüfungen auf Größen-Vielfaches und Alignment ein *zweites, unabhängiges Tor*, das der Typ passieren muss, separat erzwungen (ein `const`-**Assert**, das zur Compile-Zeit scheitert, nicht zur Laufzeit).
> EN: So the checks for size multiple and alignment are a *second, independent gate* that the type must pass, enforced separately (a `const` **Assert** that fails at compile time, not at runtime).

**das compare-and-swap** · das, -s (CAS) · `T4` — compare-and-swap (CAS) atomic operation
> DE: Von gerade → ungerade zu erhöhen ist kein blindes Inkrement mehr, sondern ein **compare-and-swap**, das nur *von einem geraden Wert aus* gelingt.
> EN: Incrementing from even → odd is no longer a blind increment, but a **compare-and-swap** that succeeds only *from an even value*.

**der Slot** · der, -s · `T4` — slot (here: the write slot)
> DE: Ungerade bedeutet schon „ein Write ist im Gange"; jetzt bedeutet es auch „der Write-**Slot** ist belegt".
> EN: Odd already means "a Write is in progress"; now it also means "the Write **slot** is occupied".

**die Spin-Schleife** · die, -n · `T4` — spin loop (busy-wait loop)
> DE: Ein zweiter Schreiber sieht ungerade und dreht in einer **Spin-Schleife**, bis der erste ihn wieder auf gerade freigibt.
> EN: A second writer sees odd and spins in a **spin loop** until the first releases it back to even.

**lock-free** · unveränderl. (Adj.) · `T4` — lock-free (never blocks on a lock)
> DE: Schreiber serialisieren sich; Leser bleiben **lock-free** und ahnungslos.
> EN: Writers serialize themselves; readers stay **lock-free** and oblivious.

**die Assertion** · die, -en · `T4` — assertion (test assertion)
> DE: Dann ist jeder Load, dessen Wörter sich unterscheiden, von Konstruktion her ein torn read — ein „zerrissener Read" —, und die **Assertion** benennt ihn:
> EN: Then any Load whose words differ is by construction a torn read — a "torn Read" — and the **assertion** names it:

**der Happy Path** · der, -s · `T4` — happy path — the trouble-free, everything-works case
> DE: Das ist ein Test, dessen Aufgabe es ist, beim Fehler zu *scheitern* — das Gegenteil eines Tests, der den **Happy Path** bestätigt.
> EN: This is a test whose job is to *fail* on the bug — the opposite of a test that confirms the **happy path**.

**die fences** · Pl.; Sg. der Fence · `T4` — memory fences (barriers enforcing ordering)
> DE: Wo der torn-read-Test ein paar Schedules abtastet, ist loom über ein beschränktes Modell erschöpfend; es ist das Nächste an einem Beweis, dass die **fences** richtig platziert sind.
> EN: Where the torn-read test samples a few schedules, loom is exhaustive over a bounded model; it is the closest thing to a proof that the **fences** are placed correctly.

**das order book** · das, -s · `T4` — order book — an exchange's list of resting buy/sell orders
> DE: Wer das annimmt, bekommt das Ding, das man einmal baut und überall dort wiederverwendet, wo ein kleiner Wert weit öfter gelesen als geschrieben wird — der chain head, der mark price, die Spitze eines **order book**.
> EN: Whoever accepts this gets the thing you build once and reuse everywhere a small value is read far more often than written — the chain head, the mark price, the top of an **order book**.

**read-mostly** · unveränderl. (Adj.) · `T4` — read-mostly — accessed by readers far more than by writers
> DE: Bei den **read-mostly** Problemen, für die er gebaut ist, ist das genau der Tausch, den man will.
> EN: For the **read-mostly** problems it is built for, that is exactly the trade you want.
