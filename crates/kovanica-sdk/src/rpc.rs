//! Typed HTTP JSON-RPC client.

#[cfg(feature = "http")]
use crate::error::SdkError;
#[cfg(feature = "http")]
use reqwest::Client;
#[cfg(feature = "http")]
use serde::de::DeserializeOwned;
#[cfg(feature = "http")]
use serde_json::{json, Value};

#[cfg(feature = "http")]
#[derive(Clone)]
pub struct RpcClient {
    url: String,
    client: Client,
}

#[cfg(feature = "http")]
impl RpcClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: Client::new(),
        }
    }

    pub async fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, SdkError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let resp = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .map_err(|e| SdkError::Rpc(e.to_string()))?;
        let v: Value = resp
            .json()
            .await
            .map_err(|e| SdkError::Rpc(e.to_string()))?;
        if let Some(err) = v.get("error") {
            return Err(SdkError::Rpc(err.to_string()));
        }
        let result = v.get("result").cloned().unwrap_or(Value::Null);
        serde_json::from_value(result).map_err(|e| SdkError::Serialization(e.to_string()))
    }

    pub async fn block_number(&self) -> Result<u64, SdkError> {
        // returns hex quantity string possibly
        let raw: Value = self.call("kovanica_blockNumber", json!([])).await?;
        parse_u64_value(&raw)
    }
}

#[cfg(feature = "http")]
fn parse_u64_value(v: &Value) -> Result<u64, SdkError> {
    if let Some(s) = v.as_str() {
        let s = s.trim_start_matches("0x");
        u64::from_str_radix(s, 16).map_err(|e| SdkError::Serialization(e.to_string()))
    } else if let Some(n) = v.as_u64() {
        Ok(n)
    } else {
        Err(SdkError::Serialization("expected number".into()))
    }
}
