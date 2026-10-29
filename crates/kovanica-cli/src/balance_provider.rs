//! RPC-backed [`BalanceProvider`] for `kovanica-wallet-core`.

use crate::rpc::{parse_quantity, RpcClient};
use async_trait::async_trait;
use kovanica_types::Address;
use kovanica_wallet_core::error::WalletError;
use kovanica_wallet_core::provider::BalanceProvider;
use serde_json::json;

pub struct RpcBalanceProvider {
    client: RpcClient,
}

impl RpcBalanceProvider {
    pub fn new(client: RpcClient) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &RpcClient {
        &self.client
    }
}

#[async_trait]
impl BalanceProvider for RpcBalanceProvider {
    async fn get_balance(&self, address: &Address) -> Result<u64, WalletError> {
        let value = self
            .client
            .call("kovanica_getBalance", json!([address.to_string()]))
            .await
            .map_err(|e| WalletError::Provider(e.to_string()))?;
        parse_quantity(&value).map_err(|e| WalletError::Provider(e.to_string()))
    }

    async fn get_nonce(&self, address: &Address) -> Result<u64, WalletError> {
        let value = self
            .client
            .call("kovanica_getNonce", json!([address.to_string()]))
            .await
            .map_err(|e| WalletError::Provider(e.to_string()))?;
        parse_quantity(&value).map_err(|e| WalletError::Provider(e.to_string()))
    }
}
