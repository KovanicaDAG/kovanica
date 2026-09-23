//! Typed HTTP client for the public Kovanica API.
//!
//! Routes verified against `kovanica-node` `explorer.rs`:
//! - `GET  /api/head`
//! - `GET  /api/utxos?address=&limit=&offset=`
//! - `GET  /api/fee_estimate`
//! - `POST /api/submit_tx` with `{"tx_hex": "<hex>"}`
//!
//! Default base URL: `https://api.kovanica.online`.
//! Tests that hit the network are gated behind the `live-testnet` feature.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

use kovanica_tx::SignedTx;
use kovanica_types::{Address, Amount, AssetId, Hash32, NetworkId, TxHash, Utxo};
use serde::{Deserialize, Serialize};

/// Default public API endpoint.
pub const DEFAULT_API_BASE: &str = "https://api.kovanica.online";

/// RPC / HTTP errors.
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    /// HTTP transport error.
    #[error("http error: {0}")]
    Http(String),
    /// JSON decode error.
    #[error("decode error: {0}")]
    Decode(String),
    /// API returned an error payload.
    #[error("api error: {0}")]
    Api(String),
    /// Unexpected status code.
    #[error("unexpected status {0}")]
    Status(u16),
}

/// Minimal shape of `/api/head` (extend as the node adds fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadInfo {
    /// Network name.
    pub network: String,
    /// Genesis hash.
    #[serde(default)]
    pub genesis: Option<String>,
    /// Current tip hash.
    #[serde(default)]
    pub tip: Option<String>,
    /// Approximate height / block count.
    #[serde(default)]
    pub blocks: Option<u64>,
    /// Minimum fee (atoms) if advertised.
    #[serde(default)]
    pub min_fee: Option<u64>,
    /// Atom scale (should be 1e8).
    #[serde(default)]
    pub atom: Option<u64>,
    /// Current block subsidy in atoms (RFC-006).
    #[serde(default)]
    pub subsidy: Option<u64>,
    /// Native KVNC minted so far (atoms).
    #[serde(default)]
    pub native_minted: Option<u64>,
    /// Total supply (atoms).
    #[serde(default)]
    pub total: Option<u64>,
    /// Circulating supply (atoms).
    #[serde(default)]
    pub circulating: Option<u64>,
    /// Burned fees (atoms).
    #[serde(default)]
    pub burned: Option<u64>,
    /// RFC-006 hard cap (90.2M KVNC in atoms).
    #[serde(default)]
    pub max_supply: Option<u64>,
}

/// One UTXO row from `/api/utxos` (node `utxos_json` shape).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoItem {
    /// Hash of the transaction that created the output (hex).
    pub tx: String,
    /// Output index within that transaction.
    pub index: u64,
    /// Value in atoms.
    pub value: u64,
    /// Asset id hex, or null for native KVNC.
    pub asset_id: Option<String>,
    /// Asset kind: "fungible", "nft", or null (native).
    pub kind: Option<String>,
    /// Optional metadata hash (hex).
    #[serde(default)]
    pub metadata_hash: Option<String>,
    /// Optional collection id (hex).
    #[serde(default)]
    pub collection_id: Option<String>,
}

impl UtxoItem {
    /// Convert a node `/api/utxos` row into a domain [`Utxo`] ready for the
    /// transaction builder, binding the *query* address as the owner (the node
    /// does not repeat it per row).
    ///
    /// Wire parity with `kovanica-node` `utxos_json` + `asset_id_to_wire`:
    /// - `tx` is the lowercase 64-hex transaction id,
    /// - `asset_id` is `"KVNC"` (native) or the lowercase 64-hex asset id.
    pub fn into_domain(&self, owner: &Address) -> Result<Utxo, RpcError> {
        let tx_hash =
            TxHash::from_hex(&self.tx).map_err(|e| RpcError::Decode(format!("utxo tx: {e}")))?;
        let asset_id = match self.asset_id.as_deref() {
            None | Some("KVNC") => AssetId::NATIVE,
            Some(hex) => {
                let hash = Hash32::from_hex(hex)
                    .map_err(|e| RpcError::Decode(format!("utxo asset_id: {e}")))?;
                AssetId(hash)
            }
        };
        Ok(Utxo {
            tx_hash,
            vout: self.index.min(u32::MAX as u64) as u32,
            amount: Amount::from_atoms(self.value),
            asset_id,
            address: *owner,
        })
    }
}

