# The Inheritance Plan v3 — On-Chain Derivatives Infra + 8-System Mastery + Founding-Engineer Readiness (36-Month Daily Plan)

> **Start**: 2026-04-27
> **Horizon**: 36 months, decision gates at **M6 / M12 / M24 / M30 / M36**
> **Commitment**: 30h/week M1–M18 → **40h/week M19–M34** (venue-build window) → 30h/week M35–M36.
> *Hour caps are a floor, not a ceiling — coverage is non-negotiable; invest more time rather than trim scope.*
> **Schedule**: Mon-Sat work, Sunday rest + weekly ritual
> **North star**: $20–100M net worth before 40, via a founding/core-engineer **token-equity bet** on
> **on-chain derivatives / perps execution infrastructure** (CLOB perp DEXs, execution/settlement engines,
> RWA-perps, supporting high-perf chains/L2s). The job is the **income floor/bridge**, not the goal.
> **Mastery target**: core techniques of Reth, Chronicle Queue, ScyllaDB/Seastar, Aeron, Qdrant, TigerBeetle,
> **Kafka (new)**, Tempo — fused into one replicated perp-DEX-core.
> (Disruptor done in a separate repo; **in** as a reference mirror only, **out** as a workspace crate.)

> **Lineage**: This is **v3**, the reconciliation of `README.md` v2 (three-track HFT-IC plan) against the north-star
> reframe in `RECONCILED_PLAN_v3.md`. The 30-crate workspace is **not reduced — it is re-aimed**: the generic `mini-db`
> capstone is replaced by a **replicated, fault-tolerant, multi-node mini-perp-DEX-core**, three domain crates are
> elevated to first-class, latency measurement is pulled to ~M6, and a Kafka leg closes a distributed-log gap. The v2
> document is preserved in git history; everything operational (daily structure, inheritance discipline, templates,
> conventions) carries forward unchanged.

### §-anchor map (citations in week files)

Week files cite "README.md §N" using the former `RECONCILED_PLAN_v3` section numbering — map them as follows:
§1 = "End state — four readiness bars" · §2 = "Strategic Frame" · §3 = "Coverage matrix" · §4/§4A = "Capstone scope
ladder" · §5 = "North Star Metrics" + "Decision Gates" · §6 = "Ecosystem-selection risk checklist" · §7 =
"Shots-on-goal structure" + Geography · §9 = "Risk Register" · §10 = "Open Questions" · §A = "v3 Crate Slotting
Schedule".

---

## What changed from v2 in one paragraph

v2 ended at *"ready to land a Tier-A HFT IC job (Path E)."* v3 ends at *"ready to place a founding/core-engineer
token-equity bet on an on-chain derivatives venue."* The job becomes the **income floor/bridge** (primary: a
crypto-MM cash seat, underwritten by the latency-craft half), not the destination. The 30-crate workspace is
**not reduced — it is re-aimed**: the generic `mini-db` capstone is replaced by a **replicated, fault-tolerant,
multi-node mini-perp-DEX-core** (`perp-dex-core`), and three domain crates (`oracle-mark`, `risk-engine`,
`liquidation-engine`) are elevated to first-class — with the **real-time cross-margin + liquidation engine** as the
principal-defining artifact (domain-hardest ∧ latency-hardest ∧ the meeting point of all lenses). Latency
measurement is pulled to ~M6 (`latency-lab`). A Kafka leg (`log-distributed`) closes a genuine distributed-log
gap. All eight reference systems are retained, each mapped to a venue component.

---

## End state — four readiness bars (the back-plan target)

The plan terminates at a **bet-readiness state**, defined crisply so milestones can be back-planned to it.

| Bar | Definition | Measured by |
|---|---|---|
| **(a) Core-skill mastery** | Production-fluent across the 8 reference systems; `jeff-dean` + `hft-review` competencies all green; bar (c) on the HFT critical path | 8-system coverage ≥ 80; coverage matrix has no red cells; ≥ 4000 runtime hours |
| **(b) Portfolio clears founding-eng bar** | A multi-node, replicated, cluster-VOPR'd **perp-DEX-core** whose centerpiece is the real-time cross-margin + liquidation engine; plus a latency report, VOPR bug-find writeups, and a flagship blog post | `perp-dex-core` v1.0 shipped (bar c); ≥ 3 supporting artifacts public |
| **(c) Activated network** | Binance-alumni map pointed at on-chain-derivatives founders/early-joins **and** crypto-MM desks | ≥ 5 warm intros logged; ≥ 2 conversations advanced to "would bring you in" |
| **(d) Financial runway** | Income floor secured; ≥ 6–12 months runway at each bet entry | **Crypto-MM cash seat** (Wintermute/Jump-style, remote) as the primary bridge — underwritten by the latency-craft half; runway tracked |

**Everything below back-plans to these four bars.**

### Scope Boundary — what the end state IS, and the three "whole-product" gaps classified

**The end state = the ability to build the ENGINE of a Hyperliquid-class venue** — matching / risk / liquidation /
oracle / settlement as a **verified deterministic replicated state machine**. That is the founding-engineer-defining
work: the hard, differentiated core, not the commodity surface. The three "whole Hyperliquid product" gaps are
classified explicitly so the plan neither over-scopes nor pretends they are done:

- **(c) Vault / spot / staking / bridge product surface — OUT OF SCOPE. Do not schedule.** Team-built product
  surface at lower depth than the engine (spot matching is commoditized; vaults/staking/bridge are business +
  integration logic). Pulling it in is **negative-ROI** — it trades engine-depth hours for breadth that does not
  move the founding-engineer needle.
- **(b) Fully-on-chain + EVM bridge — CONDITIONAL v1.5/v2 extension, NOT core scope.** The async hybrid-settle
  interface (`SettlementId` = VSR op-number, already locked) plus EVM fluency from the Reth spine mean a real chain
  wires into the **same signature** with no engine rewrite. Schedule **only** as a conditional extension *if a
  specific bet demands on-chain settlement.*
- **(a) BFT-on-the-hot-path — COMMITTED TERMINAL APEX (non-optional).** Built before the plan closes, bounded to a
  numeric artifact, sequenced *after* Bet #1 readiness. See **"Terminal Apex — BFT on the hot path"** below and its
  two guardrails. This is the only one of the three that is scheduled, and it is scheduled as the apex, not the core.

---

## Strategic Frame (v3: One Profile, Two Halves, Two Option Legs)

The three v2 tracks collapse into **one profile with two halves and two retained option legs**. Both halves are
**retained in full** — no time constraint forces trimming one for the other.

- **Half 1 — Crypto-protocol spine (the Reth half).** `storage-trie`, `exec-vm` (+ block-stm),
  `consensus-engine`/Engine API. Purpose in v3: (i) protocol fluency to design the **on-chain settlement** side of a
  hybrid CLOB; (ii) optionality into high-perf chains/L2s (a named target sub-category); (iii) *secondary* bridge
  (crypto-infra IC) if the crypto-MM seat isn't taken. *No longer a parallel career bet — it is the spine.*
- **Half 2 — The derivatives venue (the product).** Matching + oracle/funding + real-time risk + liquidation +
  deterministic ledger + replicated SM + distributed log + thread-per-core store, assembled into `perp-dex-core`.
  **This is where domain and systems fuse.** It is the output that clears bar (b).
- **Bridge income (the floor).** Primary: a **crypto-MM cash seat** (remote), hireable on Half-2 latency craft —
  which is exactly why `latency-lab` is pulled to M6. Secondary fallbacks (no skill stranded): crypto-infra IC on the
  Reth spine, or Multiplier coast. The Binance-alumni map points at **two** destinations: on-chain-derivatives
  founders (the bets) **and** crypto-MM desks (the floor).
- **Option leg A — AI-infra (`vector-db`/Qdrant).** Kept at v0.5; repositioned as real-time risk analytics /
  anomaly-on-flow + cheap AI-infra fallback. Not on the critical path.
- **Option leg B — Payments/RWA rails (`tempo-*`).** Kept; repositioned as **RWA-perps collateral rails** — the
  fintech/quote-to-cash edge (the three-way fit). Not on the critical path.

Reth and matching-engine are **two halves of one profile**, and on-chain derivatives infra is precisely their
intersection.

**The principle (unchanged from v2)**: no primitive is built twice. A `wal` crate ships once and is consumed by
`storage-trie`, `ledger-deterministic`, the venue position store, `consensus-raft`/`consensus-vsr` snapshots, and
`matching-engine` command logging. Layer-5 products inherit ~70% of their components; the capstone inherits the most.

---

## Decisions Locked (v3)

These were the open questions; the answers below are baked into the plan.

1. **Both halves retained in full** — Reth spine and the derivatives venue are both first-class core advance crates.
   No "domain wins, spine trims." Reth core is primary M1–M18, maintenance M19–M36; the venue is primary M19–M34.
2. **Inheritance discipline** — every Layer-5 product crate explicitly lists which Layer-0–Layer-4 primitives it
   inherits and which 3–5 components are net-new. No primitive built twice.
3. **`perp-dex-core` is the CAPSTONE** (W103–W113), replacing `mini-db`. It is a **replicated, fault-tolerant,
   multi-node** hybrid CLOB assembling matching + oracle + risk + liquidation + ledger + VSR + log-distributed +
   thread-per-core store. Cluster-level VOPR. Scope ladder is locked below.
4. **`mini-db` demoted + restructured (not deleted)** — the DB substrate (LSM/B-tree, MVCC, WAL, txn, compaction) is
   built once as reusable L2/L3 crates and exercised **inside** the venue position/account/orderbook store (the
   ScyllaDB leg) consumed by `perp-dex-core`. DB-infra optionality is a **deferred thin KV/query facade v0.5** —
   **default = DO NOT BUILD**; build only if all three hold: (a) a specific opportunity whose core deliverable is a
   storage/DB engine; (b) it passes the ecosystem-selection filter; (c) a legible hiring bar where the standalone DB
   artifact actually moves the needle. Conditional exception, not a scheduled milestone.
5. **VSR is the single hot-path consensus + log** — the order/event log, the settlement journal, and the replicated
   state machine are the *same* VSR-replicated log (TigerBeetle pattern). `consensus-raft` is built once for
   `log-distributed` and as the Aeron-Cluster mirror, **never on the matching/settlement hot path**. `consensus-bft`
   ships at v0.5 (PoS fork-choice analogue) and is later **promoted to the committed BFT terminal apex** (item 15) on
   the hot path — VSR remains the v1.0 impl, BFT is the post-readiness swap behind the `ConsensusBackbone` interface.
   No second hot-path consensus or log *concurrently*; the apex is a swap, not a duplicate.
6. **Three domain crates elevated to first-class** — `oracle-mark`, `risk-engine` (**principal-defining**),
   `liquidation-engine`. Funding/mark move out of matching-engine → `oracle-mark`; liquidation/ADL/insurance move out
   → `liquidation-engine` (separation of concerns; each primitive once).
7. **`latency-lab` pulled to M6** — tail-latency + coordinated-omission + tick-to-trade + kernel-bypass-awareness
   harness, reused by **every** bar-(c) crate's bench. Closes the hft-review measurement gap early.
8. **`log-distributed` (Kafka) added off the hot path** — partitioned log, ISR, consumer groups, exactly-once, log
   compaction as a standalone learning artifact **+ the venue's read-side fan-out** via a *projection* of the VSR log,
   exposing two staleness contracts (read-after-commit control plane / bounded-staleness + op-token analytics plane).
9. **RWA-aware from v1.0, RWA features at v1.5** — crypto-native core ships first; v1.0 carries the seams (abstract
   oracle interface, parameterized funding, instrument abstraction) and the narrative edge.
10. **Deterministic-core discipline (cross-cutting)** — the VSR-replicated core stays deterministic and replay-safe;
    all non-determinism and auxiliary state is pushed to the edges; margin/liquidation/settlement replay bit-identically
    across replicas; cluster-level VOPR proves it.
11. **Bar policy (carried from v2)** — bar (c) (VOPR-grade) for the HFT critical path + Layer-1 substrate + domain
    crates; bar (b) (fuzz + property + bench + perf-CI + docs) for the Reth spine + database/vector + Tempo. Uniform
    bar (c) is dishonest mirroring (Reth itself is bar b). Each crate's bar is labelled in the Workspace Layout.
12. **8-system mastery anchor** — every additive crate or scope expansion maps to a named core technique of Reth,
    Chronicle Queue, ScyllaDB/Seastar, Aeron, Qdrant, TigerBeetle, **Kafka**, or Tempo, AND underwrites a portfolio
    artifact or interview question on the bet path. If neither, it doesn't ship.
13. **Disruptor is OUT as a crate, IN as a mirror** — ring-buffer techniques live in `concurrent` (bounded MPMC
    Vyukov) and `mmap-queue` (Chronicle excerpt cursor); cite Disruptor in matching-engine design.
14. **Geography: pre-stage, don't pre-commit** — Dubai (crypto-native/Binance-dense) or Singapore (institutional/Asia)
    decided at Bet #1 time. Now: cheap prep only (visa + token-tax for both; event trips). Relocate only on the flip
    trigger.
15. **BFT-on-the-hot-path is a COMMITTED TERMINAL APEX (non-optional), bounded by numbers** — promote `consensus-bft`
    to a pipelined-HotStuff hot-path protocol (N=4, p99≤2ms, fixed Byzantine-VOPR scenario set), swapped into
    `perp-dex-core` behind the `ConsensusBackbone` interface. **Guardrail 1**: does NOT gate Bet #1 (CFT/VSR core at
    M30 is the readiness bar); sequenced after readiness (W118-W143), may overlap / be built inside the bet.
    **Guardrail 2**: "non-optional" attaches to the bounded artifact, not to open-ended research. See "Terminal Apex."
16. **Consensus sits behind a `ConsensusBackbone` interface — HARD v1.0 acceptance criterion #9.** VSR is the v1.0
    impl; `perp-dex-core` depends on the trait, never on `consensus-vsr` directly. This seam makes item 15 payable.
17. **Scope boundary locked (a/b/c).** Core = the verified deterministic replicated *engine*. **(c)** vault/spot/
    staking/bridge product surface = OUT (do not schedule). **(b)** fully-on-chain + EVM bridge = CONDITIONAL v1.5/v2
    (wires into the locked `SettlementId` async signature; schedule only if a bet demands it). **(a)** BFT apex =
    committed (item 15). See "Scope Boundary."

---

## Workspace Layout (Eight Layers, Built Incrementally)

Every crate below sits in `crates/<name>/` of a single Cargo workspace. Builds proceed bottom-up; each layer is fully
usable in isolation. **v3 adds 6 crates** (`latency-lab`, `log-distributed`, `oracle-mark`, `risk-engine`,
`liquidation-engine`, `perp-dex-core`) and reframes `mini-db`, taking the workspace to ~36 crates; **v3.2 adds 2 more**
(`query-columnar`, `model-check`) + the `txn` v1.1 Percolator milestone + the `sim-storage` harness module → **~38 crates**.

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
                                      v1.5 (W94): block-stm parallel execution variant —
                                      optimistic concurrent execution with read/write-set tracking,
                                      versioned memory, dependency-driven re-execution.   [bar b]
                                      (Spine: on-chain settlement design + high-perf-L2 optionality)
  eth-trie/              W10-W20  -> alloy-trie; Nibbles, HashBuilder, ProofRetainer

