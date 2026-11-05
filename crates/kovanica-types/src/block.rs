//! Block and StatementBlock definitions for the DAG.

use crate::crypto::Signature;
use crate::hash::Hash;
use crate::merkle::{self, MerkleProof};
use crate::transaction::Transaction;
use crate::{AuthorityIndex, Round};
use serde::{Deserialize, Serialize};

/// Magic prefix of the VRF blob carried in [`StatementBlock::statements`].
/// The one definition; every encoder, decoder and digest check uses it.
pub const VRF_STATEMENT_MAGIC: &[u8; 9] = b"KUNA-VRF1";

/// Length of the VRF blob: magic (9) ‖ output hash (32) ‖ proof (96).
pub const VRF_STATEMENT_LEN: usize = 137;

/// Domain prefix of the VRF *input* (`KUNA-VRF1-v1:<chain_id>:<round>:<author>:0`).
pub const VRF_INPUT_DOMAIN: &str = "KUNA-VRF1-v1";

/// VRF input for `author`'s block at `round` on `chain_id`.
#[must_use]
pub fn vrf_input(chain_id: u64, round: Round, author: AuthorityIndex) -> String {
    format!("{VRF_INPUT_DOMAIN}:{chain_id}:{round}:{author}:0")
}

/// Whether `statements` carry the VRF blob (magic + at least
/// [`VRF_STATEMENT_LEN`] bytes). Only then does the block digest cover them
/// (see [`StatementBlock::compute_digest_statements`]).
#[must_use]
pub fn is_vrf_statement(statements: &[u8]) -> bool {
    statements.len() >= VRF_STATEMENT_LEN && statements[..9] == VRF_STATEMENT_MAGIC[..]
}

/// Length in bytes of the block signature preimage (`KUNA/block/v1` 16 B ‖ chain_id 8 B ‖ digest 32 B).
pub const BLOCK_SIGNATURE_DATA_LEN: usize = 56;

/// Domain tag for PoW block header preimage hashing (ADR 0003).
pub const DOMAIN_POW_HEADER: &[u8] = Hash::DOMAIN_POW_HEADER;

