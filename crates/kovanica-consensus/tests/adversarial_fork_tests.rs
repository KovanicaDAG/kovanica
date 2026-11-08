//! Adversarial Equivocation and Fork Handling Tests
//!
//! Formal tests exercising:
//! 1. Equivocation with `use_min_digest_fork = true`: Two competing blocks B1 and B2
//!    created for the same round/leader with different transactions/digests. Verify
//!    all nodes deterministically choose the candidate with `min(digest(B1), digest(B2))`.
//! 2. Equivocation detection & proof generation: When both competing blocks are
//!    observed by a validator, verify `EquivocationProof` is generated containing
//!    signatures for both blocks by the offending validator.
//! 3. Reorg rejection past commit certification: A block that has been certified
//!    in a wave commit cannot be displaced by an alternate branch.

use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::sync::Arc;

use kovanica_consensus::engine::{BlockManagerTrait, DagStoreTrait};
use kovanica_consensus::types::{AuthorityInfo, CommitteeInfo};
use kovanica_consensus::{ConsensusConfig, ConsensusEngine, ConsensusError};
use kovanica_dag::{BlockManagerError, DagStoreError};
use kovanica_mempool::Mempool;
use kovanica_params::SLASHING_ACTIVE_FROM_ROUND;
use kovanica_types::block::{BlockReference, StatementBlock};
use kovanica_types::evidence::{EvidenceError, SignedBlockHeader};
use kovanica_types::hash::Hash;
use kovanica_types::{
    Address, AuthorityIndex, EquivocationEvidence, PublicKey, Round, Signature, SigningContext,
    SigningKey, Transaction, TransactionKind,
};

/// Type alias for specification conformance:
/// `EquivocationProof` is the canonical double-signing evidence.
pub type EquivocationProof = EquivocationEvidence;

const TEST_CTX: SigningContext = SigningContext::new(kovanica_types::signing::chain_id::LOCAL);

/// In-memory DAG store implementing [`DagStoreTrait`] for fork and reorg tests.
#[derive(Default)]
struct ForkTestDag {
    blocks: RwLock<HashMap<Hash, StatementBlock>>,
    decisions: Mutex<Vec<(Round, Hash)>>,
}

impl ForkTestDag {
    fn new() -> Self {
        Self::default()
    }

    fn put(&self, block: StatementBlock) {
        self.blocks.write().insert(block.digest, block);
    }
}

impl DagStoreTrait for ForkTestDag {
    fn get_block(&self, hash: &Hash) -> Result<StatementBlock, DagStoreError> {
        self.blocks
            .read()
            .get(hash)
            .cloned()
            .ok_or_else(|| DagStoreError::NotFound(hash.to_string()))
    }