LAYER 1 — universal low-level primitives + runtime substrate + latency harness (W4-W84)
  concurrent/            W4-W37   -> crossbeam-utils + crossbeam-queue + crossbeam-channel + crossbeam-skiplist mirror;
                                      CachePadded, Backoff, AtomicCell, Parker (W4), bounded MPMC Vyukov ring (W11),
                                      unbounded MPMC SegQueue (W26), select!-style multi-channel (W63),
                                      lock-free concurrent skiplist (W37, consumes epoch-gc).   [bar c]
                                      INHERITED BY: backpressure, wal, lsm-core, matching-engine,
                                      messaging-aeron, runtime-thread-per-core, latency-lab, risk-engine
  time/                  W6       -> monotonic + Lamport + HLC stub + hardware-ts trait   [bar c]
                                      INHERITED BY: wal, recovery, txn, matching-engine, ledger,
                                      messaging-aeron, oracle-mark (funding clock), latency-lab (rdtsc),
                                      runtime-thread-per-core, tempo-tx-envelope
  backpressure/          W11      -> extracted from eth-network-codec's BackpressureStrategy enum
                                      INHERITS: concurrent (bounded MPMC)   [bar c]
                                      INHERITED BY: matching-engine, messaging-aeron, marketdata,
                                      runtime-thread-per-core, log-distributed
  bufpool/               W12-W14  -> LRU-K page cache + pin/unpin + dirty page tracking
                                      EXTRACTED FROM eth-storage-cache Page work (W2)   [bar c]
                                      INHERITED BY: storage-trie, wal, mini-db, vector-db, mmap-queue,
                                      perp-dex-core (venue store)
  epoch-gc/              W33-W37  -> crossbeam-epoch mirror; epoch-based memory reclamation   [bar c]
                                      INHERITED BY: concurrent::skiplist, matching-engine lock-free
                                      price level, runtime-thread-per-core cross-shard queue
  runtime-thread-per-core/ W30 v0.1 -> Seastar-style scheduler. v0.1: per-core pinned worker threads +
                          W57 v0.5    futures executor + cross-shard channel via concurrent::SegQueue.
                          W84 v1.0    v0.5: submit_to<F: FnOnce + Send>() cross-shard message passing +
                                      sharded reactor on epoll/io_uring + back-pressure aware scheduling.
                                      v1.0: bar (c) — zero-alloc hot path, deterministic test harness with
                                      injected clock, 1000h runtime burn-in.   [bar c]
                                      MIRROR: Seastar (ScyllaDB runtime substrate); glommio + monoio (Rust-native
                                      thread-per-core io_uring runtimes — the day-to-day Rust mirrors)   [NEW v3.2 ref]
                                      INHERITS: concurrent, time, backpressure, epoch-gc
                                      INHERITED BY: matching-engine (1 symbol = 1 shard),
                                      marketdata-kernelbypass, messaging-aeron, mini-db substrate,
                                      ledger-deterministic, risk-engine (shard by account),
                                      log-distributed, perp-dex-core
  [NEW v3]
  latency-lab/           W21 v0.1 -> HdrHistogram + coordinated-omission + rdtsc tick-to-trade + perf
                                      (LLC/false-sharing/NUMA) + kernel-bypass-awareness harness.
                                      Reused by EVERY bar-(c) crate's bench. First tick-to-trade report at M6.
                                      MIRROR: HdrHistogram, perf, rdtsc   [bar c]
                                      INHERITS: time, concurrent
                                      INHERITED BY: every bar-(c) crate bench (matching, ledger, aeron,
                                      marketdata, risk-engine, perp-dex-core latency report)

LAYER 2 — durability + queues (W26-W31, W78)
  wal/                   W26      -> segment + group commit + checksums + replay
                                      INHERITS: time, bufpool, eth-storage-cache::Page   [bar c]
                                      INHERITED BY: storage-trie, ledger, matching-engine (durable command log),
                                      consensus-raft, consensus-vsr, mini-db substrate, mmap-queue, log-distributed
  recovery/              W29-W30  -> ARIES 3-pass: analysis, redo, undo
                                      INHERITS: wal, time   [bar c]
                                      INHERITED BY: storage-trie, ledger, mini-db substrate,
                                      matching-engine (replay after replica failover)
  mmap-queue/            W31 v0.1 -> Chronicle Queue mirror. v0.1: roll-cycle file layout + excerpt cursor
                         W78 v0.5    API + Wire-style framed records + per-cycle index file. v0.5: pretouch
                                      (page-warming) + bar (c) determinism harness.
                                      MIRROR: Chronicle Queue. DISTINCT from wal (journal vs random-access queue).
                                      INHERITS: bufpool, time, eth-storage-cache::Page   [bar c]
                                      INHERITED BY: messaging-aeron Archive, matching-engine MBO feed recording,
                                      ledger-deterministic snapshot archival

LAYER 3 — concurrency + transaction primitives (W38-W42, W72)
  bloom/                 W34      -> classic + counting + scalable variants
                                      INHERITED BY: storage-trie, mini-db substrate, vector-db filter sets
  lsm-core/              W38-W42  -> memtable (skip list) + SSTable + block format + merge iter +
                                      STCS (W40) + LCS (W41) + TWCS (W42) compaction strategies
                                      INHERITS: bufpool, wal, bloom
                                      INHERITED BY: mini-db substrate, storage-trie (alt engine),
                                      perp-dex-core position/orderbook store (ScyllaDB leg)
  txn/                   W42 v0.5 -> lifecycle + 2PL + deadlock detect + OCC
                         W72 v1.0 -> + 2PC for distributed
                         W93 v1.1 -> [NEW v3.2] Percolator MVCC (snapshot isolation; lock/write/data CFs) +
                                      coprocessor predicate pushdown into lsm-core scans. TIMESTAMP ORACLE =
                                      the VSR commit point / op-number (reuse — NO separate TSO service).
                                      MIRROR: TiKV / Percolator.   [bar b]
                                      INHERITS: time, wal, recovery; (v1.1) consensus-vsr (commit point), lsm-core
                                      INHERITED BY: storage-trie, ledger, mini-db substrate, matching-engine;
                                      (v1.1) the venue position/account store (snapshot reads for risk-engine W97+)

LAYER 4 — distribution + transport primitives (W52-W90)
  p2p/                   W52-W55  -> Kademlia + Noise XX + gossip
                                      INHERITS: time, eth-network-codec   [bar c]
                                      INHERITED BY: consensus-raft, consensus-bft, consensus-vsr, messaging-aeron
  consensus-raft/        W56-W67  -> election + log replication + membership + log compaction
                                      MIRROR: [NEW v3.2 ref] openraft (production Rust Raft)
                                      INHERITS: time, wal, p2p, txn   [bar c]
                                      INHERITED BY: log-distributed (OFF hot path) + Aeron-Cluster mirror ONLY.
                                      NOT on the matching/settlement hot path (that is VSR).
  consensus-bft/         W64-W73  -> 3-phase voting + locking + fork-choice + evidence
                                      MIRROR: [NEW v3.2 ref] Malachite (Informal Systems' Rust BFT,
                                      Tendermint/HotStuff-class) — the Rust mirror for the W118 Jolteon apex
                                      INHERITS: time, wal, p2p   [bar c]
                                      INHERITED BY: consensus-engine (Engine API fork-choice analogue).
                                      Spine/optionality only — venue uses CFT (VSR), not BFT. Hold at v0.5.
  consensus-vsr/         W68 v0.1 -> TigerBeetle's Viewstamped Replication. v0.1: ViewChange +
                         W74 v0.5    NormalOperation + view-number messages + DVC quorum.
                         W90 v1.0    v0.5: full state-transfer + recovery + reconfiguration.
                                      v1.0: bar (c) — VOPR-style simulator harness, zero-alloc message path,
                                      static memory for in-flight messages.
                                      MIRROR: TigerBeetle's VSR (lib/tigerbeetle/src/vsr/*.zig)
                                      INHERITS: time, wal, p2p, runtime-thread-per-core
                                      ** THE single hot-path consensus + order/event log + settlement journal. **
                                      INHERITED BY: ledger-deterministic v1.0, perp-dex-core (venue backbone),
                                      log-distributed (read-side projection source)
  messaging-aeron/       W76-W84  -> v0.5 (W79): term buffer + flow control + NAK gap recovery + IPC + UDP
                                      v0.7 (W84): UDP multicast + Image (per-publication subscriber state +
                                      loss-detector) + Aeron Archive (recording via mmap-queue + bounded replay).
                                      (Aeron Cluster Raft = design mirror only; venue hot path is VSR.)
                                      INHERITS: time, backpressure, bufpool, runtime-thread-per-core, mmap-queue   [bar c]
                                      INHERITED BY: matching-engine MD fan-out, marketdata downstream, perp-dex-core
  marketdata-kernelbypass/ W85-W90 -> epoll baseline + io_uring + AF_XDP (DPDK/Onload/ef_vi awareness)
                                      INHERITS: time, backpressure, runtime-thread-per-core, latency-lab   [bar c]
                                      INHERITED BY: matching-engine exchange-facing feed handler, perp-dex-core
  [NEW v3]
  log-distributed/       W63 v0.1 -> Kafka mirror, OFF the hot path. v0.1: partitioned log + ISR seed.
                         W80 v0.5    v0.5: consumer groups + exactly-once + log compaction + the venue's
                                      read-side fan-out via a PROJECTION of the VSR log. Two staleness contracts:
                                      control/risk = read-after-commit; analytics = bounded-staleness + op-token
                                      (zookie). NOT a second hot-path log (that is the VSR log).
                                      MIRROR: Kafka. Closes the jeff-dean distributed-log gap without duplicating
                                      consensus.
                                      INHERITS: wal, mmap-queue, consensus-raft, runtime-thread-per-core, backpressure
                                      INHERITED BY: perp-dex-core (market-data / analytics fan-out)

LAYER 5 — products (capstones; each is ≥70% inherited)
  storage-trie/          W23-W44  -> MDBX-backed state DB + trie   [bar b]  (spine)
                                      INHERITS: bufpool, wal, recovery, txn, eth-trie, eth-storage-cache
                                      NET-NEW: MdbxTrieStorage, MerkleStage, pruning, snapshots
  consensus-engine/      W24-W91  -> reth consensus + Engine API   [bar b]  (spine)
                                      INHERITS: eth-consensus, eth-stage, exec-vm, storage-trie, consensus-bft
                                      NET-NEW: engine_api server + JWT + payload builder + fork-choice glue
  matching-engine/       W58-W82  -> v1.0 (W74): multi-symbol L2 order book + perpetuals + VSR-replicable
                                      v1.5 (W82): + STP (cancel-oldest/newest/decrement-both) + iceberg + stop/
                                      stop-limit + auction matching (open/close uncrossing) + MBO feed (via aeron,
                                      recorded via mmap-queue) + FIX 4.4 session + LULD circuit breakers +
                                      reduce-only/post-only/IOC-FOK/trailing + MARK-PRICE-KEYED TRIGGER SUBSYSTEM +
                                      position-aware orders + pro-rata vs price-time.   [bar c]
                                      ** Funding/mark move OUT -> oracle-mark; liquidation/ADL/insurance move OUT
                                         -> liquidation-engine (each primitive once). **
                                      INHERITS: time, backpressure, wal, recovery, runtime-thread-per-core,
                                      messaging-aeron, mmap-queue, latency-lab, consensus-vsr (replication)
                                      NET-NEW: order book (RB-tree + price-time priority), risk pre-trade,
                                               STP modes, iceberg SM, stop ladder, auction uncrossing, FIX, CB
  ledger-deterministic/  W80-W92  -> v0.5 (W83): deterministic SM + double-entry + journal (TigerBeetle).
                                      v0.7 (W88): VOPR-style simulator (random op stream, fault injection:
                                      partition/reorder/crash-restart, assertion engine).
                                      v1.0 (W92): static-memory invariants (zero heap alloc in apply-loop),
                                      io_uring submission ring, VSR replication via consensus-vsr.   [bar c]
                                      INHERITS: time, wal, recovery, txn, runtime-thread-per-core, consensus-vsr,
                                      mmap-queue, latency-lab
                                      NET-NEW: deterministic op set, accounts/transfers schema, snapshots,
                                               VOPR harness, static-mem pools, io_uring ring, VSR-glue
  [NEW v3 — domain crates]
  oracle-mark/           W77 v0.5 -> Mark vs index vs last; funding rate (premium + interest, periodic).
                         W97+      v1.0 RWA-AWARE SEAMS: abstract oracle interface (crypto-spot vs
                                      TradFi-with-market-hours; funding parameterized for carry/basis).
                                      v1.5 RWA FEATURES: after-hours EMA, holiday calendars. Drives settlement loop.
                                      MIRROR: Hyperliquid/dYdX mechanics; RWA calendars   [bar c]
                                      INHERITS: time, log-distributed
                                      INHERITED BY: risk-engine, liquidation-engine, perp-dex-core
  risk-engine/           W85 v0.5 -> ** PRINCIPAL-DEFINING. ** Real-time IM/MM, cross vs isolated.
                         W97 v1.0    v0.5: multi-instrument cross-margin with fixed haircut (BTC + ETH).
                                      v1.0: cross-margin NETS RISK across positions crediting only 70% of the
                                      netting offset (const 30% haircut, ~0.7 corr, deterministic); recompute
                                      portfolio margin PER MARK TICK across all accounts. Incremental-margin hot
                                      path mandatory at 100k accounts; full recompute only on circuit-breaker/param
                                      change. v1.5: rolling empirical correlation + full SPAN scenario margining +
                                      multi-collateral.
                                      MIRROR: TradFi SPAN / portfolio-margin docs   [bar c]
                                      INHERITS: oracle-mark, ledger-deterministic, runtime-thread-per-core (shard by
                                      account), concurrent, latency-lab
                                      INHERITED BY: liquidation-engine, perp-dex-core
                                      ** CONVERGENCE CELL: jeff-dean-hard (per-tick fan-out) ∧ hft-review-hard
                                         (zero-alloc sub-tick budget) ∧ domain-hard (no spot analog). **
  liquidation-engine/    W100-W102 v1.0 -> Margin-ratio triggers; partial liquidation against book + minimal insurance
                                      fund (fixed-bps fee accrual, default 10% taker, declared param) absorbing the
                                      below-bankruptcy-price shortfall via WATERFALL (position-margin -> fund);
                                      cascade detection + circuit breakers. Tripwire never hit (deterministic
                                      external rate sweep 5/10/20%). v1.5: ADL + socialized loss + keeper market.
                                      The backstop is the moat — not stubbed.
                                      MIRROR: Hyperliquid/dYdX/GMX liquidation   [bar c]
                                      INHERITS: risk-engine, matching-engine, ledger-deterministic
                                      INHERITED BY: perp-dex-core
  perp-dex-core/         W103-W113 -> ** L5 CAPSTONE (replaces mini-db). ** Replicated, fault-tolerant, multi-node
                                      hybrid CLOB. Assembles matching + oracle + risk + liquidation + ledger +
                                      consensus-vsr (Aeron-Cluster mirror only) + log-distributed + thread-per-core
                                      store. v1.0 settle-stub implements the REAL hybrid-boundary interface
                                      (off-chain match / on-chain settle) so v1.5/v2 wire a real chain into the SAME
                                      interface. Cluster-level VOPR. Scope ladder below.
                                      MIRROR: Hyperliquid / dYdX v4 architecture   [bar c]
                                      INHERITS: matching-engine, oracle-mark, risk-engine, liquidation-engine,
                                      ledger-deterministic, consensus-vsr, log-distributed, runtime-thread-per-core,
                                      lsm-core (venue store), bufpool, latency-lab, messaging-aeron, marketdata
  [NEW v3.2 — durable-infra crates (HyperCore/TiKV/DataFusion additions; see .rework/HYPERCORE_ADDITIONS.md)]
  query-columnar/        W84 v0.1 -> Arrow-layout columnar trade-log store + vectorized scan/filter/aggregate
                         W110 v0.5   over the OFF-PATH analytics projection of the VSR log (never the hot path).
                         M31 v0.7    v0.1: RecordBatch + columnar append from the log-distributed projection.
                                      v0.5: vectorized kernels + zone-map (min/max) pushdown + group-by aggregate.
                                      v0.7 (M31 buffer): dictionary/RLE encoding + late-materialization join.
                                      MIRROR: Apache DataFusion (vectorized execution) + arrow-rs (columnar layout).
                                      In-process Rust query API only — NO SQL/RPC surface.   [bar b]
                                      INHERITS: log-distributed (projection source), lsm-core (column-chunk store),
                                      bufpool, time, latency-lab (kernel microbench).  Target >=0.50.
                                      INHERITED BY: perp-dex-core (analytics fan-out / trade-log queries),
                                      risk-engine v1.5 (historical rolling-correlation backfill)
  model-check/           W90  -> Stateright bounded model checker of the CONSENSUS PROTOCOLS (not the impls):
                         W129    W90: VSR safety (single-log linearizability + no-two-leaders-per-view +
                                      commit-monotonicity, N=3). W129: Jolteon 2-chain + TC view-change
                                      (safety under equivocation + liveness after GST, N=4 f=1).
                                      Exhaustive complement to the sampled VOPR (each catches different bugs).
                                      MIRROR: Stateright (Rust); published TLA+ specs of VSR/HotStuff as model SOURCE
                                      (read for invariants — NOT a TLA+ build).   [supports bar (c) consensus claims]
                                      INHERITS: net-new (re-encodes the protocol state machine; no crate dep)
                                      INHERITED BY: the safety claims of consensus-vsr (W90) + BFT apex (W143)
  [Option legs]
  mini-db/               (deferred) -> DEMOTED + RESTRUCTURED (not deleted). DB substrate (lsm-core/wal/recovery/txn/
                                      bufpool/bloom) built once and exercised INSIDE the venue store. DB-infra
                                      optionality preserved as a deferred thin KV/query facade v0.5 — DEFAULT DO NOT
                                      BUILD (build only on the (a)∧(b)∧(c) condition). No standalone capstone tag.
  vector-db/             W101-W104 -> HNSW + SQ/PQ + filtered search + segment manager (STOPS AT v0.5, single-node).
                         (or M31)    Option leg: AI-infra + real-time risk analytics / anomaly-on-flow. May slip to
                                      an optional W-slot / M31 buffer (capstone window takes priority).   [bar b]
                                      INHERITS: bufpool, bloom, txn (limited), time, mini-db substrate (storage)
                                      NET-NEW: HNSW graph + greedy search, SQ/PQ quantizers, filtered search, segments

