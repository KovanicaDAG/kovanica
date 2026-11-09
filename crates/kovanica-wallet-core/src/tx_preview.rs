#![deny(unsafe_code)]

//! Transparent transaction preview and sighash binding.
//!
//! Enforces that the caller (webview or mobile UI) submits human-interpretable fields,
//! and the Rust core deterministically constructs the v1 preimage, computes the
//! keyed BLAKE3 hash, and signs it.

use kovanica_types::signing::TX_DOMAIN_TAG;
use kovanica_types::transaction::{Transaction, TransactionKind};
use kovanica_types::{Address, Hash, PublicKey, SigningKey};
use serde::{Deserialize, Serialize};

use crate::error::WalletError;

/// Human-readable transaction preview strictly bound to the signing preimage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxPreview {
    /// Sender address (canonical PRO-21).
    pub sender: Address,
    /// Recipient address (canonical PRO-21).
    pub to: Address,
    /// Transfer amount in atoms (1 KVNC = 10^9 atoms).
    pub amount: u64,
    /// Gas / network fee in atoms.
    pub fee: u64,
    /// Sequential account nonce.
    pub nonce: u64,
    /// Target network chain ID.
    pub chain_id: u64,
}

impl TxPreview {
    /// Construct a new transfer preview.
    pub fn new(
        sender: Address,
        to: Address,
        amount: u64,
        fee: u64,
        nonce: u64,
        chain_id: u64,
    ) -> Self {
        Self {
            sender,
            to,
            amount,
            fee,
            nonce,
            chain_id,
        }
    }

    /// Formatted KVNC display string with 9 decimal places (e.g. "1.500000000 KVNC").
    pub fn display_amount_kvnc(&self) -> String {
        format_atoms_kvnc(self.amount)
    }

    /// Formatted fee string in KVNC.
    pub fn display_fee_kvnc(&self) -> String {
        format_atoms_kvnc(self.fee)
    }

    /// Construct the deterministic v1 little-endian signing preimage.
    ///
    /// Layout:
    /// - `DomainTag16("KUNA/tx/v1")` (16 bytes)
    /// - `chain_id` (8 bytes LE)
    /// - `sender` pubkey (32 bytes)
    /// - `nonce` (8 bytes LE)
    /// - `kind`: Transfer tag `0` (1 byte), `to` (32 bytes), `amount` (8 bytes LE)
    /// - `fee` (8 bytes LE)
    pub fn signing_preimage(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(16 + 8 + 32 + 8 + 1 + 32 + 8 + 8);
        data.extend_from_slice(&TX_DOMAIN_TAG);
        data.extend_from_slice(&self.chain_id.to_le_bytes());
        data.extend_from_slice(&self.sender.0);
        data.extend_from_slice(&self.nonce.to_le_bytes());
        // Transfer kind: signing tag 0
        data.push(0);
        data.extend_from_slice(&self.to.0);
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&self.fee.to_le_bytes());
        data
    }

    /// Compute the 32-byte signing hash using BLAKE3 keyed with `pad32("KUNA-TX-v1")`.
    pub fn signing_hash(&self) -> Hash {
        Hash::new_keyed(Hash::DOMAIN_TX, self.signing_preimage())
    }

    /// Sign the preview with an Ed25519 signing key, asserting that the key matches the sender.
    /// Returns the verified [`Transaction`] and the raw broadcast hex string (lowercase, no `0x`).
    pub fn sign(&self, signing_key: &SigningKey) -> Result<(Transaction, String), WalletError> {
        let derived_pk = PublicKey::from_bytes(signing_key.verifying_key().to_bytes());
        if derived_pk.0 != self.sender.0 {
            return Err(WalletError::Signing(format!(
                "signing key public key {} does not match preview sender {}",
                hex::encode(derived_pk.as_bytes()),
                self.sender
            )));
        }

        let sighash = self.signing_hash();
        let signature = kovanica_crypto::sign(signing_key, &sighash.0);

        let tx = Transaction {
            sender: self.sender,
            nonce: self.nonce,
            kind: TransactionKind::Transfer {
                to: self.to,
                amount: self.amount,
            },
            fee: self.fee,
            signature,
            hash: sighash,
        };

        // Wire broadcast encoding matching packages/signer/src/tx.ts
        let raw_hex = encode_broadcast_tx(&tx, &sighash);
        Ok((tx, raw_hex))
    }
}

/// Format atoms into decimal KVNC string without floating-point math.
pub fn format_atoms_kvnc(atoms: u64) -> String {
    let whole = atoms / 1_000_000_000;
    let fraction = atoms % 1_000_000_000;
    format!("{}.{:09} KVNC", whole, fraction)
}

/// Encode transaction for broadcast over RPC kovanica_sendRawTransaction.
/// Matches the binary wire layout in packages/signer/src/tx.ts:
/// - sender: 32 bytes
/// - nonce: 8 bytes LE
/// - wire kind: Transfer tag 0, recipient 32 bytes, amount 8 bytes LE
/// - fee: 8 bytes LE
/// - signature length: 8 bytes LE (value = 64)
/// - signature: 64 bytes
/// - hash: 32 bytes
pub fn encode_broadcast_tx(tx: &Transaction, hash: &Hash) -> String {
    let mut buf = Vec::with_capacity(32 + 8 + 1 + 32 + 8 + 8 + 8 + 64 + 32);
    buf.extend_from_slice(&tx.sender.0);
    buf.extend_from_slice(&tx.nonce.to_le_bytes());

    match &tx.kind {
        TransactionKind::Transfer { to, amount } => {
            buf.push(0); // Wire tag for Transfer is 0
            buf.extend_from_slice(&to.0);
            buf.extend_from_slice(&amount.to_le_bytes());
        }
        _ => {
            // For other kinds in future expansions
            unimplemented!("only Transfer wire encoding in Phase 1");
        }
    }

    buf.extend_from_slice(&tx.fee.to_le_bytes());
    // Signature vector: u64 length prefix followed by 64 bytes
    buf.extend_from_slice(&64u64.to_le_bytes());
    buf.extend_from_slice(&tx.signature.0);
    buf.extend_from_slice(&hash.0);

    hex::encode(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_atoms_tests() {
        assert_eq!(format_atoms_kvnc(1_000_000_000), "1.000000000 KVNC");
        assert_eq!(format_atoms_kvnc(500), "0.000000500 KVNC");
        assert_eq!(format_atoms_kvnc(0), "0.000000000 KVNC");
    }

    #[test]
    fn golden_vector_t1_acceptance() {
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
        assert!(!raw_hex.starts_with("0x"));
    }
}
