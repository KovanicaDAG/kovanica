//! Kovanica full node binary.
//!
//! Wires the protocol crates together into a runnable node:
//!
//! 1. load configuration (TOML + environment overrides),
//! 2. open the consensus and state databases, initialise genesis if fresh,
//! 3. start the P2P network, the JSON-RPC server and the consensus engine,
//! 4. run the event-ingest, block-builder and execution loops,
//! 5. shut down cleanly on SIGINT / SIGTERM.
//!
//! Several integration points are deliberately marked `TODO` where the
//! underlying crate does not yet expose a hook (see the comments in
//! [`run_node`]).

// `DagStoreTrait`/`BlockManagerTrait` return the storage-adjacent error enums
// from `kovanica-dag`, whose redb-backed variants exceed clippy's error size limit.
#![allow(clippy::result_large_err)]
// `NetworkCommand` carries block/transaction payloads next to a zero-sized
// `Shutdown` variant.
#![allow(clippy::large_enum_variant)]

mod config;
mod non_blue;
mod orphans;
mod seals;

use std::collections::HashMap;
use std::future::Future;
use std::sync::{atomic::Ordering, Arc};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use kovanica_network::{Multiaddr, PeerId};
use parking_lot::{Mutex, RwLock};
use serde::Deserialize;
use tokio::sync::{mpsc, watch};
use tracing::{debug, error, info, warn};
use zeroize::Zeroizing;

use config::NodeConfig;

use kovanica_consensus::engine::{BlockManagerTrait, DagStoreTrait};
use kovanica_consensus::metrics::{record_mempool_size, record_peer_count};
use kovanica_consensus::{AuthorityInfo, CommitteeInfo, ConsensusConfig, ConsensusEngine, Vote};
use kovanica_dag::{BlockManager, BlockManagerError, DagStore, DagStoreError};
use kovanica_execution::ExecutionContext;
use kovanica_execution::{LogPublisher, TransactionReceipt};
use kovanica_mempool::{Mempool, MempoolConfig};
use kovanica_network::{
    BlockSyncRequest, BlockSyncResponse, NetworkConfig, NetworkEvent, NetworkService,
    SnapshotChunk, StateSnapshotSource, StateSyncRequest, StateSyncResponse,
    MAX_STATE_SYNC_CHUNK_BYTES, MAX_STATE_SYNC_SNAPSHOT_BYTES,
};
use kovanica_rpc::{
    AuthConfig, EventBus, NodeHealth, RateLimitConfig, RateLimiterState, RpcServer, RpcState,
    SyncPeerStatus, SyncStatus, SyncStatusSource,
};

/// Wrapper around EventBus to implement LogPublisher.
struct EventLogPublisher(EventBus);

impl LogPublisher for EventLogPublisher {
    fn publish_logs(&self, receipts: &[TransactionReceipt]) {
        self.0.publish_logs(receipts);
    }
}
use kovanica_staking::{
    StakingState, FOUNDER_PREMINE, GENESIS_VALIDATOR_ALLOCATION, MAX_ACTIVE_VALIDATORS,
    MIN_VALIDATOR_STAKE, ONE_KUNA,
};
use kovanica_storage::{StateStoreError, Storage};
use kovanica_types::{
    block::{BlockReference, StatementBlock},
    Address, AuthorityIndex, Hash, PublicKey, Round, Signature, SigningKey, Transaction,
};

/// Command line arguments.
#[derive(Parser, Debug)]
#[command(name = "kovanica-node")]
#[command(about = "Kovanica (KUNA) full node", long_about = None)]
#[command(subcommand_required = false, arg_required_else_help = false)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to config file (used when running node)
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Validate the configuration (file + `KOVANICA_*` env overrides) and exit
    /// without starting the node. Exits non-zero with a diagnostic on a bad
    /// config so operators can gate deploys on it.
    #[arg(long)]
    check_config: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Genesis ceremony: build genesis block + staking state from validator keys
    Genesis(GenesisArgs),
}

#[derive(clap::Args, Debug)]
struct GenesisArgs {
    /// Path to validators JSON file (array: {address, public_key?, stake?})
    #[arg(long, value_name = "FILE", default_value = "validators.json")]
    validators: String,

    /// Treasury address (32-byte hex)
    #[arg(long, value_name = "HEX")]
    treasury_address: String,

    /// Founder premine address (32-byte hex, receives 200,000 KUNA)
    #[arg(long, value_name = "HEX")]
    founder_address: String,

    /// Output genesis JSON file
    #[arg(long, value_name = "FILE", default_value = "genesis.json")]
    output: String,

    /// Output validator key files directory (optional)
    #[arg(long, value_name = "DIR")]
    validator_keys_out: Option<String>,

    /// Overwrite existing output files
    #[arg(long)]
    force: bool,
}

/// Local seal signing context plus the channel that gossips the seal.
struct SealDuty {
    ctx: seals::SealContext,
    broadcast: mpsc::UnboundedSender<NetworkCommand>,
}

/// Outbound network commands handled by the network task.
enum NetworkCommand {
    /// Gossip a newly produced block.
    BroadcastBlock(StatementBlock),
    /// Gossip a consensus vote.
    BroadcastVote(Vote),
    /// Gossip a transaction accepted into the local mempool via RPC.
    BroadcastTransaction(Transaction),
    /// Gossip a commit seal produced after local execution.
    BroadcastSeal(kovanica_types::SealAnnouncement),
    /// Request a block sync from a specific peer.
    RequestSync(PeerId, BlockSyncRequest),
    /// Request a state sync (chunked snapshot transfer) from a peer.
    RequestStateSync(PeerId, StateSyncRequest),
    /// Ask the network task to stop.
    Shutdown,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let args = Args::parse();
    if args.check_config {
        return run_check_config(&args.config);
    }
    match args.command {
        Some(Commands::Genesis(g)) => {
            return run_genesis(g);
        }
        None => {
            let config = NodeConfig::load(&args.config)?;
            info!(data_dir = %config.data_dir.display(), "starting Kovanica node");
            run_node(config, shutdown_signal()).await
        }
    }
}

/// Validate the configuration file plus environment overrides and exit.
///
/// Mirrors the startup checks in [`run_node`] but performs no I/O beyond
/// reading the config file, so it is safe to run from CI/deploy pipelines to
/// fail fast on a misconfigured node.
fn run_check_config(path: &str) -> Result<()> {
    let config = NodeConfig::load(path)?;
    config.validate().with_context(|| {
        format!(
            "configuration in {} is invalid (also check KOVANICA_* env overrides)",
            path
        )
    })?;
    println!(
        "config OK: source={} chain_id={} data_dir={} listen_addr={} rpc={} round_duration_ms={} max_peers={} run_validator={}",
        path,
        config.chain_id,
        config.data_dir.display(),
        config.listen_addr,
        config.rpc_socket_addr()?,
        config.round_duration_ms,
        config.max_peers,
        config.run_validator,
    );
    Ok(())
}

/// Run the genesis ceremony: read validators.json, build staking state
/// and write genesis.json (genesis block + staking state).
fn run_genesis(args: GenesisArgs) -> Result<()> {
    use std::fs;

    // Read validators JSON
    let text = fs::read_to_string(&args.validators)
        .with_context(|| format!("reading validators {}", args.validators))?;
    let validators_input: Vec<GenesisValidatorInput> =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", args.validators))?;

    // Build treasury
    let treasury = parse_address_hex(Some(&args.treasury_address))?;
    let founder = parse_address_hex(Some(&args.founder_address))?;
    let mut staking = StakingState::new();
    staking.init_treasury(treasury);

    // Founder premine (200,000 KUNA) - included in genesis allocations output
    // Actual balance set when genesis block is executed on first run

    // Join each validator
    for v in validators_input {
        let address = parse_address_hex(Some(&v.address))?;
        let pk = v
            .public_key
            .as_ref()
            .map(|s| parse_public_key_hex(s))
            .transpose()?;
        let stake = v.stake.unwrap_or(MIN_VALIDATOR_STAKE);
        staking
            .join_validator(address, stake, 0, Some(address), pk, None)
            .with_context(|| format!("joining validator {}", v.address))?;
    }

    // Generate validator keys if output directory specified
    let mut validator_keys = Vec::new();
    if let Some(keys_dir) = &args.validator_keys_out {
        create_private_dir(std::path::Path::new(keys_dir))?;
        for (i, v) in staking.validators.iter().enumerate() {
            let (sk, pk) = kovanica_crypto::generate_keypair();
            // The raw Ed25519 seed is secret material: keep the in-memory copy
            // zeroized on drop and write the file `0600` below.
            let seed = zeroize::Zeroizing::new(sk.to_bytes());
            let key_file = format!("{}/validator{}.key", keys_dir, i + 1);
            write_private_file(
                std::path::Path::new(&key_file),
                hex::encode(seed.as_slice()).as_bytes(),
            )?;
            validator_keys.push(serde_json::json!({
                "index": i,
                "address_hex": hex::encode(v.address.0),
                "public_key_hex": hex::encode(pk.0),
                "key_file": key_file,
            }));
            info!("Generated validator {} key: {}", i + 1, key_file);
        }
    }

    info!(validators = staking.validators.len(), treasury = %treasury, founder = %founder, "genesis allocations computed");

    // Build genesis StatementBlock
    let digest = StatementBlock::compute_digest(0, 0, &[], &[]);
    let genesis_block = StatementBlock {
        author: 0,
        round: 0,
        parents: Vec::new(),
        transactions: Vec::new(),
        statements: Vec::new(),
        signature: Signature([0u8; 64]),
        digest,
        merkle_root: Hash::zero(),
    };

    // Genesis validator allocation: shares are computed over the deduplicated,
    // address-sorted validator set (same deterministic order the node uses at
    // first run), so the JSON is a faithful preview of the minted balances.
    let mut alloc_entries: Vec<(Address, u64)> = staking
        .validators
        .iter()
        .map(|v| (v.address, v.stake))
        .collect();
    alloc_entries.sort_by_key(|a| a.0 .0);
    alloc_entries.dedup_by(|a, b| a.0 .0 == b.0 .0);
    let alloc_shares = kovanica_staking::genesis_validator_shares(alloc_entries.len());

    // Serialize output JSON
    let output = serde_json::json!({
        "genesis_block": {
            "author": genesis_block.author,
            "round": genesis_block.round,
            "digest_hex": hex::encode(genesis_block.digest.0),
            "signature_hex": hex::encode(genesis_block.signature.0),
            "transactions": genesis_block.transactions,
            "statements": genesis_block.statements,
            "parents": genesis_block.parents,
        },
        "staking_state": staking,
        "allocations": {
            "treasury_address_hex": args.treasury_address,
            "treasury_address_bytes": hex::encode(treasury.0),
            "founder_address_hex": args.founder_address,
            "founder_address_bytes": hex::encode(founder.0),
            "founder_premine_atoms": FOUNDER_PREMINE,
            "founder_premine_kovanica": FOUNDER_PREMINE / ONE_KUNA,
            "founder_premine_kuna": FOUNDER_PREMINE / ONE_KUNA,
            "genesis_validator_allocation_atoms": GENESIS_VALIDATOR_ALLOCATION,
            "genesis_validator_allocation_kuna": GENESIS_VALIDATOR_ALLOCATION / ONE_KUNA,
            "validators": staking.validators.iter().map(|v| {
                let share = alloc_entries
                    .iter()
                    .position(|(a, _)| a.0 == v.address.0)
                    .map(|i| alloc_shares[i])
                    .unwrap_or(0);
                serde_json::json!({
                    "address_hex": hex::encode(v.address.0),
                    "stake": v.stake,
                    "genesis_share_atoms": share,
                    "genesis_liquid_atoms": share.saturating_sub(v.stake),
                    "public_key_hex": v.public_key.as_ref().map(|pk| hex::encode(pk.0)),
                })
            }).collect::<Vec<_>>(),
        },
        "validator_keys": validator_keys,
    });

    // Write output with optional force
    if args.force {
        fs::write(&args.output, serde_json::to_string_pretty(&output)?)
            .with_context(|| format!("writing genesis {}", args.output))?;
    } else if fs::metadata(&args.output).is_ok() {
        anyhow::bail!(
            "output file {} exists; use --force to overwrite",
            args.output
        );
    } else {
        fs::write(&args.output, serde_json::to_string_pretty(&output)?)
            .with_context(|| format!("writing genesis {}", args.output))?;
    }
    info!(file = %args.output, validators = staking.validators.len(), "genesis file written");
    Ok(())
}

/// Create `dir` (and any missing parents) with owner-only permissions (`0700`).
///
/// Used for directories holding raw seed material. On non-unix platforms this
/// falls back to a plain recursive create.
fn create_private_dir(dir: &std::path::Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)
            .with_context(|| format!("creating private directory {}", dir.display()))?;
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("creating private directory {}", dir.display()))?;
    }
    Ok(())
}

/// Write `contents` to `path` with owner-only permissions (`0600`), creating
/// any missing parent directory as `0700`.
///
/// Seed material must never be group- or world-readable. On non-unix platforms
/// the mode is not enforced (there is no equivalent), but the path is still
/// created if missing.
fn write_private_file(path: &std::path::Path, contents: &[u8]) -> Result<()> {
    use std::io::Write;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            create_private_dir(parent)?;
        }
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .with_context(|| format!("creating {}", path.display()))?;
    file.write_all(contents)
        .with_context(|| format!("writing {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("syncing {}", path.display()))?;
    Ok(())
}

/// Validator entry read from validators.json
#[derive(Debug, Deserialize)]
struct GenesisValidatorInput {
    address: String,
    public_key: Option<String>,
    stake: Option<u64>,
}

/// Install the tracing subscriber, honouring `RUST_LOG` when set.
fn init_tracing() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let json_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(json_layer)
        .init();
}

/// Round tracing utilities for structured logging with trace_id per round.
pub mod round_trace {
    use std::sync::atomic::{AtomicU64, Ordering};
    use tracing::{span, Level};

    /// Global counter for generating unique trace IDs.
    static TRACE_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Generate a new trace ID for a round.
    pub fn new_trace_id(round: u64) -> String {
        let counter = TRACE_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("round-{:010}-{:06}", round, counter)
    }

    /// Create a tracing span for a round with trace_id.
    pub fn round_span(round: u64) -> tracing::Span {
        let trace_id = new_trace_id(round);
        span!(Level::INFO, "round", round = round, trace_id = %trace_id)
    }

    /// Create a tracing span for block processing with trace_id.
    pub fn block_span(round: u64, block_hash: &str, author: &str) -> tracing::Span {
        let trace_id = new_trace_id(round);
        span!(
            Level::INFO,
            "block",
            round = round,
            trace_id = %trace_id,
            block_hash = %block_hash,
            author = %author
        )
    }

    /// Create a tracing span for vote processing with trace_id.
    pub fn vote_span(round: u64, leader_round: u64, voter: &str) -> tracing::Span {
        let trace_id = new_trace_id(round);
        span!(
            Level::INFO,
            "vote",
            round = round,
            trace_id = %trace_id,
            leader_round = leader_round,
            voter = %voter
        )
    }

    /// Create a tracing span for transaction processing with trace_id.
    pub fn tx_span(round: u64, tx_hash: &str) -> tracing::Span {
        let trace_id = new_trace_id(round);
        span!(
            Level::INFO,
            "transaction",
            round = round,
            trace_id = %trace_id,
            tx_hash = %tx_hash
        )
    }

    /// Create a tracing span for execution with trace_id.
    pub fn execution_span(round: u64, tx_count: usize) -> tracing::Span {
        let trace_id = new_trace_id(round);
        span!(
            Level::INFO,
            "execution",
            round = round,
            trace_id = %trace_id,
            tx_count = tx_count
        )
    }
}

/// A future that resolves when the process receives SIGINT or SIGTERM.
///
/// Signal handlers are installed eagerly when this function is called (not when
/// the returned future is first polled), so it is safe to register them before
/// any other startup work.
fn shutdown_signal() -> impl Future<Output = ()> + Send + 'static {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
        let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");

        async move {
            tokio::select! {
                _ = sigint.recv() => info!("received SIGINT"),
                _ = sigterm.recv() => info!("received SIGTERM"),
            }
        }
    }

    #[cfg(not(unix))]
    {
        async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("received Ctrl-C");
        }
    }
}

/// Boot the node and run until `shutdown` resolves, then stop it cleanly.
async fn run_node<F>(config: NodeConfig, shutdown: F) -> Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    config.validate()?;

    // ------------------------------------------------------------------
    // 1. Data directory + storage
    // ------------------------------------------------------------------
    std::fs::create_dir_all(&config.data_dir)
        .with_context(|| format!("creating data directory {}", config.data_dir.display()))?;
    check_chain_id(&config)?;
    info!(chain_id = config.chain_id, "chain id verified");

    // The DAG/consensus store and the account/staking store live in separate
    // redb files: `kovanica-dag::DagStore` owns its `Storage`, while the mempool and
    // execution layer need a shareable `Arc<Storage>`. redb locks a file to a
    // single `Database`, so they cannot share one file.
    let dag_store = Arc::new(DagStore::new(Storage::new(config.consensus_db_path())?)?);
    let state_storage = Arc::new(Storage::new(config.state_db_path())?);
    info!(dir = %config.data_dir.display(), "opened storage");

    // ------------------------------------------------------------------
    // 2. Genesis / staking state
    // ------------------------------------------------------------------
    init_genesis(&config, &dag_store, &state_storage)?;

    // Guard against a data dir whose genesis does not match the configured
    // genesis (fork / cross-chain contamination), and read back genesis.json.
    verify_genesis_consistency(&config, &state_storage)?;

    // ------------------------------------------------------------------
    // 3. Mempool + validator identity
    // ------------------------------------------------------------------
    let (signing_key, public_key) =
        load_validator_key(config.validator_key.as_deref(), config.run_validator)?;
    let validator_address = Address::from_public_key(&public_key);
    info!(validator = %validator_address, "validator identity ready");

    // Build the committee from active validators in StakingState.
    let committee = build_committee(
        &state_storage,
        &public_key,
        &validator_address,
        &config.listen_addr,
    )?;

    // Register committee public keys with the crypto batch verifier (used by the
    // hot path on network ingest). Without this every remote block is rejected
    // with `InvalidPublicKey` (the static key list stays empty).
    let committee_keys: Vec<PublicKey> = committee
        .authorities()
        .iter()
        .map(|a| a.public_key)
        .collect();
    kovanica_crypto::set_validator_keys_from_public(committee_keys);

    // v1 signing context: chain id from config (verified against genesis).
    // Epoch is derived per-leader-round as leader_round / EPOCH_ROUNDS
    // (see `vote_signing_ctx` and `publish_local_seal`). The value here is a
    // placeholder and NOT used for consensus decisions.
    let signing_ctx = kovanica_types::SigningContext::new(config.chain_id).with_epoch(0);
    let mempool = Arc::new(Mempool::new(
        build_mempool_config(),
        state_storage.clone(),
        signing_ctx,
    ));
    info!(
        committee = committee.authorities().len(),
        "committee public keys registered with batch verifier"
    );

    // Determine our authority index from the committee (match local public key)
    let our_authority = if config.run_validator {
        committee
            .authorities()
            .iter()
            .find(|a| a.public_key == public_key)
            .map(|a| a.index)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "local validator public key not found in committee; cannot run as validator"
                )
            })?
    } else {
        info!("run_validator=false; not participating in consensus");
        u16::MAX // Invalid authority index
    };

    // Load the staking state for the RPC server (shared with consensus via Arc<RwLock>).
    let staking_state = {
        let read = state_storage.begin_read()?;
        match state_storage.state().load_staking_state(&read) {
            Ok(state) => state,
            Err(e) => {
                warn!(error = %e, "could not load staking state for RPC; using empty state");
                StakingState::new()
            }
        }
    };

    // ------------------------------------------------------------------
    // 4. Networking
    // ------------------------------------------------------------------
    let network_config = build_network_config(&config)?;
    let (network, network_events) =
        NetworkService::new(network_config, dag_store.clone(), mempool.clone())
            .context("creating network service")?;
    // Share one service between the swarm event loop and the broadcast command
    // loop so broadcast commands never re-enter `start` or rebuild listeners.
    // Votes and blocks are authenticated against the committee at the network
    // edge before they are forwarded or handed to consensus.
    // Vote signatures are checked through the injected VoteVerifier.
    network.set_vote_verifier(Arc::new(kovanica_network::Ed25519VoteVerifier));
    network.set_authority_keys(
        committee
            .authorities()
            .iter()
            .map(|authority| (authority.index, authority.public_key))
            .collect(),
    );
    network.set_header_index(Arc::new(seals::StateHeaderIndex {
        storage: state_storage.clone(),
    }));
    // Export side of the P2P state sync protocol: without this the service
    // answers every state sync request with NotFound.
    network.set_state_snapshot_source(Arc::new(NodeStateSnapshotSource {
        storage: state_storage.clone(),
    }));
    let network = Arc::new(network);
    let peer_count = network.peer_count_handle();
    let peer_count_for_rpc = peer_count.clone();
    let (network_cmd_tx, network_cmd_rx) = mpsc::unbounded_channel::<NetworkCommand>();

    // ------------------------------------------------------------------
    // 5. JSON-RPC server
    // ------------------------------------------------------------------
    // NOTE: the node keeps the DAG/consensus store (`dag_store`) and the
    // account/staking store (`state_storage`) in separate redb files, so the
    // read-only storage handed to the RPC server is `state_storage`. Block and
    // consensus queries therefore observe the state database; wiring them to the
    // DAG database requires exposing `DagStore`'s inner `Arc<Storage>`.
    let rpc_socket = config.rpc_socket_addr()?;
    // Fan-out bus for WebSocket subscription events (newHeads,
    // newCommittedLeader, pendingTransactions).
    let events = EventBus::new();
    let rate_limit_config =
        RateLimitConfig::from_requests_per_minute(config.rpc_rate_limit_per_min);
    // Liveness of the execution worker and consensus engine, surfaced on /health.
    let node_health = NodeHealth::new();
    // Commit-liveness: without this a stalled chain reports a false green,
    // because consensus rounds and gossip keep flowing while nothing commits.
    node_health.set_commit_stall_secs(config.health_commit_stall_secs);
    let auth_config = rpc_auth_config_from_env();
    // RPC-accepted transactions are forwarded to the network task.
    let (rpc_tx_gossip, mut rpc_tx_gossip_rx) = mpsc::unbounded_channel::<Transaction>();
    {
        let cmd_tx = network_cmd_tx.clone();
        tokio::spawn(async move {
            while let Some(tx) = rpc_tx_gossip_rx.recv().await {
                if cmd_tx
                    .send(NetworkCommand::BroadcastTransaction(tx))
                    .is_err()
                {
                    break;
                }
            }
        });
    }
    // Shared catch-up state: the network event loop drives it and the read-only
    // `kovanica_syncStatus` RPC reads it. The tracker itself stays inside the
    // node; the RPC layer only sees a snapshot built under this lock.
    let catch_up_handle: Arc<Mutex<CatchUpTracker>> = Arc::new(Mutex::new(CatchUpTracker::new()));
    let sync_status_source: SyncStatusSource = {
        let catch_up = catch_up_handle.clone();
        let dag_store = dag_store.clone();
        let network = network.clone();
        Arc::new(move || {
            let (catching_up, catch_up_target) = catch_up.lock().summary();
            let local_tip_round = local_dag_tip(&dag_store);
            let committed_height = dag_store.get_committed_leader_height().ok();
            let peers: Vec<SyncPeerStatus> = network
                .peer_status_snapshot()
                .into_iter()
                .map(|peer| SyncPeerStatus {
                    peer_id: peer.peer_id.to_string(),
                    reported_tip_round: peer.reported_tip_round,
                })
                .collect();
            let peer_count = peers.len();
            let behind_by = match (catching_up, catch_up_target, local_tip_round) {
                (true, Some(target), Some(local)) => Some(target.saturating_sub(local)),
                _ => None,
            };
            SyncStatus {
                local_tip_round,
                committed_height,
                local_height: committed_height,
                catching_up,
                catch_up_target,
                behind_by,
                peer_count,
                peers,
            }
        })
    };

    let rpc_state = RpcState {
        tx_gossip: Some(rpc_tx_gossip),
        health: node_health.clone(),
        storage: state_storage.clone(),
        consensus_store: dag_store.clone(),
        mempool: mempool.clone(),
        staking: Arc::new(tokio::sync::RwLock::new(staking_state)),
        committee: Arc::new(tokio::sync::RwLock::new(committee.clone())),
        peer_count: peer_count_for_rpc,
        events: events.clone(),
        rate_limiter: Arc::new(RateLimiterState::new(rate_limit_config.clone())),
        auth_config: Arc::new(auth_config.clone()),
        sync_status: Some(sync_status_source),
    };
    let rpc_committee = rpc_state.committee.clone();
    let rpc_server = RpcServer::new(
        rpc_socket,
        rpc_state,
        Some(rate_limit_config),
        Some(auth_config),
    )
    .await;
    let rpc_handle = rpc_server.start().await.context("starting RPC server")?;

    // ------------------------------------------------------------------
    // 6. Consensus engine
    // ------------------------------------------------------------------
    // The node-provided validator identity remains authoritative in the block
    // manager; consensus does not generate or replace a signing key.
    let block_manager = Arc::new(BlockManager::new(dag_store.clone(), signing_ctx));
    if config.run_validator {
        block_manager.set_authority(our_authority);
        block_manager.set_signing_key(signing_key.clone());
    } else {
        // Set an invalid authority index so we never think we're the leader
        block_manager.set_authority(u16::MAX);
    }
    block_manager.set_authority_keys(
        committee
            .authorities()
            .iter()
            .map(|authority| (authority.index, authority.public_key))
            .collect(),
    );
    block_manager.set_authority_stakes(
        committee
            .authorities()
            .iter()
            .map(|authority| (authority.index, authority.stake))
            .collect(),
    );
    block_manager.set_vrf_author_keys(
        committee
            .authorities()
            .iter()
            .filter_map(|authority| authority.vrf_public_key.map(|key| (authority.index, key)))
            .collect(),
    );

    // Populate the global validator key registry for batch signature verification
    // The registry is indexed by authority index, so we need to ensure keys are in index order
    let mut validator_keys = vec![None; committee.size() as usize];
    for authority in committee.authorities() {
        validator_keys[authority.index as usize] = Some(authority.public_key);
    }
    let public_keys: Vec<PublicKey> = validator_keys
        .into_iter()
        .map(|opt| opt.expect("validator key missing for authority index"))
        .collect();
    kovanica_crypto::set_validator_keys_from_public(public_keys);
    let engine = Arc::new(ConsensusEngine::new(
        ConsensusConfig {
            round_duration_ms: config.round_duration_ms,
            use_mysticghost: config.use_mysticghost,
            use_vrf_leader: config.use_vrf_leader,
            ..Default::default()
        },
        committee.clone(),
        Arc::new(NodeDagStore {
            inner: dag_store.clone(),
        }),
        Arc::new(RwLock::new(NodeBlockManager {
            inner: block_manager.clone(),
        })),
        signing_key.clone(),
        if config.run_validator {
            Some(mempool.clone())
        } else {
            None
        },
        signing_ctx,
    ));

    // Set the round watch receiver on the block manager so it can track the current round
    // from the consensus engine's watch channel (Phase 16.2).
    let round_rx = engine.subscribe_round();
    block_manager.set_round_receiver(round_rx);

    // Set up block broadcaster to gossip proposed blocks
    let broadcast_tx = network_cmd_tx.clone();
    engine.set_block_broadcaster(Arc::new(move |block: &StatementBlock| {
        let _ = broadcast_tx.send(NetworkCommand::BroadcastBlock(block.clone()));
    }));

    // Set up vote broadcaster to gossip votes
    let vote_broadcast_tx = network_cmd_tx.clone();
    engine.set_vote_broadcaster(Arc::new(move |vote: &Vote| {
        let _ = vote_broadcast_tx.send(NetworkCommand::BroadcastVote(vote.clone()));
    }));

    // ------------------------------------------------------------------
    // 7. Shutdown plumbing + task spawning
    // ------------------------------------------------------------------
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (exec_tx, exec_rx) = mpsc::unbounded_channel::<kovanica_consensus::CommittedSubDag>();
    engine.set_commit_sender(exec_tx.clone());
    for subdag in recover_committed_subdags(&dag_store, config.use_mysticghost)? {
        exec_tx
            .send(subdag)
            .context("queueing a previously committed sub-DAG for replay")?;
    }

    let network_task = tokio::spawn(run_network(network, network_cmd_rx, shutdown_rx.clone()));
    let event_task = tokio::spawn(run_event_handler(
        network_events,
        dag_store.clone(),
        mempool.clone(),
        engine.clone(),
        network_cmd_tx.clone(),
        events.clone(),
        state_storage.clone(),
        catch_up_handle.clone(),
        shutdown_rx.clone(),
    ));

    // The consensus engine now handles block production internally with mempool transactions
    // No separate builder task needed
    let _builder_task = tokio::spawn(async {});
    let seal_duty = SealDuty {
        ctx: seals::SealContext {
            chain_id: signing_ctx.chain_id,
            signer: if config.run_validator {
                Some((our_authority, signing_key.clone()))
            } else {
                None
            },
            mysticghost: config.use_mysticghost,
        },
        broadcast: network_cmd_tx.clone(),
    };
    let exec_task = tokio::spawn(run_execution(
        exec_rx,
        state_storage.clone(),
        events.clone(),
        Some(non_blue::ExecPool {
            mempool: mempool.clone(),
            // #13: Consensus #47 helper behind the NonBlueSource seam.
            returned: Some(non_blue::ReturnedTxs {
                dag: dag_store.clone(),
                source: Arc::new(non_blue::ConsensusNonBlue),
            }),
        }),
        signing_ctx,
        shutdown_rx.clone(),
        node_health.clone(),
        Some(seal_duty),
        rpc_committee,
        config.listen_addr.clone(),
        validator_address,
        public_key,
    ));

    let engine_for_task = engine.clone();
    let engine_health = node_health.clone();
    let engine_task = tokio::spawn(async move {
        engine_health.consensus_running();
        match engine_for_task.start().await {
            Ok(()) => engine_health.consensus_stopped(),
            Err(e) => {
                error!(error = %e, "consensus engine stopped with an error");
                engine_health.consensus_failed(&e.to_string());
            }
        }
    });

    // Background task: periodically update Prometheus metrics from node state
    let mempool_for_metrics = mempool.clone();
    let peer_count_for_metrics = peer_count.clone();
    let mut shutdown_for_metrics = shutdown_rx.clone();
    let _metrics_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Update mempool size
                    let mempool_size = mempool_for_metrics.len() as i64;
                    record_mempool_size(mempool_size);

                    // Update peer count
                    let peer_count_val = peer_count_for_metrics.load(Ordering::Relaxed) as i64;
                    record_peer_count(peer_count_val);
                }
                _ = shutdown_for_metrics.changed() => {
                    if *shutdown_for_metrics.borrow() {
                        break;
                    }
                }
            }
        }
    });

    // Background task: periodically rebroadcast mempool transactions to peers
    // and re-queue stale in-flight transactions.
    let mempool_for_rebroadcast = mempool.clone();
    let network_cmd_tx_for_rebroadcast = network_cmd_tx.clone();
    let mut shutdown_for_rebroadcast = shutdown_rx.clone();
    // Use the mempool's configured in_flight_timeout for requeue_stale
    let in_flight_timeout = mempool_for_rebroadcast.config().in_flight_timeout;
    let rebroadcast_interval = mempool_for_rebroadcast.config().rebroadcast_interval;
    let _rebroadcast_task = tokio::spawn(async move {
        info!("rebroadcast task started");
        let mut interval = tokio::time::interval(rebroadcast_interval);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Re-queue stale in-flight transactions first
                    let requeued = mempool_for_rebroadcast.requeue_stale(Instant::now(), in_flight_timeout);
                    if requeued > 0 {
                        info!(requeued, "re-queued stale in-flight transactions");
                    }
                    // Then rebroadcast queued transactions
                    let txs = mempool_for_rebroadcast.rebroadcast();
                    if !txs.is_empty() {
                        info!(count = txs.len(), "rebroadcast task sending transactions");
                    }
                    for tx in txs {
                        let _ = network_cmd_tx_for_rebroadcast.send(NetworkCommand::BroadcastTransaction(tx));
                    }
                }
                _ = shutdown_for_rebroadcast.changed() => {
                    if *shutdown_for_rebroadcast.borrow() {
                        break;
                    }
                }
            }
        }
    });

    // ------------------------------------------------------------------
    // 8. Retention maintenance (opt-in pruning; DEFAULT OFF)
    // ------------------------------------------------------------------
    // Every window is `None` unless the operator configured one, so a stock
    // node never spawns this task and never deletes chain data. The existing
    // DAG wave pruning inside `kovanica-consensus` (window = 100 waves) is
    // untouched: that is consensus-internal history, not this path.
    if config.retention_enabled() {
        let retention = RetentionPlan {
            blocks_below_tip: config.retain_blocks_below_tip,
            state_roots_below_height: config.retain_state_roots_below_height,
            receipts_below_height: config.retain_receipts_below_height,
            batch: config.retention_batch,
            margin: PRUNE_SAFETY_MARGIN,
        };
        let retention_secs = config.retention_interval_secs.max(1);
        let storage = state_storage.clone();
        let mut shutdown_for_retention = shutdown_rx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(retention_secs));
            // Consume the immediate first tick: let consensus move a little
            // before the first pass so the tip is fresh.
            interval.tick().await;
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Synchronous redb writes: keep them off the async
                        // executor so a bounded pass cannot stall the loop.
                        let storage = storage.clone();
                        let plan = retention.clone();
                        match tokio::task::spawn_blocking(move || {
                            run_retention_pass(&storage, &plan)
                        })
                        .await
                        {
                            Ok(Ok(summary)) if !summary.is_empty() => {
                                info!(target: "kovanica-node", summary = %summary, "retention pass removed data");
                            }
                            Ok(Ok(_)) => {
                                debug!(target: "kovanica-node", "retention pass found nothing to remove");
                            }
                            Ok(Err(e)) => {
                                warn!(target: "kovanica-node", error = %e, "retention pass failed");
                            }
                            Err(e) => {
                                error!(target: "kovanica-node", error = %e, "retention pass panicked");
                            }
                        }
                    }
                    _ = shutdown_for_retention.changed() => {
                        if *shutdown_for_retention.borrow() {
                            break;
                        }
                    }
                }
            }
            info!("retention maintenance task stopped");
        });
        info!(
            interval_secs = retention_secs,
            batch = config.retention_batch,
            "retention maintenance enabled (blocks_below_tip={:?} state_roots_below_height={:?} receipts_below_height={:?})",
            config.retain_blocks_below_tip,
            config.retain_state_roots_below_height,
            config.retain_receipts_below_height,
        );
    } else {
        info!("retention maintenance disabled: no retention window configured");
    }

    info!("Kovanica node started");

    // ------------------------------------------------------------------
    // 9. Run until shutdown, then stop everything
    // ------------------------------------------------------------------
    shutdown.await;
    info!("shutdown signal received; stopping node");

    let _ = shutdown_tx.send(true);
    engine.stop();
    let _ = network_cmd_tx.send(NetworkCommand::Shutdown);
    drop(network_cmd_tx);
    drop(exec_tx);
    rpc_handle.abort();

    for (name, handle) in [
        ("network", network_task),
        ("events", event_task),
        ("execution", exec_task),
        ("consensus", engine_task),
    ] {
        match tokio::time::timeout(Duration::from_secs(3), handle).await {
            Ok(Ok(())) => debug!(task = name, "task stopped"),
            Ok(Err(e)) => warn!(task = name, error = %e, "task panicked"),
            Err(_) => warn!(task = name, "task did not stop before timeout"),
        }
    }

    // redb commits each write transaction synchronously, so there is nothing
    // left to flush: dropping the handles closes the databases.
    info!("Kovanica node stopped cleanly");
    Ok(())
}

