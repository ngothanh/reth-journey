//! Shared cross-thread state about Ethereum's canonical chain.
//!
//! This crate is the first one in the workspace that **combines** the
//! pure-data layer (`eth-primitives`) with the sync-primitive layer
//! (`concurrent`). It mirrors reth's `reth-chain-state` and exists so that
//! `eth-primitives` can stay free of sync dependencies (matching the
//! `alloy-primitives` mirror discipline).
//!
//! ## Inhabitants
//!
//! - [`chain_head`] — `ChainHead` SeqLock over `(B256, u64)`, the canonical
//!   chain tip pointer. Many readers (RPC, peers), one writer (engine).
//!
//! Future siblings (post-W4): canonical-chain notifications, safe/finalized
//! tip pointers, reorg broadcaster, sync-status tracker.

pub mod chain_head;
