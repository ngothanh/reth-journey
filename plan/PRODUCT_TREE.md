# The Big Picture — 5 product roots and the dependency tree

> This document changes the organizing axis of the plan: no longer **week × concept**, but
> **product × vertical slice**. We do not work through the calendar day by day — we go straight at a
> product and take it from naive to production grade.
>
> The week-by-week plan is **archived, not discarded**: it lives in
> [`reference/`](reference/) (see [`reference/README.md`](reference/README.md)) together with
> `../README.md`, and remains the depth treatment to look things up in. §6 below maps every advanced
> core technique to the product that now owns it — that table is the guarantee nothing was lost.

---

## 0. The rule: what to write, what to borrow

**Core = something with a state machine, design trade-offs, and behaviour that changes under load.**
B+tree, index, buffer pool, disruptor ring, consensus, storage engine, matching engine, scheduler — core.

**Not core = something with a spec, where reading the spec is enough to write it**, with no design
decisions to make. Encode/decode, derive macros, ser-de, hex, error enums, crypto permutations —
borrow, or write once and stop.

Bugs in the first category are *design bugs* — they only surface under load, under crash, under race.
That is where the learning is. Bugs in the second category are *spec bugs* — a table of test vectors
catches all of them in one afternoon.

Four operating laws:

1. **Every primitive gets written exactly once.** If two products need it, it is a shared crate, not a
   second implementation.
2. **Scaffolding is allowed.** To verify an upper layer you sometimes need a deliberately stupid
   implementation underneath (e.g. an in-memory `BTreeMap` backend for the trie). Write ~50 lines, know
   up front it gets thrown away, mark it `// SCAFFOLD — replaced in vX`. That is not "borrowing a
   library", that is leverage to get something runnable early.
3. **Borrow first, write later — when writing it blocks the acceptance test.** E.g. `rayon` in P1 v2,
   swapped for the hand-written thread pool once `runtime-thread-per-core` exists in P4.
4. **Every rung is measured, not asserted.** Keep the old plan's two-track discipline: a
   **back-of-envelope prediction first** (what the hardware cost model says this should cost), then the
   **measured number** (ns / cycles / allocations), then reconcile the gap. A rung is only "done" when
   you can state the wall in numbers — that is what makes the next rung necessary rather than optional.
   Notes land in `notes/*.md`.

---

## 1. The five product roots

Each root is a complete system with **one acceptance test that cannot be faked**, and its own
**maturity axis**.

| # | Product | What it is | Acceptance test (cannot be faked) | Maturity axis |
|---|---|---|---|---|
| **P1** | `ethdb` | Ethereum state store: storage engine + Merkle Patricia Trie | Apply mainnet blocks N→N+k, **state root matches `header.stateRoot`** every block; `kill -9` mid-write, restart, state still correct | naive → production |
| **P2** | `exec-vm` | EVM: interpreter → gas → journal → parallel | Replay a mainnet block, **produce the state diff yourself** (no more RPC), root still matches → P1∘P2 closes the loop | sequential → parallel |
| **P3** | `eth-node` | Execution client: devp2p + staged sync + txpool + Engine API | Sync from a checkpoint and **stay at the mainnet tip** without falling behind | sync → async, standalone → networked |
| **P4** | `matching-engine` | Single-node CLOB, HFT-grade | Sustained N msg/s at a **p99 tick-to-trade under a declared budget**, zero allocation in the apply loop | slow → fast |
| **P5** | `perp-dex-core` | Replicated multi-node perp DEX | **Cluster VOPR** + Byzantine scenario suite pass; every replica replays to bit-identical state | single node → cluster + consensus |

Those five axes are exactly the five you named — and they **do not overlap**.

---

## 2. The maturity ladder — the spine of the work

Every rung ends at a concrete, **measurable wall**. That wall is the reason the next rung exists.

### P1 — `ethdb`

