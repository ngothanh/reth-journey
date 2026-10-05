# SMR coverage ledger

Every one of the **192 rows** in `notes/smr_inventory.md`, with a disposition. This exists
because two mechanisms were found silently absent from `plan/reclaim_ladder.md` — the hashed
guarded set and the time trigger — and neither was findable by auditing the plan's list of
*claimed* omissions, which by construction cannot contain the silent ones. Only a sweep from
the inventory side terminates.

Dispositions: **port Cn** · **ADD Cn** (gap found by this sweep) · **FIX** (plan and inventory
contradict) · **defer** (with a trigger) · **conditional** · **experiment** (a step-4
before/after, not scope) · **skip** · **decision** (deliberately different, with the reason) ·
**family out** (a reclamation family, not a mechanism) · **done**.

## Reclamation families and the theory — 41 rows

| Line | Bucket | Mechanism | Disposition | Note |
|---:|---|---|---|---|
| 41 | CORE | `No reclamation / leaking (retire-and-never-free)` | done | leak baseline — step 2 `Leak` |
| 46 | CORE | `Per-object lock-free reference counting` | done | built and **proven unsound**; the proof is the deliverable |
| 51 | CORE | `Hazard pointers (Michael)` | port C5 |  |
| 56 | CORE | `protect-and-validate (the HP/HE read protocol)` | port C5 | the announce/validate protocol itself |
| 61 | CORE | `folly hazptr_obj_base_linked (link counting)` | port C10 | link counting |
| 66 | CORE | `HP++ (Jung, Lee, Kim, Kang)` | defer | **HP++** — the research alternative to C10 for optimistic traversal. Measurably slower than HP, needs a per-node invalidate bit and an extended unlink API. We take folly's link counting instead; revisit if C10's reader cost turns out non-zero |
| 71 | CORE | `Epoch-based reclamation (EBR)` | port C6 |  |
| 76 | CORE | `crossbeam-epoch (Rust EBR)` | port C6 | the mirror target |
| 81 | CORE | `QSBR (quiescent-state-based reclamation)` | family out | QSBR — needs a quiescent state the clients do not have |
| 86 | CORE | `RCU (read-copy-update)` | family out | RCU — raised and declined on merits (see the plan's *RCU, specifically*): same ERA cell as epoch, read side unmeasurable without `membarrier`, unbounded garbage and blocking progress. `synchronize_rcu` survives as C4's `cleanup()`. Triggers: a read-mostly pointer with a named consumer, or a move to Linux |
| 91 | CORE | `Hazard eras (HE)` | family out | hazard eras |
| 96 | CORE | `Interval-based reclamation (IBR / 2GEIBR)` | family out | IBR / 2GEIBR |
| 101 | CORE | `Hyaline / Hyaline-1 / Hyaline-S / Hyaline-1S (reserve-to-free)` | family out | Hyaline |
| 106 | CORE | `VBR (version-based reclamation)` | family out | VBR — the interesting one: drops requirement (A) entirely, paying with a type-preserving allocator and a version word per mutable field |
| 111 | CORE | `crossbeam-queue SegQueue per-slot WRITE/READ/DESTROY state bits` | port 5b | per-slot WRITE/READ/DESTROY — outside the trait on purpose |
| 116 | CORE | `The ERA theorem (Sheffi & Petrank impossibility result)` | **ADD** docs | **the ERA theorem.** No scheme gets Ease of integration + Robustness + Applicability — at most two. EBR/RCU take integration+applicability, HP/HE take integration+robustness, VBR/NBR take robustness+applicability. This is the one-paragraph answer to *why does each scheme have exactly one glaring weakness*, and it belongs in step 1's docs. 0 h |
| 123 | IMPORTANT | `Split / deferred reference counting (DRC, update coalescing, immediate RC)` | family out | split / deferred refcounting |
| 128 | IMPORTANT | `folly::atomic_shared_ptr` | family out | `folly::atomic_shared_ptr` |
| 133 | IMPORTANT | `arc_swap 'debt' slots (Rust ArcSwap)` | family out | `arc_swap` debt slots — the Rust answer for the read-mostly case, and RCU's first revisit trigger. Its own artifact if it lands, not a `Reclaim` implementor |
| 138 | IMPORTANT | `Pass-the-buck (Herlihy, Luchangco, Martin, Moir)` | family out | **pass-the-buck** — the ancestor of Hyaline's reserve-to-free; named here because the old out-of-scope list omitted it |
| 143 | IMPORTANT | `Asymmetric thread fence (membarrier-based; folly asymmetric_thread_fence_light/heavy)` | port C5 | asymmetric fence interface; `membarrier` impl skipped |
| 148 | IMPORTANT | `folly hazptr_obj_cohort` | port C11 | cohorts |
| 153 | IMPORTANT | `DEBRA (distributed epoch-based reclamation)` | family out | DEBRA |
| 158 | IMPORTANT | `DEBRA+ (fault-tolerant EBR via neutralizing signals)` | family out | DEBRA+ |
| 163 | IMPORTANT | `NBR / NBR+ (neutralization-based reclamation)` | family out | NBR / NBR+ |
| 168 | IMPORTANT | `Wait-free eras (WFE)` | family out | wait-free eras |
| 173 | IMPORTANT | `Crystalline-L / Crystalline-LW / Crystalline-W` | family out | Crystalline |
| 178 | IMPORTANT | `Optimistic Access / Automatic Optimistic Access / Free Access (Cohen & Petrank)` | family out | optimistic access / free access |
| 183 | IMPORTANT | `Tagged pointers / ABA stamps + free-list reuse` | decision | **tagged pointers + free-list reuse.** Not an omission — a different answer, and per the row "still the right answer in bounded, fixed-type systems (and in kernels/HFT where you preallocate anyway)". `bufpool` is where this lives in this tree. Recognising that a version tag is VBR in miniature collapses two literatures |
| 190 | OPTIONAL | `folly CoreCachedSharedPtr / ReadMostlySharedPtr` | family out | CoreCached / ReadMostlySharedPtr |
| 195 | OPTIONAL | `SCOT / immutability-based HP validation (PLDI 2025)` | family out | SCOT |
| 200 | OPTIONAL | `Publish-on-ping / EpochPOP` | family out | publish-on-ping |
| 205 | OPTIONAL | `Type-preserving allocator / page-remapping lock-free allocator (Moreno & Rocha 2023)` | family out | type-preserving allocator |
| 210 | OPTIONAL | `Stamp-it` | family out | stamp-it |
| 215 | OPTIONAL | `QSense` | family out | QSense |
| 220 | OPTIONAL | `Drop the Anchor (DTA)` | family out | drop-the-anchor |
| 225 | OPTIONAL | `ThreadScan / ForkScan / StackTrack (conservative stack scanning via OS or HTM)` | family out | ThreadScan / ForkScan / StackTrack |
| 230 | OPTIONAL | `OrcGC / FreeAccess (lock-free tracing collectors for lock-free structures)` | family out | OrcGC / FreeAccess |
| 235 | OPTIONAL | `PEBR (pointer-and-epoch-based reclamation)` | family out | PEBR |
| 240 | OPTIONAL | `Expediting hazard pointers with bounded RCU critical sections (Kim, Jung, Kang 2024)` | family out | bounded-RCU HP expediting |
| 247 | SKIP | `Conditional Access (hardware/software co-design)` | family out | conditional access — hardware co-design |

## folly `hazptr` — 73 rows

| Line | Bucket | Mechanism | Disposition | Note |
|---:|---|---|---|---|
| 312 | CORE | `hazptr_rec (Atom<const void*> hazptr_, alignas(hardware_destructive_interference_size))` | port C3/C5 | the slot; padding is row 1101 |
| 317 | CORE | `hazptr_rec::reset_hazptr(const void* p = nullptr)` | port C5 | `reset_hazptr` — one entry point for begin / end / **retarget**, which is shape B |
| 322 | CORE | `hazptr_holder<Atom> (RAII, move-only, nullable hprec_)` | port C5 | RAII guard, move-only, nullable |
| 327 | CORE | `hazptr_holder::try_protect(T*& ptr, const Atom<T*>& src) — load / publish / full fence / re-load / compare` | port C1/C5 | `try_protect` — the required method |
| 332 | CORE | `hazptr_holder::protect(src) retry loop` | port C1 | `protect` — the **provided default** |
| 337 | CORE | `reset_protection(ptr) / reset_protection(nullptr)` | port C5 | retarget / clear |
| 342 | CORE | `make_hazard_pointer(domain) free function (and hazard_pointer alias)` | port C4 | `domain.guard()` |
| 347 | CORE | `hazptr_domain<Atom>` | port C4 |  |
| 352 | CORE | `load_hazptr_vals() → folly::F14FastSet<const void*>` | port C5 | hashed guarded set — found absent in the first audit |
| 357 | CORE | `hazptr_obj<Atom> base: ReclaimFnPtr reclaim_, Obj* next_, uintptr_t cohort_tag_` | port C2/C11 | `Retired` + `RetireLink` + cohort tag word |
| 362 | CORE | `next_(this) sentinel + pre_retire_check() / pre_retire_check_fail()` | **ADD** C2 | **double-retire detection.** `next = self` sentinel at construction; `retire` asserts it. The inventory's own suggested ladder puts this in stage 1. Turns `# Safety` clause 2 from an unchecked promise into a panic. Corroborated by 1096. **+1 h** |
| 367 | CORE | `hazptr_obj_base<T, Atom, D>::retire(D deleter, hazptr_domain& domain) + set_reclaim()` | port C1/C2 | `Retire` trait + retire path |
| 372 | CORE | `hazptr_obj_list<Atom> (head, tail, count)` | port C2 | `(head, tail, count)` batch, spliced in one operation |
| 377 | CORE | `hazptr_detail::linked_list<Node>` | port C2 | non-atomic list component |
| 382 | CORE | `hazptr_detail::shared_head_only_list<Node, Atom> with kLockBit in head_, owner_ thread id, reentrance_` | port C2 | lock-free push, wait-free `pop_all` via exchange. Lock bit + reentrance only with tagged lists → conditional |
| 387 | CORE | `count_ as Atom<int> (signed, 'may transiently be negative') with add_count / exchange_count / cas_count` | port C4 | **signed** count |
| 392 | CORE | `threshold() = std::max(kThreshold /*1000*/, kMultiplier /*2*/ * hcount_)` | **ADD** C4 | `threshold() = max(1000, 2 × hcount)`. The **proportionality to hazard count** is the actual proof that unreclaimed objects are O(hazard pointers) — hazptr's headline advantage over RCU. The plan said "a threshold"; the formula is the claim. **0 h**, acceptance-test wording |
| 397 | CORE | `check_threshold_and_reclaim() / check_count_threshold()` | **ADD** C4 | count trigger first (cheap load), then time trigger (clock read, slow path); **whoever CASes the count to 0 owns the round**, so exactly one thread runs it. **0 h**, acceptance wording |
| 402 | CORE | `do_reclamation(int rcount) outer while(true) loop with the `done` flag` | **ADD** C5 | the `done` re-check loop — a round re-checks the threshold and repeats, so it never returns leaving work it could have done. **0 h** |
| 407 | CORE | `The heavy fence placement in do_reclamation (asymmetric_thread_fence_heavy(seq_cst) after extract, before load_hazptr_vals)` | port C5 | **heavy fence placement**: after extracting the retired lists, before snapshotting hazards. Name the placement in the acceptance test, not just the fence |
| 414 | IMPORTANT | `try_protect(ptr, src, Func f) — pointer-word filter` | **ADD** C1/C5 | **pointer-word filter.** Announcing a *tagged* word never matches the retired object's real address, so a marked node could be freed while protected — a soundness bug, not an optimisation. Needed the moment a client puts a mark bit in a pointer. CORE in the haphazard section (1059). **+1 h** |
| 419 | IMPORTANT | `hprecs_ as atomic_grow_array<Rec, HazptrRecArrayPolicy> + hcount_` | port C3 | grow-only, reference-stable, indexable rec array + published count |
| 424 | IMPORTANT | `avail_ : Atom<uintptr_t> free list of recs with kLockBit = 1 (tagged-pointer lock) + detail::Sleeper` | port C3 | `avail_` free list, tagged-pointer lock bit |
| 429 | IMPORTANT | `acquire_hprecs(num) / release_hprec / release_hprecs(head, tail) + Rec::next_avail()/set_next_avail()` | defer | bulk acquire/release — travels with the thread cache |
| 434 | IMPORTANT | `create_new_hprec() with hcount_.fetch_add(1, relaxed) and its happens-before argument` | port C3 | **name the happens-before argument in the acceptance test**: a new rec that a scan misses means its protected object is freed under a live reader. Row 437 calls it the subtlest ordering argument in the library |
| 439 | IMPORTANT | `Relaxed slot loads + one trailing acquire fence in load_hazptr_vals, with kIsSanitizeThread ? acquire : relaxed` | **ADD** C5 | relaxed slot loads + one trailing acquire fence instead of N acquire loads. **+0.5 h** |
| 444 | IMPORTANT | `asymmetric_thread_fence_light / asymmetric_thread_fence_heavy (P1202r4)` | port C5 | light/heavy pair — interface only |
| 449 | IMPORTANT | `hazptr_detail::shared_head_tail_list<Node, Atom>` | port C11 | shared head+tail list component |
| 454 | IMPORTANT | `kNumShards = 8 sharded untagged_[] and tagged_[] + calc_shard (std::hash(ftag) >> kIgnoredLowBits & kShardMask)` | port C2 | 8 shards, address-hashed, low 8 bits discarded |
| 459 | IMPORTANT | `kSyncTimePeriod = 2000000000 ns, due_time_, check_due_time(), set_due_time()` | port C4 | time trigger — found absent in this audit |
| 464 | IMPORTANT | `asymmetric_thread_fence_light(seq_cst) in push_list (the retire path)` | port C5 | light fence on the **retire** path: the unlink must not be observed after the retired-list push |
| 469 | IMPORTANT | `match_reclaim_untagged(untagged[], hs, done) and the ObjList& children return channel` | port C5/C10 | match-and-reclaim, plus the **children return channel** — reclaim can produce more objects (link counting) |
| 474 | IMPORTANT | `num_bulk_reclaims_ + inc/dec + wait_for_zero_bulk_reclaims() + domain::cleanup()` | **ADD** C4 | **`cleanup()` with bulk-reclaim quiescence.** Guarantees everything currently reclaimable is gone when it returns, including batches a concurrent round already popped out. This is what makes a zero-leak assertion meaningful, and the plan's "a dropped domain asserts its retired list is empty" test cannot be written honestly without it. Corroborated by 1126. **+2 h** |
| 479 | IMPORTANT | `schedule_reclamation / exec_fn_ / set_executor / clear_executor / enable_hazptr_thread_pool_executor() / FLAGS_folly_hazptr_use_executor` | port C4 | offload executor |
| 484 | IMPORTANT | `~hazptr_domain: shutdown_ flag, reclaim_all_objects(), reclaim_list_transitive(), reclaim_unconditional(), free_hazptr_recs()` | **ADD** C4 | **unconditional teardown.** At domain death there are by contract no readers, so the hazard check is pointless work; and the default domain's recs are **deliberately leaked** because a thread cache can outlive the global domain and freeing recs it still points at is a UAF during process exit. **+2 h** |
| 489 | IMPORTANT | `hazptr_tc<Atom> thread cache + hazptr_tc_entry + hazptr_tc_tls() via SingletonThreadLocal` | defer | thread cache — shape B makes acquisition ~1/op; step 4 measures it |
| 494 | IMPORTANT | `hazptr_tc::try_get / try_put / fill(num) / evict(num) / evict(), kCapacity = 16` | defer | with the thread cache |
| 499 | IMPORTANT | `hazptr_array<M, Atom> + make_hazard_pointer_array<M>() + aligned_hazptr_holder (aligned_storage_for_t)` | defer | `hazptr_array<M>` — growable guards cover the need |
| 504 | IMPORTANT | `hazptr_obj_cohort<Atom> + set_cohort_tag / set_cohort_no_tag / cohort() / tagged() with kTagBit = 1 stolen from the cohort pointer` | port C11 |  |
| 509 | IMPORTANT | `cohort kThreshold = 20 + check_threshold_push() + pushed_to_domain_tagged_` | port C2 | batch at 20 |
| 514 | IMPORTANT | `cohort active_ / clear_active() / shutdown_and_reclaim() / ~hazptr_obj_cohort` | port C11 | `active_` + `shutdown_and_reclaim` |
| 519 | IMPORTANT | `hazptr_obj_linked<Atom>: single Atom<uint64_t> count_ packing kLink = 1<<32 and kRef = 1 with kLinkMask / kRefMask` | port C10 | packed {link\|ref} |
| 524 | IMPORTANT | `release_link / release_ref / downgrade_link and the off-by-one ref-count convention` | port C10 | release/downgrade + off-by-one convention |
| 529 | IMPORTANT | `hazptr_obj_base_linked<T, Atom, D>: retire() vs unlink() vs unlink_and_reclaim_unchecked(), and the for_each_link(bool m, F&& f) contract` | port C10 | `retire` vs `unlink`, and the `for_each_link` aliasing rule |
| 534 | IMPORTANT | `The Atom template-template parameter threaded through every class` | port C1 | the `Atom` template param **is** `src/sync.rs` |
| 539 | IMPORTANT | `default_hazptr_domain() via detail::createGlobal<impl, void> plus extern hazptr_domain<std::atomic> default_domain with FOLLY_STATIC_CTOR_PRIORITY_MAX` | port C4 | `global()` with controlled construction order |
| 546 | OPTIONAL | `try_protect(ptr, Src&& src) — callable source overload` | conditional | callable-source overload — see 1064 |
| 551 | OPTIONAL | `is_default_domain_ with its own alignas(hardware_destructive_interference_size)` | skip | `is_default_domain_` fast path — thread-cache machinery |
| 556 | OPTIONAL | `Chunked hazptr load loop in load_hazptr_vals (constexpr size_t chunk_width = kNumShards; const void* ptrs[chunk_width])` | experiment | chunked hazard load loop — a step-4 before/after, not scope |
| 561 | OPTIONAL | `asymmetric_thread_fence_heavy_fn::impl_ via sysMembarrierPrivateExpedited(), cached by sysMembarrierAvailableCached()` | skip | `membarrier` impl — unavailable on macOS/aarch64 |
| 566 | OPTIONAL | `detail::hazptr_prefer_fence_light = kIsArchAArch64 && kIsLinux && !kIsSanitizeThread` | skip | `prefer_fence_light` — Linux/aarch64 only |
| 571 | OPTIONAL | `hazptr_deleter<T, D> with specialization for std::default_delete<T>` | decision | `hazptr_deleter<T, D>` superseded by `Retire::reclaim` on the object's trait |
| 576 | OPTIONAL | `hazptr_domain::retire(T* obj, D reclaim) — nonintrusive, allocating` | skip | non-intrusive allocating retire. The escape hatch if a consumer cannot embed a `RetireLink`; none of the four named consumers needs it |
| 581 | OPTIONAL | `list_walk_sharded + FOLLY_BUILTIN_PREFETCH(next, 0, 2)` | experiment | sharded walk + prefetch — step-4 before/after |
| 586 | OPTIONAL | `extract_retired_objects() lock dance (check_lock() → pop_all(kAlsoLock) → push_unlock(empty) if nothing found)` | conditional | extract lock dance — with tagged lists |
| 591 | OPTIONAL | `match_tagged(tagged[], hs) with per-shard run batching into cohorts[s] / safe[s]` | conditional | `match_tagged` — with tagged lists |
| 596 | OPTIONAL | `detail::hazptr_inline_executor_add — thread_local std::queue<Function<void()>>` | port C4 | inline recursion flattener |
| 601 | OPTIONAL | `'Tagged objects remain' warning in ~hazptr_domain (kIsDebug && !tagged_empty()) and kListTooLarge = 100000 / hazptr_warning_list_too_large` | **ADD** C4 | **diagnostics**: `kListTooLarge = 100000` warning, and an executor-backlog warning above 10 queued rounds. Cheap, and they are how unbounded growth announces itself instead of being discovered as an OOM. Corroborated by 1173. **+0.5 h** |
| 606 | OPTIONAL | `hazptr_tc decay mechanism: kDecayThreshold = 1024, put_tick_, window_start_, decay(), shrink_to(), try_put_slow()` | defer | tc decay — with the thread cache |
| 611 | OPTIONAL | `hazptr_tc::local_ / local() / set_local() (kIsDebug only)` | skip | tc debug flag |
| 616 | OPTIONAL | `hazptr_local<M, Atom>` | skip | `hazptr_local<M>` — non-composable for 3 ns |
| 621 | OPTIONAL | `cohort safe_list_top_ + push_safe_objs() + reclaim_safe_list()` | conditional | cohort safe list — with tagged lists |
| 626 | OPTIONAL | `hazptr_domain::cleanup_cohort_tag(cohort) + list_match_tag / list_match_condition` | conditional | `cleanup_cohort_tag` — with tagged lists |
| 631 | OPTIONAL | `acquire_link / acquire_ref versus acquire_link_safe / acquire_ref_safe` | port C10 | `acquire_link` vs `acquire_link_safe` |
| 636 | OPTIONAL | `downgrade_retire_immutable_descendants / release_delete_immutable_descendants / release_retire_mutable_children + Worklist = small_vector<..., 2>` | port C10 | one-pass reclamation of long immutable chains — a 10k chain must not take 10k rounds |
| 641 | OPTIONAL | `hazptr_root<T, Atom>` | port C10 | `hazptr_root<T>` — a holder for a link from a static root. Declared roots landed, so this is its natural pairing: a declared root that owns a link |
| 646 | OPTIONAL | `detail::Sleeper (bounded spin then kMinYieldingSleep)` | port C3 | back-off — reuse `concurrent::Backoff` |
| 651 | OPTIONAL | `FOLLY_ALWAYS_INLINE / FOLLY_NOINLINE / FOLLY_LIKELY / FOLLY_UNLIKELY discipline` | skip | inline-attribute discipline — revisit only if a profile asks |
| 658 | SKIP | `mprotectMembarrier() TLB-shootdown fallback` | skip | `mprotect` TLB-shootdown fallback |
| 663 | SKIP | `FOLLY_HAZPTR_THR_LOCAL macro (false when FOLLY_MOBILE)` | skip | compile-out-the-thread-cache macro |
| 668 | SKIP | `hazard_pointer / hazard_pointer_domain / hazard_pointer_obj_base / hazard_pointer_default_domain / hazard_pointer_clean_up aliases` | skip | WG21 P1121 name aliases |
| 673 | SKIP | `hazptr_obj_retired_list<Atom> (with check_threshold_try_zero_count)` | skip | retired-list wrapper type |
| 678 | SKIP | `delete_hazard_pointers() / hazptr_tc_evict() / hazptr_tc_tester` | experiment | `delete_hazard_pointers` / tc tester — benchmark-only; relevant if step 4 wants a cold-start number |

## `crossbeam-epoch` — 47 rows

| Line | Bucket | Mechanism | Disposition | Note |
|---:|---|---|---|---|
| 711 | CORE | `Collector` | port C4 | `Collector` ≈ `Domain<Epoch>` |
| 716 | CORE | `Global (global epoch + participant list + garbage queue)` | port C6 | global epoch + participant list + garbage queue |
| 721 | CORE | `LocalHandle` | port C6 | `LocalHandle` |
| 726 | CORE | `Local (the participant record)` | port C6 | `Local` participant record |
| 731 | CORE | `Participant registration (`Local::register`)` | port C3/C6 | registration |
| 736 | CORE | `guard_count — nesting / re-entrancy` | port C6 | `guard_count` re-entrancy |
| 741 | CORE | `Guard` | port C6 | `Guard` as proof of pinned-ness |
| 746 | CORE | `pin() fast path (relaxed load of global, store local = global\|PINNED, SeqCst fence)` | port C6 | pin fast path: relaxed load, store `global\|PINNED`, SeqCst fence |
| 751 | CORE | `unpin (`Local::unpin`)` | port C6 | unpin |
| 756 | CORE | `Epoch representation: LSB = pinned flag, rest = wrapping counter` | port C6 | LSB = pinned flag, rest = wrapping counter |
| 761 | CORE | `The three-epoch rule (`SealedBag::is_expired`: `global.wrapping_sub(seal) >= 2`)` | port C6 | **the three-epoch rule** — build it with two first and let loom produce the counterexample |
| 766 | CORE | ``Global::try_advance`` | port C6 | `try_advance` |
| 771 | CORE | ``Global::collect` + `COLLECT_STEPS = 8`` | port C6 | `collect` + `COLLECT_STEPS = 8` |
| 776 | CORE | `Bag + `MAX_OBJECTS = 64`` | port C6 | `Bag` + `MAX_OBJECTS = 64` |
| 781 | CORE | `SealedBag (epoch, Bag) + `unsafe impl Sync`` | port C6 | `SealedBag` |
| 786 | CORE | ``Global::push_bag` and its leading `fence(SeqCst)`` | port C6 | `push_bag` and its leading SeqCst fence |
| 791 | CORE | `defer / defer_unchecked / defer_destroy` | port C6 | `defer` / `defer_unchecked` / `defer_destroy` |
| 796 | CORE | ``Atomic<T: ?Sized + Pointable>`` | port C7 | `Atomic<T>` |
| 801 | CORE | ``Owned<T>`` | port C7 | `Owned<T>` |
| 806 | CORE | ``Shared<'g, T>` and its lifetime` | port C7 | `Shared<'g, T>` |
| 811 | CORE | `Pointer tagging (`low_bits`, `compose_tag`, `decompose_tag`, `ensure_aligned`, `map_addr`)` | port C7 | pointer tagging |
| 816 | CORE | ``compare_exchange` / `compare_exchange_weak` + `CompareExchangeError`` | port C7 | the CAS family |
| 823 | IMPORTANT | `handle_count + `Local::finalize` (self-hosted participant reclamation)` | decision | `handle_count` + `Local::finalize` — **not ported, and this is a deliberate divergence**: C3 chose *immortal* records (folly's shape), so participants are never reclaimed and the self-hosted reclamation of `Local` has nothing to do. Simpler, at the cost of never returning a dead thread's record memory |
| 828 | IMPORTANT | `repin / repin_after` | **ADD** C6 | **`repin` / `repin_after`.** A long reader pins one epoch and blocks *all* reclamation in the domain for as long as it runs. `repin` republishes the current epoch so a long traversal yields a grace period instead of starving the collector. A queue consumer does not need it; a skiplist traversal does. **+1.5 h** |
| 833 | IMPORTANT | ``pin_count` + `PINNINGS_BETWEEN_COLLECT = 128` (the epoch-advance policy)` | **ADD** C6 | `pin_count` + `PINNINGS_BETWEEN_COLLECT = 128` — the **advance policy**. The plan had `try_advance` and `collect` but never said *when* to try. **+0.5 h** |
| 838 | IMPORTANT | ``Deferred` — inline small-closure optimisation` | port C6 | `Deferred` — closures of ≤ 3 words stored inline |
| 843 | IMPORTANT | ``Guard::flush`` | **ADD** C6 | **`Guard::flush`.** Push a partial bag early. This is epoch's answer to the same failure the time trigger fixes for HP: a thread holding a half-full bag that goes quiet. Worth seeing that the two schemes need the same fix in different shapes. **+0.5 h** |
| 848 | IMPORTANT | ``unprotected()`` | **ADD** C6/C7 | **`unprotected()`** — a `&'static Guard` with a null local, for single-threaded construction and teardown paths. Registration itself needs it. **+0.5 h** |
| 853 | IMPORTANT | ``fetch_and` / `fetch_or` / `fetch_xor` on tags` | **ADD** C7 | **`fetch_and` / `fetch_or` / `fetch_xor` on tags.** Harris's logical delete *is* `fetch_or(1)`. The plan listed the `compare_exchange` family and omitted these, so C9 would have had nothing to mark with. **+1 h** |
| 858 | IMPORTANT | ``sync::list` — intrusive lock-free participant registry` | port C3 | intrusive participant registry — this **is** C3, and the old not-ported list disclaimed it in error |
| 863 | IMPORTANT | ``Entry::delete` — logical delete via `fetch_or(1, Release)` + lazy physical unlink` | decision | `Entry::delete` logical-delete-then-lazy-unlink — not needed: C3's records are immortal |
| 868 | IMPORTANT | ``IterError::Stalled` and the restart-from-head protocol` | decision | `IterError::Stalled` restart-from-head — same reason; only arises when records can be deleted |
| 873 | IMPORTANT | ``sync::queue` — Michael-Scott queue of SealedBags, with `try_pop_if`` | port C4/C6 | MS queue of sealed bags — also disclaimed in error |
| 878 | IMPORTANT | ``default.rs` — the process-wide default collector` | port C4 | process-wide default collector → `global()` |
| 883 | IMPORTANT | ``CachePadded` on the global epoch and each local epoch` | port C3/C6 | `CachePadded` on the global epoch and each local epoch |
| 888 | IMPORTANT | `loom / Miri test shims (`mod primitive`, `UnsafeCell` wrapper, reduced constants)` | port C1 | loom / Miri shims — `src/sync.rs` |
| 893 | IMPORTANT | ``unsafe impl` audit trail (`Bag: Send`, `SealedBag: Sync`, `Collector: Send + Sync`, `Atomic: Send + Sync where T: Send + Sync`)` | **ADD** C1 | the **`unsafe impl` audit trail**: every place the crate asserts what the compiler cannot see carries its reasoning inline. A house rule to state once in C1 and then keep. **0 h** |
| 900 | OPTIONAL | `x86 `lock cmpxchg`-instead-of-`mfence` hack in pin()` | skip | x86 `lock cmpxchg`-for-`mfence` — this machine is aarch64 |
| 905 | OPTIONAL | `AtomicEpoch` | port C6 | `AtomicEpoch` typed wrapper |
| 910 | OPTIONAL | ``Deferred::NO_OP`` | port C6 | `Deferred::NO_OP` |
| 915 | OPTIONAL | ``fetch_update`` | skip | `fetch_update` convenience |
| 920 | OPTIONAL | ``load_consume` (crossbeam-utils `AtomicConsume`)` | skip | `load_consume` — real on aarch64, but **disabled under Miri, loom and TSan**; an optimisation none of this project's verification tools can see |
| 925 | OPTIONAL | ``Pointable` trait (`ALIGN`, `Init`, `init`, `as_ptr`, `as_mut_ptr`, `drop`)` | port C7 | `Pointable` |
| 930 | OPTIONAL | ``IsElement` trait + `#[repr(C)]` with `entry` first` | decision | `IsElement` + `repr(C)` entry-first — not needed with immortal records |
| 937 | SKIP | ``[MaybeUninit<T>]` Pointable impl / `Array<T> { len, elements: [_; 0] }`` | **FIX** C7 | `[MaybeUninit<T>]` `Pointable` impl. The inventory marks it **SKIP**; **C7's acceptance test requires it**, justified as "which is what the bags need" — and that justification is **wrong**: `Bag` is `[Deferred; 64]`, a fixed array needing no `Pointable`. The real consumer is a variable-length allocation — the skiplist's tower. Keep the mechanism, fix the reason, and note that it is not needed until the skiplist. **0 h** |
| 942 | SKIP | ``crossbeam_sanitize` / `crossbeam_sanitize_thread` knobs` | skip | TSan knobs |
| 947 | SKIP | ``alloc_helper` / `no_std` + `target_has_atomic` cfg gating` | skip | `no_std` gating |

## What `haphazard` did not port — 31 rows

| Line | Bucket | Mechanism | Disposition | Note |
|---:|---|---|---|---|
| 1039 | CORE | `hazptr_tc — thread-local cache of hazard-pointer records` | defer | thread cache. Marked CORE *as a haphazard gap*; our defer rests on shape B making acquisition ~1/op, and step 4 measures it rather than assuming |
| 1044 | CORE | `Real asymmetric barriers (membarrier MEMBARRIER_CMD_PRIVATE_EXPEDITED) for light/heavy fence pair` | skip | real `membarrier` barriers — platform |
| 1049 | CORE | `Empty hazard-pointer state + empty()` | **ADD** C1/C5 | **the empty state.** P1121 distinguishes a guard that owns *no* record from one owning an **unassociated** record. This is the open question "what does `as_ref` do on a guard that announces nothing" — and it is CORE, not a detail. Three states, not two. **+0.5 h** |
| 1054 | CORE | `swap(hazard_pointer&, hazard_pointer&) — hand-over-hand traversal` | **ADD** C1/C5 | **`swap(a, b)`.** Swap record ownership between two guards without ending either protection period — *the* way P1121's own example advances a two-pointer traversal. This also **settles the deferred return-type question**: a `&'a T` borrowed from `&'a mut self` cannot coexist with a swap, because the swap needs `&mut` while the reference is live. Raw pointer, or no hand-over-hand. **+0.5 h** |
| 1059 | CORE | `try_protect with a pointer-bit filter function (tagged/marked pointers)` | **ADD** C1/C5 | pointer-bit filter — CORE here, same as folly 414 |
| 1064 | CORE | `try_protect over an arbitrary validating source, not just &AtomicPtr<T>` | conditional | validating source ≠ protected source. Trigger: a client that must validate against a different location than it protects. None does yet |
| 1069 | CORE | `Batched retire (hazptr_obj_list / hazptr_domain_push_retired)` | port C2 | batched retire |
| 1076 | IMPORTANT | `hazptr_prefer_fence_light + relaxed hazard store behind a light release fence` | skip | `prefer_fence_light` — Linux only |
| 1081 | IMPORTANT | `Per-object runtime deleter (hazard_pointer_obj_base<T,D>::retire(D d) / hazptr_retire(obj, reclaim))` | decision | per-object **runtime** deleter. `Retire::reclaim` is type-level, which is enough only if the object carries whatever state its reclaim needs (a pool pointer, an arena handle). State that constraint in the trait docs |
| 1086 | IMPORTANT | `hazptr_obj_cohort + tagged retired lists + cleanup_cohort_tag + shutdown_and_reclaim (synchronous reclamation)` | port C11 | cohorts + synchronous reclamation |
| 1091 | IMPORTANT | `hazptr_obj_linked / hazptr_obj_base_linked / hazptr_root — link and ref counting` | port C10 | link and ref counting |
| 1096 | IMPORTANT | `Double-retire detection` | **ADD** C2 | double-retire detection — same as folly 362 |
| 1101 | IMPORTANT | `Cache-line-aligned hazard records` | **ADD** C3 | **cache-line-aligned records.** Readers on different cores false-sharing each other's hazard slot defeats the whole point. **+0.5 h** |
| 1106 | IMPORTANT | `Retired-list shards actually spread across cache lines` | **ADD** C2 | **shard heads on separate cache lines.** "If the shard heads share a line, sharding buys nothing." The shards were just added; this is the thing that makes them work. **0 h** if done with them |
| 1111 | IMPORTANT | `Contiguous grow-array of records + chunked batched scan (load_hazptr_vals)` | port C3 | contiguous grow-array (chunked scan is 556, an experiment) |
| 1116 | IMPORTANT | `Fast flat hash set for the guarded set instead of BTreeSet` | port C5 | hashed guarded set |
| 1121 | IMPORTANT | `compare_exchange returning the actual observed current value on failure` | port C1 | CAS returning the observed value on failure — this is exactly what `try_protect`'s `Err` payload does; the design already agrees |
| 1126 | IMPORTANT | `hazard_pointer_clean_up with the "deleter completion synchronizes-with return" guarantee` | **ADD** C4 | synchronizing `cleanup()` — same as folly 474 |
| 1131 | IMPORTANT | `Sound unique-domain construction (haphazard's unique_domain! soundness hole)` | decision | sound unique-domain construction. haphazard's `unique_domain!` can mint two domains sharing one family (its open issue #54). We avoid type-level families entirely: the structure **stores** its domain, so cross-domain retire is prevented by construction |
| 1136 | IMPORTANT | `Specified semantics for re-protecting without reset` | **ADD** C1 | **specified semantics for re-protecting without reset.** What happens when a guard protects a second pointer while still protecting a first — which under shape B is the *normal* path, not an edge case. Must be written, not implied. **0 h** |
| 1143 | OPTIONAL | `No public "is this pointer protected?" query (the guarded-pointer scan is private to do_reclamation)` | decision | no public "is this pointer protected?" query. Already argued in the 5b section: epoch has no per-pointer information and could not answer it |
| 1148 | OPTIONAL | `Reclamation offload / retire-without-reclaim (set_executor, hazptr_use_executor, exec_backlog warnings)` | port C4 | reclamation offload |
| 1153 | OPTIONAL | `Domain-level allocator (pmr::polymorphic_allocator)` | skip | domain-level allocator |
| 1158 | OPTIONAL | `delete_hazard_pointers / shrinking the record pool` | skip | shrinking the record pool — records are immortal |
| 1163 | OPTIONAL | `hazptr_local<M> — stack-local fast holder array` | skip | `hazptr_local<M>` |
| 1168 | OPTIONAL | `Time-based reclamation trigger on no_std` | port C4 | time trigger |
| 1173 | OPTIONAL | `Diagnostics: list-too-large and executor-backlog warnings` | **ADD** C4 | diagnostics — same as folly 601 |
| 1178 | OPTIONAL | `fetch_or-based available-list lock instead of CAS + yield spin` | experiment | `fetch_or`-based avail-list lock |
| 1185 | SKIP | `is_default_domain() fast path` | skip | `is_default_domain` fast path |
| 1190 | SKIP | `The documentation that was never written (empty "Differences from the specification" / "Differences from the folly" sections)` | skip | haphazard's own missing docs — not a mechanism |
| 1195 | SKIP | `ABA-prevention framing` | decision | ABA framing. Hazard pointers also solve ABA, since an address cannot be recycled while a hazard pointer names it. Worth one line in step 1's docs |

