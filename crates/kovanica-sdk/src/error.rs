use thiserror::Error;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("invalid address: {0}")]
    InvalidAddress(String),
    #[error("signing failed: {0}")]
    Signing(String),
    #[error("rpc error: {0}")]
    Rpc(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("invalid params: {0}")]
    InvalidParams(String),
}
