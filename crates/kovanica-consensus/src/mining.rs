//! Proof-of-work mining primitives for Hybrid PoW + GHOSTDAG (Chain ID 5).
//!
//! Provides synchronous and asynchronous mining functions that iterate over nonces
//! until a candidate [`BlockHeader`] satisfies its declared [`BlockHeader::target_bits`].

use kovanica_crypto::{kovanica_hash, KovanicaHasher};
use kovanica_types::block::{check_pow_hash_meets_target, compact_to_target, BlockHeader};

/// Mine a block header synchronously by searching for a valid nonce up to `max_iterations`.
///
/// Returns `Some(BlockHeader)` with `pow_hash` set if a valid nonce is found, or `None`
/// if `max_iterations` is exhausted without finding a solution or if target is invalid.
#[must_use]
pub fn mine_header(mut header: BlockHeader, max_iterations: u64) -> Option<BlockHeader> {
    let target = compact_to_target(header.target_bits);
    if target == [0u8; 32] {
        return None;
    }

    let mut hasher = KovanicaHasher::new();

    for _ in 0..max_iterations {
        let hash = hasher.hash(&header.preimage());
        if check_pow_hash_meets_target(&hash, header.target_bits) {
            header.pow_hash = Some(hash);
            return Some(header);
        }
        header.nonce = header.nonce.wrapping_add(1);
    }

    None
}

/// Asynchronously mine a block header using a background blocking task.
pub async fn mine_header_async(header: BlockHeader, max_iterations: u64) -> Option<BlockHeader> {
    tokio::task::spawn_blocking(move || mine_header(header, max_iterations))
        .await
        .ok()
        .flatten()
}

/// Dedicated CPU mining worker retaining a preallocated 2 MiB hasher context.
pub struct MiningWorker {
    hasher: KovanicaHasher,
}

impl Default for MiningWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl MiningWorker {
    /// Create a new worker allocating the 2 MiB memory mix buffer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            hasher: KovanicaHasher::new(),
        }
    }

    /// Attempt to mine `header` up to `max_iterations`.
    pub fn mine_header(
        &mut self,
        mut header: BlockHeader,
        max_iterations: u64,
    ) -> Option<BlockHeader> {
        let target = compact_to_target(header.target_bits);
        if target == [0u8; 32] {
            return None;
        }

        for _ in 0..max_iterations {
            let hash = self.hasher.hash(&header.preimage());
            if check_pow_hash_meets_target(&hash, header.target_bits) {
                header.pow_hash = Some(hash);
                return Some(header);
            }
            header.nonce = header.nonce.wrapping_add(1);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::hash::Hash;

    #[test]
    fn mine_header_finds_valid_nonce_high_target() {
        // High target: top byte <= 0x7f (~50% chance per nonce)
        let header = BlockHeader::new(
            5,
            1,
            0,
            vec![],
            Hash::zero(),
            1_700_000_000_000,
            0x207fffff,
            0,
        );

        let mut worker = MiningWorker::new();
        let mined = worker
            .mine_header(header, 1_000)
            .expect("must find solution within 1000 iterations for high target");

        // Header must have pow_hash set
        assert!(mined.pow_hash.is_some());
        let declared = mined.pow_hash.unwrap();

        // Must match independent kovanica_hash calculation
        assert_eq!(declared, kovanica_hash(&mined.preimage()));

        // Header verification must pass cleanly
        assert!(mined.verify_pow().is_ok());
    }

    #[tokio::test]
    async fn mine_header_async_finds_valid_nonce() {
        let header = BlockHeader::new(
            5,
            2,
            1,
            vec![],
            Hash::zero(),
            1_700_000_004_000,
            0x207fffff,
            42,
        );

        let mined = mine_header_async(header, 1_000)
            .await
            .expect("async mining should find solution");

        assert!(mined.verify_pow().is_ok());
    }

    #[test]
    fn mine_header_exhausted_iterations_returns_none() {
        let header = BlockHeader::new(
            5,
            1,
            0,
            vec![],
            Hash::zero(),
            1_700_000_000_000,
            0x04000001, // extremely difficult target
            0,
        );

        // With only 2 iterations on extreme target, must return None
        let mined = mine_header(header, 2);
        assert!(mined.is_none());
    }
}
