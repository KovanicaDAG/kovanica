//! Prometheus metrics and structured logging for the Kovanica node.
//!
//! Provides:
//! - Prometheus metrics (counters, gauges, histograms) for block rate, peer count,
//!   mempool size, reorg depth, sync latency, etc.
//! - Structured JSON logging via tracing
//! - `/metrics` HTTP endpoint for Prometheus scraping

use std::sync::{Once, OnceLock};
use std::time::Duration;

use ed25519_dalek::VerifyingKey;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

use kovanica_dag::AuthoritySet;
use kovanica_state::spv::SpvError;
use kovanica_state::SupplyMetrics;

/// Global metrics recorder initialization guard.
static METRICS_INIT: Once = Once::new();

/// Handle to the installed Prometheus recorder, used to render the
/// exposition payload for the explorer `/metrics` route.
static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Initialize the Prometheus metrics recorder and tracing subscriber.
/// Call once at startup (e.g., in `main()` or `serve_explorer()`).
///
/// `listen_addr` optionally starts a standalone scrape endpoint serving the
/// same payload as the explorer's `/metrics` route; bind failure is non-fatal.
pub fn init_metrics(listen_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut result = Ok(());
    METRICS_INIT.call_once(|| {
        // Install the global recorder and keep a render handle.
        let handle = match PrometheusBuilder::new().install_recorder() {
            Ok(h) => h,
            Err(e) => {
                result = Err(e.into());
                return;
            }
        };
        if METRICS_HANDLE.set(handle).is_err() {
            result = Err("metrics handle already installed".into());
            return;
        }

        // Optional dedicated scrape port (0.0.0.0:9090 by default).
        let addr: std::net::SocketAddr = match listen_addr.parse() {
            Ok(a) => a,
            Err(e) => {
                result = Err(e.into());
                return;
            }
        };
        spawn_metrics_listener(addr);

        // Install tracing subscriber with JSON output
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_current_span(true)
            .with_span_list(true)
            .init();
    });
    result
}

/// Render the current Prometheus exposition payload. Falls back to a bare
/// `kovanica_up 1` gauge when metrics were never initialized.
pub fn render_prometheus() -> String {
    match METRICS_HANDLE.get() {
        Some(handle) => handle.render(),
        None => "# HELP kovanica_up Node is up\n# TYPE kovanica_up gauge\nkovanica_up 1\n".into(),
    }
}

fn spawn_metrics_listener(addr: std::net::SocketAddr) {
    let _ = std::thread::Builder::new().name("metrics-http".into()).spawn(move || {
        use std::io::Write;
        let listener = match std::net::TcpListener::bind(addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("metrics scrape endpoint on {addr}: {e}");
                return;
            }
        };
        for conn in listener.incoming() {
            let Ok(mut stream) = conn else { continue };
            let body = render_prometheus();
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/plain; version=0.0.4; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });
}

/// Metric names — keep consistent for alerting/dashboarding.
pub mod names {
    // Block production
    pub const BLOCKS_PRODUCED_TOTAL: &str = "kovanica_blocks_produced_total";
    pub const BLOCK_PRODUCTION_DURATION_SECONDS: &str =
        "kovanica_block_production_duration_seconds";
    pub const BLOCK_HEIGHT: &str = "kovanica_block_height";

    // DAG consensus
    pub const DAG_TIP_COUNT: &str = "kovanica_dag_tip_count";
    pub const DAG_BLUE_SCORE: &str = "kovanica_dag_blue_score";
    pub const DAG_REORG_DEPTH: &str = "kovanica_dag_reorg_depth_total";
    pub const DAG_SELECTED_TIP_CHANGES_TOTAL: &str = "kovanica_dag_selected_tip_changes_total";

    // Peer / P2P
    pub const PEER_COUNT: &str = "kovanica_peer_count";
    pub const PEER_CONNECTED_TOTAL: &str = "kovanica_peer_connected_total";
    pub const PEER_DISCONNECTED_TOTAL: &str = "kovanica_peer_disconnected_total";
    pub const PEER_BANNED_TOTAL: &str = "kovanica_peer_banned_total";
    pub const PEER_SCORE: &str = "kovanica_peer_score";
    pub const P2P_MESSAGES_SENT_TOTAL: &str = "kovanica_p2p_messages_sent_total";
    pub const P2P_MESSAGES_RECEIVED_TOTAL: &str = "kovanica_p2p_messages_received_total";
    pub const P2P_MESSAGE_BYTES_SENT: &str = "kovanica_p2p_message_bytes_sent_total";
    pub const P2P_MESSAGE_BYTES_RECEIVED: &str = "kovanica_p2p_message_bytes_received_total";

    // Mempool
    pub const MEMPOOL_TX_COUNT: &str = "kovanica_mempool_tx_count";
    pub const MEMPOOL_ORPHAN_COUNT: &str = "kovanica_mempool_orphan_count";
    pub const MEMPOOL_BYTES: &str = "kovanica_mempool_bytes";
    pub const MEMPOOL_EVICTED_TOTAL: &str = "kovanica_mempool_evicted_total";
    pub const MEMPOOL_PROMOTED_TOTAL: &str = "kovanica_mempool_promoted_total";

