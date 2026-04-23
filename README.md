# Reth Contributor — 24-Month Daily Plan

> **Start**: 2026-04-25
> **Horizon**: 24 months, reassess at Month 24
> **Commitment**: 5h/day × 6 days/week = 30h/week
> **Schedule**: Mon-Sat work, Sunday rest + weekly ritual

**Deliverables**:
- `storage-trie` crate (Month 7-12) — reth storage + trie re-implementation
- `exec-vm` crate (Month 13-18) — revm + reth evm re-implementation
- `consensus-engine` crate (Month 19-24) — reth consensus + engine API re-implementation

---

## How to Use

Check off tasks as completed. One day = one section. If you fall behind, adjust forward — don't delete the plan. Sunday ritual reviews the week and recalibrates.

**Daily 5h block structure**:
- 15 min warm-up (review notes, set intent)
- 90 min deep work 1
- 10 min break
- 90 min deep work 2
- 15 min wrap-up (commit, log, questions)

---

# PHASE 1: RUST FOUNDATION (Month 1-3)

## Month 1: Core Rust Syntax + Ownership

### Week 1 — Ownership fundamentals

**Pre-week setup (Sunday before, 1-2h, not counted)**:
- [X] Install rustup, verify `cargo --version`, `rustc --version`
- [X] Install VSCode + rust-analyzer OR nvim + rust-analyzer LSP
- [X] Install `cargo install cargo-nextest cargo-expand cargo-watch`
- [X] Create private GitHub repo `rust-reth-journey` with `progress.md`, `questions.md`, `notes/` folder
- [X] Clone Rustlings: `cargo install rustlings`
- [X] Create Twitter account, follow: paradigmxyz, gakonst, mattsse_, dan_cline, DragonRakita
- [X] Follow paradigmxyz, bluealloy/revm, alloy-rs, foundry-rs on GitHub

**Monday — Book ch1-2, Rustlings intro**
- [ ] Rust Book Chapter 1 (Getting Started)
- [ ] Rust Book Chapter 2 (Guessing Game)
- [ ] Rustlings `intro` (3 exercises)
- [ ] Rustlings `variables` (6 exercises)
- [ ] Rustlings `functions` (5 exercises)
- [ ] Build guessing game from scratch without looking at book
- [ ] Commit to repo with message "Day 1"
- [ ] Log in progress.md

**Tuesday — Book ch3, Rustlings control flow**
- [ ] Rust Book Chapter 3 (Common Programming Concepts)
- [ ] Rustlings `if` (3 exercises)
- [ ] Rustlings `primitive_types` (6 exercises)
- [ ] Mini exercise: implement Fibonacci (iterative + recursive)
- [ ] Mini exercise: temperature converter C↔F
- [ ] Mini exercise: simple calculator with match on operator
- [ ] Note: expression vs statement in notes/
- [ ] Commit + log

**Wednesday — Book ch4.1-4.2 (Ownership & References)**
- [ ] Rust Book Chapter 4.1 (What is Ownership) — read 2x
- [ ] Rust Book Chapter 4.2 (References and Borrowing)
- [ ] Rustlings `move_semantics` (6 exercises — do all)
- [ ] Draw diagram: stack vs heap with String example in notes/
- [ ] Write 5 small programs forcing compiler errors to learn borrow checker
- [ ] Document 3 rules of ownership in own words
- [ ] Commit + log

**Thursday — Book ch4.3 + ch5 (Slices, Structs)**
- [ ] Rust Book Chapter 4.3 (Slice Type)
- [ ] Rust Book Chapter 5.1 (Defining Structs)
- [ ] Rust Book Chapter 5.2 (Example Program)
- [ ] Rustlings `strings` (4 exercises)
- [ ] Rustlings `structs` (3 exercises)
- [ ] Exercise: String vs &str in function signatures
- [ ] Exercise: implement `Point` struct with methods
- [ ] Commit + log

**Friday — Book ch5.3 + ch6 (Methods, Enums)**
- [ ] Rust Book Chapter 5.3 (Method Syntax)
- [ ] Rust Book Chapter 6.1 (Defining an Enum)
- [ ] Rust Book Chapter 6.2 (match Control Flow)
- [ ] Rustlings `enums` (3 exercises)
- [ ] Rustlings `vecs` (2 exercises)
- [ ] Exercise: implement `Shape` enum with area() method via match
- [ ] Exercise: implement Option-like enum from scratch
- [ ] Commit + log

**Saturday — Book ch6.3 + ch7 (if let, Modules)**
- [ ] Rust Book Chapter 6.3 (if let)
- [ ] Rust Book Chapter 7 (Modules, Packages, Crates)
- [ ] Rustlings `modules` (3 exercises)
- [ ] Rustlings `hashmaps` (3 exercises)
- [ ] Exercise: reorganize previous exercises into multi-module crate
- [ ] Practice `pub`, `use`, re-exports
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual (1h)**
- [ ] Review all notes from the week
- [ ] Dedup and close questions.md items
- [ ] Update progress.md with weekly summary
- [ ] Honest assessment: "Did I understand ownership or just pass exercises?"
- [ ] If ownership unclear, add catch-up slot to Week 2

---

### Week 2 — Error handling, generics, traits intro

**Monday — Book ch8 (Collections) + ch9.1 (panic)**
- [ ] Rust Book Chapter 8 (Common Collections) — Vec, String, HashMap in depth
- [ ] Rust Book Chapter 9.1 (Unrecoverable Errors with panic!)
- [ ] Rustlings `options` (3 exercises)
- [ ] Exercise: implement `Stack<T>` using Vec
- [ ] Exercise: word frequency counter with HashMap
- [ ] Commit + log

**Tuesday — Book ch9.2-9.3 (Error Handling)**
- [ ] Rust Book Chapter 9.2 (Recoverable Errors with Result)
- [ ] Rust Book Chapter 9.3 (To panic! or Not)
- [ ] Rustlings `error_handling` (6 exercises)
- [ ] Read `thiserror` crate docs
- [ ] Exercise: rewrite calculator with Result and custom error enum using thiserror
- [ ] Practice `?` operator in chains
- [ ] Commit + log

**Wednesday — Book ch10.1 (Generics)**
- [ ] Rust Book Chapter 10.1 (Generic Data Types)
- [ ] Rustlings `generics` (2 exercises)
- [ ] Exercise: generic `largest<T: PartialOrd>` function
- [ ] Exercise: generic `Pair<T, U>` struct
- [ ] Exercise: generic `Container<T>` with Vec backing
- [ ] Note: monomorphization concept in notes/
- [ ] Commit + log

**Thursday — Book ch10.2 (Traits)**
- [ ] Rust Book Chapter 10.2 (Traits)
- [ ] Rustlings `traits` (5 exercises)
- [ ] Exercise: implement `Summary` trait for 3 types
- [ ] Exercise: trait bounds with multiple constraints
- [ ] Exercise: blanket implementation pattern
- [ ] Note: static vs dynamic dispatch in notes/
- [ ] Commit + log

**Friday — Book ch10.3 (Lifetimes)**
- [ ] Rust Book Chapter 10.3 (Lifetimes)
- [ ] Rustlings `lifetimes` (3 exercises)
- [ ] Watch Crust of Rust: Lifetime Annotations (first 30 min)
- [ ] Exercise: `longest<'a>` function
- [ ] Exercise: struct with lifetime parameter
- [ ] Note: lifetime elision rules in notes/
- [ ] Commit + log

**Saturday — Crust of Rust Lifetimes full + consolidation**
- [ ] Watch Crust of Rust: Lifetime Annotations completely
- [ ] Code along with Jon Gjengset's `StrSplit` implementation
- [ ] Review week's notes
- [ ] Start Project scaffold: `xcsv` CLI tool (args parsing only, no logic yet)
- [ ] Read `clap` crate docs
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] Review notes
- [ ] Close questions
- [ ] Update progress.md
- [ ] Assessment: "Can I explain lifetimes without looking up?"

---

### Week 3 — Testing, closures, iterators

**Monday — Book ch11 (Writing Tests)**
- [ ] Rust Book Chapter 11 (Writing Automated Tests)
- [ ] Rustlings `tests` (3 exercises)
- [ ] Setup `cargo-nextest` for xcsv project
- [ ] Add unit tests to existing exercises
- [ ] Practice `#[cfg(test)]` module pattern
- [ ] Commit + log

**Tuesday — Book ch12 (CLI project)**
- [ ] Rust Book Chapter 12 (I/O Project minigrep)
- [ ] Build minigrep-like but for CSV: `xcsv` — read CSV, filter by column
- [ ] Use `clap` for arg parsing
- [ ] Use `thiserror` for error types
- [ ] Add integration tests
- [ ] Commit + log

**Wednesday — Book ch13.1 (Closures)**
- [ ] Rust Book Chapter 13.1 (Closures)
- [ ] Exercise: `map`, `filter`, `fold` using closures
- [ ] Exercise: closure capturing environment
- [ ] Understand FnOnce, FnMut, Fn trait differences
- [ ] Apply closures in xcsv for custom filters
- [ ] Commit + log

**Thursday — Book ch13.2 (Iterators)**
- [ ] Rust Book Chapter 13.2 (Iterators)
- [ ] Rustlings `iterators` (5 exercises)
- [ ] Watch Crust of Rust: Iterators
- [ ] Exercise: implement custom Iterator for counter type
- [ ] Exercise: chain iterator adaptors for data processing
- [ ] Commit + log

**Friday — Iterator deep dive**
- [ ] Code along Jon Gjengset iterator video — implement StrSplit Iterator
- [ ] Exercise: implement `flatten()` from scratch
- [ ] Refactor xcsv to use iterator chains where possible
- [ ] Add benchmarks comparing loop vs iterator (expect similar perf)
- [ ] Commit + log

**Saturday — Project day: finish xcsv**
- [ ] Complete xcsv CLI with clap, thiserror, iterator chains
- [ ] Add streaming parser for large files (read line by line)
- [ ] Write README documenting usage
- [ ] Ensure 80%+ test coverage
- [ ] Tag as v0.1.0 in repo
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 4 — Smart pointers, interior mutability

**Monday — Book ch15.1-15.2 (Box, Deref)**
- [ ] Rust Book Chapter 15.1 (Box)
- [ ] Rust Book Chapter 15.2 (Deref Trait)
- [ ] Exercise: implement `MyBox<T>` with Deref
- [ ] Exercise: recursive types using Box (linked list, binary tree)
- [ ] Commit + log

**Tuesday — Book ch15.3-15.4 (Drop, Rc)**
- [ ] Rust Book Chapter 15.3 (Drop Trait)
- [ ] Rust Book Chapter 15.4 (Rc)
- [ ] Rustlings `smart_pointers` (2 exercises)
- [ ] Exercise: shared state with Rc
- [ ] Exercise: tree where nodes share children via Rc
- [ ] Commit + log

**Wednesday — Book ch15.5-15.6 (RefCell, reference cycles)**
- [ ] Rust Book Chapter 15.5 (RefCell and Interior Mutability)
- [ ] Rust Book Chapter 15.6 (Reference Cycles, Weak)
- [ ] Watch Crust of Rust: Smart Pointers and Interior Mutability
- [ ] Exercise: `Rc<RefCell<T>>` pattern
- [ ] Exercise: parent-child tree with Weak for upward refs
- [ ] Commit + log

**Thursday — Book ch16.1-16.2 (Threads, Channels)**
- [ ] Rust Book Chapter 16.1 (Threads)
- [ ] Rust Book Chapter 16.2 (Message Passing with Channels)
- [ ] Exercise: spawn thread, join handle
- [ ] Exercise: producer-consumer with mpsc channel
- [ ] Commit + log

**Friday — Book ch16.3 (Shared State) + Crust of Rust Channels**
- [ ] Rust Book Chapter 16.3 (Shared State, Mutex)
- [ ] Watch Crust of Rust: Channels
- [ ] Code along: implement MPSC channel from scratch
- [ ] Exercise: `Arc<Mutex<T>>` counter shared across threads
- [ ] Understand Send/Sync marker traits
- [ ] Commit + log

