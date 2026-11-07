//! Canonical protocol parameters for Kovanica.
//!
//! Single source of truth for consensus, tokenomics, gas, size limits and
//! activation-round constants. Pure data + accessors only.

#![deny(unsafe_code)]

// ============================================================
// Chain / identity
// ============================================================
pub const TICKER: &str = "KVNC";
pub const DECIMALS: u32 = 9;
/// Legacy alias name for base unit multiplier (1 KVNC = 10^9 base units).
/// Kept as `ONE_KUNA` until a follow-up rename PR across the workspace.
pub const ONE_KUNA: u64 = 1_000_000_000;
pub const ONE_KVNC: u64 = ONE_KUNA;

/// Mainnet chain id (`1`). Canonical PRO-21 addresses only.
pub const CHAIN_ID_MAINNET: u64 = 1;
/// Legacy public testnet chain id (`2`).
pub const CHAIN_ID_TESTNET: u64 = 2;
/// Devnet chain id (`3`).
pub const CHAIN_ID_DEVNET: u64 = 3;
/// Internal testnet v2 chain id (`4`).
pub const CHAIN_ID_TESTNET_V2: u64 = 4;
/// Planned hybrid PoW + GHOSTDAG testnet chain id (`5`).
///
/// **Activation intent:** Chain ID 5 is dedicated to the hybrid PoW + GHOSTDAG
/// testnet track (ADR 0003), gated under feature flag `hybrid-pow`.
/// Under hybrid consensus, block authoring uses PoW mining (KovanicaHash)
/// and block ordering uses GHOSTDAG (k=5, 4 s block time) without stake weighting.
pub const CHAIN_ID_HYBRID_TESTNET: u64 = 5;
/// Local development network chain id (`1337`).
pub const CHAIN_ID_LOCAL: u64 = 1337;

/// Canonical network tag for chain 5 hybrid testnet.
pub const NETWORK_TAG_HYBRID_TESTNET: &str = "hybrid-testnet";

// ============================================================
// Consensus
// ============================================================
/// Hybrid PoW + GHOSTDAG default target block time in milliseconds (Toni 2026-11-05).
/// Inactive until hybrid-pow consensus path is wired.
pub const TARGET_BLOCK_MS: u64 = 4_000;
/// Hybrid PoW + GHOSTDAG default parameter k (Toni 2026-11-05).
pub const GHOSTDAG_K: u32 = 5;

/// Initial/maximum target bits for hybrid PoW testnet (Chain 5).
/// Compact representation `0x1f00ffff` (exponent 31, mantissa 0x00ffff).
pub const INITIAL_TARGET_BITS_CHAIN5: u32 = 0x1f00_ffff;
/// Minimum difficulty target bits (ceiling on target, easiest difficulty).
pub const MAX_TARGET_BITS: u32 = 0x1f00_ffff;
/// Maximum difficulty target bits (floor on target, hardest difficulty).
pub const MIN_TARGET_BITS: u32 = 0x0101_0000;
/// Default DAA adjustment window in blocks.
pub const DAA_WINDOW_BLOCKS: u64 = 60;
/// Maximum future timestamp drift in milliseconds (30 seconds).
pub const MAX_FUTURE_TIMESTAMP_DRIFT_MS: u64 = 30_000;

// Legacy Mysticeti timing constants (still in tree until hybrid flag):
pub const WAVE_LENGTH: u64 = 3;
pub const EPOCH_ROUNDS: u64 = 7_200;
pub const EPOCH_LEADER_HEIGHTS: u64 = EPOCH_ROUNDS / WAVE_LENGTH;
pub const LEADER_INTERVAL_SECS: u64 = 12;
pub const ROUND_SECS: u64 = 4;

pub const fn next_leader_slot(round: u64) -> u64 {
    (round / WAVE_LENGTH + 1).saturating_mul(WAVE_LENGTH)
}

// ============================================================
// Tokenomics
// ============================================================
pub const TOTAL_SUPPLY: u64 = 90_200_000 * ONE_KUNA;
pub const FOUNDER_PREMINE: u64 = 200_000 * ONE_KUNA;
pub const GENESIS_VALIDATOR_ALLOCATION: u64 = 1_000_000 * ONE_KUNA;
pub const TREASURY_TOTAL: u64 = 8_000_000 * ONE_KUNA;
pub const TREASURY_ANNUAL: u64 = 1_000_000 * ONE_KUNA;
pub const TREASURY_YEARS: u32 = 8;
pub const BLOCKS_PER_YEAR: u64 = 31_536_000 / LEADER_INTERVAL_SECS;
pub const MINING_SUBSIDY_BUDGET: u64 =
    TOTAL_SUPPLY - FOUNDER_PREMINE - TREASURY_TOTAL - GENESIS_VALIDATOR_ALLOCATION;
pub const INITIAL_BLOCK_REWARD: u64 = 5 * ONE_KUNA / 2;
pub const SUBSIDY_ERA_BLOCKS: u64 = 8_100_000;
pub const DECAY_NUM: u64 = 3;
pub const DECAY_DEN: u64 = 4;

// NEW NO-stake policy tokenomics constants (Approved 2026-11-06)
pub const GENESIS_VALIDATOR_ALLOCATION_NO_STAKE: u64 = 0;
pub const MINING_SUBSIDY_BUDGET_NO_STAKE_82M: u64 =
    TOTAL_SUPPLY - FOUNDER_PREMINE - TREASURY_TOTAL - GENESIS_VALIDATOR_ALLOCATION_NO_STAKE;
