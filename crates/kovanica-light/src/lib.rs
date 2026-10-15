//! Light-client checks for Kovanica.
//!
//! This crate verifies sealed committed-leader headers, transaction
//! inclusion, and account proofs. It does not replay the DAG or execute
//! WASM. A rewarded light node is not a validator.

#![deny(unsafe_code)]

use kovanica_crypto::CommitSealError;
use kovanica_types::crypto::Signature;
use kovanica_types::{
    committee_seal_hash, merkle, verify_colouring, Address, Authority, AuthorityIndex,
    ColouringCertificate, CommitSeal, Committee, Hash, LeaderHeader, MerkleProof, PublicKey,
    SealAnnouncement, SigningContext, SigningKey, Stake, StateProof, Transaction, TransactionKind,
};

/// Why a light client rejected a header or a proof.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LightError {
    /// The producer timestamp is not zero.
    #[error("leader header timestamp is not zero")]
    Timestamp,
    /// The header's committee hash is not the configured committee.
    #[error("leader header committee hash does not match")]
    Committee,
    /// The header does not follow the previous sealed leader.
    #[error("leader header does not follow the previous sealed leader")]
    Parent,
    /// Commit seals failed signature or quorum checks.
    #[error("commit seals rejected")]
    Seals(CommitSealError),
    /// MysticGhost roots do not match the supplied mergeset.
    #[error("mysticghost colouring does not match the header")]
    Colouring,
}

/// Check one sealed header against the configured committee.
///
/// `parent` is the previous header this client accepted. `None` treats
/// `header` as a checkpoint: the seal quorum is enough, and the parent
/// digest is not compared. A later header must name the previous leader
/// digest and use the next committed height.
///
/// The timestamp must be 0. Statement blocks have no clock, and a non-zero
/// timestamp would make validators disagree.
pub fn verify_sealed_header(
    chain_id: u64,
    committee: &Committee,
    header: &LeaderHeader,
    seals: &[CommitSeal],
    parent: Option<&LeaderHeader>,
) -> Result<(), LightError> {
    if header.timestamp != 0 {
        return Err(LightError::Timestamp);
    }
    let members: Vec<(AuthorityIndex, PublicKey, Stake)> = committee
        .authorities
        .iter()
        .map(|authority| (authority.index, authority.public_key, authority.stake))
        .collect();
    if header.committee_hash != committee_seal_hash(committee.epoch, &members) {
        return Err(LightError::Committee);
    }
    if let Some(parent) = parent {
        if header.parent_leader_digest != parent.leader_digest
            || header.committed_height != parent.committed_height.saturating_add(1)
        {
            return Err(LightError::Parent);
        }
    }
    kovanica_crypto::verify_commit_seals(chain_id, header, seals, committee).map_err(LightError::Seals)
}

/// Re-check MysticGhost roots when the caller has the mergeset.
pub fn verify_header_colouring(
    header: &LeaderHeader,
    certificate: &ColouringCertificate,
) -> Result<(), LightError> {
    verify_colouring(header, certificate)
        .map(|_| ())
        .map_err(|_| LightError::Colouring)
}

/// Transaction inclusion in the executed-transaction root.
pub fn verify_executed_tx(header: &LeaderHeader, tx_hash: &Hash, proof: &MerkleProof) -> bool {
    merkle::verify(tx_hash, proof, &header.executed_tx_root)
}

/// Transaction inclusion in the leader block's own merkle root.
pub fn verify_leader_tx(header: &LeaderHeader, tx_hash: &Hash, proof: &MerkleProof) -> bool {
    merkle::verify(tx_hash, proof, &header.tx_merkle_root)
}

/// Account balance and nonce against the header's state root.
pub fn verify_account(header: &LeaderHeader, proof: &StateProof) -> bool {
    proof.state_root == header.state_root && proof.verify()
}

/// Build a committee from seal-relevant fields.
///
/// Network addresses and VRF keys are not part of the seal hash.
pub fn committee_from_members(
    epoch: u64,
    members: &[(AuthorityIndex, PublicKey, Stake, Address)],
) -> Committee {
    let authorities = members
        .iter()
        .map(|(index, public_key, stake, address)| Authority {
            index: *index,
            public_key: *public_key,
            vrf_public_key: None,
            address: *address,
            stake: *stake,
            network_address: String::new(),
        })
        .collect();
    Committee::new(epoch, authorities)
}

