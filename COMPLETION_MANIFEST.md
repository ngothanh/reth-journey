# COMPLETION_MANIFEST — Inheritance Plan v3.1 (production-grade, locked scope)

> **Scope (locked, reaffirmed):** the ENGINE of a Hyperliquid-class venue — matching / risk / liquidation / oracle / settlement as a **verified deterministic replicated state machine**, plus the **committed BFT terminal apex**. Product surface (vault/spot/staking/bridge) is OUT (scope-c); fully-on-chain is CONDITIONAL (scope-b). See `README.md` "Scope Boundary" + `memory/project_plan_scope_lock.md`.
> **Nature:** this is a **planning repo** (week-by-week spec). "Green" below = **spec-level** (artifact authored, internally consistent, acceptance stated as checkable conditions). **Execution-green** (compiling + running tests) is discharged when each crate is built during the W1→W144 execution window; the specs are written so pass/fail is unambiguous then.

## Review score: **10 / 10 as a plan** (2026-06-10 triple adversarial re-review: jeff-dean × hft-review × consensus-expert; **re-affirmed 2026-07-17** after a five-axis re-audit — G9 paper-drill sweep, G10 cadence-ledger verification, v3.2-integration audit, locked-corrections regression grep, 10-week rubric spot-check — surfaced 9 blocking + ~25 major findings across the layers added post-attestation, all fixed and grep-verified; see §7)
Three independent adversarial reviews (jeff-dean on the strategy layer; hft-review on the W091–W117 seam spine; hybrid Thompson×consensus-expert on Track B W118–W143) surfaced **10 blocking + ~30 major findings**, all fixed and grep-verified the same day. The load-bearing corrections, now locked: the 2-chain commit rule carries the **consecutive-round condition** (B′.round == B.round + 1, Jolteon); the permanent-prune predicate keys on **ancestry against the high-QC chain**, not the committed chain; quorum = **n − ⌊(n−1)/3⌋** (2f+1 only on the 3f+1 lattice); QC uniqueness is per **round**, not per height; persist-before-vote is in the 2 ms budget; the cross-margin conservatism bound is **0.65×iso (tight)**; `Account` fits 256B via the `NonZeroU16` niche (216B); `settle()` returns a `SettleTicket(ProposalHandle)` per the O5 surface; W099 is a genuine delta on W098 (dense-Vec pinned); the 8-scenario / p99≤2ms W118 locks are propagated to README+INDEX; runtime hours are defined as **node-hours**; the coverage score is a declared **uniform mean** with gates 45 (M24) / 65 (M30) / 80 (M36); every published latency is topology-labeled (loopback/LAN); 76 dead `RECONCILED_PLAN_v3` references repointed at README (§-anchor map added).
"10/10 as a plan" means: spec-complete under the locked scope, cross-document-consistent, physically-plausible numbers, protocol statements correct, every acceptance criterion owned by a week with a checkable condition. **Execution-green is explicitly out of a plan's scope** — it is the W1→W144 build's obligation, for which these specs are the contract; the next real defect found in execution re-opens this score honestly rather than falsifying it.

---

## 1. Remaining-work queue — authored + static-consistency-clean

| Cluster | Weeks | State | Note |
|---|---|---|---|
| Frozen-surface reconciles (Phase 5/6) | W74, W92, W103, W104, W108, W117 | ✅ Y | amended-`ConsensusBackbone` notes; #4/#5 at W108; #9 at W103+W117 |
| Phase 1–6 deep weeks | W1–W117 | ✅ Y (pre-authored, v3-reconciled) | line-count audit confirmed these are full weeks, not stubs |
| Phase 7 bet-path light weeks | W120–W144 (Track A) | ✅ Y (pre-authored light weeks) | ops/blog/bet-path; legitimately shorter (no bar-c Build) |
| **BFT apex — build** | W119 (gate+Quorum+Auth), W120–W121 (ViewManager), W122 (QC), W123 (commit rule), W124 (Equivocation), W125 (liveness), W126 (happy-path), W127 (handle wiring), W128 (integration) | ✅ Y | all four W91 seams + commit rule + liveness + trait-integration; empty `Replica`-core diffs |
| **BFT apex — prove** | W129 (Byzantine VOPR, 8 scenarios + canary) | ✅ Y | reuses W108 harness + W128 adapter |
| **BFT apex — sustain** | W130–W134, W136, W139–W142 | ✅ Y (reference-notes) | nightly VOPR seed-sweep + latency hardening; **no new mechanism** (handoffs in W129/W135/W138) |
| **BFT apex — latency/swap/accept** | W135 (X≤2ms), W137 (VSR→BFT swap = v2), W138 (whole-venue v2 VOPR), W143 (acceptance gate) | ✅ Y | empty engine-diff swap; v1.0 ladder holds under BFT |

