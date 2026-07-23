use eth_primitives::{Address, Bytes, FixedBytes, B256, U256};

pub(crate) trait FromAlloy<T> {
    fn from(alloy: T) -> Self;
}

pub(crate) trait TryFromAlloy<T>: Sized {
    type Error;
    fn try_from(alloy: T) -> Result<Self, Self::Error>;
}

/// The reverse direction. Needed by the round-trip tests, and blocked by the orphan rule
/// for the same reason as `FromAlloy` — so it is likewise a trait defined here.
pub(crate) trait ToAlloy<T> {
    fn to_alloy(self) -> T;
}

/// Errors raised crossing the RPC boundary.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BoundaryError {
    /// A JSON-RPC field that is `null` on pending blocks and transactions was required.
    MissingField(&'static str),
    /// An RPC quantity did not fit the narrower local type.
    Overflow(&'static str),
}

impl core::fmt::Display for BoundaryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingField(name) => write!(f, "RPC response field `{name}` was null"),
            Self::Overflow(name) => write!(f, "RPC value for `{name}` overflows the local type"),
        }
    }
}

impl std::error::Error for BoundaryError {}

impl FromAlloy<alloy_primitives::B256> for B256 {
    fn from(alloy: alloy_primitives::B256) -> Self {
        // Byte copy, not a transmute: if alloy ever changes `FixedBytes`'s inner repr
        // this stops compiling, instead of silently yielding byte-reversed hashes.
        FixedBytes(alloy.0)
    }
}

impl ToAlloy<alloy_primitives::B256> for B256 {
    fn to_alloy(self) -> alloy_primitives::B256 {
        alloy_primitives::B256::from(self.0)
    }
}

impl TryFromAlloy<Option<alloy_primitives::B256>> for B256 {
    type Error = BoundaryError;

    fn try_from(alloy: Option<alloy_primitives::B256>) -> Result<Self, Self::Error> {
        alloy
            .map(<B256 as FromAlloy<_>>::from)
            .ok_or(BoundaryError::MissingField("blockHash"))
    }
}

// --------------------------------------------------------------------- Address

/// Needs its own impl rather than riding on the `B256` one: alloy's `Address` is a
/// distinct newtype wrapping `FixedBytes<20>`, whereas ours is a `FixedBytes<20>` alias.
/// So the inner array is one level deeper on the alloy side.
impl FromAlloy<alloy_primitives::Address> for Address {
    fn from(alloy: alloy_primitives::Address) -> Self {
        FixedBytes(alloy.0 .0)
    }
}

impl ToAlloy<alloy_primitives::Address> for Address {
    fn to_alloy(self) -> alloy_primitives::Address {
        alloy_primitives::Address::from(self.0)
    }
}

// ----------------------------------------------------------------------- Bytes

/// Copies. Ours is a refcounted buffer with its own allocation strategy and alloy's
/// wraps `bytes::Bytes`; there is no way to hand ownership across without one of them
/// knowing about the other's vtable. Payloads here are calldata-sized, and this runs
/// once per RPC response, not per byte.
impl FromAlloy<alloy_primitives::Bytes> for Bytes {
    fn from(alloy: alloy_primitives::Bytes) -> Self {
        Bytes::from_vec(alloy.to_vec())
    }
}

impl ToAlloy<alloy_primitives::Bytes> for Bytes {
    fn to_alloy(self) -> alloy_primitives::Bytes {
        alloy_primitives::Bytes::from(self.to_vec())
    }
}

// ------------------------------------------------------------------------ U256

/// Identity, and that is a finding rather than a convenience — see the module note on
/// `U256` below. Kept for symmetry so callsites read the same as every other type and
/// so there is one place to edit if the two ever diverge.
impl FromAlloy<alloy_primitives::U256> for U256 {
    fn from(alloy: alloy_primitives::U256) -> Self {
        alloy
    }
}

impl ToAlloy<alloy_primitives::U256> for U256 {
    fn to_alloy(self) -> alloy_primitives::U256 {
        self
    }
}