LAYER 6 — production tooling (W105+)
  ops-monitoring/        W105     -> Prometheus exporter + tracing-jaeger bridge
  ops-deploy/            W107     -> k8s manifests + blue/green; cargo-dist binary release
  ops-chaos/             W108     -> deterministic-time chaos harness; fault injection
  ops-runbooks/          W109     -> incident playbooks; auto-paging via Alertmanager

LAYER 7 — Tempo application layer (additive; RWA-perps collateral rails)
  tempo-tx-envelope/     W66      -> type 0x76; extends eth-consensus::TxEnvelope
                                      INHERITS: eth-rlp, eth-primitives, eth-consensus, time
  tempo-evm-ext/         W54 scaf -> mirrors TempoEvm extending revm; precompiles + tx handler
                         W91 v0.1 INHERITS: exec-vm, eth-primitives
  tempo-payment-lane/    W83 scaf -> lane reservation strategy (RWA-perps collateral rails framing)
                         W91 v0.1 INHERITS: consensus-engine, matching-engine (priority queue idea)
```

If you ever feel a crate is "just to learn syntax," stop — find a mirror target in alloy / reth / revm / TigerBeetle /
mini-lsm / Qdrant / Aeron / Chronicle / Seastar / Kafka / Hyperliquid-dYdX-GMX mechanics. The **~36 crates** of this
workspace are the deliverable. Everything else is means.

---

## Reference-system → venue-component map (all 8 retained)

```
Disruptor   -> matching hot-path ring (ref only; embodied in concurrent)
Chronicle   -> wal / mmap-queue order journal
Aeron       -> messaging-aeron (inter-node transport); Aeron Cluster Raft = design mirror only
Kafka       -> log-distributed (OFF hot path): partitioned log/ISR/consumer-groups/exactly-once/compaction
               + read-side fan-out (a PROJECTION of the VSR log); not a second hot-path log
ScyllaDB    -> lsm-core + runtime-thread-per-core (venue position store)
TigerBeetle -> ledger-deterministic + consensus-vsr: the SINGLE hot-path source of truth —
               settlement + consensus backbone + order/event log are one VSR-replicated log