/// Canonical bytes signed for a block (signature format v1, 56 bytes).
///
/// Layout (PRO-20):
/// `DomainTag16("KUNA/block/v1")` (16 bytes) || `chain_id` u64 LE (8 bytes) || `digest` (32 bytes).
#[must_use]
pub fn block_signature_data(chain_id: u64, digest: &Hash) -> [u8; BLOCK_SIGNATURE_DATA_LEN] {
    let mut data = [0u8; BLOCK_SIGNATURE_DATA_LEN];
    data[..16].copy_from_slice(&crate::signing::BLOCK_DOMAIN_TAG);
    data[16..24].copy_from_slice(&chain_id.to_le_bytes());
    data[24..56].copy_from_slice(&digest.0);
    data
}

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
        // The digest covers `statements` ONLY when they carry the VRF blob
        // (`is_vrf_statement`: `KUNA-VRF1` magic and at least 137 bytes).
        // Any other statements content is outside the digest, so it is not
        // authenticated by the block signature and must never be interpreted.
        if let Some(s) = statements {
            if is_vrf_statement(s) {
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

    /// Canonical 56-byte message signed for this block on `chain_id` (signature format v1).
    #[must_use]
    pub fn signature_data(&self, chain_id: u64) -> [u8; BLOCK_SIGNATURE_DATA_LEN] {
        block_signature_data(chain_id, &self.digest)
    }
    /// Create a [`BlockHeader`] from this block with explicit PoW metadata.
    pub fn to_pow_header(
        &self,
        chain_id: u64,
        timestamp: u64,
        target_bits: u32,
        nonce: u64,
    ) -> BlockHeader {
        BlockHeader::from_statement_block(self, chain_id, timestamp, target_bits, nonce)
    }

    /// Extract a default [`BlockHeader`] from this block bound to `chain_id`.
    pub fn pow_header(&self, chain_id: u64) -> BlockHeader {
        BlockHeader::from_statement_block(self, chain_id, 0, 0, 0)
    }
}

/// Simplified Block alias used in higher layers.
pub type Block = StatementBlock;

/// Proof-of-Work block header for hybrid PoW + GHOSTDAG consensus (ADR 0003, P1/P2).
///
/// Contains consensus metadata, DAG parent references, transaction Merkle root,
/// timestamp, compact difficulty target, mining nonce, and optional PoW hash.
///
/// # Preimage format (exact byte layout)
/// Canonical preimage hashing for PoW mining and difficulty verification:
/// 1. Domain tag: `16` bytes (`POW_HEADER_DOMAIN_TAG` = `b"KVNC-POW-HDR-v1\0"`)
/// 2. `chain_id`: `8` bytes (`u64` little-endian)
/// 3. `round`: `8` bytes (`u64` little-endian DAG round / height)
/// 4. `author`: `2` bytes (`u16` little-endian authority index or miner id)
/// 5. `parents_count`: `4` bytes (`u32` little-endian count of DAG parents)
/// 6. Each parent in `parents`:
///    - `author`: `2` bytes (`u16` little-endian)
///    - `round`: `8` bytes (`u64` little-endian)
///    - `digest`: `32` bytes raw digest
/// 7. `merkle_root`: `32` bytes transaction Merkle root
/// 8. `timestamp`: `8` bytes (`u64` little-endian unix milliseconds)
/// 9. `target_bits`: `4` bytes (`u32` little-endian compact difficulty target)
/// 10. `nonce`: `8` bytes (`u64` little-endian mining nonce)
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct BlockHeader {
    /// Network identifier (`chain_id`), committing the header to its chain.
    pub chain_id: u64,
    /// Round or height of this block in the DAG.
    pub round: Round,
    /// Authority index or author identifier.
    pub author: AuthorityIndex,
    /// Parent block references in the DAG.
    pub parents: Vec<BlockReference>,
    /// Binary Merkle root over included transactions.
    pub merkle_root: Hash,
    /// Timestamp in unix milliseconds.
    pub timestamp: u64,
    /// Compact difficulty target (`nBits`).
    pub target_bits: u32,
    /// Mining nonce.
    pub nonce: u64,
    /// Cached or verified PoW hash (None until computed/verified).
    #[serde(default)]
    pub pow_hash: Option<Hash>,
}

/// Alias for [`BlockHeader`].
pub type PowHeader = BlockHeader;
/// Alias for [`BlockHeader`].
pub type HeaderV2 = BlockHeader;

/// Errors that can occur during block header PoW verification.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum HeaderError {
    /// The header does not have a PoW hash declared.
    #[error("Missing PoW hash on block header")]
    MissingPowHash,
    /// The declared PoW hash does not match the computed KovanicaHash.
    #[error("Invalid PoW hash: computed {computed}, declared {declared}")]
    InvalidPowHash {
        /// Hash computed from the header preimage.
        computed: Hash,
        /// Hash declared on the block header.
        declared: Hash,
    },
    /// The PoW hash does not satisfy the difficulty target.
    #[error("PoW target not met: hash {hash} does not satisfy target_bits {target_bits:#010x}")]
    TargetNotMet {
        /// The PoW hash that failed the check.
        hash: Hash,
        /// The compact target bits that were not satisfied.
        target_bits: u32,
    },
    /// The header timestamp is too far in the future relative to local clock.
    #[error("Timestamp drift exceeded: header timestamp {timestamp} ms, local clock {now} ms")]
    TimestampDrift {
        /// Header timestamp in unix milliseconds.
        timestamp: u64,
        /// Current wall-clock time in unix milliseconds.
        now: u64,
    },
    /// The header timestamp is not strictly greater than parent timestamps.
    #[error("Timestamp not monotonic: header timestamp {timestamp} ms <= parent timestamp {parent_timestamp} ms")]
    TimestampNotMonotonic {
        /// Header timestamp in unix milliseconds.
        timestamp: u64,
        /// Offending parent timestamp in unix milliseconds.
        parent_timestamp: u64,
    },
    /// Invalid target bits representation (e.g. zero or negative).
    #[error("Invalid compact target bits: {0:#010x}")]
    InvalidTargetBits(u32),
    /// Invalid parent references or round sequence.
    #[error("Invalid parent references")]
    InvalidParents,
}

