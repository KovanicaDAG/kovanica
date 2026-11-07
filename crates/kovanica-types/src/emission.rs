//! Emission and genesis policy types for Kovanica.

use core::fmt;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

/// Policy governing token genesis allocation and mining subsidy emission.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EmissionPolicy {
    /// Legacy 81M mining subsidy with 1M genesis validator allocation,
    /// 8.1M block era length, decaying by 3/4.
    #[default]
    Legacy81M,
    /// Approved 2026-11-06 NO-stake policy with 82M mining subsidy,
    /// 0 genesis validator allocation, 8.2M block era length, decaying by 3/4,
    /// with one-time terminal remainder payout when scheduled reward reaches 0.
    NoStake82M,
}

impl EmissionPolicy {
    /// Canonical kebab-case identifier for this policy.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Legacy81M => "legacy-81m",
            Self::NoStake82M => "no-stake-82m",
        }
    }

    /// Mining subsidy budget in base units (atoms, 9 decimals) for this policy.
    pub const fn mining_budget(&self) -> u64 {
        match self {
            Self::Legacy81M => kovanica_params::MINING_SUBSIDY_BUDGET,
            Self::NoStake82M => kovanica_params::MINING_SUBSIDY_BUDGET_NO_STAKE_82M,
        }
    }

    /// Genesis validator allocation in base units (atoms, 9 decimals) for this policy.
    pub const fn genesis_validator_allocation(&self) -> u64 {
        match self {
            Self::Legacy81M => kovanica_params::GENESIS_VALIDATOR_ALLOCATION,
            Self::NoStake82M => kovanica_params::GENESIS_VALIDATOR_ALLOCATION_NO_STAKE,
        }
    }

    /// Number of rewarded blocks per subsidy era for this policy.
    pub const fn subsidy_era_blocks(&self) -> u64 {
        match self {
            Self::Legacy81M => kovanica_params::SUBSIDY_ERA_BLOCKS,
            Self::NoStake82M => kovanica_params::SUBSIDY_ERA_BLOCKS_NO_STAKE_82M,
        }
    }

    /// Whether this policy is a NO-stake network with 0 genesis validator allocation.
    pub const fn is_no_stake(&self) -> bool {
        matches!(self, Self::NoStake82M)
    }
}

impl fmt::Display for EmissionPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error returned when parsing an unrecognized [`EmissionPolicy`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("unknown emission policy '{0}'; expected 'legacy-81m' or 'no-stake-82m'")]
pub struct ParseEmissionPolicyError(String);

impl FromStr for EmissionPolicy {
    type Err = ParseEmissionPolicyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "legacy-81m" | "legacy_81m" | "legacy" => Ok(Self::Legacy81M),
            "no-stake-82m" | "no_stake_82m" | "nostake-82m" | "nostake_82m" | "no-stake"
            | "nostake" => Ok(Self::NoStake82M),
            _ => Err(ParseEmissionPolicyError(s.to_string())),
        }
    }
}

/// Persisted deterministic mining reward execution state for NO-stake networks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiningRewardState {
    /// Number of eligible blocks that have received mining rewards so far.
    pub rewarded_block_count: u64,
    /// Cumulative mining rewards minted in base units (atoms) so far.
    pub cumulative_mining_issued: u64,
    /// Whether the one-time terminal remainder payout has been paid.
    pub terminal_payout_done: bool,
    /// The block index that received the terminal payout, if already paid.
    pub terminal_payout_block_index: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_defaults_and_parsing() {
        assert_eq!(EmissionPolicy::default(), EmissionPolicy::Legacy81M);
        assert_eq!(
            EmissionPolicy::from_str("legacy-81m").unwrap(),
            EmissionPolicy::Legacy81M
        );
        assert_eq!(
            EmissionPolicy::from_str("legacy").unwrap(),
            EmissionPolicy::Legacy81M
        );
        assert_eq!(
            EmissionPolicy::from_str("no-stake-82m").unwrap(),
            EmissionPolicy::NoStake82M
        );
        assert_eq!(
            EmissionPolicy::from_str("nostake-82m").unwrap(),
            EmissionPolicy::NoStake82M
        );
        assert!(EmissionPolicy::from_str("invalid").is_err());
    }

    #[test]
    fn policy_constants() {
        let legacy = EmissionPolicy::Legacy81M;
        assert_eq!(legacy.mining_budget(), 81_000_000 * 1_000_000_000);
        assert_eq!(
            legacy.genesis_validator_allocation(),
            1_000_000 * 1_000_000_000
        );
        assert_eq!(legacy.subsidy_era_blocks(), 8_100_000);
        assert!(!legacy.is_no_stake());

        let nostake = EmissionPolicy::NoStake82M;
        assert_eq!(nostake.mining_budget(), 82_000_000 * 1_000_000_000);
        assert_eq!(nostake.genesis_validator_allocation(), 0);
        assert_eq!(nostake.subsidy_era_blocks(), 8_200_000);
        assert!(nostake.is_no_stake());
    }
}
