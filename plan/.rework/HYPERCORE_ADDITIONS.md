# HyperCore + Durable-Infra Additions — additive integration spec (v3.2)

**Status:** PROPOSAL FOR REVIEW. Nothing in `README.md` or any `WNNN.md` is edited until this file is
approved. On approval, Part 4 (README edits) + Part 5 (week-file insertions) are applied as one pass.

**Provenance.** Derived from the 2026-06-26 coverage audit of the on-disk plan (`README.md` v3.1 +
`plan/INDEX.md`) against (A) Hyperliquid HyperCore build techniques and (B) four reference repos
(TiKV, Malachite/openraft, glommio/monoio, DataFusion/arrow-rs). The audit found **8/13 (A)** and
**2/4 (B)** already Covered. This spec integrates the gaps.

**Decision applied (2026-06-26):** durable-infra gaps → **first-class crates**; perp-domain + test gaps →
**sub-tasks** on existing crates; reference repos → **mirror assignments**.

## Hard constraints (every item below obeys these)
- **Additive only** — no existing milestone, week, gate, crate, layer, or version tag is removed, renamed,
  merged, or resequenced. New crates enter the workspace layout as `[NEW v3.2]`; new work enters weeks as
  additive bullets, exactly like the v2→v3 `[NEW]` pattern.
- **No primitive twice** — where a capability extends an existing crate (MVCC→`txn`, storage-fault→the VOPR
  harness, columnar source→`log-distributed` projection), it is a *version milestone / module*, not a
  duplicate crate.
- **Simulation-only consensus** · **perpetuals only, no spot** · **no API/RPC surface** · **simulate oracle
  feeds, never wire real venues** · **BFT stays alongside Raft/VSR, never replacing**.
- **Bar policy** — bar (c) VOPR-grade for HFT critical path + L1 substrate + domain crates; bar (b) for
  spine + database/vector/analytics + Tempo.
- Every new Build's Phase 1 carries a 🧮 **Paper drill** (catalog: `PAPER_DRILLS.md`), per rubric G9; any
  drought-window touchpoint per G10.

---

## Part 0 — Summary table

| ID | Gap (audit) | Form | Crate / milestone | Mirror | Bar | Inherit target | Proposed slot | Effort |
|----|-------------|------|-------------------|--------|-----|----------------|---------------|--------|
| **C1** | B4 columnar/vectorized analytics | **NEW crate** `query-columnar` | L5 analytics product | DataFusion + arrow-rs | b | ≥0.50 | seed W84 → v0.5 W110–112 → v0.7 M31 | 2–3 wk |
| **C2** | A9 model checking | **NEW crate** `model-check` | L4-adjacent verification | Stateright (+ TLA⁺ specs as model source) | (c)-supporting | net-new | VSR model W90 → BFT model W129 | 1–1.5 wk |
| **C3** | B1 Percolator MVCC + coprocessor | **`txn` v1.1** milestone (+ `lsm-core` pushdown) | L3 | TiKV / Percolator | b | n/a (extends txn) | seed W90 → build W93–94 → exercised W97+ | 4–5 d |
| **C4** | A1 storage-fault injection | **harness module** `sim-storage` (in the VOPR harness) | bar-(c) test infra | TigerBeetle VOPR | (c)-supporting | n/a | W88 (ledger) + W108 (cluster) + W129 (BFT) | 2–3 d |
| **D2** | A2 open-order IM reservation | sub-task | `risk-engine` v1.0 (+`matching` hook) | Hyperliquid/dYdX | c | — | W97–99 | 2 d |
| **D3** | A3 tiered maintenance-margin | sub-task | `risk-engine` v0.5→v1.0 | Binance/Hyperliquid tiers | c | — | W85–99 | 1 d |
| **D5** | A5 multi-source index + clamped basis EMA | sub-task | `oracle-mark` v0.5 | Hyperliquid oracle | c | — | W77–80 | 2 d |
| **D8** | A8 named invariants | sub-task | `matching` v1.0 + `ledger` VOPR | TigerBeetle | c | — | W74 + W88 | 2 d |
| **D10** | A10 venue differential testing | sub-task | `matching` + `risk-engine` | TigerBeetle model-based | c | — | W74 + W99 | 2–3 d |
| **R1** | B2 Malachite / openraft | mirror assignment | `consensus-bft` / `consensus-raft` | — | — | — | W118 / W56 | 0 |
| **R2** | B3 glommio / monoio | mirror assignment | `runtime-thread-per-core` | — | — | — | W30/57/85 | 0 |