pub const SUBSIDY_ERA_BLOCKS_NO_STAKE_82M: u64 = 8_200_000;

// ============================================================
// Staking
// ============================================================
pub const MIN_VALIDATOR_STAKE: u64 = 50_000 * ONE_KUNA;
pub const MIN_DELEGATION: u64 = 50_000 * ONE_KUNA;
pub const MIN_ACTIVE_VALIDATORS: u32 = 15;
pub const UNBONDING_ROUNDS: u64 = 100_000;
pub const JAIL_ROUNDS: u64 = 7_200;
pub const MAX_MISSED_LEADERS_BEFORE_OFFLINE: u64 = 100;
pub const COMMISSION_CHANGE_COOLDOWN: u64 = 3 * EPOCH_LEADER_HEIGHTS;
pub const SLASH_PCT_BPS: u16 = 500;

// Light nodes
pub const LIGHT_REWARD_BPS: u16 = 2_000;
pub const MIN_LIGHT_BOND: u64 = 100 * ONE_KUNA;
pub const MAX_LIGHT_NODES: usize = 4_096;
pub const LIGHT_WITNESS_SLOTS: usize = 32;

// ============================================================
// Execution / gas
// ============================================================
pub const MAX_TX_GAS_LIMIT: u64 = 10_000_000;
pub const BLOCK_GAS_BUDGET_FACTOR: u64 = 8;
pub const GAS_PER_MODULE_BYTE: u64 = 8;
pub const DEFAULT_CONTRACT_MEMORY_PAGES: u32 = 256;

// ============================================================
// Size limits
// ============================================================
pub const MAX_TXS_PER_BLOCK: usize = 10_000;
pub const MAX_PARENT_ROUND_GAP: u64 = 300;
pub const MAX_TX_BYTES: usize = 512 * 1024;
pub const MAX_BLOCK_BYTES: usize = 960 * 1024;
pub const GENESIS_ROUND: u64 = 0;

// ============================================================
// Activation gates (inert by default)
// ============================================================
pub const GAS_HARDENING_ACTIVE_FROM_ROUND: u64 = u64::MAX;
pub const CONTRACT_HARDENING_ACTIVE_FROM_ROUND: u64 = u64::MAX;
pub const SLASHING_ACTIVE_FROM_ROUND: u64 = u64::MAX;
pub const STATE_COMMITMENT_ACTIVE_FROM_ROUND: u64 = 500_000;
pub const SIZE_LIMITS_ACTIVE_FROM_ROUND: u64 = u64::MAX;

#[allow(clippy::absurd_extreme_comparisons)]
pub fn gas_hardening_active_from(round: u64) -> bool {
    round >= GAS_HARDENING_ACTIVE_FROM_ROUND
}
#[allow(clippy::absurd_extreme_comparisons)]
pub fn contract_hardening_active_from(round: u64) -> bool {
    round >= CONTRACT_HARDENING_ACTIVE_FROM_ROUND
}
#[allow(clippy::absurd_extreme_comparisons)]
pub fn slashing_active_from(round: u64) -> bool {
    round >= SLASHING_ACTIVE_FROM_ROUND
}
pub fn state_commitment_active_from(round: u64) -> bool {
    round >= STATE_COMMITMENT_ACTIVE_FROM_ROUND
}
#[allow(clippy::absurd_extreme_comparisons)]
pub fn size_limits_active_from(round: u64) -> bool {
    round >= SIZE_LIMITS_ACTIVE_FROM_ROUND
}

// ============================================================
// Tests
// ============================================================
#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod tests {
    use super::*;

    #[test]
    fn supply_decomposition() {
        assert_eq!(
            FOUNDER_PREMINE + GENESIS_VALIDATOR_ALLOCATION + TREASURY_TOTAL + MINING_SUBSIDY_BUDGET,
            TOTAL_SUPPLY
        );
    }

    #[test]
    fn committee_bounds() {
        assert!(MIN_ACTIVE_VALIDATORS >= 1);
        assert_eq!(MIN_VALIDATOR_STAKE, 50_000 * ONE_KUNA);
        assert_eq!(MIN_DELEGATION, 50_000 * ONE_KUNA);
    }

    #[test]
    fn gates_inert() {
        assert_eq!(GAS_HARDENING_ACTIVE_FROM_ROUND, u64::MAX);
        assert_eq!(CONTRACT_HARDENING_ACTIVE_FROM_ROUND, u64::MAX);
    }

    #[test]
    fn no_stake_supply_decomposition() {
        assert_eq!(
            FOUNDER_PREMINE
                + GENESIS_VALIDATOR_ALLOCATION_NO_STAKE
                + TREASURY_TOTAL
                + MINING_SUBSIDY_BUDGET_NO_STAKE_82M,
            TOTAL_SUPPLY
        );
        assert_eq!(MINING_SUBSIDY_BUDGET_NO_STAKE_82M, 82_000_000 * ONE_KUNA);
        assert_eq!(SUBSIDY_ERA_BLOCKS_NO_STAKE_82M, 8_200_000);
    }

    #[test]
    fn era_math() {
        assert!(SUBSIDY_ERA_BLOCKS > 0);
        assert!(DECAY_DEN > 0);
    }
}