**Saturday — Book ch16.4 + Start Project 2 (shardkv)**
- [ ] Rust Book Chapter 16.4 (Send and Sync)
- [ ] Create new crate `shardkv`
- [ ] Define `KVStore<K, V>` trait with get/set/delete
- [ ] Implement `HashMapStore<K, V>` with Mutex
- [ ] Write basic tests
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**
- [ ] End of Month 1 review: honest check against Phase 1 trajectory
- [ ] Update North Star M1 metrics

---

## Month 2: Advanced Rust + Async

### Week 5 — Complete sync patterns, start OOP-ish features

**Monday — Book ch17 (OOP features)**
- [ ] Rust Book Chapter 17 (Object-Oriented Programming Features)
- [ ] Understand trait objects vs generics tradeoffs
- [ ] Exercise: dyn Trait vs impl Trait in function signatures
- [ ] Commit + log

**Tuesday — shardkv: sharding**
- [ ] Implement sharded storage: N independent hashmaps with hash-based routing
- [ ] Benchmark: single Mutex<HashMap> vs sharded (use criterion)
- [ ] Read `ArcSwap` crate docs
- [ ] Commit + log

**Wednesday — shardkv: RwLock variant**
- [ ] Add `RwLockStore` implementation
- [ ] Benchmark Mutex vs RwLock under varied read/write ratios
- [ ] Document findings in notes/
- [ ] Commit + log

**Thursday — shardkv: eviction trait**
- [ ] Define `EvictionPolicy` trait
- [ ] Implement LRU eviction
- [ ] Implement TTL eviction
- [ ] Make KVStore generic over eviction
- [ ] Commit + log

**Friday — shardkv: polish**
- [ ] Add proper error types with thiserror
- [ ] Add tracing instrumentation
- [ ] Full test coverage including concurrent access tests
- [ ] README documenting design choices
- [ ] Tag v0.1.0
- [ ] Commit + log

**Saturday — Start Tokio tutorial**
- [ ] Read Tokio tutorial: Hello Tokio, Spawning
- [ ] Setup tokio in a new scratch project
- [ ] Exercise: concurrent HTTP fetcher with tokio
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 6 — Tokio fundamentals

**Monday — Tokio: Shared State, Channels**
- [ ] Read Tokio tutorial: Shared State
- [ ] Read Tokio tutorial: Channels (tokio mpsc)
- [ ] Exercise: shared state counter across async tasks
- [ ] Exercise: broadcast channel for pub/sub
- [ ] Commit + log

**Tuesday — Tokio: I/O, Framing**
- [ ] Read Tokio tutorial: I/O
- [ ] Read Tokio tutorial: Framing
- [ ] Exercise: TCP echo server (from scratch, no tutorial copy)
- [ ] Handle multiple connections concurrently
- [ ] Commit + log

**Wednesday — Async Book ch1-4**
- [ ] Async Book Chapter 1-4 (futures, async/await, pinning)
- [ ] Understand Future trait
- [ ] Understand Pin at high level
- [ ] Commit + log

**Thursday — Async Book ch5-7 + Rustlings threads**
- [ ] Async Book Chapter 5-7 (executors, streams, TLS async)
- [ ] Rustlings `threads` (3 exercises, sync not async)
- [ ] Exercise: manual Future implementation (counter that resolves after N polls)
- [ ] Commit + log

**Friday — Async Book ch8-9 + async ecosystem**
- [ ] Async Book Chapter 8-9 (TLS, async gotchas)
- [ ] Read docs: `tracing`, `tracing-subscriber`
- [ ] Read docs: `reqwest` for async HTTP client
- [ ] Exercise: async HTTP client with tracing instrumentation
- [ ] Commit + log

**Saturday — Start Project 3 (backpressure-net)**
- [ ] Create crate `backpressure-net`
- [ ] Define TCP server framework with tokio
- [ ] Support graceful shutdown via SIGTERM/SIGINT
- [ ] Basic connection handler trait
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 7 — Advanced async, Pin/Unpin

**Monday — Crust of Rust: Async/Await**
- [ ] Watch Crust of Rust: Async/Await (full)
- [ ] Code along — implement simple executor
- [ ] Commit + log

**Tuesday — Pin/Unpin deep**
- [ ] Watch Crust of Rust: The Drop Check
- [ ] Read std::pin docs carefully
- [ ] Exercise: self-referential struct demonstrating need for Pin
- [ ] Write explanation of Pin/Unpin in notes/ in own words
- [ ] Commit + log

**Wednesday — backpressure-net: rate limiting**
- [ ] Add per-connection rate limiting with token bucket
- [ ] Implement token bucket as custom Future
- [ ] Test under load with mock clients
- [ ] Commit + log

**Thursday — backpressure-net: backpressure strategies**
- [ ] Define `BackpressureStrategy` enum: DropOldest, DropNewest, Block
- [ ] Implement all three
- [ ] Test each under queue overflow
- [ ] Commit + log

**Friday — backpressure-net: observability**
- [ ] Add tracing spans for connection lifecycle
- [ ] Add Prometheus-compatible metrics (use `metrics` crate)
- [ ] Expose /metrics endpoint
- [ ] Commit + log

**Saturday — backpressure-net: polish + load test**
- [ ] Load test with 10,000 concurrent connections
- [ ] Document findings
- [ ] README with usage examples
- [ ] Tag v0.1.0
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 8 — Rustonomicon intro, unsafe basics

**Monday — Rustonomicon ch1 (Meet Safe and Unsafe)**
- [ ] Rustonomicon Chapter 1 (Meet Safe and Unsafe)
- [ ] Understand unsafe superpowers
- [ ] Understand undefined behavior concept
- [ ] Commit + log

**Tuesday — Rustonomicon ch2 (Data Layout)**
- [ ] Rustonomicon Chapter 2 (Data Layout)
- [ ] `repr(C)`, `repr(transparent)`, alignment
- [ ] Exercise: inspect layout with `std::mem::size_of` and `align_of`
- [ ] Commit + log

**Wednesday — Rustonomicon ch3 (Ownership intro for unsafe)**
- [ ] Rustonomicon Chapter 3 selected sections
- [ ] Read about lifetimes in unsafe context
- [ ] Exercise: small unsafe block with raw pointer usage
- [ ] Commit + log

**Thursday — Crust of Rust: Atomics and Memory Ordering**
- [ ] Watch Crust of Rust: Atomics and Memory Ordering
- [ ] Read std::sync::atomic docs
- [ ] Review your own disruptor Ryuo code with fresh eyes
- [ ] Exercise: implement spinlock with AtomicBool
- [ ] Commit + log

**Friday — Crust of Rust: Subtyping and Variance**
- [ ] Watch Crust of Rust: Subtyping and Variance
- [ ] Understand PhantomData use cases
- [ ] Exercise: invariant marker in struct
- [ ] Commit + log

**Saturday — Consolidation + start Rust for Rustaceans**
- [ ] Begin Rust for Rustaceans ch1-2
- [ ] Note takeaways in notes/
- [ ] Review all Month 2 learnings
- [ ] Commit + log