Total net-new build content: **~5–7 weeks of additive bullets** spread across existing weeks (funded by
invested time per the "coverage non-negotiable" rule — no existing deliverable is displaced). Two new crates
(`query-columnar`, `model-check`), one new version milestone (`txn` v1.1), one new harness module
(`sim-storage`). Workspace goes ~36 → **~38 crates**.

---

## Part 1 — First-class durable-infra crates

### C1 — `query-columnar` (NEW crate) — DataFusion + arrow-rs mirror

**10–15yr surface.** Columnar layout + vectorized execution is the OLAP / data-infra career surface
(DataFusion, DuckDB, ClickHouse, Polars, Arrow Flight). It is the single biggest transferable bet here and
the natural "second specialty" beside the latency/consensus core.

**Mental model.** A column-oriented store of *committed* trades/fills/funding events, materialized **off the
hot path** from the analytics-plane projection of the VSR log (`log-distributed` v0.5). Data lives in Arrow
`RecordBatch` layout (validity bitmaps + contiguous typed buffers); queries are vectorized scan → filter →
project → aggregate kernels over batches, with predicate/projection pushdown. No SQL surface, no RPC — an
in-process Rust query API only (constraint: no API/RPC surface).

**Why it can't break the one-hot-path-log rule.** It *reads* the bounded-staleness projection (acceptance #5
analytics contract). It never sits on the order/settlement path. A query served into a risk/liquidation
decision is the same correctness bug acceptance #5 already forbids.

- **Layer / bar:** L5 analytics product · **bar (b)** (database/vector/analytics tier).
- **Mirror:** Apache DataFusion (vectorized execution) + arrow-rs (columnar layout). Secondary: DuckDB
  vectorized push-model.
- **Inherits:** `log-distributed` (projection source), `lsm-core` (block/SSTable storage of column chunks),
  `bufpool`, `time`, `latency-lab` (kernel microbench). Target **≥0.50** (kernels + Arrow encode are net-new;
  storage + projection inherited).
- **Inherited by:** `perp-dex-core` (analytics fan-out / trade-log queries on the M28+ rig); optionally
  `risk-engine` historical-correlation backfill (the v1.5 rolling-correlation input).
- **Slot (additive bullets):**
  - **v0.1 seed — W84** (rides `log-distributed` v0.5 first projection): Arrow `RecordBatch` for the trade
    schema; columnar append from the projection stream.
  - **v0.5 — W110–W112** (P6 sustained-ops window, the lightest venue weeks): vectorized scan/filter/
    aggregate kernels; column chunking + min/max zone maps; predicate + projection pushdown; one
    group-by-symbol-and-window aggregate (the funding/volume rollup).
  - **v0.7 — M31 buffer** (alongside `vector-db`): dictionary encoding + run-length + a late-materialization
    join (trade × mark). Optional.
- **Acceptance (v0.5):** (1) ingests the W108 cluster-VOPR trade stream into Arrow batches; (2) a vectorized
  filter+aggregate over ≥10M rows beats a row-at-a-time baseline by ≥5× on `latency-lab` (the back-of-envelope
  drill must predict the band first); (3) zone-map pushdown skips ≥1 chunk on a selective predicate, proven by
  a counter; (4) zero alloc in the per-batch kernel inner loop (audited like the hot-path crates, though the
  bar is (b)).
