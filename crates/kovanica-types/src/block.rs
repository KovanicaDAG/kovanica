//! Block and StatementBlock definitions for the DAG.

use crate::crypto::Signature;
use crate::hash::Hash;
use crate::merkle::{self, MerkleProof};
use crate::transaction::Transaction;
use crate::{AuthorityIndex, Round};
use serde::{Deserialize, Serialize};

/// Reference to a block in the DAG (author + round + digest).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct BlockReference {
    /// Authority index of the block author.
    pub author: AuthorityIndex,
    /// Round number of the block.
    pub round: Round,
    /// Digest/hash of the block.
    pub digest: Hash,
}

/// Full block as used in the DAG (Mysticeti-style StatementBlock).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct StatementBlock {
    /// Author of this block.
    pub author: AuthorityIndex,
    /// Round number.
    pub round: Round,
    /// Parents (at least 2f+1 from previous round).
    pub parents: Vec<BlockReference>,
    /// Transactions included in this block.
    pub transactions: Vec<Transaction>,
    /// Optional: additional statements / votes.
    pub statements: Vec<u8>, // placeholder for future vote data
    /// Signature over the block content.
    pub signature: Signature,
    /// Cached digest.
    pub digest: Hash,
    /// Binary merkle root over included transaction hashes.
    pub merkle_root: Hash,
}

impl StatementBlock {
    /// Compute digest with optional VRF-authenticated statements.
    /// Old calls (no statements / not VRF) yield exactly the previous digest.
    pub fn compute_digest(
        author: AuthorityIndex,
        round: Round,
        parents: &[BlockReference],
        transactions: &[Transaction],
    ) -> Hash {
        Self::compute_digest_statements(author, round, parents, transactions, None)
    }

    /// Digest that includes `statements` only when a VRF payload is present.
    /// This is the wire-upgrade digest; old empty-statement blocks stay unchanged.
    pub fn compute_digest_statements(
        author: AuthorityIndex,
        round: Round,
        parents: &[BlockReference],
        transactions: &[Transaction],
        statements: Option<&[u8]>,
    ) -> Hash {
        let mut data = Vec::new();
        data.extend_from_slice(&author.to_le_bytes());
        data.extend_from_slice(&round.to_le_bytes());
        for p in parents {
            data.extend_from_slice(&p.author.to_le_bytes());
            data.extend_from_slice(&p.round.to_le_bytes());
            data.extend_from_slice(&p.digest.0);
        }
        let merkle = Self::compute_merkle_root(transactions);
        data.extend_from_slice(&merkle.0);
        for tx in transactions {
            data.extend_from_slice(&tx.hash().0);
        }
        // Wire upgrade: only include statements when they carry the VRF blob.
        if let Some(s) = statements {
            if s.len() >= 137 && &s[..9] == b"KVNC-VRF1" {
                data.extend_from_slice(s);
            }
        }
        Hash::new_keyed(Hash::DOMAIN_BLOCK, &data)
    }

    /// Binary Merkle root over the included transaction hashes.
    /// Empty transaction list yields a fixed zero hash.
    pub fn merkle_root(&self) -> Hash {
        Self::compute_merkle_root(&self.transactions)
    }

    /// Compute merkle root from a slice of transactions (static).
    pub fn compute_merkle_root(transactions: &[Transaction]) -> Hash {
        let leaves: Vec<Hash> = transactions.iter().map(|tx| tx.hash()).collect();
        merkle::root(&leaves)
    }

    /// Inclusion proof for `transactions[index]` against [`Self::compute_merkle_root`].
    ///
    /// `None` when `index` is out of range. An empty block has no proof; its
    /// root is the zero hash.
    pub fn tx_inclusion_proof(&self, index: usize) -> Option<MerkleProof> {
        let leaves: Vec<Hash> = self.transactions.iter().map(|tx| tx.hash()).collect();
        merkle::proof(&leaves, index)
    }

