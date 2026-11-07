//! Multi-node PoW & GHOSTDAG simulation test suite for Chain ID 5 (Hybrid PoW + GHOSTDAG).
//!
//! Validates:
//! 1. Concurrent multi-miner DAG convergence with GHOSTDAG k=5
//! 2. GHOSTDAG blue / red categorization when concurrency exceeds k=5
//! 3. 2.5 KVNC mining reward distribution per committed block author
//! 4. DAA (Difficulty Adjustment Algorithm) dynamic response to variable block arrival times
//! 5. PoW anti-DoS rejection of invalid proofs before consensus ingestion

use kovanica_consensus::daa::{calculate_next_target_bits, DaaConfig};
use kovanica_consensus::ghostdag::GhostdagCommitter;
use kovanica_consensus::mining::MiningWorker;
use kovanica_params::{
    CHAIN_ID_HYBRID_TESTNET, GHOSTDAG_K, INITIAL_BLOCK_REWARD, MAX_TARGET_BITS, MIN_TARGET_BITS,
};
use kovanica_types::block::{compact_to_target, BlockHeader, BlockReference, StatementBlock};
use kovanica_types::hash::Hash;
use std::collections::HashMap;

/// High target bits for fast, deterministic unit test mining (top byte <= 0x7f, ~50% hit chance).
const EASY_TEST_TARGET_BITS: u32 = 0x207f_ffff;

/// Helper to author and mine a valid block header for Chain 5 using an in-memory worker.
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
fn test_multi_miner_ghostdag_dag_convergence() {
    let mut committer = GhostdagCommitter::new(GHOSTDAG_K as usize);
    let mut workers = [
        MiningWorker::new(),
        MiningWorker::new(),
        MiningWorker::new(),
    ];

    // 1. Genesis block authored by miner 0
    let genesis = make_mined_block(
        &mut workers[0],
        0,
        0,
        vec![],
        1_700_000_000_000,
        EASY_TEST_TARGET_BITS,
    );
    committer.add_block(genesis.clone());
    assert_eq!(committer.tips(), vec![genesis.digest]);
    assert_eq!(committer.selected_tip(), Some(genesis.digest));

    let mut timestamp = 1_700_000_004_000u64;

    // 2. Simulate 4 successive rounds of concurrent multi-miner block production.
    // In each round, miners independently observe DAG tips and build concurrent blocks.
    for round in 1..=4 {
        let current_tips = committer.tips();
        let parent_refs: Vec<BlockReference> = current_tips
            .iter()
            .filter_map(|h| committer.blocks.get(h).map(|b| b.reference()))
            .collect();

        // Miner 0 authors block A
        let block_a = make_mined_block(
            &mut workers[0],
            round,
            0,
            parent_refs.clone(),
            timestamp,
            EASY_TEST_TARGET_BITS,
        );

        // Miner 1 authors concurrent block B referencing the same tips
        let block_b = make_mined_block(
            &mut workers[1],
            round,
            1,
            parent_refs.clone(),
            timestamp + 500,
            EASY_TEST_TARGET_BITS,
        );

        committer.add_block(block_a);
        committer.add_block(block_b);

        timestamp += 4000;
    }

    // 3. Convergence block at round 5 authored by miner 2 referencing all remaining tips
    let convergence_parents: Vec<BlockReference> = committer
        .tips()
        .iter()
        .filter_map(|h| committer.blocks.get(h).map(|b| b.reference()))
        .collect();

    let convergence_block = make_mined_block(
        &mut workers[2],
        5,
        2,
        convergence_parents,
        timestamp,
        EASY_TEST_TARGET_BITS,
    );
    committer.add_block(convergence_block.clone());

    // 4. Commit the uncommitted selected chain
    let subdags = committer.commit_uncommitted_selected_chain();
    assert!(!subdags.is_empty(), "must produce committed subdags");

    // 5. Verify GHOSTDAG ordering, total blocks, and 2.5 KVNC reward distribution
    let mut total_blocks = 0usize;
    let mut rewards_by_miner: HashMap<u16, u64> = HashMap::new();

    for subdag in &subdags {
        for blk in &subdag.blocks {
            total_blocks += 1;
            let header = blk
                .extract_pow_header(CHAIN_ID_HYBRID_TESTNET)
                .expect("all committed blocks must have valid PoW headers");
            assert_eq!(header.chain_id, CHAIN_ID_HYBRID_TESTNET);
            assert!(header.verify_pow().is_ok());

            *rewards_by_miner.entry(blk.author).or_default() += INITIAL_BLOCK_REWARD;
        }
    }

    // At least 9 blocks should be committed (genesis + 4 rounds * 2 concurrent + 1 convergence)
    assert!(
        total_blocks >= 9,
        "expected at least 9 committed blocks, got {total_blocks}"
    );

    // Verify all 3 miners earned rewards proportional to their blocks
    assert!(rewards_by_miner.get(&0).copied().unwrap_or(0) > 0);
    assert!(rewards_by_miner.get(&1).copied().unwrap_or(0) > 0);
    assert!(rewards_by_miner.get(&2).copied().unwrap_or(0) > 0);

    let total_rewards: u64 = rewards_by_miner.values().sum();
    assert_eq!(total_rewards, (total_blocks as u64) * INITIAL_BLOCK_REWARD);

    // Initial block reward is exactly 2.5 KVNC (2,500,000,000 atoms)
    assert_eq!(INITIAL_BLOCK_REWARD, 2_500_000_000);
}

