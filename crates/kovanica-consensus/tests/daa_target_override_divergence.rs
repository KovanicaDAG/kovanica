//! Deterministic reproduction tests for `KOVANICA_TEST_TARGET_BITS` and ancestry fallbacks.
//!
//! Investigates:
//! 1. Empty ancestry fallback (round 0 genesis bootstrap) under different override values.
//! 2. Unavailable ancestry fallback (missing parents / orphans) under different override values.
//! 3. Insufficient history fallback (< 2 ancestor timestamps) under different override values.
//! 4. Clamping divergence on identical multi-block DAG history.
//! 5. Proof of live consensus partition: when two nodes evaluate the same block with differing
//!    override environments, `header.target_bits != required_bits`, triggering immediate block
//!    rejection in `ingest_block` and `handle_submit_work`.

use kovanica_consensus::daa::{required_target_bits, DaaConfig};
use kovanica_consensus::mining::MiningWorker;
use kovanica_dag::DagStore;
use kovanica_params::{
    CHAIN_ID_HYBRID_TESTNET, DAA_WINDOW_BLOCKS, MAX_TARGET_BITS, MIN_TARGET_BITS, TARGET_BLOCK_MS,
};
use kovanica_storage::Storage;
use kovanica_types::block::{BlockHeader, BlockReference, StatementBlock};
use kovanica_types::hash::Hash;

const TEST_OVERRIDE_EASY: u32 = 0x207f_ffff; // Easy target used in tests / devnet
const PRODUCTION_DEFAULT: u32 = MAX_TARGET_BITS; // Canonical consensus ceiling (0x1e0ffff0)

fn make_mined_block(
    worker: &mut MiningWorker,
    round: u64,
    author: u16,
    parents: Vec<BlockReference>,
    timestamp: u64,
    target_bits: u32,
) -> StatementBlock {
    let header = BlockHeader::new(
        CHAIN_ID_HYBRID_TESTNET,
        round,
        author,
        parents,
        Hash::zero(),
        timestamp,
        target_bits,
        0,
    );
    let mined_header = worker
        .mine_header(header, 1_000)
        .expect("mining must succeed with easy target");
    assert!(mined_header.verify_pow().is_ok());
    mined_header.to_statement_block(vec![])
}

fn create_test_config(max_target_bits: u32) -> DaaConfig {
    DaaConfig {
        target_block_time_ms: TARGET_BLOCK_MS,
        window_size: DAA_WINDOW_BLOCKS,
        max_target_bits,
        min_target_bits: MIN_TARGET_BITS,
        max_adjustment_factor: 4,
    }
}

#[test]
fn test_empty_ancestry_fallback_diverges_by_override() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let config_easy = create_test_config(TEST_OVERRIDE_EASY);
    let config_prod = create_test_config(PRODUCTION_DEFAULT);

    // Empty parents (round 0 bootstrap)
    let empty_parents: Vec<BlockReference> = vec![];

    let target_easy =
        required_target_bits(&dag, &empty_parents, CHAIN_ID_HYBRID_TESTNET, &config_easy);
    let target_prod =
        required_target_bits(&dag, &empty_parents, CHAIN_ID_HYBRID_TESTNET, &config_prod);

    assert_eq!(target_easy, TEST_OVERRIDE_EASY);
    assert_eq!(target_prod, PRODUCTION_DEFAULT);
    assert_ne!(
        target_easy, target_prod,
        "Empty ancestry directly returns config.max_target_bits, causing divergence between test and production targets"
    );
}

#[test]
fn test_unavailable_ancestry_fallback_diverges_by_override() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let config_easy = create_test_config(TEST_OVERRIDE_EASY);
    let config_prod = create_test_config(PRODUCTION_DEFAULT);

    // Parent reference pointing to a non-existent block hash in DAG storage
    let missing_parent = BlockReference {
        digest: Hash::new([0xbb; 32]),
        round: 1,
        author: 0,
    };
    let parents = vec![missing_parent];

    let target_easy = required_target_bits(&dag, &parents, CHAIN_ID_HYBRID_TESTNET, &config_easy);
    let target_prod = required_target_bits(&dag, &parents, CHAIN_ID_HYBRID_TESTNET, &config_prod);

    assert_eq!(target_easy, TEST_OVERRIDE_EASY);
    assert_eq!(target_prod, PRODUCTION_DEFAULT);
    assert_ne!(
        target_easy, target_prod,
        "Unavailable ancestry falls back to config.max_target_bits when primary parent is missing from storage"
    );
}