- **Paper drills (Phase 1):**
  - 🧮 D-LAYOUT on a 4-column `RecordBatch` (i64 price, i64 qty, u64 ts, u8 side): draw the byte map —
    validity bitmap, buffer offsets, alignment, null packing.
  - 🧮 D-COST on the vectorized filter kernel: cost ledger for scalar-per-row vs SIMD-batch (cycles/row,
    branch-mispredict cost, L1/L2 residency of a column chunk) — predict the ≥5× band.
  - 🧮 D-WALK on one query (`SELECT sum(qty) ... WHERE symbol=BTC AND ts IN window GROUP BY minute`): hand-trace
    scan → zone-map skip → filter → group → aggregate over a 12-row, 3-chunk instance.
  - 🧮 D-CAP: analytics fan-out back-of-envelope — projection lag vs query QPS at the ~100k-account /
    10-tick/s rate; confirm it stays inside the bounded-staleness budget (≤500ms / ≤1000 ops).

### C2 — `model-check` (NEW crate) — Stateright mirror

**10–15yr surface.** Lightweight formal methods (model checking) is the correctness-engineering surface that
distinguishes systems builders at the top tier (TigerBeetle, FoundationDB, AWS-via-TLA⁺). Stateright keeps it
**in-Rust, in-workspace** — buildable, not a separate toolchain.

**Mental model.** Exhaustive (bounded) state-space exploration of the *protocol* (not the impl): enumerate
reachable states under all interleavings up to a small bound, assert safety + liveness invariants. Complements
VOPR (which samples a huge space with one seed) by *exhausting* a small one — they catch different bugs.

- **Layer / bar:** L4-adjacent verification dev-crate · **bar-(c)-supporting** (it is how a bar-(c) consensus
  crate earns its safety claim; not itself bar-graded).
- **Mirror:** Stateright (Rust actor/model checker). Model *source* = the published TLA⁺ specs of VSR and
  HotStuff (read to extract the invariants — not a TLA⁺ build).
- **Inherits:** net-new (it models protocols; takes no crate dependency on them — it re-encodes the state
  machine). Like `latency-lab`, scored net-new.
- **Inherited by:** the safety claims of `consensus-vsr` (W90) and the BFT apex (W143 acceptance). Cited in
  Blog #4 / the flagship blog as the "exhaustive + sampled" verification story.
- **Slot (additive bullets):**
  - **VSR model — W90** (when `consensus-vsr` v1.0 ships): model ViewChange + NormalOperation; check
    **linearizability of the single log** + **no-two-leaders-per-view** + **commit-monotonicity** at N=3,
    bounded ops. This is the exhaustive complement to the W88/W108 VOPR.
  - **BFT model — W129** (alongside the Byzantine VOPR): model Jolteon 2-chain commit + TC view-change at
    N=4, f=1; check **safety under equivocation** (no two conflicting committed QCs) + **liveness after GST**
    (a decision is reached once the network heals). Pairs 1:1 with the 8 VOPR scenarios.
- **Acceptance:** both models run green in CI within a bounded depth; each reproduces ≥1 *planted* bug
  (remove a lock / swap an ordering) as a found counterexample — the negative control proving the checker
  bites.
- **Paper drills (Phase 1):**
  - 🧮 D-LIN on a 3-op, 2-client VSR history: build the partial order, find the linearization (or prove none
    exists) — the property the model encodes.
  - 🧮 D-QUORUM on the BFT 2-chain rule: trace a safe commit, then hand-construct the equivocation the model
    must reject (two QCs in the same view) — write the invariant that forbids it.
  - 🧮 D-STRUCT: hand-execute the Stateright `next_state` on a 3-state instance to confirm the encoded
    transition relation matches the protocol before letting the checker run.

### C3 — `txn` v1.1 — Percolator MVCC + coprocessor pushdown (TiKV)

**10–15yr surface.** Distributed-transaction internals (snapshot isolation, MVCC GC, the read/lock/write
column families, predicate pushdown) are the distributed-database career surface (TiKV, CockroachDB, Spanner-
likes). Built as a **mode of the existing `txn` crate** — no duplicate primitive.