| Rung | What you build | The wall that forces the next rung |
|---|---|---|
| v0 | `HashMap` state + MPT rebuilt **entirely** every block | Correct, but every block re-hashes the whole state — unusable |
| v0.5 | `HashBuilder` (sorted stream) + `PrefixSet` + `TrieWalker` → touch only changed subtries | Fast, but everything is gone on restart |
| v1 | Hand-written: page manager + mmap + **B+tree** + cursor/dupsort + **WAL** + ARIES recovery | Crash mid group-commit; and one writer blocks every reader |
| v1.5 | MVCC snapshots + group commit + LRU-K buffer pool + pin/unpin | Single-threaded root computation becomes the bottleneck; hot path still hits disk |
| v2 | Parallel storage root (on `concurrent`) + in-memory **sparse trie** + pruning + snapshots | — (production grade) |

### P2 — `exec-vm`

| Rung | What you build | The wall |
|---|---|---|
| v0 | Interpreter `match`-loop, no gas | Runs a few opcodes then diverges — no gas means no Ethereum semantics |
| v0.5 | Gas + journal/revert + `Host` trait wired into P1 | Correct but dispatch-bound, one branch per opcode |
| v1 | Opcode table + tightened stack machine + memory expansion + precompiles | The whole block executes serially; the other cores idle |
| v1.5 | **Block-STM**: optimistic parallel + read/write sets + versioned memory + re-execution | — |

### P3 — `eth-node`

| Rung | What you build | The wall |
|---|---|---|
| v0 | Pull blocks over RPC, no p2p | You depend on someone else's node — that is not a node |
| v0.5 | RLPx + ECIES handshake + discv4 + `eth/68`, header sync | Headers work, bodies/state stall; serial sync is far too slow |
| v1 | Staged sync (headers→bodies→execution→merkle) + txpool + Engine API + JWT | Syncing from genesis takes weeks |
| v1.5 | Snap sync + pruning + static files | — |

### P4 — `matching-engine`

| Rung | What you build | The wall |
|---|---|---|
| v0 | `BTreeMap<Price, VecDeque<Order>>`, allocate freely, synchronous | Logic is right, latency is ragged — allocation and cache misses show up immediately |
| v0.5 | Intrusive linked list per price level + object pool + **zero-alloc apply loop** | Cross-thread contention and false sharing become the ceiling |
| v1 | **Thread-per-core** (1 symbol = 1 shard) + SPSC/MPMC rings between stages (the Disruptor, embodied) | The kernel network stack and syscalls now dominate the budget |
| v1.5 | io_uring + AF_XDP kernel bypass + `latency-lab` (HdrHistogram, coordinated omission, rdtsc) | — |

### P5 — `perp-dex-core`

| Rung | What you build | The wall |
|---|---|---|
| v0 | Single node, in-memory: matching + oracle-mark + risk + liquidation + ledger | Lose the node, lose everything |
| v0.5 | WAL + deterministic replay (single PRNG seed, integer-only money math) | It comes back up, but it is still a single point of failure |
| v1 | **VSR** replication N=3 + state transfer + reconfiguration + **VOPR** fault injection | Survives crashes, does not survive a replica that lies |
| v1.5 | **BFT** (pipelined HotStuff / Jolteon) N=4 f=1 + Byzantine scenario suite + model checking | — |

---

## 3. The full dependency tree

Legend: **✍️ hand-written** · **📦 borrowed crate** · **🧪 scaffold, later replaced** · `[✓]` already in the repo

