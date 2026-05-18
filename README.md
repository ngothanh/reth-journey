# The Inheritance Plan v2 — Reth Core + 7-System Mastery + HFT Destination (36-Month Daily Plan)

> **Start**: 2026-04-27
> **Horizon**: 36 months, decision gates at M12 / M24 / M30 / M36
> **Commitment (v2)**: 30h/week M1–M18 → **40h/week M19–M34** (HFT-primary window) → 30h/week M35–M36
> **Schedule**: Mon-Sat work, Sunday rest + weekly ritual
> **Destination**: Path E — Tier A HFT destination-tier IC
> **Mastery target**: core techniques of Reth, Chronicle Queue, ScyllaDB/Seastar, Aeron, Qdrant, TigerBeetle, Tempo
> (Disruptor done in separate repo, OUT of this workspace)

---

## Strategic Frame (v2: Three-Track + 7-System Mastery Anchor)

This plan integrates three tracks that share a single substrate of low-level primitives, anchored to mastery of
the core techniques of 7 reference systems:

- **Reth core (M1–M18 primary, M19–M36 maintenance)** — workspace crates mirroring alloy / reth / revm, scaling to
  three flagship deliverables (`storage-trie` v1.0, `exec-vm` v1.0, `consensus-engine` v1.0). Plus parallel EVM
  (block-stm) and era-file snapshot sync added in v2 to close Reth coverage gaps.
- **HFT depth (M15 scaffold, M19–M34 primary)** — matching engine (with STP, stop-limit, auction, MBO, FIX, circuit
  breakers), deterministic ledger (with VOPR + static-mem + io_uring throughout), kernel-bypass market data, Aeron-style
  messaging (with multicast + Image/loss-detector + Archive/replay), Raft + BFT + **VSR (new)** consensus, mini-LSM
  database (with LCS + TWCS compaction), HNSW vector index. Built on a Seastar-style **shard-per-core runtime (new)**.
- **Tempo application layer (additive throughout)** — `tempo-tx-envelope`, `tempo-evm-ext`, `tempo-payment-lane`. Layer
  7 on top of everything below. Tempo remains optionality; Path D at M24 is conditional on the three-condition test (≥15
  Tempo PRs merged, ≥2 maintainer relationships, upstream substantively engaged with `tempo-payment-lane`). Scope capped
  at 3 crates per v2 confirmation.

**The principle**: no primitive is built twice. A `wal` crate ships once at W26 and is consumed by `storage-trie` (
Reth), `ledger-deterministic` (HFT), `mini-db` (databases), and downstream by `consensus-raft` snapshots and
`matching-engine` durable command logging. Same for `bufpool`, `time`, `txn`, `p2p`, `consensus-raft`, `consensus-bft`,
`consensus-vsr` (v2), `runtime-thread-per-core` (v2), `mmap-queue` (v2). Layer-5 products inherit ~70% of their
components; Tempo at Layer 7 inherits ~85%.

**7-system mastery anchor (v2)**: every additive crate or expansion in v2 underwrites a specific core technique of
one of the 7 reference systems — Reth, Chronicle Queue, ScyllaDB/Seastar, Aeron, Qdrant, TigerBeetle, Tempo. If a
proposed item doesn't map to a named core technique of one of those 7 (and underwrite a Tier A HFT interview question
or portfolio artifact), it doesn't ship.

**Final-phase deliverables (v2)**:

- `storage-trie` v1.0 (W44) — reth storage + trie re-implementation **[bar b]**
- `exec-vm` v1.0 (W68) — revm + reth evm re-implementation; **block-stm parallel execution variant by W94 (v2)** **[bar b]**
- `consensus-engine` v1.0 (W91) — reth consensus + engine API re-implementation **[bar b]**
- `matching-engine` v1.0 (W74) — multi-symbol order book + perpetuals + raft-replicated; **v1.5 (W82) adds STP +
  iceberg + stop/stop-limit + auction matching + MBO feed + FIX session + circuit breakers (v2)** **[bar c]**
- `ledger-deterministic` v0.5 (W83) → **v1.0 (W92) adds VOPR-style deterministic simulator + static-memory invariants +
  io_uring I/O + VSR replication (v2)** **[bar c]**
- `messaging-aeron` v0.5 (W79) → **v0.7 (W84) adds UDP multicast + Image/loss-detector + Aeron Archive (recording +
  replay) (v2)** **[bar c]**
- `marketdata-kernelbypass` v0.5 (W90) — epoll + io_uring + AF_XDP **[bar c]**
- `mini-db` v1.0 (W100) — full LSM database (CAPSTONE: assembles `wal` + `recovery` + `txn` + `bufpool` +
  `storage-trie` + `bloom` + `lsm-core` with **LCS + TWCS + STCS compaction (v2)**) **[bar b]**
- `vector-db` v0.5 (W104) — HNSW + SQ/PQ + filtered search + **segment manager (v2)** **[bar b]**

**Net-new crates introduced in v2** (close coverage gaps for Scylla/Seastar, Chronicle, TigerBeetle):

- `runtime-thread-per-core` v0.1 (W30) → v0.5 (W57) → v1.0 (W84) — Seastar-style shard-per-core scheduler +
  `submit_to` cross-shard message passing + reactor loop. Layer-1 substrate. **[bar c]**
- `mmap-queue` v0.1 (W31) → v0.5 (W78) — Chronicle Queue mirror: excerpt cursor API + roll cycles + pretouch /
  page-warming + Wire-style schema + index files. Layer-2 alongside `wal`. **[bar c]**
- `consensus-vsr` v0.1 (W68) → v0.5 (W74) → v1.0 (W90) — TigerBeetle's Viewstamped Replication. Layer-4 alongside
  `consensus-raft` and `consensus-bft`. Consumed by `ledger-deterministic`. **[bar c]**

**Tempo crate deliverables (additive, optionality preserved, capped per v2)**:

- `tempo-tx-envelope` v0.1.0 — W66 **[bar b]**
- `tempo-evm-ext` scaffold W54 → v0.1.0 W91 **[bar b]**
- `tempo-payment-lane` scaffold W83 → v0.1.0 W91 **[bar b]**

**Bar policy (v2)**:
- **bar (b)**: well-tested + fuzz + property tests + criterion benchmarks + perf-regression CI + documentation.
  Applied to: Reth track (`storage-trie`, `exec-vm`, `consensus-engine`), database/vector (`lsm-core`, `mini-db`,
  `vector-db`), Tempo.
- **bar (c)**: bar (b) PLUS deterministic simulator (VOPR-style) where applicable + static-memory invariants +
  zero-allocation hot paths + 4000+ runtime hours by M36. Applied to: HFT critical path (`matching-engine`,
  `ledger-deterministic`, `messaging-aeron`, `marketdata-kernelbypass`, `consensus-raft`, `consensus-bft`,
  `consensus-vsr`) and the Layer-1 substrate (`concurrent`, `time`, `bufpool`, `wal`, `recovery`, `txn`, `epoch-gc`,
  `bloom`, `runtime-thread-per-core`, `mmap-queue`).

If a crate carries the **[bar c]** label, missing the bar at v1.0 means the scope was wrong — audit before tagging.

---

## Decisions Locked (v2)

These were the open questions; the answers below are baked into the plan.

1. **Three concurrent tracks from M15** — Reth core remains primary M1–M18; HFT begins as scaffold at W58 (
   matching-engine v0.0) and becomes primary M19–M34; Tempo is additive throughout.
2. **Inheritance discipline** — every Layer-5 product crate explicitly lists which Layer-0–Layer-4 primitives it
   inherits and which 3–5 components are net-new. No primitive built twice.
3. **Layer-1 primitives ship before products** — `time` (W6), `backpressure` (W11), `bufpool` (W14), `wal` (W26),
   `recovery` (W29–W30), **`runtime-thread-per-core` v0.1 (W30) → v0.5 (W57) → v1.0 (W84) (v2)**, `txn` (W42 v0.5, W72
   v1.0 with 2PC), `p2p` (W52–W55), `consensus-raft` (W56–W67), `consensus-bft` (W64–W73), **`consensus-vsr` v0.5 (W74)
   → v1.0 (W90) (v2)**, `messaging-aeron` (W76–W79, expanded v2), **`mmap-queue` v0.1 (W31) → v0.5 (W78) (v2)**,
   `marketdata-kernelbypass` (W85–W90).
4. **`mini-db` is the CAPSTONE** — W95–W100. It is the integration moment where five years of "build the primitive once"
   pays off in a single product where ≥70% of LOC is wired-up inheritance. Now includes LCS + TWCS + STCS compaction
   strategies (v2).
5. **`vector-db` stops at v0.5** — W101–W104, single-node, HNSW + SQ/PQ + filtered search + segment manager. Not
   pursued to distributed because Raft and sharding are already covered by `matching-engine` v1.0 (W74) and `mini-db`
   (W100); reuse, don't rebuild.
6. **Operations-hours target**: 2000+ runtime hours by M30; 4000+ by M36. Live deployment begins W106 (M25 mid).
7. **M24 decision is five-pathed** — Path A (extend Reth), Path B (post-Reth systems), Path C (catch-up), Path D (Tempo
   pivot, conditional), Path E (HFT destination-tier IC track, **default destination per v2**).
8. **Destination landing in Phase 7** — applications start W125; interview prep W119; flagship blog post W121; offer
   decision W131; resignation + relocation W132; arrival W134; first month at new firm W135–W144.
9. **Hours schedule (v2)** — 30h/wk M1–M18 (W1–W72), **40h/wk M19–M34 (W73–W136)**, 30h/wk M35–M36 (W137–W144). The
   bump from 30 → 40 funds the bar-(c) work on the HFT critical path (VOPR, static-mem, multicast, Archive, VSR) plus
   the 3 net-new substrate crates.