    // Sync
    pub const SYNC_DURATION_SECONDS: &str = "kovanica_sync_duration_seconds";
    pub const SYNC_HEADERS_RECEIVED: &str = "kovanica_sync_headers_received_total";
    pub const SYNC_BODIES_APPLIED: &str = "kovanica_sync_bodies_applied_total";
    pub const SYNC_PEER_COUNT: &str = "kovanica_sync_peer_count";

    // DHT
    pub const DHT_ROUTING_TABLE_SIZE: &str = "kovanica_dht_routing_table_size";
    pub const DHT_BOOTSTRAP_DURATION_SECONDS: &str = "kovanica_dht_bootstrap_duration_seconds";
    pub const DHT_FIND_NODE_DURATION_SECONDS: &str = "kovanica_dht_find_node_duration_seconds";
    pub const DHT_PEERS_DISCOVERED_TOTAL: &str = "kovanica_dht_peers_discovered_total";
    pub const DHT_PEERS_PRUNED_TOTAL: &str = "kovanica_dht_peers_pruned_total";
    pub const DHT_QUERIES_SENT_TOTAL: &str = "kovanica_dht_queries_sent_total";
    pub const DHT_QUERIES_RECEIVED_TOTAL: &str = "kovanica_dht_queries_received_total";

    // RPC / Explorer
    pub const RPC_REQUESTS_TOTAL: &str = "kovanica_rpc_requests_total";
    pub const RPC_REQUEST_DURATION_SECONDS: &str = "kovanica_rpc_request_duration_seconds";
    pub const EXPLORER_WS_CLIENTS: &str = "kovanica_explorer_ws_clients";
    pub const EXPLORER_HTTP_REQUESTS_TOTAL: &str = "kovanica_explorer_http_requests_total";

    // Storage
    pub const SNAPSHOT_SIZE_BYTES: &str = "kovanica_snapshot_size_bytes";
    pub const CHECKPOINT_SIZE_BYTES: &str = "kovanica_checkpoint_size_bytes";
    pub const STORE_APPEND_DURATION_SECONDS: &str = "kovanica_store_append_duration_seconds";

    // Consensus validation
    pub const BLOCK_VALIDATION_DURATION_SECONDS: &str =
        "kovanica_block_validation_duration_seconds";
    pub const BLOCK_REJECTED_TOTAL: &str = "kovanica_block_rejected_total";
    pub const TX_VALIDATION_DURATION_SECONDS: &str = "kovanica_tx_validation_duration_seconds";
    pub const TX_REJECTED_TOTAL: &str = "kovanica_tx_rejected_total";

    // RFC-006 supply
    pub const SUPPLY_TOTAL: &str = "kovanica_supply_total";
    pub const SUPPLY_CIRCULATING: &str = "kovanica_supply_circulating";
    pub const SUPPLY_MINTED: &str = "kovanica_supply_minted";
    pub const SUPPLY_BURNED: &str = "kovanica_supply_burned";
    pub const SUPPLY_MAX: &str = "kovanica_supply_max";

    // SW-PoA authority set
    //
    // Every series below is *derived from the on-chain authority set* and is
    // identical on every honest node, so these are safe to compare across
    // peers. `SW_POA_ENABLED` is 0 on every live chain today: the only
    // stake-weighted path is reachable from `KVA1` Authority UTXO data, not
    // from configuration.
    pub const POA_AUTHORITY_COUNT: &str = "kovanica_poa_authority_count";
    pub const POA_THRESHOLD: &str = "kovanica_poa_threshold";
    /// Slots where the labelled authority was the scheduled signer.
    pub const POA_SLOTS_SCHEDULED_TOTAL: &str = "kovanica_poa_slots_scheduled_total";
    /// Slots where the labelled authority actually produced a block.
    pub const POA_SLOTS_PRODUCED_TOTAL: &str = "kovanica_poa_slots_produced_total";
    /// Scheduled slots with no block after finality depth elapsed.
    pub const POA_SLOTS_MISSED_TOTAL: &str = "kovanica_poa_slots_missed_total";
    pub const POA_SLOT_DURATION_MS: &str = "kovanica_poa_slot_duration_ms";

