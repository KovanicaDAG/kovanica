use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("account not found: {0}")]
    AccountNotFound(String),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("signing error: {0}")]
    Signing(String),
    #[error("keystore error: {0}")]
    Keystore(String),
    #[error("sdk error: {0}")]
    Sdk(String),
}
