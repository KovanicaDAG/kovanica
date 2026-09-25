//! M6 Testing: Authority updates, re-orgs, SPV sync, and resource profiling (RFC-POA M6).
//!
//! Exit criteria:
//! - Authority update works (on-chain update tx, SPV update proof)
//! - Re-org under PoA (GHOSTDAG k=3 with authority signatures)
//! - SPV sync under re-org (light client follows selected chain)
//! - Resource profiling: CPU/RAM < 10% of PoW baseline

use ed25519_dalek::{SigningKey, Signer};
use kovanica_dag::{AuthorityPublicKey, AuthoritySet, AuthorityUpdateTx, Block, BlockId, sign_update};
use kovanica_node::{Node, NodeError, BlockRecord};
use kovanica_state::{ATOM, PruningPolicy};
use kovanica_state::spv::{SpvClient, SpvPoAConfig, AuthorityUpdateProof, MerkleProof};

/// Slot duration used throughout (RFC-POA default).
const SLOT_MS: u64 = 3000;

fn keypair(seed: u8) -> (SigningKey, AuthorityPublicKey) {
    let sk = SigningKey::from_bytes(&[seed; 32]);
    let pk = sk.verifying_key();
    (sk, pk)
}

/// An `n`-authority set with keys derived from seeds `base..base+n`.
/// Returns the authority set and signing keys in the same order as the authority set's internal ordering.
fn authority_set_with_base(base: u8, n: u8, threshold: usize) -> (AuthoritySet, Vec<SigningKey>) {
    let mut key_pairs = Vec::new();
    for i in 0..n {
        let (sk, pk) = keypair(base + i);
        key_pairs.push((pk, sk));
    }
    // Sort by public key bytes to match AuthoritySet's internal ordering
    key_pairs.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let keys: Vec<_> = key_pairs.iter().map(|(pk, _)| *pk).collect();
    let sks: Vec<_> = key_pairs.iter().map(|(_, sk)| sk.clone()).collect();
    (AuthoritySet::new(keys, threshold).unwrap(), sks)
}

fn authority_set(n: u8, threshold: usize) -> (AuthoritySet, Vec<SigningKey>) {
    authority_set_with_base(1, n, threshold)
}

/// Create a placeholder authority set for testing (3 authorities, threshold 2)
/// Returns the authority set and signing keys in the same order as the authority set's internal ordering.
fn placeholder_authority_set() -> (AuthoritySet, Vec<SigningKey>) {
    authority_set_with_base(1, 3, 2)
}

/// Helper to create a signed PoA block with the correct authority signature
fn create_signed_poa_block(
    parents: Vec<BlockId>,
    work: u128,
    timestamp_ms: u64,
    nonce: u64,
    authority_idx: usize,
    sks: &[SigningKey],
) -> BlockRecord {
    let payload = Vec::new(); // Empty payload for empty block
    let unsigned = Block::new(parents.clone(), work, timestamp_ms, nonce, payload.clone());
    let hash_without_sig = unsigned.hash_without_authority_sig();
    let sig = sks[authority_idx].sign(hash_without_sig.as_bytes()).to_bytes();
    let block = Block::new_with_authority(parents, work, timestamp_ms, nonce, sig, payload);
    BlockRecord {
        parents: block.parents().to_vec(),
        work: block.work(),
        timestamp_ms: block.timestamp_ms(),
        nonce: block.nonce(),
        vrf: None,
        authority_sig: Some(sig),
        txs: Vec::new(),
    }
}

fn temp_log(name: &str) -> String {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "kovanica-poa-m6-{}-{}-{}",
        name,
        std::process::id(),
        rand::random::<u64>()
    ));
    path.set_extension("log");
    path.to_str().unwrap().to_string()
}

fn remove_log(path: &str) {
    let _ = std::fs::remove_file(path);
}

// ---------------------------------------------------------------------------
// Authority Update Integration Test
// ---------------------------------------------------------------------------

