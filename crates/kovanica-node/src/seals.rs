//! Post-execution commit seals.
//!
//! After a committed sub-DAG is executed, the node stores the leader header
//! it computed and, when it is a validator, signs that header. Consensus
//! does not wait for the seal. A gossiped seal is stored only when its
//! header hash equals the header local execution already saved.

use std::sync::Arc;

use anyhow::Result;
use kovanica_storage::Storage;
use kovanica_types::{CommitSeal, CommittedSubDag, Hash, LeaderHeader, SealAnnouncement, SigningKey};
use tracing::warn;

use kovanica_network::LocalHeaderHash;

/// Header stored for a commit, plus the seal to gossip when we signed one.
pub struct LocalSeal {
    /// Header written to the state database.
    pub header: LeaderHeader,
    /// Our seal, present only for a committee signer.
    pub gossip: Option<SealAnnouncement>,
}

/// What this process needs in order to seal a commit.
pub struct SealContext {
    /// Chain id mixed into the seal message. Taken from local configuration.
    pub chain_id: u64,
    /// Committee epoch. Zero until epoch rotation is live.
    pub epoch: u64,
    /// Validator index and key. `None` on a full node that does not vote.
    pub signer: Option<(u16, SigningKey)>,
    /// [`kovanica_types::committee_seal_hash`] of the live committee.
    pub committee_hash: Hash,
    /// Mirrors `ConsensusConfig::use_mysticghost`. Default on.
    pub mysticghost: bool,
}

/// Reads leader-header hashes out of the state database for the gossip edge.
pub struct StateHeaderIndex {
    /// State database that holds `leader_headers`.
    pub storage: Arc<Storage>,
}

impl LocalHeaderHash for StateHeaderIndex {
    fn local_header_hash(&self, committed_height: u64) -> Option<Hash> {
        let txn = self.storage.begin_read().ok()?;
        let header = self
            .storage
            .state()
            .get_leader_header(&txn, committed_height)
            .ok()??;
        Some(header.hash())
    }
}

/// Save the header for this commit and sign it when we are a validator.
///
/// Returns the stored header and, for a validator, the announcement to
/// gossip. Replacing a stored header with a different hash drops seals for
/// that height before the new seal is written.
pub fn publish_local_seal(
    storage: &Storage,
    ctx: &SealContext,
    subdag: &CommittedSubDag,
    committed_height: u64,
    state_root: Hash,
) -> Result<LocalSeal> {
    let parent_leader_digest = parent_digest(storage, committed_height)?;
    let header = kovanica_types::leader_header(
        subdag,
        committed_height,
        parent_leader_digest,
        state_root,
        ctx.epoch,
        ctx.committee_hash,
        ctx.mysticghost,
    );
    let hashes = kovanica_types::executed_tx_hashes(subdag);
    let txn = storage.begin_write()?;
    storage.state().save_leader_header(&txn, &header)?;
    storage
        .state()
        .save_executed_tx_hashes(&txn, committed_height, &hashes)?;
    let gossip = match ctx.signer.as_ref() {
        Some((index, key)) if *index != u16::MAX => {
            let signature = kovanica_crypto::sign(key, &header.seal_message(ctx.chain_id));
            let seal = CommitSeal {
                authority_index: *index,
                signature,
            };
            storage
                .state()
                .save_commit_seal(&txn, committed_height, &seal)?;
            Some(SealAnnouncement {
                header: header.clone(),
                seal,
            })
        }
        _ => None,
    };
    txn.commit()?;
    Ok(LocalSeal { header, gossip })
}

/// Store a gossiped seal when it signs the header local execution saved.
///
/// Returns `false` when this node has no header at that height, or the
/// header hash differs. The foreign header is not kept.
pub fn store_matching_seal(storage: &Storage, announcement: &SealAnnouncement) -> Result<bool> {
    let txn = storage.begin_write()?;
    let stored = storage.state().save_seal_if_header(
        &txn,
        announcement.header.committed_height,
        &announcement.header.hash(),
        &announcement.seal,
    )?;
    if !stored {
        return Ok(false);
    }
    txn.commit()?;
    Ok(true)
}

