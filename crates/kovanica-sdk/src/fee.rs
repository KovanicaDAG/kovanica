//! Fee helpers.

pub use kovanica_fee::{estimate_call_gas, fee_from_gas, min_fee_for_tx};

/// Minimum fee floor (atoms) per the protocol: at least 1 atom/byte.
pub fn min_fee_bytes(len: usize) -> u64 {
    kovanica_fee::min_fee_for_tx(len)
}

pub fn estimate_transfer() -> u64 {
    1000
}