10. **Bar policy (v2)** — bar (b) for Reth + database/vector + Tempo; bar (c) for HFT critical path + Layer-1
    substrate. Uniform bar (c) is rejected as dishonest mirroring (Reth itself doesn't apply bar (c)). See "Final-phase
    deliverables" above for per-crate bar labels.
11. **Disruptor is OUT OF SCOPE** (v2) — completed in a separate repo. Do not propose Disruptor work in this workspace.
    Ring-buffer techniques are already covered in `concurrent` (bounded MPMC Vyukov W11) and `mmap-queue` (Chronicle
    excerpt cursor W31).
12. **7-system mastery anchor (v2)** — every additive crate or scope expansion must map to a named core technique of
    Reth, Chronicle Queue, ScyllaDB/Seastar, Aeron, Qdrant, TigerBeetle, or Tempo, AND underwrite a specific Tier A HFT
    interview question or portfolio artifact. See "v2 Crate Slotting Schedule" section below.

---

## Workspace Layout (Seven Layers, Built Incrementally)

Every crate below sits in `crates/<name>/` of a single Cargo workspace. Builds proceed bottom-up; each layer is fully
usable in isolation.

```
LAYER 0 — bootstrap primitives (W1-W11)
  eth-primitives/        W1-W4    -> alloy-primitives mirror; types, hashing, atomic-cached hashes
  eth-storage-cache/     W2       -> revm::CacheDB mirror; Page, Account, ShardedCache
  eth-network-codec/     W3       -> reth-eth-wire framing; LengthDelimitedCodec, BackpressureStrategy enum
  eth-rlp/               W5       -> alloy-rlp; Encodable/Decodable + derive
  eth-primitives-derive/ W4       -> proc macros: b256!, address!, RlpEncodable
  eth-consensus/         W6-W13   -> alloy-consensus; Header, TxEnvelope, EIP fee math
  eth-eips/              W14-W15  -> alloy-eips; 1559/4844/7702/7685 isolated
  eth-rpc-types/         W16      -> alloy-rpc-types; Block, Filter, TransactionRequest
  eth-stage/             W22      -> reth-stages Stage trait + Pipeline
  exec-vm/               W9-W94   -> revm; opcodes, gas, journal, precompiles.
                                      v1.0 (W68): sequential EVM.
                                      v1.5 (W94, v2): block-stm parallel execution variant —
                                      optimistic concurrent execution with read/write-set tracking,
                                      versioned memory, dependency-driven re-execution.   [bar b]
  eth-trie/              W10-W20  -> alloy-trie; Nibbles, HashBuilder, ProofRetainer

LAYER 1 — universal low-level primitives (W4-W84)
  concurrent/            W4-W37   -> crossbeam-utils + crossbeam-queue + crossbeam-channel + crossbeam-skiplist mirror;
                                      CachePadded, Backoff, AtomicCell, Parker (W4), bounded MPMC Vyukov ring (W11),
                                      unbounded MPMC SegQueue (W26), select!-style multi-channel (W63),
                                      lock-free concurrent skiplist (W37, consumes epoch-gc).
                                      [bar c]
                                      INHERITED BY: backpressure (W11), wal (W26), lsm-core (W38),
                                      matching-engine (W58+), messaging-aeron (W77), consensus-raft commands,
                                      runtime-thread-per-core (W30)
  time/                  W6       -> monotonic + Lamport + HLC stub + hardware-ts trait
                                      [bar c]
                                      INHERITED BY: wal (W26), recovery (W29), txn (W42),
                                      matching-engine (W58), ledger (W80), messaging-aeron (W76),
                                      tempo-tx-envelope (valid_before/valid_after timestamps),
                                      runtime-thread-per-core (W30)
  backpressure/          W11      -> extracted from eth-network-codec's BackpressureStrategy enum
                                      INHERITS: concurrent (bounded MPMC)   [bar c]
                                      INHERITED BY: matching-engine, messaging-aeron, marketdata,
                                      runtime-thread-per-core
  bufpool/               W12-W14  -> LRU-K page cache + pin/unpin + dirty page tracking
                                      EXTRACTED FROM eth-storage-cache Page work (W2)   [bar c]
                                      INHERITED BY: storage-trie, wal, mini-db, vector-db, mmap-queue (W31)
  epoch-gc/              W33-W37  -> crossbeam-epoch mirror; epoch-based memory reclamation foundation
                                      for any lock-free data structure that hands out pointers.   [bar c]
                                      INHERITED BY: concurrent::skiplist (W37), matching-engine lock-free
                                      price level (W74), runtime-thread-per-core cross-shard queue
  [NEW v2]
  runtime-thread-per-core/ W30 v0.1 -> Seastar-style scheduler. v0.1: per-core pinned worker threads +
                          W57 v0.5    futures executor + cross-shard channel via concurrent::SegQueue.
                          W84 v1.0    v0.5: submit_to<F: FnOnce + Send>() cross-shard message passing +
                                      sharded reactor on epoll/io_uring + back-pressure aware scheduling.
                                      v1.0: bar (c) — zero-alloc hot path, deterministic test harness with
                                      injected clock, 1000h runtime burn-in.   [bar c]
                                      MIRROR: Seastar (ScyllaDB runtime substrate)
                                      INHERITS: concurrent, time, backpressure, epoch-gc
                                      INHERITED BY: matching-engine (W58+ runs each symbol on a pinned shard),
                                      marketdata-kernelbypass (W85+ feeds bound to NIC RX queues),
                                      messaging-aeron (W76+ subscribers bound to consumer shards),
                                      mini-db (W95+ uses per-shard memtables),
                                      ledger-deterministic (W80+ runs single-shard deterministic SM)

LAYER 2 — durability primitives (W26-W31, W78)
  wal/                   W26      -> segment + group commit + checksums + replay
                                      INHERITS: time, bufpool, eth-storage-cache::Page   [bar c]
                                      INHERITED BY: storage-trie (W31+), ledger (W80),
                                      matching-engine (durable command log, W74),
                                      consensus-raft (snapshot log), consensus-vsr (W68),
                                      mini-db (W95), mmap-queue (W31, distinct semantics)
  recovery/              W29-W30  -> ARIES 3-pass: analysis, redo, undo
                                      INHERITS: wal, time   [bar c]
                                      INHERITED BY: storage-trie, ledger, mini-db,
                                      matching-engine (replay after replica failover)
  [NEW v2]
  mmap-queue/            W31 v0.1 -> Chronicle Queue mirror. v0.1: roll-cycle file layout
                         W78 v0.5    (daily/hourly cycles) + excerpt cursor API (cursor.next_excerpt(),
                                      cursor.move_to_index(idx)) + Wire-style framed records (typed
                                      header + payload) + per-cycle index file. v0.5: pretouch
                                      (page-warming worker pre-faults next N pages) + bar (c)
                                      determinism harness.
                                      MIRROR: Chronicle Queue (net.openhft:chronicle-queue)
                                      DISTINCT from wal: wal is journal-style for crash recovery;
                                      mmap-queue is reader/writer queue with random-access cursor.
                                      INHERITS: bufpool, time, eth-storage-cache::Page   [bar c]
                                      INHERITED BY: messaging-aeron Archive (W80-W84 recording layer),
                                      matching-engine MBO feed recording (W76),
                                      ledger-deterministic snapshot-stream archival (W88)

LAYER 3 — concurrency + transaction primitives (W38-W42, W72)
  bloom/                 W34      -> classic + counting + scalable variants
                                      INHERITED BY: storage-trie, mini-db, vector-db filter sets
  lsm-core/              W38-W40  -> memtable (skip list) + SSTable + block format + merge iter
                                      INHERITS: bufpool, wal, bloom
                                      INHERITED BY: mini-db, storage-trie (alternative engine experiment)
  txn/                   W42      -> v0.5: lifecycle + 2PL + deadlock detect + OCC
                         W72      -> v1.0: + 2PC for distributed
                                      INHERITS: time, wal, recovery
                                      INHERITED BY: storage-trie, ledger, mini-db, matching-engine

LAYER 4 — distribution + transport primitives (W52-W90)
  p2p/                   W52-W55  -> Kademlia + Noise XX + gossip
                                      INHERITS: time, eth-network-codec   [bar c]
                                      INHERITED BY: consensus-raft, consensus-bft, consensus-vsr,
                                      messaging-aeron (peer discovery)
  consensus-raft/        W56-W67  -> election + log replication + membership + log compaction
                                      INHERITS: time, wal, p2p, txn   [bar c]
                                      INHERITED BY: matching-engine v1.0 (W74),
                                      mini-db distribution (W99-100, scope-trimmed)
  consensus-bft/         W64-W73  -> 3-phase voting + locking + fork-choice + evidence
                                      INHERITS: time, wal, p2p   [bar c]
                                      INHERITED BY: consensus-engine (Engine API fork-choice analogue)
  [NEW v2]
  consensus-vsr/         W68 v0.1 -> TigerBeetle's Viewstamped Replication. v0.1: ViewChange +
                         W74 v0.5    NormalOperation + view-number messages + DVC quorum.
                         W90 v1.0    v0.5: full state-transfer + recovery + reconfiguration.
                                      v1.0: bar (c) — VOPR-style simulator harness integrated
                                      (10M scenarios @ M30), zero-alloc message path, static
                                      memory for in-flight messages.
                                      MIRROR: TigerBeetle's VSR (lib/tigerbeetle/src/vsr/*.zig)
                                      INHERITS: time, wal, p2p, runtime-thread-per-core
                                      INHERITED BY: ledger-deterministic v1.0 (W92 replication
                                      protocol — chosen over Raft to honestly mirror TigerBeetle)
  messaging-aeron/       W76-W84  -> v0.5 (W79): term buffer + flow control + NAK gap recovery + IPC + UDP
                                      v0.7 (W84): UDP multicast + Image (per-publication subscriber state +
                                      loss-detector) + Aeron Archive (stream recording to mmap-queue +
                                      bounded replay from recorded position).
                                      INHERITS: time, backpressure, bufpool, runtime-thread-per-core,
                                      mmap-queue (for Archive)   [bar c]
                                      INHERITED BY: matching-engine market data fan-out,
                                      marketdata-kernelbypass downstream
  marketdata-kernelbypass/ W85-W90 -> epoll baseline + io_uring + AF_XDP
                                      INHERITS: time, backpressure, runtime-thread-per-core   [bar c]
                                      INHERITED BY: matching-engine exchange-facing feed handler

LAYER 5 — products (capstones; each is ≥70% inherited)
  storage-trie/          W23-W44  -> MDBX-backed state DB + trie   [bar b]
                                      INHERITS: bufpool, wal, recovery, txn, eth-trie, eth-storage-cache
                                      NET-NEW: MdbxTrieStorage, MerkleStage, pruning, snapshots
  matching-engine/       W58-W82  -> v1.0 (W74): multi-symbol L2 order book + perpetuals + raft-replicated
                                      v1.5 (W82, v2): + STP (self-trade prevention: cancel-oldest /
                                      cancel-newest / decrement-both modes) + iceberg orders + stop +
                                      stop-limit orders + auction matching (open/close uncrossing) +
                                      MBO (market-by-order) feed publishing through messaging-aeron +
                                      FIX 4.4 session layer (logon/heartbeat/resend) +
                                      symbol-level circuit breakers (LULD-style price bands).   [bar c]
                                      INHERITS: time, backpressure, wal, recovery, consensus-raft,
                                      messaging-aeron, runtime-thread-per-core (1 symbol per shard),
                                      mmap-queue (MBO feed recording)
                                      NET-NEW: order book (RB-tree + price-time priority),
                                               risk pre-trade, funding + liquidation engines, ADL,
                                               STP modes, iceberg state machine, stop trigger ladder,
                                               auction uncrossing algo, FIX session, circuit breakers
  ledger-deterministic/  W80-W92  -> v0.5 (W83): deterministic SM + double-entry + journal (TigerBeetle).
                                      v0.7 (W88, v2): VOPR-style simulator harness — random op stream,
                                      injected fault model (network partitions, message reorder, crash
                                      restart), assertion engine, 1M scenario runs/day.
                                      v1.0 (W92, v2): static-memory invariants (zero heap alloc in
                                      apply-loop, all in-flight transfers in pre-sized pools), io_uring
                                      I/O throughout, VSR replication via consensus-vsr.   [bar c]
                                      INHERITS: time, wal, recovery, txn, runtime-thread-per-core,
                                      consensus-vsr (v1.0 replication), mmap-queue (snapshot archival)
                                      NET-NEW: deterministic op set, accounts/transfers schema, snapshots,
                                               VOPR harness, static-mem pools, io_uring submission ring,
                                               VSR-glue layer
  consensus-engine/      W24-W91  -> reth consensus + Engine API   [bar b]
                                      INHERITS: eth-consensus, eth-stage, exec-vm, storage-trie, consensus-bft (fork-choice)
                                      NET-NEW: engine_api server + JWT + payload builder + fork-choice glue
  mini-db/               W95-W100 -> full LSM database (CAPSTONE — inheritance ratio target ≥0.70)   [bar b]
                                      INHERITS: lsm-core, wal, recovery, txn, bufpool, bloom, time, backpressure,
                                      runtime-thread-per-core (per-shard memtables)
                                      NET-NEW: kv API, range scan, snapshot iterator, distributed sharding stub,
                                               LCS + TWCS compaction strategies (alongside existing STCS, v2)
  vector-db/             W101-W104 -> HNSW + SQ/PQ + filtered search + segment manager (STOPS AT v0.5, single-node)   [bar b]
                                      INHERITS: bufpool, bloom, txn (limited), time, mini-db (storage backend)
                                      NET-NEW: HNSW graph construction + greedy search, SQ/PQ quantizers,
                                               filtered search, segment manager (sealed/active segments + merge, v2)

LAYER 6 — production tooling (W105+)
  ops-monitoring/        W105     -> Prometheus exporter + tracing-jaeger bridge
  ops-chaos/             W108     -> deterministic-time chaos harness; fault injection
  ops-deploy/            W107     -> k8s manifests + blue/green; cargo-dist binary release
  ops-runbooks/          W109     -> incident playbooks; auto-paging via Alertmanager

LAYER 7 — Tempo application layer (additive throughout)
  tempo-tx-envelope/     W66      -> type 0x76; extends eth-consensus::TxEnvelope
                                      INHERITS: eth-rlp, eth-primitives, eth-consensus, time
  tempo-evm-ext/         W54 scaf -> mirrors TempoEvm extending revm; precompiles + tx handler
                         W91 v0.1 INHERITS: exec-vm, eth-primitives
  tempo-payment-lane/    W83 scaf -> lane reservation strategy
                         W91 v0.1 INHERITS: consensus-engine, matching-engine (priority queue idea)
```

If you ever feel a crate is "just to learn syntax," stop — find a mirror target in alloy / reth / revm / TigerBeetle /
mini-lsm / Qdrant / Aeron / Chronicle / Seastar. The **27 + 3 (v2) = 30 crates** of this workspace are the deliverable.
Everything else is means. (Layer-1 added `concurrent/` and `epoch-gc/` — crossbeam family mirrors. **v2 adds**:
`runtime-thread-per-core/` (Layer-1, Seastar mirror), `mmap-queue/` (Layer-2, Chronicle Queue mirror), `consensus-vsr/`
(Layer-4, TigerBeetle VSR mirror).)

---

## How to Use

Check off tasks as completed. One day = one section. If you fall behind, adjust forward — don't delete. Sunday ritual
reviews the week.

**Daily 5h block structure**:

- 15 min warm-up (review notes, set intent)
- 90 min deep work 1
- 10 min break
- 90 min deep work 2
- 15 min wrap-up (commit, log, questions)

**Track markers**:

- `[Tempo]` — Tempo-track bullets, additive on top of primary
- `[HFT]` — HFT-track bullets, primary M19–M34; scaffold M15–M18
- `[NEW]` — bullets new to this revision (Layer-1 primitive extractions, new products), not in the original 24-month
  Reth plan

If a Reth (primary M1–M18) task is over time budget, drop the `[Tempo]` bullet for the day. From M19, if an HFT track
task is over budget, drop the Reth maintenance bullet. The plan never delegates the critical-path day to the secondary
track.

---

## Curriculum Principle: Inherited Exercises

**No throwaways.** Every exercise builds a real component in a workspace crate that mirrors a specific upstream module
AND is reused in a later phase. The inheritance discipline isn't aspirational — it is enforced by the workspace
structure. If a crate isn't being consumed by a downstream crate within 6 months, it is over-scoped and should be
pruned.

Inheritance ratio target by crate type:

- Layer-0 mirror crates: 100% net-new (they are the foundation)
- Layer-1 primitive crates: 100% net-new but immediately inherited 3+ ways
- Layer-2/3 primitives: ≥30% inherited from Layer-0/1
- Layer-4 distribution primitives: ≥50% inherited
- Layer-5 products (matching-engine, ledger, mini-db, vector-db, consensus-engine): **≥70%** inheritance ratio
- Layer-7 Tempo crates: **≥85%** inheritance ratio (Tempo crates are thin extensions)

If a Layer-5 crate breaks the 70% rule at v0.5, the scope was wrong. Audit before shipping v1.0.

---

## North Star Metrics

### Reth track (primary)

| Metric                                     | M6 | M12 | M18 | M24 | M30 | M36 |
|--------------------------------------------|----|-----|-----|-----|-----|-----|
| Paradigm ecosystem PRs merged              | 10 | 25  | 50  | 80  | 100 | 110 |
| Reth PRs merged                            | 0  | 15  | 35  | 60  | 75  | 85  |
| Storage/Trie PRs                           | 0  | 10  | 20  | 30  | 35  | 40  |
| Execution PRs (revm + reth evm)            | 0  | 3   | 10  | 20  | 25  | 28  |
| Consensus/Engine PRs                       | 0  | 0   | 3   | 10  | 12  | 15  |
| PR reviews given (substantive)             | 0  | 10  | 40  | 100 | 150 | 200 |
| Features led end-to-end                    | 0  | 0   | 1   | 3   | 5   | 7   |
| Reth-side production crates shipped        | 0  | 1   | 2   | 3   | 3   | 3   |
| Direct relationships with Reth maintainers | 1  | 3   | 5   | 8   | 9   | 10  |
| Conferences attended                       | 0  | 0   | 0   | 2   | 3   | 4   |

### HFT track (begins M15) — v2

| Metric                                          | M18  | M24   | M30  | M36  |
|-------------------------------------------------|------|-------|------|------|
| HFT-side production crates shipped              | 1    | 5     | 9    | 12   |
| matching-engine version                         | scaf | v1.0  | v1.5 | v1.5 |
| matching-engine advanced features at v1.5       | —    | —     | STP+iceberg+stop-limit+auction+MBO+FIX+CB | same |
| ledger-deterministic version                    | —    | v0.5  | v1.0 | v1.0 |
| ledger VOPR scenarios run cumulatively (v2)     | —    | —     | 10M  | 100M |
| messaging-aeron version                         | —    | v0.5  | v0.7 | v1.0 |
| aeron multicast + Image + Archive shipped (v2)  | —    | —     | yes  | yes  |
| marketdata-kernelbypass version                 | —    | v0.5  | v0.7 | v1.0 |
| mini-db version (incl. LCS + TWCS, v2)          | —    | —     | v1.0 | v1.0 |
| vector-db version                               | —    | —     | v0.5 | v0.5 |
| **runtime-thread-per-core version (v2)**        | —    | v0.5  | v1.0 | v1.0 |
| **mmap-queue version (v2)**                     | —    | v0.1  | v0.5 | v0.5 |
| **consensus-vsr version (v2)**                  | —    | v0.5  | v1.0 | v1.0 |
| **exec-vm block-stm variant (v2)**              | —    | —     | v1.5 | v1.5 |
| Runtime hours on chaos-tested rig               | 0    | 200   | 2000 | 4000 |
| P99 matching latency (single-symbol, 1M orders) | —    | <5μs  | <2μs | <1μs |
| P99 marketdata fan-out latency (IPC)            | —    | <10μs | <5μs | <2μs |
| P99 ledger commit latency (single-shard, v2)    | —    | <10μs | <3μs | <1μs |
| Public blog posts shipped                       | 0    | 1     | 4    | 7    |

### 7-system coverage score (v2) — read at each decision gate

| Reference system  | M18  | M24  | M30  | M36  |
|-------------------|------|------|------|------|
| Reth              | 60   | 78   | 85   | 90   |
| Chronicle Queue   | 25   | 40   | 70   | 80   |
| ScyllaDB/Seastar  | 10   | 35   | 65   | 75   |
| Aeron             | 30   | 55   | 80   | 88   |
| Qdrant            | 0    | 40   | 80   | 85   |
| TigerBeetle       | 20   | 50   | 80   | 88   |
| Tempo             | 60   | 85   | 95   | 95   |
| **Weighted total**| **30** | **55** | **78** | **85** |

The weighted score is the same metric used in the v2 gap audit (per-system weight by Path E underwriting power).
Audit at each decision gate. If you're under target at M24, trigger Path C (catch-up) or trim Tempo additive time.

### Tempo track (additive; runway optionality)

| Metric                                        | M6 | M12 | M18 | M24 | M30 | M36 |
|-----------------------------------------------|----|-----|-----|-----|-----|-----|
| Tempo orientation depth (0-5)                 | 1  | 3   | 4   | 5   | 5   | 5   |
| Tempo PRs merged                              | 0  | 0   | 10  | 25  | 32  | 38  |
| TIP specs read end-to-end                     | 1  | 5   | 10  | all | all | all |
| TIP discussions participated in (substantive) | 0  | 1   | 3   | 8   | 12  | 16  |
| Tempo-flavored workspace crates shipped       | 0  | 0   | 2   | 3   | 3   | 3   |
| Direct relationships with Tempo maintainers   | 0  | 1   | 2   | 4   | 5   | 6   |
| Tempo design-partner-facing feature shipped   | 0  | 0   | 0   | 1   | 1   | 2   |

### Runway / lifestyle

| Metric                                     | M6   | M12  | M18  | M24    | M30    | M36        |
|--------------------------------------------|------|------|------|--------|--------|------------|
| Months of runway remaining                 | 30   | 24   | 18   | 12     | 6      | (employed) |
| Sleep ≥7h average                          | yes  | yes  | yes  | yes    | yes    | yes        |
| Fitness sessions/week                      | 3    | 3    | 3    | 3      | 3      | 3          |
| Public visibility (followers, talks given) | none | warm | warm | active | strong | strong     |

**Not goals**: "core maintainer of X" or "lead of all current maintainers" or "youngest Tempo contributor." Status is
the OUTPUT of shipped code, reviews, and design engagement — not a directly addressable target.

---

## Risk Register

| Risk                                                                         | Prob | Mitigation                                                                                      | Status |
|------------------------------------------------------------------------------|------|-------------------------------------------------------------------------------------------------|--------|
| Rust foundations extend Phase 1                                              | 70%  | Budget +4 wk, weekly monitor                                                                    | —      |
| Reth PR cycles slow                                                          | 80%  | Many small PRs in parallel                                                                      | —      |
| Reth major arch change                                                       | 60%  | Telegram presence, release notes                                                                | —      |
| Day-job demand spike                                                         | 70%  | Coast mode, 4h floor                                                                            | —      |
| Burnout M8-14                                                                | 80%  | Rest weeks, energy monitor                                                                      | —      |
| Conference budget delay                                                      | 30%  | Year 1 savings earmarked                                                                        | —      |
| Family emergency                                                             | 40%  | Accept slip, adjust                                                                             | —      |
| Motivation dip M12-14                                                        | 70%  | Pre-commit to Phase 4                                                                           | —      |
| Crypto winter                                                                | 40%  | Storage/exec/HFT portable                                                                       | —      |
| Crate scope creep                                                            | 60%  | Lock scope at phase start                                                                       | —      |
| Tempo closes-sources or becomes Paradigm-internal                            | 25%  | Reth contributions remain floor; Tempo time falls back to upstream revm/reth                    | —      |
| Tempo design-partner walks                                                   | 15%  | Same as above; Tempo skills still transfer to other Reth-SDK chains                             | —      |
| Tempo PRs crowd out Reth PRs                                                 | 50%  | Hour cap enforced; Reth velocity primary metric reviewed monthly                                | —      |
| You like Tempo and abandon Reth core                                         | 30%  | 70/30 Reth/Tempo ratio enforced through M18; revisit at M22 only                                | —      |
| Tempo TIPs evolve faster than you can track                                  | 60%  | Weekly Sunday ritual skim; don't aim to know all TIPs                                           | —      |
| Tempo's compliance/KYC features pull you toward business work you don't want | 30%  | Reject contribution paths requiring KYC implementation; stay in execution + consensus + storage | —      |
| HFT track scope balloons (esp. options/exotic features)                      | 60%  | Strict scope lock at W58; options/exotics dropped if behind schedule                            | —      |
| matching-engine doesn't reach v1.0 by W74                                    | 40%  | Drop perp features first, keep spot matching + raft replication                                 | —      |
| matching-engine v1.5 advanced features (STP/auction/FIX/CB) slip past W82    | 50%  | Drop in order: FIX → circuit breakers → auction → MBO. Keep STP + iceberg + stop-limit minimum  | —      |
| ledger VOPR simulator doesn't catch real bugs by M30                         | 40%  | Add canary fault injection (known bugs) weekly; if simulator misses them, harden assertions    | —      |
| consensus-vsr v1.0 slips past W90                                            | 50%  | Drop to v0.7 (reconfig later); ledger-deterministic temporarily uses consensus-raft as fallback | —      |
| runtime-thread-per-core v0.5 not ready by W57 (blocks matching-engine)       | 40%  | v0.3 single-shard fallback for matching-engine W58-W63, swap to v0.5 at W64                     | —      |
| mmap-queue v0.5 conflict with wal API surface                                | 30%  | Audit at W31 design phase: distinct types, no shared trait between wal and mmap-queue           | —      |
| exec-vm block-stm parallel variant memory blowup vs sequential               | 50%  | Time-box at 4 weeks (W91-W94); if blowup > 4×, ship sequential only and write postmortem        | —      |
| Aeron multicast IGMP issues in cloud environment                             | 70%  | Test on bare-metal rig (W82-W84); cloud retrospective uses unicast fanout                       | —      |
| FIX 4.4 session layer scope expands toward FIX 5.0 / FAST                    | 50%  | Lock at FIX 4.4 only. Newer FIX versions are NOT in scope                                       | —      |
| 40h/wk M19-M34 not sustainable past M27                                      | 40%  | Drop Reth maintenance bullet first; if still insufficient, slip ledger v1.0 to M31              | —      |
| mini-db CAPSTONE inheritance ratio falls below 70%                           | 35%  | Audit at W98; cut scope rather than reimplement primitives                                      | —      |
| Operations rig (W106 onward) hardware failure                                | 40%  | Two-machine cluster; spare drive; off-site git mirror                                           | —      |
| Destination-tier interview cycle saturates Phase 7                           | 70%  | Apply early (W125), space interviews, drop weakest opportunities                                | —      |
| Relocation paperwork slips                                                   | 50%  | Start visa research W120; have backup employer                                                  | —      |
| Day-job exit lacks 30-day notice grace                                       | 30%  | Negotiate notice in advance W125; offer transition help                                         | —      |

---

## Principles

1. Deliverables over hours. 5h target, 4h floor. Done at 3h → rest. Stuck at 6h → diagnose.
2. The 9 production crates (3 Reth + 5 HFT + 1 vector) plus the 3 Tempo crates are the real output. Everything else is
   means.
3. Depth over breadth. 3 Reth subsystems + 5 HFT products mastered > 12 shallow.
4. Code reading > code writing in Phase 3+.
5. Ship imperfect > perfect never.
6. AI leverage for architecture research; AI cannot substitute for the reading hours.
7. Blogging optional through M18; expected M19+ (one post per major capstone).
8. Post-Reth trajectory deferred at M12, not forgotten. Reassessed at M24 and M30.
9. Conferences non-negotiable Year 2 (EthCC, Devcon) and Year 3 (one HFT-adjacent: e.g. Distributed Systems Conference,
   QCon).
10. Day-job is infrastructure. Coast mode. Sleep 7h, fitness 3x/week minimum.
11. Energy is the only real budget. Track weekly.
12. M12 / M24 / M30 / M36 are decision points. Each has explicit criteria below.
13. Scope discipline on crates. No feature creep. v0.5 → v0.7 → v1.0 cadence enforced.
14. **Tempo is leverage on the Reth bet, not a parallel bet.** Treat it as such in budgeting, framing, and CV
    positioning through M24. At M24, re-evaluate against Path D criteria.
15. **Path D at M24 is conditional, not aspirational.** Unlocks only if PRs, relationships, and design engagement are
    real.
16. **The deliverable is shipped code on systems that process real production-shape workloads.** Not "core maintainer
    status." Status is downstream of shipped code.
17. **The HFT track is the new optionality.** If Reth ecosystem hiring is soft at M24, the HFT crates plus 2000+ hours
    runtime by M30 (4000+ by M36) is the bridge to destination-tier IC compensation that the Reth track alone may not
    be.
18. **No primitive twice.** This is the most-violated principle in ambitious self-directed plans, and the discipline of
    the workspace layout is what enforces it.
19. **Bar policy (v2).** Bar (c) — VOPR-grade — for HFT critical path + Layer-1 substrate. Bar (b) — fuzz + property +
    bench + perf-CI — for Reth track + database/vector + Tempo. Uniform bar (c) is dishonest mirroring (Reth itself is
    bar b). Each crate's bar is labelled in the Workspace Layout. Missing the bar at tag is a scope failure, not a
    velocity issue.
20. **40h/wk during HFT-primary window (v2).** M1–M18 stays 30h/wk to protect Reth PR velocity. M19–M34 bumps to 40h/wk
    to fund bar-(c) work (VOPR, static-mem, multicast, Archive, VSR). M35–M36 returns to 30h/wk to protect Phase 7
    interview prep + relocation. The 10h/wk bump is non-negotiable for hitting 85/100 coverage by M36.
21. **7-system mastery is the anchor (v2).** Every additive crate or scope expansion must map to a named core technique
    of Reth, Chronicle Queue, ScyllaDB/Seastar, Aeron, Qdrant, TigerBeetle, or Tempo, AND underwrite a Tier A HFT
    interview question or portfolio artifact. If neither, it doesn't ship.
22. **Disruptor is OUT OF SCOPE (v2).** Completed in a separate repo. Ring-buffer-style work in THIS workspace is
    bounded MPMC Vyukov in `concurrent` (W11) and excerpt cursor in `mmap-queue` (W31). No Disruptor-specific crate.

---

## v2 Crate Slotting Schedule

The 3 net-new crates and 5 scope expansions added in v2. Each entry: slot weeks, mirror target, displaced work,
inheritance lineage, Path E justification (what interview question or portfolio claim it underwrites), risk-if-skipped.

### Net-new crate 1: `runtime-thread-per-core` (Seastar mirror)

- **Slot**: v0.1 W30 (1wk seed) → v0.3 W43-W44 (2wk during storage-trie ship buffer) → v0.5 W57-W60 (4wk before
  matching-engine) → v1.0 W83-W85 (3wk alongside ledger v0.7) — **10 weeks total across the timeline**.
- **Mirror target**: Seastar (ScyllaDB substrate). Specifically: `seastar::smp`, `seastar::sharded<T>`,
  `seastar::submit_to()`, reactor loop with epoll backend.
- **Displaced work**: 4wk from Phase 1-2 Rustlings overgenerosity (compressing W4 + W11 redundant exercises),
  3wk from M9 lsm-core padding (W37-W39 had three-week scaffold, compressed to two), 3wk from M14 buffer.
- **Inheritance**: consumes `concurrent`, `time`, `backpressure`, `epoch-gc`. Inherited by: matching-engine,
  marketdata, messaging-aeron, ledger, mini-db.
- **Path E justification**: Tier A HFT firms running Rust/C++ systems universally use thread-per-core. Interview
  question: "Walk me through how you'd design a single-threaded reactor with cross-shard message passing for a
  matching engine handling 1000 symbols across 16 cores." Without this crate, the answer is theoretical; with it,
  it's "here's the code."
- **Risk if skipped**: 30/100 Scylla coverage caps the overall score below 80, and matching-engine v1.5 quality
  is compromised because per-symbol shards become an afterthought rather than a substrate.

### Net-new crate 2: `mmap-queue` (Chronicle Queue mirror)

- **Slot**: v0.1 W31 (1wk inline with storage-trie mdbx work which is also mmap-heavy) → v0.5 W77-W79 (3wk before
  messaging-aeron Archive consumer at W80) — **4 weeks total**.
- **Mirror target**: Chronicle Queue (net.openhft:chronicle-queue). Roll cycles, excerpt cursor, Wire schema,
  index files, pretouch worker.
- **Displaced work**: 2wk from W77-W79 messaging-aeron padding (Archive feature added in v2 anyway, so reuse the
  weeks), 2wk from Phase 4 buffer (W71 reflection + W72 prep compressed).
- **Inheritance**: consumes `bufpool`, `time`. Inherited by: messaging-aeron Archive, matching-engine MBO feed
  recording, ledger snapshot archival.
- **Path E justification**: Chronicle Queue is used in many UK HFT shops (LMAX heritage). Interview: "Design a
  trade-tick recording system that survives crashes and supports random-access replay from any timestamp." Also
  underwrites compliance/audit conversations.
- **Risk if skipped**: 25/100 Chronicle coverage. No portfolio artifact for "I built a journaled queue with
  excerpt-cursor random-access." Aeron Archive can't be done honestly without it (we'd hand-roll the recording
  layer).