impl Default for BlockHeader {
    fn default() -> Self {
        Self {
            chain_id: 0,
            round: 0,
            author: 0,
            parents: Vec::new(),
            merkle_root: Hash::zero(),
            timestamp: 0,
            target_bits: 0,
            nonce: 0,
            pow_hash: None,
        }
    }
}

impl BlockHeader {
    /// Create a new block header.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chain_id: u64,
        round: Round,
        author: AuthorityIndex,
        parents: Vec<BlockReference>,
        merkle_root: Hash,
        timestamp: u64,
        target_bits: u32,
        nonce: u64,
    ) -> Self {
        Self {
            chain_id,
            round,
            author,
            parents,
            merkle_root,
            timestamp,
            target_bits,
            nonce,
            pow_hash: None,
        }
    }

    /// Extract a block header from a [`StatementBlock`], binding to `chain_id`.
    pub fn from_statement_block(
        block: &StatementBlock,
        chain_id: u64,
        timestamp: u64,
        target_bits: u32,
        nonce: u64,
    ) -> Self {
        Self {
            chain_id,
            round: block.round,
            author: block.author,
            parents: block.parents.clone(),
            merkle_root: block.merkle_root,
            timestamp,
            target_bits,
            nonce,
            pow_hash: None,
        }
    }

    /// Compute the exact canonical preimage bytes for this header.
    pub fn preimage(&self) -> Vec<u8> {
        Self::compute_preimage(
            self.chain_id,
            self.round,
            self.author,
            &self.parents,
            &self.merkle_root,
            self.timestamp,
            self.target_bits,
            self.nonce,
        )
    }

    /// Compute the exact canonical preimage bytes from individual header fields.
    #[allow(clippy::too_many_arguments)]
    pub fn compute_preimage(
        chain_id: u64,
        round: Round,
        author: AuthorityIndex,
        parents: &[BlockReference],
        merkle_root: &Hash,
        timestamp: u64,
        target_bits: u32,
        nonce: u64,
    ) -> Vec<u8> {
        let mut data = Vec::with_capacity(16 + 8 + 8 + 2 + 4 + parents.len() * 42 + 32 + 8 + 4 + 8);
        data.extend_from_slice(&crate::signing::POW_HEADER_DOMAIN_TAG);
        data.extend_from_slice(&chain_id.to_le_bytes());
        data.extend_from_slice(&round.to_le_bytes());
        data.extend_from_slice(&author.to_le_bytes());
        data.extend_from_slice(&(parents.len() as u32).to_le_bytes());
        for p in parents {
            data.extend_from_slice(&p.author.to_le_bytes());
            data.extend_from_slice(&p.round.to_le_bytes());
            data.extend_from_slice(&p.digest.0);
        }
        data.extend_from_slice(&merkle_root.0);
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&target_bits.to_le_bytes());
        data.extend_from_slice(&nonce.to_le_bytes());
        data
    }

    /// Compute the preimage digest of this header (pre-PoW evaluation).
    pub fn header_preimage_digest(&self) -> Hash {
        Hash::new_keyed(DOMAIN_POW_HEADER, self.preimage())
    }

    /// Check whether this header's PoW meets its difficulty target.
    ///
    /// If `pow_hash` is present, checks it against `target_bits`. Otherwise
    /// evaluates against `header_preimage_digest()`.
    pub fn meets_target(&self) -> bool {
        let hash = self
            .pow_hash
            .unwrap_or_else(|| self.header_preimage_digest());
        check_pow_hash_meets_target(&hash, self.target_bits)
    }

    /// Compute the KovanicaHash PoW hash for this header's canonical preimage.
    #[must_use]
    pub fn compute_pow_hash(&self) -> Hash {
        crate::pow::kovanica_hash(&self.preimage())
    }

    /// Compute the KovanicaHash PoW hash using an existing [`crate::pow::KovanicaHasher`] buffer.
    pub fn compute_pow_hash_with(&self, hasher: &mut crate::pow::KovanicaHasher) -> Hash {
        hasher.hash(&self.preimage())
    }

    /// Verify this header's PoW hash and difficulty target.
    ///
    /// Validates that:
    /// 1. `pow_hash` is present.
    /// 2. `target_bits` produces a non-zero valid target.
    /// 3. Declared `pow_hash` equals `compute_pow_hash()`.
    /// 4. Declared `pow_hash` meets `target_bits`.
    pub fn verify_pow(&self) -> Result<(), HeaderError> {
        self.verify_pow_with_clock(None)
    }

    /// Verify this header's PoW hash, difficulty target, and future timestamp drift.
    pub fn verify_pow_with_clock(&self, now_ms: Option<u64>) -> Result<(), HeaderError> {
        let declared = self.pow_hash.ok_or(HeaderError::MissingPowHash)?;

        let target = compact_to_target(self.target_bits);
        if target == [0u8; 32] {
            return Err(HeaderError::InvalidTargetBits(self.target_bits));
        }

        let computed = self.compute_pow_hash();
        if declared != computed {
            return Err(HeaderError::InvalidPowHash { computed, declared });
        }

        if !check_pow_hash_meets_target(&declared, self.target_bits) {
            return Err(HeaderError::TargetNotMet {
                hash: declared,
                target_bits: self.target_bits,
            });
        }

        if let Some(now) = now_ms {
            if self.timestamp > now.saturating_add(kovanica_params::MAX_FUTURE_TIMESTAMP_DRIFT_MS) {
                return Err(HeaderError::TimestampDrift {
                    timestamp: self.timestamp,
                    now,
                });
            }
        }

        Ok(())
    }

    /// Full verification of this header against parent timestamps and current wall clock.
    pub fn verify_pow_full(
        &self,
        parent_timestamps: &[u64],
        now_ms: Option<u64>,
    ) -> Result<(), HeaderError> {
        self.verify_pow_with_clock(now_ms)?;

        for &parent_ts in parent_timestamps {
            if self.timestamp <= parent_ts {
                return Err(HeaderError::TimestampNotMonotonic {
                    timestamp: self.timestamp,
                    parent_timestamp: parent_ts,
                });
            }
        }

        Ok(())
    }

    /// Return a [`BlockReference`] corresponding to this header.
    pub fn reference(&self) -> BlockReference {
        BlockReference {
            author: self.author,
            round: self.round,
            digest: self.header_preimage_digest(),
        }
    }
}