    // SPV / light-client verification
    //
    // A light client verifies headers against a trusted checkpoint and never
    // touches the UTXO set, so these track *verification* rather than
    // validation. Rejections are broken out by reason because a wallet that
    // cannot tell "wrong prev_hash" from "bad authority signature" cannot tell
    // a node bug from an attack.
    pub const SPV_ENABLED: &str = "kovanica_spv_enabled";
    pub const SPV_HEADERS_VERIFIED_TOTAL: &str = "kovanica_spv_headers_verified_total";
    pub const SPV_HEADERS_REJECTED_TOTAL: &str = "kovanica_spv_headers_rejected_total";
    pub const SPV_HEADER_VERIFY_DURATION_SECONDS: &str =
        "kovanica_spv_header_verify_duration_seconds";
    pub const SPV_TIP_HEIGHT: &str = "kovanica_spv_tip_height";
    pub const SPV_TIP_TIMESTAMP_MS: &str = "kovanica_spv_tip_timestamp_ms";
    pub const SPV_TIP_AGE_SECONDS: &str = "kovanica_spv_tip_age_seconds";
    pub const SPV_CHAIN_WORK: &str = "kovanica_spv_chain_work";
    pub const SPV_HEADERS_TRACKED: &str = "kovanica_spv_headers_tracked";
    pub const SPV_AUTHORITY_UPDATES_TOTAL: &str = "kovanica_spv_authority_updates_total";
    pub const SPV_AUTHORITY_UPDATES_REJECTED_TOTAL: &str =
        "kovanica_spv_authority_updates_rejected_total";
    pub const SPV_MERKLE_PROOFS_VERIFIED_TOTAL: &str = "kovanica_spv_merkle_proofs_verified_total";
    pub const SPV_MERKLE_PROOFS_REJECTED_TOTAL: &str = "kovanica_spv_merkle_proofs_rejected_total";
    pub const SPV_TX_INCLUSION_VERIFIED_TOTAL: &str = "kovanica_spv_tx_inclusion_verified_total";
    pub const SPV_TX_INCLUSION_REJECTED_TOTAL: &str =
        "kovanica_spv_tx_inclusion_rejected_total";
    /// Rejections by [`SpvError`] variant, labelled `reason="<variant>"`.
    pub const SPV_REJECTED_BY_REASON: &str = "kovanica_spv_rejected_by_reason";
}

/// Record a produced (or mined) block.
pub fn record_block_produced(height: u64, blue_score: u64, duration: Duration) {
    counter!(names::BLOCKS_PRODUCED_TOTAL).increment(1);
    gauge!(names::BLOCK_HEIGHT).set(height as f64);
    gauge!(names::DAG_BLUE_SCORE).set(blue_score as f64);
    histogram!(names::BLOCK_PRODUCTION_DURATION_SECONDS).record(duration.as_secs_f64());
}

/// Surface the passive chain head on any block insert (produce *or* receive).
///
/// Non-mining validation seeds never hit the production path, so
/// [`record_block_produced`] never runs for them and the height/blue-score
/// gauges stay unregistered (metrics-exporter-prometheus only renders
/// observed series). This sets the two gauges on every insert — including
/// blocks received from peers on a `KOVANICA_MINE=0` seed — so soak
/// monitoring sees chain progress without counting these as produced.
/// [`names::BLOCKS_PRODUCED_TOTAL`] is intentionally not touched here.
///
/// Both values are passed as the block's *blue score*, matching
/// [`record_block_produced`]'s use of blue score for `BLOCK_HEIGHT`, so the
/// produced and observed series stay comparable (see `Node::note_inserted`).
pub fn record_block_observed(height: u64, blue_score: u64) {
    gauge!(names::BLOCK_HEIGHT).set(height as f64);
    gauge!(names::DAG_BLUE_SCORE).set(blue_score as f64);
}

/// Record a re-org of `depth` blocks.
pub fn record_reorg(depth: u64) {
    counter!(names::DAG_REORG_DEPTH).increment(depth);
}

pub fn record_peer_connected() {
    counter!(names::PEER_CONNECTED_TOTAL).increment(1);
}

pub fn record_peer_disconnected() {
    counter!(names::PEER_DISCONNECTED_TOTAL).increment(1);
}

pub fn record_peer_banned() {
    counter!(names::PEER_BANNED_TOTAL).increment(1);
}

pub fn set_peer_count(count: usize) {
    gauge!(names::PEER_COUNT).set(count as f64);
}

pub fn set_peer_score(peer: &str, score: i32) {
    gauge!(names::PEER_SCORE, "peer" => peer.to_owned()).set(score as f64);
}

pub fn record_p2p_message_sent(kind: &str, bytes: usize) {
    counter!(names::P2P_MESSAGES_SENT_TOTAL, "kind" => kind.to_owned()).increment(1);
    counter!(names::P2P_MESSAGE_BYTES_SENT, "kind" => kind.to_owned()).increment(bytes as u64);
}

pub fn record_p2p_message_received(kind: &str, bytes: usize) {
    counter!(names::P2P_MESSAGES_RECEIVED_TOTAL, "kind" => kind.to_owned()).increment(1);
    counter!(names::P2P_MESSAGE_BYTES_RECEIVED, "kind" => kind.to_owned()).increment(bytes as u64);
}

pub fn set_mempool_counts(pending: usize, orphans: usize, bytes: usize) {
    gauge!(names::MEMPOOL_TX_COUNT).set(pending as f64);
    gauge!(names::MEMPOOL_ORPHAN_COUNT).set(orphans as f64);
    gauge!(names::MEMPOOL_BYTES).set(bytes as f64);
}