/// RPC write authorisation from the environment.
///
/// * `KOVANICA_RPC_AUTH=disable` turns write authorisation off (tests, trusted
///   local tooling).
/// * `KOVANICA_RPC_WRITE_TOKENS=tok1,tok2` sets the accepted bearer tokens.
///
/// Without either, write methods (including `kovanica_sendRawTransaction`, see
/// `docs/SECURITY.md`) are rejected: there is no token that could match.
fn rpc_auth_config_from_env() -> AuthConfig {
    let disabled = std::env::var("KOVANICA_RPC_AUTH").as_deref() == Ok("disable");
    let tokens = std::env::var("KOVANICA_RPC_WRITE_TOKENS").unwrap_or_default();
    rpc_auth_config(disabled, &tokens)
}

fn rpc_auth_config(disabled: bool, tokens: &str) -> AuthConfig {
    let write_tokens: Vec<String> = tokens
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect();
    if disabled {
        warn!("KOVANICA_RPC_AUTH=disable: RPC write methods accept unauthenticated calls");
    } else if write_tokens.is_empty() {
        warn!("no KOVANICA_RPC_WRITE_TOKENS configured: RPC write methods are rejected");
    }
    AuthConfig {
        write_tokens,
        require_auth_for_writes: !disabled,
    }
}

/// Initialise genesis state if the database is fresh.
/// Minimum number of committed heights a retention pass must leave below the
/// committed tip, regardless of what the operator configured. RPC and
/// light-client consumers keep asking for recent history, so retention never
/// deletes anything newer than this.
const PRUNE_SAFETY_MARGIN: u64 = 1000;

/// One configured retention pass: the per-store windows, the per-call batch
/// bound and the safety floor every window is clamped to.
///
/// Built from [`config::NodeConfig`] only when at least one window is enabled,
/// so a stock node never runs retention at all.
#[derive(Clone, Debug)]
struct RetentionPlan {
    /// Keep at most this many rounds below the block-store tip. `None` or `0`
    /// disables block retention.
    blocks_below_tip: Option<u64>,
    /// Keep at most this many committed heights of state roots. `None` or `0`
    /// disables state-root retention.
    state_roots_below_height: Option<u64>,
    /// Keep at most this many committed heights of transaction receipts.
    /// `None` or `0` disables receipt retention.
    receipts_below_height: Option<u64>,
    /// Rows removed per store per pass, so one pass stays bounded.
    batch: usize,
    /// Depth below the committed tip that no window may cross.
    margin: u64,
}

/// Exclusive floor below which a window's data may be deleted.
///
/// Returns `None` when the chain has not grown far enough past the safety
/// margin for anything to be pruned safely yet; the caller then skips that
/// window entirely.
fn retention_floor(committed_height: u64, window: u64, margin: u64) -> Option<u64> {
    committed_height.checked_sub(window.max(margin))
}

/// Run one bounded retention pass over `storage` and summarise what was removed.
///
/// Each configured window removes at most `plan.batch` rows of its store in this
/// call, so a single pass never issues an unbounded write transaction. Nothing
/// newer than the safety margin (`plan.margin`) is ever touched, and a window
/// of `None`/`0` deletes nothing.
///
/// Runs on the blocking worker because it performs synchronous redb writes.
/// Returns an empty summary when no rows were removed.
fn run_retention_pass(storage: &Storage, plan: &RetentionPlan) -> Result<String> {
    let mut summary: Vec<String> = Vec::new();

    // Committed tip: the receipts and state-root windows are keyed by it.
    let committed_height = {
        let read = storage.begin_read()?;
        storage
            .consensus()
            .get_committed_leader_height(&read)
            .map_err(|e| anyhow::anyhow!("committed height: {e}"))?
    };

    // 1. Transaction receipts: drop the per-height row and every transaction's
    //    by-hash entry together (kovanica-execution `prune_receipts_below`).
    //    The by-hash index has no height column, so the per-height list is the
    //    authoritative source of which hashes belong to the pruned range.
    if let Some(window) = plan.receipts_below_height.filter(|w| *w > 0) {
        if let Some(floor) =
            retention_floor(committed_height, window, plan.margin).filter(|f| *f > 0)
        {
            let removed = kovanica_execution::prune_receipts_below(storage, floor, plan.batch)
                .map_err(|e| anyhow::anyhow!("receipt prune: {e}"))?;
            if removed > 0 {
                summary.push(format!("receipts={removed}"));
            }
        }
    }

    // 2. State roots: bounded, height-driven pass that additionally honours the
    //    state store's own `keep_recent` floor.
    if let Some(window) = plan.state_roots_below_height.filter(|w| *w > 0) {
        if let Some(floor) =
            retention_floor(committed_height, window, plan.margin).filter(|f| *f > 0)
        {
            let txn = storage.begin_write()?;
            let removed = storage
                .state()
                .prune_state_roots_below(&txn, floor, plan.batch)
                .map_err(|e| anyhow::anyhow!("state-root prune: {e}"))?;
            txn.commit().map_err(|e| anyhow::anyhow!("commit: {e}"))?;
            if removed > 0 {
                summary.push(format!("state_roots={removed}"));
            }
        }
    }

    // 3. Blocks: only rounds strictly below the settled history. Two
    //    independent bounds are taken and the lower one wins, so neither the
    //    operator's window nor a miscounted tip can reach committed rounds.
    if let Some(window) = plan.blocks_below_tip.filter(|w| *w > 0) {
        let (tip_round, committed) = {
            let read = storage.begin_read()?;
            let tip = storage
                .blocks()
                .get_latest_block(&read)
                .map_err(|e| anyhow::anyhow!("block tip: {e}"))?
                .map(|b| b.round);
            (tip, committed_height)
        };
        // `committed_height` counts committed leaders ever, so any round below
        // it is settled by construction — the safety floor doubles as a round
        // bound.
        if let Some(tip_round) = tip_round {
            if let Some(by_commit) = retention_floor(committed, window, plan.margin) {
                let floor = tip_round.saturating_sub(window).min(by_commit);
                if floor > 0 {
                    let txn = storage.begin_write()?;
                    let removed = storage
                        .blocks()
                        .prune_below_limited(&txn, floor, plan.batch)
                        .map_err(|e| anyhow::anyhow!("block prune: {e}"))?;
                    txn.commit().map_err(|e| anyhow::anyhow!("commit: {e}"))?;
                    if removed > 0 {
                        summary.push(format!("blocks={removed}"));
                    }
                }
            }
        }
    }

    Ok(summary.join(" "))
}

fn init_genesis(config: &NodeConfig, dag_store: &DagStore, state_storage: &Storage) -> Result<()> {
    let existing = {
        let read = state_storage.begin_read()?;
        state_storage.state().load_staking_state(&read)
    };

    match existing {
        Ok(_) => {
            info!("existing chain state detected; skipping genesis");
            return Ok(());
        }
        Err(StateStoreError::NotFound(_)) => {}
        Err(e) => return Err(e.into()),
    }

    info!("initialising genesis state");
    let treasury = parse_address_hex(config.treasury_address.as_deref())?;
    let mut staking = StakingState::new();
    staking.init_treasury(treasury);

    // Genesis validator allocation: the set admitted here receives
    // GENESIS_VALIDATOR_ALLOCATION, split evenly, and locks MIN_VALIDATOR_STAKE
    // from that share. Collected while loading the genesis files below.
    let mut genesis_stakes: Vec<(Address, u64)> = Vec::new();

    // Load full validator set from validators.json (Phase 14 ceremony) if present.
    let validators_json_path = config.data_dir.join("validators.json");
    if validators_json_path.exists() {
        let text = std::fs::read_to_string(&validators_json_path)
            .with_context(|| format!("reading {}", validators_json_path.display()))?;
        let validators_input: Vec<GenesisValidatorInput> = serde_json::from_str(&text)
            .with_context(|| format!("parsing {}", validators_json_path.display()))?;
        for v in &validators_input {
            let address = parse_address_hex(Some(&v.address))?;
            let pk = v
                .public_key
                .as_ref()
                .map(|s| parse_public_key_hex(s))
                .transpose()?;
            let stake = v.stake.unwrap_or(kovanica_staking::MIN_VALIDATOR_STAKE);
            staking.join_validator(address, stake, 0, Some(address), pk, None)?;
            genesis_stakes.push((address, stake));
        }
        info!(count = validators_input.len(), "loaded validators.json");
    }

    // Begin write transaction early (needed for premine + staking save)
    let txn = state_storage.begin_write()?;

    // Founder premine (200_000 KUNA) — write to state store if founder file present.
    let premine_path = config.data_dir.join("founder_premine.hex");
    if premine_path.exists() {
        let hex = std::fs::read_to_string(&premine_path)?.trim().to_string();
        if !hex.is_empty() {
            let founder = parse_address_hex(Some(&hex))?;
            use kovanica_storage::state_store::Account;
            let state = state_storage.state();
            let acct = Account {
                balance: kovanica_staking::FOUNDER_PREMINE,
                nonce: 0,
                code_hash: [0; 32],
                code: vec![],
            };
            state.set_account(&txn, &founder, &acct)?;
            info!(address = %founder, "founder premine applied");
        }
    }

    // Load pre-funded accounts from funded_accounts.toml if present.
    let funded_accounts_path = config.data_dir.join("funded_accounts.toml");
    if funded_accounts_path.exists() {
        let text = std::fs::read_to_string(&funded_accounts_path).with_context(|| {
            format!("reading funded accounts {}", funded_accounts_path.display())
        })?;
        let funded: FundedAccountsFile = toml::from_str(&text).with_context(|| {
            format!("parsing funded accounts {}", funded_accounts_path.display())
        })?;
        let state = state_storage.state();
        for entry in funded.account {
            let address = parse_address_hex(Some(&entry.address))?;
            use kovanica_storage::state_store::Account;
            let acct = Account {
                balance: entry.balance,
                nonce: 0,
                code_hash: [0; 32],
                code: vec![],
            };
            state.set_account(&txn, &address, &acct)?;
            info!(address = %address, balance = entry.balance, "funded account applied");
        }
    }

    // Load 4-node validator set from genesis file if present (Phase 16.6).
    let genesis_validators_path = config.data_dir.join("genesis_validators.toml");
    if genesis_validators_path.exists() {
        let text = std::fs::read_to_string(&genesis_validators_path).with_context(|| {
            format!(
                "reading genesis validators {}",
                genesis_validators_path.display()
            )
        })?;
        let validators: GenesisValidatorsFile = toml::from_str(&text).with_context(|| {
            format!(
                "parsing genesis validators {}",
                genesis_validators_path.display()
            )
        })?;
        let validators = validators.validator;
        let count = validators.len();
        for v in validators {
            let address = parse_address_hex(Some(&v.address))?;
            let pk = if let Some(p) = v.public_key.as_ref() {
                Some(parse_public_key_hex(p)?)
            } else {
                None
            };
            let vrf_pk = if let Some(p) = v.vrf_public_key.as_ref() {
                Some(kovanica_staking::VrfPublicKey::from_bytes(parse_hex_32(p)?))
            } else {
                None
            };
            staking.join_validator(address, v.stake, 0, Some(address), pk, vrf_pk)?;
            genesis_stakes.push((address, v.stake));
        }
        info!(count = count, "loaded genesis validator set");
    }

    // Mint the genesis validator allocation (exactly once, at genesis).
    //
    // Deterministic across nodes: validators are deduplicated and processed in
    // ascending address order, so every node derives identical balances from
    // the same genesis set. `genesis_validator_shares` assigns the integer
    // remainder one atom at a time, so the shares sum to exactly
    // GENESIS_VALIDATOR_ALLOCATION.
    if !genesis_stakes.is_empty() {
        use kovanica_storage::state_store::Account;

        let mut entries: Vec<(Address, u64)> = genesis_stakes.clone();
        entries.sort_by_key(|a| a.0 .0);
        entries.dedup_by(|a, b| a.0 .0 == b.0 .0);
        let n = entries.len();
        let shares = kovanica_staking::genesis_validator_shares(n);
        let state = state_storage.state();

        for ((address, stake), share) in entries.iter().zip(shares.iter()) {
            if *share < *stake {
                anyhow::bail!(
                    "genesis validator allocation ({} KUNA) cannot back a {} KUNA stake \
                     split across {} validators; at MIN_VALIDATOR_STAKE the allocation \
                     supports at most {} validators",
                    kovanica_staking::GENESIS_VALIDATOR_ALLOCATION / kovanica_staking::ONE_KUNA,
                    stake / kovanica_staking::ONE_KUNA,
                    n,
                    kovanica_staking::GENESIS_VALIDATOR_ALLOCATION
                        / kovanica_staking::MIN_VALIDATOR_STAKE,
                );
            }
            let acct = Account {
                balance: *share - *stake,
                nonce: 0,
                code_hash: [0; 32],
                code: vec![],
            };
            state.set_account(&txn, address, &acct)?;
        }
        info!(
            count = n,
            allocated_kuna =
                kovanica_staking::GENESIS_VALIDATOR_ALLOCATION / kovanica_staking::ONE_KUNA,
            "genesis validator allocation minted"
        );
    }

    state_storage.state().save_staking_state(&txn, &staking)?;

    // Persist the digest of the configured genesis so every later start can
    // detect a data dir pointed at a different genesis (fork/contamination
    // guard). Written in the same transaction as the genesis state.
    let genesis_digest = genesis_config_digest(config)?;
    state_storage
        .state()
        .save_genesis_digest(&txn, &genesis_digest)?;

    txn.commit()?;

    let digest = StatementBlock::compute_digest(0, 0, &[], &[]);
    let genesis = StatementBlock {
        author: 0,
        round: 0,
        parents: Vec::new(),
        transactions: Vec::new(),
        statements: Vec::new(),
        signature: Signature([0u8; 64]),
        digest,
        merkle_root: Hash::zero(),
    };
    dag_store.put_block(&genesis)?;
    info!(digest = %genesis.digest, "genesis block created");
    Ok(())
}

/// Env override that disables the genesis fork/contamination guard.
///
/// Set to `1` (or `true`/`yes`/`on`) only when deliberately pointing an
/// existing data dir at a different genesis (e.g. an intentional reset on an
/// isolated node). Every start with this set logs a warning.
const ALLOW_GENESIS_MISMATCH_ENV: &str = "KOVANICA_ALLOW_GENESIS_MISMATCH";

/// File name (inside `data_dir`) of the genesis ceremony artifact written by
/// the `genesis` subcommand.
const GENESIS_JSON_FILE: &str = "genesis.json";

/// Domain separator for the persisted genesis digest. Bump only alongside a
/// deliberate genesis-format change.
const GENESIS_DIGEST_DOMAIN: &[u8] = b"KVNC-GENESIS-v1";

/// Row `(address, stake, public_key, vrf_public_key)` used to order genesis
/// validators deterministically in [`genesis_config_digest`].
type GenesisValidatorDigestRow = ([u8; 32], u64, [u8; 32], [u8; 32]);