impl From<&StatementBlock> for BlockHeader {
    fn from(block: &StatementBlock) -> Self {
        Self {
            chain_id: 0,
            round: block.round,
            author: block.author,
            parents: block.parents.clone(),
            merkle_root: block.merkle_root,
            timestamp: 0,
            target_bits: 0,
            nonce: 0,
            pow_hash: None,
        }
    }
}

/// Convert compact difficulty target representation (`nBits`) into 256-bit target bytes.
///
/// The compact format uses a 24-bit mantissa and an 8-bit exponent:
/// `target = mantissa * 256^(exponent - 3)`.
///
/// If the negative sign bit `0x0080_0000` is set, or if mantissa/exponent is 0,
/// the target is 0 (`[0u8; 32]`).
/// If the value overflows 256 bits, `[0xff; 32]` is returned.
#[must_use]
pub fn compact_to_target(bits: u32) -> [u8; 32] {
    let exponent = (bits >> 24) as usize;
    let mantissa = bits & 0x007f_ffff;

    if (bits & 0x0080_0000) != 0 || mantissa == 0 || exponent == 0 {
        return [0u8; 32];
    }

    let m_bytes = [
        ((mantissa >> 16) & 0xff) as u8,
        ((mantissa >> 8) & 0xff) as u8,
        (mantissa & 0xff) as u8,
    ];

    if exponent > 33 || (exponent == 33 && m_bytes[0] != 0) {
        return [0xff; 32];
    }

    let mut target = [0u8; 32];
    if exponent == 33 {
        target[0] = m_bytes[1];
        target[1] = m_bytes[2];
    } else if exponent <= 3 {
        let val = mantissa >> (8 * (3 - exponent));
        let bytes = val.to_be_bytes();
        target[32 - bytes.len()..].copy_from_slice(&bytes);
    } else {
        let start = 32 - exponent;
        target[start] = m_bytes[0];
        target[start + 1] = m_bytes[1];
        target[start + 2] = m_bytes[2];
    }
    target
}