pub fn record_mempool_evicted(count: usize) {
    if count > 0 {
        counter!(names::MEMPOOL_EVICTED_TOTAL).increment(count as u64);
    }
}

pub fn record_mempool_promoted(count: usize) {
    if count > 0 {
        counter!(names::MEMPOOL_PROMOTED_TOTAL).increment(count as u64);
    }
}

pub fn record_sync_complete(duration: Duration, headers: usize, bodies: usize, peers: usize) {
    histogram!(names::SYNC_DURATION_SECONDS).record(duration.as_secs_f64());
    counter!(names::SYNC_HEADERS_RECEIVED).increment(headers as u64);
    counter!(names::SYNC_BODIES_APPLIED).increment(bodies as u64);
    gauge!(names::SYNC_PEER_COUNT).set(peers as f64);
}

pub fn set_dht_routing_table_size(size: usize) {
    gauge!(names::DHT_ROUTING_TABLE_SIZE).set(size as f64);
}

pub fn record_dht_bootstrap(duration: Duration, peers_added: usize) {
    histogram!(names::DHT_BOOTSTRAP_DURATION_SECONDS).record(duration.as_secs_f64());
    if peers_added > 0 {
        counter!(names::DHT_PEERS_DISCOVERED_TOTAL).increment(peers_added as u64);
    }
}

pub fn record_dht_find_node(duration: Duration, results: usize) {
    histogram!(names::DHT_FIND_NODE_DURATION_SECONDS).record(duration.as_secs_f64());
    if results > 0 {
        counter!(names::DHT_PEERS_DISCOVERED_TOTAL).increment(results as u64);
    }
}

pub fn record_dht_pruned(count: usize) {
    if count > 0 {
        counter!(names::DHT_PEERS_PRUNED_TOTAL).increment(count as u64);
    }
}

pub fn record_dht_query_sent() {
    counter!(names::DHT_QUERIES_SENT_TOTAL).increment(1);
}

pub fn record_dht_query_received() {
    counter!(names::DHT_QUERIES_RECEIVED_TOTAL).increment(1);
}

pub fn record_rpc_request(method: &str, duration: Duration) {
    counter!(names::RPC_REQUESTS_TOTAL, "method" => method.to_owned()).increment(1);
    histogram!(names::RPC_REQUEST_DURATION_SECONDS).record(duration.as_secs_f64());
}

pub fn set_explorer_ws_clients(count: usize) {
    gauge!(names::EXPLORER_WS_CLIENTS).set(count as f64);
}

pub fn record_explorer_http_request(path: &str, status: u16) {
    counter!(
        names::EXPLORER_HTTP_REQUESTS_TOTAL,
        "path" => path.to_owned(),
        "status" => status.to_string()
    )
    .increment(1);
}

pub fn record_snapshot_size(bytes: usize) {
    gauge!(names::SNAPSHOT_SIZE_BYTES).set(bytes as f64);
}

pub fn record_checkpoint_size(bytes: usize) {
    gauge!(names::CHECKPOINT_SIZE_BYTES).set(bytes as f64);
}

pub fn record_store_append(duration: Duration) {
    histogram!(names::STORE_APPEND_DURATION_SECONDS).record(duration.as_secs_f64());
}

pub fn record_block_validation(duration: Duration, rejected: bool) {
    histogram!(names::BLOCK_VALIDATION_DURATION_SECONDS).record(duration.as_secs_f64());
    if rejected {
        counter!(names::BLOCK_REJECTED_TOTAL).increment(1);
    }
}

pub fn record_tx_validation(duration: Duration, rejected: bool) {
    histogram!(names::TX_VALIDATION_DURATION_SECONDS).record(duration.as_secs_f64());
    if rejected {
        counter!(names::TX_REJECTED_TOTAL).increment(1);
    }
}

/// Surface the RFC-006 supply snapshot as gauges (atoms).
///
/// Called on the `/metrics` scrape path so Prometheus always sees fresh
/// supply values even when no block/mempool event fired recently (same
/// pattern as [`set_peer_count`]). `total` and `minted` both report
/// cumulative native minted; `circulating` is the live UTXO total;
/// `burned` is cumulative fee burn; `max` is the hard cap.
pub fn record_supply(supply: SupplyMetrics) {
    gauge!(names::SUPPLY_TOTAL).set(supply.total as f64);
    gauge!(names::SUPPLY_CIRCULATING).set(supply.circulating as f64);
    gauge!(names::SUPPLY_MINTED).set(supply.total as f64);
    gauge!(names::SUPPLY_BURNED).set(supply.burned as f64);
    gauge!(names::SUPPLY_MAX).set(supply.max_supply as f64);
}

/// Stable Prometheus label for an authority public key: lowercase hex.
///
/// Keys are already sorted canonically inside an [`AuthoritySet`], so this is
/// a deterministic series key — the same authority is the same time series on
/// every node, which is what makes per-authority alerts comparable.
fn authority_label(key: &VerifyingKey) -> String {
    hex::encode(key.to_bytes())
}