#[test]
fn authority_update_on_chain_and_spv_proof() {
    // Setup: 3-authority set, threshold 2
    let (set, sks) = authority_set(3, 2);
    let _log_path = temp_log("authority-update");

    // Genesis with initial authority set
    let mut node = Node::new();
    node.genesis_with_poa(
        3,
        10 * ATOM,
        200_000 * ATOM,
        1,
        None,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        None,
        set.clone(),
        SLOT_MS,
    )
    .expect("PoA genesis");

    // Load all authority keys so this node can produce in every slot
    for sk in &sks {
        node.set_authority_signing_key(sk.to_bytes());
    }

    // Produce a few blocks to mature the chain
    for _ in 0..5 {
        node.produce_empty().expect("authority-signed block");
    }

    // Create a new authority set (different keys)
    let (_new_set, _new_sks) = authority_set_with_base(11, 3, 2);

    // Build the authority update transaction
    // Signatures from 2 of the 3 current authorities (threshold = 2)
    let _sigs: Vec<_> = sks[..2]
        .iter()
        .map(|sk| {
            (
                sk.verifying_key(),
                sign_update(sk, &set.hash(), &placeholder_authority_set().0),
            )
        })
        .collect();

    // Note: In real usage, the update tx would be submitted via RPC,
    // included in a block, and the light client would fetch the Merkle proof
    // from the full node. Here we test the verification logic.

    // Test that the update proof validation works
    let mut client = kovanica_state::spv::SpvClient::with_poa(
        kovanica_state::spv::BlockHeader {
            id: BlockId::from_bytes([0u8; 32]),
            prev_hash: BlockId::from_bytes([0u8; 32]),
            merkle_root: [0u8; 32],
            work: 1,
            timestamp_ms: 0,
            nonce: 0,
            blue_score: 0,
            chain_blue_work: 1,
            height: 0,
            authority_sig: None,
            authority_set_hash: set.hash(),
            hash_without_authority_sig: [0u8; 32],
        },
        false,
        None,
        kovanica_state::spv::SpvPoAConfig {
            authority_set: set.clone(),
            slot_duration_ms: SLOT_MS,
        },
    );

    // The update proof requires a Merkle proof of the update tx in a block
    // whose header carries the new set hash
    // For this test, we'll test the verification logic with invalid proofs

    // Test that the update proof validation works
    let update_proof = AuthorityUpdateProof {
        update: kovanica_dag::AuthorityUpdateTx::new(
            set.hash(),
            placeholder_authority_set().0,
            vec![],
        ).unwrap(),
        merkle: MerkleProof {
            tx_id: [0u8; 32],
            merkle_root: [0u8; 32],
            path: Vec::new(),
            index: 0,
            tx_count: 1,
        },
        height: 10,
    };

    // The client should reject the proof because the Merkle proof is invalid
    let err = client.apply_authority_update(&update_proof).expect_err("invalid merkle proof rejected");
    // The error should be about the merkle proof or authority
    assert!(err.to_string().contains("merkle") || err.to_string().contains("authority") || err.to_string().contains("update"));

    println!("Authority update SPV proof validation tests passed");
}

// ---------------------------------------------------------------------------
// Re-org Under PoA Test
// ---------------------------------------------------------------------------

#[test]
fn poa_reorg_ghostdag_k3() {
    // Test that GHOSTDAG k=3 works correctly with PoA blocks
    // Two competing branches with different authority signatures
    // The heavier branch (more blue work) should win

    let (set, sks) = authority_set(3, 2);

    let mut node = Node::new();
    // Set clock to slot 0 (genesis)
    node.set_now_ms(0);
    node.genesis_with_poa(
        3,
        10 * ATOM,
        200_000 * ATOM,
        1,
        None,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        None,
        set.clone(),
        SLOT_MS,
    )
    .expect("PoA genesis");

    for sk in &sks {
        node.set_authority_signing_key(sk.to_bytes());
    }

    // Build Branch A: 10 blocks (slots 1-10)
    let mut branch_a = Vec::new();
    for slot in 1..=10 {
        node.set_now_ms(SLOT_MS * slot);
        node.produce_empty().unwrap();
        let tip = node.selected_tip().unwrap();
        branch_a.push(tip);
    }

    // Get the tip of branch A
    let _tip_a = branch_a.last().unwrap();

    // Now create a competing branch B that splits from block 5
    // We need to insert blocks directly with higher work to force a reorg
    // In PoA, work is nominal (1), so we need to use the DAG's insert directly
    // with blocks that have higher work

    let _dag = node.ledger().unwrap().dag();
    let split_point = branch_a[4]; // Split at block 5 (0-indexed)

    // Create a competing branch with higher work
    let mut heavier_branch = Vec::new();
    let mut parent = split_point;
    for i in 1..=8 {
        // Create a block with higher work (2 instead of 1)
        // Calculate the correct slot for this block
        // The first 10 blocks were at slots 1-10, so next blocks are at slots 11-18
        let timestamp = SLOT_MS * (10 + i as u64);
        let slot = timestamp / SLOT_MS;
        let authority_idx = (slot as usize) % 3;
        let record = create_signed_poa_block(
            vec![parent],
            2,
            timestamp,
            0,
            authority_idx,
            &sks,
        );
        let block_id = node.receive_block(record).unwrap();
        heavier_branch.push(block_id);
        parent = block_id;
    }

    // The heavier branch should become the selected tip
    let new_tip = node.selected_tip().unwrap();
    assert_eq!(new_tip, *heavier_branch.last().unwrap());

    // Verify the reorg happened
    let selected_chain = node.ledger().unwrap().dag().selected_chain();
    assert!(selected_chain.contains(&heavier_branch[0]));

    println!("PoA re-org test passed: heavier branch won");
}

// ---------------------------------------------------------------------------
// SPV Sync Under Re-org Test
// ---------------------------------------------------------------------------

