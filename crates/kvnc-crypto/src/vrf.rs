//! Verifiable Random Function (VRF) implementation for Kovanica.
//!
//! Implements ECVRF-ED25519-SHA512-Elligator2 per RFC 9381 via `ecvrf` crate.
//! Used for stake-weighted, unpredictable leader selection in consensus.
//!
//! The VRF output is a pair (hash, proof):
//! - `hash` = 32-byte VRF hash output (DETERMINISTIC, used as randomness source)
//! - `proof` = 96-byte VRF proof (NON-DETERMINISTIC, verifiable by anyone with the public key)

use ecvrf::{keygen, prove, verify, VrfProof, VrfPk, VrfSk};
use kvnc_types::crypto::PublicKey;
use thiserror::Error;

/// VRF output: 32-byte hash (deterministic) + 96-byte proof (non-deterministic)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VRFOutput {
    /// VRF hash output (32 bytes) — DETERMINISTIC for given (input, secret key)
    pub hash: [u8; 32],
    /// VRF proof (96 bytes) — NON-DETERMINISTIC, verifiable by anyone with the public key
    pub proof: [u8; 96],
}

impl VRFOutput {
    /// Create from raw bytes
    pub fn new(hash: [u8; 32], proof: [u8; 96]) -> Self {
        Self { hash, proof }
    }

    /// Get the hash as a slice
    pub fn hash(&self) -> &[u8; 32] {
        &self.hash
    }

    /// Get the proof as a slice
    pub fn proof(&self) -> &[u8; 96] {
        &self.proof
    }
}

/// VRF keypair (secret + public key)
#[derive(Clone, Debug)]
pub struct VRFKeyPair {
    /// Secret key (32-byte scalar)
    secret: [u8; 32],
    /// Public key (32-byte compressed Ristretto point)
    public: [u8; 32],
}

impl VRFKeyPair {
    /// Generate a new VRF keypair
    pub fn generate() -> Self {
        let (sk, pk) = keygen();
        let mut secret = [0u8; 32];
        let mut public = [0u8; 32];
        secret.copy_from_slice(&sk.to_bytes());
        public.copy_from_slice(&pk.to_bytes());
        Self { secret, public }
    }

    /// Create from raw secret key bytes (32-byte scalar)
    pub fn from_secret(secret: [u8; 32]) -> Result<Self, VRFError> {
        let sk = VrfSk::from_bytes(&secret)
            .map_err(|_| VRFError::InvalidSecretKey)?;
        let pk = VrfPk::new(&sk);
        let mut public = [0u8; 32];
        public.copy_from_slice(&pk.to_bytes());
        Ok(Self { secret, public })
    }

    /// Get the secret key as ecvrf VrfSk (for proving)
    pub fn secret_key(&self) -> VrfSk {
        VrfSk::from_bytes(&self.secret).expect("valid secret key")
    }

    /// Get the public key as ecvrf VrfPk (for verification)
    pub fn public_key(&self) -> VrfPk {
        VrfPk::from_bytes(&self.public).expect("valid public key")
    }

    /// Get the public key as kvnc_types::PublicKey (32-byte Ristretto point)
    pub fn public_key_typed(&self) -> PublicKey {
        PublicKey(self.public)
    }
}

/// Compute VRF output for given input
pub fn vrf_prove(keypair: &VRFKeyPair, input: &[u8]) -> VRFOutput {
    let secret_key = keypair.secret_key();
    let (hash, proof) = prove(input, &secret_key);
    let mut out_hash = [0u8; 32];
    let mut out_proof = [0u8; 96];
    out_hash.copy_from_slice(&hash);
    out_proof.copy_from_slice(&proof.to_bytes());
    VRFOutput::new(out_hash, out_proof)
}

