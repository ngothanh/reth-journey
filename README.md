# The Inheritance Plan — Reth Core + HFT Depth + Tempo Application Layer (36-Month Daily Plan)

> **Start**: 2026-04-27
> **Horizon**: 36 months, decision gates at M12 / M24 / M30 / M36
> **Commitment**: 5h/day × 6 days/week = 30h/week
> **Schedule**: Mon-Sat work, Sunday rest + weekly ritual

---

## Strategic Frame (Revised Three-Track Architecture)

This plan integrates three tracks that share a single substrate of low-level primitives:

- **Reth core (M1–M18 primary, M19–M36 maintenance)** — the original 24-month Reth contributor plan: workspace crates
  mirroring alloy / reth / revm, scaling to three flagship deliverables (`storage-trie` v1.0, `exec-vm` v1.0,
  `consensus-engine` v1.0).
- **HFT depth (M15 scaffold, M19–M34 primary)** — matching engine, deterministic ledger, kernel-bypass market data,
  ARIES-style WAL/recovery, Aeron-style messaging, BFT consensus, mini-LSM database, HNSW vector index. All built on the
  same primitives as the Reth track.
- **Tempo application layer (additive throughout)** — `tempo-tx-envelope`, `tempo-evm-ext`, `tempo-payment-lane`. Layer
  7 on top of everything below. Tempo remains optionality; Path D at M24 is conditional on the three-condition test (≥15
  Tempo PRs merged, ≥2 maintainer relationships, upstream substantively engaged with `tempo-payment-lane`).

**The principle**: no primitive is built twice. A `wal` crate ships once at W26 and is consumed by `storage-trie` (
Reth), `ledger-deterministic` (HFT), `mini-db` (databases), and downstream by `consensus-raft` snapshots and
`matching-engine` durable command logging. Same for `bufpool`, `time`, `txn`, `p2p`, `consensus-raft`, `consensus-bft`.
Layer-5 products inherit ~70% of their components; Tempo at Layer 7 inherits ~85%.

**Final-phase deliverables**:

- `storage-trie` v1.0 (W44) — reth storage + trie re-implementation
- `exec-vm` v1.0 (W68) — revm + reth evm re-implementation
- `consensus-engine` v1.0 (W91) — reth consensus + engine API re-implementation
- `matching-engine` v1.0 (W74) — multi-symbol order book + perpetuals + raft-replicated
- `ledger-deterministic` v0.5 (W83) — deterministic SM + double-entry bookkeeping + journal
- `messaging-aeron` v0.5 (W79) — term-buffer + flow control + NAK gap recovery
- `marketdata-kernelbypass` v0.5 (W90) — epoll + io_uring + AF_XDP
- `mini-db` v1.0 (W100) — full LSM database (CAPSTONE: assembles `wal` + `recovery` + `txn` + `bufpool` +
  `storage-trie` + `bloom`)
- `vector-db` v0.5 (W104) — HNSW + SQ/PQ + filtered search

**Tempo crate deliverables (additive, optionality preserved)**:

- `tempo-tx-envelope` v0.1.0 — W66
- `tempo-evm-ext` scaffold W54 → v0.1.0 W91
- `tempo-payment-lane` scaffold W83 → v0.1.0 W91

---

## Decisions Locked

These were the open questions; the answers below are baked into the plan.

1. **Three concurrent tracks from M15** — Reth core remains primary M1–M18; HFT begins as scaffold at W58 (
   matching-engine v0.0) and becomes primary M19–M34; Tempo is additive throughout.
2. **Inheritance discipline** — every Layer-5 product crate explicitly lists which Layer-0–Layer-4 primitives it
   inherits and which 3–5 components are net-new. No primitive built twice.
3. **Layer-1 primitives ship before products** — `time` (W6), `backpressure` (W11), `bufpool` (W14), `wal` (W26),
   `recovery` (W29–W30), `txn` (W42 v0.5, W72 v1.0 with 2PC), `p2p` (W52–W55), `consensus-raft` (W56–W67),
   `consensus-bft` (W64–W73), `messaging-aeron` (W76–W79), `marketdata-kernelbypass` (W85–W90).
4. **`mini-db` is the CAPSTONE** — W95–W100. It is the integration moment where five years of "build the primitive once"
   pays off in a single product where ≥70% of LOC is wired-up inheritance.
5. **`vector-db` stops at v0.5** — W101–W104, single-node, HNSW + SQ/PQ + filtered search. Not pursued to distributed
   because Raft and sharding are already covered by `matching-engine` v1.0 (W74) and `mini-db` (W100); reuse, don't
   rebuild.
6. **Operations-hours target**: 2000+ runtime hours by M30; 4000+ by M36. Live deployment begins W106 (M25 mid).
7. **M24 decision is five-pathed** — Path A (extend Reth), Path B (post-Reth systems), Path C (catch-up), Path D (Tempo
   pivot, conditional), Path E (HFT destination-tier IC track, new default if signals strong).
8. **Destination landing in Phase 7** — applications start W125; interview prep W119; flagship blog post W121; offer
   decision W131; resignation + relocation W132; arrival W134; first month at new firm W135–W144.

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
  exec-vm/               W9-W68   -> revm; opcodes, gas, journal, precompiles
  eth-trie/              W10-W20  -> alloy-trie; Nibbles, HashBuilder, ProofRetainer

LAYER 1 — universal low-level primitives (W4-W37)
  concurrent/            W4-W37   -> crossbeam-utils + crossbeam-queue + crossbeam-channel + crossbeam-skiplist mirror;
                                      CachePadded, Backoff, AtomicCell, Parker (W4), bounded MPMC Vyukov ring (W11),
                                      unbounded MPMC SegQueue (W26), select!-style multi-channel (W63),
                                      lock-free concurrent skiplist (W37, consumes epoch-gc).
                                      INHERITED BY: backpressure (W11), wal (W26), lsm-core (W38),
                                      matching-engine (W58+), messaging-aeron (W77), consensus-raft commands
  time/                  W6       -> monotonic + Lamport + HLC stub + hardware-ts trait
                                      INHERITED BY: wal (W26), recovery (W29), txn (W42),
                                      matching-engine (W58), ledger (W80), messaging-aeron (W76),
                                      tempo-tx-envelope (valid_before/valid_after timestamps)
  backpressure/          W11      -> extracted from eth-network-codec's BackpressureStrategy enum
                                      INHERITS: concurrent (bounded MPMC)
                                      INHERITED BY: matching-engine, messaging-aeron, marketdata
  bufpool/               W12-W14  -> LRU-K page cache + pin/unpin + dirty page tracking
                                      EXTRACTED FROM eth-storage-cache Page work (W2)
                                      INHERITED BY: storage-trie, wal, mini-db, vector-db
  epoch-gc/              W33-W37  -> crossbeam-epoch mirror; epoch-based memory reclamation foundation
                                      for any lock-free data structure that hands out pointers.
                                      INHERITED BY: concurrent::skiplist (W37), matching-engine lock-free
                                      price level (W74)

LAYER 2 — durability primitives (W26-W30)
  wal/                   W26      -> segment + group commit + checksums + replay
                                      INHERITS: time, bufpool, eth-storage-cache::Page
                                      INHERITED BY: storage-trie (W31+), ledger (W80),
                                      matching-engine (durable command log, W74),
                                      consensus-raft (snapshot log), mini-db (W95)
  recovery/              W29-W30  -> ARIES 3-pass: analysis, redo, undo
                                      INHERITS: wal, time
                                      INHERITED BY: storage-trie, ledger, mini-db,
                                      matching-engine (replay after replica failover)

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

LAYER 4 — distribution + transport primitives (W52-W73, W76-W79)
  p2p/                   W52-W55  -> Kademlia + Noise XX + gossip
                                      INHERITS: time, eth-network-codec
                                      INHERITED BY: consensus-raft, consensus-bft,
                                      messaging-aeron (peer discovery)
  consensus-raft/        W56-W67  -> election + log replication + membership + log compaction
                                      INHERITS: time, wal, p2p, txn
                                      INHERITED BY: matching-engine v1.0 (W74),
                                      mini-db distribution (W99-100, scope-trimmed)
  consensus-bft/         W64-W73  -> 3-phase voting + locking + fork-choice + evidence
                                      INHERITS: time, wal, p2p
                                      INHERITED BY: consensus-engine (Engine API fork-choice analogue)
  messaging-aeron/       W76-W79  -> term buffer + flow control + NAK gap recovery
                                      INHERITS: time, backpressure, bufpool
                                      INHERITED BY: matching-engine market data fan-out,
                                      marketdata-kernelbypass downstream
  marketdata-kernelbypass/ W85-W90 -> epoll baseline + io_uring + AF_XDP
                                      INHERITS: time, backpressure
                                      INHERITED BY: matching-engine exchange-facing feed handler

LAYER 5 — products (capstones; each is ≥70% inherited)
  storage-trie/          W23-W44  -> MDBX-backed state DB + trie
                                      INHERITS: bufpool, wal, recovery, txn, eth-trie, eth-storage-cache
                                      NET-NEW: MdbxTrieStorage, MerkleStage, pruning, snapshots
  matching-engine/       W58-W74  -> multi-symbol L2 order book + perpetuals + raft-replicated
                                      INHERITS: time, backpressure, wal, recovery, consensus-raft, messaging-aeron
                                      NET-NEW: order book (RB-tree + price-time priority),
                                               risk pre-trade, funding + liquidation engines, ADL
  ledger-deterministic/  W80-W83  -> deterministic SM + double-entry + journal (TigerBeetle-style)
                                      INHERITS: time, wal, recovery, txn
                                      NET-NEW: deterministic op set, accounts/transfers schema, snapshots
  consensus-engine/      W24-W91  -> reth consensus + Engine API
                                      INHERITS: eth-consensus, eth-stage, exec-vm, storage-trie, consensus-bft (fork-choice)
                                      NET-NEW: engine_api server + JWT + payload builder + fork-choice glue
  mini-db/               W95-W100 -> full LSM database (CAPSTONE — inheritance ratio target ≥0.70)
                                      INHERITS: lsm-core, wal, recovery, txn, bufpool, bloom, time, backpressure
                                      NET-NEW: kv API, range scan, snapshot iterator, distributed sharding stub
  vector-db/             W101-W104 -> HNSW + SQ/PQ + filtered search (STOPS AT v0.5, single-node)
                                      INHERITS: bufpool, bloom, txn (limited), time
                                      NET-NEW: HNSW graph construction + greedy search, SQ/PQ quantizers,
                                               filtered search

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
mini-lsm / Qdrant / Aeron / Chronicle. The 15 + 9 + 3 = 27 crates of this workspace are the deliverable. Everything else
is means. (Layer-1 added `concurrent/` and `epoch-gc/` — crossbeam family mirrors required for lock-free DS depth.)

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

### HFT track (begins M15)

| Metric                                          | M18  | M24   | M30  | M36  |
|-------------------------------------------------|------|-------|------|------|
| HFT-side production crates shipped              | 1    | 5     | 8    | 9    |
| matching-engine version                         | scaf | v1.0  | v1.0 | v1.0 |
| ledger-deterministic version                    | —    | v0.5  | v0.7 | v1.0 |
| messaging-aeron version                         | —    | v0.5  | v0.7 | v1.0 |
| marketdata-kernelbypass version                 | —    | v0.5  | v0.7 | v1.0 |
| mini-db version                                 | —    | —     | v1.0 | v1.0 |
| vector-db version                               | —    | —     | v0.5 | v0.5 |
| Runtime hours on chaos-tested rig               | 0    | 200   | 2000 | 4000 |
| P99 matching latency (single-symbol, 1M orders) | —    | <5μs  | <2μs | <1μs |
| P99 marketdata fan-out latency (IPC)            | —    | <10μs | <5μs | <2μs |
| Public blog posts shipped                       | 0    | 1     | 4    | 7    |

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

---

# PHASE 1: RUST MASTERY (Month 1-3)

> **No Tempo entries this phase.** Rust mastery is the prerequisite.
> Layer-1 primitives extracted as a side effect: `time` (W6), `backpressure` (W11), `bufpool` scaffold (W12). No HFT
> entries this phase.

## Month 1: Rust Core (Weeks 1-4)

### Week 1 — Ownership/borrowing/lifetimes via `eth-primitives` foundation

**Mirror target**: `alloy-primitives` (Address, B256, U256, Bytes, FixedBytes)
**Crate created**: `crates/eth-primitives/`.
**Feeds into**: every later week.

**Pre-week setup**: ✓ already done

**Monday — Skim the Book chs 1-9, write nothing + workspace scaffold**

- [X] Speed-read Book ch1-3 (~30 min)
- [X] Speed-read Book ch5-9 (~90 min): structs, enums, modules, collections, error handling
- [X] Skip beginner Rustlings: `intro`, `variables`, `functions`, `if`, `primitive_types`, `strings`, `vecs`,
  `hashmaps`, `modules`
- [X] Write `notes/01_kotlin_to_rust_delta.md`
- [X] Create workspace `Cargo.toml` (resolver = "2", members = ["crates/*"])
- [X] Create `crates/eth-primitives` with Cargo.toml, src/lib.rs, src/error.rs skeleton
- [X] Read alloy-primitives top-level lib.rs + map the 8 types you'll build this week
- [X] Commit + log

**Tuesday — Book ch4 + `FixedBytes<const N: usize>`**

- [X] Book ch4.1 (Ownership) — read twice
- [X] Book ch4.2 (References and Borrowing) — read twice
- [X] Book ch4.3 (Slices)
- [X] Rustlings `move_semantics` (all 6)
- [X] **Build**: `crates/eth-primitives/src/fixed_bytes.rs` — `FixedBytes<const N: usize>([u8; N])` with Copy, Default,
  From, AsRef, AsMut, Deref, PartialEq, Hash. repr(transparent).
- [X] Test: zero-init, equality, slice access, hash stability. Match alloy-primitives test cases.
- [X] Borrow-checker drill: `fn split(&mut self) -> (&mut [u8], &mut [u8])` resolved via split_at_mut. Document in
  `notes/02_borrow_checker_errors.md`.
- [X] Commit + log

**Wednesday — Lifetimes + `Bytes` + `BytesView<'a>`**

- [X] Book ch10.3 (Lifetimes) — read twice
- [X] Watch Crust of Rust: Lifetime Annotations (full)
- [X] **Build**: `crates/eth-primitives/src/bytes.rs` — `Bytes(Arc<[u8]>)` cheap-clone wrapper. Methods: new,
  from_static, slice, len, is_empty, as_ref.
- [X] **Build**: `BytesView<'a>(&'a [u8])` borrowed views. Add `Bytes::view(&self) -> BytesView<'_>`.
- [X] Implement `From<Vec<u8>>`, `From<&'static [u8]>`, `Display` (lowercase hex with 0x prefix).
- [X] Document lifetime elision rules in `notes/03_lifetimes.md`.
- [X] Commit + log

**Thursday — Traits + `Address` + `B256` + sealed-trait pattern**

- [X] Book ch10.1 + ch10.2 (Generics, Traits)
- [X] Rustlings `generics`, `traits` — all
- [X] Read about orphan rule, coherence, sealed traits
- [X] **Build**: `crates/eth-primitives/src/address.rs` — `pub type Address = FixedBytes<20>;` + EIP-55 checksum
  encoding.
- [X] **Build**: `crates/eth-primitives/src/aliases.rs` — `B256 = FixedBytes<32>`, `B64 = FixedBytes<8>`.
- [X] **Build**: sealed-trait pattern — `mod private { pub trait Sealed {} }`. impl Sealed for Address, B256, Bytes.
- [X] Write 4 functions over the sealed trait (&dyn, Box<dyn>, impl, <T:>). Observe what compiles.
- [X] Notes in `notes/04_traits.md`: static vs dynamic dispatch.
- [X] Commit + log

**Friday — Error handling + iterators via `PrimitivesError` + hex parsing**

- [X] Book ch9 + ch13.1 + ch13.2
- [X] Rustlings `error_handling`, `options`, `iterators` — all
- [X] Read `thiserror` and `anyhow` docs end-to-end
- [X] **Build**: `crates/eth-primitives/src/error.rs` — `PrimitivesError` enum with thiserror::Error.
- [X] **Build**: `FromStr` for Address, B256, Bytes — iterator-driven byte-pair decoder.
- [X] Three rewrites of `parse_address`: panic, Result+thiserror, anyhow.
- [X] Watch Crust of Rust: Iterators (full).
- [X] Implement `flatten()` from scratch — applied as `Bytes::concat`.
- [X] Commit + log

**Saturday — Closures + Fn/FnMut/FnOnce + `U256` + R4R**

- [X] Book ch13.1 (Closures) — FnOnce/FnMut/Fn semantics
- [X] **Build**: `crates/eth-primitives/src/uint.rs` — `pub use ruint::aliases::U256;` + extension trait `U256Ext`.
- [X] Closure exercise: `Bytes::map_chunks<F: FnMut(&[u8]) -> Bytes>` — used by RLP encoder Week 5.
- [X] Read Rust for Rustaceans ch1-2.
- [X] `cargo clippy --all -- -D warnings`, `cargo test`, tag `eth-primitives v0.1.0-week1`.
- [X] Commit + log

**Sunday — Rest + Weekly Ritual**

- [X] "Can I explain ownership/borrowing/lifetimes using `Bytes::slice` and `FixedBytes` without looking up?"
- [X] Inheritance check: eth-primitives exports complete.

---

### Week 2 — Smart pointers + sync concurrency via `eth-storage-cache`

**Mirror target**: `revm::db::CacheDB` + reth_provider in-memory layer + `revm_primitives::Database` trait shape.
**Crate created**: `crates/eth-storage-cache/`.
**Inherits from**: `eth-primitives`.
**Feeds into**: `exec-vm` Phase 4; `storage-trie` Phase 3; [NEW] `bufpool` extraction W12.

**Monday — Box, Deref, Drop via `Page` primitive**

- [X] Book ch15.1-15.4
- [X] **Build**: `crates/eth-storage-cache/src/page.rs` — `Page(Box<[u8; 4096]>)` with Deref, DerefMut, Drop
  instrumented via tracing::trace!. This 4 KiB page is reused for mmap-backed layout in Phase 3, AND is the primitive
  that `bufpool` (W12) will own and pool.
- [X] Implement `MyBox<T>` exercise applied as `PageBox<T: ?Sized>` — single-allocation deserialize-in-place. Shape MDBX
  cursors use.
- [X] Single-linked list of Pages as a free-list allocator (`PageAllocator`). Attempt doubly-linked free list to feel
  the pain → motivates Rc/Weak Tuesday.
- [X] Commit + log

**Tuesday — RefCell, Rc, Arc via `Account` cache**

- [X] Book ch15.5-15.6
- [X] Watch Crust of Rust: Smart Pointers and Interior Mutability
- [X] **Build**: `crates/eth-storage-cache/src/account.rs` —
  `Account { nonce: u64, balance: U256, code_hash: B256, code: Option<Bytes> }` mirroring revm_primitives::Account.
- [X] **Build**: `LocalAccountCache(HashMap<Address, Rc<RefCell<Account>>>)` first — single-threaded. Add get_or_load,
  commit. Use RefCell::borrow_mut and observe the runtime panic when you double-borrow.
- [X] **Migrate**: clone the file to `SharedAccountCache(HashMap<Address, Arc<RwLock<Account>>>)`. Document the diff in
  `notes/05_smart_pointers.md`.
- [X] Commit + log

**Wednesday — Threads, channels, Mutex via `StateCache` trait**

- [X] Book ch16 (whole chapter)
- [X] Watch Crust of Rust: Channels — implement bounded MPSC from scratch
- [X] **Build**: `crates/eth-storage-cache/src/database.rs` — `StateCache` trait shaped like revm's `Database` (basic,
  code_by_hash, storage, block_hash).
- [X] Implement `MutexCache` and `RwLockCache`. Apply bounded-MPSC as write-batch queue.
- [X] Read `parking_lot::Mutex` vs std — keep parking_lot (reth uses it).
- [X] Commit + log

**Thursday — Send/Sync via `ShardedCache`**

- [X] Book ch16.4 (Send and Sync)
- [X] Read `std::marker` docs carefully
- [X] **Build**: `ShardedCache<const N: usize>` — `[parking_lot::RwLock<HashMap<Address, Account>>; N]` hash-routed by
  `Address::word()[0] % N`. Implement StateCache.
- [X] Send/!Sync + !Send/Sync exercises grounded in the cache.
- [X] Commit + log

**Friday — `EvictionPolicy` + criterion benches**

- [X] **Build**: `crates/eth-storage-cache/src/eviction.rs` — `EvictionPolicy` trait. LruEviction + BlockTagEviction.
  The LRU-K variant gets extracted into `bufpool` at W12.
- [X] Wire eviction into ShardedCache.
- [X] criterion bench: Mutex vs RwLock vs Sharded(N=16, N=64). Plot and commit.
- [X] Read parking_lot, dashmap, arc-swap docs — 1-paragraph summary each.
- [X] Commit + log

**Saturday — Polish + R4R + tag v0.1.0**

- [X] thiserror StateCacheError, tracing spans, loom tests on tiny subset.
- [X] Read Rust for Rustaceans ch1-2.
- [X] README + tag `eth-storage-cache v0.1.0`.
- [X] Commit + log

**Sunday — Rest + Weekly Ritual**

- [X] Inheritance check: StateCache mirrors revm's Database. Page primitive is now the seed for `bufpool` W12.

---

### Week 3 — Async/Pin/Future via `eth-network-codec`

**Mirror target**: `reth-eth-wire` framing layer + `tokio_util::codec::Framed`.
**Crate created**: `crates/eth-network-codec/`.
**[NEW] Side product**: `BackpressureStrategy` enum (Saturday) is the seed for the `backpressure` crate extracted at
W11.

**Monday — Tokio fast track + transport scaffold**

- [X] Read Tokio tutorial cover-to-cover. (~90 min)
- [X] **Build**: `crates/eth-network-codec/src/transport.rs` — TcpStream wrapper + `LengthDelimitedCodec` with 1 MiB max
  frame. (~60 min)
- [X] Manual TCP echo via framed transport. (~30 min)
- [X] Commit + log (~10 min)

**Tuesday — Manual Future + `MessageRequest`**

- [ ] Async Book ch1-7 in one go. (~90 min)
- [ ] Watch Crust of Rust: Async/Await (full) — implement trivial executor. (~75 min)
- [ ] **Build**: `crates/eth-network-codec/src/request.rs` — `MessageRequest<R>` future. (~60 min)
- [ ] Counter Future applied: `RetryFuture<F: Future>`. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Pin/Unpin via `MessageStream`**

- [ ] Watch Crust of Rust: The Drop Check; read `std::pin` docs. (~75 min)
- [ ] **Build**: `crates/eth-network-codec/src/stream.rs` — `MessageStream<C: Codec, IO>` implementing
  `tokio_stream::Stream`. Use `pin_project_lite`. (~60 min)
- [ ] Demonstrate why MessageStream cannot be Unpin. Rewrite once with manual unsafe pin projection, then with
  pin_project_lite. Compare. (~45 min)
- [ ] `notes/06_pin_unpin.md` — worked example. (~25 min)
- [ ] Commit + log (~10 min)

**Thursday — `EthMessage` enum + `Codec` trait**

- [ ] **Build**: `crates/eth-network-codec/src/codec.rs` — `Codec` trait. (~60 min)
- [ ] **Build**: `crates/eth-network-codec/src/message.rs` — `EthMessage` enum subset: Status, BlockHeaders,
  BlockBodies, NewBlock, GetBlockHeaders. (~45 min)
- [ ] RLP placeholder (tagged-byte format; full RLP comes Week 5). (~30 min)
- [ ] tokio TCP server with graceful shutdown. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Token bucket as custom `Future` + per-peer rate limiting**

- [ ] **Build**: `crates/eth-network-codec/src/rate_limit.rs` — `TokenBucket` as custom Future. (~60 min)
- [ ] **Build**: `RateLimitedStream<S: Stream>`. (~75 min)
- [ ] Test under load (1k concurrent peers). (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — `BackpressureStrategy` + observability + tag v0.1.0**

- [ ] **Build**: `BackpressureStrategy` enum (DropOldest, DropNewest, Block). **[NEW]** Mark this enum with a doc
  comment: "extracted into `backpressure` crate W11." (~45 min)
- [ ] Tracing spans for connection lifecycle. (~30 min)
- [ ] Prometheus metrics via `metrics` crate. (~30 min)
- [ ] Load test with 10k concurrent connections. (~30 min)
- [ ] Tag `eth-network-codec v0.1.0`. (~5 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] Inheritance check. (~15 min)

---

### Week 4 — Atomics, unsafe, variance, macros via `eth-primitives` v0.2

**Crate extended**: `eth-primitives` v0.1 → v0.2. Add `crates/eth-primitives-derive/`.

**Monday — Layout audit on existing `eth-primitives`**

- [ ] Rustonomicon ch1, ch2, ch3. (~30 min)
- [ ] Run size_of/align_of over every type. Verify FixedBytes<N> is repr(transparent). (~30 min)
- [ ] Add repr(C) to Account in eth-storage-cache. (~30 min)
- [ ] Inspect Bytes layout — Arc<[u8]> 2-word size; `notes/07_variance.md`. (~25 min)
- [ ] Commit + log (~10 min)

**Tuesday — Atomics via `SealedHeader` + `ChainHead` SeqLock + [NEW] `concurrent/` Layer-1 scaffold**

- [ ] Watch Crust of Rust: Atomics and Memory Ordering (full). (~75 min)
- [ ] **Build**: `crates/eth-primitives/src/atomic_hash.rs` — OnceLock<B256> lazy hash cache. `Sealable` trait. (~75 min)
- [ ] **Build**: `crates/eth-primitives/src/chain_head.rs` — `ChainHead { hash, number }` protected by SeqLock. (~75 min)
- [ ] **Build**: `crates/concurrent/Cargo.toml` workspace member (Layer-1 primitive, no deps). Mirror of
  `crossbeam-utils` + `crossbeam-queue` + `crossbeam-channel` + `crossbeam-skiplist`. Builds incrementally through W37. (~20 min)
- [ ] **Build**: `crates/concurrent/src/cache_padded.rs` — `CachePadded<T>` mirror of `crossbeam-utils`.
  `#[repr(align(128))]` wrapper. (~75 min)
  - **Expect to hit #1**: you'll pick `align(64)` (the textbook x86 cache line). On Apple Silicon (your dev machine)
    the L2 prefetcher pulls **128-byte pairs**, so producer/consumer counters still false-share. Bench shows ~30%
    regression vs unpadded baseline that "shouldn't" happen.
    **Fix**: align(128) on aarch64-apple-darwin; align(64) on x86_64-linux. cfg-gate.
  - **Expect to hit #2**: you put `CachePadded<AtomicU64>` inside a struct without `repr(C)`. Rust reorders fields
    and your padding ends up wedged between unrelated members. False-sharing returns.
    **Fix**: force `repr(C)` on any struct where padding is load-bearing.
  - **Muscle**: cache-line padding is **target-dependent and layout-dependent**. **Reapplies at**: matching-engine
    `PriceLevel` (W58), messaging-aeron term rotation counters (W77), Disruptor claim/cursor pair (W65).
- [ ] **Build**: `crates/concurrent/src/backoff.rs` — adaptive `Backoff { step: Cell<u32> }` mirror of
  `crossbeam-utils`. `spin()` escalates `core::hint::spin_loop()` → `thread::yield_now()` → park over ~10 steps. (~75 min)
  - **Expect to hit #1**: tight `while !ready.load(Relaxed) {}` without `hint::spin_loop()` (PAUSE on x86). 100% CPU
    on the waiter *and* slower wake-ups, because SMT siblings are starved.
    **Fix**: every busy wait calls `hint::spin_loop()` per iteration.
  - **Expect to hit #2**: pure exponential spin works on a CAS retry (transient contention) but **fails** on
    structural contention (a writer holding a lock for 1ms). The spinner starves the holder.
    **Fix**: escalate to `yield_now` then park — never spin indefinitely.
  - **Muscle**: spin loops are a 3-stage ladder — `hint::spin_loop` → `yield_now` → block. **Reapplies at**: bounded
    MPMC retry (W11), SegQueue retry (W26), skiplist CAS retry (W37), term-buffer claim spin (W77).
- [ ] **Build**: `crates/concurrent/src/atomic_cell.rs` — `AtomicCell<T: Copy>` with fast path for
  `size_of::<T>() == 8 && align_of::<T>() == 8` (transmute to AtomicU64); spinlock fallback otherwise. (~75 min)
  - **Expect to hit #1**: `struct S { a: u8, b: u64 }` is `Copy` and 16 bytes — but transmuting it to bytes exposes
    padding that is uninitialized. `cargo +nightly miri test` fires "encountered uninitialized memory."
    **Fix**: zero-initialize via `MaybeUninit::zeroed()` then write field-by-field; or require
    `Pod`-trait callers.
  - **Expect to hit #2**: you write the fast path for ≤8 bytes but never test the slow path (spinlock fallback).
    `T = [u8; 16]` silently takes the wrong branch and you don't notice.
    **Fix**: const-assert the branch (`const _: () = assert!(size_of::<T>() == 8)` in a gated module) AND ship a
    test for both `T = u64` and `T = [u8; 16]`.
  - **Muscle**: every `unsafe` invariant needs both a static assertion AND a test that exercises the gated branch.
    **Reapplies at**: SegQueue per-slot writes (W26), epoch-gc `Atomic<T>` (W33).
- [ ] **Build**: `crates/concurrent/src/parker.rs` — `Parker` + `Unparker` pair. State machine:
  `EMPTY` ⟷ `PARKED` ⟷ `NOTIFIED`. (~75 min)
  - **Expect to hit #1**: the classic **lost wakeup**. Wrong sequence: check flag → flag false → call `park`.
    Between check and park, `unpark` fires and sets `NOTIFIED`; your `park` doesn't first check for `NOTIFIED`,
    so you sleep forever. Loom finds this in <100 iterations.
    **Fix**: `park` first CAS `EMPTY → PARKED`; if CAS fails because state is `NOTIFIED`, return immediately.
  - **Expect to hit #2**: spurious wakeups (Linux futex, macOS dispatch). If your `park` isn't in a loop, you wake
    on noise and proceed as if `unpark` was called.
    **Fix**: caller wraps `parker.park()` in `while !condition { parker.park() }`.
  - **Muscle**: every blocking primitive needs `state + wait`. Never just `wait`. The state must be CAS'd before
    sleeping; the wait loop must re-check on every wake. **Reapplies at**: bounded MPMC `recv` block-on (W11), WAL
    group-commit oneshot ack (W26), every channel you'll ever write.
- [ ] Re-read Ryuo disruptor code with fresh atomics eyes. Note: the SeqLock pattern here is identical to the one
  matching-engine (W58) will use for L1 best-bid/ask publishing. (~45 min)
- [ ] Commit + log (~10 min)

**Wednesday — Variance + PhantomData via `Sealed<T>`**

- [ ] Watch Crust of Rust: Subtyping and Variance. (~75 min)
- [ ] **Build**: `crates/eth-primitives/src/sealed.rs` — `Sealed<T> { inner, hash: OnceLock<B256> }`. (~75 min)
- [ ] Make covariant via PhantomData<&'a T> for SealedRef<'a, T>. (~30 min)
- [ ] R4R ch6. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Unsafe + miri via `BytesMut::reserve`**

- [ ] Read Nomicon chapters on aliasing, UB. (~75 min)
- [ ] **Build**: `crates/eth-primitives/src/bytes_mut.rs` — `BytesMut`. reserve + extend_from_slice with raw pointer
  arithmetic. `BytesMut::freeze` to Bytes. (~75 min)
- [ ] Run `cargo +nightly miri test -p eth-primitives`. Chase every UB report. (~60 min)
- [ ] Commit + log (~10 min)

**Friday — Macros via `b256!` + `SimpleEncode` derive**

- [ ] Read R4R ch7 + Little Book of Rust Macros. (~75 min)
- [ ] **Build**: `crates/eth-primitives/src/macros.rs` — `b256!`, `address!` const macros. (~75 min)
- [ ] **Build**: `crates/eth-primitives-derive/` proc-macro crate (syn + quote). `#[derive(SimpleEncode)]` placeholder
  for Week 5's RlpEncodable. (~45 min)
- [ ] Test the derive on a 3-field struct. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — R4R + integration polish**

- [ ] R4R ch1-5 — finish. (~30 min)
- [ ] Apply at least one R4R insight to refactor existing crates. (~30 min)
- [ ] Tag `eth-primitives v0.2.0`. (~5 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 1 review**

- [ ] Honest assessment: "Could I read reth-trie source today?" (~30 min)
- [ ] Inheritance check: 4 crates shipped. (~15 min)
- [ ] Update North Star M1 metrics. (~15 min)

---

## Month 2: Production Rust + Early Alloy (Weeks 5-8)

### Week 5 — `eth-rlp` crate + Alloy onboarding

**Mirror target**: `alloy-rlp` + `alloy-rlp-derive`.

**Monday — Spec + traits**

- [ ] Re-read W4 Fri's `eth-primitives-derive` scaffold (Cargo.toml proc-macro = true, syn/quote/proc-macro2 deps, basic
  DeriveInput parsing). 5 min refresh so you don't spend 30 min re-orienting Friday. (~5 min)
- [ ] Read RLP spec. Read alloy-rlp's `Encodable` and `Decodable` source. (~60 min)
- [ ] **Build**: `crates/eth-rlp/src/lib.rs` — `Encodable` and `Decodable` traits matching alloy's signatures. (~75 min)
- [ ] R4R ch7 cross-reference. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — `Header` + scalar encoding**

- [ ] **Build**: `crates/eth-rlp/src/header.rs` — Header { list, payload_length }. Test against ethereumjs fixtures. (~60 min)
- [ ] **Build**: `crates/eth-rlp/src/encodable.rs` — impls for u8..u64, U256, bool, slices, Vec, String, Address, B256,
  Bytes. (~75 min)
- [ ] R4R ch9-11. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — List encoding + `Vec<T>` + `length_of_length`**

- [ ] **Build**: Encodable for `Vec<T: Encodable>`, Option<T>, tuples, arrays. length_of_length helper. (~75 min)
- [ ] Nested list test: `Vec<Vec<u64>>` matches Geth's RLP byte-for-byte. (~30 min)
- [ ] Buffer-size-class optimization: pre-size BytesMut. (~30 min)
- [ ] R4R ch12. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Alloy code tour (compare not copy)**

- [ ] Clone alloy-rs/alloy. Read alloy-primitives source AND DIFF against your eth-primitives. Note 5 divergences. (~60 min)
- [ ] Read alloy-rlp source — confirm trait signatures match. (~60 min)
- [ ] Commit notes + diff log. (~10 min)
- [ ] Commit + log (~10 min)

**Friday — `RlpEncodable` / `RlpDecodable` derive macros**

- [ ] **Build**: extend `crates/eth-primitives-derive/` with `#[derive(RlpEncodable, RlpDecodable)]`. Mirror
  alloy-rlp-derive API. (~75 min)
- [ ] Test on 5-field struct — bytes match alloy's derive output. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — `etherscanlite` CLI**

- [ ] **Build**: `crates/etherscanlite/` — CLI fetching balance/nonce/last-5-tx via alloy-provider, parsed into your
  types. ~500 LOC. (~75 min)
- [ ] First Alloy issue scan. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] Tag `eth-rlp v0.1.0`. (~5 min)

