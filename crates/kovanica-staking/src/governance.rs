//! On-chain governance proposal tracking, stake-weighted voting, and lifecycle execution.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

use crate::{StakingState, MIN_VALIDATOR_STAKE};
use kovanica_params::EPOCH_ROUNDS;
use kovanica_types::{Address, GovernanceVote, Proposal, ProposalKind, ProposalStatus, Stake};

/// Governance tracking errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GovernanceError {
    /// Proposal with the given ID does not exist.
    #[error("proposal {0} not found")]
    ProposalNotFound(u64),

    /// Proposal is not in a voting state.
    #[error("proposal {0} is not active for voting (status: {1:?})")]
    ProposalNotActive(u64, ProposalStatus),

    /// Attempted to vote outside the voting window.
    #[error("voting window for proposal {0} is closed (current round: {1}, deadline: {2})")]
    VotingClosed(u64, u64, u64),

    /// A validator has already voted on this proposal.
    #[error("voter {0} has already voted on proposal {1}")]
    AlreadyVoted(Address, u64),

    /// The voter address is not an active validator meeting the minimum stake.
    #[error("voter {0} is not an active validator in the committee")]
    VoterNotActiveValidator(Address),

    /// The Ed25519 signature on the vote is invalid.
    #[error("invalid vote signature from {0} for proposal {1}")]
    InvalidSignature(Address, u64),

    /// Attempted to execute a proposal that has not been approved.
    #[error("proposal {0} is not approved for execution (status: {1:?})")]
    ProposalNotApproved(u64, ProposalStatus),
}

/// State of all governance proposals and validator votes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceState {
    /// All proposals tracked by proposal ID.
    pub proposals: BTreeMap<u64, Proposal>,
    /// All cast votes indexed by (proposal_id, voter_address).
    pub votes: HashMap<(u64, Address), GovernanceVote>,
    /// Next available proposal ID.
    pub next_proposal_id: u64,
}

impl GovernanceState {
    /// Create a new, empty [`GovernanceState`].
    pub fn new() -> Self {
        Self {
            proposals: BTreeMap::new(),
            votes: HashMap::new(),
            next_proposal_id: 1,
        }
    }

    /// Register a new proposal, setting its voting window to [`EPOCH_ROUNDS`].
    pub fn register_proposal(
        &mut self,
        proposer: Address,
        title: String,
        kind: ProposalKind,
        justification_hash: [u8; 32],
        current_round: u64,
    ) -> u64 {
        let id = self.next_proposal_id;
        self.next_proposal_id += 1;

        let proposal = Proposal {
            id,
            proposer,
            title,
            kind,
            justification_hash,
            start_round: current_round,
            voting_end_round: current_round.saturating_add(EPOCH_ROUNDS),
            stake_voted_for: 0,
            stake_voted_against: 0,
            status: ProposalStatus::Voting,
        };

        self.proposals.insert(id, proposal);
        id
    }

    /// Cast a stake-weighted vote on an active proposal.
    ///
    /// The voter's stake weight is evaluated from their active validator total
    /// stake (self-stake + delegations) in [`StakingState`].
    pub fn cast_vote(
        &mut self,
        vote: GovernanceVote,
        current_round: u64,
        staking_state: &StakingState,
    ) -> Result<(), GovernanceError> {
        // 1. Verify cryptographic signature.
        if !vote.verify_signature() {
            return Err(GovernanceError::InvalidSignature(
                vote.voter,
                vote.proposal_id,
            ));
        }

        // 2. Fetch the proposal.
        let proposal = self
            .proposals
            .get_mut(&vote.proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(vote.proposal_id))?;

        // 3. Check status is Voting (or Open).
        if proposal.status != ProposalStatus::Voting && proposal.status != ProposalStatus::Open {
            return Err(GovernanceError::ProposalNotActive(
                vote.proposal_id,
                proposal.status,
            ));
        }

        // 4. Enforce voting window [start_round, voting_end_round].
        if current_round < proposal.start_round || current_round > proposal.voting_end_round {
            return Err(GovernanceError::VotingClosed(
                vote.proposal_id,
                current_round,
                proposal.voting_end_round,
            ));
        }

        // 5. Check if the validator already voted.
        let key = (vote.proposal_id, vote.voter);
        if self.votes.contains_key(&key) {
            return Err(GovernanceError::AlreadyVoted(vote.voter, vote.proposal_id));
        }

        // 6. Look up active validator total stake (self-stake + delegations).
        let validator = staking_state
            .validators
            .iter()
            .find(|v| v.address == vote.voter && v.active && v.stake >= MIN_VALIDATOR_STAKE)
            .ok_or(GovernanceError::VoterNotActiveValidator(vote.voter))?;

        let weight: Stake = validator.stake;

        // 7. Accumulate stake weight.
        if vote.approve {
            proposal.stake_voted_for = proposal.stake_voted_for.saturating_add(weight);
        } else {
            proposal.stake_voted_against = proposal.stake_voted_against.saturating_add(weight);
        }
        proposal.status = ProposalStatus::Voting;

        self.votes.insert(key, vote);
        Ok(())
    }