    fn get_ancestors(&self, hash: &Hash, min_round: Round) -> Result<Vec<Hash>, DagStoreError> {
        let blocks = self.blocks.read();
        let mut ancestors = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(*hash);

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            if let Some(block) = blocks.get(&current) {
                for parent in &block.parents {
                    if parent.digest != current {
                        if let Some(pb) = blocks.get(&parent.digest) {
                            if pb.round >= min_round {
                                ancestors.push(parent.digest);
                                queue.push_back(parent.digest);
                            }
                        }
                    }
                }
            }
        }
        Ok(ancestors)
    }

    fn get_parents(&self, hash: &Hash) -> Result<Vec<Hash>, DagStoreError> {
        let blocks = self.blocks.read();
        let block = blocks
            .get(hash)
            .ok_or_else(|| DagStoreError::NotFound(hash.to_string()))?;
        Ok(block.parents.iter().map(|p| p.digest).collect())
    }

    fn get_block_by_author_round(
        &self,
        author: AuthorityIndex,
        round: Round,
    ) -> Result<Option<StatementBlock>, DagStoreError> {
        Ok(self
            .blocks
            .read()
            .values()
            .find(|b| b.author == author && b.round == round)
            .cloned())
    }

    fn get_blocks_by_round(&self, round: Round) -> Result<Vec<StatementBlock>, DagStoreError> {
        Ok(self
            .blocks
            .read()
            .values()
            .filter(|b| b.round == round)
            .cloned()
            .collect())
    }

    fn has_block(&self, hash: &Hash) -> Result<bool, DagStoreError> {
        Ok(self.blocks.read().contains_key(hash))
    }

    fn put_block(&self, block: &StatementBlock) -> Result<(), DagStoreError> {
        self.blocks.write().insert(block.digest, block.clone());
        Ok(())
    }

    fn replace_block(
        &self,
        existing: &StatementBlock,
        incoming: &StatementBlock,
    ) -> Result<(), DagStoreError> {
        if existing.author != incoming.author || existing.round != incoming.round {
            return Err(DagStoreError::NotFound(
                "replace requires same author and round".into(),
            ));
        }
        // Invariant 1: Round must not already be decided.
        if self
            .decisions
            .lock()
            .iter()
            .any(|(r, _)| *r == existing.round)
        {
            return Err(DagStoreError::NotFound(format!(
                "round {} is already decided",
                existing.round
            )));
        }
        // Invariant 2: Existing leader block must have no dependent children in the DAG.
        let mut blocks = self.blocks.write();
        let has_children = blocks.values().any(|b| {
            b.parents
                .iter()
                .any(|p| p.digest == existing.digest || p.digest == incoming.digest)
        });
        if has_children {
            return Err(DagStoreError::NotFound(
                "existing leader block already has children".into(),
            ));
        }
        if !blocks.contains_key(&existing.digest) {
            return Err(DagStoreError::NotFound(existing.digest.to_string()));
        }
        // Atomic swap
        blocks.remove(&existing.digest);
        blocks.insert(incoming.digest, incoming.clone());
        Ok(())
    }

    fn find_parents(
        &self,
        _round: Round,
        _max_parents: usize,
    ) -> Result<Vec<BlockReference>, DagStoreError> {
        Ok(Vec::new())
    }

    fn commit_leader(&self, _leader_hash: &Hash) -> Result<u64, DagStoreError> {
        Ok(self.decisions.lock().len() as u64)
    }

    fn mark_decided_and_commit_leader(
        &self,
        round: Round,
        leader_hash: &Hash,
    ) -> Result<u64, DagStoreError> {
        self.mark_round_decided(round, leader_hash)?;
        self.commit_leader(leader_hash)
    }

    fn mark_round_decided(&self, round: Round, leader_hash: &Hash) -> Result<(), DagStoreError> {
        let mut decisions = self.decisions.lock();
        if !decisions.contains(&(round, *leader_hash)) {
            decisions.push((round, *leader_hash));
        }
        Ok(())
    }

    fn mergeset(&self, _leader: &Hash) -> Result<Vec<Hash>, DagStoreError> {
        Ok(Vec::new())
    }

    fn get_blocks(&self, hashes: &[Hash]) -> Result<Vec<StatementBlock>, DagStoreError> {
        let blocks = self.blocks.read();
        Ok(hashes
            .iter()
            .filter_map(|h| blocks.get(h).cloned())
            .collect())
    }

    fn get_decided_leaders(&self, round: Round) -> Result<Vec<Hash>, DagStoreError> {
        Ok(self
            .decisions
            .lock()
            .iter()
            .filter(|(r, _)| *r == round)
            .map(|(_, h)| *h)
            .collect())
    }

    fn get_decided_rounds(&self, max_round: Round) -> Result<Vec<Round>, DagStoreError> {
        Ok(self
            .decisions
            .lock()
            .iter()
            .filter(|(r, _)| *r <= max_round)
            .map(|(r, _)| *r)
            .collect())
    }

    fn prune_non_blue(&self, _blue: &[Hash], _wave: u64) -> Result<u64, DagStoreError> {
        Ok(0)
    }

    fn prune_waves_before(&self, _wave: u64, _window: u64) -> Result<u64, DagStoreError> {
        Ok(0)
    }
}

/// Block manager test double.
struct ForkTestBlockManager {
    authority: AuthorityIndex,
    dag: Arc<ForkTestDag>,
    signing_key: RwLock<SigningKey>,
}

impl ForkTestBlockManager {
    fn new(authority: AuthorityIndex, dag: Arc<ForkTestDag>, signing_key: SigningKey) -> Self {
        Self {
            authority,
            dag,
            signing_key: RwLock::new(signing_key),
        }
    }
}