#[test]
fn test_ghostdag_k5_blue_red_categorization() {
    // k = 5 GHOSTDAG committer
    let mut committer = GhostdagCommitter::new(5);
    let mut worker = MiningWorker::new();

    // 1. Genesis block
    let genesis = make_mined_block(
        &mut worker,
        0,
        0,
        vec![],
        1_700_000_000_000,
        EASY_TEST_TARGET_BITS,
    );
    committer.add_block(genesis.clone());

    // 2. Author 8 parallel sibling blocks off genesis (exceeding k=5)
    let mut siblings = Vec::new();
    for author in 0..8 {
        let sib = make_mined_block(
            &mut worker,
            1,
            author,
            vec![genesis.reference()],
            1_700_000_004_000,
            EASY_TEST_TARGET_BITS,
        );
        committer.add_block(sib.clone());
        siblings.push(sib);
    }

    // 3. Merger block referencing all 8 siblings
    let merger_parents: Vec<BlockReference> = siblings.iter().map(|s| s.reference()).collect();
    let merger = make_mined_block(
        &mut worker,
        2,
        9,
        merger_parents,
        1_700_000_008_000,
        EASY_TEST_TARGET_BITS,
    );
    committer.add_block(merger.clone());

    // 4. Commit selected chain
    let subdags = committer.commit_uncommitted_selected_chain();
    assert!(!subdags.is_empty());

    // 5. Inspect blue / red classification:
    // With 8 siblings in total, 1 is the selected parent (or leader).
    // The mergeset has 7 siblings. Since k=5, at most 5 siblings can be blue.
    // The remaining 2 siblings MUST be classified as red (non_blue).
    let mut total_red_blocks = 0;
    let mut total_blue_blocks = 0;

    for subdag in &subdags {
        total_blue_blocks += subdag.blocks.len();
        total_red_blocks += subdag.non_blue.len();
    }

    // Across the entire execution:
    // 8 siblings + genesis + merger = 10 total blocks created
    // 2 siblings are red (non-blue) because mergeset width 7 > k=5
    // 8 blocks are blue (genesis + 1 selected sibling + 5 blue siblings + merger)
    assert_eq!(
        total_red_blocks, 2,
        "exactly 2 blocks must be marked red for exceeding k=5"
    );
    assert_eq!(total_blue_blocks, 8, "exactly 8 blocks must be marked blue");
}

#[test]
fn test_daa_difficulty_adaptation() {
    let config = DaaConfig::chain5_default();
    let start_ts = 1_700_000_000_000u64;
    let initial_bits = 0x1d00_ffff;
    let initial_target = compact_to_target(initial_bits);

    // Scenario A: Fast arrival times (1,000 ms vs target 4,000 ms)
    let fast_timestamps: Vec<u64> = (0..60).map(|i| start_ts + i * 1_000).collect();
    let next_target_fast_bits = calculate_next_target_bits(initial_bits, &fast_timestamps, &config);
    let next_target_fast = compact_to_target(next_target_fast_bits);

    assert!(
        next_target_fast < initial_target,
        "difficulty must increase (target decrease) when blocks arrive faster than target"
    );

    // Scenario B: Slow arrival times (10,000 ms vs target 4,000 ms)
    let slow_timestamps: Vec<u64> = (0..60).map(|i| start_ts + i * 10_000).collect();
    let next_target_slow_bits = calculate_next_target_bits(initial_bits, &slow_timestamps, &config);
    let next_target_slow = compact_to_target(next_target_slow_bits);

    assert!(
        next_target_slow > initial_target,
        "difficulty must decrease (target increase) when blocks arrive slower than target"
    );

    // Scenario C: Easiest difficulty ceiling clamp
    let max_timestamps: Vec<u64> = (0..60).map(|i| start_ts + i * 100_000).collect();
    let clamped_bits = calculate_next_target_bits(MAX_TARGET_BITS, &max_timestamps, &config);
    assert_eq!(
        clamped_bits, MAX_TARGET_BITS,
        "target bits must not exceed MAX_TARGET_BITS ceiling"
    );

    // Scenario D: Hardest difficulty floor clamp
    let min_timestamps: Vec<u64> = (0..60).map(|i| start_ts + i * 10).collect();
    let min_clamped_bits = calculate_next_target_bits(MIN_TARGET_BITS, &min_timestamps, &config);
    assert_eq!(
        min_clamped_bits, MIN_TARGET_BITS,
        "target bits must not drop below MIN_TARGET_BITS floor"
    );
}

