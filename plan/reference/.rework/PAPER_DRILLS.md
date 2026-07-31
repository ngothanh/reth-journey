# Paper Drills — by-hand reasoning artifacts, one per concept family

**Why this exists.** The learner retains a concept only after reasoning it out **by hand on paper**, not
just reading or implementing it. (Validated empirically: re-deriving happens-before edges on paper hardened
ordering concepts that reading alone left fuzzy.) So every `**Build**`/`**Extend**`/`**Refactor**` exercise
must, in **Phase 1**, carry a 🧮 **Paper drill** block that selects the artifact(s) below matching the
exercise's core concepts — for **every** advanced concept AND domain skill it teaches, not only memory
ordering — and the learner writes them out in `notes/NN_<name>_drill.md` (transcribe physical paper if drawn
by hand).

**Distinct from the other Phase-1 docs:** the *prediction doc* is sealed guesses; the *design doc* is R/D
clauses. The **drill is the worked reasoning** — the drawing/chaining/tracing that actually builds the muscle.

**How an author uses it.** Per Build, pick 1–3 drills by ID keyed to the exercise's concepts. State the
**concrete instance** to work (never abstract — a specific struct, a 3-node tree, a 5-record log), the steps,
and the verdict/output line. Domain-knowledge builds get a domain drill (D-WALK / D-LAYOUT) just like
concurrency builds get D-HB.

---

## Catalog