fn parent_digest(storage: &Storage, committed_height: u64) -> Result<Hash> {
    if committed_height == 0 {
        return Ok(Hash::zero());
    }
    let txn = storage.begin_read()?;
    match storage
        .state()
        .get_leader_header(&txn, committed_height - 1)?
    {
        Some(header) => Ok(header.leader_digest),
        None => {
            warn!(
                height = committed_height,
                "no parent leader header; parent digest is zero"
            );
            Ok(Hash::zero())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::{block::StatementBlock, Signature};
    use tempfile::tempdir;

    fn empty_block(author: u16, round: u64) -> StatementBlock {
        let digest = StatementBlock::compute_digest(author, round, &[], &[]);
        StatementBlock {
            author,
            round,
            parents: vec![],
            transactions: vec![],
            statements: vec![],
            signature: Signature([0; 64]),
            digest,
            merkle_root: Hash::zero(),
        }
    }

    fn subdag(round: u64) -> CommittedSubDag {
        let leader = empty_block(0, round);
        CommittedSubDag {
            blocks: vec![leader.clone()],
            leader,
            leader_round: round,
            leader_author: 0,
            non_blue: vec![],
        }
    }

    fn context(signer: Option<(u16, SigningKey)>) -> SealContext {
        SealContext {
            chain_id: 1337,
            epoch: 0,
            signer,
            committee_hash: Hash::zero(),
            mysticghost: false,
        }
    }

    #[test]
    fn local_seal_chains_parent_digests_and_rejects_a_foreign_header() {
        let dir = tempdir().unwrap();
        let storage = Storage::new(dir.path().join("state.redb")).unwrap();
        let key = SigningKey::from_bytes(&[7u8; 32]);
        let ctx = context(Some((0, key.clone())));
        let first = subdag(3);
        let announcement = publish_local_seal(&storage, &ctx, &first, 0, Hash::new([1u8]))
            .unwrap()
            .gossip
            .expect("validator signs");
        assert_eq!(announcement.header.timestamp, 0);
        assert_eq!(announcement.header.parent_leader_digest, Hash::zero());
        assert_eq!(announcement.header.blue_root, Hash::zero());
        assert!(announcement.header.colouring_roots_match_flag());
        kovanica_crypto::verify_commit_seal(
            1337,
            &announcement.header,
            &announcement.seal,
            &kovanica_types::PublicKey::from(key.verifying_key()),
        )
        .unwrap();

        let second = subdag(6);
        let next = publish_local_seal(&storage, &ctx, &second, 1, Hash::new([2u8]))
            .unwrap()
            .gossip
            .unwrap();
        assert_eq!(next.header.parent_leader_digest, first.leader.digest);

        let mut foreign = announcement.clone();
        foreign.header.state_root = Hash::new([9u8]);
        foreign.seal.authority_index = 1;
        assert!(!store_matching_seal(&storage, &foreign).unwrap());

        let mut other = next.clone();
        other.seal.authority_index = 3;
        other.seal.signature = Signature([4; 64]);
        assert!(store_matching_seal(&storage, &other).unwrap());

        let read = storage.begin_read().unwrap();
        let seals = storage.state().seals_for(&read, 1).unwrap();
        assert_eq!(seals.len(), 2);
        assert!(storage
            .state()
            .get_leader_header(&read, 0)
            .unwrap()
            .is_some());
    }

    #[test]
    fn a_non_validator_stores_the_header_without_a_seal() {
        let dir = tempdir().unwrap();
        let storage = Storage::new(dir.path().join("state.redb")).unwrap();
        let local =
            publish_local_seal(&storage, &context(None), &subdag(3), 0, Hash::zero()).unwrap();
        assert!(local.gossip.is_none());
        let read = storage.begin_read().unwrap();
        assert!(storage.state().seals_for(&read, 0).unwrap().is_empty());
        assert!(storage
            .state()
            .get_leader_header(&read, 0)
            .unwrap()
            .is_some());
    }
}