```
                              ┌─────────────────────────────────────────┐
                              │  P5  perp-dex-core   (cluster + BFT)    │ ✍️
                              └───┬──────────┬──────────┬───────────┬───┘
                    ┌─────────────┘          │          │           └──────────────┐
        ┌───────────▼──────────┐  ┌──────────▼───────┐  ┌▼───────────────┐  ┌──────▼────────┐
        │ P4 matching-engine   │✍️│ risk-engine      │✍️│ consensus-vsr  │✍️│ ledger-       │✍️
        │  order book, STP,    │  │ oracle-mark      │✍️│ consensus-bft  │✍️│ deterministic │
        │  iceberg, triggers   │  │ liquidation-eng  │✍️│ model-check 📦 │  │               │
        └───────────┬──────────┘  └──────────┬───────┘  └┬───────────────┘  └──────┬────────┘
                    │                        │           │                          │
                    └────────────┬───────────┴───────────┴──────────────────────────┘
                                 │
   ┌─────────────────────────────▼──────────────────────────────────────────────┐
   │ HFT / RUNTIME SUBSTRATE                                                     │
   │  runtime-thread-per-core ✍️   messaging-aeron ✍️   mmap-queue ✍️            │
   │  marketdata-kernelbypass ✍️(📦 io-uring, libc)     latency-lab ✍️(📦 hdrhist)│
   │  backpressure ✍️              log-distributed ✍️                            │
   └─────────────────────────────┬──────────────────────────────────────────────┘
                                 │
        ┌────────────────────────┴─────────────────────────┐
        │                                                   │
┌───────▼──────────┐                            ┌───────────▼─────────────┐
│ P3  eth-node     │✍️                          │  (shared by P4 / P5)    │
│  p2p/RLPx/discv4 │✍️ (📦 secp256k1, aes)      │                         │
│  eth-stage       │✍️                          │                         │
│  txpool          │✍️                          │                         │
│  engine-api      │✍️ (📦 jsonrpsee, jwt)      │                         │
└───┬──────────┬───┘                            └─────────────────────────┘
    │          │
┌───▼──────┐ ┌─▼──────────────────────────────────────┐
│ P2       │ │ P1  ethdb                              │
│ exec-vm  │✍️│  ┌──────────────────────────────────┐ │
│  interp  │ │  │ eth-trie                        ✍️│ │
│  gas     │ │  │  Nibbles · TrieMask · TrieNode   │ │
│  journal │ │  │  HashBuilder ⭐ · PrefixSet      │ │
│  block-  │ │  │  TrieWalker ⭐ · Proof           │ │
│   stm    │ │  │  SparseTrie ⭐ · ParallelRoot ⭐  │ │
│ 📦 k256, │ │  └──────────────┬───────────────────┘ │
│   ark-bn │ │  ┌──────────────▼───────────────────┐ │
└──────────┘ │  │ storage-engine                 ✍️│ │
             │  │  pager/mmap · bufpool (LRU-K)   │ │
             │  │  btree (split/merge/cursor) ⭐   │ │
             │  │  wal (group commit) · recovery  │ │
             │  │  txn (MVCC/2PL/OCC)             │ │
             │  │  bloom · lsm-core (alt engine)  │ │
             │  │  🧪 BTreeMap backend (v0 only)   │ │
             │  └──────────────────────────────────┘ │
             └────────────────────┬──────────────────┘
                                  │
   ┌──────────────────────────────▼──────────────────────────────────────┐
   │ LAYER 1 — substrate shared by EVERY product                          │
   │  concurrent ✍️[✓]  CachePadded, Backoff, AtomicCell, Parker, Pod,   │
   │                    Mutex, RwLock, Semaphore, Condvar, Arc,          │
   │                    SeqLock<T: Pod>, MPMC ring (Vyukov), SegQueue,   │
   │                    channel + select, epoch-gc, skiplist             │
   │  bufpool ✍️[✓]     Page, PageBox, PageAllocator, eviction, sharding  │
   │  backpressure ✍️[✓] BackpressureStrategy, BoundedBuffer              │
   │  time ✍️           Monotonic, Lamport, HLC, rdtsc                    │
   │  📦 loom (test) · criterion (bench) · rayon (temporary, dropped @P4) │
   └──────────────────────────────┬──────────────────────────────────────┘
                                  │
   ┌──────────────────────────────▼──────────────────────────────────────┐
   │ LAYER 0 — DONE, FROZEN                                               │
   │  eth-primitives ✍️[✓]  B256/U256/Address/Bytes/FixedBytes/Sealed     │
   │  eth-rlp ✍️[✓]         encode/decode/header                          │
   │  📦 tiny-keccak (keccak256)                                          │
   └─────────────────────────────────────────────────────────────────────┘
```