**Sunday — Rest + End Month 2 review**
- [ ] Update North Star M2 metrics (should still be mostly 0s, that's expected)
- [ ] Honest assessment of Rust capability

---

## Month 3: Rust Mastery + Alloy Exposure

### Week 9 — Rust for Rustaceans core chapters

**Monday — Rust for Rustaceans ch1-3**
- [ ] Rust for Rustaceans Chapter 1 (Foundations)
- [ ] Rust for Rustaceans Chapter 2 (Types)
- [ ] Rust for Rustaceans Chapter 3 (Designing Interfaces)
- [ ] Apply learnings to refactor xcsv or shardkv
- [ ] Commit + log

**Tuesday — Rust for Rustaceans ch4-5**
- [ ] Rust for Rustaceans Chapter 4 (Error Handling)
- [ ] Rust for Rustaceans Chapter 5 (Project Structure)
- [ ] Refactor error types across your projects to match best practices
- [ ] Commit + log

**Wednesday — Rust for Rustaceans ch6-7**
- [ ] Rust for Rustaceans Chapter 6 (Testing)
- [ ] Rust for Rustaceans Chapter 7 (Macros)
- [ ] Exercise: write a declarative macro
- [ ] Commit + log

**Thursday — Rust for Rustaceans ch8-9**
- [ ] Rust for Rustaceans Chapter 8 (Async)
- [ ] Rust for Rustaceans Chapter 9 (Unsafe Code)
- [ ] Cross-reference with Async Book and Rustonomicon
- [ ] Commit + log

**Friday — Rust for Rustaceans ch10-11**
- [ ] Rust for Rustaceans Chapter 10 (Concurrency)
- [ ] Rust for Rustaceans Chapter 11 (FFI)
- [ ] Note FFI takeaways for future C library bindings (MDBX)
- [ ] Commit + log

**Saturday — Rust for Rustaceans ch12 + consolidation**
- [ ] Rust for Rustaceans final chapter
- [ ] Review book entirely via notes
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 10 — Start reading production Rust (Alloy)

**Monday — Alloy exploration**
- [ ] Clone alloy-rs/alloy repo
- [ ] Read top-level README
- [ ] Browse crate structure in Cargo.toml
- [ ] Read alloy-primitives docs on docs.rs
- [ ] Commit notes

**Tuesday — Alloy primitives deep read**
- [ ] Read alloy-primitives source: Address, U256, B256, Bytes types
- [ ] Note patterns: type newtype wrappers, From/TryFrom implementations
- [ ] Exercise: create own small crate using alloy-primitives
- [ ] Commit + log

**Wednesday — Alloy provider**
- [ ] Read alloy-provider docs
- [ ] Read source of Provider trait
- [ ] Exercise: write program fetching latest mainnet block via public RPC
- [ ] Commit + log

**Thursday — Alloy RPC types**
- [ ] Read alloy-rpc-types source
- [ ] Note serde patterns used extensively
- [ ] Exercise: parse and manipulate Transaction types
- [ ] Commit + log

**Friday — Alloy signer + wallet**
- [ ] Read alloy-signer docs and source
- [ ] Understand EIP-191, EIP-712 signing
- [ ] Exercise: sign a message, recover signer
- [ ] Commit + log

**Saturday — Mini-project using Alloy**
- [ ] Build mini-Etherscan CLI: input address, output balance, nonce, recent txs
- [ ] Use Alloy for all Ethereum interaction
- [ ] ~500 LOC
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 11 — Jon Gjengset advanced videos

**Monday — Crust of Rust: Iterators deep**
- [ ] Rewatch Iterators video if needed
- [ ] Implement more iterator adaptors from scratch
- [ ] Commit + log

**Tuesday — Crust of Rust: Channels deep**
- [ ] Rewatch Channels, implement bounded channel from scratch
- [ ] Add flavor: rendezvous channel (capacity 0)
- [ ] Commit + log

**Wednesday — Custom Future implementation**
- [ ] Implement timer Future from scratch using std::task primitives
- [ ] Integrate with tokio runtime
- [ ] Commit + log

**Thursday — State machine async**
- [ ] Build small async state machine: simple consensus round or leader election heartbeat
- [ ] Use custom Futures + tokio
- [ ] Property tests
- [ ] Commit + log

**Friday — Advanced trait patterns**
- [ ] Practice extension traits
- [ ] Practice sealed trait pattern
- [ ] Practice type-state pattern
- [ ] Apply to refactor one existing project
- [ ] Commit + log

**Saturday — Rustlings finish**
- [ ] Complete remaining Rustlings sections
- [ ] Verify 100+ exercises completed
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 12 — Phase 1 consolidation + Phase 2 prep

**Monday — Revm passive exposure**
- [ ] Clone bluealloy/revm
- [ ] Browse structure
- [ ] Read top-level docs (don't deep dive yet, just familiarize)
- [ ] Commit notes

**Tuesday — Reth passive exposure**
- [ ] Clone paradigmxyz/reth
- [ ] `cargo build --release` (30-60 min first time)
- [ ] Browse structure
- [ ] Run `reth --help`
- [ ] Commit notes

**Wednesday — Read Mastering Ethereum prep**
- [ ] Start Mastering Ethereum Chapter 1-2 (intro)
- [ ] Start reading ethereum.org developer intro
- [ ] Commit notes

**Thursday — Twitter warm-up**
- [ ] First thoughtful reply to a reth/paradigm tweet
- [ ] First technical tweet (not marketing)
- [ ] Follow 20 more Ethereum infra engineers
- [ ] Log presence activities

**Friday — GitHub presence warm-up**
- [ ] Star key repos: reth, revm, alloy, foundry, ethers-rs, erigon (reference)
- [ ] Watch reth repo for notifications
- [ ] Read 5 recent merged PRs in reth to see style
- [ ] Commit notes

**Saturday — Phase 1 review**
- [ ] Review all 3 projects: xcsv, shardkv, backpressure-net
- [ ] Write tagged releases v0.1.0 if not already
- [ ] Verify all exit criteria for Phase 1
- [ ] Document Phase 1 reflection in progress.md
- [ ] Commit + log

**Sunday — End Phase 1 ritual (1-2h)**
- [ ] Full Phase 1 assessment against exit criteria
- [ ] Update North Star M3 metrics
- [ ] Celebrate completion
- [ ] Prep for Phase 2 start tomorrow

---

# PHASE 2: ETHEREUM FOUNDATION + ECOSYSTEM PRs (Month 4-6)

## Month 4: Ethereum Protocol + Alloy PRs

### Week 13 — Ethereum fundamentals

**Monday — Ethereum developer docs + Mastering Ethereum ch3**
- [ ] Ethereum.org developer docs: "Intro to Ethereum"
- [ ] Mastering Ethereum Chapter 3 (Ethereum Clients)
- [ ] Setup: run `reth` node on Sepolia testnet
- [ ] Observe sync logs
- [ ] Commit notes

**Tuesday — Mastering Ethereum ch4 (Cryptography)**
- [ ] Mastering Ethereum Chapter 4 (Cryptography)
- [ ] Understand keccak256, secp256k1
- [ ] Exercise: implement simple keypair generator using k256 crate
- [ ] Commit + log

**Wednesday — Mastering Ethereum ch5 (Wallets) + ch6 (Transactions)**
- [ ] Mastering Ethereum Chapter 5 (Wallets)
- [ ] Mastering Ethereum Chapter 6 (Transactions)
- [ ] Understand legacy vs EIP-1559 vs EIP-4844 tx types
- [ ] Exercise: build and sign each tx type using Alloy
- [ ] Commit + log

**Thursday — Mastering Ethereum ch7 (Smart Contracts)**
- [ ] Mastering Ethereum Chapter 7 (Smart Contracts Solidity)
- [ ] Deploy simple contract on Sepolia using Foundry
- [ ] Interact with it using Alloy
- [ ] Commit + log

**Friday — Yellow Paper: World State section**
- [ ] Read Ethereum Yellow Paper Section 4 (Block, State, Account)
- [ ] Note concepts: state trie, account trie, storage trie
- [ ] Start drawing Ethereum state diagrams in notes/
- [ ] Commit + log

**Saturday — Yellow Paper: Transactions section**
- [ ] Read Yellow Paper Section 6 (Transaction Execution)
- [ ] Understand intrinsic gas, execution gas
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 14 — EIPs deep dive + Alloy PR hunt

**Monday — EIP-1559**
- [ ] Read EIP-1559 full specification
- [ ] Understand base fee, priority fee mechanics
- [ ] Read Paradigm's analysis blog posts on 1559
- [ ] Commit notes

**Tuesday — EIP-4844 (blobs)**
- [ ] Read EIP-4844 full specification
- [ ] Understand blob transactions, KZG commitments at high level
- [ ] Read Proto-Danksharding roadmap
- [ ] Commit notes

**Wednesday — EIP-7702 (account abstraction)**
- [ ] Read EIP-7702 full specification
- [ ] Understand authorization list mechanics
- [ ] Commit notes

**Thursday — Alloy issues scan**
- [ ] Browse alloy-rs/alloy issues
- [ ] Filter `good first issue`, `help wanted`, `docs`
- [ ] Identify 3-5 candidate issues
- [ ] Pick one, comment claiming
- [ ] Commit notes

**Friday — First Alloy PR work**
- [ ] Read CONTRIBUTING.md carefully
- [ ] Fork alloy repo
- [ ] Create branch with convention
- [ ] Start implementation for chosen issue
- [ ] Commit + log

**Saturday — First Alloy PR complete**
- [ ] Finish implementation
- [ ] Run `cargo fmt`, `cargo clippy --all`, `cargo nextest`
- [ ] Write clear PR description with context
- [ ] Open PR
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 15 — More Alloy PRs + EOF EIPs

**Monday — Respond to Alloy PR reviews**
- [ ] Address any reviewer feedback on first PR
- [ ] Learn from suggestions
- [ ] Iterate until merge or close
- [ ] Commit + log

**Tuesday — EIP-7685 (execution layer requests)**
- [ ] Read EIP-7685 specification
- [ ] Understand request/receipt mechanism
- [ ] Commit notes

**Wednesday — EOF EIPs batch**
- [ ] Read EIP-3540 (EVM Object Format)
- [ ] Read EIP-3670 (Code Validation)
- [ ] Read EIP-4200 (Static Relative Jumps)
- [ ] Read EIP-4750 (Functions)
- [ ] Note EOF implications for EVM implementation
- [ ] Commit notes

**Thursday — Second Alloy PR**
- [ ] Pick next candidate issue
- [ ] Implement
- [ ] Commit + log

**Friday — Third Alloy PR work**
- [ ] Pick and work on medium-difficulty issue
- [ ] Aim for substantive contribution not just typo fix
- [ ] Commit + log

**Saturday — Third Alloy PR complete**
- [ ] Finish, submit
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 16 — Alloy PR velocity + Foundry intro

**Monday — Fourth Alloy PR**
- [ ] Pick and implement
- [ ] Commit + log

**Tuesday — Foundry codebase intro**
- [ ] Clone foundry-rs/foundry
- [ ] Read Foundry Book (book.getfoundry.sh) for user perspective
- [ ] Browse forge crate source
- [ ] Commit notes

**Wednesday — Foundry cast crate**
- [ ] Read cast crate source
- [ ] Understand CLI tool structure
- [ ] Commit notes

**Thursday — First Foundry PR**
- [ ] Browse Foundry issues, pick good first
- [ ] Implement
- [ ] Commit + log

**Friday — Foundry PR complete + respond to reviews**
- [ ] Finish Foundry PR
- [ ] Address any outstanding Alloy review feedback
- [ ] Commit + log

**Saturday — Fifth Alloy PR or consolidation**
- [ ] Either pick one more Alloy PR or polish existing ones
- [ ] Commit + log

**Sunday — Rest + End Month 4 review**
- [ ] Update North Star M4 metrics
- [ ] Target check: 5+ Alloy PRs opened, some merged

---

## Month 5: EVM Deep Dive + revm PRs

### Week 17 — Mastering Ethereum ch13 (EVM)

**Monday — Mastering Ethereum ch13 part 1**
- [ ] Mastering Ethereum Chapter 13 (EVM) first half
- [ ] Understand EVM as stack machine
- [ ] Memorize top 20 opcodes from evm.codes
- [ ] Commit notes

**Tuesday — Mastering Ethereum ch13 part 2**
- [ ] Mastering Ethereum Chapter 13 (EVM) second half
- [ ] Understand gas metering basics
- [ ] Understand storage vs memory vs stack
- [ ] Commit notes

**Wednesday — evm.codes deep practice**
- [ ] Go through every opcode on evm.codes
- [ ] Practice reading bytecode
- [ ] Exercise: manually trace simple contract execution
- [ ] Commit notes

**Thursday — Tiny EVM exercise start**
- [ ] Start toy EVM implementation: stack + 5 arithmetic opcodes only
- [ ] This is throwaway learning — NOT production
- [ ] Write interpreter loop
- [ ] Commit + log

**Friday — Tiny EVM: more opcodes**
- [ ] Add memory operations (MSTORE, MLOAD)
- [ ] Add control flow (JUMP, JUMPI, JUMPDEST)
- [ ] Add comparison/logic opcodes
- [ ] Commit + log

**Saturday — Tiny EVM: wrap up**
- [ ] 15-20 opcodes total
- [ ] Pass a few manual test cases
- [ ] Discard without publishing (learning artifact only)
- [ ] Commit notes on learnings
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 18 — revm exploration

**Monday — revm overview**
- [ ] Clone bluealloy/revm
- [ ] Read README, architecture doc
- [ ] Browse crate structure
- [ ] Commit notes

**Tuesday — revm-primitives**
- [ ] Read revm-primitives source
- [ ] Compare with alloy-primitives
- [ ] Note Database, Host traits
- [ ] Commit notes

**Wednesday — revm-interpreter**
- [ ] Read revm-interpreter source
- [ ] Study opcode dispatch mechanism
- [ ] Note gas calculation patterns
- [ ] Commit notes

**Thursday — revm-interpreter hot path**
- [ ] Trace execution of a simple opcode (ADD) end-to-end
- [ ] Note every function call
- [ ] Document in notes/
- [ ] Commit + log

**Friday — revm handler, precompiles**
- [ ] Read revm handler abstraction
- [ ] Read precompiles crate
- [ ] Commit notes

**Saturday — First revm PR**
- [ ] Browse revm issues
- [ ] Pick good first issue
- [ ] Implement, submit
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 19 — revm PRs velocity

**Monday — Second revm PR**
- [ ] Pick and implement
- [ ] Commit + log

**Tuesday — revm PR review response**
- [ ] Address any reviewer feedback
- [ ] Iterate
- [ ] Commit + log

**Wednesday — Third revm PR**
- [ ] Pick medium-difficulty issue
- [ ] Implement
- [ ] Commit + log

**Thursday — EVM comparison study**
- [ ] Read geth's core/vm package (as Go, but compare design)
- [ ] Note differences in opcode dispatch
- [ ] Commit notes

**Friday — evmone comparison**
- [ ] Read evmone README and architecture
- [ ] Note C++ optimization techniques
- [ ] Commit notes

**Saturday — Continue revm PRs**
- [ ] Work on outstanding PRs or start new
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 20 — MPT deep dive begins

**Monday — MPT theory**
- [ ] Ethereum.org docs on Merkle Patricia Trie
- [ ] Read multiple blog post explanations to triangulate
- [ ] Commit notes

**Tuesday — MPT structure deep**
- [ ] Understand extension, branch, leaf, hash nodes
- [ ] Draw diagrams showing insertions
- [ ] Commit notes

**Wednesday — Tiny MPT start**
- [ ] Start throwaway MPT implementation
- [ ] Support insert only (happy path, no pruning)
- [ ] Commit + log

**Thursday — Tiny MPT: get**
- [ ] Add get operation
- [ ] Add simple test cases
- [ ] Commit + log

**Friday — Tiny MPT: root hash**
- [ ] Implement root hash computation
- [ ] Test against known Ethereum test vectors (simplest ones)
- [ ] Commit + log

**Saturday — Tiny MPT: finish**
- [ ] Polish to pass basic vectors
- [ ] Discard as learning artifact (the real production MPT comes in Phase 3)
- [ ] Commit notes on learnings
- [ ] Commit + log

**Sunday — Rest + End Month 5 review**
- [ ] Update North Star M5 metrics

---

## Month 6: MPT Understanding + First Maintainer Interactions

### Week 21 — RLP encoding + more revm PRs

**Monday — RLP specification**
- [ ] Read RLP spec (Recursive Length Prefix)
- [ ] Understand encoding rules
- [ ] Exercise: implement RLP encoder/decoder (throwaway)
- [ ] Commit + log

**Tuesday — Reth RLP implementation**
- [ ] Read reth's RLP usage patterns
- [ ] Read alloy-rlp source
- [ ] Commit notes

**Wednesday — More revm PR**
- [ ] Fourth revm PR
- [ ] Commit + log

**Thursday — Review Foundry PRs + submit another**
- [ ] Second Foundry PR
- [ ] Commit + log

**Friday — Maintainer engagement**
- [ ] Identify patterns of which maintainers review which areas
- [ ] Engage thoughtfully in an issue discussion
- [ ] Do NOT pester — substantive only
- [ ] Commit notes

**Saturday — Consolidation**
- [ ] Review all open PRs
- [ ] Close out review comments
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 22 — Staged sync architecture

**Monday — Erigon staged sync**
- [ ] Read Erigon staged sync design doc
- [ ] Understand stage concept, unwind
- [ ] Commit notes

**Tuesday — Reth stages**
- [ ] Browse reth/crates/stages
- [ ] Identify all stages in pipeline
- [ ] Commit notes

**Wednesday — Stage dependencies**
- [ ] Map out: headers → bodies → senders → execution → hashing → merkle
- [ ] Draw diagram in notes/
- [ ] Commit + log

**Thursday — More revm or Alloy PRs**
- [ ] Keep PR velocity up
- [ ] Commit + log

**Friday — Reth Telegram + Discord**
- [ ] Join reth main Telegram if haven't
- [ ] Observe discussion style for a week before posting
- [ ] Commit notes

**Saturday — Tiny RLP + consolidation**
- [ ] Polish or discard experimental code
- [ ] Review Month 6 progress
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 23 — Ready up for Phase 3

**Monday — Reth storage crate survey**
- [ ] Browse reth/crates/storage
- [ ] Identify sub-crates: db, provider, etc.
- [ ] Commit notes (passive reading, not deep yet)

**Tuesday — MDBX first look**
- [ ] Read libmdbx high-level README
- [ ] Understand what MDBX is (mmap B-tree)
- [ ] Commit notes

**Wednesday — More Alloy/revm PRs**
- [ ] Keep contribution streak
- [ ] Commit + log

**Thursday — Conference research**
- [ ] Research EthCC Paris 2027 dates
- [ ] Research Devcon 2027 dates
- [ ] Start budgeting
- [ ] Commit notes

**Friday — Relationship review**
- [ ] Update maintainer tracker
- [ ] Note who has reviewed your PRs
- [ ] Identify target mentor (likely Matthias Seitz)
- [ ] Commit notes

**Saturday — Month 6 consolidation**
- [ ] Review all PRs merged / in review across Paradigm ecosystem
- [ ] Check target: 5+ Alloy, 3+ revm, 2+ Foundry
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 24 — Phase 2 close + Phase 3 prep

**Monday — Mastering Ethereum consensus chapter**
- [ ] Mastering Ethereum on consensus (for background)
- [ ] Understand The Merge at high level
- [ ] Commit notes

**Tuesday — Reth architecture talk/video**
- [ ] Watch any available gakonst reth architecture talk on YouTube
- [ ] Watch any Paradigm Frontiers talk
- [ ] Commit notes

**Wednesday — Deep breath + planning**
- [ ] Read Phase 3 section of this plan carefully
- [ ] Understand scope of `storage-trie` crate
- [ ] Outline approach for Month 7
- [ ] Commit notes

**Thursday — Setup Phase 3 repo scaffolding**
- [ ] Create crate `storage-trie`
- [ ] Setup CI (GitHub Actions with fmt, clippy, test, bench)
- [ ] Initial README with vision
- [ ] Commit + log

**Friday — Final Phase 2 PRs**
- [ ] Wrap up any outstanding PRs
- [ ] Commit + log

**Saturday — Phase 2 review**
- [ ] Full assessment against Phase 2 exit criteria
- [ ] Update progress.md with Phase 2 summary
- [ ] Commit + log

**Sunday — End Phase 2 + Phase 3 prep**
- [ ] Full rest
- [ ] Mentally prepare for Phase 3 intensity
- [ ] Phase 3 starts tomorrow

---

# PHASE 3: STORAGE + TRIE DEEP DIVE (Month 7-12)

**Deliverable**: `storage-trie` production crate

## Month 7: MDBX Foundation + First Reth Storage PRs

### Week 25 — MDBX documentation deep

**Monday — MDBX overview**
- [ ] Read libmdbx.dqdkfa.ru full overview
- [ ] Understand mmap-based design
- [ ] Commit notes

**Tuesday — MDBX internals: B-tree**
- [ ] Read MDBX B-tree structure section
- [ ] Compare with standard B+tree concepts
- [ ] Commit notes

**Wednesday — MDBX internals: MVCC**
- [ ] Read MVCC section
- [ ] Understand read tx during write tx
- [ ] Commit notes

**Thursday — MDBX internals: Durability**
- [ ] Read write-ahead logging / sync modes
- [ ] Understand crash recovery
- [ ] Commit notes

**Friday — MDBX cursor semantics**
- [ ] Read cursor documentation
- [ ] Understand efficient range scan
- [ ] Commit notes

**Saturday — libmdbx-rs source**
- [ ] Clone and read libmdbx-rs crate
- [ ] Understand Rust bindings to C library
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 26 — Research reth storage architecture

**Monday — Reth storage survey**
- [ ] Browse every crate in reth/crates/storage/
- [ ] Identify: db, provider, api, codecs
- [ ] Map high-level relationships
- [ ] Commit notes

**Tuesday — reth-db deep read part 1**
- [ ] Read reth-db/src/lib.rs
- [ ] Read table definitions
- [ ] Commit notes

**Wednesday — reth-db deep read part 2**
- [ ] Read transaction implementation
- [ ] Read cursor wrappers
- [ ] Commit notes

**Thursday — reth-provider read**
- [ ] Read reth-provider crate
- [ ] Understand abstraction over db
- [ ] Commit notes

**Friday — First reth storage PR hunt**
- [ ] Browse reth issues tagged storage
- [ ] Find good-first-issue or docs issue in storage area
- [ ] Claim issue
- [ ] Commit notes

**Saturday — First reth storage PR work**
- [ ] Implement
- [ ] Run full test suite
- [ ] Submit PR
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 27 — `storage-trie` crate: mmap scaffold

**Monday — Research mmap in Rust**
- [ ] Read memmap2 crate docs
- [ ] Read about Rust + mmap safety considerations
- [ ] Commit notes

**Tuesday — mmap B-tree research**
- [ ] Research B-tree on mmap techniques
- [ ] Review academic papers if applicable
- [ ] AI-assisted research on MDBX design decisions
- [ ] Commit notes

**Wednesday — Crate structure**
- [ ] Design crate module structure
- [ ] Define page abstraction
- [ ] Define transaction abstraction
- [ ] Commit + log

**Thursday — Page manager skeleton**
- [ ] Implement Page struct with fixed size
- [ ] Implement PageManager for allocation
- [ ] Unit tests
- [ ] Commit + log

**Friday — mmap wrapper**
- [ ] Implement mmap-backed file wrapper
- [ ] Support growth
- [ ] Unit tests
- [ ] Commit + log

**Saturday — Respond to reth PR review + continue crate**
- [ ] Address reth PR feedback
- [ ] Continue crate work
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 28 — `storage-trie` crate: B-tree core

**Monday — B-tree node design**
- [ ] Design leaf vs internal node layout
- [ ] Key/value storage in pages
- [ ] Commit + log

**Tuesday — B-tree insert**
- [ ] Implement insert with node splitting
- [ ] Unit tests
- [ ] Commit + log

**Wednesday — B-tree get**
- [ ] Implement lookup by key
- [ ] Range iteration
- [ ] Unit tests
- [ ] Commit + log

**Thursday — B-tree delete**
- [ ] Implement delete with node merging
- [ ] Unit tests
- [ ] Commit + log

**Friday — Second reth storage PR**
- [ ] Pick next issue
- [ ] Implement
- [ ] Commit + log

**Saturday — Crate polish**
- [ ] Document public API
- [ ] Run clippy, fmt
- [ ] Benchmark setup
- [ ] Commit + log

**Sunday — Rest + End Month 7 review**
- [ ] Update North Star M7 metrics

---

## Month 8: MVCC + Reth Storage Contribution Velocity

### Week 29 — MVCC in `storage-trie`

**Monday — MVCC design**
- [ ] Design MVCC approach (version chain vs copy-on-write)
- [ ] AI research: how MDBX implements MVCC
- [ ] Commit notes

**Tuesday — Read transaction**
- [ ] Implement read transaction with snapshot
- [ ] Unit tests for concurrent reads
- [ ] Commit + log

**Wednesday — Write transaction**
- [ ] Implement write transaction with copy-on-write
- [ ] Unit tests
- [ ] Commit + log

**Thursday — Concurrent read during write**
- [ ] Test read tx during write tx
- [ ] Verify snapshot isolation
- [ ] Commit + log

**Friday — Third reth storage PR**
- [ ] Pick medium-difficulty issue this time
- [ ] Implement
- [ ] Commit + log

**Saturday — Crate: durability**
- [ ] Implement fsync strategies
- [ ] Crash recovery basic
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 30 — Reth trie crate reading

**Monday — reth-trie overview**
- [ ] Browse reth/crates/trie
- [ ] Read top-level lib.rs
- [ ] Commit notes

**Tuesday — reth-trie node types**
- [ ] Read node definitions: extension, branch, leaf
- [ ] Commit notes

**Wednesday — reth-trie state root**
- [ ] Read state root computation
- [ ] Understand incremental computation
- [ ] Commit notes

**Thursday — reth-trie hashed state**
- [ ] Read hashed state abstraction
- [ ] Understand why hashing keys
- [ ] Commit notes

**Friday — First reth trie PR**
- [ ] Find trie-related issue
- [ ] Implement
- [ ] Commit + log

**Saturday — Crate: benchmarks**
- [ ] Add criterion benchmarks for B-tree ops
- [ ] Baseline vs sled, redb
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 31 — MPT in `storage-trie`

**Monday — MPT design in crate**
- [ ] Design MPT module structure
- [ ] Leverage B-tree as backing storage
- [ ] Commit + log

**Tuesday — MPT nodes**
- [ ] Implement extension node
- [ ] Implement branch node
- [ ] Implement leaf node
- [ ] Commit + log

**Wednesday — MPT insert**
- [ ] Implement MPT insert
- [ ] Test against simple vectors
- [ ] Commit + log

**Thursday — MPT get**
- [ ] Implement get with path traversal
- [ ] Commit + log

**Friday — MPT root hash**
- [ ] Implement keccak-based root hash
- [ ] Test against Ethereum test vectors
- [ ] Commit + log

**Saturday — Reth trie second PR**
- [ ] Continue trie contribution
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 32 — MPT proofs + more reth PRs

**Monday — MPT proof generation**
- [ ] Implement Merkle proof generation
- [ ] Unit tests
- [ ] Commit + log

**Tuesday — MPT proof verification**
- [ ] Implement standalone proof verification
- [ ] Unit tests
- [ ] Commit + log

**Wednesday — MPT delete**
- [ ] Implement MPT delete with rebalancing
- [ ] Commit + log

**Thursday — Ethereum test vectors**
- [ ] Integrate official Ethereum trie test vectors
- [ ] Pass basic suite
- [ ] Commit + log

**Friday — Reth PR volume**
- [ ] Another reth PR (storage or trie)
- [ ] Commit + log

**Saturday — Crate docs**
- [ ] Write comprehensive docs for all public APIs
- [ ] Examples in docs
- [ ] Commit + log

**Sunday — Rest + End Month 8 review**

---

## Month 9: Trie Depth + Staged Sync Understanding

### Week 33 — Advanced trie: path compression

**Monday — Path compression theory**
- [ ] Research path compression in tries
- [ ] Study Ethereum's specific approach
- [ ] Commit notes

**Tuesday — Implement path compression**
- [ ] Add path compression to crate MPT
- [ ] Verify correctness
- [ ] Commit + log

**Wednesday — Benchmark path compression**
- [ ] Benchmark with/without
- [ ] Document findings
- [ ] Commit + log

**Thursday — Reth staged sync survey**
- [ ] Browse reth/crates/stages deeply
- [ ] Identify every stage and its purpose
- [ ] Commit notes

**Friday — Stage dependencies diagram**
- [ ] Draw detailed flow diagram
- [ ] Document unwind paths
- [ ] Commit notes

**Saturday — Reth PR day**
- [ ] Another reth PR
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 34 — Pruning strategies

**Monday — Pruning research**
- [ ] Research Ethereum pruning modes: full, archive, pruned
- [ ] Understand state vs history pruning
- [ ] Commit notes

**Tuesday — Reth pruning code**
- [ ] Read reth pruner crate
- [ ] Understand configuration
- [ ] Commit notes

**Wednesday — Crate: pruning design**
- [ ] Design pruning strategy trait
- [ ] Plan integration with MPT
- [ ] Commit + log

**Thursday — Implement full pruning**
- [ ] Implement "full" retention (prune history beyond N blocks)
- [ ] Commit + log

**Friday — Implement archive mode**
- [ ] Keep everything mode
- [ ] Commit + log

**Saturday — Reth PR + integration testing**
- [ ] Reth PR in pruning area if possible
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 35 — State commitment deep

**Monday — State commitment theory**
- [ ] Read about state commitment schemes
- [ ] Understand MPT vs Verkle tradeoffs
- [ ] Commit notes

**Tuesday — Verkle Trees reading**
- [ ] Read Verkle Trees research posts (Vitalik, EF)
- [ ] Understand polynomial commitments at high level
- [ ] Commit notes

**Wednesday — Crate: incremental root**
- [ ] Design incremental state root computation
- [ ] Implement
- [ ] Commit + log

**Thursday — Benchmark incremental vs full**
- [ ] Benchmark root computation
- [ ] Document findings
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Continue PR velocity
- [ ] Commit + log

**Saturday — Crate polish**
- [ ] Clean up APIs
- [ ] Update docs
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 36 — Snapshot sync research

**Monday — Snapshot sync theory**
- [ ] Read about Ethereum snapshot sync
- [ ] Understand why different from state sync
- [ ] Commit notes

**Tuesday — Erigon snapshots**
- [ ] Read Erigon's snapshot strategy
- [ ] Note file format
- [ ] Commit notes

**Wednesday — Reth snapshots**
- [ ] Read reth's snapshot approach
- [ ] Commit notes

**Thursday — Crate: snapshot export**
- [ ] Design snapshot export format
- [ ] Implement basic export
- [ ] Commit + log

**Friday — Crate: snapshot import**
- [ ] Implement snapshot import
- [ ] Commit + log

**Saturday — End Month 9 PR push**
- [ ] 1-2 more reth PRs
- [ ] Commit + log

**Sunday — Rest + End Month 9 review**
- [ ] Update North Star M9 metrics
- [ ] Check: 15+ reth PRs, 10+ in storage/trie

---

## Month 10: Cross-Subsystem Storage PRs + Integration

### Week 37 — Medium-sized reth PRs

**Monday — Identify meaningful PR target**
- [ ] Look for enhancement issues (not just docs)
- [ ] Identify 1 medium PR candidate
- [ ] Design approach
- [ ] Commit notes

**Tuesday — Medium PR: implement**
- [ ] Start implementation
- [ ] Commit + log

**Wednesday — Medium PR: tests**
- [ ] Comprehensive testing
- [ ] Commit + log

**Thursday — Medium PR: benchmark**
- [ ] Add perf measurements if relevant
- [ ] Commit + log

**Friday — Medium PR: submit**
- [ ] Polish and submit
- [ ] Commit + log

**Saturday — Crate work**
- [ ] Continue `storage-trie` enhancements
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 38 — Codec / compression deep

**Monday — Reth codecs**
- [ ] Read reth codecs crate
- [ ] Understand compact encoding
- [ ] Commit notes

**Tuesday — Zstd compression in reth**
- [ ] Read how reth uses compression
- [ ] Commit notes

**Wednesday — Crate: codec support**
- [ ] Add compact encoding to crate
- [ ] Commit + log

**Thursday — Crate: compression**
- [ ] Optional compression layer
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Codec-related PR ideally
- [ ] Commit + log

**Saturday — Crate benchmarks**
- [ ] Bench compression tradeoffs
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 39 — Reth storage architecture contributions

**Monday — Read storage discussions**
- [ ] Read all recent GitHub discussions on storage
- [ ] Commit notes

**Tuesday — Substantive comment**
- [ ] Find appropriate discussion
- [ ] Write substantive technical comment
- [ ] Commit notes

**Wednesday — More reth PR**
- [ ] Continue velocity
- [ ] Commit + log

**Thursday — Crate: composition test**
- [ ] Integration test: B-tree + MPT + transaction combined
- [ ] Commit + log

**Friday — Crate: example**
- [ ] Write example showing typical usage
- [ ] Commit + log

**Saturday — Consolidation**
- [ ] Review everything
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 40 — Medium reth feature development

**Monday — Feature proposal**
- [ ] Identify storage improvement opportunity from own reading
- [ ] Draft proposal comment on GitHub
- [ ] Commit notes

**Tuesday — Discuss with maintainers**
- [ ] If got feedback, iterate
- [ ] If approved, start design
- [ ] Commit notes

**Wednesday — Feature implementation part 1**
- [ ] Start coding
- [ ] Commit + log

**Thursday — Feature implementation part 2**
- [ ] Continue
- [ ] Commit + log

**Friday — Feature: tests**
- [ ] Comprehensive tests
- [ ] Commit + log

**Saturday — Feature: submit**
- [ ] Submit PR
- [ ] Commit + log

**Sunday — Rest + End Month 10 review**

---

## Month 11: Feature Shipping + Crate v1.0

### Week 41 — Ship reth feature

**Monday — Address feature PR reviews**
- [ ] Iterate on reviews
- [ ] Commit + log

**Tuesday — More iteration**
- [ ] Address remaining feedback
- [ ] Commit + log

**Wednesday — Feature merged ideally**
- [ ] If merged, celebrate + blog draft
- [ ] If not, keep iterating
- [ ] Commit + log

**Thursday — Crate: performance pass**
- [ ] Profile `storage-trie`
- [ ] Identify hot paths
- [ ] Commit + log

**Friday — Crate: optimizations**
- [ ] Implement optimizations
- [ ] Commit + log

**Saturday — Another reth PR**
- [ ] Keep velocity
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 42 — Geth comparison study

**Monday — Geth core/state package**
- [ ] Read Geth's Go implementation
- [ ] Note architectural differences from reth
- [ ] Commit notes

**Tuesday — Geth core/trie**
- [ ] Read Geth trie implementation
- [ ] Commit notes

**Wednesday — Write comparison doc**
- [ ] Internal doc: reth vs Geth storage decisions
- [ ] Potential blog material
- [ ] Commit + log

**Thursday — Reth PR**
- [ ] Continue
- [ ] Commit + log

**Friday — Crate: fuzz targets**
- [ ] Add cargo-fuzz targets to crate
- [ ] Commit + log

**Saturday — Crate: property tests**
- [ ] Add proptest for MPT invariants
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 43 — Crate v1.0 preparation

**Monday — API review**
- [ ] Review all public APIs
- [ ] Stabilize naming, signatures
- [ ] Commit + log

**Tuesday — Documentation pass**
- [ ] Every public item has docs
- [ ] Top-level crate-level docs comprehensive
- [ ] Commit + log

**Wednesday — Examples expansion**
- [ ] Multiple examples in examples/
- [ ] Cover main use cases
- [ ] Commit + log

**Thursday — CI hardening**
- [ ] All CI checks pass
- [ ] Coverage reporting
- [ ] MSRV policy
- [ ] Commit + log

**Friday — README + design doc**
- [ ] Comprehensive README
- [ ] DESIGN.md explaining architectural choices (AI-researched)
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] More PR activity
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 44 — Crate v1.0 ship

**Monday — Final benchmarks**
- [ ] Comprehensive bench suite
- [ ] Compare against reth, sled, redb
- [ ] Commit + log

**Tuesday — Security review self-audit**
- [ ] Review unsafe blocks
- [ ] Review error handling
- [ ] Commit + log

**Wednesday — Crate v1.0 tag**
- [ ] Tag release
- [ ] Consider crates.io publication (optional)
- [ ] Commit + log

**Thursday — Blog: crate intro**
- [ ] If in writing mood, draft "Building storage-trie" post in your disruptor style
- [ ] No deadline
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Continue reth contributions
- [ ] Commit + log

**Saturday — Month 11 review**
- [ ] Assess crate quality
- [ ] Assess reth PR portfolio
- [ ] Commit + log

**Sunday — Rest + End Month 11 review**

---

## Month 12: Phase 3 Close + Phase 4 Prep

### Week 45 — Final reth storage feature

**Monday — Identify second feature**
- [ ] Find another meaningful opportunity
- [ ] Design
- [ ] Commit notes

**Tuesday — Implement**
- [ ] Code
- [ ] Commit + log

**Wednesday — Continue**
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Submit**
- [ ] Submit PR
- [ ] Commit + log

**Saturday — Iterate on reviews**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 46 — Recognition signals

**Monday — Review PRs of others**
- [ ] Start reviewing others' storage PRs substantively
- [ ] Commit notes

**Tuesday — Help newcomers**
- [ ] Answer questions in Telegram
- [ ] Commit notes

**Wednesday — More PR reviews**
- [ ] Build reviewing muscle
- [ ] Commit notes

**Thursday — Maintainer relationship check**
- [ ] Which maintainers have engaged with me?
- [ ] Update tracker
- [ ] Commit notes

**Friday — Active issue engagement**
- [ ] Participate in design discussions
- [ ] Commit notes

**Saturday — Another small reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 47 — revm preview for Phase 4

**Monday — revm architecture refresher**
- [ ] Re-read revm with Phase 3 eyes
- [ ] Note storage integration points
- [ ] Commit notes

**Tuesday — Identify revm learning gaps**
- [ ] Map what needs deep understanding in Phase 4
- [ ] Commit notes

**Wednesday — Reth evm crate**
- [ ] Read reth/crates/evm
- [ ] Understand reth's revm integration
- [ ] Commit notes

**Thursday — More reth PR**
- [ ] Continue
- [ ] Commit + log

**Friday — Crate maintenance**
- [ ] Any bug fixes on storage-trie
- [ ] Commit + log

**Saturday — Prep Phase 4 scaffolding**
- [ ] Create `exec-vm` crate scaffold
- [ ] Initial README
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 48 — Phase 3 close

**Monday — Phase 3 reflection**
- [ ] Full assessment vs exit criteria
- [ ] Commit notes

**Tuesday — Metrics update**
- [ ] Update all North Star metrics
- [ ] Target check: 30 storage PRs, 1+ feature
- [ ] Commit notes

**Wednesday — Blog if in mood**
- [ ] Consider writing Phase 3 retrospective in disruptor style
- [ ] Focus on storage-trie design choices
- [ ] No pressure if not ready
- [ ] Commit + log

**Thursday — Relationship stock-take**
- [ ] Update maintainer tracker
- [ ] Identify mentor candidate (likely Matthias)
- [ ] Commit notes

**Friday — Final Phase 3 PRs**
- [ ] Wrap any outstanding
- [ ] Commit + log

**Saturday — Clean transition prep**
- [ ] Mental prep for Phase 4
- [ ] Storage maintenance minimum during Phase 4
- [ ] Commit notes

**Sunday — End Phase 3 rest**
- [ ] Full rest
- [ ] Phase 4 starts tomorrow

---

# PHASE 4: EXECUTION DEEP DIVE (Month 13-18)

**Deliverable**: `exec-vm` production crate

## Month 13: Revm Full Codebase + First revm Perf PRs

### Week 49 — Revm architecture deep

**Monday — Revm top-level**
- [ ] Re-read revm from top
- [ ] Map all crates
- [ ] Commit notes

**Tuesday — Revm interpreter core**
- [ ] Read revm-interpreter in full
- [ ] Focus on main execution loop
- [ ] Commit notes

**Wednesday — Revm Host trait**
- [ ] Read Host trait and implementations
- [ ] Understand decoupling from storage
- [ ] Commit notes

**Thursday — Revm Database trait**
- [ ] Read Database trait
- [ ] Note how it integrates with any storage
- [ ] Commit notes

**Friday — Revm precompiles**
- [ ] Read revm-precompiles crate
- [ ] Understand each precompile
- [ ] Commit notes

**Saturday — First revm perf-oriented PR**
- [ ] Find performance issue
- [ ] Implement
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 50 — Revm journaling

**Monday — Journaling design**
- [ ] Read revm-interpreter journal module
- [ ] Understand revert semantics
- [ ] Commit notes

**Tuesday — Nested checkpoints**
- [ ] Study nested call handling
- [ ] Commit notes

**Wednesday — State access patterns**
- [ ] Read state management in revm
- [ ] Map read/write patterns
- [ ] Commit notes

**Thursday — Second revm PR**
- [ ] Another contribution
- [ ] Commit + log

**Friday — `exec-vm` scaffold**
- [ ] Setup crate structure
- [ ] Host trait, Database trait (mirroring revm)
- [ ] Commit + log

**Saturday — `exec-vm` main loop**
- [ ] Implement core interpreter loop
- [ ] Match-based dispatch first
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 51 — `exec-vm` basic opcodes

**Monday — Arithmetic opcodes**
- [ ] ADD, MUL, SUB, DIV, SDIV, MOD, SMOD, ADDMOD, MULMOD, EXP, SIGNEXTEND
- [ ] Unit tests for each
- [ ] Commit + log

**Tuesday — Comparison and logic**
- [ ] LT, GT, SLT, SGT, EQ, ISZERO, AND, OR, XOR, NOT, BYTE, SHL, SHR, SAR
- [ ] Commit + log

**Wednesday — SHA3, environmental**
- [ ] KECCAK256, ADDRESS, BALANCE, ORIGIN, CALLER, CALLVALUE, CALLDATALOAD, CALLDATASIZE, CALLDATACOPY, CODESIZE, CODECOPY, GASPRICE
- [ ] Commit + log

**Thursday — Block info opcodes**
- [ ] BLOCKHASH, COINBASE, TIMESTAMP, NUMBER, PREVRANDAO, GASLIMIT, CHAINID, SELFBALANCE, BASEFEE, BLOBHASH, BLOBBASEFEE
- [ ] Commit + log

**Friday — Stack/memory opcodes**
- [ ] POP, MLOAD, MSTORE, MSTORE8, JUMP, JUMPI, PC, MSIZE, GAS, JUMPDEST, PUSH0-PUSH32, DUP1-DUP16, SWAP1-SWAP16
- [ ] Commit + log

**Saturday — Storage opcodes**
- [ ] SLOAD, SSTORE, TLOAD, TSTORE
- [ ] Integrate with Database trait
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 52 — `exec-vm` control flow + gas

**Monday — Gas accounting**
- [ ] Implement gas tracking
- [ ] Gas schedule for London
- [ ] Commit + log

**Tuesday — Memory expansion gas**
- [ ] Quadratic memory gas cost
- [ ] Commit + log

**Wednesday — Gas schedule per fork**
- [ ] Shanghai, Cancun schedules
- [ ] Commit + log

**Thursday — LOG opcodes**
- [ ] LOG0-LOG4
- [ ] Commit + log

**Friday — CREATE/CALL family**
- [ ] CREATE, CREATE2
- [ ] CALL, CALLCODE, DELEGATECALL, STATICCALL
- [ ] Commit + log

**Saturday — Another revm PR + reth storage PR**
- [ ] Maintain storage PR velocity
- [ ] revm contribution
- [ ] Commit + log

**Sunday — Rest + End Month 13 review**

---

## Month 14: Full Opcode Coverage + Precompiles

### Week 53 — Complete opcode set

**Monday — RETURN, REVERT, INVALID**
- [ ] Terminal opcodes
- [ ] Commit + log

**Tuesday — SELFDESTRUCT**
- [ ] Implement
- [ ] Commit + log

**Wednesday — EIP-1153 transient storage**
- [ ] TLOAD/TSTORE if not done
- [ ] Commit + log

**Thursday — Test vector integration**
- [ ] Integrate Ethereum execution test vectors
- [ ] Start passing basic suite
- [ ] Commit + log

**Friday — revm PR**
- [ ] Contribution
- [ ] Commit + log

**Saturday — Reth evm PR**
- [ ] Find reth evm crate issue
- [ ] Implement
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 54 — Precompiles in `exec-vm`

**Monday — ecrecover**
- [ ] Implement ecrecover precompile
- [ ] Test vectors
- [ ] Commit + log

**Tuesday — sha256, ripemd160, identity**
- [ ] Implement all three
- [ ] Commit + log

**Wednesday — modexp**
- [ ] Implement (using num-bigint)
- [ ] Commit + log

**Thursday — BN256 operations**
- [ ] BN256Add, BN256ScalarMul, BN256Pairing
- [ ] Use ark or similar
- [ ] Commit + log

**Friday — blake2f**
- [ ] Implement Blake2 F compression
- [ ] Commit + log

**Saturday — KZG precompile**
- [ ] Point evaluation precompile (EIP-4844)
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 55 — Journaling in `exec-vm`

**Monday — Journal design**
- [ ] Design journal structure
- [ ] Mirror revm's approach
- [ ] Commit + log

**Tuesday — Account journal**
- [ ] Track account changes with undo log
- [ ] Commit + log

**Wednesday — Storage journal**
- [ ] Track storage changes
- [ ] Commit + log

**Thursday — Nested checkpoints**
- [ ] Support nested call checkpoint/commit
- [ ] Commit + log

**Friday — Revert semantics tests**
- [ ] Test revert properly undoes all changes
- [ ] Commit + log

**Saturday — revm PR**
- [ ] Contribution
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 56 — Test vector push

**Monday — Ethereum tests repo**
- [ ] Integrate comprehensive test vectors
- [ ] Commit + log

**Tuesday — General state tests**
- [ ] Run general state tests
- [ ] Fix failures
- [ ] Commit + log

**Wednesday — More failure fixing**
- [ ] Continue
- [ ] Commit + log

**Thursday — Validator tests**
- [ ] Run validator test suite if applicable
- [ ] Commit + log

**Friday — reth PR**
- [ ] Storage or evm
- [ ] Commit + log

**Saturday — revm PR**
- [ ] Contribution
- [ ] Commit + log

**Sunday — Rest + End Month 14 review**
- [ ] Crate passing majority of test vectors

---

## Month 15: Dispatch Strategies + EthCC Prep

### Week 57 — EthCC Paris trip

**Monday-Friday — Conference attendance**
- [ ] Attend EthCC sessions
- [ ] Target: 1-on-1 with 3 reth core contributors
- [ ] Arrange meetings in advance via Twitter DM
- [ ] Attend side events (hacker houses, dinners)
- [ ] Take notes on talks

**Saturday — Travel home**
- [ ] Rest

**Sunday — Post-conference ritual**
- [ ] Update maintainer tracker with new connections
- [ ] Capture insights
- [ ] Follow-up emails/DMs

---

### Week 58 — Back to work: dispatch strategies

**Monday — Match dispatch (baseline)**
- [ ] Current implementation is match-based
- [ ] Baseline benchmark
- [ ] Commit + log

**Tuesday — Jump table research**
- [ ] Research function pointer jump tables
- [ ] Commit notes

**Wednesday — Implement jump table dispatch**
- [ ] Implement in `exec-vm`
- [ ] Feature-flagged
- [ ] Commit + log

**Thursday — Computed goto research**
- [ ] Research unsafe computed goto via asm
- [ ] Note portability tradeoffs
- [ ] Commit notes

**Friday — Benchmark match vs jump table**
- [ ] Measure instruction-level differences
- [ ] Commit + log

**Saturday — Dispatch strategy docs**
- [ ] Document findings (blog material in your style)
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 59 — evmone comparison

**Monday — evmone overview**
- [ ] Read evmone README deeply
- [ ] Note architecture choices
- [ ] Commit notes

**Tuesday — evmone basic interpreter**
- [ ] Read evmone basic mode
- [ ] Commit notes

**Wednesday — evmone advanced mode**
- [ ] Read advanced interpreter with caching
- [ ] Commit notes

**Thursday — Apply learnings to `exec-vm`**
- [ ] Implement any applicable optimizations
- [ ] Commit + log

**Friday — Benchmark exec-vm vs revm**
- [ ] Comprehensive benchmark
- [ ] Identify gaps
- [ ] Commit + log

**Saturday — revm PR**
- [ ] Another contribution
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 60 — Hot path optimization

**Monday — Profile `exec-vm`**
- [ ] Profile with perf or similar
- [ ] Identify hot spots
- [ ] Commit notes

**Tuesday — Stack optimization**
- [ ] Inline stack ops
- [ ] Commit + log

**Wednesday — Memory access**
- [ ] Optimize memory reads/writes
- [ ] Commit + log

**Thursday — Gas calculation**
- [ ] Optimize gas tracking in hot path
- [ ] Commit + log

**Friday — Benchmark improvements**
- [ ] Measure gains
- [ ] Commit + log

**Saturday — More reth PRs**
- [ ] Keep reth velocity up (10+ execution PRs target M18)
- [ ] Commit + log

**Sunday — Rest + End Month 15 review**

---

## Month 16: EOF + Integration with storage-trie

### Week 61 — EOF implementation

**Monday — EOF EIP deep re-read**
- [ ] Re-read EIP-3540, 3670
- [ ] Understand EOF container format
- [ ] Commit notes

**Tuesday — EOF validation**
- [ ] Implement stack validation per EIP-3670
- [ ] Commit + log

**Wednesday — Static relative jumps**
- [ ] Implement EIP-4200 opcodes
- [ ] Commit + log

**Thursday — Functions (EIP-4750)**
- [ ] Implement CALLF, RETF, JUMPF
- [ ] Commit + log

**Friday — EOF tests**
- [ ] Integrate EOF test vectors if available
- [ ] Commit + log

**Saturday — revm EOF PR**
- [ ] If revm has EOF issues, contribute
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 62 — `exec-vm` + `storage-trie` integration

**Monday — Integration design**
- [ ] Design how `exec-vm` uses `storage-trie` via Database trait
- [ ] Commit + log

**Tuesday — Implement integration**
- [ ] Wire up the two crates
- [ ] Commit + log

**Wednesday — Integration tests**
- [ ] End-to-end execution with real storage
- [ ] Commit + log

**Thursday — Benchmark integrated stack**
- [ ] Measure performance vs revm + reth storage
- [ ] Commit + log

**Friday — reth evm PR**
- [ ] Reth-side contribution
- [ ] Commit + log

**Saturday — Crate maintenance**
- [ ] storage-trie fixes if needed
- [ ] exec-vm polish
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 63 — Fuzz targets

**Monday — Fuzz setup**
- [ ] Setup cargo-fuzz
- [ ] First fuzz target on opcode sequences
- [ ] Commit + log

**Tuesday — Run fuzz, fix findings**
- [ ] Run fuzzer
- [ ] Address any crashes
- [ ] Commit + log

**Wednesday — More fuzz targets**
- [ ] Fuzz gas metering
- [ ] Fuzz call operations
- [ ] Commit + log

**Thursday — Differential fuzzing**
- [ ] Fuzz `exec-vm` vs revm for consistency
- [ ] Commit + log

**Friday — reth or revm PR**
- [ ] Contribution
- [ ] Commit + log

**Saturday — Docs pass**
- [ ] `exec-vm` documentation
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 64 — Revm performance PR push

**Monday — Identify revm perf opportunity**
- [ ] Deep profile revm in common scenarios
- [ ] Find improvement area
- [ ] Commit notes

**Tuesday — Design optimization**
- [ ] Plan approach
- [ ] Commit notes

**Wednesday — Implement**
- [ ] Code optimization
- [ ] Commit + log

**Thursday — Benchmark**
- [ ] Measure improvement
- [ ] Commit + log

**Friday — Submit revm PR**
- [ ] Clean PR
- [ ] Commit + log

**Saturday — Respond to reviews**
- [ ] Iterate
- [ ] Commit + log

**Sunday — Rest + End Month 16 review**

---

## Month 17: Architectural Discussions + Reth evm Features

### Week 65 — Architectural engagement

**Monday — GitHub discussions**
- [ ] Browse ongoing execution-layer architecture discussions
- [ ] Commit notes

**Tuesday — Substantive comment**
- [ ] Write a substantive architectural comment
- [ ] Commit notes

**Wednesday — Proposal draft**
- [ ] Draft a small design proposal for reth evm
- [ ] Commit notes

**Thursday — Submit proposal**
- [ ] Post as GitHub discussion
- [ ] Commit notes

**Friday — Engage discussion**
- [ ] Respond to feedback
- [ ] Commit notes

**Saturday — Reth PR**
- [ ] Storage or evm
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 66 — Reth evm feature

**Monday — Feature identification**
- [ ] Find meaningful reth evm improvement
- [ ] Design
- [ ] Commit notes

**Tuesday — Implementation**
- [ ] Start coding
- [ ] Commit + log

**Wednesday — Continue**
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Submit**
- [ ] PR ready
- [ ] Commit + log

**Saturday — Another storage PR (maintain velocity)**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 67 — exec-vm v1.0 prep

**Monday — API stabilization**
- [ ] Review all public APIs
- [ ] Freeze signatures
- [ ] Commit + log

**Tuesday — Docs pass**
- [ ] Every item documented
- [ ] Examples
- [ ] Commit + log

**Wednesday — Final benchmarks**
- [ ] Comprehensive suite
- [ ] Commit + log

**Thursday — DESIGN.md**
- [ ] Document architectural decisions
- [ ] AI-researched insights from revm
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Commit + log

**Saturday — Crate polish**
- [ ] Final cleanup
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 68 — exec-vm v1.0 ship

**Monday — Tag v1.0**
- [ ] Tag release
- [ ] Commit + log

**Tuesday — Blog if ready**
- [ ] Consider writing exec-vm intro blog in your style
- [ ] No pressure
- [ ] Commit + log

**Wednesday — Reth feature iteration**
- [ ] Address reviews on feature PR
- [ ] Commit + log

**Thursday — More reth**
- [ ] Continue velocity
- [ ] Commit + log

**Friday — Reviews given**
- [ ] Review 5+ others' PRs substantively
- [ ] Commit notes

**Saturday — Month 17 close**
- [ ] Commit + log

**Sunday — Rest + End Month 17 review**

---

## Month 18: Phase 4 Close + Consensus Prep

### Week 69 — Final execution PRs

**Monday — Final feature push**
- [ ] Last medium-sized feature
- [ ] Commit + log

**Tuesday — Implementation**
- [ ] Commit + log

**Wednesday — Tests + submit**
- [ ] Commit + log

**Thursday — Reviews**
- [ ] Commit + log

**Friday — Another small PR**
- [ ] Commit + log

**Saturday — Close outstanding work**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 70 — Consensus layer preview

**Monday — Ethereum consensus overview**
- [ ] Read Ethereum consensus layer intro
- [ ] Understand proof-of-stake at high level
- [ ] Commit notes

**Tuesday — Engine API spec preview**
- [ ] Read Engine API specification at high level
- [ ] Commit notes

**Wednesday — Lighthouse survey**
- [ ] Browse Lighthouse code at a high level
- [ ] Commit notes

**Thursday — Reth engine crate preview**
- [ ] Browse reth/crates/engine
- [ ] Commit notes

**Friday — Reth consensus crate preview**
- [ ] Browse reth/crates/consensus
- [ ] Commit notes

**Saturday — Phase 5 scaffolding**
- [ ] Create `consensus-engine` crate scaffold
- [ ] Initial README
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 71 — Phase 4 reflection

**Monday — Full Phase 4 assessment**
- [ ] Check exit criteria
- [ ] Commit notes

**Tuesday — Metrics**
- [ ] Update North Star M18 metrics
- [ ] Check 20+ execution PRs
- [ ] Commit notes

**Wednesday — Relationship update**
- [ ] Which maintainers engaged? Depth?
- [ ] Update tracker
- [ ] Commit notes

**Thursday — Blog consideration**
- [ ] Consider Phase 4 retrospective
- [ ] No deadline
- [ ] Commit notes

**Friday — Wrap**
- [ ] Close outstanding PRs
- [ ] Commit + log

**Saturday — Rest prep**
- [ ] Light day
- [ ] Commit notes

**Sunday — Rest**

---

### Week 72 — Transition week

**Monday — Mental prep Phase 5**
- [ ] Read Phase 5 section of plan
- [ ] Outline Month 19
- [ ] Commit notes

**Tuesday — Reading list for consensus**
- [ ] Compile reading list
- [ ] Commit notes

**Wednesday — Reach out to Lighthouse folks**
- [ ] If any connections, warm up
- [ ] Commit notes

**Thursday — Maintenance on previous crates**
- [ ] storage-trie, exec-vm bug fixes
- [ ] Commit + log

**Friday — Final exec-vm polish**
- [ ] Any remaining items
- [ ] Commit + log

**Saturday — Month 18 close**
- [ ] Final PRs
- [ ] Commit + log

**Sunday — Rest**
- [ ] Phase 5 starts tomorrow

---

# PHASE 5: CONSENSUS + ENGINE API (Month 19-24)

**Deliverable**: `consensus-engine` production crate + three-crate integration

## Month 19: Engine API Deep Dive

### Week 73 — Engine API specification

**Monday — Engine API full read part 1**
- [ ] Read Engine API spec sections 1-3
- [ ] Commit notes

**Tuesday — Engine API full read part 2**
- [ ] Read sections 4-6
- [ ] Commit notes

**Wednesday — newPayload deep**
- [ ] Study newPayload V1, V2, V3, V4
- [ ] Commit notes

**Thursday — forkchoiceUpdated deep**
- [ ] Study fcU variants
- [ ] Commit notes

**Friday — getPayload deep**
- [ ] Study getPayload variants
- [ ] Commit notes

**Saturday — JWT auth**
- [ ] Study JWT authentication used by Engine API
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 74 — Reth engine crate

**Monday — reth-engine structure**
- [ ] Browse reth/crates/engine
- [ ] Map files
- [ ] Commit notes

**Tuesday — Engine tree**
- [ ] Read engine tree implementation
- [ ] Understand block tree for forks
- [ ] Commit notes

**Wednesday — Payload builder**
- [ ] Read reth payload builder
- [ ] Commit notes

**Thursday — First engine PR**
- [ ] Find docs or small fix
- [ ] Commit + log

**Friday — `consensus-engine` scaffold**
- [ ] Start crate skeleton
- [ ] Engine API trait
- [ ] Commit + log

**Saturday — JWT implementation**
- [ ] Implement JWT auth in crate
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 75 — `consensus-engine`: core methods

**Monday — newPayload implementation**
- [ ] Implement newPayload V3 handler
- [ ] Commit + log

**Tuesday — Payload validation**
- [ ] Block header validation
- [ ] Commit + log

**Wednesday — forkchoiceUpdated**
- [ ] Implement fcU handler
- [ ] Commit + log

**Thursday — getPayload**
- [ ] Implement getPayload
- [ ] Commit + log

**Friday — Storage + engine integration**
- [ ] Wire up with storage-trie
- [ ] Commit + log

**Saturday — Engine + exec-vm integration**
- [ ] Execute payload using exec-vm
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 76 — Lighthouse CL perspective

**Monday — Lighthouse code survey**
- [ ] Browse Lighthouse execution interaction layer
- [ ] Commit notes

**Tuesday — Lighthouse Engine API client**
- [ ] Read Lighthouse's side of Engine API
- [ ] Commit notes

**Wednesday — Prysm perspective**
- [ ] Read Prysm equivalent (less depth)
- [ ] Commit notes

**Thursday — CL/EL lifecycle**
- [ ] Map full CL/EL communication flow
- [ ] Commit notes

**Friday — Another reth engine PR**
- [ ] Continue velocity
- [ ] Commit + log

**Saturday — Crate: connection handling**
- [ ] Websocket/HTTP Engine API transport
- [ ] Commit + log

**Sunday — Rest + End Month 19 review**

---

## Month 20: Full Engine API + State Transition Validation

### Week 77 — State transition validation

**Monday — STF theory**
- [ ] Read state transition function theory
- [ ] Commit notes

**Wednesday — Consensus rules in execution**
- [ ] What execution layer validates per consensus rules
- [ ] Commit notes

**Wednesday — Block validation**
- [ ] Implement block validation in crate
- [ ] Commit + log

**Thursday — Receipt validation**
- [ ] Receipt consistency checks
- [ ] Commit + log

**Friday — Gas limit validation**
- [ ] Block gas limit checks
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Engine or consensus area
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 78 — Fork choice integration

**Monday — Fork choice theory**
- [ ] Read fork choice rule (LMD-GHOST, Casper FFG)
- [ ] Commit notes

**Tuesday — Reth fork choice code**
- [ ] Read reth's fork choice handling
- [ ] Commit notes

**Wednesday — Crate: fork choice**
- [ ] Implement fork choice processing
- [ ] Commit + log

**Thursday — Safe/finalized tracking**
- [ ] Track safe, finalized, head blocks
- [ ] Commit + log

**Friday — Reorg detection**
- [ ] Detect reorgs from fork choice updates
- [ ] Commit + log

**Saturday — More reth PRs**
- [ ] Consensus or engine
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 79 — Reorg handling

**Monday — Reorg theory**
- [ ] Deep understand reorg handling in execution
- [ ] Commit notes

**Tuesday — State rollback**
- [ ] Implement state rollback on reorg
- [ ] Leverage storage-trie snapshots
- [ ] Commit + log

**Wednesday — Receipt reindexing**
- [ ] Handle receipt/log reindexing
- [ ] Commit + log

**Thursday — Transaction re-pool**
- [ ] Handle moving txs back to mempool on reorg
- [ ] Commit + log

**Friday — Reorg integration tests**
- [ ] Test various reorg scenarios
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 80 — Multi-branch state

**Monday — Multi-branch theory**
- [ ] Understand maintaining state across forks
- [ ] Commit notes

**Tuesday — Branch state design**
- [ ] Design multi-branch state in crate
- [ ] Commit + log

**Wednesday — Implement**
- [ ] Code branch state management
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Integration with exec-vm**
- [ ] Speculative execution across branches
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + End Month 20 review**

---

## Month 21: PBS + Builder API + Invalid Payload Handling

### Week 81 — Invalid payload handling

**Monday — Invalid payload scenarios**
- [ ] Catalog all invalid payload cases from spec
- [ ] Commit notes

**Tuesday — Invalid header**
- [ ] Handle invalid headers
- [ ] Commit + log

**Wednesday — Invalid transactions**
- [ ] Handle invalid tx in payload
- [ ] Commit + log

**Thursday — Invalid state root**
- [ ] Handle state root mismatch
- [ ] Commit + log

**Friday — Latest valid hash logic**
- [ ] Implement LVH tracking
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 82 — PBS introduction

**Monday — PBS theory**
- [ ] Read PBS (Proposer-Builder Separation) spec
- [ ] Commit notes

**Tuesday — MEV-Boost architecture**
- [ ] Read MEV-Boost architecture
- [ ] Commit notes

**Wednesday — Builder API spec**
- [ ] Read Builder API specification
- [ ] Commit notes

**Thursday — Builder API in reth**
- [ ] Check reth's builder API support
- [ ] Commit notes

**Friday — Crate: Builder API compat**
- [ ] Design builder API support
- [ ] Commit + log

**Saturday — Implementation start**
- [ ] Begin builder API endpoints
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 83 — Builder API implementation

**Monday — Header submissions**
- [ ] Implement header submission flow
- [ ] Commit + log

**Tuesday — Block submissions**
- [ ] Implement block submission flow
- [ ] Commit + log

**Wednesday — Builder client**
- [ ] Implement builder client perspective
- [ ] Commit + log

**Thursday — Builder integration tests**
- [ ] Commit + log

**Friday — Reth PR**
- [ ] Commit + log

**Saturday — Flashbots docs**
- [ ] Study Flashbots additional docs
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 84 — Test harness

**Monday — Test harness design**
- [ ] Design CL/EL test harness
- [ ] Commit notes

**Tuesday — Deterministic CL**
- [ ] Implement mock CL for testing
- [ ] Commit + log

**Wednesday — Scenario DSL**
- [ ] Define DSL for test scenarios
- [ ] Commit + log

**Thursday — Reorg scenarios**
- [ ] Reorg simulation tests
- [ ] Commit + log

**Friday — Engine API conformance**
- [ ] Run crate against spec conformance tests
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + End Month 21 review**

---

## Month 22: Cross-Subsystem Features + Integration Push

### Week 85 — Three-crate integration push

**Monday — Integration architecture**
- [ ] Design toy execution client using all 3 crates
- [ ] Commit + log

**Tuesday — Boot sequence**
- [ ] Implement node startup
- [ ] Commit + log

**Wednesday — Engine API → execution → storage flow**
- [ ] End-to-end flow
- [ ] Commit + log

**Thursday — Sync from testnet**
- [ ] Attempt sync from Sepolia using own stack
- [ ] Commit + log

**Friday — Debug failures**
- [ ] Fix issues
- [ ] Commit + log

**Saturday — More debugging**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 86 — Cross-subsystem reth feature

**Monday — Feature identification**
- [ ] Find reth feature touching engine + storage
- [ ] Commit notes

**Tuesday — Design**
- [ ] Commit notes

**Wednesday — Implementation**
- [ ] Commit + log

**Thursday — Continue**
- [ ] Commit + log

**Friday — Tests**
- [ ] Commit + log

**Saturday — Submit**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 87 — PR reviews velocity

**Monday — Review 2 PRs substantively**
- [ ] Commit notes

**Tuesday — Review 2 more**
- [ ] Commit notes

**Wednesday — Review discussion comments**
- [ ] Engage design discussions
- [ ] Commit notes

**Thursday — Reth PR**
- [ ] Commit + log

**Friday — Review 2 more**
- [ ] Commit notes

**Saturday — Crate maintenance**
- [ ] All three crates
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 88 — Devcon attendance

**Monday-Friday — Devcon (dates vary)**
- [ ] Attend Devcon
- [ ] Meet maintainers in person
- [ ] Side events
- [ ] Notes

**Saturday — Travel home**

**Sunday — Post-conference ritual**
- [ ] Update tracker
- [ ] Follow-ups

---

## Month 23: Mentorship + RFC Work

### Week 89 — RFC consideration

**Monday — Identify RFC opportunity**
- [ ] Find area needing design doc
- [ ] Commit notes

**Tuesday — Draft RFC**
- [ ] Write initial draft
- [ ] Commit notes

**Wednesday — Refine RFC**
- [ ] Iterate
- [ ] Commit notes

**Thursday — Post RFC**
- [ ] Post as GitHub discussion
- [ ] Commit notes

**Friday — Respond to feedback**
- [ ] Engage commenters
- [ ] Commit notes

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 90 — Mentorship practice

**Monday — Identify newcomer**
- [ ] Find newer contributor in Telegram
- [ ] Offer help on their first PR
- [ ] Commit notes

**Tuesday — Help them**
- [ ] Pair review
- [ ] Commit notes

**Wednesday — Another mentee**
- [ ] Help another newcomer
- [ ] Commit notes

**Thursday — Crate PR**
- [ ] Reth contribution
- [ ] Commit + log

**Friday — Consensus-engine v1.0 prep**
- [ ] API review
- [ ] Commit + log

**Saturday — Docs pass**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 91 — consensus-engine v1.0 ship

**Monday — Final benchmarks**
- [ ] Commit + log

**Tuesday — DESIGN.md**
- [ ] Commit + log

**Wednesday — Release tag**
- [ ] v1.0 tag
- [ ] Commit + log

**Thursday — Integration example**
- [ ] Full 3-crate example
- [ ] Commit + log

**Friday — Blog consideration**
- [ ] If ready, draft consensus-engine post
- [ ] Commit + log

**Saturday — Reth PR**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 92 — Recognition push

**Monday — Engage major discussions**
- [ ] Architecture-level discussion contributions
- [ ] Commit notes

**Tuesday — More PR reviews**
- [ ] 5+ substantive reviews
- [ ] Commit notes

**Wednesday — Second RFC**
- [ ] If applicable, another design proposal
- [ ] Commit notes

**Thursday — Reth PR**
- [ ] Commit + log

**Friday — Maintainer touch points**
- [ ] Engage each target maintainer at least once
- [ ] Commit notes

**Saturday — End Month 23**
- [ ] Commit + log

**Sunday — Rest + End Month 23 review**

---

## Month 24: Phase 5 Close + Reassessment

### Week 93 — Final feature push

**Monday — Feature identification**
- [ ] Last major reth feature for Phase 5
- [ ] Commit notes

**Tuesday — Implementation**
- [ ] Commit + log

**Wednesday — Continue**
- [ ] Commit + log

**Thursday — Tests**
- [ ] Commit + log

**Friday — Submit**
- [ ] Commit + log

**Saturday — Reviews**
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 94 — Final PR push

**Monday — PR volume**
- [ ] Multiple smaller PRs
- [ ] Commit + log

**Tuesday — Continue**
- [ ] Commit + log

**Wednesday — Reviews given**
- [ ] 5+ reviews
- [ ] Commit notes

**Thursday — Continue**
- [ ] Commit + log

**Friday — Final PRs**
- [ ] Commit + log

**Saturday — Wrap up**
- [ ] All outstanding items
- [ ] Commit + log

**Sunday — Rest + Weekly Ritual**

---

### Week 95 — Reassessment preparation

**Monday — Data collection**
- [ ] Count all PRs merged
- [ ] List all features shipped
- [ ] Update maintainer tracker
- [ ] Commit notes

**Tuesday — Signal assessment**
- [ ] Any approaches from companies?
- [ ] Any mentions by maintainers?
- [ ] Commit notes

**Wednesday — Three crates assessment**
- [ ] Quality of each crate
- [ ] Commit notes

**Thursday — Energy assessment**
- [ ] Sustainability check
- [ ] Commit notes

**Friday — Post-reth pull**
- [ ] Chronicle Queue / matching engine urges?
- [ ] Commit notes

**Saturday — Market state**
- [ ] Crypto cycle, Rust infra hiring climate
- [ ] Commit notes

**Sunday — Rest + Weekly Ritual**

---

### Week 96 — Month 24 Decision

**Monday — Path A analysis**
- [ ] Extend reth for 6-12 months
- [ ] What would this look like?
- [ ] Commit notes

**Tuesday — Path B analysis**
- [ ] Pivot to post-reth (Chronicle Queue etc)
- [ ] What would this look like?
- [ ] Commit notes

**Wednesday — Path C analysis**
- [ ] Catch-up if slipped
- [ ] What would this look like?
- [ ] Commit notes

**Thursday — Decision**
- [ ] Choose path based on signals
- [ ] Commit notes

**Friday — Month 25-30 rough plan**
- [ ] Draft based on chosen path
- [ ] Commit notes

**Saturday — Phase 5 close**
- [ ] Full Phase 5 review
- [ ] Commit + log

**Sunday — End 24-month plan**
- [ ] Celebrate milestone
- [ ] Prep for new chapter

---

# Tracking Sections

## Daily Log

```
| Date | Hrs | Phase | Focus | Output | Energy |
|------|-----|-------|-------|--------|--------|
```

Fill one row per day.

---

## North Star Metrics

| Metric | M6 | M12 | M18 | M24 |
|--------|----|----|----|----|
| Paradigm ecosystem PRs merged | 10 | 25 | 50 | 80 |
| Reth PRs merged | 0 | 15 | 35 | 60 |
| Storage/Trie PRs | 0 | 10 | 20 | 30 |
| Execution PRs (revm + reth evm) | 0 | 3 | 10 | 20 |
| Consensus/Engine PRs | 0 | 0 | 3 | 10 |
| PR reviews given (substantive) | 0 | 10 | 40 | 100 |
| Features led end-to-end | 0 | 0 | 1 | 3 |
| Production crates shipped | 0 | 1 | 2 | 3 |
| Direct relationships with maintainers | 1 | 3 | 5 | 8 |
| Conferences attended | 0 | 0 | 0 | 1 |

---

## Open Questions

*Running list. Close what resolves, carry what doesn't. If survives 2 weeks, dedicated slot.*

- [ ]
- [ ]

---

## Maintainer Relationship Tracker

| Name | Role | First interaction | Last interaction | Depth 0-5 | Notes |
|------|------|------------------|-----------------|-----------|-------|
| Matthias Seitz (mattsse_) | Core reth | — | — | 0 | Target primary mentor |
| Georgios Konstantopoulos (gakonst) | CTO Paradigm | — | — | 0 | Ecosystem leader |
| Dan Cline | Core reth | — | — | 0 | Storage/trie area |
| Oliver Nordbjerg | Core reth | — | — | 0 | — |
| Roman Krasiuk | Core reth | — | — | 0 | — |
| Dragan Rakita | Core reth / revm author | — | — | 0 | Critical Phase 4 |
| joshieDo | Core reth | — | — | 0 | — |

Depth: 0=none, 1=reviewed PR, 2=back-and-forth, 3=tags you for area reviews, 4=DM relationship, 5=co-design

---

## Risk Register

| Risk | Prob | Mitigation | Status |
|------|------|-----------|--------|
| Rust foundations extend Phase 1 | 70% | Budget +4 wk, weekly monitor | — |
| Reth PR cycles slow | 80% | Many small PRs in parallel | — |
| Reth major arch change | 60% | Telegram presence, release notes | — |
| Multiplier spike | 70% | Coast mode, 4h floor | — |
| Burnout M8-14 | 80% | Rest weeks, energy monitor | — |
| Conference budget delay | 30% | Year 1 savings earmarked | — |
| Family emergency | 40% | Accept slip, adjust | — |
| Motivation dip M12-14 | 70% | Pre-commit to Phase 4 | — |
| Crypto winter | 40% | Storage/exec portable | — |
| Crate scope creep | 60% | Lock scope at phase start | — |

---

## Principles

1. Deliverables over hours. 5h target, 4h floor. Done at 3h → rest. Stuck at 6h → diagnose.
2. Three production crates are the real output. Phase 1-2 exercises are disposable.
3. Depth over breadth. 3 subsystems mastered > 6 shallow.
4. Code reading > code writing in Phase 3+.
5. Ship imperfect > perfect never.
6. AI leverage for architecture research.
7. Blogging optional. Write when material earns essay.
8. Post-reth trajectory deferred, not forgotten.
9. Conferences non-negotiable Year 2.
10. Multiplier is infrastructure. Coast mode.
11. Energy is the only real budget. Sleep 7h, fitness 3x/week minimum.
12. Month 24 is a decision point.
13. Scope discipline on crates. No feature creep.

---

*Plan is a skeleton. Adjust weekly. Review monthly. Recalibrate quarterly. Reassess at Month 24.*