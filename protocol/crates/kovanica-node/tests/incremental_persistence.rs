//! Integration tests for C1 incremental persistence.
//!
//! These exercise [`Node::persist_incremental`] and [`Node::load_log`]: a node
//! is rebuilt from its append-only replay log, derived state is recomputed from
//! the records, and the chain can continue afterwards.

use std::fs;

use kovanica_node::Node;
use kovanica_state::{HybridConfig, KeyPair};

fn temp_log(name: &str) -> String {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "kovanica-incr-{}-{}-{}",
        name,
        std::process::id(),
        rand::random::<u64>()
    ));
    path.set_extension("log");
    path.to_str().unwrap().to_string()
}

fn remove_log(path: &str) {
    let _ = fs::remove_file(path);
}

#[test]
fn log_roundtrip_recovers_blocks_and_continues() {
    let log_path = temp_log("roundtrip");
    let founder = KeyPair::from_u64(1);
    let recipient = KeyPair::from_u64(2);

    // Produce blocks on the original node and persist incrementally.
    let mut node = Node::new();
    node.genesis(3, 1_000, 1_000, 1, None).unwrap();
    node.set_miner(founder.address());
    node.produce_empty().unwrap();
    node.pool(1, 100, 2).unwrap();
    node.produce_block().unwrap().unwrap();
    let headers_before: Vec<_> = node.export_headers().iter().map(|h| h.id).collect();
    node.persist_incremental(&log_path).unwrap();

    // Rebuild from the log and verify the same non-genesis blocks are present
    // in the same order.
    let mut recovered = Node::load_log(&log_path).unwrap();
    let headers_after: Vec<_> = recovered.export_headers().iter().map(|h| h.id).collect();
    assert_eq!(
        headers_before, headers_after,
        "block ids must match across restart"
    );
    assert_eq!(
        recovered.balance(&recipient.address()).unwrap(),
        100u128,
        "recipient balance must be recovered from replay"
    );

    // The recovered chain must accept new blocks.
    recovered.set_miner(founder.address());
    recovered.produce_empty().unwrap();
    assert!(
        recovered.block_count().unwrap() > node.block_count().unwrap(),
        "recovered chain must continue"
    );

    remove_log(&log_path);
}

#[test]
fn loaded_node_reapplies_finality_policy() {
    // A node loaded from a replay log starts with finality disabled (the log
    // does not persist the policy). Re-applying the depth — what
    // `load_or_genesis` does from the network profile — must prune the
    // now-final blocks without corrupting the current state, and the chain
    // must keep building on the (non-final) tip.
    let log_path = temp_log("finality");
    let founder = KeyPair::from_u64(1);

    let mut node = Node::new();
    node.genesis(3, 1_000, 1_000, 1, None).unwrap();
    node.set_miner(founder.address());
    for _ in 0..10 {
        node.produce_empty().unwrap();
    }
    node.persist_incremental(&log_path).unwrap();

    let mut recovered = Node::load_log(&log_path).unwrap();
    assert_eq!(
        recovered.finality_depth(),
        u64::MAX,
        "log load starts with finality disabled"
    );

    let balance_before = recovered.balance(&founder.address()).unwrap();
    recovered.set_finality_depth(3).unwrap();
    assert_eq!(recovered.finality_depth(), 3);
    assert_eq!(
        recovered.balance(&founder.address()).unwrap(),
        balance_before,
        "enabling finality must not corrupt the current state"
    );

    // The recovered chain must still accept new blocks on the (non-final) tip.
    recovered.set_miner(founder.address());
    recovered.produce_empty().unwrap();
    assert!(
        recovered.block_count().unwrap() > node.block_count().unwrap(),
        "recovered chain must continue after enabling finality"
    );

    remove_log(&log_path);
}

