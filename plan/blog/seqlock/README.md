# Designing a SeqLock — how to read a value while someone rewrites it

A design investigation, in six parts, into the SeqLock — the primitive that lets
a horde of readers keep reading a shared value while a writer keeps rewriting it,
with the readers never blocking and never even writing to shared memory. It's the
lock inside the Linux kernel's `clock_gettime` fast path, and the natural fit for
a blockchain's chain head or an exchange's mark price.

We start from the problem it exists to solve, watch the locks you'd normally reach
for fail one constraint each, and then make the bet that defines a SeqLock: don't
stop the reader from seeing a half-written value — let it happen, and teach the
reader to notice. From there the design forces itself: a version counter, a precise
dance of memory fences (which a real ARM chip will punish you for getting wrong),
and a way to read bytes mid-write without it being undefined behaviour. The last
part is the write-up and the proof — including a benchmark where the read path
beats a `RwLock` by 450× under eight readers.

No lock-free background needed. Part 1 starts from a value that won't fit in a
register.

## Languages / Ngôn ngữ / Sprachen

- 🇬🇧 **English** — [`en/00_index.md`](en/00_index.md)
- 🇩🇪 **Deutsch** — [`de/00_index.md`](de/00_index.md)

## Parts

| # | English | Deutsch |
|---|---|---|
| 1 | [The problem, and why the obvious locks don't fit](en/01_the_problem.md) | [Das Problem, und warum die naheliegenden Locks nicht passen](de/01_the_problem.md) |
| 2 | [The bet: let it tear, and catch it](en/02_the_bet.md) | [Die Wette: reißen lassen und ertappen](de/02_the_bet.md) |
| 3 | [Getting the memory ordering right](en/03_memory_ordering.md) | [Das Memory Ordering richtig hinbekommen](de/03_memory_ordering.md) |
| 4 | [Reading without UB, and trusting it](en/04_trusting_it.md) | [Ohne UB lesen, und ihm trauen](de/04_trusting_it.md) |