Qdrant      -> vector-db (option leg)
Tempo       -> tempo-* (RWA collateral option leg)
```

> **One log on the hot path (no primitive twice):** the order/event log, the settlement journal, and the replicated
> state machine are the *same* VSR-replicated log. Kafka-style partitioning/ISR/consumer-groups are exercised **off**
> the hot path — as `log-distributed` and the market-data fan-out reading a projection of the VSR log. Aeron Cluster's
> Raft is a design mirror only. This resolves "Kafka vs VSR." Six systems integrate into the capstone; Qdrant + Tempo
> are retained option legs.

---

## Coverage matrix — competency × lens × (input | output)

Every competency maps to a **learning input (mirror)** and an **output artifact (crate)**, justified by at least one
of {`jeff-dean`, `hft-review`, derivatives-domain}. **No primitive twice** — each output crate is built once and
consumed by the capstone.

| # | Core competency | jeff-dean | hft-review | Derivatives-domain | Mirror | Output crate |
|---|---|---|---|---|---|---|
| 1 | LSM storage engine | Bigtable/LSM | per-shard compaction | orderbook/position store | RocksDB, ScyllaDB | `lsm-core` |
| 2 | Thread-per-core runtime | scheduling/locality | shard-per-core, pinned, SPSC | 1 symbol = 1 shard | Seastar/ScyllaDB | `runtime-thread-per-core` |
| 3 | Distributed log / event sourcing | per-consumer staleness (zookie) | zero-copy append | order-flow / MD backbone | Kafka (off-hot-path) | hot path = the VSR log; `log-distributed` = off-path Kafka + two-contract read-side fan-out |
| 4 | Durability: WAL + mmap queue | commit log | pretouch, CAS header, zero-copy | order/event journal + recovery | Chronicle Queue | `wal`, `mmap-queue`, `recovery` |
| 5 | Consensus / replicated SM **+ the one hot-path log** | Paxos/Spanner; consensus-derived identity + free idempotency | ring replication, zero-alloc msg path | replicated matching; the VSR log is also the order/event log | TigerBeetle VSR (hot path) + HotStuff/HyperBFT (apex); **Aeron-Cluster Raft + Kafka-log = off-path mirrors, no duplication** | **`consensus-vsr` = v1.0 hot-path impl behind a `ConsensusBackbone` interface (acceptance #9); BFT apex = the committed swap behind the same interface** (`consensus-bft` → hot-path-grade); `consensus-raft` → `log-distributed` (off-path) only |
| 6 | Deterministic financial ledger | payment-ledger blueprint | static-mem, io_uring, VOPR | settlement / clearing / continuous PnL | TigerBeetle | `ledger-deterministic` |
| 7 | Matching engine / order book | state machine | single-writer ring, <1μs p99 | derivatives order types + STP + mark-keyed triggers | Disruptor (ref), LMAX | `matching-engine` |
| 8 | Oracle / mark-index-funding | fan-out, freshness | tick ingestion latency | funding; mark vs index vs last; RWA seams v1.0, features v1.5 | Hyperliquid/dYdX; RWA calendars | `oracle-mark` |
| 9 | **Real-time cross/portfolio margin** | per-tick fan-out recompute + capacity math | recompute-per-mark-tick, zero-alloc, all accounts | IM/MM, cross vs isolated; **multi-instrument netting ACTUALLY EXERCISED (BTC-PERP + ETH-PERP, acceptance #1 — single-instrument is hollow & fails)** w/ const 30% haircut; rolling-corr + SPAN → v1.5 | TradFi SPAN | **`risk-engine` (principal-defining, STANDALONE milestone W85–99 — not folded into the capstone)** |
| 10 | Liquidation engine | cascade/backpressure | cascade-detection latency, breakers | margin triggers, partial/full, ADL + insurance fund + socialized loss, keepers | Hyperliquid/dYdX/GMX | `liquidation-engine` |
| 11 | Transport / messaging | RPC deadline/backpressure | zero-copy term buffer, NAK, <10μs | MD fan-out + inter-node | Aeron | `messaging-aeron` |
| 12 | Kernel-bypass market data | data-path separation | AF_XDP/io_uring + DPDK/Onload/ef_vi awareness | exchange-facing feed handler | AF_XDP, io_uring | `marketdata-kernelbypass` |
| 13 | **Latency measurement / mech. sympathy** | back-of-envelope | tail p99.99, coordinated omission, rdtsc, perf, NUMA | why latency matters to makers | HdrHistogram, perf, rdtsc | **`latency-lab` (lands ~M6)** |
| 14 | Concurrency primitives | lock-free | CachePadded, SeqLock, MPMC, single-writer | hot-path ring (Disruptor embodiment) | crossbeam, Disruptor (ref) | `concurrent`, `epoch-gc`, `backpressure`, `time` |
| 15 | Txn / OCC / 2PL / 2PC | Spanner txns | bounded slots | settlement atomicity | Spanner/F1 | `txn` |
| 16 | EVM / on-chain execution *(spine)* | execution engine | block-stm parallelism | on-chain settlement + L2 optionality | revm, reth, Aptos block-stm | `exec-vm` (+ v1.5 block-stm) |
| 17 | State DB + trie *(spine)* | storage | — | on-chain state for settlement | reth, MDBX, alloy-trie | `storage-trie` |
| 18 | Engine API / fork-choice *(spine/opt)* | coordination | — | PoS settlement chain | reth consensus, Engine API | `consensus-engine`, `consensus-bft` |
| 19 | Vector search *(option)* | — | SIMD, quantization | real-time risk analytics / anomaly-on-flow | Qdrant | `vector-db` (v0.5) |
| 20 | Payment/stablecoin rails *(option)* | — | — | RWA-perps collateral rails | Tempo | `tempo-*` |
| 21 | **Deterministic-core discipline + per-consumer staleness + consensus-derived identity** *(cross-cutting)* | replay-safe replicated core; non-determinism at edges; **per-consumer staleness contracts (zookie/Spanner): read-after-commit control plane vs bounded-staleness+op-token analytics plane**; **identity derived from consensus (`SettlementId` = VSR op-number → free idempotency, no side table)** | zero-alloc, single PRNG seed, VOPR; integer-only money math (no float across replicas) | margin/liquidation/settlement replay bit-identically; a risk read served from the analytics contract is a correctness bug | TigerBeetle VOPR; Spanner/Zanzibar zookie | the VSR-core design across `perp-dex-core` + cluster-VOPR; the two staleness contracts (acceptance #5); `SettlementId` (acceptance #4) |
| 22 | **Columnar / vectorized analytics** *(v3.2)* | columnar scan, compression-as-architecture | SIMD batch kernels, zero-alloc inner loop, zone-map skip | trade-log / fills / funding analytics, OFF the hot path | Apache DataFusion + arrow-rs | **`query-columnar`** (bar b; reads the analytics projection) |
| 23 | **Model checking (exhaustive)** *(v3.2)* | correctness-from-unreliable-parts | — | proves consensus safety/liveness the venue depends on | Stateright (+ VSR/HotStuff TLA+ specs as source) | **`model-check`** (VSR W90 + BFT apex W129; complements VOPR) |
| 24 | **Distributed MVCC / snapshot isolation** *(v3.2)* | Spanner/Percolator timestamps | bounded slots, coprocessor pushdown | consistent position-snapshot reads for risk without blocking writers | TiKV / Percolator | **`txn` v1.1** (TSO = VSR commit point; no second clock) |
| 25 | **Perp-domain + test refinements** *(v3.2 sub-tasks)* | per-tick fan-out (margin); fault-as-common-case (storage) | O(1) order-path margin reservation; integer-money invariants | open-order IM reservation + tiered MM (risk); multi-source index + clamped basis EMA (oracle); no-crossed-book + conservation-of-value; venue differential | Hyperliquid/dYdX; TigerBeetle | `risk-engine`, `oracle-mark`, `matching-engine`, `ledger` + `sim-storage` |

**Convergence cell (the capstone's heart):** row 9 (`risk-engine`) is the only cell simultaneously `jeff-dean`-hard
(per-mark-tick portfolio recompute across all accounts), `hft-review`-hard (zero-alloc, deterministic, sub-tick
budget), and domain-hard (cross/portfolio/SPAN margin has *no spot analog*). It is the principal-defining artifact and
the spine of `perp-dex-core`.

---

## Capstone scope ladder — `perp-dex-core` v1.0 → v1.5 → v2

**Sequencing principle:** **v1.0 proves the hard production distributed engine** (founding-engineer credibility,
Bet #1 readiness); **v1.5 is the differentiator no one else builds** (RWA + multi-collateral + ADL) — it *raises the
tier* of Bet #1/#2 (the moat narrative).

| Dimension | **v1.0 — "prove the engine" (Bet #1 readiness)** | **v1.5 — "the differentiator" (tier-raiser)** | v2 |
|---|---|---|---|
| Consensus + hot-path log | **3-node VSR** — the order/event log **is** the VSR log; cluster-VOPR'd | 5-node + reconfiguration | geo-distributed |
| Instruments | **Single-collateral but MULTI-INSTRUMENT** (BTC-PERP + ETH-PERP) | + RWA instruments | + options/exotics |
| Cross-margin | **Multi-instrument netting** w/ const 30% netting-benefit haircut (≈0.7 corr, deterministic) | rolling empirical correlation + full SPAN + multi-collateral | — |
| Liquidation + fund | **Partial liquidation** + fixed-bps fee-accrued fund (default 10% taker) w/ waterfall (position-margin → fund); tripwire never hit | **ADL + socialized loss** engage exactly at tripwire; keeper market | — |
| Settlement | **`SettlementId`=VSR op-number** (instant-Final); settle-**stub** behind the async signature | optimistic-settle + challenge window into the *same* signature; wire a real chain | multi-chain |
| Projection reads | **Two contracts**: control/risk = read-after-commit; analytics = bounded-staleness + op-token (zookie) | richer consumer-group fan-out | — |
| RWA | **RWA-aware seams only** (abstract oracle iface; funding parameterized; instrument abstraction) | **RWA features** (after-hours EMA, holiday calendars, compliance) + Tempo collateral rails | — |
| Order types | mark-keyed trigger subsystem; reduce-only/post-only/IOC-FOK | trailing, advanced auction | — |

**Locked v1.0 interface/invariant shapes (so v1.5 extends without a rewrite).** *Unifying principle: the
VSR-replicated core stays deterministic and replay-safe — all non-determinism and auxiliary state is pushed to the
edges.*

- **Settlement — identity IS the VSR op-number (snapshot-epoch-relative); status derived from the commit point:**
  ```
  struct SettlementId(snapshot_epoch: u64, op_number: u64, checksum: u128)  // newtype; safe to expose externally
  settle(batch) -> SettlementId                          // returns immediately; id = the op's VSR log position
  finality_status(SettlementId) -> { Pending, Final, Reverted }
  ```
  `Final = (op_number <= VSR commit point)` — no auxiliary state, no side mapping table. `snapshot_epoch` makes the
  identity survive log compaction / snapshots and view changes. Idempotency/replay are free: resubmitting the same op
  yields the same id; `checksum` is the dedup/validation key. v1.0: `Final` immediate, `Reverted` unreachable, no
  fraud-proof/bonding/reorg. v1.5/v2 slot optimistic-settle + challenge window into the *same* signature.
- **Cross-margin — const 30% netting-benefit haircut (deterministic):** v1.0 credits only **70% of the netting
  offset** across instruments (≈ effective correlation 0.7). Conservative by construction: under-credits netting,
  never over-credits. *Rationale:* rolling empirical correlation injects non-deterministic floating state the VSR
  cluster would have to replicate bit-identically; correlation breaks toward 1 in a crash, so under-crediting is the
  safe default. **Review trigger → replace with rolling empirical correlation (v1.5) when ANY of:** (a) an instrument
  with materially different correlation structure is added (e.g. an RWA-perp); (b) VOPR shows 30% is too loose; (c) the
  v1.5 RWA milestone.
- **Insurance fund — fixed-bps fee accrual (declared model param); invariant = waterfall + tripwire:** accrual = a
  fixed fraction of taker fees, default 10%, declared as an explicit model parameter (not per-VOPR-seed). Sensitivity
  is a deterministic external sweep in the harness (5% / 10% / 20%). Shortfalls absorbed in waterfall order: remaining
  position margin → insurance fund; the tripwire = the point the fund can't cover the next shortfall, evaluated
  relative to the declared rate.
- **Projection staleness — two contracts, split by consumer (zookie/Spanner analogue), not one global setting:**
  - **Control/risk plane** (mark price into liquidation, position into margin check) = **read-after-commit**, served
    from local applied committed state on the leader shard (no consensus round-trip per read). A stale tick = a wrong
    liquidation, so no bounded staleness here.
  - **Analytics/market-data plane** (dashboards, history, external metrics) = **best-effort bounded staleness**,
    published as a metric not an SLA: target ≤ 500 ms / ≤ 1000 ops after commit; reads carry an op-number token gating
    "at least this fresh" (the zookie mechanism). No enforcement/eviction machinery in v1.0.

**v1.0 capacity target (aggressive / Hyperliquid-ish) + back-of-envelope:**

- **Target:** ~100k accounts, 2 instruments, ~1M orders/s ingest, mark ticks ~10/s. Maximizes the latency narrative
  and stresses the convergence cell (`risk-engine`) the most.
- **Back-of-envelope:** a *naïve full* portfolio recompute = 100k accounts × 10 ticks/s = 1M recomputes/s. At ~1µs/
  account that's ~1 CPU-second/second single-threaded — only fits with shard-per-core (16 shards → ~62.5k/s/shard ≈
  62ms/s/shard). Full recompute costs ~6.25 ms *burst per tick per shard* (6,250 accounts × ~1 µs) — two orders of
  magnitude over the sub-tick p99.99 budget even though average utilization is only ~6%; incremental margin is mandated
  by the tail, not the mean. **Therefore the incremental-margin hot
  path is mandatory, not a fallback:** margin updates as a delta on each position/price change; full portfolio
  recompute is reserved for circuit-breaker / parameter-change events. Local applied state (leader-co-located) handles
  read *latency*; incremental margin handles compute *cost*. Both required.
- **Consensus is for reliability, NOT throughput (the `jeff-dean`×`hft-review` convergence — make this argument explicit).**
  A single machine handles the order rate: the matching engine is **single-writer per symbol/shard** (the LMAX Disruptor
  did ~6M orders/s on *one thread*; one box clears ~1M orders/s with headroom). The venue therefore **scales by adding
  symbols/shards, never by hot-path consensus.** The 3-node VSR cluster exists for **durability + availability** (survive
  a node/rack failure with no lost commits), *not* because one machine can't keep up — "distributed is a tax, paid here
  only for reliability." This is the HyperCore insight (a deterministic single-writer SM, replicated for fault-tolerance)
  and the cleanest "why this is the company" point in a founder conversation: consensus is off the throughput critical
  path by construction. *Both lenses agree outright — Dean ("prove one machine can't before distributing") and Thompson
  (single-writer is his thesis); the architecture is already this, so the deliverable is to state the argument, not
  change the design.*

**v1.0 acceptance criteria (the lock):**
1. ≥ 2 instruments live (BTC-PERP + ETH-PERP); cross-margin demonstrably nets risk across both crediting only 70% of
   the netting offset. Single-instrument cross-margin is hollow and does **not** pass.
2. 3-node **VSR** cluster survives single-node crash/restart with no lost commits, and the order/event log **is** the
   VSR log (no second hot-path log); cluster-VOPR proves it.
3. Partial liquidation against book; fee-accrued fund (default 10% taker, declared param) absorbs the
   below-bankruptcy-price shortfall via the waterfall — exercised in simulation, not stubbed. Accrual rate swept
   deterministically (5/10/20%), never randomized inside the core.
4. Settlement: `SettlementId(op_number, checksum)` = the VSR op-number; `finality_status` derived from the commit
   point with no side mapping table; resubmit = no-op. A v1.5 spike swaps a real chain + challenge window into the same
   signature with no engine rewrite.
5. Two projection staleness contracts surfaced: control/risk = read-after-commit from local applied state on the
   leader shard; analytics = best-effort bounded-staleness (≤500 ms/≤1000 ops, metric not SLA) + op-number token. A
   risk decision served from the analytics contract **fails**.
6. RWA-aware seams present (oracle interface, parameterized funding, instrument abstraction) and demonstrated in the
   narrative before any RWA feature ships.
7. cluster-VOPR StateChecker green: linearizability + replica convergence + insurance-fund tripwire NEVER hit across
   all explored schedules at the declared accrual rate. (A schedule that hits the tripwire precisely defines the v1.5
   ADL/socialized-loss trigger — the clean v1.0 → v1.5 seam.)
8. Scale demonstrated at the aggressive target: ~100k accounts / 2 instruments / ~1M orders/s with the incremental
   margin hot path; full recompute only on circuit-breaker/param change. The latency report (`latency-lab`) shows the
   per-mark-tick p99.99 budget is met at this account count.
9. **Consensus sits behind a `ConsensusBackbone` interface — HARD criterion (NEW, non-optional).** VSR is the v1.0
   impl; `perp-dex-core` depends on the **trait**, never on `consensus-vsr` directly. Proven by: (i) `cargo tree -i
   consensus-vsr` shows only the thin adapter crate, never the engine crates; (ii) a no-op / single-node stub impl of
   the trait compiles and swaps in with zero engine changes. **Do NOT hard-wire VSR into the hot path.** This seam is
   what makes VSR → BFT a *swap, not a rewrite* — it is what makes the non-optional BFT apex architecturally payable
   (see "Terminal Apex" below).

**The three-way-fit edge is positioning from v1.0, feature from v1.5:** the RWA narrative is earned by the v1.0 design
choices (seams), then delivered in v1.5 — so the fintech edge is credible the moment Bet #1 conversations start.

---

## Terminal Apex — BFT on the hot path (committed, non-optional, bounded)

The CFT (VSR) core is the founding-engineer bar. The **terminal apex** is promoting consensus to a
**Byzantine-fault-tolerant hot-path protocol** — and this is a **trajectory commitment, not optionality.**

**"Apex" means positioning-pinnacle, NOT systems-quality-pinnacle (state this honestly — it's founder ammunition).**
On the pure-systems axis, BFT is a **deliberate tax**: Google ran planetary scale on **CFT** (Spanner, Bigtable, Borg,
Chubby — never Byzantine), because within a single trust domain CFT is sufficient and strictly faster (the apex's own
numbers prove it: BFT p99 ≤ 2 ms on 4-node LAN vs VSR p99 < 500 µs — loopback bench at W91; the 3-node-LAN figure is
measured at the W111 perf pass). BFT is justified by **one thing only — a decentralized, mutually
distrusting validator set**, which is exactly the on-chain-derivatives bet's trust model and the one place Google's
CFT-everywhere rule doesn't apply. And the tax is **quarantined off the matching hot path**: matching stays
optimistic/single-writer/local; only *settlement finality* (which tolerates ms) is BFT-committed. So in a founder
conversation the answer to "why BFT and not just fast Paxos?" is precise: *"a decentralized validator set demands it; I
pay the latency tax deliberately and isolate it off the order path."* That articulation is the deliverable here — the
architecture already quarantines it; making the trade legible is what converts the bet.

**Why non-optional (as positioning).** The on-chain-derivatives frontier is moving to purpose-built, trading-optimized
BFT (HyperBFT, Monad-class). BFT *fast enough for hot-path settlement* is becoming top-tier table-stakes; building it is
how this plan positions for the frontier / top-tier bet (the $20–100M upper band). It **will** be built before the plan
closes — as the bet's frontier credential, not as a claim that BFT is "better systems" than the VSR core.

**Guardrail 1 — it does NOT gate Bet #1.** Shipping the CFT (VSR) core = the Bet #1 readiness bar (M30, W117). The
BFT apex is sequenced *after* readiness and **may overlap early-bet work** — building a HyperBFT-class engine *inside a
company with real stakes* is the ideal scenario, strictly better than a solo portfolio piece. Non-optional on the
trajectory; **not** on the pre-bet critical path.

**Guardrail 2 — bounded MVP, defined by numbers** (like the CFT capstone, not open-ended research). "Non-optional"
attaches to **this bounded artifact**, not to "make BFT fast":

| Dimension | Bound |
|---|---|
| Protocol family | HotStuff-derived, pipelined — **locked W118: Jolteon 2-chain commit + Timeout-Certificate view-change** (TC pacemaker, leader rotation, QC chaining) |
| Node count | **fixed N = 4** (tolerates f = 1 Byzantine); reconfiguration deferred to a follow-on |
| Hot-path latency target | BFT path commits a matched batch at **p99 ≤ 2 ms** (locked W118; derivation: 2 rounds × [RTT + batched-verify] ≈ ~700 µs median + pacemaker tail), inside the venue tick budget (same discipline as the CFT p99.99 budget) |
| Byzantine VOPR scenario set | equivocation (double-propose), vote-withholding, leader-equivocation, conflicting-QC, partition+Byzantine combined, **+ the 3 added at W118**: orphaned-proposal-before-slot (O5 proof), OpNumber-contiguity-under-view-churn (O4), safety-attack/planted-fork — cluster-VOPR StateChecker green under all eight |
| Integration | swapped into `perp-dex-core` **behind the `ConsensusBackbone` interface** — VSR → BFT is a swap, **no engine rewrite** (acceptance #9 above is the enabler) |

**BFT-apex acceptance (the lock):** protocol family ✓ Jolteon 2-chain + TC; N = 4 cluster survives f = 1 Byzantine node
with no safety violation; the eight Byzantine VOPR scenarios run green; the hot-path p99 meets the 2 ms bound; and the swap into
`perp-dex-core` touches **zero** lines of matching/risk/liquidation/ledger code (only the adapter crate). Sequenced at
**W118–W143** (after readiness, overlapping bet work); see Appendix D for the exact weeks.

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
- Layer-5 products (matching-engine, ledger, oracle-mark, risk-engine, liquidation-engine, storage-trie,
  consensus-engine, vector-db): **≥70%** inheritance ratio
- Layer-5 capstone (`perp-dex-core`): **≥75%** — the integration moment where "build the primitive once" pays off
- Layer-7 Tempo crates: **≥85%** (thin extensions)

If a Layer-5 crate breaks the 70% rule at v0.5, the scope was wrong. Audit before shipping v1.0.

---

## North Star Metrics

### Reth spine (income floor + on-chain settlement design + L2 optionality)

| Metric                                     | M6 | M12 | M18 | M24 | M30 | M36 |
|--------------------------------------------|----|-----|-----|-----|-----|-----|
| Paradigm ecosystem PRs merged              | 10 | 25  | 50  | 80  | 100 | 110 |
| Reth PRs merged                            | 0  | 15  | 35  | 60  | 75  | 85  |
| Storage/Trie PRs                           | 0  | 10  | 20  | 30  | 35  | 40  |
| Execution PRs (revm + reth evm)            | 0  | 3   | 10  | 20  | 25  | 28  |
| Consensus/Engine PRs                       | 0  | 0   | 3   | 10  | 12  | 15  |
| PR reviews given (substantive)             | 0  | 10  | 40  | 100 | 150 | 200 |
| Spine production crates shipped            | 0  | 1   | 2   | 3   | 3   | 3   |
| Direct relationships with Reth maintainers | 1  | 3   | 5   | 8   | 9   | 10  |
| Conferences attended                       | 0  | 0   | 0   | 2   | 3   | 4   |

### Venue (derivatives) track — primary M19–M34

| Metric                                          | M6   | M18  | M24   | M30  | M36  |
|-------------------------------------------------|------|------|-------|------|------|
| Venue-side production crates shipped            | 1*   | 1    | 6     | 11   | 12   |
| **latency-lab version** (v3, lands M6)          | v0.1 | v0.3 | v1.0  | v1.0 | v1.0 |
| matching-engine version                         | —    | scaf | v1.5  | v1.5 | v1.5 |
| matching-engine advanced features at v1.5       | —    | —    | STP+iceberg+stop-limit+auction+MBO+FIX+CB+mark-triggers | same | same |
| **oracle-mark version** (v3)                    | —    | —    | v0.5  | v1.0 | v1.5 |
| **risk-engine version** (v3, principal)         | —    | —    | v0.5  | v1.0 | v1.5 |
| **liquidation-engine version** (v3)             | —    | —    | —     | v1.0 | v1.5 |
| **log-distributed version** (v3, Kafka off-path)| —    | —    | v0.5  | v0.5 | v0.5 |
| ledger-deterministic version                    | —    | —    | v0.5  | v1.0 | v1.0 |
| ledger VOPR scenarios run cumulatively          | —    | —    | —     | 10M  | 100M |
| **cluster-VOPR (whole venue) scenarios**        | —    | —    | —     | 1M   | 10M  |
| messaging-aeron version                         | —    | —    | v0.5  | v0.7 | v1.0 |
| marketdata-kernelbypass version                 | —    | —    | v0.5  | v0.7 | v1.0 |
| runtime-thread-per-core version                 | —    | —    | v0.5  | v1.0 | v1.0 |
| mmap-queue version                              | —    | —    | v0.1  | v0.5 | v0.5 |
| consensus-vsr version                           | —    | —    | v0.5  | v1.0 | v1.0 |
| **query-columnar version** (v3.2, DataFusion/arrow off-path) | — | — | v0.1 | v0.5 | v0.7 |
| **model-check** (v3.2, Stateright)              | —    | —    | —     | VSR  | VSR+BFT |
| **txn MVCC milestone** (v3.2, Percolator)       | —    | —    | —     | v1.1 | v1.1 |
| **`ConsensusBackbone` interface (hard accept. #9)** | — | —  | —     | shipped | shipped |
| **BFT apex (pipelined-HotStuff, N=4, Byzantine-VOPR)** | — | — | —    | —    | **v1.0 (W143, terminal apex)** |
| exec-vm block-stm variant                       | —    | —    | —     | v1.5 | v1.5 |
| **perp-dex-core version** (v3 CAPSTONE)         | —    | —    | —     | v1.0 | v1.5→v2(BFT) |
| vector-db version (option)                      | —    | —    | —     | —    | v0.5 |
| Runtime hours on chaos-tested rig†              | 0    | 0    | 200   | 2000 | 4000 |
| P99 matching latency (single-symbol, 1M orders) | —    | —    | <5μs  | <2μs | <1μs |
| **P99 per-mark-tick margin recompute @100k acct** | —  | —    | —     | <100 µs p99.99 per shard per tick (provisional; pinned by latency-lab at W98) | same |
| P99 marketdata fan-out latency (IPC)            | —    | —    | <10μs | <5μs | <2μs |
| P99 ledger **apply** latency (single-shard; durability via group commit, amortized — group-commit flush p99 < 500µs) | — | — | <10μs | <3μs | <1μs |
| P99 VSR replicated commit (3-node LAN)          | —    | —    | —     | <500µs | <500µs |
| Public blog posts shipped‡                      | 0    | 0    | 1     | 4    | 6    |

*\*M6 venue crate = `latency-lab` v0.1 (the latency leg lands first).*
*†Runtime hours = cumulative **node-hours** across cluster nodes (3-node VSR ⇒ up to ~504 node-h/week at 24/7),
chaos-drill downtime included; accrual starts when the W92 paper-trade rig powers on.*
*‡Blog count: #1 optional Phase-5 retro; #4–#7 = W109/W112/W115/W117; flagship W121; optional close W141 —
6 committed, 7 if the optional Phase-5 post ships.*
*VOPR-count derivation (nightly sweep): scenarios/s × runner-hours; ledger ~5-6 seeds/s from W88, cluster ~2/s from
W108 — feasible 24/7 alongside node-hour accrual.*

### 8-system coverage score — read at each decision gate

| Reference system  | M6   | M18  | M24  | M30  | M36  |
|-------------------|------|------|------|------|------|
| Reth              | 30   | 60   | 78   | 85   | 90   |
| Chronicle Queue   | 5    | 25   | 40   | 70   | 80   |
| ScyllaDB/Seastar  | 0    | 10   | 35   | 65   | 75   |
| Aeron             | 0    | 30   | 55   | 80   | 88   |
| Qdrant            | 0    | 0    | 0    | 0    | 60   |
| TigerBeetle       | 0    | 20   | 50   | 80   | 88   |
| **Kafka (v3)**    | 0    | 0    | 45   | 70   | 75   |
| Tempo             | 10   | 60   | 85   | 95   | 95   |
| **Total (uniform mean)**| **—** | **26** | **49** | **68** | **81** |

*Score = uniform mean of the 8 system rows (no hidden weights). Qdrant reflects the vector-db slip to the M31+
buffer; option legs are not excluded — the totals are honest, the gates are set accordingly.*

Audit at each decision gate. If under target at M24, trigger catch-up or trim option-leg time. The derivatives domain
(oracle/funding, cross-margin, liquidation) is scored *inside* the relevant reference systems (TigerBeetle for
determinism, Hyperliquid/dYdX/GMX mechanics tracked in `progress.md` domain notes).

### Network / bet-path (replaces v2's uncontrollable "recruiter inbound")

| Metric                                              | M24 | M30 | M36 |
|-----------------------------------------------------|-----|-----|-----|
| Targeted conversations w/ on-chain-derivatives teams| 20  | 35  | 50  |
| Outbound response rate                              | ≥25%| ≥25%| ≥25%|
| Binance-alumni warm intros logged                   | 3   | 5   | 7   |
| Conversations advanced to "would bring you in"      | 0   | 2   | 3   |
| Ecosystem shortlist (venues passing §checklist)     | —   | 5   | 5   |
| Crypto-MM desk conversations (the floor)            | 2   | 4   | 6   |

### Runway / lifestyle

| Metric                                     | M6   | M12  | M18  | M24    | M30    | M36        |
|--------------------------------------------|------|------|------|--------|--------|------------|
| Months of runway remaining                 | 30   | 24   | 18   | 12     | 6–12   | (bridge/bet) |
| Sleep ≥7h average                          | yes  | yes  | yes  | yes    | yes    | yes        |
| Fitness sessions/week                      | 3    | 3    | 3    | 3      | 3      | 3          |
| Public visibility (followers, talks given) | none | warm | warm | active | strong | strong     |

**Not goals**: "core maintainer of X" or "youngest contributor." Status is the OUTPUT of shipped code, reviews, and
design engagement — not a directly addressable target. The terminal state is a **founding token-equity bet**, not a job.

---

## Decision Gates (re-sequenced around the bet path)

### M6 (W24) — Latency leg live *(new in v3)*

`latency-lab` shipped; first rdtsc tick-to-trade + coordinated-omission report on the matching/ring path. This is the
artifact that underwrites the crypto-MM cash bridge. (Was implicitly M19 in v2.)

### M12 (W48 Wed) — Spine + substrate gate / calibration

- **Reth velocity**: on track for 35 Reth PRs merged by M18?
- **Inheritance discipline**: storage-trie v1.0 shipped with ≥0.70 ratio?
- **Latency leg**: `latency-lab` integrated into every bar-(c) bench?
- **Energy / sustainability**: sleep, fitness, day-job satisfaction green for the past 4 weeks?

No path change at M12 — too early. Calibrate within the spine + substrate plan.

### M24 (W96 Fri) — Derivatives-infra readiness checkpoint

`matching-engine` v1.5 + `oracle-mark` v0.5 + `risk-engine` v0.5 (cross-margin) at bar (c); 8-system coverage ≥ 45.

**Inbound criterion fixed (controllable outbound, not uncontrollable inbound):** ≥ 20 targeted conversations with
on-chain-derivatives teams, response rate ≥ 25%, ≥ 3 Binance-alumni warm intros logged. (Relabel any inbound as
"crypto-infra/derivatives OR crypto-MM OR HFT.")

Catch-up trigger if: matching-engine v1.5 not shipped OR risk-engine v0.5 cross-margin not netting OR 8-system coverage
< 40 OR < 100 runtime hours.

### M30 (W117 Tue) — Bet-readiness gate (the pivot point)

Five questions:

- **Capstone**: `perp-dex-core` multi-node **v1.0** shipped with cluster-VOPR green (all **9** acceptance criteria, incl. #9 stub-swap from W103)?
- **Runtime hours trajectory**: 2000 node-hours reached?
- **Crate quality / bar (c) audit**: do all bar-(c) crates meet the bar — VOPR ≥10M cumulative, cluster-VOPR ≥1M,
  zero-alloc audit passed on matching + ledger + aeron + risk-engine hot paths, deterministic harness operational?
- **8-system coverage**: ≥ 65?
- **Network**: ecosystem shortlist ≥ 5 venues passing the checklist; ≥ 2 founding/core-eng conversations advanced; flagship blog live?

This gate decides: place **Bet #1** now, or engage bridge income (crypto-MM) and keep iterating. If two are red,
re-evaluate.

### M36 (W144) — Bet #1 placed OR bridge engaged

Land a founding/core-eng role at a checklist-passing venue (token-equity tier 1), OR take the income-floor bridge
(crypto-MM cash) while continuing Bet #1 outbound. **No pre-committed relocation** — Dubai/SG decided at Bet #1 time
per the flip-trigger (hub follows the bet: Dubai = crypto-native/Binance-dense; SG = institutional/Asia). Capstone
**v1.5** (RWA features + multi-collateral + ADL + real-chain settle) slots here as the tier-raiser if Bet #1 timing
allows. The plan closes; the work continues.

---

## Ecosystem-selection risk checklist (reusable gate — apply before ANY bet)

A bet is **GO** only if all seven pass; otherwise it is a bridge job, not a bet.

1. **Founding-tier allocation** — equity/token grant in the founding-engineer band (not late-employee scraps).
   *Airwallex anti-pattern: late-stage, tiny employee equity → never a bet.*
2. **Tier-1 backing** — ≥ 1 of {Paradigm, a16z crypto, Dragonfly, Polychain, 1kx, Multicoin, Jump, Wintermute} on the
   cap table.
3. **Core-criticality** — I build the engine that **is** the company (matching/risk/settlement core), not a peripheral
   integration.
4. **Multi-billion-FDV ceiling** — TAM + comparables (Hyperliquid, dYdX, GMX, Variational, Ostium) support a ≥ $1B FDV
   outcome.
5. **Token reality** — pre-TGE entry; sane vesting/cliff (≈ 1y cliff / 4y vest); entry FDV leaves ≥ 10× headroom; no
   predatory unlock dumping on early contributors.
6. **Remote-native** — team is remote-distributed; Vietnam-remote works; no colo/on-site requirement.
7. **Cycle timing** — entering seed–Series B / pre-TGE with a plausible market-cycle tailwind inside the bet's vesting
   window.

---

## Shots-on-goal structure

The $20–100M outcome is a **deliberate tail bet** structured as **2–3 sequential shots**, never one all-in.

```
 Bridge₀ ──► Bet #1 ──► Bridge₁ ──► Bet #2 ──► (Bet #3) ──► capture