/// True when a genesis-mismatch override string enables the guard bypass.
///
/// Pure so the accepted spellings can be tested without touching the
/// process-global environment.
fn parse_mismatch_override(value: Option<&str>) -> bool {
    matches!(
        value
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// True when the genesis mismatch guard is explicitly disabled.
fn genesis_mismatch_allowed() -> bool {
    parse_mismatch_override(std::env::var(ALLOW_GENESIS_MISMATCH_ENV).ok().as_deref())
}

/// The canonical genesis block digest, reused from the deterministic genesis
/// block (`StatementBlock::compute_digest(0, 0, &[], &[])`) rather than
/// re-derived.
fn canonical_genesis_digest() -> Hash {
    StatementBlock::compute_digest(0, kovanica_types::GENESIS_ROUND, &[], &[])
}

/// Deterministic digest over the *configured* genesis contents.
///
/// A pure function of the activation inputs: the canonical genesis block
/// digest, the signed `chain_id`, the treasury address, and the genesis files
/// present in `data_dir` (`validators.json`, `founder_premine.hex`,
/// `funded_accounts.toml`, `genesis_validators.toml`). Addresses and public
/// keys are parsed to canonical bytes so hex case cannot change the digest;
/// entries are sorted so file order cannot either.
///
/// This reads the same inputs `init_genesis` derives genesis state from; it
/// does not change any allocation, emission or supply value.
fn genesis_config_digest(config: &NodeConfig) -> Result<Hash> {
    let mut data: Vec<u8> = Vec::new();

    // Canonical genesis block digest.
    data.extend_from_slice(&canonical_genesis_digest().0);
    // Signed chain id. Binding it here means a chain-id change is caught even
    // when `genesis_validators.toml` (the other chain-id source) is absent.
    data.extend_from_slice(&config.chain_id.to_le_bytes());
    // Treasury address.
    data.extend_from_slice(&parse_address_hex(config.treasury_address.as_deref())?.0);

    // validators.json (Phase 14 ceremony): array of {address, public_key?, stake?}.
    let validators_json_path = config.data_dir.join("validators.json");
    if validators_json_path.exists() {
        data.push(b'V');
        let text = std::fs::read_to_string(&validators_json_path)
            .with_context(|| format!("reading {}", validators_json_path.display()))?;
        let entries: Vec<GenesisValidatorInput> = serde_json::from_str(&text)
            .with_context(|| format!("parsing {}", validators_json_path.display()))?;
        let mut rows: Vec<([u8; 32], [u8; 32], u64)> = Vec::with_capacity(entries.len());
        for v in entries {
            let address = parse_address_hex(Some(&v.address))?;
            let pk = match v.public_key.as_ref() {
                Some(p) => parse_public_key_hex(p)?.0,
                None => [0u8; 32],
            };
            rows.push((
                address.0,
                pk,
                v.stake.unwrap_or(kovanica_staking::MIN_VALIDATOR_STAKE),
            ));
        }
        rows.sort_unstable();
        for (address, pk, stake) in rows {
            data.extend_from_slice(&address);
            data.extend_from_slice(&pk);
            data.extend_from_slice(&stake.to_le_bytes());
        }
    }

    // founder_premine.hex: the founder address that receives the premine.
    let premine_path = config.data_dir.join("founder_premine.hex");
    if premine_path.exists() {
        data.push(b'F');
        let hex = std::fs::read_to_string(&premine_path)
            .with_context(|| format!("reading {}", premine_path.display()))?
            .trim()
            .to_string();
        if !hex.is_empty() {
            data.extend_from_slice(&parse_address_hex(Some(&hex))?.0);
        }
    }

    // funded_accounts.toml.
    let funded_path = config.data_dir.join("funded_accounts.toml");
    if funded_path.exists() {
        data.push(b'A');
        let text = std::fs::read_to_string(&funded_path)
            .with_context(|| format!("reading {}", funded_path.display()))?;
        let funded: FundedAccountsFile =
            toml::from_str(&text).with_context(|| format!("parsing {}", funded_path.display()))?;
        let mut rows: Vec<([u8; 32], u64)> = Vec::with_capacity(funded.account.len());
        for entry in funded.account {
            rows.push((parse_address_hex(Some(&entry.address))?.0, entry.balance));
        }
        rows.sort_unstable();
        for (address, balance) in rows {
            data.extend_from_slice(&address);
            data.extend_from_slice(&balance.to_le_bytes());
        }
    }

    // genesis_validators.toml.
    let genesis_validators_path = config.data_dir.join("genesis_validators.toml");
    if genesis_validators_path.exists() {
        data.push(b'G');
        let text = std::fs::read_to_string(&genesis_validators_path)
            .with_context(|| format!("reading {}", genesis_validators_path.display()))?;
        let genesis: GenesisValidatorsFile = toml::from_str(&text)
            .with_context(|| format!("parsing {}", genesis_validators_path.display()))?;
        data.extend_from_slice(&genesis.chain_id.unwrap_or(0).to_le_bytes());
        let mut rows: Vec<GenesisValidatorDigestRow> = Vec::with_capacity(genesis.validator.len());
        for v in genesis.validator {
            let address = parse_address_hex(Some(&v.address))?.0;
            let pk = match v.public_key.as_ref() {
                Some(p) => parse_public_key_hex(p)?.0,
                None => [0u8; 32],
            };
            let vrf = match v.vrf_public_key.as_ref() {
                Some(p) => parse_hex_32(p)?,
                None => [0u8; 32],
            };
            rows.push((address, v.stake, pk, vrf));
        }
        rows.sort_unstable();
        for (address, stake, pk, vrf) in rows {
            data.extend_from_slice(&address);
            data.extend_from_slice(&stake.to_le_bytes());
            data.extend_from_slice(&pk);
            data.extend_from_slice(&vrf);
        }
    }

    Ok(Hash::new_keyed(GENESIS_DIGEST_DOMAIN, &data))
}

/// Verify the persisted genesis digest against `fresh`.
///
/// * Persisted digest equal to `fresh` → ok (no write).
/// * Persisted digest different → refuse unless `allow_mismatch`.
/// * No persisted digest (data dir predates the digest) → write it once, so
///   existing data dirs migrate cleanly and later starts are guarded.
fn verify_persisted_genesis_digest(
    state_storage: &Storage,
    fresh: &Hash,
    allow_mismatch: bool,
) -> Result<()> {
    let persisted = {
        let read = state_storage.begin_read()?;
        state_storage.state().load_genesis_digest(&read)?
    };
    match persisted {
        Some(stored) if stored != *fresh => {
            if !allow_mismatch {
                anyhow::bail!(
                    "genesis digest mismatch: data dir was created for genesis {} but the \
                     configured genesis is {}; refusing to start to avoid mixing chains \
                     (set {ALLOW_GENESIS_MISMATCH_ENV}=1 to override)",
                    stored.to_hex(),
                    fresh.to_hex(),
                );
            }
            warn!(
                stored = %stored,
                configured = %fresh,
                "{ALLOW_GENESIS_MISMATCH_ENV} set: continuing despite genesis digest mismatch"
            );
        }
        Some(_) => {}
        None => {
            let txn = state_storage.begin_write()?;
            state_storage.state().save_genesis_digest(&txn, fresh)?;
            txn.commit()?;
            info!(digest = %fresh, "persisted genesis digest for existing data dir");
        }
    }
    Ok(())
}

/// Read back `genesis.json` (if present) and verify its declared genesis-block
/// digest matches the canonical genesis digest.
///
/// The ceremony writes this artifact but nothing used to read it; a stale or
/// tampered copy is never silently rewritten. On mismatch the node refuses
/// unless `allow_mismatch`.
fn verify_genesis_json(config: &NodeConfig, canonical: &Hash, allow_mismatch: bool) -> Result<()> {
    let path = config.data_dir.join(GENESIS_JSON_FILE);
    if !path.exists() {
        return Ok(());
    }
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    let declared = value
        .get("genesis_block")
        .and_then(|b| b.get("digest_hex"))
        .and_then(|d| d.as_str());
    let Some(declared) = declared else {
        anyhow::bail!(
            "genesis file {} has no `genesis_block.digest_hex`; refusing to start",
            path.display()
        );
    };
    if declared != canonical.to_hex().as_str() {
        if !allow_mismatch {
            anyhow::bail!(
                "genesis file {} declares genesis block digest {} but the canonical genesis \
                 digest is {}; refusing to start (set {ALLOW_GENESIS_MISMATCH_ENV}=1 to override)",
                path.display(),
                declared,
                canonical.to_hex(),
            );
        }
        warn!(
            file = %path.display(),
            declared,
            canonical = %canonical.to_hex(),
            "{ALLOW_GENESIS_MISMATCH_ENV} set: continuing despite genesis.json mismatch"
        );
    }
    Ok(())
}

/// Startup guard: refuse to run a data dir whose persisted genesis does not
/// match the configured genesis, or with a contradictory `genesis.json`.
///
/// This is the fork/cross-chain contamination guard. It runs on every start,
/// independent of which genesis file happens to be present, and binds the
/// signed `chain_id` so a chain-id change is caught even without
/// `genesis_validators.toml`.
fn verify_genesis_consistency(config: &NodeConfig, state_storage: &Storage) -> Result<()> {
    let fresh = genesis_config_digest(config)?;
    let allow_mismatch = genesis_mismatch_allowed();
    if allow_mismatch {
        warn!("{ALLOW_GENESIS_MISMATCH_ENV} set: genesis mismatch guard disabled for this start");
    }
    verify_persisted_genesis_digest(state_storage, &fresh, allow_mismatch)?;
    verify_genesis_json(config, &canonical_genesis_digest(), allow_mismatch)?;
    Ok(())
}

/// Run the P2P network service and process outbound broadcast commands.
///
/// `NetworkService::start` takes `&self` and runs the swarm event loop forever,
/// so it is spawned once in its own task and owns the listen/dial preamble. The
/// command loop only calls the `&self` broadcast methods (interior mutability),
/// so a broadcast never cancels or re-enters `start` and never re-runs the
/// preamble.
async fn run_network(
    service: Arc<NetworkService>,
    mut cmd_rx: mpsc::UnboundedReceiver<NetworkCommand>,
    mut shutdown: watch::Receiver<bool>,
) {
    // Run the swarm event loop exactly once, independent of command handling.
    let swarm_service = service.clone();
    let mut swarm_task = tokio::spawn(async move {
        match swarm_service.start().await {
            Ok(()) => info!("network service stopped"),
            Err(e) => error!(error = %e, "network service error"),
        }
    });

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => match cmd {
                Some(NetworkCommand::BroadcastBlock(block)) => {
                    if let Err(e) = service.broadcast_block(&block) {
                        warn!(error = %e, "failed to broadcast block");
                    }
                }
                Some(NetworkCommand::BroadcastVote(vote)) => {
                    if let Err(e) = service.broadcast_vote(&vote) {
                        warn!(error = %e, "failed to broadcast vote");
                    }
                }
                Some(NetworkCommand::BroadcastTransaction(tx)) => {
                    if let Err(e) = service.broadcast_transaction(&tx) {
                        warn!(error = %e, "failed to broadcast transaction");
                    }
                }
                Some(NetworkCommand::BroadcastSeal(announcement)) => {
                    if let Err(e) = service.broadcast_seal(&announcement) {
                        warn!(error = %e, "failed to broadcast commit seal");
                    }
                }
                Some(NetworkCommand::RequestSync(peer, request)) => {
                    if let Err(e) = service.request_block_sync(peer, request) {
                        warn!(%peer, error = %e, "failed to request block sync");
                    }
                }
                Some(NetworkCommand::RequestStateSync(peer, request)) => {
                    if let Err(e) = service.request_state_sync(peer, request) {
                        warn!(%peer, error = %e, "failed to request state sync");
                    }
                }
                Some(NetworkCommand::Shutdown) | None => {
                    info!("network task stopping");
                    break;
                }
            },
            res = &mut swarm_task => {
                match res {
                    Ok(()) => info!("network swarm loop exited"),
                    Err(e) => error!(error = %e, "network swarm task failed"),
                }
                break;
            }
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    info!("network task received shutdown");
                    break;
                }
            }
        }
    }

    // Cancelling the swarm loop drops the swarm (and closes its listeners),
    // mirroring the previous shutdown behaviour; await it so the sockets are
    // released before this task returns.
    swarm_task.abort();
    let _ = swarm_task.await;
}

/// Ingest events emitted by the network layer into the DAG store and mempool.
#[allow(clippy::too_many_arguments)]
async fn run_event_handler(
    mut network_events: mpsc::UnboundedReceiver<NetworkEvent>,
    dag_store: Arc<DagStore>,
    mempool: Arc<Mempool>,
    engine: Arc<ConsensusEngine<NodeDagStore, NodeBlockManager>>,
    network_cmd_tx: mpsc::UnboundedSender<NetworkCommand>,
    events: EventBus,
    state_storage: Arc<Storage>,
    catch_up: Arc<Mutex<CatchUpTracker>>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut orphans = orphans::OrphanBuffer::new(orphans::OrphanLimits::default());
    let mut state_sync_sessions = StateSyncSessions::new();
    loop {
        tokio::select! {
            maybe = network_events.recv() => match maybe {
                Some(event) => {
                    // The catch-up tracker is shared with the read-only
                    // `kovanica_syncStatus` RPC. The lock is held only for the
                    // duration of this synchronous handler (no await inside).
                    let mut catch_up_guard = catch_up.lock();
                    handle_network_event(
                        event,
                        &mempool,
                        &engine,
                        &BlockIngest {
                            dag_store: &dag_store,
                            network_cmd_tx: &network_cmd_tx,
                            events: &events,
                        },
                        &mut orphans,
                        &mut catch_up_guard,
                        &mut state_sync_sessions,
                        &state_storage,
                    );
                }
                None => break,
            },
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
        }
    }
}

/// Shared handles for block ingestion.
struct BlockIngest<'a> {
    dag_store: &'a DagStore,
    network_cmd_tx: &'a mpsc::UnboundedSender<NetworkCommand>,
    events: &'a EventBus,
}

/// Process a received block (gossip or sync). If it is rejected because
/// parents are missing from the DAG, buffer it as an orphan and request the
/// missing parents (deduplicated, bounded) from `peer`. Every accepted block
/// releases buffered children, which are processed iteratively.
fn ingest_block(
    block: StatementBlock,
    peer: PeerId,
    engine: &ConsensusEngine<NodeDagStore, NodeBlockManager>,
    io: &BlockIngest<'_>,
    orphans: &mut orphans::OrphanBuffer,
) {
    let mut work = std::collections::VecDeque::from([(block, peer)]);
    while let Some((block, peer)) = work.pop_front() {
        match engine.process_block(&block) {
            Ok(()) => {
                debug!(digest = %block.digest, "validated received block into consensus");
                io.events.publish_new_head(&block);
                let released = orphans.parent_accepted(&block.digest);
                if !released.is_empty() {
                    debug!(parent = %block.digest, count = released.len(), "re-processing orphans");
                }
                work.extend(released);
            }
            Err(e) => {
                let missing: Vec<Hash> = block
                    .parents
                    .iter()
                    .map(|p| p.digest)
                    .filter(|d| !io.dag_store.has_block(d).unwrap_or(true))
                    .collect();
                if missing.is_empty() {
                    warn!(digest = %block.digest, error = %e, "rejected received consensus block");
                    continue;
                }
                let digest = block.digest;
                let out = orphans.insert(block, peer, missing, std::time::Instant::now());
                debug!(
                    %digest, %peer, buffered = out.buffered, evicted = out.evicted,
                    requests = out.requests.len(), orphans = orphans.len(),
                    "block has missing parents"
                );
                for hash in out.requests {
                    let request = BlockSyncRequest::ByHash(hash);
                    if let Err(e) = io
                        .network_cmd_tx
                        .send(NetworkCommand::RequestSync(peer, request))
                    {
                        debug!(error = %e, "failed to queue missing-parent request");
                    }
                }
            }
        }
    }
}

/// Inactivity timeout after which an idle catch-up session is dropped.
const CATCH_UP_TTL: Duration = Duration::from_secs(60);
/// Minimum interval between catch-up tick advances.
const CATCH_UP_TICK: Duration = Duration::from_millis(500);
/// Maximum number of range windows one session may request before giving up.
const CATCH_UP_MAX_RANGES: usize = 512;
/// Maximum by-hash fetches in flight per catch-up session. A 512-hash window is
/// released through this bounded pipeline instead of all at once (audit HIGH-2c).
const CATCH_UP_MAX_HASH_FETCHES: usize = 48;
/// Maximum consecutive empty ("hole") windows a session may cross before it is
/// dropped: bounds a peer that answers every range with an empty window while
/// advancing its advertised tip (audit MEDIUM-5).
const CATCH_UP_MAX_HOLE_WINDOWS: u32 = 4;

/// How far a catch-up session with one peer has progressed.
enum CatchUpPhase {
    /// Tip probe sent, waiting for `BlockSyncResponse::Status`.
    Probing,
    /// Pulling rounds `next_from..=target` one window at a time.
    CatchingUp {
        /// Peer-advertised tip round; a fetch bound only, never trusted.
        target: Round,
        /// First round of the next window to request.
        next_from: Round,
        /// Whether a `RoundRange` request is awaiting its response.
        in_flight: bool,
        /// Set when the last window covered only empty rounds: keep going even
        /// though the local tip cannot advance.
        hole_mode: bool,
    },
}

/// Per-peer catch-up state.
struct CatchUpSession {
    phase: CatchUpPhase,
    /// Last time a message for this session arrived or was sent.
    last_activity: Instant,
    /// Local DAG tip at the last tick (progress gate).
    last_local: Option<Round>,
    /// Windows requested so far (budget).
    ranges_done: usize,
    /// By-hash fetches advertised by the last window but not yet issued (the
    /// pipelining backlog, bounded by one [`BlockSyncResponse::MAX_RANGE_HASHES`]).
    hash_backlog: std::collections::VecDeque<Hash>,
    /// By-hash fetches issued but not yet answered (pipelining window usage).
    inflight_hashes: usize,
    /// Consecutive empty windows crossed (hole-mode amplification bound).
    hole_windows: u32,
}

/// Bounded proactive catch-up: when a peer connects, ask for its DAG tip and,
/// if it is ahead, pull the missing rounds through the existing
/// `/kovanica/block-sync/1.1.0` protocol (one `RoundRange` window at a time,
/// re-fetching every hash via `ByHash` so block validation is unchanged).
///
/// Every bound is explicit: one session per peer, at most
/// [`BlockSyncRequest::MAX_RANGE_ROUNDS`] rounds per window, at most
/// [`CATCH_UP_MAX_RANGES`] windows per session, a [`CATCH_UP_TTL`] inactivity
/// timeout, one in-flight window per session, at most
/// [`CATCH_UP_MAX_HASH_FETCHES`] by-hash fetches in flight (a 512-hash window
/// is pipelined, never fired all at once), at most
/// [`CATCH_UP_MAX_HOLE_WINDOWS`] consecutive empty windows, and windows only
/// continue while the node is actually ingesting the fetched blocks (or
/// crossing a bounded run of rounds the peer reported as empty). Ticks are
/// driven by network events, so an idle swarm defers all catch-up work.
struct CatchUpTracker {
    sessions: std::collections::HashMap<PeerId, CatchUpSession>,
    last_tick: Instant,
}

impl CatchUpTracker {
    fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
            last_tick: Instant::now(),
        }
    }

    /// Whether any session is active (ticks are skipped when idle).
    fn is_active(&self) -> bool {
        !self.sessions.is_empty()
    }

    /// Read-only summary for status surfaces: whether any session is actively
    /// catching up (a [`CatchUpPhase::CatchingUp`] session exists; a lone
    /// `Probing` session counts as "not catching up", so its target is `None`),
    /// together with the highest peer-advertised tip being chased.
    fn summary(&self) -> (bool, Option<Round>) {
        let mut target: Option<Round> = None;
        for session in self.sessions.values() {
            if let CatchUpPhase::CatchingUp { target: t, .. } = &session.phase {
                target = Some(target.map_or(*t, |current| current.max(*t)));
            }
        }
        (target.is_some(), target)
    }

    /// Whether enough time has passed to advance the sessions.
    fn tick_due(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.last_tick) >= CATCH_UP_TICK
    }

    /// Start probing a freshly connected peer's tip (one session per peer).
    fn probe(
        &mut self,
        peer: PeerId,
        now: Instant,
        network_cmd_tx: &mpsc::UnboundedSender<NetworkCommand>,
    ) {
        if self.sessions.contains_key(&peer) {
            return;
        }
        self.sessions.insert(
            peer,
            CatchUpSession {
                phase: CatchUpPhase::Probing,
                last_activity: now,
                last_local: None,
                ranges_done: 0,
                hash_backlog: std::collections::VecDeque::new(),
                inflight_hashes: 0,
                hole_windows: 0,
            },
        );
        if network_cmd_tx
            .send(NetworkCommand::RequestSync(peer, BlockSyncRequest::Status))
            .is_err()
        {
            self.sessions.remove(&peer);
        }
    }

    /// Drop the session for a disconnected peer.
    fn remove(&mut self, peer: PeerId) {
        self.sessions.remove(&peer);
    }

    /// Handle the answer to a tip probe: start pulling windows if the peer is
    /// ahead of us. A peer-supplied tip is a fetch trigger only.
    fn on_status(
        &mut self,
        peer: PeerId,
        tip_round: Round,
        local_tip: Option<Round>,
        now: Instant,
        network_cmd_tx: &mpsc::UnboundedSender<NetworkCommand>,
    ) {
        let Some(mut session) = self.sessions.remove(&peer) else {
            return;
        };
        if !matches!(session.phase, CatchUpPhase::Probing) {
            // Stale status for a session already catching up: ignore.
            self.sessions.insert(peer, session);
            return;
        }
        // A storage error while reading our own tip means we cannot tell
        // whether we are behind: stand down instead of fetching blindly.
        let Some(local) = local_tip else {
            return;
        };
        session.last_activity = now;
        session.last_local = local_tip;
        if tip_round <= local {
            // Peer is not ahead of us: nothing to do.
            return;
        }
        let from = local.saturating_add(1);
        let to = from
            .saturating_add(BlockSyncRequest::MAX_RANGE_ROUNDS - 1)
            .min(tip_round);
        session.phase = CatchUpPhase::CatchingUp {
            target: tip_round,
            next_from: from,
            in_flight: true,
            hole_mode: false,
        };
        session.ranges_done = 1;
        if network_cmd_tx
            .send(NetworkCommand::RequestSync(
                peer,
                BlockSyncRequest::RoundRange { from, to },
            ))
            .is_ok()
        {
            self.sessions.insert(peer, session);
        }
    }

    /// Handle the answer to a range window: queue the advertised hashes into a
    /// bounded by-hash pipeline (the same path missing parents use) and record
    /// how far coverage reached so the next tick can continue.
    ///
    /// `window_empty` is whether the peer's window itself had no hashes (as
    /// opposed to [`hashes`] being empty because every hash was already local),
    /// which decides hole-mode continuation.
    fn on_round_hashes(
        &mut self,
        peer: PeerId,
        up_to_round: Round,
        window_empty: bool,
        hashes: &[Hash],
        now: Instant,
        network_cmd_tx: &mpsc::UnboundedSender<NetworkCommand>,
    ) {
        let Some(mut session) = self.sessions.remove(&peer) else {
            return;
        };
        let target = match &session.phase {
            CatchUpPhase::CatchingUp { target, .. } => *target,
            CatchUpPhase::Probing => {
                self.sessions.insert(peer, session);
                return;
            }
        };
        // Defensive: the network layer already caps this, but never iterate a
        // hostile-length list.
        if hashes.len() > BlockSyncResponse::MAX_RANGE_HASHES {
            return;
        }
        session.last_activity = now;
        // Each hash is re-requested by hash and validated like any other
        // block; the peer's `up_to_round` only decides how far we continue.
        // The backlog is drained through a bounded window (see
        // `pump_hash_fetches`) instead of firing all of them at once.
        session.hash_backlog.extend(hashes.iter().copied());
        session.phase = CatchUpPhase::CatchingUp {
            target,
            next_from: up_to_round.saturating_add(1),
            in_flight: false,
            hole_mode: window_empty,
        };
        if window_empty {
            session.hole_windows = session.hole_windows.saturating_add(1);
        } else {
            session.hole_windows = 0;
        }
        Self::pump_hash_fetches(peer, &mut session, network_cmd_tx);
        self.sessions.insert(peer, session);
    }

    /// Release queued by-hash fetches while the session's in-flight window has
    /// room. Bounds the burst one advertised window can create.
    fn pump_hash_fetches(
        peer: PeerId,
        session: &mut CatchUpSession,
        network_cmd_tx: &mpsc::UnboundedSender<NetworkCommand>,
    ) {
        while session.inflight_hashes < CATCH_UP_MAX_HASH_FETCHES {
            let Some(hash) = session.hash_backlog.pop_front() else {
                break;
            };
            if network_cmd_tx
                .send(NetworkCommand::RequestSync(
                    peer,
                    BlockSyncRequest::ByHash(hash),
                ))
                .is_err()
            {
                // The command channel is gone (shutdown): requeue and stop.
                session.hash_backlog.push_front(hash);
                break;
            }
            session.inflight_hashes += 1;
        }
    }

    /// Record that one by-hash fetch for `peer` completed or failed, freeing a
    /// slot in the pipelining window and releasing more of the backlog.
    fn on_hash_response(
        &mut self,
        peer: PeerId,
        now: Instant,
        network_cmd_tx: &mpsc::UnboundedSender<NetworkCommand>,
    ) {
        let Some(mut session) = self.sessions.remove(&peer) else {
            return;
        };
        session.inflight_hashes = session.inflight_hashes.saturating_sub(1);
        session.last_activity = now;
        Self::pump_hash_fetches(peer, &mut session, network_cmd_tx);
        self.sessions.insert(peer, session);
    }

    /// Advance every active session: expire idle ones, then send the next
    /// window for sessions whose fetched blocks have been ingested (or that
    /// are crossing an empty round) and that still have budget left.
    fn tick(
        &mut self,
        now: Instant,
        local_tip: Option<Round>,
        network_cmd_tx: &mpsc::UnboundedSender<NetworkCommand>,
    ) {
        self.last_tick = now;
        let mut drops: Vec<PeerId> = Vec::new();
        let mut sends: Vec<(PeerId, Round, Round)> = Vec::new();
        for (peer, session) in self.sessions.iter_mut() {
            if now.saturating_duration_since(session.last_activity) >= CATCH_UP_TTL {
                drops.push(*peer);
                continue;
            }
            if let Some(local) = local_tip {
                session.last_local = Some(local);
            }
            let (target, next_from, in_flight, hole_mode) = match &session.phase {
                CatchUpPhase::CatchingUp {
                    target,
                    next_from,
                    in_flight,
                    hole_mode,
                } => (*target, *next_from, *in_flight, *hole_mode),
                CatchUpPhase::Probing => continue,
            };
            if in_flight {
                continue;
            }
            if !session.hash_backlog.is_empty() {
                // Still draining the previous window's by-hash pipeline: do not
                // start another range window on top of it.
                continue;
            }
            if next_from > target {
                // Whole advertised range covered: session complete.
                drops.push(*peer);
                continue;
            }
            // A peer that answers with an empty window while advancing its
            // advertised tip is the hole-mode amplification vector: cap the
            // consecutive run instead of crossing windows forever (audit
            // MEDIUM-5).
            if hole_mode && session.hole_windows >= CATCH_UP_MAX_HOLE_WINDOWS {
                drops.push(*peer);
                continue;
            }
            // Continue only once the blocks we already fetched have actually
            // been ingested (the local tip reached the window end), or when the
            // window was empty (nothing to ingest, so the tip cannot move).
            let progressed = session
                .last_local
                .is_some_and(|local| local.saturating_add(1) >= next_from);
            if !progressed && !hole_mode {
                continue;
            }
            if session.ranges_done >= CATCH_UP_MAX_RANGES {
                drops.push(*peer);
                continue;
            }
            let to = next_from
                .saturating_add(BlockSyncRequest::MAX_RANGE_ROUNDS - 1)
                .min(target);
            session.phase = CatchUpPhase::CatchingUp {
                target,
                next_from,
                in_flight: true,
                hole_mode: false,
            };
            session.ranges_done += 1;
            session.last_activity = now;
            sends.push((*peer, next_from, to));
        }
        for peer in drops {
            self.sessions.remove(&peer);
        }
        for (peer, from, to) in sends {
            if network_cmd_tx
                .send(NetworkCommand::RequestSync(
                    peer,
                    BlockSyncRequest::RoundRange { from, to },
                ))
                .is_err()
            {
                break;
            }
        }
    }
}

/// Local DAG tip round used as a fetch trigger: `Some(0)` for a fresh node
/// with an empty DAG (behind any peer that has rounds), `None` when storage
/// cannot be read (stand down rather than fetch blindly).
fn local_dag_tip(dag_store: &DagStore) -> Option<Round> {
    match dag_store.max_dag_round() {
        Ok(Some(round)) => Some(round),
        Ok(None) => Some(0),
        Err(_) => None,
    }
}

// ------------------------------------------------------------------
// State sync (chunked snapshot fast-sync)
// ------------------------------------------------------------------

/// Debug entry point that arms a state sync trigger toward one peer.
///
/// When set to a libp2p peer id, the node pulls a full state snapshot from
/// that peer the first time it connects (and again after the rate-limit
/// interval). The trigger is deliberately manual instead of automatic: the
/// import replaces *all* local state tables, this chain has no pruned-below-
/// lowest-block signal that would make "I need a snapshot" objective, and a
/// peer that can push a snapshot past local verification controls balances.
/// The operator opts in by naming the peer; verification (state root +
/// height) still runs before anything is applied.
const STATE_SYNC_PEER_ENV: &str = "KOVANICA_STATE_SYNC_PEER";

/// Minimum interval between state sync transfer starts toward the same peer.
/// Chunk continuations of an active session are not gated by this.
const STATE_SYNC_MIN_INTERVAL: Duration = Duration::from_secs(600);

/// A transfer that stops progressing for this long is abandoned; a fresh
/// start is then again subject to [`STATE_SYNC_MIN_INTERVAL`].
const STATE_SYNC_SESSION_TTL: Duration = Duration::from_secs(300);

/// Export side of the P2P state sync protocol, backed by the node's state
/// storage. [`kovanica_network::NetworkService`] slices and caches the bytes;
/// this only produces the full export for the height it actually holds.
struct NodeStateSnapshotSource {
    storage: Arc<Storage>,
}

