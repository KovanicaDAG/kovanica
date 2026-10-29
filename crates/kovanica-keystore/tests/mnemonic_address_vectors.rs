//! Frozen mnemonic → address vectors, shared with the web wallet
//! (`kovanica-web/tests/fixtures/vectors.json`) and the Android apps.
//!
//! A 24-word phrase *is* the Ed25519 seed (ADR-0002: no PBKDF2, no derivation
//! path), so the same phrase must restore the same wallet everywhere. If one of
//! these assertions fails, clients have silently stopped interoperating — treat
//! it as a data-loss bug, not a cosmetic one.

use kovanica_keystore::{address, from_seed, seed_from_mnemonic, seed_to_mnemonic};

const ABANDON_ART: &str = "abandon abandon abandon abandon abandon abandon abandon abandon \
     abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon \
     abandon abandon abandon abandon art";

const LEGAL_WINNER: &str = "legal winner thank year wave sausage worth useful legal winner \
     thank year wave sausage worth useful legal winner thank year wave sausage worth title";

fn address_of(phrase: &str) -> String {
    let seed = seed_from_mnemonic(phrase).expect("phrase is valid BIP-39");
    let keystore = from_seed(&seed, "vectors").expect("seed becomes a keystore");
    address(&keystore)
        .expect("keystore derives an address")
        .encode()
}

#[test]
fn known_phrases_derive_the_frozen_addresses() {
    assert_eq!(
        address_of(ABANDON_ART),
        "kvnc8d4z00xwk6jz6c4r4rgz5mcdwdjny9thrh3y8f36cpy2rz6emg54rufktdag"
    );
    assert_eq!(
        address_of(LEGAL_WINNER),
        "kvnck2559l6vnpcchmtkuf2eslmdtxc6wtfm9nf9zqqructs4336nla58y78mdag"
    );
}

#[test]
fn seed_survives_a_mnemonic_round_trip() {
    for seed in [[0x11u8; 32], [0x00u8; 32], [0xffu8; 32]] {
        let phrase = seed_to_mnemonic(&seed).expect("any 32 bytes encode");
        let restored = seed_from_mnemonic(&phrase).expect("our own phrase decodes");
        assert_eq!(&*restored, &seed);
        assert_eq!(address_of(&phrase), address_of(&phrase));
    }
}

#[test]
fn whitespace_around_a_phrase_is_ignored() {
    let padded = format!("  {ABANDON_ART}\n");
    assert_eq!(address_of(&padded), address_of(ABANDON_ART));
}