(crypto-  (seed–B,    (only if    (higher   (optional  (cycle +
 MM cash   founding    Bet#1       equity     top-tier)  vest)
 seat —    tier,       didn't      tier,
 latency   checklist)  capture)    upgraded
 craft)                            by #1 net-
                                   work+rep)
```

- **Bridge income funds runway between bets.** Primary: crypto-MM cash seat (remote), hireable on Half-2 latency
  craft. Secondary fallbacks (no skill stranded): crypto-infra IC on the Reth spine, or Multiplier coast.
- **Each bet upgrades the next:** reputation + network + capital from Bet *N* raise the allocation tier and ecosystem
  quality reachable at Bet *N+1*.
- **No skill stranded on a single ecosystem's death:** the transferable core (`jeff-dean` systems + `hft-review`
  latency + Reth spine) keeps me hireable → re-bridge → re-bet. AI-infra and DB-infra remain cheap fallbacks via
  `vector-db` and `lsm-core`/`storage-trie`.
- **Never quit a bridge before vest milestones** of the live bet.

**Capstone ↔ bets mapping:** `perp-dex-core` **v1.0** = "prove the hard production distributed engine" → Bet #1
readiness. **v1.5** = "the differentiator no one else builds" (RWA + multi-collateral + ADL) → *raises the tier* of
Bet #1/#2.

### Geography (Dubai/SG) — pre-stage optionality, do NOT pre-commit

- **Decision point = Bet #1 time**, hub follows the bet: **Dubai** for crypto-native / Binance-dense bets;
  **Singapore** for institutional / Asia-facing bets.
- **Now (cheap prep only):** confirm visa eligibility + pre-TGE token tax treatment for *both* hubs. No move, no lease.
- **Compress network density without relocating:** targeted event trips (Token2049 Dubai/SG, etc.) to source Bet #1
  and warm the Binance-alumni map in person.
- **Explicit flip trigger (prep → move):** convert to relocation only when ≥ N warm conversations cluster in one hub
  *or* a presence-contingent offer premium appears. Until the trigger fires, Vietnam-remote remains the default.

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

- `[Spine]` — Reth-spine bullets (primary M1–M18, maintenance M19–M36)
- `[Venue]` — derivatives-venue bullets, primary M19–M34; latency/scaffold from M6/M15
- `[Tempo]` — RWA-collateral option-leg bullets, additive on top of primary
- `[NEW]` — bullets new to v3 (latency-lab, log-distributed, the three domain crates, perp-dex-core)

If a spine (primary M1–M18) task is over time budget, drop the `[Tempo]` bullet for the day. From M19, if a venue task
is over budget, drop the spine maintenance bullet. The plan never delegates the critical-path day to a secondary
track. *Hour caps are a floor — prefer investing more time over trimming coverage.*

---

## Risk Register

### v3 additions (derivatives / bet-specific)

| Risk | Mitigation |
|---|---|
| On-chain derivatives niche cools before Bet #1 | Core is transferable (AI-infra/DB-infra); bridge on crypto-MM; checklist defers GO until a real bet exists. |
| `perp-dex-core` scope balloons (a venue is enormous) | Lock v1.0 per the scope ladder: single-collateral but multi-instrument cross-margin + partial liquidation + minimal insurance fund + 3-node VSR + real hybrid-boundary settle interface (stub behind it) + RWA-aware seams. Defer RWA features / multi-collateral / ADL / SPAN / full-FIX to v1.5. |
| Multi-instrument cross-margin model too heavy for v1.0 | v1.0 uses a const 30% netting-benefit haircut (≈0.7 corr, deterministic); rolling-empirical + full SPAN → v1.5 via a wired review trigger. Acceptance still requires demonstrable cross-instrument netting. |
| Insurance fund stubbed away (kills the moat) | Fund is fixed-bps fee-accrued (default 10% taker, declared param) and exercised in cluster-VOPR via the waterfall. Rate swept deterministically (5/10/20%), never randomized in-core. Invariant = tripwire never hit at the declared rate; a hit defines the v1.5 ADL trigger. |
| Settle-stub becomes throwaway | The stub sits behind the async signature with `SettlementId(op_number, checksum)` = the VSR op-number and `finality_status` from the commit point (no side mapping; free idempotency). v1.0 acceptance requires a v1.5 real-chain + challenge-window swap into the same signature, no engine rewrite. |
| Read-after-commit on every fan-out path throttles throughput | Two staleness contracts (zookie): only control/risk reads pay read-after-commit; analytics reads take bounded-staleness + op-number token. A risk decision served from the analytics contract is a correctness bug, caught in acceptance #5. |
| `risk-engine` per-mark-tick recompute misses budget at 100k accounts | Mandatory at this scale (not a fallback): incremental margin (delta on position/price change) is the hot path; full recompute only on circuit-breaker/param change. Reads from local applied state on the leader shard; shard-by-account. `latency-lab` proves the p99.99 budget. |
| `log-distributed` (Kafka) adds scope / would duplicate consensus | Resolved: the hot-path order/event log **is** the VSR log (one impl). `log-distributed` is off the hot path — a standalone Kafka-fidelity learning crate + the market-data fan-out reading a projection of the VSR log. No second hot-path consensus or log. |
| Token allocation illiquid / FDV craters post-TGE | Only the checklist + the 2–3-shot structure mitigates; keep bridge income through cliff; size each bet so failure ≠ ruin. |
| Best venues want SG/Dubai presence | Pre-stage, don't pre-commit: visa + token-tax prep for both hubs now; event trips to compress network; relocate only when the flip trigger fires. |
| M24 outbound response rate low | It's controllable: ship flagship blog earlier, widen Binance-alumni outreach, temporarily lower target tier for Bridge₀. |
| Cluster-VOPR can't be honest without all I/O behind interfaces | Enforce `MessageBus`/`Storage`/`Clock` trait seams from the first venue crate (matching), not retrofitted. |
| 2-machine rig under-demonstrates quorum durability | Document replica placement; rent a 3rd box for the W108 cluster-VOPR + W135 latency acceptance. |
| Latency numbers measured on loopback links don't transfer | Label every published p99 with its topology; LAN-validate at W111/W135. |
| No 1M orders/s load generator | ops-monitoring (W105) owns a loadgen at 2-4 dedicated cores; 1M/s = 1k Prepares/s × 1k ops/Prepare batching envelope. |
| Residential power/ISP outage in Vietnam threatens node-hour accrual + auto-paging | UPS + 4G failover; the node-hours metric counts cluster nodes, not wall-clock. |
| **[v3.2] `query-columnar` scope balloons** | Lock v0.5 to scan/filter/aggregate + one group-by + zone-map pushdown; defer joins/encodings to v0.7/M31. Fallback: slip v0.5 to the M31 buffer beside `vector-db` if the W110–112 window is tight. |
| **[v3.2] Stateright state-space explosion** | Model the *protocol*, not the impl; bound N (3 for VSR / 4 for BFT) + op-depth; the model is a complement to VOPR, not a replacement — keep both. |
| **[v3.2] Percolator MVCC introduces a second clock** | The timestamp oracle IS the VSR commit point / op-number — no standalone TSO service (would break "one hot-path log"). Enforced by a seam test (`cargo tree`-style proof + a determinism replay). |
| **[v3.2] `sim-storage` makes a test unwinnable** | `FaultAtlas` keeps ≥1 valid replica per block, so a green-then-red flip is a real bug, not an impossible scenario; the planted-bug negative control proves the harness bites. |

### v2 risk register (retained in full)

| Risk                                                                         | Prob | Mitigation                                                                                      |
|------------------------------------------------------------------------------|------|-------------------------------------------------------------------------------------------------|
| Rust foundations extend Phase 1                                              | 70%  | Budget +4 wk, weekly monitor                                                                    |
| Reth PR cycles slow                                                          | 80%  | Many small PRs in parallel                                                                      |
| Reth major arch change                                                       | 60%  | Telegram presence, release notes                                                                |
| Day-job demand spike                                                         | 70%  | Coast mode, 4h floor                                                                            |
| Burnout M8-14                                                                | 80%  | Rest weeks, energy monitor                                                                      |
| Conference budget delay                                                      | 30%  | Year 1 savings earmarked                                                                        |
| Family emergency                                                             | 40%  | Accept slip, adjust                                                                             |
| Motivation dip M12-14                                                        | 70%  | Pre-commit to Phase 4                                                                           |
| Crypto winter                                                                | 40%  | Storage/exec/venue/latency portable                                                             |
| Crate scope creep                                                            | 60%  | Lock scope at phase start                                                                       |
| Tempo closes-sources or becomes Paradigm-internal                            | 25%  | Reth contributions remain floor; Tempo time falls back to upstream revm/reth                    |
| HFT track scope balloons (esp. options/exotic features)                      | 60%  | Strict scope lock at W58; options/exotics dropped if behind schedule                            |
| matching-engine doesn't reach v1.0 by W74                                    | 40%  | Drop perp polish first, keep spot matching + replication                                        |
| matching-engine v1.5 advanced features (STP/auction/FIX/CB) slip past W82    | 50%  | Drop in order: FIX → circuit breakers → auction → MBO. Keep STP + iceberg + stop-limit minimum  |
| ledger VOPR simulator doesn't catch real bugs by M30                         | 40%  | Add canary fault injection (known bugs) weekly; if simulator misses them, harden assertions    |
| consensus-vsr v1.0 slips past W90                                            | 50%  | Drop to v0.7 (reconfig later); ledger temporarily uses consensus-raft as fallback (off-path)    |
| runtime-thread-per-core v0.5 not ready by W57 (blocks matching-engine)       | 40%  | v0.3 single-shard fallback for matching-engine W58-W63, swap to v0.5 at W64                     |
| mmap-queue v0.5 conflict with wal API surface                                | 30%  | Audit at W31 design phase: distinct types, no shared trait between wal and mmap-queue           |
| exec-vm block-stm parallel variant memory blowup vs sequential               | 50%  | Time-box at 4 weeks (W91-W94); if blowup > 4×, ship sequential only and write postmortem        |
| Aeron multicast IGMP issues in cloud environment                             | 70%  | Test on bare-metal rig (W82-W84); cloud retrospective uses unicast fanout                       |
| FIX 4.4 session layer scope expands toward FIX 5.0 / FAST                    | 50%  | Lock at FIX 4.4 only                                                                            |
| 40h/wk M19-M34 not sustainable past M27                                      | 40%  | Drop spine maintenance bullet first; if still insufficient, slip ledger v1.0 to M31            |
| Operations rig hardware failure                                             | 40%  | Two-machine cluster; spare drive; off-site git mirror                                           |
| Founding-eng / interview cycle saturates Phase 7                            | 70%  | Start outbound early (W118); space conversations; drop weakest opportunities                    |
| Relocation paperwork slips (if flip trigger fires)                          | 50%  | Visa research is pre-staged; allow 12 weeks                                                      |

---

## Principles

1. Deliverables over hours. 5h target, 4h floor. Done at 3h → rest. Stuck at 6h → diagnose. *Coverage is
   non-negotiable: invest more time rather than trim scope.*
2. The deliverables are the spine crates (storage-trie, exec-vm, consensus-engine), the venue stack (matching, oracle,
   risk, liquidation, ledger, perp-dex-core) and the substrate. Everything else is means.
3. Depth over breadth. 3 spine subsystems + the venue mastered > 12 shallow.
4. Code reading > code writing in Phase 3+.
5. Ship imperfect > perfect never. v0.5 → v0.7 → v1.0 cadence enforced.
6. AI leverage for architecture research; AI cannot substitute for the reading hours.
7. Blogging optional through M18; expected M19+ (one post per major capstone; flagship at M31).
8. Spine trajectory deferred at M12, not forgotten. Reassessed at M24 and M30.
9. Conferences non-negotiable Year 2 (EthCC, Devcon) and Year 3 (one distributed-systems/HFT-adjacent + one
   crypto-derivatives event for bet sourcing).
10. Day-job is infrastructure. Coast mode. Sleep 7h, fitness 3x/week minimum.
11. Energy is the only real budget. Track weekly.
12. M6 = latency leg. M12 = calibrate. M24 = derivatives-infra readiness. M30 = bet-readiness pivot. M36 = bet placed
    or bridge engaged.
13. Scope discipline on crates. No feature creep. Lock v1.0 per the capstone scope ladder.
14. **The job is the income floor, not the goal.** Crypto-MM cash is the bridge; the founding token-equity bet is the
    terminal state.
15. **A bet is GO only if all seven checklist gates pass.** Otherwise it is a bridge job.
16. **The deliverable is shipped code on systems that process real production-shape workloads.** Status is downstream
    of shipped code.
17. **Both halves retained in full.** The spine is not trimmed for the domain; both are first-class.
18. **No primitive twice.** The most-violated principle in ambitious self-directed plans; the workspace layout enforces
    it.
19. **Bar policy.** Bar (c) — VOPR-grade — for the HFT critical path + Layer-1 substrate + domain crates. Bar (b) —
    fuzz + property + bench + perf-CI — for the Reth spine + database/vector + Tempo. Uniform bar (c) is dishonest
    mirroring.
20. **Deterministic-core discipline.** The VSR-replicated core replays bit-identically; non-determinism + auxiliary
    state live at the edges; identity is derived from consensus; staleness contracts split per-consumer.
21. **8-system mastery is the anchor.** Every additive crate maps to a named core technique of Reth, Chronicle,
    Scylla/Seastar, Aeron, Qdrant, TigerBeetle, Kafka, or Tempo, AND underwrites a bet-path artifact or interview
    question.
22. **Disruptor is OUT as a crate, IN as a mirror.** Ring-buffer work lives in `concurrent` and `mmap-queue`.

---

## v3 Crate Slotting Schedule (additive on top of the v2 schedule)

The v2 crate-slotting schedule (`runtime-thread-per-core`, `mmap-queue`, `consensus-vsr`, matching v1.5, ledger
v0.7/v1.0, aeron v0.7, exec-vm block-stm, lsm-core LCS+TWCS) carries forward unchanged. v3 **adds** the following.
Per the no-time-constraint call, these are **additive** — funded by extra invested time, not by trimming the spine.

### New crate: `latency-lab` (HdrHistogram / perf / rdtsc mirror) — pulled to M6
- **Slot**: v0.1 W21 (M6) → v0.3 W43-W44 → v0.5 W57-W60 → v1.0 W83-W85, woven into every bar-(c) bench thereafter.
- **Mirror**: HdrHistogram, `perf`, rdtsc. Tail latency, coordinated omission, tick-to-trade, LLC/false-sharing/NUMA.
- **Justification**: underwrites the crypto-MM cash bridge (the latency report is the hireable artifact) and proves the
  p99.99 budget for `risk-engine` and `perp-dex-core`. **Risk if skipped**: no honest latency narrative; the bridge
  job's core proof is missing.

### New crate: `log-distributed` (Kafka mirror, off hot path)
- **Slot**: v0.1 W63-W66 (rides `wal` + `consensus-raft`) → v0.5 W80-W84 (first VSR-log projection / read-side fan-out).
- **Mirror**: Kafka. Partitioned log, ISR, consumer groups, exactly-once, log compaction + two-contract read-side
  fan-out (read-after-commit / bounded-staleness + op-token).
- **Justification**: closes the jeff-dean distributed-log gap and the Kafka 8th-system coverage without a second
  hot-path log. **Risk if skipped**: 0/100 Kafka coverage; no read-side fan-out story for the venue.

### New domain crate: `oracle-mark` (Hyperliquid/dYdX mechanics)
- **Slot**: v0.5 W77-W80 (RWA-aware seams) → v1.0 W97+ → v1.5 in M31+ (RWA features).
- **Justification**: mark vs index vs last + funding is the heartbeat of a perp venue; the abstract oracle interface is
  the RWA seam. **Risk if skipped**: the venue has no settlement loop and no RWA narrative.

### New domain crate: `risk-engine` (TradFi SPAN / portfolio-margin) — **principal-defining**
- **Slot**: v0.5 W85-W90 (multi-instrument cross-margin, fixed haircut) → v1.0 W97-W100 (BTC+ETH netting, per-tick).
- **Justification**: the convergence cell — jeff-dean-hard ∧ hft-review-hard ∧ domain-hard. The single artifact that
  most distinguishes a founding derivatives engineer. **Risk if skipped**: no principal-defining artifact; the
  portfolio is "another matching engine."

### New domain crate: `liquidation-engine` (Hyperliquid/dYdX/GMX)
- **Slot**: v1.0 W100-W102 (partial-liq + minimal insurance fund + waterfall + tripwire).
- **Justification**: the backstop is the moat; insurance-fund waterfall + tripwire is the v1.0→v1.5 ADL seam.
  **Risk if skipped**: the venue can't survive a below-bankruptcy-price fill; the moat is stubbed.

### New capstone: `perp-dex-core` (Hyperliquid / dYdX v4 architecture)
- **Slot**: assembly W103-W113 (3-node VSR + real hybrid-boundary settle interface + RWA-aware seams + cluster-VOPR) →
  2000h runtime + flagship blog + ecosystem shortlist W114-W117 → v1.5 in M31+.
- **Displaces**: v2's `mini-db` W97-W100 + `vector-db` W101-W104 capstone slots. The DB substrate is consumed inside
  the venue store; `mini-db` v0.5 KV/query facade stays deferred (default DO NOT BUILD). `vector-db` v0.5 slips to an
  optional W-slot / M31 buffer.
- **Justification**: the founding-engineer proof — a replicated, fault-tolerant, multi-node hybrid CLOB. **Risk if
  skipped**: no bar-(b) portfolio artifact; nothing clears the founding-eng bar.

### NEW seam: `ConsensusBackbone` interface (the VSR→BFT swap point) — hard v1.0 acceptance criterion #9
- **Slot**: trait **declared W91** (alongside consensus-engine v1.0 + consensus-vsr v1.0 from W90); VSR wired behind it
  at the **W103** capstone-assembly start; the no-op stub-swap test lands **at W103 (dependency formation, Flag-A)** and is re-verified end-to-end at the **W117** capstone v1.0 acceptance gate.
- **Justification**: a thin `ConsensusBackbone` trait (propose/commit/view + the order/event-log read API) that
  `perp-dex-core` depends on **instead of** `consensus-vsr` directly. This is the single architectural decision that
  makes the non-optional BFT apex *payable* — VSR → BFT becomes an adapter swap, not an engine rewrite. **Risk if
  skipped**: the BFT apex would require rewriting the matching/risk/liquidation/ledger wiring; the commitment becomes
  unaffordable.

### NEW terminal apex: BFT-on-the-hot-path (bounded MVP) — committed, non-optional, sequenced AFTER readiness
- **Slot**: design + seed **W118-W120** (HotStuff-derived/pipelined protocol design behind the `ConsensusBackbone`
  interface) → core protocol build **W122-W128** (leader/QC-chaining/pacemaker; interleaved with bet outreach) →
  Byzantine VOPR **W129-W134** (all 8 scenarios: equivocation, vote-withholding, leader-equivocation, conflicting-QC,
  partition+Byzantine, orphaned-proposal-before-slot, OpNumber-contiguity-under-view-churn, safety-attack/planted-fork)
  → hot-path latency target + **swap into `perp-dex-core` via the interface** (`perp-dex-core` v2 = BFT-replicated)
  **W135-W142** → **BFT-apex acceptance W143**. Builds on `consensus-bft` (W64-73 v0.5) promoted to hot-path grade.
- **Guardrails**: (1) does NOT gate Bet #1 — the CFT/VSR core at M30 (W117) is the readiness bar; BFT is sequenced
  after and may be built *inside* the bet. (2) Bounded MVP, defined by numbers (protocol family / N=4 / p99≤2ms /
  Byzantine scenario set) — "non-optional" attaches to this artifact, not to open-ended "make BFT fast." See the
  **Terminal Apex** section above for the full acceptance lock.
- **Justification**: positions for the trading-optimized-BFT frontier (HyperBFT / Monad-class) and the $20-100M
  upper-band bet. **Risk if skipped**: stranded at the CFT tier while the frontier moves to fast BFT.

---

## v3.2 Crate Slotting (additive — HyperCore + TiKV + DataFusion durable-infra; see `.rework/HYPERCORE_ADDITIONS.md`)

These are the "stand on it for 10–15 years" durable-infra additions from the 2026-06 coverage audit. Additive on
top of the v2 + v3 schedules; funded by invested time, not by trimming the spine or venue.

### NEW crate: `query-columnar` (DataFusion + arrow-rs mirror) — columnar/vectorized analytics
- **Slot**: v0.1 W84 (rides `log-distributed` v0.5 first projection) → v0.5 W110–W112 (sustained-ops window) →
  v0.7 M31 buffer (beside `vector-db`). Fallback: slip v0.5 to M31 if W110–112 is tight.
- **Mirror**: Apache DataFusion (vectorized execution) + arrow-rs (columnar layout). In-process Rust query API
  only — NO SQL/RPC surface. Reads the OFF-PATH analytics projection of the VSR log (never the hot path).
- **Justification**: the OLAP/data-infra career surface (DataFusion/DuckDB/ClickHouse/Polars) — the biggest
  transferable second-specialty beside the latency/consensus core. **Risk if skipped**: no columnar/vectorized
  story; the analytics plane stays a row-at-a-time toy.

### NEW crate: `model-check` (Stateright mirror) — exhaustive protocol verification
- **Slot**: VSR safety model W90 (when `consensus-vsr` v1.0 ships) → BFT-apex safety/liveness model W129
  (alongside the Byzantine VOPR). Minimal scope (consensus only); matching/ledger linearizability noted optional.
- **Mirror**: Stateright (Rust, in-workspace — buildable, not a separate toolchain). Published TLA+ specs of VSR
  and HotStuff read as the model SOURCE (for invariants), not built.
- **Justification**: lightweight formal methods — the correctness-engineering surface that marks top-tier systems
  builders (TigerBeetle/FoundationDB/AWS-via-TLA+). *Exhausts* a small space; VOPR *samples* a huge one — they
  catch different bugs. **Risk if skipped**: the consensus safety claim rests on sampling alone.

### NEW milestone: `txn` v1.1 — Percolator MVCC + coprocessor pushdown (TiKV)
- **Slot**: seed W90 (after VSR v1.0 gives a commit point) → build W93–W94 (P5 close, beside exec-vm block-stm)
  → exercised W97+ (the venue position store; `risk-engine` reads positions at a snapshot).
- **Mirror**: TiKV / Percolator. Built as a MODE of the existing `txn` crate — no duplicate primitive. Timestamp
  oracle = the VSR commit point (no standalone TSO).
- **Justification**: the distributed-database career surface (TiKV/CockroachDB/Spanner-likes): snapshot isolation,
  MVCC GC, lock/write/data CFs, predicate pushdown. **Risk if skipped**: the venue store has no consistent
  non-blocking snapshot reads; the TiKV technique cluster is uncovered.

### NEW harness module: `sim-storage` — storage-fault injection (TigerBeetle)
- **Slot**: W88 (fold into `ledger-deterministic` VOPR v0.7) → W108 (cluster-VOPR across the 3-node placement)
  → W129 (compose with Byzantine faults). A module of the deterministic-sim harness behind the `Storage` seam,
  reused by every VOPR leg — not a standalone crate (no primitive twice).
- **Mirror**: TigerBeetle VOPR storage faults (read/write-fault probability, torn writes, bit-corruption,
  misdirected I/O, `ClusterFaultAtlas`).
- **Justification**: the storage-reliability surface ("assume the disk lies") that hardens every durability claim.
  **Risk if skipped**: recovery/VOPR only exercise process/network faults; disk-level faults go untested.

> **Reconciliation note (vs the current v3 file):** `oracle-mark` / `risk-engine` / `liquidation-engine` /
> `perp-dex-core` / VSR-backbone were **already** standalone in v3 — this resolution *hardens* them (risk + liquidation
> are now explicitly "do-not-fold" standalone milestones; the consensus interface is now a *hard* acceptance criterion)
> and **adds** the two genuinely new items: the `ConsensusBackbone` seam (W91) and the BFT terminal apex (W118-W143).
> `vector-db` survives as an optionality leg (v0.5) in the M31 buffer; W60 Tempo PR and W90 marketdata are unchanged.

---

## Daily Log Template

Fill one row per work-day in `progress.md`. Single source of truth for retrospective queries.

```
| Date       | Hrs | Phase | Track  | Crate(s) touched       | Output                                             | Energy 1-5 |
|------------|-----|-------|--------|------------------------|----------------------------------------------------|------------|
| 2026-04-27 | 5.0 | P1/W1 | Spine  | eth-primitives         | FixedBytes built; W1.Tue tasks checked off         | 4          |
| 2026-04-28 | 4.5 | P1/W1 | Spine  | eth-primitives, notes  | Bytes + BytesView built; notes/03 written          | 5          |
```

Notes:
- Date is ISO-8601 (yyyy-mm-dd).
- Track: Spine, Venue, Tempo, Ops, Personal. Multi-track days get a comma list.
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
- PRs merged (Reth spine: N, Tempo: M, others: K)
- Blog posts (if any)

### Inheritance check
- This week's audits, if any. Pass/fail with ratio.

### Energy + sustainability
- Sleep average this week: Nh
- Fitness sessions: K
- Day-job satisfaction (1-5): X
- Burnout warnings: yes/no

### Bet-path (M19+)
- Targeted conversations this week: N. Response rate. Warm intros logged.

### Surprises
- 2-4 things that didn't go as planned. What I learned.

### Next week
- Top 3 priorities. Top risk.

### Open questions carried forward
- New questions added. Questions answered, removed. Questions surviving ≥2 weeks → dedicated slot next week.
```

---

## §10 — Open Questions / Assumptions (post scope+sequencing resolution)

**Resolved in this revision:**

- ✅ **Scope boundary (a/b/c).** **(c)** vault/spot/staking/bridge product surface = OUT (do not schedule; negative-ROI
  breadth). **(b)** fully-on-chain + EVM bridge = CONDITIONAL v1.5/v2 (wires into the locked `SettlementId` async
  signature; schedule only if a bet demands on-chain settlement). **(a)** BFT-on-hot-path = COMMITTED TERMINAL APEX.
- ✅ **BFT apex = non-optional but correctly bound.** Built before plan close; **Guardrail 1**: does not gate Bet #1
  (CFT/VSR core at M30 is the readiness bar; apex sequenced W118-W143, may overlap/be built inside the bet).
  **Guardrail 2**: bounded MVP by numbers (pipelined-HotStuff / N=4 / p99≤2ms / 8-scenario Byzantine VOPR / zero-rewrite
  swap), not "make BFT fast."
- ✅ **X resolved at W118: p99 ≤ 2 ms** (set relative to the VSR commit budget).
- ✅ **N locked at W118: N=4 (f=1); reconfiguration deferred.**
- ✅ **Consensus-backbone interface is a HARD v1.0 acceptance criterion (#9).** VSR is the v1.0 impl behind the trait;
  `perp-dex-core` never depends on `consensus-vsr` directly. This is what makes the BFT commitment architecturally
  payable (swap, not rewrite).
- ✅ **Margin/risk engine and liquidation engine are STANDALONE milestones, do-not-fold.** risk-engine W85-W90 (v0.5) →
  W97-W99 (v1.0); liquidation-engine W100-W102 (v1.0), **before** the capstone. Multi-instrument netting is *exercised*
  (BTC-PERP + ETH-PERP, acceptance #1), not stubbed.

**NEW open questions surfaced by this resolution (to decide in the relevant week's daily file, not strategy):**

1. **Dependency ordering of the standalone milestones vs the matching engine.** risk-engine (W85+) needs `oracle-mark`
   (mark, W80) and `ledger-deterministic` (positions, W83); liquidation-engine (W100+) needs *both* risk-engine
   (`MarginVerdict`) **and** matching-engine (to liquidate against the book, W74/W82). The edge direction is fixed
   (positions+marks → risk → liquidation → capstone; matching is parallel), but the **exact interface contracts** at
   each seam (does liquidation call matching synchronously or post intents to the VSR log?) are unresolved — decide at
   the W100 design phase. *Risk:* a wrong seam here forces a capstone-assembly rework at W103.
2. **BFT apex vs live-bet time-slicing.** The apex (W118-W143) overlaps Bet #1 sourcing/onboarding. If Bet #1 lands
   early (W126-ish) at a venue **not** building BFT, does the apex continue as a solo artifact, or pivot to the bet's
   consensus? Open: a decision rule for "build the apex inside the bet" vs "build it solo in parallel." *Assumption:*
   if the bet *is* building fast BFT, the apex folds into bet work (ideal); otherwise it stays a solo terminal artifact
   on reduced hours. Decide at the W131 bet-decision gate.
3. **Conditional (b) trigger wording.** "A bet demands on-chain settlement" needs a crisp test (does the venue settle
   on its own L1? require a fraud-proof window?) so the conditional extension isn't accidentally pulled in. Draft the
   test at the W116 ecosystem-filter week.
4. **[v3.2] `query-columnar` ↔ `log-distributed` projection seam.** Push (projection drives columnar append) vs
   pull (columnar polls the projection)? Decide at the W84 v0.1 design phase. *Risk:* a pull model re-introduces
   staleness coupling the two-contract design was meant to remove.
5. **[v3.2] `model-check` depth bounds.** Exact N + op-depth that keeps the state space tractable while still
   exhausting the safety-critical interleavings — tune at W90 (VSR) and W129 (BFT). Default minimal; matching/ledger
   linearizability models are optional stretch.
6. **[v3.2] `query-columnar` v0.7 timing.** Ships in M31 buffer or slips with `vector-db`? Decide at the W116/M30
   capacity check — the W110–112 v0.5 is the committed bar; v0.7 is optional.

**Open strategic questions: none.** Every scope call (a/b/c), the BFT commitment + its two guardrails, and the
consensus-interface acceptance criterion are locked. Remaining items above are *implementation-detail* questions for
the relevant week's daily file.

---

## Daily Plan

The full day-by-day plan (Mon–Sat checklists for every week from W1 to W144) lives in
[`plan/INDEX.md`](plan/INDEX.md), with one markdown file per week (`plan/W001.md` through
`plan/W144.md`).

This README holds the **strategic plan only**: frame, readiness bars, decisions, workspace layout, coverage matrix,
capstone scope ladder, North Star metrics, decision gates, ecosystem checklist, shots-on-goal, risk register,
principles, crate slotting, inheritance map, dependency graph, and appendices.

Open `plan/INDEX.md` to navigate by phase/month/week, or jump directly to the week file you need (e.g. `plan/W021.md`
for the `latency-lab` seed week, `plan/W105.md` for the `perp-dex-core` assembly start).

> **Note**: the daily `WNNN.md` files are being regenerated from this v3 frame (the deferred big pass). Where a daily
> file still reflects v2 framing (HFT-job destination, `mini-db` capstone at W97-W100), this README is authoritative;
> the week file will be re-slotted to the v3 schedule above.

---

# FINAL SYNTHESIS

## Inheritance Map (ASCII)

This is the data structure the entire plan exists to produce. Read top-down: each product is mostly the primitives
below it, wired together with thin glue.

```
                     LAYER 7 — Tempo (RWA collateral option leg)
                tempo-tx-envelope   tempo-evm-ext   tempo-payment-lane
                        │ inherits eth-rlp/primitives/consensus, exec-vm, consensus-engine, matching-engine

                     LAYER 5 — products + the CAPSTONE
                                  ┌──────────────────────────────┐
                                  │      perp-dex-core (W113)     │  ◄── CAPSTONE (replaces mini-db)
                                  │  multi-node, replicated,      │
                                  │  cluster-VOPR'd hybrid CLOB   │
                                  └───────────────┬──────────────┘
       ┌──────────────┬───────────────┬──────────┴────┬───────────────┬───────────────┐
  storage-trie   matching-engine  oracle-mark    risk-engine     liquidation-    ledger-
  (W44)          (W74/W82)        (W80)          (W100, principal) engine (W104)  deterministic (W92)
  consensus-engine (W91)                                                          vector-db (option)
       │              │               │               │               │               │
       ↓ inherits ↓ inherits ↓ inherits ↓ inherits ↓ inherits ↓ inherits
       bufpool, wal, recovery, txn, bloom, lsm-core, eth-trie, eth-storage-cache,
       time, backpressure, runtime-thread-per-core, mmap-queue, latency-lab,
       consensus-vsr (hot-path log+SM), messaging-aeron, marketdata-kernelbypass,
       log-distributed (read-side fan-out), consensus-bft (spine fork-choice)

                     LAYER 4 — distribution + transport
       p2p   consensus-raft(off-path)   consensus-bft(spine)   consensus-vsr(HOT PATH)
             messaging-aeron   marketdata-kernelbypass   log-distributed [Kafka, off-path]

                     LAYER 3 — concurrency + transactions
       bloom        lsm-core (STCS+LCS+TWCS)        txn (2PL/OCC/2PC)

                     LAYER 2 — durability + queues
       wal          recovery (ARIES)        mmap-queue (Chronicle)

                     LAYER 1 — universal primitives + runtime + latency
       time   backpressure   bufpool   concurrent   epoch-gc
       runtime-thread-per-core (Seastar)   latency-lab (HdrHistogram/perf/rdtsc)

                     LAYER 0 — bootstrap mirror crates
       eth-primitives  eth-storage-cache  eth-network-codec  eth-rlp  eth-consensus
       eth-eips  eth-rpc-types  eth-stage  exec-vm (+block-stm v1.5)  eth-trie  eth-primitives-derive
```

**Inheritance Ratios (target ≥0.70 Layer-5, ≥0.75 capstone, ≥0.85 Layer-7)**:

| Crate                            | Layer | Inherits from                                                                              | Target | Audit Week        |
|----------------------------------|-------|--------------------------------------------------------------------------------------------|--------|-------------------|
| storage-trie v1.0                | 5     | bufpool, wal, recovery, txn, bloom, eth-trie, eth-storage-cache                            | ≥0.70  | W44 Mon           |
| matching-engine v1.0            | 5     | time, backpressure, wal, recovery, runtime-thread-per-core, messaging-aeron, consensus-vsr  | ≥0.70  | W74 Thu           |
| matching-engine v1.5            | 5     | + mmap-queue (MBO), + messaging-aeron v0.7, + latency-lab                                   | ≥0.70  | W82 Fri           |
| oracle-mark v0.5                 | 5     | time, log-distributed                                                                       | ≥0.40 (domain-math-dominated; rationale: funding/mark is mostly net-new domain logic over {time, log-distributed}) | W80 Fri           |
| risk-engine v1.0 (principal)     | 5     | oracle-mark, ledger-deterministic, runtime-thread-per-core, concurrent, latency-lab         | ≥0.70  | W99 Wed (before the Friday v1.0 tag) |
| liquidation-engine v1.0          | 5     | risk-engine, matching-engine, ledger-deterministic                                          | ≥0.70  | W102 Tue (before the tag) |
| ledger-deterministic v1.0        | 5     | time, wal, recovery, txn, runtime-thread-per-core, consensus-vsr, mmap-queue, latency-lab   | ≥0.70  | W92 Wed           |
| consensus-engine v1.0            | 5     | eth-consensus, eth-stage, exec-vm, storage-trie, consensus-bft                              | ≥0.70  | W91 Wed           |
| **perp-dex-core v1.0 (CAPSTONE)**| 5     | matching, oracle-mark, risk-engine, liquidation-engine, ledger, consensus-vsr, log-distributed, runtime-tpc, lsm-core, messaging-aeron, marketdata, latency-lab | **≥0.75** | W117 (acceptance gate, Flag-A) |
| exec-vm v1.5 block-stm           | 0/5   | exec-vm v1.0 + concurrent (versioned memory)                                                | ≥0.70  | W94 Fri           |
| vector-db v0.5 (option)          | 5     | bufpool, bloom, mini-db substrate, wal, time                                                | ≥0.70  | W104 Tue / M31    |
| tempo-tx-envelope v0.1.0         | 7     | eth-rlp, eth-primitives, eth-consensus, time                                                | ≥0.85  | W66 Fri           |
| tempo-evm-ext v0.1.0             | 7     | exec-vm, eth-primitives                                                                     | ≥0.85  | W91 Thu           |
| tempo-payment-lane v0.1.0        | 7     | consensus-engine, matching-engine (priority idea)                                           | ≥0.85  | W91 Thu           |
| runtime-thread-per-core v1.0     | 1     | concurrent, time, backpressure, epoch-gc                                                    | net-new| W85 Wed           |
| latency-lab v1.0                 | 1     | time, concurrent                                                                            | net-new| W85 Wed           |
| mmap-queue v0.5                  | 2     | bufpool, time, eth-storage-cache::Page                                                      | ≥0.50  | W79 Fri           |
| log-distributed v0.5             | 2/4   | wal, mmap-queue, consensus-raft, runtime-thread-per-core, backpressure                      | ≥0.50  | W84 Fri           |
| consensus-vsr v1.0               | 4     | time, wal, p2p, runtime-thread-per-core                                                     | ≥0.50  | W90 Fri           |
| **query-columnar v0.5** (v3.2)   | 5     | log-distributed, lsm-core, bufpool, time, latency-lab                                       | ≥0.50  | W112 (before tag) |
| **model-check** (v3.2)           | 4-adj | net-new (re-encodes the protocol state machine; no crate dep)                               | net-new| W90 / W129        |
| **txn v1.1 Percolator** (v3.2)   | 3     | txn v1.0 + consensus-vsr (commit point) + lsm-core                                          | ≥0.50  | W94 Fri           |

If any crate falls below its target ratio at audit, scope is wrong — audit before tagging.

---

## Workspace Dependency Graph (Final, ASCII)

```
                                LAYER 7 (Tempo, RWA collateral option leg)
                                  tempo-tx-envelope / tempo-evm-ext / tempo-payment-lane
                                         │ depends on Layer 5 + Layer 0
                                         ↓
                                LAYER 6 (Ops; deployment-only)
                                  ops-monitoring / ops-deploy / ops-chaos / ops-runbooks
                                         │ wraps Layer 5
                                         ↓
                                LAYER 5 (Products + CAPSTONE)
                                  ┌──────────────── perp-dex-core ────────────────┐
                                  │  (matching + oracle + risk + liquidation +     │
                                  │   ledger + VSR + log-distributed + tpc store)  │
                                  └────────────────────┬───────────────────────────┘
   storage-trie  consensus-engine  matching-engine  oracle-mark  risk-engine  liquidation-engine
   ledger-deterministic  vector-db(option)
                                         ↓
                                LAYER 4 (Distribution + transport)
   p2p   consensus-raft(off-path)   consensus-bft(spine)   consensus-vsr(HOT PATH)
   messaging-aeron(v0.7)   marketdata-kernelbypass   log-distributed[Kafka,off-path]
                                         ↓
                                LAYER 3 (Concurrency + txn)
   bloom        lsm-core (LCS+TWCS+STCS)        txn
                                         ↓
                                LAYER 2 (Durability + queues)
   wal          recovery        mmap-queue
                                         ↓
                                LAYER 1 (Universal primitives + runtime + latency)
   time   backpressure   bufpool   concurrent   epoch-gc
   runtime-thread-per-core (Seastar)   latency-lab (HdrHistogram/perf/rdtsc)
                                         ↓
                                LAYER 0 (Bootstrap mirror crates)
   eth-primitives  eth-rlp  eth-storage-cache  eth-network-codec  eth-consensus  eth-eips
   eth-rpc-types  eth-stage  exec-vm  eth-trie  eth-primitives-derive
```

Read bottom-up: Layer 0 is built in Phase 1-2. Each layer above is fully built before the layer above it consumes it.

> **[v3.2] additions slot into this graph without restructuring it:** `query-columnar` (L5, off-path analytics) hangs
> off `log-distributed`'s projection + `lsm-core` and feeds `perp-dex-core`'s analytics plane; `model-check` (L4-adjacent
> verification) takes no runtime dependency — it re-encodes the `consensus-vsr` / BFT-apex protocols for exhaustive
> checking; `txn` v1.1 (Percolator MVCC) is a mode of the existing L3 `txn`, with its timestamp oracle = the
> `consensus-vsr` commit point (no new ordering authority); `sim-storage` is a fault module inside the deterministic-sim
> harness behind the existing `Storage` seam. None sit on the matching/settlement hot path.

---

## AI Calibration Review (Quarterly)

Every M3, M6, M9, ... reserve 60 minutes on a Sunday for AI-tool calibration:

- What's the new state of code-generation tools? (claude.ai / cursor / aider / continue.dev)
- Which tasks am I doing manually that AI could do better now?
- Which tasks did I delegate to AI that produced bad output? Why? Stop delegating those.
- Which workspace crates are good prompts? Use the crate as a template for "build me a similar primitive in pattern X."

Treat AI like any other tool: regularly audit fit-for-purpose. AI cannot do the reading hours for you; AI can
accelerate the writing hours.

---

## APPENDIX A: Venue-Track Maintenance Schedule (M25-M36 weekly cadence)

The venue crates produced in Phase 4-6 need sustained operational time to demonstrate production-readiness.

### Weekly venue maintenance template (W97-W144)

- **[Venue] Monday**: 15 min — review weekend runtime alerts. P99 spikes? Replica divergence? Disk-fill warnings?
- **[Venue] Wednesday**: 30 min — check ops dashboards. Capture runtime hours total. Update `progress.md` venue row.
- **[Venue] Saturday**: 60 min — chaos drill (rotated: process kill, network partition, clock skew, disk fill, slow
  subscriber, **liquidation-cascade injection**). Capture in `chaos_log.md`.
- Hours accrual starts W92 (paper-trade rig live).

### Rig spec + topology (the substrate of every published number)

- **Machine count**: 2 boxes, plus a **rented 3rd** for the W108 cluster-VOPR and the W135 latency weeks.
- **Required class**: ~32 cores/node — sized for the 16-shard margin math + matching + VSR + aeron + loadgen
  co-residency; NVMe storage; XDP-capable NIC (for AF_XDP / `marketdata-kernelbypass`).
- **Node placement**: document which VSR/BFT replicas fate-share a box, and which inter-replica links are loopback vs
  a real NIC — quorum-durability claims are only as honest as the placement map.
- **Rule**: every published p99 is labeled with its topology (loopback / LAN / mixed).

### Monthly venue polish checkpoints (M25-M36)

- **[Venue] M25 end (W100)**: matching-engine v1.5 stable on rig; `risk-engine` v1.0 (BTC+ETH netting) shipped.
- **[Venue] M26 end (W104)**: `liquidation-engine` v1.0 shipped (W102); ops rig provisioned.
- **[Venue] M27 end (W109)**: ops-monitoring + ops-deploy + ops-chaos shipped; first chaos cycle; blog #4 live.
- **[Venue] M28 end (W113)**: **`perp-dex-core` v1.0 assembled**; cluster-VOPR green; 1200+ node-hours.
- **[Venue] M29 end (W117)**: 1300+ node-hours; ledger v1.0 + aeron v0.7 real-load polish.
- **[Venue] M30 end (W117)**: 2000+ node-hours (W92→W117 ≈ 26 wks × 504 node-h max ⇒ 2000 ≈ 15% average utilization);
  flagship-prep blog; M30 bet-readiness gate complete.
- **[Venue] M31 end (W121)**: 2500+ runtime hours; flagship blog live; ecosystem shortlist + outbound started.
- **[Venue] M32 end (W124)**: 2900+ runtime hours; event trip / recon complete.
- **[Venue] M33 end (W128)**: 3300+ runtime hours; founding-eng + crypto-MM conversations underway.
- **[Venue] M34 end (W132)**: 3500+ runtime hours; Bet #1 placed OR Bridge₀ engaged.
- **[Venue] M35 end (W138)**: 3800+ runtime hours; capstone v1.5 spike (RWA/ADL) if Bet #1 timing allows.
- **[Venue] M36 end (W144)**: 4000+ runtime hours; plan close.

### Venue-track inheritance audit cadence

- **W74 Thu**: matching-engine v1.0 — ≥0.70.
- **W92 Wed**: ledger-deterministic v1.0 — ≥0.70.
- **W99 Wed (before the Friday v1.0 tag)**: risk-engine v1.0 (principal) — ≥0.70.
- **W102 Tue (before the tag)**: liquidation-engine v1.0 + cross-crate audit on the venue stack.
- **W113 Sat**: `perp-dex-core` CAPSTONE — most-important audit. **≥0.75** is the bar.
- **W117 Tue**: M30 audit across all venue crates.
- **W144 Tue**: M36 final audit. Capture in retrospective.

---

## APPENDIX B: Conference + Public-Visibility Schedule

4 conferences across 36 months:

- **EthCC Paris** — W57 (M15) — spine + Tempo focus. Aim: 3 Reth maintainer 1-on-1s.
- **Devcon** — W88 (M22) — spine + Tempo focus. Deepen 3 maintainer relationships.
- **Distributed-Systems / HFT conference** (QCon, P99 CONF) — W113-W124 (M28-M32) — venue focus; present if accepted.
- **Crypto-derivatives event** (Token2049 Dubai/SG) — W120-W130 (M30-M33) — **bet sourcing**: warm the Binance-alumni
  map, source Bet #1, scout the geography hub.

### Public-visibility cadence

- **Phase 1-3 (M1-M12)**: zero public posts. Build in private. (Exception: M6 `latency-lab` tick-to-trade report may
  seed a first technical thread.)
- **Phase 4 (M13-M18)**: Twitter warm-up only. Star repos, technical replies.
- **Phase 5 (M19-M24)**: optional blog post (storage-trie / consensus-engine / matching-engine retrospective).
- **Phase 6 (M25-M30)**: 4 blog posts (#4 chaos engineering W109; #5 **risk-engine / cross-margin** W112; #6
  **perp-dex-core architecture** W115; #7 Phase 6 retro W117).
- **Phase 7 (M31-M36)**: flagship blog W121 (the public artifact for bet-sourcing inbound) + optional close W141.
- **Count**: #1 optional Phase-5 retro; #4–#7 = W109/W112/W115/W117; flagship W121; optional close W141 —
  **6 committed**, 7 if the optional Phase-5 post ships.

---

## APPENDIX C: Day-job + Plan Integration Notes

The plan assumes a coast-mode day-job providing infrastructure (salary, health insurance, sponsored hours).

- **Hours**: 5h/day × 6 days/week. Day-job is the remaining bandwidth.
- **Coast mode**: meeting expectations cleanly so the day-job stays infrastructure rather than crisis.
- **No on-call**: heavy on-call is a structural conflict — negotiate or change before Phase 3.
- **Exit timing**: at Bet #1 placement OR Bridge₀ start (M34-ish). 30-day notice; don't burn bridges.

### Energy budget per phase (target hours/week)

| Phase | Months  | Spine | Venue | Tempo | Ops | Day-job                 | Sleep+health |
|-------|---------|-------|-------|-------|-----|-------------------------|--------------|
| 1     | M1-M3   | 30    | 0     | 0     | 0   | ~40                     | 14 (2h/d)    |
| 2     | M4-M6   | 26    | 2*    | 2     | 0   | ~40                     | 14           |
| 3     | M7-M12  | 27    | 0     | 3     | 0   | ~40                     | 14           |
| 4     | M13-M18 | 22    | 3     | 5     | 0   | ~40                     | 14           |
| 5     | M19-M24 | 12    | 23    | 5     | 0   | ~40                     | 14           |
| 6     | M25-M30 | 4     | 28    | 3     | 5   | ~40                     | 14           |
| 7     | M31-M34 | 3     | 29    | 3     | 5   | ~40 (then bridge/bet)   | 14           |
| 7     | M35-M36 | 2     | 20    | 3     | 5   | ~40 (then bridge/bet)   | 14           |

*\*M6 venue hours = `latency-lab`. Numbers approximate; the constraint is total work hours (30/wk → 40/wk M19-M34).*

---

## APPENDIX D: Cross-Reference — Which Week Builds Which Crate (v3 deltas marked)

| Week      | Primary crate work                                                                                         |
|-----------|------------------------------------------------------------------------------------------------------------|
| W1-W22    | Layer-0 bootstrap (eth-* mirrors) + spine seeds (exec-vm, eth-trie, storage-trie/consensus-engine scaffold)|
| **W21**   | **[NEW] latency-lab v0.1 (M6 latency leg)**                                                                |
| W23-W44   | storage-trie build-out → **v1.0 (W44)**; wal/recovery/bloom/lsm-core(+LCS+TWCS)/txn; runtime-tpc v0.1/v0.3 |
| W45-W48   | Spine maintenance + M12 gate                                                                               |
| W49-W57   | exec-vm hardening; p2p; consensus-raft; runtime-tpc v0.5; EthCC (W57)                                      |
| W58-W67   | **[Venue] matching-engine scaffold→v0.7**; consensus-raft v1.0; consensus-bft v0.1/v0.5; **exec-vm v1.0 (W68)** |
| **W63-W66** | **[NEW] log-distributed v0.1 seed (off-path Kafka)**                                                     |
| W66       | tempo-tx-envelope v0.1.0                                                                                   |
| W68-W74   | consensus-vsr v0.1/v0.5; txn v1.0; consensus-bft v1.0; **[Venue] matching-engine v1.0 (W74, VSR-replicable)** |
| W75-W82   | **[Venue] matching-engine v1.5 (STP/iceberg/stop-limit/auction/MBO/FIX/CB/mark-triggers)**; mmap-queue v0.5 |
| **W77-W80** | **[NEW] oracle-mark v0.5 (RWA-aware seams)**                                                             |
| **W80-W84** | **[NEW] log-distributed v0.5 (first VSR-log projection / read-side fan-out)**; messaging-aeron v0.7    |
| W80-W92   | ledger-deterministic v0.5→v0.7(VOPR)→v1.0(static-mem+io_uring+VSR); runtime-tpc v1.0; consensus-vsr v1.0   |
| **W85-W90** | **[NEW] risk-engine v0.5 (multi-instrument cross-margin, fixed haircut)**; marketdata-kernelbypass v0.5 |
| **W91**   | consensus-engine v1.0; **[NEW] `ConsensusBackbone` interface DECLARED (VSR wired behind it; acceptance #9)**; tempo-evm-ext + tempo-payment-lane v0.1.0; exec-vm v1.5 block-stm (W91-W94) |
| W95-W96   | M24 derivatives-infra readiness checkpoint                                                                 |
| **W97-W99** | **[NEW] risk-engine v1.0 (BTC-PERP + ETH-PERP netting, per-mark-tick)** + oracle-mark v1.0             |
| **W100-W102**| **[NEW] liquidation-engine v1.0 (partial-liq + insurance fund + waterfall + tripwire)**               |
| **W103**  | **[NEW] perp-dex-core depends on `ConsensusBackbone` trait (VSR adapter wired); NOT on consensus-vsr directly — acceptance #9 stub-swap test lives HERE (Flag-A: at dependency formation)** |
| W105      | ops-monitoring + ops-deploy                                                                                |
| **W103-W113**| **[NEW] perp-dex-core CAPSTONE assembly (3-node VSR-behind-interface + hybrid-boundary settle + RWA seams + cluster-VOPR)**|
| W106      | [Venue] live deployment begins (paper-trade rig)                                                           |
| **W108**  | ops-chaos + **cluster-VOPR: acceptance #4 (`finality_status` from `commit_point`) + #5 (two projection contracts) verified at the cluster level (Flag-A — distributed properties)** |
| W109      | ops-runbooks + Blog #4 (chaos engineering)                                                                 |
| W110-W112 | Sustained operations + matching-engine v1.5.1 (perf-tuned) + Blog #5 (risk-engine / cross-margin)          |
| W113      | ledger v1.0 + aeron v0.7 + marketdata v0.7 real-load polish; **Binance-alumni map activation + bet-outreach ramp** (per INDEX; capstone acceptance is NOT here — see W117) |
| W114-W116 | 2000h runtime + Blog #6 (perp-dex-core arch) + ecosystem shortlist                                         |
| **W117**  | **CAPSTONE v1.0 ACCEPTANCE GATE (Flag-A): converges #4/#5/#9 + multi-instrument netting + minimal insurance fund + hybrid-settle interface + stub-swap re-verified end-to-end against the locked MVP boundary (no NEW test); inheritance ratio ≥0.75** + Blog #7 + M30 Bet-Readiness gate |
| W118-W120 | perp-dex-core v1.5 (RWA features) start + bet-path prep (outreach, checklist, visa/token-tax); **[NEW] BFT apex DESIGN + SEED (pipelined-HotStuff behind `ConsensusBackbone`)** |
| W121      | **Flagship blog post (the public artifact for bet-sourcing inbound)** + perp-dex-core v1.5 (ADL/multi-collateral)|
| W122-W124 | Outbound triage + event/recon trip (geography scout); **[NEW] BFT apex core protocol build (leader / QC-chaining / pacemaker)** (interleaved) |
| W125-W128 | Founding-eng + crypto-MM conversations; checklist applied per opportunity; **[NEW] BFT apex core protocol (cont.)** |
| W129-W131 | **Bet #1 decision** (GO only if checklist passes) OR engage Bridge₀; **[NEW] BFT apex Byzantine VOPR (all 8 scenarios: equivocation / withholding / leader-equivocation / conflicting-QC / partition+Byzantine / orphaned-proposal-before-slot / OpNumber-contiguity-under-view-churn / safety-attack-planted-fork)** |
| W132-W134 | Resignation + (conditional) relocation; start at bet/bridge; **[NEW] BFT apex Byzantine-VOPR green** |
| W135-W142 | First weeks at bet/bridge; **[NEW] BFT apex hot-path latency target (p99≤2ms) + SWAP into perp-dex-core via the interface (perp-dex-core v2 = BFT-replicated, no engine rewrite)** — may be built inside the bet |
| **W139-W142** | **aeron + marketdata v1.0 hardening (sustain window)**                                                 |
| **W143**  | **[NEW] BFT-apex ACCEPTANCE (protocol family ✓ / N=4 / p99≤2ms / 8 Byzantine scenarios green / zero-rewrite swap ✓)** |
| W144      | **M36 retrospective + plan close** (8-system coverage ≥80; Bet #2 thesis seeds)                            |

---

## APPENDIX E: Personal Workspace Conventions (Adopted From Day One)

### Naming
- Crates: `kebab-case`. No `_`. Modules within a crate: `snake_case`.
- Type aliases for mirror crates: same name as the source upstream type (e.g. `Address` matches
  `alloy_primitives::Address`).
- Inherited primitive imports: always `use crate_name::TypeName` at file top; never re-export across crates unless
  explicitly re-exporting for a public API.

### Versioning
- v0.0.x: scaffold / seed. v0.x.x: development; minor versions break API freely. v1.0.0: API frozen, SemVer applies.
- Inheritance audits happen ONLY at v0.5+, v0.7+, and v1.0 tags. v0.0.x / v0.1.x scaffolds are exempt.

### Branches + PRs
- `main` always green (CI passes, clippy clean, miri clean on annotated crates).
- Local feature branches; rebase before merging. One commit per logical change; never amend a published commit.

### Testing
- Every crate: unit tests + ≥1 integration test by v0.5.
- Layer-2/3/4 crates: proptest by v1.0.
- Layer-5 products: cargo-fuzz + loom for race-prone modules by v0.5.
- Bar-(c) crates: VOPR-style deterministic harness; the venue carries **cluster-level VOPR** (linearizability + replica
  convergence + insurance-fund tripwire StateChecker).
- **[NEW v3.2] `sim-storage` fault module** (TigerBeetle mirror) — a fault-injecting `Storage` impl behind the mandated
  `Storage` trait seam, reused by every VOPR leg (no primitive twice): per-seed read/write-fault probability, torn
  writes, bit-corruption, misdirected I/O, crash-corruption of in-flight writes, with a `FaultAtlas` keeping ≥1 valid
  replica per block. Folded into `ledger` VOPR (W88), cluster-VOPR (W108), and the Byzantine VOPR (W129, faults⊕Byzantine).
- Inheritance audit at v1.0 (and v0.5 for Layer-5 capstones).

### Docs
- Every public item has a doc comment by v0.5.
- Every crate has a DESIGN.md by v1.0 with the inheritance tree as ASCII.
- README at workspace root updated whenever a new crate ships v0.5+.

### Notes folder structure
- `notes/01_kotlin_to_rust_delta.md` … `notes/07_variance.md` (W1-W4)
- `notes/08_revm_diff.md` (W18); `notes/tempo_*` (W16+)
- `notes/matching_engine_design.md` (W58); `notes/latency_lab_notes.md` (W21)
- `notes/oracle_mark_design.md` (W77); `notes/risk_engine_design.md` (W85); `notes/liquidation_design.md` (W101)
- `notes/perp_dex_core_design.md` (W105); `notes/cluster_vopr_log.md` (W105+)
- `notes/chaos_log.md` (W108+); `notes/network_map.md` (W113); `notes/ecosystem_shortlist.md` (W114)
- `notes/inbound.md` (W121+); `notes/relocation_research.md` (W118); `notes/recon_notes.md` (W123-W124)
- `progress.md` (W1 → W144, the source of truth)

---

## APPENDIX F: One-Line Rules To Live By

In rough order of importance:

1. No primitive twice.
2. Inheritance ratio ≥0.70 on Layer-5 products, ≥0.75 on the capstone, ≥0.85 on Layer-7 Tempo crates.
3. Ship at v0.5 if scope is at risk; v0.7 / v1.0 catch up later. Lock v1.0 per the capstone scope ladder.
4. 4-hour floor, 5-hour target. Done at 3h → rest. Coverage is non-negotiable — invest more time, don't trim.
5. Sleep 7h. Fitness 3x/week. Coast mode on day-job.
6. Sunday ritual is non-negotiable.
7. Read before writing.
8. M6 = latency leg. M12 = calibrate. M24 = derivatives-infra readiness. M30 = bet-readiness pivot. M36 = bet placed
   or bridge engaged.
9. The job is the income floor; the founding token-equity bet is the goal.
10. A bet is GO only if all seven checklist gates pass. Otherwise it's a bridge job.
11. The VSR core replays bit-identically; non-determinism lives at the edges.
12. The ~36 crates are the deliverable — and `perp-dex-core` is the one that clears the founding-eng bar. Everything
    else is means.

---

**Plan complete (v3). Move to W1 Monday.**