/// Convert 256-bit target bytes into compact representation (`nBits`).
///
/// Returns 0 for an all-zero target.
#[must_use]
pub fn target_to_compact(target: &[u8; 32]) -> u32 {
    let mut first_non_zero = 32;
    for (i, &b) in target.iter().enumerate() {
        if b != 0 {
            first_non_zero = i;
            break;
        }
    }
    if first_non_zero == 32 {
        return 0;
    }

    let mut exponent = 32 - first_non_zero;
    let mantissa = if target[first_non_zero] >= 0x80 {
        exponent += 1;
        let b0 = 0u32;
        let b1 = target[first_non_zero] as u32;
        let b2 = if first_non_zero + 1 < 32 {
            target[first_non_zero + 1] as u32
        } else {
            0
        };
        (b0 << 16) | (b1 << 8) | b2
    } else {
        let b0 = target[first_non_zero] as u32;
        let b1 = if first_non_zero + 1 < 32 {
            target[first_non_zero + 1] as u32
        } else {
            0
        };
        let b2 = if first_non_zero + 2 < 32 {
            target[first_non_zero + 2] as u32
        } else {
            0
        };
        (b0 << 16) | (b1 << 8) | b2
    };

    ((exponent as u32) << 24) | (mantissa & 0x007f_ffff)
}

