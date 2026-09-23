//! Transaction builders for Kovanica.
//!
//! Builders produce unsigned [`Transaction`] values that are then signed with a
//! [`Keypair`]. The canonical wire format + sighash live in `kovanica-types`
//! (byte-identical to `kovanica-state` `tx.rs`); signatures go into the input
//! witness stack exactly like the node's `TxInput::single_sig`.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

use kovanica_keys::Keypair;
use kovanica_types::{Address, Amount, AssetId, NetworkId, Transaction, TxInput, TxOutput, Utxo};

/// Errors from transaction construction / signing.
#[derive(Debug, thiserror::Error)]
pub enum TxError {
    /// Missing required field.
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    /// No inputs provided.
    #[error("transaction has no inputs")]
    NoInputs,
    /// No outputs provided.
    #[error("transaction has no outputs")]
    NoOutputs,
    /// Value conservation / fee inconsistency (best-effort check).
    #[error("value mismatch: inputs {inputs} atoms, outputs+fee {outputs} atoms")]
    ValueMismatch {
        /// Sum of input values in atoms.
        inputs: u64,
        /// Sum of outputs + fee in atoms.
        outputs: u64,
    },
    /// Signing failed.
    #[error("signing failed: {0}")]
    Signing(String),
}

/// Builder for a simple native (or single-asset) transfer.
#[derive(Debug, Default)]
pub struct TransferBuilder {
    network: Option<NetworkId>,
    inputs: Vec<Utxo>,
    outputs: Vec<TxOutput>,
    fee: Option<Amount>,
    change_address: Option<Address>,
    tag: Vec<u8>,
    n_lock_time: u32,
    sequence: u32,
}

impl TransferBuilder {
    /// New empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Target network (client-side label; never serialised).
    pub fn network(mut self, network: NetworkId) -> Self {
        self.network = Some(network);
        self
    }

    /// Add a spendable UTXO.
    pub fn add_input(mut self, utxo: Utxo) -> Self {
        self.inputs.push(utxo);
        self
    }

    /// Add a recipient output.
    pub fn add_output(mut self, address: Address, amount: Amount, asset_id: AssetId) -> Self {
        let asset = (!asset_id.is_native()).then_some(asset_id);
        self.outputs
            .push(TxOutput::new(amount.atoms(), asset, address));
        self
    }

    /// Convenience: native KVNC output.
    pub fn add_native_output(self, address: Address, amount: Amount) -> Self {
        self.add_output(address, amount, AssetId::NATIVE)
    }

    /// Explicit fee (atoms). If omitted, caller should compute via `kovanica-fee`.
    pub fn set_fee(mut self, fee: Amount) -> Self {
        self.fee = Some(fee);
        self
    }

    /// Change address for leftover value after outputs + fee.
    pub fn set_change(mut self, address: Address) -> Self {
        self.change_address = Some(address);
        self
    }

    /// Extra committed bytes carried in the transaction tag (default: empty).
    pub fn tag(mut self, tag: impl Into<Vec<u8>>) -> Self {
        self.tag = tag.into();
        self
    }

    /// Lock time (BIP-65 CLTV); bound into the sighash.
    pub fn n_lock_time(mut self, value: u32) -> Self {
        self.n_lock_time = value;
        self
    }

    /// Sequence (BIP-112 CSV); bound into the sighash.
    pub fn sequence(mut self, value: u32) -> Self {
        self.sequence = value;
        self
    }

    /// Build an unsigned transaction.
    ///
    /// Performs basic value conservation if fee and change are set.
    pub fn build(self) -> Result<Transaction, TxError> {
        let network = self.network.ok_or(TxError::MissingField("network"))?;
        if self.inputs.is_empty() {
            return Err(TxError::NoInputs);
        }
        if self.outputs.is_empty() && self.change_address.is_none() {
            return Err(TxError::NoOutputs);
        }

        let mut outputs = self.outputs;
        let fee = self.fee.unwrap_or(Amount::ZERO);

        // Best-effort change calculation for native asset only.
        let native_in: u64 = self
            .inputs
            .iter()
            .filter(|u| u.asset_id.is_native())
            .map(|u| u.amount.atoms())
            .sum();
        let native_out: u64 = outputs
            .iter()
            .filter(|o| o.asset_id.map(|a| a.is_native()).unwrap_or(true))
            .map(|o| o.value)
            .sum::<u64>()
            .saturating_add(fee.atoms());

        if let Some(change_addr) = self.change_address {
            if native_in > native_out {
                let change = native_in - native_out;
                if change > 0 {
                    outputs.push(TxOutput::native(change, change_addr));
                }
            } else if native_in < native_out {
                return Err(TxError::ValueMismatch {
                    inputs: native_in,
                    outputs: native_out,
                });
            }
        }

        let inputs: Vec<TxInput> = self
            .inputs
            .into_iter()
            .map(|u| TxInput::fresh(u.tx_hash, u.vout))
            .collect();

        let mut tx = Transaction::new(network, inputs, outputs, self.tag);
        tx.n_lock_time = self.n_lock_time;
        tx.sequence = self.sequence;
        Ok(tx)
    }
}

