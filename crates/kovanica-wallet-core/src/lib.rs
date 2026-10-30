//! Kovanica wallet core.

#![deny(unsafe_code)]

pub mod account;
pub mod error;
pub mod provider;

pub use account::Wallet;
pub use account::WalletAccount;
pub use error::WalletError;
pub use provider::BalanceProvider;
#[cfg(feature = "hw-signer")]
pub mod hw_signer {
    pub use kovanica_hw_signer::*;
}
