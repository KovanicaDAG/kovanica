//! Fee-market integration tests: fee-rate eviction, RBF, and the fee estimate
//! endpoint.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::num::NonZeroUsize;

use ed25519_dalek::SigningKey;
use kovanica_dag::{AuthorityPublicKey, AuthoritySet};
use kovanica_node::explorer::{handle, Explorer};
use kovanica_node::{MempoolConfig, Node};
use kovanica_state::{KeyPair, OutPoint, Transaction, TxOutput};

fn send_req(app: &mut Explorer, req: &str) -> (u16, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let mut client = TcpStream::connect(addr).unwrap();
    let (server_stream, _) = listener.accept().unwrap();

    client.write_all(req.as_bytes()).unwrap();
    client.shutdown(std::net::Shutdown::Write).unwrap();

    let _ = handle(app, server_stream);

    let mut resp = Vec::new();
    let _ = client.read_to_end(&mut resp);
    let resp_str = String::from_utf8_lossy(&resp).to_string();

    let status = resp_str
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);

    let body = if let Some(pos) = resp_str.find("\r\n\r\n") {
        resp_str[pos + 4..].to_string()
    } else {
        String::new()
    };
    (status, body)
}

fn tx_spending(op: OutPoint, _input_value: u64, output_value: u64) -> Transaction {
    let kp = KeyPair::from_u64(1);
    Transaction::signed(
        &[(op, &kp)],
        vec![TxOutput::native(output_value, kp.address())],
        vec![],
    )
}

fn actor1_utxos(node: &Node) -> Vec<(OutPoint, u64)> {
    node.utxos_of(&Node::address(1)).unwrap()
}

/// Slot duration used throughout (RFC-POA default).
const SLOT_MS: u64 = 3000;
/// Authority seeds for [`poa_node_producing_for_actor1`].
const FEE_MARKET_AUTHORITIES: u64 = 3;

/// A PoA node whose block rewards are credited to `Node::address(1)`.
///
/// Under PoA the coinbase pays the authority **scheduled for the block's
/// slot** (`Node::poa_signer_for`) — not the first key loaded into the node.
/// Paying the first loaded key was a real bug: a node producing in a slot it
/// did not own still paid the reward to whichever key it happened to load
/// first. That was fixed in #70 (`db1b0c6`), and this helper previously
/// encoded the buggy behaviour, so it has to pin the clock to land on the
/// right slot rather than assume every reward follows the first key.
///
/// Two things make "the right slot" non-obvious:
///
/// - `AuthoritySet::new` **sorts authorities by public-key bytes**, so index 0
///   is not necessarily the key for seed 1.
/// - Classic round-robin schedules `authorities[slot % len]`, so the slot
///   owned by seed 1's key is found by search rather than assumed to be 0.
///
/// `next_timestamp` clamps monotonically above the parent (genesis is at t=0),
/// so pinning to `slot * SLOT_MS` puts the first block at exactly that slot and
/// the second at `slot * SLOT_MS + 1` — the same slot. Both pay actor 1. Left on
/// the wall clock the slot would be `now_ms / SLOT_MS`, a value in the hundreds
/// of millions that rotates through the authority set, making the UTXO count
/// below wall-clock dependent.
///
/// A 3-key set is the minimum `AuthoritySet` accepts (`MIN_AUTHORITIES`), so the
/// two filler keys are required; all three signing keys are loaded so the node
/// can produce in whatever slot it is scheduled for, but only actor 1's key
/// ever receives a reward here.
fn poa_node_producing_for_actor1(config: MempoolConfig) -> Node {
    let keys: Vec<AuthorityPublicKey> = (1..=FEE_MARKET_AUTHORITIES)
        .map(|i| SigningKey::from_bytes(&KeyPair::from_u64(i).seed()).verifying_key())
        .collect();
    let set = AuthoritySet::new(keys, 2).expect("valid authority set");
    // Which slot does round-robin schedule to seed 1's key? (see doc comment)
    let actor1_pk = SigningKey::from_bytes(&KeyPair::from_u64(1).seed()).verifying_key();
    let actor1_slot = (0..FEE_MARKET_AUTHORITIES)
        .find(|slot| *set.active_authority(*slot).as_bytes() == actor1_pk.to_bytes())
        .expect("seed 1 is in the authority set");
    let mut node = Node::with_mempool_config(config);
    node.genesis_with_poa(
        3,
        100_000,
        100_000,
        1,
        None,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        None,
        set,
        SLOT_MS,
    )
    .unwrap();
    for i in 1..=FEE_MARKET_AUTHORITIES {
        node.set_authority_signing_key(KeyPair::from_u64(i).seed());
    }
    // Pin the clock into the slot round-robin schedules to actor 1 (see the doc
    // comment): under PoA the coinbase follows the scheduled authority, so an
    // unpinned wall clock would send these rewards to an arbitrary one.
    node.set_now_ms(actor1_slot * SLOT_MS);
    node
}