**Mental model.** Percolator snapshot isolation with **the VSR commit point as the timestamp oracle** (reuse —
*no separate TSO service*, which would be a second ordering authority and break "one hot-path log"). A txn
reads at `start_ts`, buffers writes, on commit picks `commit_ts` = the VSR op-number of its commit record;
visibility = "version with the greatest `commit_ts ≤ my start_ts`." Coprocessor = predicate/aggregate pushdown
evaluated at the `lsm-core` scan layer so filtered rows never cross the call boundary.

- **Layer / bar:** L3 · **bar (b)** (database substrate tier).
- **Mirror:** TiKV (Percolator transactions; coprocessor). Source: the Percolator paper's lock/write/data CFs.
- **Reuses:** `txn` (lifecycle/2PL/OCC/2PC already shipped W42/W72), `consensus-vsr` (commit point = the
  timestamp source), `lsm-core` (the versioned KV + the pushdown hook), `time`.
- **Inherited by:** the venue position/account store (the ScyllaDB leg consumed by `risk-engine` W97+ and
  `perp-dex-core`): MVCC snapshot reads let risk read a consistent position snapshot without blocking writers.
  **Hot-path boundary (Thompson):** the **per-mark-tick incremental-margin path stays on local applied state**
  (read-after-commit, in-memory, ~30 ns, zero-alloc) — it does NOT take an MVCC snapshot read per account
  (a ~2 µs LSM seek × ~6,250 accounts/shard/tick ≈ ~12.5 ms would blow the <100 µs p99.99 budget by ~100×).
  MVCC snapshots are for the **off-tick consistent multi-position reads** (full recompute on
  circuit-breaker/param-change, admin/reconciliation), never the per-tick hot loop.
- **Slot (additive bullets):**
  - **seed W90** (after VSR v1.0 gives a commit point): the version-key encoding + the start_ts/commit_ts
    read rule.
  - **build W93–W94** (P5 close has capacity beside exec-vm block-stm): Percolator lock/write/data layout;
    snapshot-isolation read path; MVCC GC of versions below the watermark; coprocessor predicate pushdown into
    `lsm-core` scans.
  - **exercised W97+**: `risk-engine` takes an MVCC snapshot for **off-tick consistent multi-position reads**
    (full recompute / reconciliation) — the per-mark-tick incremental path stays on local applied state
    (zero-alloc, ~30 ns). Demonstrated in the venue store.
- **Acceptance:** (1) snapshot-isolation property test — a long reader sees a consistent snapshot while a
  writer commits newer versions (no torn read, no write-skew on the disjoint-write case); (2) the timestamp
  oracle is provably the VSR commit point — `cargo tree`/a seam test shows no standalone clock; (3) coprocessor
  pushdown skips ≥N rows at the scan layer, proven by a counter, and returns identical results to the
  no-pushdown path (differential).
- **Paper drills (Phase 1):**
  - 🧮 D-STRUCT on a 3-transaction instance (T1 reads, T2 writes+commits, T3 reads): hand-execute the
    start_ts/commit_ts visibility rule; show the version each read resolves to.
  - 🧮 D-WALK on the Percolator commit: trace primary-lock → secondary-locks → the data/write/lock CF rows
    written at each step; mark the point the txn becomes atomically visible.
  - 🧮 D-DET: argue the commit_ts is replay-deterministic because it is the VSR op-number (not a wall clock) —
    the one line that keeps MVCC inside the deterministic core.

### C4 — `sim-storage` (NEW harness module, not a standalone crate) — TigerBeetle storage-fault model

**10–15yr surface.** "Assume the disk lies" (torn writes, bit-rot, misdirected I/O, fsync-gap corruption) is
the storage-reliability surface (TigerBeetle, ZFS, Ceph). It hardens every durability claim the plan makes.

**Mental model.** A fault-injecting `Storage` implementation behind the **already-mandated `Storage` trait
seam** (Risk Register: "all I/O behind interfaces"). Per-seed it injects: read-fault / write-fault probability,
**torn writes** (only a prefix of a sector lands), **bit-corruption** (flip bits in a returned block),
**misdirected I/O** (read/write hits the wrong offset), and **crash-corruption of in-flight writes**. A
`FaultAtlas` coordinates faults so at least one replica always holds a valid copy of each block (else the test
is unwinnable, not a real bug).