/// Check whether `hash` meets the difficulty target specified by `target_bits`.
///
/// Returns `true` if `hash <= target` as big-endian 256-bit integers and target > 0.
#[must_use]
pub fn check_pow_hash_meets_target(hash: &Hash, target_bits: u32) -> bool {
    let target = compact_to_target(target_bits);
    if target == [0u8; 32] {
        return false;
    }
    hash.0 <= target
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::Address;
    use crate::crypto::{PublicKey, Signature, SigningKey};
    use ed25519_dalek::Signer;

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
    fn block_signature_data_layout() {
        let digest = Hash([0x11; 32]);
        let data = block_signature_data(1, &digest);
        assert_eq!(data.len(), BLOCK_SIGNATURE_DATA_LEN);
        assert_eq!(data.len(), 56);
        assert_eq!(&data[..16], &crate::signing::BLOCK_DOMAIN_TAG);
        assert_eq!(&data[16..24], &1u64.to_le_bytes());
        assert_eq!(&data[24..56], &digest.0);
    }

    #[test]
    fn block_signature_data_binds_chain_id() {
        let digest = Hash([0x11; 32]);
        let mainnet_data = block_signature_data(crate::signing::chain_id::MAINNET, &digest);
        let testnet4_data = block_signature_data(crate::signing::chain_id::TESTNET_V2, &digest);
        assert_ne!(mainnet_data, testnet4_data);
    }

    #[test]
    fn block_signature_golden_vector() {
        // Standard test key: seed = 07 * 32
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let pk = PublicKey::from(sk.verifying_key());
        assert_eq!(
            hex::encode(pk.0),
            "ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c"
        );

        let digest = Hash([0x11; 32]);
        let chain_id = 1u64; // Mainnet
        let data = block_signature_data(chain_id, &digest);
        assert_eq!(
            hex::encode(data),
            "4b554e412f626c6f636b2f763100000001000000000000001111111111111111111111111111111111111111111111111111111111111111"
        );

        let sig: Signature = sk.sign(&data).into();
        let sig_hex = hex::encode(sig.0);
        // Verify strict passes on matching chain
        assert!(sig.verify_strict(&data, &pk));

        // Verify fails on chain 4 (Testnet v2)
        let data_chain4 = block_signature_data(4, &digest);
        assert!(!sig.verify_strict(&data_chain4, &pk));

        // Verify fails on raw digest
        assert!(!sig.verify_strict(digest.as_ref(), &pk));

        assert_eq!(
            sig_hex,
            "68c9290f99c396e18e2f65abb9d6b925de87a576d5fdef8a1c3e0c05dc9b23f7254deddbfd4dba99c6c3a30f30e59a9484920eb96b1ad40e6a40087228f02703"
        );
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

#[cfg(test)]
mod vrf_statement_tests {
    use super::*;

    #[test]
    fn vrf_framing_constants() {
        assert_eq!(VRF_STATEMENT_MAGIC, b"KUNA-VRF1");
        assert_eq!(VRF_STATEMENT_LEN, 9 + 32 + 96);
        assert_eq!(VRF_INPUT_DOMAIN, "KUNA-VRF1-v1");
        assert_eq!(vrf_input(4, 12, 3), "KUNA-VRF1-v1:4:12:3:0");
    }

    /// The digest covers `statements` only when they are a VRF blob
    /// (block.rs `compute_digest_statements`).
    #[test]
    fn digest_covers_statements_only_for_the_vrf_blob() {
        let mut blob = VRF_STATEMENT_MAGIC.to_vec();
        blob.resize(VRF_STATEMENT_LEN, 7);
        assert!(is_vrf_statement(&blob));
        let short = &blob[..VRF_STATEMENT_LEN - 1];
        assert!(!is_vrf_statement(short));
        let mut wrong = blob.clone();
        wrong[0] = b'X';
        assert!(!is_vrf_statement(&wrong));
        let mut legacy = b"KVNC-VRF1".to_vec();
        legacy.resize(VRF_STATEMENT_LEN, 7);
        assert!(
            !is_vrf_statement(&legacy),
            "the pre-chain-4 magic is not accepted"
        );

        let none = StatementBlock::compute_digest_statements(1, 3, &[], &[], None);
        let with_blob = StatementBlock::compute_digest_statements(1, 3, &[], &[], Some(&blob));
        let with_other = StatementBlock::compute_digest_statements(1, 3, &[], &[], Some(&wrong));
        assert_ne!(none, with_blob, "a VRF blob is covered");
        assert_eq!(none, with_other, "anything else is outside the digest");
    }
}

#[cfg(test)]
mod pow_header_tests {
    use super::*;

    #[test]
    fn block_header_default_and_new() {
        let def = BlockHeader::default();
        assert_eq!(def.chain_id, 0);
        assert_eq!(def.round, 0);
        assert_eq!(def.author, 0);
        assert!(def.parents.is_empty());
        assert_eq!(def.merkle_root, Hash::zero());
        assert_eq!(def.timestamp, 0);
        assert_eq!(def.target_bits, 0);
        assert_eq!(def.nonce, 0);
        assert!(def.pow_hash.is_none());

        let p1 = BlockReference {
            author: 1,
            round: 0,
            digest: Hash([0xaa; 32]),
        };
        let header = BlockHeader::new(
            5,
            10,
            2,
            vec![p1],
            Hash([0xbb; 32]),
            1_700_000_000_000,
            0x1d00ffff,
            1234567,
        );
        assert_eq!(header.chain_id, 5);
        assert_eq!(header.round, 10);
        assert_eq!(header.author, 2);
        assert_eq!(header.parents.len(), 1);
        assert_eq!(header.timestamp, 1_700_000_000_000);
        assert_eq!(header.target_bits, 0x1d00ffff);
        assert_eq!(header.nonce, 1234567);
    }

    #[test]
    fn block_header_conversion_from_statement_block() {
        let block = StatementBlock {
            author: 3,
            round: 42,
            parents: vec![BlockReference {
                author: 0,
                round: 41,
                digest: Hash([0x12; 32]),
            }],
            transactions: vec![],
            statements: vec![],
            signature: Signature([0; 64]),
            digest: Hash([0x34; 32]),
            merkle_root: Hash([0x56; 32]),
        };

        let h1: BlockHeader = (&block).into();
        assert_eq!(h1.round, 42);
        assert_eq!(h1.author, 3);
        assert_eq!(h1.merkle_root, Hash([0x56; 32]));
        assert_eq!(h1.chain_id, 0);
        assert_eq!(h1.nonce, 0);

        let h2 = block.to_pow_header(5, 1_000_000, 0x1d00ffff, 999);
        assert_eq!(h2.chain_id, 5);
        assert_eq!(h2.timestamp, 1_000_000);
        assert_eq!(h2.target_bits, 0x1d00ffff);
        assert_eq!(h2.nonce, 999);
        assert_eq!(h2.round, 42);

        let h3 = block.pow_header(5);
        assert_eq!(h3.chain_id, 5);
        assert_eq!(h3.nonce, 0);
    }

    #[test]
    fn preimage_layout_exact_bytes() {
        let p1 = BlockReference {
            author: 7,
            round: 9,
            digest: Hash([0x77; 32]),
        };
        let header = BlockHeader::new(
            5,
            10,
            2,
            vec![p1],
            Hash([0xbb; 32]),
            123_456_789,
            0x1d00ffff,
            42,
        );

        let preimage = header.preimage();
        // 16 (tag) + 8 (chain_id) + 8 (round) + 2 (author) + 4 (parents_count)
        // + 1 * (2 + 8 + 32) (parents) + 32 (merkle) + 8 (time) + 4 (bits) + 8 (nonce) = 130
        let expected_len = 16 + 8 + 8 + 2 + 4 + 42 + 32 + 8 + 4 + 8;
        assert_eq!(preimage.len(), expected_len);
        assert_eq!(expected_len, 132);

        // Check fields at exact offsets:
        assert_eq!(&preimage[..16], &crate::signing::POW_HEADER_DOMAIN_TAG);
        assert_eq!(&preimage[16..24], &5u64.to_le_bytes());
        assert_eq!(&preimage[24..32], &10u64.to_le_bytes());
        assert_eq!(&preimage[32..34], &2u16.to_le_bytes());
        assert_eq!(&preimage[34..38], &1u32.to_le_bytes());
        assert_eq!(&preimage[38..40], &7u16.to_le_bytes());
        assert_eq!(&preimage[40..48], &9u64.to_le_bytes());
        assert_eq!(&preimage[48..80], &[0x77; 32]);
        assert_eq!(&preimage[80..112], &[0xbb; 32]);
        assert_eq!(&preimage[112..120], &123_456_789u64.to_le_bytes());
        assert_eq!(&preimage[120..124], &0x1d00ffffu32.to_le_bytes());
        assert_eq!(&preimage[124..132], &42u64.to_le_bytes());
    }

    #[test]
    fn preimage_digest_sensitivity() {
        let h1 = BlockHeader::new(5, 1, 0, vec![], Hash::zero(), 1000, 0x1d00ffff, 1);
        let mut h_nonce = h1.clone();
        h_nonce.nonce = 2;
        assert_ne!(
            h1.header_preimage_digest(),
            h_nonce.header_preimage_digest()
        );

        let mut h_time = h1.clone();
        h_time.timestamp = 1001;
        assert_ne!(h1.header_preimage_digest(), h_time.header_preimage_digest());

        let mut h_chain = h1.clone();
        h_chain.chain_id = 1;
        assert_ne!(
            h1.header_preimage_digest(),
            h_chain.header_preimage_digest()
        );

        let mut h_target = h1.clone();
        h_target.target_bits = 0x1d00fffe;
        assert_ne!(
            h1.header_preimage_digest(),
            h_target.header_preimage_digest()
        );
    }

    #[test]
    fn compact_target_conversions_and_bounds() {
        // Zero target bits -> zero target bytes
        assert_eq!(compact_to_target(0), [0u8; 32]);
        assert_eq!(target_to_compact(&[0u8; 32]), 0);

        // Negative sign bit set (0x00800000) -> 0
        assert_eq!(compact_to_target(0x1d800000), [0u8; 32]);
        assert_eq!(compact_to_target(0x01800000), [0u8; 32]);

        // Standard difficulty 1 (0x1d00ffff)
        let t1 = compact_to_target(0x1d00ffff);
        // Exponent 29 -> 32 - 29 = 3 leading zero bytes
        assert_eq!(&t1[..3], &[0, 0, 0]);
        assert_eq!(t1[3], 0x00);
        assert_eq!(t1[4], 0xff);
        assert_eq!(t1[5], 0xff);
        assert!(t1[6..].iter().all(|&b| b == 0));
        assert_eq!(target_to_compact(&t1), 0x1d00ffff);

        // Exponent 1, 2, 3
        let t_exp3 = compact_to_target(0x03123456);
        assert_eq!(target_to_compact(&t_exp3), 0x03123456);

        // Value with high bit set at byte boundary (needs extra byte for sign bit 0)
        let mut high_byte_target = [0u8; 32];
        high_byte_target[0] = 0x80;
        let compact_high = target_to_compact(&high_byte_target);
        assert_eq!(compact_high, 0x21008000);
        let back = compact_to_target(compact_high);
        assert_eq!(back, high_byte_target);

        // Overflow bounds: exponent > 33
        assert_eq!(compact_to_target(0x22010000), [0xff; 32]);
        assert_eq!(compact_to_target(0xff000001), [0xff; 32]);
    }

    #[test]
    fn pow_hash_meets_target() {
        let target_bits = 0x1d00ffff;
        let target = compact_to_target(target_bits);

        // Equal to target -> meets
        let hash_eq = Hash(target);
        assert!(check_pow_hash_meets_target(&hash_eq, target_bits));

        // Strictly smaller -> meets
        let mut smaller = target;
        smaller[4] = 0xfe;
        assert!(check_pow_hash_meets_target(&Hash(smaller), target_bits));

        // Strictly greater -> fails
        let mut greater = target;
        greater[2] = 0x01;
        assert!(!check_pow_hash_meets_target(&Hash(greater), target_bits));

        // Zero target -> always fails
        assert!(!check_pow_hash_meets_target(&Hash::zero(), 0));
        assert!(!check_pow_hash_meets_target(&Hash::zero(), 0x00800000));

        // Header meets_target helper
        let mut header = BlockHeader {
            target_bits,
            pow_hash: Some(Hash(smaller)),
            ..Default::default()
        };
        assert!(header.meets_target());

        header.pow_hash = Some(Hash(greater));
        assert!(!header.meets_target());
    }

    #[test]
    fn block_header_compute_and_verify_pow() {
        let mut header = BlockHeader::new(
            5,
            1,
            0,
            vec![],
            Hash::zero(),
            1_700_000_000_000,
            0x207fffff, // Easiest target: top byte <= 0x7f (~50% pass rate)
            0,
        );

        // Missing pow_hash fails
        assert_eq!(header.verify_pow(), Err(HeaderError::MissingPowHash));

        // Compute real pow hash
        let pow_hash = header.compute_pow_hash();
        assert_eq!(pow_hash, crate::pow::kovanica_hash(&header.preimage()));

        // Invalid declared pow_hash fails
        header.pow_hash = Some(Hash([0xaa; 32]));
        assert!(matches!(
            header.verify_pow(),
            Err(HeaderError::InvalidPowHash { .. })
        ));

        // Set correct pow_hash
        header.pow_hash = Some(pow_hash);

        // If target is too strict, fails TargetNotMet
        header.target_bits = 0x04000001; // extremely strict valid target
        let strict_hash = header.compute_pow_hash();
        header.pow_hash = Some(strict_hash);
        assert_eq!(
            header.verify_pow(),
            Err(HeaderError::TargetNotMet {
                hash: strict_hash,
                target_bits: 0x04000001,
            })
        );

        // Reset target_bits to maximum possible target ([0xff; 32]) and recompute hash
        header.target_bits = 0x22010000;
        header.pow_hash = Some(header.compute_pow_hash());

        // Invalid target bits (zero)
        let mut zero_target_header = header.clone();
        zero_target_header.target_bits = 0;
        zero_target_header.pow_hash = Some(zero_target_header.compute_pow_hash());
        assert_eq!(
            zero_target_header.verify_pow(),
            Err(HeaderError::InvalidTargetBits(0))
        );

        // Valid header passes
        assert!(header.verify_pow().is_ok());

        // Timestamp drift check
        let now = 1_700_000_000_000;
        assert!(header.verify_pow_with_clock(Some(now)).is_ok());
        // In past is ok
        assert!(header.verify_pow_with_clock(Some(now + 100_000)).is_ok());
        // Far in future (> 30s) fails
        let drifted = header.verify_pow_with_clock(Some(now - 35_000));
        assert!(matches!(drifted, Err(HeaderError::TimestampDrift { .. })));

        // Parent timestamp monotonicity check
        let parent_ts = 1_699_999_999_000;
        assert!(header
            .verify_pow_full(&[parent_ts], Some(now + 1_000))
            .is_ok());

        // Header timestamp <= parent timestamp fails
        let non_monotonic = header.verify_pow_full(&[1_700_000_000_000], Some(now + 1_000));
        assert!(matches!(
            non_monotonic,
            Err(HeaderError::TimestampNotMonotonic { .. })
        ));
    }
}
