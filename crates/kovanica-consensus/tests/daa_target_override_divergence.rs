//! Deterministic tests verifying that DAA required-target derivation is strictly network-bound,
//! identical across differing environment variables, and unreadable ancestry is rejected.
//!
//! Validates:
//! 1. Empty ancestry (round 0 genesis bootstrap) yields canonical network-bound target (INITIAL_TARGET_BITS_HYBRID).
//! 2. Unreadable / missing ancestry returns DaaError rather than silently falling back to a permissive target.
//! 3. Insufficient history (< 2 ancestor timestamps) yields canonical network-bound target.
//! 4. Identical multi-block DAG history yields identical required target under DaaConfig::hybrid_default()
//!    regardless of any ambient process environment variables.
//! 5. Explicit test configuration in test harnesses can specify custom max_target_bits.

use kovanica_consensus::daa::{
    required_target_bits, try_required_target_bits, DaaConfig, DaaError,
};
use kovanica_consensus::mining::MiningWorker;
use kovanica_dag::DagStore;
use kovanica_params::{CHAIN_ID_HYBRID_TESTNET, MAX_TARGET_BITS};
use kovanica_storage::Storage;
use kovanica_types::block::{BlockHeader, BlockReference, StatementBlock};
use kovanica_types::hash::Hash;

const TEST_OVERRIDE_EASY: u32 = 0x207f_ffff; // Easy target used in tests / devnet
const PRODUCTION_DEFAULT: u32 = MAX_TARGET_BITS; // Canonical consensus ceiling (0x1f00ffff)

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

#[test]
fn test_empty_ancestry_bootstrap_returns_canonical_network_bound_target() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let config_default = DaaConfig::hybrid_default();
    let empty_parents: Vec<BlockReference> = vec![];

    let target = required_target_bits(
        &dag,
        &empty_parents,
        CHAIN_ID_HYBRID_TESTNET,
        &config_default,
    )
    .unwrap();
    assert_eq!(
        target, PRODUCTION_DEFAULT,
        "Empty ancestry must return canonical INITIAL_TARGET_BITS_HYBRID"
    );

    let try_target = try_required_target_bits(
        &dag,
        &empty_parents,
        CHAIN_ID_HYBRID_TESTNET,
        &config_default,
    )
    .unwrap();
    assert_eq!(try_target, PRODUCTION_DEFAULT);
}

#[test]
fn test_unreadable_missing_ancestry_returns_error_instead_of_permissive_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let config = DaaConfig::hybrid_default();

    // Parent reference pointing to a non-existent block hash in DAG storage
    let missing_parent = BlockReference {
        digest: Hash::new([0xbb; 32]),
        round: 1,
        author: 0,
    };
    let parents = vec![missing_parent];

    let result = try_required_target_bits(&dag, &parents, CHAIN_ID_HYBRID_TESTNET, &config);
    assert!(
        matches!(result, Err(DaaError::ParentBlockUnreadable(h, _)) if h == Hash::new([0xbb; 32])),
        "Unreadable missing ancestry must return Err(DaaError::ParentBlockUnreadable), got {result:?}"
    );
}

#[test]
fn test_invalid_parent_header_at_round_ge_1_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let config = DaaConfig::hybrid_default();

    // Non-genesis block at round 1 without PoW statements (corrupted / legacy block)
    let non_pow_parent = StatementBlock {
        author: 0,
        round: 1,
        parents: vec![],
        transactions: vec![],
        statements: vec![], // Missing Statement::PowHeader
        signature: kovanica_types::Signature([0u8; 64]),
        digest: StatementBlock::compute_digest(0, 1, &[], &[]),
        merkle_root: Hash::zero(),
    };
    dag.put_block(&non_pow_parent).unwrap();

    let parents = vec![non_pow_parent.reference()];
    let result = try_required_target_bits(&dag, &parents, CHAIN_ID_HYBRID_TESTNET, &config);
    assert!(
        matches!(result, Err(DaaError::InvalidParentHeader(h, r, _)) if h == non_pow_parent.digest && r == 1),
        "Non-genesis parent lacking PoW header must fail closed with Err(DaaError::InvalidParentHeader), got {result:?}"
    );
}

#[test]
fn test_insufficient_timestamps_fallback_returns_canonical_target() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let config = DaaConfig::hybrid_default();

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

    let target = required_target_bits(&dag, &parents, CHAIN_ID_HYBRID_TESTNET, &config).unwrap();
    assert_eq!(
        target, PRODUCTION_DEFAULT,
        "Canonical genesis without PoW header must yield canonical INITIAL_TARGET_BITS_HYBRID"
    );

    let try_target =
        try_required_target_bits(&dag, &parents, CHAIN_ID_HYBRID_TESTNET, &config).unwrap();
    assert_eq!(try_target, PRODUCTION_DEFAULT);
}