/// Sign a light-node bond. The stake does not enter the validator committee.
pub fn sign_register_light_node(
    chain_id: u64,
    signing_key: &SigningKey,
    sender: Address,
    vrf_public_key: PublicKey,
    stake: u64,
    nonce: u64,
    fee: u64,
) -> Transaction {
    sign_kind(
        chain_id,
        signing_key,
        sender,
        nonce,
        fee,
        TransactionKind::RegisterLightNode {
            vrf_public_key,
            stake,
        },
    )
}

/// Sign a light-node exit. A pending registration refunds in execution.
pub fn sign_exit_light_node(
    chain_id: u64,
    signing_key: &SigningKey,
    sender: Address,
    nonce: u64,
    fee: u64,
) -> Transaction {
    sign_kind(
        chain_id,
        signing_key,
        sender,
        nonce,
        fee,
        TransactionKind::ExitLightNode,
    )
}

/// Sign one ECVRF witness for `epoch`.
pub fn sign_light_witness(
    chain_id: u64,
    signing_key: &SigningKey,
    sender: Address,
    epoch: u64,
    vrf_hash: [u8; 32],
    vrf_proof: [u8; 96],
    nonce: u64,
    fee: u64,
) -> Transaction {
    sign_kind(
        chain_id,
        signing_key,
        sender,
        nonce,
        fee,
        TransactionKind::SubmitLightWitness {
            epoch,
            vrf_hash,
            vrf_proof,
        },
    )
}

/// Sign a native transfer. `chain_id` is the caller's configured network.
pub fn sign_transfer(
    chain_id: u64,
    signing_key: &SigningKey,
    sender: Address,
    to: Address,
    amount: u64,
    nonce: u64,
    fee: u64,
) -> Transaction {
    sign_kind(
        chain_id,
        signing_key,
        sender,
        nonce,
        fee,
        TransactionKind::Transfer { to, amount },
    )
}

fn sign_kind(
    chain_id: u64,
    signing_key: &SigningKey,
    sender: Address,
    nonce: u64,
    fee: u64,
    kind: TransactionKind,
) -> Transaction {
    let mut transaction = Transaction {
        sender,
        nonce,
        kind,
        fee,
        signature: Signature([0u8; 64]),
        hash: Hash::zero(),
    };
    let ctx = SigningContext::new(chain_id);
    let hash = transaction.signing_hash(&ctx);
    transaction.signature = kovanica_crypto::sign(signing_key, hash.as_ref());
    transaction.hash = hash;
    transaction
}

/// A seal announcement is only a convenience for tests and gossip decoding.
pub fn announcement_seal(announcement: &SealAnnouncement) -> &CommitSeal {
    &announcement.seal
}

