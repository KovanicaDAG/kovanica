//! Difficulty Adjustment Algorithm (DAA) for Hybrid PoW + GHOSTDAG (Chain ID 2).
//!
//! # Specification
//!
//! Chain ID 2 targets a block interval of [`kovanica_params::TARGET_BLOCK_MS`] (4,000 ms).
//! The difficulty adjustment algorithm monitors a sliding window of recent block timestamps,
//! compares the actual elapsed duration against the target duration, and dynamically scales
//! the 256-bit target difficulty:
//!
//! $$\text{new\_target} = \text{old\_target} \times \frac{\text{actual\_duration\_ms}}{\text{target\_duration\_ms}}$$
//!
//! To prevent severe oscillation and difficulty manipulation:
//! - Adjustments per window are clamped by [`DaaConfig::max_adjustment_factor`] (default 4x / 0.25x).
//! - Target cannot exceed [`kovanica_params::MAX_TARGET_BITS`] (easiest difficulty ceiling).
//! - Target cannot fall below [`kovanica_params::MIN_TARGET_BITS`] (hardest difficulty floor).

use kovanica_dag::DagStore;
use kovanica_types::block::{compact_to_target, target_to_compact, BlockReference};
use kovanica_types::hash::Hash;
use std::collections::{HashSet, VecDeque};
use thiserror::Error;

/// Errors arising during required DAA target derivation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DaaError {
    /// Referenced parent block could not be read from storage.
    #[error("parent block {0} could not be read from consensus DAG storage: {1}")]
    ParentBlockUnreadable(Hash, String),
    /// Primary parent or ancestor block has invalid, missing, or corrupted PoW header.
    #[error("parent or ancestor block {0} at round {1} has invalid or missing PoW header: {2}")]
    InvalidParentHeader(Hash, u64, String),
}

/// Canonical genesis digest for round 0.
#[must_use]
pub fn canonical_genesis_digest() -> Hash {
    kovanica_types::block::StatementBlock::compute_digest(0, 0, &[], &[])
}

/// Check if a block reference identifies the canonical genesis block.
#[must_use]
pub fn is_canonical_genesis(round: u64, digest: &Hash) -> bool {
    round == 0 && *digest == canonical_genesis_digest()
}

/// Configuration parameters for difficulty adjustment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaaConfig {
    /// Target block time in milliseconds (default 4,000 ms).
    pub target_block_time_ms: u64,
    /// Number of blocks in the adjustment window (default 60).
    pub window_size: u64,
    /// Maximum allowed target bits (easiest difficulty ceiling).
    pub max_target_bits: u32,
    /// Minimum allowed target bits (hardest difficulty floor).
    pub min_target_bits: u32,
    /// Maximum scaling factor allowed in a single window adjustment (e.g. 4).
    pub max_adjustment_factor: u64,
}

impl Default for DaaConfig {
    fn default() -> Self {
        Self::hybrid_default()
    }
}

impl DaaConfig {
    /// Standard consensus parameters for Chain ID 2 (Hybrid PoW testnet).
    ///
    /// Pure protocol configuration strictly bound to canonical parameters:
    /// - Max target ceiling: [`kovanica_params::MAX_TARGET_BITS`] (`0x1f00_ffff`).
    /// - Min target floor: [`kovanica_params::MIN_TARGET_BITS`].
    /// - Target block time: [`kovanica_params::TARGET_BLOCK_MS`] (4,000 ms).
    /// - Adjustment window: [`kovanica_params::DAA_WINDOW_BLOCKS`] (60 blocks).
    /// - Max factor: 4x.
    ///
    /// Runtime process environment variables (e.g. `KOVANICA_TEST_TARGET_BITS`) are
    /// strictly ignored to ensure consensus determinism across all nodes.
    #[must_use]
    pub const fn hybrid_default() -> Self {
        Self {
            target_block_time_ms: kovanica_params::TARGET_BLOCK_MS,
            window_size: kovanica_params::DAA_WINDOW_BLOCKS,
            max_target_bits: kovanica_params::MAX_TARGET_BITS,
            min_target_bits: kovanica_params::MIN_TARGET_BITS,
            max_adjustment_factor: 4,
        }
    }
}