**Static-consistency:** the BFT Track-B sections share a consistent vocabulary (`Replica<SM,Q,A,V,E>`, the four seam names, `ProposalHandle`/`drain_abandoned`/`commit_point`/`OpNumber`) across W119→W143; each cites the prior week's handoff. No signature drift found.

## 2. Frozen-surface reference check — `ConsensusBackbone` (post-O5 amendment)
Every week touching consensus uses the **amended** surface consistently: `propose → ProposalHandle` (not OpNumber), `drain_committed` + `Commit::proposal_handle()`, `drain_abandoned`, `commit_point()`/`applied_point()`, `OpNumber` commit-time-only.
- Declared: W91. VSR impl behind it: W91/W92. Engine depends on the trait: W103 (acceptance #9, dependency formation) + W104 (settle) + W117 (convergence). BFT impl of the same trait: W128. Swap: W137 (empty engine diff). ✅ consistent.

## 3. O1–O6 ledger (none silently dropped)
| O | Disposition | Owning week(s) |
|---|---|---|
| O1 — auth cost | **Resolved impl-only**: ed25519 + batch-verify at N=4; BLS deferred to large-N behind the `Authenticator` seam | W118 (decision), W119 (`SignedPeers`), W135 (batch-verify meets X) |
| O2 — pipeline depth / view-change shape | **Impl-only** (FIX-3 associated types settled the interface); pipeline depth absorbed into X | W118, W121, W123, W135 |
| O3 — certificate shape | **Impl-only**: `Certificate` opaque at the trait boundary; conditional-(b) external verifier would reopen | W118, W122 |
| O4 — OpNumber densification | **Impl-only**: dense contiguous OpNumber decoupled from BFT height, assigned at commit | W118, W123, W129 (#8) |
| O5 — op-level abandonment surface | **RESOLVED → outer-trait amendment** (`ProposalHandle`/`drain_abandoned`/`proposal_handle`); applied W91; discharged | W118 (resolution), W91 (amendment), W125 (liveness), W127 (end-to-end) |
| O6 — speculative apply | **Consciously deferred to v2** (not needed to meet X); trigger recorded | W118 |

## 4. Dependency check (no forward dependencies)
Build order holds: substrate (L0–L4, W1–W90) → ledger/matching on the trait (W74/W91/W92) → domain crates (oracle W77–80, risk W85–99, liquidation W100–102) → capstone assembly (W103–113) → ops + cluster-VOPR (W105–108) → M30 gate (W117) → v1.5 + BFT apex (W118–143). The BFT apex consumes only already-built artifacts; W137's swap consumes W128 (adapter) which consumes W119–125 (seams). ✅ no week depends on a later week's artifact.

## 5. Acceptance-criteria index (→ owning week + checkable condition, for the execution phase)
| Criterion | Week | Checkable condition |
|---|---|---|
| #1 multi-instrument cross-margin netting | W97–99 / W117 | BTC+ETH cross-margin credits exactly 70% of the offset; single-instrument fails |
| #2 3-node VSR, one hot-path log | W74/W92/W103 / W108 | order log = settlement journal = VSR log; crash/restart no lost commits |
| #3 partial-liq + fee-accrued insurance fund waterfall | W101–102 / W108 | waterfall position-margin→fund; tripwire never hit at 10% (swept 5/10/20%) |
| #4 SettlementId=op-number, finality from commit point | W104 / W108 | `finality_status` derives from `commit_point()`; no side table; resubmit=no-op |
| #5 two projection staleness contracts | W103/W107 / W108 | control=read-after-commit local; analytics=bounded-staleness+op-token; risk-on-analytics FAILS |
| #6 RWA-aware seams before features | W104 / W117 | oracle iface + parameterized funding + instrument abstraction; demonstrated pre-feature |
| #7 cluster-VOPR StateChecker green | W108 / W117 | linearizability + convergence + tripwire-never-hit across schedules |
| #8 scale @100k accounts, incremental margin | W98–99/W111 / W117 | per-mark-tick p99.99 met; full recompute only on circuit-breaker |
| #9 consensus behind `ConsensusBackbone` interface | **W103** (dependency formation) / W117 (converge) | engine binds the trait, not consensus-vsr; stub-swap test green |
| **BFT MVP — protocol family** | W121/W123 / W143 | Jolteon 2-chain + Timeout-Certificate |
| **BFT MVP — N=4 (f=1)** | W119 / W143 | `BftQuorum::threshold(4)=3` |
| **BFT MVP — latency X** | W135 / W143 | committed-batch p99 ≤ 2 ms, N=4 LAN, coordinated-omission-corrected |
| **BFT MVP — 8 Byzantine scenarios** | W129 / W138 / W143 | StateChecker green + planted-bug canary catches a reverted seam |
| **BFT MVP — zero-rewrite swap** | W137 / W143 | VSR→BFT = four-type-params; empty `perp-dex-core` engine diff |

## 6. HALTs hit + clearance
| HALT | Resolution | Cleared by |
|---|---|---|
| W91 O5-amendment gate | amendment applied (8 items); orphan test = orphan-without-supersession | **user** |
| Flag-A (acceptance placement) | #9→W103, #4/#5→W108, convergence→W117 | **user** |
| W103 #9-at-dependency-formation | stub-swap test placed at W103, not deferred | **user** |
| W125–W128 permanent-prune liveness | **impl-only**, no outer-trait touch (drain_abandoned from O5 sufficient) | **user delegated → cleared this turn** |
| RWA-v1.5 re-slot | **moot** — scope kept (no full-Hyperliquid expansion) | **user (scope decision)** |

---

## Completion statement
**Every cluster in §1 is authored and static-consistency-clean; every frozen-surface reference uses the amended trait; O1–O6 are all accounted; no forward dependencies; every acceptance criterion (incl. BFT MVP + the 8 Byzantine scenarios) maps to an owning week + a checkable condition; all HALTs were cleared by the user.**

The plan is **complete to v3.1 at the spec level (spec-complete; score re-stated after the 2026-06 adversarial re-review — see header)** under the locked scope. Not self-certified as "executed" — execution-green is the obligation of the W1→W144 build itself, for which these specs are the contract.

## 7. v3.2 + pedagogy-layer addendum (2026-07-17)

Addendum only — the v3.1 attestation above and the header score are unchanged and not re-certified here.

1. **v3.2 durable-infra additions authored + audited.** `query-columnar` (W84 v0.1 → W110 v0.5), `model-check` (W90 VSR model + W129 BFT model), `txn` v1.1 Percolator MVCC (W94), and the `sim-storage` folds (W88 / W108 / W129) + perp-domain folds are authored per `plan/.rework/HYPERCORE_ADDITIONS.md`. Forward-dependency sweep of the v3.2 flows — W84→W110→W103, W90→W94→W103, W88→W108→W129 — clean (no forward dependencies).
2. **Coverage + gates unchanged.** The 8-system coverage uniform mean and the 45/65/80 gates are unchanged by v3.2: the new crates map into existing system rows per HYPERCORE_ADDITIONS Part 4 item 5 (no new system rows).
3. **Pedagogy layer re-audited.** The paper-drill layer (🧮 per-Build Phase-1 drills, rubric G9; catalog at `plan/.rework/PAPER_DRILLS.md`) and the concept-cadence ledger (`plan/concept_cadence.md`, rubric G10) were added after the v3.1 attestation and re-audited 2026-07-17 (G9: six blocking drill absences closed — W025/W054/W056/W060/W061 + the W084 v3.2 Builds; G10: the proc-macro W39→W60 and decl-macro W38→W51/W63 droughts closed with landed touchpoints, Tier-2 ledger rows re-trued against actual drill tags).
4. **W143 BFT acceptance row hardened.** The Byzantine-scenario acceptance row now includes the W129 `model-check` Stateright complement (Jolteon 2-chain + TC, N=4 f=1; bounded-depth exhaustive + planted-bug counterexample) as a co-equal FAIL condition alongside the sampled VOPR — a FAIL in either fails the row.
