# Safe memory reclamation: inventory and contract

Source research for `crates/reclaim`. Four parallel source reads: folly's
`Hazptr*.h`, `crossbeam-epoch`, the haphazard-vs-folly gap, and the SMR
literature. 192 items. Each is marked core / important / optional / skip for a
from-scratch learning reimplementation.

---

## The finding that matters

A reclamation scheme's safety argument has **two halves**, and every
grace-period or reservation scheme implements only one of them:

> **(A) Unobtainability** — after `retire(p)`, no thread that does not already
> hold `p` may obtain it.
>
> **(B) Grace** — threads that already hold `p` eventually let go.
>
> Schemes supply (B). **The data structure must supply (A).**
> "Unlink before retire" is not a style guideline. It is the missing half of
> the proof.

The precise form of (A) is about **protect sources**, not heap edges: the set
of atomics ever passed to `protect` is the root set, and `retire(p)` is sound
once `p` is unreachable *from that root set*. A heap edge nobody protects
through is not a root.

This is why hazard pointers apply to only 3 of 18 data structures in Singh's
survey, and it is the whole reason `SegQueue` was hard: it protects through
`cur.next`, which makes every retired segment permanently obtainable.

---

## The landscape

Every reclamation family, its read cost, whether garbage is bounded, and the structural precondition it imposes. The Notes section answers the applied question: which of these can serve a chain whose retired nodes stay reachable through a predecessor forever.

### CORE (16)

**`No reclamation / leaking (retire-and-never-free)`**
— Mechanism: retire() appends the node to a list and never frees it, so readers can never observe freed memory; per-read cost: zero; garbage: UNBOUNDED (grows with every retire); progress: wait-free; precondition: none at all — this is the only scheme with no structural requirement on the data structure.
  
*Forced by:* It is the correctness baseline every SMR scheme is measured against: Nikolaev/Ravindran state the whole point of SMR is bounding memory, 'as otherwise we could simply leak memory', so leaking is the control experiment that isolates what a reclaimer actually costs.

**`Per-object lock-free reference counting`**
— Mechanism: each node carries an atomic count; a reader increments before dereferencing and decrements when done, and whoever drives the count to zero frees it; per-read cost: 2 atomic RMWs per node touched plus the cache-line invalidation they cause; garbage: BOUNDED (freed the instant the last reference drops, except cycles); progress: lock-free (needs a safe increment-if-not-zero, historically DCAS or split counting); precondition: every link that can reach the node must itself own a count, pointer fields need a deref/CAS wrapper, and cycles must be broken by hand.
  
*Forced by:* It is the only family that is 'asynchronous' (the thread holding the last reference frees, so reclamation work is balanced across threads) and needs no unreachability premise — but the per-access RMW on a shared counter is so expensive on read-dominated workloads that it 'can diminish or negate any performance gains over traditional locking'.

**`Hazard pointers (Michael)`**
— Mechanism: before dereferencing, a reader publishes the pointer into a single-writer/multi-reader slot and validates that it is still reachable; a reclaimer scans all slots and frees only retired nodes nobody announced; per-read cost: one store to the slot + one store-load fence + one validating re-load, per pointer dereferenced (~2 ns for protect+reset in folly when the fence is made asymmetric via membarrier, otherwise a full fence); garbage: BOUNDED at k·p nodes (k slots per thread, p threads); progress: lock-free in general, wait-free if the data structure can restart; precondition: a node must be unlinked (unreachable from the roots) BEFORE retire, and the validation must actually prove reachability-at-announce-time — so traversals must never pass through an already-retired node.
  
*Forced by:* EBR lets one stalled thread pin unbounded memory; HP pays a per-pointer fence to get per-object precision and a hard garbage bound, which is why it is the scheme shipping in folly, MongoDB and the C++26 proposal. The fence in step 2 is not optional: without it the announcement can be reordered after the reachability check, and you announce an already-freed node.

**`protect-and-validate (the HP/HE read protocol)`**
— Mechanism: the three-step contract — (1) write the pointer to a shared SWMR slot, (2) store-load fence, (3) re-read the source location and confirm the pointer is unchanged/still reachable, else retry; per-read cost: it *is* the per-read cost of HP/HE/IBR/WFE/Crystalline; garbage: n/a; progress: the retry loop is the single non-wait-free point in all of these schemes (WFE/Crystalline-W exist only to remove it); precondition: 'unchanged' must imply 'not retired' — the protocol is vacuous in any structure where a retired node stays linked.
  
*Forced by:* Isolating this protocol is what makes the whole reservation family legible: every scheme in it differs only in *what* you publish (a pointer, an era, an interval) and how you bound the retry, and every applicability failure in the literature is a failure of step 3.

**`folly hazptr_obj_base_linked (link counting)`**
— Mechanism: hybrid — hazard pointers protect readers, while a per-object *link count* tracks how many links still reach the object, so an object is retired automatically when its last incoming link goes away; per-read cost: 'no extra overhead for readers' (the counting is on the mutator side); garbage: BOUNDED; progress: lock-free; precondition: the structure must declare its links (hazptr_root) and the retiring thread must know the link topology — it handles 'removal of objects is uncertain', i.e. when you cannot tell whether a node became unreachable.
  
*Forced by:* Plain HP needs you to know the exact moment a node became unreachable before you may retire it; in linked structures (lists, tries, maps with chained nodes) that moment is genuinely unknowable to the thread doing the unlink, so folly reintroduces reference counting *on links only* to compute it — this is the production answer to exactly the 'still reachable through a predecessor' problem.

**`HP++ (Jung, Lee, Kim, Kang)`**
— Mechanism: under-approximate unreachability during validation and then *patch up* the resulting false negatives — the thread that unlinks a node also invalidates (via an added invalidate bit) the nodes a concurrent optimistic traversal could still reach from it, so the traversal detects the hazard; per-read cost: HP's cost plus the invalidate-bit check (HP++ is measurably slower than HP); garbage: BOUNDED; progress: lock-free; precondition: a per-node invalidate bit plus an extended API for unlink and for computing the set to invalidate — and reachability-after-retire must be TRANSIENT, with a thread eventually doing the unlink.
  
*Forced by:* HP is applicable to only 3 of 18 surveyed data structures because the validation step fails for any structure that traverses logically-deleted nodes (Harris list, lazy list, almost every BST and skip list); HP++ is the first extension that keeps optimistic traversal and still bounds garbage.

**`Epoch-based reclamation (EBR)`**
— Mechanism: a global epoch counter advances only when every thread has announced the latest epoch; each thread keeps three limbo bags (one per recent epoch) and frees its oldest bag on each epoch change; per-read cost: zero — one announce at operation entry and one at exit (~5 ns for folly's rcu_reader ctor+dtor), amortized over the whole operation; garbage: UNBOUNDED (one stalled thread freezes the epoch and pins everything); progress: BLOCKING (unbounded memory means blocking once memory is exhausted); precondition: unlink-before-retire, and no reference may be held across the end of a critical section.
  
*Forced by:* HP's per-pointer fence is the dominant cost in read-heavy traversals; EBR moves all the bookkeeping to operation boundaries so traversal is free, accepting that memory safety now depends on every thread making progress.

**`crossbeam-epoch (Rust EBR)`**
— Mechanism: pin() returns a Guard that registers the thread in the current epoch; Atomic<T>/Shared<T> wrap loads, defer_destroy() queues garbage into per-thread bags that flush to a global queue, and the collector advances the epoch when all pinned participants are current; per-read cost: a thread-local pin counter bump (nested pins are free) plus one SeqCst fence on the outermost pin; garbage: UNBOUNDED while a Guard is held; progress: blocking; precondition: unlink-before-retire, and a Guard must not be held across an await or a long loop.
  
*Forced by:* It is the Rust-native reference implementation the learner will be integrating against, and the API (Guard lifetime ties the safety proof to the borrow checker) is the single best demonstration of how to make the 'no reference outlives the critical section' precondition a type-system invariant rather than a comment.

**`QSBR (quiescent-state-based reclamation)`**
— Mechanism: identical to EBR but the announcement is implicit — threads pass through designated quiescent states (points where they provably hold no references to shared nodes) and the grace period ends when every thread has been quiescent once; per-read cost: literally zero in the read path, one announcement per outer loop iteration or per operation; garbage: UNBOUNDED; progress: BLOCKING; precondition: the program must have identifiable quiescent states, which is a whole-program property, not a data-structure property.
  
*Forced by:* EBR's entry/exit announce is still a shared store per operation; QSBR deletes even that by hoisting the announcement to a natural program boundary (end of a request, bottom of an event loop) — the cheapest possible reader, bought with the strongest possible precondition on the surrounding application.

**`RCU (read-copy-update)`**
— Mechanism: readers mark a critical section (rcu_read_lock/rcu_reader/rcu_domain scoped_lock); writers publish a new version, then wait for a grace period — all pre-existing critical sections to end — before freeing the old one (synchronize_rcu / rcu_synchronize, or asynchronously via call_rcu / rcu_retire); per-read cost: ~0 (Linux non-preemptible RCU: nothing at all; folly: ~5 ns with asymmetric fences; folly uses two epochs because of late readers of the version counter); garbage: UNBOUNDED (a blocking reader delays every deferred callback in the domain); progress: BLOCKING; precondition: unlink-before-retire (publish-then-wait), no reference may escape the critical section, and the deleter must not block on anything a reader holds.
  
*Forced by:* RCU is EBR generalized into a whole synchronization discipline: it protects an entire critical section rather than individual pointers, which makes code dramatically simpler but makes reclamation all-or-nothing. Its flavors (liburcu QSBR/memb/signal/bp, Linux Tree RCU and SRCU) are a catalogue of every way to detect a grace period.

**`Hazard eras (HE)`**
— Mechanism: a global monotonically increasing era counter; each node records a birth era and a retire era, and a reader reserves the current *era* (not the pointer) when it accesses a node — a node is freed only if its [birth, retire] lifespan intersects no reserved era; per-read cost: load the global era and compare it with the era already in your slot; you only store+fence when the era changed, so the fence frequency is proportional to the era-advance rate (a tunable trade against memory); garbage: bounded by the live set at each reserved era — more than HP's k·p and proportional to structure size, and sources disagree on whether to call it bounded; progress: lock-free (wait-free with restarting, blocking across multiple data structures because the global era may never converge); precondition: unlink-before-retire, a 3-word node header (birth era, retire era, list link), and the same no-traversal-through-retired-nodes rule as HP.
  
*Forced by:* HP's cost is one fence per *pointer*; HE's insight is that one era reservation covers every pointer you read while that era lasts, so an N-node traversal costs one fence instead of N. You pay for it with node-layout changes and coarser garbage bounds.

**`Interval-based reclamation (IBR / 2GEIBR)`**
— Mechanism: a thread reserves an *interval* of eras — from the era it saw when its operation began up to the latest era it saw on its last node access — and a node is freed only if its lifespan misses every reserved interval; per-read cost: update the interval's upper bound only when the global epoch changed, so amortized ~0; garbage: UNBOUNDED in theory (a long operation keeps extending its interval and pins everything allocated meanwhile; IBR's authors advise restarting such operations); progress: lock-free only if restarting is allowed, otherwise blocking; precondition: unlink-before-retire, 3-word node header, and the data structure must be able to restart long operations — trivial for lists, 'more problematic for complex data structures'.
  
*Forced by:* It is the honest middle of the design space and the cleanest statement of the real trade: HP reserves a point (precise, expensive), EBR reserves everything from now on (free, unbounded), IBR reserves an interval. Seeing it next to HE makes 'precision of the reservation' a single tunable axis.

**`Hyaline / Hyaline-1 / Hyaline-S / Hyaline-1S (reserve-to-free)`**
— Mechanism: invert the direction — instead of readers reserving nodes they will access, the *retiring* thread records how many threads are currently active and links its batch of retired nodes into a 2D grid (rows = active threads, columns = batches); each thread, when it goes inactive, walks its row decrementing batch reference counts, and the last decrementer frees the batch; per-read cost: ~0 like EBR, since the only reference-count traffic is at retire and at operation end; garbage: UNBOUNDED for Hyaline/Hyaline-1 (a stalled thread never decrements, and others keep attaching to its row), bounded-ish for Hyaline-S/-1S which add birth eras so nodes allocated after a thread stalled can still be freed; progress: BLOCKING (Hyaline-1), lock-free with restarting (Hyaline-1S); precondition: unlink-before-retire, a 3-word header, batched retirement, and restart-on-long-operation for the -S variants.
  