/// Multiply a 256-bit big-endian integer by `num: u64` and divide by `den: u64`.
///
/// Uses exact 320-bit intermediate multi-limb arithmetic with zero overflow
/// and division-by-zero safety.
#[must_use]
pub fn mul_div_u256(val: [u8; 32], num: u64, den: u64) -> [u8; 32] {
    if den == 0 {
        return val;
    }
    if num == 0 {
        return [0u8; 32];
    }
    if num == den {
        return val;
    }

    // Convert 256-bit big-endian byte array to 8 little-endian 32-bit limbs
    let mut limbs = [0u32; 8];
    for (i, limb) in limbs.iter_mut().enumerate() {
        let offset = 32 - (i + 1) * 4;
        *limb = u32::from_be_bytes([
            val[offset],
            val[offset + 1],
            val[offset + 2],
            val[offset + 3],
        ]);
    }

    // 1. Multiplication: 8 limbs * u64 -> up to 10 limbs (320 bits)
    let mut prod = [0u32; 10];
    let mut carry: u64 = 0;
    for (p, &limb) in prod.iter_mut().take(8).zip(&limbs) {
        let cur = (limb as u128) * (num as u128) + (carry as u128);
        *p = (cur & 0xffff_ffff) as u32;
        carry = (cur >> 32) as u64;
    }
    prod[8] = (carry & 0xffff_ffff) as u32;
    prod[9] = (carry >> 32) as u32;

    // 2. Division: 10 limbs / u64 -> quotient (up to 10 limbs)
    let mut quotient = [0u32; 10];
    let mut rem: u128 = 0;
    for i in (0..10).rev() {
        let cur = (rem << 32) | (prod[i] as u128);
        quotient[i] = (cur / (den as u128)) as u32;
        rem = cur % (den as u128);
    }

    // Overflow check: if quotient exceeds 256 bits, saturate to [0xff; 32]
    if quotient[8] != 0 || quotient[9] != 0 {
        return [0xff; 32];
    }

    // Convert 8 little-endian limbs back to 32 big-endian bytes
    let mut result = [0u8; 32];
    for (i, &q) in quotient.iter().take(8).enumerate() {
        let offset = 32 - (i + 1) * 4;
        let bytes = q.to_be_bytes();
        result[offset..offset + 4].copy_from_slice(&bytes);
    }
    result
}

/// Calculate the next difficulty target bits given recent block timestamps.
///
/// `block_timestamps` must be provided in ascending chronological order.
#[must_use]
pub fn calculate_next_target_bits(
    current_target_bits: u32,
    block_timestamps: &[u64],
    config: &DaaConfig,
) -> u32 {
    if block_timestamps.len() < 2 {
        return current_target_bits;
    }

    // Use at most `window_size` most recent timestamps
    let window_slice = if block_timestamps.len() as u64 > config.window_size {
        let start = block_timestamps.len() - config.window_size as usize;
        &block_timestamps[start..]
    } else {
        block_timestamps
    };

    let first_ts = *window_slice.first().unwrap();
    let last_ts = *window_slice.last().unwrap();

    // Actual duration elapsed in the window (minimum 1 ms)
    let actual_duration_ms = last_ts.saturating_sub(first_ts).max(1);

    // Target duration for (N - 1) block intervals
    let num_intervals = (window_slice.len() - 1) as u64;
    let target_duration_ms = num_intervals
        .saturating_mul(config.target_block_time_ms)
        .max(1);

    // Clamp adjustment factor to prevent extreme jumps
    let max_factor = config.max_adjustment_factor.max(1);
    let min_allowed_duration = target_duration_ms / max_factor;
    let max_allowed_duration = target_duration_ms.saturating_mul(max_factor);

    let clamped_actual = actual_duration_ms.clamp(min_allowed_duration, max_allowed_duration);

    // Current 256-bit target
    let current_target = compact_to_target(current_target_bits);
    if current_target == [0u8; 32] {
        return config.max_target_bits;
    }

    // Scale target: new = old * (clamped_actual / target_duration)
    let mut new_target = mul_div_u256(current_target, clamped_actual, target_duration_ms);

    // Clamp against max target (easiest difficulty)
    let max_target = compact_to_target(config.max_target_bits);
    if max_target != [0u8; 32] && new_target > max_target {
        new_target = max_target;
    }

    // Clamp against min target (hardest difficulty)
    let min_target = compact_to_target(config.min_target_bits);
    if min_target != [0u8; 32] && new_target < min_target {
        new_target = min_target;
    }

    target_to_compact(&new_target)
}