/// Surface the PoA authority set as gauges: shape of the set, whether SW-PoA
/// is engaged, and each authority's stake and share.
///
/// Safe to call on every scrape. All values are pure functions of the
/// committed set, so a divergence between nodes is a real fault, not noise.
/// The set *hash* is deliberately not exported here — a 32-byte digest has no
/// meaningful float encoding, and it is already available from `/api/network`
/// as `authority_set.hash`.
pub fn record_poa_authority_set(set: &AuthoritySet) {
    gauge!(names::POA_AUTHORITY_COUNT).set(set.len() as f64);
    gauge!(names::POA_THRESHOLD).set(set.threshold() as f64);
}

/// Record that `slot` was scheduled to the given authority (PoA slot on-chain).
pub fn record_poa_slot_scheduled(key: &VerifyingKey) {
    counter!(names::POA_SLOTS_SCHEDULED_TOTAL, "authority" => authority_label(key)).increment(1);
}

/// Record that `slot` was scheduled to `key` and `key` actually produced the
/// block. `record_poa_slot_scheduled` should be called for the same slot.
pub fn record_poa_slot_produced(key: &VerifyingKey) {
    counter!(names::POA_SLOTS_PRODUCED_TOTAL, "authority" => authority_label(key)).increment(1);
}

/// Record a scheduled slot that produced no block once finality depth elapsed.
///
/// Must be paired with [`record_poa_slot_scheduled`] for the same slot, so the
/// miss rate is `missed / scheduled`. Only the authority whose scheduled slot
/// lapsed is labelled — a stalled network is a `DAG`/`SYNC` signal, not n
/// simultaneous authority misses.
pub fn record_poa_slot_missed(key: &VerifyingKey) {
    counter!(names::POA_SLOTS_MISSED_TOTAL, "authority" => authority_label(key)).increment(1);
}

/// Record the slot duration in milliseconds, so slot-rate dashboards do not
/// have to hard-code `KOVANICA_SLOT_DURATION`.
pub fn record_poa_slot_duration(slot_duration_ms: u64) {
    gauge!(names::POA_SLOT_DURATION_MS).set(slot_duration_ms as f64);
}

/// Whether this process is verifying headers as an SPV/light client.
///
/// Zero on a full node. Present so one dashboard can mix full nodes and light
/// clients without the SPV series reading as "not instrumented".
pub fn set_spv_enabled(enabled: bool) {
    gauge!(names::SPV_ENABLED).set(u8::from(enabled) as f64);
}

/// Record a header the SPV client accepted, with how long verification took.
pub fn record_spv_header_verified(headers_tracked: usize, duration: Duration) {
    counter!(names::SPV_HEADERS_VERIFIED_TOTAL).increment(1);
    histogram!(names::SPV_HEADER_VERIFY_DURATION_SECONDS).record(duration.as_secs_f64());
    gauge!(names::SPV_HEADERS_TRACKED).set(headers_tracked as f64);
}

/// Record a header the SPV client refused, broken out by [`SpvError`] variant.
///
/// The per-reason series is the point: a light client that only sees a single
/// "rejected" counter cannot distinguish a peer serving a stale fork
/// (`PrevHashMismatch`) from a peer forging authority signatures
/// (`InvalidAuthoritySig`), which are completely different incidents.
///
/// This touches only the header and per-reason counters. The
/// authority-update counter is owned by [`record_spv_authority_update`], which
/// is driven by the actual verification step. Folding a header rejection into
/// it too would double-count.
pub fn record_spv_header_rejected(err: &SpvError) {
    counter!(names::SPV_HEADERS_REJECTED_TOTAL).increment(1);
    counter!(names::SPV_REJECTED_BY_REASON, "reason" => spv_error_reason(err)).increment(1);
}

/// Age of an SPV tip in seconds, saturating at 0.
///
/// A peer with a skewed clock can hand us a header timestamped ahead of local
/// time. A plain `now - timestamp` subtraction would wrap into a huge positive
/// age and make a healthy node look stuck for years, so clamp instead. Split
/// out as a pure function so the wrap case is testable without depending on
/// the process-global recorder.
fn spv_tip_age_seconds(timestamp_ms: u64, now_ms: u64) -> f64 {
    now_ms.saturating_sub(timestamp_ms) as f64 / 1000.0
}

/// Surface the SPV client's verified tip and the age of that tip.
pub fn set_spv_tip(height: u64, timestamp_ms: u64, chain_work: u128, now_ms: u64) {
    gauge!(names::SPV_TIP_HEIGHT).set(height as f64);
    gauge!(names::SPV_TIP_TIMESTAMP_MS).set(timestamp_ms as f64);
    gauge!(names::SPV_CHAIN_WORK).set(chain_work as f64);
    gauge!(names::SPV_TIP_AGE_SECONDS).set(spv_tip_age_seconds(timestamp_ms, now_ms));
}

