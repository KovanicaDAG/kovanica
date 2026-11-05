//! KovanicaHash Proof-of-Work algorithm (ADR 0003).
//!
//! KovanicaHash is a memory-hard proof-of-work algorithm designed for Kovanica
//! (KVNC) hybrid PoW + GHOSTDAG consensus (Chain ID 5 / testnet track).
//!
//! # Architecture & Phases
//!
//! 1. **Seed Phase**:
//!    The canonical block header preimage (132+ bytes) is hashed using BLAKE3 in
//!    keyed mode under domain `KVNC-POW-SEED-v1` to produce a 32-byte seed.
//!
//! 2. **Memory Mix Phase (2 MiB)**:
//!    A 2 MiB scratch buffer (65,536 chunks of 32 bytes) is deterministically
//!    generated from the seed using BLAKE3 in extendable output (XOF) mode.
//!    The state then traverses and mutates the buffer over 1,024 pseudo-random
//!    mixing rounds, indexing memory chunks non-linearly with word-level bit
//!    rotations, arithmetic wrapping additions, and memory write-backs.
//!
//! 3. **Finalize Phase**:
//!    A final keyed BLAKE3 compression digests the seed, mixed state, and memory
//!    boundary samples under domain `KVNC-POW-FINL-v1` to produce the final
//!    32-byte [`Hash`].
//!
//! # Architecture Invariants
//!
//! - Strictly little-endian across all integer operations (x86_64, aarch64 identical).
//! - Exactly 2 MiB RAM footprint per [`KovanicaHasher`] context (safe for 2–4 GB VPS).
//! - Zero memory allocations in [`KovanicaHasher::hash`] hot path (reusable buffer).

use crate::hash::Hash;

/// Total memory mix buffer size: 2 MiB (2,097,152 bytes).
pub const KOVANICA_HASH_MEMORY_SIZE: usize = 2 * 1024 * 1024;

/// Memory chunk size: 32 bytes.
pub const KOVANICA_HASH_CHUNK_SIZE: usize = 32;

/// Number of 32-byte chunks in the 2 MiB buffer: 65,536 (2^16).
pub const KOVANICA_HASH_NUM_CHUNKS: usize = KOVANICA_HASH_MEMORY_SIZE / KOVANICA_HASH_CHUNK_SIZE;

/// Number of pseudo-random memory mix rounds: 1,024.
pub const KOVANICA_HASH_MIX_ROUNDS: usize = 1_024;

/// Domain separator tag for the seed phase (16 bytes).
pub const DOMAIN_KOVANICA_HASH_SEED: &[u8; 16] = b"KVNC-POW-SEED-v1";

/// Domain separator tag for the finalize phase (16 bytes).
pub const DOMAIN_KOVANICA_HASH_FINAL: &[u8; 16] = b"KVNC-POW-FINL-v1";

/// Reusable hasher context maintaining a preallocated 2 MiB memory buffer.
///
/// Designed for mining loops to prevent per-hash heap allocations.
pub struct KovanicaHasher {
    memory: Box<[u8]>,
}

impl Default for KovanicaHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl KovanicaHasher {
    /// Create a new hasher with an allocated 2 MiB heap buffer.
    ///
    /// Allocates heap memory directly via `Vec` to avoid stack overflow
    /// on threads with small default stack sizes (e.g. Windows 1 MiB).
    #[must_use]
    pub fn new() -> Self {
        Self {
            memory: vec![0u8; KOVANICA_HASH_MEMORY_SIZE].into_boxed_slice(),
        }
    }