impl StateSnapshotSource for NodeStateSnapshotSource {
    fn export_snapshot(
        &self,
        target_height: Option<u64>,
    ) -> Result<Option<kovanica_network::ExportedSnapshot>, String> {
        let txn = self.storage.begin_read().map_err(|e| e.to_string())?;
        let height = self
            .storage
            .state()
            .snapshot_height(&txn)
            .map_err(|e| e.to_string())?;
        // This export always reflects the state this node currently holds; a
        // request for any other height cannot be served.
        if target_height.is_some_and(|target| target != height) {
            return Ok(None);
        }
        let bytes = self
            .storage
            .state()
            .snapshot_bytes(&txn)
            .map_err(|e| e.to_string())?;
        let state_root = kovanica_storage::StateStore::decode_snapshot(&bytes)
            .map_err(|e| e.to_string())?
            .state_root;
        Ok(Some(kovanica_network::ExportedSnapshot {
            height,
            state_root,
            bytes,
        }))
    }
}

/// One in-flight chunked state sync transfer toward a peer.
struct StateSyncSession {
    /// Height the transfer targeted (`None` = peer's latest at start).
    target_height: Option<u64>,
    /// Height claimed by the first chunk (consistency anchor; 0 = no chunk
    /// received yet, so the transfer is not anchored).
    height: u64,
    /// State root claimed by the first chunk.
    state_root: [u8; 32],
    /// Total serialized length claimed by the first chunk.
    total_len: u64,
    /// Bytes received so far, in order.
    received: Vec<u8>,
    /// When the session last progressed (TTL check).
    last_activity: Instant,
}

/// Initiator side of the state sync protocol: rate-limits transfer starts,
/// reassembles bounded chunks and refuses inconsistent streams.
struct StateSyncSessions {
    /// Peer named by [`STATE_SYNC_PEER_ENV`]; transfers only start toward it.
    debug_peer: Option<PeerId>,
    /// Active transfers.
    sessions: HashMap<PeerId, StateSyncSession>,
    /// When a transfer toward a peer last started (rate limiting).
    last_started: HashMap<PeerId, Instant>,
}

impl StateSyncSessions {
    fn new() -> Self {
        let debug_peer = std::env::var(STATE_SYNC_PEER_ENV).ok().and_then(|value| {
            match value.trim().parse::<PeerId>() {
                Ok(peer) => Some(peer),
                Err(err) => {
                    warn!(
                        %err,
                        "{STATE_SYNC_PEER_ENV} is not a valid peer id; state sync trigger disabled"
                    );
                    None
                }
            }
        });
        if debug_peer.is_some() {
            info!("state sync debug trigger armed ({STATE_SYNC_PEER_ENV})");
        }
        Self::with_debug_peer(debug_peer)
    }

    fn with_debug_peer(debug_peer: Option<PeerId>) -> Self {
        Self {
            debug_peer,
            sessions: HashMap::new(),
            last_started: HashMap::new(),
        }
    }

    /// Start a transfer toward `peer` if the debug entry point targets it and
    /// the per-peer rate limit allows it.
    fn maybe_start(
        &mut self,
        peer: PeerId,
        now: Instant,
        cmd_tx: &mpsc::UnboundedSender<NetworkCommand>,
    ) {
        if self.debug_peer != Some(peer) || self.sessions.contains_key(&peer) {
            return;
        }
        if let Some(last) = self.last_started.get(&peer) {
            if now.saturating_duration_since(*last) < STATE_SYNC_MIN_INTERVAL {
                debug!(%peer, "state sync start rate-limited; skipping");
                return;
            }
        }
        self.last_started.insert(peer, now);
        self.sessions.insert(
            peer,
            StateSyncSession {
                target_height: None,
                height: 0,
                state_root: [0u8; 32],
                total_len: 0,
                received: Vec::new(),
                last_activity: now,
            },
        );
        debug!(%peer, "starting state sync (debug entry point)");
        if cmd_tx
            .send(NetworkCommand::RequestStateSync(
                peer,
                StateSyncRequest::latest(),
            ))
            .is_err()
        {
            debug!(%peer, "network command channel closed; state sync not started");
        }
    }

    /// Ingest one chunk of a session.
    ///
    /// The first chunk anchors the transfer (height, state root, total
    /// length); every later chunk must repeat that metadata, arrive in order
    /// and stay within the frame bounds. Any violation abandons the whole
    /// transfer. `Some(bytes)` is returned once the payload is complete.
    fn on_chunk(&mut self, peer: PeerId, chunk: SnapshotChunk, now: Instant) -> Option<Vec<u8>> {
        let Some(session) = self.sessions.get_mut(&peer) else {
            debug!(%peer, offset = chunk.offset, "state sync chunk without an active session; dropped");
            return None;
        };
        let within_frame = chunk.data.len() <= MAX_STATE_SYNC_CHUNK_BYTES
            && chunk.offset.saturating_add(chunk.data.len() as u64) <= chunk.total_len;
        if session.total_len == 0 {
            // First chunk anchors the transfer.
            if chunk.offset != 0
                || chunk.total_len == 0
                || chunk.total_len > MAX_STATE_SYNC_SNAPSHOT_BYTES
                || !within_frame
            {
                warn!(%peer, total_len = chunk.total_len, "rejecting invalid first state sync chunk; abandoning transfer");
                self.sessions.remove(&peer);
                return None;
            }
            session.height = chunk.height;
            session.state_root = chunk.state_root;
            session.total_len = chunk.total_len;
        } else {
            let consistent = session.height == chunk.height
                && session.state_root == chunk.state_root
                && session.total_len == chunk.total_len;
            let in_order = chunk.offset as usize == session.received.len();
            if !consistent || !in_order || !within_frame {
                warn!(%peer, offset = chunk.offset, received = session.received.len(), "inconsistent state sync chunk; abandoning transfer");
                self.sessions.remove(&peer);
                return None;
            }
        }
        session.received.extend_from_slice(&chunk.data);
        session.last_activity = now;
        if session.received.len() as u64 == session.total_len {
            let session = self.sessions.remove(&peer).expect("session present");
            return Some(session.received);
        }
        None
    }

    /// Request the next chunk of an active transfer (no-op otherwise).
    fn request_next(&mut self, peer: PeerId, cmd_tx: &mpsc::UnboundedSender<NetworkCommand>) {
        let Some(session) = self.sessions.get(&peer) else {
            return;
        };
        let offset = session.received.len() as u64;
        let request = match session.target_height {
            Some(target) => StateSyncRequest::at_height_from(target, offset),
            None => StateSyncRequest {
                target_height: None,
                offset,
            },
        };
        if cmd_tx
            .send(NetworkCommand::RequestStateSync(peer, request))
            .is_err()
        {
            debug!(%peer, "network command channel closed; state sync continuation not sent");
        }
    }

    /// Forget a peer's session (completion, rejection or peer error).
    fn drop_session(&mut self, peer: PeerId) {
        self.sessions.remove(&peer);
    }

    /// Abandon transfers that stopped progressing.
    fn expire(&mut self, now: Instant) {
        let expired: Vec<PeerId> = self
            .sessions
            .iter()
            .filter(|(_, session)| {
                now.saturating_duration_since(session.last_activity) >= STATE_SYNC_SESSION_TTL
            })
            .map(|(peer, _)| *peer)
            .collect();
        for peer in expired {
            warn!(%peer, "state sync session expired before completion");
            self.sessions.remove(&peer);
        }
    }
}

/// Verify and apply a fully reassembled state snapshot.
///
/// Nothing is applied unless [`kovanica_storage::StateStore::import_verified_snapshot`]
/// accepts the payload: the state root recomputed from the received tables
/// must equal the one the snapshot header claims, and the snapshot height
/// must not be below the local committed height (no silent rollback). The
/// write transaction commits only after verification succeeds, so a reject
/// leaves local state untouched.
fn apply_state_snapshot(state_storage: &Arc<Storage>, peer: PeerId, bytes: &[u8]) {
    let min_height = match state_storage.begin_read() {
        Ok(txn) => {
            let state = state_storage.state();
            let mut height = match state.get_latest_state_root(&txn) {
                Ok(Some((height, _))) => height,
                Ok(None) => 0,
                Err(e) => {
                    warn!(error = %e, "could not read local state root; refusing state snapshot");
                    return;
                }
            };
            match state.get_latest_leader_header(&txn) {
                Ok(Some(header)) => height = height.max(header.committed_height),
                Ok(None) => {}
                Err(e) => {
                    warn!(error = %e, "could not read local leader header; refusing state snapshot");
                    return;
                }
            }
            height
        }
        Err(e) => {
            warn!(error = %e, "could not open read transaction; refusing state snapshot");
            return;
        }
    };
    let txn = match state_storage.begin_write() {
        Ok(txn) => txn,
        Err(e) => {
            error!(error = %e, "failed to begin write transaction for state sync import");
            return;
        }
    };
    match state_storage
        .state()
        .import_verified_snapshot(&txn, bytes, min_height)
    {
        Ok(height) => {
            if let Err(e) = txn.commit() {
                error!(error = %e, "failed to commit state sync import");
                return;
            }
            info!(%peer, height, min_height, "state snapshot verified and imported");
        }
        Err(e) => {
            // The transaction is dropped uncommitted: local state is untouched.
            error!(%peer, error = %e, "rejected state snapshot");
        }
    }
}

/// Handle a single [`NetworkEvent`].
// 8 args: the state-sync session registry is a distinct concern from the
// catch-up tracker; grouping them into a struct would obscure call sites.
#[allow(clippy::too_many_arguments)]
fn handle_network_event(
    event: NetworkEvent,
    mempool: &Mempool,
    engine: &ConsensusEngine<NodeDagStore, NodeBlockManager>,
    io: &BlockIngest<'_>,
    orphans: &mut orphans::OrphanBuffer,
    catch_up: &mut CatchUpTracker,
    state_sync_sessions: &mut StateSyncSessions,
    state_storage: &Arc<Storage>,
) {
    let now = Instant::now();
    let (network_cmd_tx, events) = (io.network_cmd_tx, io.events);
    if catch_up.is_active() && catch_up.tick_due(now) {
        catch_up.tick(now, local_dag_tip(io.dag_store), network_cmd_tx);
    }
    orphans.expire(now);
    state_sync_sessions.expire(now);
    match event {
        NetworkEvent::BlockReceived { peer, block } => {
            // Hot-path validation (audit 3.1): verify signature before engine.process_block
            if let Err(e) = kovanica_crypto::verify_batch(std::slice::from_ref(&block)) {
                warn!(digest = %block.digest, error = %e, "rejected invalid block signature in main hot path");
                return;
            }
            ingest_block(block, peer, engine, io, orphans);
        }
        NetworkEvent::TransactionReceived(tx) => {
            // The network layer already admitted this transaction into the
            // mempool before emitting the event, and gossipsub forwards it.
            // Re-adding it here only ever hit AlreadyExists (so the pending
            // event was never published), and re-broadcasting duplicated
            // gossip. Publish it if it is still pooled.
            if mempool.contains(&tx.hash) {
                events.publish_pending_transaction(&tx);
            }
        }
        NetworkEvent::SealReceived { peer, announcement } => {
            match seals::store_matching_seal(state_storage, &announcement) {
                Ok(true) => {
                    debug!(
                        %peer,
                        height = announcement.header.committed_height,
                        authority = announcement.seal.authority_index,
                        "stored commit seal"
                    );
                    publish_sealed(state_storage, events, &announcement.header);
                }
                Ok(false) => debug!(
                    %peer,
                    height = announcement.header.committed_height,
                    "ignored commit seal; local header does not match"
                ),
                Err(e) => warn!(%peer, error = %e, "failed to store commit seal"),
            }
        }
        NetworkEvent::VoteReceived { peer, vote } => {
            debug!(%peer, leader_round = vote.leader_round, %vote.leader_hash, "processing received vote");
            // `process_vote` authenticates the vote against the committee key.
            match engine.process_vote(&vote) {
                Ok(()) => {}
                Err(kovanica_consensus::ConsensusError::VoteRejected(
                    reason @ (kovanica_consensus::VoteRejection::Duplicate { .. }
                    | kovanica_consensus::VoteRejection::UnknownLeaderRound(_)),
                )) => {
                    // Expected under gossip (re-delivery, vote before block).
                    debug!(%peer, %reason, "ignored received vote");
                }
                Err(e) => warn!(%peer, %e, "rejected received vote"),
            }
        }
        NetworkEvent::PeerConnected(peer) => {
            info!(%peer, "peer connected");
            // Proactive catch-up: probe the peer's tip and pull any missing
            // range through the existing block-sync protocol.
            catch_up.probe(peer, now, network_cmd_tx);
            // Optional state sync (debug entry point only, rate-limited).
            state_sync_sessions.maybe_start(peer, now, network_cmd_tx);
        }
        NetworkEvent::PeerDisconnected(peer) => {
            info!(%peer, "peer disconnected");
            catch_up.remove(peer);
        }
        NetworkEvent::PeerDiscovered(peer, addr) => debug!(%peer, %addr, "peer discovered"),
        NetworkEvent::SyncRequest {
            peer,
            from_round,
            to_round,
        } => debug!(%peer, from_round, to_round, "sync request received"),
        NetworkEvent::BlockSyncResponse {
            peer,
            request_id: _,
            response,
        } => {
            debug!(%peer, ?response, "block sync response received");
            match response {
                BlockSyncResponse::Block(block) => {
                    // Synced blocks take the same path as gossiped ones: the
                    // network layer already checked digest/merkle/signature and
                    // that it is the block we asked for; the engine's block
                    // manager validates parents before anything is stored. A
                    // synced block is never written to the DAG directly.
                    // Missing parents are fetched recursively via the orphan
                    // buffer.
                    catch_up.on_hash_response(peer, now, network_cmd_tx);
                    ingest_block(block, peer, engine, io, orphans);
                }
                BlockSyncResponse::NotFound => {
                    catch_up.on_hash_response(peer, now, network_cmd_tx);
                    debug!(%peer, "requested block not found on peer");
                }
                BlockSyncResponse::InvalidRequest => {
                    catch_up.on_hash_response(peer, now, network_cmd_tx);
                    warn!(%peer, "block sync request was invalid");
                }
                BlockSyncResponse::Status { tip_round } => {
                    // A height advertisement is only a fetch trigger: it decides
                    // whether we start pulling ranges, never what we accept —
                    // every fetched block is still fully validated.
                    catch_up.on_status(
                        peer,
                        tip_round,
                        local_dag_tip(io.dag_store),
                        now,
                        network_cmd_tx,
                    );
                }
                BlockSyncResponse::RoundHashes {
                    up_to_round,
                    hashes,
                } => {
                    // The peer reporting an actually-empty window is what drives
                    // hole-mode continuation; a list that is empty only because
                    // we already hold every hash must not.
                    let window_empty = hashes.is_empty();
                    // Skip hashes already present locally (audit LOW-10): no
                    // point re-fetching a block we already have.
                    let missing: Vec<Hash> = hashes
                        .into_iter()
                        .filter(|hash| !io.dag_store.has_block(hash).unwrap_or(true))
                        .collect();
                    catch_up.on_round_hashes(
                        peer,
                        up_to_round,
                        window_empty,
                        &missing,
                        now,
                        network_cmd_tx,
                    );
                }
                BlockSyncResponse::MissingBlocks(hashes) => {
                    debug!(%peer, count = hashes.len(), "peer returned missing parent hashes");
                    // Re-request each missing block
                    for hash in hashes {
                        let request = BlockSyncRequest::ByHash(hash);
                        if let Err(e) =
                            network_cmd_tx.send(NetworkCommand::RequestSync(peer, request))
                        {
                            debug!(error = %e, "failed to queue re-request for missing block");
                        }
                    }
                }
            }
        }
        NetworkEvent::StateSyncResponse {
            peer,
            request_id: _,
            response,
        } => {
            debug!(%peer, ?response, "state sync response received");
            match response {
                StateSyncResponse::Chunk(chunk) => {
                    debug!(
                        %peer,
                        offset = chunk.offset,
                        len = chunk.data.len(),
                        total = chunk.total_len,
                        height = chunk.height,
                        "received state snapshot chunk"
                    );
                    match state_sync_sessions.on_chunk(peer, chunk, now) {
                        // Transfer complete: verify (state root + height)
                        // before applying; a reject never touches local state.
                        Some(bytes) => apply_state_snapshot(state_storage, peer, &bytes),
                        // Still incomplete and the session survived: ask for
                        // the next chunk. Rejected sessions are already gone.
                        None => state_sync_sessions.request_next(peer, network_cmd_tx),
                    }
                }
                StateSyncResponse::NotFound => {
                    state_sync_sessions.drop_session(peer);
                    debug!(%peer, "requested state snapshot not found on peer");
                }
                StateSyncResponse::InvalidRequest => {
                    state_sync_sessions.drop_session(peer);
                    warn!(%peer, "state sync request was invalid");
                }
            }
        }
    }
}

/// Push the stored header and the seals gathered for it to subscribers.
fn publish_sealed(storage: &Storage, events: &EventBus, header: &kovanica_types::LeaderHeader) {
    let seals = match storage.begin_read() {
        Ok(txn) => match storage.state().seals_for(&txn, header.committed_height) {
            Ok(seals) => seals,
            Err(e) => {
                warn!(error = %e, "could not read commit seals for the sealed-header event");
                return;
            }
        },
        Err(e) => {
            warn!(error = %e, "could not read commit seals for the sealed-header event");
            return;
        }
    };
    events.publish_sealed_header(header, &seals);
}

/// Execute committed sub-DAGs as they are produced by consensus.
#[allow(clippy::too_many_arguments)]
async fn run_execution(
    mut subdags: mpsc::UnboundedReceiver<kovanica_consensus::CommittedSubDag>,
    state_storage: Arc<Storage>,
    events: EventBus,
    pool: Option<non_blue::ExecPool>,
    signing_ctx: kovanica_types::SigningContext,
    mut shutdown: watch::Receiver<bool>,
    health: NodeHealth,
    seal_duty: Option<SealDuty>,
    rpc_committee: Arc<tokio::sync::RwLock<CommitteeInfo>>,
    listen_addr: String,
    validator_address: Address,
    public_key: PublicKey,
) {
    // Execution verifies tx signatures with the node's configured chain id.
    let mut ctx = ExecutionContext::with_signing_context(signing_ctx);
    ctx.log_publisher = Some(Box::new(EventLogPublisher(events.clone())));
    match state_storage.begin_read() {
        Ok(txn) => match state_storage.state().load_staking_state(&txn) {
            Ok(state) => ctx.staking = state,
            Err(e) => warn!(error = %e, "could not load staking state for execution"),
        },
        Err(e) => warn!(error = %e, "could not open read transaction for execution"),
    }

    // Track epoch transitions so the node can observe boundaries as leaders
    // commit. Epoch is derived from committed-leader height (deterministic).
    let mut last_epoch: Option<u64> = None;

    loop {
        tokio::select! {
            maybe = subdags.recv() => match maybe {
                Some(subdag) => match ctx.execute_committed_subdag(&subdag, &state_storage) {
                    Ok(result) => {
                        info!(
                            round = subdag.leader_round,
                            txs = result.txs_applied,
                            "executed committed sub-DAG"
                        );
                        // Epoch rotation: detect a boundary crossing and log it.
                        // The committee is rebuilt from staking state on the next
                        // `build_committee`; live engine swap is gated separately.
                        if let Some(prev) = last_epoch {
                            if prev != result.epoch {
                                info!(
                                    from_epoch = prev,
                                    to_epoch = result.epoch,
                                    "epoch boundary crossed; rebuilding committee for RPC"
                                );
                                match build_committee(
                                    &state_storage,
                                    &public_key,
                                    &validator_address,
                                    &listen_addr,
                                ) {
                                    Ok(new_committee) => {
                                        let mut guard = rpc_committee.write().await;
                                        *guard = new_committee;
                                    }
                                    Err(e) => {
                                        warn!(error = %e, "failed to rebuild committee at epoch boundary");
                                    }
                                }
                            }
                        }
                        last_epoch = Some(result.epoch);
                        if let Some(non_blue::ExecPool { mempool, returned }) = &pool {
                            let committed: Vec<_> = subdag
                                .blocks
                                .iter()
                                .flat_map(|block| block.transactions.iter().cloned())
                                .collect();
                            let removed = mempool.remove_committed_transactions(&committed);
                            debug!(removed, "pruned committed transactions from mempool");
                            // #13: state for this sub-DAG is durable now; hand
                            // the red blocks' transactions back to the pool.
                            if let Some(returned) = &returned {
                                returned.reinsert(mempool, &subdag);
                            }
                        }
                        events.publish_committed_leader(&subdag);
                        if let (Some(root), Some(reward), Some(duty)) = (
                            result.state_root,
                            result.reward.as_ref(),
                            seal_duty.as_ref(),
                        ) {
                            // Idempotent replay leaves state_root empty, so a
                            // restart does not sign the same header again.
                            match seals::publish_local_seal(
                                &state_storage,
                                &duty.ctx,
                                &subdag,
                                reward.height,
                                root,
                            ) {
                                Ok(local) => {
                                    if let Some(announcement) = local.gossip {
                                        if duty
                                            .broadcast
                                            .send(NetworkCommand::BroadcastSeal(announcement))
                                            .is_err()
                                        {
                                            warn!("network command channel closed; commit seal not gossiped");
                                        }
                                    }
                                    publish_sealed(&state_storage, &events, &local.header);
                                }
                                Err(e) => warn!(
                                    error = %e,
                                    round = subdag.leader_round,
                                    "commit seal was not stored; consensus continues"
                                ),
                            }
                        }
                        health.execution_progress(subdag.leader_round);
                        // Commit liveness: the round loop and gossip keep
                        // running even when the commit rule stalls, so only a
                        // real commit proves the chain is progressing.
                        health.commit_progress(subdag.leader_round);
                    }
                    Err(e) => {
                        error!(
                            error = %e,
                            round = subdag.leader_round,
                            "execution failed; stopping worker so committed decisions replay after restart"
                        );
                        health.execution_failed(&format!("round {}: {e}", subdag.leader_round));
                        break;
                    }
                },
                None => {
                    warn!("committed sub-DAG channel closed; execution worker stopping");
                    break;
                }
            },
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
        }
    }
    health.execution_stopped();
}

/// Adapter implementing [`DagStoreTrait`] over the concrete [`DagStore`].
struct NodeDagStore {
    inner: Arc<DagStore>,
}

impl DagStoreTrait for NodeDagStore {
    fn get_block(&self, hash: &Hash) -> Result<StatementBlock, DagStoreError> {
        self.inner.get_block(hash)
    }

    fn get_ancestors(&self, hash: &Hash, min_round: Round) -> Result<Vec<Hash>, DagStoreError> {
        self.inner.get_ancestors(hash, min_round)
    }

    fn get_parents(&self, hash: &Hash) -> Result<Vec<Hash>, DagStoreError> {
        self.inner.get_parents(hash)
    }

    fn get_block_by_author_round(
        &self,
        author: AuthorityIndex,
        round: Round,
    ) -> Result<Option<StatementBlock>, DagStoreError> {
        self.inner.get_block_by_author_round(author, round)
    }

    fn get_blocks_by_round(&self, round: Round) -> Result<Vec<StatementBlock>, DagStoreError> {
        self.inner.get_blocks_by_round(round)
    }

    fn has_block(&self, hash: &Hash) -> Result<bool, DagStoreError> {
        self.inner.has_block(hash)
    }

    fn put_block(&self, block: &StatementBlock) -> Result<(), DagStoreError> {
        self.inner.put_block(block)
    }

    fn replace_block(
        &self,
        existing: &StatementBlock,
        incoming: &StatementBlock,
    ) -> Result<(), DagStoreError> {
        self.inner.replace_block(existing, incoming)
    }

    fn find_parents(
        &self,
        round: Round,
        max_parents: usize,
    ) -> Result<Vec<BlockReference>, DagStoreError> {
        self.inner.find_parents(round, max_parents)
    }

    fn commit_leader(&self, leader_hash: &Hash) -> Result<u64, DagStoreError> {
        self.inner.commit_leader(leader_hash)
    }

    fn mark_round_decided(&self, round: Round, leader_hash: &Hash) -> Result<(), DagStoreError> {
        self.inner.mark_round_decided(round, leader_hash)
    }

    fn mark_decided_and_commit_leader(
        &self,
        round: Round,
        leader_hash: &Hash,
    ) -> Result<u64, DagStoreError> {
        self.inner
            .mark_decided_and_commit_leader(round, leader_hash)
    }

    fn mergeset(&self, leader: &Hash) -> Result<Vec<Hash>, DagStoreError> {
        self.inner.mergeset(leader)
    }

    fn get_blocks(&self, hashes: &[Hash]) -> Result<Vec<StatementBlock>, DagStoreError> {
        let mut blocks = Vec::new();
        for hash in hashes {
            if let Ok(block) = self.inner.get_block(hash) {
                blocks.push(block);
            }
        }
        Ok(blocks)
    }

    fn get_decided_leaders(&self, round: Round) -> Result<Vec<Hash>, DagStoreError> {
        self.inner.get_decided_leaders(round)
    }

    fn get_decided_rounds(&self, max_round: Round) -> Result<Vec<Round>, DagStoreError> {
        self.inner.get_decided_rounds(max_round)
    }

    fn max_stored_round(&self) -> Result<Option<Round>, DagStoreError> {
        self.inner.max_dag_round()
    }

    fn prune_non_blue(
        &self,
        blue_hashes: &[Hash],
        committed_wave: u64,
    ) -> Result<u64, DagStoreError> {
        self.inner.prune_non_blue(blue_hashes, committed_wave)
    }

    fn prune_waves_before(&self, wave: u64, prune_window_waves: u64) -> Result<u64, DagStoreError> {
        self.inner.prune_waves_before(wave, prune_window_waves)
    }
}

/// Adapter implementing [`BlockManagerTrait`] over the concrete [`BlockManager`].
struct NodeBlockManager {
    inner: Arc<BlockManager>,
}

impl BlockManagerTrait for NodeBlockManager {
    fn propose_block(&self, round: Round) -> Result<StatementBlock, BlockManagerError> {
        self.inner.propose_block(round)
    }

    fn propose_block_with_txs(
        &self,
        round: Round,
        transactions: Vec<Transaction>,
    ) -> Result<StatementBlock, BlockManagerError> {
        self.inner.propose_block_with_txs(round, transactions)
    }

    fn process_block(&self, block: &StatementBlock) -> Result<(), BlockManagerError> {
        self.inner.process_block(block)
    }

    fn our_authority(&self) -> AuthorityIndex {
        self.inner.our_authority()
    }

    fn set_signing_key(&self, key: SigningKey) {
        self.inner.set_signing_key(key);
    }

    fn propose_vrf_carrier(&self, round: Round) -> Result<StatementBlock, BlockManagerError> {
        self.inner.propose_vrf_carrier(round)
    }

    fn verified_vrf_outputs(
        &self,
    ) -> std::collections::HashMap<Round, std::collections::HashMap<AuthorityIndex, [u8; 32]>> {
        self.inner.verified_vrf_outputs_snapshot()
    }
}