- **Layer / bar:** bar-(c) test infra; a module of the deterministic-sim harness (lives beside the VOPR
  runner). Not a crate — it *is* a primitive only in the sense of "the Storage fault seam," built once and
  reused by every VOPR leg (no primitive twice).
- **Mirror:** TigerBeetle VOPR storage faults (`read_fault_probability`, `write_misdirect_probability`,
  `ClusterFaultAtlas`).
- **Reuses:** the `Storage` trait already required for cluster-VOPR honesty; `recovery` (the ARIES passes it
  exercises); the existing seeded PRNG.
- **Slot (additive bullets):**
  - **W88** — fold the storage-fault model into `ledger-deterministic` VOPR v0.7 (currently only
    partition/reorder/crash-restart): add the five storage faults + the `FaultAtlas`.
  - **W108** — cluster-VOPR drill uses it across the 3-node placement (the rented-3rd-box week).
  - **W129** — the Byzantine VOPR composes storage faults with Byzantine faults (a Byzantine node *and* a
    lying disk).
- **Acceptance:** (1) recovery survives torn-write + crash-restart with no lost committed op, proven across
  the seed sweep; (2) a planted bug (skip a checksum verify) is caught as a corrupted-read that the StateChecker
  flags — negative control; (3) the `FaultAtlas` invariant (≥1 valid replica per block) holds, asserted in the
  harness.
- **Paper drills (Phase 1):**
  - 🧮 D-VOPR on a 5-op log + 1 torn write: trace analysis→redo→undo recovery; mark where the checksum catches
    the torn sector; then the nondeterminism-leak hunt (does any fault path read a wall clock?).
  - 🧮 D-FAIL: blast-radius table — for each of the 5 fault types, which replica/block is affected and which
    invariant (durability / convergence / no-lost-commit) it threatens.

---

## Part 2 — Perp-domain + test sub-tasks (on existing crates)

> These attach as additive bullets to existing weeks; each adds its 🧮 paper drill to that Build's Phase 1.

### D2 — Open-order IM reservation → `risk-engine` v1.0 (W97–99) + `matching-engine` pre-trade hook (reuse "risk pre-trade")
Resting limit orders **lock initial margin**; fills convert reservation→position margin; cancels release it.
The matching engine's existing pre-trade risk check calls `risk-engine.reserve_im(order)`; rejection on
insufficient free margin. **Acceptance:** an account cannot rest orders whose summed IM exceeds free collateral;
cancel restores free margin exactly (conservation). **Drill:** 🧮 D-WALK — trace one account placing 3 resting
orders + 1 fill + 1 cancel; show free-margin at each step. 🧮 D-COST — the reservation update must be O(1) on
the order hot path (incremental, not a recompute).

### D3 — Tiered maintenance-margin schedule → `risk-engine` v0.5→v1.0 (W85–99)
A deterministic **margin-tier bracket table**: MM% steps up with position notional (tier 0: ≤X notional → m0%;
tier 1 → m1% …). Swept deterministically like the haircut (never randomized in-core). **Acceptance:** crossing a
tier boundary raises required MM at exactly the declared notional; the table is a declared model param.
**Drill:** 🧮 D-WALK — a position growing across two tier boundaries; compute MM at each; mark the discontinuity.

