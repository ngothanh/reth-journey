# Daily Plan Index — v3

This folder contains the day-by-day plan for each of the 144 weeks of the 36-month engineering plan.
For the strategic frame, decisions locked, workspace layout, North Star metrics, bar policy,
crate slotting schedule, decision gates, scope boundary (a/b/c), the BFT terminal apex, the
`ConsensusBackbone` interface, inheritance map, and dependency graph, see
[`../README.md`](../README.md) — **the authoritative v3 strategy** (the former `RECONCILED_PLAN_v3.md`
has been merged into it; the scope+sequencing resolution of the standalone margin/liquidation milestones,
the consensus-backbone interface, and the BFT apex is applied there).

**Format**: one file per week, named `WNNN.md` (zero-padded). Open the week you're working on.

---

## ⚠️ v3 migration status (read before opening any week file ≥ W73)

The strategy was re-aimed (v2 → **v3**) at the **on-chain derivatives / perps-infra founding-engineer** north star.
The reframe is **directed, not destructive**: it keeps the full primitive substrate and re-aims the products.

**Daily-file status by phase:**

| Weeks | v3 status | Action |
|---|---|---|
| **W1–W48** (Phases 1–3) | **CURRENT — unchanged.** Layer-0/1/2/3 *primitives* are north-star-invariant ("no primitive twice"). | Use the existing files as-is. |
| **W21–W24** (M6) | **+ `latency-lab`** (latency-measurement leg pulled early — HdrHistogram + coordinated omission + rdtsc tick-to-trade + `perf` + kernel-bypass awareness). | Regenerate these 4 when reached. |
| **W49–W72** (Phase 4) | **Mostly current.** + `log-distributed` v0.1 seed (off-hot-path, W63–66); `matching-engine` scaffold unchanged. | Light edits when reached. |
| **W73–W144** (Phases 5–7) | **SUPERSEDED by v3.** Existing files execute the v2 `mini-db` capstone + job-hunt Phase 7. | **Regenerate JIT** (≈1 month before each week) against README.md "Capstone scope ladder". |