impl BlockManagerTrait for ForkTestBlockManager {
    fn propose_block(&self, round: Round) -> Result<StatementBlock, BlockManagerError> {
        self.propose_block_with_txs(round, Vec::new())
    }

    fn propose_block_with_txs(
        &self,
        round: Round,
        transactions: Vec<Transaction>,
    ) -> Result<StatementBlock, BlockManagerError> {
        let block = make_test_block(
            &self.signing_key.read(),
            self.authority,
            round,
            Vec::new(),
            transactions,
        );
        self.dag.put(block.clone());
        Ok(block)
    }

    fn process_block(&self, block: &StatementBlock) -> Result<(), BlockManagerError> {
        self.dag.put(block.clone());
        Ok(())
    }

    fn our_authority(&self) -> AuthorityIndex {
        self.authority
    }

    fn set_signing_key(&self, key: SigningKey) {
        *self.signing_key.write() = key;
    }
}

fn sign_block_digest(key: &SigningKey, digest: &Hash) -> Signature {
    kovanica_crypto::sign(
        key,
        &kovanica_types::block::block_signature_data(TEST_CTX.chain_id, digest),
    )
}

fn make_transfer_tx(sender_key: &SigningKey, nonce: u64, amount: u64) -> Transaction {
    let sender = Address::from_public_key(&PublicKey(sender_key.verifying_key().to_bytes()));
    let recipient = Address([99u8; 32]);
    let mut tx = Transaction {
        sender,
        nonce,
        kind: TransactionKind::Transfer {
            to: recipient,
            amount,
        },
        fee: 1000,
        signature: Signature([0u8; 64]),
        hash: Hash::zero(),
    };
    tx.hash = tx.signing_hash(&TEST_CTX);
    tx.signature = Signature(kovanica_crypto::sign(sender_key, &tx.hash.0).0);
    tx
}

fn make_test_block(
    key: &SigningKey,
    author: AuthorityIndex,
    round: Round,
    parents: Vec<BlockReference>,
    transactions: Vec<Transaction>,
) -> StatementBlock {
    let digest = StatementBlock::compute_digest(author, round, &parents, &transactions);
    let signature = sign_block_digest(key, &digest);
    StatementBlock {
        author,
        round,
        parents,
        transactions,
        statements: Vec::new(),
        signature,
        digest,
        merkle_root: Hash::zero(),
    }
}

fn make_test_vote(
    key: &SigningKey,
    leader_round: Round,
    voter: AuthorityIndex,
    leader_hash: Hash,
) -> kovanica_types::Vote {
    let epoch = kovanica_types::epoch_for_round(leader_round);
    let ctx = SigningContext {
        chain_id: TEST_CTX.chain_id,
        epoch,
        genesis_hash: TEST_CTX.genesis_hash,
    };
    let mut vote = kovanica_types::Vote {
        leader_round,
        leader_hash,
        voter,
        signature: Signature([0u8; 64]),
    };
    vote.signature = kovanica_crypto::sign(key, &vote.signature_data(&ctx));
    vote
}

fn test_committee(signing_keys: &[SigningKey]) -> CommitteeInfo {
    let authorities = signing_keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let public_key = PublicKey(key.verifying_key().to_bytes());
            let address = Address::from_public_key(&public_key);
            AuthorityInfo {
                index: i as AuthorityIndex,
                stake: 1,
                public_key,
                vrf_public_key: None,
                address,
                network_address: format!("/ip4/127.0.0.1/tcp/90{:02}", i),
            }
        })
        .collect();
    CommitteeInfo::try_new(0, authorities).expect("test committee is valid")
}

fn test_node(
    index: AuthorityIndex,
    committee: CommitteeInfo,
    config: ConsensusConfig,
    signing_keys: &[SigningKey],
    mempool: Option<Arc<Mempool>>,
) -> (
    ConsensusEngine<ForkTestDag, ForkTestBlockManager>,
    Arc<ForkTestDag>,
) {
    let dag = Arc::new(ForkTestDag::new());
    let manager = Arc::new(RwLock::new(ForkTestBlockManager::new(
        index,
        dag.clone(),
        signing_keys[index as usize].clone(),
    )));
    let engine = ConsensusEngine::new(
        config,
        committee,
        dag.clone(),
        manager,
        signing_keys[index as usize].clone(),
        mempool,
        TEST_CTX,
    );
    (engine, dag)
}

