//! Negative vectors N1 and N3 from docs/BLOCK_SIGNATURE_V2.md (PR #40).
//! Both are accepted by `ed25519_dalek::verify_batch`; kovanica-crypto must reject
//! them on every path (single block, batch alone, batch mixed with a valid
//! item, and generic tx verification).
use ed25519_dalek::{SigningKey, VerifyingKey};
use kovanica_crypto::{sign, verify, verify_batch, verify_block_signature};
use kovanica_types::{block::StatementBlock, crypto::PublicKey, crypto::Signature, hash::Hash};
use std::sync::Mutex;

// verify_batch reads a process-global key table; serialize tests touching it.
static LOCK: Mutex<()> = Mutex::new(());

const SMALL_ORDER_PK: &str = "0100000000000000000000000000000000000000000000000000000000000000";
const N1_SIG: &str = "01000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
const N3_SIG: &str = "eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f0000000000000000000000000000000000000000000000000000000000000000";

fn b32(h: &str) -> [u8; 32] {
    hex::decode(h).unwrap().try_into().unwrap()
}
fn b64(h: &str) -> [u8; 64] {
    hex::decode(h).unwrap().try_into().unwrap()
}

fn block(author: u16, digest: Hash, sig: Signature) -> StatementBlock {
    StatementBlock {
        author,
        round: 1,
        parents: vec![],
        transactions: vec![],
        statements: vec![],
        signature: sig,
        digest,
        merkle_root: Default::default(),
    }
}

fn check_block_vector(sig_hex: &str) {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let good_sk = SigningKey::from_bytes(&[7u8; 32]);
    let bad_vk = VerifyingKey::from_bytes(&b32(SMALL_ORDER_PK)).unwrap();
    kovanica_crypto::set_validator_keys(vec![good_sk.verifying_key(), bad_vk]);

    let chain_id = 1337;
    let digest = StatementBlock::compute_digest(0, 1, &[], &[]);
    let msg = kovanica_types::block::block_signature_data(chain_id, &digest);
    let n0 = block(0, digest, sign(&good_sk, &msg));
    let nx = block(1, digest, Signature(b64(sig_hex)));

    // Sanity: the dalek batch path really does accept this vector over the message.
    assert!(ed25519_dalek::verify_batch(
        &[&msg],
        &[ed25519_dalek::Signature::from_bytes(&nx.signature.0)],
        &[bad_vk]
    )
    .is_ok());

    assert!(verify_block_signature(
        chain_id,
        &PublicKey(b32(SMALL_ORDER_PK)),
        &digest,
        &nx.signature
    )
    .is_err());
    assert!(verify_batch(chain_id, std::slice::from_ref(&n0)).is_ok());
    assert!(verify_batch(chain_id, std::slice::from_ref(&nx)).is_err());
    assert!(verify_batch(chain_id, &[n0.clone(), nx.clone()]).is_err());
    assert!(verify_batch(chain_id, &[nx, n0.clone()]).is_err());

    // Cross-chain verification must fail
    assert!(verify_block_signature(
        chain_id + 1,
        &PublicKey(good_sk.verifying_key().to_bytes()),
        &digest,
        &n0.signature
    )
    .is_err());
    assert!(verify_batch(chain_id + 1, std::slice::from_ref(&n0)).is_err());
}

fn check_tx_vector(sig_hex: &str) {
    let msg = Hash::new("tx-signing-hash");
    let pk = PublicKey(b32(SMALL_ORDER_PK));
    assert!(verify(&pk, msg.as_ref(), &Signature(b64(sig_hex))).is_err());
}

#[test]
fn n1_small_order_pubkey_block_rejected() {
    check_block_vector(N1_SIG);
}
#[test]
fn n3_noncanonical_r_block_rejected() {
    check_block_vector(N3_SIG);
}
#[test]
fn n1_small_order_pubkey_tx_rejected() {
    check_tx_vector(N1_SIG);
}
#[test]
fn n3_noncanonical_r_tx_rejected() {
    check_tx_vector(N3_SIG);
}
#[test]
fn valid_tx_signature_still_verifies() {
    let sk = SigningKey::from_bytes(&[7u8; 32]);
    let msg = Hash::new("tx-signing-hash");
    let pk = PublicKey(sk.verifying_key().to_bytes());
    assert!(verify(&pk, msg.as_ref(), &sign(&sk, msg.as_ref())).is_ok());
}
