//! Thin HTTP client for the Kovanica explorer JSON API.
//!
//! Routes and shapes mirror `kovanica-node`'s `explorer.rs`:
//!   * `GET  /api/head`              — chain head summary
//!   * `GET  /api/p2p`               — p2p listen/peers/bootstrap
//!   * `GET  /api/bootstrap`         — network parameters
//!   * `GET  /api/state`             — full snapshot (includes the block DAG)
//!   * `GET  /api/utxos?address=…`   — balance + unspent outputs
//!   * `POST /api/prepare?from&to&amount`  — returns the sighash to sign
//!   * `POST /api/submit?from&to&amount&sig` — broadcasts the signed transfer
//!
//! Note: `/api/blocks` returns a binary record export, not JSON, so the `blocks`
//! command reads the `node.dag` array out of `/api/state` instead.

use anyhow::{anyhow, bail, Result};
use kovanica_state::AssetId;
use serde_json::Value;

/// A client bound to one explorer base URL (no trailing slash).
pub struct Client {
    base: String,
}

impl Client {
    pub fn new(base: &str) -> Self {
        Self {
            base: base.trim_end_matches('/').to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    fn call(builder: ureq::Request) -> Result<Value> {
        match builder.call() {
            Ok(resp) => {
                let text = resp.into_string()?;
                serde_json::from_str(&text)
                    .map_err(|e| anyhow!("response was not valid JSON: {e}\n{text}"))
            }
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                bail!("HTTP {code}: {}", body.trim())
            }
            Err(e) => Err(anyhow!("request failed: {e}")),
        }
    }

    fn get(&self, path: &str) -> Result<Value> {
        Self::call(ureq::get(&self.url(path)))
    }

    fn post(&self, path: &str) -> Result<Value> {
        Self::call(ureq::post(&self.url(path)))
    }

    fn post_json(&self, path: &str, body: &serde_json::Value) -> Result<Value> {
        let body_str = serde_json::to_string(body)?;
        let resp = ureq::post(&self.url(path))
            .set("Content-Type", "application/json")
            .send_string(&body_str)?;
        let text = resp.into_string()?;
        serde_json::from_str(&text)
            .map_err(|e| anyhow!("response was not valid JSON: {e}\n{text}"))
    }

    pub fn head(&self) -> Result<Value> {
        self.get("/api/head")
    }

    pub fn p2p(&self) -> Result<Value> {
        self.get("/api/p2p")
    }

    pub fn bootstrap(&self) -> Result<Value> {
        self.get("/api/bootstrap")
    }

    pub fn state(&self) -> Result<Value> {
        self.get("/api/state")
    }

    /// The block DAG, pulled out of the full state snapshot.
    pub fn blocks(&self) -> Result<Value> {
        let mut state = self.state()?;
        match state.get_mut("node").and_then(|n| n.get_mut("dag")) {
            Some(dag) => Ok(dag.take()),
            None => Ok(state),
        }
    }

    /// Balance + unspent outputs for an address (hex or `kvnc…dag`).
    pub fn utxos(&self, address: &str) -> Result<Value> {
        self.get(&format!("/api/utxos?address={address}"))
    }

    /// Ask the node to build a transfer and return its signature hash.
    pub fn prepare(&self, from: &str, to: &str, amount: u64) -> Result<Value> {
        self.post(&format!("/api/prepare?from={from}&to={to}&amount={amount}"))
    }

    /// Broadcast a signed transfer. `sig` is 128 lowercase hex chars.
    pub fn submit(&self, from: &str, to: &str, amount: u64, sig: &str) -> Result<Value> {
        self.post(&format!(
            "/api/submit?from={from}&to={to}&amount={amount}&sig={sig}"
        ))
    }

    /// Ask the node to build an asset transfer and return its signature hash.
    pub fn prepare_transfer_asset(
        &self,
        from: &str,
        amount: u64,
        to: &str,
        asset_id: Option<kovanica_state::AssetId>,
    ) -> Result<Value> {
        let mut query = format!("/api/prepare?from={from}&to={to}&amount={amount}");
        if let Some(asset) = asset_id {
            query.push_str(&format!("&asset_id={}", hex::encode(asset.as_bytes())));
        }
        self.post(&query)
    }

    /// Broadcast a signed asset transfer. `sig` is 128 lowercase hex chars.
    pub fn submit_transfer_asset(
        &self,
        from: &str,
        to: &str,
        amount: u64,
        asset_id: Option<kovanica_state::AssetId>,
        sig: &str,
    ) -> Result<Value> {
        let mut query = format!(
            "/api/submit?from={from}&to={to}&amount={amount}&sig={sig}"
        );
        if let Some(asset) = asset_id {
            query.push_str(&format!("&asset_id={}", hex::encode(asset.as_bytes())));
        }
        self.post(&query)
    }

    /// Derive RWA asset_id from issuer key and parameters.
    pub fn rwa_derive(&self, issuer: &str, class: &str, id: &str, version: u8) -> Result<Value> {
        let body = serde_json::json!({
            "issuer": issuer,
            "class": class,
            "id": id,
            "version": version,
        });
        self.post_json("/api/rwa/derive", &body)
    }

    /// Get RWA detail by asset ID.
    pub fn rwa_detail(&self, asset_id: &str) -> Result<Value> {
        self.get(&format!("/api/rwa/{asset_id}"))
    }

    /// Get NFT detail by asset ID.
    pub fn nft_detail(&self, asset_id: &str) -> Result<Value> {
        self.get(&format!("/api/nft/{asset_id}"))
    }

    /// Get collection detail by collection ID.
    pub fn collection_detail(&self, collection_id: &str) -> Result<Value> {
        self.get(&format!("/api/collection/{collection_id}"))
    }
}

/// Pretty-print a JSON value to stdout.
pub fn print_json(value: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