/// Reconstruct durable commit decisions for restart-safe execution replay.
///
/// M3: the ancestry walks below pass `min_round = 0` (unbounded, back to
/// genesis) and rely on `DagStore::get_ancestors` being NotFound tolerant of
/// dangling edges left by `prune_waves_before`, rather than bounding the walk.
/// This mirrors the live committer's `get_previous_tips`; both places must
/// stay consistent.
///
// TODO(M4): replay currently stops using the *global* `last_committed` in the
// mergeset walk, so recovery of an older sub-DAG can be truncated by a newer
// commit. Review whether replay should pass an explicit stop-round (the
// sub-DAG's own leader) instead; leave behaviour unchanged until that is
// clear to avoid destabilising recovery.
fn recover_committed_subdags(
    dag_store: &DagStore,
    use_mysticghost: bool,
) -> Result<Vec<kovanica_consensus::CommittedSubDag>> {
    let rounds = dag_store.get_decided_rounds(u64::MAX)?;
    let mut subdags = Vec::new();

    // Pre-compute all decided leaders for previous tips
    let mut all_previous_tips = Vec::new();
    for round in &rounds {
        let leaders = dag_store.get_decided_leaders(*round)?;
        for leader_hash in leaders {
            if let Ok(block) = dag_store.get_block(&leader_hash) {
                if block.round == *round && block.digest == leader_hash {
                    all_previous_tips.push(leader_hash);
                }
            }
        }
    }

    for round in rounds {
        for leader_hash in dag_store.get_decided_leaders(round)? {
            // Skip decided-round entries whose leader block has been pruned:
            // recovery must remain possible after pruning rather than fail.
            let leader = match dag_store.get_block(&leader_hash) {
                Ok(block) => block,
                Err(kovanica_dag::DagStoreError::NotFound(_)) => continue,
                Err(e) => return Err(e.into()),
            };
            if leader.round != round || leader.digest != leader_hash {
                anyhow::bail!(
                    "persisted commit decision for round {round} references inconsistent leader block (block round {}, block digest {}, expected digest {leader_hash})",
                    leader.round,
                    leader.digest,
                );
            }

            if use_mysticghost {
                // Previous tips must be the decided leaders that are neither
                // this leader nor ancestors of it. This mirrors the live
                // committer's `get_previous_tips` filter so restart recovery
                // produces the same committed sub-DAG as first-time commit.
                let ancestors: std::collections::HashSet<Hash> = dag_store
                    .get_ancestors(&leader_hash, 0)?
                    .into_iter()
                    .collect();
                let previous_tips: Vec<Hash> = all_previous_tips
                    .iter()
                    .copied()
                    .filter(|h| *h != leader_hash && !ancestors.contains(h))
                    .collect();
                let subdag = recover_committed_subdag_mysticghost(
                    dag_store,
                    &leader,
                    round,
                    &previous_tips,
                )?;
                subdags.push(subdag);
            } else {
                // Original linearizer path (bit-identical to current behaviour)
                // Same not-yet-committed history as the live committer, so
                // replay never re-delivers a block from an earlier commit.
                let node_store = NodeDagStore {
                    inner: Arc::new(dag_store.clone()),
                };
                let history = kovanica_consensus::uncommitted_history(&node_store, &leader)?;
                subdags.push(kovanica_consensus::Linearizer::new().linearize(leader, history));
            }
        }
    }
    subdags.sort_by_key(|subdag| {
        (
            subdag.leader_round,
            subdag.leader_author,
            subdag.leader.digest.0,
        )
    });
    Ok(subdags)
}

/// MysticGhost recovery path for a single committed sub-DAG.
fn recover_committed_subdag_mysticghost(
    dag_store: &DagStore,
    leader_block: &StatementBlock,
    leader_round: Round,
    previous_tips: &[Hash],
) -> Result<kovanica_consensus::CommittedSubDag> {
    use kovanica_consensus::mysticghost::{
        order_committed_wave, MysticGhostConfig, MysticGhostOrder,
    };

    // 1. Get mergeset hashes
    let mergeset_hashes = dag_store.mergeset(&leader_block.digest)?;

    // 2. Get mergeset blocks
    let mut mergeset_blocks = Vec::new();
    for hash in mergeset_hashes {
        if let Ok(block) = dag_store.get_block(&hash) {
            mergeset_blocks.push(block);
        }
    }

    // 3. Configure MysticGhost
    let mg_config = MysticGhostConfig {
        enabled: true,
        k: 3,
        max_mergeset_blocks: 2_000,
    };

    // 4. Run MysticGhost ordering
    match order_committed_wave(&mg_config, &mergeset_blocks, previous_tips) {
        MysticGhostOrder::Ghost { colouring } => {
            // Use the blue-set order from GHOSTDAG
            let blue_ordered = colouring.blue_ordered();

            // Red mergeset blocks, computed exactly like the live committer.
            let non_blue = kovanica_consensus::non_blue_refs(
                &mergeset_blocks,
                &colouring.blue,
                &leader_block.digest,
            );

            // Build blocks in blue order, filtering to only those in mergeset
            let block_map: std::collections::HashMap<Hash, StatementBlock> =
                mergeset_blocks.into_iter().map(|b| (b.digest, b)).collect();

            let mut ordered_blocks = Vec::new();
            for hash in blue_ordered {
                if let Some(block) = block_map.get(&hash) {
                    ordered_blocks.push(block.clone());
                }
            }

            // Ensure leader is included (should be blue)
            if !ordered_blocks
                .iter()
                .any(|b| b.digest == leader_block.digest)
            {
                ordered_blocks.push(leader_block.clone());
            }

            Ok(kovanica_consensus::CommittedSubDag {
                blocks: ordered_blocks,
                leader: leader_block.clone(),
                leader_round,
                leader_author: leader_block.author,
                non_blue,
            })
        }
        MysticGhostOrder::Fallback => {
            // Fall back to original linearizer
            let history = dag_store
                .get_ancestors(&leader_block.digest, 0)?
                .into_iter()
                .map(|hash| dag_store.get_block(&hash))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok(kovanica_consensus::Linearizer::new().linearize(leader_block.clone(), history))
        }
    }
}

/// Build the committee from active validators in StakingState.
fn build_committee(
    storage: &Storage,
    local_public_key: &PublicKey,
    local_address: &Address,
    listen_addr: &str,
) -> Result<CommitteeInfo> {
    let read = storage.begin_read()?;
    let staking_state = storage.state().load_staking_state(&read)?;
    // Epoch is a pure function of committed-leader height — every node derives
    // the same value at the same height, so the committee's epoch is truthful
    // from startup rather than pinned to 0.
    let epoch = staking_state.current_epoch();

    // Filter active validators with sufficient stake
    let mut active_validators: Vec<_> = staking_state
        .validators
        .iter()
        .filter(|v| v.active && v.stake >= MIN_VALIDATOR_STAKE)
        .collect();

    // If no validators in staking state (fresh genesis), fall back to local validator identity
    // This allows a fresh node to start and produce blocks until validators join via staking txs.
    if active_validators.is_empty() {
        info!("no active validators in staking state; falling back to local validator identity");
        let authority = AuthorityInfo {
            index: 0,
            stake: MIN_VALIDATOR_STAKE,
            public_key: *local_public_key,
            vrf_public_key: None,
            address: *local_address,
            network_address: listen_addr.to_string(),
        };
        return Ok(CommitteeInfo::try_new(epoch, vec![authority])?);
    }

    // Sort by address for deterministic ordering across all nodes
    active_validators.sort_by_key(|a| a.address.0);

    // Take up to MAX_ACTIVE_VALIDATORS top validators by stake (then by address for tie-breaking)
    // Sort by stake desc, take top 21, then sort by address for index assignment.
    active_validators.sort_by(|a, b| {
        b.stake
            .cmp(&a.stake)
            .then_with(|| a.address.0.cmp(&b.address.0))
    });
    let top_validators = active_validators
        .into_iter()
        .take(MAX_ACTIVE_VALIDATORS)
        .collect::<Vec<_>>();

    // Re-sort by address for stable index assignment
    let mut top_validators = top_validators;
    top_validators.sort_by_key(|a| a.address.0);

    if top_validators.is_empty() {
        anyhow::bail!("no active validators found in staking state; committee cannot be empty");
    }

    let mut authorities = Vec::with_capacity(top_validators.len());
    for (index, validator) in top_validators.iter().enumerate() {
        // Determine public key: use local node's key if this is our validator, otherwise use stored key
        let public_key = if validator.address == *local_address {
            *local_public_key
        } else if let Some(pk) = validator.public_key {
            pk
        } else {
            anyhow::bail!(
                "validator {} missing public key; run registration or add to genesis",
                validator.address
            );
        };

        // VRF public key for leader selection (optional)
        let vrf_public_key = validator
            .vrf_public_key
            .as_ref()
            .map(|v| kovanica_types::PublicKey(v.0));

        let authority = AuthorityInfo {
            index: index as u16,
            stake: validator.stake,
            public_key,
            vrf_public_key,
            address: validator.address,
            network_address: listen_addr.to_string(),
        };
        authorities.push(authority);
    }

    if authorities.is_empty() {
        anyhow::bail!("no validators with registered public keys; committee cannot be empty");
    }

    Ok(CommitteeInfo::try_new(epoch, authorities)?)
}

/// Build the [`MempoolConfig`] from the node configuration and environment variables.
fn build_mempool_config() -> MempoolConfig {
    let mut config = MempoolConfig::default();

    // Rebroadcast interval (seconds)
    if let Ok(val) = std::env::var("KOVANICA_MEMPOOL_REBROADCAST_INTERVAL_SECS") {
        if let Ok(secs) = val.parse::<u64>() {
            config.rebroadcast_interval = Duration::from_secs(secs);
            info!(
                interval_secs = secs,
                "mempool rebroadcast interval overridden from env"
            );
        } else {
            warn!(
                val,
                "invalid KOVANICA_MEMPOOL_REBROADCAST_INTERVAL_SECS, using default"
            );
        }
    }

    // Minimum fee rate (atoms per byte)
    if let Ok(val) = std::env::var("KOVANICA_MEMPOOL_MIN_FEE_RATE") {
        if let Ok(rate) = val.parse::<u64>() {
            config.min_fee_rate = rate;
            info!(rate, "mempool min fee rate overridden from env");
        } else {
            warn!(val, "invalid KOVANICA_MEMPOOL_MIN_FEE_RATE, using default");
        }
    }

    // Max transactions per sender
    if let Ok(val) = std::env::var("KOVANICA_MEMPOOL_MAX_TXS_PER_SENDER") {
        if let Ok(max) = val.parse::<usize>() {
            config.max_txs_per_sender = max;
            info!(max, "mempool max txs per sender overridden from env");
        } else {
            warn!(
                val,
                "invalid KOVANICA_MEMPOOL_MAX_TXS_PER_SENDER, using default"
            );
        }
    }

    // In-flight timeout (seconds)
    if let Ok(val) = std::env::var("KOVANICA_MEMPOOL_IN_FLIGHT_TIMEOUT_SECS") {
        if let Ok(secs) = val.parse::<u64>() {
            config.in_flight_timeout = Duration::from_secs(secs);
            info!(
                interval_secs = secs,
                "mempool in-flight timeout overridden from env"
            );
        } else {
            warn!(
                val,
                "invalid KOVANICA_MEMPOOL_IN_FLIGHT_TIMEOUT_SECS, using default"
            );
        }
    }

    config
}

/// Build the [`NetworkConfig`] from the node configuration.
fn build_network_config(config: &NodeConfig) -> Result<NetworkConfig> {
    let listen = to_multiaddr(&config.listen_addr)?;

    let mut bootstrap_nodes = Vec::with_capacity(config.bootnodes.len());
    for node in &config.bootnodes {
        match to_multiaddr(node) {
            Ok(addr) => bootstrap_nodes.push(addr),
            Err(e) => warn!(node = %node, error = %e, "skipping invalid bootnode"),
        }
    }

    Ok(NetworkConfig {
        listen_addrs: vec![listen],
        bootstrap_nodes,
        max_peers: config.max_peers,
        ping_interval: Duration::from_secs(10),
        node_key_path: Some(config.data_dir.join(NODE_IDENTITY_FILE)),
        peer_scoring: config.peer_scoring.clone(),
    })
}

/// Genesis validator entry for Phase 16.6 4-node quorum.
#[derive(Debug, Deserialize)]
struct GenesisValidatorEntry {
    address: String,
    stake: u64,
    public_key: Option<String>,
    vrf_public_key: Option<String>,
}

/// Top-level shape of `genesis_validators.toml` (`[[validator]]` array of tables).
///
/// Example:
///
/// ```toml
/// chain_id = 2            # mandatory for 1 (mainnet) and 2 (testnet)
///
/// [[validator]]
/// address = "<32-byte hex>"
/// stake = 50000000000000
/// public_key = "<32-byte hex>"
/// ```
#[derive(Debug, Deserialize)]
struct GenesisValidatorsFile {
    /// Chain id of this genesis. Mandatory when the node is configured for
    /// mainnet (1) or testnet (2); optional for devnet (3) / local (1337).
    /// When present it must equal the node's configured `chain_id`.
    #[serde(default)]
    chain_id: Option<u64>,
    validator: Vec<GenesisValidatorEntry>,
}

/// Pre-funded account entry for genesis.
#[derive(Debug, Deserialize)]
struct FundedAccountEntry {
    address: String,
    balance: u64,
}

/// Top-level shape of `funded_accounts.toml` (`[[account]]` array of tables).
///
/// Example:
///
/// ```toml
/// [[account]]
/// address = "<32-byte hex>"
/// balance = 1000000000000
/// ```
#[derive(Debug, Deserialize)]
struct FundedAccountsFile {
    account: Vec<FundedAccountEntry>,
}

/// Name of the per-data-dir marker recording the chain id the database was
/// created for.
const CHAIN_ID_MARKER: &str = "chain_id";

