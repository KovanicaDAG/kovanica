use crate::error::WalletError;
use async_trait::async_trait;
use kovanica_types::Address;

#[async_trait]
pub trait BalanceProvider: Send + Sync {
    async fn get_balance(&self, address: &Address) -> Result<u64, WalletError>;
    async fn get_nonce(&self, address: &Address) -> Result<u64, WalletError>;
}