// =========================================================================
// Test 1: Equivocation with use_min_digest_fork = true
// =========================================================================

#[test]
fn test_equivocation_min_digest_fork_convergence() {
    let signing_keys: Vec<SigningKey> = (0..4)
        .map(|i| {
            let mut seed = [0u8; 32];
            seed[..2].copy_from_slice(&(i as u16).to_le_bytes());
            seed[31] = 0xfa;
            SigningKey::from_bytes(&seed)
        })
        .collect();

    let committee = test_committee(&signing_keys);
    let config = ConsensusConfig {
        round_duration_ms: 50,
        use_min_digest_fork: true,
        use_vrf_leader: false,
        ..ConsensusConfig::default()
    };

    // Spin up 4 validator nodes.
    let (node0, dag0) = test_node(0, committee.clone(), config.clone(), &signing_keys, None);
    let (node1, dag1) = test_node(1, committee.clone(), config.clone(), &signing_keys, None);
    let (node2, dag2) = test_node(2, committee.clone(), config.clone(), &signing_keys, None);
    let (node3, dag3) = test_node(3, committee.clone(), config.clone(), &signing_keys, None);

    // Round 3 is slot 1 (WAVE_LENGTH = 3).
    // In round-robin with 4 authorities, scheduled leader for round 3 is authority 1.
    assert_eq!(node0.scheduled_leader_for_round(3), Some(1));
    assert_eq!(node1.scheduled_leader_for_round(3), Some(1));
    assert_eq!(node2.scheduled_leader_for_round(3), Some(1));
    assert_eq!(node3.scheduled_leader_for_round(3), Some(1));

    // Adversarial leader 1 produces two competing blocks B1 and B2 with different transactions.
    let tx_a = make_transfer_tx(&signing_keys[1], 1, 10_000_000_000); // 10 KUNA
    let tx_b = make_transfer_tx(&signing_keys[1], 1, 20_000_000_000); // 20 KUNA

    let block_1 = make_test_block(&signing_keys[1], 1, 3, Vec::new(), vec![tx_a]);
    let block_2 = make_test_block(&signing_keys[1], 1, 3, Vec::new(), vec![tx_b]);

    assert_ne!(block_1.digest, block_2.digest, "digests must differ");
    assert_eq!(block_1.author, block_2.author);
    assert_eq!(block_1.round, block_2.round);

    // Identify lexicographically smaller and larger blocks.
    let (min_block, max_block) = if block_1.digest.0 < block_2.digest.0 {
        (block_1, block_2)
    } else {
        (block_2, block_1)
    };

    assert!(min_block.digest.0 < max_block.digest.0);

    // Nodes 0 and 2 observe min_block first, then max_block.
    // Node 0:
    node0.process_block(&min_block).expect("min_block accepted");
    let err0 = node0
        .process_block(&max_block)
        .expect_err("max_block rejected");
    assert!(
        matches!(err0, ConsensusError::InvalidVote(_)),
        "expected InvalidVote fork rejection, got {:?}",
        err0
    );

    // Node 2:
    node2.process_block(&min_block).expect("min_block accepted");
    let err2 = node2
        .process_block(&max_block)
        .expect_err("max_block rejected");
    assert!(matches!(err2, ConsensusError::InvalidVote(_)));

    // Nodes 1 and 3 observe max_block first, then min_block.
    // When min_block arrives, min-digest swap activates: drops max_block, stores min_block!
    // Node 1:
    node1
        .process_block(&max_block)
        .expect("max_block accepted initially");
    node1
        .process_block(&min_block)
        .expect("min_block replaces max_block via min_digest swap");

    // Node 3:
    node3
        .process_block(&max_block)
        .expect("max_block accepted initially");
    node3
        .process_block(&min_block)
        .expect("min_block replaces max_block via min_digest swap");

    // INVARIANT VERIFICATION:
    // Every single node deterministically chose the min_digest candidate!
    for (idx, (engine, dag)) in [
        (0, (&node0, &dag0)),
        (1, (&node1, &dag1)),
        (2, (&node2, &dag2)),
        (3, (&node3, &dag3)),
    ] {
        assert!(
            dag.has_block(&min_block.digest).unwrap(),
            "node {} store must hold min_digest block",
            idx
        );
        assert!(
            !dag.has_block(&max_block.digest).unwrap(),
            "node {} store must NOT hold max_digest block",
            idx
        );
        let leader_info = engine.registered_leader(3).expect("leader info present");
        assert_eq!(
            leader_info.block_hash,
            Some(min_block.digest),
            "node {} leader info must match min_digest",
            idx
        );
    }
}