#[test]
fn loaded_node_reapplies_block_pruning() {
    // A node loaded from a replay log starts with block pruning disabled (the
    // log does not persist the policy). Re-applying the depth — what
    // `load_or_genesis` does from the network profile — must evict the
    // now-final blocks (and their reachability-oracle entries) without
    // corrupting the current state, and the chain must keep building on the
    // (non-final) tip (RFC-008).
    let log_path = temp_log("blockprune");
    let founder = KeyPair::from_u64(1);

    let mut node = Node::new();
    node.genesis(3, 1_000, 1_000, 1, None).unwrap();
    node.set_miner(founder.address());
    for _ in 0..10 {
        node.produce_empty().unwrap();
    }
    node.persist_incremental(&log_path).unwrap();
    let blocks_before = node.block_count().unwrap();

    let mut recovered = Node::load_log(&log_path).unwrap();
    assert_eq!(
        recovered.block_pruning_depth(),
        u64::MAX,
        "log load starts with block pruning disabled"
    );

    let balance_before = recovered.balance(&founder.address()).unwrap();
    // Block pruning is only safe once finality is on (RFC-008 invariant: the
    // effective depth is clamped to `>= finality_depth`), so enable finality
    // first — the order `restore_miner_and_policy` uses.
    recovered.set_finality_depth(3).unwrap();
    recovered.set_block_pruning_depth(3).unwrap();
    assert_eq!(recovered.block_pruning_depth(), 3);
    assert_eq!(
        recovered.balance(&founder.address()).unwrap(),
        balance_before,
        "enabling block pruning must not corrupt the current state"
    );
    assert!(
        recovered.block_count().unwrap() < blocks_before,
        "final blocks evicted from the loaded DAG"
    );
    let after_prune = recovered.block_count().unwrap();
    let height_after_prune = recovered.chain_height().unwrap();

    // The recovered chain must still accept new blocks on the (non-final) tip.
    // `block_count` is steady-state under pruning (each new block advances the
    // tip and evicts one more old block), so assert on the chain height.
    recovered.set_miner(founder.address());
    recovered.produce_empty().unwrap();
    assert!(
        recovered.chain_height().unwrap() > height_after_prune,
        "recovered chain must continue after enabling block pruning"
    );
    assert!(
        recovered.block_count().unwrap() >= after_prune,
        "the DAG stays bounded but non-empty"
    );

    remove_log(&log_path);
}

#[test]
fn hybrid_log_preserves_staked_block_id() {
    let log_path = temp_log("hybrid");
    let cfg = HybridConfig {
        rate_num: 1,
        rate_den: 1,
        stake_nominal_work: 1,
        use_epoch_beacon: true,
        retarget: None,
    };
    let founder = KeyPair::from_u64(1);

    // Bond the founder's coin so the validator can win a staked-VRF block.
    let mut node = Node::new();
    node.genesis(3, 1_000, 1_000, 1, None).unwrap();
    node.enable_hybrid(cfg.clone()).unwrap();
    node.set_validator_seed([7u8; 32]);

    let (coin, _) = node
        .utxos_of(&founder.address())
        .unwrap()
        .first()
        .copied()
        .unwrap();
    let pk = *node.validator_public_key().unwrap().as_bytes();
    let bond = kovanica_state::Transaction::signed(
        &[(coin, &founder)],
        vec![kovanica_state::TxOutput::native(1_000, founder.address())],
        kovanica_state::bond_tag(kovanica_state::NATIVE_ASSET_ID, &pk),
    );
    node.submit_tx(bond).unwrap();
    node.produce_block().unwrap().unwrap();

    // Produce the staked-VRF empty block.
    let staked_id = node.produce_empty().unwrap();
    node.persist_incremental(&log_path).unwrap();

    // Replay under the same hybrid policy: the staked block must keep its id.
    let recovered = Node::load_log_with_hybrid(&log_path, cfg).unwrap();
    let header_ids: Vec<_> = recovered.export_headers().iter().map(|h| h.id).collect();
    assert!(
        header_ids.contains(&staked_id),
        "staked block id must be preserved across hybrid replay"
    );

    remove_log(&log_path);
}