---

### Week 6 — `eth-consensus` core + [NEW] `time` crate v0.1

**Mirror target**: `alloy-consensus` + bespoke `time` crate that becomes the universal time substrate.

**Monday — Yellow Paper §4 + `Header`**

- [ ] ME ch3, ch4. Yellow Paper §4. (~75 min)
- [ ] Run reth on Sepolia. (~30 min)
- [ ] **Build**: `crates/eth-consensus/src/header.rs` — Header mirroring alloy_consensus::Header (all fields incl.
  requests_hash). `#[derive(RlpEncodable, RlpDecodable)]`. (~150 min)
- [ ] Test: encode mainnet block 1's header → bytes match `cast block 1 --raw`. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — Tx types + `Transaction` trait**

- [ ] ME ch5-6. Yellow §6. (~60 min)
- [ ] **Build**: TxLegacy, TxEip1559, TxEip4844. (~75 min)
- [ ] **Build**: `Transaction` trait matching alloy. (~75 min)
- [ ] Sign each tx type via alloy-signer; verify recovery. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — EIP-1559 + EIP-4844 fee math**

- [ ] Read EIP-1559 + EIP-4844 specs. (~60 min)
- [ ] **Build**: `crates/eth-consensus/src/eip1559.rs` — calc_next_block_base_fee. (~150 min)
- [ ] **Build**: `crates/eth-consensus/src/eip4844.rs` — calc_excess_blob_gas, calc_blob_fee. (~150 min)
- [ ] Commit + log (~10 min)

**Thursday — [NEW] `time` crate v0.1 scaffold**

- [ ] **Build**: `crates/time/Cargo.toml` workspace member. No deps except `core` and feature-gated `chrono` for Display
  only. (~20 min)
- [ ] **Build**: `crates/time/src/monotonic.rs` — `Monotonic(u64)` newtype over
  `std::time::Instant::elapsed_since_anchor`. Anchor set once at process start. `now() -> Monotonic` is the universal
  monotonic source for the entire workspace from this week forward. (~75 min)
- [ ] **Build**: `crates/time/src/lamport.rs` — `LamportClock { counter: AtomicU64 }`. `tick() -> u64` increments +
  returns. `observe(other: u64)` sets counter to max(self+1, other+1). (~75 min)
- [ ] **Build**: `crates/time/src/hlc.rs` — `HybridLogicalClock { wall: AtomicU64, logical: AtomicU64 }` stub with
  `now() -> Hlc { ms_since_epoch, logical }`. Full implementation comes when distributed tx (W42) needs cross-node
  ordering. (~105 min)
- [ ] **Build**: `crates/time/src/hw.rs` — `HardwareTimestamp` trait stub. Default impl returns Monotonic::now(). PTP
  and TSC variants land later. (~45 min)
- [ ] Test: monotonic always non-decreasing; Lamport gives total order under concurrent thread spawn (1000 threads × 100
  ticks each); HLC stub returns same value within 1 ms. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — `time` crate v0.1 ship + Alloy issue hunt**

- [ ] **Build**: `crates/time/src/lib.rs` — re-exports + crate-level docs. Document the contract that `bufpool`, `wal`,
  `txn`, `matching-engine`, `ledger`, `messaging-aeron`, `consensus-raft` will all share this single time substrate. (~150 min)
- [ ] Tag `time v0.1.0`. (~5 min)
- [ ] Browse alloy issues. Prefer alloy-consensus, alloy-eips, alloy-rlp. (~30 min)
- [ ] Read CONTRIBUTING.md + 5 recently merged PRs. (~45 min)
- [ ] Pick one, claim. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — First Alloy PR work + `Signed<T>`**

- [ ] Fork, branch, implement. (~120 min)
- [ ] cargo fmt, clippy, nextest. Open PR. (~15 min)
- [ ] **Build**: `crates/eth-consensus/src/signed.rs` — `Signed<T> { tx, signature, hash: OnceLock<B256> }`. (~150 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] Inheritance check: `time` now exists; mark each future Layer-1+ crate as "must depend on `time`." (~15 min)

---

### Week 7 — `eth-consensus`: EIP-7702, EIP-7685, EOF + more PRs

**Crate extended**: `eth-consensus` v0.1 → v0.2.

**Monday — PR #1 review iteration + EIP-2930 access list**

- [ ] Address Alloy PR #1 review. (~60 min)
- [ ] **Build**: `crates/eth-consensus/src/eip2930.rs` — AccessList. Wire into TxEip1559 + TxEip4844. (~150 min)
- [ ] Commit + log (~10 min)

**Tuesday — EIP-7702 `Authorization` + `TxEip7702`**

- [ ] Read EIP-7702 + EIP-7685 specs end-to-end. (~60 min)
- [ ] **Build**: `crates/eth-consensus/src/eip7702.rs` — Authorization, SignedAuthorization, recover_authority. (~150 min)
- [ ] **Build**: TxEip7702 with authorization_list. (~75 min)
- [ ] Commit + log (~10 min)

**Wednesday — EIP-7685 + EOF skeleton**

- [ ] Read EOF EIPs: 3540, 3670, 4200, 4750. (~60 min)
- [ ] **Build**: `crates/eth-consensus/src/eip7685.rs` — Request enum + requests_root. (~150 min)
- [ ] **Build**: `crates/eth-consensus/src/bytecode.rs` — Bytecode enum. EOF parser skeleton. (~150 min)
- [ ] Commit + log (~10 min)

**Thursday — `TxEnvelope` + Second Alloy PR**

- [ ] **Build**: `crates/eth-consensus/src/envelope.rs` — TxEnvelope enum dispatching across all tx types. (~150 min)
- [ ] Pick + implement second Alloy PR. (~150 min)
- [ ] Commit + log (~10 min)

**Friday — Third Alloy PR (medium)**

- [ ] Substantive PR — prefer alloy-consensus or alloy-eips. (~150 min)
- [ ] Commit + log (~10 min)

**Saturday — PR #3 submitted + Foundry intro**

- [ ] Submit PR #3. (~30 min)
- [ ] Clone foundry-rs/foundry, browse forge + cast. (~30 min)
- [ ] Commit notes. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 8 — Foundry PR + revm familiarization + `eth-consensus` Receipt/Log

**Crate extended**: `eth-consensus` v0.2 → v0.3.

**Monday — Foundry issue hunt + claim**

- [ ] Browse Foundry issues, pick good first. Prefer `cast`. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — First Foundry PR**

- [ ] Implement + submit. (~120 min)
- [ ] Commit + log (~10 min)

**Wednesday — revm overview (read with `exec-vm` Phase 4 in mind)**

- [ ] Clone bluealloy/revm, read README + arch doc. (~30 min)
- [ ] Cross-reference revm-primitives::Database against your eth-storage-cache::StateCache. Adjust StateCache if needed. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — revm-interpreter + `Receipt` build**

- [ ] Read revm-primitives, compare with eth-primitives. (~45 min)
- [ ] Read revm-interpreter — opcode dispatch, gas. Trace ADD end-to-end. (~90 min)
- [ ] **Build**: `crates/eth-consensus/src/receipt.rs` — Receipt + ReceiptEnvelope. RLP derive. (~150 min)
- [ ] Commit + log (~10 min)

**Friday — ME ch13 + `Log` + `Bloom`**

- [ ] ME ch13 full. Walk evm.codes top 20 opcodes. (~60 min)
- [ ] **Build**: `crates/eth-consensus/src/log.rs` — Log + bloom_filter(logs). Note: the Bloom filter here is a
  fixed-size 2048-bit Ethereum bloom. The general `bloom` crate (classic + counting + scalable) ships W34. (~150 min)
- [ ] Commit notes. (~10 min)

**Saturday — PR cleanup + tag `eth-consensus v0.3.0`**