// =========================================================================
// Test 2: Equivocation detection & proof generation
// =========================================================================

#[test]
fn test_equivocation_detection_and_proof_generation() {
    let signing_keys: Vec<SigningKey> = (0..4)
        .map(|i| {
            let mut seed = [0u8; 32];
            seed[..2].copy_from_slice(&(i as u16).to_le_bytes());
            seed[31] = 0xbb;
            SigningKey::from_bytes(&seed)
        })
        .collect();

    let committee = test_committee(&signing_keys);
    let offender_key = &signing_keys[1];
    let offender_public = PublicKey(offender_key.verifying_key().to_bytes());
    let offender_address = Address::from_public_key(&offender_public);

    // ---------------------------------------------------------------------
    // Part A: Direct EquivocationProof verification rules
    // ---------------------------------------------------------------------
    let tx1 = make_transfer_tx(offender_key, 1, 1_000_000_000);
    let tx2 = make_transfer_tx(offender_key, 1, 2_000_000_000);
    let b1 = make_test_block(offender_key, 1, 3, Vec::new(), vec![tx1]);
    let b2 = make_test_block(offender_key, 1, 3, Vec::new(), vec![tx2]);

    assert_ne!(b1.digest, b2.digest);

    // Construct EquivocationProof from the two conflicting blocks.
    let proof: EquivocationProof =
        EquivocationEvidence::from_blocks(&b1, &b2, offender_address).expect("valid proof");

    // Offender check.
    assert_eq!(proof.offender().expect("offender valid"), offender_address);

    // Header structure and signature check.
    assert_eq!(proof.header_a.author, offender_address);
    assert_eq!(proof.header_b.author, offender_address);
    assert_eq!(proof.header_a.round, 3);
    assert_eq!(proof.header_b.round, 3);
    assert_eq!(proof.header_a.digest, b1.digest);
    assert_eq!(proof.header_b.digest, b2.digest);
    assert_eq!(proof.header_a.signature, b1.signature);
    assert_eq!(proof.header_b.signature, b2.signature);

    // Both signatures strictly verify under the local chain ID.
    assert!(proof.header_a.verify_chain(TEST_CTX.chain_id));
    assert!(proof.header_b.verify_chain(TEST_CTX.chain_id));

    // Negative verification cases:
    // 1. Same digest is not equivocation.
    let same_err = EquivocationEvidence::from_blocks(&b1, &b1, offender_address).unwrap_err();
    assert_eq!(same_err, EvidenceError::SameDigest);

    // 2. Author mismatch.
    let other_address = Address([88u8; 32]);
    let mismatched_proof = EquivocationProof {
        header_a: SignedBlockHeader::from_block(&b1, offender_address),
        header_b: SignedBlockHeader::from_block(&b2, other_address),
    };
    assert_eq!(
        mismatched_proof.offender().unwrap_err(),
        EvidenceError::AuthorMismatch
    );

    // 3. Round mismatch.
    let b3 = make_test_block(offender_key, 1, 6, Vec::new(), Vec::new());
    let round_mismatch = EquivocationEvidence::from_blocks(&b1, &b3, offender_address).unwrap_err();
    assert_eq!(round_mismatch, EvidenceError::RoundMismatch);

    // 4. Bad / forged signature.
    let mut bad_proof = proof.clone();
    bad_proof.header_a.signature = Signature([0xee; 64]);
    assert_eq!(
        bad_proof.offender().unwrap_err(),
        EvidenceError::BadSignature
    );

    // ---------------------------------------------------------------------
    // Part B: End-to-end consensus engine equivocation detection & mempool reporting
    // ---------------------------------------------------------------------
    let dir = tempfile::tempdir().expect("tempdir");
    let storage = Arc::new(
        kovanica_storage::Storage::new(dir.path().join("equivocation.redb")).expect("storage"),
    );

    let observer_key = &signing_keys[0];
    let observer_address =
        Address::from_public_key(&PublicKey(observer_key.verifying_key().to_bytes()));

    // Populate StakingState so both observer and offender are recognized active validators.
    let mut staking = kovanica_staking::StakingState::new();
    staking
        .join_validator(
            observer_address,
            kovanica_staking::MIN_VALIDATOR_STAKE,
            0,
            None,
            None,
            None,
        )
        .expect("join observer");
    staking
        .join_validator(
            offender_address,
            kovanica_staking::MIN_VALIDATOR_STAKE,
            0,
            None,
            None,
            None,
        )
        .expect("join offender");

    let txn = storage.begin_write().expect("write");
    storage
        .state()
        .save_staking_state(&txn, &staking)
        .expect("save staking");
    txn.commit().expect("commit");

    let mempool = Arc::new(Mempool::new(Default::default(), storage.clone(), TEST_CTX));

    let config = ConsensusConfig {
        round_duration_ms: 100,
        use_min_digest_fork: false, // In first-valid mode, second block always triggers equivocation
        use_vrf_leader: false,
        ..ConsensusConfig::default()
    };

    let (engine, dag) = test_node(
        0,
        committee.clone(),
        config,
        &signing_keys,
        Some(mempool.clone()),
    );

    // Target round at or above SLASHING_ACTIVE_FROM_ROUND (u64::MAX).
    // u64::MAX % 3 == 0 (leader round), and (u64::MAX / 3) % 4 == 1 (scheduled leader is authority 1).
    let slash_round = SLASHING_ACTIVE_FROM_ROUND;
    assert_eq!(engine.scheduled_leader_for_round(slash_round), Some(1));

    let slash_tx1 = make_transfer_tx(offender_key, 1, 5_000_000_000);
    let slash_tx2 = make_transfer_tx(offender_key, 1, 9_000_000_000);

    let slash_b1 = make_test_block(offender_key, 1, slash_round, Vec::new(), vec![slash_tx1]);
    let slash_b2 = make_test_block(offender_key, 1, slash_round, Vec::new(), vec![slash_tx2]);

    assert_eq!(mempool.len(), 0, "mempool starts empty");

    // Ingest first block: accepted.
    engine.process_block(&slash_b1).expect("slash_b1 accepted");
    assert_eq!(mempool.len(), 0, "no evidence on first block");
    assert!(dag.has_block(&slash_b1.digest).unwrap());

    // Ingest second conflicting block: equivocation detected and reported to mempool!
    let err = engine
        .process_block(&slash_b2)
        .expect_err("slash_b2 must be rejected as equivocation");
    assert!(matches!(err, ConsensusError::InvalidVote(_)));

    // Verify evidence transaction admitted to the mempool.
    assert_eq!(mempool.len(), 1, "evidence transaction must be submitted");
    let tx_hash = mempool.pending_hashes()[0];
    let tx = mempool.get(&tx_hash).expect("evidence tx present");

    // The observing validator signed the report with fee 0.
    assert_eq!(tx.sender, observer_address);
    assert_eq!(tx.fee, 0);
    assert!(
        tx.verify_signature(&TEST_CTX),
        "evidence tx signature must verify"
    );

    // Verify EquivocationProof payload.
    match &tx.kind {
        TransactionKind::Evidence { evidence } => {
            assert_eq!(
                evidence.offender().expect("valid offender"),
                offender_address
            );
            assert_eq!(evidence.round(), slash_round);
            assert_eq!(evidence.header_a.digest, slash_b1.digest);
            assert_eq!(evidence.header_b.digest, slash_b2.digest);
            assert_eq!(evidence.header_a.signature, slash_b1.signature);
            assert_eq!(evidence.header_b.signature, slash_b2.signature);
            assert!(evidence.header_a.verify_chain(TEST_CTX.chain_id));
            assert!(evidence.header_b.verify_chain(TEST_CTX.chain_id));
        }
        other => panic!("expected TransactionKind::Evidence, got {:?}", other),
    }
}