#[test]
fn test_identical_dag_produces_identical_target_regardless_of_ambient_env() {
    static ENV_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
    let _guard = ENV_LOCK.lock();

    let prev = std::env::var("KOVANICA_TEST_TARGET_BITS").ok();

    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let mut worker = MiningWorker::new();
    let start_ts = 1_700_000_000_000u64;

    // Build a 5-block chain
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

    // 1. Evaluate with env unset
    std::env::remove_var("KOVANICA_TEST_TARGET_BITS");
    let config_unset = DaaConfig::hybrid_default();
    let target_unset = required_target_bits(
        &dag,
        &candidate_parents,
        CHAIN_ID_HYBRID_TESTNET,
        &config_unset,
    );

    // 2. Evaluate with env set to easy override "0x207fffff"
    std::env::set_var("KOVANICA_TEST_TARGET_BITS", "0x207fffff");
    let config_with_env = DaaConfig::hybrid_default();
    let target_with_env = required_target_bits(
        &dag,
        &candidate_parents,
        CHAIN_ID_HYBRID_TESTNET,
        &config_with_env,
    );

    // 3. Evaluate with env set to another override "0x12345678"
    std::env::set_var("KOVANICA_TEST_TARGET_BITS", "0x12345678");
    let config_another = DaaConfig::hybrid_default();
    let target_another = required_target_bits(
        &dag,
        &candidate_parents,
        CHAIN_ID_HYBRID_TESTNET,
        &config_another,
    );

    assert_eq!(
        target_unset, target_with_env,
        "DaaConfig::hybrid_default() must produce identical target regardless of KOVANICA_TEST_TARGET_BITS"
    );
    assert_eq!(
        target_unset, target_another,
        "DaaConfig::hybrid_default() must be completely immune to ambient process environment variables"
    );

    // Restore env
    match prev {
        Some(v) => std::env::set_var("KOVANICA_TEST_TARGET_BITS", v),
        None => std::env::remove_var("KOVANICA_TEST_TARGET_BITS"),
    }
}

#[test]
fn test_explicit_test_overrides_still_work_where_explicitly_constructed() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let config_easy = DaaConfig {
        max_target_bits: TEST_OVERRIDE_EASY,
        ..DaaConfig::hybrid_default()
    };
    assert_eq!(config_easy.max_target_bits, TEST_OVERRIDE_EASY);

    let empty_parents: Vec<BlockReference> = vec![];
    let target_easy =
        required_target_bits(&dag, &empty_parents, CHAIN_ID_HYBRID_TESTNET, &config_easy).unwrap();
    assert_eq!(
        target_easy, TEST_OVERRIDE_EASY,
        "Explicit test override must be respected when passed via explicit test config"
    );
}

#[test]
fn test_unreadable_deeper_ancestor_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let mut worker = MiningWorker::new();
    let config = DaaConfig::hybrid_default();

    // Canonical genesis at round 0
    let genesis = StatementBlock {
        author: 0,
        round: 0,
        parents: vec![],
        transactions: vec![],
        statements: vec![],
        signature: kovanica_types::Signature([0u8; 64]),
        digest: StatementBlock::compute_digest(0, 0, &[], &[]),
        merkle_root: Hash::zero(),
    };
    dag.put_block(&genesis).unwrap();

    // Block 1 referencing genesis
    let b1 = make_mined_block(
        &mut worker,
        1,
        0,
        vec![genesis.reference()],
        1_700_000_000_000,
        TEST_OVERRIDE_EASY,
    );
    // Deliberately DO NOT persist b1 in dag_store!
    let missing_grandparent_digest = b1.digest;

    // Block 2 referencing b1 (b2 IS in dag_store)
    let b2 = make_mined_block(
        &mut worker,
        2,
        0,
        vec![b1.reference()],
        1_700_000_010_000,
        TEST_OVERRIDE_EASY,
    );
    dag.put_block(&b2).unwrap();

    // Block 3 candidate referencing b2
    let candidate_parents = vec![b2.reference()];

    let res = required_target_bits(&dag, &candidate_parents, CHAIN_ID_HYBRID_TESTNET, &config);
    assert!(
        matches!(res, Err(DaaError::ParentBlockUnreadable(h, _)) if h == missing_grandparent_digest),
        "Unreadable deeper ancestor must fail closed with ParentBlockUnreadable, got: {res:?}"
    );
}