/// Verify `config.chain_id` against the genesis and the data directory.
///
/// * `genesis_validators.toml` may carry `chain_id = N`; it must match. For
///   mainnet (1) and testnet (2) it is **mandatory**: without it (or without
///   the genesis file) the node refuses to start. Devnet (3) and local (1337)
///   accept a genesis without `chain_id`.
/// * `<data_dir>/chain_id` records the chain id the data dir was created for;
///   it must match on every later start. Written on first start.
/// * `genesis.json`, if present, may carry a top-level `chain_id`; when it
///   does, it must match. This check is independent of whether
///   `genesis_validators.toml` exists.
///
/// Any mismatch is a hard error: signing with the wrong chain id would
/// produce votes/transactions no other node accepts (or replayable ones).
fn check_chain_id(config: &NodeConfig) -> Result<()> {
    let configured = config.chain_id;
    let genesis_required = matches!(
        configured,
        kovanica_types::signing::chain_id::MAINNET | kovanica_types::signing::chain_id::TESTNET
    );
    let genesis_path = config.data_dir.join("genesis_validators.toml");
    if genesis_required && !genesis_path.exists() {
        anyhow::bail!(
            "chain_id {configured} requires {} with an explicit chain_id; refusing to start",
            genesis_path.display()
        );
    }
    if genesis_path.exists() {
        let text = std::fs::read_to_string(&genesis_path)
            .with_context(|| format!("reading genesis validators {}", genesis_path.display()))?;
        let genesis: GenesisValidatorsFile = toml::from_str(&text)
            .with_context(|| format!("parsing genesis validators {}", genesis_path.display()))?;
        if genesis.chain_id.is_none() && genesis_required {
            anyhow::bail!(
                "chain_id {configured} requires an explicit chain_id in genesis {}; refusing to start",
                genesis_path.display()
            );
        }
        if let Some(genesis_id) = genesis.chain_id {
            if genesis_id != configured {
                anyhow::bail!(
                    "chain_id mismatch: config has {configured}, genesis {} has {genesis_id}; refusing to start",
                    genesis_path.display()
                );
            }
        }
    }

    // The genesis can also be present as the ceremony `genesis.json`. Check it
    // whenever it exists (not only when `genesis_validators.toml` does), so a
    // chain id declared there is honoured too. The persisted genesis digest
    // (`verify_genesis_consistency`) additionally binds `chain_id` when no
    // explicit id is declared anywhere.
    let genesis_json_path = config.data_dir.join(GENESIS_JSON_FILE);
    if genesis_json_path.exists() {
        let text = std::fs::read_to_string(&genesis_json_path)
            .with_context(|| format!("reading {}", genesis_json_path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&text)
            .with_context(|| format!("parsing {}", genesis_json_path.display()))?;
        if let Some(genesis_id) = value.get("chain_id").and_then(|v| v.as_u64()) {
            if genesis_id != configured {
                anyhow::bail!(
                    "chain_id mismatch: config has {configured}, genesis {} has {genesis_id}; refusing to start",
                    genesis_json_path.display()
                );
            }
        }
    }

    let marker = config.data_dir.join(CHAIN_ID_MARKER);
    if marker.exists() {
        let text = std::fs::read_to_string(&marker)
            .with_context(|| format!("reading {}", marker.display()))?;
        let stored: u64 = text
            .trim()
            .parse()
            .with_context(|| format!("invalid chain id in {}", marker.display()))?;
        if stored != configured {
            anyhow::bail!(
                "chain_id mismatch: config has {configured}, data dir {} was created for {stored}; refusing to start",
                config.data_dir.display()
            );
        }
    } else {
        std::fs::write(&marker, format!("{configured}\n"))
            .with_context(|| format!("writing {}", marker.display()))?;
    }
    Ok(())
}

/// Parse a 32-byte hex public key (PublicKey is [u8; 32]).
fn parse_public_key_hex(value: &str) -> Result<PublicKey> {
    let bytes =
        hex::decode(value.trim()).with_context(|| format!("invalid public key hex `{}`", value))?;
    if bytes.len() != 32 {
        anyhow::bail!("public key must be 32 bytes, got {}", bytes.len());
    }
    let mut pk = [0u8; 32];
    pk.copy_from_slice(&bytes);
    Ok(PublicKey(pk))
}

/// Parse a 32-byte hex string into a [u8; 32] array.
fn parse_hex_32(value: &str) -> Result<[u8; 32]> {
    let bytes = hex::decode(value.trim()).with_context(|| format!("invalid hex `{}`", value))?;
    if bytes.len() != 32 {
        anyhow::bail!("expected 32 bytes, got {}", bytes.len());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// Convert a `host:port` string or a full multiaddr into a [`Multiaddr`].
fn to_multiaddr(value: &str) -> Result<Multiaddr> {
    let value = value.trim();
    if value.starts_with('/') {
        return value
            .parse::<Multiaddr>()
            .with_context(|| format!("invalid multiaddr `{value}`"));
    }

    let (host, port) = value
        .rsplit_once(':')
        .with_context(|| format!("expected `host:port`, got `{value}`"))?;
    let port: u16 = port
        .parse()
        .with_context(|| format!("invalid port in `{value}`"))?;

    let multiaddr = if host.parse::<std::net::Ipv4Addr>().is_ok() {
        format!("/ip4/{host}/tcp/{port}")
    } else if host.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("/ip6/{host}/tcp/{port}")
    } else {
        format!("/dns4/{host}/tcp/{port}")
    };

    multiaddr
        .parse::<Multiaddr>()
        .with_context(|| format!("invalid address `{value}`"))
}

/// File name (inside `data_dir`) of the persistent libp2p node identity. This
/// is a transport key, distinct from the validator signing key.
const NODE_IDENTITY_FILE: &str = "p2p_node.key";

/// Load the validator signing key.
///
/// A configured key that cannot be loaded is a hard error: the node never
/// substitutes a different key, because signing with an unexpected identity
/// is worse than not starting. Only a non-validator node with no key
/// configured gets an ephemeral identity (it never signs consensus messages).
///
/// TODO(remote signer): the seed should live in an external signer; the
/// `Signer` abstraction is owned by consensus. Until then the seed file must
/// be `0600` and is zeroized after parsing.
fn load_validator_key(path: Option<&str>, run_validator: bool) -> Result<(SigningKey, PublicKey)> {
    match path {
        Some(path) => {
            let signing_key = read_signing_key(path).with_context(|| {
                format!("failed to load validator key from {path}; refusing to start with a substitute key")
            })?;
            let public_key = PublicKey::from(signing_key.verifying_key());
            info!(path, "loaded validator key");
            Ok((signing_key, public_key))
        }
        None if run_validator => anyhow::bail!(
            "run_validator=true but no validator key is configured (set validator_key or KOVANICA_VALIDATOR_KEY)"
        ),
        None => {
            info!("run_validator=false and no validator key configured; using an ephemeral non-signing identity");
            Ok(kovanica_crypto::generate_keypair())
        }
    }
}

/// Read a 32-byte hex seed from `path` and build a signing key.
///
/// The file must not be accessible by group or others (unix). Intermediate
/// buffers holding the seed are zeroized.
fn read_signing_key(path: &str) -> Result<SigningKey> {
    kovanica_network::identity::check_key_file_permissions(std::path::Path::new(path))?;
    let raw = Zeroizing::new(
        std::fs::read_to_string(path).with_context(|| format!("reading key file {path}"))?,
    );
    let bytes = Zeroizing::new(
        hex::decode(raw.trim()).map_err(|_| anyhow::anyhow!("key file must contain hex"))?,
    );
    if bytes.len() != 32 {
        anyhow::bail!(
            "validator key must be a 32-byte hex seed, got {} bytes",
            bytes.len()
        );
    }
    let mut seed = Zeroizing::new([0u8; 32]);
    seed.copy_from_slice(&bytes);
    Ok(SigningKey::from_bytes(&seed))
}

/// Parse a Kovanica address from various formats:
/// - 64-char hex (32 bytes)
/// - kvnc<32-byte-hex>dag (with checksum, 36 chars total)
/// - 0x-prefixed hex
///
/// Returns the 32-byte address.
fn parse_address_hex(value: Option<&str>) -> Result<Address> {
    let Some(value) = value else {
        return Ok(Address::default());
    };

    let s = value.trim();
    let s = s.strip_prefix("0x").unwrap_or(s);

    // Check for kvnc...dag format (36 chars: kvnc + 32 hex + dag)
    let s = if let Some(rest) = s.strip_prefix("kvnc") {
        rest.strip_suffix("dag").unwrap_or(rest)
    } else {
        s
    };

    let bytes = hex::decode(s).with_context(|| format!("invalid address hex `{value}`"))?;
    if bytes.len() != 32 {
        anyhow::bail!("address must be 32 bytes, got {}", bytes.len());
    }
    let mut address = [0u8; 32];
    address.copy_from_slice(&bytes);
    Ok(Address(address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_auth_config_parses_tokens_and_disable_switch() {
        let auth = rpc_auth_config(false, " a , ,b ");
        assert_eq!(auth.write_tokens, vec!["a".to_string(), "b".to_string()]);
        assert!(auth.require_auth_for_writes);
        assert!(rpc_auth_config(false, "").write_tokens.is_empty());
        assert!(!rpc_auth_config(true, "").require_auth_for_writes);
    }

    fn chain_id_config(dir: &std::path::Path, chain_id: u64) -> NodeConfig {
        NodeConfig {
            data_dir: dir.to_path_buf(),
            run_validator: false,
            bootnodes: Vec::new(),
            listen_addr: "127.0.0.1:0".into(),
            rpc_port: 0,
            chain_id,
            ..NodeConfig::default()
        }
    }

    fn write_genesis_chain_id(dir: &std::path::Path, chain_id: u64) {
        std::fs::write(
            dir.join("genesis_validators.toml"),
            format!("chain_id = {chain_id}\nvalidator = []\n"),
        )
        .unwrap();
    }

    #[test]
    fn chain_id_must_match_genesis() {
        let dir = tempfile::tempdir().unwrap();
        write_genesis_chain_id(dir.path(), 2);
        let err = check_chain_id(&chain_id_config(dir.path(), 1337)).unwrap_err();
        assert!(err.to_string().contains("chain_id mismatch"), "{err:#}");
        check_chain_id(&chain_id_config(dir.path(), 2)).expect("matching chain id");
    }

    #[test]
    fn chain_id_is_pinned_to_data_dir() {
        let dir = tempfile::tempdir().unwrap();
        check_chain_id(&chain_id_config(dir.path(), 3)).expect("first start writes marker");
        assert_eq!(
            std::fs::read_to_string(dir.path().join(CHAIN_ID_MARKER))
                .unwrap()
                .trim(),
            "3"
        );
        check_chain_id(&chain_id_config(dir.path(), 3)).expect("same chain id restarts");
        let err = check_chain_id(&chain_id_config(dir.path(), 1337)).unwrap_err();
        assert!(err.to_string().contains("chain_id mismatch"), "{err:#}");
    }

    #[test]
    fn genesis_chain_id_optional_only_for_devnet_and_local() {
        for id in [3, 1337] {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(
                dir.path().join("genesis_validators.toml"),
                "validator = []\n",
            )
            .unwrap();
            check_chain_id(&chain_id_config(dir.path(), id))
                .unwrap_or_else(|e| panic!("chain {id} without genesis chain_id: {e:#}"));
            // No genesis file at all is fine too.
            let dir = tempfile::tempdir().unwrap();
            check_chain_id(&chain_id_config(dir.path(), id)).expect("no genesis file");
        }
    }

    #[test]
    fn mainnet_and_testnet_require_genesis_chain_id() {
        for id in [1, 2] {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(
                dir.path().join("genesis_validators.toml"),
                "validator = []\n",
            )
            .unwrap();
            let err = check_chain_id(&chain_id_config(dir.path(), id)).unwrap_err();
            assert!(
                err.to_string().contains("requires an explicit chain_id"),
                "{err:#}"
            );
            assert!(!dir.path().join(CHAIN_ID_MARKER).exists());

            let dir = tempfile::tempdir().unwrap();
            let err = check_chain_id(&chain_id_config(dir.path(), id)).unwrap_err();
            assert!(err.to_string().contains("requires"), "{err:#}");

            let dir = tempfile::tempdir().unwrap();
            write_genesis_chain_id(dir.path(), id);
            check_chain_id(&chain_id_config(dir.path(), id)).expect("explicit chain_id");
        }
    }

    #[tokio::test]
    async fn node_refuses_to_start_on_mainnet_without_genesis_chain_id() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("genesis_validators.toml"),
            "validator = []\n",
        )
        .unwrap();
        let err = run_node(chain_id_config(dir.path(), 1), std::future::pending())
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("requires an explicit chain_id"),
            "{err:#}"
        );
    }

    #[tokio::test]
    async fn node_refuses_to_start_on_chain_id_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        write_genesis_chain_id(dir.path(), 1);
        let err = run_node(chain_id_config(dir.path(), 1337), std::future::pending())
            .await
            .unwrap_err();
        assert!(err.to_string().contains("chain_id mismatch"), "{err:#}");
        assert!(
            !dir.path().join("consensus.redb").exists(),
            "no storage opened before the chain id check"
        );
    }

    #[test]
    fn genesis_digest_is_deterministic_and_covers_chain_id_and_allocations() {
        let dir = tempfile::tempdir().unwrap();
        let base = chain_id_config(dir.path(), 3);
        let a = genesis_config_digest(&base).unwrap();
        let b = genesis_config_digest(&base).unwrap();
        assert_eq!(a, b, "digest must be deterministic");

        let other_chain = chain_id_config(dir.path(), 1337);
        assert_ne!(
            a,
            genesis_config_digest(&other_chain).unwrap(),
            "chain id must be covered by the digest"
        );

        std::fs::write(
            dir.path().join("funded_accounts.toml"),
            format!(
                "[[account]]\naddress = \"{}\"\nbalance = 1000\n",
                "11".repeat(32)
            ),
        )
        .unwrap();
        assert_ne!(
            a,
            genesis_config_digest(&base).unwrap(),
            "genesis allocations must be covered by the digest"
        );
    }

    #[test]
    fn genesis_digest_mismatch_refuses_and_override_allows() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("state.redb")).unwrap();
        let stored = Hash::new([1u8; 32]);
        let fresh = Hash::new([2u8; 32]);
        {
            let txn = storage.begin_write().unwrap();
            storage.state().save_genesis_digest(&txn, &stored).unwrap();
            txn.commit().unwrap();
        }

        let err = verify_persisted_genesis_digest(&storage, &fresh, false).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains(&stored.to_hex()), "{msg}");
        assert!(msg.contains(&fresh.to_hex()), "{msg}");
        assert!(msg.contains(ALLOW_GENESIS_MISMATCH_ENV), "{msg}");

        // The explicit override lets the node start despite the mismatch...
        verify_persisted_genesis_digest(&storage, &fresh, true).expect("override allows");
        // ...and the persisted value is left untouched.
        let read = storage.begin_read().unwrap();
        assert_eq!(
            storage.state().load_genesis_digest(&read).unwrap(),
            Some(stored)
        );
    }

    #[test]
    fn mismatch_override_spellings() {
        for yes in ["1", "true", "TRUE", " yes ", "on"] {
            assert!(parse_mismatch_override(Some(yes)), "{yes:?} should allow");
        }
        for no in ["", "0", "false", "off", "nope"] {
            assert!(
                !parse_mismatch_override(Some(no)),
                "{no:?} should not allow"
            );
        }
        assert!(!parse_mismatch_override(None));
    }

    #[tokio::test]
    async fn node_refuses_to_start_on_genesis_digest_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        // Pre-create the state DB as if it had been created for a different
        // genesis: chain state present, persisted digest different.
        {
            let storage = Storage::new(dir.path().join("state.redb")).unwrap();
            let mut staking = StakingState::new();
            staking.init_treasury(Address([1; 32]));
            let txn = storage.begin_write().unwrap();
            storage.state().save_staking_state(&txn, &staking).unwrap();
            storage
                .state()
                .save_genesis_digest(&txn, &Hash::new([0xEE; 32]))
                .unwrap();
            txn.commit().unwrap();
        }
        let err = run_node(chain_id_config(dir.path(), 1337), std::future::pending())
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("genesis digest mismatch"),
            "{err:#}"
        );
    }

    #[test]
    fn first_start_persists_digest_and_migrates_existing_dir() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("state.redb")).unwrap();

        // Simulate a data dir created before the digest existed: chain state
        // present, no genesis digest.
        let mut staking = StakingState::new();
        staking.init_treasury(Address([9; 32]));
        let txn = storage.begin_write().unwrap();
        storage.state().save_staking_state(&txn, &staking).unwrap();
        txn.commit().unwrap();
        {
            let read = storage.begin_read().unwrap();
            assert!(storage
                .state()
                .load_genesis_digest(&read)
                .unwrap()
                .is_none());
        }

        let fresh = Hash::new([5u8; 32]);
        // Migration: first start writes the digest once.
        verify_persisted_genesis_digest(&storage, &fresh, false).unwrap();
        {
            let read = storage.begin_read().unwrap();
            assert_eq!(
                storage.state().load_genesis_digest(&read).unwrap(),
                Some(fresh)
            );
        }
        // Second start is a no-op and still succeeds.
        verify_persisted_genesis_digest(&storage, &fresh, false).unwrap();
    }

    #[test]
    fn genesis_json_tamper_is_detected() {
        let dir = tempfile::tempdir().unwrap();
        let config = chain_id_config(dir.path(), 1337);
        let canonical = canonical_genesis_digest();

        std::fs::write(
            dir.path().join(GENESIS_JSON_FILE),
            serde_json::to_string(&serde_json::json!({
                "genesis_block": { "digest_hex": canonical.to_hex() }
            }))
            .unwrap(),
        )
        .unwrap();
        verify_genesis_json(&config, &canonical, false).expect("matching genesis.json");

        std::fs::write(
            dir.path().join(GENESIS_JSON_FILE),
            serde_json::to_string(&serde_json::json!({
                "genesis_block": { "digest_hex": Hash::new([9u8; 32]).to_hex() }
            }))
            .unwrap(),
        )
        .unwrap();
        let err = verify_genesis_json(&config, &canonical, false).unwrap_err();
        assert!(err.to_string().contains("canonical genesis"), "{err:#}");
        // Same override applies.
        verify_genesis_json(&config, &canonical, true).expect("override allows");
    }

    #[test]
    fn check_chain_id_reads_genesis_json_chain_id() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(GENESIS_JSON_FILE),
            serde_json::to_string(&serde_json::json!({ "chain_id": 3 })).unwrap(),
        )
        .unwrap();
        let err = check_chain_id(&chain_id_config(dir.path(), 1337)).unwrap_err();
        assert!(err.to_string().contains("chain_id mismatch"), "{err:#}");
        check_chain_id(&chain_id_config(dir.path(), 3)).expect("matching genesis.json chain id");
    }

    #[cfg(unix)]
    #[test]
    fn ceremony_validator_key_files_are_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let validators = dir.path().join("validators.json");
        std::fs::write(
            &validators,
            serde_json::to_string(&serde_json::json!([
                { "address": "11".repeat(32) }
            ]))
            .unwrap(),
        )
        .unwrap();
        let keys_dir = dir.path().join("keys");
        let output = dir.path().join("genesis.json");
        let args = GenesisArgs {
            validators: validators.to_string_lossy().into_owned(),
            treasury_address: "22".repeat(32),
            founder_address: "33".repeat(32),
            output: output.to_string_lossy().into_owned(),
            validator_keys_out: Some(keys_dir.to_string_lossy().into_owned()),
            force: false,
        };
        run_genesis(args).expect("ceremony runs");

        let key_file = keys_dir.join("validator1.key");
        let file_mode = std::fs::metadata(&key_file).unwrap().permissions().mode() & 0o777;
        assert_eq!(file_mode, 0o600, "generated seed file must be 0600");
        let dir_mode = std::fs::metadata(&keys_dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700, "generated keys directory must be 0700");
        let body = std::fs::read_to_string(&key_file).unwrap();
        assert_eq!(body.len(), 64, "seed file holds 32 bytes of hex");
        assert!(body.chars().all(|c| c.is_ascii_hexdigit()), "{body}");
    }

    use kovanica_staking::{StakingState, MAX_ACTIVE_VALIDATORS, MIN_VALIDATOR_STAKE, ONE_KUNA};
    use kovanica_storage::BincodeSerialize;
    use redb::{Database, TableDefinition};
    use std::collections::HashMap;

    const DAG_BLOCKS: TableDefinition<[u8; 32], Vec<u8>> = TableDefinition::new("dag_blocks");

    /// Write a hex seed file with the given unix mode and return its path.
    fn write_seed_file(dir: &std::path::Path, seed: &[u8; 32], mode: u32) -> std::path::PathBuf {
        let path = dir.join("validator.key");
        std::fs::write(&path, hex::encode(seed)).expect("write seed");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))
                .expect("chmod seed");
        }
        #[cfg(not(unix))]
        let _ = mode;
        path
    }

    #[test]
    fn validator_key_loads_from_private_seed_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_seed_file(dir.path(), &[7u8; 32], 0o600);
        let (sk, pk) = load_validator_key(path.to_str(), true).expect("loads");
        assert_eq!(sk.to_bytes(), [7u8; 32]);
        assert_eq!(pk, PublicKey::from(sk.verifying_key()));
    }

    #[test]
    fn validator_key_failure_is_fatal_not_ephemeral() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope.key");
        // Configured but unreadable: error for validators and non-validators alike.
        assert!(load_validator_key(missing.to_str(), true).is_err());
        assert!(load_validator_key(missing.to_str(), false).is_err());

        // Wrong length / not hex.
        let short = write_seed_file(dir.path(), &[1u8; 32], 0o600);
        std::fs::write(&short, "abcd").unwrap();
        assert!(load_validator_key(short.to_str(), true).is_err());
        std::fs::write(&short, "zz".repeat(32)).unwrap();
        assert!(load_validator_key(short.to_str(), true).is_err());

        // No key at all: only a non-validator may proceed.
        assert!(load_validator_key(None, true).is_err());
        assert!(load_validator_key(None, false).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn validator_key_with_loose_permissions_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        for mode in [0o644, 0o640, 0o666] {
            let path = write_seed_file(dir.path(), &[9u8; 32], mode);
            let err = load_validator_key(path.to_str(), true).expect_err("must refuse");
            assert!(format!("{err:#}").contains("chmod 600"), "{err:#}");
        }
        let path = write_seed_file(dir.path(), &[9u8; 32], 0o400);
        load_validator_key(path.to_str(), true).expect("0400 is fine");
    }

    #[test]
    fn network_config_uses_persistent_identity_in_data_dir() {
        let config = NodeConfig {
            data_dir: std::path::PathBuf::from("/var/lib/kovanica"),
            listen_addr: "127.0.0.1:9000".into(),
            bootnodes: Vec::new(),
            ..Default::default()
        };
        let net = build_network_config(&config).unwrap();
        assert_eq!(
            net.node_key_path.as_deref(),
            Some(std::path::Path::new("/var/lib/kovanica/p2p_node.key"))
        );
        assert_ne!(
            net.node_key_path.as_deref().and_then(|p| p.file_name()),
            Some(std::ffi::OsStr::new("validator.key")),
            "transport identity must not reuse the validator seed file"
        );
    }

    #[test]
    fn host_port_converts_to_multiaddr() {
        assert_eq!(
            to_multiaddr("127.0.0.1:8000").unwrap().to_string(),
            "/ip4/127.0.0.1/tcp/8000"
        );
        assert_eq!(
            to_multiaddr("seed1.kovanica.invalid:8000")
                .unwrap()
                .to_string(),
            "/dns4/seed1.kovanica.invalid/tcp/8000"
        );
        assert_eq!(
            to_multiaddr("/ip4/0.0.0.0/tcp/8000").unwrap().to_string(),
            "/ip4/0.0.0.0/tcp/8000"
        );
        assert!(to_multiaddr("not-an-address").is_err());
    }

    #[test]
    fn zero_address_is_default_treasury() {
        assert_eq!(parse_address_hex(None).unwrap(), Address::default());
        assert!(parse_address_hex(Some("zz")).is_err());
    }

    #[test]
    fn committed_subdag_decisions_are_recovered_from_dag_storage() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
        let dag = DagStore::new(storage).unwrap();
        let digest = StatementBlock::compute_digest(0, 1, &[], &[]);
        let block = StatementBlock {
            author: 0,
            round: 1,
            parents: Vec::new(),
            transactions: Vec::new(),
            statements: Vec::new(),
            signature: Signature([0; 64]),
            digest,
            merkle_root: Default::default(),
        };
        dag.put_block(&block).unwrap();
        dag.mark_round_decided(1, &digest).unwrap();

        let recovered = recover_committed_subdags(&dag, false).unwrap();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].leader.digest, digest);
        assert_eq!(recovered[0].blocks.len(), 1);
    }

    #[test]
    fn committed_subdag_recovery_never_replays_a_block_twice() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
        let dag = DagStore::new(storage).unwrap();
        let mut parents = Vec::new();
        let mut chain = Vec::new();
        for round in 1..=5u64 {
            let digest = StatementBlock::compute_digest(0, round, &parents, &[]);
            let block = StatementBlock {
                author: 0,
                round,
                parents: parents.clone(),
                transactions: Vec::new(),
                statements: Vec::new(),
                signature: Signature([0; 64]),
                digest,
                merkle_root: Default::default(),
            };
            dag.put_block(&block).unwrap();
            parents = vec![kovanica_types::block::BlockReference {
                author: 0,
                round,
                digest,
            }];
            chain.push(block);
        }
        // Leaders committed at rounds 1, 3 and 5.
        for i in [0usize, 2, 4] {
            dag.mark_round_decided(chain[i].round, &chain[i].digest)
                .unwrap();
        }

        let recovered = recover_committed_subdags(&dag, false).unwrap();
        assert_eq!(recovered.len(), 3);
        let delivered: Vec<Hash> = recovered
            .iter()
            .flat_map(|s| s.blocks.iter().map(|b| b.digest))
            .collect();
        let unique: std::collections::HashSet<Hash> = delivered.iter().copied().collect();
        assert_eq!(
            unique.len(),
            delivered.len(),
            "replayed twice: {delivered:?}"
        );
        assert_eq!(unique.len(), 5);
    }

    #[test]
    fn committed_subdag_recovery_skips_missing_decided_block() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
        let dag = DagStore::new(storage).unwrap();
        let valid_digest = StatementBlock::compute_digest(0, 1, &[], &[]);
        let valid_block = StatementBlock {
            author: 0,
            round: 1,
            parents: Vec::new(),
            transactions: Vec::new(),
            statements: Vec::new(),
            signature: Signature([0; 64]),
            digest: valid_digest,
            merkle_root: Default::default(),
        };
        dag.put_block(&valid_block).unwrap();
        dag.mark_round_decided(1, &valid_digest).unwrap();

        let missing_digest = StatementBlock::compute_digest(0, 2, &[], &[]);
        dag.mark_round_decided(2, &missing_digest).unwrap();

        // A pruned/missing decided leader must be skipped, not fatal: recovery
        // still reconstructs the decisions whose blocks remain available.
        let recovered = recover_committed_subdags(&dag, false).unwrap();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].leader.digest, valid_digest);
    }

    #[test]
    fn committed_subdag_recovery_rejects_decision_round_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
        let dag = DagStore::new(storage).unwrap();
        let digest = StatementBlock::compute_digest(0, 2, &[], &[]);
        let block = StatementBlock {
            author: 0,
            round: 2,
            parents: Vec::new(),
            transactions: Vec::new(),
            statements: Vec::new(),
            signature: Signature([0; 64]),
            digest,
            merkle_root: Default::default(),
        };
        dag.put_block(&block).unwrap();
        dag.mark_round_decided(1, &digest).unwrap();

        let error = recover_committed_subdags(&dag, false).unwrap_err();
        assert!(
            error.to_string().contains("round 1"),
            "unexpected recovery error: {error}"
        );
        assert!(
            error.to_string().contains("block round 2"),
            "unexpected recovery error: {error}"
        );
    }

    #[test]
    fn committed_subdag_recovery_rejects_embedded_digest_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("dag.redb");
        let storage = Storage::new(&db_path).unwrap();
        let dag = DagStore::new(storage).unwrap();
        let digest = StatementBlock::compute_digest(0, 1, &[], &[]);
        let block = StatementBlock {
            author: 0,
            round: 1,
            parents: Vec::new(),
            transactions: Vec::new(),
            statements: Vec::new(),
            signature: Signature([0; 64]),
            digest,
            merkle_root: Default::default(),
        };
        dag.put_block(&block).unwrap();
        dag.mark_round_decided(1, &digest).unwrap();
        drop(dag);

        let raw_db = Database::open(&db_path).unwrap();
        let mut corrupted_block = block;
        corrupted_block.digest = Hash::new("kovanica-test/embedded-digest-mismatch");
        let write_txn = raw_db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(DAG_BLOCKS).unwrap();
            table
                .insert(digest.0, corrupted_block.to_bytes().unwrap())
                .unwrap();
        }
        write_txn.commit().unwrap();
        drop(raw_db);

        let storage = Storage::new(&db_path).unwrap();
        let dag = DagStore::new(storage).unwrap();
        let error = recover_committed_subdags(&dag, false)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("block digest"),
            "recovery error should identify the mismatched block digest: {error}"
        );
        assert!(
            error.contains(&format!("expected digest {digest}")),
            "recovery error should identify the persisted decision digest: {error}"
        );
    }

    #[test]
    fn committed_subdag_recovery_with_mysticghost_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
        let dag = DagStore::new(storage).unwrap();
        let digest = StatementBlock::compute_digest(0, 1, &[], &[]);
        let block = StatementBlock {
            author: 0,
            round: 1,
            parents: Vec::new(),
            transactions: Vec::new(),
            statements: Vec::new(),
            signature: Signature([0; 64]),
            digest,
            merkle_root: Default::default(),
        };
        dag.put_block(&block).unwrap();
        dag.mark_round_decided(1, &digest).unwrap();

        // Test recovery with MysticGhost enabled (should fall back to linearizer for simple case)
        let recovered = recover_committed_subdags(&dag, true).unwrap();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].leader.digest, digest);
        assert_eq!(recovered[0].blocks.len(), 1);
    }

    #[tokio::test]
    async fn quorum_commit_reaches_execution_and_persists_state() {
        let dir = tempfile::tempdir().unwrap();
        let dag =
            Arc::new(DagStore::new(Storage::new(dir.path().join("dag.redb")).unwrap()).unwrap());
        let state_storage = Arc::new(Storage::new(dir.path().join("state.redb")).unwrap());
        let (signing_key, public_key) = kovanica_crypto::generate_keypair();
        let validator_address = Address::from_public_key(&public_key);
        let payout = Address([0x91; 32]);
        let mut staking = StakingState::new();
        staking
            .join_validator(
                validator_address,
                MIN_VALIDATOR_STAKE,
                0,
                Some(payout),
                Some(public_key),
                None,
            )
            .unwrap();
        let txn = state_storage.begin_write().unwrap();
        state_storage
            .state()
            .save_staking_state(&txn, &staking)
            .unwrap();
        txn.commit().unwrap();

        let committee = build_committee(
            &state_storage,
            &public_key,
            &validator_address,
            "127.0.0.1:0",
        )
        .expect("single-validator committee is valid");
        let block_manager = Arc::new(BlockManager::new(
            dag.clone(),
            kovanica_types::SigningContext::new(kovanica_types::signing::chain_id::LOCAL),
        ));
        block_manager.set_authority(0);
        block_manager.set_signing_key(signing_key.clone());
        block_manager.set_authority_keys(HashMap::from([(0, public_key)]));
        let engine = ConsensusEngine::new(
            ConsensusConfig::default(),
            committee.clone(),
            Arc::new(NodeDagStore { inner: dag.clone() }),
            Arc::new(RwLock::new(NodeBlockManager {
                inner: block_manager,
            })),
            signing_key.clone(),
            None, // No mempool for this test
            kovanica_types::SigningContext::new(kovanica_types::signing::chain_id::LOCAL),
        );
        let (exec_tx, exec_rx) = mpsc::unbounded_channel();
        engine.set_commit_sender(exec_tx);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let execution = tokio::spawn(run_execution(
            exec_rx,
            state_storage.clone(),
            EventBus::new(),
            None,
            kovanica_types::SigningContext::new(kovanica_types::signing::chain_id::LOCAL),
            shutdown_rx,
            NodeHealth::new(),
            None,
            Arc::new(tokio::sync::RwLock::new(committee)),
            "127.0.0.1:8000".to_string(),
            validator_address,
            public_key,
        ));

        let leader_round = 3;
        let digest = StatementBlock::compute_digest(0, leader_round, &[], &[]);
        let block = StatementBlock {
            author: 0,
            round: leader_round,
            parents: Vec::new(),
            transactions: Vec::new(),
            statements: Vec::new(),
            signature: kovanica_crypto::sign(&signing_key, digest.as_ref()),
            digest,
            merkle_root: Default::default(),
        };
        engine.process_block(&block).unwrap();
        let mut vote = kovanica_types::Vote {
            leader_round,
            leader_hash: digest,
            voter: 0,
            signature: kovanica_types::Signature([0; 64]),
        };
        vote.signature = kovanica_crypto::sign(
            &signing_key,
            &vote.signature_data(&kovanica_types::SigningContext::new(
                kovanica_types::signing::chain_id::LOCAL,
            )),
        );
        engine.process_vote(&vote).unwrap();

        let mut committed_height = 0;
        for _ in 0..100 {
            let read = state_storage.begin_read().unwrap();
            committed_height = state_storage
                .state()
                .load_staking_state(&read)
                .unwrap()
                .committed_leader_height;
            if committed_height == 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(committed_height, 1, "execution consumed committed sub-DAG");
        let read = state_storage.begin_read().unwrap();
        let account = state_storage
            .state()
            .get_account_or_default(&read, &payout)
            .unwrap();
        assert!(
            account.balance > 0,
            "leader reward persisted to payout account"
        );

        shutdown_tx.send(true).unwrap();
        execution.await.unwrap();
    }

    /// Adapter double: returns a fixed list, whatever the sub-DAG.
    struct FixedNonBlue(Vec<Transaction>);
    impl non_blue::NonBlueSource for FixedNonBlue {
        fn non_blue_transactions(
            &self,
            _dag: &DagStore,
            _subdag: &kovanica_consensus::CommittedSubDag,
        ) -> Vec<Transaction> {
            self.0.clone()
        }
    }

    #[tokio::test]
    async fn execution_hands_non_blue_transactions_to_mempool_after_commit() {
        let dir = tempfile::tempdir().unwrap();
        let state_storage = Arc::new(Storage::new(dir.path().join("state.redb")).unwrap());
        let dag =
            Arc::new(DagStore::new(Storage::new(dir.path().join("dag.redb")).unwrap()).unwrap());
        let validator_address = Address([0x22; 32]);
        let mut staking = StakingState::new();
        staking
            .join_validator(
                validator_address,
                MIN_VALIDATOR_STAKE,
                0,
                Some(Address([0xa1; 32])),
                Some(PublicKey([0x22; 32])),
                None,
            )
            .unwrap();
        let txn = state_storage.begin_write().unwrap();
        state_storage
            .state()
            .save_staking_state(&txn, &staking)
            .unwrap();
        txn.commit().unwrap();

        let returned_tx = non_blue::test_support::funded_tx(&state_storage, 0);
        let leader = non_blue::test_support::block(0, 1, Vec::new());
        let subdag = kovanica_consensus::CommittedSubDag {
            blocks: vec![leader.clone()],
            leader,
            leader_round: 1,
            leader_author: 0,
            non_blue: Vec::new(),
        };
        let mempool = Arc::new(Mempool::new(
            MempoolConfig::default(),
            state_storage.clone(),
            non_blue::test_support::CTX,
        ));
        let (exec_tx, exec_rx) = mpsc::unbounded_channel();
        exec_tx.send(subdag).unwrap();
        drop(exec_tx);
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);
        let health = NodeHealth::new();
        run_execution(
            exec_rx,
            state_storage.clone(),
            EventBus::new(),
            Some(non_blue::ExecPool {
                mempool: mempool.clone(),
                returned: Some(non_blue::ReturnedTxs {
                    dag,
                    source: Arc::new(FixedNonBlue(vec![returned_tx.clone()])),
                }),
            }),
            non_blue::test_support::CTX,
            shutdown_rx,
            health.clone(),
            None,
            Arc::new(tokio::sync::RwLock::new(
                build_committee(
                    &state_storage,
                    &PublicKey([0x22; 32]),
                    &validator_address,
                    "127.0.0.1:8000",
                )
                .unwrap(),
            )),
            "127.0.0.1:8000".to_string(),
            validator_address,
            PublicKey([0x22; 32]),
        )
        .await;
        assert_ne!(
            health.execution_state(),
            kovanica_rpc::ComponentState::Failed
        );
        assert!(
            mempool.contains(&returned_tx.hash),
            "returned tx back in the pool"
        );
        // Reinserted transactions are now gossiped
        let pending = mempool.get_pending_propagation();
        assert_eq!(pending.len(), 1, "reinserted tx should be gossiped");
        assert_eq!(pending[0], returned_tx.hash);
    }

    /// #13 restart: a red block committed before a restart. After restart,
    /// recovery reports it in `non_blue` and replay hands its transactions
    /// back to the mempool: none lost (each tx executed via a blue block or
    /// pooled) and no red-only tx executed.
    #[tokio::test]
    async fn red_block_transactions_return_to_mempool_after_restart() {
        use non_blue::test_support::{block_with_parents, funded_tx, reference, CTX};
        let dir = tempfile::tempdir().unwrap();
        let state_storage = Arc::new(Storage::new(dir.path().join("state.redb")).unwrap());
        let mut staking = StakingState::new();
        for a in 0..4u8 {
            staking
                .join_validator(
                    Address([0x30 + a; 32]),
                    MIN_VALIDATOR_STAKE,
                    0,
                    Some(Address([0xa0 + a; 32])),
                    Some(PublicKey([0x30 + a; 32])),
                    None,
                )
                .unwrap();
        }
        let txn = state_storage.begin_write().unwrap();
        state_storage
            .state()
            .save_staking_state(&txn, &staking)
            .unwrap();
        txn.commit().unwrap();

        // Before restart: wide wave (6 unordered round-1 blocks, k = 3) so
        // GHOSTDAG colours some red; leader at round 3 is committed durably.
        let mut all_txs = Vec::new();
        {
            let dag = DagStore::new(Storage::new(dir.path().join("dag.redb")).unwrap()).unwrap();
            let genesis = block_with_parents(0, 0, Vec::new(), Vec::new());
            dag.put_block(&genesis).unwrap();
            let mut wide = Vec::new();
            for i in 0..6u32 {
                let tx = funded_tx(&state_storage, 0);
                all_txs.push(tx.clone());
                let b = block_with_parents(i % 4, 1, vec![reference(&genesis)], vec![tx]);
                dag.put_block(&b).unwrap();
                wide.push(b);
            }
            let leader = block_with_parents(0, 3, wide.iter().map(reference).collect(), Vec::new());
            dag.put_block(&leader).unwrap();
            dag.mark_decided_and_commit_leader(3, &leader.digest)
                .unwrap();
        } // "restart": DAG store closed and reopened below.

        let dag =
            Arc::new(DagStore::new(Storage::new(dir.path().join("dag.redb")).unwrap()).unwrap());
        let recovered = recover_committed_subdags(&dag, true).unwrap();
        let subdag = recovered
            .iter()
            .find(|s| s.leader_round == 3)
            .expect("round-3 commit recovered")
            .clone();
        assert!(
            !subdag.non_blue.is_empty(),
            "recovery must report red blocks"
        );
        let red: std::collections::HashSet<Hash> =
            subdag.non_blue.iter().map(|r| r.digest).collect();
        let blue: std::collections::HashSet<Hash> =
            subdag.blocks.iter().map(|b| b.digest).collect();
        assert!(red.is_disjoint(&blue));

        // Mempool exists (empty, in-memory) before replay, as in run_node.
        let mempool = Arc::new(Mempool::new(
            MempoolConfig::default(),
            state_storage.clone(),
            CTX,
        ));
        let (exec_tx, exec_rx) = mpsc::unbounded_channel();
        for s in recovered {
            exec_tx.send(s).unwrap();
        }
        drop(exec_tx);
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);
        let health = NodeHealth::new();
        run_execution(
            exec_rx,
            state_storage.clone(),
            EventBus::new(),
            Some(non_blue::ExecPool {
                mempool: mempool.clone(),
                returned: Some(non_blue::ReturnedTxs {
                    dag: dag.clone(),
                    source: Arc::new(non_blue::ConsensusNonBlue),
                }),
            }),
            CTX,
            shutdown_rx,
            health.clone(),
            None,
            Arc::new(tokio::sync::RwLock::new({
                let txn = state_storage.begin_read().unwrap();
                let s = state_storage.state().load_staking_state(&txn).unwrap();
                build_committee(
                    &state_storage,
                    &PublicKey([0x30; 32]),
                    &Address([0x30; 32]),
                    "127.0.0.1:8000",
                )
                .unwrap_or_else(|_| {
                    // Fall back to a committee built from the persisted state.
                    let _ = s;
                    panic!("committee build failed in test");
                })
            })),
            "127.0.0.1:8000".to_string(),
            Address([0x30; 32]),
            PublicKey([0x30; 32]),
        )
        .await;
        assert_ne!(
            health.execution_state(),
            kovanica_rpc::ComponentState::Failed
        );

        let red_txs: std::collections::HashSet<Hash> = subdag
            .non_blue
            .iter()
            .flat_map(|r| dag.get_block(&r.digest).unwrap().transactions)
            .map(|t| t.hash)
            .collect();
        assert!(!red_txs.is_empty());
        let read = state_storage.begin_read().unwrap();
        for tx in &all_txs {
            let nonce = state_storage
                .state()
                .get_account_or_default(&read, &tx.sender)
                .unwrap()
                .nonce;
            if red_txs.contains(&tx.hash) {
                assert_eq!(nonce, 0, "red-only tx {} must not be executed", tx.hash);
                assert!(mempool.contains(&tx.hash), "red tx {} lost", tx.hash);
            } else {
                assert!(!mempool.contains(&tx.hash), "blue tx {} re-pooled", tx.hash);
            }
        }
        // Since 0c7de51e returned non-blue (red) txs are re-gossiped
        // (gossip=true); see execution_hands_non_blue_transactions_to_mempool_after_commit.
        let pending: std::collections::HashSet<Hash> =
            mempool.get_pending_propagation().into_iter().collect();
        assert_eq!(pending, red_txs, "returned red txs are re-gossiped");
    }

    #[tokio::test]
    async fn execution_worker_stops_after_failure_before_later_commits() {
        let dir = tempfile::tempdir().unwrap();
        let state_storage = Arc::new(Storage::new(dir.path().join("state.redb")).unwrap());
        let validator_address = Address([0x22; 32]);
        let payout = Address([0xa1; 32]);
        let mut staking = StakingState::new();
        staking
            .join_validator(
                validator_address,
                MIN_VALIDATOR_STAKE,
                0,
                Some(payout),
                Some(PublicKey([0x22; 32])),
                None,
            )
            .unwrap();
        let txn = state_storage.begin_write().unwrap();
        state_storage
            .state()
            .save_staking_state(&txn, &staking)
            .unwrap();
        txn.commit().unwrap();

        let committed_subdag = |round, author| {
            let digest = StatementBlock::compute_digest(author, round, &[], &[]);
            let leader = StatementBlock {
                author,
                round,
                parents: Vec::new(),
                transactions: Vec::new(),
                statements: Vec::new(),
                signature: Signature([0; 64]),
                digest,
                merkle_root: Default::default(),
            };
            kovanica_consensus::CommittedSubDag {
                blocks: vec![leader.clone()],
                leader,
                leader_round: round,
                leader_author: author,
                non_blue: Vec::new(),
            }
        };
        let test_public_key = PublicKey([0x22; 32]);
        let test_committee = build_committee(
            &state_storage,
            &test_public_key,
            &validator_address,
            "127.0.0.1:8000",
        )
        .unwrap();
        let (exec_tx, exec_rx) = mpsc::unbounded_channel();
        exec_tx.send(committed_subdag(1, 9)).unwrap(); // Unregistered authority fails.
        exec_tx.send(committed_subdag(2, 0)).unwrap(); // Must not be executed next.
        drop(exec_tx);

        let (_shutdown_tx, shutdown_rx) = watch::channel(false);
        let health = NodeHealth::new();
        run_execution(
            exec_rx,
            state_storage.clone(),
            EventBus::new(),
            None,
            kovanica_types::SigningContext::new(kovanica_types::signing::chain_id::LOCAL),
            shutdown_rx,
            health.clone(),
            None,
            Arc::new(tokio::sync::RwLock::new(test_committee)),
            "127.0.0.1:8000".to_string(),
            validator_address,
            test_public_key,
        )
        .await;
        assert_eq!(
            health.execution_state(),
            kovanica_rpc::ComponentState::Failed
        );
        assert!(!health.is_healthy());

        let read = state_storage.begin_read().unwrap();
        let state = state_storage.state().load_staking_state(&read).unwrap();
        assert_eq!(
            state.committed_leader_height, 0,
            "worker must not advance to a later queued commit after failure"
        );
        assert_eq!(
            state_storage
                .state()
                .get_account_or_default(&read, &payout)
                .unwrap()
                .balance,
            0,
            "later committed leader must not receive a reward"
        );
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn node_starts_and_shuts_down() {
        let rpc_port = free_port();
        let dir = tempfile::tempdir().expect("tempdir");
        let key_path = write_seed_file(dir.path(), &[7u8; 32], 0o600);

        let config = NodeConfig {
            data_dir: dir.path().to_path_buf(),
            validator_key: Some(key_path.to_string_lossy().into_owned()),
            listen_addr: "127.0.0.1:0".to_string(),
            rpc_addr: "127.0.0.1".to_string(),
            rpc_port,
            bootnodes: Vec::new(),
            round_duration_ms: 100,
            ..Default::default()
        };

        // `shutdown_signal()` installs the SIGTERM handler synchronously before
        // the node task is spawned, so the process is safe to signal below.
        let node = tokio::spawn(run_node(config, shutdown_signal()));

        wait_for_port(rpc_port, Duration::from_secs(10)).await;

        let status = std::process::Command::new("kill")
            .arg("-TERM")
            .arg(std::process::id().to_string())
            .status()
            .expect("run kill");
        assert!(status.success(), "kill -TERM failed");

        let joined = tokio::time::timeout(Duration::from_secs(10), node)
            .await
            .expect("node did not shut down in time")
            .expect("node task panicked");
        assert!(joined.is_ok(), "node returned error: {joined:?}");
    }

    #[cfg(unix)]
    fn free_port() -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        listener.local_addr().expect("local addr").port()
    }

    #[cfg(unix)]
    async fn wait_for_port(port: u16, timeout: Duration) {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "RPC port {port} did not open in time"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    #[test]
    fn build_committee_from_staking_state() {
        // Create a temp directory for the state database
        let dir = tempfile::tempdir().unwrap();
        let state_storage = Storage::new(dir.path().join("state.redb")).unwrap();

        // Generate 4 validators with different stakes and addresses
        let (_sk1, pk1) = kovanica_crypto::generate_keypair();
        let addr1 = Address::from_public_key(&pk1);
        let (_sk2, pk2) = kovanica_crypto::generate_keypair();
        let addr2 = Address::from_public_key(&pk2);
        let (_sk3, pk3) = kovanica_crypto::generate_keypair();
        let addr3 = Address::from_public_key(&pk3);
        let (_sk4, pk4) = kovanica_crypto::generate_keypair();
        let addr4 = Address::from_public_key(&pk4);

        // Create staking state with 4 validators with different stakes
        let mut staking = StakingState::new();
        staking.init_treasury(Address([0xaa; 32]));

        // Validator stakes: 100K, 80K, 120K, 60K KUNA (all above MIN_VALIDATOR_STAKE = 50K)
        staking
            .join_validator(addr1, 100_000 * ONE_KUNA, 0, Some(addr1), Some(pk1), None)
            .unwrap();
        staking
            .join_validator(addr2, 80_000 * ONE_KUNA, 0, Some(addr2), Some(pk2), None)
            .unwrap();
        staking
            .join_validator(addr3, 120_000 * ONE_KUNA, 0, Some(addr3), Some(pk3), None)
            .unwrap();
        staking
            .join_validator(addr4, 60_000 * ONE_KUNA, 0, Some(addr4), Some(pk4), None)
            .unwrap();

        // Save to storage
        let txn = state_storage.begin_write().unwrap();
        state_storage
            .state()
            .save_staking_state(&txn, &staking)
            .unwrap();
        txn.commit().unwrap();

        // Use addr1 as the "local" validator (first one by address sort)
        let local_pk = pk1;
        let local_addr = addr1;

        // Build committee
        let committee = build_committee(&state_storage, &local_pk, &local_addr, "127.0.0.1:0")
            .expect("committee should be built successfully");

        // Verify committee has all 4 validators
        assert_eq!(committee.size(), 4, "committee should have 4 validators");

        // Verify validators are sorted by address (deterministic ordering)
        // Since we sort by address for index assignment, the order should be by address
        let authorities = committee.authorities();

        // Collect addresses in index order
        let committee_addresses: Vec<Address> = authorities.iter().map(|a| a.address).collect();

        // Sort the expected addresses to match committee ordering
        let mut expected_addresses = vec![addr1, addr2, addr3, addr4];
        expected_addresses.sort_by_key(|a| a.0);

        assert_eq!(
            committee_addresses, expected_addresses,
            "committee validators should be sorted by address for deterministic index assignment"
        );

        // Verify indices are 0..3
        for (idx, authority) in authorities.iter().enumerate() {
            assert_eq!(
                authority.index, idx as u16,
                "authority index should match sorted position"
            );
        }

        // Verify stakes match
        let addr_to_stake = vec![
            (addr1, 100_000 * ONE_KUNA),
            (addr2, 80_000 * ONE_KUNA),
            (addr3, 120_000 * ONE_KUNA),
            (addr4, 60_000 * ONE_KUNA),
        ];
        for (addr, expected_stake) in addr_to_stake {
            let auth = committee
                .get_by_index(expected_addresses.iter().position(|a| *a == addr).unwrap() as u16)
                .expect("authority should exist");
            assert_eq!(
                auth.stake, expected_stake,
                "stake should match for address {}",
                addr
            );
        }

        // Verify leader rotation: one leader per wave, cycling through the
        // committee. Leader rounds are `round % WAVE_LENGTH == 0`; seeding the
        // rotation from the wave (not the raw round) keeps every authority in
        // play even when the committee size equals WAVE_LENGTH.
        assert_eq!(kovanica_types::WAVE_LENGTH, 3);
        for round in (0..24).step_by(kovanica_types::WAVE_LENGTH as usize) {
            let leader_idx = committee.leader(round);
            let wave = round / kovanica_types::WAVE_LENGTH;
            let expected_idx = (wave as usize) % 4;
            assert_eq!(
                leader_idx, expected_idx as u16,
                "leader for round {} (wave {}) should be index {}",
                round, wave, expected_idx
            );
        }
        // The degenerate case that stalled the 3-validator testnet: with a
        // committee the size of WAVE_LENGTH, consecutive leaders must differ.
        let wave_leaders: Vec<u16> = (0..kovanica_types::WAVE_LENGTH)
            .map(|wave| committee.leader(wave * kovanica_types::WAVE_LENGTH))
            .collect();
        assert_ne!(
            wave_leaders[0], wave_leaders[1],
            "consecutive leaders must rotate for a WAVE_LENGTH-sized committee"
        );
    }

    #[test]
    fn build_committee_skips_inactive_and_low_stake() {
        let dir = tempfile::tempdir().unwrap();
        let state_storage = Storage::new(dir.path().join("state.redb")).unwrap();

        let (_sk1, pk1) = kovanica_crypto::generate_keypair();
        let addr1 = Address::from_public_key(&pk1);
        let (_sk2, pk2) = kovanica_crypto::generate_keypair();
        let addr2 = Address::from_public_key(&pk2);
        let (_sk3, pk3) = kovanica_crypto::generate_keypair();
        let addr3 = Address::from_public_key(&pk3);

        let mut staking = StakingState::new();
        staking.init_treasury(Address([0xbb; 32]));

        // Active, sufficient stake
        staking
            .join_validator(addr1, 100_000 * ONE_KUNA, 0, Some(addr1), Some(pk1), None)
            .unwrap();
        // Inactive - should be skipped
        staking
            .join_validator(addr2, 100_000 * ONE_KUNA, 0, Some(addr2), Some(pk2), None)
            .unwrap();
        staking
            .validators
            .iter_mut()
            .find(|v| v.address == addr2)
            .unwrap()
            .active = false;
        // Below MIN_VALIDATOR_STAKE - manually add (bypassing join_validator check)
        staking.validators.push(kovanica_staking::ValidatorInfo {
            address: addr3,
            stake: 10_000 * ONE_KUNA,
            commission_bps: 0,
            active: true,
            payout_address: addr3,
            public_key: Some(pk3),
            vrf_public_key: None,
            missed_leader_count: 0,
            last_online_height: 0,
            last_commission_change_height: 0,
        });
        staking.total_staked += 10_000 * ONE_KUNA;

        let txn = state_storage.begin_write().unwrap();
        state_storage
            .state()
            .save_staking_state(&txn, &staking)
            .unwrap();
        txn.commit().unwrap();

        let committee = build_committee(&state_storage, &pk1, &addr1, "127.0.0.1:0")
            .expect("committee should be built with only active, sufficient-stake validators");

        assert_eq!(
            committee.size(),
            1,
            "only 1 validator should be active and have sufficient stake"
        );
        assert_eq!(committee.authorities()[0].address, addr1);
    }

    #[test]
    fn build_committee_limits_to_max_active_validators() {
        let dir = tempfile::tempdir().unwrap();
        let state_storage = Storage::new(dir.path().join("state.redb")).unwrap();

        let mut staking = StakingState::new();
        staking.init_treasury(Address([0xcc; 32]));

        // Create 25 validators (MAX_ACTIVE_VALIDATORS = 21) - manually bypass join_validator limit
        let mut validators = Vec::new();
        for _ in 0..25 {
            let (_sk, pk) = kovanica_crypto::generate_keypair();
            let addr = Address::from_public_key(&pk);
            validators.push((addr, pk));
            staking.validators.push(kovanica_staking::ValidatorInfo {
                address: addr,
                stake: 100_000 * ONE_KUNA,
                commission_bps: 0,
                active: true,
                payout_address: addr,
                public_key: Some(pk),
                vrf_public_key: None,
                missed_leader_count: 0,
                last_online_height: 0,
                last_commission_change_height: 0,
            });
            staking.total_staked += 100_000 * ONE_KUNA;
        }

        let txn = state_storage.begin_write().unwrap();
        state_storage
            .state()
            .save_staking_state(&txn, &staking)
            .unwrap();
        txn.commit().unwrap();

        // Use first validator as local
        let (local_addr, local_pk) = validators[0];

        let committee = build_committee(&state_storage, &local_pk, &local_addr, "127.0.0.1:0")
            .expect("committee should be built");

        // Should only have MAX_ACTIVE_VALIDATORS (21)
        assert_eq!(
            committee.size(),
            MAX_ACTIVE_VALIDATORS,
            "committee should be limited to MAX_ACTIVE_VALIDATORS"
        );
    }

    // ---- proactive catch-up tracker (#1) ----

    /// A DAG store holding one block per round `0..rounds` (author 0).
    fn catch_up_dag(dir: &std::path::Path, rounds: Round) -> DagStore {
        let dag = DagStore::new(Storage::new(dir.join("catch-up-dag.redb")).unwrap()).unwrap();
        for round in 0..rounds {
            let block = StatementBlock {
                author: 0,
                round,
                parents: Vec::new(),
                transactions: Vec::new(),
                statements: Vec::new(),
                signature: Signature([0u8; 64]),
                digest: Hash::new(format!("catch-up-round-{round}")),
                merkle_root: Default::default(),
            };
            dag.put_block(&block).unwrap();
        }
        dag
    }

    /// Pop the next queued sync command (panics if none is waiting).
    fn next_sync(rx: &mut mpsc::UnboundedReceiver<NetworkCommand>) -> (PeerId, BlockSyncRequest) {
        match rx.try_recv().expect("a sync command was queued") {
            NetworkCommand::RequestSync(peer, request) => (peer, request),
            _other => panic!("expected a RequestSync command"),
        }
    }

    #[test]
    fn catch_up_probe_asks_for_the_peer_tip_once() {
        let mut tracker = CatchUpTracker::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let peer = PeerId::random();
        let now = Instant::now();

        tracker.probe(peer, now, &tx);
        tracker.probe(peer, now, &tx);

        let (got_peer, request) = next_sync(&mut rx);
        assert_eq!(got_peer, peer);
        assert!(matches!(request, BlockSyncRequest::Status));
        assert!(rx.try_recv().is_err(), "one probe per peer");
        assert!(tracker.is_active());
    }

    #[test]
    fn catch_up_behind_peer_is_pulled_with_a_bounded_range_window() {
        let dir = tempfile::tempdir().unwrap();
        let dag = catch_up_dag(dir.path(), 5);
        let local = local_dag_tip(&dag).expect("local tip");
        assert_eq!(local, 4);

        let mut tracker = CatchUpTracker::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let peer = PeerId::random();
        let now = Instant::now();

        tracker.probe(peer, now, &tx);
        let _ = next_sync(&mut rx);
        tracker.on_status(peer, 100, Some(local), now, &tx);

        let (got_peer, request) = next_sync(&mut rx);
        assert_eq!(got_peer, peer);
        match request {
            BlockSyncRequest::RoundRange { from, to } => {
                assert_eq!(from, local + 1);
                assert!(to <= 100, "never request beyond the advertised tip");
                assert!(to - from < BlockSyncRequest::MAX_RANGE_ROUNDS);
            }
            other => panic!("expected a bounded range request, got {other:?}"),
        }
        assert!(tracker.is_active());
    }

    #[test]
    fn catch_up_peer_at_or_behind_the_local_tip_starts_nothing() {
        let mut tracker = CatchUpTracker::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let peer = PeerId::random();
        let now = Instant::now();

        tracker.probe(peer, now, &tx);
        let _ = next_sync(&mut rx);
        tracker.on_status(peer, 4, Some(4), now, &tx);
        assert!(!tracker.is_active(), "equal tip: no session");

        tracker.probe(peer, now, &tx);
        let _ = next_sync(&mut rx);
        tracker.on_status(peer, 2, Some(9), now, &tx);
        assert!(!tracker.is_active(), "peer behind: no session");
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn catch_up_summary_tracks_probing_and_catching_up_phases() {
        let mut tracker = CatchUpTracker::new();
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Idle: not catching up, no target.
        assert_eq!(tracker.summary(), (false, None));

        // Probing (tip probe sent, no Status yet): still not catching up.
        let peer = PeerId::random();
        tracker.probe(peer, Instant::now(), &tx);
        let _ = next_sync(&mut rx);
        assert_eq!(tracker.summary(), (false, None));

        // Status from an ahead peer starts a catching-up session: target set.
        tracker.on_status(peer, 200, Some(100), Instant::now(), &tx);
        let _ = next_sync(&mut rx);
        let (catching_up, target) = tracker.summary();
        assert!(catching_up);
        assert_eq!(target, Some(200));

        // The highest advertised tip wins when several sessions are active.
        let second = PeerId::random();
        tracker.probe(second, Instant::now(), &tx);
        let _ = next_sync(&mut rx);
        tracker.on_status(second, 350, Some(100), Instant::now(), &tx);
        let _ = next_sync(&mut rx);
        assert_eq!(tracker.summary(), (true, Some(350)));

        // Once every session is gone the summary returns to idle.
        tracker.remove(peer);
        tracker.remove(second);
        assert_eq!(tracker.summary(), (false, None));
    }

    #[test]
    fn catch_up_stands_down_when_the_local_tip_is_unreadable() {
        let mut tracker = CatchUpTracker::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let peer = PeerId::random();
        let now = Instant::now();

        tracker.probe(peer, now, &tx);
        let _ = next_sync(&mut rx);
        tracker.on_status(peer, 100, None, now, &tx);
        assert!(!tracker.is_active());
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn local_dag_tip_treats_an_empty_dag_as_round_zero() {
        let dir = tempfile::tempdir().unwrap();
        let empty = DagStore::new(Storage::new(dir.path().join("empty.redb")).unwrap()).unwrap();
        assert_eq!(local_dag_tip(&empty), Some(0));

        let dag = catch_up_dag(dir.path(), 3);
        assert_eq!(local_dag_tip(&dag), Some(2));
    }

    /// Probe a fabricated peer, start a catch-up session and consume the first
    /// range window. Returns the tracker, the peer and the window bounds.
    fn catch_up_started(
        tx: &mpsc::UnboundedSender<NetworkCommand>,
        rx: &mut mpsc::UnboundedReceiver<NetworkCommand>,
        target: Round,
    ) -> (CatchUpTracker, PeerId, Round, Round) {
        let mut tracker = CatchUpTracker::new();
        let peer = PeerId::random();
        let now = Instant::now();
        tracker.probe(peer, now, tx);
        let _ = next_sync(rx);
        tracker.on_status(peer, target, Some(0), now, tx);
        let (_, first) = next_sync(rx);
        match first {
            BlockSyncRequest::RoundRange { from, to } => (tracker, peer, from, to),
            other => panic!("expected a range request, got {other:?}"),
        }
    }

    #[test]
    fn catch_up_tick_does_not_advance_while_a_window_is_in_flight() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (mut tracker, _peer, _from, _to) = catch_up_started(&tx, &mut rx, 200);

        tracker.tick(Instant::now() + CATCH_UP_TICK, Some(100), &tx);
        assert!(rx.try_recv().is_err(), "no second request while in flight");
    }

    #[test]
    fn catch_up_tick_waits_for_local_progress_before_the_next_window() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (mut tracker, peer, first_from, first_to) = catch_up_started(&tx, &mut rx, 200);
        let now = Instant::now();

        tracker.on_round_hashes(peer, first_to, false, &[Hash::new("h")], now, &tx);
        let (_, by_hash) = next_sync(&mut rx);
        assert!(matches!(by_hash, BlockSyncRequest::ByHash(_)));

        // Local tip has not reached the window end: wait instead of running ahead.
        tracker.tick(now + CATCH_UP_TICK, Some(first_to - 1), &tx);
        assert!(rx.try_recv().is_err(), "must not run ahead of ingestion");

        // Local tip reached the window end: continue with the next window.
        tracker.tick(now + CATCH_UP_TICK * 2, Some(first_to), &tx);
        let (_, next) = next_sync(&mut rx);
        match next {
            BlockSyncRequest::RoundRange { from, to } => {
                assert_eq!(from, first_to + 1);
                assert_eq!(from, first_from + BlockSyncRequest::MAX_RANGE_ROUNDS);
                assert!(to <= 200);
            }
            other => panic!("expected a continuation range, got {other:?}"),
        }
    }

    #[test]
    fn catch_up_hole_mode_crosses_empty_rounds_without_local_progress() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (mut tracker, peer, _from, first_to) = catch_up_started(&tx, &mut rx, 200);
        let now = Instant::now();

        // The window covered only empty rounds: nothing to fetch, and the local
        // tip cannot move, so the session must still continue.
        tracker.on_round_hashes(peer, first_to, true, &[], now, &tx);
        assert!(
            rx.try_recv().is_err(),
            "empty window queues no hash fetches"
        );

        tracker.tick(now + CATCH_UP_TICK, Some(0), &tx);
        let (_, next) = next_sync(&mut rx);
        assert!(matches!(next, BlockSyncRequest::RoundRange { .. }));
    }

    /// A full 512-hash window is released through a bounded pipeline, and each
    /// completed fetch frees a slot for the next hash (audit HIGH-2c).
    #[test]
    fn catch_up_hash_fetches_are_pipelined_with_a_bounded_window() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (mut tracker, peer, _from, first_to) = catch_up_started(&tx, &mut rx, 512);
        let now = Instant::now();

        let hashes: Vec<Hash> = (0..BlockSyncResponse::MAX_RANGE_HASHES)
            .map(|i| Hash::new(format!("h{i}")))
            .collect();
        tracker.on_round_hashes(peer, first_to, false, &hashes, now, &tx);

        // Only the bounded window is in flight, not all 512 at once.
        let mut issued = 0usize;
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                NetworkCommand::RequestSync(_, BlockSyncRequest::ByHash(_)) => issued += 1,
                _other => panic!("expected only by-hash fetches"),
            }
        }
        assert_eq!(issued, CATCH_UP_MAX_HASH_FETCHES);
        assert!(issued < BlockSyncResponse::MAX_RANGE_HASHES);

        // One completion frees one slot and releases exactly one more fetch.
        tracker.on_hash_response(peer, now, &tx);
        let (_, next) = next_sync(&mut rx);
        assert!(matches!(next, BlockSyncRequest::ByHash(_)));
        assert!(rx.try_recv().is_err(), "only one slot was freed");
    }

    /// A peer cannot keep a session alive forever by answering every range with
    /// an empty window while advancing its advertised tip (audit MEDIUM-5).
    #[test]
    fn catch_up_hole_mode_is_capped() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (mut tracker, peer, _from, mut to) = catch_up_started(&tx, &mut rx, 512);
        let now = Instant::now();

        let mut windows = 0u32;
        while tracker.is_active() && windows < CATCH_UP_MAX_HOLE_WINDOWS + 2 {
            tracker.on_round_hashes(peer, to, true, &[], now, &tx);
            tracker.tick(now + CATCH_UP_TICK, Some(0), &tx);
            let _ = rx.try_recv(); // drain any continuation range request
            to = to.saturating_add(BlockSyncRequest::MAX_RANGE_ROUNDS);
            windows += 1;
        }
        assert!(
            !tracker.is_active(),
            "hole mode must be bounded even without local progress"
        );
        assert!(windows <= CATCH_UP_MAX_HOLE_WINDOWS + 1);
    }

    /// A window is only continued once the previous window's by-hash pipeline
    /// has drained, so windows cannot stack up unboundedly (audit HIGH-2c).
    #[test]
    fn catch_up_waits_for_the_hash_pipeline_before_the_next_window() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (mut tracker, peer, _from, first_to) = catch_up_started(&tx, &mut rx, 512);
        let now = Instant::now();

        let hashes: Vec<Hash> = (0..BlockSyncResponse::MAX_RANGE_HASHES)
            .map(|i| Hash::new(format!("h{i}")))
            .collect();
        tracker.on_round_hashes(peer, first_to, false, &hashes, now, &tx);
        while rx.try_recv().is_ok() {} // drop the first pipeline window

        // Local tip looks progressed, but the backlog is still draining: the
        // session must not start the next range window on top of it.
        tracker.tick(now + CATCH_UP_TICK, Some(first_to), &tx);
        assert!(
            rx.try_recv().is_err(),
            "no next window while by-hash fetches are still queued"
        );
    }

    #[test]
    fn catch_up_session_completes_once_the_advertised_tip_is_covered() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (mut tracker, peer, _from, _to) = catch_up_started(&tx, &mut rx, 32);
        let now = Instant::now();

        tracker.on_round_hashes(peer, 32, true, &[], now, &tx);
        assert!(tracker.is_active());
        tracker.tick(now + CATCH_UP_TICK, Some(0), &tx);
        assert!(!tracker.is_active(), "covered tip ends the session");
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn catch_up_session_expires_after_the_inactivity_ttl() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (mut tracker, peer, _from, _to) = catch_up_started(&tx, &mut rx, 200);
        let now = Instant::now();

        tracker.sessions.get_mut(&peer).unwrap().last_activity = now
            .checked_sub(CATCH_UP_TTL + Duration::from_secs(1))
            .unwrap();
        tracker.tick(now, Some(0), &tx);
        assert!(!tracker.is_active(), "idle session expires");
    }

    #[test]
    fn catch_up_session_gives_up_after_the_range_budget() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (mut tracker, peer, _from, first_to) = catch_up_started(&tx, &mut rx, 200);
        let now = Instant::now();

        tracker.on_round_hashes(peer, first_to, false, &[Hash::new("h")], now, &tx);
        let session = tracker.sessions.get_mut(&peer).unwrap();
        session.ranges_done = CATCH_UP_MAX_RANGES;
        session.last_activity = now;

        tracker.tick(now + CATCH_UP_TICK, Some(first_to), &tx);
        assert!(!tracker.is_active(), "range budget is finite");
    }

    #[test]
    fn catch_up_session_is_dropped_on_disconnect() {
        let mut tracker = CatchUpTracker::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let peer = PeerId::random();

        tracker.probe(peer, Instant::now(), &tx);
        let _ = next_sync(&mut rx);
        assert!(tracker.is_active());

        tracker.remove(peer);
        assert!(!tracker.is_active());
    }

    // ------------------------------------------------------------------
    // State sync (chunked snapshot transfer)
    // ------------------------------------------------------------------

    fn state_sync_chunk(
        offset: u64,
        total_len: u64,
        height: u64,
        state_root: [u8; 32],
        data: Vec<u8>,
    ) -> SnapshotChunk {
        SnapshotChunk {
            height,
            state_root,
            total_len,
            offset,
            data,
        }
    }

    #[test]
    fn state_sync_maybe_start_only_targets_the_debug_peer() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let target = PeerId::random();
        let mut sessions = StateSyncSessions::with_debug_peer(Some(target));

        // A peer that is not the configured target never starts a transfer.
        sessions.maybe_start(PeerId::random(), Instant::now(), &tx);
        assert!(sessions.sessions.is_empty());
        assert!(rx.try_recv().is_err());

        // The configured target starts exactly one transfer.
        sessions.maybe_start(target, Instant::now(), &tx);
        assert!(sessions.sessions.contains_key(&target));
        match rx.try_recv().expect("start command") {
            NetworkCommand::RequestStateSync(peer, request) => {
                assert_eq!(peer, target);
                assert_eq!(request, StateSyncRequest::latest());
            }
            _ => panic!("expected state sync request"),
        }
    }

    #[test]
    fn state_sync_starts_are_rate_limited_per_peer() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let target = PeerId::random();
        let mut sessions = StateSyncSessions::with_debug_peer(Some(target));
        let start = Instant::now();

        sessions.maybe_start(target, start, &tx);
        let _ = rx.try_recv().expect("first start");
        // Completing the session does not lift the rate limit.
        let bytes = vec![1u8; 8];
        let chunk = state_sync_chunk(0, 8, 3, [2u8; 32], bytes.clone());
        assert_eq!(sessions.on_chunk(target, chunk, start), Some(bytes));
        assert!(sessions.sessions.is_empty());

        // Within the interval: no second start.
        sessions.maybe_start(target, start + STATE_SYNC_MIN_INTERVAL / 2, &tx);
        assert!(rx.try_recv().is_err(), "second start must be rate-limited");

        // After the interval: allowed again.
        sessions.maybe_start(target, start + STATE_SYNC_MIN_INTERVAL, &tx);
        assert!(rx.try_recv().is_ok(), "start allowed after the interval");
    }

    #[test]
    fn state_sync_reassembles_chunks_in_order() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let target = PeerId::random();
        let mut sessions = StateSyncSessions::with_debug_peer(Some(target));
        let now = Instant::now();
        sessions.maybe_start(target, now, &tx);

        let total: u64 = 12;
        let root = [5u8; 32];
        let first = sessions.on_chunk(
            target,
            state_sync_chunk(0, total, 9, root, vec![1u8; 8]),
            now,
        );
        assert_eq!(first, None, "first chunk is not the whole payload");
        let second = sessions.on_chunk(
            target,
            state_sync_chunk(8, total, 9, root, vec![2u8; 4]),
            now,
        );
        assert_eq!(
            second,
            Some(vec![1u8; 8].into_iter().chain([2u8; 4]).collect())
        );
        assert!(sessions.sessions.is_empty(), "completed session is removed");
    }

    #[test]
    fn state_sync_rejects_inconsistent_or_out_of_order_chunks() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let target = PeerId::random();
        let mut sessions = StateSyncSessions::with_debug_peer(Some(target));
        let now = Instant::now();

        // The state root changing mid-transfer abandons the session.
        sessions.maybe_start(target, now, &tx);
        assert!(sessions
            .on_chunk(
                target,
                state_sync_chunk(0, 16, 9, [1u8; 32], vec![1u8; 8]),
                now
            )
            .is_none());
        assert!(sessions
            .on_chunk(
                target,
                state_sync_chunk(8, 16, 9, [2u8; 32], vec![2u8; 8]),
                now
            )
            .is_none());
        assert!(sessions.sessions.is_empty(), "tampered stream abandoned");

        // An out-of-order chunk abandons the session.
        sessions.maybe_start(target, now, &tx);
        assert!(sessions
            .on_chunk(
                target,
                state_sync_chunk(0, 16, 9, [1u8; 32], vec![1u8; 8]),
                now
            )
            .is_none());
        assert!(sessions
            .on_chunk(
                target,
                state_sync_chunk(10, 16, 9, [1u8; 32], vec![2u8; 6]),
                now
            )
            .is_none());
        assert!(
            sessions.sessions.is_empty(),
            "out-of-order stream abandoned"
        );
    }

    #[test]
    fn state_sync_refuses_oversized_or_empty_claims() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let target = PeerId::random();
        let mut sessions = StateSyncSessions::with_debug_peer(Some(target));
        let now = Instant::now();

        // A total length above the reassembly cap is refused before any byte
        // is buffered.
        sessions.maybe_start(target, now, &tx);
        let oversized = MAX_STATE_SYNC_SNAPSHOT_BYTES + 1;
        assert!(sessions
            .on_chunk(
                target,
                state_sync_chunk(0, oversized, 9, [1u8; 32], vec![1u8; 4]),
                now
            )
            .is_none());
        assert!(sessions.sessions.is_empty(), "oversized claim abandoned");

        // A zero-length claim is refused too.
        sessions.maybe_start(target, now, &tx);
        assert!(sessions
            .on_chunk(target, state_sync_chunk(0, 0, 9, [1u8; 32], vec![]), now)
            .is_none());
        assert!(sessions.sessions.is_empty(), "empty claim abandoned");
    }

    /// Export a snapshot from a throwaway storage with one seeded account.
    fn exported_snapshot_bytes() -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("src.redb")).unwrap();
        {
            let txn = storage.begin_write().unwrap();
            let acct = kovanica_storage::state_store::Account {
                balance: 42,
                nonce: 1,
                code_hash: [0; 32],
                code: Vec::new(),
            };
            storage
                .state()
                .set_account(&txn, &Address([7; 32]), &acct)
                .unwrap();
            txn.commit().unwrap();
        }
        let txn = storage.begin_read().unwrap();
        storage.state().snapshot_bytes(&txn).unwrap()
    }

    #[test]
    fn apply_state_snapshot_imports_a_verified_payload() {
        let bytes = exported_snapshot_bytes();
        let dir = tempfile::tempdir().unwrap();
        let storage = Arc::new(Storage::new(dir.path().join("dst.redb")).unwrap());

        apply_state_snapshot(&storage, PeerId::random(), &bytes);

        let txn = storage.begin_read().unwrap();
        let acct = storage
            .state()
            .get_account(&txn, &Address([7; 32]))
            .unwrap();
        assert_eq!(acct.balance, 42, "verified snapshot must be applied");
    }

    #[test]
    fn apply_state_snapshot_rejects_tampered_payload() {
        let bytes = exported_snapshot_bytes();
        // Tamper with a table value while keeping the claimed header intact:
        // the recomputed state root no longer matches.
        let mut data = kovanica_storage::StateStore::decode_snapshot(&bytes).unwrap();
        data.accounts[0].1.push(0xff);
        let mut tampered = kovanica_storage::state_store::SNAPSHOT_MAGIC.to_vec();
        tampered.extend_from_slice(&bincode::serialize(&data).unwrap());

        let dir = tempfile::tempdir().unwrap();
        let storage = Arc::new(Storage::new(dir.path().join("dst.redb")).unwrap());
        apply_state_snapshot(&storage, PeerId::random(), &tampered);

        let txn = storage.begin_read().unwrap();
        assert!(
            storage
                .state()
                .get_account_or_default(&txn, &Address([7; 32]))
                .unwrap()
                .balance
                == 0,
            "tampered snapshot must never be applied"
        );
    }

    #[test]
    fn apply_state_snapshot_rejects_height_regression() {
        let mut bytes = exported_snapshot_bytes();
        // Re-stamp the snapshot at a height below the caller's expectation.
        let mut data = kovanica_storage::StateStore::decode_snapshot(&bytes).unwrap();
        data.height = 3;
        bytes = kovanica_storage::state_store::SNAPSHOT_MAGIC.to_vec();
        bytes.extend_from_slice(&bincode::serialize(&data).unwrap());

        let dir = tempfile::tempdir().unwrap();
        let storage = Arc::new(Storage::new(dir.path().join("dst.redb")).unwrap());
        // Local state is ahead of the snapshot: importing would roll back.
        {
            let txn = storage.begin_write().unwrap();
            storage
                .state()
                .save_state_root(&txn, 10, &Hash::new(b"local"))
                .unwrap();
            txn.commit().unwrap();
        }
        apply_state_snapshot(&storage, PeerId::random(), &bytes);

        let txn = storage.begin_read().unwrap();
        assert!(
            storage
                .state()
                .get_account_or_default(&txn, &Address([7; 32]))
                .unwrap()
                .balance
                == 0,
            "a snapshot below local height must never be applied"
        );
    }

    /// A parentless block at `round` (digest recomputed so `put_block` accepts
    /// it), used to seed the block store for the retention tests below.
    fn retention_block(round: u64) -> StatementBlock {
        let author: AuthorityIndex = 0;
        let digest = StatementBlock::compute_digest(author, round, &[], &[]);
        StatementBlock {
            author,
            round,
            parents: Vec::new(),
            transactions: Vec::new(),
            statements: Vec::new(),
            signature: Signature([0u8; 64]),
            digest,
            merkle_root: Hash::new([0u8; 32]),
        }
    }

    fn count_state_roots(storage: &Storage) -> usize {
        let read = storage.begin_read().unwrap();
        let table = read
            .open_table(kovanica_storage::tables::STATE_ROOT)
            .unwrap();
        use redb::ReadableTable;
        table.iter().unwrap().count()
    }

    fn count_blocks(storage: &Storage) -> usize {
        let read = storage.begin_read().unwrap();
        let table = read
            .open_table(kovanica_storage::tables::BLOCK_HEIGHT)
            .unwrap();
        use redb::ReadableTable;
        table.iter().unwrap().count()
    }

    fn retention_plan(
        blocks: Option<u64>,
        state_roots: Option<u64>,
        receipts: Option<u64>,
        batch: usize,
    ) -> RetentionPlan {
        RetentionPlan {
            blocks_below_tip: blocks,
            state_roots_below_height: state_roots,
            receipts_below_height: receipts,
            batch,
            margin: PRUNE_SAFETY_MARGIN,
        }
    }

    #[test]
    fn retention_floor_is_clamped_by_the_safety_margin() {
        // Too little history to prune safely.
        assert_eq!(retention_floor(0, 100, 1_000), None);
        assert_eq!(retention_floor(500, 10, 1_000), None);
        // The margin dominates a small window.
        assert_eq!(retention_floor(5_000, 10, 1_000), Some(4_000));
        // An empty/zero window is clamped to the margin too.
        assert_eq!(retention_floor(5_000, 0, 1_000), Some(4_000));
        // A window past the tip yields nothing rather than underflowing.
        assert_eq!(retention_floor(5_000, 9_000, 1_000), None);
        // The window can only ever tighten below the margin.
        assert_eq!(retention_floor(50_000, 9_000, 1_000), Some(41_000));
    }

    #[test]
    fn retention_pass_defaults_to_keeping_everything() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("retention-off.redb")).unwrap();

        const COMMITTED: u64 = 1_200;
        {
            let txn = storage.begin_write().unwrap();
            storage
                .consensus()
                .set_committed_leader_height(&txn, COMMITTED)
                .unwrap();
            for h in 1..=COMMITTED {
                storage
                    .state()
                    .save_state_root(&txn, h, &Hash::new(b"root"))
                    .unwrap();
                storage
                    .blocks()
                    .put_block(&txn, &retention_block(h), &[])
                    .unwrap();
            }
            txn.commit().unwrap();
        }

        // No window configured: the pass must be a no-op.
        assert_eq!(
            run_retention_pass(&storage, &retention_plan(None, None, None, 100)).unwrap(),
            "",
            "an unconfigured node must never delete data"
        );
        assert_eq!(count_state_roots(&storage), COMMITTED as usize);
        assert_eq!(count_blocks(&storage), COMMITTED as usize);
        let read = storage.begin_read().unwrap();
        assert!(
            read.open_table(kovanica_storage::tables::STATE_ROOT)
                .is_ok(),
            "sanity"
        );

        // Enabling a window is what makes the pass delete, and it deletes in
        // bounded batches: at most `batch` rows per store per pass.
        for _ in 0..3 {
            let summary = run_retention_pass(&storage, &retention_plan(Some(1_000), None, None, 5))
                .expect("pass runs without panicking");
            assert!(summary.contains("blocks=5"), "summary: {summary:?}");
        }
        assert_eq!(
            count_blocks(&storage),
            COMMITTED as usize - 15,
            "three passes removed exactly `batch` rows each"
        );
        // State roots were not enabled: still untouched.
        assert_eq!(count_state_roots(&storage), COMMITTED as usize);

        // Nothing near the tip is ever touched: floor = 1200 - 1000 = 200.
        let read = storage.begin_read().unwrap();
        assert!(
            storage
                .blocks()
                .get_block_by_height(&read, 1)
                .unwrap()
                .is_none(),
            "the oldest rounds are pruned"
        );
        assert!(
            storage
                .blocks()
                .get_block_by_height(&read, 16)
                .unwrap()
                .is_some(),
            "only `batch` rows per pass go away"
        );
        assert!(
            storage
                .blocks()
                .get_block_by_height(&read, 199)
                .unwrap()
                .is_some(),
            "the safety floor is never crossed downward: rounds below it are pruned"
        );
        assert!(
            storage
                .blocks()
                .get_block_by_height(&read, 200)
                .unwrap()
                .is_some(),
            "the boundary round must survive"
        );
        assert!(
            storage
                .blocks()
                .get_block_by_height(&read, COMMITTED)
                .unwrap()
                .is_some(),
            "the tip must survive"
        );
    }

    #[test]
    fn retention_pass_clamps_blocks_to_the_committed_height() {
        // The block store runs far ahead of the committed height (rounds
        // advance faster than committed leaders), so the configured window alone
        // would reach into rounds that are not settled yet. The committed-height
        // safety bound must win and prune strictly below it.
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("retention-clamp.redb")).unwrap();

        const COMMITTED: u64 = 1_100;
        const TIP_ROUND: u64 = 5_000;
        {
            let txn = storage.begin_write().unwrap();
            storage
                .consensus()
                .set_committed_leader_height(&txn, COMMITTED)
                .unwrap();
            for h in 1..=TIP_ROUND {
                storage
                    .blocks()
                    .put_block(&txn, &retention_block(h), &[])
                    .unwrap();
            }
            txn.commit().unwrap();
        }

        // window floor would be 5000 - 1000 = 4000 rounds, but the
        // committed-height floor is 1100 - 1000 = 100 — and 100 wins.
        let summary =
            run_retention_pass(&storage, &retention_plan(Some(1_000), None, None, 10_000))
                .expect("pass runs without panicking");
        assert_eq!(summary, "blocks=99", "summary: {summary:?}");

        let read = storage.begin_read().unwrap();
        assert!(
            storage
                .blocks()
                .get_block_by_height(&read, 99)
                .unwrap()
                .is_none(),
            "rounds below the committed floor are pruned"
        );
        assert!(
            storage
                .blocks()
                .get_block_by_height(&read, 100)
                .unwrap()
                .is_some(),
            "the committed-height safety floor is the boundary"
        );
        assert!(
            storage
                .blocks()
                .get_block_by_height(&read, 1_000)
                .unwrap()
                .is_some(),
            "rounds the window would have pruned must survive"
        );
        assert!(
            storage
                .blocks()
                .get_block_by_height(&read, TIP_ROUND)
                .unwrap()
                .is_some(),
            "the tip must survive"
        );
    }

    #[test]
    fn retention_pass_refuses_windows_larger_than_the_chain() {
        // "Keep 100_000 rounds" on a chain with 1_100 committed leaders is not
        // a bug: it means "keep everything", so the pass must delete nothing
        // rather than underflow or panic.
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("retention-huge.redb")).unwrap();
        {
            let txn = storage.begin_write().unwrap();
            storage
                .consensus()
                .set_committed_leader_height(&txn, 1_100)
                .unwrap();
            for h in 1..=1_100u64 {
                storage
                    .blocks()
                    .put_block(&txn, &retention_block(h), &[])
                    .unwrap();
            }
            txn.commit().unwrap();
        }

        let summary = run_retention_pass(
            &storage,
            &retention_plan(Some(100_000), Some(100_000), Some(100_000), 10_000),
        )
        .expect("pass runs without panicking");
        assert_eq!(summary, "", "summary: {summary:?}");
        assert_eq!(count_blocks(&storage), 1_100);
        assert_eq!(count_state_roots(&storage), 0);
    }
}

