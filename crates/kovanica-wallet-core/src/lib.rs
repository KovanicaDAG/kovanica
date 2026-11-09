//! Kovanica wallet core.

#![deny(unsafe_code)]

pub mod account;
pub mod error;
pub mod hd;
pub mod keystore_vault;
pub mod provider;
pub mod tx_preview;

pub use account::Wallet;
pub use account::WalletAccount;
pub use error::WalletError;
pub use hd::{Slip10Node, BIP44_PURPOSE, HARDENED_OFFSET, KOVANICA_COIN_TYPE};
pub use keystore_vault::{VaultPayload, VAULT_ENVELOPE_MAGIC};
pub use provider::BalanceProvider;
pub use tx_preview::TxPreview;
#[cfg(feature = "hw-signer")]
pub mod hw_signer {
    pub use kovanica_hw_signer::*;
}