---

## 4. The reuse matrix — why the tree is the thing that matters

Columns are product roots. A mark means the crate is **consumed again**, not rewritten.

| Crate | P1 ethdb | P2 exec-vm | P3 node | P4 matching | P5 perp |
|---|:--:|:--:|:--:|:--:|:--:|
| `eth-primitives`, `eth-rlp` | ● | ● | ● | | |
| `concurrent` | ● | ● | ● | ● | ● |
| `time` | ● | | ● | ● | ● |
| `bufpool` / pager | ● | | ● | | ● |
| `btree` / storage-engine | ● | | ● | | ● |
| `wal` + `recovery` | ● | | ● | ● | ● |
| `txn` (MVCC) | ● | | ● | | ● |
| `eth-trie` | ● | | ● | | |
| `exec-vm` | | ● | ● | | |
| `backpressure` | | | ● | ● | ● |
| `runtime-thread-per-core` | | | ○ | ● | ● |
| `latency-lab` | ○ | | | ● | ● |
| `mmap-queue` | | | | ● | ● |
| `messaging-aeron` | | | | ● | ● |
| `p2p` | | | ● | | ● |
| `epoch-gc` | ● | | | ● | ● |
| `lsm-core` + `bloom` | ○ | | | | ● |

● = required · ○ = optional / used for benchmarking

Read it by row: `concurrent` appears in **all five** products — which is why finishing it first is the
right call. `wal` + `recovery` appear in four — written once in P1, reused untouched in P4/P5. That is
the reason the storage engine comes before the venue, and not the other way round.

---

## 5. The borrow list — and why

| Borrowed | Used in | Why not hand-written |
|---|---|---|
| `tiny-keccak` | P1, P2, P3 | Crypto permutation, fixed spec, no design decision |
| `k256` / `secp256k1` | P2 (ecrecover), P3 (ECIES) | Constant-time crypto — writing it yourself is *dangerous*, not educational |
| `ark-bn254`, `bls12_381` | P2 precompiles | Pairing crypto, same reason |
| `libc`, `io-uring`, `memmap2` | P1, P4 | Syscall bindings, not design |
| `loom`, `criterion`, `proptest` | everywhere | Test / bench tooling |
| `rayon` | P1 v2 (temporary) | Replaced by the hand-written `runtime-thread-per-core` once P4 lands |
| `hdrhistogram` | P4 (temporary) | Could be written later — log-linear bucketing is a real data structure, but it is not on the critical path |
| `tokio` | P3 (temporary) | Async runtime for the node; P4/P5 use the hand-written runtime |
| `jsonrpsee`, `jwt` | P3 Engine API | Transport plumbing |
| `stateright` | P5 model checking | A verification tool, not a system |
| `alloy-*` | tests only | Used as a **differential oracle** in tests, never linked into the hot path |

**Deleted, not to be rebuilt:** proc-macro derives, anything serde/ser-de, and pure-format codec
layers. They served their purpose; see §9.

---

## 6. Technique index — nothing is lost

Every advanced core technique in `README.md` has a new home. A technique with no "Lives in" entry is a
bug in the plan.