#[test]
fn test_insufficient_timestamps_fallback_diverges_by_override() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let config_easy = create_test_config(TEST_OVERRIDE_EASY);
    let config_prod = create_test_config(PRODUCTION_DEFAULT);

    // Canonical round 0 genesis block (no PoW statements) as created by init_genesis
    let canonical_genesis = StatementBlock {
        author: 0,
        round: 0,
        parents: vec![],
        transactions: vec![],
        statements: vec![],
        signature: kovanica_types::Signature([0u8; 64]),
        digest: StatementBlock::compute_digest(0, 0, &[], &[]),
        merkle_root: Hash::zero(),
    };
    dag.put_block(&canonical_genesis).unwrap();

    // Round 1 candidate referencing canonical round 0 genesis
    let parents = vec![canonical_genesis.reference()];

    let target_easy = required_target_bits(&dag, &parents, CHAIN_ID_HYBRID_TESTNET, &config_easy);
    let target_prod = required_target_bits(&dag, &parents, CHAIN_ID_HYBRID_TESTNET, &config_prod);

    assert_eq!(target_easy, TEST_OVERRIDE_EASY);
    assert_eq!(target_prod, PRODUCTION_DEFAULT);
    assert_ne!(
        target_easy, target_prod,
        "Canonical genesis without PoW header falls back to config.max_target_bits, diverging across differing override configurations"
    );
}

#[test]
fn test_identical_dag_clamping_divergence_and_live_admission_rejection() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let mut worker = MiningWorker::new();
    let start_ts = 1_700_000_000_000u64;

    // Build a 5-block chain with 10-second block intervals (slower than target 4s -> target increases / difficulty eases)
    let mut prev_block = make_mined_block(&mut worker, 0, 0, vec![], start_ts, TEST_OVERRIDE_EASY);
    dag.put_block(&prev_block).unwrap();

    for round in 1..=5 {
        let ts = start_ts + round * 10_000;
        let block = make_mined_block(
            &mut worker,
            round,
            0,
            vec![prev_block.reference()],
            ts,
            TEST_OVERRIDE_EASY,
        );
        dag.put_block(&block).unwrap();
        prev_block = block;
    }

    let candidate_parents = vec![prev_block.reference()];

    // Node A runs with TEST_OVERRIDE_EASY (e.g. KOVANICA_TEST_TARGET_BITS="0x207fffff")
    let config_node_a = create_test_config(TEST_OVERRIDE_EASY);
    let required_bits_a = required_target_bits(
        &dag,
        &candidate_parents,
        CHAIN_ID_HYBRID_TESTNET,
        &config_node_a,
    );

    // Node B runs with PRODUCTION_DEFAULT (default MAX_TARGET_BITS, no env override)
    let config_node_b = create_test_config(PRODUCTION_DEFAULT);
    let required_bits_b = required_target_bits(
        &dag,
        &candidate_parents,
        CHAIN_ID_HYBRID_TESTNET,
        &config_node_b,
    );

    // 1. Target values diverge because Node B clamps to PRODUCTION_DEFAULT
    assert_ne!(
        required_bits_a, required_bits_b,
        "Node A (with test override) and Node B (production ceiling) compute divergent required targets"
    );

    // 2. Node A mines a candidate block using its required bits
    let block_from_node_a = make_mined_block(
        &mut worker,
        6,
        0,
        candidate_parents.clone(),
        start_ts + 60_000,
        required_bits_a,
    );
    let header_node_a = block_from_node_a
        .extract_pow_header(CHAIN_ID_HYBRID_TESTNET)
        .unwrap();

    // 3. Live admission check simulation (main.rs:2742):
    //    if header.target_bits != required_bits { reject block }
    assert_eq!(header_node_a.target_bits, required_bits_a);
    let admission_check_node_b = header_node_a.target_bits == required_bits_b;

    assert!(
        !admission_check_node_b,
        "CRITICAL CONSENSUS DIVERGENCE: Node B MUST REJECT Node A's block because declared target (0x{:08x}) \
         does not match Node B's required target (0x{:08x}). Ambient env var differences cause a hard network fork!",
        header_node_a.target_bits,
        required_bits_b
    );
}