/// The real narrowing at this boundary. Nonce, gas, and block number are `u64` locally
/// but arrive as 256-bit quantities; a silent truncation here signs the wrong
/// transaction, so it is an error rather than an `as` cast.
impl TryFromAlloy<alloy_primitives::U256> for u64 {
    type Error = BoundaryError;

    fn try_from(alloy: alloy_primitives::U256) -> Result<Self, Self::Error> {
        // Fully qualified: bare `u64::try_from` is ambiguous between `core::TryFrom`
        // and this trait, since both methods are named `try_from` and both are in scope.
        <u64 as TryFrom<alloy_primitives::U256>>::try_from(alloy)
            .map_err(|_| BoundaryError::Overflow("u64 quantity"))
    }
}

// ------------------------------------------------------------------ Transaction

/// The fields R2 prints, in local types.
///
/// `eth-primitives` has no transaction type yet — that arrives with `TxEnvelope` in W9 —
/// so the local shape lives here. When W9 lands, this struct is what gets replaced; the
/// conversion below is the only thing that has to change.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TxSummary {
    pub(crate) hash: B256,
    pub(crate) value: U256,
    pub(crate) gas: u64,
}

impl TryFromAlloy<alloy_rpc_types_eth::Transaction> for TxSummary {
    type Error = BoundaryError;

    fn try_from(alloy: alloy_rpc_types_eth::Transaction) -> Result<Self, Self::Error> {
        use alloy_consensus::Transaction as _;

        Ok(TxSummary {
            hash: <B256 as FromAlloy<_>>::from(*alloy.inner.hash()),
            value: <U256 as FromAlloy<_>>::from(alloy.inner.value()),
            gas: alloy.inner.gas_limit(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R3 round trips. Each asserts the pair is lossless in both directions — a
    /// conversion that only works one way corrupts silently on write paths.
    #[test]
    fn b256_round_trips() {
        let alloy = alloy_primitives::B256::from([0xab; 32]);
        let local = <B256 as FromAlloy<_>>::from(alloy);
        assert_eq!(local.0, [0xab; 32]);
        assert_eq!(local.to_alloy(), alloy);
    }

    #[test]
    fn address_round_trips() {
        let alloy = alloy_primitives::Address::from([0x11; 20]);
        let local = <Address as FromAlloy<_>>::from(alloy);
        assert_eq!(local.0, [0x11; 20]);
        assert_eq!(local.to_alloy(), alloy);
    }

    #[test]
    fn bytes_round_trips() {
        let alloy = alloy_primitives::Bytes::from_static(&[0xde, 0xad, 0xbe, 0xef]);
        let local = <Bytes as FromAlloy<_>>::from(alloy.clone());
        assert_eq!(&local[..], &[0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(local.to_alloy(), alloy);
    }

    #[test]
    fn u256_round_trips_at_the_edges() {
        for alloy in [
            alloy_primitives::U256::ZERO,
            alloy_primitives::U256::from(1u64),
            alloy_primitives::U256::MAX,
        ] {
            let local = <U256 as FromAlloy<_>>::from(alloy);
            assert_eq!(local.to_alloy(), alloy);
        }
    }

    #[test]
    fn optional_b256_rejects_null() {
        let present = Some(alloy_primitives::B256::from([0x01; 32]));
        assert!(<B256 as TryFromAlloy<_>>::try_from(present).is_ok());

        assert_eq!(
            <B256 as TryFromAlloy<Option<alloy_primitives::B256>>>::try_from(None).unwrap_err(),
            BoundaryError::MissingField("blockHash")
        );
    }

    #[test]
    fn u256_to_u64_rejects_overflow() {
        // The largest value that fits, and the first that does not. A silent truncation
        // here would turn a nonce into a different, valid-looking nonce.
        let fits = alloy_primitives::U256::from(u64::MAX);
        assert_eq!(<u64 as TryFromAlloy<_>>::try_from(fits), Ok(u64::MAX));

        let overflows = alloy_primitives::U256::from(u64::MAX) + alloy_primitives::U256::from(1u64);
        assert_eq!(
            <u64 as TryFromAlloy<_>>::try_from(overflows),
            Err(BoundaryError::Overflow("u64 quantity"))
        );
    }
}