    /// Evaluate quorum and approval for a proposal once voting has closed.
    ///
    /// - Quorum: 2f + 1 (~67% of active committee stake).
    /// - Validity turnout: f + 1 (~33% of active committee stake).
    /// - Approval: > 50% of voted stake in favour.
    pub fn evaluate_proposal(
        &mut self,
        proposal_id: u64,
        current_round: u64,
        staking_state: &StakingState,
    ) -> Result<ProposalStatus, GovernanceError> {
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        // If not past the voting window, current status remains unchanged.
        if current_round <= proposal.voting_end_round {
            return Ok(proposal.status);
        }

        // If already finalized, return existing status.
        if matches!(
            proposal.status,
            ProposalStatus::Approved | ProposalStatus::Rejected | ProposalStatus::Executed
        ) {
            return Ok(proposal.status);
        }

        // Calculate total active committee stake.
        let committee_stake: Stake = staking_state
            .committee_order()
            .iter()
            .map(|v| v.stake)
            .sum();

        let total_voted = proposal
            .stake_voted_for
            .saturating_add(proposal.stake_voted_against);

        let validity_threshold = committee_stake / 3 + 1;
        let quorum_threshold = (committee_stake * 2) / 3 + 1;

        // Check turnout validity (f+1) and quorum (2f+1).
        let quorum_met = total_voted >= validity_threshold && total_voted >= quorum_threshold;

        // Check majority approval (>50% of voted stake in favor).
        let approved = quorum_met && proposal.stake_voted_for > proposal.stake_voted_against;

        proposal.status = if approved {
            ProposalStatus::Approved
        } else {
            ProposalStatus::Rejected
        };

        Ok(proposal.status)
    }

