//! Integration tests for the mempool and block production.
//!
//! These drive the `Node` API directly rather than the `rpc` string surface.
//! The RPC `genesis` command builds a non-PoA genesis, and since PoA is the
//! only admission regime (RFC-POA §0) such a node can never produce — the
//! `rpc` surface has no PoA genesis / authority-key command yet. See
//! `docs/RFC-POA-Migration.md` §0.9 (blockers B1/B2).

use ed25519_dalek::SigningKey;
use kovanica_dag::{AuthorityPublicKey, AuthoritySet};
use kovanica_node::Node;
use kovanica_state::KeyPair;

/// Slot duration used throughout (RFC-POA default).
const SLOT_MS: u64 = 3000;
/// Authority count (the `AuthoritySet` minimum is 3).
const AUTHORITIES: u64 = 3;

/// A PoA node holding every authority signing key, so it produces in any
/// slot. Rewards are credited to the *first* loaded authority key
/// (`authority_public_key()`), which is seed 1 — so the subsidy assertions
/// below read as "actor 1 earned the subsidy" while the actual signer may be
/// any of the three.
fn poa_node(subsidy: u64, premine: u64) -> Node {
    let keys: Vec<AuthorityPublicKey> = (1..=AUTHORITIES)
        .map(|i| SigningKey::from_bytes(&KeyPair::from_u64(i).seed()).verifying_key())
        .collect();
    let set = AuthoritySet::new(keys, 2).expect("valid authority set");
    let mut node = Node::new();
    node.genesis_with_poa(
        3,
        subsidy,
        premine,
        1,
        None,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        None,
        set,
        SLOT_MS,
    )
    .expect("genesis");
    for i in 1..=AUTHORITIES {
        node.set_authority_signing_key(KeyPair::from_u64(i).seed());
    }
    node
}

fn bal(node: &mut Node, seed: u64) -> u128 {
    node.balance(&Node::address(seed)).unwrap()
}

#[test]
fn a_pooled_transfer_is_packed_into_a_block() {
    let mut node = poa_node(1000, 1000);
    node.pool(1, 400, 2).unwrap();
    assert_eq!(node.pending_count(), 1);

    assert!(node.produce_block().unwrap().is_some());
    assert_eq!(node.pending_count(), 0);
    assert_eq!(bal(&mut node, 2), 400);
    // 599 change (1000 - 400 - 1 fee) + 1000 KVNC subsidy.
    assert_eq!(bal(&mut node, 1), 1599);
    assert_eq!(node.block_count().unwrap(), 2); // genesis + produced block
}

#[test]
fn non_conflicting_entries_from_two_actors_pack_together() {
    let mut node = poa_node(1000, 1000);
    // Fund actor 2 immediately so two actors each have a spendable output.
    node.send(1, 500, 2).unwrap();
    // Now pool spends from each (different outputs — no conflict).
    node.pool(1, 100, 3).unwrap();
    node.pool(2, 100, 4).unwrap();
    assert_eq!(node.pending_count(), 2);

    assert!(node.produce_block().unwrap().is_some());
    assert_eq!(node.pending_count(), 0);
    // 398 change (499 - 100 - 1 fee) + 1000 subsidy coinbase.
    assert_eq!(bal(&mut node, 1), 1398);
    assert_eq!(bal(&mut node, 2), 399); // 500 - 100 - 1 fee
    assert_eq!(bal(&mut node, 3), 100);
    assert_eq!(bal(&mut node, 4), 100);
}

#[test]
fn conflicting_pool_entries_are_partially_included() {
    // Both pooled transfers spend actor 1's single 1000 output, so only one can
    // be included. The loser is then evicted: its input is gone from the
    // selected-tip UTXO, so it can never apply on this branch.
    let mut node = poa_node(1000, 1000);
    node.pool(1, 400, 2).unwrap();
    node.pool(1, 300, 3).unwrap();
    assert_eq!(node.pending_count(), 2);

    assert!(node.produce_block().unwrap().is_some());
    let (b2, b3) = (bal(&mut node, 2), bal(&mut node, 3));
    assert!((b2 == 400 && b3 == 0) || (b2 == 0 && b3 == 300));
    assert_eq!(node.pending_count(), 0);

    assert!(node.produce_block().unwrap().is_none());
    assert_eq!(node.pending_count(), 0);
}

#[test]
fn producing_from_an_empty_mempool_is_a_noop() {
    let mut node = poa_node(1000, 500);
    assert!(node.produce_block().unwrap().is_none());
    assert_eq!(node.block_count().unwrap(), 1);
}