**v2 → v3 deliverable remap (governs the superseded weeks):**
- `matching-engine` keeps the book/STP/triggers; **funding/mark → `oracle-mark`**, **liquidation/ADL/insurance → `liquidation-engine`**.
- New domain crates: **`oracle-mark`** (mark/index/funding + RWA-aware seams), **`risk-engine`** (multi-instrument cross-margin — *principal-defining*), **`liquidation-engine`** (partial-liq + fee-accrued insurance fund).
- **Capstone swap**: `mini-db` (W97–100) → **`perp-dex-core`** — a replicated, fault-tolerant, multi-node hybrid CLOB; the v1.0 lock is in README.md "Capstone scope ladder". `mini-db` is **demoted** to a deferred KV/query facade (its DB substrate lives inside the venue store).
- **Consensus backbone = VSR** (`consensus-vsr`) **behind a `ConsensusBackbone` interface (W91, hard acceptance #9)**; the order/event log **is** the VSR log (one hot-path log). Aeron Cluster + Kafka-`log-distributed` are off-path mirrors.
- **[v3.1] BFT terminal apex (COMMITTED, non-optional, bounded):** `consensus-bft` → hot-path pipelined-HotStuff (N=4, p99≤2ms, 8-scenario Byzantine VOPR), swapped into `perp-dex-core` via the interface. Sequenced **W118–W143** — AFTER the M30 readiness gate, overlapping bet work; does **not** gate Bet #1. See README "Terminal Apex."
- **[v3.1] Scope boundary (a/b/c):** core = the verified deterministic replicated *engine*. **(c)** vault/spot/staking/bridge = OUT (do not schedule). **(b)** fully-on-chain + EVM bridge = CONDITIONAL v1.5/v2 (wires into the `SettlementId` async signature). **(a)** BFT apex = committed (above).
- **`vector-db`** → retained optionality (AI-infra / risk analytics), v0.5, M31 buffer.
- **Phase 7** → bet-path (founding/core-eng token-equity bet; crypto-MM cash bridge) instead of job-landing, **+ the BFT apex track**.
- **[v3.2] Durable-infra additions (HyperCore/TiKV/DataFusion coverage audit):** two NEW crates — **`query-columnar`** (W84 v0.1 → W110–112 v0.5; DataFusion+arrow off-path columnar/vectorized analytics) and **`model-check`** (W90 VSR + W129 BFT; Stateright exhaustive protocol verification) — plus **`txn` v1.1** (W93–94 Percolator MVCC, TSO = VSR commit point) and the **`sim-storage`** fault module (W88/W108/W129). Perp-domain + test sub-tasks fold into existing weeks (open-order IM + tiered MM, multi-source index + clamped basis EMA, no-crossed-book + conservation-of-value invariants, venue differential). Reference mirrors: Malachite/openraft (consensus), glommio/monoio (runtime). Full spec: `.rework/HYPERCORE_ADDITIONS.md`.

The sections below are annotated **[v3]** where the deliverable changed; un-annotated weeks are unchanged from v2.

**v2 conventions**:
- `[NEW v2]` bullets are additions to existing daily content (new crates: `runtime-thread-per-core`,
  `mmap-queue`, `consensus-vsr`; scope expansions: matching-engine v1.5, ledger v0.7→v1.0 with VOPR,
  messaging-aeron v0.7, exec-vm v1.5 block-stm, lsm-core LCS+TWCS).
- Frontmatter `> **v2 modifications**:` summarizes per-week deltas from v1.

**Hours schedule (v2)**: 30h/wk M1–M18 (W1–W72) → 40h/wk M19–M34 (W73–W136) → 30h/wk M35–M36 (W137–W144).

---

## Phase 1 — Rust Mastery (M1–M3, W1–W12, 30h/wk)

- Month 1: Rust Core
  - [Week 1](done/W001.md) — Ownership/borrowing/lifetimes via `eth-primitives` foundation ✓
  - [Week 2](done/W002.md) — Smart pointers + sync concurrency via `eth-storage-cache` ✓
  - [Week 3](done/W003.md) — Async/Pin/Future via `eth-network-codec` ✓
  - [Week 4](done/W004.md) — Atomics, unsafe, variance, macros via `eth-primitives` v0.2 ✓
- Month 2: Production Rust + Early Alloy
  - [Week 5](W005.md) — `eth-rlp` crate + Alloy onboarding
  - [Week 6](W006.md) — `eth-consensus` core + [NEW] `time` crate v0.1
  - [Week 7](W007.md) — `eth-consensus`: EIP-7702, EIP-7685, EOF + more PRs
  - [Week 8](W008.md) — Foundry PR + revm familiarization + `eth-consensus` Receipt/Log
- Month 3: `exec-vm` + `eth-trie` seeds
  - [Week 9](W009.md) — `exec-vm` Phase-1 seed
  - [Week 10](W010.md) — `eth-trie` Phase-1 seed
  - [Week 11](W011.md) — Type-state + `HashedPostState` + [NEW] `backpressure` crate v0.1
  - [Week 12](W012.md) — Phase 1 close + [NEW] `bufpool` scaffold + Phase 2 prep

## Phase 2 — Ethereum Foundation + Ecosystem PRs (M4–M6, W13–W24, 30h/wk)

- Month 4: Ethereum Protocol + Alloy PRs
  - [Week 13](W013.md) — Ethereum fundamentals + `eth-consensus` deepening
  - [Week 14](W014.md) — EIP deep dives + [NEW] `bufpool` v1.0 + medium Alloy PRs
  - [Week 15](W015.md) — EIP-7685 + EOF parser in `exec-vm` + more PRs
  - [Week 16](W016.md) — Alloy/Foundry PRs + `eth-rpc-types` extraction
- Month 5: EVM Deep Dive + revm PRs
  - [Week 17](W017.md) — `exec-vm` expansion (DOUBLES opcode coverage)
  - [Week 18](W018.md) — revm deep-read (diffing against your `exec-vm`)
  - [Week 19](W019.md) — revm PR velocity + `exec-vm` precompile skeleton
  - [Week 20](W020.md) — `eth-trie` expansion
- Month 6: MPT Understanding + First Maintainer Interactions **[v3: + `latency-lab` v0.1 — measurement leg pulled here]**
  - [Week 21](W021.md) — `eth-rlp` extension + maintainer engagement **[v3: + `latency-lab` HdrHistogram + coordinated omission]**
  - [Week 22](W022.md) — Staged sync architecture + `eth-stage` trait skeleton **[v3: + rdtsc tick-to-trade harness]**
  - [Week 23](W023.md) — Ready up for Phase 3 (`storage-trie` scaffold pre-wiring) **[v3: + `perf` LLC/false-sharing + kernel-bypass awareness]**
  - [Week 24](W024.md) — Phase 2 close + Phase 3 prep **[v3: + first tick-to-trade report; `latency-lab` woven into all bar-(c) benches from here]**

## Phase 3 — Storage + Trie Deep Dive + Durability Primitives (M7–M12, W25–W48, 30h/wk)

- Month 7: MDBX Foundation + [NEW] `wal` Primitive
  - [Week 25](W025.md) — MDBX documentation deep
  - [Week 26](W026.md) — [NEW] `wal` crate v0.1 + reth storage architecture
  - [Week 27](W027.md) — `storage-trie::mdbx`: mmap scaffold + `wal` integration
  - [Week 28](W028.md) — `storage-trie` crate: B-tree core
- Month 8: [NEW] `recovery` Crate + MVCC
  - [Week 29](W029.md) — [NEW] `recovery` crate v0.5 (ARIES analysis + redo)
  - [Week 30](W030.md) — [NEW] `recovery` undo pass + ship v0.5 **[v2: + runtime-thread-per-core v0.1 SEED + mmap-queue v0.1 SEED]**
  - [Week 31](W031.md) — Persistent MPT in `storage-trie::mpt` **[v2: + mmap-queue v0.1 CONTINUATION]**
  - [Week 32](W032.md) — MPT proofs + more reth PRs
- Month 9: LSM Trees + [NEW] `bloom` Crate
  - [Week 33](W033.md) — Advanced trie: path compression + [NEW] `epoch-gc` v0.1 scaffold
  - [Week 34](W034.md) — [NEW] `bloom` crate v0.1 + Pruning strategies
  - [Week 35](W035.md) — State commitment deep
  - [Week 36](W036.md) — Snapshot sync research
- Month 10: [NEW] `lsm-core` Build
  - [Week 37](W037.md) — [NEW] `epoch-gc` v0.1 ship + `concurrent::skiplist` scaffold
  - [Week 38](W038.md) — [NEW] `lsm-core` v0.1: memtable + merge iterator
  - [Week 39](W039.md) — [NEW] `lsm-core` v0.3: SSTable + bloom + read path
  - [Week 40](W040.md) — [NEW] `lsm-core` v0.5: STCS compaction + ship
- Month 11: [NEW] `txn` v0.5 + Feature Shipping
  - [Week 41](W041.md) — Ship reth feature **[v2: + lsm-core::LCS]**
  - [Week 42](W042.md) — [NEW] `txn` crate v0.5: 2PL + OCC + deadlock detect **[v2: + lsm-core::TWCS]**
  - [Week 43](W043.md) — Crate v1.0 preparation **[v2: + runtime-thread-per-core v0.3 Tue-Wed]**
  - [Week 44](W044.md) — `storage-trie` v1.0 ship **[v2: + runtime-thread-per-core v0.3 SHIP Mon]**
- Month 12: Phase 3 Close + M12 Decision Gate
  - [Week 45](W045.md) — Final reth storage feature
  - [Week 46](W046.md) — Recognition signals
  - [Week 47](W047.md) — revm preview for Phase 4
  - [Week 48](W048.md) — Phase 3 close + M12 Decision Gate

## Phase 4 — Execution Deep Dive + Distribution Primitives + HFT Track Begins (M13–M18, W49–W72, 30h/wk)

- Month 13: Revm Full Codebase
  - [Week 49](W049.md) — Revm architecture deep
  - [Week 50](W050.md) — Revm journaling
  - [Week 51](W051.md) — Opcode coverage gap-fill
  - [Week 52](W052.md) — [NEW] `p2p` crate v0.1: Kademlia discovery
- Month 14: Full Opcode Coverage + [NEW] `p2p` Noise + Tempo Precompiles
  - [Week 53](W053.md) — Complete opcode set
  - [Week 54](W054.md) — Precompiles in `exec-vm` + [Tempo] `tempo-evm-ext` scaffold
  - [Week 55](W055.md) — Journaling in `exec-vm` + [NEW] `p2p` v0.5 ship
  - [Week 56](W056.md) — Test vector push + [NEW] `consensus-raft` v0.1: election
- Month 15: Dispatch + EthCC + [HFT] `matching-engine` scaffold
  - [Week 57](W057.md) — EthCC Paris trip **[v2: + runtime-thread-per-core v0.5 cross-shard + NUMA Fri-Sat]**
  - [Week 58](W058.md) — Dispatch + matching-engine scaffold **[v2: + runtime io_uring backend + timer wheel Tue+Fri]**
  - [Week 59](W059.md) — evmone comparison + consensus-raft v0.3 **[v2: + runtime backpressure scheduling + instrumentation Wed+Sat]**
  - [Week 60](W060.md) — Hot path + first Tempo PR + matching-engine order book **[v2: + runtime det-test harness + SHIP v0.5 Wed+Fri]**
- Month 16: EOF + Integration + consensus-raft v0.7 + matching-engine v0.5
  - [Week 61](W061.md) — EOF implementation + matching cross logic
  - [Week 62](W062.md) — `exec-vm` + `storage-trie` integration + consensus-raft membership
  - [Week 63](W063.md) — Fuzz targets + matching-engine v0.5
  - [Week 64](W064.md) — Revm performance PRs + [NEW] `consensus-bft` v0.1 scaffold
- Month 17: Architectural + matching-engine v0.7
  - [Week 65](W065.md) — Architectural engagement + matching multi-symbol
  - [Week 66](W066.md) — Reth evm feature + [Tempo] `tempo-tx-envelope` v0.1.0 + perpetuals
  - [Week 67](W067.md) — exec-vm v1.0 prep + consensus-raft v1.0 ship
  - [Week 68](W068.md) — exec-vm v1.0 ship + consensus-bft v0.5 ship **[v2: + consensus-vsr v0.1 SEED Sat]**
- Month 18: Phase 4 Close + `txn` v1.0
  - [Week 69](W069.md) — Final execution PRs
  - [Week 70](W070.md) — Consensus layer preview
  - [Week 71](W071.md) — Phase 4 reflection + matching: ADL + funding
  - [Week 72](W072.md) — Transition + [NEW] `txn` v1.0 (2PC)

## Phase 5 — Consensus + Engine API + HFT Primary (M19–M24, W73–W96, **40h/wk**)

- Month 19: Engine API + consensus-bft v1.0 + matching-engine v1.0
  - [Week 73](W073.md) — Engine API spec + consensus-bft v1.0 + matching-engine wire raft
  - [Week 74](W074.md) — Reth engine + `matching-engine` v1.0 raft-replicated **[v2: + consensus-vsr NormalOperation Tue-Wed]**
  - [Week 75](W075.md) — consensus-engine core + matching polish **[v2: + consensus-vsr ViewChange Mon-Tue + STP Wed-Thu]**
  - [Week 76](W076.md) — Lighthouse CL + `messaging-aeron` v0.1 **[v2: + consensus-vsr v0.5 SHIP + iceberg Thu-Fri]**
- Month 20: Full Engine API + STF + `messaging-aeron` v0.5
  - [Week 77](W077.md) — STF validation + messaging-aeron term buffer **[v2: + matching stop_ladder + mmap-queue pretouch]**
  - [Week 78](W078.md) — Fork choice + messaging-aeron UDP unicast **[v2: + matching auction + mmap-queue/Archive integration]**
  - [Week 79](W079.md) — Reorg + `messaging-aeron` v0.5 ship **[v2: + matching MBO + mmap-queue v0.5 SHIP]**
  - [Week 80](W080.md) — Multi-branch state + `ledger-deterministic` v0.1 **[v2: + matching FIX 4.4 + aeron multicast]**
- Month 21: PBS + Builder + ledger v0.5
  - [Week 81](W081.md) — Invalid payload + Tempo payment-lane **[v2: + matching circuit breakers + aeron SSM]**
  - [Week 82](W082.md) — PBS + Builder + payment-lane design **[v2: matching-engine v1.5 SHIP + aeron Image]**
  - [Week 83](W083.md) — Builder API + payment-lane + ledger v0.5 **[v2: + aeron Archive recording + runtime v1.0 zero-alloc audit]**
  - [Week 84](W084.md) — Test harness + matching+ledger integration **[v2: + aeron Archive replay + aeron v0.7 SHIP + runtime det-harness]**
- Month 22: Cross-Subsystem + Sepolia + `marketdata-kernelbypass`
  - [Week 85](W085.md) — Three-crate Reth + marketdata-kernelbypass v0.1 **[v2: + runtime-thread-per-core v1.0 SHIP Thu]**
  - [Week 86](W086.md) — Cross-subsystem reth + marketdata: io_uring path
  - [Week 87](W087.md) — PR reviews + marketdata: feed handler **[v2: + ledger VOPR op_gen/assertions/faults Mon-Thu]**
  - [Week 88](W088.md) — Devcon attendance **[v2: + ledger VOPR runner + ship v0.7 + consensus-vsr VOPR integration]**
- Month 23: Mentorship + RFC + marketdata v0.5 + Tempo crates v0.1.0
  - [Week 89](W089.md) — RFC + marketdata: AF_XDP **[v2: + ledger v1.0 static-memory invariants Mon-Tue + consensus-vsr zero-alloc Wed-Thu]**
  - [Week 90](W090.md) — Mentorship + `marketdata-kernelbypass` v0.5 ship **[v2: + ledger v1.0 io_uring journal Mon-Tue + consensus-vsr reconfig + v1.0 SHIP Wed-Thu]**
  - [Week 91](W091.md) — consensus-engine v1.0 + Tempo crates v0.1.0 **[v2: + ledger v1.0 VSR replication wire-up Mon-Tue + exec-vm v1.5 block-stm rwset Thu-Fri]** **[v3.1: + DECLARE the `ConsensusBackbone` interface — VSR wired behind the trait; hard v1.0 acceptance #9; the VSR→BFT swap seam. perp-dex-core (W103) depends on the trait, never on `consensus-vsr` directly]** **[v3.1 + O5 AMENDMENT (applied post-W118): `ConsensusBackbone` v1.0 + the O5 abandonment-surface amendment — `propose → ProposalHandle`, `drain_abandoned`, `Commit::proposal_handle`; trait RE-FROZEN. Gate SPEC-GREEN (incl. `test_orphaned_proposal_before_slot_is_abandoned`); BFT permanent-prune liveness flagged as a W125–W128 obligation.]**
  - [Week 92](W092.md) — Recognition push + paper-trade rig **[v2: ledger-deterministic v1.0 SHIP Wed + exec-vm v1.5 versioned memory Thu-Fri]**; paper-trade rig live — node-hour accrual starts
- Month 24: Phase 5 Close + **M24 Derivatives-Infra Readiness Checkpoint** **[v3: replaces Five-Path decision]**
  - [Week 93](W093.md) — Final feature + 72hr soak **[v2: + exec-vm v1.5 optimistic executor]**
  - [Week 94](W094.md) — Final PR push **[v2: exec-vm v1.5 SHIP block-stm]**
  - [Week 95](W095.md) — Reassessment (M24 calibration)
  - [Week 96](W096.md) — **[v3]** M24 checkpoint: `matching-engine`+`oracle-mark`+`risk-engine` at bar (c); 8-system coverage ≥45; **controllable outbound metric** (≥20 targeted on-chain-derivatives conversations, ≥25% response, ≥3 Binance-alumni warm intros) — *replaces the v2 "1 HFT recruiter inbound" criterion*

## Phase 6 — CAPSTONE (`perp-dex-core`) + Operations + Bet Recon (M25–M30, W97–W117, 40h/wk) **[v3: capstone re-aimed]**

- Month 25: **`risk-engine` v1.0 — multi-instrument cross-margin (principal-defining; STANDALONE milestone, do-NOT-fold-into-capstone)** **[v3.1: hardened standalone milestone; netting EXERCISED BTC+ETH per acceptance #1]**
  - [Week 97](W097.md) — **[v3]** `risk-engine` scaffold + IM/MM + multi-instrument netting (BTC+ETH, const 30% haircut) *(was: mini-db scaffold — DB substrate now consumed inside the venue store)*
  - [Week 98](W098.md) — **[v3]** `risk-engine` per-mark-tick recompute + **incremental margin hot path** (mandatory at 100k-account scale) + local-applied-state reads
  - [Week 99](W099.md) — **[v3]** `risk-engine` v1.0 + zero-alloc audit + `latency-lab` p99.99 report; tag v0.5→v1.0
  - [Week 100](W100.md) — **[v3]** `liquidation-engine` scaffold — margin-ratio triggers + partial-liq against book
- Month 26: **`liquidation-engine` v1.0 (STANDALONE milestone, BEFORE the capstone, do-NOT-fold) + capstone assembly begins** **[v3.1: hardened standalone milestone]**
  - [Week 101](W101.md) — **[v3]** `liquidation-engine` **fee-accrued insurance fund** (10% taker, declared param) + waterfall + circuit breakers
  - [Week 102](W102.md) — **[v3]** `liquidation-engine` v1.0 + cascade detection; tag *(vector-db v0.5 → retained-optionality, slips to M31 buffer)*
  - [Week 103](W103.md) — **[v3.1]** `perp-dex-core` assembly start: wires VSR **behind the `ConsensusBackbone` trait** (depends on the trait, NOT `consensus-vsr` directly) + the order/event log = VSR log + **async settle interface** (`SettlementId`=`commit_point`-derived). **⚠️ Flag-A: acceptance #9 stub-swap test lives HERE (at dependency formation, corrected from "W113").** Uses the amended trait (`propose`→`ProposalHandle`, `drain_abandoned`, `Commit::proposal_handle`).
  - [Week 104](W104.md) — **[v3]** `perp-dex-core` hybrid boundary interface (off-chain match / on-chain settle stub) + RWA-aware seams (oracle iface, parameterized funding)
- Month 27: **`perp-dex-core` multi-node + cluster-VOPR + live deploy** **[v3]**
  - [Week 105](W105.md) — **[v3]** 3-node VSR deploy + `ops-monitoring`/`ops-deploy` (monitor runtime/VSR/insurance-fund/VOPR metrics) + loadgen (1M orders/s synthetic flow, 2-4 dedicated cores)
  - [Week 106](W106.md) — **[v3]** Live deployment begins (paper-trade venue rig, multi-node)
  - [Week 107](W107.md) — **[v3]** `ops-deploy` polish + first uptime week + projection fan-out (two staleness contracts)
  - [Week 108](W108.md) — **[v3.1]** `ops-chaos` + **cluster-VOPR** drill (crash/restart/partition; insurance-fund tripwire search). **⚠️ Flag-A: acceptance #4 (`finality_status` from `commit_point`) + #5 (two projection contracts: read-after-commit vs bounded-staleness) verified HERE at the cluster level (distributed properties — only meaningful across nodes).**
  - [Week 109](W109.md) — **[v3]** `ops-runbooks` + Blog #4 (cluster-VOPR / VSR-log-as-source-of-truth)
- Month 28: Sustained operations + **bet-outreach ramp begins** **[v3]**
  - [Week 110](W110.md) — Sustained operations + dashboard polish
  - [Week 111](W111.md) — **[v3]** `perp-dex-core` performance pass (per-mark-tick p99.99 at 100k accounts)
  - [Week 112](W112.md) — **[v3]** Blog #5 (cross-margin + liquidation engine — the principal artifact) + sustained ops
  - [Week 113](W113.md) — **[v3]** Binance-alumni map activation + bet-outreach ramp (founders **and** crypto-MM desks)
- Month 29–30: Public visibility + 2000 node-hour milestone **[v3]**
  - [Week 114](W114.md) — **[v3]** Public visibility + M30 bet-readiness prep
  - [Week 115](W115.md) — **[v3]** Blog #6 (hybrid CLOB / on-chain settle interface) + ecosystem shortlist build (§6 checklist)
  - [Week 116](W116.md) — **[v3]** Flagship-blog draft + ecosystem-selection filter applied to ≥5 venues
  - [Week 117](W117.md) — **[v3.1]** **CAPSTONE v1.0 ACCEPTANCE GATE (Flag-A)**: converges the full ladder — #4/#5 (from W108) + #9 (from W103) + multi-instrument netting + minimal insurance fund + hybrid-settle interface + stub-swap re-verified end-to-end against the locked MVP boundary (NO new test; it is the convergence gate) + inheritance ratio ≥0.75 — **AND** M30 **Bet-Readiness Gate** (2000h + shortlist ≥5 + ≥2 founding-eng convos) + Blog #7; 8-system coverage ≥68 (projection; gate ≥65)

## Phase 7 — **Bet Placement / Bridge + BFT TERMINAL APEX** (M31–M36, W118–W144, 40h/wk W118–W136 then 30h/wk W137–W144) **[v3.1: bet-path + the committed BFT apex run as parallel tracks]**

> **Two parallel tracks this phase.** Track A = **bet path** (sourcing → Bet #1/bridge → ramp). Track B = **the BFT
> terminal apex** (`consensus-bft` W64-73 v0.5 → hot-path-grade pipelined-HotStuff, swapped into `perp-dex-core` behind
> the `ConsensusBackbone` interface). **Guardrail 1**: the apex is sequenced AFTER the M30 readiness gate (W117) and
> does NOT gate Bet #1 — if the bet *is* building fast BFT, the apex folds into bet work (ideal). **Guardrail 2**:
> bounded MVP (pipelined-HotStuff / N=4 / p99≤2ms / 8-scenario Byzantine VOPR / zero-rewrite swap). See README "Terminal
> Apex." `perp-dex-core` v1.5 (RWA/ADL) still ships early-phase; v2 = the BFT-replicated capstone at W143.

- Month 31: **Capstone v1.5 differentiator + bet-prep + BFT apex DESIGN** **[v3.1]**
  - [Week 118](W118.md) — **[v3.1 — RE-DESIGNATED: BFT terminal-apex DESIGN/DECISION gate]** resolves **O5** (op-level abandonment → **NEEDS outer-trait surface**: `propose`→`ProposalHandle` + `drain_abandoned` + `Commit::proposal_handle`), **O1** (ed25519 + batch-verify; BLS deferred to large-N), and the **MVP numeric bounds** (Jolteon 2-chain+TC, N=4, p99 ≤ 2 ms, 8-scenario Byzantine VOPR, zero-rewrite swap). **Design-only, no BFT code.** ⚠️ **GATE: O5 touched the outer trait → W91 amendment required → W119 does NOT start (and W119–W143 are NOT regenerated) until the amendment is reviewed + applied + `test_orphaned_proposal_before_slot_is_abandoned` green.** *(The RWA-v1.5 build formerly here is preserved in git history and re-slots into Track A as part of the gated W119+ regeneration.)*
  - ✅ **W119–W143: GATE CLEARED** — the O5 amendment is applied at W91 (SPEC-GREEN) and W119–W143 are regenerated post-gate as dual-track files (Track A bet-path + Track B BFT-apex appendix). See COMPLETION_MANIFEST.md §6.
  - [Week 119](W119.md) — **[v3]** v1.5: ADL + socialized loss + multi-collateral **[v3.1 BFT: GATE week — apply W91 O5 amendment FIRST, then `BftQuorum` + `SignedPeers` (ed25519+batch)]**
  - [Week 120](W120.md) — **[v3]** Warm-network activation; Dubai/SG cheap prep **[v3.1 BFT: + apex leader rotation + view structure + high-QC (ViewManager 1/2)]**
- Month 32: **Flagship blog + event-trip recon + BFT apex core build** **[v3.1]**
  - [Week 121](W121.md) — **[v3]** Flagship blog post (cross-margin + liquidation engine; "architected for RWA-perps") **[v3.1 BFT: + apex pacemaker / TC view-change (ViewManager 2/2)]**
  - [Week 122](W122.md) — **[v3]** Inbound triage + bet-outreach (≥20 targeted convos) **[v3.1 BFT: + apex QC formation (opaque Certificate)]**
  - [Week 123](W123.md) — **[v3]** Event-trip recon part 1 (Token2049-style) — source Bet #1, **no move** **[v3.1 BFT: + apex 2-chain commit rule (consecutive rounds) + O4 OpNumber densification]**
  - [Week 124](W124.md) — **[v3]** Recon part 2; evaluate the flip-trigger **[v3.1 BFT: + apex DetectEquivocation (evidence)]**
- Month 33: **Active bet outreach + BFT apex core (cont.)** **[v3.1]**
  - [Week 125](W125.md) — **[v3]** Active outreach (target list via §6 checklist) **[v3.1 BFT: + apex permanent-prune liveness (drain_abandoned; impl-only, no trait touch)]**
  - [Week 126](W126.md) — **[v3]** First founding/core-eng conversations (apply §6) **[v3.1 BFT: + apex happy-path 4-node commit]**
  - [Week 127](W127.md) — **[v3]** Deep-dive / trial collaborations; crypto-MM bridge interviews **[v3.1 BFT: + apex ProposalHandle/abandonment wiring end-to-end]**
  - [Week 128](W128.md) — **[v3]** Finalist conversations / work-trials **[v3.1 BFT: + apex BftReplica integrated behind ConsensusBackbone]**
- Month 34: **Bet decision OR bridge + BFT apex Byzantine VOPR** **[v3.1]**
  - [Week 129](W129.md) — **[v3]** Remaining finalist conversations **[v3.1 BFT: + Byzantine cluster-VOPR — ALL 8 scenarios + planted-bug canary]**
  - [Week 130](W130.md) — **[v3]** Bridge safety net: crypto-MM seat if no GO-bet **[v3.1 BFT: + Byzantine VOPR sustain (nightly seed-sweep)]**
  - [Week 131](W131.md) — **[v3]** **Bet #1 decision** (GO only if §6 passes) OR engage bridge **[v3.1 BFT: apex-vs-bet decision rule — fold apex into bet if the bet builds fast BFT, else continue solo]**
  - [Week 132](W132.md) — **[v3]** Resignation + (conditional) relocation **[v3.1 BFT: + Byzantine VOPR — partition+Byzantine combined; 8-scenario sustain green]**
  - [Week 133](W133.md) — **[v3]** Day-job final 2 weeks + ramp-down
  - [Week 134](W134.md) — **[v3]** Start at the bet (or bridge); remote-first **[v3.1 BFT: apex may now be built INSIDE the bet]**
- Month 35: First month at the bet/bridge + BFT hot-path latency & swap (30h/wk begins W137) **[v3.1]**
  - [Week 135](W135.md) — **[v3]** First month: ramp into the core engine **[v3.1 BFT: + apex hot-path latency p99≤2ms]**
  - [Week 136](W136.md) — **[v3]** First month: deeper critical-path task **[v3.1 BFT: + apex zero-alloc / static-mem message path]**
  - [Week 137](W137.md) — **[v3]** Cultural integration + vesting tracked **[v3.1 BFT: + SWAP apex into perp-dex-core via the interface (no engine rewrite)]**
  - [Week 138](W138.md) — **[v3]** First month: midpoint check **[v3.1 BFT: + perp-dex-core v2 = BFT-replicated; cluster-VOPR re-run]**
- Month 36: Settling + BFT-apex acceptance + M36 retrospective **[v3.1]**
  - [Week 139](W139.md) — **[v3]** Settling in; Bet #2 thesis seeds **[v3.1 BFT: + apex p99≤2ms validated in `latency-lab`]**
  - [Week 140](W140.md) — Permanent housing (hub if relocated, else remote base)
  - [Week 141](W141.md) — **[v3]** Public artifact update (perp-dex-core + the BFT apex + the bet)
  - [Week 142](W142.md) — Settle + first quarterly review prep
  - [Week 143](W143.md) — **[v3.1 BFT: BFT-APEX ACCEPTANCE — protocol family ✓ / N=4 / p99≤2ms / 8 Byzantine scenarios green / zero-rewrite swap ✓]**
  - [Week 144](W144.md) — **[v3]** M36 retrospective + Bet #2 prep; **8-system coverage ≥80** + BFT apex shipped as close metrics

---

## v3 ship calendar (quick reference) — **[v3 additions in bold]**

| Week | Event | Bar |
|---|---|---|
| **W24** | **`latency-lab` v0.1 (HdrHistogram + coordinated omission + rdtsc + perf)** | **c** |
| W30 | runtime-thread-per-core v0.1 seed + mmap-queue v0.1 seed | c |
| W31 | mmap-queue v0.1 ship | c |
| W41 | lsm-core LCS | b |
| W42 | lsm-core TWCS (lsm-core v0.6) | b |
| W44 | runtime-thread-per-core v0.3 | c |
| W60 | runtime-thread-per-core v0.5 | c |
| W68 | consensus-vsr v0.1 seed | c |
| W76 | consensus-vsr v0.5 | c |
| W79 | mmap-queue v0.5 | c |
| W82 | matching-engine v1.5 (STP/iceberg/stop-limit/auction/MBO/FIX/CB) | c |
| W84 | messaging-aeron v0.7 (multicast/Image/Archive) | c |
| W85 | runtime-thread-per-core v1.0 | c |
| W88 | ledger-deterministic v0.7 (VOPR simulator) | c |
| W90 | consensus-vsr v1.0 | c |
| W92 | ledger-deterministic v1.0 (static-mem + io_uring + VSR replication) | c |
| W94 | exec-vm v1.5 (block-stm parallel EVM) | b |
| **W80** | **`oracle-mark` v0.5 (mark/index/funding + RWA-aware seams)** | **c** |
| **W84** | **`log-distributed` v0.5 (Kafka-fidelity off-path + VSR-log projection)** | **c** |
| **W90** | **`risk-engine` v0.5 (multi-instrument cross-margin, 30% haircut)** | **c** |
| **W99** | **`risk-engine` v1.0 (per-mark-tick incremental margin, 100k accounts)** | **c** |
| **W102** | **`liquidation-engine` v1.0 (partial-liq + fee-accrued insurance fund + waterfall)** | **c** |
| **W91** | **`ConsensusBackbone` interface DECLARED — VSR behind the trait (hard v1.0 acceptance #9; the VSR→BFT swap seam) + O5 abandonment-surface amendment (`propose→ProposalHandle`, `drain_abandoned`, `Commit::proposal_handle`); trait RE-FROZEN** | **c** |
| **W113** | **`perp-dex-core` v1.0 (3-node VSR-behind-interface, hybrid settle interface, cluster-VOPR; acceptance #9 stub-swap test)** | **c** |
| **W119** | **`perp-dex-core` v1.5 (RWA features + ADL + multi-collateral)** | **c** |
| **W118–W134** | **BFT terminal apex: design→core→Byzantine-VOPR (pipelined-HotStuff, N=4) — AFTER readiness, overlaps bet work** | **c** |
| **W137–W138** | **BFT apex SWAP into `perp-dex-core` via the interface → perp-dex-core v2 (BFT-replicated, no engine rewrite)** | **c** |
| **W143** | **BFT-APEX ACCEPTANCE (protocol family / N=4 / p99≤2ms / 8 Byzantine scenarios green / zero-rewrite swap) — committed terminal apex** | **c** |
| **W84** | **[v3.2] `query-columnar` v0.1 (Arrow RecordBatch + columnar append from the VSR-log analytics projection)** | **b** |
| **W88** | **[v3.2] `sim-storage` fault model folded into ledger VOPR (torn-write/bit-rot/misdirect/FaultAtlas)** | **c** |
| **W90** | **[v3.2] `model-check` VSR safety model (Stateright) + `txn` v1.1 Percolator-MVCC seed (TSO = VSR commit point)** | **b** |
| **W94** | **[v3.2] `txn` v1.1 (Percolator MVCC snapshot isolation + coprocessor pushdown)** | **b** |
| **W112** | **[v3.2] `query-columnar` v0.5 (vectorized scan/filter/aggregate + zone-map pushdown + group-by)** | **b** |
| **W129** | **[v3.2] `model-check` BFT-apex safety/liveness model (Jolteon 2-chain + TC) + sim-storage⊕Byzantine fault composition** | **c** |
