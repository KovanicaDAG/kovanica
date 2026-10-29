//! Hardware/external signer interface (stub) for kovanica.
//!
//! Defines a minimal [`Signer`] trait and a software-backed [`SoftSigner`]
//! adapter. Real devices plug in by implementing [`Signer`]. No vendor lock-in.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;

/// A signer capable of exposing a public key and signing a 32-byte hash.
pub trait Signer {
    /// 32-byte public key (or compressed pubkey depending on impl).
    fn pubkey(&self) -> [u8; 32];
    /// Sign a 32-byte hash, returning a signature blob.
    fn sign_hash(&self, hash: &[u8; 32]) -> Result<Vec<u8>, SignerError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignerError {
    Unavailable,
    InvalidInput,
    Internal,
}

impl core::fmt::Display for SignerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SignerError::Unavailable => write!(f, "signer unavailable"),
            SignerError::InvalidInput => write!(f, "invalid input"),
            SignerError::Internal => write!(f, "internal error"),
        }
    }
}

/// Software-backed signer wrapping a 32-byte seed (demo only).
pub struct SoftSigner {
    seed: [u8; 32],
    pubkey: [u8; 32],
}

impl SoftSigner {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        // Derive a placeholder pubkey by hashing the seed. Real impls use
        // kovanica-crypto key derivation.
        let pk = kovanica_common::hash(&seed);
        Self { seed, pubkey: pk }
    }
}

impl Signer for SoftSigner {
    fn pubkey(&self) -> [u8; 32] {
        self.pubkey
    }
    fn sign_hash(&self, hash: &[u8; 32]) -> Result<Vec<u8>, SignerError> {
        // Stub: hash(seed || hash). NOT a real signature scheme.
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(&self.seed);
        buf[32..].copy_from_slice(hash);
        Ok(kovanica_common::hash(&buf).to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn soft_signer_roundtrip() {
        let s = SoftSigner::from_seed([7u8; 32]);
        let pk = s.pubkey();
        assert_ne!(pk, [0u8; 32]);
        let sig = s.sign_hash(&[1u8; 32]).unwrap();
        assert_eq!(sig.len(), 32);
    }
    #[test]
    fn different_seeds_differ() {
        let a = SoftSigner::from_seed([1u8; 32]);
        let b = SoftSigner::from_seed([2u8; 32]);
        assert_ne!(a.pubkey(), b.pubkey());
    }
}
