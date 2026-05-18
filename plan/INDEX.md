# Daily Plan Index — v2

This folder contains the day-by-day plan for each of the 144 weeks of the 36-month engineering plan.
For the strategic frame, decisions locked, workspace layout, North Star metrics, bar policy,
v2 crate slotting schedule, decision gates, inheritance map, and dependency graph, see
[`../README.md`](../README.md).

**Format**: one file per week, named `WNNN.md` (zero-padded). Open the week you're working on.

**v2 conventions**:
- `[NEW v2]` bullets are additions to existing daily content (new crates: `runtime-thread-per-core`,
  `mmap-queue`, `consensus-vsr`; scope expansions: matching-engine v1.5, ledger v0.7→v1.0 with VOPR,
  messaging-aeron v0.7, exec-vm v1.5 block-stm, lsm-core LCS+TWCS).
- Frontmatter `> **v2 modifications**:` summarizes per-week deltas from v1.

**Hours schedule (v2)**: 30h/wk M1–M18 (W1–W72) → 40h/wk M19–M34 (W73–W136) → 30h/wk M35–M36 (W137–W144).

---

## Phase 1 — Rust Mastery (M1–M3, W1–W12, 30h/wk)

- Month 1: Rust Core
  - [Week 1](W001.md) — Ownership/borrowing/lifetimes via `eth-primitives` foundation
  - [Week 2](W002.md) — Smart pointers + sync concurrency via `eth-storage-cache`
  - [Week 3](W003.md) — Async/Pin/Future via `eth-network-codec`
  - [Week 4](W004.md) — Atomics, unsafe, variance, macros via `eth-primitives` v0.2
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
- Month 6: MPT Understanding + First Maintainer Interactions
  - [Week 21](W021.md) — `eth-rlp` extension + maintainer engagement
  - [Week 22](W022.md) — Staged sync architecture + `eth-stage` trait skeleton
  - [Week 23](W023.md) — Ready up for Phase 3 (`storage-trie` scaffold pre-wiring)
  - [Week 24](W024.md) — Phase 2 close + Phase 3 prep

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
  - [Week 91](W091.md) — consensus-engine v1.0 + Tempo crates v0.1.0 **[v2: + ledger v1.0 VSR replication wire-up Mon-Tue + exec-vm v1.5 block-stm rwset Thu-Fri]**
  - [Week 92](W092.md) — Recognition push + paper-trade rig **[v2: ledger-deterministic v1.0 SHIP Wed + exec-vm v1.5 versioned memory Thu-Fri]**
- Month 24: Phase 5 Close + M24 Five-Path Decision
  - [Week 93](W093.md) — Final feature + 72hr soak **[v2: + exec-vm v1.5 optimistic executor]**
  - [Week 94](W094.md) — Final PR push **[v2: exec-vm v1.5 SHIP block-stm]**
  - [Week 95](W095.md) — Reassessment (M24 calibration)
  - [Week 96](W096.md) — Month 24 Decision (Five Paths)

## Phase 6 — Capstone + Operations + Destination Recon (M25–M30, W97–W117, 40h/wk)

- Month 25: mini-db CAPSTONE
  - [Week 97](W097.md) — mini-db scaffold + Mini-LSM re-read
  - [Week 98](W098.md) — mini-db transactions + MVCC
  - [Week 99](W099.md) — mini-db distributed-stub + tag v0.5
  - [Week 100](W100.md) — mini-db v1.0 ship + ops integration
- Month 26: vector-db + operations setup
  - [Week 101](W101.md) — vector-db scaffold + HNSW construction
  - [Week 102](W102.md) — vector-db: quantization + filtered search
  - [Week 103](W103.md) — vector-db: persistence + crash recovery
  - [Week 104](W104.md) — vector-db v0.5 ship (STOPS HERE) + ops planning
- Month 27: Operations + Live deployment
  - [Week 105](W105.md) — `ops-monitoring` + `ops-deploy` build **[v2: must monitor runtime/mmap-queue/vsr/VOPR metrics]**
  - [Week 106](W106.md) — Live deployment begins (paper-trade rig)
  - [Week 107](W107.md) — `ops-deploy` polish + first uptime week
  - [Week 108](W108.md) — `ops-chaos` build + first chaos drill
  - [Week 109](W109.md) — `ops-runbooks` + Blog #4 (chaos)
- Month 28: Sustained operations + interview ramp begins
  - [Week 110](W110.md) — Sustained operations + dashboard polish
  - [Week 111](W111.md) — matching-engine performance pass
  - [Week 112](W112.md) — Blog #5 (mini-db inheritance) + sustained ops
  - [Week 113](W113.md) — Network warm-up + interview-prep ramp
- Month 29: Public visibility + 1500hr milestone
  - [Week 114](W114.md) — Public visibility + M30 calibration prep
  - [Week 115](W115.md) — Blog #6 (vector-db) + interview prep
  - [Week 116](W116.md) — Interview prep concentrated week
  - [Week 117](W117.md) — M30 Decision Gate + Blog #7 (Phase 6 retro) **[v2: 5-question criteria + 7-system coverage ≥75]**

## Phase 7 — Destination Landing (M31–M36, W118–W144, 40h/wk W118–W136 then 30h/wk W137–W144)

- Month 31: Skill gap remediation + interview-prep curriculum
  - [Week 118](W118.md) — Interview prep: data structures + algorithms
  - [Week 119](W119.md) — Interview prep: systems design
  - [Week 120](W120.md) — Warm-network activation
- Month 32: Flagship blog + recon trip
  - [Week 121](W121.md) — Flagship blog post **[v2: showcases 7-system mastery anchor]**
  - [Week 122](W122.md) — Inbound triage + interview prep
  - [Week 123](W123.md) — Destination recon trip (part 1)
  - [Week 124](W124.md) — Destination recon (part 2) / post-trip wrap
- Month 33: Active applications + first interviews
  - [Week 125](W125.md) — Active applications begin **[v2: target list anchored on Tier A HFT]**
  - [Week 126](W126.md) — First-round interviews begin
  - [Week 127](W127.md) — Second-round interviews begin
  - [Week 128](W128.md) — On-site / final rounds begin
- Month 34: Final rounds + offer wait + offer decision
  - [Week 129](W129.md) — Remaining final rounds
  - [Week 130](W130.md) — Offer wait + tier-B safety net activation
  - [Week 131](W131.md) — Offer decision
  - [Week 132](W132.md) — Resignation + relocation logistics begin
  - [Week 133](W133.md) — Day-job final 2 weeks + ramp-down
  - [Week 134](W134.md) — Arrival at destination
- Month 35: First month at new firm (30h/wk begins W137)
  - [Week 135](W135.md) — First month: ramp-up
  - [Week 136](W136.md) — First month: deeper task
  - [Week 137](W137.md) — First month: cultural integration
  - [Week 138](W138.md) — First month: midpoint check
- Month 36: Settling + M36 retrospective
  - [Week 139](W139.md) — Settling into new role
  - [Week 140](W140.md) — Permanent housing
  - [Week 141](W141.md) — Public artifact update
  - [Week 142](W142.md) — Settle + first quarterly review prep
  - [Week 143](W143.md) — Last full week of the plan
  - [Week 144](W144.md) — M36 retrospective + next chapter prep **[v2: 7-system coverage ≥85 as close metric]**

---

## v2 ship calendar (quick reference)

| Week | Event | Bar |
|---|---|---|
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
