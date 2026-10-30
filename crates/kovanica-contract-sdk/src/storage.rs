//! Shared contract helpers: storage-key prefix and bincode packing.

use alloc::vec::Vec;
use kovanica_common::ContractError;
use serde::{Deserialize, Serialize};

/// `kvnc/v1` — the only top-level storage namespace.
pub const KVNC_V1: &[u8] = b"kvnc/v1";

/// Prefix for all v1 table rows (`kvnc/v1/<table>/<record>`).
pub fn storage_key(table: &[u8], record: &[u8]) -> Vec<u8> {
    let mut k = Vec::with_capacity(KVNC_V1.len() + table.len() + 1 + record.len());
    k.extend_from_slice(KVNC_V1);
    k.push(b'/');
    k.extend_from_slice(table);
    k.push(b'/');
    k.extend_from_slice(record);
    k
}

/// Pack a value exactly like every other native contract does.
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, ContractError> {
    bincode::serialize(value)
        .map_err(|_| ContractError::Overflow)
}

/// Unpack a value exactly like every other native contract does.
pub fn decode<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, ContractError> {
    bincode::deserialize(bytes)
        .map_err(|_| ContractError::InvalidInput)
}

/// Pack a contract error into the wasm entry-point status word.
pub fn encode_error(err: &ContractError) -> i32 {
    -1 - err.code()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_key_roundtrip() {
        assert_eq!(
            storage_key(b"state", b"abc"),
            b"kvnc/v1/state/abc".to_vec()
        );
        assert_eq!(storage_key(b"", b""), b"kvnc/v1//".to_vec());
    }
}