| Technique (HFT / systems) | Lives in | From old plan |
|---|---|---|
| False sharing, CachePadded, MESI cost model | `concurrent` | W4, W10 drill #11 |
| Memory ordering, Release/Acquire vs SeqCst, loom | `concurrent` | W6 drill #3, W1–37 drills |
| Lock-free MPMC (Vyukov), SegQueue, skiplist | `concurrent` | W11, W26, W37 |
| Epoch-based reclamation | `epoch-gc` → bufpool page reclaim, P4 price levels | W33–37 |
| Disruptor / SPSC ring, single-writer principle | `concurrent` rings → P4 stage pipeline | W10, reference-only |
| Thread-per-core, shard-by-key, cross-shard queues | `runtime-thread-per-core` → P4 v1 | W30/57/84 |
| io_uring, epoll, AF_XDP kernel bypass | `marketdata-kernelbypass` → P4 v1.5 | W85–90 |
| HdrHistogram, coordinated omission, rdtsc, `perf`, NUMA | `latency-lab` → every bar-(c) bench | W21–24 |
| Zero-alloc / static-memory hot path | P4 v0.5, P5 apply loop | W92 ledger |
| Group commit, WAL segments, checksums | `wal` → P1 v1 | W26 |
| ARIES 3-pass recovery | `recovery` → P1 v1 | W29–30 |
| B+tree split/merge, cursors, dupsort | `btree` → P1 v1 | W28 (expanded) |
| Buffer pool LRU-K, pin/unpin, dirty tracking | `bufpool` → P1 v1 | W12–14 |
| mmap, pretouch, page warming | `pager`, `mmap-queue` | W27, W31/78 |
| MVCC, snapshot isolation, 2PL, OCC, Percolator | `txn` → P1 v1.5 | W42, W72, W94 |
| LSM: memtable, SSTable, STCS/LCS/TWCS | `lsm-core` → P5 venue store | W38–42 |
| Bloom filters (classic/counting/scalable) | `bloom` | W34 |
| MPT: HashBuilder, walker, proofs, sparse trie | `eth-trie` → P1 | W10, W20, W31–32 |
| Block-STM optimistic parallel execution | P2 v1.5 | W94 |
| Staged sync pipeline | P3 v1 | W22 |
| Kademlia, Noise/ECIES, gossip | `p2p` → P3, P5 | W52–55 |
| VSR: view change, state transfer, reconfiguration | `consensus-vsr` → P5 v1 | W68/74/90 |
| BFT: pipelined HotStuff, TC view change, evidence | `consensus-bft` → P5 v1.5 | W118–143 |
| VOPR deterministic simulator, fault injection | P5 v1, `sim-storage` | W88, W108 |
| Bounded model checking (Stateright) | P5 v1.5 | W90, W129 |
| Aeron: term buffer, NAK, flow control, Archive | `messaging-aeron` → P4/P5 | W76–84 |
| Kafka-style partitioned log, staleness contracts | `log-distributed` → P5 off-path | W63/80 |
| Columnar / vectorized scan, zone maps | `query-columnar` → P5 analytics | W84/110 |
| Cross-margin, SPAN, incremental margin | `risk-engine` → P5 | W85/97 |
| Mark/index/funding, partial liq, ADL, insurance fund | `oracle-mark`, `liquidation-engine` → P5 | W77, W100–102 |
| SeqLock, single-writer many-reader snapshots | `concurrent` → P1 head, P4 book snapshot | W002 |
| Monotonic / Lamport / HLC clocks, rdtsc | `time` | W006 |
| Async, `Pin`, `Future` state machines, backpressure | P3 node I/O; `backpressure` | W003, W011 |
| Type-state, variance, phantom lifetimes, guard types | `concurrent` guards, P1 cursor lifetimes | W004, W011 |

The "From old plan" column points at `reference/WNNN.md` — that is where the depth treatment lives
(mental model, contract, named pitfalls, paper drill, back-of-envelope cost model). Look it up when you
reach the component; do not schedule from it.

**The three spaced-repetition systems from the old plan stay** (`concept_cadence.md`,
`rebuild_ladder.md`, `reference/.rework/PAPER_DRILLS.md`) — they are orthogonal to the product axis,
not in conflict with it. The Rebuild Ladder is especially worth keeping: rebuilding `concurrent` from a
blank file is exactly what the semaphore work is.