    /// Execute an approved proposal.
    pub fn execute_proposal(
        &mut self,
        proposal_id: u64,
        current_round: u64,
        staking_state: &mut StakingState,
    ) -> Result<Proposal, GovernanceError> {
        self.evaluate_proposal(proposal_id, current_round, staking_state)?;

        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        if proposal.status != ProposalStatus::Approved {
            return Err(GovernanceError::ProposalNotApproved(
                proposal_id,
                proposal.status,
            ));
        }

        proposal.status = ProposalStatus::Executed;
        Ok(proposal.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::crypto::SigningKey;

    fn key(byte: u8) -> SigningKey {
        SigningKey::from_bytes(&[byte; 32])
    }

    fn addr(byte: u8) -> Address {
        Address(key(byte).verifying_key().to_bytes())
    }

    fn setup_staking_with_validators() -> (StakingState, Vec<(SigningKey, Address)>) {
        let mut st = StakingState::new();
        let mut keys = Vec::new();

        // 3 validators:
        // V1: 100_000 KUNA
        // V2: 60_000 KUNA
        // V3: 50_000 KUNA
        // Total active stake = 210_000 KUNA
        // Validity (f+1) = 210_000 / 3 + 1 = 70_001
        // Quorum (2f+1) = (210_000 * 2) / 3 + 1 = 140_001
        let stakes = [
            100_000 * 1_000_000_000,
            60_000 * 1_000_000_000,
            50_000 * 1_000_000_000,
        ];
        for (i, &stake) in stakes.iter().enumerate() {
            let sk = key((i + 1) as u8);
            let a = addr((i + 1) as u8);
            st.join_validator(a, stake, 0, None, None, None).unwrap();
            keys.push((sk, a));
        }

        (st, keys)
    }

    #[test]
    fn test_governance_lifecycle_approved() {
        let (mut st, keys) = setup_staking_with_validators();
        let mut gov = GovernanceState::new();

        let pid = gov.register_proposal(
            keys[0].1,
            "Param Change 1".to_string(),
            ProposalKind::ParameterChange {
                target_key: "MIN_VALIDATOR_STAKE".to_string(),
                new_value: 55_000 * 1_000_000_000,
            },
            [0u8; 32],
            100,
        );

        assert_eq!(pid, 1);
        let prop = gov.proposals.get(&pid).unwrap();
        assert_eq!(prop.start_round, 100);
        assert_eq!(prop.voting_end_round, 100 + EPOCH_ROUNDS);
        assert_eq!(prop.status, ProposalStatus::Voting);

        // V1 votes YES (100k stake)
        let vote1 = GovernanceVote::new(keys[0].1, pid, true, &keys[0].0);
        gov.cast_vote(vote1, 200, &st).unwrap();

        // V2 votes YES (60k stake)
        let vote2 = GovernanceVote::new(keys[1].1, pid, true, &keys[1].0);
        gov.cast_vote(vote2, 250, &st).unwrap();

        let prop = gov.proposals.get(&pid).unwrap();
        assert_eq!(prop.stake_voted_for, (100_000 + 60_000) * 1_000_000_000);
        assert_eq!(prop.stake_voted_against, 0);

        // Before voting deadline, evaluate doesn't finalize
        let status = gov.evaluate_proposal(pid, 500, &st).unwrap();
        assert_eq!(status, ProposalStatus::Voting);

        // After deadline, evaluate resolves to Approved (160k > 140_001 quorum, 100% yes)
        let status = gov
            .evaluate_proposal(pid, 100 + EPOCH_ROUNDS + 1, &st)
            .unwrap();
        assert_eq!(status, ProposalStatus::Approved);

        // Execute proposal
        let executed = gov
            .execute_proposal(pid, 100 + EPOCH_ROUNDS + 1, &mut st)
            .unwrap();
        assert_eq!(executed.status, ProposalStatus::Executed);
    }

    #[test]
    fn test_governance_quorum_failure() {
        let (st, keys) = setup_staking_with_validators();
        let mut gov = GovernanceState::new();

        let pid = gov.register_proposal(
            keys[0].1,
            "Treasury Spend".to_string(),
            ProposalKind::TreasurySpend {
                amount_atoms: 10_000 * 1_000_000_000,
                recipient: keys[1].1,
            },
            [1u8; 32],
            0,
        );

        // Only V2 votes YES (60k stake)
        // Quorum is 140_001, so 60k fails quorum
        let vote = GovernanceVote::new(keys[1].1, pid, true, &keys[1].0);
        gov.cast_vote(vote, 100, &st).unwrap();

        let status = gov.evaluate_proposal(pid, EPOCH_ROUNDS + 10, &st).unwrap();
        assert_eq!(status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_governance_rejection_by_majority_against() {
        let (st, keys) = setup_staking_with_validators();
        let mut gov = GovernanceState::new();

        let pid = gov.register_proposal(
            keys[0].1,
            "Param Change 2".to_string(),
            ProposalKind::ParameterChange {
                target_key: "TARGET_BLOCK_MS".to_string(),
                new_value: 2000,
            },
            [2u8; 32],
            0,
        );

        // V1 (100k) votes NO
        let vote1 = GovernanceVote::new(keys[0].1, pid, false, &keys[0].0);
        gov.cast_vote(vote1, 50, &st).unwrap();

        // V2 (60k) votes YES
        let vote2 = GovernanceVote::new(keys[1].1, pid, true, &keys[1].0);
        gov.cast_vote(vote2, 60, &st).unwrap();

        // Turnout = 160k >= quorum 140_001
        // But Against (100k) > For (60k)
        let status = gov.evaluate_proposal(pid, EPOCH_ROUNDS + 1, &st).unwrap();
        assert_eq!(status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_voting_window_enforcement_and_duplicate_prevention() {
        let (st, keys) = setup_staking_with_validators();
        let mut gov = GovernanceState::new();

        let pid = gov.register_proposal(
            keys[0].1,
            "Param Change 3".to_string(),
            ProposalKind::ParameterChange {
                target_key: "MIN_DELEGATION".to_string(),
                new_value: 10_000 * 1_000_000_000,
            },
            [3u8; 32],
            100,
        );

        // Before start_round -> rejected
        let vote_early = GovernanceVote::new(keys[0].1, pid, true, &keys[0].0);
        assert!(matches!(
            gov.cast_vote(vote_early, 50, &st),
            Err(GovernanceError::VotingClosed(..))
        ));

        // After voting_end_round -> rejected
        let vote_late = GovernanceVote::new(keys[0].1, pid, true, &keys[0].0);
        assert!(matches!(
            gov.cast_vote(vote_late, 100 + EPOCH_ROUNDS + 1, &st),
            Err(GovernanceError::VotingClosed(..))
        ));

        // Valid vote
        let vote = GovernanceVote::new(keys[0].1, pid, true, &keys[0].0);
        gov.cast_vote(vote.clone(), 200, &st).unwrap();

        // Duplicate vote -> rejected
        assert_eq!(
            gov.cast_vote(vote, 210, &st),
            Err(GovernanceError::AlreadyVoted(keys[0].1, pid))
        );
    }

    #[test]
    fn test_delegation_included_in_validator_stake_weight() {
        let (mut st, keys) = setup_staking_with_validators();
        let mut gov = GovernanceState::new();

        // Delegate 50_000 KUNA to V3 (who had 50_000 self-stake -> now 100_000 total stake)
        let delegator = addr(99);
        st.delegate(delegator, keys[2].1, 50_000 * 1_000_000_000)
            .unwrap();

        let pid = gov.register_proposal(
            keys[0].1,
            "Proposal with Delegation".to_string(),
            ProposalKind::ParameterChange {
                target_key: "TEST".to_string(),
                new_value: 1,
            },
            [0u8; 32],
            0,
        );

        // V3 votes YES
        let vote3 = GovernanceVote::new(keys[2].1, pid, true, &keys[2].0);
        gov.cast_vote(vote3, 10, &st).unwrap();

        let prop = gov.proposals.get(&pid).unwrap();
        // Weight must be 100_000 KUNA (50k self + 50k delegation)
        assert_eq!(prop.stake_voted_for, 100_000 * 1_000_000_000);
    }
}
