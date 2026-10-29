//! Fee helpers.

/// Minimum fee floor (atoms) per the protocol: at least 1 atom/byte.
pub fn min_fee_bytes(len: usize) -> u64 {
    len as u64
}

pub fn estimate_transfer() -> u64 {
    1000
}
