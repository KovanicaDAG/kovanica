//! Known-answer vectors for the frozen SLIP-0010 ed25519 derivation.
//!
//! Official SLIP-0010 vectors 1 & 2 (public spec) plus the **frozen**
//! Kovanica path `m/44'/917'/0'/0'/i'` vectors from
//! `docs/backlog/DERIVATION.md`. These constants are shared with the
//! TypeScript web wallet (`web/site/src/lib/wallet/keys.ts`) — if either side
//! drifts, this suite (or the web suite) fails loudly.
//!
//! The frozen-path inputs use the standard **zero-entropy 128-bit phrase**
//! (12 words, empty passphrase) — a public test constant, never a real
//! wallet. The phrase is built from entropy here so no mnemonic-like string
//! appears in source.

fn input_from_entropy(entropy: &[u8], passphrase: &str) -> [u8; 64] {
    let m = bip39::Mnemonic::from_entropy_in(bip39::Language::English, entropy)
        .expect("entropy is valid length");
    let mut out = [0u8; 64];
    out.copy_from_slice(&m.to_seed(passphrase));
    out
}

/// Zero-entropy 128-bit input (canonical 12-word test phrase).
fn zero_input() -> [u8; 64] {
    input_from_entropy(&[0u8; 16], "")
}

#[test]
fn frozen_path_index_0() {
    let out = kovanica_keys::slip10::derive_ed25519(&zero_input(), 0);
    let expected = [
        0x01u8, 0xd2, 0xbd, 0xbe, 0xba, 0xce, 0xa6, 0xea, 0xef, 0xd3, 0x6b, 0x7f, 0xec, 0xe9, 0x64,
        0x1d, 0x68, 0x54, 0x8e, 0xa0, 0x28, 0xb2, 0xc0, 0xde, 0xb0, 0xaf, 0x2a, 0x3a, 0xd5, 0x98,
        0xc5, 0x6b,
    ];
    assert_eq!(out, expected);
}

#[test]
fn frozen_path_index_1() {
    let out = kovanica_keys::slip10::derive_ed25519(&zero_input(), 1);
    let expected = [
        0x9du8, 0xd6, 0x1f, 0xca, 0xe2, 0x11, 0xc2, 0x8c, 0x27, 0xc9, 0x80, 0xdb, 0x26, 0x8e, 0x23,
        0x48, 0x45, 0x0e, 0xa9, 0xb3, 0xcb, 0x72, 0x5d, 0x3c, 0x96, 0xdb, 0x68, 0x50, 0x49, 0xd7,
        0xb1, 0x83,
    ];
    assert_eq!(out, expected);
}

#[test]
fn frozen_path_index_2() {
    let out = kovanica_keys::slip10::derive_ed25519(&zero_input(), 2);
    let expected = [
        0x99u8, 0x03, 0x5c, 0xdc, 0x69, 0xcf, 0xa4, 0x16, 0x7f, 0x5d, 0x3a, 0x4f, 0x34, 0x25, 0x0b,
        0x19, 0x42, 0x77, 0x79, 0xa3, 0x66, 0x33, 0xf8, 0x21, 0xd0, 0xad, 0x6d, 0xe8, 0x5e, 0xfb,
        0xdd, 0x96,
    ];
    assert_eq!(out, expected);
}

#[test]
fn passphrase_changes_key() {
    let k0 = kovanica_keys::slip10::derive_ed25519(&zero_input(), 0);
    let k1 = kovanica_keys::slip10::derive_ed25519(
        &input_from_entropy(&[0u8; 16], "test passphrase"),
        0,
    );
    assert_ne!(k0, k1);
}

#[test]
fn different_indices_differ() {
    let s = zero_input();
    let a = kovanica_keys::slip10::derive_ed25519(&s, 0);
    let b = kovanica_keys::slip10::derive_ed25519(&s, 1);
    assert_ne!(a, b);
}

/// M-09 property test: round-trip generate → restore produces identical keys.
///
/// Deterministic sweep (no RNG in the test): several fixed entropies × both
/// word counts × passphrase on/off. The public `from_mnemonic_at` path
/// (mnemonic → BIP39 seed → SLIP-0010) must round-trip byte-identically, and
/// the address derived from the mnemonic must match the raw-key derivation.
#[test]
fn roundtrip_generate_restore_same_keys() {
    use kovanica_keys::{Keypair, Mnemonic, WordCount};

    let entropies: &[&[u8]] = &[
        &[0u8; 16],
        &[0xab; 16],
        &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10,
        ],
        &[0x42; 32],
    ];
    let passphrases = ["", "hunter2", "kova nica 25th word"];

    for (i, entropy) in entropies.iter().enumerate() {
        for words in [WordCount::Words12, WordCount::Words24] {
            // Generate: fresh mnemonic from fixed entropy.
            let raw = bip39::Mnemonic::from_entropy_in(bip39::Language::English, *entropy)
                .expect("valid entropy len");
            let mnemonic = Mnemonic::from_phrase(raw.to_string().as_str()).expect("parses");
            let phrase = mnemonic.phrase().to_string();

            for pass in passphrases {
                // Restore: re-parse the phrase (what a user would type in).
                let restored = Mnemonic::from_phrase(&phrase).expect("phrase re-parses");
                assert_eq!(mnemonic.phrase(), restored.phrase());

                // Same passphrase → same keypair, same address, same pubkey.
                let kp1 = Keypair::from_mnemonic_at(&mnemonic, pass, 0);
                let kp2 = Keypair::from_mnemonic_at(&restored, pass, 0);
                assert_eq!(kp1.address(), kp2.address(), "case {i}");
                assert_eq!(kp1.public_key(), kp2.public_key());

                // A wrong passphrase NEVER round-trips to the same keys.
                let kp_wrong = Keypair::from_mnemonic_at(&mnemonic, "wrong", 0);
                assert_ne!(kp_wrong.address(), kp1.address(), "case {i}");

                // Index 0 of `from_mnemonic` must equal `from_mnemonic_at(.., 0)`
                // (the CLI/web default account), and different indices differ.
                let kp_def = Keypair::from_mnemonic(&mnemonic, pass);
                assert_eq!(kp_def.address(), kp1.address(), "case {i}");
                let kp_5 = Keypair::from_mnemonic_at(&mnemonic, pass, 5);
                assert_ne!(kp_5.address(), kp1.address(), "case {i}");
            }
        }
    }
}