#[test]
fn test_corrupted_deeper_ancestor_header_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let mut worker = MiningWorker::new();
    let config = DaaConfig::hybrid_default();

    let genesis = StatementBlock {
        author: 0,
        round: 0,
        parents: vec![],
        transactions: vec![],
        statements: vec![],
        signature: kovanica_types::Signature([0u8; 64]),
        digest: StatementBlock::compute_digest(0, 0, &[], &[]),
        merkle_root: Hash::zero(),
    };
    dag.put_block(&genesis).unwrap();

    // Block 1 with corrupted/missing PoW header at round 1
    let corrupted_b1 = StatementBlock {
        author: 0,
        round: 1,
        parents: vec![genesis.reference()],
        transactions: vec![],
        statements: vec![], // Missing PoW statement!
        signature: kovanica_types::Signature([0u8; 64]),
        digest: StatementBlock::compute_digest(0, 1, &[], &[]),
        merkle_root: Hash::zero(),
    };
    let corrupted_digest = corrupted_b1.digest;
    dag.put_block(&corrupted_b1).unwrap();

    // Block 2 referencing corrupted_b1
    let b2 = make_mined_block(
        &mut worker,
        2,
        0,
        vec![corrupted_b1.reference()],
        1_700_000_010_000,
        TEST_OVERRIDE_EASY,
    );
    dag.put_block(&b2).unwrap();

    // Block 3 candidate referencing b2
    let candidate_parents = vec![b2.reference()];

    let res = required_target_bits(&dag, &candidate_parents, CHAIN_ID_HYBRID_TESTNET, &config);
    assert!(
        matches!(res, Err(DaaError::InvalidParentHeader(h, 1, _)) if h == corrupted_digest),
        "Corrupted deeper ancestor header must fail closed with InvalidParentHeader, got: {res:?}"
    );
}

#[test]
fn test_non_canonical_round_zero_parent_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let config = DaaConfig::hybrid_default();

    // Bogus non-canonical round 0 block (e.g. author 99 or with statements/txs)
    let bogus_round_zero = StatementBlock {
        author: 99,
        round: 0,
        parents: vec![],
        transactions: vec![],
        statements: vec![],
        signature: kovanica_types::Signature([0u8; 64]),
        digest: Hash::new([0x99; 32]),
        merkle_root: Hash::zero(),
    };
    dag.put_block(&bogus_round_zero).unwrap();

    let parents = vec![bogus_round_zero.reference()];
    let res = required_target_bits(&dag, &parents, CHAIN_ID_HYBRID_TESTNET, &config);
    assert!(
        matches!(res, Err(DaaError::InvalidParentHeader(h, 0, _)) if h == Hash::new([0x99; 32])),
        "Non-canonical round 0 parent without PoW header must fail closed, got: {res:?}"
    );
}

#[test]
fn test_non_canonical_round_zero_deeper_ancestor_fails_closed() {
    let dir = tempfile::tempdir().unwrap();
    let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
    let dag = DagStore::new(storage).unwrap();

    let mut worker = MiningWorker::new();
    let config = DaaConfig::hybrid_default();

    // Bogus non-canonical round 0 block
    let bogus_round_zero = StatementBlock {
        author: 99,
        round: 0,
        parents: vec![],
        transactions: vec![],
        statements: vec![],
        signature: kovanica_types::Signature([0u8; 64]),
        digest: Hash::new([0x99; 32]),
        merkle_root: Hash::zero(),
    };
    dag.put_block(&bogus_round_zero).unwrap();

    // Block 1 referencing bogus round 0
    let b1 = make_mined_block(
        &mut worker,
        1,
        0,
        vec![bogus_round_zero.reference()],
        1_700_000_000_000,
        TEST_OVERRIDE_EASY,
    );
    dag.put_block(&b1).unwrap();

    // Block 2 referencing b1
    let b2 = make_mined_block(
        &mut worker,
        2,
        0,
        vec![b1.reference()],
        1_700_000_010_000,
        TEST_OVERRIDE_EASY,
    );
    dag.put_block(&b2).unwrap();

    let candidate_parents = vec![b2.reference()];
    let res = required_target_bits(&dag, &candidate_parents, CHAIN_ID_HYBRID_TESTNET, &config);
    assert!(
        matches!(res, Err(DaaError::InvalidParentHeader(h, 0, _)) if h == Hash::new([0x99; 32])),
        "Deeper non-canonical round 0 ancestor must fail closed, got: {res:?}"
    );
}
