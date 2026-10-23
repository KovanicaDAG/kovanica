//! BLAKE3-based hash type.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 32-byte BLAKE3 hash.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    /// Compute BLAKE3 hash of the given data.
    pub fn new(data: impl AsRef<[u8]>) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(data.as_ref());
        Self(*hasher.finalize().as_bytes())
    }

    /// Compute BLAKE3 hash with a domain key (domain separation).
    pub fn new_keyed(domain: &[u8], data: impl AsRef<[u8]>) -> Self {
        let mut key = [0u8; 32];
        let len = domain.len().min(32);
        key[..len].copy_from_slice(&domain[..len]);
        let mut hasher = Hasher::new_keyed(&key);
        hasher.update(data.as_ref());
        Self(*hasher.finalize().as_bytes())
    }

    /// Domain tag for block digests.
    pub const DOMAIN_BLOCK: &'static [u8] = b"KUNA-BLOCK-v1";
    /// Domain tag for transactions.
    pub const DOMAIN_TX: &'static [u8] = b"KUNA-TX-v1";
    /// Domain tag for digest/merkle pairing.
    pub const DOMAIN_DIGEST: &'static [u8] = b"KUNA-DIGEST-v1";
    /// Domain tag for merkle tree pairing.
    pub const DOMAIN_MERKLE: &'static [u8] = b"KUNA-MERKLE-v1";
    /// Domain tag for a committed-leader header.
    pub const DOMAIN_HEADER: &'static [u8] = b"KUNA-HEADER-v1";
    /// Domain tag for an account-tree leaf.
    pub const DOMAIN_ACCOUNT: &'static [u8] = b"KUNA-ACCOUNT-v1";
    /// Domain tag for an empty account-tree subtree.
    pub const DOMAIN_ACCOUNT_EMPTY: &'static [u8] = b"KUNA-ACCT-EMPTY";
    /// Domain tag for an account-tree internal node.
    pub const DOMAIN_SMT: &'static [u8] = b"KUNA-SMT-v1";
    /// Domain tag for the committee bound into a commit seal.
    pub const DOMAIN_COMMITTEE: &'static [u8] = b"KUNA-COMMITTEE-v1";
    /// Domain tag for an equivocation-evidence id.
    pub const DOMAIN_EVIDENCE: &'static [u8] = b"KUNA-EVIDENCE-v1";

    /// Zero hash (useful for genesis).
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..16]) // short form
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", self.to_hex())
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod domain_tests {
    use super::Hash;

    /// Toni's 2026-10-19 decision: every hash domain is `KUNA-*` (chain 4
    /// reset). The tags must also stay pairwise distinct so no two hash uses
    /// can collide.
    #[test]
    fn hash_domains_are_kuna_and_distinct() {
        let domains: [&[u8]; 10] = [
            Hash::DOMAIN_BLOCK,
            Hash::DOMAIN_TX,
            Hash::DOMAIN_DIGEST,
            Hash::DOMAIN_MERKLE,
            Hash::DOMAIN_HEADER,
            Hash::DOMAIN_ACCOUNT,
            Hash::DOMAIN_ACCOUNT_EMPTY,
            Hash::DOMAIN_SMT,
            Hash::DOMAIN_COMMITTEE,
            Hash::DOMAIN_EVIDENCE,
        ];
        for (i, d) in domains.iter().enumerate() {
            assert!(
                d.starts_with(b"KUNA-"),
                "domain {} is not KUNA-*",
                String::from_utf8_lossy(d)
            );
            for other in &domains[i + 1..] {
                assert_ne!(d, other, "duplicate hash domain");
            }
        }
    }
}