*Forced by:* Reclamation in EBR/HP/IBR is *synchronous and unbalanced* — only threads that modify data ever reclaim, so in a read-dominated workload most threads do no reclamation work and memory piles up. Hyaline makes reclamation asynchronous like reference counting (any thread can free any other thread's garbage) while keeping the counter traffic off the read path.

**`VBR (version-based reclamation)`**
— Mechanism: fully optimistic — retired nodes are recycled immediately without waiting for anyone; each node carries a birth epoch and a retire epoch, each mutable field carries a version, and a reader compares the global epoch against its last-read value on every field read, rolling back to a checkpoint if it changed; writes are WCAS against the version so a stale write fails; per-read cost: on strongly ordered machines (x86/SPARC TSO) *no* extra shared writes and *no* fences — just a comparison against a usually-cached global epoch; garbage: BOUNDED and small (quarantine is tiny, a non-cooperative thread cannot stall reclamation); progress: lock-free (not wait-free); precondition: a type-preserving allocator (memory is reused as the same type and pages are never returned to the OS), per-field version numbers, WCAS for every write, and the operation must be rollback-able — but NO unreachability requirement: it explicitly permits reading a node that has already been reclaimed and reused.
  
*Forced by:* Every other robust scheme pays on the read path to *prevent* access to freed memory. VBR asks the opposite question — let the access happen, then detect it — which removes fences entirely and removes the unlink-before-retire premise. The price is that it 'lacks a uniform set of API operations' and needs data-structure changes (roll-back instructions), so it forfeits easy integration.

**`crossbeam-queue SegQueue per-slot WRITE/READ/DESTROY state bits`**
— Mechanism: each slot carries WRITE (producer finished), READ (consumer finished) and DESTROY bits; the consumer that leaves a block first advances head.block to the next block, then walks the slots setting DESTROY — if it finds a slot with READ unset it hands destruction off to that straggler, and the last participant frees the block; per-read cost: zero extra (the fetch_or that sets READ is needed anyway); garbage: BOUNDED at essentially one block; progress: lock-free (with a spin in wait_write/wait_next); precondition: the set of threads that can touch a slot must be statically bounded and each must announce departure, indices must be monotone so no new accessor can arrive, and head.block must be advanced off the block before destruction begins — i.e. this is still unlink-before-retire plus a distributed refcount with an exactly known participant set.
  
*Forced by:* When you know a priori that a block has exactly BLOCK_CAP slots and at most one producer and one consumer per slot, a general SMR scheme is enormous overkill — a 3-bit-per-slot handshake gives immediate, bounded, zero-read-cost reclamation. It is the best example in production Rust of 'restructure the data structure instead of adding a reclaimer'.

**`The ERA theorem (Sheffi & Petrank impossibility result)`**
— Mechanism: a proof that no SMR scheme can provide all three of Ease of integration, Robustness (bounded garbage despite stalls) and Applicability (works for a wide class of data structures) — at most two of the three; per-read cost: n/a; garbage: n/a; progress: n/a; precondition: it is the theorem that explains why every scheme above has exactly one glaring weakness.
  
*Forced by:* It converts the confusing scheme zoo into a 2-of-3 lattice: EBR/RCU pick integration+applicability (not robust), HP/HE/Hyaline/Crystalline pick integration+robustness (not applicable to optimistic traversals), VBR/NBR/FA pick robustness+applicability (not easily integrated). Knowing it stops you looking for the scheme that wins everything.

### IMPORTANT (13)

**`Split / deferred reference counting (DRC, update coalescing, immediate RC)`**
— Mechanism: keep the per-object count but use a cheaper scheme (hazard pointers, IBR or Hyaline) to protect the *counter* so that increments/decrements can be batched, coalesced or deferred out of the read path; per-read cost: one hazard-pointer protect (a store + fence) instead of two shared RMWs, amortized toward constant; garbage: BOUNDED; progress: lock-free (DRC is not wait-free); precondition: same as the underlying protector, plus a managed-pointer API that wraps every load.
  
*Forced by:* Classical RC's cost is all in the read path while its *usability* (automatic, no manual retire, no unreachability reasoning) is its main virtue; deferred RC buys the usability back at near-HP cost — Anderson et al. achieve constant-time overhead, and showed the HP protector can be swapped for IBR or Hyaline.

**`folly::atomic_shared_ptr`**
— Mechanism: packs a 48-bit control-block address with a 16-bit 'local' count into one 64-bit word so a load() is a single lock xchg instead of cmpxchg16b, flushing the local count into the global count only occasionally; per-read cost: usually one atomic RMW (batched), not three; garbage: BOUNDED (it is reference counting); progress: lock-free only while fewer than EXTERNAL_OFFSET (0x2000) threads contend, otherwise it spin-yields — not wait-free; precondition: the link must *be* the atomic_shared_ptr (no raw next pointer), pointer alignment/tag bits are consumed, and aliased shared_ptrs cost an extra allocation.
  
*Forced by:* An atomic<shared_ptr> cannot be done naively because loading needs to bump a count that lives at the far end of the pointer you have not yet safely acquired; packing a local count beside the pointer makes the acquire a single atomic, and batching the flush removes the second and third atomics.

**`arc_swap 'debt' slots (Rust ArcSwap)`**
— Mechanism: stripped-down hazard pointers where a 'debt' is an owed Arc strong count — a reader parks the pointer in one of a few per-thread slots (fast, fallible) or falls back to a real Arc clone, and a writer that wants to drop the old value walks the registry and 'pays' the debts by incrementing the real count; per-read cost: one relaxed load plus one CAS into a slot on the fast path, no fence; garbage: BOUNDED (it degrades to Arc refcounting); progress: lock-free fast path with a sharded-spinlock-ish fallback (the 'generation lock'); precondition: the slot must be an ArcSwap and Guards must be dropped promptly, and per-thread nodes are never removed from the registry.
  
*Forced by:* It is the production answer to 'I want a cheap read-mostly atomic Arc in Rust' and is the clearest small example of the hazard-pointer idea applied to a *smart pointer* rather than to nodes — the debt framing (you owe a count, the writer pays it) is a genuinely different way to see HP.

**`Pass-the-buck (Herlihy, Luchangco, Martin, Moir)`**
— Mechanism: readers post 'guards' on values they hold; a thread that wants to free a value it cannot free hands ('passes') responsibility for it to the guarding thread via a trap/hand-off, so no one ever waits; per-read cost: comparable to HP (a post plus a validation); garbage: BOUNDED; progress: closer to wait-free than plain HP because the freeing obligation is transferred rather than retried; precondition: same unlink-before-retire premise as HP.
  
*Forced by:* It answers 'who frees a node that is still guarded?' with delegation instead of a limbo list and a rescan — the same move that reappears decades later in Hyaline's 'reserve to free' and in deferred RC.

**`Asymmetric thread fence (membarrier-based; folly asymmetric_thread_fence_light/heavy)`**
— Mechanism: replace the reader's hardware store-load fence with a bare compiler barrier, and make the rare reclaimer pay a process-wide fence via the membarrier() syscall (MEMBARRIER_CMD_PRIVATE_EXPEDITED); per-read cost: drops from a real fence to zero instructions — folly measures ~2 ns for hazptr protect+reset or rcu_reader construct+destruct on a 2.5 GHz Haswell; garbage: unchanged; progress: the reclaimer side becomes a syscall (so blocking); precondition: OS support, and the reclaimer must be rare relative to reads.
  
*Forced by:* The HP fence is pure asymmetric-synchronization waste: billions of readers pay it so that an occasional reclaimer sees fresh announcements. Pushing it onto the reclaimer is the single biggest constant-factor win available in HP, and is why folly's HP can compete with EBR.

**`folly hazptr_obj_cohort`**
— Mechanism: group retired objects into a cohort whose retired-list length is bounded, so reclamation is amortized and tied to the cohort rather than to a global domain list, and the cohort can be reclaimed synchronously on destruction; per-read cost: zero (retire-side only); garbage: BOUNDED per cohort; progress: unchanged from HP; precondition: objects must be created knowing their cohort, which is an intrusive change to the object.
  
*Forced by:* A single global domain retire-list gives unpredictable reclamation latency and unbounded per-domain garbage when one container is hot; cohorts make 'this container's garbage' a first-class, destructible unit — the thing you need when a map is destroyed and its nodes must provably be gone.

**`DEBRA (distributed epoch-based reclamation)`**
— Mechanism: EBR with the epoch-advance work distributed — O(1) steps at the start and end of each operation and O(1) per retired record, with each thread checking only a slice of the other threads' announcements per operation; per-read cost: zero; garbage: UNBOUNDED; progress: BLOCKING; precondition: unlink-before-retire plus a per-operation announce.
  
*Forced by:* Classical EBR's epoch advance scans all p announcements, so its overhead grows with thread count exactly where you wanted scalability; DEBRA amortizes the scan to O(1) per operation, making EBR's constant genuinely small.

**`DEBRA+ (fault-tolerant EBR via neutralizing signals)`**
— Mechanism: when a thread stalls and blocks the epoch, the reclaimer sends it a POSIX signal that 'neutralizes' it — the signalled thread abandons and restarts its operation, releasing its reservations; per-read cost: zero; garbage: BOUNDED in compatible non-blocking structures; progress: BLOCKING in the strict sense (POSIX signal delivery takes locks internally); precondition: the data structure must be *recoverable* — a thread may be restarted mid-update, so you must supply structure-specific recovery code, and no lock may be held (restarting a lock holder deadlocks), which is why it applies to only 4 of 18 surveyed structures.
  
*Forced by:* It is the first scheme to bound EBR's garbage, and it does so by inverting who waits: instead of the reclaimer waiting for the straggler, the reclaimer *interrupts* it. That inversion — neutralization — becomes its own paradigm (NBR, publish-on-ping).

**`NBR / NBR+ (neutralization-based reclamation)`**
— Mechanism: split each operation into a read phase and a write phase; during the read phase a thread is restartable and a reclaimer may neutralize it with a signal, and before entering the write phase the thread reserves (HP-style) every pointer it will write; per-read cost: two plain stores per *phase*, not per read, and no fences — atomic reads and writes only; garbage: BOUNDED at k(p−1) records; progress: conditionally lock-free (depends on OS signals); precondition: every read phase must restart from the root, and all pointers used in a write phase must be reserved before entering it — it applies to 11 of 18 surveyed structures.
  
*Forced by:* HP pays a fence per pointer; EBR pays unbounded memory. NBR notices that you only need per-pointer precision for the pointers you will *write*, and that a signal can replace the fence for everything you merely read — giving near-EBR speed with HP-like bounds, at the cost of a restart-from-root requirement.

**`Wait-free eras (WFE)`**
— Mechanism: HE/IBR plus a fast-path/slow-path helping protocol — a thread that keeps failing to reserve the current era (because the era keeps changing) publishes a descriptor, and any thread about to advance the era first helps it by reading the pointer on its behalf and stamping the descriptor; per-read cost: HE's cost on the fast path; garbage: BOUNDED; progress: WAIT-FREE, and wait-free even across multiple data structures; precondition: HE's preconditions plus hardware FAA and WCAS (wide CAS, for version-tagged slots and descriptors to prevent repeated helping).
  
*Forced by:* Every protect-and-validate scheme has exactly one unbounded loop — the retry in step 3 — and WFE is the first scheme to remove it, using the standard fast-path/slow-path + helping recipe. It makes concrete why 'HP is wait-free' (as Michael's paper claimed) was an overstatement.

**`Crystalline-L / Crystalline-LW / Crystalline-W`**
— Mechanism: Hyaline's 2D grid with HE-style fine-grained reservations — NUM_HE rows per thread instead of one, so a stalled thread pins only the eras it actually reserved, and batches attach only to rows of threads active in the current era; wait-freedom comes from replacing the batch-attaching CAS loop with an unconditional SWAP (tainting next pointers to resolve the resulting races) and from a fast-path/slow-path protect; per-read cost: HE-like; garbage: BOUNDED even with starving threads, and for Crystalline-W bounded across multiple data structures; progress: Crystalline-L lock-free, Crystalline-LW wait-free with restarting, Crystalline-W fully WAIT-FREE; precondition: 3-word header, unlink-before-retire, hardware FAA + SWAP (LW/W) and WCAS (W).
  
*Forced by:* It is the current state of the art and the only scheme that is simultaneously fast, memory-bounded, balanced, and wait-free — and its three-step construction (L → LW → W) is a readable map of exactly which property costs which hardware primitive.

**`Optimistic Access / Automatic Optimistic Access / Free Access (Cohen & Petrank)`**
— Mechanism: allocate and retire from a pre-allocated pool for one 'round'; when the pool is exhausted, recycle all retired nodes immediately and set every thread's *warning bit* — each thread checks its warning bit before trusting a read and restarts from a checkpoint if set; writes are still protected by hazard pointers; AOA automates the normalized-form transformation, FA replaces it with a compiler pass over read-only/write-only phases; per-read cost: one local warning-bit check, no fence; garbage: BOUNDED by the round pool; progress: lock-free (with helping during recycling); precondition: a type-stable pool, data structures in normalized form (OA/AOA) or a compiler pass (FA), checkpoints that are reachable from an entry point, and nodes retired in one round may only be reclaimed in a later round.
  
*Forced by:* It is the origin of the optimistic-access paradigm that VBR perfects, and its warning-bit-versus-global-epoch difference is instructive: a warning bit tells you 'something was reclaimed', a version tells you 'this specific thing was reclaimed', and the extra precision is what lets VBR drop hazard pointers for writes too.

**`Tagged pointers / ABA stamps + free-list reuse`**
— Mechanism: never return nodes to the allocator — recycle them through a free list and attach a monotonically increasing tag to every pointer so a CAS fails if the node was reused (Treiber stack with a counted pointer, Michael-Scott queue with a node pool); per-read cost: zero, but every CAS becomes a wide/tagged CAS; garbage: BOUNDED (the pool is the bound) but memory is never returned to the OS; progress: lock-free; precondition: a pool per node type, spare pointer bits or WCAS, and the data structure must tolerate reading a recycled node's stale contents.
  
*Forced by:* It is the historical pre-SMR answer and still the right answer in bounded, fixed-type systems (and in kernels/HFT where you preallocate anyway); recognizing that a version tag is exactly VBR's mechanism in miniature collapses two literatures into one idea.

### OPTIONAL (11)

**`folly CoreCachedSharedPtr / ReadMostlySharedPtr`**
— Mechanism: shard the reference count — keep one shared_ptr per core (64 slots by default) via an aliasing trick, or per-thread counters, so acquiring a reference touches a private line; per-read cost: one uncontended local increment; garbage: BOUNDED; progress: lock-free; precondition: only the *acquire* is sharded — copies of the resulting shared_ptr still hit the shared count, so the pattern only works when references are acquired-and-dropped locally.
  
*Forced by:* On many-core machines a hot reference count serializes and the cache line ping-pongs between cores; sharding trades O(cores) space and an O(cores) read-of-all-shards on the writer side for a contention-free reader.

**`SCOT / immutability-based HP validation (PLDI 2025)`**
— Mechanism: instead of validating the protected node, validate that the *predecessor's* link has not changed while you walk the 'dangerous zone' of logically-deleted nodes, exploiting fields that are immutable after publication; per-read cost: HP cost plus a predecessor re-check per dangerous step; garbage: BOUNDED; progress: lock-free; precondition: the data structure must support deferred deletion of logically-deleted nodes and expose an immutable field to validate against.
  
*Forced by:* HP++ fixes applicability by changing the SMR; SCOT fixes it by changing the validation rule, so existing HP implementations keep working — it is the cleanest demonstration that 'what counts as a valid protection' is a design knob, not a law.

**`Publish-on-ping / EpochPOP`**
— Mechanism: readers track their reservations in *private* memory with no fence at all; a reclaimer that needs to know them 'pings' (signals) the other threads, which publish on demand; per-read cost: a private store, zero fences — it is a drop-in replacement for HP and HE that deletes their per-read fence; garbage: BOUNDED; progress: conditionally lock-free (signal-based); precondition: POSIX signals plus an interface where reservations are buffered privately; EpochPOP combines epochs with HP robustness to approach EBR speed.
  
*Forced by:* HP and HE eagerly publish reservations billions of times so a reclaimer can read them a handful of times — a pure asymmetric-synchronization mismatch. Publish-on-ping makes publication *reactive*, which is the signal-based analogue of the asymmetric fence.

**`Type-preserving allocator / page-remapping lock-free allocator (Moreno & Rocha 2023)`**
— Mechanism: the optimistic family needs memory to stay mapped and type-stable so a stale read returns garbage-but-legal data rather than a SIGSEGV; the 2023 lock-free allocator restores the ability to return memory to the OS by remapping freed pages onto a single shared physical frame, so stale reads still land on valid memory; per-read cost: zero; garbage: n/a (it is the allocator, not the reclaimer); progress: lock-free; precondition: virtual-memory remapping support.
  
*Forced by:* The standard objection to optimistic access is 'you either handle segfaults — losing a debugging tool — or you never free pages, which is fatal for long-running services'. This removes the objection, and it shows that the allocator is part of the SMR design surface, not a black box beneath it.

**`Stamp-it`**
— Mechanism: an EBR-family scheme that bounds the *reclamation overhead* (the amortized work per operation and per reclaimed node) rather than the garbage, using a stamped, doubly-linked reservation structure; per-read cost: EBR-like; garbage: UNBOUNDED; progress: blocking; precondition: unlink-before-retire.
  
*Forced by:* It isolates a dimension the rest of the literature conflates: a scheme can be memory-unbounded but *time*-bounded, and a scan of all p threads per reclamation is a scalability bug independent of how much garbage you hold.

**`QSense`**
— Mechanism: hybrid — an EBR-like fast path for the common case, falling back to a hazard-pointer-inspired slow path when a thread is delayed, with reservation freshness guaranteed by relying on the OS scheduler (periodic auxiliary processes per core forcing context switches to flush hazard pointers); per-read cost: EBR-like when fast, HP-like when degraded, plus the periodic publication cost even when nobody is reclaiming; garbage: BOUNDED; progress: hard to call non-blocking (depends on scheduler behaviour); precondition: inherits HP's applicability restrictions, plus assumptions about the scheduler.
  
*Forced by:* It is the canonical 'fast path EBR, slow path HP' hybrid and the clearest example of buying robustness with an OS assumption — and of why such assumptions are 'not always well specified'.

**`Drop the Anchor (DTA)`**
— Mechanism: primarily EBR, with periodically published hazard pointers used only as a recovery mechanism — when a thread is detected stalled, another thread duplicates the range of objects reachable from the stalled thread back into the data structure so everyone else can resume reclaiming; per-read cost: EBR-like plus a periodic HP publish; garbage: BOUNDED; progress: lock-free; precondition: the structure must tolerate having a sub-range duplicated/rebuilt, which is list-specific.
  
*Forced by:* It is a third answer to the stalled-thread problem: don't wait (EBR), don't interrupt (DEBRA+/NBR) — *copy around* the obstruction. Worth knowing because it reframes the stalled thread as a structural problem rather than a scheduling one.

**`ThreadScan / ForkScan / StackTrack (conservative stack scanning via OS or HTM)`**
— Mechanism: automatic reclamation with no annotations — a reclaimer signals every thread (ThreadScan/ForkScan, using POSIX signals and copy-on-write pages) or splits operations into short hardware transactions that publish stack and register contents (StackTrack), then conservatively skips freeing anything found in another thread's stack or registers; per-read cost: zero annotations, but transaction start/commit or signal costs; garbage: BOUNDED; progress: not strictly non-blocking (signals take locks; HTM aborts); precondition: the program must not hide pointers — any pointer tagging, bit stealing or special pointer arithmetic breaks it, which rules out most lock-free code.
  
*Forced by:* It is the 'just scan the roots like a GC' branch, and its failure mode is the most instructive one in the whole landscape: lock-free data structures hide pointers *by design* (mark bits, counted pointers), so conservative root scanning and lock-free programming are structurally at odds.

**`OrcGC / FreeAccess (lock-free tracing collectors for lock-free structures)`**
— Mechanism: a real garbage collector with lock-free progress — FreeAccess uses a mark-and-sweep collector driven by a compiler pass over read/write phases; OrcGC uses deferred reference counting ('orcs') so retirement is automatic; per-read cost: instrumented loads (OrcGC can be slower than HP in some tests); garbage: BOUNDED; progress: lock-free; precondition: FreeAccess removes the normalized-form requirement but does not transparently handle SWAP; both need instrumentation of every pointer access.
  
*Forced by:* It closes the loop with managed languages: the reason Java programmers never think about SMR is that a collector computes reachability for them, and these show what it costs to get that in C++ — plus they explain why 'use a GC' is not free for lock-free code (SWAP, pointer hiding, and instrumented loads).

**`PEBR (pointer-and-epoch-based reclamation)`**
— Mechanism: EBR that *ejects* a stalled thread from the epoch-advance mechanism and falls back to hazard pointers to protect that thread's references; per-read cost: a reservation on every read as in HP, made cheaper with page-protection tricks; garbage: no explicit bound on how many blocks a thread may reserve; progress: requires restarting to retain EBR-like semantics, so not lock-free in general; precondition: restartable operations; measured at 85–90% of EBR and, in its Rust implementation, slower than crossbeam-epoch.
  
*Forced by:* It is the most direct 'marriage of pointer- and epoch-based reclamation', and its disappointing numbers are the data point that motivated the Hyaline/Crystalline line — worth knowing so you don't reinvent it.

**`Expediting hazard pointers with bounded RCU critical sections (Kim, Jung, Kang 2024)`**
— Mechanism: alternate HP-protected regions with RCU-protected regions — protect the first few nodes with HP, then open an RCU region covering the next n nodes, so per-node reservation cost falls by a factor of n, with the preceding HP region acting as a checkpoint a neutralizing signal can restart from; per-read cost: one HP protect per n nodes instead of per node; garbage: BOUNDED (reclaimers may selectively signal threads lingering in RCU regions); progress: conditionally lock-free; precondition: the structure must be able to verify that the HP-protected checkpoint nodes are not logically deleted, and must support restarting from a checkpoint rather than from the root.
  
*Forced by:* It removes NBR's hardest requirement (restart from the root) by making the last HP-protected node act as a synthetic entry point, extending neutralization to structures with auxiliary updates — the current frontier of the 'how little can a reader pay' question.

### SKIP (1)

**`Conditional Access (hardware/software co-design)`**
— Mechanism: new hardware instructions that piggyback on cache coherence to detect a potential use-after-free without any explicit shared-memory communication or extra coherence traffic, enabling *immediate* reclamation with no batching at all; per-read cost: essentially zero; garbage: BOUNDED at zero — nodes are freed immediately like in a sequential program; progress: n/a; precondition: hardware support (evaluated only in the Graphite multi-core simulator) and optimistic data structures.
  
*Forced by:* It is the lower bound of the design space: it shows that every software scheme's cost is really the cost of *communicating* reservations, and that if coherence told you the answer for free, all of SMR would collapse into 'free it now'.

### Notes

APPLIED QUESTION — a retired node that REMAINS reachable through its predecessor's next pointer forever (a forward-walkable segment chain that is never unlinked).

The reason this is the right question to ask is that essentially every SMR scheme implements only HALF of the safety argument. The contract has two halves:
(A) after retire(p), no thread that has not already got p may obtain p — i.e. p is unreachable from the roots; and
(B) threads that already hold p eventually let go.
Grace-period and reservation schemes implement (B) only. They *assume* (A) and the data structure must supply it. "Unlink before retire" is not a style guideline, it is the missing half of the proof.

COMPATIBLE (and why)
1. No reclamation / leaking — trivially; no precondition at all. This is the honest starting point for a never-unlinked chain.
2. Reference counting where the LINK itself owns a count (classical RC, folly::atomic_shared_ptr, Rust Arc in the next field, arc_swap, folly hazptr_obj_base_linked link counting). Safe because reachability *is* a reference: while the predecessor's next pointer exists the count is >= 1 and the node cannot be freed. Note the consequence honestly: if the chain is genuinely never unlinked, nothing is ever freed, so RC is a *safe* leak, not a reclaimer. What RC actually buys you is a change of question — "retire" is replaced by "drop the link", so you must decide when the link goes away. folly's hazptr_obj_base_linked exists precisely for the case where "the removal of objects is uncertain" and is the closest production analogue of this shape.
3. The optimistic-access / version-validation family — VBR above all, and OA/AOA/FA, SOMAR/NOVA, and in spirit Conditional Access. This is the ONE family that genuinely drops precondition (A). VBR explicitly "allows access to reclaimed nodes for reads as well as writes": each node carries birth and retire epochs, each mutable field a version; every read compares the global epoch against the thread's last-read value and rolls back on a mismatch; every write is a WCAS against the version. Reaching a reclaimed-and-reused node through a stale next pointer is an *expected* event that validation catches. The price list is explicit: a type-preserving allocator (or Moreno & Rocha's page-remapping allocator) so stale reads land on mapped memory of the same type; a version word per mutable field; WCAS for writes; and every operation must be rollback-able to a checkpoint. On x86/TSO this costs zero fences and zero shared stores on the read path.
4. Explicit cooperative hand-off with per-slot state bits (crossbeam-queue's WRITE/READ/DESTROY). This works only because the participant set per slot is statically known and finite, and it still relies on a weaker form of (A): see the caveat below.
5. Partial credit — HP++ and SCOT. Both are built exactly for "a traversal can reach a retired node through a logically deleted predecessor", and both are bounded and lock-free. But they handle TRANSIENT reachability-after-retire: HP++ has the unlinking thread invalidate the nodes a concurrent traversal could still reach, SCOT validates that the predecessor's link has not changed while crossing the dangerous zone. Both still require that someone eventually unlinks. PERMANENT reachability defeats them: HP++'s invalidate set is unbounded and SCOT's "predecessor link unchanged" check becomes vacuously true forever.

INCOMPATIBLE (and the exact failure)
- Hazard pointers / pass-the-buck / folly hazptr: the read protocol is (1) announce, (2) fence, (3) validate that the pointer is still reachable. Step 3 is a proof only because "the link still holds p" implies "p was not yet retired". If a retired node stays linked, the implication is false, validation succeeds on a dangling pointer, and you have a textbook use-after-free with a *passing* validation. This is the same failure that makes HP applicable to only 3 of 18 data structures in Singh's survey.
- EBR / QSBR / RCU / DEBRA / crossbeam-epoch: a grace period proves that every thread active at retire time has since quiesced. It says nothing about a thread whose operation STARTS after the grace period and then walks the still-present link to p. UAF.
- Hazard eras / IBR / WFE: the node's retire era is stamped at retire; a reader reserving a later era holds a reservation that does not intersect [birth, retire], so the node is freed while that reader can still reach it. UAF. (Same root cause as HP — they share the protect-and-validate protocol.)
- Hyaline / Crystalline: the batch's reference count is "number of threads active at retire time". A thread that becomes active later is not counted, yet it can still reach the node. UAF.
- DEBRA+ / NBR / publish-on-ping: these bound garbage by interrupting stragglers, which addresses (B) only; (A) is still assumed. Additionally NBR requires every read phase to restart from the root, which a forever-reachable chain satisfies in the worst possible way — restarting from the root still reaches the retired node.
- crossbeam-queue's DESTROY bit, as a caveat: it looks like an exception but is not. In SegQueue the consumer stores head.block = next BEFORE calling Block::destroy, so the root moves off the block and monotone indices prove no new consumer can re-enter it; the old block's own next pointer still points forward, but nothing live points *to* it. It is unlink-before-retire plus a distributed refcount over an exactly-known participant set. If the spine were forward-walkable from a live root forever, "the last accessor" would be unknowable and the handshake would have nothing to terminate on.

THE PRACTICAL CONCLUSION for the SegQueue work: if the segment chain is never unlinked there is no grace period to wait for, and the fix is structural, not a cleverer reclaimer. Three real options, in increasing exoticism: (i) make the spine's reachability monotone — advance a head/root pointer so old segments become unreachable, and only then does every scheme above apply (this is what crossbeam does); (ii) split the problem — leak or bound the spine (segments are O(capacity/BLOCK_CAP), not O(items)) and reclaim only the payloads, which is the classic "reclaim the expensive thing, leak the cheap thing" trade; (iii) go VBR-shaped — version-stamp the segments, allocate them from a type-preserving pool, validate every read, and accept rollback. Option (i) is almost always right; option (iii) is the only one that keeps the never-unlinked invariant.

WHAT IS ACTUALLY IN FOLLY (the user asked for the full inventory, not the SegQueue-sized slice)
- folly/synchronization/Hazptr.h + Hazptr-fwd.h + HazptrDomain.h + HazptrHolder.h + HazptrObj.h + HazptrObjLinked.h + HazptrRec.h + HazptrThrLocal.h + HazptrThreadPoolExecutor.h. That is: hazptr_domain (multiple independent domains, so one domain's reservations never block another's reclamation); hazptr_holder (protect/reset); hazptr_array<M> (cheaper construction for M>1); hazptr_local<M> (~2 ns vs ~5 ns, but you may hold no other holder concurrently); hazptr_obj_base<T, Deleter> (retire); hazptr_obj_cohort (bounded per-cohort garbage, tagged lists, synchronous reclamation on destruction); hazptr_obj_base_linked + hazptr_root (LINK COUNTING — automatic retirement when removal is uncertain, no extra reader overhead); thread-cached hazptr records; and an executor for asynchronous reclamation.
- folly/synchronization/Rcu.h — rcu_domain, rcu_reader, rcu_retire, rcu_synchronize, rcu_barrier. Two epochs (because of late readers of the version counter), ThreadCachedReaders counters with waitForZero, ~5 ns reader critical section, rcu_retire ~150 ns. Documented caveats: fork safety, deleters must not throw, a blocking reader delays every callback in the domain, deadlock if a deleter takes a lock held across retire.
- folly/synchronization/AsymmetricThreadFence.h — the membarrier() trick that makes HP and RCU readers nearly free (light fence = compiler barrier only; heavy fence = syscall on the reclaimer). Used from HazptrDomain.h, detail/ThreadCachedReaders.h and ThreadPoolExecutor.cpp.
- folly/concurrency/AtomicSharedPtr.h — packed 48-bit control block + 16-bit local count, batched flushes, lock-free below 8192 contending threads, ALIASED_PTR tag bit.
- folly/concurrency/CoreCachedSharedPtr.h (and ReadMostlySharedPtr) — sharded/per-core reference counts for read-mostly pointers.
- folly/concurrency/UnboundedQueue.h and DynamicBoundedQueue.h — segmented queues whose segment reclamation uses hazard pointers (the direct analogue of the SegQueue question).
Folly has no epoch-based *reclaimer* separate from Rcu.h, and nothing in the HE/IBR/Hyaline/VBR families. Those live in research code (ssrg.ece.vt.edu's Crystalline/WFE/Hyaline, Wen et al.'s IBR benchmark, Setbench, kaist-cp/smr-benchmark) and in Rust as crossbeam-epoch, haphazard, seize and arc-swap.

SUGGESTED BUILD ORDER for "a hazard crate, then an epoch crate, then integrate" — each stage adds exactly one new idea, and each has a tool that catches the bug if you get it wrong:
1. hazard crate stage 1: domain + thread records + protect/retire/scan. The bug to be caught: omit the store-load fence in protect and watch loom or a stress test find the announce-after-free reorder.
2. hazard crate stage 2: the validation step, and discover for yourself that it is a proof only under unlink-before-retire. This is where the SegQueue problem becomes visible as a theorem rather than a surprise.
3. hazard crate stage 3: bound the garbage (a HiWatermark / cohort) and measure k*p; then make the fence asymmetric (membarrier on Linux) and measure the delta — that is the single most instructive benchmark in the whole area.
4. epoch crate stage 1: three limbo bags + global epoch + announce at entry/exit; demonstrate unbounded garbage by sleeping one thread inside a pin. Stage 2: QSBR variant (move the announce to a quiescent point) and measure how much the per-operation announce actually cost.
5. epoch crate stage 3 (the step that makes it worth building separately from crossbeam-epoch): add birth/retire eras and turn it into hazard eras, then widen the reservation into an interval and you have IBR. One codebase, three schemes, one knob — reservation precision. This is where "epoch reclamation" stops being a single technique and becomes a family.
6. Only then integrate with SegQueue, and let the integration *fail* first: the right failure is discovering that a never-unlinked chain has no grace period, and the right fix is restructuring the spine, not patching the reclaimer.

UNCERTAINTY AND CONFLICTS TO BE AWARE OF
- Hazard eras' boundedness is genuinely disputed in the sources. Singh's thesis says in prose "Thus, HE has bounded garbage" but its own Table 5.1 marks HE as NOT bounding garbage, and its stalled-thread experiment shows HE's memory rising "marginally". Nikolaev/Ravindran's Table 1 marks HE lock-free for one data structure and blocking across multiple. The resolution is that HE's garbage is bounded by the live set at the reserved eras (proportional to structure size), not by k*p as in HP — so it is "bounded" in a weaker sense than HP and unbounded in the allocation-count sense that EBR fails.
- Progress labels depend on an assumption that is often left implicit: whether the data structure may restart. Nikolaev/Ravindran's table is the one to trust because it separates [with restart] from [without restart] and 1 data structure from 2+ data structures — HE, IBR and Hyaline-1S are all blocking across multiple data structures because their global era clock may never converge, while HP stays non-blocking because it converges on a pointer local to each structure.
- Every ns/cycle figure here is from a specific paper on specific hardware (folly's ~2 ns protect+reset and ~5 ns rcu_reader on a 2.5 GHz Haswell; rcu_retire ~150 ns) and should be re-measured, not quoted as a constant.
- Also unverified from primary source: folly's Rcu.h doc comment does not itself mention membarrier, but AsymmetricThreadFence is documented as being used from detail/ThreadCachedReaders.h, which is RCU's reader-counter machinery; treat "folly RCU uses asymmetric fences" as very likely but read ThreadCachedReaders.h to confirm.
- One honest gap in the literature worth knowing: Singh reports that for the lock-free interpolation tree, "we are unaware of any SMR technique with bounded garbage that is compatible". Permanent reachability-after-retire is close to that frontier, which is why the structural fix is the mainstream answer.

Sources: [A Family of Fast and Memory Efficient Lock- and Wait-Free Reclamation (PLDI 2024, Nikolaev & Ravindran)](https://www.ssrg.ece.vt.edu/papers/pldi24.pdf) · [Safe Memory Reclamation Techniques (Ajay Singh, PhD thesis, 2024/arXiv 2025)](https://arxiv.org/pdf/2509.02457) · [The ERA Theorem for Safe Memory Reclamation](https://arxiv.org/pdf/2211.04351) · [VBR: Version Based Reclamation](https://arxiv.org/html/2107.13843) · [Crystalline: Fast and Memory Efficient Wait-Free Reclamation](https://arxiv.org/pdf/2108.02763) · [Hyaline: Snapshot-Free, Transparent, and Robust Memory Reclamation](https://arxiv.org/pdf/1905.07903) · [Interval-Based Reclamation (Wen et al.)](https://www.researchgate.net/publication/322972080_Interval-based_memory_reclamation) · [DEBRA: Reclaiming Memory for Lock-Free Data Structures](https://mc.uwaterloo.ca/pubs/debra/paper.podc15.pdf) · [Fixing Non-blocking Data Structures for Better Compatibility with Memory Reclamation Schemes (SCOT)](https://arxiv.org/html/2504.06254) · [Applying Hazard Pointers to More Concurrent Data Structures (HP++)](https://dl.acm.org/doi/abs/10.1145/3558481.3591102) · [Leveraging Immutability to Validate Hazard Pointers for Optimistic Traversals (PLDI 2025)](https://iris-project.org/pdfs/2025-pldi-hp-revisited.pdf) · [Expediting Hazard Pointers with Bounded RCU Critical Sections](https://dl.acm.org/doi/10.1145/3626183.3659941) · [Turning Manual Concurrent Memory Reclamation into Automatic Reference Counting (DRC)](https://arxiv.org/pdf/2204.05985) · [Stamp-it comparison of six reclamation schemes](https://arxiv.org/pdf/1712.06134) · [Publish on Ping](https://arxiv.org/html/2501.04250) · [P1121R3 Hazard Pointers (WG21)](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2021/p1121r3.pdf) · [P1202 Asymmetric fences](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2018/p1202r0.pdf) · [folly Hazptr.h](https://raw.githubusercontent.com/facebook/folly/main/folly/synchronization/Hazptr.h) · [folly Rcu.h](https://raw.githubusercontent.com/facebook/folly/main/folly/synchronization/Rcu.h) · [folly AtomicSharedPtr.h](https://raw.githubusercontent.com/facebook/folly/main/folly/concurrency/AtomicSharedPtr.h) · [folly CoreCachedSharedPtr.h](https://github.com/facebook/folly/blob/main/folly/concurrency/CoreCachedSharedPtr.h) · [Tricks in ArcSwap](https://vorner.github.io/2019/04/06/tricks-in-arc-swap.html) · [arc-swap debt module](https://github.com/vorner/arc-swap/blob/master/src/debt/mod.rs) · [kaist-cp/smr-benchmark](https://github.com/kaist-cp/smr-benchmark/blob/main/README.md). Local extracts kept at /private/tmp/claude-501/-Users-thanhngo-tngo-projects-reth-journey/a8002ece-ec52-48d3-9876-8b160b9ef895/scratchpad/pldi24.txt and /private/tmp/claude-501/-Users-thanhngo-tngo-projects-reth-journey/a8002ece-ec52-48d3-9876-8b160b9ef895/scratchpad/smr_survey.txt; the crossbeam DESTROY-bit code read is /Users/thanhngo/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/crossbeam-queue-0.3.14/src/seg_queue.rs (lines 113-131 for Block::destroy, 415-450 for the pop path that advances head.block before destroying).

---

## folly `hazptr` — the complete feature surface

73 mechanisms, with the performance or correctness pressure that forced each one. This is the answer to "what is actually in folly".

### CORE (20)

**`hazptr_rec (Atom<const void*> hazptr_, alignas(hardware_destructive_interference_size))`**
— The actual hazard pointer: one single-writer/multi-reader slot holding the address a thread is currently protecting, plus domain_ and nextAvail_ links.
  
*Forced by:* Every reader writes its own slot on every protect while reclaimers read all slots — without cacheline alignment two readers on adjacent slots ping-pong one line and the reclaimer's scan invalidates reader stores. The slot is deliberately only a void* (type-erased) so one scan matches retired objects of all types.

**`hazptr_rec::reset_hazptr(const void* p = nullptr)`**
— Single entry point for both beginning a reader critical section (store non-null) and ending one (store null or retarget).
  
*Forced by:* Correctness asymmetry spelled out in the comment: 'When beginning a reader critical section, the pointer is not-null and no memory order is needed... When ending... a memory order is needed. The store-release synchronizes-with the load-acquire on this same rec in subsequent reclamation path rec walks.' The release is what makes the reader's accesses to the now-unprotected object happen-before the reclaimer's free. Getting this backwards (release on entry, relaxed on exit) is a silent UAF.

**`hazptr_holder<Atom> (RAII, move-only, nullable hprec_)`**
— Owns at most one hazptr_rec; destructor resets the slot and returns the rec to the thread cache or domain.
  
*Forced by:* Raw hazard pointers are never exposed ('In the hazptr library, raw hazard pointers are not exposed to users'). A forgotten reset leaks protection forever and stalls all reclamation globally; a forgotten release leaks a rec, permanently enlarging every future O(hcount_) scan. RAII makes both impossible.

**`hazptr_holder::try_protect(T*& ptr, const Atom<T*>& src) — load / publish / full fence / re-load / compare`**
— Publish a candidate pointer into the hazard slot, then re-read the source and confirm it did not change; returns false to retry.
  
*Forced by:* The central TOCTOU of hazard pointers: between reading src and publishing the hazard pointer, a writer can unlink and the reclaimer can free. The re-read is the validation. The `asymmetric_thread_fence_light(seq_cst)` between `reset_protection(f(p))` and `ptr = src()` is a StoreLoad barrier — without it the CPU (and the compiler) may reorder the hazard store after the reload, so the reclaimer's scan can miss the store while the reader sees a stale-but-equal src and concludes it is safe.

**`hazptr_holder::protect(src) retry loop`**
— Loop try_protect until it succeeds; the ordinary user-facing API.
  
*Forced by:* Convenience wrapper, but note it starts with `src.load(relaxed)` because the first candidate is validated anyway — the acquire is only paid inside try_protect's confirming load.

**`reset_protection(ptr) / reset_protection(nullptr)`**
— Retarget or clear the owned slot without destroying the holder.
  
*Forced by:* Hand-over-hand list traversal needs to move a protection from node N to N->next thousands of times per operation; constructing/destroying a holder each step would cost thread-cache traffic per hop. Also the documented way to shrink a critical section before blocking.

**`make_hazard_pointer(domain) free function (and hazard_pointer alias)`**
— Factory for a non-empty holder; checks the thread cache first, falls back to domain.acquire_hprecs(1).
  
*Forced by:* Non-empty construction is a factory rather than a constructor so the fast path can branch on `domain.is_default_domain()` and return a cached rec without touching the domain at all. Naming mirrors WG21 P1121.

**`hazptr_domain<Atom>`**
— Owns the set of hazptr_rec-s and the set of retired objects; the unit of reclamation.
  
*Forced by:* Isolation: independent subsystems (and tests) must not have their reclamation latency coupled, and a test needs a domain it can destroy and assert empty. Also gives the retired-list count a well-defined scope so 'bounded unreclaimed garbage' is a provable property.

**`load_hazptr_vals() → folly::F14FastSet<const void*>`**
— Snapshot all currently protected addresses into a hash set.
  
*Forced by:* Naive matching is O(retired x hazards). With ~1000 retired objects and hundreds of recs that is ~10^5 comparisons per reclamation round. A hash set makes the match O(retired + hazards). This set, plus the sharded lists, is why folly can afford a 1000-object threshold at all.

**`hazptr_obj<Atom> base: ReclaimFnPtr reclaim_, Obj* next_, uintptr_t cohort_tag_`**
— Private base giving every protectable object an intrusive list link, a type-erased reclaim function, and a cohort/tag word.
  
*Forced by:* The domain must hold a heterogeneous list of retired objects of unrelated types, so the destructor has to be erased into a plain function pointer (not a vtable — no virtual dispatch, no vptr required). next_ being intrusive means retire() allocates nothing, which matters because retire sits on the critical path of a lock-free writer. Exactly three words of overhead per protected object.

**`next_(this) sentinel + pre_retire_check() / pre_retire_check_fail()`**
— Every constructor sets next_ = this so that a second retire of the same object is detected immediately.
  
*Forced by:* 'All constructors set next_ to this in order to catch misuse bugs such as double retire.' Double retire is the classic lock-free bug (two threads both believe they unlinked the node); without this check it surfaces much later as a double free or a cycle in the retired list, with no connection to the real culprit. Free diagnostic, zero extra state.

**`hazptr_obj_base<T, Atom, D>::retire(D deleter, hazptr_domain& domain) + set_reclaim()`**
— The user-facing intrusive retire: install the deleter, install the type-erased reclaim lambda, push to cohort-or-domain.
  
*Forced by:* Allocation-free retire path. set_reclaim() installs a capture-less lambda (convertible to a plain function pointer) that downcasts back to T — that is how one void*-typed list reclaims many types with no virtual table.

**`hazptr_obj_list<Atom> (head, tail, count)`**
— A batch of retired objects carried as head/tail/count so it can be spliced into a shared list in one operation.
  
*Forced by:* Pushing N objects individually is N CASes on a contended head plus N count fetch_adds. Carrying the tail and count lets push_list() do one CAS and one add_count(l.count()). The count is what feeds the threshold logic, so it must travel with the batch.

**`hazptr_detail::linked_list<Node>`**
— Non-atomic head/tail singly linked list over nodes that already have next()/set_next().
  
*Forced by:* Most list manipulation during reclamation is thread-private (the popped batch is owned by one reclaimer), so paying atomics for it would be waste. Separating 'private list' from 'shared list' types makes the atomic surface explicit and small.

**`hazptr_detail::shared_head_only_list<Node, Atom> with kLockBit in head_, owner_ thread id, reentrance_`**
— The retired-list type: lock-free push, wait-free pop_all via exchange, plus an optional pop_all(kAlsoLock) / push_unlock locked mode.
  
*Forced by:* push must be lock-free because it sits inside lock-free writers. pop_all is just head_.exchange(0) — wait-free — which is why asynchronous reclamation can grab the whole list in one instruction. The lock bit exists only for tagged lists: a synchronous cleanup_cohort_tag must not miss objects that a concurrent async reclamation popped out, so the list is held locked between pop_all and push-back of non-matching objects. owner_ + reentrance_ exist to prevent self-deadlock when the same thread re-locks ('Locking is reentrant to prevent self deadlock'). Documented valid combinations: push(kMayBeLocked)/pop_all(kAlsoLock)/push_unlock, or push(kMayNotBeLocked)/pop_all(kDontLock) — mixing them is unsound.

**`count_ as Atom<int> (signed, 'may transiently be negative') with add_count / exchange_count / cas_count`**
— Approximate global count of retired-but-unreclaimed objects, driving the threshold trigger.
  
*Forced by:* The source comment: 'Using signed int for rcount_ because it may transiently be negative. Using signed int for all integer variables that may be involved in calculations related to the value of rcount_.' do_reclamation zeroes the count, then subtracts what it actually reclaimed, while other threads concurrently add — so the counter legitimately goes negative and an unsigned type would wrap into a bogus huge value and trigger runaway reclamation.

**`threshold() = std::max(kThreshold /*1000*/, kMultiplier /*2*/ * hcount_)`**
— How many retired objects must accumulate before an asynchronous reclamation round is triggered.
  
*Forced by:* Two pressures at once. Amortization: a round costs O(retired + hazards), so retired must scale with hazards or the scan dominates — hence 2 * hcount_. Bounded garbage: because the threshold is a function of hcount_ and nothing else, unreclaimed objects are provably O(hazard pointers), which is the headline property of hazard pointers versus RCU ('The number of unreclaimed objects is bounded when protected by hazard pointers, but is unbounded when protected by RCU').

**`check_threshold_and_reclaim() / check_count_threshold()`**
— Try the count trigger, then the time trigger; whoever CASes the count to 0 owns the round.
  
*Forced by:* Exactly one thread must run a round per trigger, so ownership is claimed by a CAS that steals the count. Count is checked first because it is a relaxed-ish atomic load; the time check calls steady_clock::now() and is therefore the slow path.

**`do_reclamation(int rcount) outer while(true) loop with the `done` flag`**
— One reclamation round: extract lists, heavy fence, snapshot hazards, match tagged, match+reclaim untagged, then re-check the threshold and repeat if not done.
  
*Forced by:* One pass is not enough for two reasons. Reclaiming a linked object can produce children (match_reclaim_untagged sets `done = false` when `!children.empty()`), and objects can arrive during the pass (`if (!untagged_empty()) done = false;`). Looping until done is what makes the bounded-garbage claim hold rather than being eventually-maybe.

**`The heavy fence placement in do_reclamation (asymmetric_thread_fence_heavy(seq_cst) after extract, before load_hazptr_vals)`**
— Establish that the extraction of retired objects precedes the snapshot of hazard pointers, globally.
  
*Forced by:* This is the pairing partner of try_protect's light fence and of push_list's light fence. If the hazard snapshot could be reordered before the extraction, a reader who published a hazard pointer and then saw a still-valid src could have its object reclaimed. Order is: extract retired, FENCE, read hazards — never the reverse.

### IMPORTANT (26)

**`try_protect(ptr, src, Func f) — pointer-word filter`**
— Protect the address obtained by passing the raw source word through f, so callers can strip tag bits before publishing.
  
*Forced by:* 'Filtering the protected pointer through function Func is useful for stealing bits of the pointer word.' ConcurrentHashMap packs flags into node pointers; without the filter the hazard slot would hold a tagged word that never matches the untagged `raw_ptr()` of the retired object, and the object would be freed under the reader.

**`hprecs_ as atomic_grow_array<Rec, HazptrRecArrayPolicy> + hcount_`**
— The rec registry: a grow-only, reference-stable, indexable array of recs, with hcount_ as the published count.
  
*Forced by:* The reclaimer scan is the hot cost of the whole scheme. A linked list of recs (the classic Michael design) makes the scan a serial dependent-load chain; an indexable array makes it a prefetchable sequential walk and enables the chunked load in load_hazptr_vals. It must be grow-only with reference stability because live holders hold raw Rec* into it — a Vec realloc would dangle them. Growth is racy: losing threads destroy the elements they built.

**`avail_ : Atom<uintptr_t> free list of recs with kLockBit = 1 (tagged-pointer lock) + detail::Sleeper`**
— LIFO pool of recs not currently owned by any holder; threads pop on acquire and push on release.
  
*Forced by:* Recs are never freed, only recycled, because freeing one while a reclaimer is mid-scan would be a UAF and because allocation is slow. A plain CAS-pop would be ABA-exposed and could not pop N recs atomically; stealing the low bit as a lock makes acquire_hprecs(num) one critical section instead of num CASes. Sleeper (bounded spin then sleep) stops the lock from burning a core.

**`acquire_hprecs(num) / release_hprec / release_hprecs(head, tail) + Rec::next_avail()/set_next_avail()`**
— Bulk acquire and release of recs, threaded on the rec's own nextAvail_ field.
  
*Forced by:* hazptr_tc::fill(num) and make_hazard_pointer_array<M> refill in bulk; without a batch API the thread cache would pay one shared-lock round trip per rec, which is exactly what the cache exists to avoid. Intrusive nextAvail_ means the free list allocates nothing.

**`create_new_hprec() with hcount_.fetch_add(1, relaxed) and its happens-before argument`**
— Publish a brand-new rec so reclaimer scans will see it.
  
*Forced by:* The relaxed increment looks like a bug and the comment defends it: the increment 'happens-before the load-relaxed in any call to load_hazptr_vals where this newly-created hprec would need to be seen... which precede the fence-light-seq-cst in try_protect... which synchronize-with the fence-heavy-seq-cst in do_reclamation'. This is the subtlest ordering argument in the library — a new rec that a scan misses means its protected object gets freed under a live reader.

**`Relaxed slot loads + one trailing acquire fence in load_hazptr_vals, with kIsSanitizeThread ? acquire : relaxed`**
— Replace N acquire loads with N relaxed loads plus a single acquire fence.
  
*Forced by:* One fence instead of one acquire per rec — on AArch64 that is N ldapr/ldar replaced by one dmb. The TSan special case is a tooling constraint: 'tsan does not instrument fences', so under TSan it must fall back to real acquire loads or the sanitizer reports false races.

**`asymmetric_thread_fence_light / asymmetric_thread_fence_heavy (P1202r4)`**
— An asymmetric fence pair: light is nearly free, heavy is expensive, and a light fence interacts with a heavy fence exactly as two normal fences would.
  
*Forced by:* The biggest single performance idea in folly's hazptr. try_protect needs a seq_cst StoreLoad on the hottest reader path (~20-30 cycles for mfence / dmb ish, paid per protect). Asymmetric fences move the entire cost onto the rare reclaimer: on Linux light becomes just `asm_volatile_memory()` (a compiler barrier, zero instructions) and heavy becomes a process-wide barrier. 'A lightweight fence does not interact with another lightweight fence' — so you must get the light/heavy sides on the right threads or there is no ordering at all.

**`hazptr_detail::shared_head_tail_list<Node, Atom>`**
— Shared list that maintains both head_ and tail_, supporting push and pop_all; used by hazptr_obj_cohort.
  
*Forced by:* A cohort hands off whole runs of objects to the domain, so it needs the tail to splice in O(1). Note pop_all here is two exchanges (head then tail) and push has two cases (push_in_empty_list / push_in_non_empty_list) — the extra complexity is exactly the price of maintaining a tail, which is why the domain's own lists are head-only.

**`kNumShards = 8 sharded untagged_[] and tagged_[] + calc_shard (std::hash(ftag) >> kIgnoredLowBits & kShardMask)`**
— Split retired-object lists across 8 heads, chosen by hashing the object address (untagged) or the cohort tag (tagged).
  
*Forced by:* A single retired-list head is a CAS hot spot under many-thread retire; 8 shards cut contention ~8x. kIgnoredLowBits = 8 discards low address bits before hashing because allocator alignment makes them non-random. Sharding is also what makes list_walk_sharded's parallel walk possible, and tagged objects must shard by tag (not address) so cleanup_cohort_tag only has to lock one shard.

**`kSyncTimePeriod = 2000000000 ns, due_time_, check_due_time(), set_due_time()`**
— Second, time-based reclamation trigger: force a round every ~2 seconds regardless of count.
  
*Forced by:* A workload that retires slowly would sit below 1000 forever and hold its garbage indefinitely — memory grows, and in a queue the segments are never freed. The time trigger bounds reclamation latency, not just volume. Note the guard `if (rcount < 0) { add_count(rcount); return 0; }`: a negative snapshot must be given back, not acted on.

**`asymmetric_thread_fence_light(seq_cst) in push_list (the retire path)`**
— Separate the writer's unlinking store from making the object visible on the retired list.
  
*Forced by:* Symmetric to the reader side: the writer must not let the retire-list push be observed before the unlink, or a reader could still legally find the node via src after it is already a reclamation candidate. Light again, because retire is on a writer's critical path.

**`match_reclaim_untagged(untagged[], hs, done) and the ObjList& children return channel`**
— For each untagged retired object: if protected, keep it; else invoke reclaim_ and splice any children it produced back onto the not-reclaimed list.
  
*Forced by:* The reclaim function signature is `void (*)(Obj*, ObjList&)` precisely so reclamation can be iterative rather than recursive. Reclaiming a node whose child's link count just hit zero would otherwise recurse to the depth of the structure and blow the stack on a long chain.

**`num_bulk_reclaims_ + inc/dec + wait_for_zero_bulk_reclaims() + domain::cleanup()`**
— cleanup() runs a reclamation round with rcount 0 and then spins (yielding) until no other bulk reclamation is in flight.
  
*Forced by:* cleanup() must guarantee that everything currently reclaimable is actually gone when it returns — including objects a concurrent async round already popped out of the lists, where they are invisible to a fresh pop_all. Without the counter, cleanup() could report done while another thread still holds a batch. Essential for tests that assert zero leaks.

**`schedule_reclamation / exec_fn_ / set_executor / clear_executor / enable_hazptr_thread_pool_executor() / FLAGS_folly_hazptr_use_executor`**
— Hand the reclamation round to a thread-pool executor instead of running it in whichever thread happened to cross the threshold.
  
*Forced by:* Latency fairness. Reclaiming 1000+ cold objects is milliseconds-scale; charging it to a random unlucky retire-er produces a huge tail-latency spike, and that thread may be holding locks. invoke_reclamation_may_deadlock's comment is explicit: 'if this program encounters deadlock, then this may be the cause. Most likely this program did not call folly::enable_hazptr_thread_pool_executor.' exec_backlog_ warns above 10 queued rounds, i.e. reclamation can't keep up with retirement.

**`~hazptr_domain: shutdown_ flag, reclaim_all_objects(), reclaim_list_transitive(), reclaim_unconditional(), free_hazptr_recs()`**
— Unconditional teardown: free everything without consulting hazard pointers, and intentionally leak the default domain's recs.
  
*Forced by:* At domain destruction there are by contract no readers left, so the hazard check is pointless work. free_hazptr_recs' comment is the real pressure: 'Leak the hazard pointers for the default domain to avoid destruction order issues with thread caches' — a thread-local cache can outlive or be destroyed after the global domain, and freeing recs it still points at is a UAF during process exit.

**`hazptr_tc<Atom> thread cache + hazptr_tc_entry + hazptr_tc_tls() via SingletonThreadLocal`**
— Per-thread LIFO of recs belonging to the default domain, so acquire/release is a plain array index bump.
  
*Forced by:* Without it, every holder construction is a CAS (or lock-bit acquire) on the shared avail_ list — non-scalable and ~5ns. The header states the design target: 'The hot paths (try_get, try_put) touch two cache lines for pure thread-local work with no atomics or contention.' This is what makes make_hazard_pointer cheap enough to call per operation.

**`hazptr_tc::try_get / try_put / fill(num) / evict(num) / evict(), kCapacity = 16`**
— Fast-path pop/push against a std::vector<Entry>, with bulk refill from and drain to the domain.
  
*Forced by:* Bulk fill/evict amortize the shared-lock round trip over up to 16 recs. kCapacity = 16 is also the static cap asserted against M in hazptr_array and hazptr_local ('M must be within the thread cache capacity'), so M is bounded at compile time and the fast path needs no overflow branch.

**`hazptr_array<M, Atom> + make_hazard_pointer_array<M>() + aligned_hazptr_holder (aligned_storage_for_t)`**
— M hazard pointers acquired and released in one thread-cache transaction, stored in raw aligned storage with placement new.
  
*Forced by:* Documented as 'most of the functionality of M hazptr_holder-s but with faster construction/destruction (for M > 1)'. Hand-over-hand traversal and CHM lookups need 2-3 simultaneously, and ~5ns x M per operation is significant. The destructor bypasses ~hazptr_holder entirely, writing hprecs straight into tc slots and calling tc.evict() only if (M + count) > cap. Cost: documented restrictions — 'Do not move from or to individual hazptr_holder-s', and swap only between same-domain, same-emptiness holders.

**`hazptr_obj_cohort<Atom> + set_cohort_tag / set_cohort_no_tag / cohort() / tagged() with kTagBit = 1 stolen from the cohort pointer`**
— A per-structure retired-object list; the tag bit in cohort_tag_ marks whether the objects also need identity for synchronous reclamation.
  
*Forced by:* Two distinct pressures, both documented. (1) Locality/straggler avoidance: cohorts 'keep related objects together instead of being spread across thread local structures and/or mixed with unrelated objects' — the UnboundedQueue Segment case, where 'keeping them in cohorts helps avoid cases of a few missing objects delaying the reclamation of large numbers of link-counted objects'. (2) Synchronous reclamation. The tag bit is stolen into the low bit of the cohort pointer 'to save the space of separate cohort and tag data members', i.e. one word per protected object. set_cohort_tag/set_cohort_no_tag are explicitly not thread-safe.

**`cohort kThreshold = 20 + check_threshold_push() + pushed_to_domain_tagged_`**
— Accumulate 20 retired objects locally, then push the whole batch to the domain in one operation.
  
*Forced by:* One shared-list CAS and one add_count per 20 retires instead of per retire. pushed_to_domain_tagged_ is a pure optimization for the other end: shutdown_and_reclaim() can skip the whole cleanup_cohort_tag scan if the cohort never actually pushed tagged objects to the domain.

**`cohort active_ / clear_active() / shutdown_and_reclaim() / ~hazptr_obj_cohort`**
— After shutdown, push_obj reclaims immediately instead of listing; the destructor drains the cohort unconditionally.
  
*Forced by:* Objects can still be retired to a cohort while it is tearing down (a last in-flight operation); without the active_ check they would be pushed onto a list nobody will ever drain. The DCHECKs (`DCHECK(!active()); DCHECK(l_.empty());`) encode the postcondition.

**`hazptr_obj_linked<Atom>: single Atom<uint64_t> count_ packing kLink = 1<<32 and kRef = 1 with kLinkMask / kRefMask`**
— Two counters in one atomic word — link count (inbound links from mutable paths) and ref count (inbound links from immutable paths).
  
*Forced by:* Two separate atomics would be two cachelines and two RMWs where the algorithms frequently need to inspect or move value between both (downgrade_link subtracts `kLink - kRef` in one CAS). Packing makes 'downgrade a mutable link into an immutable ref' a single atomic step; splitting them would need a lock or a two-step window where the object looks unreferenced.

**`release_link / release_ref / downgrade_link and the off-by-one ref-count convention`**
— Decrement the packed counts, returning whether the object is now unreferenced.
  
*Forced by:* The documented convention is the trick: 'a new object without inbound links has a ref count of 0 and an about-to-be-reclaimed object can be viewed to have a ref count of -1'. That makes the overwhelmingly common case (`oldval == 0u` in release_ref, `oldval == kLink` in release_link) a plain store with no CAS at all, so the fast path of 'last reference goes away' costs nothing. Readers pay zero regardless: 'All the link counting features incur no extra overhead for readers.'

**`hazptr_obj_base_linked<T, Atom, D>: retire() vs unlink() vs unlink_and_reclaim_unchecked(), and the for_each_link(bool m, F&& f) contract`**
— Base class for link-counted objects, supporting explicit retirement when removal is certain and implicit retirement (via unlink) when it is uncertain.
  
*Forced by:* A hazard pointer on node A does not protect A->next — that is the fundamental gap for linked structures. Link counting closes it so a reader that reached a child through a protected parent is safe, at zero reader cost. The certain/uncertain split is concrete: 'Removal in UnboundedQueue is certain, whereas removal is ConcurrentHashMap is uncertain.' The for_each_link contract carries a sharp aliasing rule: 'for_each_link must therefore read each child pointer before invoking f on it, and must not access that child again after the call', because f may have already freed it.

**`The Atom template-template parameter threaded through every class`**
— Substitute an instrumented atomic type (e.g. DeterministicAtomic) for std::atomic throughout the library.
  
*Forced by:* Named as a deliberate deviation from the standard: 'This library uses an extra atomic template parameter for testing and debugging.' A reclamation scheme's bugs are rare interleavings; without a way to swap the atomic type you cannot drive deterministic interleaving tests, and the correctness argument stays unverified. It also forces the canUseExecutor check `std::is_same<Atom<int>, std::atomic<int>>{}` — the executor path is real-atomics-only.

**`default_hazptr_domain() via detail::createGlobal<impl, void> plus extern hazptr_domain<std::atomic> default_domain with FOLLY_STATIC_CTOR_PRIORITY_MAX`**
— Process-global default domain with controlled construction order.
  
*Forced by:* The default domain must be constructed before any translation unit's static that retires an object, and must survive until after the last thread's cache is gone — hence max static-ctor priority on one side and the deliberate rec leak in free_hazptr_recs() on the other. Static init/destruction order is the classic way a global reclamation registry crashes at startup or exit.

### OPTIONAL (22)

**`try_protect(ptr, Src&& src) — callable source overload`**
— Protect from any invocable returning T*, not just an Atom<T*>.
  
*Forced by:* Real data structures read the candidate out of a packed word, a struct field, or a tagged head; forcing every source to literally be std::atomic<T*> would make the primitive unusable for them.

**`is_default_domain_ with its own alignas(hardware_destructive_interference_size)`**
— A bool telling the fast paths whether thread-cache shortcuts are legal.
  
*Forced by:* Quoted directly: 'is_default_domain_ is loaded in every call to make_hazard_pointer and ~hazptr_holder. This is the only piece of domain state that is loaded for reader chrome or critical sections when using the default domain.' If it shared a line with count_ / avail_ / due_time_ (all CAS targets), every reader would eat a coherence miss on a line other threads are writing.

**`Chunked hazptr load loop in load_hazptr_vals (constexpr size_t chunk_width = kNumShards; const void* ptrs[chunk_width])`**
— Load 8 hazard slots into registers first, then test them, instead of load-test-load-test.
  
*Forced by:* Pure ILP: 'load a batch of hazard pointers up-front so that the branches below can run in parallel with each other on x86... the loads are served either from registers or from the store buffer and do not block each other'. When the rec array is large but sparse (most slots null) the dependent load→branch chain stalls the front end.

**`asymmetric_thread_fence_heavy_fn::impl_ via sysMembarrierPrivateExpedited(), cached by sysMembarrierAvailableCached()`**
— Force a memory barrier on every core currently running one of this process's threads.
  
*Forced by:* That is what makes the reader's zero-cost light fence sound: the heavy side pays for both. Availability is probed once and cached in a relaxed_atomic<char> because the probe is a syscall.

**`detail::hazptr_prefer_fence_light = kIsArchAArch64 && kIsLinux && !kIsSanitizeThread`**
— Compile-time switch that converts reset_hazptr's store-release into light-fence + relaxed store, and load_hazptr_vals' acquire fence into a heavy fence.
  
*Forced by:* Architecture-specific cost: on x86 a release store is a plain mov (free), so the trick buys nothing and costs reclaimer time; on AArch64 stlr is materially more expensive than str, so shifting the cost to the reclaimer wins. Encodes 'know which of your orderings are free on your target'.

**`hazptr_deleter<T, D> with specialization for std::default_delete<T>`**
— Hold a custom deleter, specialized to be empty when the deleter is plain delete.
  
*Forced by:* Empty base optimization, stated in the header: 'For empty base optimization.' Every protected node inherits it, so a non-specialized version would add a word (or padding) to every node in every hazptr-protected data structure for a feature almost nobody uses.

**`hazptr_domain::retire(T* obj, D reclaim) — nonintrusive, allocating`**
— Retire an object that does not derive from hazptr_obj, by heap-allocating a hazptr_retire_node wrapper holding a unique_ptr<T, D>.
  
*Forced by:* Marked in the source as '/** retire - nonintrusive - allocates memory */'. Needed for third-party / non-inheritable types, but the comment is a warning: it puts a malloc on the retire path, which is exactly what the intrusive design avoids.

**`list_walk_sharded + FOLLY_BUILTIN_PREFETCH(next, 0, 2)`**
— Walk the 8 shard lists round-robin, one node per shard per round, prefetching each successor into L2 as soon as its address is known.
  
*Forced by:* The longest comment in the file, and it is all about microarchitecture: 'A long linked-list can create long loop-carried dependency chains, eventually stalling the processor... Retired objects are typically cold, so this is a real miss per node. Prefetch into L2, not L1. L1 line-fill buffer entries are scarce, so pinning one per node for a whole fill from DRAM is discouraged.' Eight interleaved chains turn 8 serial DRAM misses into 8 overlapped ones. Note it is instruction-level parallelism, not threads.

**`extract_retired_objects() lock dance (check_lock() → pop_all(kAlsoLock) → push_unlock(empty) if nothing found)`**
— Grab untagged lists unlocked and wait-free, and tagged lists only if not already locked by a synchronous cleanup.
  
*Forced by:* 'Tagged lists need to be locked because tagging is used to guarantee the identification of all objects with a specific tag. Locking protects against concurrent hazptr_cleanup_tag() calls missing tagged objects.' If a shard is already locked, async reclamation skips it rather than blocking — asynchronous work must never be allowed to stall on synchronous work.

**`match_tagged(tagged[], hs) with per-shard run batching into cohorts[s] / safe[s]`**
— Sort tagged objects into protected (pushed back under the lock) and safe; safe ones are pushed to their owning cohort, coalesced into runs of the same cohort.
  
*Forced by:* Async reclamation must not free a tagged object itself — a concurrent cleanup_cohort_tag could be mid-flight for that tag. So it hands the object back to the cohort, which owns the race. The run batching ('if past the end of a run of the same cohort, push the run') turns one CAS per object into one CAS per run, which matters because retired objects of one structure tend to arrive contiguously.

**`detail::hazptr_inline_executor_add — thread_local std::queue<Function<void()>>`**
— When no executor exists, run reclamation inline but flatten re-entrancy: if already inside one, enqueue instead of recursing.
  
*Forced by:* Reclaiming objects can retire more objects, which can cross the threshold, which schedules another round — unbounded recursion and stack overflow. The thread-local queue converts that recursion into a loop, with catch_exception around each so a throwing destructor cannot abort the drain.

**`'Tagged objects remain' warning in ~hazptr_domain (kIsDebug && !tagged_empty()) and kListTooLarge = 100000 / hazptr_warning_list_too_large`**
— Diagnostics for higher-level leaks of cohort-using objects and for retired lists that have grown pathologically.
  
*Forced by:* A reclamation scheme fails silently: the symptom of a bug is memory that never comes back, with no crash and no stack. These two warnings are the only signal; both are rate-limited (`warning_count++ % 10000`) so a pathological program does not drown in logs.

**`hazptr_tc decay mechanism: kDecayThreshold = 1024, put_tick_, window_start_, decay(), shrink_to(), try_put_slow()`**
— Grow the cache past kCapacity on demand, then gradually give the excess recs back to the domain when a 1024-put window shows they were not actually used.
  
*Forced by:* Recs are never freed, so a thread that transiently needed 64 hazard pointers would hoard 64 cacheline-sized recs forever — and because the reclaimer scan is O(hcount_), one thread's hoarding permanently taxes every reclamation in the process. Decay bounds that global cost. try_put_slow resetting put_tick_ = 0 is what makes 'active use starts a new decay window' work.

**`hazptr_tc::local_ / local() / set_local() (kIsDebug only)`**
— Debug-only flag asserting that at most one hazptr_local exists per thread at a time.
  
*Forced by:* hazptr_local's whole speed advantage comes from not popping recs out of the cache — it just borrows tc[0..M). Two concurrent hazptr_locals would therefore alias the same recs. The header is explicit that this 'is not tracked and checked by the implementation (except in debug mode) because it would negate the performance gains of this class'.

**`hazptr_local<M, Atom>`**
— Borrow tc[0..M) in place: constructor does not pop from the cache, destructor only calls reset_protection().
  
*Forced by:* Quantified in the header: '~2 ns vs ~5 ns for construction/destruction' even at M=1, and '(by ~3 ns.)' faster than hazptr_holder in the Config example. The price is a severe non-composability contract: 'it is unsafe for the current thread to construct any other holder-type objects... while the current instance exists', so nothing the critical section calls may itself use hazard pointers. A good lesson in what the last 3 ns costs in API safety.

**`cohort safe_list_top_ + push_safe_objs() + reclaim_safe_list()`**
— Channel by which the domain's async reclamation returns tagged-but-unprotected objects to their cohort for the cohort to free.
  
*Forced by:* Resolves the race between async and sync reclamation of the same tagged object: the async round has already proven it is unprotected, but freeing it could collide with a cleanup_cohort_tag for that tag, so ownership is handed back to the single entity that can serialize both. push_obj() checks `if (safe_list_top())` on the retire path so draining is opportunistic rather than requiring another trigger.

**`hazptr_domain::cleanup_cohort_tag(cohort) + list_match_tag / list_match_condition`**
— Synchronous reclamation: guarantee that every object with the given tag is reclaimed before the call returns, without checking hazard pointers at all.
  
*Forced by:* Hazard pointers alone give no deadline — a ConcurrentHashMap destructor must destroy all its Key and Value objects before returning, and 'all objects with the specified tag must be reclaimed unconditionally'. Skipping the hazard check is sound only because the cohort is already shut down. This is the one capability folly has that the WG21 proposal does not: 'The standard proposal does not include cohorts and the associated synchronous reclamation capabilities.'

**`acquire_link / acquire_ref versus acquire_link_safe / acquire_ref_safe`**
— Atomic increment versus plain load+store increment of the packed counts.
  
*Forced by:* Before an object is published, no other thread can see it, so a fetch_add (~20 cycles, locks the line) is pure waste; count_inc_safe() does a relaxed-ish load then store. The '_safe' suffix means 'you, the caller, guarantee thread safety' — the API makes the cheap path available but forces the caller to own the proof.

**`downgrade_retire_immutable_descendants / release_delete_immutable_descendants / release_retire_mutable_children + Worklist = small_vector<..., 2>`**
— Walk a node's outbound links iteratively, retiring or deleting descendants whose counts hit zero in the same pass.
  
*Forced by:* Named in the class docs as 'One-pass reclamation of long immutable chains of objects'. Without it, freeing a 10k-node immutable chain takes 10k reclamation rounds, each with its own full hazard scan and threshold wait — unacceptable for UnboundedQueue segment chains. small_vector<_, 2> keeps the worklist off the heap for the typical fanout of one or two links, since this runs inside reclamation.

**`hazptr_root<T, Atom>`**
— A holder for a link from a static root whose destructor calls p->unlink() on the linked object.
  
*Forced by:* 'Use example: Bucket heads in ConcurrentHashMap.' Static roots are the base case of link counting — without a root type that participates, the counts for top-level nodes never reach zero and nothing in an acyclic structure is ever reclaimed. It is the mechanism that makes 'Automatic reclamation of acyclic structures' true.

**`detail::Sleeper (bounded spin then kMinYieldingSleep)`**
— Back-off used by the avail_ lock bit and by shared_head_only_list::pop_all_lock().
  
*Forced by:* Both of those are short critical sections guarded by a bit, so blocking primitives are too heavy, but pure spinning burns a core and can livelock under oversubscription. A bounded spin then sleep is the standard compromise.

**`FOLLY_ALWAYS_INLINE / FOLLY_NOINLINE / FOLLY_LIKELY / FOLLY_UNLIKELY discipline`**
— Force the reader fast paths inline and push every slow path out of line (get_or_create_hprecs_slow, try_put_slow, decay, fill, evict, pre_retire_check_fail, both warning functions).
  
*Forced by:* protect/try_protect/holder ctor+dtor are inlined into every reader call site, so their icache footprint is multiplied across the program. Anything rare that would bloat them is deliberately a non-inlined call. The FOLLY_LIKELY on `hprec_ != nullptr`, `domain->is_default_domain()`, `tc.try_put(hprec_)` encodes the expected path explicitly for the branch predictor and the layout.

### SKIP (5)

**`mprotectMembarrier() TLB-shootdown fallback`**
— Emulate a process-wide barrier where membarrier(2) PRIVATE_EXPEDITED is unavailable, by mmap-ing a dummy page, upgrading it to PROT_READ|PROT_WRITE, touching it to force residency, then downgrading to PROT_READ.
  
*Forced by:* 'We cannot force (1) and we cannot force (2). But we can force at least one of the outcomes (1) or (2) to happen!' — either the page was swapped out (barrier already happened) or the protection downgrade sends TLB-shootdown IPIs to every core running the process, each of which is a barrier. A leaked Indestructible<std::mutex> serializes it because it must be shutdown-safe.

**`FOLLY_HAZPTR_THR_LOCAL macro (false when FOLLY_MOBILE)`**
— Compile the thread cache in or out entirely; every holder/array/local path has a #else branch that goes straight to the domain.
  
*Forced by:* Matches the library's own stated limitation: 'When not to use hazard pointers: when thread local data is not supported efficiently.' On platforms where TLS access is a function call, the cache is a pessimization and the fallback must still be correct.

**`hazard_pointer / hazard_pointer_domain / hazard_pointer_obj_base / hazard_pointer_default_domain / hazard_pointer_clean_up aliases`**
— WG21 P1121-conformant names layered over the folly names.
  
*Forced by:* Migration path toward std, documented under 'Differences from the Standard Proposal'. Purely nominal.

**`hazptr_obj_retired_list<Atom> (with check_threshold_try_zero_count)`**
— A wrapper around shared_head_only_list adding a count and the lock-mode constants.
  
*Forced by:* Appears to be vestigial: hazptr_domain aliases `RetiredList = hazptr_detail::shared_head_only_list<Obj, Atom>` directly and keeps its own count_, so this class is not on the live path. Its alignas(hardware_destructive_interference_size) on retired_ shows the original intent of one padded list-plus-count per shard.

**`delete_hazard_pointers() / hazptr_tc_evict() / hazptr_tc_tester`**
— Tear down the rec array and thread cache so a benchmark can measure cold-start cost.
  
*Forced by:* Both are labelled 'Used only for benchmarking'. They exist because the thread cache and the never-shrinking rec array make the steady state unmeasurable otherwise — and delete_hazard_pointers() must call cleanup() first 'to ensure that there is no lagging concurrent asynchronous reclamation in progress'.

### Notes

SOURCES. All eight requested paths exist on main (no 404s). Downloaded to /private/tmp/claude-501/-Users-thanhngo-tngo-projects-reth-journey/a8002ece-ec52-48d3-9876-8b160b9ef895/scratchpad/folly/ — Hazptr.h (209L), HazptrDomain.h (937L), HazptrHolder.h (461L), HazptrObj.h (537L), HazptrObjLinked.h (328L), HazptrRec.h (90L), HazptrThrLocal.h (277L), AsymmetricThreadFence.h (89L). I also pulled four files the task did not name but that carry load-bearing mechanism: Hazptr-fwd.h (all config constants and the hazptr_prefer_fence_light switch), detail/HazptrUtils.h (the three list templates — this is where the lock bit and reentrance live, arguably the most reusable code in the library), AsymmetricThreadFence.cpp (the mprotect TLB-shootdown trick), HazptrDomain.cpp (the inline-executor recursion flattener), plus Hazptr.cpp and HazptrThreadPoolExecutor.h.

THE SHORT VERSION OF WHAT FORCED WHAT. Six pressures explain almost every line:
(1) Reader cost must be ~1-2 ns. Forces: thread cache, hazptr_array/hazptr_local, asymmetric fences, hazptr_prefer_fence_light, the FOLLY_ALWAYS_INLINE discipline, is_default_domain_ on its own cacheline.
(2) Reclamation is O(retired + hazards), so it must be amortized. Forces: threshold() = max(1000, 2*hcount_), cohort batching at 20, the F14FastSet, bulk acquire/release.
(3) Garbage must be bounded, and reclamation latency too. Forces: the hcount_-proportional threshold (the actual proof of hazptr's advantage over RCU), kSyncTimePeriod = 2s time trigger, the do_reclamation `done` loop, tc decay (a hoarding thread inflates hcount_ and taxes everyone).
(4) A hazard pointer protects one object, not what it points to. Forces: the whole HazptrObjLinked.h layer, hazptr_root, the ObjList& children return channel, the iterative descendant walks.
(5) Hazard pointers give no deadline. Forces: cohorts, tags, the kTagBit, cleanup_cohort_tag, the lock bit on tagged lists, safe_list_top_, the whole async/sync coexistence protocol.
(6) Cold retired objects and long lists stall the pipeline. Forces: kNumShards = 8, list_walk_sharded with L2 prefetch, the chunked load loop, the `counts[kNumShards]` arrays that exist only to break loop-carried dependencies.

CONSTANTS AS MEASURED, for a back-of-envelope track: kThreshold 1000 retired objects; kMultiplier 2; kNumShards 8; kIgnoredLowBits 8; kSyncTimePeriod 2e9 ns; kListTooLarge 100000; cohort kThreshold 20; tc kCapacity 16; tc kDecayThreshold 1024 puts; executor backlog warning at 10. Folly's own quoted latencies: hazptr_holder ctor+dtor ~5 ns, hazptr_local ~2 ns, and ~3 ns saved in the Config example by using hazptr_local<1>.

A SUGGESTED LADDER if this becomes a from-scratch Rust crate. Stage 1 (everything marked core): a Domain holding a Vec/grow-array of HazPtr slots, a Guard with Drop, protect() with the load/store/SeqCst-fence/reload loop, a Retired trait object list, a HashSet scan, a count threshold, double-retire detection, and cleanup(). That stage is already enough to back a lock-free queue correctly, and it is where loom and Miri pay off most. Stage 2 (important): thread-local cache, free list, sharding, time trigger, cohorts/batches, array-of-M guards, the executor split. Stage 3 (optional, each a self-contained experiment with a measurable before/after): asymmetric fences, the prefetching sharded walk, cache decay, link counting.

TWO THINGS WORTH FLAGGING FOR THE SEGQUEUE ENDGAME. First, folly's UnboundedQueue uses hazptr_obj_base_linked with an *immutable* link between segments plus set_cohort_no_tag — so for a segmented queue the cohort and the link-counting layers are not optional extras, they are the mechanism folly actually ships. Second, folly deliberately centralizes retired lists in the sharded domain rather than per-thread as in Michael's 2004 paper; that is what makes the global garbage bound provable and lets one scan amortize over all threads' garbage, but it is also why kNumShards and the lock bit exist at all. Worth deciding which of the two shapes to build before writing the Rust Domain.

UNCERTAINTY. (a) hazptr_obj_retired_list looks dead on the live path (the domain aliases shared_head_only_list directly and keeps its own count_); I did not grep all of folly to confirm nothing else uses it, so 'skip' is a judgement call, not a proven fact. (b) I did not read Hazptr-test / HazptrBench, so folly's current measured numbers may differ from the ~2 ns / ~5 ns figures in the header comments, which are old. (c) I did not read atomic_grow_array's implementation in full, only its doc comment and the policy hook the domain supplies — the reference-stability and monotonic-size guarantees quoted above are from that comment. (d) The task framing mentions epoch reclamation; folly's hazptr has no epoch mechanism at all (folly's RCU is a separate file, Rcu.h), so nothing here maps onto crossbeam-epoch's global epoch / local pin / three-bag design. That comparison needs a separate read of Rcu.h and crossbeam-epoch.

---

## `crossbeam-epoch` — the complete design

47 mechanisms, including the three-epoch argument and the per-pin cost against hazard pointers' per-protect cost.

### CORE (22)

**`Collector`**
— A standalone, shareable epoch-based GC domain: `Collector { global: Arc<Global> }`, cloneable, `PartialEq` via `Arc::ptr_eq`, with `register() -> LocalHandle`.
  
*Forced by:* Every data structure sharing one domain shares garbage and one epoch clock; separate domains let an unrelated stalled thread not block your reclamation, and let a structure's garbage die with the structure. `Arc` so a `LocalHandle` outlives the `Collector` that made it.

**`Global (global epoch + participant list + garbage queue)`**
— The shared state of one domain: `locals: List<Local>`, `queue: Queue<SealedBag>`, `epoch: CachePadded<AtomicEpoch>`.
  
*Forced by:* Exactly three pieces of global state are required by the algorithm: a clock, a way to enumerate who is pinned, and a place for ripe garbage. `CachePadded` on the epoch because it is read by every single pin and written on every advance — false sharing with `locals`/`queue` would turn every pin into a coherence miss.

**`LocalHandle`**
— A `*const Local` wrapper; `pin()`, `is_pinned()`, `collector()`; `Drop` decrements the handle count.
  
*Forced by:* Separates 'this thread is registered with the domain' (handle) from 'this thread is currently in a critical section' (guard), so registration cost is paid once per thread rather than once per pin.

**`Local (the participant record)`**
— Per-thread state: intrusive `entry`, `collector`, `bag: UnsafeCell<Bag>`, `guard_count: Cell<usize>`, `handle_count: Cell<usize>`, `pin_count: Cell<Wrapping<usize>>`, `epoch: CachePadded<AtomicEpoch>`.
  
*Forced by:* Only the owning thread mutates it, so everything except `epoch` and `entry.next` is a non-atomic `Cell` — that is the whole reason a pin is cheap. `epoch` is cache-padded because other threads poll it during `try_advance`.

**`Participant registration (`Local::register`)`**
— Heap-allocates a `Local` via `Owned::new`, pushes it onto `global.locals` using `unprotected()`, returns a `LocalHandle` with handle_count = 1.
  
*Forced by:* `try_advance` must enumerate every potentially-pinned thread; a lock-free push-at-head list is the cheapest enumerable registry. `unprotected()` is legal here because registration dereferences no shared pointer.

**`guard_count — nesting / re-entrancy`**
— `pin()` increments it and only touches the epoch + fence when it transitions 0 → 1; `unpin` only clears the local epoch at 1 → 0.
  
*Forced by:* Composed data structures pin inside code that is already pinned. Without a nesting counter an inner guard's drop would unpin the outer critical section and free memory the outer frame still holds. It also makes a nested pin cost one non-atomic increment and zero fences.

**`Guard`**
— `Guard { local: *const Local }` — proof of pinned-ness; gates every `Atomic::load`, carries `defer*`, `flush`, `repin`, `repin_after`, `collector`; `Drop` unpins.
  
*Forced by:* Turns 'I am inside a critical section' into a lifetime the borrow checker enforces, so `Shared<'g, T>` cannot escape the section. A raw pointer rather than `&Local` so the null value can encode `unprotected()`.

**`pin() fast path (relaxed load of global, store local = global|PINNED, SeqCst fence)`**
— The entire reader-side protocol: read the global epoch, publish it into the local epoch marked pinned, then a full barrier.
  
*Forced by:* The barrier is mandatory and is the whole cost of EBR: it stops any later `Atomic` load from being reordered before the publication of the local epoch. Without it a reclaimer could run `try_advance`, see this thread unpinned, free a node, and only then would this thread's load of the pointer execute.

**`unpin (`Local::unpin`)`**
— Decrement guard_count; at 1 → 0 store `Epoch::starting()` (unpinned, epoch 0) into the local epoch with `Release`; then finalize if the handle count is also 0.
  
*Forced by:* `Release` (not Relaxed) so reads performed inside the critical section cannot be reordered after the announcement that the section is over. No fence needed — on x86 this is a plain `mov`, which is why leaving a critical section is nearly free.

**`Epoch representation: LSB = pinned flag, rest = wrapping counter`**
— One word holds both 'is this participant pinned' and 'in which epoch'. `successor()` adds 2, `pinned()` sets bit 0, `wrapping_sub` shifts out the flag and returns a signed distance in `(MIN/2)..(MAX/2)`.
  
*Forced by:* `try_advance` must read a participant's pinned-ness and epoch in a *single* atomic load — two words would admit a torn read where a thread looks unpinned while holding an old epoch. Prefers `AtomicU64` even on 32-bit so wraparound is unreachable in practice.

**`The three-epoch rule (`SealedBag::is_expired`: `global.wrapping_sub(seal) >= 2`)`**
— The single predicate that decides when garbage may be freed.
  
*Forced by:* One advance only proves 'everyone pinned is currently at X'; it does not prove the X-era readers have left. Two advances are needed — see notes for the full argument. This is the heart of EBR and the one line a reimplementation must get right.

**``Global::try_advance``**
— Relaxed-load the global epoch, `fence(SeqCst)`, walk every registered `Local`, and if any is pinned in an epoch other than the current one, bail out returning the unchanged epoch; otherwise `fence(Acquire)` and `store(global.successor(), Release)`.
  
*Forced by:* The quiescence check. The leading SeqCst fence pairs with the fence in `pin()` — together they rule out the interleaving where the pinner reads the old epoch and the advancer reads the stale not-yet-pinned local. The plain `store` (not CAS) is safe precisely because a pinned participant can witness at most one advancement, so a late store can only rewrite the same value. `#[cold]`.

**``Global::collect` + `COLLECT_STEPS = 8``**
— Call `try_advance`, then pop and drop at most 8 expired bags from the global queue.
  
*Forced by:* Incremental, bounded work so no single operation pays an unbounded reclamation pause — the latency-smoothing choice. The 8 is arbitrary and tunable. `#[cold]` so the compiler lays the branch out of the pin fast path.

**`Bag + `MAX_OBJECTS = 64``**
— A fixed array of 64 `Deferred`s plus a length, with `try_push` returning the deferred back on overflow; `Drop` runs all of them.
  
*Forced by:* Batching: the global queue is a lock-free MS queue, so one push per retired object would be one CAS-contended node allocation per object. 64 objects per push amortises that to ~1/64. Fixed-size (no Vec) so the bag never allocates in the retire path. Dropped to 4 under Miri/sanitize to force the race windows open.

**`SealedBag (epoch, Bag) + `unsafe impl Sync``**
— A bag stamped with the global epoch at the moment it was handed off.
  
*Forced by:* The epoch stamp is what makes `is_expired` possible. `Sync` is asserted on the grounds that the only cross-thread inspection is `is_expired`, which touches only the epoch field, never the closures.

**``Global::push_bag` and its leading `fence(SeqCst)``**
— Swap the thread's bag for a fresh one, `fence(SeqCst)`, read the global epoch Relaxed, push the sealed bag.
  
*Forced by:* The fence forces the unlinking stores (which made these objects unreachable) to be ordered before the epoch read. Without it the bag could be stamped with an epoch *older* than the unlink and be freed one grace period too early. Note the stamp is the *global* epoch, which may be one ahead of the pinner's — conservatively safe, since a larger stamp only delays the free.

**`defer / defer_unchecked / defer_destroy`**
— `defer` (safe, requires `F: Send + 'static`), `defer_unchecked` (drops the bounds, requires `move` by convention), `defer_destroy(Shared<T>)` = `defer_unchecked(move || ptr.into_owned())`. All push into the local bag, flushing it to the global queue when full. On an `unprotected()` guard they execute immediately.
  
*Forced by:* `defer_unchecked` exists because the type system cannot prove the usual case safe: `Shared` is not `Send`, yet handing its destructor to another thread *is* safe after a grace period. `defer_destroy` is the 99% ergonomic path.

**``Atomic<T: ?Sized + Pointable>``**
— `AtomicPtr<()>` + `PhantomData<*mut T>`; every load takes a `&'g Guard` and returns `Shared<'g, T>`. Full op set: `load`, `load_consume`, `store`, `swap`, `compare_exchange`, `compare_exchange_weak`, `fetch_update`, `fetch_and/or/xor`, `into_owned`, `try_into_owned`.
  
*Forced by:* The guard parameter is the type-level enforcement of the protocol — you cannot obtain a dereferenceable pointer without proof of pinning, and the `'g` lifetime stops it outliving the section. Erasing to `*mut ()` is what makes `?Sized` support possible in one word.

**``Owned<T>``**
— A `Box`-like unique owner of a heap object not yet published: `new`, `init`, `from_raw`, `into_box`, `into_shared(&guard)`, `tag`/`with_tag`, `Deref`/`DerefMut`, `Drop` deallocates.
  
*Forced by:* Models the pre-publication phase where unique ownership makes `&mut` safe, and converts into `Shared` exactly at the publication point. Also the type reclamation returns to: `Shared::into_owned()` then drop.

**``Shared<'g, T>` and its lifetime`**
— `Copy` pointer-plus-tag tied to the guard's `'g`; `is_null`, `as_raw`, `tag`/`with_tag`, and the unsafe `deref`/`deref_mut`/`as_ref`/`into_owned`/`try_into_owned`.
  
*Forced by:* The protection witness. Deref is still `unsafe` because the guard only promises 'not yet reclaimed', not 'logically still valid or uniquely owned' — EBR gives you memory safety, not ABA safety or exclusivity.

**`Pointer tagging (`low_bits`, `compose_tag`, `decompose_tag`, `ensure_aligned`, `map_addr`)`**
— Steal `align_of::<T>().trailing_zeros()` low bits of every pointer for a small tag, with an assert that stored pointers are aligned.
  
*Forced by:* Lock-free algorithms need to atomically change a pointer *and* a flag in one word — the registry list's DELETED mark, Harris/Michael logical deletion, SegQueue's DESTROY bit. `map_addr` (rather than a plain cast) preserves pointer provenance for Miri/strict-provenance.

**``compare_exchange` / `compare_exchange_weak` + `CompareExchangeError``**
— CAS taking `Shared` as expected and any `Pointer<T>` (`Owned` or `Shared`) as new; on failure returns both the observed `current` and the `new` value back, so an `Owned` is not leaked.
  
*Forced by:* Returning ownership of `new` on failure is the whole ergonomic point — a failed CAS in a retry loop must not drop or leak the node you were trying to install. (Master has since split this into `CompareExchangeValue { old, new }`; 0.9.21 returns `Shared`.)

### IMPORTANT (15)

**`handle_count + `Local::finalize` (self-hosted participant reclamation)`**
— When guard_count and handle_count both reach 0, `finalize` temporarily bumps handle_count, pins, pushes the local bag into the global queue, reads out the `Collector`, marks `entry` deleted, then drops the `Collector` Arc (possibly destroying `Global` and running all remaining deferreds).
  
*Forced by:* A thread can exit while another thread is mid-`try_advance` walking the list and holding a `&Local`. So the `Local` itself has to be reclaimed by the very scheme it implements — logical delete now, physical free after a grace period. The handle_count bump is needed because the internal `pin()` would otherwise recurse into `finalize`.

**`repin / repin_after`**
— `repin` republishes the current global epoch into the local epoch if this is the only guard (`Release` store, deliberately no following SeqCst fence). `repin_after` acquires a handle, fully unpins, runs a closure, and re-pins in a panic-safe drop guard.
  
*Forced by:* A thread pinned in an old epoch blocks *all* reclamation in the domain. These give long-running or blocking code a way to stop being the bottleneck. `repin` needs no trailing fence because publishing the new epoch late only delays GC; it never frees too early. Both take `&mut self` so no `Shared` can survive across the call.

**``pin_count` + `PINNINGS_BETWEEN_COLLECT = 128` (the epoch-advance policy)`**
— Non-atomic wrapping counter incremented on each outermost pin; when `count % 128 == 0` (count read *before* increment, so the very first pin fires) the thread calls `collect()`.
  
*Forced by:* Amortisation: scanning the participant list and advancing the epoch is O(threads) with cache misses, so it must not happen on every pin. Pinning frequency is a decent proxy for garbage-production rate, and the counter is thread-local so it costs one non-atomic add.

**``Deferred` — inline small-closure optimisation`**
— An erased `FnOnce()` stored as `{ call: unsafe fn(*mut u8), data: MaybeUninit<[usize; 3]> }`; closures of ≤ 3 words and compatible alignment are written inline, larger ones are boxed and the `Box` is stored inline instead.
  
*Forced by:* Makes `Deferred` `Sized` so a `Bag` can be a plain array, and makes the overwhelmingly common retire (`move || ptr.into_owned()`, one word) allocation-free. Three words is chosen to fit a fat pointer plus a function pointer. `PhantomData<*mut ()>` keeps it !Send/!Sync so the unsafety is forced through `Bag`'s explicit `unsafe impl Send`.

**``Guard::flush``**
— Push a non-empty local bag to the global queue even if not full, then `collect()`.
  
*Forced by:* Garbage parked in a thread-local bag is invisible to every other thread — if that thread goes idle, those objects are never freed. `flush` is the escape hatch for 'retire it now, I may not come back', and is what `Local::finalize` uses at thread exit.

**``unprotected()``**
— A `&'static Guard` with a null `local`. Loads/derefs are allowed but unsafe-by-contract (caller guarantees no concurrent mutation); `defer` runs immediately, `flush`/`repin` are no-ops, `collector()` is `None`.
  
*Forced by:* Construction and `Drop` of a data structure are exclusive by definition — pinning there would be pure cost and would needlessly hold back the epoch. Also breaks the bootstrap cycle in `Local::register` (you must touch `Atomic`s before any participant exists). Stored in a static via a `Sync` newtype since `Guard` is not `Sync`.

**``fetch_and` / `fetch_or` / `fetch_xor` on tags`**
— Atomic bit operations that mutate only the tag and return the previous pointer-with-tag.
  
*Forced by:* Logical deletion must be a single atomic RMW that cannot lose a concurrent pointer update — `Entry::delete` is exactly `next.fetch_or(1, Release)`. A load-modify-CAS loop would be both slower and ABA-prone here.

**``sync::list` — intrusive lock-free participant registry`**
— Michael-style singly linked list with push-at-head `insert` (`compare_exchange_weak(.., Release, Relaxed)`), `iter(&guard)`, and a `Drop` that asserts every node was logically deleted.
  
*Forced by:* `try_advance` must enumerate participants without taking a lock (a lock here would serialise every GC attempt and could be held by a descheduled thread). Intrusive (`Entry` embedded in `Local`) so registration allocates nothing extra. The source itself flags the design cost: pointer-chasing a list is cache-miss-heavy.

**``Entry::delete` — logical delete via `fetch_or(1, Release)` + lazy physical unlink`**
— Deregistration sets the DELETED tag on `next`; the physical unlink is done opportunistically by whichever thread next iterates past it, which then calls `C::finalize` → `defer_destroy`.
  
*Forced by:* The classic Harris/Michael split: you cannot CAS the predecessor's `next` safely without first marking, or a concurrent insert after a removed node is lost silently (hence the `debug_assert!(curr.tag() == 0)`). Deferring the physical unlink to readers means thread exit is O(1) and never blocks.

**``IterError::Stalled` and the restart-from-head protocol`**
— If the iterator's own predecessor turns out to be logically deleted, iteration restarts from head and yields `Err(Stalled)` once. `try_advance` treats `Stalled` as 'abandon this advance attempt, the racing thread will do it'.
  
*Forced by:* A traversal of a concurrently mutated list has no consistent position to resume from. Surfacing it as an error rather than silently restarting lets the caller choose — and `try_advance` choosing to bail is a deliberate livelock-avoidance / no-wasted-work decision, at the price that a contended registry can delay epoch advancement indefinitely.

**``sync::queue` — Michael-Scott queue of SealedBags, with `try_pop_if``**
— MS lock-free queue with a sentinel head, lagging tail that poppers help advance, and a `try_pop_if(cond)` that only dequeues when the head's predicate holds.
  
*Forced by:* `try_pop_if` is what lets `collect()` peek at the head bag's epoch and leave it alone if unexpired, without ever removing-and-reinserting. The 'advance the tail before retiring the old head' step exists so a reachable node is never retired. Notably the queue's own nodes are reclaimed by epoch GC — the collector is built on itself.

**``default.rs` — the process-wide default collector`**
— `static COLLECTOR: OnceLock<Collector>` plus `thread_local! { static HANDLE: LocalHandle }`; `pin()`/`is_pinned()` go through `with_handle`, which falls back to `collector().register()` when `try_with` fails.
  
*Forced by:* The fallback is the subtle part: another TLS destructor can call `pin()` *after* `HANDLE` has been destroyed, so the lazy path must tolerate a dead TLS slot by registering a throwaway handle rather than panicking. There is a regression test (`pin_while_exiting`) for exactly this.

**``CachePadded` on the global epoch and each local epoch`**
— Keeps the two hottest words in the algorithm off each other's and their neighbours' cache lines.
  
*Forced by:* The global epoch is read by every pin and written by every advance; each local epoch is written by its owner and polled by every advancer. Co-locating either with ordinary fields turns every pin into a coherence transaction. Measurable, not cosmetic.

**`loom / Miri test shims (`mod primitive`, `UnsafeCell` wrapper, reduced constants)`**
— A `primitive` module swapping std atomics/`UnsafeCell`/`thread_local`/`Arc` for loom's under `cfg(crossbeam_loom)`, a std `UnsafeCell` wrapper exposing loom's `with`/`with_mut` API, and `MAX_OBJECTS = 4` under Miri/sanitize.
  
*Forced by:* The algorithm's bugs are interleaving bugs; it is written to be model-checkable from day one rather than retrofitted. The `with`/`with_mut` shim means production code is already in loom's required shape. Shrinking the bag forces the push-to-global path to fire inside tiny Miri runs.

**``unsafe impl` audit trail (`Bag: Send`, `SealedBag: Sync`, `Collector: Send + Sync`, `Atomic: Send + Sync where T: Send + Sync`)`**
— The four places the crate asserts a safety property the compiler cannot see, each with the reasoning inline.
  
*Forced by:* `Deferred` is deliberately `!Send + !Sync`, so every crossing of a thread boundary is forced through one of these documented assertions instead of being implicit. Worth copying as a discipline.

### OPTIONAL (7)

**`x86 `lock cmpxchg`-instead-of-`mfence` hack in pin()`**
— On x86/x86_64 (not Miri) the SeqCst fence is produced by `self.epoch.compare_exchange(starting, new, SeqCst, SeqCst)` plus a `compiler_fence(SeqCst)`, instead of `store(Relaxed) + fence(SeqCst)`.
  
*Forced by:* Measured: a `lock`-prefixed RMW on a line this thread already owns exclusively beat `mfence`. The comment admits it may not be justified by the C++ model (SC fences and SC accesses differ) — a pure micro-optimisation with a correctness caveat.

**`AtomicEpoch`**
— Typed wrapper over `AtomicU64`/`AtomicUsize` exposing load/store/compare_exchange in terms of `Epoch`.
  
*Forced by:* Keeps the pin-flag bit arithmetic in one module instead of scattering `| 1` / `& !1` across the pin path.

**``Deferred::NO_OP``**
— A const no-op used to fill the unused array slots and to `mem::replace` out of a slot while draining.
  
*Forced by:* `Deferred` isn't `Default`-able by derive and the array needs a const initialiser; replacement-with-NO_OP is how a slot is moved out without a partial-move or double-call.

**``fetch_update``**
— CAS-loop helper: `FnMut(Shared) -> Option<Shared>` retried until success or `None`.
  
*Forced by:* Convenience over hand-rolled retry loops; carries no new mechanism.

**``load_consume` (crossbeam-utils `AtomicConsume`)`**
— A dependent-load: on ARM/AArch64 a `Relaxed` load plus `compiler_fence(Acquire)`; elsewhere a plain `Acquire` load.
  
*Forced by:* On weak-memory machines Acquire costs a `dmb ishld`; address-dependency ordering is free in hardware. Disabled under Miri/loom/TSan because none of them model consume. Pure performance, zero cost on x86 where Acquire is already free.

**``Pointable` trait (`ALIGN`, `Init`, `init`, `as_ptr`, `as_mut_ptr`, `drop`)`**
— Generalises `Box<T>` to anything representable by a single word, so `Atomic`/`Owned`/`Shared` can hold `?Sized` payloads.
  
*Forced by:* Atomic ops are single-word only, so a fat pointer cannot be stored. `Pointable` is the abstraction that forces any DST into one word. Needed only if you want slice payloads.

**``IsElement` trait + `#[repr(C)]` with `entry` first`**
— Maps element ↔ embedded `Entry` by plain pointer cast (`local.cast::<Entry>()`), and names the finalizer for an unlinked node.
  
*Forced by:* Avoids an `offset_of!`/memoffset dependency (removed in 0.9.17) and lets one type sit in several lists via several `Entry` fields. Guarded only by a comment that `entry` must stay first — a real footgun.

### SKIP (3)

**``[MaybeUninit<T>]` Pointable impl / `Array<T> { len, elements: [_; 0] }``**
— Stores the slice length *inside* the allocation (unlike `Box<[T]>`, which keeps it in the fat pointer), with a hand-built `Layout::extend` + `pad_to_align`.
  
*Forced by:* Lets a dynamically sized array live behind a one-word atomic pointer — used by crossbeam-skiplist's variable-height nodes. Pure extra; irrelevant to a SegQueue or a learning build.

**``crossbeam_sanitize` / `crossbeam_sanitize_thread` knobs`**
— Unbounded `COLLECT_STEPS`, Acquire loads substituted for fences, and an allocating per-local re-load so TSan (which cannot reason about fences) stops false-positiving.
  
*Forced by:* ThreadSanitizer does not model `fence`, so the fence-based design is invisible to it; the code carries an alternate, fence-free shape just for that tool.

**``alloc_helper` / `no_std` + `target_has_atomic` cfg gating`**
— Allocator shims and per-feature module gating so the crate builds on `no_std` and on targets without atomic CAS.
  
*Forced by:* Portability requirement of the crossbeam project; no algorithmic content.

### Notes

SOURCES. Full master sources downloaded and read at /private/tmp/claude-501/-Users-thanhngo-tngo-projects-reth-journey/a8002ece-ec52-48d3-9876-8b160b9ef895/scratchpad/ce/{lib.rs, epoch.rs, internal.rs, collector.rs, guard.rs, atomic.rs, deferred.rs, default.rs, sync/list.rs, sync/queue.rs, alloc_helper.rs}. Released version is 0.9.21. One API drift to know: master splits CAS results into `CompareExchangeValue { old, new }`; 0.9.21's `compare_exchange` returns `Result<Shared<'g,T>, CompareExchangeError<'g,T,P>>`. Everything else matches.

EXACT CONSTANTS (all from source, not recalled): `MAX_OBJECTS = 64` (4 under miri/sanitize); `Local::PINNINGS_BETWEEN_COLLECT = 128`; `Global::COLLECT_STEPS = 8`; `Deferred::DATA_WORDS = 3`; expiry threshold `global.wrapping_sub(seal) >= 2`; epoch LSB is the pin flag so `successor()` adds 2.

=== THE THREE-EPOCH ARGUMENT ===

Setup. The global epoch is an unbounded wrapping integer. The *only* invariant `try_advance` enforces is:

  (I) global may go X -> X+1 only if, at the instant of the scan, every pinned participant has local epoch == X.

Two consequences follow, and they are the whole algorithm.

(A) A pinned participant can witness at most one advancement. If thread A is pinned at epoch E, the advance E -> E+1 can happen (A satisfies (I)). The advance E+1 -> E+2 cannot: it requires every pinned participant to be at E+1, and A is at E. So while A stays pinned, global is in {E, E+1}. (This is also why `try_advance` may finish with a plain `store` rather than a CAS: a concurrent advancer can only ever be writing the *same* successor value, never a value two steps ahead.)

(B) Therefore a reader lags the global epoch by at most 1, which is exactly why 2 is not enough and 3 is.

Why one generation is unsound. Concretely:
  1. Thread A pins: reads global = E, publishes local = E|pinned, fences. It loads `p` from an `Atomic`, and holds it.
  2. Thread B pins in E, unlinks that node, `defer_destroy`s it. The bag is sealed with global epoch E.
  3. B unpins. Someone calls `try_advance`: the only pinned participant is A, at E. (I) is satisfied -> global becomes E+1.
  4. If the rule were `global - seal >= 1`, the node is now freed. But A is still pinned at E and still holds `p`. Use-after-free.

The flaw is that the advance E -> E+1 certifies "everyone currently pinned is *at* E". It says nothing about those E-era readers having *left*. One more advance is what certifies that: the advance E+1 -> E+2 can only succeed when every pinned participant is at E+1, i.e. every thread that was pinned at E has either unpinned or re-pinned at E+1 — and either way has dropped its `Shared<'g>` references, because `'g` is the guard's lifetime. So:

  seal epoch E  ->  advance to E+1 (everyone pinned is at E)  ->  advance to E+2 (everyone E-era reader is gone)  ->  free.

Hence three simultaneously-meaningful generations at any instant: E+1 = current, E = lagging-but-legal readers, E-1 = garbage now ripe. In Fraser's original formulation that is literally epoch mod 3 with three garbage lists — with only two values (mod 2) you cannot distinguish "one behind" (a live reader) from "two behind" (safe to free), and you free under a lagging reader. crossbeam avoids modular arithmetic by using a 63-bit wrapping counter and `wrapping_sub(..) >= 2`, which is the same requirement expressed as a threshold instead of a ring size. The module header states it as: "If an object became garbage in some epoch, then we can be sure that after two advancements no participant will hold a reference to it."

Two supporting details that are easy to miss and are load-bearing:
- `push_bag` stamps with the *global* epoch, which by (A) may be one ahead of the retiring thread's own pin epoch. That is conservative (a larger stamp only delays freeing), so it is safe — but it means EBR here is slightly lazier than the minimum.
- `push_bag`'s leading `fence(SeqCst)` orders the unlinking stores before the epoch read. Without it the bag could be stamped with an epoch *older* than the unlink and be freed a full grace period early. This fence is as essential as the one in `pin()`.

Fence pairing: `pin()` = publish local epoch, then SeqCst fence, then loads. `try_advance` = load global, SeqCst fence, then read locals. The two SeqCst fences are what forbid the interleaving "pinner reads old global; advancer reads stale unpinned local" — i.e. they are the reason the quiescence test is not a race.

=== READER COST: EBR PIN vs HAZARD-POINTER PROTECT ===

I am giving exact *operation counts* (read off the source, trustworthy) and separately flagged ballpark cycle figures (published/folklore, NOT measured in this session — measure locally before quoting).

EBR, outermost `pin()` — per CRITICAL SECTION:
  - 1 TLS lookup (`HANDLE.try_with`, on x86-64 a `mov fs:[..]` plus an initialised-check branch)
  - 1 Relaxed load of the global epoch (shared read-mostly line, usually S-state hit)
  - 1 full barrier publishing the local epoch: on x86 a `lock cmpxchg` on this thread's *own* cache line (so uncontended, E-state); elsewhere `store(Relaxed) + fence(SeqCst)`
  - 1 non-atomic `Cell` increment of guard_count, 1 of pin_count, 1 `% 128` test
  - 0 allocations
`unpin()`: 1 Release store (a plain `mov` on x86) + 1 `Cell` decrement. 0 allocations.
Nested `pin()`: 1 `Cell` increment. No atomics, no fence, no TLS cost beyond the lookup.
Per *pointer* dereferenced inside the section: ZERO extra cost — just whatever ordering the data structure itself needs (`Acquire`, or `load_consume`).
Retire: 1 `Deferred` construction (0 allocations when the closure is ≤ 3 words — the normal `move || p.into_owned()` case), 1 array store. Amortised ~1/64 of (one MS-queue node allocation + a push CAS). Every 128th outermost pin additionally pays a `try_advance` (O(#threads) pointer-chasing walk, cache-miss dominated) plus up to 8 bag drops.
Ballpark: the single full barrier dominates, order ~20-40 cycles (~6-12 ns at 3GHz); a nested pin is ~1 ns.

Hazard pointers — per PROTECTED POINTER:
  - 1 Acquire load of the source
  - 1 store publishing the pointer into a hazard slot, which MUST be ordered before the re-read: a seq_cst store or `store(Release) + fence(SeqCst)` — a store-load barrier, so the same ~20-40 cycle class as the epoch pin's fence
  - 1 Acquire re-load of the source to validate; if it changed, retry the whole triple. The retry loop is unbounded under heavy mutation.
  - plus slot acquire/release: amortised ~free when the per-thread slot cache hits (folly caches a handful per thread via `hazptr_tc` / reusable `hazptr_holder`), but a cold miss walks a global slot list.
  - 0 allocations in the steady state (folly's `hazptr_obj` is intrusive).
Retire + reclaim: push onto a thread-local retired list (0 allocations), and when the retired count crosses a threshold scaled by the number of live hazard slots, a scan: snapshot ALL hazard pointers (O(H) global-list walk, cache-miss heavy), build a set, match against the retired list (O(R)) — so reclamation is O(H + R), versus EBR's O(#threads) scan plus O(1) per bag.

The asymmetry, stated plainly:
  - HP pays per *protected pointer*, and pays a validation retry loop. EBR pays once per *critical section*, regardless of how many pointers you touch. A traversal that hand-over-hands 30 nodes costs EBR one fence and HP up to 30 fences + 30 validations (or 2 rotating slots with 30 publish-validate rounds).
  - HP's memory is *bounded*: a stalled reader blocks only the O(k) objects it has actually published. EBR's is *unbounded*: a single stalled-or-descheduled pinned thread blocks every object retired from its epoch onward, forever. EBR is lock-free in progress but blocking in memory.
  - The folly result that inverts the per-pointer verdict: with asymmetric fences (`folly::asymmetricLightBarrier` on the reader + `membarrier(MEMBARRIER_CMD_PRIVATE_EXPEDITED)` on the reclaimer), the reader's publish becomes a plain store and the reader-side barrier disappears entirely — making an HP protect *cheaper* than an epoch pin. The cost moves to the writer: a syscall that IPIs every thread (~microseconds), amortised over a bulk reclamation. That trade is the single most important design idea to carry over from folly, and it is orthogonal to EBR vs HP.

=== STALLED PINNED THREAD ===
Three distinct stall paths in this code, all ending in "reclamation just stops":
  1. `try_advance` sees a `Local` pinned in a different epoch -> returns the unchanged global epoch immediately. Nothing is freed beyond already-expired bags, and the clock never moves while that thread stays pinned.
  2. `try_advance` gets `IterError::Stalled` from the registry iterator -> also bails, explicitly delegating to the racing thread ("in which case we leave the job to it. Otherwise, the epoch will not be advanced"). Under a churning registry this can delay advancement indefinitely.
  3. Garbage parked in an idle thread's local bag (up to 64 objects) is invisible to everyone until that thread pins again, overflows the bag, calls `flush()`, or exits. `flush()` and `Local::finalize` are the only cures.
A descheduled pinned thread is indistinguishable from a crashed one, so the memory bound is unbounded by construction. `repin`/`repin_after` exist solely as the user-side mitigation, and separate `Collector`s are the blast-radius mitigation.

=== REIMPLEMENTATION LADDER (what "core" buys you) ===
A sound minimal EBR is genuinely small: global `AtomicU64` epoch; a registry (start with `Mutex<Vec<Arc<Participant>>>` — it is NOT on the critical path, only in `try_advance`, and swapping it for the intrusive lock-free list later is an isolated step); per-thread `AtomicU64` local epoch with the LSB pin flag; `Guard` with `guard_count` nesting; `Vec<Deferred>` bags stamped with an epoch; `try_advance` + `is_expired >= 2`. Everything marked "important" is a performance or robustness layer you can add one at a time and measure: bag batching, the 128/8 amortisation policy, inline `Deferred`, `CachePadded`, the lock-free registry list, the MS garbage queue, the TLS-destruction fallback. The two things worth doing early despite being "important" rather than "core" are the loom/Miri shim shape (retrofitting it is painful) and `CachePadded` (so your first benchmark is not measuring false sharing).

A deliberate self-hosting note for the plan: in crossbeam the registry nodes, the garbage-queue nodes, AND the `Local`s themselves are all reclaimed by the collector itself. That bootstrap (which is what `unprotected()` exists to break) is the hardest single thing to get right in a from-scratch build, and it is skippable at first by leaking participant records.

UNCERTAINTY. The cycle/ns figures above are order-of-magnitude from published microbenchmarks and the x86 cost model, not measured here — they belong in a bar-(c) back-of-envelope section with a "measure before quoting" flag. The folly comparison is from memory of folly/synchronization/Hazptr*; I read crossbeam-epoch's source in this session but did NOT fetch folly's, so treat the folly specifics (slot cache size, threshold multiplier, exact asymmetric-fence entry points) as needing verification against the real folly sources.

---

## What `haphazard` did not port

The gap between Jon Gjengset's crate, folly, and WG21 P1121 — useful as a list of decisions you will also have to make.

### CORE (7)

**`hazptr_tc — thread-local cache of hazard-pointer records`**
— Makes acquiring and releasing a hazard pointer a pure thread-local operation (no atomics, two cache lines touched) instead of a global CAS.
  
*Forced by:* folly's kCapacity=16 TLS vector with a usage-aware decay mechanism exists because make_hazard_pointer/~hazptr_holder sit on the hottest reader path; haphazard instead takes a lock-bit spinlock on one global `head_available` list for every HazardPointer::new() and every drop, so N readers serialize on one word.

**`Real asymmetric barriers (membarrier MEMBARRIER_CMD_PRIVATE_EXPEDITED) for light/heavy fence pair`**
— Lets the reader side pay almost nothing (a compiler fence) while the reclaimer pays the full cross-core sync.
  
*Forced by:* haphazard declares `fn asymmetric_light_barrier()` and `fn asymmetric_heavy_barrier()` (lib.rs:146,157) as private fns that both just do `fence(SeqCst)`, with TODOs pointing at folly's Asm.h and AsymmetricMemoryBarrier.cpp; PR #25 ("Use fastbarriers", via the `membarrier` crate) has been open and unmerged for years. So every reader protect pays a full SeqCst fence.

**`Empty hazard-pointer state + empty()`**
— P1121 distinguishes an empty hazard_pointer (owns no record) from one owning an unassociated record, so holders can be default-constructed, moved-from, and stored in arrays without acquiring domain resources.
  
*Forced by:* haphazard's HazardPointer::new() always calls domain.acquire(); there is no empty state, no `empty()`, and no move-out-leaves-empty semantics. This blocks the P1121 move-assignment and swap contracts.

**`swap(hazard_pointer&, hazard_pointer&) — hand-over-hand traversal`**
— Swap record ownership between two holders without ending either protection epoch, which is how P1121's own list-traversal example advances a two-pointer cursor.
  
*Forced by:* P1121 §?.6.4 explicitly guarantees "No protection epochs are ended or initiated"; its §5.2 example calls `swap(hptr_curr, hptr_prev)` in the loop. haphazard has no swap, and ties the returned reference's lifetime to `&'l mut self` (`fn protect<'l,T>(&'l mut self, ...) -> Option<&'l T>`), so you cannot mem::swap two HazardPointers while holding the references you just loaded — the pattern is structurally unexpressible.

**`try_protect with a pointer-bit filter function (tagged/marked pointers)`**
— folly's `try_protect(T*& ptr, const Atom<T*>& src, Func f)` runs the loaded word through `f` before storing it in the hazard slot, so the low bits can carry a mark.
  
*Forced by:* Comment in HazptrHolder.h: "Filtering the protected pointer through function Func is useful for stealing bits of the pointer word". Harris-style list deletion and most lock-free sets need a (pointer, mark) CAS. haphazard's try_protect_ptr compares with `core::ptr::eq(ptr, ptr2)` and has no filter; issue #27 ("Musings on marked atomic pointers") was written up and then closed as out of scope, pointing users at oliver-giersch/hazptr instead.

**`try_protect over an arbitrary validating source, not just &AtomicPtr<T>`**
— folly overloads try_protect with `Src&& src` (any invocable returning T*), so validation can re-read a different location than the one protected.
  
*Forced by:* Michael-Scott and DoubleLink queues protect `head->next` but must re-validate by re-reading the entry pointer to head, because next never changes after dequeue. haphazard's only escape hatch is `protect_raw` — and open issue #55 points out that the needed `asymmetric_light_barrier` is private, so you cannot correctly synchronize with the reclaimer. PR #56 fixing this is still open. This is directly on the path for any segmented queue.

**`Batched retire (hazptr_obj_list / hazptr_domain_push_retired)`**
— Push a whole linked list of retired objects to the domain under one CAS and one count update.
  
*Forced by:* haphazard's `push_list` hard-asserts `"only single item retiring is supported atm"`, so retiring a chain of K nodes costs K CASes on the shard head plus K separate `Box<Retired>` allocations. A segmented queue retiring a 64-slot segment hits this immediately.

### IMPORTANT (13)

**`hazptr_prefer_fence_light + relaxed hazard store behind a light release fence`**
— On aarch64/Linux, store the hazard slot Relaxed and emit one light release fence, moving the cost to the reclaimer's heavy acquire fence.
  
*Forced by:* folly's HazptrRec.h reset_hazptr branches on `detail::hazptr_prefer_fence_light` (kIsArchAArch64 && kIsLinux && !kIsSanitizeThread); haphazard's record.rs unconditionally does `self.ptr.store(ptr, Ordering::Release)`.

**`Per-object runtime deleter (hazard_pointer_obj_base<T,D>::retire(D d) / hazptr_retire(obj, reclaim))`**
— Lets each retired object carry a stateful deleter instance (return to a pool, close an fd, decrement an arena refcount) rather than a type-level one.
  
*Forced by:* P1121 move-assigns a value `d` of type D into the object and requires only Cpp17DefaultConstructible + Cpp17MoveAssignable. haphazard stores `deleter: unsafe fn(ptr: *mut dyn Reclaim)` — a bare fn pointer derived from the type parameter `P: Pointer<T>` (default Box<T>), so no closure state, no custom allocator deallocation.

**`hazptr_obj_cohort + tagged retired lists + cleanup_cohort_tag + shutdown_and_reclaim (synchronous reclamation)`**
— Groups retired objects into a cohort whose destructor guarantees every member's deleter has completed, so a container can be dropped and have all its memory and resources actually gone.
  
*Forced by:* P3135R0 recommends exactly this for C++29 and says global cleanup is "impractical" by comparison; folly has shipped it since 2018. haphazard has only `untagged: [RetiredList; NUM_SHARDS]` — no `tagged_` array, no kTagBit, and its reclaim function is literally named `match_reclaim_untagged` with no tagged counterpart. Its own docs admit "there is no guarantee that retired objects will be cleaned up by the time your data structure is dropped" and tell you to build a whole private Domain per instance instead.

**`hazptr_obj_linked / hazptr_obj_base_linked / hazptr_root — link and ref counting`**
— Two inbound counts (link count for mutable paths, ref count for immutable paths) that let a whole chain of objects be reclaimed in ONE hazard-pointer pass and allow protecting descendants of an already-protected object.
  
*Forced by:* P3135R0 §4 "Integrated Protection Counting" spells out the cost: with noncommutative counting a 1000-node chain takes "at least 1000 hazard pointer reclamation passes"; with commutative (link) counting it "may be reclaimed in one". This is the exact failure mode for a segment-chained queue. haphazard has two bare TODOs where this would go: `// TODO: Support linked nodes for more efficient deallocation (children).` (domain.rs:830) and `// TODO: handle children` (domain.rs:862, inside reclaim_list_transitive, which currently just forwards to reclaim_unconditional).

**`Double-retire detection`**
— Catch the most common hazard-pointer misuse — retiring the same object twice — instead of pushing it into the caller's unsafe contract.
  
*Forced by:* folly sets `next_(this)` in every hazptr_obj constructor specifically "in order to catch misuse bugs such as double retire", checked by pre_retire_check(). haphazard makes it safety requirement #2 on retire_ptr and leaves an open question in its own source: `// TODO: - requires double-retire protection?` (lib.rs:233).

**`Cache-line-aligned hazard records`**
— Stop readers on different cores from false-sharing each other's hazard slot.
  
*Forced by:* folly: `class alignas(hardware_destructive_interference_size) hazptr_rec`. haphazard's `pub(crate) struct HazPtrRecord { ptr, next, available_next }` has no alignment attribute, so up to 2-3 independent readers' hazard slots land on one 64-byte line and every protect/reset invalidates the others' line.

**`Retired-list shards actually spread across cache lines`**
— The point of sharding the retired list is to spread the push CAS across lines; if the shard heads share a line, sharding buys nothing against cross-core contention.
  
*Forced by:* haphazard's `struct RetiredList { head: AtomicPtr<Retired> }` is 8 bytes and NUM_SHARDS=8, so `untagged: [RetiredList; 8]` is exactly 64 bytes — one cache line. folly's shard element (shared_head_only_list: head + owner thread::id + reentrance counter) is ~24 bytes, spreading 8 shards over ~3 lines. Note: this is my inference from struct layout, not a measured benchmark.

**`Contiguous grow-array of records + chunked batched scan (load_hazptr_vals)`**
— Collect the guarded set by walking a contiguous array in chunks of 8 with the loads hoisted, so the dependent branches issue in parallel.
  
*Forced by:* folly uses an `atomic_grow_array<Rec>` plus an explicitly chunked loop with a comment explaining the loop-carried-dependency and store-buffer reasoning. haphazard chases a `next` pointer through a heap-allocated linked list (`let mut node = self.hazptrs.head.load(Acquire); while !node.is_null()`), which is a dependent-load chain per record.

**`Fast flat hash set for the guarded set instead of BTreeSet`**
— Build the protected-pointer set in O(1) per insert with no per-node heap allocation.
  
*Forced by:* folly uses `F14FastSet<const void*>`. haphazard uses `alloc::collections::BTreeSet` and flags it itself: `//XXX: Maybe use a sorted vec to reduce heap allocations, and have O(log(n)) lookups` (domain.rs:728). Every reclaim pass therefore allocates.

**`compare_exchange returning the actual observed current value on failure`**
— Let a CAS retry loop reuse the value that caused the failure instead of issuing another load.
  
*Forced by:* haphazard's `AtomicPtr::compare_exchange` throws it away: `r.map_err(move |_ptr| { // TODO: Return _ptr to the caller somehow. // Adding this to the API is a breaking change ... })` (lib.rs:588-597). Open issue #61 is a user asking for a working compare_exchange example at all. If you are designing a fresh API, return it from day one.

**`hazard_pointer_clean_up with the "deleter completion synchronizes-with return" guarantee`**
— A public, synchronizing cleanup: all definitely-reclaimable objects are reclaimed and their deleters' completion synchronizes with the return.
  
*Forced by:* P1121 §?.4 specifies exactly that. haphazard's public `eager_reclaim()` does NOT wait for concurrent bulk reclaims; the function that does (`fn cleanup(&self)` — "Only used for tests -- waits for no outstanding reclaims") is marked `#[doc(hidden)]`. So the standardized guarantee is unavailable in the public API.

**`Sound unique-domain construction (haphazard's unique_domain! soundness hole)`**
— The Singleton family mechanism is what makes cross-domain retire a compile error; it is currently breakable.
  
*Forced by:* Open issue #54: `unique_domain!` generates one unique type per macro invocation, but if the expansion runs twice (in a loop or a constructor) you get two Domains with the same family, which is exactly what `unsafe trait Singleton` promises cannot happen. The author: "I don't see an easy solution here, esp. not at compile time." If you copy the families design, know this hole exists; the later `static_unique_domain!` is the partial answer.

**`Specified semantics for re-protecting without reset`**
— Define what happens when a holder protects a second pointer while still protecting a first.
  
*Forced by:* P1121 nails it: try_protect's Effects first evaluate `reset_protection(old)`, and the spec defines protection epochs so that "Changing the association (possibly to the same object) initiates a new protection epoch and ends the preceding one." haphazard leaves it undocumented — open issue #28 asks exactly this and is unanswered. Cheap to get right; cheap to get subtly wrong.

### OPTIONAL (8)

**`No public "is this pointer protected?" query (the guarded-pointer scan is private to do_reclamation)`**
— Answers the specific question: haphazard offers no way to ask whether an address is currently covered by some hazard pointer except by retiring it and reading the reclaim count.
  
*Forced by:* The answer is stale the instant it is returned, so both haphazard and folly keep the scan inside the reclaim pass that already atomically stole the retired list; exposing it would hand users a racy predicate they would inevitably treat as a fence.

**`Reclamation offload / retire-without-reclaim (set_executor, hazptr_use_executor, exec_backlog warnings)`**
— Keep a latency-sensitive thread from ever paying the reclaim scan by handing the work to an executor, with backlog detection.
  
*Forced by:* folly's schedule_reclamation() tries exec_fn_ first and warns at backlog >= 10; P3135R0 §5 calls this "Dedicated Reclamation Execution". haphazard has none, and open issue #18 ("Feature: retire without reclaim") requests it for soft-real-time/audio/GUI threads with two proposed signatures and no resolution.

**`Domain-level allocator (pmr::polymorphic_allocator)`**
— Route all hazard-pointer bookkeeping allocation through a caller-supplied allocator.
  
*Forced by:* P1121 has `explicit hazard_pointer_domain(pmr::polymorphic_allocator<byte>)` with "All allocation and deallocation related to hazard pointers belonging to this domain use a copy of poly_alloc". haphazard hardcodes `alloc::boxed::Box` for both HazPtrRecord and Retired. Relevant if you want a no_std/arena build.

**`delete_hazard_pointers / shrinking the record pool`**
— Release hazard-pointer records back to the allocator while the domain is still alive.
  
*Forced by:* folly exposes `delete_hazard_pointers()` (benchmarking) and the TLS cache has a decay mechanism that returns excess records to the domain. haphazard's records are allocated monotonically and its comment is explicit: "HazPtrRecords are never de-allocated while the domain lives"; `free_hazptr_recs` is private and runs only in Drop. The global domain is a `static` and is never dropped, so its record count is a high-water mark for process lifetime.

**`hazptr_local<M> — stack-local fast holder array`**
— Construct M nonempty holders with no move/TLS-eviction bookkeeping, for a tight local scope.
  
*Forced by:* folly ships it with two documented warnings (no moving individual holders; only one hazptr_local active per thread, debug-checked only, "because it would negate the performance gains"). haphazard's HazardPointerArray is the rough analogue but is backed by the same global available-list, so it does not get the win. P3428R1 measures batch construct+destroy of 3 hazard pointers at 2 ns vs 6 ns individually.

**`Time-based reclamation trigger on no_std`**
— Trigger a reclaim pass every SYNC_TIME_PERIOD (2s) even when the retired count stays below threshold, so memory is not pinned indefinitely by a quiet writer.
  
*Forced by:* haphazard gates its due_time logic on `cfg(all(feature="std", target_pointer_width="64", not(loom)))` with `// TODO: Implement some kind of mock time for no_std. // Currently we reclaim only based on rcount on no_std` (domain.rs:694). So a no_std or 32-bit build silently loses half of the bounded-garbage mechanism the crate advertises.

**`Diagnostics: list-too-large and executor-backlog warnings`**
— Detect in production that retired lists are growing without bound (typically a leaked cohort or a never-released hazard pointer).
  
*Forced by:* folly has kListTooLarge=100000, hazptr_warning_list_too_large(), hazptr_warning_executor_backlog(), and a destructor warning "Tagged objects remain. This may indicate a higher-level leak". haphazard has no such instrumentation; a stuck hazard pointer just leaks silently.

**`fetch_or-based available-list lock instead of CAS + yield spin`**
— Let a thread claim the available-list lock with one unconditional RMW even when the head has changed.
  
*Forced by:* haphazard's own XXX (domain.rs:454): "This could be a fetch_or and allow progress even if there's a new (but unlocked) head. However, AtomicPtr doesn't support fetch_or ... This will in turn make Miri fail to track the provenance". It currently CASes and then `yield_now()`s on failure — a yield on the hazard-pointer acquire path. folly uses a uintptr_t with a lock bit. Moot if you take the TLS-cache route, which removes this list from the hot path entirely.

### SKIP (3)

**`is_default_domain() fast path`**
— A single cache-line-aligned bool read on the hot holder construct/destruct path to avoid touching the rest of the domain state.
  
*Forced by:* folly comments it as "the only piece of domain state that is loaded for reader chrome or critical sections when using the default domain". haphazard's family/Singleton generics give it the same information at compile time, so this one it arguably already wins on.

**`The documentation that was never written (empty "Differences from the specification" / "Differences from the folly" sections)`**
— The crate's own gap list does not exist, which is why this analysis had to be done from source.
  
*Forced by:* lib.rs has literal empty headings `# Differences from the specification` and `# Differences from the folly` followed by `//! TODO: Note differences from spec and from folly.` (lib.rs:116-119), plus `TODO: Ref section 3`, `TODO: Ref sections 3.4 and 4`, `TODO: Ref section 5`, and `TODO: Incorporate doc strings around expectations from section 6 of the hazptr TS2 proposal` (lib.rs:130). Open issue #20, "Make issues for missing features", says plainly: "There are a number of features from folly, and possibly from the spec, that we did not initially port" — and it was never acted on.

**`ABA-prevention framing`**
— Hazard pointers also solve ABA, because an address cannot be recycled while any hazard pointer names it.
  
*Forced by:* P1121 §1.1: "Solutions for the safe reclamation problem can also be used to prevent the ABA problem". haphazard notes it as `//! TODO: Can also help with the ABA problem` (lib.rs:19) and never develops it. Worth understanding, nothing to implement.

### Notes

## The specific question, answered with quotes

**No. haphazard exposes no way, at any visibility, to ask whether an address is currently protected, other than retiring it and reading the returned reclaim count.**

The guarded set is computed in exactly one place, inside a private method, and is never returned — `/private/tmp/claude-501/-Users-thanhngo-tngo-projects-reth-journey/a8002ece-ec52-48d3-9876-8b160b9ef895/scratchpad/hz/domain.rs:710-739`:

```rust
    fn do_reclamation(&self, mut rcount: isize) -> usize {   // <- private
...
                // Find all guarded addresses.
                #[allow(clippy::mutable_key_type)]
                //XXX: Maybe use a sorted vec to reduce heap allocations, and have O(log(n)) lookups
                let mut guarded_ptrs = BTreeSet::new();
                let mut node = self.hazptrs.head.load(Ordering::Acquire);
                while !node.is_null() {
                    // Safety: HazPtrRecords are never de-allocated while the domain lives.
                    let n = unsafe { &*node };
                    guarded_ptrs.insert(n.ptr.load(Ordering::Acquire));
                    node = n.next.load(Ordering::Relaxed);
                }

                let (nreclaimed, is_done) =
                    self.match_reclaim_untagged(stolen_heads, &guarded_ptrs);
```

`guarded_ptrs` is a local, passed by reference to another private fn, and dropped. The only callers of `do_reclamation` are `pub fn eager_reclaim(&self) -> usize` and `fn check_threshold_and_reclaim(&self) -> usize`, which is called from `fn push_list` — i.e. from `pub unsafe fn retire_ptr`. Both channels return only a `usize` count.

Every supporting datum is sealed:

- `record.rs:4-8`: `pub(crate) struct HazPtrRecord { pub(crate) ptr: AtomicPtr<u8>, pub(crate) next: ..., pub(crate) available_next: ... }` — the type itself is crate-private and is not re-exported from `lib.rs` (`pub mod raw` exports only `Domain`, `Global`, `HazardPointer`, `Pointer`, `Reclaim`).
- `domain.rs:149-158`: `pub struct Domain<F> { hazptrs: HazPtrRecords, untagged: [RetiredList; NUM_SHARDS], ... }` — every field private, no accessor, no iterator.
- `hazard.rs:30-33`: `pub struct HazardPointer<'domain, F = crate::Global> { hazard: &'domain HazPtrRecord, pub(crate) domain: &'domain Domain<F>, }` — `hazard` is private and there is no getter, so you cannot even read back what **your own** hazard pointer is currently protecting. There is no `empty()`, no `protected()`, no `get()`.
- A full grep for a query API returns nothing: `grep -iE "fn (is_)?protected|fn empty|is_protected|guarded" src/*.rs` hits only the comments and locals inside `do_reclamation`/`match_reclaim_untagged`.

Context worth carrying forward: **folly does not expose it either** — `Set load_hazptr_vals()` is a private member of `hazptr_domain`, and the comparison happens inside the private `match_reclaim_untagged` / `match_tagged`. P1121R3's synopsis has no such function. This is a deliberate property of the interface, not a haphazard omission: the predicate is stale the moment it returns, and it is only meaningful when the asker already holds the atomically-stolen retired list (folly's `extract_retired_objects` → `load_hazptr_vals` → match sequence, with the heavy fence between extraction and the scan). If your own crate wants a "can I free this now?" primitive, the sanctioned shape is retire-then-count, or your own reclaim pass that owns the retired list — not a public `is_protected(ptr)`.

## Complete TODO / XXX inventory in haphazard src (no FIXMEs exist)

Mechanism TODOs:
- `lib.rs:147` `// TODO: if cfg!(linux) {` + folly Asm.h#L28 link — light barrier unimplemented
- `lib.rs:158` `// TODO: if cfg!(linux) {` + folly AsymmetricMemoryBarrier.cpp#L84 — heavy barrier unimplemented
- `lib.rs:232-233` `// TODO: - copy_and_move test. - requires double-retire protection?`
- `lib.rs:590` `// TODO: Return \`_ptr\` to the caller somehow.` (compare_exchange failure value discarded; "breaking change, so we plan add this later")
- `domain.rs:694-695` `// TODO: Implement some kind of mock time for no_std. // Currently we reclaim only based on rcount on no_std`
- `domain.rs:830` `// TODO: Support linked nodes for more efficient deallocation (\`children\`).`
- `domain.rs:862` `// TODO: handle children`
- `hazard.rs:297` `// TODO: replace with \`self.haz_ptrs.each_mut().map(...)\` when each_mut stabilizes` (blocked on MSRV; PR #58 open)

Doc TODOs (the crate's own gap list, never written):
- `lib.rs:19` ABA problem
- `lib.rs:29` `TODO: Ref section 3 of [the proposal][cts]` — "High-level API structure" section is an empty heading
- `lib.rs:33` `TODO: Ref sections 3.4 and 4` — "Hazard pointers vs. other deferred reclamation mechanisms" empty
- `lib.rs:40` `TODO: Ref section 5` — "Examples"
- `lib.rs:118` `TODO: Note differences from spec and from folly.` — **both** "# Differences from the specification" and "# Differences from the folly" are empty headings
- `lib.rs:130` `TODO: Incorporate doc strings around expectations from section 6 of the hazptr TS2 proposal.`

XXX comments:
- `hazard.rs:129` borrowck bug workaround (rust-lang/rust#51545, #54663, #58910, #84361) — can't thread the `PhantomData<&'l T>` through `protect_ptr`
- `domain.rs:454` `XXX: This could be a fetch_or ... Miri provenance` (rust-lang/miri#1993)
- `domain.rs:522` `XXX: check that head and tail are connected` — debug assertion never written
- `domain.rs:728` `XXX: Maybe use a sorted vec to reduce heap allocations, and have O(log(n)) lookups`
- `domain.rs:770` `XXX: This can probably also be hoisted out of the loop, and we can do a _single_ reclaim_unprotected call as well.`

Ordering notes where haphazard deliberately diverges from folly (all in domain.rs): `:567` and `:990` "Folly uses Release, but needs to be both for the load on success"; `:572` "Folly uses SeqCst because it's the default, not clear if necessary"; `:808` "We're _not_ respecting sharding here, presumably to avoid multiple push CASes"; `:882` "folly skips this step for the global domain".

## What the open issues admit (10 open: 6 issues, 4 PRs)

- **#20 "Make issues for missing features"** — the blanket admission: *"There are a number of features from folly, and possibly from the spec, that we did not initially port. We should create an issue for each one..."* Never done.
- **#18** retire without reclaim (latency-sensitive threads) — unresolved
- **#55 / PR #56** `asymmetric_light_barrier` is private, so `protect_raw` users (Michael-Scott, DoubleLink queues) *"cannot synchronize the reading thread with the reclaiming thread"* — PR open, unmerged
- **#54** `unique_domain!` can produce two Domains with the same family, violating the `unsafe trait Singleton` contract — *"I don't see an easy solution here"*
- **#28** semantics of re-protecting without reset are unspecified
- **#61** no usable `compare_exchange` example
- **PR #25** real membarrier-based fast barriers — open
- **PR #12** better shard hash (128-bit multiply + xor) — open
- Closed-as-wontfix: **#27** marked/tagged pointers, closed by the author with "this is a bad idea / probably not even in scope", recommending `oliver-giersch/hazptr` instead.

Maintenance signal: the last three commits (through 2026-07-05) are all dependabot action bumps; the crate is at 0.1.8 and is effectively feature-frozen with 4 stale PRs.

## Where haphazard is actually AHEAD

Don't write these off when porting: (1) it has real custom domains with compile-time family enforcement — P3135R0 lists "Custom Domains" as a *future extension*, because C++26-as-voted (P2530R3) dropped the domain parameter that P1121R3/TS2 had; (2) its lifetime-based protection scoping (`&'l mut self` → `&'l T`) statically prevents use-after-reset, which neither folly nor P1121 can express; (3) it has loom tests (`tests/loom.rs`, 8.7 KB) and Miri support including a `miri_static_root` hook for the global domain.

## Uncertainty / caveats

- The cache-line claims (unpadded `HazPtrRecord`; all 8 `RetiredList` shard heads fitting in one 64-byte line because `RetiredList` is a single `AtomicPtr`) are read off struct definitions, **not benchmarked**. Verify with a `size_of`/`align_of` test and a contended microbench before quoting numbers.
- P1121R3 is the Concurrency TS2 wording and is what haphazard's docs link to. The version actually voted into C++26 is **P2530R3**, which drops the `hazard_pointer_domain` parameter from `retire()` and `make_hazard_pointer()`. P3135R0 (extensions: batches, cohorts, protection counting, reclamation execution, custom domains) and P3428R1 (batches, aiming at C++29) are the papers that codify the folly features missing from both. I read P1121R3, P3135R0 and P3428R1 directly; I did **not** fetch P2530R3's own wording, so treat the C++26-vs-TS2 delta as second-hand via P3135R0's "Background" section.
- PDF text was extracted with pypdf, which mangles whitespace in these WG21 documents; the API declarations I quote are reconstructed from that text and should be re-checked against the PDFs before being copied into code or docs.

## Local artifacts (all absolute paths, reusable)

Downloaded sources in `/private/tmp/claude-501/-Users-thanhngo-tngo-projects-reth-journey/a8002ece-ec52-48d3-9876-8b160b9ef895/scratchpad/hz/`:
- haphazard: `domain.rs` (1243 L), `hazard.rs` (366), `lib.rs` (709), `pointer.rs` (53), `record.rs` (18), `sync.rs` (18), `Cargo.toml`, `issues.json`, `contents.json`
- folly: `Hazptr-fwd.h`, `Hazptr.h`, `HazptrDomain.h` (937 L), `HazptrHolder.h`, `HazptrObj.h`, `HazptrObjLinked.h`, `HazptrRec.h`, `HazptrThrLocal.h`, `HazptrUtils.h`
- WG21: `p1121r3.pdf` + `p1121r3.txt`, `p3135r0.pdf` + `.txt`, `p3428r1.pdf` + `.txt`

## Sources

- [jonhoo/haphazard src (GitHub API)](https://api.github.com/repos/jonhoo/haphazard/contents/src)
- [haphazard domain.rs](https://raw.githubusercontent.com/jonhoo/haphazard/main/src/domain.rs), [lib.rs](https://raw.githubusercontent.com/jonhoo/haphazard/main/src/lib.rs), [hazard.rs](https://raw.githubusercontent.com/jonhoo/haphazard/main/src/hazard.rs), [record.rs](https://raw.githubusercontent.com/jonhoo/haphazard/main/src/record.rs), [pointer.rs](https://raw.githubusercontent.com/jonhoo/haphazard/main/src/pointer.rs)
- [haphazard open issues](https://api.github.com/repos/jonhoo/haphazard/issues?state=open&per_page=30)
- [P1121R3: Hazard Pointers — Proposed Interface and Wording for Concurrency TS 2](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2021/p1121r3.pdf)
- [P3135R0: Hazard Pointer Extensions](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2024/p3135r0.pdf)
- [P3428R1: Hazard Pointer Batches](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2024/p3428r1.pdf)
- [P2530R3: Why Hazard Pointers Should be in C++26](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2023/p2530r3.pdf)
- folly: [Hazptr-fwd.h](https://raw.githubusercontent.com/facebook/folly/main/folly/synchronization/Hazptr-fwd.h), [HazptrDomain.h](https://raw.githubusercontent.com/facebook/folly/main/folly/synchronization/HazptrDomain.h), [HazptrObj.h](https://raw.githubusercontent.com/facebook/folly/main/folly/synchronization/HazptrObj.h), [HazptrObjLinked.h](https://raw.githubusercontent.com/facebook/folly/main/folly/synchronization/HazptrObjLinked.h), [HazptrHolder.h](https://raw.githubusercontent.com/facebook/folly/main/folly/synchronization/HazptrHolder.h), [HazptrRec.h](https://raw.githubusercontent.com/facebook/folly/main/folly/synchronization/HazptrRec.h), [HazptrThrLocal.h](https://raw.githubusercontent.com/facebook/folly/main/folly/synchronization/HazptrThrLocal.h)
