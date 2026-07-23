// R3: the orphan rule forbids `From` here. `From` is `core`'s, `eth_primitives::B256` is
// `eth-primitives`', `alloy_primitives::B256` is `alloy-primitives`' — nothing is local to
// `etherscanlite`, so this must not compile. The local `FromAlloy` trait exists precisely
// to make the same conversion legal.
use eth_primitives::{FixedBytes, B256};

impl From<alloy_primitives::B256> for B256 {
    fn from(value: alloy_primitives::B256) -> Self {
        FixedBytes(value.0)
    }
}

fn main() {}