/// Full `/api/utxos` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxosResponse {
    /// Requested address (66-hex form).
    pub address: String,
    /// Native balance in atoms.
    pub balance: u64,
    /// Per-asset balances: asset hex → atoms.
    #[serde(default)]
    pub balances: std::collections::BTreeMap<String, u64>,
    /// UTXO rows (offset/limit applied).
    pub utxos: Vec<UtxoItem>,
    /// Applied limit.
    pub limit: u64,
    /// Applied offset.
    pub offset: u64,
    /// Total rows available.
    pub total: u64,
}

/// `/api/fee_estimate` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    /// Fee rate in atoms per byte.
    pub fee_rate: u64,
    /// Unit string ("atoms/byte").
    pub unit: String,
    /// Number of pending mempool transactions.
    pub mempool: u64,
    /// Total mempool bytes.
    pub bytes: u64,
}

/// `/api/bootstrap` response — node/consensus parameters the client needs to
/// know (RFC-006 tokenomics, GHOSTDAG k, network identity).
///
/// Read-only mirror of the node's JSON; fields the node may omit stay `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapInfo {
    /// Network name (e.g. "kovanica-testnet").
    #[serde(default)]
    pub network: Option<String>,
    /// Genesis hash (hex).
    #[serde(default)]
    pub genesis: Option<String>,
    /// GHOSTDAG k parameter (3 on testnet).
    #[serde(default)]
    pub k: Option<u32>,
    /// Native token symbol.
    #[serde(default)]
    pub token: Option<String>,
    /// Atom scale (should be 1e8 = atoms per KVNC).
    #[serde(default)]
    pub atom: Option<u64>,
    /// Minimum fee (atoms/byte) the node advertises.
    #[serde(default)]
    pub min_fee: Option<u64>,
    /// Current block subsidy in atoms (RFC-006).
    #[serde(default)]
    pub subsidy: Option<u64>,
    /// RFC-006 hard cap in atoms (90.2M KVNC).
    #[serde(default)]
    pub max_supply: Option<u64>,
    /// Native KVNC minted so far (atoms).
    #[serde(default)]
    pub native_minted: Option<u64>,
    /// Total supply (atoms).
    #[serde(default)]
    pub total: Option<u64>,
    /// Circulating supply (atoms).
    #[serde(default)]
    pub circulating: Option<u64>,
    /// Burned fees (atoms).
    #[serde(default)]
    pub burned: Option<u64>,
    /// Configured P2P peer list (DNS seed names / origin IPs).
    #[serde(default)]
    pub peers: Vec<String>,
}

/// HTTP client.
#[derive(Clone)]
pub struct Client {
    base_url: String,
    http: reqwest::Client,
}

impl Client {
    /// Custom base URL.
    pub fn new(base_url: impl Into<String>) -> Result<Self, RpcError> {
        let http = reqwest::Client::builder()
            .user_agent(concat!("kovanica-sdk/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| RpcError::Http(e.to_string()))?;
        Ok(Client {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            http,
        })
    }

    /// Convenience: public testnet/mainnet shared API.
    pub fn default_public() -> Result<Self, RpcError> {
        Self::new(DEFAULT_API_BASE)
    }

    /// Alias for clarity in examples (shared public API).
    pub fn testnet() -> Result<Self, RpcError> {
        Self::default_public()
    }

    /// Alias for clarity in examples (shared public API).
    pub fn mainnet() -> Result<Self, RpcError> {
        Self::default_public()
    }

    /// GET /api/head
    pub async fn get_head(&self) -> Result<HeadInfo, RpcError> {
        let url = format!("{}/api/head", self.base_url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| RpcError::Http(e.to_string()))?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(RpcError::Status(status));
        }
        resp.json::<HeadInfo>()
            .await
            .map_err(|e| RpcError::Decode(e.to_string()))
    }

    /// GET /api/utxos for an address (address sent as 66-hex, as the node renders it).
    pub async fn get_utxos(
        &self,
        address: &Address,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<UtxosResponse, RpcError> {
        let mut url = format!("{}/api/utxos?address={}", self.base_url, address.to_hex());
        if let Some(l) = limit {
            url.push_str(&format!("&limit={l}"));
        }
        if let Some(o) = offset {
            url.push_str(&format!("&offset={o}"));
        }
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| RpcError::Http(e.to_string()))?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            let text = resp.text().await.unwrap_or_default();
            return Err(RpcError::Api(format!("status {status}: {text}")));
        }
        resp.json::<UtxosResponse>()
            .await
            .map_err(|e| RpcError::Decode(e.to_string()))
    }

