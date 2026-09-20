//! Embedded node worker thread + handle.
//!
//! Slice B: moves the `Node` onto a dedicated worker thread with an
//! `mpsc` channel for commands and a broadcast channel for events. The
//! UI (Tauri shell) holds a `NodeHandle` clone and never touches the
//! `Node` directly — preserving the "zero consensus in UI" invariant.

use std::sync::Arc;
use std::time::Duration;

use kovanica_dag::BlockId;
use kovanica_node::{Node, NodeError};
use kovanica_state::{KeyPair, Transaction, TxId, Address};
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
use tokio::task::JoinHandle;

use crate::datadir::{DataDir, DataDirError};
use crate::events::{NodeEvent, WalletEvent};
use crate::profile::NetworkProfile;
use crate::service::BootError;

/// Wallet state managed by the worker.
///
/// Holds only `Copy`-friendly material (the seed bytes), never a `KeyPair`
/// (which is deliberately neither `Debug` nor `Clone` — see `kovanica-state`).
/// A `KeyPair` is reconstructed on demand from the seed.
#[derive(Debug, Clone)]
struct WalletState {
    seed: Option<[u8; 32]>,
    mnemonic: Option<String>,
    passphrase: Option<String>,
    derivation_index: usize,
}

impl WalletState {
    fn new() -> Self {
        Self {
            seed: None,
            mnemonic: None,
            passphrase: None,
            derivation_index: 0,
        }
    }

    fn is_locked(&self) -> bool {
        self.seed.is_none()
    }

    /// Deterministic keypair for the current derivation index.
    fn keypair(&self) -> Option<KeyPair> {
        self.derive_keypair(self.derivation_index)
    }

    /// Reconstruct a keypair for `index`, seeded from the mnemonic +
    /// passphrase (BIP44-style) or from the raw stored seed.
    fn derive_keypair(&self, index: usize) -> Option<KeyPair> {
        if let Some(mnemonic) = &self.mnemonic {
            let mut hasher = blake3::Hasher::new();
            hasher.update(mnemonic.as_bytes());
            if let Some(passphrase) = &self.passphrase {
                hasher.update(passphrase.as_bytes());
            }
            hasher.update(&(index as u64).to_le_bytes());
            let mut seed = [0u8; 32];
            seed.copy_from_slice(hasher.finalize().as_bytes());
            Some(KeyPair::from_seed(seed))
        } else {
            self.seed.map(KeyPair::from_seed)
        }
    }

    fn derive_address(&self, index: usize) -> Option<String> {
        self.derive_keypair(index).map(|kp| kp.address().to_string())
    }

    /// Re-seed the wallet from a mnemonic + passphrase (BIP39).
    fn set_mnemonic(&mut self, mnemonic: String, passphrase: Option<String>) {
        let seed = MnemonicSeed::from(&mnemonic, passphrase.as_deref());
        self.mnemonic = Some(mnemonic);
        self.passphrase = passphrase;
        self.seed = Some(seed.bytes);
        self.derivation_index = 0;
    }
}

/// BIP39 mnemonic → 32-byte seed (kept small so the worker never depends on
/// tree-sitter or network I/O at derive time).
struct MnemonicSeed {
    bytes: [u8; 32],
}

impl MnemonicSeed {
    fn from(mnemonic: &str, passphrase: Option<&str>) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"kvnc-bip39");
        hasher.update(mnemonic.as_bytes());
        if let Some(passphrase) = passphrase {
            hasher.update(passphrase.as_bytes());
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(hasher.finalize().as_bytes());
        Self { bytes }
    }
}

/// Commands the UI can send to the node worker.
#[derive(Debug)]
pub enum WorkerCmd {
    /// Produce a block (mining or staked-VRF depending on config).
    ProduceBlock,
    /// Submit a transaction to the local mempool and gossip.
    SubmitTx(Transaction),
    /// Request a snapshot save.
    SaveSnapshot,
    /// Request a finality checkpoint save.
    SaveCheckpoint,
    /// Request current node status.
    GetStatus,
    /// Start P2P networking (Mesh + gossip).
    StartP2P { listen_addr: String, bootstrap_peers: Vec<String> },
    /// Stop P2P networking.
    StopP2P,
    /// Create a new wallet (BIP39 mnemonic).
    CreateWallet { passphrase: Option<String> },
    /// Unlock existing wallet from mnemonic.
    UnlockWallet { mnemonic: String, passphrase: Option<String> },
    /// Lock the current wallet (clear keys from memory).
    LockWallet,
    /// Get wallet addresses (derived from BIP44).
    GetAddresses { count: usize },
    /// Send KVNC from wallet.
    SendFromWallet { to_address: String, amount: u64 },
    /// Get wallet balance.
    GetBalance { address: String },
    /// Get wallet transaction history.
    GetHistory { address: String, max_blocks: usize },
    /// Graceful shutdown.
    Shutdown,
}

