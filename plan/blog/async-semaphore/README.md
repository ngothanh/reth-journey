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
- 🇩🇪 **Deutsch** — [`de/00_index.md`](de/00_index.md) *(follows once the English is signed off)*

## Parts

| # | English | Deutsch |
|---|---|---|
| 1 | [What a semaphore is, and when you reach for one](en/01_what_is_a_semaphore.md) | — |
| 2 | [The interface, read off the use cases](en/02_the_interface.md) | — |
| 3 | [Where does the waiting live?](en/03_where_waiting_lives.md) | — |
| 4 | [Fairness: who gets the freed permit?](en/04_fairness.md) | — |
| 5 | [Cancellation: when a waiter vanishes](en/05_cancellation.md) | — |
| 6 | [Where the waiters live: memory, and what Pin is for](en/06_memory_and_pin.md) | — |
| 7 | [Writing it down, and trusting it](en/07_implementation.md) | — |
