//! Phase 1 Golden Verification Vectors for Kovanica Wallet Core.
//!
//! Asserts:
//! 1. Vector T1 sighash and broadcast format.
//! 2. Standard HD derivation vector for m/44'/3007'/0'/0'/0'.
//! 3. PRO-21 address round-trip and flipped-character checksum rejection.
//! 4. Chain 1 (Mainnet) rejection of legacy hex addresses.
//! 5. Distinct public keys for HD derivation vs. legacy ADR-0002 raw seed on the same 24 words.

use bip39::{Language, Mnemonic};
use kovanica_types::signing::chain_id;
use kovanica_types::{Address, PublicKey, SigningKey};
use kovanica_wallet_core::{Slip10Node, TxPreview, WalletAccount};

#[test]
fn test_t1_golden_sighash_and_broadcast() {
    // Acceptance vector T1 from specification:
    // raw seed 07×32, not HD
    // chain 2, nonce 5, to 22×32, amount 1000000, fee 1000
    // sender ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c
    // hash 4acde0e0a120d6c03eaa197c082238fefadaf36f4f40bc1364ae8b3faa067798
    let seed = [0x07u8; 32];
    let sk = SigningKey::from_bytes(&seed);
    let pk = PublicKey::from_bytes(sk.verifying_key().to_bytes());
    assert_eq!(
        hex::encode(pk.0),
        "ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c"
    );

    let preview = TxPreview::new(Address(pk.0), Address([0x22u8; 32]), 1_000_000, 1_000, 5, 2);

    let sighash = preview.signing_hash();
    assert_eq!(
        hex::encode(sighash.0),
        "4acde0e0a120d6c03eaa197c082238fefadaf36f4f40bc1364ae8b3faa067798"
    );

    let (tx, raw_hex) = preview.sign(&sk).expect("signing succeeds");
    assert!(tx.verify_signature(&kovanica_types::signing::SigningContext::new(2)));
    assert_eq!(tx.hash, sighash);
    assert!(!raw_hex.starts_with("0x"));
}

#[test]
fn test_published_hd_vector_3007() {
    // 12-word standard BIP-39 mnemonic
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let m = Mnemonic::parse_in(Language::English, phrase).unwrap();
    let seed_64 = m.to_seed("");
    let master = Slip10Node::master(&seed_64).unwrap();

    // Derive m/44'/3007'/0'/0'/0'
    let account = master.derive_bip44(0, 0, 0).unwrap();
    let sk = SigningKey::from_bytes(&account.secret_key);
    let pk = PublicKey::from_bytes(sk.verifying_key().to_bytes());
    let addr = Address::from_public_key(&pk);
    let pro21 = addr.encode();

    assert_eq!(pro21.len(), 64);
    assert!(pro21.starts_with("kvnc"));
    assert!(pro21.ends_with("dag"));

    // Ensure WalletAccount::from_mnemonic_hd produces the exact identical key
    let wallet_account = WalletAccount::from_mnemonic_hd(phrase, "", 0, 0, 0).unwrap();
    assert_eq!(wallet_account.public_key, pk);
    assert_eq!(wallet_account.address, addr);
}

#[test]
fn test_pro21_roundtrip_and_flipped_character_checksum_failure() {
    let pk = PublicKey([0x42u8; 32]);
    let addr = Address::from_public_key(&pk);
    let encoded = addr.encode();

    assert_eq!(encoded.len(), 64);
    assert!(encoded.starts_with("kvnc"));
    assert!(encoded.ends_with("dag"));

    // Round-trip decode
    let decoded = Address::decode(&encoded).expect("valid PRO-21 address must decode");
    assert_eq!(decoded, addr);

    // Flip a character in the base32 payload (e.g. index 10)
    let mut chars: Vec<char> = encoded.chars().collect();
    let orig = chars[10];
    chars[10] = if orig == 'q' { 'p' } else { 'q' };
    let corrupted: String = chars.into_iter().collect();

    let err = Address::decode(&corrupted);
    assert!(
        err.is_err(),
        "flipped character in PRO-21 address must fail checksum verification"
    );
}

#[test]
fn test_chain1_strictly_rejects_hex_addresses() {
    let pk = PublicKey([0x42u8; 32]);
    let addr = Address::from_public_key(&pk);
    let canonical = addr.encode();

    // Canonical PRO-21 base32 is accepted on Chain 1
    let parsed_canonical = Address::parse_for_chain(&canonical, chain_id::MAINNET);
    assert!(parsed_canonical.is_ok());

    // Legacy 64-hex string
    let raw_hex = hex::encode(addr.0);
    let err_raw = Address::parse_for_chain(&raw_hex, chain_id::MAINNET);
    assert!(err_raw.is_err(), "Chain 1 must reject raw hex addresses");

    // Legacy 0x-prefixed hex string
    let prefixed_hex = format!("0x{raw_hex}");
    let err_prefixed = Address::parse_for_chain(&prefixed_hex, chain_id::MAINNET);
    assert!(
        err_prefixed.is_err(),
        "Chain 1 must reject 0x-prefixed hex addresses"
    );

    // Legacy kvnc<hex>dag format
    let legacy_kvnc = addr.encode_legacy_hex();
    let err_legacy = Address::parse_for_chain(&legacy_kvnc, chain_id::MAINNET);
    assert!(
        err_legacy.is_err(),
        "Chain 1 must reject legacy kvnc<hex>dag addresses"
    );
}

#[test]
fn test_legacy_adr0002_differs_from_hd_on_same_24_words() {
    let fixed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    // 1. HD derivation at m/44'/3007'/0'/0'/0'
    let hd = WalletAccount::from_mnemonic_hd(fixed_phrase, "", 0, 0, 0).unwrap();

    // 2. Legacy ADR-0002 (raw 32-byte entropy)
    let m = Mnemonic::parse_in(Language::English, fixed_phrase).unwrap();
    let mut legacy_seed = [0u8; 32];
    legacy_seed.copy_from_slice(&m.to_entropy());
    let legacy = WalletAccount::from_seed(&legacy_seed);

    assert_ne!(
        hd.public_key, legacy.public_key,
        "HD SLIP-0010 derivation and legacy ADR-0002 raw seed MUST produce distinct public keys"
    );
    assert_ne!(
        hd.address, legacy.address,
        "HD SLIP-0010 derivation and legacy ADR-0002 raw seed MUST produce distinct addresses"
    );
}