---

## 7. Execution order and risks

**Order:** finish `concurrent` → **P1** → **P2** → **P3** → **P4** → **P5**.

Why this order and not another:
- `concurrent` first because it is the only crate that appears in all five products.
- P1 before P2 because the EVM needs a state store to write into; the reverse is not true.
- P3 after P1+P2 because the node is mostly *assembly* of those two plus networking — it is the test of
  whether the reuse actually holds.
- P4 split from P5 so there is a standalone "single-node, very fast" milestone before consensus enters.

**Three risks to watch:**

1. **P1 v1 (hand-written B+tree + WAL + recovery) is the longest stretch before visible payoff.**
   Mitigate with the `BTreeMap` scaffold at v0/v0.5: you get a matching mainnet state root **before**
   writing a single line of B+tree. When the real engine is swapped in, the tests are already there to
   catch it.
2. **P3 expands without limit** (snap sync, every fork, archive mode). Hard cap: v1 = sync from a
   checkpoint and hold the tip. No archive mode, no pre-Merge fork archaeology.
3. **P2 precompiles can burn time for nothing.** Borrow all the crypto; hand-write only the gas
   accounting and the host boundary.

---

## 8. Effort estimate — implementation only

Excludes: writing, reading, upstream PR work, job hunting. Pure "hands on the product" hours.

### Calibration anchor

**~11.2 KLOC of Rust** (src + tests + benches) was produced between 2026-04-23 and 2026-07-29 — about
14 calendar weeks. At a realistic 20–25 h/week that is **≈280–350 h for 11.2 KLOC**, i.e. **~25–30 h per
KLOC** for this kind of code — hand-written systems Rust with loom tests, benches, and correctness
pinned against real specs.

(The §9 pruning cut the tree to 6.6 KLOC. That does not change the rate: those hours were still spent,
and the learning they bought is why the surviving 6.6 KLOC is worth what it is. Calibrate on what was
*written*, not on what was *kept*.)

That rate is the basis for everything below. It is slow compared with ordinary application code, and it
should be: the verification burden (loom, proptest, differential tests against upstream) is most of the
cost, not the typing.

### Per product

| Stage | Scope | Estimate | Range |
|---|---|---:|---|
| **Finish `concurrent` + `time`** | semaphore, Vyukov MPMC ring, SegQueue, channel + select, `epoch-gc`, skiplist, time substrate | **180 h** | 150–220 |
| **P1 `ethdb`** | v0 nibbles/node/naive MPT · v0.5 HashBuilder+walker+proofs · **v1 pager+bufpool+B+tree+WAL+ARIES** · v1.5 MVCC · v2 parallel+sparse+pruning | **740 h** | 600–900 |
| **P2 `exec-vm`** | v0.5 interpreter+gas+journal · v1 full opcodes+precompiles+mainnet conformance · v1.5 block-STM | **470 h** | 400–600 |
| **P3 `eth-node`** | v0.5 RLPx+ECIES+discv4+eth/68 · v1 staged sync+txpool+Engine API · v1.5 snap sync | **550 h** | 450–750 |
| **P4 `matching-engine`** | book+order types+zero-alloc · thread-per-core runtime · `mmap-queue`+`messaging-aeron` · io_uring/AF_XDP+`latency-lab` | **740 h** | 600–900 |
| **P5 `perp-dex-core`** | oracle+risk+liquidation+ledger · VSR+VOPR · cluster assembly · BFT apex+model-check | **1000 h** | 800–1300 |
| | **Total** | **≈3700 h** | **3000–4700** |

### Cross-check

The archived 144-week plan budgets **≈4960 h** (30 h/wk × W1–72, 40 h/wk × W73–136, 30 h/wk × W137–144).
This estimate is ~75% of that — which is the right shape, since the old number also carried reading,
PR hunting, writing, and job search. Two independent routes landing within 25% of each other is
reassuring, not proof.

### Calendar, at various intensities