/// Select parent hashes from recent DAG tips for block authoring on Chain 2.
#[must_use]
pub fn select_parents_from_tips(dag_store: &DagStore, max_parents: usize) -> Vec<Hash> {
    let max_round = match dag_store.max_dag_round() {
        Ok(Some(r)) => r,
        _ => return Vec::new(),
    };

    let mut parents = Vec::new();
    let mut round = max_round;
    loop {
        if let Ok(mut blocks) = dag_store.get_blocks_by_round(round) {
            // Sort deterministically: highest round first, then author, then digest
            blocks.sort_by(|a, b| {
                (b.round, a.author, a.digest.0).cmp(&(a.round, b.author, b.digest.0))
            });
            for block in blocks {
                if !parents.contains(&block.digest) {
                    parents.push(block.digest);
                    if parents.len() >= max_parents {
                        return parents;
                    }
                }
            }
        }
        if round == 0 || parents.len() >= max_parents {
            break;
        }
        round = round.saturating_sub(1);
        if max_round.saturating_sub(round) > 2 {
            break;
        }
    }
    parents
}

/// Derive the required DAA difficulty target bits for a block given its parents and local DAG storage.
///
/// Consensus rules:
/// - Exact ancestry/window: Evaluates up to [`DaaConfig::window_size`] (60) ancestor block timestamps
///   reachable by following `parents` backwards through DAG history.
/// - Deterministic ordering: Traversal is bound entirely to the block's parent cone. Timestamps are
///   sorted chronologically in ascending order, ensuring that arrival order or current tip does not
///   affect the required target.
/// - Bootstrap:
///   * Empty parents (round 0 canonical genesis): returns `config.max_target_bits`.
///   * Primary parent without PoW header (e.g. canonical round 0 genesis with no PoW statements):
///     returns `config.max_target_bits`.
///   * Fewer than 2 timestamps (e.g. round 1, whose only parent is round 0 genesis with no PoW timestamp):
///     returns `current_target_bits` which defaults to `config.max_target_bits`.
///   * For non-empty parents, `current_target_bits` is derived from the primary parent (`parents[0]`).
/// - Strict storage validation: If a parent or ancestor in the window is missing or unreadable from storage,
///   or contains a malformed PoW header, returns `Err(DaaError)`. Never falls back to a permissive target.
pub fn required_target_bits(
    dag_store: &DagStore,
    parents: &[BlockReference],
    chain_id: u64,
    config: &DaaConfig,
) -> Result<u32, DaaError> {
    if parents.is_empty() {
        return Ok(config.max_target_bits);
    }

    // Verify all referenced direct parents exist in storage and round > 0 blocks have valid headers
    for p in parents {
        let block = dag_store
            .get_block(&p.digest)
            .map_err(|e| DaaError::ParentBlockUnreadable(p.digest, e.to_string()))?;
        if block.round == 0 {
            if !is_canonical_genesis(block.round, &p.digest) {
                if let Err(e) = block.extract_pow_header(chain_id) {
                    return Err(DaaError::InvalidParentHeader(
                        p.digest,
                        0,
                        format!("non-canonical round-0 parent is not valid genesis: {e}"),
                    ));
                }
            }
        } else if let Err(e) = block.extract_pow_header(chain_id) {
            return Err(DaaError::InvalidParentHeader(
                p.digest,
                block.round,
                e.to_string(),
            ));
        }
    }

    let parent_hashes: Vec<Hash> = parents.iter().map(|p| p.digest).collect();

    // The primary parent's target_bits defines current_target_bits
    let first_parent = parent_hashes[0];
    let parent_block = dag_store
        .get_block(&first_parent)
        .map_err(|e| DaaError::ParentBlockUnreadable(first_parent, e.to_string()))?;

    let current_target_bits = match parent_block.extract_pow_header(chain_id) {
        Ok(header) => header.target_bits,
        Err(_) if is_canonical_genesis(parent_block.round, &first_parent) => {
            // Canonical round-0 genesis block has no PoW header
            config.max_target_bits
        }
        Err(e) => {
            return Err(DaaError::InvalidParentHeader(
                first_parent,
                parent_block.round,
                if parent_block.round == 0 {
                    "non-canonical round-0 block without PoW header is not valid genesis"
                        .to_string()
                } else {
                    e.to_string()
                },
            ));
        }
    };

    let timestamps = extract_ancestor_timestamps(
        dag_store,
        &parent_hashes,
        chain_id,
        config.window_size as usize,
    )?;

    Ok(calculate_next_target_bits(
        current_target_bits,
        &timestamps,
        config,
    ))
}

/// Alias for [`required_target_bits`], returning an explicit [`Result`].
#[inline]
pub fn try_required_target_bits(
    dag_store: &DagStore,
    parents: &[BlockReference],
    chain_id: u64,
    config: &DaaConfig,
) -> Result<u32, DaaError> {
    required_target_bits(dag_store, parents, chain_id, config)
}

