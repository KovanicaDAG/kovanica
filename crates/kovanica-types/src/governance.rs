//! Governance proposal types and voting records.

use crate::crypto::{PublicKey, Signature, SigningKey};
use crate::Address;
use ed25519_dalek::Signer;
use serde::{Deserialize, Serialize};

/// Lifecycle status of an on-chain governance proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    /// Newly submitted proposal waiting for voting to begin.
    Open,
    /// Actively receiving validator votes.
    Voting,
    /// Passed quorum (2f+1) and approval (>50% voted stake in favour).
    Approved,
    /// Failed to reach quorum or failed majority approval.
    Rejected,
    /// Execution applied to state at epoch boundary.
    Executed,
}

/// The payload kind of a proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalKind {
    /// Proposal to modify a consensus or staking parameter.
    ParameterChange {
        /// Target parameter name (e.g. "MIN_VALIDATOR_STAKE").
        target_key: String,
        /// New proposed parameter value.
        new_value: u64,
    },
    /// Proposal to disburse funds from the chain treasury.
    TreasurySpend {
        /// Amount in atoms to spend.
        amount_atoms: u64,
        /// Recipient address for payout.
        recipient: Address,
    },
}

/// An on-chain governance proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique proposal identifier.
    pub id: u64,
    /// Proposer address.
    pub proposer: Address,
    /// Human-readable title.
    pub title: String,
    /// Specific proposal payload (parameter change or treasury spend).
    pub kind: ProposalKind,
    /// BLAKE3 digest of proposal description/justification.
    pub justification_hash: [u8; 32],
    /// DAG round when the voting window opens.
    pub start_round: u64,
    /// DAG round when the voting window closes.
    pub voting_end_round: u64,
    /// Total stake voted in favor.
    pub stake_voted_for: u64,
    /// Total stake voted against.
    pub stake_voted_against: u64,
    /// Current lifecycle status.
    pub status: ProposalStatus,
}

/// A stake-weighted vote cast by an active validator on a proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceVote {
    /// Address of the voting validator.
    pub voter: Address,
    /// ID of the proposal.
    pub proposal_id: u64,
    /// True if voting in favour; false if voting against.
    pub approve: bool,
    /// Ed25519 signature over canonical vote bytes.
    pub signature: Signature,
}

impl GovernanceVote {
    /// Domain tag for governance vote signature data.
    pub const DOMAIN: &'static [u8; 16] = b"KVNC/gov/vote\0\0\0";

    /// Canonical bytes signed by the voter.
    pub fn signature_data(proposal_id: u64, approve: bool) -> Vec<u8> {
        let mut data = Vec::with_capacity(32);
        data.extend_from_slice(Self::DOMAIN);
        data.extend_from_slice(&proposal_id.to_le_bytes());
        data.push(if approve { 1 } else { 0 });
        data
    }

    /// Construct and sign a new [`GovernanceVote`].
    pub fn new(voter: Address, proposal_id: u64, approve: bool, signing_key: &SigningKey) -> Self {
        let data = Self::signature_data(proposal_id, approve);
        let signature = signing_key.sign(&data);
        Self {
            voter,
            proposal_id,
            approve,
            signature: signature.into(),
        }
    }

    /// Verify the signature using the voter's public key (derived from Address).
    pub fn verify_signature(&self) -> bool {
        let data = Self::signature_data(self.proposal_id, self.approve);
        self.signature.verify(&data, &PublicKey(self.voter.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_vote_signature_and_verification() {
        let signing_key = SigningKey::from_bytes(&[42u8; 32]);
        let voter = Address(signing_key.verifying_key().to_bytes());

        let vote = GovernanceVote::new(voter, 1, true, &signing_key);
        assert!(vote.verify_signature());

        // Tampered approval
        let mut tampered_vote = vote.clone();
        tampered_vote.approve = false;
        assert!(!tampered_vote.verify_signature());

        // Tampered proposal id
        let mut tampered_id = vote.clone();
        tampered_id.proposal_id = 2;
        assert!(!tampered_id.verify_signature());
    }

    #[test]
    fn test_proposal_serialization_roundtrip() {
        let proposal = Proposal {
            id: 1,
            proposer: Address([1u8; 32]),
            title: "Increase Minimum Validator Stake".to_string(),
            kind: ProposalKind::ParameterChange {
                target_key: "MIN_VALIDATOR_STAKE".to_string(),
                new_value: 60_000_000_000_000,
            },
            justification_hash: [2u8; 32],
            start_round: 100,
            voting_end_round: 7_300,
            stake_voted_for: 1_000_000,
            stake_voted_against: 200_000,
            status: ProposalStatus::Voting,
        };

        let json_str = serde_json::to_string(&proposal).unwrap();
        let decoded: Proposal = serde_json::from_str(&json_str).unwrap();
        assert_eq!(proposal, decoded);
    }
}
