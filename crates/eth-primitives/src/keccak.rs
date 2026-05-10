use crate::{FixedBytes, B256};
use tiny_keccak::{Hasher, Keccak};

pub fn keccak256(bytes: impl AsRef<[u8]>) -> B256 {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(bytes.as_ref());
    hasher.finalize(&mut output);
    FixedBytes(output)
}