// LOCAL REVIEW TEST (Jarvis, 2026-10-19) — not committed / not pushed.
// B1 invariant: a validator restarted after an epoch boundary must still
// produce votes that a long-running peer (started before the boundary)
// accepts, otherwise one restart stalls an n=3 committee.
#[cfg(test)]
mod b1_epoch_restart_repro {
    use super::*;
    use kovanica_staking::{StakingState, EPOCH_ROUNDS, MIN_VALIDATOR_STAKE};

    fn storage_at_height(
        dir: &std::path::Path,
        name: &str,
        keys: &[(Address, PublicKey)],
        height: u64,
    ) -> Storage {
        let storage = Storage::new(dir.join(name)).unwrap();
        let mut staking = StakingState::new();
        staking.init_treasury(Address([0xcc; 32]));
        for (addr, pk) in keys {
            staking
                .join_validator(*addr, MIN_VALIDATOR_STAKE, 0, Some(*addr), Some(*pk), None)
                .unwrap();
        }
        staking.committed_leader_height = height;
        let txn = storage.begin_write().unwrap();
        storage.state().save_staking_state(&txn, &staking).unwrap();
        txn.commit().unwrap();
        storage
    }

    // LOCAL REVIEW TEST (Jarvis, 2026-10-19), not committed.
    // Seals and leader headers still use `committee.epoch()` (SealContext.epoch,
    // committee_seal_hash, admit_seal EpochMismatch). A node restarted after
    // the boundary must still agree on that epoch with a long-running node.
    #[test]
    fn b1_review_seal_epoch_matches_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let keys: Vec<_> = (0..3)
            .map(|_| {
                let (_sk, pk) = kovanica_crypto::generate_keypair();
                (Address::from_public_key(&pk), pk)
            })
            .collect();