/// Extract timestamps in ascending order from ancestor blocks for DAA.
///
/// Fails closed with [`DaaError`] if any ancestor in the window cannot be read from
/// storage or contains an invalid/corrupted PoW header.
pub fn extract_ancestor_timestamps(
    dag_store: &DagStore,
    parent_hashes: &[Hash],
    chain_id: u64,
    limit: usize,
) -> Result<Vec<u64>, DaaError> {
    let mut timestamps = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    for h in parent_hashes {
        if visited.insert(*h) {
            queue.push_back(*h);
        }
    }

    while let Some(h) = queue.pop_front() {
        let block = dag_store
            .get_block(&h)
            .map_err(|e| DaaError::ParentBlockUnreadable(h, e.to_string()))?;

        if is_canonical_genesis(block.round, &h) {
            // Canonical genesis has no PoW header and no parents; terminal
            continue;
        }

        match block.extract_pow_header(chain_id) {
            Ok(header) => {
                timestamps.push(header.timestamp);
            }
            Err(e) => {
                return Err(DaaError::InvalidParentHeader(
                    h,
                    block.round,
                    format!("ancestor block PoW header invalid or unreadable: {e}"),
                ));
            }
        }

        if timestamps.len() >= limit {
            break;
        }

        for p in &block.parents {
            if visited.insert(p.digest) {
                queue.push_back(p.digest);
            }
        }
    }

    timestamps.sort_unstable();
    Ok(timestamps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_div_u256_exact_arithmetic() {
        // Identity: num == den
        let val = [0x55u8; 32];
        assert_eq!(mul_div_u256(val, 100, 100), val);

        // Zero numerator
        assert_eq!(mul_div_u256(val, 0, 100), [0u8; 32]);

        // Zero denominator: returns original value safely
        assert_eq!(mul_div_u256(val, 100, 0), val);

        // Simple doubling: num = 2, den = 1
        let mut small_val = [0u8; 32];
        small_val[31] = 50;
        let doubled = mul_div_u256(small_val, 2, 1);
        assert_eq!(doubled[31], 100);

        // Simple halving: num = 1, den = 2
        let halved = mul_div_u256(doubled, 1, 2);
        assert_eq!(halved[31], 50);

        // Saturation on overflow
        let max_val = [0xffu8; 32];
        let overflow = mul_div_u256(max_val, 10, 1);
        assert_eq!(overflow, [0xff; 32]);
    }

    #[test]
    fn daa_target_stable_when_blocks_on_time() {
        let config = DaaConfig::hybrid_default();
        let target_bits = 0x1d00ffff;

        // 10 blocks spaced exactly 4,000 ms apart
        let timestamps: Vec<u64> = (0..10).map(|i| 1_000_000 + i * 4_000).collect();

        let next = calculate_next_target_bits(target_bits, &timestamps, &config);
        assert_eq!(next, target_bits);
    }

    #[test]
    fn daa_difficulty_increases_when_blocks_too_fast() {
        let config = DaaConfig::hybrid_default();
        let target_bits = 0x1d00ffff;

        // 10 blocks spaced 2,000 ms apart (half the target time -> difficulty doubles, target halves)
        let timestamps: Vec<u64> = (0..10).map(|i| 1_000_000 + i * 2_000).collect();

        let next = calculate_next_target_bits(target_bits, &timestamps, &config);
        let old_target = compact_to_target(target_bits);
        let next_target = compact_to_target(next);

        // Target should be strictly smaller
        assert!(next_target < old_target);
    }

    #[test]
    fn daa_difficulty_decreases_when_blocks_too_slow() {
        let config = DaaConfig::hybrid_default();
        let target_bits = 0x1d00ffff;

        // 10 blocks spaced 8,000 ms apart (twice the target time -> difficulty halves, target doubles)
        let timestamps: Vec<u64> = (0..10).map(|i| 1_000_000 + i * 8_000).collect();

        let next = calculate_next_target_bits(target_bits, &timestamps, &config);
        let old_target = compact_to_target(target_bits);
        let next_target = compact_to_target(next);

        // Target should be strictly larger
        assert!(next_target > old_target);
    }

    #[test]
    fn daa_clamps_to_max_adjustment_factor() {
        let config = DaaConfig {
            max_adjustment_factor: 4,
            ..DaaConfig::hybrid_default()
        };
        let target_bits = 0x1d00ffff;

        // Super slow blocks (100x slower)
        let timestamps: Vec<u64> = (0..10).map(|i| 1_000_000 + i * 400_000).collect();
        let next = calculate_next_target_bits(target_bits, &timestamps, &config);

        let old_target = compact_to_target(target_bits);
        let next_target = compact_to_target(next);

        // Ratio should be at most 4x
        let max_expected = mul_div_u256(old_target, 4, 1);
        assert!(next_target <= max_expected);
    }
}
