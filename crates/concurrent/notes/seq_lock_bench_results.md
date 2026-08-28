# `SeqLock<T>` vs `RwLock<T>` — bench results

**Machine:** aarch64-apple-darwin (Apple M2, 8 cores: 4P + 4E).
**Payload:** `[u64; 4]` (32 bytes) — bigger than a word (so `AtomicCell`'s
lock-free path doesn't apply; its >8-byte fallback is a spinlock), and a
multi-field snapshot whose honest lock alternative is `RwLock`.
**Method:** criterion `iter_custom` per-op latency, `black_box` elision guard,
background contenders spawned once and kept hot outside the timed region.
Criterion reports [p5 p50 p95]; p50 quoted below. Bench: `benches/seq_lock.rs`.

## 1. Uncontended read

| | p50 |
|---|---|
| `SeqLock::load` | **0.75 ns** |
| `RwLock::read`  | 5.20 ns |

Even with nobody contending, RwLock is ~7× slower: `read()` does a
`fetch_add`/`fetch_sub` RMW pair on the reader counter (two atomics + the guard's
Drop), while SeqLock's read is two Relaxed seq loads + a Relaxed payload copy +
an acquire fence — no RMW, no shared write.

## 2. Read scaling (THE point) — measured reader latency, N total readers

| readers | SeqLock | RwLock | ratio |
|--------:|--------:|-------:|------:|
| 1 | 0.75 ns | 5.2 ns | 7× |
| 2 | 0.78 ns | 66 ns | 85× |
| 4 | 0.79 ns | 182 ns | 230× |
| 8 | 1.5 ns | 680 ns | **450×** |

SeqLock reader latency is **flat** as readers are added; RwLock's grows roughly
linearly. Cause: RwLock readers don't conflict *logically* (all read-only) but
every `read()` WRITES the shared reader counter, so that one cache line ping-pongs
between cores under MESI — physical serialization on metadata the lock needs to
exist. SeqLock readers only ever LOAD the payload, so its cache line stays in
Shared state in every core's L1 simultaneously; adding readers adds no coherence
traffic.

The small SeqLock bump at 8 (0.79 → 1.5 ns) is scheduler oversubscription — 7
background + 1 measured + main = 9 threads on 8 cores — not memory contention;
it's ~3 orders of magnitude below RwLock's climb.

## 3. Read under a continuous writer

| | p50 |
|---|---|
| `SeqLock::load` (1 hot writer) | **7.7 ns** |
| `RwLock::read`  (1 hot writer) | 62 ns |

SeqLock read rises from 0.75 → 7.7 ns: the reader occasionally catches an odd seq
or a changed seq and retries, but it never *waits* on a lock word. RwLock read
rises to 62 ns because the reader BLOCKS whenever the writer holds the lock —
the writer wins the cache line and the reader stalls on acquire. ~8× gap, and
unlike RwLock, SeqLock gives the writer no way to be blocked by readers either.

## 4. Uncontended write (context)

| | p50 |
|---|---|
| `SeqLock::store` | 4.3 ns |
| `RwLock::write`  | 8.6 ns |

SeqLock's write is also cheaper here (CAS even→odd to acquire the writer slot,
two seq bumps, Relaxed payload stores) vs RwLock's writer-lock acquire/release.
SeqLock's trade isn't "slower writes for faster reads" on this payload — it's
"the writer always wins and never coordinates with readers", which is what makes
the read side free to scale.

## Takeaway

For a small, read-mostly, multi-word snapshot (chain head, mark price, book
top-of-book), SeqLock buys a read path that (a) is ~7× cheaper uncontended,
(b) stays flat under many readers where RwLock degrades ~linearly (450× at 8
readers), and (c) doesn't stall under a live writer. The cost is the constraints
that made it possible: `T: Pod`, word-multiple size + word alignment, copy-out
(no `&T`), and reader retry under a hot writer.