- [ ] Address all reviewer feedback. (~60 min)
- [ ] Tag `eth-consensus v0.3.0`. (~5 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 2 review**

- [ ] Target check: 3+ Alloy PRs, 1+ Foundry PR. (~15 min)

---

## Month 3: `exec-vm` + `eth-trie` seeds (Weeks 9-12)

### Week 9 — `exec-vm` Phase-1 seed

**Mirror target**: revm-interpreter subset.
**Crate created**: `crates/exec-vm/`.

**Monday — `Stack` + arithmetic opcodes**

- [ ] **Build**: `crates/exec-vm/src/interpreter/stack.rs` — 1024-deep Stack mirroring revm_interpreter::Stack. (~105 min)
- [ ] **Build**: `crates/exec-vm/src/instructions/arithmetic.rs` — ADD, SUB, MUL, DIV, MOD. 3 gas/op. (~75 min)
- [ ] **Build**: `crates/exec-vm/src/interpreter/mod.rs` — Interpreter skeleton + step() dispatcher. (~105 min)
- [ ] Commit + log (~10 min)

**Tuesday — `SharedMemory` + control flow**

- [ ] **Build**: `crates/exec-vm/src/interpreter/memory.rs` — SharedMemory. mload/mstore/mstore8/resize with quadratic
  gas. (~105 min)
- [ ] **Build**: `crates/exec-vm/src/instructions/control.rs` — JUMP, JUMPI, JUMPDEST, PC, STOP, INVALID. (~75 min)
- [ ] **Build**: `crates/exec-vm/src/instructions/comparison.rs` — LT, GT, SLT, SGT, EQ, ISZERO, AND, OR, XOR, NOT. (~75 min)
- [ ] Commit + log (~10 min)

**Wednesday — `Gas` + SSTORE/SLOAD against `StateCache`**

- [ ] **Build**: `crates/exec-vm/src/interpreter/gas.rs` — Gas { limit, remaining, refunded }. (~105 min)
- [ ] **Build**: `crates/exec-vm/src/instructions/host.rs` — SSTORE, SLOAD, BALANCE, EXTCODESIZE. `Host` trait
  delegating to eth_storage_cache::StateCache. (~75 min)
- [ ] 15-20 opcodes total. Test against hand-rolled bytecode. (~30 min)
- [ ] cargo test -p exec-vm. Tag `exec-vm v0.0.1`. (~15 min)
- [ ] Commit + log (~10 min)

**Thursday — First revm PR**

- [ ] Browse revm issues, pick good first, implement, submit. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — `eth-rlp` extension: typed envelopes**

- [ ] **Extend**: `crates/eth-consensus/src/envelope.rs` — implement RLP for TxEnvelope with leading type byte per
  EIP-2718. Test against mainnet typed-tx test vectors. (~60 min)
- [ ] **Extend**: same for ReceiptEnvelope. (~60 min)
- [ ] Diff against alloy-eips::eip2718. (~45 min)
- [ ] Commit + log (~10 min)

**Saturday — More PRs (Alloy/revm)**

- [ ] Whichever is unblocked. Prefer revm now that exec-vm is bootstrapped. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 10 — `eth-trie` Phase-1 seed

**Mirror target**: `alloy-trie` subset.
**Crate created**: `crates/eth-trie/`.

**Monday — MPT theory + `Nibbles`**

- [ ] ethereum.org MPT docs + 2-3 blog explanations. (~30 min)
- [ ] Draw extension/branch/leaf/hash node diagrams. (~30 min)
- [ ] **Build**: `crates/eth-trie/src/nibbles.rs` — `Nibbles(SmallVec<[u8; 64]>)`. Hex-prefix encoding. (~105 min)
- [ ] Commit + log (~10 min)

**Tuesday — `Node` enum + insert/get**

- [ ] **Build**: `crates/eth-trie/src/node.rs` — Node enum (Empty, Leaf, Extension, Branch). (~105 min)
- [ ] **Build**: `crates/eth-trie/src/storage.rs` — `TrieStorage` trait. Initial impl `MemoryStorage`. (~105 min)
- [ ] Insert + get on trie. Test on `[("do","verb"),("dog","puppy"),("doge","coin")]`. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — `HashBuilder` + root hash**

- [ ] **Build**: `crates/eth-trie/src/hash_builder.rs` — HashBuilder. Stream-builds root via keccak256 of RLP-encoded
  nodes. (~105 min)
- [ ] Test against EIP-1186 vectors + alloy-trie fixtures. (~30 min)
- [ ] Tag `eth-trie v0.0.1`. (~5 min)
- [ ] Commit + log (~10 min)

**Thursday — Second revm PR**

- [ ] Pick, implement, submit. (~120 min)
- [ ] Commit + log (~10 min)

**Friday — Reth passive exposure**

- [ ] Clone paradigmxyz/reth. cargo build --release. (~30 min)
- [ ] Browse `reth/crates/trie`. Identify HashBuilder, TrieWalker, HashedPostState, TrieUpdates. (~45 min)
- [ ] Read 5 recently merged trie/storage PRs for style. (~45 min)
- [ ] Commit notes. (~10 min)

**Saturday — `peer-keepalive` state machine on `eth-network-codec`**

- [ ] Build peer-keepalive ping/pong oscillator inside eth-network-codec. (~60 min)
- [ ] Property tests with proptest. (~45 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 11 — Type-state + `HashedPostState` + [NEW] `backpressure` crate v0.1

**[NEW] crate created**: `crates/backpressure/` — extraction of `eth-network-codec`'s BackpressureStrategy enum into a
free-standing crate, then re-exported.

**Monday — Type-state pattern applied to `eth-network-codec`**

- [ ] Type-state, sealed trait, extension trait reading. (~30 min)
- [ ] **Refactor**: `crates/eth-network-codec/src/connection.rs` — `Connection<S>` with phantom states
  Disconnected/Handshaking/Established. (~60 min)
- [ ] Commit + log (~10 min)

**Tuesday — Erigon staged sync (read with your crates as the substrate)**

- [ ] Read Erigon staged sync design doc. (~30 min)
- [ ] Map: headers → bodies → senders → execution → hashing → merkle. For each stage, name the eth-* crate that feeds
  it. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — `HashedPostState` + `TrieUpdates`**

- [ ] Browse reth/crates/trie source. (~45 min)
- [ ] **Build**: `crates/eth-trie/src/hashed_state.rs` — HashedPostState mirroring reth_trie. (~105 min)
- [ ] **Build**: TrieUpdates struct. (~105 min)
- [ ] Commit + log (~10 min)

**Thursday — [NEW] `backpressure` crate v0.1 extraction + `concurrent::bounded` MPMC build**

- [ ] **Build**: `crates/backpressure/Cargo.toml` workspace member. Depends on `time` + `concurrent`. (~20 min)
- [ ] **Build**: `crates/backpressure/src/strategy.rs` — move `BackpressureStrategy` enum from eth-network-codec.
  Variants: `DropOldest`, `DropNewest`, `Block`, `BlockWithTimeout(Monotonic)`, `Spill { dst: Box<dyn SpillSink> }`. (~45 min)
- [ ] **Build**: `crates/backpressure/src/sink.rs` — `SpillSink` trait: where to dump on overflow (used by
  messaging-aeron W76 to spill to disk via wal). (~105 min)
- [ ] **Build**: `crates/backpressure/src/credit.rs` —
  `CreditFlowControl { credits: AtomicU64, low_water: u64, high_water: u64 }` — used by messaging-aeron flow control AND
  matching-engine inbound queue. (~150 min)
- [ ] **Build**: `crates/concurrent/src/channel/bounded.rs` — Vyukov-style MPMC bounded channel. Power-of-2 capacity;
  per-slot `seq: AtomicUsize`; `head` and `tail` cursors each in `CachePadded` (W4 Tue). Producers CAS-then-write the
  slot; consumers CAS-then-read. (~75 min)
  - **Expect to hit #1**: you'll accept any capacity. Modulo via `%` produces aliasing when capacity isn't
    power-of-2. Loom fails with "slot N holds two values" within seconds.
    **Fix**: assert power-of-2 in `new()`; use `idx & (cap - 1)` mask, never `%`.
  - **Expect to hit #2**: producer writes the slot data **before** bumping the seq counter, but uses `Relaxed` on
    the counter store. On Apple Silicon (weakly ordered), the consumer observes the counter bump *before* the data
    write and reads uninit. x86 hides this in 100k runs — that's the trap.
    **Fix**: store seq with `Release`; consumer loads seq with `Acquire`. Test under loom (`Acquire`-`Release`
    model, not `SeqCst`).
  - **Expect to hit #3**: two producers observe the same `tail`, both CAS. Loser increments seq and writes to the
    next slot — but it reused the *stale* tail index. Data ends up in the wrong slot.
    **Fix**: every CAS failure restarts from a fresh load. Never reuse the stale observation.
  - **Expect to hit #4**: `recv()` on empty without `Parker`. You'll either burn CPU spinning forever, or add a
    `Mutex<Condvar>` and rediscover the lost-wakeup bug at scale.
    **Fix**: the `Parker` you built in W4 Tue plugs in here. Producer signals via `Unparker` after successful send;
    consumer parks after observing empty.
  - **Muscle**: per-slot sequence-counter dance is the universal lock-free queue. **Reapplies at**: WAL group-commit
    (W26, but unbounded → SegQueue), matching-engine command bus (W63), messaging-aeron term rotation (W77, where
    per-term seq plays the same role).
- [ ] `backpressure::Block` and `backpressure::BlockWithTimeout` strategies wire to `concurrent::bounded::Receiver`.
  The MPMC is the substrate; the strategy is the policy. (~30 min)
- [ ] Tag `backpressure v0.1.0`. (~5 min)
- [ ] Re-export from eth-network-codec, deprecate the local copy with a TODO to remove next minor. (~30 min)
- [ ] Third revm PR (medium difficulty) — pick substantive issue, implement. (~180 min)
- [ ] Commit + log (~10 min)

**Friday — Twitter + GitHub presence warm-up**

- [ ] First thoughtful technical reply on a reth/paradigm tweet. (~30 min)
- [ ] Star key repos. Follow 20 more Ethereum infra engineers. (~10 min)
- [ ] Commit notes. (~10 min)

**Saturday — Outstanding PR cleanup + tag**

- [ ] Address all reviewer feedback. (~60 min)
- [ ] Tag `eth-trie v0.1.0`. `eth-network-codec v0.2.0` (now depends on `backpressure`). (~5 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 12 — Phase 1 close + [NEW] `bufpool` scaffold + Phase 2 prep

**[NEW] crate scaffolded**: `crates/bufpool/` — empty lib.rs this week, fully built W14.

**Monday — MDBX overview**

- [ ] Read libmdbx high-level README + libmdbx-rs crate skim. (~60 min)
- [ ] Sketch layering: StateCache → MdbxStateCache (Phase 3) → MDBX env. Bufpool sits between the Page primitive (W2)
  and MDBX's mmap. (~20 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Reth architecture talk + consensus background**

- [ ] Watch gakonst reth architecture talk on YouTube. (~75 min)
- [ ] Mastering Ethereum consensus chapter; The Merge high level. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Final Alloy/revm PR for Phase 1**

- [ ] Push one more PR over the finish line. (~150 min)
- [ ] Commit + log (~10 min)

**Thursday — Maintainer tracker**

- [ ] Note which maintainers reviewed PRs. (~20 min)
- [ ] Identify mentor candidate (likely Matthias Seitz). (~20 min)
- [ ] Commit notes. (~10 min)

**Friday — Reth Telegram + Discord + [NEW] `bufpool` scaffold**

- [ ] Join reth Telegram, observe (don't post yet). (~15 min)
- [ ] **Build**: `crates/bufpool/Cargo.toml` workspace member. Depends on `time`, `eth-storage-cache` (for the Page
  type). (~20 min)
- [ ] **Build**: `crates/bufpool/src/lib.rs` — empty `BufferPool<P: PageRef>` skeleton + module headers for `lru_k.rs`,
  `pin.rs`, `dirty.rs`. cargo build --workspace green with 11 crates. (~15 min)
- [ ] Commit notes. (~10 min)

**Saturday — Phase 1 review**

- [ ] Verify shipped crates: eth-primitives v0.2, eth-rlp v0.1, eth-storage-cache v0.1, eth-network-codec v0.2,
  eth-consensus v0.3, exec-vm v0.0.1, eth-trie v0.1, eth-primitives-derive v0.1, **time v0.1**, **backpressure v0.1**, *
  *bufpool scaffold**. (~20 min)
- [ ] Verify: 3-5 Alloy PRs, 2-3 revm PRs, 1-2 Foundry PRs. (~30 min)
- [ ] cargo test --workspace green; clippy clean; miri clean on eth-primitives. (~15 min)
- [ ] Phase 1 reflection in `progress.md`. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — End Phase 1 ritual**

- [ ] Full assessment. (~30 min)
- [ ] Update North Star M3 metrics. (~15 min)
- [ ] Phase 2 starts tomorrow. (~30 min)

---

# PHASE 2: ETHEREUM FOUNDATION + ECOSYSTEM PRs (Month 4-6)

> Tempo enters this phase as reading/orientation only. First touch W16 Tue. No Tempo PRs in Phase 2.
> [NEW] `bufpool` v1.0 ships W14 Sat — the first Layer-1 primitive at production version. From W14, all Page traffic in
> the workspace flows through `bufpool`.

## Month 4: Ethereum Protocol + Alloy PRs

### Week 13 — Ethereum fundamentals + `eth-consensus` deepening

**Monday — ME ch3 + `SealedHeader` finalize**

- [ ] ME ch3 (Clients) + ethereum.org intro skim. (~60 min)
- [ ] Run reth on Sepolia, observe sync logs. (~30 min)
- [ ] **Build**: `crates/eth-consensus/src/sealed.rs` — SealedHeader mirroring reth_primitives. hash_ref via keccak256(
  rlp(header)). (~150 min)
- [ ] Test: hash matches mainnet block hashes via alloy-provider. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — ME ch4 + signer recovery**

- [ ] ME ch4 (Cryptography). keccak256, secp256k1. (~60 min)
- [ ] **Build**: `crates/eth-consensus/src/recovery.rs` — recover_signer using k256 directly. (~150 min)
- [ ] **Build**: `Signed<T>::recover_signer()`. (~75 min)
- [ ] Commit + log (~10 min)

**Wednesday — ME ch5-6 + `Block` + `Body`**

- [ ] ME ch5-6. (~60 min)
- [ ] **Build**: `crates/eth-consensus/src/block.rs` — Block, BlockBody, SealedBlock. (~150 min)
- [ ] Sign each tx type via your signature_hash() then alloy-signer; assert recovered address matches. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — ME ch7 + `encode_tx` round-trip**

- [ ] ME ch7 (Smart Contracts Solidity). (~60 min)
- [ ] Deploy simple contract on Sepolia via Foundry. (~30 min)
- [ ] **Build**: `crates/eth-consensus/src/encode_tx.rs` — encode_signed_tx. Send via eth_sendRawTransaction against
  Sepolia. (~150 min)
- [ ] Commit + log (~10 min)

**Friday — Yellow Paper §4 + `Account` + `StorageEntry`**

- [ ] Yellow Paper §4. (~75 min)
- [ ] **Build**: `crates/eth-consensus/src/account.rs` — Account (RLP on-disk form). From/To conversions to
  eth-storage-cache's in-memory Account. (~150 min)
- [ ] Draw state diagrams. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Yellow Paper §6 + intrinsic gas calculator**

- [ ] Yellow Paper §6. (~75 min)
- [ ] **Build**: `crates/eth-consensus/src/gas.rs` — intrinsic_gas. Test against revm's validate_initial_tx_gas. (~150 min)
- [ ] Tag `eth-consensus v0.4.0`. (~5 min)
- [ ] Commit notes. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 14 — EIP deep dives + [NEW] `bufpool` v1.0 + medium Alloy PRs

**Crate created**: `crates/eth-eips/`. **[NEW] crate version**: `bufpool` v1.0 ships Saturday.

**Monday — EIP-1559 deep + extract `eth-eips/eip1559`**

- [ ] Re-read EIP-1559 + Paradigm analysis. (~45 min)
- [ ] **Refactor**: move to crates/eth-eips/src/eip1559.rs. Add BaseFeeParams for Optimism/Base chain-specific
  overrides. (~60 min)
- [ ] Test against mainnet, Optimism, Base genesis. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — EIP-4844 (blobs) deep + KZG**

- [ ] Read EIP-4844 + Proto-Danksharding roadmap. (~60 min)
- [ ] **Refactor**: move blob fee math to crates/eth-eips/src/eip4844.rs. Add BlobTransactionSidecar. KZG placeholder. (~60 min)
- [ ] Commit notes. (~10 min)

**Wednesday — [NEW] `bufpool` build: LRU-K + pin/unpin**

- [ ] **Build**: `crates/bufpool/src/lru_k.rs` — LRU-K (K=2) eviction policy. Tracks K-most-recent access timestamps via
  `time::Monotonic`. Frame with longest "backward K-distance" is evicted. (~105 min)
- [ ] **Build**: `crates/bufpool/src/pin.rs` — `PinnedPage<'a>` RAII guard. `pool.pin(page_id) -> PinnedPage<'a>`
  increments pin count; drop decrements. Pinned pages are evict-immune. (~105 min)
- [ ] **Build**: `crates/bufpool/src/dirty.rs` — `DirtyPageTracker` with a `HashSet<PageId>` and a counter. Marks pages
  for write-back. Wal (W26) drains the dirty set on group commit. (~105 min)
- [ ] **Build**: `crates/bufpool/src/lib.rs` — `BufferPool<P: PageProvider>` API: `new(capacity, k=2)`,
  `pin(id) -> PinnedPage`, `mark_dirty(id)`, `flush_all(&self, sink: &mut dyn WriteBack) -> Result<()>`. (~105 min)
- [ ] Commit + log (~10 min)

**Thursday — Alloy issues scan (target `alloy-eips`) + `bufpool` benchmarks**

- [ ] Browse alloy issues. PREFER alloy-eips. (~30 min)
- [ ] Identify 3-5 candidates, pick one, claim. (~20 min)
- [ ] criterion bench on `bufpool`: 1M ops mixed read/write at hit rates 50/75/90/99%. LRU-K vs naive LRU vs FIFO. Plot. (~60 min)
- [ ] Commit notes + bench numbers. (~10 min)

**Friday — Medium-difficulty Alloy PR work + EIP-7702 in `eth-eips`**

- [ ] Substantive change in alloy-eips or alloy-consensus. (~30 min)
- [ ] Re-read EIP-7702. (~45 min)
- [ ] **Refactor**: move Authorization + SignedAuthorization from eth-consensus to eth-eips. (~60 min)
- [ ] Tag `eth-eips v0.1.0`. (~5 min)
- [ ] Commit + log (~10 min)

**Saturday — Alloy PR submitted + tag `bufpool v1.0`**

- [ ] Finish Alloy PR. Open with motivation referencing eth-eips design notes. (~20 min)
- [ ] **Refactor**: `eth-storage-cache::Page` no longer owns its memory; it borrows from `bufpool::PinnedPage`. This is
  the first "primitive once" wire-up moment — Page memory is now sourced from a single pool that storage-trie (W31),
  wal (W26), and mini-db (W95) will all share. (~60 min)
- [ ] Tag `bufpool v1.0.0`. Update `eth-storage-cache` to v0.2.0 depending on bufpool. (~5 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 15 — EIP-7685 + EOF parser in `exec-vm` + more PRs

**Monday — Respond to Alloy PR reviews**

- [ ] Address feedback; iterate. (~60 min)
- [ ] Commit + log (~10 min)

**Tuesday — EIP-7685 finalize in `eth-eips`**

- [ ] Re-read EIP-7685. (~45 min)
- [ ] **Refactor**: move Requests from eth-consensus to crates/eth-eips/src/eip7685.rs. (~60 min)
- [ ] Tag `eth-eips v0.2.0`. (~5 min)
- [ ] Commit notes. (~10 min)

**Wednesday — EOF parser deepening in `exec-vm`**

- [ ] Re-read EIP-3540, 3670, 4200, 4750. (~45 min)
- [ ] **Build**: `crates/exec-vm/src/eof/parser.rs` — full EOF container parser. (~75 min)
- [ ] **Build**: `crates/exec-vm/src/eof/validate.rs` — EIP-3670 code validation. (~75 min)
- [ ] Test against revm's EOF vectors. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Second Alloy PR**

- [ ] Pick next candidate, implement. (~120 min)
- [ ] Commit + log (~10 min)

**Friday — Third Alloy PR work (medium)**

- [ ] Substantive contribution. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Third Alloy PR complete**

- [ ] Finish, submit. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 16 — Alloy/Foundry PRs + `eth-rpc-types` extraction

**Crate created**: `crates/eth-rpc-types/`.

**Monday — `eth-rpc-types` + 4th Alloy PR**

- [ ] **Build**: `crates/eth-rpc-types/src/block.rs` — RPC Block, Transaction. (~75 min)
- [ ] Pick + implement 4th Alloy PR. (~150 min)
- [ ] Commit + log (~10 min)

**Tuesday — Foundry codebase intro**

- [ ] Clone foundry-rs/foundry. Read Foundry Book briefly. (~30 min)
- [ ] Browse forge crate source. (~45 min)
- [ ] [Tempo] 15 min at end of day: open `github.com/tempoxyz/tempo-foundry` in browser. Read top of README. Note: fork
  of Foundry adding TempoEvm extending revm, plus `--tempo.fee-token` support. Close tab. No PR, no commit. Awareness
  only. (~15 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Foundry cast + `eth-rpc-types/filter`**

- [ ] Read cast crate source. (~60 min)
- [ ] **Build**: `crates/eth-rpc-types/src/filter.rs` — Filter, FilterBlockOption, Topic. (~75 min)
- [ ] Commit notes. (~10 min)

**Thursday — First Foundry PR**

- [ ] Browse Foundry issues, pick good first. Prefer cast. (~30 min)
- [ ] Implement. (~120 min)
- [ ] [Tempo] 30 min at end of day: open `tempoxyz/tempo` README + `docs.tempo.xyz` landing page. Create
  `notes/tempo_orientation.md` with one paragraph: "What Tempo is, why it's relevant to my Reth bet." (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Foundry PR complete + Alloy review responses**

- [ ] Finish Foundry PR. Address Alloy review feedback. (~60 min)
- [ ] Commit + log (~10 min)

**Saturday — `eth-rpc-types/transaction_request` + 5th Alloy PR**

- [ ] **Build**: `crates/eth-rpc-types/src/transaction_request.rs` — TransactionRequest. (~75 min)
- [ ] Tag `eth-rpc-types v0.1.0`. (~5 min)
- [ ] Submit 5th Alloy PR or polish existing. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 4 review**

- [ ] Target check: 5+ Alloy PRs opened, some merged. (~15 min)

---

## Month 5: EVM Deep Dive + revm PRs

### Week 17 — `exec-vm` expansion (DOUBLES opcode coverage)

**Crate extended**: exec-vm v0.0.1 → v0.1.0.

**Monday — ME ch13 part 1 + `Env` types**

- [ ] ME ch13 first half. Memorize top 20 opcodes. (~60 min)
- [ ] **Build**: `crates/exec-vm/src/env.rs` — Env, BlockEnv, TxEnv, CfgEnv mirroring revm_primitives::Env. (~105 min)
- [ ] **Build**: `From<&TxEnvelope> for TxEnv`, `From<&Header> for BlockEnv`. (~60 min)
- [ ] Commit notes. (~10 min)

**Tuesday — ME ch13 part 2 + `instructions/system.rs`**

- [ ] ME ch13 second half. (~60 min)
- [ ] **Build**: `crates/exec-vm/src/instructions/system.rs` — RETURN, REVERT, INVALID, SELFDESTRUCT (skeleton — full
  impl Phase 4 needs journal). (~45 min)
- [ ] Commit notes. (~10 min)

**Wednesday — evm.codes deep + `instructions/stack.rs`**

- [ ] Walk every opcode on evm.codes. (~30 min)
- [ ] **Build**: `crates/exec-vm/src/instructions/stack.rs` — PUSH0..PUSH32, DUP1..DUP16, SWAP1..SWAP16, POP. All 96. (~75 min)
- [ ] Manual trace simple bytecode through interpreter. (~30 min)
- [ ] [Tempo] 30 min at end of day: read `tempoxyz/tempo` repo top-level Cargo.toml. Note which reth-* and revm-* crates
  it pins. Confirm: every Reth crate you've been mirroring is also depended on by Tempo. Add 3-line note to
  tempo_orientation.md. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — `instructions/contract.rs` (CALL family)**

- [ ] **Build**: `crates/exec-vm/src/instructions/contract.rs` — CALL, CALLCODE, DELEGATECALL, STATICCALL. EIP-150
  63/64ths gas. (~75 min)
- [ ] **Build**: `crates/exec-vm/src/instructions/create.rs` — CREATE, CREATE2 with init code analysis. (~75 min)
- [ ] Test: simple call-with-return via two hand-rolled programs. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — `instructions/host.rs` extension against `StateCache`**

- [ ] **Build**: extend host.rs with BALANCE, EXTCODESIZE, EXTCODEHASH, EXTCODECOPY, BLOCKHASH, COINBASE, TIMESTAMP,
  NUMBER, DIFFICULTY/PREVRANDAO, GASLIMIT, CHAINID, SELFBALANCE, BASEFEE, BLOBHASH, BLOBBASEFEE. (~60 min)
- [ ] All routed through Host trait → eth-storage-cache::StateCache. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — `instructions/log.rs` + ethereum-tests subset green**

- [ ] **Build**: `crates/exec-vm/src/instructions/log.rs` — LOG0..LOG4. (~75 min)
- [ ] Total opcode count: 60+. Pass GeneralStateTests/stArithmetic + stMemoryTest subsets. (~30 min)
- [ ] Tag `exec-vm v0.1.0`. README documents opcode coverage matrix. (~5 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 18 — revm deep-read (diffing against your `exec-vm`)

**Monday — revm overview + diff to `exec-vm`**

- [ ] Re-clone bluealloy/revm latest. (~30 min)
- [ ] Read README, arch doc fresh — with 60+ opcodes implemented. (~30 min)
- [ ] **Diff log**: for each revm crate, name 3 design choices that differ. Save to `notes/08_revm_diff.md`. (~25 min)
- [ ] Commit notes. (~10 min)

**Tuesday — revm-primitives + `Database` trait alignment**

- [ ] Read revm-primitives source. (~60 min)
- [ ] Confirm your StateCache trait can be a Database for unmodified revm. Adjust if not. (~20 min)
- [ ] Commit notes. (~10 min)

**Wednesday — revm-interpreter dispatch**

- [ ] Read revm-interpreter source. Study opcode dispatch. (~60 min)
- [ ] Identify revm perf optimizations your exec-vm lacks. Add to `EXEC_VM_PERF_BACKLOG.md`. (~20 min)
- [ ] Commit notes. (~10 min)

**Thursday — revm hot path + ADD trace**

- [ ] Trace ADD end-to-end through revm AND your exec-vm. Compare overhead. (~60 min)
- [ ] Commit + log (~10 min)

**Friday — revm handler + precompile reading**

- [ ] Read revm Handler trait and precompile crate. (~60 min)
- [ ] Sketch where Handler plugs into your exec-vm. Phase 4 W53 adds it. (~20 min)
- [ ] Commit notes. (~10 min)

**Saturday — First revm PR informed by the diff**

- [ ] Browse revm issues. Pick something where your exec-vm gives informed perspective. (~30 min)
- [ ] Implement, submit. (~120 min)
- [ ] [Tempo] 45 min at end of day: while revm is fresh, browse `tempoxyz/tempo`'s evm crate. Note how TempoEvm wraps
  revm's Evm. Sketch in `notes/tempo_diff.md` the 3 most obvious extension points (precompile registry, tx handler, fee
  accounting). No code. (~45 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 19 — revm PR velocity + `exec-vm` precompile skeleton

**Monday — Second revm PR**

- [ ] Pick and implement. (~120 min)
- [ ] Commit + log (~10 min)

**Tuesday — revm PR review response + `exec-vm` precompile registry**

- [ ] Address reviewer feedback. (~60 min)
- [ ] **Build**: `crates/exec-vm/src/precompile/mod.rs` — Precompile trait, PrecompileRegistry. Implement ECRECOVER
  first. (~75 min)
- [ ] Commit + log (~10 min)

**Wednesday — Third revm PR (medium)**

- [ ] Pick medium-difficulty issue, implement. (~120 min)
- [ ] Commit + log (~10 min)

**Thursday — geth core/vm comparison**

- [ ] Read geth's core/vm package. (~45 min)
- [ ] Add geth-specific notes to 08_revm_diff.md. (~30 min)
- [ ] Commit notes. (~10 min)

**Friday — evmone comparison**

- [ ] Read evmone README + architecture. (~30 min)
- [ ] Note C++ optimizations. Add to EXEC_VM_PERF_BACKLOG.md. (~20 min)
- [ ] Commit notes. (~10 min)

**Saturday — Continue revm PRs**

- [ ] Work on outstanding or start new. (~30 min)
- [ ] [Tempo] 30 min at end of day: skim Tempo TIP index on docs.tempo.xyz. Read just titles and one-line summaries.
  List the 5 most execution-relevant TIPs (likely: TIP-20, TIP-1020, TIP-1031, TIP-403, plus one more). (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 20 — `eth-trie` expansion

**Crate extended**: eth-trie v0.1 → v0.2.

**Monday — MPT deeper theory + `BranchNodeCompact`**

- [ ] Re-read ethereum.org MPT docs + 2-3 blog explanations. (~45 min)
- [ ] **Build**: `crates/eth-trie/src/branch_compact.rs` — BranchNodeCompact mirroring reth_trie. (~105 min)
- [ ] Commit notes. (~10 min)

**Tuesday — `TrieStorage` abstraction over `StateCache`**

- [ ] **Refactor**: split TrieStorage into HashedNodeStorage + IntermediateStorage. (~60 min)
- [ ] **Build**: `CachedStorage<C: StateCache>` delegating to eth-storage-cache. (~75 min)
- [ ] Commit + log (~10 min)

**Wednesday — `TrieWalker` cursor**

- [ ] **Build**: `crates/eth-trie/src/walker.rs` — TrieWalker<S: TrieStorage> streaming traversal. (~105 min)
- [ ] Commit + log (~10 min)

**Thursday — `ProofRetainer` + EIP-1186 proofs**

- [ ] **Build**: `crates/eth-trie/src/proof/retainer.rs` — ProofRetainer mirroring alloy_trie. (~105 min)
- [ ] **Build**: `crates/eth-trie/src/proof/verify.rs` — verify_proof. (~105 min)
- [ ] Test against EIP-1186 vectors + captured mainnet eth_getProof response. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — `StateRoot` orchestrator**

- [ ] **Build**: `crates/eth-trie/src/state_root.rs` — StateRoot<S> with compute(). Heart of MerkleStage. (~105 min)
- [ ] Test: reconstruct block 1 mainnet state root from genesis + block 1 changes. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — `StorageRoot` + tag**

- [ ] **Build**: `crates/eth-trie/src/storage_root.rs`. (~105 min)
- [ ] Pass simplest Ethereum trie test vectors end-to-end. (~30 min)
- [ ] Tag `eth-trie v0.2.0`. (~5 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 45 min during ritual: pick ONE TIP (recommended: TIP-1020 since signature verification ties to your W19
  ECRECOVER work). Read end-to-end. Note in tempo_orientation.md how it would plug into exec-vm precompile registry. (~45 min)
- [ ] Update North Star M5 metrics. (~15 min)

---

## Month 6: MPT Understanding + First Maintainer Interactions

### Week 21 — `eth-rlp` extension + maintainer engagement

**Monday — `eth-rlp` extension: trie-friendly encoding**

- [ ] Re-read RLP spec sections relevant to trie nodes. (~45 min)
- [ ] **Build**: `crates/eth-rlp/src/trie.rs` — encode_branch_node, encode_extension_node, encode_leaf_node. (~105 min)
- [ ] **Build**: refactor EipTransactionRlp helper to use eth-rlp helpers consistently. (~75 min)
- [ ] Commit + log (~10 min)

**Tuesday — Reth RLP usage patterns + `eth-rlp` derive enhancements**

- [ ] Read reth's RLP usage patterns + alloy-rlp source freshly. (~60 min)
- [ ] **Extend**: eth-rlp-derive to support `#[rlp(trailing)]`. (~60 min)
- [ ] Tag `eth-rlp v0.2.0`. (~5 min)
- [ ] Commit + log (~10 min)

**Wednesday — Fourth revm PR**

- [ ] Pick + implement. (~120 min)
- [ ] Commit + log (~10 min)

**Thursday — Second Foundry PR**

- [ ] Pick + implement. (~120 min)
- [ ] Commit + log (~10 min)

**Friday — Maintainer engagement**

- [ ] Identify maintainers per area (alloy-eips: gakonst/yash; revm: rakita; reth-trie: rakita/mattsse). (~20 min)
- [ ] Engage thoughtfully in an issue discussion. (~30 min)
- [ ] [Tempo] 15 min at end of day: identify which Tempo maintainers overlap with your Reth tracker. Update Tempo
  maintainer tracker (cross-references for gakonst, rakita, joshieDo). No outreach yet. (~15 min)
- [ ] Commit notes. (~10 min)

**Saturday — Consolidation**

- [ ] Review all open PRs. Close out review comments. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 22 — Staged sync architecture + `eth-stage` trait skeleton

**Crate created**: `crates/eth-stage/`.

**Monday — Erigon staged sync (deeper this time)**

- [ ] Re-read Erigon staged sync doc with implementation eye. (~45 min)
- [ ] Stage concept, unwind, checkpoints. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Reth stages source dive**

- [ ] Browse reth/crates/stages. (~45 min)
- [ ] **Build**: `crates/eth-stage/src/lib.rs` — Stage trait (id, execute, unwind) matching reth shape. (~105 min)
- [ ] **Build**: Pipeline runner with checkpoint persistence via eth-storage-cache::StateCache. (~75 min)
- [ ] Commit + log (~10 min)

**Wednesday — Stage dependency map**

- [ ] Diagram: headers → bodies → senders → execution → hashing → merkle. (~30 min)
- [ ] **Build**: `crates/eth-stage/src/stages/headers.rs` — skeleton HeaderStage. (~105 min)
- [ ] Commit + log (~10 min)

**Thursday — More revm or Alloy PRs**

- [ ] Keep PR velocity. (~150 min)
- [ ] Commit + log (~10 min)

**Friday — Reth Telegram + Discord**

- [ ] Join reth main Telegram. Observe discussion style for 4 weeks before posting. (~15 min)
- [ ] [Tempo] 15 min: also join Tempo's public community channel (Discord/Telegram per `tempoxyz/tempo`
  CONTRIBUTING.md). Observe, post nothing for 4 weeks. (~15 min)
- [ ] Commit notes. (~10 min)

**Saturday — `eth-stage` consolidation + tag**

- [ ] Skeleton stages for senders, execution, hashing, merkle. (~30 min)
- [ ] Tag `eth-stage v0.0.1`. (~5 min)
- [ ] [Tempo] 45 min: read TIP-20 (stablecoin token standard) end-to-end. Note differences from ERC-20. Add to
  tempo_diff.md — fee-token semantics, policy registry hook (TIP-403). (~45 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 23 — Ready up for Phase 3 (`storage-trie` scaffold pre-wiring)

**Monday — Reth storage crate survey + `storage-trie` workspace setup**

- [ ] Browse reth/crates/storage (db, provider, codecs, api). (~30 min)
- [ ] **Build**: `crates/storage-trie/Cargo.toml` workspace member. Empty lib.rs. Declare deps: `bufpool`, `time`,
  `eth-trie`, `eth-storage-cache`. (~20 min)
- [ ] Confirm cargo build --workspace succeeds. (~20 min)
- [ ] Commit notes + scaffold. (~10 min)

**Tuesday — MDBX first look + `Database` trait sketch**

- [ ] Read libmdbx high-level README. (~30 min)
- [ ] **Sketch**: in storage-trie/src/lib.rs, define Database trait shape. (~30 min)
- [ ] [Tempo] 30 min at end of day: read TIP-1031 (consensus context in block header). Matters for Phase 5
  consensus-engine — Tempo's header has extra fields your engine_newPayload handler needs to carry through (gated behind
  feature flag). (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — More Alloy/revm PRs**

- [ ] Keep contribution streak. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Conference research**

- [ ] EthCC Paris 2027 + Devcon 2027 dates. Start budgeting. (~30 min)
- [ ] Commit notes. (~10 min)

**Friday — Relationship review**

- [ ] Update maintainer tracker. Identify target mentor. (~15 min)
- [ ] Commit notes. (~10 min)

**Saturday — Month 6 consolidation**

- [ ] Review all PRs. Check target: 5+ Alloy, 3+ revm, 2+ Foundry. (~30 min)
- [ ] [Tempo] 1 hr: read tempoxyz/tempo's storage-adjacent crates. Note divergence from upstream Reth —
  payment-lane-aware indexing if any. Add to tempo_diff.md. (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 24 — Phase 2 close + Phase 3 prep

**Monday — Mastering Ethereum consensus + `consensus-engine` placeholder crate**

- [ ] ME consensus chapter. The Merge high level. (~30 min)
- [ ] **Build**: `crates/consensus-engine/Cargo.toml` workspace member. Empty lib.rs. (~20 min)
- [ ] cargo build --workspace green with all 14 crates (eth-* + exec-vm + time + backpressure + bufpool + storage-trie
  scaffold + consensus-engine scaffold). (~15 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Reth architecture talk/video**

- [ ] Watch gakonst reth architecture talk + Paradigm Frontiers talk. (~75 min)
- [ ] Map every component to one of YOUR workspace crates. (~30 min)
- [ ] [Tempo] 20 min: `git clone https://github.com/tempoxyz/tempo /tmp/tempo` and `cargo build --release` (runs in
  background). Confirms toolchain works against their pinned reth revision. If build fails, file failure mode in
  `notes/tempo_build_blockers.md` for later. Do NOT debug. (~20 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Phase 3 scope + outline**

- [ ] Read Phase 3 section. (~45 min)
- [ ] Outline approach for Month 7. Note: the `wal` extraction at W26 is a critical new dependency for storage-trie.
  Plan accordingly. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Phase 3 scaffolding + CI**

- [ ] CI in `.github/workflows/ci.yml` running fmt --check, clippy, nextest, miri (weekly). (~60 min)
- [ ] README at workspace root with dependency graph showing all 14 crates. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Final Phase 2 PRs**

- [ ] Wrap outstanding. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Phase 2 review**

- [ ] Full assessment. (~30 min)
- [ ] Verify shipped crates: eth-primitives v0.2, eth-rlp v0.2, eth-storage-cache v0.2 (now bufpool-backed),
  eth-network-codec v0.2, eth-consensus v0.4, eth-eips v0.2, eth-rpc-types v0.1, eth-trie v0.2, eth-stage v0.0.1,
  exec-vm v0.1, eth-primitives-derive v0.1, **time v0.1**, **backpressure v0.1**, **bufpool v1.0**, storage-trie
  scaffold, consensus-engine scaffold. (~20 min)
- [ ] Update progress.md. (~30 min)
- [ ] [Tempo] End-of-phase Tempo metrics check: orientation depth target 1 (should hit), TIPs read target 1 — at 3 (
  TIP-1020 W20, TIP-20 W22, TIP-1031 W23). PRs: 0 (correct). (~10 min)
- [ ] Commit + log (~10 min)

**Sunday — End Phase 2 + Phase 3 prep**

- [ ] Full rest. (~5 min)
- [ ] Phase 3 starts tomorrow. (~30 min)

---

# PHASE 3: STORAGE + TRIE DEEP DIVE + DURABILITY PRIMITIVES (Month 7-12)

**Reth Deliverable**: `storage-trie` v1.0 (W44) — MDBX-backed persistent state DB.
**[NEW] Layer-2/3 Deliverables**: `wal` v0.1 (W26), `recovery` v0.5 (W30), `bloom` v0.1 (W34), `lsm-core` v0.5 (W40),
`txn` v0.5 (W42).

> Tempo Phase 3 budget: 2-3 hrs/wk. Reading + bookmarking PR candidates + Sunday release skims. First Tempo PR scheduled
> W60-62 (Phase 4).

`storage-trie` consumes seed crates from Phase 1-2 AND the new durability primitives shipped this phase: provides
MDBX-backed Database implementing eth-storage-cache::StateCache, TrieStorage impl replacing MemoryStorage (W10),
MerkleStage impl plugging into eth-stage::Stage (W22), wal-backed transaction log (W31+), recovery-driven crash safety (
W36+).

---

## Month 7: MDBX Foundation + [NEW] `wal` Primitive + First Reth Storage PRs

### Week 25 — MDBX documentation deep

**Monday — MDBX overview**

- [ ] Read libmdbx.dqdkfa.ru full overview. mmap-based design. (~45 min)
- [ ] Commit notes (~10 min)

**Tuesday — MDBX internals: B-tree**

- [ ] B-tree structure section. Compare with B+tree. (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — MDBX internals: MVCC**

- [ ] MVCC section. Read tx during write tx. (~45 min)
- [ ] Commit notes (~10 min)

**Thursday — MDBX internals: Durability**

- [ ] WAL / sync modes. Crash recovery. **Critical reading** — informs the W26 `wal` crate design. (~30 min)
- [ ] Commit notes (~10 min)

**Friday — MDBX cursor semantics**

- [ ] Cursor documentation. Range scan. (~30 min)
- [ ] Commit notes (~10 min)

**Saturday — libmdbx-rs source**

- [ ] Clone and read libmdbx-rs. (~45 min)
- [ ] [Tempo] 1 hr at end of day: re-read tempoxyz/tempo's storage-adjacent crates with MDBX knowledge fresh. Note
  divergence from upstream Reth — payment-lane-aware indexing if any. Update tempo_diff.md. (~60 min)
- [ ] Commit notes (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 26 — [NEW] `wal` crate v0.1 + reth storage architecture

**[NEW] crate created**: `crates/wal/`. The first universal durability primitive. Will be inherited by storage-trie (
W31), ledger (W80), matching-engine (W74), consensus-raft (W64+ for snapshot logs), and mini-db (W95).

**Monday — `wal` crate scaffold + segment layout**

- [ ] **Build**: `crates/wal/Cargo.toml` workspace member. Deps: `time`, `bufpool`, `eth-storage-cache` (Page). Optional
  dep on `crc32fast` for checksums. (~20 min)
- [ ] **Build**: `crates/wal/src/segment.rs` — `Segment` is an mmap-backed file with header
  `{ magic, version, segment_id, first_lsn }`, body of length-prefixed records, footer `{ last_lsn, checksum }`. Default
  segment size 64 MiB. Pre-allocated. (~105 min)
- [ ] **Build**: `crates/wal/src/record.rs` — `WalRecord { lsn: u64, kind: RecordKind, payload: Bytes }`. RecordKind
  enum (Insert, Update, Delete, Begin, Commit, Abort, Checkpoint). (~105 min)
- [ ] Test: append 10k records, fsync, re-open, scan from offset 0 — all 10k recovered in order. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — `wal` group commit + [NEW] `concurrent::SegQueue` build**

- [ ] **Build**: `crates/concurrent/src/queue/seg_queue.rs` — `SegQueue<T>` unbounded MPMC, linked segments of N=32
  slots. Head and tail are `CachePadded<AtomicPtr<Segment>>`. Each segment carries a per-slot seq (same Vyukov idea as
  the bounded channel, per-segment). Memory reclamation: simple atomic refcount per segment for now; once `epoch-gc`
  lands W33, swap to `Guard::defer_destroy`. (~150 min)
  - **Expect to hit #1**: current segment fills. Multiple producers race to allocate the next segment. Naive: each
    allocates its own, all CAS the `next` pointer, only one wins — losers leak their `Box`.
    **Fix**: try `next.load(Acquire)` first; allocate only on null; on CAS failure, `Box::from_raw` your loser and
    drop it explicitly.
  - **Expect to hit #2**: consumer pops the last item of segment S. S is now empty. Who frees it? If you `Box::drop`
    while a producer still holds a stale `head` reference, UAF.
    **Fix**: refcount: producers and consumers bump on linkage, decrement on departure. Only refcount-zero segments
    are freed. (W33+: migrate to `Guard::defer_destroy`; refcount becomes redundant.)
  - **Expect to hit #3**: segment `next` pointer stored `Relaxed`. Consumer follows non-null `next` and reads slot 0
    of the new segment, expecting initialized memory — but on weak hardware, the producer's seg-header init may not
    be visible yet. UAF or torn read.
    **Fix**: `next.store(new, Release)`; readers `next.load(Acquire)`.
  - **Muscle**: "who owns the node, when does it die" is the central question of every lock-free DS. Refcount
    answers it eagerly; EBR answers it lazily by epoch. Today you build the refcount answer so you have a baseline
    to bench against EBR in W33. **Reapplies at**: skiplist node reclamation (W37, via EBR), matching-engine order
    pool (W74).
- [ ] **Build**: `crates/wal/src/group_commit.rs` —
  `GroupCommit { pending: SegQueue<(WalRecord, oneshot::Sender<Lsn>)>, fsync_signal: Notify }`. Background fsync
  thread drains every 10 µs or 1024 records. Producers fan in lock-free; the fsync thread is the single consumer. (~150 min)
  - **Expect to hit**: under heavy load, producer drops its oneshot sender mid-flight (caller cancelled). The fsync
    thread reads `Some((record, dead_sender))`; sending on a dead oneshot is silently lost. Caller can't tell if its
    record was durable.
    **Fix**: producer's drop path enqueues a "best-effort durable" record; durability is observed via a monotonic
    LSN watermark, not the oneshot. The oneshot becomes a *fast-path optimization*, not the source of truth.
- [ ] criterion bench: throughput at group-size 1, 8, 64, 1024 — compare SegQueue fan-in against `Mutex<Vec<T>>`
  baseline. Expect SegQueue to win at ≥4 producers. (~60 min)
- [ ] Commit + log (~10 min)

**Wednesday — `wal` checksums + replay**

- [ ] **Build**: `crates/wal/src/checksum.rs` — CRC32C per record + per-segment-footer rollup. (~105 min)
- [ ] **Build**: `crates/wal/src/replay.rs` — `Replayer<'a, S: SegmentStream>` yields `Result<WalRecord, ReplayError>`.
  Stops cleanly at the first checksum mismatch (partial-tail handling). (~105 min)
- [ ] Test: write 10k records, truncate the last segment mid-record, re-open, replay — gets first 9_973 records then
  stops with a tail-truncation diagnostic. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — `wal` v0.1 ship + reth-db deep read part 1**

- [ ] **Build**: `crates/wal/src/lib.rs` — `Wal { dir: PathBuf, segments: Vec<Segment>, ... }` public API:
  `append(record) -> Future<Lsn>`, `replay() -> Replayer`, `checkpoint(lsn)`, `truncate_below(lsn)`. (~105 min)
- [ ] Tag `wal v0.1.0`. (~5 min)
- [ ] Read reth-db/src/lib.rs. Table definitions. (~45 min)
- [ ] Commit + log (~10 min)

**Friday — reth-db deep read part 2 + first reth storage PR hunt**

- [ ] Transaction impl. Cursor wrappers. (~30 min)
- [ ] Browse reth issues tagged storage. (~30 min)
- [ ] Find good-first-issue or docs issue. Claim. (~30 min)
- [ ] [Tempo] 15 min after Reth issue claimed: scan `tempoxyz/tempo` issues filtered by storage, db, state-root labels.
  Bookmark 2-3 candidate "future second-PR" issues. Do NOT claim. Reth comes first. (~15 min)
- [ ] Commit notes (~10 min)

**Saturday — First reth storage PR work**

- [ ] Implement. Submit PR. (~150 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 27 — `storage-trie::mdbx`: mmap scaffold + `wal` integration

**Monday — Research mmap in Rust**

- [ ] memmap2 crate docs. Rust + mmap safety. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — mmap B-tree research (decide: thin wrapper vs from-scratch)**

- [ ] B-tree on mmap techniques. (~30 min)
- [ ] **Decision**: thin wrapper over libmdbx-rs vs from-scratch. Default wrapper unless explicit re-implementation
  milestone. Record in notes/. (~120 min)
- [ ] Commit notes (~10 min)

**Wednesday — Crate structure (extending the W23 scaffold)**

- [ ] Lay out storage-trie/src/{mdbx, tables, mpt, state_root, merkle_stage, lib.rs}. mpt and state_root re-export from
  eth-trie. (~30 min)
- [ ] Sketch Tx / Cursor traits matching reth-db-api. (~20 min)
- [ ] Commit + log (~10 min)

**Thursday — Page provider over mmap + `bufpool` integration**

- [ ] Implement MmapPageProvider returning eth_storage_cache::Page views. (~120 min)
- [ ] Wire `bufpool::BufferPool<MmapPageProvider>` as the L1 cache in front of MDBX. Pinned pages survive eviction;
  dirty pages are written back to mmap on flush. (~45 min)
- [ ] Free-list allocation. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — mmap wrapper + `wal` write path**

- [ ] mmap-backed file wrapper with safe remap on growth. (~30 min)
- [ ] **Build**: storage-trie's write transaction now appends to `wal::Wal` BEFORE updating mmap pages. WAL-first means
  crash recovery is well-defined. (~105 min)
- [ ] Commit + log (~10 min)

**Saturday — Respond to reth PR review + continue crate**

- [ ] Address reth PR feedback. (~60 min)
- [ ] Continue crate work. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 20 min: skim Tempo releases page (tempoxyz/tempo/releases). Identify which TIPs are landing this week. Add
  to notes/tempo_roadmap.md. (Weekly ritual from now on.) (~20 min)

---

### Week 28 — `storage-trie` crate: B-tree core

**Monday — B-tree node design**

- [ ] Design leaf vs internal node layout. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — B-tree insert**

- [ ] Insert with node splitting. Unit tests. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — B-tree get**

- [ ] Lookup by key. Range iteration. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — B-tree delete**

- [ ] Delete with node merging. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Second reth storage PR**

- [ ] Pick next issue. Implement. (~120 min)
- [ ] Commit + log (~10 min)

**Saturday — Crate polish**

- [ ] Document public API. Benchmark setup. (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 7 review**

- [ ] Update North Star M7 metrics. (~15 min)
- [ ] [Tempo] 20 min during ritual: Tempo releases skim. (~20 min)

---

## Month 8: [NEW] `recovery` Crate + MVCC + Reth Storage Contribution Velocity

### Week 29 — [NEW] `recovery` crate v0.5 (ARIES analysis + redo)

**[NEW] crate created**: `crates/recovery/`. ARIES-style recovery as a reusable component.

**Monday — ARIES paper + design**

- [ ] Read Mohan et al. ARIES paper sections 1-4. (~90 min)
- [ ] **Build**: `crates/recovery/Cargo.toml` workspace member. Deps: `wal`, `time`. The wal crate gives us LSNs;
  recovery layers on top. (~20 min)
- [ ] **Build**: `crates/recovery/src/lib.rs` skeleton with module headers: `analysis.rs`, `redo.rs`, `undo.rs`,
  `checkpoint.rs`. (~105 min)
- [ ] Commit + log (~10 min)

**Tuesday — ARIES analysis pass**

- [ ] **Build**: `crates/recovery/src/analysis.rs` — `AnalysisPass<'a, R: Replayer>`. Walks WAL forward from last
  checkpoint LSN. Builds two tables: `DirtyPageTable` (PageId → recovery LSN) and `TransactionTable` (TxId → LastLsn).
  Stops at end of WAL. (~105 min)
- [ ] Test: 100 txs, half committed, replay analysis pass — TxTable correctly identifies in-flight txs. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — MVCC design (in `storage-trie`)**

- [ ] Design MVCC (version chain vs copy-on-write). (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — ARIES redo pass**

- [ ] **Build**: `crates/recovery/src/redo.rs` — `RedoPass`. Re-applies every WAL record from min(recoveryLSN) forward
  to a page if pageLSN < recordLSN. Idempotent. (~105 min)
- [ ] Test: crash mid-write, replay analysis + redo → page state matches pre-crash committed state. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Third reth storage PR + MVCC read tx**

- [ ] Medium-difficulty issue. (~30 min)
- [ ] Implement storage-trie read tx with snapshot (MVCC). (~45 min)
- [ ] Commit + log (~10 min)

**Saturday — Crate: durability + wal integration**

- [ ] fsync strategies. Crash recovery wired through `recovery::AnalysisPass` + `RedoPass`. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 20 min: Tempo releases skim. (~20 min)

---

### Week 30 — [NEW] `recovery` undo pass + ship v0.5; reth trie crate reading

**Monday — ARIES undo pass**

- [ ] **Build**: `crates/recovery/src/undo.rs` — `UndoPass` rolls back all txs in `TransactionTable` from analysis.
  Walks each tx's chain via prevLSN field of each WAL record. Writes Compensation Log Records (CLRs) so undo is
  idempotent under re-crash. (~105 min)
- [ ] Test: write 10 txs, abort 5 mid-flight via simulated crash, run analysis+redo+undo — final state matches "5
  committed, 5 never happened." (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — `recovery::checkpoint` + ship**

- [ ] **Build**: `crates/recovery/src/checkpoint.rs` — `Checkpointer` writes Begin-Checkpoint and End-Checkpoint WAL
  records with current DPT + TT. Bounds recovery time. (~105 min)
- [ ] **Build**: `crates/recovery/src/lib.rs` —
  `Recovery::recover(wal: &Wal, page_provider: &mut impl PageProvider) -> Result<(), Error>` orchestrating analysis →
  redo → undo. (~105 min)
- [ ] Tag `recovery v0.5.0`. (~5 min)
- [ ] Commit + log (~10 min)

**Wednesday — reth-trie state root**

- [ ] State root computation. Incremental. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — reth-trie hashed state**

- [ ] Hashed state abstraction. (~30 min)
- [ ] Commit notes (~10 min)

**Friday — First reth trie PR**

- [ ] Find trie-related issue. Implement. (~120 min)
- [ ] Commit + log (~10 min)

**Saturday — Crate: benchmarks + recovery integration in storage-trie**

- [ ] criterion benchmarks for B-tree ops. Baseline vs sled, redb. (~60 min)
- [ ] storage-trie::open() now calls `recovery::Recovery::recover(...)` before serving reads. (~30 min)
- [ ] [Tempo] 1 hr at end of day: while reth-trie is fresh, browse Tempo's trie integration. Note: Tempo uses Reth's
  trie wholesale; divergence is in state schema (TIP-20 token balances first-class). 2-paragraph note to tempo_diff.md. (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 31 — Persistent MPT in `storage-trie::mpt`

**Inheritance**: Nibbles, Node, HashBuilder, TrieStorage, StateRoot, ProofRetainer all in eth-trie. This week adds *
*persistent** backing — MdbxTrieStorage implementing eth-trie::TrieStorage against W27-29 MDBX + wal + recovery. Do NOT
reimplement.

**Monday — `MdbxTrieStorage` design**

- [ ] Design table layout (state nodes by hash, intermediate nodes by Nibbles path). (~30 min)
- [ ] Implement `eth_trie::TrieStorage for MdbxTrieStorage` skeleton. (~120 min)
- [ ] Commit + log (~10 min)

**Tuesday — Wire `eth-trie::Node` to the table layout**

- [ ] Cursor-based read path + dirty-set write path against MDBX. (~45 min)
- [ ] Commit + log (~10 min)

**Wednesday — Persistent insert via existing HashBuilder**

- [ ] Drive eth_trie::HashBuilder with MdbxTrieStorage as read source. (~30 min)
- [ ] Test: round-trip small trie through MDBX; assert root matches W10/W20 in-memory. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Persistent get via existing walker**

- [ ] Drive eth_trie::TrieWalker against MdbxTrieStorage. (~30 min)
- [ ] Range scans via MDBX cursor. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Root hash regression suite against `eth-trie` v0.2 fixtures**

- [ ] Re-run W20's Ethereum test vectors with persistent backing — assert byte-identical roots. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth trie second PR**

- [ ] Continue trie contribution. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 30 min during ritual: read one Tempo TIP from queue. Storage-relevant if available. Aim for "one TIP per
  ritual" through Phase 3. (~30 min)

---

### Week 32 — MPT proofs + more reth PRs

**Monday — MPT proof generation**

- [ ] Implement Merkle proof generation. Unit tests. (~120 min)
- [ ] Commit + log (~10 min)

**Tuesday — MPT proof verification**

- [ ] Standalone proof verification. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — MPT delete**

- [ ] MPT delete with rebalancing. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Ethereum test vectors**

- [ ] Integrate official trie test vectors. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Reth PR volume**

- [ ] Another reth PR (storage or trie). (~150 min)
- [ ] Commit + log (~10 min)

**Saturday — Crate docs**

- [ ] Comprehensive docs for all public APIs. Examples in docs. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 8 review**

- [ ] [Tempo] 20 min: Tempo releases skim. (~20 min)

---

## Month 9: LSM Trees + [NEW] `bloom` Crate + Advanced Trie + Staged Sync

### Week 33 — Advanced trie: path compression + [NEW] `epoch-gc` v0.1 scaffold + LSM extension reading

**[NEW] crate scaffolded**: `crates/epoch-gc/`. Mirror of `crossbeam-epoch`. **Epoch-based memory reclamation is the
foundation for every lock-free data structure that hands out pointers** — the skiplist memtable (W37 → W38), the
lock-free price level (W74), and any future "give me a pointer, I'll defer-free it when nobody's looking" primitive.
EBR is the hardest piece of crossbeam. Plan budget: W33 Wed-Fri scaffold + W37 Mon ship.

**Why EBR exists** (read before Wednesday): in any lock-free DS, removing a node creates a problem — other threads
may still hold pointers to it. Free too early → UAF. Never free → leak. EBR solves this by tagging each "in-progress
operation" with a global epoch counter; a freed pointer is only reclaimed once every thread has either left its
current epoch or advanced to a new one. The cost: per-thread bookkeeping plus a 3-epoch lag on actual frees. The
alternative — hazard pointers — uses per-thread "I'm reading this pointer" slots; simpler model, but every read pays
a fence-and-publish overhead. Crossbeam picks EBR for read-heavy data structures; you should understand both.

**Monday — Path compression theory + skyzh Week 1 Day 1-2 reading**

- [ ] Research path compression. Ethereum's approach. (~45 min)
- [ ] **Read** mini-lsm tutorial https://skyzh.github.io/mini-lsm/ Week 1 Day 1 (Memtable) and Day 2 (Merge Iterator).
  This is the alternative engine model — informs the W38-W40 `lsm-core` extraction. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Implement path compression in storage-trie**

- [ ] Add to crate MPT. Verify correctness. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — `epoch-gc` scaffold: `Atomic<T>`, `Owned<T>`, `Shared<'g, T>`**

- [ ] **Read** crossbeam-epoch's README + Keir Fraser's "Practical lock-freedom" PhD chapter on epoch reclamation
  (~2 hrs). Map the type hierarchy you'll mirror:
  - `Owned<T>` — heap-allocated, single-owner. Same shape as `Box<T>` but interoperates with `Atomic<T>::store`.
  - `Shared<'g, T>` — borrowed pointer valid only for the lifetime of guard `'g`. The lifetime tag IS the EBR
    correctness story expressed in the type system.
  - `Atomic<T>` — `AtomicPtr<T>` newtype with `load(&'g Guard) -> Shared<'g, T>` and CAS-with-Owned/Shared.
  - `Guard` — RAII pin handle. Holding one prevents the local epoch from advancing.
- [ ] **Read** crossbeam-epoch source: `src/atomic.rs`, `src/collector.rs`, `src/guard.rs`, `src/internal.rs`.
  Don't mirror line-by-line; sketch the state machine on paper first. (~30 min)
- [ ] **Build**: `crates/epoch-gc/Cargo.toml` workspace member. No deps (uses `core` + `alloc` only). (~20 min)
- [ ] **Build**: `crates/epoch-gc/src/atomic.rs` — `Atomic<T>`, `Owned<T>`, `Shared<'g, T>`. CAS via
  `compare_exchange_weak`. `Shared` carries `PhantomData<&'g T>` to bind to the guard's lifetime. (~150 min)
  - **Expect to hit #1**: you'll let `Shared::deref` work without a guard in scope (just `&self`). The lifetime
    `'g` is on `Shared` but you forgot to plumb it through `deref`, so `Shared::deref` returns a `&T` with the
    wrong lifetime. The compiler accepts it; the program UAFs.
    **Fix**: `Shared<'g, T>::deref(&self) -> &'g T` — the `'g` must propagate. If
    `let s = atomic.load(&guard); drop(guard); s.deref()` compiles, your lifetimes are wrong.
  - **Expect to hit #2**: tagged pointers. Crossbeam steals the low bits of pointers for "marked for deletion."
    Naive `Shared::deref` derefs the tagged pointer → misaligned load → miri error.
    **Fix**: every deref masks the tag. Provide `Shared::tag()` and `Shared::with_tag(usize)` explicitly; mask
    inside `deref` and inside CAS.
  - **Expect to hit #3**: `Owned<T>` and `Shared<'g, T>` share a representation but `Owned` must move-into-CAS
    (transferring ownership) while `Shared` is just a borrow. Mistakenly using `Shared` where CAS wants `Owned`
    means the node is dropped twice on success.
    **Fix**: `compare_and_set` consumes `Owned` by value (returns it on failure); `Shared` is only for reads.
  - **Muscle**: lifetimes are the proof system for "this pointer is reclaim-safe right now." If your `unsafe`
    block can't be expressed as "I have a `&'g`-tagged thing, so the guard must still be alive," your reclamation
    is unsound. **Reapplies at**: skiplist node access (W37), price-level order traversal (W74), any future
    lock-free DS.
- [ ] Commit + log (~10 min)

**Thursday — `epoch-gc::Guard` + `defer_destroy` + reth-stages reading (compressed)**

- [ ] **Build**: `crates/epoch-gc/src/guard.rs` — `pin() -> Guard` registers the current thread in the active epoch.
  `Guard::defer(closure)` and `Guard::defer_destroy(Shared)` enqueue cleanup into the thread's local garbage bag,
  drained when the thread observes that all threads have advanced past that epoch. (~150 min)
  - **Expect to hit #1**: **the missing fence.** Pinning has to be SeqCst — both the store ("I'm pinned in epoch E")
    and the load ("what's the global epoch?") must be SeqCst, OR you need an explicit `fence(SeqCst)` after the
    store and before reading any `Atomic<T>`. Otherwise the CPU reorders: thread A reads a pointer it thinks is
    protected, *then* publishes its pin, *then* thread B sees no pin, advances epoch, frees the pointer A is now
    dereferencing. UAF. Loom catches this *only* if your model uses SeqCst orderings; weaker orderings hide it.
    **Fix**: `pin()` does `epoch.store(active_epoch, SeqCst); fence(SeqCst);`. No shortcut — both pieces are
    load-bearing.
  - **Expect to hit #2**: **defer into the current epoch, not the next.** If thread A defers a destroy into epoch
    E, and the collector advances to E+1 immediately, A's defer can fire before thread B (still pinned in E) has
    unpinned. UAF.
    **Fix**: defers always go into the *current* epoch's bag; that bag is only drained two epochs later (E+2). The
    "3 garbage bags rotating" pattern exists for exactly this reason — slack so observers can drain.
  - **Expect to hit #3**: **reentrant pins.** Tests will eventually nest `pin()` calls (one in an outer fn, one in
    a helper called under that fn). If pin is a boolean flag, the inner drop unpins prematurely; the outer continues
    reading a now-unprotected pointer.
    **Fix**: pin is a counter, not a flag. Outer holds the +1, inner +1; drops decrement; only the last drop
    publishes "unpinned."
  - **Expect to hit #4**: **`defer_destroy` of a tagged pointer.** You defer a `Shared` with its tag bits still
    set; the destroy callback tries to free the tagged address; `dealloc` on a misaligned pointer is UB. Miri
    catches it.
    **Fix**: mask the tag before scheduling free. Belt-and-suspenders: assert alignment at defer-time.
  - **Muscle**: SeqCst is expensive but here it's *correctness*, not perf. Weakening EBR's pin/advance fences is
    the single most common EBR bug in the wild. **Reapplies at**: every future EBR-protected DS, the lock-free
    price level (W74).
- [ ] Browse reth/crates/stages deeply (compressed from Thursday's original slot). (~30 min)
- [ ] Commit + log (~10 min)

**Friday — `epoch-gc::Collector` + advance-epoch + loom + path-comp bench + stage diagram**

- [ ] **Build**: `crates/epoch-gc/src/internal.rs` —
  `Collector { global_epoch: AtomicUsize, locals: ThreadLocal<Local> }`. Each `Local` has a 3-slot ring of garbage
  bags (one per epoch). `advance()` checks every Local's pin-epoch; if all are either unpinned or pinned in the
  current epoch, it can bump `global_epoch` and reclaim the bag two epochs back. (~150 min)
  - **Expect to hit #1**: **the unpinned-but-about-to-pin thread.** A thread is unpinned momentarily; you advance;
    the thread re-pins — but the pointer it now reads was freed during the advance window.
    **Fix**: pinning reads `global_epoch` *under SeqCst*; if the thread observes the new epoch, it's protected
    against frees from prior epochs. The pin-store / global-load order is load-bearing.
  - **Expect to hit #2**: **a thread that pins, allocates 10MB into its garbage bag, never unpins.** Memory grows
    unbounded; `advance()` can never reclaim.
    **Fix**: ship a `Guard::flush()` API that pushes the bag to the collector mid-pin; document the failure mode
    for callers who hold guards across `await` points (don't).
  - **Expect to hit #3**: **`ThreadLocal` cost on the hot path.** Each pin is a TLS lookup. Bench shows this
    dominates the EBR overhead.
    **Fix**: cache the `Local*` raw pointer in a stack frame for the duration of the guard. Pinning becomes a
    single atomic store on the cached pointer.
  - **Expect to hit #4**: **double-advance.** Two threads call `advance()` concurrently; both observe "all locals
    are quiescent," both bump `global_epoch` by 1. You skipped a generation. Some local still pinned in the skipped
    epoch UAFs on its bag.
    **Fix**: advance via `compare_exchange` on `global_epoch`. Only one bumps; the other retries the quiescence
    check post-CAS.
  - **Muscle**: every "global coordinator + per-thread state" design has a quiescence-detection problem. EBR's
    advance is the canonical case. **Reapplies at**: matching-engine snapshot publishing (W74), consensus-raft
    log compaction barrier (W67).
- [ ] **Loom test**: 4 threads, each looping `{ pin; allocate Owned; atomic.store; load other thread's pointer;
  defer_destroy; unpin }` 3 times. Loom must explore all interleavings with **zero** UAF and **zero** double-free.
  Use the SeqCst model — Acquire/Release will hide the missing-fence bug. (~60 min)
- [ ] criterion: path compression on/off (the bench that moved here from Wednesday). (~60 min)
- [ ] Stage-dependencies diagram (compressed from Friday's original slot). (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth PR day + skyzh Week 2 (compaction) reading**

- [ ] Another reth PR. (~150 min)
- [ ] **Read** mini-lsm Week 2 in full (STCS compaction). (~30 min)
- [ ] [Tempo] 30 min at end of day: read 2 of the most storage-relevant TIPs end-to-end from W19 list. Add 1-page "TIP
  storage impact" summary to notes/. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 34 — [NEW] `bloom` crate v0.1 + Pruning strategies

**[NEW] crate created**: `crates/bloom/`. General bloom filter primitive — classic, counting, scalable. Note: Ethereum's
2048-bit Logs Bloom from eth-consensus stays where it is (it's a domain-specific encoding). The `bloom` crate is the
general engine used by storage-trie's pruning index, mini-db SSTable filters, and vector-db filtered-search bitmap.

**Monday — Bloom theory + design**

- [ ] Re-read mini-lsm bloom section. Read "Cache, Hash, and Space-Efficient Bloom Filters" (Putze et al.). FPR math: m
  bits, k hashes, n insertions → fpr ≈ (1 - e^(-kn/m))^k. (~45 min)
- [ ] **Build**: `crates/bloom/Cargo.toml` workspace member. Deps: `time` (for instrumentation only). (~20 min)
- [ ] **Build**: `crates/bloom/src/classic.rs` — `BloomFilter { bits: BitVec, k: u8, hasher_seeds: [u64; 4] }` with
  `with_fpr_at_capacity(fpr, n)` constructor. (~75 min)
- [ ] Test: 1M insertions, target FPR 1%, measured FPR ≤ 1.2%. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — Counting bloom + Reth pruning code**

- [ ] **Build**: `crates/bloom/src/counting.rs` — `CountingBloom` with u4 or u8 counters. Supports deletion. (~75 min)
- [ ] Read reth pruner crate. (~60 min)
- [ ] Commit + log (~10 min)

**Wednesday — Scalable bloom + storage-trie pruning design**

- [ ] **Build**: `crates/bloom/src/scalable.rs` — `ScalableBloomFilter` chains classic filters with tightening FPR (
  Almeida et al.). Grows without rebuild. (~75 min)
- [ ] Design pruning strategy trait in storage-trie. Plan MPT integration. (~30 min)
- [ ] Tag `bloom v0.1.0`. (~5 min)
- [ ] Commit + log (~10 min)

**Thursday — Implement full pruning**

- [ ] "full" retention (prune history beyond N blocks). (~30 min)
- [ ] Use `bloom::ScalableBloomFilter` to track which keys exist post-prune (avoids false-positive deletions). (~60 min)
- [ ] Commit + log (~10 min)

**Friday — Implement archive mode**

- [ ] Keep everything mode. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth PR + integration testing**

- [ ] Reth PR in pruning area if possible. (~150 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 20 min: Tempo releases skim. (~20 min)

---

### Week 35 — State commitment deep

**Monday — State commitment theory**

- [ ] State commitment schemes. MPT vs Verkle tradeoffs. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Verkle Trees reading**

- [ ] Verkle Trees research (Vitalik, EF). (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — Crate: incremental root**

- [ ] Design incremental state root computation. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Benchmark incremental vs full**

- [ ] Benchmark root computation. (~60 min)
- [ ] Commit + log (~10 min)

**Friday — Reth PR**

- [ ] Continue velocity. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Crate polish**

- [ ] Clean up APIs. Update docs. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 20 min: Tempo releases skim. (~20 min)

---

### Week 36 — Snapshot sync research

**Monday — Snapshot sync theory**

- [ ] Ethereum snapshot sync. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Erigon snapshots**

- [ ] Erigon's snapshot strategy. File format. (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — Reth snapshots**

- [ ] reth's snapshot approach. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Crate: snapshot export**

- [ ] Design export format. Basic export. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Crate: snapshot import + recovery integration**

- [ ] Snapshot import. Wire `recovery::Checkpointer` so a snapshot can be treated as a checkpoint barrier — recovery
  after snapshot import starts from the snapshot's LSN. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — End Month 9 PR push**

- [ ] 1-2 more reth PRs. (~30 min)
- [ ] [Tempo] 45 min at end of day: read tempoxyz/tidx README (Tempo's PostgreSQL + ClickHouse chain indexer). Note:
  analytics path separate from node snapshots. Document architectural split in tempo_diff.md. NOT something to build —
  interview context. (~45 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 9 review**

- [ ] Check: 15+ reth PRs, 10+ in storage/trie. (~30 min)

---

## Month 10: [NEW] `lsm-core` Build + Cross-Subsystem Storage PRs

### Week 37 — [NEW] `epoch-gc` v0.1 ship + `concurrent::skiplist` scaffold + medium reth PR

This is the keystone concurrency week. Two artifacts ship: (1) `epoch-gc v0.1.0` — fully loom-tested EBR; (2) the
lock-free concurrent skiplist that becomes `lsm-core`'s memtable next week. The skiplist is the place where every
muscle from W4 (CachePadded, Backoff), W11 (CAS retry idiom), W26 (segment reclamation), and W33 (EBR pin/defer)
comes together. Medium reth PR runs in parallel as background load.

**Monday — `epoch-gc` advance-epoch + loom hardening + tag v0.1.0**

- [ ] Adversarial loom test: 4 threads, each pinning, allocating, defer_destroying, unpinning in random order.
  Loom must explore the SeqCst model and exit clean. (~60 min)
- [ ] **Expect to hit**: loom flags a UAF on iteration ~3000 because `advance()` doesn't re-check the quiescence
  after winning the CAS — a thread can pin between your check and your bump. (~60 min)
  **Fix**: advance is a CAS loop, not a single CAS. On CAS success, re-check; if a Local now pins the new epoch,
  that's fine; if a Local pins the old epoch (it didn't observe your advance yet), revert the reclamation.
- [ ] **Expect to hit**: loom flags a double-free on iteration ~7000 because `Guard::defer_destroy` is called twice
  on the same `Shared` (a CAS-loop wrote the same pointer twice into the queue). (~60 min)
  **Fix**: dedup defers per-Local before the bag flush; OR establish the invariant "the thread that does the
  successful unlink is the unique caller of defer_destroy for that pointer" and audit your callers.
- [ ] Tag `epoch-gc v0.1.0` once loom is green. (~5 min)
- [ ] **Read** mini-lsm Week 3 (MVCC) Day 1 — half of the original W37 reading; finish Saturday. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — Concurrent skiplist theory + node scaffold**

- [ ] **Read** crossbeam-skiplist source. Map the node layout: each `Node<K, V>` has
  `tower: [Atomic<Node>; height]`, height generated by geometric distribution (p=0.5, max 12). The variable-length
  tail is allocated inline via `Layout::extend`. (~30 min)
- [ ] **Read** Herlihy/Shavit chapter 14 (concurrent skiplists), or the original Pugh paper if you prefer. (~30 min)
- [ ] **Build**: `crates/concurrent/src/skiplist/node.rs` — `Node` with variable-length tail (`tower` length =
  height). Tower entries are `epoch_gc::Atomic<Node>`. Custom `Drop` is empty (frees go through `defer_destroy`). (~150 min)
  - **Expect to hit #1**: you'll write the node as `struct Node<K, V> { key: K, val: V, tower: Box<[Atomic<Node>]> }`.
    That's *two* allocations per node (the node + the tower box), plus a pointer chase on every tower access.
    crossbeam-skiplist allocates inline.
    **Fix**: use `Layout::for_value` with `extend` to compute the combined layout; allocate once; expose
    `tower(&self) -> &[Atomic<Node>]` that returns a slice over the tail.
  - **Expect to hit #2**: tower height generation `rand::random::<u32>().leading_zeros()` looks correct but
    collides with retry logic. If insert retries, calling `rng.gen()` again gives a *different* height for the
    same key, changing the node footprint mid-flight.
    **Fix**: generate height *once*, before allocation; height is fixed for the node's lifetime.
  - **Muscle**: lock-free node memory layout is a one-shot decision. Re-deriving any per-node invariant during
    retry breaks the algorithm. **Reapplies at**: matching-engine order pool (W74), price-level bucket allocation.
- [ ] Commit + log (~10 min)

**Wednesday — Concurrent skiplist `find` + `insert` + medium reth PR (in parallel)**

- [ ] **Build**: `crates/concurrent/src/skiplist/find.rs` —
  `find(key, &Guard) -> Position { preds: [Shared<Node>; H], succs: [Shared<Node>; H] }`. At each level, walk right
  until you find a node `>=` key, recording the (pred, succ) pair. (~150 min)
  - **Expect to hit #1**: tagged-pointer marker bit on `tower[i]` indicates the node is logically deleted.
    `find` must (a) skip marked successors, AND (b) help-unlink them when observed (cooperative cleanup).
    Without help-unlink, dead nodes accumulate in the chain and `find` cost grows unboundedly.
    **Fix**: when you observe `succ` marked, CAS `pred.tower[i]` to skip `succ`; on success, defer_destroy succ
    (if you're the unlinker at level 0); on failure, restart from the top level.
  - **Expect to hit #2**: stale-pred race. You observe `pred → succ` at level i, descend to level i-1, but
    `pred → succ` was already unlinked at level i-1 by another thread. Your level-i-1 traversal starts from a
    stale pred.
    **Fix**: every level traversal validates that the pred from the upper level is still linked at this level
    (i.e., `pred.tower[i-1].load(Acquire)` still points where you expect). On mismatch, restart from the head.
- [ ] **Build**: `crates/concurrent/src/skiplist/insert.rs` — bottom-up linking, CAS at each level. (~150 min)
  - **Expect to hit #3**: insert at level 0 succeeds (the linearization point), but link at level 1 races with a
    delete. If you bail out, the node is partially linked — find() observes it at level 0 only. Correctness is
    preserved (level 0 is the truth), but performance degrades because higher levels are sparse.
    **Fix**: this is expected behavior. Document. Higher-level help-link is performed lazily by future inserts.
  - **Expect to hit #4**: classic CAS retry — you observe pred/succ via find(), do work, CAS pred. CAS fails
    because pred's tower changed. If you retry without re-running find(), you'll CAS into a stale pred and corrupt
    the chain.
    **Fix**: every CAS failure restarts find(). The cost is non-trivial; that's why the data structure has
    `height` levels — most inserts settle quickly.
- [ ] Reth PR (medium difficulty) — background load. (~150 min)
- [ ] Commit + log (~10 min)

**Thursday — Concurrent skiplist `delete` + loom + skyzh MVCC Day 2-3 reading**

- [ ] **Build**: `crates/concurrent/src/skiplist/delete.rs` — two-phase: (a) **logical** mark — set the marker bit
  on `tower[0]` via CAS; (b) **physical** unlink — CAS each pred.tower[i] to bypass the marked node, top-down.
  After all levels unlinked, `Guard::defer_destroy(node)`. (~150 min)
  - **Expect to hit #1**: you mark level 0 successfully, but another delete on the *same key* races. Both observe
    the unmarked state; one CAS wins, the other returns "not found" — but the loser is the user's `remove()` call,
    which should have returned the value. Your API loses data.
    **Fix**: the loser still won a useful race — it observed the value at find() time. Return the value the loser
    saw. The mark is the linearization point of the *one* successful delete.
  - **Expect to hit #2**: you `defer_destroy(node)` after marking but before all levels are physically unlinked.
    Some other thread's find() is still traversing levels >0 through this node. The defer fires; that thread now
    deref a freed `Atomic`.
    **Fix**: defer_destroy fires only after the *unlinker* (the thread that successfully CAS'd level 0's pred to
    bypass the marked node) completes. The marker (logical delete) does not free; only the unlinker does.
  - **Expect to hit #3**: ABA on the tower entry. `pred.tower[i]` was `A`; A gets unlinked, freed, the allocator
    reuses the address for `B`; `pred.tower[i]` is now `B`. Your CAS expecting A succeeds because the bit pattern
    matches.
    **Fix**: EBR is the answer — A cannot be reclaimed while any thread holds a guard pinned in A's epoch, and
    your `find()` runs under a guard. CAS won't observe a recycled address while the guard is live. This is the
    whole reason EBR exists; you'll feel it here for the first time.
  - **Muscle**: lock-free DS invariants must hold at *every* atomic observation point. The two-phase delete is the
    canonical example. **Reapplies at**: matching-engine order cancel (W74 — partial fills + cancel race), all
    future lock-free maps.
- [ ] Loom test: 4 threads racing insert + delete on the same 4 keys, 100 iterations each. Zero UAF, zero
  double-insert (per key), zero lost insert. SeqCst model. (~60 min)
- [ ] **Read** mini-lsm Week 3 Day 2-3 (compressed from original Friday). (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Skiplist range iter + benches + reth PR submit**

- [ ] **Build**: `crates/concurrent/src/skiplist/iter.rs` — forward iterator. At each step, follow `tower[0]`;
  skip nodes whose `tower[0]` is marked. Iterator holds a `Guard` for its lifetime (snapshot semantics). (~150 min)
  - **Expect to hit**: iterator-held guard prevents EBR advance for its entire lifetime. If a caller iterates a
    1M-entry range slowly, memory pressure builds.
    **Fix**: API contract: iterator is `!Send` and short-lived. For long scans, expose a `chunk_iter` that
    re-pins every N items, accepting that nodes inserted mid-scan may or may not be observed.
- [ ] criterion: `concurrent::SkipMap` vs `Arc<RwLock<BTreeMap>>` baseline at 1/4/16 threads, 50/50 read/write mix.
  Expect SkipMap to win at ≥4 threads; lose at 1 (no contention to amortize over). (~60 min)
- [ ] Reth PR submit. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth PR feedback + skyzh MVCC Day 4-7 reading**

- [ ] Address review comments on the Wed-Fri PR. (~60 min)
- [ ] **Read** mini-lsm Week 3 Day 4-7 (the rest of MVCC). (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 20 min: Tempo releases skim. (~20 min)

---

### Week 38 — [NEW] `lsm-core` v0.1: memtable + merge iterator

**[NEW] crate created**: `crates/lsm-core/`. Mini-LSM-equivalent core. The alternative storage engine the `mini-db`
CAPSTONE (W95) consumes.

**Monday — `lsm-core` scaffold + memtable**

- [ ] **Build**: `crates/lsm-core/Cargo.toml` workspace member. Deps: `time`, `bufpool`, `wal`, `bloom`. (~20 min)
- [ ] **Build**: `crates/lsm-core/src/memtable.rs` —
  `MemTable { skip_list: concurrent::SkipMap<Key, ValueWithTombstone>, size: AtomicUsize }`. Consumes the lock-free
  concurrent skiplist built in W37 (mirror of `crossbeam-skiplist`, EBR-backed). Skyzh's `Arc<RwLock<BTreeMap>>`
  design from tutorial Day 1 is the baseline you must beat under ≥4 concurrent writers; if your skiplist doesn't
  win at 4 threads, EBR overhead is mistuned (revisit W37 Fri bench). (~150 min)
  - **Expect to hit**: under the LSM write path, the memtable's `size: AtomicUsize` is bumped on every insert.
    On 16-thread bench, the atomic counter becomes the bottleneck — CachePadded on `size` doesn't help because
    the contention is *on the counter itself*, not false-sharing.
    **Fix**: per-thread size shards; sum lazily when the flusher needs the total. **Muscle**: counter sharding
    is the canonical fix for "atomic on a hot path is now the hot path." **Reapplies at**: matching-engine
    fill-counter (W74), messaging-aeron position publishing (W77).
- [ ] Test: insert 10k, scan in order, size accounting correct. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — `lsm-core` merge iterator**

- [ ] **Build**: `crates/lsm-core/src/iterator.rs` — `MergeIterator<I: KvIterator>` k-way merge via a min-heap over (
  key, source_idx, value). Tie-break by source_idx (newer wins). (~150 min)
- [ ] Test: merge 4 sorted streams of 1k each → 4k sorted with newer tombstones winning. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Reth codecs read**

- [ ] Read reth codecs crate. (~60 min)
- [ ] Commit notes (~10 min)

**Thursday — Zstd compression in reth + lsm-core block design**

- [ ] How reth uses compression. (~30 min)
- [ ] **Build**: `crates/lsm-core/src/block.rs` — `Block { offsets: Vec<u16>, data: Bytes, restart_interval: u16 }`.
  Restart-point compression (LevelDB-style). (~150 min)
- [ ] Commit notes (~10 min)

**Friday — Reth PR**

- [ ] Codec-related PR ideally. (~150 min)
- [ ] Commit + log (~10 min)

**Saturday — `lsm-core` block bench**

- [ ] criterion: block encoding/decoding throughput at restart_interval 8, 16, 64. (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 20 min: Tempo releases skim. (~20 min)

---

### Week 39 — [NEW] `lsm-core` v0.3: SSTable + bloom + read path

**Monday — `lsm-core` SSTable format**

- [ ] **Build**: `crates/lsm-core/src/sst/mod.rs` —
  `SSTable { blocks: Vec<Block>, index: BlockIndex, bloom: bloom::ClassicBloomFilter, meta: SstMetadata }`. Footer
  encodes index + bloom offsets. (~150 min)
- [ ] **Build**: `crates/lsm-core/src/sst/builder.rs` — `SstBuilder` streams kvs into blocks, populating bloom + index
  incrementally. (~150 min)
- [ ] Commit + log (~10 min)

**Tuesday — `lsm-core` SST read path**

- [ ] **Build**: `crates/lsm-core/src/sst/reader.rs` — `SstReader` mmap-backed, bloom-filter-first lookup, block-load
  via `bufpool`. (~150 min)
- [ ] Test: write 100k unique keys to SST, read back 1M random gets — bloom rejects ≥99% of non-existent keys. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Read storage discussions + lsm-core write path**

- [ ] All recent GitHub discussions on storage. (~30 min)
- [ ] **Build**: `crates/lsm-core/src/write_path.rs` —
  `flush_memtable_to_l0(mem: MemTable, dir: &Path) -> Result<SstReader>`. Wal-backed. (~150 min)
- [ ] Commit + log (~10 min)

**Thursday — Crate: composition test**

- [ ] Integration test: B-tree + MPT + transaction combined. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Crate: example + substantive Reth comment**

- [ ] Example showing typical storage-trie usage. (~30 min)
- [ ] Find appropriate discussion. Substantive technical comment. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Consolidation + more Reth PR**

- [ ] Review everything. (~30 min)
- [ ] Continue velocity. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 20 min: Tempo releases skim + 1-sentence summary of "what shipped in Tempo this month" to
  tempo_roadmap.md. (~20 min)

---

### Week 40 — [NEW] `lsm-core` v0.5: STCS compaction + ship

**Monday — STCS theory + implementation start**

- [ ] **Build**: `crates/lsm-core/src/compaction/stcs.rs` — Size-Tiered. Trigger when level i has ≥4 similarly-sized
  SSTs. Merge them via MergeIterator into level i+1. (~150 min)
- [ ] Commit + log (~10 min)

**Tuesday — Compaction scheduler**

- [ ] **Build**: `crates/lsm-core/src/compaction/scheduler.rs` — background thread runs compaction picks. Throttling via
  the same `backpressure::CreditFlowControl` matching-engine will use. (~150 min)
- [ ] Commit + log (~10 min)

**Wednesday — Feature implementation: lsm-backed alt-engine in storage-trie (experimental)**

- [ ] **Build**: `storage-trie::lsm_engine` feature flag wires lsm-core in as an alternative to MDBX. Not for
  production — proof of inheritance, and an option for chains with high write amplification budgets. (~150 min)
- [ ] Commit + log (~10 min)

**Thursday — Tag `lsm-core v0.5.0` + feature PR continuation**

- [ ] Tag. (~5 min)
- [ ] Continue reth feature PR. (~150 min)
- [ ] Commit + log (~10 min)

**Friday — Feature: tests**

- [ ] Comprehensive tests. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Feature: submit**

- [ ] Submit PR. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 10 review**

- [ ] [Tempo] 30 min during ritual: identify by name which 2 Tempo maintainers have been most active in the last month's
  PR/release activity (likely klkvr, legion2002, or 0xrusowsky). Update Tempo maintainer tracker. (~30 min)

---

## Month 11: [NEW] `txn` v0.5 + Feature Shipping + Crate v1.0 Prep

### Week 41 — Ship reth feature

**Monday — Address feature PR reviews**

- [ ] Iterate on reviews. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — More iteration**

- [ ] Address remaining feedback. (~60 min)
- [ ] Commit + log (~10 min)

**Wednesday — Feature merged ideally**

- [ ] If merged, celebrate + blog draft. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Crate: performance pass**

- [ ] Profile storage-trie. Hot paths. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Crate: optimizations**

- [ ] Implement optimizations. (~120 min)
- [ ] Commit + log (~10 min)

**Saturday — Another reth PR**

- [ ] Keep velocity. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 42 — [NEW] `txn` crate v0.5: 2PL + OCC + deadlock detect

**[NEW] crate created**: `crates/txn/`. Transaction lifecycle and concurrency control. Inherited by storage-trie (
rewires existing tx logic), ledger (W80), mini-db (W95), matching-engine (W74 — for per-symbol order-book mutations
under raft). v1.0 with 2PC ships W72.

**Monday — Geth comparison study (background reading)**

- [ ] Read Geth's Go implementation core/state package. (~45 min)
- [ ] Commit notes (~10 min)

**Tuesday — `txn` crate scaffold + lifecycle**

- [ ] **Build**: `crates/txn/Cargo.toml` workspace member. Deps: `time`, `wal`, `recovery`. (~20 min)
- [ ] **Build**: `crates/txn/src/lifecycle.rs` —
  `Txn { id: TxId, started_at: Hlc, state: AtomicState, locks: LockSet, log_records: SmallVec<Lsn> }`. State enum:
  Active, Preparing, Committed, Aborted. (~105 min)
- [ ] **Build**: `crates/txn/src/manager.rs` —
  `TxnManager { next_id: AtomicU64, active: DashMap<TxId, Arc<Txn>>, ... }`. (~105 min)
- [ ] Commit + log (~10 min)

**Wednesday — `txn` 2PL + deadlock detect**

- [ ] **Build**: `crates/txn/src/locks.rs` — `LockManager` with shared/exclusive lock table per key. Wound-Wait deadlock
  prevention (newer transactions wound older ones). (~105 min)
- [ ] **Build**: `crates/txn/src/deadlock.rs` — wait-for graph + cycle detection (Tarjan SCC) as a fallback for explicit
  deadlock detection. (~105 min)
- [ ] Test: 4 txs in a cycle, deadlock detector picks the youngest as victim. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — `txn` OCC + write phase**

- [ ] **Build**: `crates/txn/src/occ.rs` — `OccValidator` validates read-set version stamps at commit. Used for
  read-heavy workloads. (~105 min)
- [ ] **Build**: `crates/txn/src/commit.rs` — commit path: prepare WAL record → fsync → mark Committed → release locks.
  Abort path: emit CLRs via `recovery::UndoPass` if necessary. (~105 min)
- [ ] Commit + log (~10 min)

**Friday — `txn` storage-trie integration**

- [ ] **Refactor**: storage-trie's existing write-tx now wraps `txn::Txn`. Per-key locks via `txn::LockManager`. WAL
  records via `txn::commit()`. (~60 min)
- [ ] Tag `txn v0.5.0`. (~5 min)
- [ ] Reth PR — pick something simple to keep velocity. (~150 min)
- [ ] Commit + log (~10 min)

**Saturday — Crate: property tests + fuzz**

- [ ] proptest for MPT invariants. (~45 min)
- [ ] cargo-fuzz target for `txn::LockManager` (random lock/unlock sequences must never deadlock with Wound-Wait). (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 43 — Crate v1.0 preparation

**Monday — API review**

- [ ] Review all public APIs. Stabilize. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — Documentation pass**

- [ ] Every public item has docs. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Examples expansion**

- [ ] Multiple examples in examples/. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — CI hardening**

- [ ] All CI checks pass. Coverage. MSRV. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — README + design doc**

- [ ] Comprehensive README. DESIGN.md showing storage-trie's full inheritance tree: bufpool ← wal ← recovery ← txn ←
  bloom ← eth-trie ← eth-storage-cache. (~60 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth PR**

- [ ] More PR activity. (~150 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 20 min: Tempo releases skim. (~20 min)

---

### Week 44 — `storage-trie` v1.0 ship

**Monday — Final benchmarks**

- [ ] Comprehensive bench suite. Compare vs reth, sled, redb. (~60 min)
- [ ] **Inheritance audit**: count LOC in storage-trie. Of those, count LOC that calls into bufpool / wal / recovery /
  txn / bloom / eth-trie / eth-storage-cache. Aim ≥70% wired-up inheritance ratio in LOC or call-site count. (~60 min)
- [ ] Commit + log (~10 min)

**Tuesday — Security review self-audit**

- [ ] Review unsafe blocks. Error handling. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Crate v1.0 tag**

- [ ] Tag `storage-trie v1.0.0`. First Layer-5 product ships. (~5 min)
- [ ] Commit + log (~10 min)

**Thursday — Blog: crate intro (Phase 3 retrospective option)**

- [ ] If writing mood, draft "Building storage-trie: how four primitives carry the weight" post. The hook is the
  inheritance discipline. No deadline. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Reth PR**

- [ ] Continue. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Month 11 review**

- [ ] Assess crate quality. PR portfolio. (~150 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 11 review**

---

## Month 12: Phase 3 Close + Phase 4 Prep + M12 Decision Gate

### Week 45 — Final reth storage feature

**Monday — Identify second feature**

- [ ] Another meaningful opportunity. Design. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Implement**

- [ ] Code. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Continue**

- [ ] Commit + log (~10 min)

**Thursday — Tests**

- [ ] Commit + log (~10 min)

**Friday — Submit**

- [ ] Submit PR. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Iterate on reviews**

- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 20 min: Tempo releases skim. (~20 min)

---

### Week 46 — Recognition signals

**Monday — Review PRs of others**

- [ ] Review others' storage PRs substantively. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Help newcomers**

- [ ] Answer questions in Telegram. (~15 min)
- [ ] Commit notes (~10 min)

**Wednesday — More PR reviews**

- [ ] Build reviewing muscle. (~75 min)
- [ ] Commit notes (~10 min)

**Thursday — Maintainer relationship check**

- [ ] Which maintainers engaged. Update tracker. (~30 min)
- [ ] Commit notes (~10 min)

**Friday — Active issue engagement**

- [ ] Participate in design discussions. (~30 min)
- [ ] Commit notes (~10 min)

**Saturday — Another small reth PR**

- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 47 — revm preview for Phase 4

**Monday — revm architecture refresher**

- [ ] Re-read revm with Phase 3 eyes. (~45 min)
- [ ] Commit notes (~10 min)

**Tuesday — Identify revm learning gaps**

- [ ] Map what needs deep understanding in Phase 4. (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — Reth evm crate**

- [ ] Read reth/crates/evm. (~60 min)
- [ ] Commit notes (~10 min)

**Thursday — More reth PR**

- [ ] Commit + log (~10 min)

**Friday — Crate maintenance**

- [ ] Any bug fixes on storage-trie. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Phase 4 prep (exec-vm already scaffolded — review state)**

- [ ] exec-vm seeded W9 + extended W17. Re-read README + opcode coverage matrix. Gap to Phase 4 v1.0. Phase 4 outline in
  notes/. (~30 min)
- [ ] [Tempo] 1 hr at end of day: re-read TIP-1020 (signature verification precompile) with exec-vm precompile registry
  in mind. Sketch in `notes/tempo_evm_ext_design.md` the 3-4 traits and types tempo-evm-ext will need so it can be a
  downstream crate of exec-vm without forking. (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 48 — Phase 3 close + M12 Decision Gate

**Monday — Phase 3 reflection**

- [ ] Full assessment vs exit criteria. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Metrics update**

- [ ] Update all North Star metrics. Target: 30 storage PRs, 1+ feature, storage-trie v1.0 ✓, 5 new primitive crates
  shipped (time, backpressure, bufpool, wal, recovery, bloom, lsm-core, txn). (~15 min)
- [ ] [Tempo] Update M12 Tempo metrics: orientation depth (target 3 — should be there), TIPs read (target 5 — count
  carefully), PRs merged (target 3 — flag if zero; first Tempo PR scheduled W60-62 so 0 here is acceptable). Do NOT
  panic-claim if zero. (~15 min)
- [ ] Commit notes (~10 min)

**Wednesday — M12 Decision Gate**

- [ ] Three questions. Answer in `progress.md`. (~30 min)
    - [ ] **Reth velocity**: am I on track for 35 Reth PRs merged by M18 (need ~3 PR-bundles per month)? If below 60% of
      trajectory, flag and discuss. (~150 min)
    - [ ] **Inheritance discipline**: did storage-trie ship with ≥70% inheritance ratio? If no, audit; the discipline
      failure here will compound through Phases 4-7. (~30 min)
    - [ ] **Energy / sustainability**: are sleep, fitness, day-job satisfaction green? If any one is red for ≥4 weeks,
      plan a 2-week rest window before Phase 4. (~30 min)
- [ ] No path change at M12 — too early. Just calibrate within the Reth-primary plan. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Relationship stock-take**

- [ ] Update maintainer tracker. Identify mentor candidate. (~15 min)
- [ ] Commit notes (~10 min)

**Friday — Final Phase 3 PRs**

- [ ] Wrap outstanding. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Clean transition prep**

- [ ] Mental prep for Phase 4. Storage maintenance minimum during Phase 4. (~30 min)
- [ ] Commit notes (~10 min)

**Sunday — End Phase 3 rest**

- [ ] Full rest. Phase 4 starts tomorrow. (~30 min)

---

# PHASE 4: EXECUTION DEEP DIVE + [NEW] DISTRIBUTION PRIMITIVES + HFT TRACK BEGINS (Month 13-18)

**Reth Deliverable**: `exec-vm` v1.0 (W68) — full revm-equivalent EVM.
**[NEW] Layer-4 Deliverables**: `p2p` v0.5 (W55), `consensus-raft` v1.0 (W67), `consensus-bft` v0.5 (W68).
**[NEW] HFT-track scaffolds**: `matching-engine` scaffold (W58), `matching-engine` v0.5 (W63), `matching-engine` v0.7 (
W73). The HFT track begins here as scaffold; primary in Phase 5.
**[NEW] Tempo crate shipments**: `tempo-evm-ext` scaffold (W54), `tempo-tx-envelope` v0.1.0 (W66).

> Tempo Phase 4 budget: 4-5 hrs/wk. First Tempo PR W60 Sat. `tempo-evm-ext` scaffolded W54 Sat. `tempo-tx-envelope`
> v0.1.0 shipped W66 Fri.
> HFT Phase 4 budget: 0 hrs M13 → 0 hrs M14 → ramp to 5 hrs/wk by M16 (W63 matching-engine v0.5) → 7-8 hrs/wk M17-M18.

## Month 13: Revm Full Codebase + First revm Perf PRs

### Week 49 — Revm architecture deep

**Monday — Revm top-level**

- [ ] Re-read revm from top. Map all crates. (~45 min)
- [ ] Commit notes (~10 min)

**Tuesday — Revm interpreter core**

- [ ] Read revm-interpreter in full. Main execution loop. (~45 min)
- [ ] Commit notes (~10 min)

**Wednesday — Revm Host trait**

- [ ] Read Host trait and impls. (~45 min)
- [ ] Commit notes (~10 min)

**Thursday — Revm Database trait**

- [ ] Read Database trait. How it integrates with any storage. (~45 min)
- [ ] Commit notes (~10 min)

**Friday — Revm precompiles**

- [ ] Read revm-precompiles crate. Each precompile. (~60 min)
- [ ] [Tempo] 1 hr: read revm-precompiles AND tempoxyz/tempo precompile extensions side-by-side. Tempo adds TIP-1020 (
  P256/WebAuthn/secp256k1 verify) as stateful precompile reusing tx-signature verification. Map: where in exec-vm does
  this plug in? (Same dispatch point as W19's ECRECOVER, but signature scheme dispatcher must be generic.) Update
  tempo_evm_ext_design.md. (~60 min)
- [ ] Commit notes (~10 min)

**Saturday — First revm perf-oriented PR**

- [ ] Find performance issue. Implement. (~120 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 50 — Revm journaling

**Monday — Journaling design**

- [ ] Read revm-interpreter journal module. Revert semantics. (~45 min)
- [ ] Commit notes (~10 min)

**Tuesday — Nested checkpoints**

- [ ] Study nested call handling. (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — State access patterns**

- [ ] Read state management in revm. (~45 min)
- [ ] Commit notes (~10 min)

**Thursday — Second revm PR**

- [ ] Another contribution. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — `exec-vm`: align traits with revm `Database`/`Host`**

- [ ] Refactor signatures so any `impl Database for T` from revm Just Works as Host for exec-vm. Goal: swap revm in/out
  with one type alias change. (~60 min)
- [ ] [Tempo] 30 min: while refactoring, verify trait shapes are compatible with TempoEvm's extension pattern. Skim
  tempoxyz/tempo's evm crate to confirm tempo-evm-ext can be downstream consumer without trait-incompatible changes. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Interpreter loop refactor**

- [ ] Consolidate match-based dispatch from W9 + W17 into `interpreter/dispatch.rs`. Set up W58 jump-table swap. (~45 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 51 — Opcode coverage gap-fill

**Inheritance**: most basic opcodes DONE (W9 + W17). This week fills missing arithmetic/bitwise and Cancun-specific
opcodes.

**Monday — Missing arithmetic: SDIV, SMOD, ADDMOD, MULMOD, EXP, SIGNEXTEND**

- [ ] Implement. Unit tests against revm for edge cases. (~120 min)
- [ ] Commit + log (~10 min)

**Tuesday — Missing bitwise: BYTE, SHL, SHR, SAR**

- [ ] Implement against revm fixtures. (~120 min)
- [ ] Commit + log (~10 min)

**Wednesday — KECCAK256 + missing call-frame envs**

- [ ] KECCAK256. CALLDATALOAD, CALLDATASIZE, CALLDATACOPY, CODESIZE, CODECOPY, RETURNDATASIZE, RETURNDATACOPY, GASPRICE,
  ORIGIN, CALLER, CALLVALUE. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — PREVRANDAO + DIFFICULTY post-Merge handling**

- [ ] Same opcode byte (0x44), different semantics. Fork-aware via CfgEnv::spec_id. (~30 min)
- [ ] PC, MSIZE, GAS, JUMPDEST coverage check. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — TLOAD/TSTORE (EIP-1153, Cancun)**

- [ ] Transient storage scoped to call frame. Adds `transient: HashMap` to call-frame state. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — MCOPY (EIP-5656, Cancun) + opcode-coverage matrix audit**

- [ ] MCOPY copies memory regions. (~30 min)
- [ ] Diff opcode coverage table against revm's instruction table. (~45 min)
- [ ] [Tempo] 30 min: cross-check opcode coverage against Tempo's EVM. They use upstream revm opcodes plus stateful
  precompiles — no opcode divergence. Note in tempo_evm_ext_design.md: "tempo-evm-ext adds precompiles + tx handler, not
  opcodes." (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 52 — [NEW] `p2p` crate v0.1: Kademlia discovery

**[NEW] crate created**: `crates/p2p/`. Layer-4 distribution primitive. Inherited by consensus-raft (W56+),
consensus-bft (W64+), messaging-aeron (W76 — peer discovery only, not data plane).

**Monday — Kademlia paper + p2p scaffold**

- [ ] Read Maymounkov & Mazières Kademlia paper. (~90 min)
- [ ] **Build**: `crates/p2p/Cargo.toml` workspace member. Deps: `time`, `eth-network-codec`, `backpressure`. (~20 min)
- [ ] **Build**: `crates/p2p/src/lib.rs` with module headers: `kademlia.rs`, `noise.rs`, `gossip.rs`, `peer.rs`. (~105 min)
- [ ] Commit + log (~10 min)

**Tuesday — `p2p::peer` + node identity**

- [ ] **Build**: `crates/p2p/src/peer.rs` — `PeerId(B256)` derived from public key. `Multiaddr` wrapper. `PeerStore` LRU
  of last-seen peers. (~105 min)
- [ ] Commit + log (~10 min)

**Wednesday — `p2p::kademlia` routing table**

- [ ] **Build**: `crates/p2p/src/kademlia/table.rs` — `RoutingTable { buckets: [KBucket; 256] }`. K-bucket eviction with
  last-seen ordering. (~105 min)
- [ ] Commit + log (~10 min)

**Thursday — `p2p::kademlia` FIND_NODE**

- [ ] **Build**: `crates/p2p/src/kademlia/protocol.rs` — FIND_NODE, FIND_VALUE, STORE, PING RPCs. Parallel α=3 lookups. (~105 min)
- [ ] Commit + log (~10 min)

**Friday — Reth PR (Engine API area preview)**

- [ ] Continue velocity. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — `p2p::gossip` skeleton**

- [ ] **Build**: `crates/p2p/src/gossip.rs` — `GossipBroadcast<T: Encodable>` with `Vec<PeerId>` fan-out, lazy push (
  IHAVE/IWANT). Inspired by Plumtree but simpler. (~105 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 13 review**

- [ ] [Tempo] 20 min: Tempo releases skim. If release touches EVM crate, read diff and note implications for
  tempo-evm-ext. (~20 min)

---

## Month 14: Full Opcode Coverage + [NEW] `p2p` Noise + Tempo Precompiles Scaffold

### Week 53 — Complete opcode set

**Monday — RETURN, REVERT, INVALID**

- [ ] Terminal opcodes. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — SELFDESTRUCT**

- [ ] Implement. (~120 min)
- [ ] Commit + log (~10 min)

**Wednesday — EIP-1153 transient storage**

- [ ] TLOAD/TSTORE if not done. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Test vector integration**

- [ ] Integrate Ethereum execution test vectors. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — revm PR**

- [ ] Contribution. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth evm PR**

- [ ] Find reth evm crate issue. Implement. (~120 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 54 — Precompiles in `exec-vm` + [Tempo] `tempo-evm-ext` scaffold + [NEW] `p2p::noise`

**Monday — ecrecover + `p2p::noise` skeleton**

- [ ] Implement ecrecover precompile in `exec-vm`. Test vectors. (~120 min)
- [ ] **Build**: `crates/p2p/src/noise.rs` — Noise_XX_25519_ChaChaPoly_BLAKE2s handshake using `snow` crate. (~105 min)
- [ ] Commit + log (~10 min)

**Tuesday — sha256, ripemd160, identity + Noise handshake tests**

- [ ] Implement all three precompiles. (~120 min)
- [ ] Test: two p2p nodes complete Noise_XX handshake over a memory pipe. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — modexp + p2p connection state machine**

- [ ] Implement modexp (using num-bigint). (~120 min)
- [ ] **Build**: `crates/p2p/src/connection.rs` — type-state state machine: Disconnected → Handshaking → Authenticated →
  Open → Closed. (~105 min)
- [ ] Commit + log (~10 min)

**Thursday — BN256 operations**

- [ ] BN256Add, BN256ScalarMul, BN256Pairing. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — blake2f + precompile-registry extension hooks**

- [ ] Implement Blake2 F compression in `exec-vm`. (~120 min)
- [ ] [Tempo] 1 hr at end of day: design exec-vm's precompile dispatch so a downstream tempo-evm-ext can register P256 +
  WebAuthn verify precompiles without forking. Registry must accept new precompile addresses via registration call (use
  `Box<dyn Precompile>` over `HashMap<Address, Box<dyn Precompile>>` — W19 skeleton supports this). Add test that
  registers a dummy "always-return-zero" precompile at address 0x100 to prove extensibility. (~60 min)
- [ ] Commit + log (~10 min)

**Saturday — KZG precompile + `tempo-evm-ext` scaffold**

- [ ] Point evaluation precompile (EIP-4844). (~30 min)
- [ ] [Tempo] 45 min: **`tempo-evm-ext` scaffold** — create `crates/tempo-evm-ext/` workspace member. `Cargo.toml`
  depends on exec-vm, eth-primitives, eth-consensus. Empty `lib.rs` with single
  `register_tempo_precompiles(registry: &mut PrecompileRegistry)` stub function. cargo build --workspace green. No real
  code yet; lands W66+. (~45 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 55 — Journaling in `exec-vm` + [NEW] `p2p` v0.5 ship

**Monday — Journal design + p2p gossip impl**

- [ ] Design journal structure in exec-vm. Mirror revm's approach. (~30 min)
- [ ] **Build**: `p2p::gossip` push path with bounded fanout (8 peers default), IHAVE pull-back. (~105 min)
- [ ] Commit + log (~10 min)

**Tuesday — Account journal**

- [ ] Track account changes with undo log. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Storage journal + p2p tag v0.5**

- [ ] Track storage changes in exec-vm. (~30 min)
- [ ] Tag `p2p v0.5.0`. The crate is now ready to back consensus-raft (W56). (~5 min)
- [ ] Commit + log (~10 min)

**Thursday — Nested checkpoints**

- [ ] Support nested call checkpoint/commit. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Revert semantics tests**

- [ ] Test revert properly undoes all changes. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — revm PR**

- [ ] Contribution. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 56 — Test vector push + [NEW] `consensus-raft` v0.1: election

**[NEW] crate created**: `crates/consensus-raft/`. Layer-4 consensus primitive. Inherited by matching-engine v1.0 (W74)
and mini-db's distributed mode stub.

**Monday — Ethereum tests integration + raft paper re-read**

- [ ] Integrate comprehensive test vectors into exec-vm. (~30 min)
- [ ] Re-read Ongaro Raft paper. Focus on Figure 2 (RPCs) and Section 5. (~45 min)
- [ ] Commit + log (~10 min)

**Tuesday — General state tests + raft scaffold**

- [ ] Run general state tests. Fix failures. (~30 min)
- [ ] **Build**: `crates/consensus-raft/Cargo.toml` workspace member. Deps: `time`, `wal`, `p2p`, `txn`. (~20 min)
- [ ] **Build**: `crates/consensus-raft/src/lib.rs` with module headers: `state.rs`, `election.rs`, `log.rs`, `rpc.rs`,
  `membership.rs`, `snapshot.rs`. (~150 min)
- [ ] Commit + log (~10 min)

**Wednesday — More failure fixing + raft state machine skeleton**

- [ ] Continue exec-vm test-vector fixes. (~30 min)
- [ ] **Build**: `crates/consensus-raft/src/state.rs` — `Role` enum (Follower, Candidate, Leader).
  `Persistent { current_term, voted_for, log }` and `Volatile { commit_index, last_applied }`. (~150 min)
- [ ] Commit + log (~10 min)

**Thursday — Validator tests + raft election RPCs**

- [ ] Run validator test suite. (~30 min)
- [ ] **Build**: `crates/consensus-raft/src/rpc.rs` — RequestVote, AppendEntries types. (~150 min)
- [ ] **Build**: `crates/consensus-raft/src/election.rs` — election timeout (150-300 ms randomized), vote casting, term
  increment. (~150 min)
- [ ] Commit + log (~10 min)

**Friday — reth PR**

- [ ] Storage or evm. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — revm PR**

- [ ] Contribution. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 14 review**

- [ ] Crate (exec-vm) passing majority of test vectors. (~30 min)
- [ ] [Tempo] 20 min: Tempo releases skim. Check Tempo PR queue for any open PRs that look like good-first-issue
  material. Bookmark for W60. (~20 min)

---

## Month 15: Dispatch Strategies + EthCC + [HFT] `matching-engine` Scaffold + Raft Replication

### Week 57 — EthCC Paris trip

**Monday-Friday — Conference attendance**

- [ ] Attend EthCC sessions. (~30 min)
- [ ] Target: 1-on-1 with 3 reth core contributors. Arrange via Twitter DM in advance. (~60 min)
- [ ] Side events (hacker houses, dinners). (~30 min)
- [ ] [Tempo] If any Tempo team members or design partners at EthCC, request 1-on-1. Same priority as Reth core 1-on-1s. (~30 min)
- [ ] Take notes on talks. (~30 min)

**Saturday — Travel home**

- [ ] Rest. (~5 min)

**Sunday — Post-conference ritual**

- [ ] Update maintainer tracker (Reth + Tempo) with new connections. (~15 min)
- [ ] Follow-up emails/DMs. (~30 min)

---

### Week 58 — Back to work: dispatch strategies + [HFT] `matching-engine` scaffold

**[NEW HFT-track entry point]**: This week the HFT track begins as scaffold. Time budget: 3 hrs/wk through W62, ramping
to 5 hrs/wk W63+.

**Monday — Match dispatch (baseline)**

- [ ] Baseline benchmark. (~60 min)
- [ ] Commit + log (~10 min)

**Tuesday — Jump table research**

- [ ] Function pointer jump tables. (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — Implement jump table dispatch + [HFT] matching-engine scaffold**

- [ ] In exec-vm. Feature-flagged. (~30 min)
- [ ] [HFT] **Build**: `crates/matching-engine/Cargo.toml` workspace member. Deps: `time`, `backpressure`, `wal`,
  `recovery`. Empty `lib.rs`. cargo build --workspace green. (~15 min)
- [ ] [HFT] Sketch in `notes/matching_engine_design.md`: data structures (price levels via RB-tree or skip list — decide
  W63), order ID generation, deterministic event log via wal, per-symbol shard plan. (~25 min)
- [ ] [Tempo] 15 min check: ensure tempo-evm-ext can plug into both match-dispatch and jump-table-dispatch code paths.
  No code today — just a comment in tempo_evm_ext_design.md. (~15 min)
- [ ] Commit + log (~10 min)

**Thursday — Computed goto research**

- [ ] Unsafe computed goto via asm. Portability tradeoffs. (~30 min)
- [ ] Commit notes (~10 min)

**Friday — Benchmark match vs jump table**

- [ ] Measure instruction-level differences. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Dispatch strategy docs**

- [ ] Document findings. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 59 — evmone comparison + [NEW] `consensus-raft` v0.3: log replication

**Monday — evmone overview**

- [ ] Read evmone README deeply. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — evmone basic interpreter + raft log replication**

- [ ] Basic mode (evmone). (~30 min)
- [ ] **Build**: `crates/consensus-raft/src/log.rs` — `LogEntry { term, index, command, lsn: Lsn }`. Backed by
  `wal::Wal` for durability. (~150 min)
- [ ] Commit notes (~10 min)

**Wednesday — evmone advanced mode**

- [ ] Advanced interpreter with caching. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Apply learnings to `exec-vm` + raft AppendEntries**

- [ ] Implement applicable optimizations in exec-vm. (~120 min)
- [ ] **Build**: `crates/consensus-raft/src/replication.rs` — leader sends AppendEntries; followers persist via wal then
  ack; leader bumps commitIndex when majority replicates. (~150 min)
- [ ] Commit + log (~10 min)

**Friday — Benchmark exec-vm vs revm**

- [ ] Comprehensive benchmark. Identify gaps. (~60 min)
- [ ] [Tempo] 20 min: note whether tempo-evm-ext's extra precompile dispatch overhead is measurable. (Likely 0 if
  feature-flagged and not registered.) (~20 min)
- [ ] Commit + log (~10 min)

**Saturday — revm PR**

- [ ] Another contribution. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 60 — Hot path optimization + [Tempo] First Tempo PR + [HFT] matching-engine: order book draft

**Monday — Profile `exec-vm`**

- [ ] Profile with perf or similar. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Stack optimization + SIMD primer**

- [ ] 60-90 min reading: `std::simd` (portable SIMD) + `std::arch::x86_64` (target-specific intrinsics). Focus on
  `__m256i` load/store and the autovectorization patterns rustc already does. Skim the `wide` crate as the stable
  ecosystem option. (~90 min)
- [ ] Inline stack ops. While you're in the stack code, evaluate whether U256 push/pop or copy paths can benefit from
  256-bit SIMD loads (likely yes for batched memory ops, marginal for single-value push). Note one candidate hot spot in
  `EXEC_VM_PERF_BACKLOG.md` for SIMD experimentation in W64. (~30 min)
- [ ] [HFT] 30 min: sketch matching-engine's L2 order book data structure. Decision point: RB-tree (e.g. via std::
  collections::BTreeMap) vs skip list (cache-friendlier for narrow price spreads, harder to implement correctly).
  Default RB-tree this week; revisit at W63. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Memory access + matching-engine order book skeleton**

- [ ] Optimize exec-vm memory reads/writes. MCOPY and CODECOPY are the obvious SIMD candidates — try
  `std::arch::x86_64::_mm256_loadu_si256` / `_mm256_storeu_si256` for aligned 32-byte block copies and bench against the
  naive loop. Feature-flag the SIMD path so non-x86_64 targets fall back cleanly. (~60 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/book.rs` —
  `OrderBook { bids: BTreeMap<Price, PriceLevel>, asks: BTreeMap<Price, PriceLevel> }`.
  `PriceLevel { orders: VecDeque<Order> }` for FIFO/price-time priority. (~150 min)
- [ ] Commit + log (~10 min)

**Thursday — Gas calculation**

- [ ] Optimize gas tracking in exec-vm hot path. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Benchmark improvements**

- [ ] Measure gains. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — More reth PRs + [Tempo] First Tempo PR + [HFT] matching-engine: Order type**

- [ ] Keep reth velocity (10+ execution PRs target M18). (~30 min)
- [ ] [Tempo] 2 hrs: **First Tempo PR claim**. You now have 12+ months of Reth/revm context AND tempo-evm-ext is
  scaffolded. Browse `tempoxyz/tempo` issues filtered by `good-first-issue` or `help-wanted`. Prefer issues touching
  TempoEvm or transaction-parsing surfaces. Pick ONE. Comment claiming. Begin implementation. (~120 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/order.rs` —
  `Order { id: u64, side: Side, price: Price, qty: Qty, ts: Monotonic, owner: AccountId, kind: OrderKind }`. OrderKind:
  Limit, Market, IOC, FOK, PostOnly. (~150 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 15 review**

---

## Month 16: EOF + Integration + [NEW] consensus-raft v0.7 + [HFT] matching-engine v0.5

### Week 61 — EOF implementation + [Tempo] PR #1 progress + [HFT] matching: cross logic

**Monday — EOF EIP deep re-read**

- [ ] Re-read EIP-3540, 3670. EOF container format. (~45 min)
- [ ] Commit notes (~10 min)

**Tuesday — EOF validation + [HFT] matching engine: match() impl**

- [ ] Stack validation per EIP-3670. (~30 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/match.rs` —
  `match_order(book: &mut OrderBook, incoming: Order) -> MatchResult`. Returns fills + remainder. Price-time priority. (~150 min)
- [ ] Commit + log (~10 min)

**Wednesday — Static relative jumps + [HFT] matching: deterministic event log**

- [ ] Implement EIP-4200 opcodes. (~120 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/event_log.rs` — every order placed, cancel, fill, partial-fill emits
  a `MatchEvent` written via `wal::Wal`. Replay reconstructs full book state. (~150 min)
- [ ] Commit + log (~10 min)

**Thursday — Functions (EIP-4750)**

- [ ] Implement CALLF, RETF, JUMPF in exec-vm. (~120 min)
- [ ] Commit + log (~10 min)

**Friday — EOF tests + Tempo PR #1 progress**

- [ ] Integrate EOF test vectors. (~30 min)
- [ ] [Tempo] 1.5 hrs at end of day: **Tempo PR #1 progress**. Continue implementation. EOF knowledge transfers. Working
  draft by EOD. (~300 min)
- [ ] Commit + log (~10 min)

**Saturday — revm EOF PR + [HFT] matching: risk pre-trade hook**

- [ ] If revm has EOF issues, contribute. (~30 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/risk.rs` — `RiskCheck` trait with pre-trade check (balance, position
  limit, fat-finger). Default impl `NoopRisk` rejects nothing; production impl plugs in W63. (~150 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 62 — `exec-vm` + `storage-trie` integration + [NEW] consensus-raft membership

**Monday — Integration design**

- [ ] Design how exec-vm uses storage-trie via Database trait. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — Implement integration + raft membership**

- [ ] Wire up the two crates. (~45 min)
- [ ] **Build**: `crates/consensus-raft/src/membership.rs` — joint-consensus membership change (C_old → C_old,new →
  C_new). Tested with one-at-a-time member additions. (~150 min)
- [ ] Commit + log (~10 min)

**Wednesday — Integration tests**

- [ ] End-to-end execution with real storage. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Benchmark integrated stack**

- [ ] Performance vs revm + reth storage. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — reth evm PR + Tempo PR #1 submit**

- [ ] Reth-side contribution. (~30 min)
- [ ] [Tempo] 1 hr at end of day: **Tempo PR #1 submit**. Finish, run their CI locally, open the PR with clear
  motivation + test plan. (~60 min)
- [ ] Commit + log (~10 min)

**Saturday — Crate maintenance + matching-engine fills emission**

- [ ] storage-trie fixes if needed. exec-vm polish. (~30 min)
- [ ] [HFT] **Build**: matching-engine emits `Fill` events through a `tokio::sync::broadcast` channel for downstream
  market-data fan-out. Channel uses `backpressure::BackpressureStrategy::DropOldest` for slow subscribers. (~150 min)
- [ ] [Tempo] 30 min: respond to any Tempo PR #1 review feedback. Don't let it sit. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 63 — Fuzz targets + [HFT] matching-engine v0.5 (single-symbol)

**Monday — Fuzz setup**

- [ ] Setup cargo-fuzz on exec-vm. First target on opcode sequences. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — Run fuzz, fix findings + [HFT] matching-engine: differential test against simple reference**

- [ ] Run fuzzer. Address crashes. (~30 min)
- [ ] [HFT] **Build**: `matching-engine/tests/diff_test.rs` — property test: random sequences of orders → compare
  matching-engine output to a slow-but-obviously-correct reference matcher (sorted Vec sweep). Must match exactly. (~150 min)
- [ ] Commit + log (~10 min)

**Wednesday — More fuzz targets + matching-engine v0.5 polish**

- [ ] Fuzz gas metering. Fuzz call operations. (~30 min)
- [ ] [HFT] criterion bench: 1M orders/sec single-symbol throughput on a single core; target P99 matching latency <10
  µs (we will tighten to <5 µs by W74). (~60 min)
- [ ] Commit + log (~10 min)

**Thursday — Differential fuzzing + matching-engine v0.5 tag**

- [ ] Fuzz exec-vm vs revm for consistency. (~30 min)
- [ ] [HFT] Tag `matching-engine v0.5.0` — single-symbol spot book, deterministic event log, basic risk hook. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — reth or revm PR**

- [ ] Contribution. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Docs pass**

- [ ] exec-vm documentation. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 20 min during ritual: review Tempo PR #1 status. If merged, log it and pick next candidate from bookmarked
  list. If still in review, address feedback. (~20 min)

---

### Week 64 — Revm performance PR push + [NEW] `consensus-bft` v0.1 scaffold

**[NEW] crate created**: `crates/consensus-bft/`. Layer-4 Byzantine-tolerant consensus. Used by `consensus-engine` for
fork-choice analogue and BFT-chain experimentation. v0.5 ships W68, v1.0 W73.

**Monday — Identify revm perf opportunity + consensus-bft scaffold**

- [ ] Deep profile revm in common scenarios. (~30 min)
- [ ] Re-open `EXEC_VM_PERF_BACKLOG.md` — specifically the SIMD candidate noted W60 Tue. If profiling confirms it's a
  hot spot in revm too, the optimization plus benchmark becomes a strong revm PR candidate this week. (~20 min)
- [ ] **Build**: `crates/consensus-bft/Cargo.toml` workspace member. Deps: `time`, `wal`, `p2p`. Read Buchman et al. "
  Latest gossip on BFT consensus" (Tendermint paper). (~20 min)
- [ ] **Build**: `crates/consensus-bft/src/lib.rs` with module headers: `propose.rs`, `prevote.rs`, `precommit.rs`,
  `lock.rs`, `evidence.rs`. (~150 min)
- [ ] Commit notes (~10 min)

**Tuesday — Design optimization**

- [ ] Plan revm perf approach. (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — Implement**

- [ ] Code revm optimization. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Benchmark**

- [ ] Measure improvement. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Submit revm PR**

- [ ] Clean PR. (~150 min)
- [ ] Commit + log (~10 min)

**Saturday — Respond to reviews + consensus-bft propose phase**

- [ ] Iterate revm PR. (~150 min)
- [ ] **Build**: `crates/consensus-bft/src/propose.rs` — round R proposer broadcasts proposal. Validators wait
  `propose_timeout` then vote. (~150 min)
- [ ] [Tempo] 30 min: address Tempo PR #1 feedback. If merged, claim Tempo PR #2 candidate from bookmarks. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 16 review**

---

## Month 17: Architectural Discussions + Reth evm Features + [HFT] matching-engine v0.7

### Week 65 — Architectural engagement + [HFT] matching-engine multi-symbol

**Monday — GitHub discussions + [HFT] matching: multi-symbol shard plan**

- [ ] Browse ongoing execution-layer architecture discussions. (~30 min)
- [ ] [HFT] Sketch: per-symbol Disruptor-style ring buffer, single-writer per symbol, multiple-reader fan-out. Each
  shard has its own `wal::Wal` segment. Symbol selection by hash. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Substantive comment + [HFT] symbol-sharded engine impl**

- [ ] Write substantive architectural comment. (~30 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/shard.rs` —
  `SymbolShard { book: OrderBook, wal: wal::Wal, event_tx: broadcast::Sender<MatchEvent> }`.
  `Engine { shards: HashMap<Symbol, Arc<Mutex<SymbolShard>>> }`. (~150 min)
- [ ] Commit notes (~10 min)

**Wednesday — Proposal draft + consensus-bft prevote**

- [ ] Draft small design proposal for reth evm. (~30 min)
- [ ] **Build**: `crates/consensus-bft/src/prevote.rs` — prevote phase. 2f+1 prevotes for a value advance the validator
  to precommit. (~150 min)
- [ ] Commit notes (~10 min)

**Thursday — Submit proposal**

- [ ] Post as GitHub discussion. (~30 min)
- [ ] Commit notes (~10 min)

**Friday — Engage discussion + Tempo discussions reconnaissance**

- [ ] Respond to feedback on Reth proposal. (~30 min)
- [ ] [Tempo] 30 min: scan Tempo discussions tab on GitHub. Pick one substantive thread to read fully (not to comment).
  Note in `notes/tempo_discussions.md` who's driving the design conversation and what the open questions are.
  Reconnaissance, not engagement. (~30 min)
- [ ] Commit notes (~10 min)

**Saturday — Reth PR + consensus-bft precommit**

- [ ] Storage or evm. (~30 min)
- [ ] **Build**: `crates/consensus-bft/src/precommit.rs` — precommit phase. 2f+1 precommits commits the block. (~150 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 66 — Reth evm feature + [Tempo] `tempo-tx-envelope` v0.1.0 + [HFT] perpetuals scaffold

**Monday — Feature identification**

- [ ] Find meaningful reth evm improvement. Design. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Implementation + [HFT] perpetuals contract types**

- [ ] Start coding reth feature. (~30 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/perps/contract.rs` —
  `LinearContract { mark_price, index_price, funding_rate, open_interest }` per TigerBeetle-style determinism. (~150 min)
- [ ] Commit + log (~10 min)

**Wednesday — Continue + [HFT] perpetuals margin**

- [ ] Reth feature. (~30 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/perps/margin.rs` — initial-margin / maintenance-margin formulas.
  Cross vs isolated. (~150 min)
- [ ] Commit + log (~10 min)

**Thursday — Tests**

- [ ] Reth feature tests. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Submit + `tempo-tx-envelope` v0.1.0 ship**

- [ ] Reth feature PR ready. (~150 min)
- [ ] [Tempo] 3 hrs split across the day: **`tempo-tx-envelope` v0.1.0 build**. Mirror `tempoxyz/tempo`'s primitives
  crate. Define `TempoTransaction` (EIP-2718 type 0x76) struct with fields: chain_id, nonce, max_fee_per_gas,
  max_priority_fee_per_gas, gas, calls: Vec<Call>, fee_token: Address, valid_before: Option<NonZeroU64>, valid_after:
  Option<NonZeroU64>, auth: Authorization. Use eth-rlp derive (W5). Reuse eth-primitives types. valid_before/valid_after
  timestamps source from `time::HybridLogicalClock`. (~180 min)
- [ ] [Tempo] Test: encode transaction with hard-coded fields, assert bytes match fixture pulled from tempoxyz/tempo's
  test data. (~30 min)
- [ ] [Tempo] Tag `tempo-tx-envelope v0.1.0` if tests pass. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Another storage PR (maintain velocity)**

- [ ] Commit + log (~10 min)
- [ ] [Tempo] 1 hr: test `tempo-tx-envelope` end-to-end. Use `tempo-foundry`'s `cast` to send transaction to Tempo
  testnet (stablecoins from docs.tempo.xyz faucet). Assert acceptance. If fails, debug — likely an RLP edge case. (~60 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 67 — exec-vm v1.0 prep + [NEW] consensus-raft v1.0 ship

**Monday — API stabilization + raft compaction**

- [ ] Review all exec-vm public APIs. Freeze signatures. (~30 min)
- [ ] **Build**: `crates/consensus-raft/src/snapshot.rs` — InstallSnapshot RPC + log compaction. Snapshots stored as
  `wal::Segment`s with special type tag. (~150 min)
- [ ] Commit + log (~10 min)

**Tuesday — Docs pass**

- [ ] exec-vm: every item documented. Examples. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Final benchmarks + consensus-raft v1.0 tag**

- [ ] Comprehensive bench suite. (~60 min)
- [ ] Tag `consensus-raft v1.0.0`. Ready to back matching-engine v1.0 (W74). (~5 min)
- [ ] Commit + log (~10 min)

**Thursday — DESIGN.md**

- [ ] Document architectural decisions in exec-vm. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Reth PR**

- [ ] Continue. (~30 min)
- [ ] [Tempo] 30 min: Tempo PR #2 progress check. By now you should have 2 Tempo PRs merged or 1 merged + 1 in review. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Crate polish + [HFT] matching-engine v0.7 (multi-symbol + perps scaffolded)**

- [ ] Final cleanup on exec-vm. (~30 min)
- [ ] [HFT] Tag `matching-engine v0.7.0` — multi-symbol + perpetuals contract types + margin scaffold. Raft replication
  arrives at v1.0 (W74). (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 68 — exec-vm v1.0 ship + [NEW] consensus-bft v0.5 ship

**Monday — Tag exec-vm v1.0**

- [ ] Tag `exec-vm v1.0.0`. Second Reth flagship deliverable. (~5 min)
- [ ] **Inheritance audit**: count exec-vm LOC. Calls into eth-primitives / eth-consensus / eth-storage-cache should be
  obvious. Native opcode/gas logic is ≤30% of LOC — that's healthy for an interpreter crate (the rest is plumbing into
  the rest of the workspace). (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — consensus-bft v0.5 ship + Blog if ready**

- [ ] **Build**: `crates/consensus-bft/src/lock.rs` — locking on prevoted value across rounds (Tendermint locking). (~150 min)
- [ ] Tag `consensus-bft v0.5.0`. (~5 min)
- [ ] Consider writing exec-vm intro blog. No pressure. (~30 min)
- [ ] [Tempo] Update M18 Tempo metrics row: PRs merged (target 10 — flag if below 6), TIPs read (target 10), crates (
  target 2 — tempo-tx-envelope ✓ + tempo-evm-ext scaffold ✓), maintainer relationships (target 2 — anyone who reviewed
  Tempo PRs). (~15 min)
- [ ] Commit + log (~10 min)

**Wednesday — Reth feature iteration**

- [ ] Address reviews on feature PR. (~60 min)
- [ ] Commit + log (~10 min)

**Thursday — More reth**

- [ ] Continue velocity. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Reviews given**

- [ ] Review 3 others' Reth PRs substantively. (~30 min)
- [ ] [Tempo] Review 2 others' Tempo PRs substantively. Even one substantive Tempo review is a relationship-warming
  signal worth more than three Reth reviews at this stage. Note who you reviewed; goes in Tempo maintainer tracker. (~20 min)
- [ ] Commit notes (~10 min)

**Saturday — Month 17 close**

- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 17 review**

---

## Month 18: Phase 4 Close + Consensus Prep + [NEW] `txn` v1.0 (2PC)

### Week 69 — Final execution PRs

**Monday — Final feature push**

- [ ] Last medium-sized feature. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — Implementation**

- [ ] Commit + log (~10 min)

**Wednesday — Tests + submit**

- [ ] Commit + log (~10 min)

**Thursday — Reviews**

- [ ] Commit + log (~10 min)

**Friday — Another small PR**

- [ ] Commit + log (~10 min)

**Saturday — Close outstanding work**

- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 70 — Consensus layer preview

**Monday — Ethereum consensus overview**

- [ ] Read Ethereum consensus layer intro. PoS high level. (~45 min)
- [ ] Commit notes (~10 min)

**Tuesday — Engine API spec preview**

- [ ] Read Engine API specification at high level. (~60 min)
- [ ] Commit notes (~10 min)

**Wednesday — Lighthouse survey**

- [ ] Browse Lighthouse code at high level. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Reth engine crate preview**

- [ ] Browse reth/crates/engine. (~45 min)
- [ ] Commit notes (~10 min)

**Friday — Reth consensus crate preview**

- [ ] Browse reth/crates/consensus. (~45 min)
- [ ] Commit notes (~10 min)

**Saturday — Phase 5 prep (consensus-engine already scaffolded W24)**

- [ ] Re-read consensus-engine empty lib.rs. Sketch module layout in notes/ (engine_api, fork_choice, payload_builder,
  jwt, builder_api, state_root_validator). Note fork-choice will lean on `consensus-bft` v1.0 primitives shipping W73. (~45 min)
- [ ] Identify which eth-* crates each module imports. Confirm dependency graph builds. (~20 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 71 — Phase 4 reflection + [HFT] matching-engine: ADL + funding scaffold

**Monday — Full Phase 4 assessment**

- [ ] Check exit criteria. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Metrics**

- [ ] Update North Star M18. Check 20+ execution PRs. (~15 min)
- [ ] Commit notes (~10 min)

**Wednesday — Relationship update**

- [ ] Which maintainers engaged. Depth. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Blog consideration + [HFT] funding scaffold**

- [ ] Phase 4 retrospective. No deadline. (~30 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/perps/funding.rs` — funding-rate calc via TWAP of (mark - index).
  Hourly tick. (~150 min)
- [ ] Commit notes (~10 min)

**Friday — Wrap + [HFT] liquidation scaffold**

- [ ] Close outstanding Reth PRs. (~30 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/perps/liquidation.rs` — liquidation threshold check (maintenance
  margin breach). Partial liquidation. Insurance-fund hook. (~150 min)
- [ ] Commit + log (~10 min)

**Saturday — Rest prep + [HFT] ADL scaffold**

- [ ] Light day. (~30 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/perps/adl.rs` — Auto-Deleveraging queue (rank by profit × leverage).
  Skeleton only; complete in Phase 5 (W74). (~150 min)
- [ ] Commit notes (~10 min)

**Sunday — Rest**

---

### Week 72 — Transition week + [NEW] `txn` v1.0 (2PC for distributed)

**Monday — Mental prep Phase 5 + txn 2PC design**

- [ ] Read Phase 5 section. Outline Month 19. (~45 min)
- [ ] **Build**: `crates/txn/src/two_phase_commit.rs` — `TwoPhaseCoordinator` and `TwoPhaseParticipant`. Standard 2PC
  with Prepare / Commit messages and timeout-based abort. (~105 min)
- [ ] Commit notes (~10 min)

**Tuesday — Reading list for consensus + txn 2PC test**

- [ ] Compile reading list. (~30 min)
- [ ] Test: 3-participant 2PC, kill coordinator mid-prepare, participants timeout and abort. Restart coordinator,
  recovers from txn log to retry from last-known state. (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — Reach out to Lighthouse folks**

- [ ] If any connections, warm up. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Maintenance on previous crates + txn v1.0 tag**

- [ ] storage-trie, exec-vm bug fixes. (~30 min)
- [ ] Tag `txn v1.0.0`. Adds 2PC for distributed transactions. Ready for `ledger-deterministic` distributed mode (Phase
  5). (~5 min)
- [ ] Commit + log (~10 min)

**Friday — Final exec-vm polish**

- [ ] Any remaining items. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Month 18 close**

- [ ] Final PRs. (~30 min)
- [ ] **Phase 4 deliverable check**: `exec-vm v1.0` ✓, `consensus-raft v1.0` ✓, `consensus-bft v0.5` ✓,
  `matching-engine v0.7` ✓, `tempo-tx-envelope v0.1.0` ✓, `tempo-evm-ext` scaffold ✓, `p2p v0.5` ✓, `txn v1.0` ✓. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest**

- [ ] Phase 5 starts tomorrow. (~30 min)

---

# PHASE 5: CONSENSUS + ENGINE API + [HFT] PRIMARY (Month 19-24)

**Reth Deliverable**: `consensus-engine` v1.0 (W91) + end-to-end integration capable of syncing Sepolia (W85).
**[NEW] HFT Deliverables**: `consensus-bft` v1.0 (W73), `matching-engine` v1.0 (W74), `messaging-aeron` v0.5 (W79),
`ledger-deterministic` v0.5 (W83), `marketdata-kernelbypass` v0.5 (W90).
**[NEW] Tempo Deliverables**: `tempo-payment-lane` scaffold (W83) → v0.1.0 (W91); `tempo-evm-ext` v0.1.0 (W91).
**M24 Decision Gate (W96)**: 5 paths (A: extend Reth, B: post-Reth systems, C: catch-up, D: Tempo pivot conditional, E:
HFT destination-tier IC track).

> Tempo Phase 5 budget: 5-7 hrs/wk. Payment lane design W82 Fri. `tempo-payment-lane` scaffold W83 Wed. Both Tempo
> crates v0.1.0 W91 Thu.
> HFT Phase 5 budget: 12-15 hrs/wk (HFT becomes primary M19-M34, sharing budget with consensus-engine work).
> Reth Phase 5: consensus-engine flagship continues primary in name; in practice budget is split 50/50 with HFT from W74
> onward.

Three-crate Reth integration target (W85 Sepolia sync): consensus-engine orchestrates eth-network-codec → block
ingestion → eth-stage::Pipeline (driving exec-vm + storage-trie + eth-trie::StateRoot) → engine_api for CL coordination.

HFT five-crate integration target (W90 paper-trade rig): matching-engine ←consensus-raft for replication,
←messaging-aeron for market-data fan-out, ←wal+recovery for crash safety, →ledger-deterministic for settlement,
←marketdata-kernelbypass for inbound external feed handling.

## Month 19: Engine API + [NEW] consensus-bft v1.0 + [HFT] matching-engine v1.0 (raft-replicated)

### Week 73 — Engine API spec + [NEW] consensus-bft v1.0 + [HFT] matching-engine v0.7 → wire raft

**Monday — Engine API full read part 1 + consensus-bft fork-choice**

- [ ] Read Engine API spec sections 1-3. (~60 min)
- [ ] **Build**: `crates/consensus-bft/src/fork_choice.rs` — fork-choice rule for chained BFT blocks. Honest validators
  with locking guarantees agreement under partial sync. (~150 min)
- [ ] Commit notes (~10 min)

**Tuesday — Engine API full read part 2 + consensus-bft evidence**

- [ ] Read Engine API sections 4-6. (~45 min)
- [ ] **Build**: `crates/consensus-bft/src/evidence.rs` — slashable-fault collection. Double-sign detector. Equivocation
  proof. (~150 min)
- [ ] Commit notes (~10 min)

**Wednesday — newPayload deep + consensus-bft v1.0 tag**

- [ ] Study newPayload V1, V2, V3, V4. (~30 min)
- [ ] Tag `consensus-bft v1.0.0`. (~5 min)
- [ ] Commit notes (~10 min)

**Thursday — forkchoiceUpdated deep**

- [ ] Study fcU variants. (~30 min)
- [ ] Commit notes (~10 min)

**Friday — getPayload deep + [HFT] matching-engine: raft integration design**

- [ ] Study getPayload variants. (~30 min)
- [ ] [HFT] Design: matching-engine commands serialized as `MatchCommand` enum → submitted to consensus-raft → applied
  in order on each replica. Raft log entries ARE the deterministic event log. (~30 min)
- [ ] Commit notes (~10 min)

**Saturday — JWT auth**

- [ ] Study JWT auth used by Engine API. (~30 min)
- [ ] [Tempo] 30 min: while JWT is fresh, read how Tempo handles engine API auth. Tempo's CL-EL split is different (
  validator set is permissioned). Note implications in `notes/tempo_engine_diff.md`. (~30 min)
- [ ] Commit notes (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 74 — Reth engine crate + [HFT] `matching-engine` v1.0 (raft-replicated)

**Monday — reth-engine structure + matching-engine: raft wire-up**

- [ ] Browse reth/crates/engine. Map files. (~45 min)
- [ ] [HFT] **Build**: `crates/matching-engine/src/replication.rs` — `RaftBackedEngine` wraps `Engine` from W67. Submits
  incoming `MatchCommand` to a `consensus_raft::RaftNode`. Replicas receive `apply(cmd)` callbacks and mutate their
  local OrderBook deterministically. (~150 min)
- [ ] Commit notes (~10 min)

**Tuesday — Engine tree + matching-engine: raft test**

- [ ] Read engine tree implementation. Block tree for forks. (~45 min)
- [ ] [HFT] Test: 3-replica cluster, leader takes 100k orders, kill leader mid-burst, new leader elected, replicated
  state matches. (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — Payload builder**

- [ ] Read reth payload builder. (~45 min)
- [ ] Commit notes (~10 min)

**Thursday — First engine PR + [HFT] matching-engine v1.0 tag**

- [ ] Find docs or small fix in reth-engine. (~30 min)
- [ ] [HFT] **Inheritance audit**: matching-engine LOC count vs LOC calling into time / backpressure / wal / recovery /
  consensus-raft / messaging-aeron-trait-shapes. Target ≥70%. Adjust scope if below. (~30 min)
- [ ] [HFT] Tag `matching-engine v1.0.0`. Multi-symbol + perpetuals (margin, funding, liquidation, ADL) +
  raft-replicated. (~30 min)
- [ ] [Tempo] 30 min: scan Tempo's engine-API-adjacent PRs to see what kinds of issues are open there. Bookmark
  candidates. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — `consensus-engine::engine_api` module skeleton**

- [ ] Create `consensus-engine/src/engine_api/{mod.rs, server.rs, types.rs}`. Define EngineApi trait with V1-V4 method
  signatures. Wire eth-network-codec::Codec for JSON-RPC framing. (~30 min)
- [ ] Re-export eth-rpc-types request/response types. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — JWT auth in `consensus-engine::engine_api::jwt`**

- [ ] Implement HS256 JWT auth middleware. Test against fixture token from Lighthouse deployment. (~30 min)
- [ ] [Tempo] 30 min: confirm JWT module is agnostic enough to work for both Ethereum-style engine API and Tempo's
  variant. If not, parameterize. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 75 — `consensus-engine`: core methods + [HFT] matching-engine: post-v1 polish

**Monday — newPayload implementation**

- [ ] Implement newPayload V3 handler in consensus-engine. (~120 min)
- [ ] Commit + log (~10 min)

**Tuesday — Payload validation**

- [ ] Block header validation in consensus-engine. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — forkchoiceUpdated**

- [ ] Implement fcU handler. Use `consensus-bft::fork_choice` as the underlying rule engine (parameterized; Ethereum's
  actual fork choice differs but the abstraction holds). (~120 min)
- [ ] Commit + log (~10 min)

**Thursday — getPayload**

- [ ] Implement getPayload. (~120 min)
- [ ] Commit + log (~10 min)

**Friday — Storage + engine integration**

- [ ] Wire consensus-engine up with storage-trie. (~45 min)
- [ ] Commit + log (~10 min)

**Saturday — Engine + exec-vm integration**

- [ ] Execute payload using exec-vm. (~30 min)
- [ ] [Tempo] 1 hr: **TIP-1031 reads-side wiring**. Confirm consensus-engine engine_newPayload handler can carry
  Tempo-style consensus-context field without breaking upstream Ethereum path. Cargo features (
  `tempo-consensus-context`) guard the field. Groundwork for W82-83 payment lane work. (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 76 — Lighthouse CL perspective + [NEW] `messaging-aeron` v0.1 scaffold

**[NEW] crate created**: `crates/messaging-aeron/`. Layer-4 reliable IPC + UDP messaging. v0.5 W79.

**Monday — Lighthouse code survey + messaging-aeron scaffold**

- [ ] Browse Lighthouse execution interaction layer. (~30 min)
- [ ] **Build**: `crates/messaging-aeron/Cargo.toml` workspace member. Deps: `time`, `backpressure`, `bufpool`, `p2p` (
  peer discovery only). (~20 min)
- [ ] **Build**: `crates/messaging-aeron/src/lib.rs` with module headers: `media_driver.rs`, `term_buffer.rs`,
  `flow_control.rs`, `nak.rs`, `transport_ipc.rs`, `transport_udp.rs`. (~105 min)
- [ ] Commit notes (~10 min)

**Tuesday — Lighthouse Engine API client + messaging-aeron media driver**

- [ ] Read Lighthouse's side of Engine API. (~45 min)
- [ ] **Build**: `crates/messaging-aeron/src/media_driver.rs` —
  `MediaDriver { shm_dir: PathBuf, conductor: ConductorThread }`. Conductor owns subscription/publication state. (~105 min)
- [ ] Commit notes (~10 min)

**Wednesday — Prysm perspective**

- [ ] Read Prysm equivalent (less depth). (~45 min)
- [ ] Commit notes (~10 min)

**Thursday — CL/EL lifecycle**

- [ ] Map full CL/EL communication flow. (~30 min)
- [ ] Commit notes (~10 min)

**Friday — Another reth engine PR**

- [ ] Continue velocity. (~30 min)
- [ ] [Tempo] 30 min: pick Tempo PR candidate from W74 bookmarks. If good one available, claim and begin. Otherwise push
  to W78. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Crate: connection handling**

- [ ] Websocket/HTTP Engine API transport in consensus-engine. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 19 review**

---

## Month 20: Full Engine API + STF Validation + [NEW] `messaging-aeron` v0.5

### Week 77 — State transition validation + messaging-aeron term buffer

**Monday — STF theory + messaging-aeron term-buffer design**

- [ ] Read state transition function theory. (~45 min)
- [ ] **Build**: `crates/messaging-aeron/src/term_buffer.rs` — fixed-size (default 1 MiB) ring with frame headers.
  Single writer, single reader per term. Three terms rotate (active, in-recovery, dirty). Lock-free. (~150 min)
- [ ] Commit notes (~10 min)

**Tuesday — Consensus rules in execution + messaging-aeron flow control**

- [ ] What execution layer validates per consensus rules. (~30 min)
- [ ] **Build**: `crates/messaging-aeron/src/flow_control.rs` — sliding window. Subscriber publishes a position;
  publisher cannot advance past last-position + window. Uses `backpressure::CreditFlowControl` underneath. (~105 min)
- [ ] Commit notes (~10 min)

**Wednesday — Block validation**

- [ ] Implement block validation in consensus-engine. (~120 min)
- [ ] Commit + log (~10 min)

**Thursday — Receipt validation**

- [ ] Receipt consistency checks. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Gas limit validation + messaging-aeron IPC transport**

- [ ] Block gas limit checks. (~30 min)
- [ ] **Build**: `crates/messaging-aeron/src/transport_ipc.rs` — shared-memory regions over the term buffer. Hot path:
  no syscalls per message. (~105 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth PR + messaging-aeron IPC bench**

- [ ] Engine or consensus area. (~30 min)
- [ ] criterion: 10M messages through IPC. Target <1µs P99 single-hop. (~60 min)
- [ ] [Tempo] 1 hr: continue Tempo PR or claim new one. Aim for Tempo PR #3-4 merged by end of M20. (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 78 — Fork choice integration + messaging-aeron UDP unicast

**Monday — Fork choice theory + UDP unicast design**

- [ ] Read fork choice rule (LMD-GHOST, Casper FFG). (~45 min)
- [ ] Design UDP unicast: socket per subscriber, batching, framing. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Reth fork choice code + messaging-aeron UDP impl**

- [ ] Read reth's fork choice handling. (~45 min)
- [ ] **Build**: `crates/messaging-aeron/src/transport_udp.rs` — UDP socket pool, recvmmsg/sendmmsg batching, MTU-aware
  fragmentation. (~105 min)
- [ ] Commit notes (~10 min)

**Wednesday — Crate: fork choice + messaging-aeron NAK protocol**

- [ ] Implement consensus-engine fork choice processing. (~120 min)
- [ ] **Build**: `crates/messaging-aeron/src/nak.rs` — gap detection by sequence number, NAK back to publisher, replay
  from term buffer's recovery slot. (~105 min)
- [ ] Commit + log (~10 min)

**Thursday — Safe/finalized tracking**

- [ ] Track safe, finalized, head blocks. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Reorg detection**

- [ ] Detect reorgs from fork choice updates. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — More reth PRs + messaging-aeron NAK test**

- [ ] Consensus or engine. (~30 min)
- [ ] Test: induce 5% UDP packet loss, NAK recovery achieves zero-loss delivery within bounded delay. (~30 min)
- [ ] [Tempo] 1 hr: Tempo PR work. (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] [Tempo] 20 min: Tempo releases skim. (~20 min)

---

### Week 79 — Reorg handling + [NEW] `messaging-aeron` v0.5 ship

**Monday — Reorg theory + messaging-aeron multi-subscriber**

- [ ] Deep understand reorg handling in execution. (~30 min)
- [ ] **Build**: `crates/messaging-aeron/src/subscription.rs` — multi-subscriber fan-out over term buffer. Each
  subscriber has its own position cursor. Slow subscribers don't block fast ones (configurable: lag-tolerant or kicked). (~105 min)
- [ ] Commit notes (~10 min)

**Tuesday — State rollback + messaging-aeron v0.5 tag**

- [ ] Implement state rollback on reorg. Leverage storage-trie snapshots. (~120 min)
- [ ] Tag `messaging-aeron v0.5.0`. Term buffer + IPC + UDP unicast + flow control + NAK recovery. (~5 min)
- [ ] Commit + log (~10 min)

**Wednesday — Receipt reindexing**

- [ ] Handle receipt/log reindexing. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Transaction re-pool**

- [ ] Handle moving txs back to mempool on reorg. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Reorg integration tests**

- [ ] Test various reorg scenarios. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth PR + [HFT] matching-engine wires messaging-aeron**

- [ ] Reth contribution. (~30 min)
- [ ] [HFT] **Build**: matching-engine market-data fan-out now goes via `messaging-aeron::Publication`. Subscribers
  receive L2 deltas over IPC for in-process clients, UDP unicast for out-of-process. (~150 min)
- [ ] [Tempo] 1 hr: Tempo PR. (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 80 — Multi-branch state + [NEW] `ledger-deterministic` v0.1 scaffold

**[NEW] crate created**: `crates/ledger-deterministic/`. TigerBeetle-style deterministic state machine + double-entry.
v0.5 W83.

**Monday — Multi-branch theory + ledger scaffold**

- [ ] Maintaining state across forks. (~30 min)
- [ ] **Build**: `crates/ledger-deterministic/Cargo.toml` workspace member. Deps: `time` (monotonic only, no wall
  clock!), `wal`, `recovery`, `txn`. (~20 min)
- [ ] **Build**: `crates/ledger-deterministic/src/lib.rs` with module headers: `state_machine.rs`, `account.rs`,
  `transfer.rs`, `journal.rs`. (~105 min)
- [ ] Commit notes (~10 min)

**Tuesday — Branch state design + ledger account schema**

- [ ] Design multi-branch state in consensus-engine. (~30 min)
- [ ] **Build**: `crates/ledger-deterministic/src/account.rs` —
  `Account { id: u128, ledger: u32, code: u16, flags: u16, debits_pending: u64, debits_posted: u64, credits_pending: u64, credits_posted: u64, timestamp: u64 }` —
  TigerBeetle account layout. (~105 min)
- [ ] Commit + log (~10 min)

**Wednesday — Implement multi-branch + ledger transfer schema**

- [ ] Code branch state management. (~30 min)
- [ ] **Build**: `crates/ledger-deterministic/src/transfer.rs` —
  `Transfer { id: u128, debit_account_id: u128, credit_account_id: u128, amount: u64, code: u16, flags: u16, timestamp: u64 }`.
  Constraints: same ledger, both accounts exist, no overflow. (~105 min)
- [ ] Commit + log (~10 min)

**Thursday — Tests**

- [ ] consensus-engine multi-branch tests. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Integration with exec-vm**

- [ ] Speculative execution across branches. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth PR + ledger: state machine framework**

- [ ] Reth PR. (~150 min)
- [ ] **Build**: `crates/ledger-deterministic/src/state_machine.rs` —
  `StateMachine { accounts: BTreeMap<u128, Account>, ... }`. `apply(op: Op) -> Result<OpResult>` is deterministic: no
  `time::wall()`, no `rand`, no f64. Only `time::Monotonic` is allowed and only for instrumentation, never for logic. (~150 min)
- [ ] [Tempo] 1 hr: Tempo PR. Tempo PR count should be 6-8 merged; flag if below 4. (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 20 review**

---

## Month 21: PBS + Builder API + [NEW] ledger v0.5 + payment-lane

### Week 81 — Invalid payload handling + Tempo payment-lane prior-art read

**Monday — Invalid payload scenarios**

- [ ] Catalog all invalid payload cases from spec. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Invalid header**

- [ ] Handle invalid headers. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Invalid transactions**

- [ ] Handle invalid tx in payload. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Invalid state root**

- [ ] Handle state root mismatch. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Latest valid hash logic**

- [ ] Implement LVH tracking. (~120 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth PR + Tempo payment-lane prior-art read + [HFT] ledger: journaled apply**

- [ ] Reth PR work. (~150 min)
- [ ] [HFT] **Build**: `crates/ledger-deterministic/src/journal.rs` — every `apply(op)` writes a `JournalRecord` to wal
  BEFORE mutating state. recovery::Recovery replays journal on open. Output: a state machine that survives crashes
  byte-identically. (~105 min)
- [ ] [Tempo] 2 hrs at end of day: **payment-lane prior-art read** (no code). Read tempoxyz/tempo's payment-lane /
  payload-builder implementation end-to-end. Locate the lane reservation logic — likely under `consensus/`,
  `payload-builder/`, or `block-builder/`. For each non-obvious choice (priority queue shape, fairness rule,
  unused-reservation handling, tip20-detection mechanism), one sentence in `notes/payment_lane_prior_art.md` capturing
  what they did and your first guess at why. This is the reading scaffold for W82 Fri's design sketch — do NOT design
  your own yet. (~120 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 82 — PBS + Builder API + [Tempo] payment-lane design

**Monday — PBS theory**

- [ ] Read PBS (Proposer-Builder Separation) spec. (~60 min)
- [ ] Commit notes (~10 min)

**Tuesday — MEV-Boost architecture**

- [ ] Read MEV-Boost architecture. (~45 min)
- [ ] Commit notes (~10 min)

**Wednesday — Builder API spec**

- [ ] Read Builder API specification. (~60 min)
- [ ] Commit notes (~10 min)

**Thursday — Builder API in reth**

- [ ] Check reth's builder API support. (~30 min)
- [ ] Commit notes (~10 min)

**Friday — Crate: Builder API compat + Tempo payment lane design**

- [ ] Design builder API support in consensus-engine. (~30 min)
- [ ] [Tempo] 2 hrs: **Tempo design-partner-facing feature design start**. Open `notes/payment_lane_prior_art.md` from
  W81 Sat as reference. Design "payment lane" support in payload builder. Rule: configurable percentage of block gas (
  default 30%) reserved for TIP-20 transfers. If TIP-20 demand below reservation, rest is general. If above, TIP-20
  wins. Sketch algorithm in `notes/payment_lane_design.md`: priority queue per category, fairness, what happens when
  reservation fully unused (give to general or burn slot). For each design choice, note whether it matches upstream
  Tempo or intentionally diverges (with reason). (~120 min)
- [ ] Commit + log (~10 min)

**Saturday — Implementation start**

- [ ] Begin builder API endpoints in consensus-engine. (~30 min)
- [ ] [Tempo] 30 min: review payment lane sketch from yesterday. Identify the 2-3 hardest design choices. Note them;
  don't solve yet. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 83 — Builder API impl + [Tempo] `tempo-payment-lane` scaffold + [NEW] ledger v0.5

**Monday — Header submissions**

- [ ] Implement header submission flow in consensus-engine. (~120 min)
- [ ] Commit + log (~10 min)

**Tuesday — Block submissions + ledger v0.5 polish**

- [ ] Implement block submission flow. (~120 min)
- [ ] [HFT] Polish ledger-deterministic: snapshot/restore + replay test. 1M transfers, crash mid-batch, recovery yields
  byte-identical state. (~45 min)
- [ ] Commit + log (~10 min)

**Wednesday — Builder client + `tempo-payment-lane` scaffold + ledger v0.5 tag**

- [ ] Implement builder client perspective. (~120 min)
- [ ] [HFT] Tag `ledger-deterministic v0.5.0`. Deterministic SM + double-entry + journal. (~30 min)
- [ ] [Tempo] 2 hrs: **`tempo-payment-lane` scaffold**. Create `crates/tempo-payment-lane/` workspace member depending
  on consensus-engine. Define `LaneStrategy` trait:
  `fn select_transactions(&self, pool: &[PoolTx], gas_limit: u64) -> Vec<PoolTx>`. Empty default impl +
  `TempoLaneStrategy { tip20_reservation_pct: u8 }` skeleton. (~120 min)
- [ ] Commit + log (~10 min)

**Thursday — Builder integration tests + lane strategy impl**

- [ ] Existing builder integration tests. (~30 min)
- [ ] [Tempo] 1 hr: implement `TempoLaneStrategy::select_transactions` for simple case: split pool into tip20 vs general
  buckets (check fee_token field from tempo-tx-envelope), fill reservation from tip20 first, then general. Test against
  synthetic pool of 100 txs. (~60 min)
- [ ] Commit + log (~10 min)

**Friday — Reth PR + lane edge case**

- [ ] Reth PR. (~150 min)
- [ ] [Tempo] 1 hr: continue tempo-payment-lane. Handle edge case where tip20 reservation is unused — give to general. (~60 min)
- [ ] Commit + log (~10 min)

**Saturday — Flashbots docs + diff against upstream**

- [ ] Study Flashbots additional docs. (~30 min)
- [ ] [Tempo] 1.5 hrs: **Diff your prototype against upstream**. Read Tempo's payment-lane implementation in
  `tempoxyz/tempo`. Compare to your tempo-payment-lane prototype. Note 3 design choices that differ. For each, decide:
  port upstream, keep yours, or document trade. Add to payment_lane_design.md. (~300 min)
- [ ] Commit notes (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 84 — Test harness + [HFT] matching-engine + ledger settlement integration

**Monday — Test harness design**

- [ ] Design CL/EL test harness for consensus-engine. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Deterministic CL**

- [ ] Implement mock CL for testing. (~120 min)
- [ ] Commit + log (~10 min)

**Wednesday — Scenario DSL + matching-engine settlement hook**

- [ ] Define DSL for test scenarios. (~30 min)
- [ ] [HFT] **Build**: matching-engine fills emit `Settlement` events. A new bridge in
  `crates/matching-engine/src/settle.rs` maps Settlement → ledger-deterministic Transfers. Now matching, replication,
  and accounting are end-to-end. (~150 min)
- [ ] Commit + log (~10 min)

**Thursday — Reorg scenarios**

- [ ] Reorg simulation tests in consensus-engine. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Engine API conformance**

- [ ] Run crate against spec conformance tests. (~30 min)
- [ ] [Tempo] 30 min: also run stack against any public Tempo conformance suite if one exists. If not, note as gap. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth PR + [HFT] full-stack integration test**

- [ ] Reth PR. (~150 min)
- [ ] [HFT] Test: 3-replica matching-engine + ledger. Submit 100k orders. Verify all replicas converge AND ledger
  balances reconcile to zero net change. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 21 review**

---

## Month 22: Cross-Subsystem Features + Sepolia Sync + [NEW] `marketdata-kernelbypass`

### Week 85 — Three-crate Reth integration push (Sepolia) + [NEW] marketdata-kernelbypass v0.1 scaffold

**[NEW] crate created**: `crates/marketdata-kernelbypass/`. Layer-4 ingestion-path primitive. v0.5 W90.

**Monday — Integration architecture + marketdata scaffold**

- [ ] Design toy execution client using all 3 Reth crates (storage-trie, exec-vm, consensus-engine). (~30 min)
- [ ] **Build**: `crates/marketdata-kernelbypass/Cargo.toml` workspace member. Deps: `time`, `backpressure`. Optional
  features: `io-uring`, `af-xdp`. (~20 min)
- [ ] **Build**: `crates/marketdata-kernelbypass/src/lib.rs` with module headers: `epoll.rs`, `io_uring.rs`,
  `af_xdp.rs`, `feed_handler.rs`, `parse.rs`. (~150 min)
- [ ] Commit + log (~10 min)

**Tuesday — Boot sequence**

- [ ] Implement node startup. (~120 min)
- [ ] Commit + log (~10 min)

**Wednesday — Engine API → execution → storage flow**

- [ ] End-to-end flow. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Sync from testnet**

- [ ] Attempt Sepolia sync using own stack (PRIMARY GOAL — THE Phase 5 deliverable). (~30 min)
- [ ] [Tempo] **Secondary stretch goal** (cap at 4 hrs total this week, not just today): attempt Tempo testnet sync
  using your stack with tempo-tx-envelope + tempo-evm-ext + tempo-payment-lane plugged in. Document blockers in
  `notes/tempo_sync_blockers.md`. Do NOT let this eat Sepolia time. If Sepolia isn't working, Tempo gets zero time. (~240 min)
- [ ] Commit + log (~10 min)

**Friday — Debug failures + marketdata: epoll baseline**

- [ ] Fix Sepolia sync issues. (~30 min)
- [ ] **Build**: `crates/marketdata-kernelbypass/src/epoll.rs` — epoll-based receiver. Baseline for io_uring/AF_XDP
  comparisons. (~150 min)
- [ ] [Tempo] 30 min only if Sepolia is green: continue Tempo sync attempt. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — More debugging**

- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 86 — Cross-subsystem reth feature + marketdata: io_uring path

**Monday — Feature identification + marketdata: io_uring design**

- [ ] Find reth feature touching engine + storage. (~30 min)
- [ ] Read tokio-uring + io_uring liburing docs. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Design + io_uring impl**

- [ ] consensus-engine cross-subsystem feature design. (~30 min)
- [ ] **Build**: `crates/marketdata-kernelbypass/src/io_uring.rs` — `IoUringReceiver` using `io_uring` crate directly.
  Submit-queue / completion-queue patterns. Multi-shot read with fixed buffers. (~150 min)
- [ ] Commit notes (~10 min)

**Wednesday — Implementation**

- [ ] Reth feature. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Continue**

- [ ] Commit + log (~10 min)

**Friday — Tests**

- [ ] Commit + log (~10 min)

**Saturday — Submit + marketdata: parser scaffold**

- [ ] Submit Reth feature PR. (~30 min)
- [ ] **Build**: `crates/marketdata-kernelbypass/src/parse.rs` — generic `FeedParser` trait. Two reference impls:
  `ItchParser` (NASDAQ ITCH 5.0 subset), `FixParser` (FIX 4.4 binary subset). (~105 min)
- [ ] [Tempo] 30 min: Tempo PR work. Target Tempo PR count by end of M22: 12-15 merged. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 87 — PR reviews velocity + marketdata: feed handler

**Monday — Review 2 PRs substantively**

- [ ] [Tempo] 1 Reth + 1 Tempo PR. Mix sources from now on. (~150 min)
- [ ] Commit notes (~10 min)

**Tuesday — Review 2 more + marketdata: feed handler**

- [ ] [Tempo] 1 Reth + 1 Tempo PR. (~150 min)
- [ ] **Build**: `crates/marketdata-kernelbypass/src/feed_handler.rs` — `FeedHandler<P: FeedParser, T: Transport>` glues
  receiver → parse → emit. Emit goes through `messaging-aeron::Publication`. (~105 min)
- [ ] Commit + log (~10 min)

**Wednesday — Review discussion comments**

- [ ] Engage design discussions. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Reth PR**

- [ ] Commit + log (~10 min)

**Friday — Review 2 more**

- [ ] [Tempo] 1 Reth + 1 Tempo PR. (~150 min)
- [ ] Commit notes (~10 min)

**Saturday — Crate maintenance**

- [ ] All Reth + HFT + Tempo crates. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 88 — Devcon attendance

**Monday-Friday — Devcon (dates vary)**

- [ ] Attend Devcon. Meet maintainers in person. Side events. (~30 min)
- [ ] [Tempo] If any Tempo team members or design partners at Devcon, request 1-on-1 in advance. Same priority as Reth
  core 1-on-1s. Update Tempo maintainer tracker after. (~30 min)
- [ ] Notes. (~30 min)

**Saturday — Travel home**

**Sunday — Post-conference ritual**

- [ ] Update Reth + Tempo trackers. (~15 min)
- [ ] Follow-ups. (~30 min)

---

## Month 23: Mentorship + RFC + [NEW] marketdata v0.5 + flagship Tempo crates v0.1.0

### Week 89 — RFC consideration + marketdata: AF_XDP path

**Monday — Identify RFC opportunity + AF_XDP design**

- [ ] Find area needing design doc in Reth. (~45 min)
- [ ] Read XDP + AF_XDP kernel docs. The `xdp` crate. (~60 min)
- [ ] Commit notes (~10 min)

**Tuesday — Draft RFC + Tempo TIP decision + AF_XDP impl**

- [ ] Write initial draft. (~30 min)
- [ ] [Tempo] Decide RFC target this week: Reth proposal OR Tempo TIP. Tempo TIPs are numbered like EIPs; bar is high
  but process is open. If your tempo-payment-lane prototype surfaced a real design issue (W83 Sat diff), a Tempo TIP
  draft is the right vehicle. Otherwise Reth RFC. Commit to one — don't do both. (~30 min)
- [ ] **Build**: `crates/marketdata-kernelbypass/src/af_xdp.rs` — AF_XDP socket using `xdp` crate. Zero-copy receive
  path; requires CAP_NET_ADMIN. (~150 min)
- [ ] Commit notes (~10 min)

**Wednesday — Refine RFC**

- [ ] Iterate. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Post RFC**

- [ ] Post as GitHub discussion (Reth OR Tempo per Tuesday's decision). (~30 min)
- [ ] Commit notes (~10 min)

**Friday — Respond to feedback + AF_XDP bench**

- [ ] Engage commenters. (~30 min)
- [ ] criterion: epoll vs io_uring vs AF_XDP at 10M, 50M, 100M packets/sec on a single NIC queue. Plot. (~60 min)
- [ ] Commit notes (~10 min)

**Saturday — Reth PR**

- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 90 — Mentorship practice + [NEW] marketdata-kernelbypass v0.5 ship

**Monday — Identify newcomer + marketdata polish**

- [ ] Find newer contributor in Telegram. Offer help on their first PR. (~180 min)
- [ ] marketdata-kernelbypass polish: error handling, tracing spans. (~30 min)
- [ ] [Tempo] 15 min: identify any newcomer on Tempo side. Same offer. (~15 min)
- [ ] Commit notes (~10 min)

**Tuesday — Help them + marketdata v0.5 tag**

- [ ] Pair review on Reth side. (~30 min)
- [ ] Tag `marketdata-kernelbypass v0.5.0`. epoll + io_uring + AF_XDP + ITCH/FIX parsers + feed handler. (~5 min)
- [ ] Commit notes (~10 min)

**Wednesday — Another mentee + matching-engine end-to-end integration**

- [ ] Help another newcomer. (~30 min)
- [ ] [HFT] **Integration test**: marketdata-kernelbypass receives synthetic ITCH feed at 5M msg/s → matching-engine
  ingests → fills emit via messaging-aeron → ledger settles. End-to-end on one machine. Capture latencies at each hop. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Crate PR**

- [ ] Reth contribution. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Consensus-engine v1.0 prep**

- [ ] API review. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Docs pass**

- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 91 — consensus-engine v1.0 ship + [Tempo] both Tempo crates v0.1.0

**Monday — Final benchmarks**

- [ ] Commit + log (~10 min)

**Tuesday — DESIGN.md**

- [ ] consensus-engine DESIGN.md showing inheritance tree. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Release tag**

- [ ] `consensus-engine v1.0.0` tag. Third Reth flagship deliverable. (~30 min)
- [ ] **Inheritance audit**: consensus-engine LOC vs LOC calling into eth-consensus / eth-stage / exec-vm /
  storage-trie / consensus-bft. Target ≥70%. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Tempo crates ship**

- [ ] Integration example: full 3-crate Reth example. (~30 min)
- [ ] [Tempo] 1 hr: **Tag `tempo-evm-ext v0.1.0`**. Finalize at least TIP-1020 P256/WebAuthn precompile impls registered
  against exec-vm's registry. Test that registration works without forking exec-vm. (~60 min)
- [ ] [Tempo] 30 min: **Tag `tempo-payment-lane v0.1.0`**. Finalize lane reservation strategy from W83. Update README
  documenting strategy and trade-offs vs upstream Tempo. (~30 min)
- [ ] [Tempo] Update workspace root README to document all 3 Tempo crates alongside the 13 Reth crates AND the 6 HFT
  crates AND the 8 Layer-1/2/3/4 primitive crates. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Blog consideration**

- [ ] If ready, draft consensus-engine post. (~30 min)
- [ ] [Tempo] If writing Phase 5 blog, decide framing: "consensus-engine + Tempo payment lanes" reads more distinctive
  than just "consensus-engine." Lean into Tempo angle for distribution reach. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth PR**

- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 92 — Recognition push + [HFT] paper-trade rig setup

**Monday — Engage major discussions + paper-trade rig planning**

- [ ] Architecture-level discussion contributions on Reth. (~30 min)
- [ ] [HFT] Plan a single-machine paper-trade rig: synthetic feed → matching-engine → ledger. Goal: 200 runtime hours by
  M24, building toward 2000 by M30, 4000 by M36. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — More PR reviews**

- [ ] 5+ substantive reviews (mixed Reth + Tempo). (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — Second RFC + paper-trade rig: 24hr soak**

- [ ] If applicable, another design proposal. (~30 min)
- [ ] [HFT] Start a 24-hour paper-trade soak. Monitor: replica divergence (should be 0), ledger reconciliation drift (
  should be 0), P99 matching latency. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Reth PR**

- [ ] Commit + log (~10 min)

**Friday — Maintainer touch points**

- [ ] Engage each target Reth maintainer at least once. (~30 min)
- [ ] [Tempo] Tempo maintainer touch points: engage each of 2-4 Tempo maintainers you've built relationship with.
  Reference your tempo-payment-lane prototype. Ask for design feedback on one specific point, not generic input. (~30 min)
- [ ] Commit notes (~10 min)

**Saturday — End Month 23**

- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 23 review**

---

## Month 24: Phase 5 Close + M24 Five-Path Decision Gate

### Week 93 — Final feature push + [HFT] 72hr soak

**Monday — Feature identification + soak start**

- [ ] Last major reth feature for Phase 5. (~30 min)
- [ ] [HFT] Start a 72-hour paper-trade soak with deliberate chaos: occasional process kills (via `ops-chaos`'s eventual
  scaffold — but we can do shell `kill -9` for now). Goal: replicas converge after every kill. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Implementation**

- [ ] Commit + log (~10 min)

**Wednesday — Continue + 72hr soak audit**

- [ ] Audit soak: divergence count, recovery times, ledger reconciliation. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Tests**

- [ ] Commit + log (~10 min)

**Friday — Submit**

- [ ] Submit final Reth feature PR. (~30 min)
- [ ] [Tempo] 30 min: Tempo PR final push if below 15 merged. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reviews**

- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 94 — Final PR push

**Monday — PR volume**

- [ ] Multiple smaller Reth PRs. (~30 min)
- [ ] [Tempo] 1 hr: any final Tempo PRs you can ship before reassessment. (~60 min)
- [ ] Commit + log (~10 min)

**Tuesday — Continue**

- [ ] Commit + log (~10 min)

**Wednesday — Reviews given**

- [ ] 5+ reviews across Reth + Tempo, mixed. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Continue**

- [ ] Commit + log (~10 min)

**Friday — Final PRs**

- [ ] Commit + log (~10 min)

**Saturday — Wrap up**

- [ ] All outstanding items. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 95 — Reassessment preparation (M24 calibration)

**Monday — Data collection**

- [ ] Count all Reth PRs merged. List features shipped. Update maintainer tracker. (~30 min)
- [ ] [HFT] Count HFT runtime hours (target 200 by M24). matching-engine P99 latency. ledger reconciliation drift.
  messaging-aeron throughput. (~10 min)
- [ ] [Tempo] Count Tempo PRs merged. Target: 25. Acceptable: 15+. Below 10 = flag the gap honestly. (~30 min)
- [ ] [Tempo] Count Tempo crates shipped. Target: 3 (all live). Acceptable: 2. (~30 min)
- [ ] [Tempo] Update Tempo maintainer tracker depth scores. (~15 min)
- [ ] Commit notes (~10 min)

**Tuesday — Signal assessment**

- [ ] Any approaches from Reth-adjacent firms? Mentions by maintainers? (~30 min)
- [ ] Any approaches from HFT firms (Tier A firm A/B/C/D/E/F responses to your public artifacts)? (~30 min)
- [ ] [Tempo] Any approaches from Tempo team? From Tempo design partners? Has anyone from upstream Tempo engaged
  substantively with tempo-payment-lane? (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — Nine crates assessment**

- [ ] Quality of each Reth: storage-trie, exec-vm, consensus-engine. (~30 min)
- [ ] Quality of each HFT: matching-engine, ledger-deterministic, messaging-aeron, marketdata-kernelbypass (
  consensus-bft v1.0 is foundational). (~30 min)
- [ ] Quality of each Tempo: tempo-tx-envelope, tempo-evm-ext, tempo-payment-lane. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Energy assessment**

- [ ] Sustainability check. (~30 min)
- [ ] Commit notes (~10 min)

**Friday — Three pulls to evaluate honestly**

- [ ] **Reth-deepening pull**: do I want another 6-12 months of Reth core? Network strong enough to land Tier A
  Reth-adjacent role? (~30 min)
- [ ] **HFT IC track pull**: matching-engine v1.0 + 200 runtime hours is a strong signal. Tier A firm A/B/C/D/E/F have
  all hired solo systems engineers off public artifacts in past 5 years. Should HFT IC track be the new primary? (~30 min)
- [ ] [Tempo] **Tempo gravity check**: have you been pulled into Tempo strongly enough that "Tempo full-time" is now a
  credible path? Specifically: ≥15 Tempo PRs merged AND ≥2 direct Tempo maintainer relationships AND has your
  tempo-payment-lane been substantively engaged by upstream? If yes to all three, Path D is real. (~30 min)
- [ ] Commit notes (~10 min)

**Saturday — Market state**

- [ ] Crypto cycle. Rust infra hiring climate. Stablecoin payments market state. HFT hiring (Tier A firms A-F + Tier B
  firms A-D). (~30 min)
- [ ] Commit notes (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 96 — Month 24 Decision (Five Paths)

**Monday — Path A analysis (extend Reth core)**

- [ ] Extend Reth core for 6-12 months. What does this look like? Likelihood you accept a Reth-adjacent Tier A offer? (~60 min)
- [ ] Strong if: ≥3 Reth maintainer Depth-3+ relationships, ≥1 feature merged, public visibility growing, no HFT
  inbound. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Path B analysis (post-Reth systems)**

- [ ] Pivot to post-Reth systems (Chronicle Queue replacement, deeper Aeron, distributed mini-db). (~30 min)
- [ ] Strong if: you want to keep building distributed-systems primitives for another 12 months before any job change. (~30 min)
- [ ] Commit notes (~10 min)

**Wednesday — Path C analysis (catch-up)**

- [ ] Catch-up if Phase 5 slipped (some weeks shifted right). (~30 min)
- [ ] Triggered if: consensus-engine v1.0 not shipped, OR matching-engine v1.0 not raft-replicated, OR Sepolia sync not
  green, OR <100 runtime hours. (~30 min)
- [ ] Commit notes (~10 min)

**Thursday — Path D analysis (Tempo pivot, conditional)**

- [ ] [Tempo] **Path D**: pivot to Tempo full-time. Apply to Tempo directly, OR to a Tempo design partner. Real option
  ONLY if W95 Friday's three-condition test passed. Otherwise, Path D drops back into Tier A/B as generic "stablecoin
  payments infra" track. (~30 min)
- [ ] [Tempo] If Path D is real: what does next 6-12 months look like? Direct application, or build-in-public to attract
  inbound? Remote-friendliness check (Tempo is distributed; check if remote roles are open from home geography). (~30 min)
- [ ] Commit notes (~10 min)

**Friday — Path E analysis ([NEW] HFT destination-tier IC track) — NEW DEFAULT IF SIGNALS STRONG**

- [ ] [HFT] **Path E**: HFT destination-tier IC track. Target: Tier A HFT firm (A/B/C/D/E/F) as IC4/IC5 systems
  engineer. Compensation: destination-tier (top-of-band). Geography: destination geography (firm-dependent, typically
  major financial hub). (~30 min)
- [ ] Strong if: matching-engine v1.0 ✓, ledger v0.5 ✓, messaging-aeron v0.5 ✓, marketdata-kernelbypass v0.5 ✓, ≥200
  runtime hours, ≥1 blog post live, ≥1 inbound from HFT recruiter. (~30 min)
- [ ] Phase 6 (M25–M30) plan: continue HFT track to v0.7 + add mini-db CAPSTONE + vector-db, run 2000+ runtime hours by
  M30. Phase 7 (M31–M36): interview prep, applications, destination landing. (~30 min)
- [ ] Commit notes (~10 min)

**Saturday — Decision + Phase 5 close + Tempo addendum close**

- [ ] Pick one path. Don't pick "do both." Five paths; pick one for next 6-12 months. (~30 min)
- [ ] Write decision in `progress.md` with supporting evidence. (~30 min)
- [ ] Full Phase 5 review. (~60 min)
- [ ] Final Tempo metrics tally. Update North Star. (~30 min)
- [ ] [Tempo] Note in progress.md: what the Tempo extension was worth. Honest assessment: did Tempo pay off as
  optionality, or did it cost Reth velocity without proportional return? (~30 min)
- [ ] [HFT] Note in progress.md: where the HFT track stands. matching-engine + ledger + messaging-aeron + marketdata are
  4 production-tier crates that didn't exist 24 months ago. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — End 24-month frame**

- [ ] Full rest. Celebrate milestone (3 Reth + 5 HFT + 3 Tempo = 11 product crates shipped on top of 8 Layer-1/2/3/4
  primitives). (~30 min)
- [ ] **Default Path going forward (this plan assumes)**: Path E (HFT destination-tier IC track) if signals are strong,
  with Reth maintenance + Tempo optionality continuing in background. The remainder of this plan (Phase 6 + Phase 7) is
  written under Path E. If you pick A/B/C/D, derive new W97+ from this plan's structure but redirect daily focus. (~30 min)

---

# PHASE 6: CAPSTONE + OPERATIONS + DESTINATION RECON (Month 25-30)

**[NEW] CAPSTONE Deliverables**: `mini-db` v1.0 (W100), `vector-db` v0.5 (W104).
**[NEW] Operations Deliverables**: `ops-monitoring` + `ops-deploy` + `ops-chaos` + `ops-runbooks` (W105-W109). Live
deployment W106. 2000+ runtime hours target M30.
**Reth track**: maintenance only (~3-5 hrs/wk).
**Tempo track**: maintenance only (~3-5 hrs/wk, ~5-8 hrs/wk M31-M34 if design-partner inbound is real).
**Blog posts**: #4 (W109 — chaos engineering), #5 (W112 — mini-db inheritance), #6 (W115 — vector-db single-node), #7 (
W117 — Phase 6 retrospective).

Phase 6 is written under Path E assumption (HFT destination-tier IC track). If Path A/B/C/D was picked at W96, redirect
daily focus but keep the same primitive-extraction discipline.

## Month 25: mini-db CAPSTONE — proving the inheritance principle

### Week 97 — mini-db scaffold + Mini-LSM Week 1-2 re-read with fresh eyes

**[NEW] crate created**: `crates/mini-db/`. The CAPSTONE. Inheritance target ≥0.70 LOC ratio.

**Monday — mini-db scaffold + skyzh mini-lsm refresher**

- [ ] **Build**: `crates/mini-db/Cargo.toml` workspace member. Deps: `lsm-core` (W40), `wal` (W26), `recovery` (W30),
  `txn` (W72), `bufpool` (W14), `bloom` (W34), `time` (W6), `backpressure` (W11). (~20 min)
- [ ] **Build**: `crates/mini-db/src/lib.rs` empty module headers: `kv.rs`, `scan.rs`, `executor.rs`, `mvcc.rs`,
  `gc.rs`, `engine.rs`, `metrics.rs`, `cli.rs`. (~150 min)
- [ ] Re-read skyzh mini-lsm Week 1 (Storage format). Confirm `lsm-core` covers it. (~45 min)
- [ ] Commit + log (~10 min)

**Tuesday — mini-db engine glue + read path**

- [ ] **Build**: `crates/mini-db/src/engine.rs` —
  `MiniDb { lsm: lsm_core::LsmTree, wal: wal::Wal, txn_manager: txn::TxnManager, bufpool: bufpool::BufferPool, ... }`. (~150 min)
- [ ] **Build**: `crates/mini-db/src/kv.rs` — `get(key) -> Option<Value>` checks memtable → SSTables in order.
  Bloom-filter rejection per `bloom::ClassicBloomFilter`. (~75 min)
- [ ] Test: insert + get round-trip. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — mini-db write path**

- [ ] **Build**: `crates/mini-db/src/kv.rs` — `put(key, value) -> Result<()>` writes WAL via `wal::Wal::append()`, then
  memtable. Group commit honored. (~105 min)
- [ ] **Build**: `delete(key)` writes tombstone. (~75 min)
- [ ] Test: 1M ops mixed put/get; verify WAL replay yields same state. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — mini-db scan path**

- [ ] **Build**: `crates/mini-db/src/scan.rs` — `Scan { iter: lsm_core::MergeIterator }`. Inclusive range scans. (~150 min)
- [ ] Test against fixture: insert 100k keys, scan [10k, 90k), assert 80k results in order. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — mini-db crash recovery via `recovery`**

- [ ] **Build**: `MiniDb::open()` calls `recovery::Recovery::recover(&wal, &mut page_provider)`. ARIES analysis → redo →
  undo from the txn log. (~150 min)
- [ ] Test: write 10k records, panic mid-write, re-open, assert state matches "everything that fsynced before panic" and
  nothing else. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth maintenance + Tempo maintenance**

- [ ] One Reth PR review. (~150 min)
- [ ] [Tempo] Skim Tempo releases (weekly ritual continues). (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 98 — mini-db transactions + MVCC

**Monday — mini-db txn integration**

- [ ] **Build**: `crates/mini-db/src/mvcc.rs` — `Txn::begin() -> Snapshot` via `txn::TxnManager`. Each key-value pair
  carries an HLC timestamp. (~150 min)
- [ ] Commit + log (~10 min)

**Tuesday — mini-db snapshot isolation**

- [ ] Reads against snapshot see only versions visible at snapshot timestamp. (~30 min)
- [ ] Test: T1 begins → T2 writes K=1 → T1 reads K, sees old value or NotFound. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — mini-db GC**

- [ ] **Build**: `crates/mini-db/src/gc.rs` — `GcWorker` runs every N seconds, drops versions older than the oldest
  active snapshot. Coordinates with txn_manager's active-set. (~105 min)
- [ ] Commit + log (~10 min)

**Thursday — mini-db: executor wrapper**

- [ ] **Build**: `crates/mini-db/src/executor.rs` — `Executor` thin layer for batch ops (BatchGet, BatchPut). No SQL —
  this is a KV store, not RDBMS. (~75 min)
- [ ] Commit + log (~10 min)

**Friday — mini-db: criterion bench**

- [ ] Target: 1M ops/sec single-node, <1ms P99 on a 50/50 read/write workload at 1M-key scale. (~30 min)
- [ ] Compare against sled, redb on the same hardware. (~45 min)
- [ ] Commit + log (~10 min)

**Saturday — mini-db: inheritance audit**

- [ ] Count LOC. Native logic (kv glue, scan glue, executor wrapper, GC orchestrator): target <30% of total LOC.
  Inheritance: ≥70% calls into lsm-core / wal / recovery / txn / bufpool / bloom. (~60 min)
- [ ] If <70%, audit the wrappers — chances are something is being reimplemented that should be inherited. (~120 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 99 — mini-db distributed-stub (raft-replicated kv) + tag v0.5

**Monday — mini-db: distributed plan**

- [ ] Design distribution as a thin layer: each shard is a `consensus-raft::RaftNode` group of 3-5 nodes. Each
  replicates a `MiniDb` instance. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — mini-db: raft wire-up**

- [ ] **Build**: `crates/mini-db/src/raft_replicated.rs` — `RaftKv` wraps `MiniDb`. Submits `KvCommand` enum (Put,
  Delete) through `consensus_raft::RaftNode`. apply() mutates local MiniDb. (~150 min)
- [ ] Commit + log (~10 min)

**Wednesday — mini-db: sharding stub**

- [ ] **Build**: `crates/mini-db/src/sharding.rs` — consistent-hash shard router, fixed N-shard cluster. NOT a research
  project — minimum viable. (~75 min)
- [ ] Commit + log (~10 min)

**Thursday — mini-db v0.5 tag**

- [ ] Tag `mini-db v0.5.0`. Single-node + raft-replicated + sharded stub. (~5 min)
- [ ] Commit + log (~10 min)

**Friday — Reth PR maintenance**

- [ ] One Reth contribution to keep relationships warm. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Tempo maintenance**

- [ ] [Tempo] Sunday-ritual-equivalent skim. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 100 — mini-db v1.0 ship + integration with operations infrastructure

**Monday — mini-db: comprehensive test suite**

- [ ] proptest: random op sequences must satisfy linearizability. (~45 min)
- [ ] Loom test for the txn_manager lock interactions. (~60 min)
- [ ] Commit + log (~10 min)

**Tuesday — mini-db: chaos test**

- [ ] Kill replicas mid-batch. Verify final convergence + linearizability. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — mini-db v1.0 tag**

- [ ] Tag `mini-db v1.0.0`. **CAPSTONE deliverable shipped**. (~5 min)
- [ ] Re-run inheritance audit: ≥70% LOC inheritance ratio. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Documentation pass**

- [ ] DESIGN.md showing inheritance tree as ASCII art. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Blog post draft: "mini-db: how 8 crates wrote my database"**

- [ ] Draft post centered on the inheritance discipline. The hook: how many LOC of mini-db are wrapper-only because the
  primitives carry the weight. (~30 min)
- [ ] No deadline yet — post #5 at W112. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth + Tempo maintenance**

- [ ] One Reth PR. One Tempo skim. (~150 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 25 review**

- [ ] North Star: mini-db v1.0 ✓. HFT-track crate count: 5 (matching-engine, ledger, messaging-aeron,
  marketdata-kernelbypass, mini-db). (~30 min)

---

## Month 26: vector-db single-node + operations setup

### Week 101 — vector-db scaffold + HNSW graph construction

**[NEW] crate created**: `crates/vector-db/`. Final HFT-track product, single-node only. v0.5 W104.

**Monday — vector-db scaffold + HNSW paper re-read**

- [ ] **Build**: `crates/vector-db/Cargo.toml` workspace member. Deps: `bufpool`, `bloom`, `txn` (limited), `time`. (~20 min)
- [ ] **Build**: `crates/vector-db/src/lib.rs` empty module headers: `hnsw.rs`, `quantize.rs`, `filter.rs`, `index.rs`,
  `engine.rs`. (~60 min)
- [ ] Re-read Malkov & Yashunin HNSW paper. Sections on multi-layer construction. (~45 min)
- [ ] Commit + log (~10 min)

**Tuesday — vector-db: vector type + distances**

- [ ] **Build**: `crates/vector-db/src/lib.rs` — `Vector<const D: usize>(Box<[f32; D]>)`. Distance trait: `cosine`,
  `dot`, `l2`. (~75 min)
- [ ] SIMD-accelerated L2 with `std::arch::x86_64::_mm256_*`. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — vector-db: HNSW graph layer**

- [ ] **Build**: `crates/vector-db/src/hnsw/graph.rs` — `HnswGraph { layers: Vec<Layer>, ep: NodeId }` with
  `Layer { neighbors: HashMap<NodeId, Vec<NodeId>> }`. (~75 min)
- [ ] Commit + log (~10 min)

**Thursday — vector-db: HNSW insert**

- [ ] **Build**: `crates/vector-db/src/hnsw/insert.rs` — multi-layer insert per paper Algorithm 1. M = 16,
  efConstruction = 200 defaults. (~75 min)
- [ ] Commit + log (~10 min)

**Friday — vector-db: HNSW search**

- [ ] **Build**: `crates/vector-db/src/hnsw/search.rs` — greedy search per Algorithm 5. ef parameter for recall tuning. (~75 min)
- [ ] Commit + log (~10 min)

**Saturday — vector-db: HNSW test**

- [ ] Test: 100k random 128-d vectors, recall@10 ≥ 0.95 vs brute-force ground truth. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 102 — vector-db: quantization + filtered search

**Monday — vector-db: Scalar Quantization (SQ)**

- [ ] **Build**: `crates/vector-db/src/quantize/sq.rs` — `ScalarQuantizer { min: f32, max: f32, scale: f32 }`. Each
  f32 → u8. (~60 min)
- [ ] Test: SQ + HNSW, recall@10 ≥ 0.93 at 4× memory reduction. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — vector-db: Product Quantization (PQ)**

- [ ] **Build**: `crates/vector-db/src/quantize/pq.rs` —
  `ProductQuantizer { num_subq: usize, codebook: Vec<Vec<Vector<8>>> }`. Train via k-means per subspace. (~75 min)
- [ ] Test: 128-d → 8 sub-vectors × 8-d, recall@10 ≥ 0.90 at 16× memory reduction. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — vector-db: filtered search via bloom-bitmap**

- [ ] **Build**: `crates/vector-db/src/filter.rs` — `FilterMask` is a `bloom::ClassicBloomFilter` or precomputed bitmap.
  Search prunes candidates against filter before distance compute. (~75 min)
- [ ] Commit + log (~10 min)

**Thursday — vector-db: payload storage via mini-db**

- [ ] **Build**: `crates/vector-db/src/payload.rs` — payload (per-vector JSON metadata) stored in an embedded
  `mini_db::MiniDb`. Cross-crate inheritance: vector-db doesn't reimplement KV. (~75 min)
- [ ] Commit + log (~10 min)

**Friday — Reth + Tempo maintenance**

- [ ] One Reth review, one Tempo skim. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — vector-db: bench**

- [ ] criterion: HNSW build at 1M vectors, search throughput, recall@10. Compare against qdrant, hnswlib. (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 103 — vector-db: persistence + crash recovery

**Monday — vector-db: HNSW graph serialization**

- [ ] **Build**: `crates/vector-db/src/index/persist.rs` — graph + vectors + payload + bloom serialized to a directory.
  Use mmap for fast cold-start. (~75 min)
- [ ] Commit + log (~10 min)

**Tuesday — vector-db: wal for writes**

- [ ] **Build**: insertions write a WAL record (via `wal::Wal`) BEFORE updating graph. Crash recovery replays into
  in-memory graph. (~105 min)
- [ ] Commit + log (~10 min)

**Wednesday — vector-db: txn wrapper (limited)**

- [ ] `Txn::insert(id, vec, payload)` is atomic. No long-running read transactions (HNSW under graph mutation is hard;
  punt). (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — vector-db: integration test**

- [ ] Insert 100k vectors, crash, re-open, search recall ≥ 0.95. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Reth PR maintenance**

- [ ] Commit + log (~10 min)

**Saturday — vector-db: docs**

- [ ] DESIGN.md showing inheritance tree (bufpool, bloom, mini-db, wal, time). (~60 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 104 — vector-db v0.5 ship (STOPS HERE) + operations planning

**Monday — vector-db: final benchmarks**

- [ ] Comprehensive bench suite. Compare vs Qdrant single-node. (~60 min)
- [ ] Commit + log (~10 min)

**Tuesday — vector-db: security review**

- [ ] Audit unsafe blocks (SIMD). (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — vector-db v0.5 tag (FINAL)**

- [ ] Tag `vector-db v0.5.0`. **STOPS HERE** — not pursued to distributed. Raft + sharding for vector-db would be a
  6-month detour; mini-db already proves the pattern. (~5 min)
- [ ] Commit + log (~10 min)

**Thursday — Operations planning: hardware**

- [ ] Order the operations rig. Target: 2× small servers (single-socket Xeon or Ryzen, 64 GB RAM, NVMe SSD, 10 GbE NIC).
  Total cost ≤ runway-tier-acceptable. (~30 min)
- [ ] Off-site backup plan (git remote + rsync to a third machine or cloud bucket). (~30 min)
- [ ] Commit notes (~10 min)

**Friday — Reth + Tempo maintenance**

- [ ] Reth PR review. Tempo skim. (~150 min)
- [ ] Commit + log (~10 min)

**Saturday — Operations: software plan**

- [ ] Decide on monitoring stack (Prometheus + Grafana + Alertmanager + Loki for logs). (~30 min)
- [ ] Decide on deploy (k8s overkill; use systemd units + a shared filesystem). (~30 min)
- [ ] Commit notes (~10 min)

**Sunday — Rest + End Month 26 review**

---

## Month 27: Operations setup + Live deployment

### Week 105 — `ops-monitoring` + `ops-deploy` build

**[NEW] crates**: `crates/ops-monitoring/`, `crates/ops-deploy/` scaffolds.

**Monday — ops-monitoring scaffold**

- [ ] **Build**: `crates/ops-monitoring/Cargo.toml` workspace member. Deps: `time`, `metrics`, `tracing`,
  `tracing-opentelemetry`. (~20 min)
- [ ] **Build**: `crates/ops-monitoring/src/prometheus.rs` — `PromExporter` wrapping the `prometheus` crate. Standard
  recorders bound at `/metrics`. (~75 min)
- [ ] **Build**: `crates/ops-monitoring/src/tracing_bridge.rs` — tracing → OTLP exporter. (~75 min)
- [ ] Commit + log (~10 min)

**Tuesday — Wire ops-monitoring into matching-engine + mini-db + vector-db**

- [ ] Every product crate now emits standard metrics: `requests_total`, `request_duration_seconds`, `errors_total`, plus
  crate-specific (matching-engine: `orders_matched_total`, `orderbook_depth`; mini-db: `lsm_compactions_total`,
  `wal_group_commit_size`). (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — ops-deploy scaffold + systemd units**

- [ ] **Build**: `crates/ops-deploy/Cargo.toml` workspace member. (~20 min)
- [ ] **Build**: `ops-deploy/templates/` — systemd unit templates for each product binary (matching-engine, mini-db,
  vector-db). (~150 min)
- [ ] **Build**: `ops-deploy/scripts/blue_green.sh` — blue/green deployment via systemd socket activation. (~75 min)
- [ ] Commit + log (~10 min)

**Thursday — Grafana dashboards**

- [ ] One dashboard per product crate. Standard panels: latency P50/P99/P999, throughput, error rate, queue depth. (~30 min)
- [ ] Commit (dashboards as JSON in `ops-monitoring/dashboards/`). (~10 min)

**Friday — Alertmanager rules**

- [ ] Alert on: P99 latency > target, error rate > 0.1%, queue depth > 80% capacity, replica divergence (
  matching-engine). (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Operations rig: provision**

- [ ] Set up the two servers. Install Debian/Ubuntu LTS, harden ssh, set up monitoring agent. (~30 min)
- [ ] Commit (provisioning scripts). (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 106 — Live deployment begins (paper-trade rig)

**Monday — Deploy matching-engine to operations rig**

- [ ] systemd unit running. 3-replica raft cluster across 2 machines (2 on one, 1 on other — small cluster). Bind to
  localhost first. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — Deploy mini-db + vector-db**

- [ ] All three products running. Monitoring stack scraping them. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Synthetic feed generator**

- [ ] **Build**: `crates/marketdata-kernelbypass/examples/synthetic_feed.rs` — generates realistic ITCH messages at
  configurable rate. (~105 min)
- [ ] Run at 100k msg/s. Observe matching-engine throughput, dashboard panels light up. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Ledger settlement loop**

- [ ] matching-engine fills route to ledger-deterministic. Reconciliation report runs every 5 min, must show zero
  net-change. (~5 min)
- [ ] Commit + log (~10 min)

**Friday — Begin uptime tracking**

- [ ] Operations runtime hours: hour 0. Tracking starts. Target M30: 2000 hrs. (≈83 days at 24/7 — possible if we start
  now.) (~120000 min)
- [ ] Reth maintenance PR. (~150 min)
- [ ] Commit + log (~10 min)

**Saturday — Tempo maintenance**

- [ ] [Tempo] Tempo PR or skim. Maintenance level only. (~150 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 107 — `ops-deploy` polish + first uptime week

**Monday — Verify uptime: 7 days of synthetic feed**

- [ ] Audit logs. Any panics? Any replica divergence? Any latency outliers? (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — ops-deploy: rolling restart**

- [ ] Validate blue/green deploy on a binary change. Zero downtime target. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — ops-deploy: tag v0.5**

- [ ] Tag `ops-deploy v0.5.0`. Tag `ops-monitoring v0.5.0`. (~5 min)
- [ ] Commit + log (~10 min)

**Thursday — Uptime: address findings**

- [ ] Fix any issues from Monday's audit. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Reth PR**

- [ ] Commit + log (~10 min)

**Saturday — Tempo maintenance**

- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 108 — `ops-chaos` build + first chaos drill

**[NEW] crate created**: `crates/ops-chaos/`.

**Monday — ops-chaos scaffold**

- [ ] **Build**: `crates/ops-chaos/Cargo.toml` workspace member. Deps: `time` (deterministic time control via test-only
  fixtures), `tokio`, `rand` (seeded, deterministic). (~20 min)
- [ ] **Build**: `crates/ops-chaos/src/lib.rs` with module headers: `fault.rs`, `scenario.rs`, `runner.rs`. (~60 min)
- [ ] Commit + log (~10 min)

**Tuesday — ops-chaos: fault types**

- [ ] **Build**: `crates/ops-chaos/src/fault.rs` — Fault enum: `KillProcess(Pid)`,
  `NetworkPartition { from: NodeId, to: NodeId }`, `LatencyInject { ms: u32 }`, `DiskFull(MountPoint)`,
  `ClockSkew { delta_ms: i64 }`. (~45 min)
- [ ] Commit + log (~10 min)

**Wednesday — ops-chaos: scenario DSL**

- [ ] **Build**: `crates/ops-chaos/src/scenario.rs` — `Scenario { steps: Vec<ScenarioStep> }`. Step is fault + delay +
  assertion. (~75 min)
- [ ] Commit + log (~10 min)

**Thursday — ops-chaos: runner**

- [ ] **Build**: `crates/ops-chaos/src/runner.rs` — executes scenarios against a live deployment. Logs results. (~75 min)
- [ ] Commit + log (~10 min)

**Friday — First chaos drill: kill matching-engine leader**

- [ ] Run a chaos drill: kill the matching-engine raft leader during a 100k order burst. Verify new leader elected
  ≤500ms, replicated state converges, no ledger reconciliation drift. (~30 min)
- [ ] Commit + log (~10 min)

**Saturday — Chaos drill: network partition**

- [ ] Partition 2 of 3 raft replicas from the third. Verify minority side stops accepting writes, majority makes
  progress. (~30 min)
- [ ] Reth PR. (~150 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 109 — `ops-runbooks` + Blog post #4: chaos engineering

**[NEW] dir created**: `ops-runbooks/`. Markdown only; not a crate. Each runbook is "what to do when X alert fires."

**Monday — Runbook: matching-engine replica divergence**

- [ ] Write a step-by-step runbook for replica divergence: identify diverged replica, capture its state, snapshot from
  majority, restart with snapshot. (~30 min)
- [ ] Commit (`ops-runbooks/matching-engine-replica-divergence.md`). (~10 min)

**Tuesday — Runbook: mini-db wal corruption**

- [ ] Steps when wal checksum mismatch detected. (~30 min)
- [ ] Commit. (~30 min)

**Wednesday — Runbook: vector-db OOM**

- [ ] Steps when HNSW graph eats all RAM. (~30 min)
- [ ] Commit. (~30 min)

**Thursday — Runbook: monitoring stack down**

- [ ] When Prometheus or Grafana stops scraping. (~30 min)
- [ ] Commit. (~30 min)

**Friday — Blog post #4: "Chaos engineering on the operations rig"**

- [ ] First public blog post. Draft, edit, post. Topic: how ops-chaos works, what we found. (~60 min)
- [ ] Cross-post to dev.to or hashnode. Twitter announcement. (~60 min)
- [ ] Commit (`blog/2027-XX-XX-chaos-on-the-rig.md`). (~10 min)

**Saturday — Reth PR + Tempo skim**

- [ ] Reth contribution. (~30 min)
- [ ] [Tempo] Sunday-ritual-equivalent. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + End Month 27 review**

- [ ] Runtime hours by M27 end: target 400-500 hrs. (~30000 min)

---

## Month 28: Sustained operations + interview-prep ramp begins

### Week 110 — Sustained operations + dashboard polish

**Monday — Audit alerts**

- [ ] Are alerts firing at appropriate thresholds? Adjust. (~30 min)
- [ ] Commit + log (~10 min)

**Tuesday — Dashboard refinement**

- [ ] Add panels exposing blame-able runaway tail latencies. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Tuning pass: matching-engine**

- [ ] Profile in production. CPU pinning. NUMA awareness if rig supports. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Tuning pass: mini-db**

- [ ] Bufpool capacity tuning. WAL group commit window. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — Reth + Tempo maintenance**

- [ ] Commit + log (~10 min)

**Saturday — Higher-throughput synthetic load**

- [ ] Push matching-engine to 1M orders/sec. Where does it break? (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 111 — matching-engine performance pass

**Monday — Identify hot spot**

- [ ] perf top under 1M orders/sec sustained load. Top 3 functions. (~30 min)
- [ ] Commit notes (~10 min)

**Tuesday — Order book tree comparison redux**

- [ ] We chose BTreeMap at W60. With profile data, consider replacing the per-price-level VecDeque with a fixed-size
  ring (cache-friendlier). (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Lock-free price level**

- [ ] One bid + one ask per symbol at any time has 99% of contention. Consider a lock-free price level for the
  top-of-book. (~30 min)
- [ ] Commit + log (~10 min)

**Thursday — Bench: P99 latency**

- [ ] Target <2µs P99 single-symbol single-core at 500k orders/sec. Current state captured. (~30 min)
- [ ] Commit + log (~10 min)

**Friday — matching-engine v1.0 → v1.1 tag**

- [ ] Tag `matching-engine v1.1.0` capturing the tuning work. (~5 min)
- [ ] Commit + log (~10 min)

**Saturday — Reth PR**

- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 112 — Blog post #5: mini-db inheritance + sustained operations

**Monday — Blog post #5 draft**

- [ ] "How 8 crates wrote my LSM database" — center on inheritance discipline. Show the dependency graph. Show the LOC
  inheritance ratio. (~30 min)
- [ ] Commit (`blog/2027-XX-XX-mini-db-inheritance.md` draft). (~10 min)

**Tuesday — Blog post #5 edit**

- [ ] Edit pass. Hold for review. (~30 min)
- [ ] Commit + log (~10 min)

**Wednesday — Post + amplify**

- [ ] Post. Cross-post. Twitter announcement. r/rust submission optional. (~60 min)
- [ ] Commit + log (~10 min)

**Thursday — Sustained operations check**

- [ ] Uptime hours total: target ~700-800 by W112 end. On track for 2000 by M30. (~30 min)
- [ ] Commit notes (~10 min)

**Friday — Reth PR + Tempo skim**

- [ ] Commit + log (~10 min)

**Saturday — Open issue triage on own repos**

- [ ] Any GitHub issues opened by people who tried to use the crates? Respond. (~30 min)
- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 113 — Network warm-up + interview-prep ramp

**Monday — Map warm network**

- [ ] List of warm connections: Reth maintainers (depth ≥2), Tempo maintainers, HFT folks met via conferences or
  Twitter, prior colleagues at tier-A / tier-B firms. (~30 min)
- [ ] Commit notes (`network_map.md`). (~10 min)

**Tuesday — Interview-prep ramp begins**

- [ ] LeetCode reactivation. 1 hard problem per workday from now through W125. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Systems design refresh**

- [ ] Re-read "Designing Data-Intensive Applications" key chapters (5-7, 9). (~45 min)
- [ ] Commit notes. (~10 min)

**Thursday — Resume draft**

- [ ] Update CV. Emphasize: 11 production crates, 25 in workspace, 60+ Reth PRs, matching-engine v1.1 at 1M+ orders/sec,
  700+ runtime hours, mini-db CAPSTONE with inheritance discipline. No geography, no comp figures, no firm names. (~30 min)
- [ ] Commit (`resume_2027.md` private). (~10 min)

**Friday — Reth + Tempo maintenance**

- [ ] Commit + log (~10 min)

**Saturday — Identify gap: low-latency networking interview topics**

- [ ] Topics to refresh: TCP/IP stack tuning, kernel-bypass, NIC queues, NUMA. (You've built marketdata-kernelbypass —
  cement the explanations.) (~30 min)
- [ ] Commit notes (~10 min)

**Sunday — Rest + End Month 28 review**

---

## Month 29: Public visibility push + 1500hr milestone

### Week 114 — Public visibility + M30 calibration prep

**Monday — Twitter / Mastodon activity**

- [ ] One thoughtful technical post per week from this point. Pin the mini-db blog post. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Talk proposal**

- [ ] Submit a talk to QCon or P99 CONF: "Inheritance discipline in systems engineering." (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Engage HFT-adjacent community**

- [ ] Identify 3-5 HFT engineers visible on Twitter / blogs. Respectful technical replies on relevant threads. (~20 min)
- [ ] Commit notes. (~10 min)

**Thursday — Reth PR (high-visibility opportunity)**

- [ ] Pick a Reth PR that touches an area where multiple maintainers will see it. (~150 min)
- [ ] Commit + log (~10 min)

**Friday — M30 calibration prep**

- [ ] Three questions ahead of W117 M30 decision review. (~30 min)
    - [ ] Public visibility: is there inbound? recruiters, maintainers, conference invites? (~30 min)
    - [ ] Runtime hours trajectory: on track for 2000 by M30 end? (~30 min)
    - [ ] HFT crate quality: are matching-engine, ledger, messaging-aeron, marketdata production-deployable for a
      paper-trade rig at a real firm? (~30 min)
- [ ] Commit notes. (~10 min)

**Saturday — Tempo maintenance**

- [ ] Commit + log (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 115 — Blog post #6: vector-db + interview prep deepening

**Monday — Blog post #6 draft**

- [ ] "vector-db: HNSW + quantization + filtered search in one weekend" — center on how mini-db backed the payload
  store. Inheritance again. (~30 min)
- [ ] Commit. (~30 min)

**Tuesday — Blog post #6 edit + post**

- [ ] Post. Amplify. (~30 min)
- [ ] Commit + log. (~10 min)

**Wednesday — LeetCode harder**

- [ ] Shift to LeetCode Hard daily. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Distributed systems mock**

- [ ] Find a friend or AI for mock systems-design interview. Topic: "design a market data fan-out for 10M msg/s." (~30 min)
- [ ] Commit notes. (~10 min)

**Friday — Reth PR maintenance**

- [ ] Commit + log. (~10 min)

**Saturday — Tempo maintenance**

- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 116 — Interview prep concentrated week

**Monday — Mock interview: matching engine deep**

- [ ] Mock: "explain your matching engine design from order ingress to fill emission." 60 minutes whiteboard-style. (~60 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Mock interview: storage**

- [ ] "Walk me through your storage-trie design. Where does it differ from MDBX?" (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Mock interview: distributed**

- [ ] "How does your matching-engine handle leader failover? What invariants must hold?" (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Reth PR + visibility**

- [ ] Commit + log (~10 min)

**Friday — Skill audit**

- [ ] Honest gaps: which interview topics am I weak on? (Likely: detailed JVM/HotSpot knowledge, distributed-DB
  internals beyond what you've built, MEV economics.) Schedule remediation weeks W118-W120. (~30 min)
- [ ] Commit notes. (~10 min)

**Saturday — Tempo maintenance**

- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 117 — M30 Decision Gate + Blog #7 (Phase 6 retro)

**Monday — Data collection**

- [ ] Runtime hours total. Target 2000 by today. (~10 min)
- [ ] PR portfolio: Reth count, Tempo count, HFT runtime, blog posts (#4-#7 should be done with #7 today). (~150 min)
- [ ] Inbound: any approaches? Recruiter pings? Maintainer DMs? (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — M30 Decision review**

- [ ] Confirm Path E still right path: HFT destination-tier IC track. (~20 min)
- [ ] If inbound has been strong from Tempo / Reth / general crypto, reconsider Path D / Path A. (~30 min)
- [ ] Commit decision in `progress.md`. (~10 min)

**Wednesday — Blog post #7 draft: Phase 6 retro**

- [ ] "30 months in: 11 production crates, 2000 runtime hours, what worked." Honest retrospective. (~30 min)
- [ ] Commit. (~30 min)

**Thursday — Blog #7 edit + post**

- [ ] Post. Amplify. This is the most-likely-to-attract-inbound post; lean into the data (LOC, latency, uptime). (~30 min)
- [ ] Commit + log. (~10 min)

**Friday — Phase 7 kickoff prep**

- [ ] Phase 7 plan review. Application target list (Tier A firm A/B/C/D/E/F + Tier B firm A/B/C/D — no firm names
  committed yet). (~60 min)
- [ ] Commit notes. (~10 min)

**Saturday — Light week wrap**

- [ ] Reth + Tempo maintenance. (~30 min)
- [ ] Commit + log. (~10 min)

**Sunday — End Phase 6**

- [ ] Full rest. (~5 min)
- [ ] Phase 7 starts tomorrow. (~30 min)

---

# PHASE 7: DESTINATION LANDING (Month 31-36)

**Goal**: Land destination-tier IC role at a Tier A HFT firm (Path E). Continued operations push to 4000+ hours by M36.
Interview prep concentrated. Warm-network activation. Flagship blog post. Recon trip. Applications. Interviews. Offer
decision. Resignation + relocation. Arrival. First month at new firm.

**Operations**: continue sustained operation. Target 4000+ runtime hours by M36.
**Reth maintenance**: ~2-3 hrs/wk.
**Tempo maintenance**: ~3-5 hrs/wk, ~5-8 hrs/wk M31-M34 only if inbound from Tempo or design partner is real.
**HFT polish**: matching-engine v1.2, ledger v0.7, messaging-aeron v0.7, marketdata-kernelbypass v0.7 along the way.
Quality bumps via real-load discoveries on the rig.

## Month 31: Skill gap remediation + interview prep curriculum

### Week 118 — Interview prep curriculum: data structures + algorithms

**Monday — Interview-curriculum gap remediation: graph algorithms**

- [ ] Refresh DFS/BFS, Dijkstra, A*, Union-Find. 5 problems each. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — DP review**

- [ ] Top 20 DP patterns. 1-hr/topic. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — String algorithms**

- [ ] KMP, suffix arrays, rolling hash. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Segment trees + lazy propagation**

- [ ] One problem per pattern. (~30 min)
- [ ] Commit notes. (~10 min)

**Friday — Mock: 2 LeetCode Hard back-to-back, 45 min each**

- [ ] Both must be no-help solves to be a green signal. (~30 min)
- [ ] Commit notes. (~10 min)

**Saturday — Reth + Tempo maintenance**

- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 119 — Interview prep curriculum: systems design

**Monday — Systems design: classic problems**

- [ ] Twitter feed, URL shortener, rate limiter, distributed cache. 1 design per day, 60 min each. (~60 min)
- [ ] Commit notes (`interview_notes/sd_<topic>.md` private). (~10 min)

**Tuesday — Systems design: HFT-flavored**

- [ ] Design an order book server. Design a market data fan-out. Design a fill router. Each comes from real
  matching-engine experience — leverage the depth. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Systems design: distributed**

- [ ] Design a distributed KV. Design a vector search service. Design a settlement ledger. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Mock: full 4-round on-site simulation**

- [ ] 60 min coding (Hard), 60 min systems design, 60 min behavioral, 60 min "deep dive into prior work." (~60 min)
- [ ] AI for coding/SD; rehearse behavioral solo. (~30 min)
- [ ] Commit notes. (~10 min)

**Friday — Resume final pass**

- [ ] Tightened, one-pager. PDF + ATS-friendly. Generic geography in resume header. (~30 min)
- [ ] Commit (private). (~10 min)

**Saturday — Reth PR**

- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 120 — Warm-network activation

**Monday — Outreach: Reth maintainers**

- [ ] Reach out to 2-3 Reth maintainers with Depth-3+ relationship. Honest message: "I'm exploring HFT-adjacent IC roles
  in destination-tier firms. If you have intros to your network there, would love to hear them. Happy to share my
  portfolio." No pressure, no urgency. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Outreach: HFT engineers met via conferences**

- [ ] Same script, tuned to context. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Outreach: prior colleagues at tier-A / tier-B firms**

- [ ] Same. Anyone who'd vouch. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Outreach: Tempo maintainers (if Path D still active)**

- [ ] [Tempo] If Path D criteria still active, send the same outreach to Tempo maintainers as a parallel option. (~30 min)
- [ ] Commit notes. (~10 min)

**Friday — Visa / relocation research**

- [ ] Destination geography visa categories. Timeline (typical 4-12 weeks for skilled-worker visa, varies by
  destination). What employer-sponsored vs self-sponsored looks like. (~30 min)
- [ ] Commit notes (`relocation_research.md` private). (~10 min)

**Saturday — Tempo maintenance**

- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

## Month 32: Flagship blog post + recon trip

### Week 121 — Flagship blog post (the public artifact for inbound)

**Monday — Outline**

- [ ] Title: "32 months of inheritance: how 8 primitives built 5 production systems." Wide hook. Story arc: started with
  Reth, extended into HFT, ended with a database that's mostly other people's primitives. (~30 min)
- [ ] Commit (outline). (~10 min)

**Tuesday — Draft section 1: the inheritance principle**

- [ ] Concrete LOC ratios across mini-db, matching-engine, storage-trie. (~30 min)
- [ ] Commit. (~30 min)

**Wednesday — Draft section 2: the workspace tour**

- [ ] Walk through the 25 crates in layers. (~30 min)
- [ ] Commit. (~30 min)

**Thursday — Draft section 3: what went wrong**

- [ ] Honest section: scope creep on matching-engine perpetuals, the txn 2PC undertaking, weeks where Tempo crowded
  Reth. (~30 min)
- [ ] Commit. (~30 min)

**Friday — Draft section 4: how to use this**

- [ ] If you're a junior engineer wanting to build serious systems: the workspace layout is the curriculum. Steal it. (~30 min)
- [ ] Commit. (~30 min)

**Saturday — Final edit + post**

- [ ] Submit to Hacker News, r/rust, r/programming, X/Twitter. Cross-post. (~30 min)
- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

- [ ] Monitor reach. Any inbound? Track in `inbound.md`. (~30 min)

---

### Week 122 — Inbound triage + interview prep

**Monday — Inbound triage**

- [ ] Anyone reach out from the flagship post? Categorize: hiring (HFT), hiring (crypto), curiosity, sales pitch (
  ignore). (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Calls scheduled**

- [ ] If any genuine inbound, schedule 30-min intro calls for W123-W124. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Continue interview prep**

- [ ] Daily LeetCode + one systems design. (~30 min)
- [ ] Commit. (~30 min)

**Thursday — Continue interview prep**

- [ ] Commit. (~30 min)

**Friday — Reth PR**

- [ ] Commit + log. (~10 min)

**Saturday — Tempo maintenance**

- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 123 — Destination geography recon trip (part 1)

**Monday-Friday — Travel to destination geography**

- [ ] Recon trip purpose: (a) meet warm contacts in person, (b) attend a meetup or industry event, (c) get a feel for
  cost of living / housing / quality of life relative to home, (d) preliminary conversations with target firms (even
  early-stage, not formal interviews). (~30 min)
- [ ] Daily: 2 coffee meetings minimum. One firm office visit if possible. (~30 min)
- [ ] Notes nightly into `recon_notes.md` (private, no firm names; recorded as Tier A firm A/B/C/D/E/F). (~30 min)

**Saturday — Travel back**

- [ ] Rest. (~5 min)

**Sunday — Recon debrief**

- [ ] Update `inbound.md` with new contacts and any tentative interview offers. (~30 min)
- [ ] If a firm offered a casual chat → next-step on-site, calibrate Phase 7 timeline forward. (~30 min)

---

### Week 124 — Destination geography recon trip (part 2 OR post-trip wrap)

**Monday — Recon notes synthesis**

- [ ] What did I learn that changes my plan? Particularly cost-of-living and housing — adjust runway expectations. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Firm shortlist**

- [ ] Top 4-6 firms to formally apply to. Tier A primary, Tier B as floor. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Application strategy**

- [ ] Approach: warm intro where available, direct recruiter for the rest. Stagger so I don't have all 6 final-rounds in
  one week. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Interview prep concentrated session**

- [ ] Topic to drill: matching-engine deep-dive. Be able to explain every design decision and trade-off in 5 min and in
  30 min. (~5 min)
- [ ] Commit notes. (~10 min)

**Friday — Reth + Tempo maintenance**

- [ ] Commit + log. (~10 min)

**Saturday — Tempo activity check**

- [ ] [Tempo] If Path D still active in background, what's status with Tempo maintainers? Any active design partner
  conversations? (~30 min)
- [ ] Commit notes. (~10 min)

**Sunday — Rest + End Month 32 review**

---

## Month 33: Active applications + first interviews

### Week 125 — Active applications begin

**Monday — Submit applications to firms 1-2**

- [ ] First-pass applications. Warm intros activated. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Submit to firms 3-4**

- [ ] Commit notes. (~10 min)

**Wednesday — Submit to firms 5-6**

- [ ] Commit notes. (~10 min)

**Thursday — Day-job notice planning**

- [ ] Mental rehearsal of resignation conversation. Plan 30-day notice with offer of transition help. Identify the
  specific day-job projects you'd want to wrap before leaving. No action yet — wait until offer in hand. (~30 min)
- [ ] Commit notes. (~10 min)

**Friday — Reth + Tempo maintenance**

- [ ] Commit + log. (~10 min)

**Saturday — Operations rig: 3000-hour milestone audit**

- [ ] Total runtime hours: target 3000+ by M33 mid. (~10 min)
- [ ] Commit notes. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 126 — First-round interviews begin

**Monday — Phone screen: firm 1**

- [ ] Coding round 1. (~30 min)
- [ ] Commit notes (post-mortem; what went well, where I stalled). (~10 min)

**Tuesday — Phone screen: firm 2**

- [ ] Commit notes. (~10 min)

**Wednesday — Day-job balancing**

- [ ] Coast-mode hours. Don't burn out on the day-job during interview cycle. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Phone screen: firm 3**

- [ ] Commit notes. (~10 min)

**Friday — Phone screen: firm 4**

- [ ] Commit notes. (~10 min)

**Saturday — Recover + post-mortem all 4 screens**

- [ ] What patterns? Where am I weak? (~30 min)
- [ ] Commit notes. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 127 — Second-round interviews begin

**Monday — Technical round: firm 1**

- [ ] 90-120 min deep technical. Coding + systems design. (~120 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Recover**

- [ ] Re-energize. Maybe 4-hour day on the day-job. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Technical round: firm 2**

- [ ] Commit notes. (~10 min)

**Thursday — Recover**

- [ ] Commit notes. (~10 min)

**Friday — Technical round: firm 3**

- [ ] Commit notes. (~10 min)

**Saturday — Recover + Reth PR maintenance**

- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 128 — On-site / final rounds begin

**Monday — Final round: firm 1 (4-5 interviewers, full day)**

- [ ] Coding, systems design, behavioral, deep-dive into matching-engine. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Recover**

- [ ] Commit notes. (~10 min)

**Wednesday — Final round: firm 2**

- [ ] Commit notes. (~10 min)

**Thursday — Recover**

- [ ] Commit notes. (~10 min)

**Friday — Final round: firm 3**

- [ ] Commit notes. (~10 min)

**Saturday — Recover + flush**

- [ ] Reflect on which firm fits best. By now you have impression-data from 3 final rounds. (~30 min)
- [ ] Commit notes. (~10 min)

**Sunday — Rest + End Month 33 review**

---

## Month 34: Final rounds + offer wait + offer decision

### Week 129 — Remaining final rounds

**Monday — Final round: firm 4**

- [ ] Commit notes. (~10 min)

**Tuesday — Recover**

- [ ] Commit notes. (~10 min)

**Wednesday — Final round: firm 5 (if scheduled)**

- [ ] Commit notes. (~10 min)

**Thursday — Recover**

- [ ] Commit notes. (~10 min)

**Friday — Final round: firm 6 (if scheduled)**

- [ ] Commit notes. (~10 min)

**Saturday — Wrap interviews**

- [ ] Commit notes. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 130 — Offer wait + tier-B safety net activation

**Monday — Tier-B firm A application**

- [ ] If by W130 there's no Tier A offer in hand, activate Tier B applications. Safety-net tier compensation is
  materially below destination-tier but still well above home; floor option. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Tier-B firm B application**

- [ ] Commit notes. (~10 min)

**Wednesday — Track Tier A status**

- [ ] Check in with each Tier A firm. Polite "any update on next steps?" message. Don't be needy; do be present. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Reth + Tempo maintenance**

- [ ] Commit + log. (~10 min)

**Friday — Tier-B firms C/D follow-through if needed**

- [ ] Commit notes. (~10 min)

**Saturday — Decision criteria pre-write**

- [ ] BEFORE offers land: write down decision criteria in `progress.md`. Compensation rank, team/manager fit,
  work-from-home flexibility, relocation logistics, role scope. Pre-writing reduces emotional bias when offers arrive. (~30 min)
- [ ] Commit notes. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 131 — Offer decision

**Monday — Initial offers in**

- [ ] By now, expect 1-3 Tier A offers OR 1-2 Tier B offers + 1 pending Tier A. (~30 min)
- [ ] Commit notes (offer details private; sums recorded as destination-tier or safety-net-tier). (~10 min)

**Tuesday — Negotiate**

- [ ] Use competing offers leverage. Polite, direct asks: base, sign-on, year-1 RSU/bonus. Standard playbook. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Compare against pre-written criteria**

- [ ] Re-read W130 Sat's criteria. Score each offer. Avoid moving the goal-posts. (~45 min)
- [ ] Commit notes. (~10 min)

**Thursday — Sleep on it**

- [ ] One night minimum. No same-day acceptance. (~30 min)
- [ ] Commit notes. (~10 min)

**Friday — Decision**

- [ ] Accept one offer. Email the team. Decline the others gracefully. (~30 min)
- [ ] Commit notes (offer details remain private). (~10 min)

**Saturday — Day-job notice**

- [ ] Formal 30-day notice to current employer. Offer transition help. (~30 min)
- [ ] Commit notes. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 132 — Resignation + relocation logistics begin

**Monday — Visa paperwork**

- [ ] Coordinate with destination firm's HR / immigration counsel. Begin paperwork. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Housing search**

- [ ] Destination geography housing. Short-term rental for arrival; permanent decision deferred 2-3 months. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Logistics: visa, shipping, bank account**

- [ ] Banking setup. Health insurance during gap. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Day-job transition**

- [ ] Write transition doc. Identify successor / coverage for each project. (~30 min)
- [ ] Commit notes. (~10 min)

**Friday — Notify warm network**

- [ ] Update warm contacts who helped. Thank-you message + new role announcement. (~30 min)
- [ ] Commit notes. (~10 min)

**Saturday — Reth + Tempo wind-down planning**

- [ ] Plan ongoing-maintenance level for Reth and Tempo from new role. Aim 2-3 hrs/wk evenings and weekends. Public
  commitment stays; depth dials down. (~30 min)
- [ ] [Tempo] Honest note: if accepted role is at a Tempo design partner, this maintenance becomes part of the day job.
  If not, it's evening/weekend. (~30 min)
- [ ] Commit notes. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 133 — Day-job final 2 weeks + ramp-down

**Monday — Day-job transition execution**

- [ ] Hand-offs. Documentation. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Operations rig: planning ongoing**

- [ ] Decide rig fate. Option A: ship the rig to destination (logistics + cost). Option B: keep at home (requires
  reliable network for remote management). Option C: shut down, archive data, restart at destination on cloud VMs. (~30 min)
- [ ] Recommendation: Option B (keep at home) — rig was built for paper-trade; if new role uses real production infra at
  the firm, the home rig becomes a hobby / personal sandbox. Continue runtime hours from afar (target 4000 by M36 is
  still reachable). (~10 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Continue ramp-down**

- [ ] Commit notes. (~10 min)

**Thursday — Continue ramp-down**

- [ ] Commit notes. (~10 min)

**Friday — Last day at current employer**

- [ ] Exit interview. Gracious departure. (~30 min)
- [ ] Commit notes. (~10 min)

**Saturday — Pre-relocation prep**

- [ ] Packing. Paperwork final checks. (~30 min)
- [ ] Commit notes. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 134 — Arrival at destination

**Monday — Arrival**

- [ ] Land in destination geography. Settle into short-term housing. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Onboarding day 1 at new firm**

- [ ] Badge, laptop, intro to manager and team. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Onboarding day 2**

- [ ] Codebase orientation. First read-only access. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Onboarding day 3**

- [ ] Pair with someone on first read-only task. (~30 min)
- [ ] Commit notes. (~10 min)

**Friday — Onboarding day 4**

- [ ] First small PR. (~180 min)
- [ ] Commit notes. (~10 min)

**Saturday — Rest + explore destination geography**

- [ ] Recover from move. No work this weekend. (~30 min)
- [ ] Commit notes. (~10 min)

**Sunday — Rest**

---

## Month 35: First month at new firm + ongoing personal projects

### Week 135 — First month: ramp-up

**Monday — Manager 1:1**

- [ ] What does success look like at 30/60/90 days? Write it down. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Codebase deepening**

- [ ] Whatever the first project is. Be a sponge first 6 weeks. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Continue**

- [ ] Commit notes. (~10 min)

**Thursday — Ask many questions**

- [ ] Stupid-questions budget is highest in week 2-3. Use it. (~30 min)
- [ ] Commit notes. (~10 min)

**Friday — First merged PR at new firm**

- [ ] Small but ships. (~30 min)
- [ ] Commit notes. (~10 min)

**Saturday — Personal projects: light maintenance**

- [ ] One Reth PR review. (~150 min)
- [ ] [Tempo] If still in maintenance mode, one Tempo skim. (~30 min)
- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 136 — First month: deeper task

**Monday — Bigger task assigned**

- [ ] Commit notes. (~10 min)

**Tuesday — Work**

- [ ] Commit notes. (~10 min)

**Wednesday — Work**

- [ ] Commit notes. (~10 min)

**Thursday — Work**

- [ ] Commit notes. (~10 min)

**Friday — Submit**

- [ ] Commit notes. (~10 min)

**Saturday — Personal projects**

- [ ] Operations rig check: still running? Runtime hours updated. Target 3500 by M35 mid. (~10 min)
- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 137 — First month: cultural integration

**Monday — Team lunch**

- [ ] Build relationships across the team. (~60 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Code review etiquette**

- [ ] Watch how senior folks review. Match the style. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Continue ramp**

- [ ] Commit notes. (~10 min)

**Thursday — Continue**

- [ ] Commit notes. (~10 min)

**Friday — Reflection: how does new firm differ from prior employers?**

- [ ] Calibrate expectations. Note what's better, what's different (not worse). (~30 min)
- [ ] Commit notes. (~10 min)

**Saturday — Personal projects maintenance**

- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 138 — First month: midpoint check

**Monday — Self-assessment**

- [ ] Am I matching the 30/60/90 expectations? If not, surface to manager. (~20 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Manager 1:1**

- [ ] Open conversation about progress. (~30 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Work**

- [ ] Commit notes. (~10 min)

**Thursday — Work**

- [ ] Commit notes. (~10 min)

**Friday — Work**

- [ ] Commit notes. (~10 min)

**Saturday — Tempo / Reth contributions**

- [ ] Note: 6 weeks into new firm. Personal project time is now ≤ 4 hrs/wk. That's the new normal. (~10 min)
- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

## Month 36: Settling + M36 retrospective

### Week 139 — Settling into new role

**Monday — Larger project ramp**

- [ ] Whatever the substantive work is. Now own scope. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Work**

- [ ] Commit notes. (~10 min)

**Wednesday — Work**

- [ ] Commit notes. (~10 min)

**Thursday — Work**

- [ ] Commit notes. (~10 min)

**Friday — Work**

- [ ] Commit notes. (~10 min)

**Saturday — Personal sandbox maintenance**

- [ ] Operations rig still running. Latest runtime tally. (~15 min)
- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 140 — Permanent housing

**Monday — Housing search**

- [ ] By now should have impression of destination geography. Choose neighborhood. Begin permanent housing search. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Work**

- [ ] Commit notes. (~10 min)

**Wednesday — Work**

- [ ] Commit notes. (~10 min)

**Thursday — Work**

- [ ] Commit notes. (~10 min)

**Friday — Work**

- [ ] Commit notes. (~10 min)

**Saturday — Personal projects + housing viewings**

- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 141 — Public artifact update

**Monday — Blog post: "From self-directed plan to destination role"**

- [ ] Optional but valuable: write a final retrospective post. 36 months. What worked. What didn't. Stay anonymous on
  geography and compensation. (~30 min)
- [ ] Commit (draft). (~10 min)

**Tuesday — Work**

- [ ] Commit notes. (~10 min)

**Wednesday — Work**

- [ ] Commit notes. (~10 min)

**Thursday — Work**

- [ ] Commit notes. (~10 min)

**Friday — Work**

- [ ] Commit notes. (~10 min)

**Saturday — Personal projects**

- [ ] Operations rig: 3800+ runtime hrs. On track for 4000 by M36 end. (~30 min)
- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 142 — Settle + first quarterly review prep

**Monday — Manager 1:1: how's it going from your side?**

- [ ] Open dialogue. Score yourself honestly. (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Work**

- [ ] Commit notes. (~10 min)

**Wednesday — Work**

- [ ] Commit notes. (~10 min)

**Thursday — Work**

- [ ] Commit notes. (~10 min)

**Friday — Work**

- [ ] Commit notes. (~10 min)

**Saturday — Personal maintenance**

- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 143 — Last full week of the plan

**Monday — Work**

- [ ] Commit notes. (~10 min)

**Tuesday — Work**

- [ ] Commit notes. (~10 min)

**Wednesday — Work**

- [ ] Commit notes. (~10 min)

**Thursday — Work**

- [ ] Commit notes. (~10 min)

**Friday — Work**

- [ ] Commit notes. (~10 min)

**Saturday — Last operations rig audit before plan close**

- [ ] Runtime hours total: target 4000. Capture stats. (~10 min)
- [ ] Commit + log. (~10 min)

**Sunday — Rest + Weekly Ritual**

---

### Week 144 — M36 retrospective + next chapter prep

**Monday — Final retrospective draft**

- [ ] What worked: the inheritance principle (above all). The workspace layout. The Sunday rituals. The decision gates
  at M12/M24/M30. The HFT track addition at M15. (~60 min)
- [ ] What didn't: budget overruns weeks W42, W63, W84. Phase 5 was tighter than budgeted. Tempo time crowded Reth more
  than ideal. (~30 min)
- [ ] What I'd do differently: more aggressive scope-cutting on perpetuals features. Earlier ops rig (W90 was fine but
  W80 would have caught issues sooner). (~30 min)
- [ ] Commit notes. (~10 min)

**Tuesday — Final metrics tally**

- [ ] Reth PRs merged total: target 85+. Actual: ? (~10 min)
- [ ] HFT crates shipped: 5 (matching-engine, ledger, messaging-aeron, marketdata-kernelbypass, mini-db) plus vector-db
  v0.5. Actual: ? (~30 min)
- [ ] Tempo PRs merged: target 38. Actual: ? (~10 min)
- [ ] Runtime hours: target 4000. Actual: ? (~10 min)
- [ ] Blog posts: target 7. Actual: ? (~10 min)
- [ ] Conferences attended: target 4. Actual: ? (~10 min)
- [ ] Direct relationships Reth maintainers: target 10. Actual: ? (~10 min)
- [ ] Direct relationships Tempo maintainers: target 6. Actual: ? (~10 min)
- [ ] Direct relationships HFT engineers met IRL: target 8+. (~10 min)
- [ ] Commit notes. (~10 min)

**Wednesday — Next-chapter plan**

- [ ] Year 4 outline. By now in a destination-tier IC role. Continue personal-projects evenings + weekends at
  sustainable level (≤ 4 hrs/wk). Aim for IC promotion within 18-24 months at new firm. (~30 min)
- [ ] Reth + Tempo + HFT crates: open source remains active, maintained, but not the day's primary work anymore. (~30 min)
- [ ] Commit notes. (~10 min)

**Thursday — Work at new firm**

- [ ] Commit notes. (~10 min)

**Friday — Work**

- [ ] Commit notes. (~10 min)

**Saturday — Close**

- [ ] Plan formally ends today. The inheritance discipline continues forever. (~30 min)
- [ ] Commit + log. (~10 min)

**Sunday — Rest + Plan Close Ritual**

- [ ] Full retrospective read of `progress.md` from W1 to today. Three years of weekly logs. Read them. Note patterns.
  Note what surprised you. (~60 min)
- [ ] The plan is done. The work continues. (~30 min)

---

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

**Inheritance Ratios (target ≥0.70 for Layer-5, ≥0.85 for Layer-7)**:

| Crate                     | Layer | Inherits from                                                      | Target ratio | Audit Week        |
|---------------------------|-------|--------------------------------------------------------------------|--------------|-------------------|
| storage-trie v1.0         | 5     | bufpool, wal, recovery, txn, bloom, eth-trie, eth-storage-cache    | ≥0.70        | W44 Mon           |
| matching-engine v1.0      | 5     | time, backpressure, wal, recovery, consensus-raft, messaging-aeron | ≥0.70        | W74 Thu           |
| ledger-deterministic v0.5 | 5     | time, wal, recovery, txn                                           | ≥0.70        | W83 Wed           |
| consensus-engine v1.0     | 5     | eth-consensus, eth-stage, exec-vm, storage-trie, consensus-bft     | ≥0.70        | W91 Wed           |
| mini-db v1.0              | 5     | lsm-core, wal, recovery, txn, bufpool, bloom, time, backpressure   | ≥0.70        | W98 Sat, W100 Wed |
| vector-db v0.5            | 5     | bufpool, bloom, mini-db, wal, time                                 | ≥0.70        | W104 Tue          |
| tempo-tx-envelope v0.1.0  | 7     | eth-rlp, eth-primitives, eth-consensus, time                       | ≥0.85        | W66 Fri           |
| tempo-evm-ext v0.1.0      | 7     | exec-vm, eth-primitives                                            | ≥0.85        | W91 Thu           |
| tempo-payment-lane v0.1.0 | 7     | consensus-engine, matching-engine (priority idea)                  | ≥0.85        | W91 Thu           |

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

### M24 (W96 Fri) — Five-path decision

- **Path A** (extend Reth core 6-12 months): strong if ≥3 Reth maintainer Depth-3+, ≥1 feature merged, no HFT inbound.
- **Path B** (post-Reth systems): strong if you want another 12 months of distributed-systems primitive work before any
  job change.
- **Path C** (catch-up): triggered if consensus-engine v1.0 missed OR matching-engine v1.0 not raft-replicated OR
  Sepolia sync not green OR <100 runtime hours.
- **Path D** (Tempo pivot, conditional): real ONLY if all three: ≥15 Tempo PRs merged AND ≥2 direct Tempo maintainer
  relationships AND upstream substantively engaged with `tempo-payment-lane`.
- **Path E** (HFT destination-tier IC track) — new default if signals strong: strong if matching-engine v1.0 ✓, ledger
  v0.5 ✓, messaging-aeron v0.5 ✓, marketdata-kernelbypass v0.5 ✓, ≥200 runtime hours, ≥1 blog post live, ≥1 inbound from
  HFT recruiter.

### M30 (W117 Tue) — Path E confirmation or pivot

Three questions:

- **Public visibility**: is there inbound? recruiters, maintainers, conference invites?
- **Runtime hours trajectory**: on track for 2000 by M30 end?
- **HFT crate quality**: are matching-engine, ledger, messaging-aeron, marketdata production-deployable at a real firm?

If all three are green, continue Path E. If two are red, re-evaluate against M24's other paths.

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
            ┌──────────────┬────────────────┬────────────────┬──────────────────┐
            │              │                │                │                  │
            p2p     consensus-raft   consensus-bft     messaging-aeron      marketdata-kernelbypass
            │              │                │                │                  │
            └──────┬───────┴────────┬───────┴────────┬───────┴──────┬───────────┘
                   ↓                ↓                ↓              ↓
                                LAYER 3 (Concurrency + txn)
                   ┌──────────────┬───────────────┬────────────────┐
                   │              │               │                │
                   bloom        lsm-core         txn
                   │              │               │
                   └──────┬───────┴───────┬───────┘
                          ↓               ↓
                                LAYER 2 (Durability)
                          ┌──────────────┬──────────────┐
                          │              │              │
                          wal           recovery
                          │              │
                          └──────┬───────┴──────┐
                                 ↓              ↓
                                LAYER 1 (Universal primitives)
                          ┌──────────────┬──────────────┬──────────────┐
                          │              │              │              │
                          time        backpressure    bufpool
                          │              │              │
                          └──────┬───────┴──────┬───────┘
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

## Final Notes (Neutral)

This plan is one path through a much larger space of possible 36-month journeys. The specifics are:

1. **The track architecture**: Reth core (primary M1-M18, maintenance M19-M36) + HFT (primary M19-M34) + Tempo (additive
   throughout) is a deliberate choice to layer skills rather than parallelize them.
2. **The inheritance principle**: 8 Layer-1/2/3 primitives + 5 Layer-4 distribution primitives + 6 Layer-5 products + 3
   Layer-7 Tempo crates = 25 crates total. About 60-70% of LOC in the products is wired-up inheritance, not net-new
   code. This is the entire point.
3. **The decision gates**: M12 (calibrate), M24 (5-path), M30 (Path E confirm), M36 (close). Each gate has explicit
   criteria to avoid drift.
4. **The Tempo optionality**: Tempo is leverage on the Reth bet, not a parallel bet. Path D (Tempo pivot) at M24 unlocks
   only if three conditions are met. Otherwise Tempo is a CV bullet and a network-warming exercise.
5. **The HFT track addition**: HFT begins at W58 (matching-engine scaffold) and becomes primary M19-M34. This is the
   optionality if Reth ecosystem hiring is soft at M24.
6. **Operations runtime hours**: 2000+ by M30, 4000+ by M36. A self-built ops rig with monitoring + deployment + chaos
   is the bridge to destination-tier IC compensation that PRs alone may not be.
7. **Destination landing in Phase 7**: applications W125, final rounds W128-W129, offer decision W131, resignation W132,
   arrival W134, first month W135-W144. Geography neutral.

This is the inheritance plan. Read it before writing any code each Monday. Audit ratios at every tag. Reject any task
that fails the "is this consumed by a downstream crate within 6 months?" test. The 25 crates are the deliverable.
Everything else is means.

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