    /// Compute KovanicaHash for the given header preimage using this context's buffer.
    ///
    /// Performs zero heap allocations.
    pub fn hash(&mut self, header_preimage: &[u8]) -> Hash {
        // --- 1. Seed Phase ---
        let mut seed_key = [0u8; 32];
        seed_key[..DOMAIN_KOVANICA_HASH_SEED.len()].copy_from_slice(DOMAIN_KOVANICA_HASH_SEED);
        let seed = blake3::keyed_hash(&seed_key, header_preimage);
        let seed_bytes = *seed.as_bytes();

        // --- 2. Memory Mix Phase: Buffer Generation ---
        let mut xof = blake3::Hasher::new_keyed(&seed_bytes).finalize_xof();
        xof.fill(&mut self.memory[..]);

        // --- 2b. Memory Mix Phase: Wandering & Mutation ---
        let mut state = seed_bytes;

        for step in 0..KOVANICA_HASH_MIX_ROUNDS {
            // Unbiased index into 65,536 chunks using the first 2 bytes of state
            let chunk_idx = u16::from_le_bytes([state[0], state[1]]) as usize;
            let offset = chunk_idx * KOVANICA_HASH_CHUNK_SIZE;

            let chunk: [u8; 32] = self.memory[offset..offset + 32]
                .try_into()
                .expect("chunk slice bounds are verified");

            // Extract four 64-bit little-endian words from state
            let s0 = u64::from_le_bytes(state[0..8].try_into().unwrap());
            let s1 = u64::from_le_bytes(state[8..16].try_into().unwrap());
            let s2 = u64::from_le_bytes(state[16..24].try_into().unwrap());
            let s3 = u64::from_le_bytes(state[24..32].try_into().unwrap());

            // Extract four 64-bit little-endian words from chunk
            let c0 = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
            let c1 = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
            let c2 = u64::from_le_bytes(chunk[16..24].try_into().unwrap());
            let c3 = u64::from_le_bytes(chunk[24..32].try_into().unwrap());

            // Non-linear round operations: wrapping addition, XOR, bit rotations
            let m0 = (s0 ^ c0).wrapping_add(c1).rotate_left(17);
            let m1 = (s1.wrapping_add(c1) ^ c2).rotate_left(29);
            let m2 = (s2 ^ c2).wrapping_add(c3).rotate_left(37);
            let m3 = (s3.wrapping_add(c3) ^ c0).rotate_left(43);

            state[0..8].copy_from_slice(&m0.to_le_bytes());
            state[8..16].copy_from_slice(&m1.to_le_bytes());
            state[16..24].copy_from_slice(&m2.to_le_bytes());
            state[24..32].copy_from_slice(&m3.to_le_bytes());

            // Write-back to memory (forces read-write memory dependency)
            self.memory[offset..offset + 32].copy_from_slice(&state);

            // Feed step counter into state
            let step_tag = (step as u64).to_le_bytes();
            for i in 0..8 {
                state[i] ^= step_tag[i];
            }
        }

        // --- 3. Finalize Phase ---
        let mut final_key = [0u8; 32];
        final_key[..DOMAIN_KOVANICA_HASH_FINAL.len()].copy_from_slice(DOMAIN_KOVANICA_HASH_FINAL);

        let mut finalize_hasher = blake3::Hasher::new_keyed(&final_key);
        finalize_hasher.update(&seed_bytes);
        finalize_hasher.update(&state);
        // Sample memory boundaries
        finalize_hasher.update(&self.memory[0..32]);
        finalize_hasher
            .update(&self.memory[KOVANICA_HASH_MEMORY_SIZE - 32..KOVANICA_HASH_MEMORY_SIZE]);

        let final_digest = finalize_hasher.finalize();
        Hash(*final_digest.as_bytes())
    }
}

/// Compute KovanicaHash for the given header preimage.
///
/// Convenient standalone function. For mining or high-throughput hashing,
/// create and reuse a [`KovanicaHasher`] instead.
#[must_use]
pub fn kovanica_hash(header_preimage: &[u8]) -> Hash {
    let mut hasher = KovanicaHasher::new();
    hasher.hash(header_preimage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kovanica_hash_buffer_size_invariants() {
        assert_eq!(KOVANICA_HASH_MEMORY_SIZE, 2 * 1024 * 1024);
        assert_eq!(KOVANICA_HASH_CHUNK_SIZE, 32);
        assert_eq!(KOVANICA_HASH_NUM_CHUNKS, 65_536);
        assert_eq!(
            KOVANICA_HASH_NUM_CHUNKS * KOVANICA_HASH_CHUNK_SIZE,
            KOVANICA_HASH_MEMORY_SIZE
        );
    }

    #[test]
    fn kovanica_hash_deterministic_and_reproducible() {
        let input1 = b"test header preimage for kovanicahash 1";
        let input2 = b"test header preimage for kovanicahash 2";

        let hash1_a = kovanica_hash(input1);
        let hash1_b = kovanica_hash(input1);
        assert_eq!(hash1_a, hash1_b);

        let hash2_a = kovanica_hash(input2);
        assert_ne!(hash1_a, hash2_a);

        // Hasher reuse gives same results as fresh hasher
        let mut reusable = KovanicaHasher::new();
        assert_eq!(reusable.hash(input1), hash1_a);
        assert_eq!(reusable.hash(input2), hash2_a);
        assert_eq!(reusable.hash(input1), hash1_a);
    }

    #[test]
    fn kovanica_hash_golden_vectors() {
        // Empty preimage
        let h_empty = kovanica_hash(b"");
        assert_eq!(
            h_empty.to_hex(),
            "05cca3939da23b1b771692875380dd70d953c98ba5498468a683add6aaa2a623"
        );

        // Fixed test preimage 1 (132 bytes like single parent header)
        let preimage_132 = [0x5au8; 132];
        let h_132 = kovanica_hash(&preimage_132);
        assert_eq!(
            h_132.to_hex(),
            "cce556a21d6efee5463d39911877bab564d5fbe70f6b07fea1dfad2c1ccb2db4"
        );

        // Fixed test preimage 2
        let preimage_fixed = b"KVNC-CHAIN5-HYBRID-POW-TEST-PREIMAGE";
        let h_fixed = kovanica_hash(preimage_fixed);
        assert_eq!(
            h_fixed.to_hex(),
            "ba2bbe92ef378bdbf970bdde68a2d5cffaaea4c5aa02c9146ddd9b1765b8b262"
        );
    }
}
