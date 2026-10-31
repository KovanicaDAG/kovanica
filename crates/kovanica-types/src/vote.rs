//! Consensus vote types.

use crate::crypto::Signature;
use crate::hash::Hash;
use crate::signing::{SigningContext, VOTE_DOMAIN_TAG};
use crate::{AuthorityIndex, Round};
use serde::{Deserialize, Serialize};

/// A consensus vote for a leader block.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote {
    /// The round of the leader being voted for.
    pub leader_round: Round,
    /// The hash of the leader block being voted for.
    pub leader_hash: Hash,
    /// The authority index of the voter.
    pub voter: AuthorityIndex,
    /// Signature of the voter over [`Vote::signature_data`] (format v2).
    pub signature: Signature,
}

impl Vote {
    /// Canonical bytes signed for a vote (signature format **v2**, 106 bytes).
    ///
    /// This is the single canonical vote encoding, used both for signing
    /// (`kovanica-consensus`) and for verification
    /// (`kovanica_crypto::verify_vote_signature`). The message is signed directly
    /// with Ed25519, without prehashing. Spec: `docs/SIGNATURE_FORMAT.md` §1.
    ///
    /// | offset | len | field                                       |
    /// |-------:|----:|---------------------------------------------|
    /// | 0      | 16  | `"KUNA/vote/v2"` zero-padded domain tag      |
    /// | 16     | 8   | `ctx.chain_id`, `u64` little-endian         |
    /// | 24     | 32  | `ctx.genesis_hash` raw bytes                |
    /// | 56     | 8   | `ctx.epoch`, `u64` little-endian            |
    /// | 64     | 2   | `voter`, `u16` little-endian                |
    /// | 66     | 8   | `leader_round`, `u64` little-endian         |
    /// | 74     | 32  | `leader_hash` raw bytes                     |
    ///
    /// `signature` is not covered. The verifier must build `ctx` from its own
    /// configuration (`chain_id`, `genesis_hash`) and current epoch, never from
    /// a network message.
    pub fn signature_data(&self, ctx: &SigningContext) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::SIGNATURE_DATA_LEN);
        data.extend_from_slice(&VOTE_DOMAIN_TAG);
        data.extend_from_slice(&ctx.chain_id.to_le_bytes());
        data.extend_from_slice(&ctx.genesis_hash.0);
        data.extend_from_slice(&ctx.epoch.to_le_bytes());
        data.extend_from_slice(&self.voter.to_le_bytes());
        data.extend_from_slice(&self.leader_round.to_le_bytes());
        data.extend_from_slice(&self.leader_hash.0);
        data
    }

    /// Length in bytes of [`Vote::signature_data`].
    pub const SIGNATURE_DATA_LEN: usize = 106;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::Hash;

    const CTX: SigningContext = SigningContext {
        chain_id: 2,
        epoch: 7,
        genesis_hash: Hash([0x22; 32]),
    };

    fn golden_vote() -> Vote {
        Vote {
            leader_round: 42,
            leader_hash: Hash([0x11; 32]),
            voter: 3,
            signature: Signature([0xAAu8; 64]),
        }
    }

    #[test]
    fn vote_signature_data_is_deterministic() {
        let vote = golden_vote();
        assert_eq!(vote.signature_data(&CTX), vote.signature_data(&CTX));
    }

    /// Golden vector V2 from `docs/SIGNATURE_FORMAT.md` §4 (106-byte v2 layout).
    #[test]
    fn vote_signature_data_v2_golden_vector() {
        let data = golden_vote().signature_data(&CTX);
        assert_eq!(data.len(), Vote::SIGNATURE_DATA_LEN);
        assert_eq!(data.len(), 106);
        assert_eq!(
            hex::encode(&data),
            "4b554e412f766f74652f76320000000002000000000000002222222222222222222222222222222222222222222222222222222222222222070000000000000003002a000000000000001111111111111111111111111111111111111111111111111111111111111111"
        );
    }

    #[test]
    fn vote_signature_data_commits_to_context_and_voter() {
        let vote = golden_vote();
        let base = vote.signature_data(&CTX);
        // Different chain, genesis_hash, epoch or voter must change the signed bytes.
        assert_ne!(
            base,
            vote.signature_data(&SigningContext { chain_id: 3, ..CTX })
        );
        assert_ne!(
            base,
            vote.signature_data(&SigningContext { epoch: 8, ..CTX })
        );
        assert_ne!(
            base,
            vote.signature_data(&SigningContext {
                genesis_hash: Hash([0x33; 32]),
                ..CTX
            })
        );
        let mut other = vote.clone();
        other.voter = 4;
        assert_ne!(base, other.signature_data(&CTX));
        // The signature itself is not covered.
        let mut resigned = vote.clone();
        resigned.signature = Signature([0u8; 64]);
        assert_eq!(base, resigned.signature_data(&CTX));
    }

    #[test]
    fn vote_round_trip() {
        let vote = Vote {
            leader_round: 42,
            leader_hash: Hash::new("test-hash"),
            voter: 5,
            signature: Signature([1u8; 64]),
        };
        let encoded = bincode::serialize(&vote).expect("serializes");
        let decoded: Vote = bincode::deserialize(&encoded).expect("deserializes");
        assert_eq!(vote, decoded);
    }
}