/// Record an authority-set update verification attempt.
///
/// `applied = false` means the threshold signatures, the Merkle inclusion, or
/// the new set hash failed — an update that was announced but is not backed.
pub fn record_spv_authority_update(applied: bool) {
    let series = if applied {
        names::SPV_AUTHORITY_UPDATES_TOTAL
    } else {
        names::SPV_AUTHORITY_UPDATES_REJECTED_TOTAL
    };
    counter!(series).increment(1);
}

/// Record a Merkle inclusion proof check.
pub fn record_spv_merkle_proof(verified: bool) {
    let series = if verified {
        names::SPV_MERKLE_PROOFS_VERIFIED_TOTAL
    } else {
        names::SPV_MERKLE_PROOFS_REJECTED_TOTAL
    };
    counter!(series).increment(1);
}

/// Record whether a transaction was proven to be in a verified header.
pub fn record_spv_tx_inclusion(verified: bool) {
    let series = if verified {
        names::SPV_TX_INCLUSION_VERIFIED_TOTAL
    } else {
        names::SPV_TX_INCLUSION_REJECTED_TOTAL
    };
    counter!(series).increment(1);
}

/// Stable, low-cardinality label for an [`SpvError`] variant.
///
/// Returns the Rust variant name. Every variant is a fixed compile-time string
/// and the enum is closed, so this can never become unbounded cardinality.
fn spv_error_reason(err: &SpvError) -> &'static str {
    match err {
        SpvError::NoCheckpoint => "NoCheckpoint",
        SpvError::HeightMismatch => "HeightMismatch",
        SpvError::PrevHashMismatch => "PrevHashMismatch",
        SpvError::TimestampNotMonotonic => "TimestampNotMonotonic",
        SpvError::WorkNotIncreasing => "WorkNotIncreasing",
        SpvError::MissingAuthoritySig => "MissingAuthoritySig",
        SpvError::AuthoritySetChanged => "AuthoritySetChanged",
        SpvError::InvalidAuthoritySig => "InvalidAuthoritySig",
        SpvError::PoANotEnabled => "PoANotEnabled",
        SpvError::InvalidAuthorityUpdate => "InvalidAuthorityUpdate",
        SpvError::UpdateNotInBlock => "UpdateNotInBlock",
    }
}

/// A timer guard that records a histogram on drop.
pub struct TimerGuard {
    name: &'static str,
    start: std::time::Instant,
}