    /// Check that `tx_hash` sits in `root` along `proof`.
    pub fn verify_tx_inclusion(tx_hash: &Hash, proof: &MerkleProof, root: &Hash) -> bool {
        merkle::verify(tx_hash, proof, root)
    }
}

/// Simplified Block alias used in higher layers.
pub type Block = StatementBlock;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::Address;
    use crate::crypto::Signature;

    fn dummy_tx(hash: Hash) -> Transaction {
        Transaction {
            sender: Address([0; 32]),
            nonce: 0,
            kind: crate::transaction::TransactionKind::Transfer {
                to: Address([1; 32]),
                amount: 100,
            },
            fee: 0,
            signature: Signature([0; 64]),
            hash,
        }
    }

    #[test]
    fn merkle_empty_is_zero() {
        let block = StatementBlock {
            author: 0,
            round: 0,
            parents: vec![],
            transactions: vec![],
            statements: vec![],
            signature: Signature([0; 64]),
            digest: Hash::zero(),
            merkle_root: Hash::zero(),
        };
        assert_eq!(block.merkle_root(), Hash::zero());
    }

    #[test]
    fn merkle_single_is_hash() {
        let h = Hash::new_keyed(Hash::DOMAIN_TX, b"tx1");
        let block = StatementBlock {
            author: 0,
            round: 1,
            parents: vec![],
            transactions: vec![dummy_tx(h)],
            statements: vec![],
            signature: Signature([0; 64]),
            digest: Hash::zero(),
            merkle_root: Hash::zero(),
        };
        assert_eq!(block.merkle_root(), h);
    }

    #[test]
    fn merkle_three_transactions() {
        let h1 = Hash::new_keyed(Hash::DOMAIN_TX, b"tx-a");
        let h2 = Hash::new_keyed(Hash::DOMAIN_TX, b"tx-b");
        let h3 = Hash::new_keyed(Hash::DOMAIN_TX, b"tx-c");
        let block = StatementBlock {
            author: 0,
            round: 2,
            parents: vec![],
            transactions: vec![dummy_tx(h1), dummy_tx(h2), dummy_tx(h3)],
            statements: vec![],
            signature: Signature([0; 64]),
            digest: Hash::zero(),
            merkle_root: Hash::zero(),
        };
        let root = block.merkle_root();
        assert_ne!(root, Hash::zero());
        assert_eq!(block.merkle_root(), root);
        let proof = block.tx_inclusion_proof(2).unwrap();
        assert!(StatementBlock::verify_tx_inclusion(&h3, &proof, &root));
        assert!(!StatementBlock::verify_tx_inclusion(&h1, &proof, &root));
    }

    #[test]
    fn domain_separation_block_vs_tx() {
        let data = b"same-data";
        let block_hash = Hash::new_keyed(Hash::DOMAIN_BLOCK, data);
        let tx_hash = Hash::new_keyed(Hash::DOMAIN_TX, data);
        assert_ne!(block_hash, tx_hash, "domain tags must separate hashes");
    }

    #[test]
    fn digest_uses_domain() {
        let d = StatementBlock::compute_digest(1, 2, &[], &[]);
        assert_ne!(d, Hash::zero());
        // Keyed with DOMAIN_BLOCK differs from unkeyed same payload
        let payload = {
            let mut v = Vec::new();
            v.extend_from_slice(&1u32.to_le_bytes());
            v.extend_from_slice(&2u32.to_le_bytes());
            v
        };
        assert_ne!(d, Hash::new(&payload), "must be keyed, not raw BLAKE3");
    }

    #[test]
    fn domain_separation_digest_vs_merkle() {
        let data = b"pair";
        let d = Hash::new_keyed(Hash::DOMAIN_DIGEST, data);
        let m = Hash::new_keyed(Hash::DOMAIN_MERKLE, data);
        assert_ne!(d, m);
    }
}