/// Signed transaction ready for broadcast.
#[derive(Debug, Clone)]
pub struct SignedTx {
    /// Underlying transaction (signatures filled into the witness stack).
    pub tx: Transaction,
}

impl SignedTx {
    /// Sign all inputs with the same keypair (single-signer / P2PK case).
    ///
    /// Per node (`tx.rs`): Ed25519 is computed over the **32-byte sighash**,
    /// and the 64-byte signature becomes the input's single witness item
    /// (`TxInput::single_sig`). The node verifies with `verify_strict`.
    pub fn sign(mut tx: Transaction, keypair: &Keypair) -> Result<Self, TxError> {
        let hash = tx.sighash();
        let signature = keypair.sign(&hash);
        for input in &mut tx.inputs {
            input.witness = vec![signature.0.to_vec()];
        }
        Ok(SignedTx { tx })
    }

    /// Canonical hex form for `POST /api/submit_tx` (`{"tx_hex": ...}`).
    pub fn tx_hex(&self) -> String {
        self.tx.encode_hex()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_address(byte: u8) -> Address {
        Address::p2pk([byte; 32])
    }

    fn dummy_utxo(atoms: u64) -> Utxo {
        Utxo {
            tx_hash: kovanica_types::TxHash::ZERO,
            vout: 0,
            amount: Amount::from_atoms(atoms),
            asset_id: AssetId::NATIVE,
            address: test_address(0x11),
        }
    }

    #[test]
    fn simple_transfer_builds() {
        let tx = TransferBuilder::new()
            .network(NetworkId::Testnet)
            .add_input(dummy_utxo(1_000_000_000))
            .add_native_output(test_address(0xAA), Amount::from_kvnc(5))
            .set_fee(Amount::from_atoms(10_000))
            .set_change(test_address(0xBB))
            .build()
            .unwrap();
        assert_eq!(tx.network, NetworkId::Testnet);
        assert_eq!(tx.inputs.len(), 1);
        assert!(tx.outputs.len() >= 2);
        // Change output is native.
        assert!(tx.outputs.last().unwrap().asset_id.is_none());
    }

    #[test]
    fn signing_fills_witness_and_verifies_at_node_like_level() {
        let mnemonic =
            kovanica_keys::Mnemonic::generate(kovanica_keys::WordCount::Words12).unwrap();
        let kp = kovanica_keys::Keypair::from_mnemonic(&mnemonic, "");
        let from = kp.address();
        let tx = TransferBuilder::new()
            .network(NetworkId::Testnet)
            .add_input(Utxo {
                tx_hash: kovanica_types::TxHash::ZERO,
                vout: 3,
                amount: Amount::from_atoms(1_000_000_000),
                asset_id: AssetId::NATIVE,
                address: from,
            })
            .add_native_output(test_address(0xAA), Amount::from_atoms(900_000_000))
            .set_fee(Amount::from_atoms(1_000))
            .set_change(from)
            .build()
            .unwrap();
        let signed = SignedTx::sign(tx, &kp).unwrap();
        // Each input witness holds exactly one 64-byte signature.
        for input in &signed.tx.inputs {
            assert_eq!(input.witness.len(), 1);
            assert_eq!(input.witness[0].len(), 64);
            // The signature verifies strictly against the sighash.
            let sig = kovanica_types::Signature(
                input.witness[0].as_slice().try_into().expect("64 bytes"),
            );
            kp.verify(&signed.tx.sighash(), &sig).unwrap();
        }
        assert!(signed.tx.sighash_hex().len() == 64);
        assert!(signed.tx_hex().len() > 64);
    }

    #[test]
    fn rejects_empty_inputs() {
        let err = TransferBuilder::new()
            .network(NetworkId::Testnet)
            .add_native_output(test_address(0xAA), Amount::from_kvnc(1))
            .build()
            .unwrap_err();
        assert!(matches!(err, TxError::NoInputs));
    }
}
