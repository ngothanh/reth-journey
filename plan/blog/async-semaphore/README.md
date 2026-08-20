# Designing an async semaphore — six decisions and one afternoon of code

A design investigation, in seven parts, into the async semaphore — the primitive
inside `tokio::sync::Semaphore` that lets a program say "at most N of these at once."
We start from the problems it exists to solve, read the interface off the use cases,
and work through the design questions one at a time: where does waiting live, who
gets a freed permit, what happens when a waiter is cancelled, where do waiter records
live in memory. Implementation is the last part, not the first — by then the code
writes itself.

No prior async-internals knowledge needed. Part 1 starts from an API server falling
over.

## Languages / Ngôn ngữ / Sprachen

- 🇬🇧 **English** — [`en/00_index.md`](en/00_index.md)
- 🇩🇪 **Deutsch** — [`de/00_index.md`](de/00_index.md)

## Parts

| # | English | Deutsch |
|---|---|---|
| 1 | [What a semaphore is, and when you reach for one](en/01_what_is_a_semaphore.md) | [Was eine Semaphore ist, und wann man zu ihr greift](de/01_what_is_a_semaphore.md) |
| 2 | [The interface, read off the use cases](en/02_the_interface.md) | [Das Interface, von den Use Cases abgelesen](de/02_the_interface.md) |
| 3 | [Where does the waiting live?](en/03_where_waiting_lives.md) | [Wo lebt das Warten?](de/03_where_waiting_lives.md) |
| 4 | [Fairness: who gets the freed permit?](en/04_fairness.md) | [Fairness: Wer bekommt das freie Permit?](de/04_fairness.md) |
| 5 | [Cancellation: when a waiter vanishes](en/05_cancellation.md) | [Cancellation: Wenn ein Wartender verschwindet](de/05_cancellation.md) |
| 6 | [Where the waiters live, and what Pin is for](en/06_memory_and_pin.md) | [Wo die Wartenden wohnen, und wofür Pin da ist](de/06_memory_and_pin.md) |
| 7 | [Writing it down, and trusting it](en/07_implementation.md) | [Aufschreiben, und ihm trauen](de/07_implementation.md) |