### Net-new crate 3: `consensus-vsr` (TigerBeetle VSR mirror)

- **Slot**: v0.1 W68 (1wk seed alongside consensus-bft completion) → v0.5 W74-W76 (3wk before ledger v0.7) →
  v1.0 W88-W90 (3wk integrated with ledger v1.0) — **7 weeks total**.
- **Mirror target**: TigerBeetle's VSR (`lib/tigerbeetle/src/vsr/*.zig`). Viewstamped Replication: ViewChange,
  NormalOperation, DVC quorum, state-transfer, reconfiguration.
- **Displaced work**: 2wk from consensus-bft polish (W73-W75 was generous), 2wk from ledger-deterministic v0.7
  buffer (W85-W86), 3wk from Phase 5 reflection buffer.
- **Inheritance**: consumes `time`, `wal`, `p2p`, `runtime-thread-per-core`. Inherited by: ledger-deterministic
  v1.0 (replaces Raft for ledger replication, per v2 architectural call — matching-engine stays on Raft).
- **Path E justification**: TigerBeetle's VSR choice over Raft is famous in the HFT/database world. Interview:
  "Why might you choose VSR over Raft for a payment ledger? Walk me through view changes." Building it makes the
  answer concrete. Also: the existence of VSR + Raft + BFT in one workspace is portfolio-tier ("I implemented
  three different consensus protocols and chose between them based on workload").
- **Risk if skipped**: 40/100 TigerBeetle coverage. Ledger v1.0 routes through Raft (less honest mirror).
  Distributed-consensus interview answer is narrow.

### Expansion 1: `matching-engine` v1.0 → v1.5 (W74 → W82)

- **Slot**: W75-W82 (8 weeks post-v1.0). New v1.5 tag at W82.
- **Net-new features**: STP (3 modes: cancel-oldest, cancel-newest, decrement-both) at W75-W76; iceberg orders at
  W77; stop / stop-limit ladder at W78; auction matching (open/close uncrossing algo) at W79; MBO feed publishing
  (via messaging-aeron, recorded via mmap-queue) at W80; FIX 4.4 session layer (logon, heartbeat, resend) at W81;
  symbol-level circuit breakers (LULD-style price bands) at W82.
- **Displaced work**: 4wk from W83-W86 ledger ramp (ledger gets compressed via inheritance from existing
  primitives + parallel work with consensus-vsr), 4wk from Phase 5 polish buffer.
- **Path E justification**: Every Tier A HFT firm interview asks about STP, iceberg, stop-limit, auction matching,
  FIX. Without these the matching engine is a toy. With them it's a portfolio artifact.

### Expansion 2: `ledger-deterministic` v0.5 → v0.7 → v1.0 (W83 → W88 → W92)

- **Slot**: v0.5 ships W83 (existing); v0.7 W87-W88 adds VOPR simulator; v1.0 W89-W92 adds static-mem + io_uring
  + VSR replication.
- **VOPR simulator (W87-W88)**: random op stream (Account / Transfer / Lookup mix), fault injection (network
  partition, message reorder, crash restart), assertion engine (double-entry invariants, no overflow, no neg
  balance), 1M scenario runs/day target by M27 / 10M cumulative by M30.
- **Static memory + io_uring (W89-W90)**: zero heap alloc in apply-loop; pre-sized pools for in-flight transfers;
  io_uring submission ring for journal writes (mirrors TigerBeetle's design exactly).
- **VSR replication (W91-W92)**: integrate consensus-vsr v1.0 as ledger's replication protocol.
- **Displaced work**: 3wk from Phase 5 polish + 1wk from W94 buffer.
- **Path E justification**: VOPR is THE signature TigerBeetle artifact. Building a working one is the single
  most-cited portfolio item in HFT/database interviews. Static-mem + io_uring throughout is what separates
  "deterministic-style" from "actually deterministic."

### Expansion 3: `messaging-aeron` v0.5 → v0.7 (W79 → W84)

- **Slot**: W80-W84 (5 weeks post-v0.5). New v0.7 tag at W84.
- **Net-new features**: UDP multicast at W80-W81 (IGMP join, source-specific multicast filtering); Image (per-
  publication subscriber state machine + loss-detector for re-ordering and gap detection) at W82; Aeron Archive
  (stream recording via mmap-queue, bounded replay from recorded position) at W83-W84.
- **Displaced work**: 3wk from W80-W82 ledger buffer (ledger expansion happens in parallel via inheritance), 2wk
  from marketdata Phase prep (W83-W84 was W85-W86 prep, marketdata starts a week earlier).
- **Path E justification**: Aeron multicast is core to HFT venue-side messaging. Image/loss-detector is the
  non-obvious part of Aeron's correctness. Archive (replay) is essential for any compliance-heavy HFT firm.

### Expansion 4: `exec-vm` v1.0 → v1.5 block-stm variant (W68 → W94)

- **Slot**: W91-W94 (4 weeks post-consensus-engine v1.0). New v1.5 tag at W94 — sequential `exec-vm` v1.0 stays
  as the default; v1.5 is a parallel variant under feature flag.
- **Net-new**: optimistic concurrent execution with versioned memory, read/write-set tracking, dependency-driven
  re-execution. Mirror: Aptos's block-stm and the upstream Reth parallel-EVM PRs (Paradigm has open work here).
- **Displaced work**: 2wk from M23 Phase 5 reflection buffer + 2wk from Phase 6 ramp.
- **Path E justification**: Parallel EVM is the hottest area in Reth right now. Tier A HFT firms that touch
  Ethereum-adjacent infrastructure interview on it. Skipping it caps Reth coverage at 78/100.
- **Risk if skipped**: parallel EVM is in active Reth development; arriving at an interview without having
  touched it is a real gap by M36.

### Expansion 5: `lsm-core` + LCS + TWCS (W40 → W42)

- **Slot**: W41-W42 (2 weeks post-STCS at W40). LCS at W41, TWCS at W42.
- **Net-new**: Leveled Compaction Strategy (RocksDB-style, fanout 10), Time-Window Compaction Strategy (one
  SSTable per time window, drop-window compaction for time-series workloads).
- **Displaced work**: 2wk from W41-W43 padding (original plan had three weeks of polish here that overlapped
  with reth-storage PR work that ships in parallel anyway).
- **Path E justification**: Scylla and Cassandra both use LCS and TWCS. Interview: "When would you pick TWCS over
  STCS for a time-series workload?" Without building both, the answer is theoretical.

### v2 capacity math

Net-new weeks added: 10 (runtime) + 4 (mmap-queue) + 7 (consensus-vsr) + 8 (matching v1.5) + 6 (ledger v0.7+v1.0)
+ 5 (aeron v0.7) + 4 (exec-vm v1.5) + 2 (lsm-core LCS+TWCS) = **46 net-new weeks of feature scope**.

Displaced from existing plan: 4 (P1-2 Rustlings) + 3 (M9 lsm padding) + 3 (M14 buffer) + 2 (W77-79 aeron padding)
+ 2 (Phase 4 buffer) + 2 (consensus-bft polish) + 2 (ledger v0.7 buffer) + 3 (Phase 5 reflection) + 4 (W83-86
ledger ramp via inheritance) + 4 (Phase 5 polish) + 3 (Phase 5 polish for VOPR) + 1 (W94 buffer) + 3 (W80-82
ledger buffer + marketdata prep) + 4 (Disruptor weeks freed) + 2 (Tempo TIP-reading consolidation) = **42 weeks
displaced**.

Net delta: 4 weeks. Funded by the M19–M34 hour bump (40h/wk × 64 weeks vs 30h/wk = +640 hours ≈ 21 weeks at
30h equivalent — far more than 4 weeks needed). Comfortable margin.

### Updated phase timeline (changes only)

- **Phase 1 (M1-M3)**: unchanged for Reth core; -2wk Rustlings overgenerosity.
- **Phase 2 (M4-M6)**: unchanged.
- **Phase 3 (M7-M12)**: W30 adds runtime-thread-per-core v0.1 + mmap-queue v0.1 seeds; W41-W42 adds LCS+TWCS;
  W43-W44 adds runtime v0.3.
- **Phase 4 (M13-M18)**: W57-W60 runtime v0.5 ships before matching-engine W58 dependency; W68 adds consensus-vsr
  v0.1 seed.
- **Phase 5 (M19-M24)**: W74-W76 consensus-vsr v0.5; W75-W82 matching-engine v1.5; W77-W79 mmap-queue v0.5;
  W80-W84 messaging-aeron v0.7; W83-W85 runtime v1.0; W87-W92 ledger v0.7→v1.0 with VOPR; W91-W94 exec-vm v1.5
  block-stm.
- **Phase 6 (M25-M30)**: as planned, but every crate now under bar policy. Operations hours target unchanged.
- **Phase 7 (M31-M36)**: unchanged (Phase 7 protected at 30h/wk).

---

# Daily Plan

The full day-by-day plan (Mon–Sat checklists for every week from W1 to W144) lives in
[`plan/INDEX.md`](plan/INDEX.md), with one markdown file per week (`plan/W001.md` through
`plan/W144.md`).

This README holds the **strategic plan only**: frame, decisions, workspace layout, North Star
metrics, risk register, principles, v2 crate slotting schedule, decision gates, inheritance map,
dependency graph, and appendices.

Open `plan/INDEX.md` to navigate by phase/month/week, or jump directly to the week file you need
(e.g. `plan/W030.md` for the runtime-thread-per-core + mmap-queue seed week).

# FINAL SYNTHESIS

## Inheritance Map (ASCII)

This is the data structure that the entire plan exists to produce. Read top-down: each product is mostly the primitives
below it, wired together with thin glue.

```
                     LAYER 7 — Tempo application layer
                ┌─────────────────┬─────────────────┐
                │                 │                 │
       tempo-tx-envelope  tempo-evm-ext   tempo-payment-lane
       (W66, v0.1.0)      (W54s,W91v01)  (W83s, W91v01)
                │                 │                 │
                ↓ inherits        ↓ inherits        ↓ inherits
                │                 │                 │
                │           ┌─────┴────────┐  ┌─────┴────────┐
                │           │              │  │              │
       eth-rlp, eth-     exec-vm   eth-primitives  consensus-engine
       primitives,       (W68)                     (W91)
       eth-consensus
                                                  matching-engine (W74)
                                                  (priority queue idea borrowed)


                     LAYER 5 — products (the deliverables)
       ┌────────────────┬───────────────┬───────────────┬──────────────┬──────────────┐
       │                │               │               │              │              │
  storage-trie    matching-       ledger-       consensus-      mini-db        vector-db
  (W44, v1.0)     engine (W74,    deterministic  engine (W91,   (W100,         (W104,
                  v1.0)           (W83, v0.5)    v1.0)          v1.0)          v0.5)
       │                │               │               │              │              │
       ↓ inherits       ↓ inherits      ↓ inherits      ↓ inherits     ↓ inherits     ↓ inherits
       │                │               │               │              │              │
       bufpool ────────►│               │               │              │              │
       wal ────────────►│ ◄─────────────┤               │              │ ◄────────────│
       recovery ───────►│ ◄─────────────┤               │              │              │
       txn ────────────►│ ◄─────────────┤ ◄─────────────┤               │ ◄───────────│
       bloom ──────────►│               │               │              │ ◄────────────│
       eth-trie ───────►│               │               │              │              │
       eth-storage-cache►│              │               │              │              │
                        consensus-raft  │               │              │              │
                        messaging-aeron │               │              │              │
                        backpressure    │               │              │              │
                        time ──────────►│ ◄─────────────┤ ◄─────────────┤ ◄─────────────│ ◄─────────────│
                                        │               │              │              │
                                                       eth-consensus   lsm-core (W40) │
                                                       eth-stage                       │
                                                       exec-vm                         │
                                                       storage-trie                    │
                                                       consensus-bft                   │
                                                                       wal             │
                                                                       recovery        │
                                                                       txn             │
                                                                       bufpool         │
                                                                       bloom           │
                                                                       lsm-core ───────►mini-db


                     LAYER 4 — distribution + transport primitives
       ┌──────────────────┬────────────────────────┬───────────────────────────────┐
       │                  │                        │                               │
       p2p (W55)    consensus-raft (W67)   consensus-bft (W73)  messaging-aeron (W79)   marketdata-
                                                                                          kernelbypass (W90)
       │                  │                        │                               │
       ↓ inherits         ↓ inherits               ↓ inherits                      ↓ inherits
       │                  │                        │                               │
       time               time, wal, p2p, txn      time, wal, p2p                  time, backpressure
       eth-network-codec
       backpressure


                     LAYER 3 — concurrency + transactions
       ┌──────────────┬─────────────────────┬──────────────────────────┐
       │              │                     │                          │
       bloom (W34)    lsm-core (W40)        txn (W42 v0.5, W72 v1.0)
       │              │                     │
       ↓ inherits     ↓ inherits            ↓ inherits
       │              │                     │
       time           bufpool, wal, bloom   time, wal, recovery


                     LAYER 2 — durability
       ┌──────────────┬───────────────────────┐
       │              │                       │
       wal (W26)     recovery (W30)
       │              │
       ↓ inherits     ↓ inherits
       │              │
       time           wal, time
       bufpool
       eth-storage-cache::Page


                     LAYER 1 — universal primitives
       ┌──────────────┬──────────────────────────┐
       │              │                          │
       time (W6)     backpressure (W11)         bufpool (W14)
       │              │                          │
       ↓ inherits     ↓ inherits                 ↓ inherits
       │              │                          │
       (nothing)     time                       time, eth-storage-cache::Page


                     LAYER 0 — bootstrap
       eth-primitives    eth-storage-cache    eth-network-codec    eth-rlp
       eth-consensus     eth-eips             eth-rpc-types        eth-stage
       exec-vm           eth-trie             eth-primitives-derive
```

**Inheritance Ratios (target ≥0.70 for Layer-5, ≥0.85 for Layer-7; v2 additions in bold)**:

| Crate                            | Layer | Inherits from                                                                              | Target ratio | Audit Week        |
|----------------------------------|-------|--------------------------------------------------------------------------------------------|--------------|-------------------|
| storage-trie v1.0                | 5     | bufpool, wal, recovery, txn, bloom, eth-trie, eth-storage-cache                            | ≥0.70        | W44 Mon           |
| matching-engine v1.0             | 5     | time, backpressure, wal, recovery, consensus-raft, messaging-aeron, runtime-thread-per-core | ≥0.70        | W74 Thu           |
| **matching-engine v1.5 (v2)**    | 5     | + mmap-queue (MBO feed recording), + messaging-aeron v0.7 (multicast/Archive)              | ≥0.70        | W82 Fri           |
| ledger-deterministic v0.5        | 5     | time, wal, recovery, txn, runtime-thread-per-core                                          | ≥0.70        | W83 Wed           |
| **ledger-deterministic v0.7 (v2)**| 5    | + VOPR harness inherits chaos primitives from ops-chaos (W108)                             | ≥0.70        | W88 Sat           |
| **ledger-deterministic v1.0 (v2)**| 5    | + consensus-vsr, + mmap-queue (snapshot archival)                                          | ≥0.70        | W92 Wed           |
| consensus-engine v1.0            | 5     | eth-consensus, eth-stage, exec-vm, storage-trie, consensus-bft                             | ≥0.70        | W91 Wed           |
| **exec-vm v1.5 block-stm (v2)**  | 0/5   | exec-vm v1.0 + concurrent (versioned memory)                                               | ≥0.70        | W94 Fri           |
| mini-db v1.0                     | 5     | lsm-core (now with LCS+TWCS+STCS), wal, recovery, txn, bufpool, bloom, time, backpressure, runtime-thread-per-core | ≥0.70 | W98 Sat, W100 Wed |
| vector-db v0.5                   | 5     | bufpool, bloom, mini-db, wal, time, + segment-manager glue                                 | ≥0.70        | W104 Tue          |
| tempo-tx-envelope v0.1.0         | 7     | eth-rlp, eth-primitives, eth-consensus, time                                               | ≥0.85        | W66 Fri           |
| tempo-evm-ext v0.1.0             | 7     | exec-vm, eth-primitives                                                                    | ≥0.85        | W91 Thu           |
| tempo-payment-lane v0.1.0        | 7     | consensus-engine, matching-engine (priority idea)                                          | ≥0.85        | W91 Thu           |
| **runtime-thread-per-core v1.0 (v2)** | 1 | concurrent, time, backpressure, epoch-gc                                                  | net-new (L1) | W85 Wed           |
| **mmap-queue v0.5 (v2)**         | 2     | bufpool, time, eth-storage-cache::Page                                                     | ≥0.50 (L2)   | W79 Fri           |
| **consensus-vsr v1.0 (v2)**      | 4     | time, wal, p2p, runtime-thread-per-core                                                    | ≥0.50 (L4)   | W90 Fri           |

If any crate falls below its target ratio at audit, scope is wrong — audit before tagging.

---

## Daily Log Template

Fill one row per work-day in `progress.md`. Single source of truth for retrospective queries.

```
| Date       | Hrs | Phase | Track | Crate(s) touched       | Output                                             | Energy 1-5 |
|------------|-----|-------|-------|-----------------------|----------------------------------------------------|------------|
| 2026-04-27 | 5.0 | P1/W1 | Reth  | eth-primitives        | FixedBytes built; W1.Tue tasks checked off         | 4          |
| 2026-04-28 | 4.5 | P1/W1 | Reth  | eth-primitives, notes | Bytes + BytesView built; notes/03 written          | 5          |
```

Notes:

- Date is ISO-8601 (yyyy-mm-dd).
- Track: Reth, Tempo, HFT, Ops, Personal. Multi-track days get a comma list.
- Crate(s) touched: comma list, abbreviated names ok.
- Output: 1-line summary. If a crate version was tagged, prefix with `[TAG vX.Y.Z]`.
- Energy 1-5: 1 = exhausted, can't think; 5 = sharp + flowing.

---

## Weekly Ritual Template

Sunday evening, 60-90 minutes. Append to `progress.md`.

```
## Week N (yyyy-mm-dd → yyyy-mm-dd)

### What shipped
- crate@version (if any)
- PRs merged (Reth: N, Tempo: M, others: K)
- Blog posts (if any)

### Inheritance check
- This week's audits, if any. Pass/fail with ratio.

### Energy + sustainability
- Sleep average this week: Nh
- Fitness sessions: K
- Day-job satisfaction (1-5): X
- Burnout warnings: yes/no

### Surprises
- 2-4 things that didn't go as planned. What I learned.

### Next week
- Top 3 priorities. Top risk.

### Open questions carried forward
- New questions added.
- Questions answered, removed.
- Questions surviving ≥2 weeks → dedicated slot in next week.
```

---

## Decision Gates

### M12 (W48 Wed) — Calibration only, no path change

Three questions. Answer in `progress.md`. No path change at M12 — too early. Just calibrate within the Reth-primary
plan.

- **Reth velocity**: on track for 35 Reth PRs merged by M18?
- **Inheritance discipline**: storage-trie v1.0 shipped with ≥0.70 ratio?
- **Energy / sustainability**: sleep, fitness, day-job satisfaction all green for the past 4 weeks?

### M24 (W96 Fri) — Five-path decision (v2 criteria)

- **Path A** (extend Reth core 6-12 months): strong if ≥3 Reth maintainer Depth-3+, ≥1 feature merged, no HFT inbound.
- **Path B** (post-Reth systems): strong if you want another 12 months of distributed-systems primitive work before any
  job change.
- **Path C** (catch-up): triggered if consensus-engine v1.0 missed OR matching-engine v1.5 not shipped OR Sepolia sync
  not green OR <100 runtime hours OR **7-system coverage score <50 (v2)**.
- **Path D** (Tempo pivot, conditional): real ONLY if all three: ≥15 Tempo PRs merged AND ≥2 direct Tempo maintainer
  relationships AND upstream substantively engaged with `tempo-payment-lane`.
- **Path E** (HFT destination-tier IC track) — **default destination per v2**: strong if matching-engine v1.5 ✓,
  ledger v0.5 ✓, messaging-aeron v0.5 ✓, marketdata-kernelbypass v0.5 ✓, **runtime-thread-per-core v0.5 ✓ (v2)**,
  **consensus-vsr v0.5 ✓ (v2)**, **mmap-queue v0.1 ✓ (v2)**, ≥200 runtime hours, ≥1 blog post live, ≥1 inbound from
  HFT recruiter, **7-system coverage ≥55 (v2)**.

### M30 (W117 Tue) — Path E confirmation or pivot (v2 criteria)

Five questions (v2 expanded from three):

- **Public visibility**: is there inbound? recruiters, maintainers, conference invites?
- **Runtime hours trajectory**: on track for 2000 by M30 end?
- **HFT crate quality (v2)**: are matching-engine v1.5, ledger v1.0 (with VOPR), messaging-aeron v0.7, marketdata,
  runtime-thread-per-core v1.0, consensus-vsr v1.0, mmap-queue v0.5 production-deployable at a real firm?
- **Bar (c) audit (v2)**: do all bar-(c) crates meet the bar — VOPR scenarios run ≥10M cumulative, zero-alloc audit
  passed on matching + ledger + aeron hot paths, deterministic test harness operational?
- **7-system coverage (v2)**: ≥75? Read the score against the M30 column in the coverage table above.

If all five are green, continue Path E. If two are red, re-evaluate against M24's other paths.

### M36 (W144) — Plan close

The plan ends. The work continues. Format the retrospective per W144 instructions.

---

## AI Calibration Review (Quarterly)

Every M3, M6, M9, ... reserve 60 minutes on a Sunday for AI-tool calibration:

- What's the new state of code-generation tools? (claude.ai / cursor / aider / continue.dev)
- Which tasks am I doing manually that AI could do better now?
- Which tasks did I delegate to AI that produced bad output? Why? Stop delegating those.
- Which workspace crates are good prompts? Use the crate as a template for "build me a similar primitive in pattern X."

Treat AI like any other tool: regularly audit fit-for-purpose. AI cannot do the reading hours for you; AI can accelerate
the writing hours.

---

## Workspace Dependency Graph (Final, ASCII)

```
                                LAYER 7 (Tempo, additive)
                              ┌──────────────────────────┐
                              │   tempo-tx-envelope       │
                              │   tempo-evm-ext           │
                              │   tempo-payment-lane      │
                              └──────────┬───────────────┘
                                         │ depends on Layer 5 + Layer 0
                                         ↓
                                LAYER 6 (Ops; deployment-only)
                              ┌──────────────────────────┐
                              │   ops-monitoring          │
                              │   ops-deploy              │
                              │   ops-chaos               │
                              │   ops-runbooks (docs)     │
                              └──────────┬───────────────┘
                                         │ wraps Layer 5
                                         ↓
                                LAYER 5 (Products)
   ┌──────────────────┬──────────────────┬──────────────────┬──────────────────┬──────────────────┐
   │                  │                  │                  │                  │                  │
storage-trie    matching-engine   ledger-determ.   consensus-engine    mini-db          vector-db
   │                  │                  │                  │                  │                  │
   └────────┬─────────┴──────┬───────────┴────────┬─────────┴─────────┬───────┴─────────┬────────┘
            ↓                ↓                    ↓                   ↓                 ↓
                                LAYER 4 (Distribution + transport)
            ┌──────────────┬────────────────┬────────────────┬────────────────────┬──────────────────┐
            │              │                │                │                    │                  │
            p2p     consensus-raft   consensus-bft     consensus-vsr [v2]   messaging-aeron     marketdata-kernelbypass
            │              │                │                │                    │ (v0.7 multicast/   │
            │              │                │                │                    │  Image/Archive v2) │
            └──────┬───────┴────────┬───────┴────────┬───────┴────────┬───────────┴──────┬───────────┘
                   ↓                ↓                ↓                ↓                  ↓
                                LAYER 3 (Concurrency + txn)
                   ┌──────────────┬───────────────┬────────────────┐
                   │              │               │                │
                   bloom        lsm-core         txn
                                 (LCS+TWCS+STCS v2)
                   │              │               │
                   └──────┬───────┴───────┬───────┘
                          ↓               ↓
                                LAYER 2 (Durability + queues)
                          ┌──────────────┬──────────────┬──────────────┐
                          │              │              │              │
                          wal           recovery        mmap-queue [v2]
                          │              │              │
                          └──────┬───────┴──────┬───────┴──────┐
                                 ↓              ↓              ↓
                                LAYER 1 (Universal primitives + runtime substrate)
                          ┌──────────────┬──────────────┬──────────────┬──────────────────────────┐
                          │              │              │              │                          │
                          time        backpressure    bufpool        runtime-thread-per-core [v2]
                          │              │              │              │ (Seastar mirror)
                          └──────┬───────┴──────┬───────┴──────┬───────┘
                                 ↓              ↓
                                LAYER 0 (Bootstrap mirror crates)
                                ┌──────────────────────────────────┐
                                │ eth-primitives    eth-rlp         │
                                │ eth-storage-cache eth-network-cdc │
                                │ eth-consensus     eth-eips        │
                                │ eth-rpc-types     eth-stage       │
                                │ exec-vm           eth-trie        │
                                │ eth-primitives-derive             │
                                └──────────────────────────────────┘
```

Read bottom-up: Layer 0 is built in Phase 1-2. Each layer above is fully built before the layer above it consumes it.
The single exception is the bootstrap-to-Layer-2 wiring (e.g. wal depends on eth-storage-cache::Page, not the other way
around) — but Layer 0's Page primitive is finalized at W14 (`bufpool` wraps it), so wal at W26 inherits a stable Page
contract.

---

## Final Notes (Neutral, v2)

This plan is one path through a much larger space of possible 36-month journeys. The v2 specifics are:

1. **The track architecture**: Reth core (primary M1-M18, maintenance M19-M36) + HFT (primary M19-M34) + Tempo (additive
   throughout) is a deliberate choice to layer skills rather than parallelize them.
2. **The inheritance principle**: 9 Layer-1/2/3 primitives (incl. runtime-thread-per-core, mmap-queue) + 6 Layer-4
   distribution primitives (incl. consensus-vsr) + 6 Layer-5 products + 3 Layer-7 Tempo crates + 4 Layer-6 ops = **28
   workspace crates** (v2: 25 → 28 with three net-new additions). About 60-70% of LOC in the products is wired-up
   inheritance, not net-new code. This is the entire point.
3. **The decision gates (v2)**: M12 (calibrate), M24 (5-path, Path E default), M30 (Path E confirm + bar-(c) audit +
   7-system coverage ≥75), M36 (close, target 85/100). Each gate has explicit criteria to avoid drift.
4. **The Tempo optionality**: capped at 3 crates per v2. Tempo is leverage on the Reth bet, not a parallel bet. Path D
   (Tempo pivot) at M24 unlocks only if three conditions are met. Otherwise Tempo is a CV bullet and a network-warming
   exercise.
5. **The HFT track addition (v2)**: HFT begins at W58 (matching-engine scaffold) and becomes primary M19-M34. New v2
   scope: STP, iceberg, stop-limit, auction, MBO, FIX, circuit breakers in matching v1.5; VOPR + static-mem + io_uring
   + VSR in ledger v1.0; multicast + Image + Archive in messaging-aeron v0.7.
6. **The 7-system mastery anchor (v2)**: every additive crate or scope expansion maps to a named core technique of
   Reth, Chronicle Queue, ScyllaDB/Seastar, Aeron, Qdrant, TigerBeetle, or Tempo. Path E destination is anchored.
7. **The bar policy (v2)**: bar (c) for HFT critical path + Layer-1 substrate; bar (b) for Reth + database/vector +
   Tempo. Uniform bar (c) is rejected as dishonest mirroring.
8. **The hours schedule (v2)**: 30h/wk M1-M18, 40h/wk M19-M34 (HFT-primary window funds bar (c) work), 30h/wk M35-M36.
9. **Operations runtime hours**: 2000+ by M30, 4000+ by M36. A self-built ops rig with monitoring + deployment + chaos
   + VOPR-style ledger simulator is the bridge to destination-tier IC compensation that PRs alone may not be.
10. **Destination landing in Phase 7**: applications W125, final rounds W128-W129, offer decision W131, resignation
    W132, arrival W134, first month W135-W144. Geography neutral.

This is the inheritance plan v2. Read it before writing any code each Monday. Audit ratios at every tag. Reject any
task that fails the "is this consumed by a downstream crate within 6 months?" AND "does this underwrite a named
core technique of one of the 7 reference systems?" tests. The 28 crates are the deliverable. Everything else is means.

---

**End of plan.md** (last line of file).

---

## APPENDIX A: HFT Track Maintenance Schedule (M25-M36 weekly cadence)

The HFT crates produced in Phase 4-5 need sustained operational time to demonstrate production-readiness. This appendix
details the weekly maintenance cadence on top of the day-by-day plan in Phases 6 + 7.

### Weekly HFT maintenance template (W97-W144)

Each week from W97 onward, alongside the daily plan above, the following HFT touchpoints recur:

- **[HFT] Monday**: 15 min — review weekend runtime alerts. Any P99 spikes? Any replica divergence events? Any disk-fill
  warnings?
- **[HFT] Wednesday**: 30 min — check operations dashboards. Capture runtime hours total. Update `progress.md` HFT row.
- **[HFT] Saturday**: 60 min — perform a small chaos drill (rotated weekly: process kill, network partition, clock skew,
  disk fill, slow subscriber). Capture results in `chaos_log.md`.

### Monthly HFT polish checkpoints (M25-M36)

- **[HFT] M25 end (W100)**: matching-engine v1.0 confirmed stable on rig. mini-db CAPSTONE shipped.
- **[HFT] M26 end (W104)**: vector-db v0.5 final. Operations rig provisioned.
- **[HFT] M27 end (W109)**: ops-monitoring + ops-deploy + ops-chaos all shipped. First chaos drill cycle complete. Blog
  #4 live.
- **[HFT] M28 end (W113)**: 800+ runtime hours. matching-engine v1.1 (perf-tuned). Interview prep curriculum begun.
- **[HFT] M29 end (W117)**: 1300+ runtime hours. ledger-deterministic v0.7 (real-load polish). messaging-aeron v0.7 (
  real-load polish).
- **[HFT] M30 end (W117)**: 2000+ runtime hours. Blog #7 (Phase 6 retro) live. M30 decision gate complete.
- **[HFT] M31 end (W121)**: 2500+ runtime hours. Flagship blog post live.
- **[HFT] M32 end (W124)**: 2900+ runtime hours. Recon trip complete.
- **[HFT] M33 end (W128)**: 3300+ runtime hours. First-round + final-round interviews underway.
- **[HFT] M34 end (W132)**: 3500+ runtime hours. Offer accepted; resignation submitted.
- **[HFT] M35 end (W138)**: 3800+ runtime hours. First month at new firm complete.
- **[HFT] M36 end (W144)**: 4000+ runtime hours. Plan close.

### HFT crate quality bumps along the way

Beyond v1.0 of matching-engine (W74), v0.5 of ledger/messaging-aeron/marketdata (W83/W79/W90), and v0.5 of vector-db (
W104), the following minor-version bumps are scheduled:

- **[HFT] matching-engine v1.1** (W111 Fri) — perf-tuned after profiling at 1M+ orders/sec sustained.
- **[HFT] matching-engine v1.2** (W124 — post-recon, before applications) — final polish + bench result update.
- **[HFT] ledger-deterministic v0.7** (W113 — sustained operations) — real-load polish.
- **[HFT] messaging-aeron v0.7** (W113 — sustained operations) — real-load polish.
- **[HFT] marketdata-kernelbypass v0.7** (W113 — sustained operations) — real-load polish.
- **[HFT] vector-db v0.6** (W122 — if used in a public demo, otherwise stay at v0.5).
- **[HFT] mini-db v1.1** (W124 — final polish + capstone bench update).

### HFT-track inheritance audit cadence

The whole point of the workspace is the inheritance ratio. Audits MUST happen at:

- **[HFT] W74 Thu**: matching-engine v1.0 — first product audit. ≥0.70.
- **[HFT] W83 Wed**: ledger-deterministic v0.5 — ≥0.70.
- **[HFT] W90 Sat**: cross-crate audit on the 5-crate HFT stack (matching-engine + ledger + messaging-aeron +
  marketdata-kernelbypass + vector-db prep). The combined system LOC vs inherited-primitive LOC.
- **[HFT] W98 Sat + W100 Wed**: mini-db CAPSTONE — most-important audit. ≥0.70 is the bar; ≥0.75 is the win.
- **[HFT] W117 Tue**: M30 audit across all HFT crates. Update the inheritance ratio table.
- **[HFT] W144 Tue**: M36 final audit. Capture in retrospective.

---

## APPENDIX B: Conference + Public-Visibility Schedule

This plan calls for 4 conferences total across 36 months. Specifics:

- **EthCC Paris** — W57 (M15) — Reth-track + Tempo-track focus. Aim: 3 Reth maintainer 1-on-1s. Tempo maintainer 1-on-1
  if available.
- **Devcon** — W88 (M22) — Reth-track + Tempo-track focus. Aim: deepen 3 existing maintainer relationships. Tempo
  design-partner 1-on-1 if possible.
- **[HFT] Distributed-Systems Conference** (e.g. QCon, P99 CONF, Distributed Systems Conf) — somewhere W113-W124 (
  M28-M32) — HFT-track focus. Aim: present talk if accepted (W114 submission); otherwise attend + 3 IRL connections with
  HFT engineers.
- **[HFT] Domain-specific HFT conference** — somewhere W120-W130 (M30-M33) — final visibility push pre-applications.
  Aim: be visible to firms in shortlist.

Per the M24 / M30 / M36 metrics tables: target 2 conferences by M24, 3 by M30, 4 by M36.

### Public-visibility cadence

- **Phase 1-3 (M1-M12)**: zero public posts. Build in private.
- **Phase 4 (M13-M18)**: Twitter warm-up only. Star repos, technical replies. No original posts.
- **Phase 5 (M19-M24)**: optional blog post if storage-trie or consensus-engine retrospective comes naturally. No
  pressure.
- **Phase 6 (M25-M30)**: 4 blog posts (#4 chaos engineering W109; #5 mini-db inheritance W112; #6 vector-db W115; #7
  Phase 6 retro W117).
- **Phase 7 (M31-M36)**: 1-2 blog posts (flagship W121; optional close W141). Maximum visibility before applications.

### Twitter / Mastodon cadence

- **Phase 1-3**: 1 thoughtful reply per week, no original posts.
- **Phase 4-5**: 1 thoughtful original post + 2 replies per week (mostly technical, mostly about workspace crates).
- **Phase 6**: 2 originals + 4 replies per week. Blog posts pinned.
- **Phase 7**: 3 originals + 5 replies per week through W125. Then back to maintenance level.

---

## APPENDIX C: Day-job + Plan Integration Notes

The plan assumes a coast-mode day-job providing infrastructure (salary, health insurance, sponsored hours). Specifics
that this plan relies on:

- **Hours**: 5h/day × 6 days/week = 30h/week for the plan. Day-job is the remaining bandwidth.
- **Coast mode**: not "phoning it in" — meeting expectations cleanly so the day-job stays infrastructure rather than
  crisis.
- **No on-call**: if your day-job has heavy on-call rotation, that's structural conflict. Negotiate or change before
  Phase 3 (heavy storage work is read-intensive and on-call interrupts ruin reading hours).
- **Resignation timing**: W132 (M34). 30-day notice. Mental health budget: don't burn bridges; help with transition.

### Energy budget per phase (target hours/week)

| Phase | Months  | Reth | HFT | Tempo | Ops | Day-job                 | Sleep+health |
|-------|---------|------|-----|-------|-----|-------------------------|--------------|
| 1     | M1-M3   | 30   | 0   | 0     | 0   | ~40                     | 14 (2h/d)    |
| 2     | M4-M6   | 28   | 0   | 2     | 0   | ~40                     | 14           |
| 3     | M7-M12  | 27   | 0   | 3     | 0   | ~40                     | 14           |
| 4     | M13-M18 | 22   | 3   | 5     | 0   | ~40                     | 14           |
| 5     | M19-M24 | 12   | 13  | 5     | 0   | ~40                     | 14           |
| 6     | M25-M30 | 4    | 18  | 3     | 5   | ~40                     | 14           |
| 7     | M31-M36 | 3    | 17  | 3     | 5   | ~40 (then 0 + new role) | 14           |

Numbers approximate; week-to-week varies. The constraint is total work hours, which stays at 30/week throughout.

---

## APPENDIX D: Risk Mitigation Quick Reference

For each risk in the Risk Register, the **first response** to be tried:

| Risk                                                | First response                                                                      |
|-----------------------------------------------------|-------------------------------------------------------------------------------------|
| Rust foundations extend Phase 1                     | Add 2-4 weeks; cut Phase 2 Foundry PR target by 1                                   |
| Reth PR cycles slow                                 | Open 5 small PRs in parallel rather than waiting on one                             |
| Reth major arch change                              | Adapt within 2 weeks; document the migration in `progress.md`                       |
| Day-job demand spike                                | Drop to 4h floor for 2 weeks; reschedule slipped weeks                              |
| Burnout M8-14                                       | Schedule a 1-week rest; cut Phase 3 sub-deliverables (not the v1.0 tag)             |
| Conference budget delay                             | Skip one conference; use Year 1 savings; rebudget Year 2                            |
| Family emergency                                    | Accept full slip; resume from where it stopped                                      |
| Motivation dip M12-14                               | Pre-commit to Phase 4 Monday W49 task; read EthCC trip details                      |
| Crypto winter                                       | HFT/distributed-systems portion of CV stays liquid; emphasis shift                  |
| Crate scope creep                                   | Cut features at v0.5 to ship; v0.7 or v1.0 picks them back up                       |
| Tempo closes-sources                                | Tempo time → upstream revm/reth contributions                                       |
| Tempo design-partner walks                          | Same as above                                                                       |
| Tempo crowds Reth                                   | Hour-cap honestly; if persistently exceeded, cut Tempo budget 50%                   |
| You like Tempo and abandon Reth                     | The Reth track is contractually primary through M18; review only at M22             |
| Tempo TIPs evolve faster than you can track         | Drop to "storage- and execution-touching TIPs only"; ignore rest                    |
| Tempo compliance/KYC features pulling business work | Stay in execution/consensus/storage; decline KYC PRs                                |
| HFT track scope balloons                            | Drop options/exotics first; keep spot + perps                                       |
| matching-engine doesn't reach v1.0 by W74           | Drop perps polish; ship spot + raft replication as v0.7; v1.0 catches up by W84     |
| mini-db inheritance ratio below 70%                 | Audit at W98; cut scope (delay distributed mode) rather than reimplement primitives |
| Operations rig hardware failure                     | Two-machine setup tolerates one; replacement orderable in 1 week                    |
| Destination-tier interview cycle saturates Phase 7  | Apply early (W125); stagger interviews; drop weakest opportunities                  |
| Relocation paperwork slips                          | W120 visa research is the early-warning; allow 12 weeks                             |
| Day-job exit lacks 30-day notice grace              | Negotiate W125 once offers are likely; offer transition help                        |

---

## APPENDIX E: Cross-Reference — Which Week Builds Which Crate

| Week      | Primary crate work                                                                                         |
|-----------|------------------------------------------------------------------------------------------------------------|
| W1-W4     | eth-primitives v0.1 → v0.2                                                                                 |
| W2        | eth-storage-cache v0.1                                                                                     |
| W3        | eth-network-codec v0.1                                                                                     |
| W4        | eth-primitives-derive v0.1                                                                                 |
| W5        | eth-rlp v0.1                                                                                               |
| W6        | eth-consensus v0.1 + **[NEW] time v0.1**                                                                   |
| W7-W8     | eth-consensus v0.2 → v0.3                                                                                  |
| W9        | exec-vm v0.0.1                                                                                             |
| W10       | eth-trie v0.1                                                                                              |
| W11       | eth-trie v0.1 + **[NEW] backpressure v0.1** + eth-network-codec v0.2                                       |
| W12       | **[NEW] bufpool scaffold** + Phase 1 close                                                                 |
| W13       | eth-consensus v0.4                                                                                         |
| W14       | eth-eips v0.1 + **[NEW] bufpool v1.0** + eth-storage-cache v0.2 (bufpool-backed)                           |
| W15       | eth-eips v0.2 + exec-vm EOF                                                                                |
| W16       | eth-rpc-types v0.1                                                                                         |
| W17       | exec-vm v0.1                                                                                               |
| W18-W21   | exec-vm hardening + eth-rlp v0.2 + eth-trie v0.2                                                           |
| W22       | eth-stage v0.0.1                                                                                           |
| W23-W24   | storage-trie scaffold + consensus-engine scaffold                                                          |
| W25-W26   | **[NEW] wal v0.1** + reth storage PRs                                                                      |
| W27-W28   | storage-trie MDBX + B-tree                                                                                 |
| W29-W30   | **[NEW] recovery v0.5** + storage-trie MVCC                                                                |
| W31-W32   | storage-trie persistent MPT                                                                                |
| W33       | mini-lsm reading + storage-trie path compression                                                           |
| W34       | **[NEW] bloom v0.1** + storage-trie pruning                                                                |
| W35-W36   | storage-trie state commitment + snapshots                                                                  |
| W37-W38   | **[NEW] lsm-core v0.3**                                                                                    |
| W39-W40   | **[NEW] lsm-core v0.5**                                                                                    |
| W41-W42   | **[NEW] txn v0.5** + storage-trie integration                                                              |
| W43-W44   | **storage-trie v1.0**                                                                                      |
| W45-W48   | Reth maintenance + M12 decision                                                                            |
| W49-W51   | exec-vm hardening                                                                                          |
| W52       | **[NEW] p2p v0.1 scaffold**                                                                                |
| W53       | exec-vm complete opcodes                                                                                   |
| W54       | exec-vm precompiles + **[NEW] p2p Noise** + **[Tempo] tempo-evm-ext scaffold**                             |
| W55       | exec-vm journaling + **[NEW] p2p v0.5**                                                                    |
| W56       | **[NEW] consensus-raft v0.1**                                                                              |
| W57       | EthCC Paris                                                                                                |
| W58       | exec-vm dispatch + **[NEW HFT] matching-engine scaffold**                                                  |
| W59       | evmone read + **[NEW] consensus-raft v0.3 (log replication)**                                              |
| W60       | exec-vm hot path + **[Tempo] First Tempo PR** + **[HFT] matching-engine: order book draft**                |
| W61       | exec-vm EOF + **[HFT] matching-engine: cross logic**                                                       |
| W62       | exec-vm + storage-trie integration + **[NEW] consensus-raft membership**                                   |
| W63       | **[HFT] matching-engine v0.5**                                                                             |
| W64       | revm perf + **[NEW] consensus-bft v0.1 scaffold**                                                          |
| W65       | **[HFT] matching-engine: multi-symbol shards**                                                             |
| W66       | **[Tempo] tempo-tx-envelope v0.1.0** + **[HFT] matching-engine perpetuals scaffold**                       |
| W67       | **[NEW] consensus-raft v1.0** + **[HFT] matching-engine v0.7**                                             |
| W68       | **exec-vm v1.0** + **[NEW] consensus-bft v0.5**                                                            |
| W69-W71   | **[HFT] matching-engine perps polish (ADL, funding, liquidation)**                                         |
| W72       | **[NEW] txn v1.0 (2PC)**                                                                                   |
| W73       | **[NEW] consensus-bft v1.0**                                                                               |
| W74       | **[HFT] matching-engine v1.0 (raft-replicated)**                                                           |
| W75       | consensus-engine core methods                                                                              |
| W76       | **[NEW] messaging-aeron v0.1 scaffold**                                                                    |
| W77-W78   | messaging-aeron term buffer + flow control + UDP                                                           |
| W79       | **[NEW] messaging-aeron v0.5**                                                                             |
| W80       | **[NEW] ledger-deterministic v0.1 scaffold**                                                               |
| W81-W82   | ledger v0.5 polish + **[Tempo] payment-lane design**                                                       |
| W83       | **[NEW] ledger-deterministic v0.5** + **[Tempo] tempo-payment-lane scaffold**                              |
| W84       | **[HFT] full-stack integration (matching + ledger + messaging)**                                           |
| W85       | Sepolia sync + **[NEW] marketdata-kernelbypass scaffold**                                                  |
| W86-W87   | marketdata-kernelbypass impl                                                                               |
| W88       | Devcon attendance                                                                                          |
| W89-W90   | **[NEW] marketdata-kernelbypass v0.5**                                                                     |
| W91       | **consensus-engine v1.0** + **[Tempo] tempo-evm-ext v0.1.0** + **[Tempo] tempo-payment-lane v0.1.0**       |
| W92       | **[HFT] paper-trade rig setup (24hr soak)**                                                                |
| W93       | **[HFT] 72hr chaos soak**                                                                                  |
| W94       | Final PR push                                                                                              |
| W95-W96   | M24 calibration + 5-path decision                                                                          |
| W97-W100  | **[NEW] mini-db v1.0 CAPSTONE**                                                                            |
| W101-W104 | **[NEW] vector-db v0.5**                                                                                   |
| W105      | **[NEW] ops-monitoring + ops-deploy**                                                                      |
| W106      | **[HFT] Live deployment begins (paper-trade rig)**                                                         |
| W107      | ops-deploy polish + first uptime week                                                                      |
| W108      | **[NEW] ops-chaos** + first chaos drill                                                                    |
| W109      | ops-runbooks + **Blog post #4: chaos engineering**                                                         |
| W110-W112 | Sustained operations + **[HFT] matching-engine v1.1 (perf-tuned)** + **Blog post #5: mini-db inheritance** |
| W113      | **[HFT] ledger v0.7 + messaging-aeron v0.7 + marketdata v0.7** + interview prep ramp                       |
| W114-W117 | Public visibility + **Blog #6: vector-db** + **Blog #7: Phase 6 retro** + M30 decision                     |
| W118-W120 | Interview prep curriculum + warm-network activation                                                        |
| W121      | **Flagship blog post (the public artifact for inbound)**                                                   |
| W122      | Inbound triage                                                                                             |
| W123-W124 | **Destination geography recon trip**                                                                       |
| W125      | **Active applications begin**                                                                              |
| W126-W129 | **Interview rounds (phone screens + technical + final)**                                                   |
| W130-W131 | **Offer wait + offer decision**                                                                            |
| W132      | **Resignation + relocation logistics**                                                                     |
| W133      | Day-job final 2 weeks + ramp-down                                                                          |
| W134      | **Arrival at destination + onboarding**                                                                    |
| W135-W138 | **First month at new firm**                                                                                |
| W139-W143 | Settling + permanent housing                                                                               |
| W144      | **M36 retrospective + plan close**                                                                         |

---

## APPENDIX F: Personal Workspace Conventions (Adopted From Day One)

These conventions are noted explicitly because they're load-bearing across 36 months.

### Naming

- Crates: `kebab-case`. No `_`.
- Modules within a crate: `snake_case`.
- Type aliases for mirror crates: same name as the source upstream type (e.g. `Address` matches
  `alloy_primitives::Address`).
- Inherited primitive imports: always `use crate_name::TypeName` at file top; never re-export across crates unless
  explicitly re-exporting for a public API.

### Versioning

- v0.0.x: scaffold / seed phase. APIs unstable.
- v0.x.x: development; minor versions break API freely.
- v1.0.0: API frozen. SemVer applies.
- Inheritance audits happen ONLY at v0.5+, v0.7+, and v1.0 tags. v0.0.x and v0.1.x scaffolds are exempt — the discipline
  applies once a crate is "real."

### Branches + PRs

- `main` always green (CI passes, clippy clean, miri clean on annotated crates).
- Local feature branches; rebase before merging.
- One commit per logical change; never amend a published commit.

### Testing

- Every crate: unit tests + at least one integration test by v0.5.
- Layer-2/3/4 crates: proptest by v1.0.
- Layer-5 products: cargo-fuzz + loom for race-prone modules by v0.5.
- Inheritance audit at v1.0 (and v0.5 for Layer-5 capstones).

### Docs

- Every public item has a doc comment by v0.5.
- Every crate has a DESIGN.md by v1.0 with the inheritance tree as ASCII.
- README at workspace root updated whenever a new crate ships v0.5+.

### Notes folder structure

- `notes/01_kotlin_to_rust_delta.md` (W1)
- `notes/02_borrow_checker_errors.md` (W1)
- `notes/03_lifetimes.md` (W1)
- `notes/04_traits.md` (W1)
- `notes/05_smart_pointers.md` (W2)
- `notes/06_pin_unpin.md` (W3)
- `notes/07_variance.md` (W4)
- `notes/08_revm_diff.md` (W18)
- `notes/tempo_orientation.md` (W16)
- `notes/tempo_diff.md` (W18+)
- `notes/tempo_evm_ext_design.md` (W47, W49, W51, W54, W58)
- `notes/tempo_roadmap.md` (W27+ weekly)
- `notes/tempo_discussions.md` (W65)
- `notes/tempo_engine_diff.md` (W73)
- `notes/tempo_build_blockers.md` (W24)
- `notes/tempo_sync_blockers.md` (W85)
- `notes/payment_lane_prior_art.md` (W81)
- `notes/payment_lane_design.md` (W82)
- `notes/matching_engine_design.md` (W58)
- `notes/chaos_log.md` (W108+)
- `notes/network_map.md` (W113)
- `notes/inbound.md` (W121+)
- `notes/relocation_research.md` (W120)
- `notes/recon_notes.md` (W123-W124)
- `EXEC_VM_PERF_BACKLOG.md` (W18+)
- `progress.md` (W1 → W144, the source of truth)

---

**True end of plan.md.**

---

## APPENDIX G: Decision-Gate Worked Examples

### M12 worked example (W48 Wed)

If, at M12 you find:

```
Reth PRs merged: 17 (target was 15)
storage-trie v1.0 shipped ✓, inheritance ratio 0.74 ✓
8 new primitive crates shipped (time, backpressure, bufpool, wal, recovery, bloom, lsm-core, txn) ✓
Sleep avg 7.2h ✓
Fitness 3.1 sessions/wk ✓
Day-job satisfaction 3.5/5 (neutral) — flag
Energy red weeks in last 4: 1
Maintainer Depth ≥2 count: 2 (target was 3)
```

Decision: **stay the course**. Reth PR target met. Inheritance discipline holding. Energy mildly soft on day-job front —
note for management 1:1 but not a path change. Maintainer Depth slightly below target — the W57 EthCC trip is the lever,
not a panic this week. Continue to Phase 4 as planned.

### M24 worked example (W96 Fri) — Path E (HFT IC track) decision

If, at M24 you find:

```
Reth PRs merged: 62 (target 60) ✓
exec-vm v1.0 ✓, consensus-engine v1.0 ✓
matching-engine v1.0 (raft-replicated) ✓, runtime hours 250 ✓
ledger v0.5 ✓, messaging-aeron v0.5 ✓, marketdata-kernelbypass v0.5 ✓
Tempo PRs merged: 11 (target 25, acceptable 15+ — BELOW)
Tempo-payment-lane engaged by upstream: NO substantive engagement (1 polite comment, no follow-through)
Tempo direct maintainer relationships at Depth ≥3: 1 (target 4)
HFT recruiter inbound from Twitter: 1 (Tier B firm)
Reth-adjacent inbound: 1 (a recruiter, no specific firm)
Public visibility: warming but not strong yet
Energy: green
```

Decision: **Path E** (HFT IC track). Tempo three-condition test FAILED (PRs below 15, maintainer relationships below 2,
upstream not engaged). Path D drops to "stablecoin payments infra" generic track inside Tier A/B. Reth ecosystem hiring
is plausible but warm-tier; HFT direction has better signal (matching-engine v1.0 is a stronger artifact for HFT firms
than for Reth-adjacent crypto firms which mostly want Reth maintainer credentials).

Path E plan: Phase 6 ramps to 2000 runtime hours by M30; Phase 7 lands destination geography role.

### M30 worked example (W117 Tue) — Path E confirmation

If, at M30 you find:

```
Runtime hours: 2150 ✓
matching-engine v1.1 perf-tuned, P99 1.4µs ✓
mini-db v1.0 shipped, inheritance ratio 0.78 ✓
vector-db v0.5 shipped ✓
4 blog posts live (#4, #5, #6, #7 today) ✓
Twitter followers from start to today: 200 → 2,400 (positive trajectory)
HFT recruiter inbound count: 4 (from flagship blog reach)
Reth ecosystem maintenance: 3 PRs/month avg, relationships warm
Energy: green
```

Decision: **continue Path E**. Strong signal across all 3 M30 gate questions. Phase 7 proceeds as planned: interview
prep concentrates W118-W120, flagship blog W121, recon trip W123-W124, applications W125, interviews W126-W129, offer
decision W131.

### M36 worked example (W144) — Plan close retrospective

If, at M36 you find:

```
Working in destination geography for ~10 weeks
Reth PRs total: 87 (target 85) ✓
HFT crates shipped: 5 (+ vector-db v0.5) ✓
Tempo PRs total: 18 (well below 38 target, but Path D was not chosen, so this is acceptable)
Runtime hours: 4050 ✓
Blog posts: 7 (target 7) ✓
Conferences: 4 ✓
Reth maintainer relationships at Depth ≥3: 5 (target 9 — below, but path is no longer Reth-primary)
HFT direct IRL connections: 12
Compensation: destination-tier at Tier A firm (specific firm not named here)
```

Retrospective tone: positive overall. Tempo target underperformed because Path E was chosen — that's by design; Tempo
was always optionality. Reth maintenance level should sustain (~2 hrs/wk). Plan closes; the work continues at new firm.

---

## APPENDIX H: One-Line Rules To Live By

In rough order of importance:

1. No primitive twice.
2. Inheritance ratio ≥0.70 on Layer-5 products, ≥0.85 on Layer-7 Tempo crates.
3. Ship at v0.5 if scope is at risk; v0.7 / v1.0 catch up later.
4. 4-hour floor, 5-hour target. Done at 3h → rest.
5. Sleep 7h. Fitness 3x/week. Coast mode on day-job.
6. Sunday ritual is non-negotiable.
7. Read before writing.
8. M12 = calibrate. M24 = five-path decide. M30 = Path-E confirm. M36 = retrospective.
9. Tempo is optionality. Don't let it crowd Reth before M22.
10. HFT track is the new optionality from M15 forward.
11. Public visibility comes online M25+, not before.
12. The 25 crates are the deliverable. Everything else is means.

---

**Plan complete. Move to W1 Monday.**