// =========================================================================
// Test 3: Reorg rejection past commit certification
// =========================================================================

#[test]
fn test_reorg_rejection_past_commit_certification() {
    let signing_keys: Vec<SigningKey> = (0..4)
        .map(|i| {
            let mut seed = [0u8; 32];
            seed[..2].copy_from_slice(&(i as u16).to_le_bytes());
            seed[31] = 0xcc;
            SigningKey::from_bytes(&seed)
        })
        .collect();

    let committee = test_committee(&signing_keys);
    let config = ConsensusConfig {
        round_duration_ms: 100,
        use_min_digest_fork: true, // Even with min-digest fork on, decided blocks cannot be replaced!
        use_vrf_leader: false,
        ..ConsensusConfig::default()
    };

    let (engine, dag) = test_node(0, committee.clone(), config, &signing_keys, None);

    // Round 3 leader is authority 1.
    assert_eq!(engine.scheduled_leader_for_round(3), Some(1));

    let tx_canonical = make_transfer_tx(&signing_keys[1], 1, 50_000_000_000); // 50 KUNA
    let canonical_block = make_test_block(&signing_keys[1], 1, 3, Vec::new(), vec![tx_canonical]);

    // Process canonical block.
    engine
        .process_block(&canonical_block)
        .expect("canonical block accepted");
    assert!(dag.has_block(&canonical_block.digest).unwrap());

    // Quorum of validators (0, 1, 2) cast votes for canonical block at round 3.
    // Quorum threshold for committee(4) is 3 votes (2f + 1 = 2*1 + 1 = 3).
    for voter in [0, 1, 2] {
        let vote = make_test_vote(
            &signing_keys[voter as usize],
            3,
            voter,
            canonical_block.digest,
        );
        engine.process_vote(&vote).expect("vote processed");
    }

    // Drain commits to certify the wave commit.
    engine.drain_commits().expect("drain commits succeeds");

    // Invariant Check 1: Round 3 is decided and durably marked.
    assert!(
        engine.last_decided_round() >= 3,
        "round 3 must be committed (cursor >= 3, got {})",
        engine.last_decided_round()
    );
    assert_eq!(
        dag.get_decided_leaders(3).unwrap(),
        vec![canonical_block.digest],
        "dag store decisions must record canonical digest"
    );

    // Now an adversary produces an alternate block for round 3 with an artificially
    // smaller digest (which would have qualified for min-digest swap if undecided).
    let mut nonce = 2;
    let mut alternate_block = make_test_block(
        &signing_keys[1],
        1,
        3,
        Vec::new(),
        vec![make_transfer_tx(&signing_keys[1], nonce, 1_000)],
    );

    // Find a transaction / digest strictly smaller than canonical_block.digest.
    while alternate_block.digest.0 >= canonical_block.digest.0 {
        nonce += 1;
        alternate_block = make_test_block(
            &signing_keys[1],
            1,
            3,
            Vec::new(),
            vec![make_transfer_tx(&signing_keys[1], nonce, 1_000)],
        );
    }
    assert!(
        alternate_block.digest.0 < canonical_block.digest.0,
        "alternate block digest must be strictly smaller"
    );

    // Attempt to displace the certified block by processing the alternate block.
    // This MUST be rejected because round 3 is already certified!
    let reorg_attempt = engine.process_block(&alternate_block);
    assert!(
        reorg_attempt.is_err(),
        "processing alternate block past certification must fail"
    );
    let err = reorg_attempt.unwrap_err();
    assert!(
        matches!(err, ConsensusError::InvalidVote(_)),
        "expected InvalidVote, got {:?}",
        err
    );

    // Direct store replacement check: `replace_block` must refuse already decided rounds.
    let direct_store_replace = dag.replace_block(&canonical_block, &alternate_block);
    assert!(
        direct_store_replace.is_err(),
        "dag_store.replace_block must fail on decided round"
    );

    // Invariant Check 2: Canonical block is immutable and remains in store.
    assert!(
        dag.has_block(&canonical_block.digest).unwrap(),
        "canonical block must remain in DAG store"
    );
    assert!(
        !dag.has_block(&alternate_block.digest).unwrap(),
        "alternate block must NOT enter DAG store"
    );
    let decided_leader = engine.registered_leader(3).expect("decided leader present");
    assert_eq!(
        decided_leader.block_hash,
        Some(canonical_block.digest),
        "decided leader must remain canonical block"
    );

    // Reorganization depth bound is 0: committed history cannot be altered.
    assert!(engine.last_decided_round() >= 3);
}