    /// GET /api/fee_estimate — current mempool fee rate (atoms/byte).
    pub async fn get_fee_estimate(&self) -> Result<FeeEstimate, RpcError> {
        let url = format!("{}/api/fee_estimate", self.base_url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| RpcError::Http(e.to_string()))?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(RpcError::Status(status));
        }
        resp.json::<FeeEstimate>()
            .await
            .map_err(|e| RpcError::Decode(e.to_string()))
    }

    /// GET /api/bootstrap — consensus + tokenomics parameters (RFC-006, k).
    pub async fn get_bootstrap(&self) -> Result<BootstrapInfo, RpcError> {
        let url = format!("{}/api/bootstrap", self.base_url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| RpcError::Http(e.to_string()))?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(RpcError::Status(status));
        }
        resp.json::<BootstrapInfo>()
            .await
            .map_err(|e| RpcError::Decode(e.to_string()))
    }

    /// Submit a signed transaction: `POST /api/submit_tx` with `{"tx_hex": …}`.
    ///
    /// Returns the accepted transaction id (hex).
    pub async fn submit_tx(&self, signed: &SignedTx) -> Result<TxHash, RpcError> {
        let url = format!("{}/api/submit_tx", self.base_url);
        let body = serde_json::json!({ "tx_hex": signed.tx_hex() });
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RpcError::Http(e.to_string()))?;
        let status = resp.status().as_u16();
        let text = resp
            .text()
            .await
            .map_err(|e| RpcError::Http(e.to_string()))?;
        if !(200..300).contains(&status) {
            return Err(RpcError::Api(format!("status {status}: {text}")));
        }
        // `{"ok":true,"tx":"<tx_id>"}` (also tolerate bare hash / tx_hash).
        #[derive(Deserialize)]
        struct SubmitResp {
            #[serde(default)]
            ok: bool,
            #[serde(default)]
            tx: Option<String>,
            #[serde(alias = "hash", alias = "txid")]
            tx_hash: Option<String>,
        }
        if let Ok(h) = TxHash::from_hex(text.trim().trim_matches('"')) {
            return Ok(h);
        }
        let parsed: SubmitResp =
            serde_json::from_str(&text).map_err(|e| RpcError::Decode(e.to_string()))?;
        if !parsed.ok {
            return Err(RpcError::Api(text));
        }
        let hex = parsed
            .tx
            .or(parsed.tx_hash)
            .ok_or_else(|| RpcError::Decode("missing tx id in response".into()))?;
        TxHash::from_hex(&hex).map_err(|e| RpcError::Decode(e.to_string()))
    }
}

/// Map network name from head into our enum when possible.
pub fn parse_network_id(s: &str) -> Option<NetworkId> {
    match s {
        "kovanica-testnet" | "testnet" => Some(NetworkId::Testnet),
        "kovanica" | "mainnet" => Some(NetworkId::Mainnet),
        _ => None,
    }
}

#[cfg(all(test, feature = "live-testnet"))]
mod live_tests {
    use super::*;

    #[tokio::test]
    async fn head_reachable() {
        let client = Client::testnet().unwrap();
        let head = client.get_head().await.expect("head");
        assert!(!head.network.is_empty());
        println!("network = {}, blocks = {:?}", head.network, head.blocks);
    }
}