/// Header field used by a light node that stores the last accepted leader.
pub fn header_leader_digest(header: &LeaderHeader) -> Hash {
    header.leader_digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::{leader_header, AccountSmt, CommittedSubDag, StatementBlock};

    fn keys(n: u16) -> (Vec<SigningKey>, Committee) {
        let mut signers = Vec::new();
        let mut members = Vec::new();
        for index in 0..n {
            let signer = SigningKey::from_bytes(&[index as u8 + 1; 32]);
            let public_key = PublicKey::from(signer.verifying_key());
            let address = Address::from_public_key(&public_key);
            members.push((index, public_key, 1u64, address));
            signers.push(signer);
        }
        (signers, committee_from_members(0, &members))
    }

    fn header_for(committee: &Committee, height: u64, parent: Hash, state: Hash) -> LeaderHeader {
        let digest = Hash::new([height as u8 + 1]);
        let leader = StatementBlock {
            author: 0,
            round: height.saturating_mul(3),
            parents: vec![],
            transactions: vec![],
            statements: vec![],
            signature: Signature([0; 64]),
            digest,
            merkle_root: Hash::zero(),
        };
        let subdag = CommittedSubDag {
            blocks: vec![leader.clone()],
            leader,
            leader_round: height.saturating_mul(3),
            leader_author: 0,
            non_blue: vec![],
        };
        let members: Vec<_> = committee
            .authorities
            .iter()
            .map(|authority| (authority.index, authority.public_key, authority.stake))
            .collect();
        leader_header(
            &subdag,
            height,
            parent,
            state,
            committee.epoch,
            committee_seal_hash(committee.epoch, &members),
            false,
        )
    }

    fn seals_for(
        signers: &[SigningKey],
        header: &LeaderHeader,
        chain_id: u64,
        n: u16,
    ) -> Vec<CommitSeal> {
        (0..n)
            .map(|index| CommitSeal {
                authority_index: index,
                signature: kovanica_crypto::sign(
                    &signers[index as usize],
                    &header.seal_message(chain_id),
                ),
            })
            .collect()
    }

    #[test]
    fn three_of_four_seals_pass_and_a_bad_seal_fails() {
        let (signers, committee) = keys(4);
        let header = header_for(&committee, 0, Hash::zero(), Hash::new([4u8]));
        let ok = seals_for(&signers, &header, 1337, 3);
        assert!(verify_sealed_header(1337, &committee, &header, &ok, None).is_ok());
        let short = seals_for(&signers, &header, 1337, 2);
        assert!(verify_sealed_header(1337, &committee, &header, &short, None).is_err());
        let mut bad = ok.clone();
        bad[0].signature = Signature([9u8; 64]);
        assert!(matches!(
            verify_sealed_header(1337, &committee, &header, &bad, None),
            Err(LightError::Seals(_))
        ));
        let mut timed = header.clone();
        timed.timestamp = 1;
        assert_eq!(
            verify_sealed_header(1337, &committee, &timed, &ok, None).unwrap_err(),
            LightError::Timestamp
        );
    }

    #[test]
    fn the_next_header_must_name_the_previous_leader() {
        let (signers, committee) = keys(4);
        let first = header_for(&committee, 0, Hash::zero(), Hash::zero());
        let second = header_for(&committee, 1, first.leader_digest, Hash::new([2u8]));
        let seals = seals_for(&signers, &second, 1337, 3);
        assert!(verify_sealed_header(1337, &committee, &second, &seals, Some(&first)).is_ok());
        let broken = header_for(&committee, 1, Hash::new([9u8]), Hash::new([2u8]));
        let broken_seals = seals_for(&signers, &broken, 1337, 3);
        assert_eq!(
            verify_sealed_header(1337, &committee, &broken, &broken_seals, Some(&first))
                .unwrap_err(),
            LightError::Parent
        );
    }

    #[test]
    fn an_account_proof_confirms_the_recipient_balance() {
        let (_signers, committee) = keys(4);
        let address = Address([7u8; 32]);
        let mut tree = AccountSmt::empty();
        tree.insert(address, 25, 2);
        let proof = tree.prove(&address, 25, 2, true);
        let header = header_for(&committee, 3, Hash::zero(), tree.root());
        let state = StateProof {
            state_root: tree.root(),
            address,
            balance: 25,
            nonce: 2,
            present: true,
            siblings: proof.siblings.to_vec(),
        };
        assert!(verify_account(&header, &state));
        let mut lie = state.clone();
        lie.balance = 26;
        assert!(!verify_account(&header, &lie));
    }

    #[test]
    fn a_signed_transfer_verifies_under_the_configured_chain_id() {
        let signer = SigningKey::from_bytes(&[9u8; 32]);
        let sender = Address::from_public_key(&PublicKey::from(signer.verifying_key()));
        let to = Address([3u8; 32]);
        let tx = sign_transfer(1337, &signer, sender, to, 10, 1, 1);
        assert!(tx.verify_signature(&SigningContext::new(1337)));
        assert!(!tx.verify_signature(&SigningContext::new(2)));
        let leaf = tx.hash;
        let proof = merkle::proof(&[leaf], 0).unwrap();
        let mut header = header_for(&keys(4).1, 0, Hash::zero(), Hash::zero());
        header.executed_tx_root = leaf;
        assert!(verify_executed_tx(&header, &leaf, &proof));
    }

    #[test]
    fn light_node_transactions_verify_under_the_configured_chain_id() {
        let signer = SigningKey::from_bytes(&[4u8; 32]);
        let sender = Address::from_public_key(&PublicKey::from(signer.verifying_key()));
        let ctx = SigningContext::new(1337);
        let register =
            sign_register_light_node(1337, &signer, sender, PublicKey([8u8; 32]), 100, 0, 1);
        let exit = sign_exit_light_node(1337, &signer, sender, 1, 1);
        let witness = sign_light_witness(1337, &signer, sender, 3, [2u8; 32], [5u8; 96], 2, 1);
        assert!(register.verify_signature(&ctx));
        assert!(exit.verify_signature(&ctx));
        assert!(witness.verify_signature(&ctx));
        assert!(!witness.verify_signature(&SigningContext::new(2)));
        assert!(matches!(
            register.kind,
            TransactionKind::RegisterLightNode { stake: 100, .. }
        ));
        assert!(matches!(exit.kind, TransactionKind::ExitLightNode));
        assert!(matches!(
            witness.kind,
            TransactionKind::SubmitLightWitness { epoch: 3, .. }
        ));
    }
}
