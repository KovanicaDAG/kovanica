use kovanica_crypto::{
    kovanica_hash, KovanicaHasher, KOVANICA_HASH_CHUNK_SIZE, KOVANICA_HASH_MEMORY_SIZE,
    KOVANICA_HASH_MIX_ROUNDS, KOVANICA_HASH_NUM_CHUNKS,
};

#[test]
fn kovanica_hash_spec_invariants() {
    assert_eq!(KOVANICA_HASH_MEMORY_SIZE, 2 * 1024 * 1024);
    assert_eq!(KOVANICA_HASH_CHUNK_SIZE, 32);
    assert_eq!(KOVANICA_HASH_NUM_CHUNKS, 65_536);
    assert_eq!(KOVANICA_HASH_MIX_ROUNDS, 1_024);
}

#[test]
fn kovanica_hash_golden_test_vectors() {
    // Vector 1: Empty input
    let h_empty = kovanica_hash(b"");
    assert_eq!(
        h_empty.to_hex(),
        "05cca3939da23b1b771692875380dd70d953c98ba5498468a683add6aaa2a623"
    );

    // Vector 2: 132-byte canonical block preimage (like 1 parent header)
    let preimage_132 = [0x5au8; 132];
    let h_132 = kovanica_hash(&preimage_132);
    assert_eq!(
        h_132.to_hex(),
        "cce556a21d6efee5463d39911877bab564d5fbe70f6b07fea1dfad2c1ccb2db4"
    );

    // Vector 3: Fixed string preimage
    let h_fixed = kovanica_hash(b"KVNC-CHAIN5-HYBRID-POW-TEST-PREIMAGE");
    assert_eq!(
        h_fixed.to_hex(),
        "ba2bbe92ef378bdbf970bdde68a2d5cffaaea4c5aa02c9146ddd9b1765b8b262"
    );
}

#[test]
fn kovanica_hasher_reuse_identical_to_fresh() {
    let mut hasher = KovanicaHasher::new();
    let sample = b"sample preimage block data for hybrid pow consensus";

    let hash_reused = hasher.hash(sample);
    let hash_fresh = kovanica_hash(sample);
    assert_eq!(hash_reused, hash_fresh);

    // Multi-turn reuse
    for i in 0..5 {
        let input = format!("iterative-input-{}", i);
        let h1 = hasher.hash(input.as_bytes());
        let h2 = kovanica_hash(input.as_bytes());
        assert_eq!(h1, h2);
    }
}