### D5 — Multi-source index + clamped basis EMA → `oracle-mark` v0.5 (W77–80)
Aggregate **N simulated price sources** (median / trimmed-mean) into an index; mark = index + a **basis EMA
clamped** to ±cap (Hyperliquid's impact-price discipline). All feeds simulated (constraint). **Acceptance:** a
single rogue source can't move the index beyond the trim; the clamp bounds mark-index divergence; both are
replay-deterministic (integer/fixed-point EMA, no float drift across replicas). **Drill:** 🧮 D-WALK — 5 sources
incl. 1 outlier → median → basis → clamped EMA over 4 ticks. 🧮 D-DET — show the fixed-point EMA replays
bit-identically.

### D8 — Named invariants → `matching` v1.0 (W74) + `ledger` VOPR (W88)
Add two **named property/StateChecker invariants**: **no-crossed-book** (best bid < best ask always holds
post-match) on `matching-engine`; **conservation-of-value** (Σ account balances + insurance fund = constant
across every applied op) on the `ledger` StateChecker. **Acceptance:** both run in proptest/VOPR; each fails a
planted bug (allow a crossed match / drop a credit). **Drill:** 🧮 D-STRUCT — hand-execute 4 orders that *almost*
cross; state the invariant. 🧮 D-DET — the value-conservation sum as a replicated invariant.

### D10 — Venue differential testing → `matching` (W74) + `risk-engine` (W99)
A **naive reference matcher** (sorted-vec, no perf) and a **brute-force margin recompute** (full portfolio, no
incremental) used as differential oracles inside VOPR: the production engine must match the reference on every
seeded op stream. Reuses the existing seeded op generator. **Acceptance:** 1M-op differential run with zero
divergence; a planted off-by-one in the production matcher is caught. **Drill:** 🧮 D-DET — why the reference and
production must agree bit-for-bit (determinism contract). 🧮 D-VOPR — the op-stream + divergence-check loop.

---

## Part 3 — Reference-repo mirror assignments (no build)

- **R1 — Malachite → `consensus-bft` apex (W118 design); openraft → `consensus-raft` (W56).** Malachite
  (Informal Systems' Rust BFT, Tendermint/HotStuff-class) is the natural Rust mirror to read against the
  Jolteon apex; openraft is the production-Raft mirror for the off-path raft. Add to each crate's "Mirror:"
  line in the workspace layout. *(Reference annotation only — the crates already exist; no new build.)*
- **R2 — glommio / monoio → `runtime-thread-per-core` (W30/57/85).** The Rust-native thread-per-core io_uring
  runtimes — better day-to-day Rust mirrors than Seastar (C++). Add beside "MIRROR: Seastar."

---

## Part 4 — README edits to apply on approval (additive)

1. **Workspace Layout** — add `[NEW v3.2]` entries:
   - L5: `query-columnar/` (slot W84 seed / W110–112 v0.5 / M31 v0.7; mirror DataFusion+arrow-rs; bar b;
     inherits log-distributed, lsm-core, bufpool, time, latency-lab; inherited by perp-dex-core).
   - L4-adj: `model-check/` (Stateright; VSR model W90, BFT model W129; net-new; supports consensus-vsr +
     BFT apex).
   - `txn/` — append "v1.1 (W93–94): Percolator MVCC (TSO = VSR commit point) + coprocessor pushdown".
   - Note `sim-storage` as a module under the deterministic-sim harness (not a crate line; a bullet under the
     VOPR/Testing conventions).
   - Append `glommio/monoio` to `runtime-thread-per-core` MIRROR; `Malachite`/`openraft` to the consensus
     crates' MIRROR.
2. **Coverage matrix** — add rows: #22 columnar/vectorized analytics (`query-columnar`, DataFusion/arrow);
   #23 model checking (`model-check`, Stateright); #24 distributed-MVCC (`txn` Percolator); and fold
   storage-fault into row 21 (deterministic-core discipline). Update row 9 (risk) with open-order IM + tiered
   MM; row 8 (oracle) with multi-source index + clamped EMA; row 7 (matching) with no-crossed-book + differential.
3. **Crate slotting schedule** — add four `### New crate / milestone` blocks (C1–C4) mirroring the existing
   `latency-lab`/`log-distributed`/domain-crate block format (slot, mirror, justification, risk-if-skipped).
4. **v3 ship calendar** — add rows: W84 `query-columnar` v0.1; W90 `model-check` VSR model + `txn` v1.1 seed;
   W94 `txn` v1.1 (Percolator MVCC); W112 `query-columnar` v0.5; W129 `model-check` BFT model.
5. **North-Star (Venue track)** — add rows: `query-columnar version` (—/—/v0.1/v0.5/v0.7); `model-check`
   (—/—/—/VSR/VSR+BFT); `txn` MVCC milestone (—/—/—/v1.1/v1.1). Reference-system 8-system table is unchanged
   (these map to existing systems / are new reference repos noted in a footnote).
6. **Risk Register** — add: "columnar engine scope balloons → lock v0.5 to scan/filter/aggregate + one group-by,
   defer joins/encodings to v0.7/M31"; "Stateright state-space explosion → bound N and op-depth, model the
   protocol not the impl"; "Percolator MVCC introduces a second clock → TSO is the VSR commit point, enforced by
   a seam test."
7. **Open Questions §10** — add: exact `query-columnar` ↔ `log-distributed` projection seam (push vs pull);
   `model-check` depth bounds; whether `query-columnar` v0.7 ships in M31 or slips with `vector-db`.

## Part 5 — Week-file insertions to apply on approval (additive bullets, existing format)

| Week | Crate(s) | Additive bullet (Build/Extend + 🧮 paper drill in Phase 1) |
|------|----------|-----------------------------------------------------------|
| W74 | matching, model-check(seed) | no-crossed-book invariant proptest (D8); naive reference matcher for differential (D10) |
| W77–80 | oracle-mark | multi-source index + clamped basis EMA (D5) |
| W84 | query-columnar | v0.1 seed: Arrow RecordBatch + columnar append from projection (C1) |
| W85–90 | risk-engine | tiered MM bracket table (D3) folded into v0.5 |
| W88 | ledger, sim-storage | storage-fault model + FaultAtlas into VOPR v0.7 (C4); conservation-of-value StateChecker (D8) |
| W90 | model-check, txn | Stateright VSR model (C2); `txn` v1.1 Percolator seed (C3) |
| W93–94 | txn, lsm-core | Percolator MVCC + coprocessor pushdown build (C3) |
| W97–99 | risk-engine, matching | open-order IM reservation (D2); brute-force margin reference for differential (D10) |
| W108 | cluster-VOPR, sim-storage | storage faults across 3-node placement (C4) |
| W110–112 | query-columnar | v0.5: vectorized kernels + zone-map pushdown + group-by aggregate (C1) |
| W129 | model-check, sim-storage | Stateright BFT model (C2); storage⊕Byzantine fault composition (C4) |

Each insertion preserves frontmatter, day-headers, the Sunday ritual, and all existing checkboxes (additive
only, per the W5–W144 drill-rollout discipline). Inheritance-audit cadence (Appendix A) gains: `query-columnar`
v0.5 ≥0.50 @ W112; `txn` v1.1 @ W94.

---

## Part 6 — Cannot add additively (carried from the audit, unchanged)

1. Real networked/gossiped BFT or Raft → keep inside the VOPR `PacketSimulator` (sim-only constraint).
2. Standalone Percolator timestamp-oracle service → TSO = VSR commit point (C3).
3. DataFusion-style SQL engine with an RPC surface → narrow in-process columnar query only (C1).
4. TiKV scheduler/coprocessor as a distributed query framework → a single pushdown method in `lsm-core` (C3).
5. Spot order book / spot collateral → OUT (scope boundary (c); perps-only).
6. TLA⁺ as a build dependency → Stateright in-workspace; TLA⁺ specs read as model source only (C2).

---

## Open questions for the reviewer (decide before apply)

1. **`query-columnar` slot pressure.** v0.5 at W110–112 assumes those sustained-ops weeks have ~1 wk of
   capacity. Confirm, or move v0.5 to the M31 buffer beside `vector-db` (safer, later).
2. **`model-check` ambition.** Minimal = VSR + BFT safety only (proposed). Stretch = also model the
   `matching`+`ledger` linearizability. Default: minimal now, stretch noted as optional.
3. **`txn` v1.1 vs venue-store timing.** Build at W93–94 (proposed) or defer into the W97+ venue-store work
   where it's first consumed? W93–94 keeps it standalone-testable; W97+ keeps it just-in-time.
4. **Crate count.** This takes the workspace to ~38. Acceptable, or fold `model-check` into each consensus
   crate's `tests/` instead of a standalone crate (smaller count, less first-class)?