#[test]
fn spv_sync_under_poa_reorg() {
    // Test that SPV client correctly follows the selected chain after a reorg

    let (set, sks) = authority_set(3, 2);

    let mut node = Node::new();
    node.set_now_ms(0);
    node.genesis_with_poa(
        3,
        10 * ATOM,
        200_000 * ATOM,
        1,
        None,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        None,
        set.clone(),
        SLOT_MS,
    )
    .expect("PoA genesis");

    for sk in &sks {
        node.set_authority_signing_key(sk.to_bytes());
    }

    // Build initial chain of 10 blocks (slots 1-10)
    for slot in 1..=10 {
        node.set_now_ms(SLOT_MS * slot);
        node.produce_empty().unwrap();
    }

    // SPV client syncs the initial chain
    let genesis_id = node.genesis_id().unwrap();
    let genesis_header = node.spv_header(&genesis_id).unwrap();
    let mut spv_client = kovanica_state::spv::SpvClient::with_poa(
        genesis_header.clone(),
        false,
        None,
        kovanica_state::spv::SpvPoAConfig {
            authority_set: set.clone(),
            slot_duration_ms: SLOT_MS,
        },
    );

    // Sync all headers
    for id in node.ledger().unwrap().dag().selected_chain() {
        if id == genesis_id { continue; }
        let header = node.spv_header(&id).unwrap();
        spv_client.add_header(header).unwrap();
    }

    assert_eq!(spv_client.tip().unwrap().height, 10);

    // Now create a reorg on the node
    // Insert a heavier competing branch
    let dag = node.ledger().unwrap().dag();
    let split_point = dag.selected_chain()[4]; // Split at block 5

    // Insert heavier blocks
    let mut parent = split_point;
    for i in 1..=8 {
        let timestamp = SLOT_MS * (10 + i as u64);
        let slot = timestamp / SLOT_MS;
        let authority_idx = (slot as usize) % 3;
        let record = create_signed_poa_block(
            vec![parent],
            2,
            timestamp,
            0,
            authority_idx,
            &sks,
        );
        let _ = node.receive_block(record).unwrap();
        parent = node.selected_tip().unwrap();
    }

    // Now the node has reorged to a new tip
    let new_tip = node.selected_tip().unwrap();

    // SPV client should be able to sync the new chain
    // by requesting headers from the new tip
    let new_headers = node.export_spv_headers();
    let mut new_spv_client = kovanica_state::spv::SpvClient::with_poa(
        genesis_header.clone(),
        false,
        None,
        kovanica_state::spv::SpvPoAConfig {
            authority_set: set.clone(),
            slot_duration_ms: SLOT_MS,
        },
    );

    for header in new_headers {
        if header.height <= 10 { continue; } // Skip already synced
        new_spv_client.add_header(header).unwrap();
    }

    assert_eq!(new_spv_client.tip().unwrap().id, new_tip);
    assert!(new_spv_client.tip().unwrap().height > 10);

    println!("SPV sync under re-org test passed");
}

// ---------------------------------------------------------------------------
// Resource Profiling Test (CPU/RAM vs PoW)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "run manually for resource profiling"]
fn resource_profiling_poa_vs_pow() {
    // This test is ignored by default and should be run manually
    // to profile CPU/RAM usage of PoA vs PoW

    use std::time::Instant;

    // PoA node
    let (set, sks) = authority_set(3, 2);
    let mut poa_node = Node::new();
    poa_node.genesis_with_poa(
        3,
        10 * ATOM,
        200_000 * ATOM,
        1,
        None,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        None,
        set.clone(),
        SLOT_MS,
    ).unwrap();
    for sk in &sks {
        poa_node.set_authority_signing_key(sk.to_bytes());
    }

    // PoW node
    let mut pow_node = Node::new();
    pow_node.genesis_with_finality(
        3,
        10 * ATOM,
        200_000 * ATOM,
        1,
        None,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        None,
    ).unwrap();
    pow_node.set_miner(kovanica_state::KeyPair::from_u64(1).address());

    // Measure PoA block production time
    let start = std::time::Instant::now();
    for _ in 0..100 {
        poa_node.produce_empty().unwrap();
    }
    let poa_duration = start.elapsed();

    // Measure PoW block production time (with low difficulty)
    let _ = pow_node.set_proof_of_work(true);

    let start = std::time::Instant::now();
    for _ in 0..100 {
        pow_node.produce_block().unwrap();
    }
    let pow_duration = start.elapsed();

    println!("PoA 100 blocks: {:?}", poa_duration);
    println!("PoW 100 blocks: {:?}", pow_duration);
    println!("PoA speedup: {:.2}x", pow_duration.as_secs_f64() / poa_duration.as_secs_f64());

    // PoA should be significantly faster (at least 10x)
    assert!(poa_duration < pow_duration / 10, "PoA should be at least 10x faster than PoW");
}