| Pace | Elapsed | Notes |
|---|---|---|
| 20 h/wk (your observed rate) | **~3.6 years** | |
| 30 h/wk | **~2.4 years** | |
| 40 h/wk | **~1.8 years** | full-time equivalent |

### When each product lands (30 h/wk, sequential)

| Milestone | Cumulative | Elapsed |
|---|---:|---|
| `concurrent` + `time` done | 180 h | ~1.5 months |
| **P1 `ethdb` v2** — mainnet state root, own storage engine | 920 h | ~7 months |
| **P2 `exec-vm` v1.5** — self-contained block replay | 1390 h | ~11 months |
| **P3 `eth-node` v1.5** — holds mainnet tip | 1940 h | ~15 months |
| **P4 `matching-engine` v1.5** — sub-µs, allocation-free | 2680 h | ~21 months |
| **P5 `perp-dex-core` v1.5** — replicated, BFT | 3680 h | ~28 months |

### How much to trust this

- **Firmest: P1 and P2.** The acceptance test is binary and external (a state root either matches or it
  does not), so scope cannot quietly drift.
- **Loosest: P3 and P5.** P3 is bounded only by a decision to stop; P5's cost is dominated by
  debugging distributed failures, which is not proportional to code volume. Treat the upper bound as
  the planning number for these two, not the midpoint.
- **Systematic bias: estimates for novel systems work run 1.5–2× low**, and this one is no exception.
  The honest read of 3700 h is "3000 if things go well, 5000+ if P3 and P5 fight back."
- **Biggest single lever: P3.** Dropping it saves ~550 h and costs nothing structurally — no other
  product depends on it (§4: `eth-trie` and `exec-vm` feed it, not the reverse). It is the natural cut
  if the timeline needs to come down.

---

## 9. Current status

The workspace was pruned to exactly what §3/§4 consume — learning-only artifacts were removed
(recoverable from git history). Five crates remain:

| Crate | Layer | Contents |
|---|---|---|
| `eth-primitives` | 0 | `B256`/`U256`/`Address`/`FixedBytes`/`Bytes`/`BytesMut`/`Sealed`/`keccak256`/`b256!`/`address!` |
| `eth-rlp` | 0 | `Encodable`/`Decodable`/`Header` — hand-written, no derive |
| `concurrent` | 1 | `CachePadded`, `Backoff`, `AtomicCell`, `Pod`, `Parker`, `Mutex`, `RwLock`, `Condvar`, `OnceFlag`, `Arc`, `Channel` |
| `bufpool` | 1 | `Page`, `PageBox`, `PageAllocator`, `LruEviction`, `ShardedCache`, `Account`, `StateCache` (was `eth-storage-cache`) |
| `backpressure` | 1 | `BackpressureStrategy`, `BoundedBuffer` (was `eth-network-codec`) |

**Removed:** `etherscanlite` (alloy-exploration CLI), `eth-primitives-derive` (proc macros — §5 frozen),
`eth-chain-state`, `dekker.rs` (drill #3), the tokio framing stack, the mutex/rwlock/local/shared cache
progression, `SimpleEncode`, and the Pin examples. `cargo test --workspace --all-features` is green.

**Layer 1, still to write before P1 can start:**

- `SeqLock<T: Pod>` — the generic version of the SeqLock in the old `ChainHead`, which was hardcoded to
  `(B256, u64)`. `Pod` already exists in the crate for `AtomicCell`. Old fence reasoning is in git history.
- `semaphore` — **in flight**
- MPMC ring (Vyukov), `SegQueue`, channel + select, `epoch-gc`, skiplist
- `time` — Monotonic, Lamport, HLC, rdtsc

⬜ P1 not started.

> **On writing:** blog posts are an *output* of finished work, not an input to it. Nothing here is
> scheduled around them. When a product reaches a rung worth writing about, the implementation
> conversation plus the measurement notes are the raw material — the post gets designed then, not now.