/// Verify VRF output
pub fn vrf_verify(public_key: &PublicKey, input: &[u8], output: &VRFOutput) -> Result<(), VRFError> {
    let pk = VrfPk::from_bytes(&public_key.0)
        .map_err(|_| VRFError::InvalidPublicKey)?;
    let proof = VrfProof::from_bytes(&output.proof)
        .map_err(|_| VRFError::InvalidProof)?;
    let valid = verify(input, &pk, &output.hash, &proof);
    if !valid {
        return Err(VRFError::VerificationFailed);
    }
    Ok(())
}

/// Convert VRF hash to a u64 for leader selection (deterministic, uniform)
pub fn vrf_hash_to_u64(hash: &[u8; 32]) -> u64 {
    u64::from_le_bytes([
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5], hash[6], hash[7],
    ])
}

/// Errors for VRF operations
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VRFError {
    #[error("Invalid VRF secret key")]
    InvalidSecretKey,
    #[error("Invalid VRF public key")]
    InvalidPublicKey,
    #[error("Invalid VRF proof")]
    InvalidProof,
    #[error("VRF verification failed")]
    VerificationFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vrf_keypair_generation() {
        let kp = VRFKeyPair::generate();
        assert_eq!(kp.secret.len(), 32);
        assert_eq!(kp.public.len(), 32);
    }

    #[test]
    fn vrf_prove_verify() {
        let kp = VRFKeyPair::generate();
        let input = b"test input for VRF";
        let output = vrf_prove(&kp, input);
        
        // Verify with correct public key
        assert!(vrf_verify(&kp.public_key_typed(), input, &output).is_ok());
        
        // Verify with wrong public key fails
        let other_kp = VRFKeyPair::generate();
        assert!(vrf_verify(&other_kp.public_key_typed(), input, &output).is_err());
        
        // Verify with wrong input fails
        assert!(vrf_verify(&kp.public_key_typed(), b"wrong input", &output).is_err());
    }

    #[test]
    fn vrf_hash_deterministic() {
        let kp = VRFKeyPair::generate();
        let input = b"deterministic test";
        let out1 = vrf_prove(&kp, input);
        let out2 = vrf_prove(&kp, input);
        // Hash MUST be deterministic, proof is non-deterministic
        assert_eq!(out1.hash, out2.hash);
        assert_ne!(out1.proof, out2.proof); // Different proofs due to random nonce
    }

    #[test]
    fn vrf_hash_to_u64_uniform() {
        let kp = VRFKeyPair::generate();
        let mut counts = [0u32; 10];
        for i in 0..10000 {
            let input = format!("test {}", i).into_bytes();
            let output = vrf_prove(&kp, &input);
            let u = vrf_hash_to_u64(&output.hash) % 10;
            counts[u as usize] += 1;
        }
        // Chi-squared test: each bucket should have ~1000
        for c in counts {
            assert!(c > 800 && c < 1200, "distribution not uniform: {:?}", counts);
        }
    }

    #[test]
    fn vrf_from_secret_roundtrip() {
        let kp1 = VRFKeyPair::generate();
        let kp2 = VRFKeyPair::from_secret(kp1.secret).unwrap();
        assert_eq!(kp1.secret, kp2.secret);
        assert_eq!(kp1.public, kp2.public);
        
        let input = b"roundtrip test";
        let out1 = vrf_prove(&kp1, input);
        let out2 = vrf_prove(&kp2, input);
        // Hash must be identical, proof will differ
        assert_eq!(out1.hash, out2.hash);
    }

    #[test]
    fn vrf_proof_non_deterministic() {
        let kp = VRFKeyPair::generate();
        let input = b"non-deterministic proof test";
        let out1 = vrf_prove(&kp, input);
        let out2 = vrf_prove(&kp, input);
        assert_ne!(out1.proof, out2.proof);
        // But verification should still pass for both
        assert!(vrf_verify(&kp.public_key_typed(), input, &out1).is_ok());
        assert!(vrf_verify(&kp.public_key_typed(), input, &out2).is_ok());
    }
}