# Rebuild Ladder — spaced re-implementation of core unsafe/concurrency Builds

> **What this is.** A third spaced system (alongside Paper Drills G9 + Concept Cadence G10):
> re-implementing hard atomics/unsafe artifacts **from a blank file** to convert *understand-when-shown*
> into *produce-from-blank*. Scheduled onto a weekly **Sunday 🔁 Rebuild Day** so you never track it yourself.

> **Selection.** IN = APEX (memory ordering / lock-free / CAS / loom / custom sync) + CORE (unsafe / raw-ptr /
> Layout / vtable / manual Send-Sync / Pin / variance). OUT = pure logic, parsing, encoding, CLI, glue.

> **Schedule (harder→forgotten sooner→more reps).** APEX 4 reps at +3/+7/+13/+25 weeks after build; CORE 3 reps at +4/+9/+18.
> Each rep: cold from blank, oracle = the artifact's own test suite, Miri gate for concurrency ones, peek-log to `notes/rebuild_<item>.md`.

**44 artifacts** (33 APEX, 11 CORE). ⚠ = not standalone-cold-buildable (rebuild after its dep).

| # | Source | Artifact | Tier | R1 | R2 | R3 | R4 | Notes |
|---|--------|----------|------|----|----|----|----|-------|
| 1 | W003·Fri | rate_limit: TokenBucket Future + RateLimitedStream<br>`eth-network-codec/src/rate_limit.rs` | CORE | W010 | W012 | W023 |  | RETRO; 2-field !Unpin pin-projection |
| 2 | W004·Sat | AtomicCell<T> (+Pod hardening)<br>`concurrent/src/atomic_cell.rs` | APEX | W007 | W013 | W023 | W034 | RETRO; transmute fast-path + spinlock fallback |
| 3 | W004·Thu | BytesMut growable + freeze<br>`eth-primitives/src/bytes_mut.rs` | CORE | W012 | W013 | W028 |  | RETRO; feeds W005 Bytes |
| 4 | W004·Thu | Parker / Unparker (lost-wakeup-safe)<br>`concurrent/src/parker.rs` | APEX | W008 | W014 | W023 | W035 | RETRO; the 'parker' primitive |
| 5 | W004·Tue | ChainHead SeqLock<br>`eth-chain-state/src/chain_head.rs` | APEX | W009 | W015 | W024 | W035 | RETRO |
| 6 | W005·Fri | Bytes zero-copy vtable redesign<br>`eth-primitives/src/bytes.rs` | APEX | W011 | W016 | W027 | W035 | RETRO (already built) |
| 7 | W011·Thu | Sharded credit flow-control<br>`concurrent/src/credit_flow.rs` | APEX | W017 | W024 | W029 | W041 |  |
| 8 | W011·Tue | Vyukov MPMC bounded channel<br>`concurrent/src/channel/bounded.rs` | APEX | W018 | W027 | W030 | W041 |  |
| 9 | W012·Tue | Claim-mode SP/MP ring split<br>`concurrent/src/channel/bounded.rs` | APEX | W019 | W028 | W031 | W042 | extends MPMC; distinct (asm CAS-elision + PhantomData variance) |
| 10 | W013·Mon | Signed-int RwLock (+writer-fairness)<br>`concurrent/src/rwlock.rs` | APEX | W020 | W029 | W032 | W042 | folds W013-Wed fairness ext |
| 11 | W014·Fri | Pin-count CAS-to-sentinel eviction<br>`bufpool/src/pin.rs` | APEX | W021 | W030 | W033 | W049 |  |
| 12 | W014·Mon | Hazard-pointer Treiber stack<br>`concurrent/src/treiber.rs (+hazard.rs)` | APEX | W022 | W031 | W034 | W049 |  |
| 13 | W015·Mon | Tagged-pointer ABA demo<br>`concurrent/src/aba_demo.rs` | APEX | W025 | W032 | W034 | W049 |  |
| 14 | W015·Wed | Counter-wrap ABA (tag sizing)<br>`concurrent/src/aba_wrap.rs` | APEX | W026 | W033 | W035 | W053 | related to ABA demo; distinct sub-problem |
| 15 | W026·Mon | mmap WAL Segment (fetch_add reserve)<br>`wal/src/segment.rs` | APEX | W036 | W041 | W043 | W059 |  |
| 16 | W026·Tue | SegQueue unbounded MPMC<br>`concurrent/src/queue/seg_queue.rs` | APEX | W037 | W042 | W044 | W062 |  |
| 17 | W026·Tue | ⚠WAL group-commit appender<br>`wal/src/group_commit.rs` | APEX | W038 | W043 | W048 | W063 | dep on SegQueue (rebuild after) |
| 18 | W027·Wed | MmapPageProvider (stripe-locked mmap)<br>`storage-trie/src/mdbx/mmap_page_provider.rs` | CORE | W039 | W039 | W053 |  |  |
| 19 | W030·Fri | mmap-queue CycleFile arena<br>`mmap-queue/src/cycle.rs` | APEX | W040 | W048 | W054 | W063 |  |
| 20 | W030·Mon | ⚠UndoRecordPool arena allocator<br>`recovery/src/undo.rs` | CORE | W044 | W049 | W059 |  | embedded sub-component of undo.rs |
| 21 | W033·Fri | epoch-gc Collector::advance (+W037 hardened)<br>`epoch-gc/src/internal.rs` | APEX | W045 | W053 | W055 | W064 | folds W037-Mon global::try_advance milestone |
| 22 | W033·Thu | epoch-gc Guard (pin/defer_destroy)<br>`epoch-gc/src/guard.rs` | APEX | W046 | W054 | W056 | W064 |  |
| 23 | W033·Wed | epoch-gc Atomic/Owned/Shared<br>`epoch-gc/src/atomic.rs` | APEX | W047 | W055 | W057 | W064 |  |
| 24 | W037·Fri | skiplist delete (mark-then-unlink)<br>`concurrent/src/skiplist/delete.rs` | APEX | W050 | W057 | W060 | W066 |  |
| 25 | W037·Thu | skiplist insert (bottom-up CAS link)<br>`concurrent/src/skiplist/insert.rs` | APEX | W051 | W058 | W061 | W066 |  |
| 26 | W037·Tue | skiplist Node (inline var-len tower)<br>`concurrent/src/skiplist/node.rs` | CORE | W056 | W058 | W063 |  |  |
| 27 | W037·Wed | skiplist find (help-unlink)<br>`concurrent/src/skiplist/find.rs` | APEX | W052 | W059 | W062 | W067 |  |
| 28 | W042·Tue | ⚠Txn lifecycle (atomic SM + slab recycle)<br>`txn/src/lifecycle.rs` | CORE | W060 | W062 | W064 |  | dep on SegQueue freelist + Hlc; unsafe refcount core standalone |
| 29 | W043·Wed | LocalExecutor (RawWakerVTable)<br>`runtime-thread-per-core/src/scheduler.rs` | CORE | W061 | W063 | W064 |  |  |
| 30 | W057·Fri | cross-shard MPMC channel (NxN)<br>`runtime-thread-per-core/src/cross_shard.rs` | APEX | W065 | W067 | W070 | W084 |  |
| 31 | W057·Sun | Sharded<T> (Seastar; unsafe impl Sync)<br>`runtime-thread-per-core/src/sharded.rs` | CORE | W066 | W068 | W076 |  |  |
| 32 | W072·Tue | 2PC Coordinator/Participant<br>`txn/src/two_phase_commit.rs` | APEX | W075 | W080 | W085 | W099 |  |
| 33 | W076·Wed | MediaDriver CnC SPSC rings<br>`messaging-aeron/src/media_driver.rs` | APEX | W079 | W085 | W089 | W101 |  |
| 34 | W077·Mon | Pretoucher (mmap page pre-faulter)<br>`mmap-queue/src/pretouch.rs` | CORE | W084 | W086 | W096 |  | folds W078-Thu pretoucher twin (same technique) |
| 35 | W077·Mon | TermBuffer (FAA-claim rotating ring)<br>`messaging-aeron/src/term_buffer.rs` | APEX | W081 | W086 | W090 | W102 |  |
| 36 | W077·Tue | FlowController (sliding-window)<br>`messaging-aeron/src/flow_control.rs` | APEX | W082 | W087 | W091 | W102 |  |
| 37 | W078·Fri | Chronicle multi-writer queue<br>`mmap-queue/src/multi_writer.rs` | APEX | W083 | W089 | W092 | W103 |  |
| 38 | W082·Tue | Image (monotonic cursor + loss detect)<br>`messaging-aeron/src/image.rs` | APEX | W088 | W090 | W096 | W107 |  |
| 39 | W085·Fri | ⚠epoll edge-triggered recvmmsg<br>`marketdata-kernelbypass/src/epoll.rs` | CORE | W091 | W094 | W103 |  | needs Transport trait + parse.rs to run |
| 40 | W089·Mon | static-mem pool + robin-hood map<br>`ledger-deterministic/src/static_mem/transfer_pool.rs (+account_map.rs, +fixed_map.rs)` | CORE | W093 | W099 | W107 |  | folds W089-Tue fixed_map (near-dup) |
| 41 | W092·Thu | versioned MVCC memory<br>`exec-vm/src/parallel/versioned_memory.rs` | APEX | W095 | W100 | W105 | W117 |  |
| 42 | W093·Mon | block-stm lock-free scheduler+executor<br>`exec-vm/src/parallel/scheduler.rs (+executor.rs)` | APEX | W097 | W101 | W106 | W118 |  |
| 43 | W094·Tue | process-lifetime worker pool (barrier)<br>`exec-vm/src/parallel/pool.rs` | APEX | W098 | W102 | W107 | W119 |  |
| 44 | W111·Wed | seqlock applied-state risk read<br>`perp-dex-core/src/risk_read.rs` | APEX | W114 | W118 | W124 | W136 |  |

## Sunday load check (after load-leveling)

- Each Sunday = one full **5-hour** work day (weekly summary/retro dropped — Sunday is *pure rebuild*).
- Hours per rep: APEX R1/R2/R3/R4 ≈ 5/3/2/1h (a hard R1 like Bytes fills the whole day); CORE R1/R2/R3 ≈ 3/2/1h.
- Peak after leveling: **5h** in a single Sunday; max push from ideal week: **+15 weeks**.
- Full (5h) Sundays: 71. **1 Sunday/week suffices** (429h total load across 99 of 139 available Sundays).
- Total rebuild-sessions: **165**.