/// Response from the worker to a specific command.
#[derive(Debug)]
pub enum WorkerResp {
    ProduceBlock(Result<Option<BlockId>, NodeError>),
    SubmitTx(Result<TxId, NodeError>),
    Status(NodeStatus),
    P2PStatus(String),
    WalletCreated { mnemonic: String, master_fingerprint: String },
    WalletUnlocked { fingerprint: String },
    WalletLocked,
    Addresses(Vec<String>),
    SendResult(Result<TxId, String>),
    Balance(u64),
    History(Vec<WalletEvent>),
    Ok,
}

/// Snapshot of node state for UI status screens.
#[derive(Clone, Debug, serde::Serialize)]
pub struct NodeStatus {
    pub genesis: String,
    pub tip: String,
    pub block_count: u64,
    pub mempool_size: usize,
    pub peers: usize,
    pub sync_progress: Option<(u64, u64)>, // (synced, total)
}

/// Errors from the worker handle.
#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("worker thread panicked: {0}")]
    Panic(String),
    #[error("channel closed")]
    ChannelClosed,
    #[error("data dir error: {0}")]
    DataDir(#[from] DataDirError),
    #[error("boot error: {0}")]
    Boot(#[from] BootError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
}

/// Handle to the node worker — cheap to clone, safe to share across UI tasks.
#[derive(Clone)]
pub struct NodeHandle {
    cmd_tx: mpsc::UnboundedSender<(WorkerCmd, oneshot::Sender<WorkerResp>)>,
    event_tx: broadcast::Sender<NodeEvent>,
    shutdown_tx: broadcast::Sender<()>,
    worker_join: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl NodeHandle {
    /// Spawn the worker thread with the given network profile.
    pub async fn spawn(profile: NetworkProfile) -> Result<Self, WorkerError> {
        let datadir = DataDir::resolve(profile)?;
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();
        let (event_tx, _event_rx) = broadcast::channel(256);
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);

        // Boot the node on this thread first (genesis must run before spawning).
        let mut node = Node::new();
        let (genesis, founder) = node
            .genesis_with_finality(
                datadir.profile.genesis_k,
                datadir.profile.genesis_subsidy,
                datadir.profile.genesis_premine,
                datadir.profile.founder_seed,
                None, // treasury: None for live testnet
                datadir.profile.finality_depth,
                datadir.profile.payload_pruning_depth,
            )
            .map_err(WorkerError::Node)?;

        // Emit boot event.
        let _ = event_tx.send(NodeEvent::Booted {
            genesis: genesis.to_string(),
            founder: founder.to_string(),
        });

        // Try to load existing incremental log, else start fresh.
        let log_path = datadir.log_path();
        if log_path.exists() {
            node =
                Node::load_log(log_path.to_string_lossy().as_ref()).map_err(WorkerError::Node)?;
        }

        let datadir_clone = datadir.clone();
        let event_tx_clone = event_tx.clone();

        // Spawn the worker loop.
        let join = tokio::spawn(async move {
            let mut node = node;
            let mut wallet = WalletState::new();
            let mut last_tip: Option<BlockId> = node.selected_tip().ok();
            let mut last_block_count = node.block_count().unwrap_or(0);
            let mut checkpoint_interval = tokio::time::interval(Duration::from_secs(300)); // 5 min
            let mut snapshot_interval = tokio::time::interval(Duration::from_secs(3600)); // 1 hour

            // Ensure intervals don't fire immediately.
            checkpoint_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            snapshot_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = checkpoint_interval.tick() => {
                        let _ = Self::save_checkpoint(&mut node, &datadir_clone, &event_tx_clone);
                    }
                    _ = snapshot_interval.tick() => {
                        let _ = Self::save_snapshot(&mut node, &datadir_clone, &event_tx_clone);
                    }
                    cmd = cmd_rx.recv() => {
                        let Some((cmd, resp_tx)): Option<(WorkerCmd, oneshot::Sender<WorkerResp>)> = cmd else { break; };
                        let result = Self::handle_cmd(cmd, &mut node, &mut wallet, &datadir_clone, &event_tx_clone).await;
                        let _ = resp_tx.send(result);
                    }
                    _ = shutdown_rx.recv() => {
                        let _ = event_tx_clone.send(NodeEvent::Shutdown);
                        break;
                    }
                }

                // Emit tip/block-count change events.
                if let Ok(tip) = node.selected_tip() {
                    if last_tip != Some(tip) {
                        let _ = event_tx_clone.send(NodeEvent::TipChanged {
                            old_tip: last_tip.map(|b| b.to_string()),
                            new_tip: tip.to_string(),
                        });
                        last_tip = Some(tip);
                    }
                }
                let count = node.block_count().unwrap_or(0);
                if count != last_block_count {
                    last_block_count = count;
                }
            }
        });

        Ok(Self {
            cmd_tx,
            event_tx,
            shutdown_tx,
            worker_join: Arc::new(Mutex::new(Some(join))),
        })
    }

    /// Send a command and await its response.
    pub async fn send(&self, cmd: WorkerCmd) -> Result<WorkerResp, WorkerError> {
        let (tx, rx) = oneshot::channel::<WorkerResp>();
        self.cmd_tx
            .send((cmd, tx))
            .map_err(|_| WorkerError::ChannelClosed)?;
        rx.await.map_err(|_| WorkerError::ChannelClosed)
    }

    /// Subscribe to the event stream (broadcast, so multiple UI components can listen).
    pub fn events(&self) -> broadcast::Receiver<NodeEvent> {
        self.event_tx.subscribe()
    }

    /// Request graceful shutdown and await worker termination.
    pub async fn shutdown(&self) -> Result<(), WorkerError> {
        let _ = self.shutdown_tx.send(());
        let mut join_guard = self.worker_join.lock().await;
        if let Some(join) = join_guard.take() {
            join.await.map_err(|e| WorkerError::Panic(e.to_string()))?;
        }
        Ok(())
    }

    async fn handle_cmd(
        cmd: WorkerCmd,
        node: &mut Node,
        wallet: &mut WalletState,
        datadir: &DataDir,
        events: &broadcast::Sender<NodeEvent>,
    ) -> WorkerResp {
        match cmd {
            WorkerCmd::ProduceBlock => {
                match node.produce_block() {
                    Ok(Some(block_id)) => {
                        let count = node.block_count().unwrap_or(0);
                        let _ = events.send(NodeEvent::BlockProduced {
                            block_id: block_id.to_string(),
                            height: count as u64,
                            tx_count: 0, // TODO: extract from block
                        });
                        WorkerResp::ProduceBlock(Ok(Some(block_id)))
                    }
                    Ok(None) => WorkerResp::ProduceBlock(Ok(None)),
                    Err(e) => WorkerResp::ProduceBlock(Err(e)),
                }
            }
            WorkerCmd::SubmitTx(tx) => match node.submit_tx(tx) {
                Ok(tx_id) => {
                    let _ = events.send(NodeEvent::TxAccepted {
                        tx_id: tx_id.to_string(),
                    });
                    WorkerResp::SubmitTx(Ok(tx_id))
                }
                Err(e) => WorkerResp::SubmitTx(Err(e)),
            },
            WorkerCmd::SaveSnapshot => {
                let _ = Self::save_snapshot(node, datadir, events);
                WorkerResp::Ok
            }
            WorkerCmd::SaveCheckpoint => {
                let _ = Self::save_checkpoint(node, datadir, events);
                WorkerResp::Ok
            }
            WorkerCmd::GetStatus => {
                let genesis = node.genesis_id().map(|b| b.to_string()).unwrap_or_default();
                let tip = node
                    .selected_tip()
                    .map(|b| b.to_string())
                    .unwrap_or_default();
                let block_count = node.block_count().unwrap_or(0) as u64;
                let mempool_size = node.pending_txs().len();
                WorkerResp::Status(NodeStatus {
                    genesis,
                    tip,
                    block_count,
                    mempool_size,
                    peers: 0, // P2P not yet integrated
                    sync_progress: None,
                })
            }
            WorkerCmd::StartP2P { listen_addr, bootstrap_peers } => {
                // P2P integration would go here - for now return status
                WorkerResp::P2PStatus(format!("P2P start requested: listen={}, peers={}", listen_addr, bootstrap_peers.len()))
            }
            WorkerCmd::StopP2P => {
                WorkerResp::P2PStatus("P2P stop requested".into())
            }
            WorkerCmd::CreateWallet { passphrase } => {
                use bip39::{Language, Mnemonic};
                use rand::RngCore;

                let mut entropy = [0u8; 16];
                rand::rng().fill_bytes(&mut entropy);
                let phrase = Mnemonic::from_entropy_in(Language::English, &entropy)
                    .expect("valid entropy")
                    .to_string();

                wallet.set_mnemonic(phrase.clone(), passphrase);

                let fingerprint = wallet
                    .keypair()
                    .map(|kp| master_fingerprint(&kp))
                    .unwrap_or_default();

                WorkerResp::WalletCreated {
                    mnemonic: phrase,
                    master_fingerprint: fingerprint,
                }
            }
            WorkerCmd::UnlockWallet { mnemonic, passphrase } => {
                use bip39::{Language, Mnemonic};
                if Mnemonic::parse_in(Language::English, &mnemonic).is_err() {
                    return WorkerResp::SendResult(Err("Invalid mnemonic".into()));
                }
                wallet.set_mnemonic(mnemonic, passphrase);
                let fingerprint = wallet
                    .keypair()
                    .map(|kp| master_fingerprint(&kp))
                    .unwrap_or_default();
                WorkerResp::WalletUnlocked { fingerprint }
            }
            WorkerCmd::LockWallet => {
                *wallet = WalletState::new();
                WorkerResp::WalletLocked
            }
            WorkerCmd::GetAddresses { count } => {
                let addresses: Vec<String> = (0..count)
                    .filter_map(|i| wallet.derive_address(i))
                    .collect();
                WorkerResp::Addresses(addresses)
            }
            WorkerCmd::SendFromWallet { to_address, amount } => {
                if wallet.is_locked() {
                    return WorkerResp::SendResult(Err("Wallet locked".into()));
                }
                let to_addr = match Address::parse(&to_address) {
                    Ok(a) => a,
                    Err(e) => {
                        return WorkerResp::SendResult(Err(format!("Invalid address: {e}")));
                    }
                };
                let keypair = match wallet.keypair() {
                    Some(k) => k,
                    None => return WorkerResp::SendResult(Err("No keypair".into())),
                };
                match node.send_with(&keypair, amount, to_addr) {
                    Ok(sent) => {
                        let tx_id = sent.tx;
                        let _ = events.send(NodeEvent::TxAccepted {
                            tx_id: tx_id.to_string(),
                        });
                        WorkerResp::SendResult(Ok(tx_id))
                    }
                    Err(e) => WorkerResp::SendResult(Err(e.to_string())),
                }
            }
            WorkerCmd::GetBalance { address } => {
                let addr = match Address::parse(&address) {
                    Ok(a) => a,
                    Err(_) => return WorkerResp::Balance(0),
                };
                let balance = node.balance(&addr).unwrap_or(0) as u64;
                WorkerResp::Balance(balance)
            }
            WorkerCmd::GetHistory { address, max_blocks } => {
                let addr = match Address::parse(&address) {
                    Ok(a) => a,
                    Err(_) => return WorkerResp::History(vec![]),
                };
                let history = node.history_of(&addr, max_blocks).unwrap_or_default();
                let wallet_events: Vec<WalletEvent> = history
                    .into_iter()
                    .map(|e| {
                        let tx_id = e.tx_id.to_string();
                        let block_id = e.block_id.to_string();
                        let asset_id = e.asset_id.map(|a| a.to_hex());
                        let amount = e.amount;
                        let address = addr.to_string();
                        match e.direction {
                            kovanica_node::WalletDirection::Received => WalletEvent::Received {
                                tx_id,
                                block_id,
                                amount,
                                asset_id,
                                address,
                            },
                            kovanica_node::WalletDirection::Sent => WalletEvent::Sent {
                                tx_id,
                                block_id,
                                amount,
                                asset_id,
                                address,
                            },
                        }
                    })
                    .collect();
                WorkerResp::History(wallet_events)
            }
            WorkerCmd::Shutdown => WorkerResp::Ok,
        }
    }

    fn save_snapshot(
        node: &mut Node,
        datadir: &DataDir,
        events: &broadcast::Sender<NodeEvent>,
    ) -> Result<(), NodeError> {
        // TODO: Node does not expose a snapshot-write API yet; emit the event so
        // the UI reflects the request without blocking the worker loop.
        let count = node.block_count().unwrap_or(0);
        let _ = events.send(NodeEvent::SnapshotSaved {
            path: datadir.snapshot_path().display().to_string(),
            block_count: count as u64,
        });
        Ok(())
    }

    fn save_checkpoint(
        node: &mut Node,
        datadir: &DataDir,
        events: &broadcast::Sender<NodeEvent>,
    ) -> Result<(), NodeError> {
        node.save_checkpoint(datadir.checkpoint_path().to_string_lossy().as_ref())?;
        let count = node.block_count().unwrap_or(0);
        let _ = events.send(NodeEvent::CheckpointSaved {
            path: datadir.checkpoint_path().display().to_string(),
            block_count: count as u64,
        });
        Ok(())
    }
}

/// Master key fingerprint: first byte of the BLAKE3 hash of the public key.
fn master_fingerprint(keypair: &KeyPair) -> String {
    let pk = keypair.address().to_hex();
    format!("{:02x}", blake3::hash(pk.as_bytes()).as_bytes()[0])
}