### D-HB — Happens-before edge drawing  *(memory ordering: C1)*
Pick the load-bearing race. Two columns (the two threads). **1.** PO edges (each thread's ops top→bottom).
**2.** Mark every Release (which store?) and Acquire (which load?). **3.** Draw the RA edge — which Acquire
reads which Release's value? **4.** Chain: is there a path from `<the write>` to `<the read>`? **5.** Verdict:
reader sees the write? If the chain breaks, name the missing edge + the ordering that fixes it. **Then the
NEGATIVE:** weaken one op to `Relaxed`, draw the interleaving that now breaks.
*Output:* the edge diagram + "sees / doesn't see" verdict + the negative interleaving.

### D-RMW — Atomic RMW / CAS interleaving table  *(atomics mechanics: C2)*
Write the RMW (or CAS retry loop) as a **2-thread step table**: row = a step, columns = thread A / thread B /
the atomic's value. Show the lost-update a plain `load;store` would cause vs the RMW that prevents it. Mark, per
decision, **which guarantee carries it: atomicity (RMW-reads-latest) or ordering (Release/Acquire)** — they are
different jobs.
*Output:* the step table + a one-line "atomicity here, ordering there" annotation.

### D-UNSAFE — Invariant ledger  *(unsafe / UnsafeCell / raw ptr: C3)*
For each `unsafe` block: a 3-column row — **the unsafe op | the precondition it requires | the runtime fact
that guarantees that precondition**. Add a "what would make this UB?" line per row. The block is sound iff every
row's guarantee is real.
*Output:* the ledger table; flag any row whose guarantee is "a promise some *other* code must keep" (name that code).

### D-VAR — Variance substitution table  *(variance / PhantomData: C4)*
For the wrapper `F<T>`: can `F<'long>` stand in where `F<'short>` is wanted? Write the **read** that's safe and
the **write** that breaks it; conclude covariant / invariant / contravariant; pick the field/marker
(`NonNull`, `PhantomData<…>`, `&mut`) that yields it. Compare against the std analog (`&T`, `*mut T`, `Cell`).
*Output:* the table + the chosen marker + "matches/differs from std because …".

### D-BORROW — Ownership / borrow graph  *(lifetimes / RAII guards: C5)*
Draw the graph: nodes = values; **solid arrow = owns**, **dashed arrow = borrows-with-lifetime `'a`**. Mark
every borrow that must not outlive its owner, and which lifetime parameter encodes that. For a guard, show the
guard→parent dashed edge and the `Drop` that runs on scope-exit.
*Output:* the graph + "the type system rejects X because the `'a` bound …".

### D-LIFE — Value-lifecycle timeline  *(ownership / Drop / refcount / leak-safety: C6)*
A timeline: **birth** (where allocated) → **handles** (clone/share/move events) → **the single event that drops
the value** → assert "dropped exactly once, freed exactly once." For refcounts, a **count-transition table**
(each op → count before/after → does it free?).
*Output:* the timeline + "drop count = 1, free count = 1" proof; the leak/double-free that a wrong transition causes.

### D-SS — Send/Sync two-question worksheet  *(Send/Sync: C7)*
**Q1:** what does *sharing `&T`* across threads expose? → does it need `T: Sync`? **Q2:** what does *sending/
owning `T`* across threads expose (incl. cross-thread `Drop`)? → does it need `T: Send`? Derive the bound, then
write the **counterexample type** (a `Sync`-but-`!Send` or `!Send` type) the bound correctly rejects.
*Output:* the two answers, the derived bound, the rejected counterexample.

### D-STRUCT — Hand-execute on a concrete small instance  *(lock-free & ordinary data structures: C8, parts of C16)*
Draw the structure with **3 concrete nodes/slots**. Hand-execute the operation (push/pop/insert/split/find)
**step by step**, redrawing after each mutation. Then the **2-thread interleaving** at the linearization point
(the single atomic that commits the op) — show the racing case and why it's safe (or the bad case to prevent).
*Output:* the before/after drawings + the linearization-point interleaving.

### D-LOOM — Minimal model sketch  *(loom: C9)*
Before coding the model: write **threads × shared-ops** (keep it 2×~3), the **exact assertion**, and an
order-of-magnitude **interleaving-count** guess. Negative-control note: a real loom run takes ≥100 ms — if it's
instant, loom didn't intercept your atomics.
*Output:* the model spec + assertion + "should explore ~N schedules / ≥100 ms".

### D-WALK — Domain worked-example by hand  *(domain knowledge: C19 — RLP/MPT/ARIES/consensus/EVM/…)*
Take **ONE concrete small input** and execute the algorithm on paper, step by step. Examples:
- RLP: encode `[42, "dog"]` **byte by byte**; then decode your own bytes back.
- MPT: insert 3 keys with a shared nibble prefix; **draw the resulting branch/extension/leaf tree**.
- ARIES: trace analysis → redo → undo over a **5-record log** with one loser txn.
- Raft/VSR: write the **4-line message exchange** for one commit; mark the quorum.
- EVM: hand-execute a 5-opcode sequence, drawing the **stack after each**.
*Output:* the hand-trace; the invariant it makes concrete ("the root hash is deterministic because …").

### D-LAYOUT — Byte-layout drawing  *(memory layout / cache-line / false-sharing: C12, C17)*
Draw the type's **byte map**: offsets, field positions, padding bytes, alignment, and the **cache-line
boundaries** (64 B x86 / 128 B aarch64). Mark what straddles a line or false-shares with a neighbor. For
allocation: each `alloc` → its `Layout` → the **matching** `dealloc`; the zero-cap/sentinel case.
*Output:* the byte map with line boundaries; the straddle/false-share/Layout-mismatch hazard called out.

### D-MACRO — Expansion by hand  *(macros: C14 decl, C15 proc)*
Write the macro **input**, then the **expanded output token by token**. For a proc-macro, draw the
`syn` AST you parse → the `quote!` tokens you emit, naming every external path (`::core::…`) and every hygienic
binding. Include one **malformed input** and the `compile_error!` it must produce.
*Output:* input → expansion; the absolute-path/hygiene checklist; the reject-case message.

### D-ASYNC — State-machine drawing  *(async / Pin / Future: C18)*
Draw the **generated state enum**: each state = a span between `.await` points; list **what is stored across
each await** (the live locals). Mark what must be **pinned** and the self-reference (or borrow held across
await) that makes moving it unsound. Show where `poll` returns `Pending` vs `Ready`.
*Output:* the state diagram + "must pin because state K holds a self-reference / borrow across await".

### D-COST — Back-of-envelope cost ledger  *(perf / hot-path: C13, C20 — bar (c))*
(Pairs with the existing `*Back-of-envelope*` line.) Per hot-path op: **ns / cycles / allocs**, using
Jeff-Dean numbers (L1 0.5 ns, branch-mispredict 5 ns, L2 7 ns, mutex 25 ns, mem 100 ns, malloc ~50 ns, syscall
300 ns). Compute the **cliff ratio** (cold vs hot). State the budget and whether the design fits.
*Output:* the per-op cost table + cliff ratio + "fits / busts the budget by Xns".

---

## Tier 2 — Distributed / HFT-systems drills  *(W73–W144; validated against /jeff-dean + /hft-review)*

The 14 drills above carry the single-node, in-process, mechanical layer. The advanced half (consensus,
replicated determinism, deterministic-simulation, derivatives domain) needs these. Use them in any Build
touching VSR/Raft/BFT, the matching/risk/liquidation engines, `perp-dex-core`, cluster-VOPR, or `log-distributed`.

### D-QUORUM — Quorum / view-change safety-liveness hand-trace  *(consensus: VSR/Raft/BFT)*
Instance: N replicas, f faults, a fixed message-delivery order, a leader-crash point. Steps: draw replicas +
logs; mark the quorum sets (CFT `⌊n/2⌋+1`; BFT `2f+1` of `3f+1`) and show two quorums intersect in ≥1
*honest* node; trace a value `prepare → quorum → commit`; inject the leader crash; run the view-change
(DoViewChange/Timeout-Certificate); verify **safety** (no two conflicting commits) and **liveness** (new
leader makes progress). Negative: a quorum that *doesn't* intersect → the split-brain double-commit.
*Output:* the schedule trace + safety/liveness verdict + each replica's commit-point/high-QC.

### D-VOPR — Deterministic-simulation hand-derivation  *(VOPR / cluster-VOPR)*
Instance: a seed, an op generator, a fault schedule (loss/delay/reorder/partition/crash/misdirect). Steps:
write the seed→decisions table (each PRNG draw → its effect); hand-run a few ticks of the **safety phase**
under faults, then the **liveness phase** (heal quorum → converge); list the invariants checked at every
commit (linearizable-log, no-divergent-state-hash, tripwire-never-hit). Then the **nondeterminism-leak hunt**:
enumerate every leak (HashMap iteration, wall-clock, tokio on the sim path, OS-thread interleave, f64) that
would void byte-identical replay.
*Output:* the tick trace + invariant outcomes + the leak checklist.

### D-LIN — Linearizability / replicated-history legality  *(state-machine replication, single-log)*
Instance: an op log + an interleaving of client reads/writes across 3 replicas. Steps: draw each op's
real-time interval; find a single total order consistent with real-time + per-object semantics (the
linearization), marking each op's linearization point — OR exhibit the anomaly no total order explains. Show
where a *second* log or a wrong apply-order produces divergent state-hashes.
*Output:* the linearization (or the anomaly) + "single-log apply-order is deterministic because …".

### D-DET — Cross-replica / cross-arch determinism audit  *(the capstone spine; integer-money)*
Instance: a hot-path code path (margin recompute, match step). Steps: trace it; list EVERY nondeterminism
source — f64 arithmetic, HashMap/HashSet iteration, wall-clock/`Instant`, thread/tokio scheduling,
address-dependent logic, uninitialized-padding hashing — and each deterministic replacement (integer math,
sorted/IndexMap, injected clock, single PRNG). Prove the path is a pure function of `(state, op)`. For money:
integer-only, no f64 across replicas.
*Output:* the nondeterminism ledger + "replays bit-identically across replicas/arches because every source is eliminated."

### D-CAP — Distributed back-of-envelope  *(extends D-COST to scale; tail-not-mean)*
Instance: a throughput/fan-out target (e.g. 100k accounts × 10 mark-ticks/s). Steps: fan-out load
(→1M recomputes/s); shard arithmetic (per-core share, ns/op → per-tick burst vs the sub-tick **p99.99**
budget → does incremental recompute become *mandatory*?); batching envelope (1k Prepares/s × 1k ops/Prepare =
1M/s); consensus-latency derivation (rounds × [RTT + batch-verify + persist]); the **why-distribute** argument
(consensus is for reliability/durability, not throughput — one box clears ~1M/s). Jeff-Dean numbers; report
the tail.
*Output:* the capacity table + budget verdict (fits/busts) + "distribute for X, not throughput."

### D-FAIL — Failure-mode / blast-radius / recovery trace  *(design-for-failure)*
Instance: a fault scenario (node crash / rack loss / partition / disk fault). Steps: a table — each component
→ failure mode → **blast radius** (who sees a user-visible error?) → recovery path (re-replicate / re-elect /
replay) → the invariant stressed (quorum overlap, insurance-fund tripwire). Add the operational subtleties:
RAII fault-handle cleanup (Drop must not need a torn-down runtime), observer-effect subtraction (subtract
injection cost from latency attribution), leaked-fault → next-drill attribution corruption.
*Output:* the failure table + "survives single-{machine,rack} failure with no data loss because …" (or the gap).

### D-STALE — Staleness-as-correctness / consistency-per-operation  *(acceptance #5)*
Instance: two reads — a control/risk read and an analytics read — against a `commit_point`, an `applied_point`,
and an op-number token. Steps: assign each read its REQUIRED contract — control/risk = **read-after-commit**
from leader-local applied state (a stale risk read is a *correctness* bug); analytics = **bounded-staleness +
op-token** (zookie), metric-not-SLA. Show the "new enemy" anomaly if the wrong contract is used. Map
`commit_point` vs `applied_point` to the two finality/read boundaries.
*Output:* per-read contract assignment + the anomaly the wrong choice causes.

### D-SEAM — Interface-leak / swap-safety / zero-cost proof  *(optional; the `ConsensusBackbone` artifact)*
Instance: a trait seam + two impls (here-now VSR, future BFT). Steps: list the trait's methods; check no
current-impl assumption leaks (no quorum/auth/view vocabulary in the outer trait); show the future impl
type-checks against the SAME trait + call site with zero call-site changes; show the monomorphized inner seams
const-fold to nothing (only the outer trait is one indirect call/commit-batch).
*Output:* the leak audit + "swap-safe + zero-cost because …".

> Note: single-writer-principle and zero-alloc-hot-path reasoning are **not** separate drills — single-writer
> folds into **D-LAYOUT** (cache-line ownership) + **D-RMW**; zero-alloc folds into **D-COST**/**D-DET**.

---

## Authoring rule (goes in SPEC §4)
Every Build's Phase 1 ends with:
```
🧮 **Paper drill** — write in `notes/NN_<name>_drill.md`:
  - <D-XX> on <concrete instance>: <the one-line task>.  (repeat for each core concept, 1–3 total)
```
A Build that teaches a concept in the catalog but omits its drill is an automatic rework reject (rubric G9).
Domain-only weeks still get **D-WALK** (and **D-LAYOUT** where a wire/disk format is involved) — comprehensive
means *every* concept, domain included, gets its by-hand artifact.

**Tier-2 is mandatory** for any W73–W144 Build touching consensus (D-QUORUM), deterministic simulation
(D-VOPR), replication/single-log (D-LIN), money/replica determinism (D-DET), distributed capacity (D-CAP),
fault-tolerance (D-FAIL), or staleness contracts (D-STALE). A consensus/VOPR/replication Build that ships only
Tier-1 drills is under-covered → reject. The capstone weeks (W103/W108/W117/W118/W143) will typically carry
3–4 Tier-2 drills each.