#[test]
fn test_chain5_default_env_parsing_and_dag_comparison() {
    static ENV_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
    let _guard = ENV_LOCK.lock();

    let prev = std::env::var("KOVANICA_TEST_TARGET_BITS").ok();

    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    // Canonical genesis (round 0, no PoW statements)
    let canonical_genesis = StatementBlock {
        author: 0,
        round: 0,
        parents: vec![],
        transactions: vec![],
        statements: vec![],
        signature: kovanica_types::Signature([0u8; 64]),
        digest: StatementBlock::compute_digest(0, 0, &[], &[]),
        merkle_root: Hash::zero(),
    };
    dag.put_block(&canonical_genesis).unwrap();

    let genesis_parent = vec![canonical_genesis.reference()];
    let missing_parent = vec![BlockReference {
        digest: Hash::new([0xee; 32]),
        round: 1,
        author: 0,
    }];

    // 1. Unset KOVANICA_TEST_TARGET_BITS -> MAX_TARGET_BITS (0x1e0ffff0)
    std::env::remove_var("KOVANICA_TEST_TARGET_BITS");
    let cfg_unset = DaaConfig::chain5_default();
    assert_eq!(cfg_unset.max_target_bits, PRODUCTION_DEFAULT);
    assert_eq!(
        required_target_bits(&dag, &[], CHAIN_ID_HYBRID_TESTNET, &cfg_unset),
        PRODUCTION_DEFAULT
    );
    assert_eq!(
        required_target_bits(&dag, &genesis_parent, CHAIN_ID_HYBRID_TESTNET, &cfg_unset),
        PRODUCTION_DEFAULT
    );
    assert_eq!(
        required_target_bits(&dag, &missing_parent, CHAIN_ID_HYBRID_TESTNET, &cfg_unset),
        PRODUCTION_DEFAULT
    );

    // 2. Empty KOVANICA_TEST_TARGET_BITS="" -> fallback to MAX_TARGET_BITS (0x1e0ffff0)
    std::env::set_var("KOVANICA_TEST_TARGET_BITS", "");
    let cfg_empty = DaaConfig::chain5_default();
    assert_eq!(cfg_empty.max_target_bits, PRODUCTION_DEFAULT);
    assert_eq!(
        required_target_bits(&dag, &genesis_parent, CHAIN_ID_HYBRID_TESTNET, &cfg_empty),
        PRODUCTION_DEFAULT
    );

    // 3. KOVANICA_TEST_TARGET_BITS="0x207fffff" -> 0x207fffff
    std::env::set_var("KOVANICA_TEST_TARGET_BITS", "0x207fffff");
    let cfg_easy = DaaConfig::chain5_default();
    assert_eq!(cfg_easy.max_target_bits, TEST_OVERRIDE_EASY);
    assert_eq!(
        required_target_bits(&dag, &[], CHAIN_ID_HYBRID_TESTNET, &cfg_easy),
        TEST_OVERRIDE_EASY
    );
    assert_eq!(
        required_target_bits(&dag, &genesis_parent, CHAIN_ID_HYBRID_TESTNET, &cfg_easy),
        TEST_OVERRIDE_EASY
    );
    assert_eq!(
        required_target_bits(&dag, &missing_parent, CHAIN_ID_HYBRID_TESTNET, &cfg_easy),
        TEST_OVERRIDE_EASY
    );

    // 4. KOVANICA_TEST_TARGET_BITS="0x1f00ffff" -> 0x1f00ffff
    std::env::set_var("KOVANICA_TEST_TARGET_BITS", "0x1f00ffff");
    let cfg_mid = DaaConfig::chain5_default();
    assert_eq!(cfg_mid.max_target_bits, 0x1f00_ffff);
    assert_eq!(
        required_target_bits(&dag, &genesis_parent, CHAIN_ID_HYBRID_TESTNET, &cfg_mid),
        0x1f00_ffff
    );

    // Restore original env
    match prev {
        Some(v) => std::env::set_var("KOVANICA_TEST_TARGET_BITS", v),
        None => std::env::remove_var("KOVANICA_TEST_TARGET_BITS"),
    }
}
