//! Minimal async JSON-RPC 2.0 client for a Kovanica node.
//!
//! Only the surface the CLI needs is implemented: a single `call` that posts a
//! JSON-RPC request to `/rpc` and unwraps the `result`, translating an `error`
//! object into an [`anyhow::Error`].

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

pub use kovanica_sdk::RpcClient as SdkRpcClient;

/// Thin adapter over `kovanica_sdk::RpcClient`.
///
/// Retains the CLI's `/rpc` path normalization and helper methods while
/// delegating all HTTP/JSON-RPC transport to the SDK.
#[derive(Clone)]
pub struct RpcClient {
    inner: kovanica_sdk::RpcClient,
    url: String,
}

impl RpcClient {
    /// Create a client for the given RPC endpoint URL (e.g.
    /// `http://127.0.0.1:8545`). The `/rpc` path is appended when missing.
    pub fn new(url: impl Into<String>) -> Self {
        let mut url = url.into();
        if !url.ends_with("/rpc") {
            url = format!("{}/rpc", url.trim_end_matches('/'));
        }
        let inner = kovanica_sdk::RpcClient::new(url.clone());
        Self { url, inner }
    }

    /// The endpoint this client posts to.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Invoke `method` with `params` and return the JSON `result` value.
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        self.inner
            .call(method, params)
            .await
            .map_err(|e| anyhow::anyhow!("RPC `{method}` failed: {e}"))
    }

    /// Read the node's `/health` peer count.
    pub async fn health_peer_count(&self) -> Option<usize> {
        self.inner.health_peer_count().await
    }
}

/// Parse the peer count from a successful node `/health` response.
///
/// A count is only trusted when the response has the expected `status: "ok"`
/// shape and an unsigned integer `peer_count` field. In particular, zero is a
/// valid count; absent or malformed data is not treated as zero.
pub fn parse_health_peer_count(value: &Value) -> Option<usize> {
    if value.get("status")?.as_str()? != "ok" {
        return None;
    }
    usize::try_from(value.get("peer_count")?.as_u64()?).ok()
}

/// Parse a JSON-RPC quantity (decimal number or `0x`-prefixed hex string).
pub fn parse_quantity(value: &Value) -> Result<u64> {
    if let Some(n) = value.as_u64() {
        return Ok(n);
    }
    let raw = value
        .as_str()
        .ok_or_else(|| anyhow!("expected an integer quantity, got {value}"))?;
    let raw = raw.strip_prefix("0x").unwrap_or(raw);
    u64::from_str_radix(raw, 16).with_context(|| format!("invalid quantity `{value}`"))
}

/// Encode a 32-byte hex string as a JSON array of bytes.
///
/// The contract RPC handlers deserialize `[u8; 32]` via serde, which maps to a
/// JSON array of numbers rather than a hex string.
pub fn bytes32_json(hex_str: &str, what: &str) -> Result<Value> {
    let raw = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(raw).with_context(|| format!("{what}: invalid hex"))?;
    if bytes.len() != 32 {
        return Err(anyhow!(
            "{what}: expected 32 bytes (64 hex chars), got {} bytes",
            bytes.len()
        ));
    }
    Ok(Value::Array(bytes.into_iter().map(|b| json!(b)).collect()))
}

/// Encode an arbitrary byte slice as a JSON array of numbers.
pub fn bytes_json(bytes: &[u8]) -> Value {
    Value::Array(bytes.iter().map(|b| json!(b)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    /// Mirrors the contract RPC parameter shapes (serde `[u8; 32]` -> JSON array).
    #[derive(Deserialize)]
    struct Mirrors {
        addr: [u8; 32],
        amount: u128,
        blob: Vec<u8>,
    }

    #[test]
    fn contract_params_match_serde_shapes() {
        let params = json!({
            "addr": bytes32_json(&"ab".repeat(32), "addr").unwrap(),
            "amount": 1000u64,
            "blob": bytes_json(&[1, 2, 3]),
        });
        let mirrored: Mirrors = serde_json::from_value(params).unwrap();
        assert_eq!(mirrored.addr, [0xab; 32]);
        assert_eq!(mirrored.amount, 1000);
        assert_eq!(mirrored.blob, vec![1, 2, 3]);
    }

    #[test]
    fn bytes32_json_validates_length_and_hex() {
        assert!(bytes32_json("not-hex", "x").is_err());
        assert!(bytes32_json("0x1234", "x").is_err());
        assert!(bytes32_json(&"cd".repeat(32), "x").is_ok());
    }

    #[test]
    fn parse_quantity_accepts_decimal_and_hex() {
        assert_eq!(parse_quantity(&json!(42)).unwrap(), 42);
        assert_eq!(parse_quantity(&json!("0x2a")).unwrap(), 42);
        assert!(parse_quantity(&json!("nope")).is_err());
    }

    #[test]
    fn health_peer_count_accepts_zero_and_nonzero() {
        assert_eq!(
            parse_health_peer_count(&json!({"status":"ok", "peer_count":0})),
            Some(0)
        );
        assert_eq!(
            parse_health_peer_count(&json!({"status":"ok", "peer_count":3})),
            Some(3)
        );
    }

    #[test]
    fn health_peer_count_rejects_missing_or_malformed_data() {
        assert_eq!(parse_health_peer_count(&json!({"status":"ok"})), None);
        assert_eq!(
            parse_health_peer_count(&json!({"status":"ok", "peer_count":"3"})),
            None
        );
        assert_eq!(
            parse_health_peer_count(&json!({"status":"error", "peer_count":3})),
            None
        );
        assert_eq!(parse_health_peer_count(&json!({"peer_count":3})), None);
    }
}