#[test]
fn fee_rate_eviction_drops_lowest_rate_tx() {
    let config = MempoolConfig {
        max_txs: Some(NonZeroUsize::new(2).unwrap()),
        max_bytes: None,
        min_fee_rate: 0,
        ..Default::default()
    };
    let mut node = poa_node_producing_for_actor1(config);

    // Produce two empty blocks to create additional actor-1 UTXOs.
    node.produce_empty().unwrap();
    node.produce_empty().unwrap();

    let mut utxos = actor1_utxos(&node);
    utxos.sort_by_key(|(_, v)| *v);
    utxos.reverse();
    assert!(utxos.len() >= 3, "need at least 3 utxos");

    let (op1, v1) = utxos[0];
    let (op2, v2) = utxos[1];
    let (op3, v3) = utxos[2];

    let tx_low = tx_spending(op1, v1, v1 - 1_000);
    let tx_mid = tx_spending(op2, v2, v2 - 10_000);
    let tx_high = tx_spending(op3, v3, v3 - 20_000);

    node.submit_tx(tx_low.clone()).unwrap();
    node.submit_tx(tx_mid.clone()).unwrap();

    // Adding the high-fee tx should evict the lowest-fee tx.
    node.submit_tx(tx_high.clone()).unwrap();

    assert_eq!(node.pending_count(), 2);
    assert!(!node.mempool_tx(&tx_low.id()).is_some());
    assert!(node.mempool_tx(&tx_mid.id()).is_some());
    assert!(node.mempool_tx(&tx_high.id()).is_some());
}

#[test]
fn replace_by_fee_succeeds_with_bump() {
    let config = MempoolConfig {
        min_fee_rate: 0,
        ..Default::default()
    };
    let mut node = Node::with_mempool_config(config);
    node.genesis(3, 100_000, 100_000, 1, None).unwrap();

    let utxos = actor1_utxos(&node);
    let (op, value) = utxos[0];

    let tx1 = tx_spending(op, value, value - 1_000);
    node.submit_tx(tx1.clone()).unwrap();

    let tx2 = tx_spending(op, value, value - 10_000);
    node.replace_by_fee(tx2.clone(), 1).unwrap();

    assert!(!node.mempool_tx(&tx1.id()).is_some());
    assert!(node.mempool_tx(&tx2.id()).is_some());
}

#[test]
fn replace_by_fee_rejects_insufficient_bump() {
    let config = MempoolConfig {
        min_fee_rate: 0,
        ..Default::default()
    };
    let mut node = Node::with_mempool_config(config);
    node.genesis(3, 100_000, 100_000, 1, None).unwrap();

    let utxos = actor1_utxos(&node);
    let (op, value) = utxos[0];

    let tx1 = tx_spending(op, value, value - 10_000);
    node.submit_tx(tx1.clone()).unwrap();

    // Lower fee rate; no bump.
    let tx2 = tx_spending(op, value, value - 5_000);
    let err = node.replace_by_fee(tx2.clone(), 1).unwrap_err();
    assert!(err.to_string().contains("insufficient fee bump"));

    assert!(node.mempool_tx(&tx1.id()).is_some());
    assert!(!node.mempool_tx(&tx2.id()).is_some());
}

#[test]
fn fee_estimate_endpoint_returns_rate() {
    let mut app = Explorer::boot();
    app.producing = false;

    let (status, body) = send_req(
        &mut app,
        "GET /api/fee_estimate HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
    );
    assert_eq!(status, 200, "body: {}", body);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["fee_rate"].as_u64().is_some());
    assert_eq!(json["unit"].as_str(), Some("atoms/byte"));
    assert!(json["mempool"].as_u64().is_some());
    assert!(json["bytes"].as_u64().is_some());
}