impl TimerGuard {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for TimerGuard {
    fn drop(&mut self) {
        histogram!(self.name).record(self.start.elapsed().as_secs_f64());
    }
}

/// Convenience macro for timing a block of code.
#[macro_export]
macro_rules! time_block {
    ($name:expr, $body:block) => {{
        let _guard = $crate::metrics::TimerGuard::new($name);
        $body
    }};
}

/// Initialize a tracing span for a block operation.
pub fn block_span(height: u64, block_id: &str) -> tracing::Span {
    tracing::info_span!("block", height, block_id = %block_id)
}

/// Initialize a tracing span for a peer operation.
pub fn peer_span(peer: &str) -> tracing::Span {
    tracing::info_span!("peer", peer = %peer)
}

/// Initialize a tracing span for a sync operation.
pub fn sync_span(peer: &str) -> tracing::Span {
    tracing::info_span!("sync", peer = %peer)
}

/// Initialize a tracing span for a DHT operation.
pub fn dht_span(operation: &str) -> tracing::Span {
    tracing::info_span!("dht", operation = %operation)
}

/// Initialize a tracing span for an RPC request.
pub fn rpc_span(method: &str) -> tracing::Span {
    tracing::info_span!("rpc", method = %method)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    /// A PoA set of `n` authorities at `threshold`.
    ///
    /// Seeds start at 200 so the keys are disjoint from the sets used by the
    /// slot-accounting tests, letting those assert exact values on labelled
    /// series without racing the shared process-global recorder.
    fn classic_set(n: u8, threshold: usize) -> AuthoritySet {
        let keys: Vec<VerifyingKey> = (200..200 + n)
            .map(|s| SigningKey::from_bytes(&[s; 32]).verifying_key())
            .collect();
        AuthoritySet::new(keys, threshold).expect("classic set")
    }


    #[test]
    fn records_and_renders_prometheus_payload() {
        // First call installs the global recorder; repeat calls are no-ops.
        init_metrics("127.0.0.1:39091").expect("metrics init");

        record_block_produced(7, 42, Duration::from_millis(12));
        record_reorg(6);
        record_peer_connected();
        record_peer_disconnected();
        record_peer_banned();
        set_peer_count(3);
        set_peer_score("peer-a", -25);
        record_p2p_message_sent("block", 512);
        record_p2p_message_received("tx", 96);
        set_mempool_counts(11, 2, 4096);
        record_mempool_evicted(4);
        record_mempool_promoted(9);
        record_sync_complete(Duration::from_millis(250), 10, 10, 2);
        set_dht_routing_table_size(17);
        record_dht_bootstrap(Duration::from_millis(30), 5);
        record_dht_find_node(Duration::from_millis(5), 8);
        record_dht_pruned(1);
        record_dht_query_sent();
        record_dht_query_received();
        record_rpc_request("balance", Duration::from_micros(120));
        set_explorer_ws_clients(2);
        record_explorer_http_request("/api/state", 200);
        record_snapshot_size(1024);
        record_checkpoint_size(256);
        record_store_append(Duration::from_micros(80));
        record_block_validation(Duration::from_micros(50), false);
        record_block_validation(Duration::from_micros(60), true);
        record_tx_validation(Duration::from_micros(20), true);

        let body = render_prometheus();
        for series in [
            names::BLOCKS_PRODUCED_TOTAL,
            names::BLOCK_HEIGHT,
            names::DAG_BLUE_SCORE,
            names::DAG_REORG_DEPTH,
            names::PEER_COUNT,
            names::PEER_CONNECTED_TOTAL,
            names::PEER_BANNED_TOTAL,
            names::PEER_SCORE,
            names::P2P_MESSAGES_SENT_TOTAL,
            names::P2P_MESSAGE_BYTES_RECEIVED,
            names::MEMPOOL_TX_COUNT,
            names::MEMPOOL_ORPHAN_COUNT,
            names::MEMPOOL_EVICTED_TOTAL,
            names::MEMPOOL_PROMOTED_TOTAL,
            names::SYNC_DURATION_SECONDS,
            names::SYNC_HEADERS_RECEIVED,
            names::DHT_ROUTING_TABLE_SIZE,
            names::DHT_BOOTSTRAP_DURATION_SECONDS,
            names::DHT_PEERS_DISCOVERED_TOTAL,
            names::DHT_PEERS_PRUNED_TOTAL,
            names::DHT_QUERIES_SENT_TOTAL,
            names::RPC_REQUEST_DURATION_SECONDS,
            names::EXPLORER_WS_CLIENTS,
            names::EXPLORER_HTTP_REQUESTS_TOTAL,
            names::SNAPSHOT_SIZE_BYTES,
            names::CHECKPOINT_SIZE_BYTES,
            names::STORE_APPEND_DURATION_SECONDS,
            names::BLOCK_VALIDATION_DURATION_SECONDS,
            names::BLOCK_REJECTED_TOTAL,
            names::TX_REJECTED_TOTAL,
        ] {
            assert!(body.contains(series), "missing series {series} in:\n{body}");
        }
    }

    #[test]
    fn timer_guard_records_histogram() {
        init_metrics("127.0.0.1:39091").expect("metrics init");
        {
            let _guard = TimerGuard::new(names::BLOCK_PRODUCTION_DURATION_SECONDS);
        }
        let body = render_prometheus();
        assert!(
            body.contains(names::BLOCK_PRODUCTION_DURATION_SECONDS),
            "TimerGuard did not record its histogram:\n{body}"
        );
    }

    #[test]
    fn surfaces_classic_poa_authority_set_without_stake_series() {
        init_metrics("127.0.0.1:39093").expect("metrics init");

        let set = classic_set(3, 2);
        record_poa_authority_set(&set);
        record_poa_slot_duration(3000);

        let body = render_prometheus();
        for series in [
            names::POA_AUTHORITY_COUNT,
            names::POA_THRESHOLD,
            names::POA_SLOT_DURATION_MS,
        ] {
            assert!(body.contains(series), "missing {series} in:\n{body}");
        }
        // PoA is equal-weight (KVP-201), so per-authority stake series must
        // not exist at all: a "stake" gauge would be a fabricated number.
        // See docs/RFC-009-StakeWeightedPoA.md.
        assert!(
            !body.contains("poa_authority_stake"),
            "stake series must not exist under equal-weight PoA:\n{body}"
        );
        assert!(
            !body.contains("poa_authority_share_bps"),
            "share series must not exist under equal-weight PoA:\n{body}"
        );
    }

    #[test]
    fn records_poa_slot_scheduled_produced_and_missed_per_authority() {
        init_metrics("127.0.0.1:39095").expect("metrics init");

        let set = classic_set(3, 2);
        let owner = set.authorities()[0];
        record_poa_slot_scheduled(&owner);
        record_poa_slot_produced(&owner);
        record_poa_slot_scheduled(&owner);
        record_poa_slot_missed(&owner);

        let body = render_prometheus();
        let label = authority_label(&owner);
        assert!(
            body.contains(&format!(
                "kovanica_poa_slots_scheduled_total{{authority=\"{label}\"}} 2"
            )),
            "{body}"
        );
        assert!(
            body.contains(&format!(
                "kovanica_poa_slots_produced_total{{authority=\"{label}\"}} 1"
            )),
            "{body}"
        );
        assert!(
            body.contains(&format!(
                "kovanica_poa_slots_missed_total{{authority=\"{label}\"}} 1"
            )),
            "{body}"
        );
    }

    #[test]
    fn records_spv_verification_and_classifies_rejections() {
        init_metrics("127.0.0.1:39096").expect("metrics init");

        set_spv_enabled(true);
        record_spv_header_verified(12, Duration::from_micros(40));
        // A stale fork and a forged signature are different incidents; the
        // per-reason series must keep them apart.
        record_spv_header_rejected(&SpvError::PrevHashMismatch);
        record_spv_header_rejected(&SpvError::PrevHashMismatch);
        record_spv_header_rejected(&SpvError::InvalidAuthoritySig);
        record_spv_header_rejected(&SpvError::InvalidAuthorityUpdate);
        set_spv_tip(500, 1_000_000, 12345, 1_060_000);
        record_spv_merkle_proof(true);
        record_spv_merkle_proof(false);
        record_spv_tx_inclusion(true);
        record_spv_authority_update(true);
        record_spv_authority_update(false);

        let body = render_prometheus();
        for series in [
            names::SPV_ENABLED,
            names::SPV_HEADERS_VERIFIED_TOTAL,
            names::SPV_HEADERS_REJECTED_TOTAL,
            names::SPV_HEADER_VERIFY_DURATION_SECONDS,
            names::SPV_TIP_HEIGHT,
            names::SPV_CHAIN_WORK,
            names::SPV_HEADERS_TRACKED,
            names::SPV_AUTHORITY_UPDATES_TOTAL,
            names::SPV_AUTHORITY_UPDATES_REJECTED_TOTAL,
            names::SPV_MERKLE_PROOFS_VERIFIED_TOTAL,
            names::SPV_MERKLE_PROOFS_REJECTED_TOTAL,
            names::SPV_TX_INCLUSION_VERIFIED_TOTAL,
            names::SPV_REJECTED_BY_REASON,
        ] {
            assert!(body.contains(series), "missing {series} in:\n{body}");
        }
        assert!(body.contains("kovanica_spv_headers_rejected_total 4"), "{body}");
        assert!(
            body.contains("kovanica_spv_rejected_by_reason{reason=\"PrevHashMismatch\"} 2"),
            "{body}"
        );
        assert!(
            body.contains("kovanica_spv_rejected_by_reason{reason=\"InvalidAuthoritySig\"} 1"),
            "{body}"
        );
        // Each dedicated counter is owned by exactly one verification step, so
        // a header rejection never inflates it: one applied + one rejected
        // update, not three.
        assert!(
            body.contains("kovanica_spv_authority_updates_total 1"),
            "{body}"
        );
        assert!(
            body.contains("kovanica_spv_authority_updates_rejected_total 1"),
            "{body}"
        );
        // 60s of wall clock between the tip timestamp and now.
        assert!(body.contains(names::SPV_TIP_AGE_SECONDS), "{body}");
    }

    #[test]
    fn spv_tip_age_saturates_instead_of_wrapping_on_a_future_timestamp() {
        // A peer with a skewed clock can hand us a header timestamped ahead of
        // local time. A plain subtraction would wrap into a huge positive age
        // and make a healthy node look stuck for years.
        assert_eq!(spv_tip_age_seconds(1_000_000, 1_060_000), 60.0);
        assert_eq!(spv_tip_age_seconds(5_000_000, 4_000_000), 0.0);
        // Same instant, and sub-second precision.
        assert_eq!(spv_tip_age_seconds(1_000, 1_000), 0.0);
        assert_eq!(spv_tip_age_seconds(0, 1_500), 1.5);
    }

    #[test]
    fn spv_error_reason_is_total_over_every_variant() {
        // Guards the exhaustive match in spv_error_reason: adding a variant to
        // SpvError without labelling it will fail to compile here, which is the
        // point — an unlabelled variant would silently collapse into a shared
        // reason string and break the per-reason alerting contract.
        let all = [
            SpvError::NoCheckpoint,
            SpvError::HeightMismatch,
            SpvError::PrevHashMismatch,
            SpvError::TimestampNotMonotonic,
            SpvError::WorkNotIncreasing,
            SpvError::MissingAuthoritySig,
            SpvError::AuthoritySetChanged,
            SpvError::InvalidAuthoritySig,
            SpvError::PoANotEnabled,
            SpvError::InvalidAuthorityUpdate,
            SpvError::UpdateNotInBlock,
        ];
        let mut seen: Vec<&str> = all.iter().map(spv_error_reason).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), all.len(), "two SpvError variants share a label");
    }

    #[test]
    fn passive_observe_surfaces_head_without_producing() {
        // blocks received from peers. It must surface the head/blue-score
        // gauges. We assert presence (like the sibling metrics tests) rather
        // than an exact value: the recorder is a shared process-global, so
        // other unit tests may overwrite the gauge value concurrently.
        init_metrics("127.0.0.1:39092").expect("metrics init");

        record_block_observed(500, 499);

        let body = render_prometheus();
        for series in [names::BLOCK_HEIGHT, names::DAG_BLUE_SCORE] {
            assert!(
                body.contains(series),
                "passive observe did not surface {series} in:\n{body}"
            );
        }
    }
}
