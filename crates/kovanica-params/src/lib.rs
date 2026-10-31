//! Canonical protocol parameters for Kovanica.
//!
//! Single source of truth for consensus, tokenomics, gas, size limits and
//! activation-round constants. Pure data + accessors only.

#![deny(unsafe_code)]

// ============================================================
// Chain / identity
// ============================================================
pub const TICKER: &str = "KUNA";
pub const DECIMALS: u32 = 9;
pub const ONE_KUNA: u64 = 1_000_000_000;

// ============================================================
// Consensus
// ============================================================
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
    fn era_math() {
        assert!(SUBSIDY_ERA_BLOCKS > 0);
        assert!(DECAY_DEN > 0);
    }
}