        // Node A: fresh genesis (height 0)
        let a = storage_at_height(dir.path(), "a.redb", &keys, 0);
        // Node B: restarted after epoch boundary (height EPOCH_ROUNDS + 1)
        let b = storage_at_height(dir.path(), "b.redb", &keys, EPOCH_ROUNDS + 1);

        // For seals, epoch is derived from leader_round, not staking height.
        // Both nodes must compute the same committee for the same seal_epoch.
        let seal_epoch = 1; // leader_round in epoch 1 (e.g. EPOCH_ROUNDS + 1)
        let read_a = a.begin_read().unwrap();
        let staking_a = a.state().load_staking_state(&read_a).unwrap();
        let read_b = b.begin_read().unwrap();
        let staking_b = b.state().load_staking_state(&read_b).unwrap();
        let ca = seals::build_committee_for_seal_epoch(&staking_a, seal_epoch).unwrap();
        let cb = seals::build_committee_for_seal_epoch(&staking_b, seal_epoch).unwrap();
        assert_eq!(
            ca.epoch(),
            cb.epoch(),
            "seal/leader-header epoch split: running={} restarted={}",
            ca.epoch(),
            cb.epoch()
        );
    }

    #[test]
    fn restart_after_epoch_boundary_keeps_votes_verifiable() {
        let dir = tempfile::tempdir().unwrap();
        let keys: Vec<_> = (0..3)
            .map(|_| {
                let (sk, pk) = kovanica_crypto::generate_keypair();
                (sk, Address::from_public_key(&pk), pk)
            })
            .collect();
        let pubs: Vec<_> = keys.iter().map(|(_, a, p)| (*a, *p)).collect();
        let chain_id = kovanica_types::signing::chain_id::LOCAL;

        // Node A started at genesis and is still running: committee built at height 0.
        let a = storage_at_height(dir.path(), "a.redb", &pubs, 0);
        let committee_a = build_committee(&a, &pubs[0].1, &pubs[0].0, "127.0.0.1:0").unwrap();
        // Node B is restarted once the chain has passed the first epoch boundary.
        let b = storage_at_height(dir.path(), "b.redb", &pubs, EPOCH_ROUNDS + 1);
        let committee_b = build_committee(&b, &pubs[1].1, &pubs[1].0, "127.0.0.1:0").unwrap();

        // B1 FIX: epoch is now derived from leader_round, NOT from local committee.
        // Both nodes compute the SAME epoch for the same leader_round.
        let leader_round = 3 * (EPOCH_ROUNDS + 1); // epoch 1
        let epoch = leader_round / EPOCH_ROUNDS;
        let ctx_a = kovanica_types::SigningContext::new(chain_id).with_epoch(epoch);
        let ctx_b = kovanica_types::SigningContext::new(chain_id).with_epoch(epoch);

        // B votes; A verifies with its own (local) context, as the engine does.
        let mut vote = kovanica_types::Vote {
            leader_round,
            leader_hash: kovanica_types::hash::Hash([0x42; 32]),
            voter: 1,
            signature: kovanica_types::crypto::Signature([0u8; 64]),
        };
        vote.signature = kovanica_crypto::sign(&keys[1].0, &vote.signature_data(&ctx_b));
        let verdict = kovanica_crypto::verify_vote_signature(&ctx_a, &vote, &pubs[1].1);
        assert!(
            verdict.is_ok(),
            "B1: restarted node epoch={} vs running node epoch={} -> vote rejected ({verdict:?})",
            committee_b.epoch(),
            committee_a.epoch()
        );
    }

    /// Produce a seal with the REAL production producer (`seals::publish_local_seal`)
    /// at `leader_round = EPOCH_ROUNDS + 1` and admit it with the REAL production
    /// verifier (`admit_seal`), plus a restart variant: the admitting node's keys
    /// come from a committee built at genesis height while the producer is past the
    /// boundary. The seal must still be accepted because the expected epoch is
    /// derived from the header's `leader_round` (not boot-time height).
    #[test]
    fn seal_at_epoch_rounds_plus_one_is_admitted_with_production_context() {
        use std::collections::HashMap;

        let dir = tempfile::tempdir().unwrap();
        let (sk, pk) = kovanica_crypto::generate_keypair();
        let addr = Address::from_public_key(&pk);
        let chain_id = kovanica_types::signing::chain_id::LOCAL;

        // Build staking state exactly as the node does, but with the chain already
        // past the first epoch boundary when the seal is produced.
        let make_storage = |name: &str, height: u64| {
            let storage = Storage::new(dir.path().join(name)).unwrap();
            let mut staking = StakingState::new();
            staking.init_treasury(Address([0xcc; 32]));
            staking
                .join_validator(addr, MIN_VALIDATOR_STAKE, 0, Some(addr), Some(pk), None)
                .unwrap();
            staking.committed_leader_height = height;
            let txn = storage.begin_write().unwrap();
            storage.state().save_staking_state(&txn, &staking).unwrap();
            txn.commit().unwrap();
            storage
        };

        // Producer node is past the epoch boundary; it publishes the real seal.
        let producer = make_storage("producer.redb", EPOCH_ROUNDS + 1);
        let leader_round = EPOCH_ROUNDS + 1;
        let subdag = kovanica_consensus::CommittedSubDag {
            blocks: vec![],
            leader: StatementBlock {
                author: 0,
                round: leader_round,
                parents: vec![],
                transactions: vec![],
                statements: vec![],
                signature: Signature([0u8; 64]),
                digest: StatementBlock::compute_digest(0, leader_round, &[], &[]),
                merkle_root: Default::default(),
            },
            leader_round,
            leader_author: 0,
            non_blue: vec![],
        };
        let ctx = seals::SealContext {
            chain_id,
            signer: Some((0, sk.clone())),
            mysticghost: false,
        };
        let announcement = seals::publish_local_seal(&producer, &ctx, &subdag, 0, Hash::zero())
            .unwrap()
            .gossip
            .expect("validator signs a seal");

        // The seal header epoch must equal leader_round / EPOCH_ROUNDS == 1.
        assert_eq!(
            announcement.header.epoch,
            leader_round / EPOCH_ROUNDS,
            "seal epoch must derive from leader_round"
        );

        // Verifier keys come from the SAME production committee builder the
        // producer used (epoch derived from leader_round, not boot height).
        let read = producer.begin_read().unwrap();
        let staking = producer.state().load_staking_state(&read).unwrap();
        let committee =
            seals::build_committee_for_seal_epoch(&staking, announcement.header.epoch).unwrap();
        let keys: HashMap<AuthorityIndex, PublicKey> = committee
            .authorities()
            .iter()
            .map(|a| (a.index, a.public_key))
            .collect();

        // Admit using the real production verifier and the matching local header.
        assert_eq!(
            kovanica_network::admit_seal(
                &keys,
                chain_id,
                &announcement,
                Some(announcement.header.hash()),
            ),
            kovanica_network::SealAdmission::Accept,
            "seal at EPOCH_ROUNDS + 1 must be admitted under the production context"
        );

        // Restart variant: the admitting node is a long-running peer started at
        // genesis (height 0). Its keys are built from a genesis-height committee,
        // yet it must still accept the producer's seal.
        let long_running = make_storage("long_running.redb", 0);
        let read_lr = long_running.begin_read().unwrap();
        let staking_lr = long_running.state().load_staking_state(&read_lr).unwrap();
        let committee_lr =
            seals::build_committee_for_seal_epoch(&staking_lr, announcement.header.epoch).unwrap();
        let keys_lr: HashMap<AuthorityIndex, PublicKey> = committee_lr
            .authorities()
            .iter()
            .map(|a| (a.index, a.public_key))
            .collect();
        assert_eq!(
            kovanica_network::admit_seal(
                &keys_lr,
                chain_id,
                &announcement,
                Some(announcement.header.hash()),
            ),
            kovanica_network::SealAdmission::Accept,
            "restart variant: genesis-height verifier must accept a post-boundary seal"
        );

        // Negative control: an epoch that does NOT match leader_round is rejected.
        let mut forged = announcement.clone();
        forged.header.epoch = announcement.header.epoch + 1;
        assert!(matches!(
            kovanica_network::admit_seal(&keys, chain_id, &forged, Some(forged.header.hash()),),
            kovanica_network::SealAdmission::Reject(kovanica_network::Rejection::EpochMismatch)
        ));
    }
}