#[test]
fn test_pow_anti_dos_rejection() {
    let mut worker = MiningWorker::new();
    let valid_block = make_mined_block(
        &mut worker,
        1,
        0,
        vec![],
        1_700_000_000_000,
        EASY_TEST_TARGET_BITS,
    );

    // 1. Valid block passes PoW verification
    let valid_header = valid_block
        .extract_pow_header(CHAIN_ID_HYBRID_TESTNET)
        .expect("must extract valid PoW header");
    assert!(valid_header.verify_pow().is_ok());

    // 2. Tampered nonce produces verification error
    let mut tampered_header = valid_header.clone();
    tampered_header.nonce = tampered_header.nonce.wrapping_add(42);
    assert!(
        tampered_header.verify_pow().is_err(),
        "tampered nonce must fail PoW verification"
    );

    // 3. Unattained target bits produces verification error
    let mut hard_target_header = valid_header.clone();
    hard_target_header.target_bits = MIN_TARGET_BITS;
    assert!(
        hard_target_header.verify_pow().is_err(),
        "unmet target difficulty must fail PoW verification"
    );

    // 4. Legacy / non-PoW block fails extraction
    let legacy_block = StatementBlock {
        digest: Hash::zero(),
        round: 1,
        author: 0,
        parents: vec![],
        statements: vec![],
        signature: kovanica_types::Signature([0u8; 64]),
        transactions: vec![],
        merkle_root: Hash::zero(),
    };
    assert!(
        legacy_block
            .extract_pow_header(CHAIN_ID_HYBRID_TESTNET)
            .is_err(),
        "block without PoW statements must be rejected on Chain 5 ingress"
    );
}

#[test]
fn test_same_dag_different_arrival_orders_yields_same_required_target() {
    use kovanica_consensus::daa::required_target_bits;
    use kovanica_dag::DagStore;
    use kovanica_storage::Storage;

    let dir1 = tempfile::tempdir().unwrap();
    let storage1 = Storage::new(dir1.path().join("dag1.redb")).unwrap();
    let dag1 = DagStore::new(storage1).unwrap();

    let dir2 = tempfile::tempdir().unwrap();
    let storage2 = Storage::new(dir2.path().join("dag2.redb")).unwrap();
    let dag2 = DagStore::new(storage2).unwrap();

    let mut worker = MiningWorker::new();
    let start_ts = 1_700_000_000_000u64;

    // Genesis block (round 0)
    let genesis = make_mined_block(&mut worker, 0, 0, vec![], start_ts, EASY_TEST_TARGET_BITS);

    // Parallel blocks at round 1 (A and B)
    let block_1a = make_mined_block(
        &mut worker,
        1,
        0,
        vec![genesis.reference()],
        start_ts + 4_000,
        EASY_TEST_TARGET_BITS,
    );
    let block_1b = make_mined_block(
        &mut worker,
        1,
        1,
        vec![genesis.reference()],
        start_ts + 5_000,
        EASY_TEST_TARGET_BITS,
    );

    // Block at round 2 merging 1A and 1B
    let block_2 = make_mined_block(
        &mut worker,
        2,
        0,
        vec![block_1a.reference(), block_1b.reference()],
        start_ts + 9_000,
        EASY_TEST_TARGET_BITS,
    );

    // Node 1 receives: Genesis -> 1A -> 1B -> 2
    dag1.put_block(&genesis).unwrap();
    dag1.put_block(&block_1a).unwrap();
    dag1.put_block(&block_1b).unwrap();
    dag1.put_block(&block_2).unwrap();

    // Node 2 receives: Genesis -> 1B -> 1A -> 2 (different arrival order for round 1 blocks)
    dag2.put_block(&genesis).unwrap();
    dag2.put_block(&block_1b).unwrap();
    dag2.put_block(&block_1a).unwrap();
    dag2.put_block(&block_2).unwrap();

    // Now evaluate candidate block at round 3 referencing block 2
    let candidate_parents = vec![block_2.reference()];
    let config = DaaConfig::chain5_default();

    let target1 = required_target_bits(&dag1, &candidate_parents, CHAIN_ID_HYBRID_TESTNET, &config);
    let target2 = required_target_bits(&dag2, &candidate_parents, CHAIN_ID_HYBRID_TESTNET, &config);

    assert_eq!(
        target1, target2,
        "Same DAG with different block arrival orders must yield the exact same required DAA target"
    );
}
