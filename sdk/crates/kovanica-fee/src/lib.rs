//! Fee estimation aligned with RFC-006 tokenomics.
//!
//! Fee floor: `max(1, subsidy / 500_000)` atoms per byte.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

use kovanica_types::Amount;

/// Divisor used in the protocol fee floor formula.
pub const FEE_FLOOR_DIVISOR: u64 = 500_000;

/// Estimate fee for a transaction of the given serialized size.
///
/// `subsidy` is the current block subsidy in atoms (from `/api/head` or chain state).
/// Returns at least 1 atom per byte when subsidy is large enough, otherwise 1 atom total minimum.
pub fn estimate(size_bytes: u64, subsidy_atoms: u64) -> Amount {
    let per_byte = std::cmp::max(1, subsidy_atoms / FEE_FLOOR_DIVISOR);
    let fee = size_bytes.saturating_mul(per_byte);
    Amount::from_atoms(std::cmp::max(1, fee))
}

/// Convenience when you already know the minimum fee advertised by the node.
pub fn estimate_with_min(size_bytes: u64, min_fee_atoms: u64) -> Amount {
    let calculated = size_bytes; // 1 atom/byte fallback if min is unknown scale
    Amount::from_atoms(std::cmp::max(min_fee_atoms, calculated))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_with_large_subsidy() {
        // 10 KVNC subsidy = 1_000_000_000 atoms → floor = 2000 atoms/byte
        let subsidy = 1_000_000_000u64;
        let fee = estimate(250, subsidy);
        assert_eq!(fee.atoms(), 250 * 2000);
    }

    #[test]
    fn never_zero() {
        let fee = estimate(1, 0);
        assert!(fee.atoms() >= 1);
    }
}
