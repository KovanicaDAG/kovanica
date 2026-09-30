//! Kovanica cross-chain HTLC relayer.
//!
//! Watches Kovanica HTLC events via WebSocket and funds corresponding
//! HTLCs on destination chains (BTC, ETH, XRP, DOGE, SOL).
//!
//! The relayer acts as the counterparty in atomic swaps:
//! 1. Monitors Kovanica for new HTLC funding transactions
//! 2. When an HTLC is funded, locks equivalent funds on destination chain
//! 3. When user claims on destination chain (reveals preimage), relayer claims on Kovanica
//! 4. If timeout expires, relayer refunds on both chains

use futures_util::{sink::SinkExt, stream::{StreamExt, TryStreamExt}};
use kovanica_bridge::{
    BridgeHtlcParams, BridgeState, BridgeSwap, DestinationChain, generate_swap_id_hex,
    calculate_timeouts,
};
use kovanica_rpc::Client;
use kovanica_types::{Address, TypesError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

#[derive(Error, Debug)]
pub enum RelayerError {
    #[error("RPC error: {0}")]
    Rpc(#[from] kovanica_rpc::RpcError),
    #[error("WebSocket error: {0}")]
    Ws(#[from] tungstenite::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Type error: {0}")]
    Types(#[from] TypesError),
    #[error("Invalid configuration: {0}")]
    Config(String),
    #[error("Destination chain error: {0}")]
    DestinationChain(String),
    #[error("Timeout waiting for event: {0}")]
    Timeout(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, RelayerError>;

/// Configuration for the relayer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerConfig {
    /// Kovanica node WebSocket URL
    pub kovanica_ws_url: String,
    /// Kovanica node HTTP API URL
    pub kovanica_api_url: String,
    /// Relayer's Kovanica address (for funding HTLCs)
    pub relayer_address: Address,
    /// Relayer's signing key (64 hex chars, Ed25519)
    pub relayer_signing_key: String,
    /// Destination chain configurations
    pub destination_chains: HashMap<DestinationChain, DestinationChainConfig>,
    /// Poll interval for checking HTLC status (ms)
    pub poll_interval_ms: u64,
    /// Maximum concurrent swaps to handle
    pub max_concurrent_swaps: usize,
    /// Minimum profit margin (basis points) to accept a swap
    pub min_profit_bps: u32,
}

/// Configuration for a destination chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationChainConfig {
    /// RPC endpoint for the destination chain
    pub rpc_url: String,
    /// Relayer's address/private key on destination chain
    pub relayer_key: String,
    /// Minimum confirmation blocks before considering funds locked
    pub min_confirmations: u32,
    /// Gas price / fee settings
    pub gas_price: Option<u64>,
    /// Whether this chain is enabled
    pub enabled: bool,
}

/// Event types from Kovanica WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum KovanicaEvent {
    #[serde(rename = "block")]
    Block { id: String, blue_score: u64 },
    #[serde(rename = "tx")]
    Tx {
        id: String,
        from: String,
        to: String,
        amount: u64,
        asset_id: Option<String>,
    },
    #[serde(rename = "htlc_created")]
    HtlcCreated {
        tx_id: String,
        htlc_script: String,
        amount: u64,
        _asset_id: Option<String>,
        sender: String,
        recipient_pk: String,
        preimage_hash: String,
        timeout: u64,
    },
    #[serde(rename = "htlc_claimed")]
    HtlcClaimed {
        tx_id: String,
        htlc_script: String,
        preimage: String,
        claimant: String,
    },
    #[serde(rename = "htlc_refunded")]
    HtlcRefunded {
        tx_id: String,
        htlc_script: String,
        refundee: String,
    },
}

/// State of a swap being managed by the relayer
#[derive(Debug, Clone)]
pub struct ManagedSwap {
    pub swap: BridgeSwap,
    pub kovanica_htlc_tx: Option<String>,
    pub destination_htlc_tx: Option<String>,
    pub preimage: Option<[u8; 32]>,
    pub last_checked: std::time::Instant,
}

/// Main relayer struct
pub struct Relayer {
    config: RelayerConfig,
    client: Client,
    swaps: Arc<RwLock<HashMap<String, ManagedSwap>>>,
    running: Arc<RwLock<bool>>,
}

impl Relayer {
    /// Create a new relayer instance
    pub fn new(config: RelayerConfig) -> Result<Self> {
        let client = Client::new(&config.kovanica_api_url)?;
        Ok(Self {
            config,
            client,
            swaps: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the relayer
    pub async fn start(&self) -> Result<()> {
        *self.running.write().await = true;
        info!("Starting Kovanica relayer");

        // Connect to WebSocket
        let ws_url = self.config.kovanica_ws_url.clone();
        let mut ws_stream = connect_websocket(&ws_url).await?;

        // Subscribe to HTLC events
        let subscribe_msg = serde_json::json!({
            "type": "subscribe",
            "events": ["htlc_created", "htlc_claimed", "htlc_refunded"]
        });
        ws_stream.send(tungstenite::Message::Text(serde_json::to_string(&subscribe_msg)?)).await?;

        // Main event loop
        while *self.running.read().await {
            tokio::select! {
                msg = ws_stream.next() => {
                    if let Some(Ok(msg)) = msg {
                        if let Err(e) = self.handle_ws_message(msg).await {
                            error!("Error handling WebSocket message: {}", e);
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(self.config.poll_interval_ms)) => {
                    if let Err(e) = self.check_pending_swaps().await {
                        error!("Error checking pending swaps: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop the relayer
    pub async fn stop(&self) {
        *self.running.write().await = false;
        info!("Relayer stopped");
    }

    /// Handle incoming WebSocket message
    async fn handle_ws_message(&self, msg: tungstenite::Message) -> Result<()> {
        if let tungstenite::Message::Text(text) = msg {
            let event: KovanicaEvent = serde_json::from_str(&text)?;
            match event {
                KovanicaEvent::HtlcCreated { tx_id, htlc_script, amount, _asset_id, sender, recipient_pk, preimage_hash, timeout } => {
                    self.on_htlc_created(tx_id, htlc_script, amount, _asset_id, sender, recipient_pk, preimage_hash, timeout).await?;
                }
                KovanicaEvent::HtlcClaimed { tx_id, htlc_script, preimage, claimant } => {
                    self.on_htlc_claimed(tx_id, htlc_script, preimage, claimant).await?;
                }
                KovanicaEvent::HtlcRefunded { tx_id, htlc_script, refundee } => {
                    self.on_htlc_refunded(tx_id, htlc_script, refundee).await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle new HTLC created on Kovanica
    async fn on_htlc_created(
        &self,
        tx_id: String,
        htlc_script: String,
        amount: u64,
        _asset_id: Option<String>,
        sender: String,
        recipient_pk: String,
        preimage_hash: String,
        timeout: u64,
    ) -> Result<()> {
        info!("New HTLC created: tx_id={}, amount={}, timeout={}", tx_id, amount, timeout);

        // Parse HTLC script to get details
        let script_bytes = hex::decode(&htlc_script).map_err(|_| RelayerError::Config("Invalid HTLC script hex".into()))?;
        if script_bytes.len() != 100 {
            return Err(RelayerError::Config("HTLC script must be 100 bytes".into()));
        }

        // Determine which destination chain this is for
        // In practice, this would come from the swap offer or a registry
        // For now, we'll assume Ethereum as default
        let destination_chain = DestinationChain::Ethereum;

        // Check if we have config for this chain
        let chain_config = self.config.destination_chains.get(&destination_chain)
            .ok_or_else(|| RelayerError::Config(format!("No config for destination chain {:?}", destination_chain)))?;

        if !chain_config.enabled {
            return Err(RelayerError::Config(format!("Destination chain {:?} is disabled", destination_chain)).into());
        }

        // Calculate timeouts
        let kovanica_tip = self.get_kovanica_tip().await?;
        let (kovanica_timeout, destination_timeout) = calculate_timeouts(destination_chain, kovanica_tip);

        // Verify the timeout matches
        if timeout < kovanica_timeout {
            warn!("HTLC timeout {} is less than recommended {}", timeout, kovanica_timeout);
        }

        // Create swap record
        let swap_id = generate_swap_id_hex();
        let swap = BridgeSwap {
            swap_id: swap_id.clone(),
            params: BridgeHtlcParams {
                kovanica_script: htlc_script.clone(),
                preimage_hash: preimage_hash.clone(),
                kovanica_timeout: timeout,
                destination_timeout,
                destination_amount: amount, // Would need conversion for non-native assets
                destination_chain,
                destination_recipient: recipient_pk,
                kovanica_sender: kovanica_types::Address::from_hex(&sender)?,
            },
            state: BridgeState::AwaitingDestinationFund,
            kovanica_outpoint: None, // Would need to parse from tx_id
            destination_txid: None,
            preimage: None,
            created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64,
            updated_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64,
        };

        // Fund HTLC on destination chain
        let dest_txid = self.fund_destination_htlc(&swap, chain_config).await?;
        info!("Funded destination HTLC: {}", dest_txid);

        // Update swap record
        let mut managed_swap = ManagedSwap {
            swap: swap.clone(),
            kovanica_htlc_tx: Some(tx_id),
            destination_htlc_tx: Some(dest_txid.clone()),
            preimage: None,
            last_checked: std::time::Instant::now(),
        };
        managed_swap.swap.state = BridgeState::ReadyForClaim;
        managed_swap.swap.destination_txid = Some(dest_txid);
        managed_swap.swap.updated_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;

        // Store swap
        self.swaps.write().await.insert(swap_id.clone(), managed_swap);

        info!("Swap {} created and funded on both chains", swap_id);
        Ok(())
    }

    /// Handle HTLC claimed on Kovanica (user revealed preimage)
    async fn on_htlc_claimed(
        &self,
        tx_id: String,
        htlc_script: String,
        preimage: String,
        claimant: String,
    ) -> Result<()> {
        info!("HTLC claimed on Kovanica: tx_id={}, claimant={}", tx_id, claimant);

        // Find the corresponding swap
        let preimage_bytes = hex::decode(&preimage).map_err(|_| RelayerError::Config("Invalid preimage hex".into()))?;
        if preimage_bytes.len() != 32 {
            return Err(RelayerError::Config("Preimage must be 32 bytes".into()));
        }
        let mut preimage_arr = [0u8; 32];
        preimage_arr.copy_from_slice(&preimage_bytes);

        // Find swap by HTLC script
        let swap_key = self.find_swap_by_script(&htlc_script).await;
        if let Some(key) = swap_key {
            let mut swaps = self.swaps.write().await;
            if let Some(managed) = swaps.get_mut(&key) {
                let preimage_hex = hex::encode(preimage_arr);
                managed.preimage = Some(preimage_arr);
                managed.swap.state = BridgeState::ClaimedKovanica;
                managed.swap.preimage = Some(preimage_hex);
                managed.swap.updated_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;

                // Now claim on destination chain using the preimage
                let dest_chain = managed.swap.params.destination_chain;
                if let Some(chain_config) = self.config.destination_chains.get(&dest_chain) {
                    if let Some(dest_txid) = &managed.destination_htlc_tx {
                        let claim_txid = self.claim_destination_htlc(dest_txid, &preimage_arr, chain_config).await?;
                        info!("Claimed destination HTLC: {}", claim_txid);
                        managed.swap.state = BridgeState::ClaimedDestination;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle HTLC refunded on Kovanica
    async fn on_htlc_refunded(
        &self,
        tx_id: String,
        htlc_script: String,
        refundee: String,
    ) -> Result<()> {
        info!("HTLC refunded on Kovanica: tx_id={}, refundee={}", tx_id, refundee);

        // Find swap and refund on destination chain
        let swap_key = self.find_swap_by_script(&htlc_script).await;
        if let Some(key) = swap_key {
            let mut swaps = self.swaps.write().await;
            if let Some(managed) = swaps.get_mut(&key) {
                managed.swap.state = BridgeState::Refunded;
                managed.swap.updated_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;

                // Refund on destination chain
                let dest_chain = managed.swap.params.destination_chain;
                if let Some(chain_config) = self.config.destination_chains.get(&dest_chain) {
                    if let Some(dest_txid) = &managed.destination_htlc_tx {
                        let refund_txid = self.refund_destination_htlc(dest_txid, chain_config).await?;
                        info!("Refunded destination HTLC: {}", refund_txid);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check pending swaps for timeouts
    async fn check_pending_swaps(&self) -> Result<()> {
        let mut swaps = self.swaps.write().await;
        let kovanica_tip = self.get_kovanica_tip().await?;

        let expired_keys: Vec<String> = swaps.iter()
            .filter(|(_, managed)| {
                // Check if Kovanica HTLC has expired
                if managed.swap.params.kovanica_timeout <= kovanica_tip {
                    matches!(managed.swap.state, BridgeState::AwaitingDestinationFund | BridgeState::ReadyForClaim)
                } else {
                    false
                }
            })
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            if let Some(managed) = swaps.get_mut(&key) {
                warn!("Swap {} expired on Kovanica, initiating refunds", key);
                managed.swap.state = BridgeState::Expired;
                managed.swap.updated_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;

                // Refund on destination chain if funded
                if let Some(dest_txid) = &managed.destination_htlc_tx {
                    let dest_chain = managed.swap.params.destination_chain;
                    if let Some(chain_config) = self.config.destination_chains.get(&dest_chain) {
                        let _ = self.refund_destination_htlc(dest_txid, chain_config).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Find swap by HTLC script
    async fn find_swap_by_script(&self, script: &str) -> Option<String> {
        let swaps = self.swaps.read().await;
        swaps.iter()
            .find(|(_, m)| m.swap.params.kovanica_script == script)
            .map(|(k, _)| k.clone())
    }

    /// Get current Kovanica tip height
    async fn get_kovanica_tip(&self) -> Result<u64> {
        let head = self.client.get_head().await.map_err(RelayerError::Rpc)?;
        Ok(head.blocks.unwrap_or(0))
    }

    /// Fund HTLC on destination chain (stub - would integrate with chain-specific SDKs)
    async fn fund_destination_htlc(&self, swap: &BridgeSwap, _config: &DestinationChainConfig) -> Result<String> {
        // This would integrate with:
        // - Bitcoin: bitcoincore-rpc or bdk
        // - Ethereum: ethers-rs or alloy
        // - XRP: xrpl-rs
        // - Dogecoin: dogecoin-rpc
        // - Solana: solana-sdk
        //
        // For now, return a mock transaction ID
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"mock_dest_fund");
        hasher.update(&swap.swap_id.as_bytes());
        Ok(hex::encode(hasher.finalize().as_bytes()))
    }

    /// Claim HTLC on destination chain using preimage
    async fn claim_destination_htlc(&self, dest_txid: &str, preimage: &[u8; 32], _config: &DestinationChainConfig) -> Result<String> {
        // Chain-specific claim implementation
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"mock_dest_claim");
        hasher.update(dest_txid.as_bytes());
        hasher.update(preimage);
        Ok(hex::encode(hasher.finalize().as_bytes()))
    }

    /// Refund HTLC on destination chain
    async fn refund_destination_htlc(&self, dest_txid: &str, _config: &DestinationChainConfig) -> Result<String> {
        // Chain-specific refund implementation
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"mock_dest_refund");
        hasher.update(dest_txid.as_bytes());
        Ok(hex::encode(hasher.finalize().as_bytes()))
    }
}

/// Connect to Kovanica WebSocket
async fn connect_websocket(url: &str) -> Result<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>> {
    let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
    Ok(ws_stream)
}

/// Relayer builder for easier configuration
pub struct RelayerBuilder {
    config: RelayerConfig,
}

impl RelayerBuilder {
    pub fn new() -> Self {
        Self {
            config: RelayerConfig {
                kovanica_ws_url: "ws://localhost:8080/ws".to_string(),
                kovanica_api_url: "http://localhost:8080".to_string(),
                relayer_address: kovanica_types::Address::from_versioned([0; 33]),
                relayer_signing_key: String::new(),
                destination_chains: HashMap::new(),
                poll_interval_ms: 5000,
                max_concurrent_swaps: 100,
                min_profit_bps: 100,
            }
        }
    }

    pub fn kovanica_ws_url(mut self, url: String) -> Self {
        self.config.kovanica_ws_url = url;
        self
    }

    pub fn kovanica_api_url(mut self, url: String) -> Self {
        self.config.kovanica_api_url = url;
        self
    }

    pub fn relayer_address(mut self, address: Address) -> Self {
        self.config.relayer_address = address;
        self
    }

    pub fn relayer_signing_key(mut self, key: String) -> Self {
        self.config.relayer_signing_key = key;
        self
    }

    pub fn add_destination_chain(mut self, chain: DestinationChain, config: DestinationChainConfig) -> Self {
        self.config.destination_chains.insert(chain, config);
        self
    }

    pub fn poll_interval_ms(mut self, ms: u64) -> Self {
        self.config.poll_interval_ms = ms;
        self
    }

    pub fn build(self) -> Result<Relayer> {
        Relayer::new(self.config)
    }
}

impl Default for RelayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relayer_builder() {
        let relayer = RelayerBuilder::new()
            .kovanica_ws_url("ws://localhost:8080/ws".to_string())
            .kovanica_api_url("http://localhost:8080".to_string())
            .poll_interval_ms(10000)
            .build();
        assert!(relayer.is_ok());
    }

    #[test]
    fn test_destination_chain_config() {
        let config = DestinationChainConfig {
            rpc_url: "http://localhost:8545".to_string(),
            relayer_key: "0x...".to_string(),
            min_confirmations: 12,
            gas_price: Some(20_000_000_000),
            enabled: true,
        };
        assert!(config.enabled);
        assert_eq!(config.min_confirmations, 12);
    }
}