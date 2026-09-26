use ed25519_dalek::SigningKey;
use getrandom::getrandom;

fn main() {
    println!("=== Generated testnet authority keys (KEEP SECRETS SECURE) ===");
    println!();
    for i in 0..3 {
        let mut seed = [0u8; 32];
        getrandom(&mut seed).expect("OS randomness");
        let sk = SigningKey::from_bytes(&seed);
        let pk = sk.verifying_key();
        println!("Authority {}:", i+1);
        println!("  Secret (64 hex): {}", hex::encode(sk.to_bytes()));
        println!("  Public (64 hex): {}", hex::encode(pk.as_bytes()));
        println!();
    }
    println!("Add to each authority's systemd unit:");
    println!("  Environment=KOVANICA_AUTHORITY_KEY=<that-authority's-secret>");
    println!();
    println!("KOVANICA_AUTHORITIES for ALL nodes (comma-separated public keys):");
    println!("  Environment=KOVANICA_AUTHORITIES=<pub1>,<pub2>,<pub3>");
    println!();
    println!("KOVANICA_AUTHORITY_THRESHOLD=2  # strict majority of 3");
    println!("KOVANICA_SLOT_DURATION=3000     # 3 second slots");
}
