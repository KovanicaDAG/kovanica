//! Standalone multithreaded solo miner for Kovanica PoW (Chain 5).
//!
//! Connects to any Kovanica node JSON-RPC endpoint, polls candidate block templates
//! via `kovanica_getWork`, partitions the nonce space across CPU worker threads using
//! preallocated 2 MiB `KovanicaHasher` contexts, computes KovanicaHash PoW solutions,
//! and submits solved headers via `kovanica_submitWork`.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Barrier, RwLock};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use kovanica_crypto::KovanicaHasher;
use kovanica_types::block::{
    check_pow_hash_meets_target, compact_to_target, BlockHeader, BlockReference,
};
use kovanica_types::signing::chain_id::HYBRID_TESTNET;
use kovanica_types::Address;
use kovanica_types::Hash;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::rpc::RpcClient;

/// Arguments for `kovanica miner run`.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct MinerRunArgs {
    /// Payout address for mining rewards (Chain 5 canonical base32 or legacy hex).
    #[arg(long, short)]
    pub address: String,

    /// Number of mining worker threads (defaults to available logical CPU cores).
    #[arg(long, short)]
    pub threads: Option<usize>,

    /// Polling interval in milliseconds for candidate block templates.
    #[arg(long, default_value = "1000")]
    pub poll_interval_ms: u64,
}

/// Arguments for `kovanica miner bench`.
#[derive(Args, Debug, Clone, PartialEq, Eq)]
pub struct MinerBenchArgs {
    /// Number of benchmarking worker threads (defaults to available logical CPU cores).
    #[arg(long, short)]
    pub threads: Option<usize>,

    /// Duration in seconds to run the benchmark.
    #[arg(long, short, default_value = "5")]
    pub duration: u64,
}

/// Miner subcommands.
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum MinerCommand {
    /// Run the standalone multithreaded solo miner.
    Run(MinerRunArgs),
    /// Benchmark local KovanicaHash PoW hashrate across CPU threads.
    Bench(MinerBenchArgs),
}

/// Validate the payout address for the given chain ID.
///
/// On Chain 5 (`HYBRID_TESTNET`), both canonical base32 (`kvnc...dag`)
/// and valid legacy hex forms are accepted.
pub fn validate_payout_address(address_str: &str, chain_id: u64) -> Result<Address> {
    Address::parse_for_chain(address_str.trim(), chain_id)
        .with_context(|| format!("invalid payout address '{address_str}' for chain {chain_id}"))
}

/// JSON-RPC work template returned by `kovanica_getWork`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkTemplate {
    pub chain_id: u64,
    pub round: u64,
    pub author: u16,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub parent_refs: Vec<BlockReference>,
    pub merkle_root: String,
    pub timestamp: u64,
    pub target_bits: Value,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub preimage: Option<String>,
    #[serde(default)]
    pub header: Option<BlockHeader>,
}

impl WorkTemplate {
    /// Parse `target_bits` from either a hex string (e.g. `"0x207fffff"`) or a number.
    pub fn parse_target_bits(&self) -> Result<u32> {
        match &self.target_bits {
            Value::Number(n) => n
                .as_u64()
                .map(|v| v as u32)
                .context("target_bits integer overflow"),
            Value::String(s) => {
                let stripped = s
                    .trim()
                    .strip_prefix("0x")
                    .or_else(|| s.trim().strip_prefix("0X"))
                    .unwrap_or(s.trim());
                u32::from_str_radix(stripped, 16)
                    .with_context(|| format!("invalid hex target_bits: {s}"))
            }
            other => anyhow::bail!("unexpected target_bits JSON value: {other:?}"),
        }
    }

    /// Construct a candidate `BlockHeader` from this work template.
    pub fn to_candidate_header(&self) -> Result<BlockHeader> {
        if let Some(h) = &self.header {
            return Ok(h.clone());
        }

        let target_bits = self.parse_target_bits()?;
        let merkle_root_clean = self
            .merkle_root
            .trim()
            .strip_prefix("0x")
            .or_else(|| self.merkle_root.trim().strip_prefix("0X"))
            .unwrap_or(self.merkle_root.trim());
        let merkle_bytes =
            hex::decode(merkle_root_clean).context("invalid merkle_root hex string")?;
        if merkle_bytes.len() != 32 {
            anyhow::bail!(
                "invalid merkle_root byte length: {} (expected 32)",
                merkle_bytes.len()
            );
        }
        let mut merkle_arr = [0u8; 32];
        merkle_arr.copy_from_slice(&merkle_bytes);

        let parent_refs = if !self.parent_refs.is_empty() {
            self.parent_refs.clone()
        } else {
            let mut refs = Vec::new();
            for p_hex in &self.parents {
                let clean = p_hex
                    .trim()
                    .strip_prefix("0x")
                    .or_else(|| p_hex.trim().strip_prefix("0X"))
                    .unwrap_or(p_hex.trim());
                let bytes = hex::decode(clean).context("invalid parent hash hex")?;
                if bytes.len() != 32 {
                    anyhow::bail!(
                        "invalid parent hash byte length: {} (expected 32)",
                        bytes.len()
                    );
                }
                let mut digest = [0u8; 32];
                digest.copy_from_slice(&bytes);
                refs.push(BlockReference {
                    author: 0,
                    round: self.round.saturating_sub(1),
                    digest: Hash(digest),
                });
            }
            refs
        };

        Ok(BlockHeader::new(
            self.chain_id,
            self.round,
            self.author,
            parent_refs,
            Hash(merkle_arr),
            self.timestamp,
            target_bits,
            0,
        ))
    }
}

/// Real-time statistics tracked during mining.
#[derive(Debug, Default)]
pub struct MiningStats {
    pub total_hashes: AtomicU64,
    pub blocks_found: AtomicU64,
    pub blocks_accepted: AtomicU64,
    pub blocks_rejected: AtomicU64,
}

impl MiningStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total_hashes(&self) -> u64 {
        self.total_hashes.load(Ordering::Relaxed)
    }

    pub fn blocks_found(&self) -> u64 {
        self.blocks_found.load(Ordering::Relaxed)
    }

    pub fn blocks_accepted(&self) -> u64 {
        self.blocks_accepted.load(Ordering::Relaxed)
    }

    pub fn blocks_rejected(&self) -> u64 {
        self.blocks_rejected.load(Ordering::Relaxed)
    }
}

/// Format a hashrate into human-readable units (H/s, kH/s, MH/s, GH/s).
pub fn format_hashrate(hashes: u64, duration_secs: f64) -> String {
    if duration_secs <= 0.0 {
        return "0.00 H/s".to_string();
    }
    let hps = (hashes as f64) / duration_secs;
    if hps >= 1_000_000_000.0 {
        format!("{:.2} GH/s", hps / 1_000_000_000.0)
    } else if hps >= 1_000_000.0 {
        format!("{:.2} MH/s", hps / 1_000_000.0)
    } else if hps >= 1_000.0 {
        format!("{:.2} kH/s", hps / 1_000.0)
    } else {
        format!("{:.2} H/s", hps)
    }
}

/// Active work shared with worker threads.
struct ActiveWork {
    version: u64,
    header: BlockHeader,
}

/// Multithreaded solo miner controller.
pub struct SoloMiner {
    client: RpcClient,
    payout_address: String,
    chain_id: u64,
    threads: usize,
    poll_interval: Duration,
    stats: Arc<MiningStats>,
    shutdown: Arc<AtomicBool>,
    shutdown_notify: Arc<tokio::sync::Notify>,
}

impl SoloMiner {
    /// Create a new solo miner instance.
    pub fn new(
        client: RpcClient,
        payout_address: String,
        chain_id: u64,
        threads: usize,
        poll_interval_ms: u64,
    ) -> Self {
        let threads = if threads == 0 {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        } else {
            threads
        };

        Self {
            client,
            payout_address,
            chain_id,
            threads,
            poll_interval: Duration::from_millis(poll_interval_ms.max(50)),
            stats: Arc::new(MiningStats::new()),
            shutdown: Arc::new(AtomicBool::new(false)),
            shutdown_notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Access stats.
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub fn stats(&self) -> Arc<MiningStats> {
        self.stats.clone()
    }

    /// Shutdown flag.
    pub fn shutdown_handle(&self) -> Arc<AtomicBool> {
        self.shutdown.clone()
    }

    /// Stop mining gracefully.
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.shutdown_notify.notify_waiters();
    }

    /// Run the solo mining loop until Ctrl-C or external shutdown.
    pub async fn run(&self) -> Result<()> {
        let start_time = Instant::now();
        let active_work: Arc<RwLock<Option<ActiveWork>>> = Arc::new(RwLock::new(None));
        let work_version = Arc::new(AtomicU64::new(0));

        let (solution_tx, mut solution_rx) = mpsc::unbounded_channel::<BlockHeader>();

        // Spawn multithreaded CPU hashing workers
        let mut worker_handles = Vec::with_capacity(self.threads);
        for thread_idx in 0..self.threads {
            let active_work_clone = active_work.clone();
            let work_version_clone = work_version.clone();
            let shutdown_clone = self.shutdown.clone();
            let stats_clone = self.stats.clone();
            let solution_tx_clone = solution_tx.clone();
            let total_threads = self.threads;

            let handle = std::thread::Builder::new()
                .name(format!("kovanica-miner-{thread_idx}"))
                .spawn(move || {
                    let mut hasher = KovanicaHasher::new();
                    let mut local_version = 0u64;
                    let mut current_header: Option<BlockHeader> = None;
                    let mut preimage_buf: Vec<u8> = Vec::new();
                    let mut nonce: u64 = thread_idx as u64;
                    let stride = total_threads as u64;

                    while !shutdown_clone.load(Ordering::Relaxed) {
                        let global_version = work_version_clone.load(Ordering::Relaxed);
                        if global_version != local_version {
                            let guard = active_work_clone.read().unwrap();
                            if let Some(work) = guard.as_ref() {
                                if work.version == global_version {
                                    current_header = Some(work.header.clone());
                                    local_version = global_version;
                                    nonce = thread_idx as u64;
                                    preimage_buf = work.header.preimage();
                                }
                            }
                        }

                        let Some(header) = &current_header else {
                            std::thread::sleep(Duration::from_millis(20));
                            continue;
                        };

                        let buf_len = preimage_buf.len();
                        if buf_len < 8 {
                            std::thread::sleep(Duration::from_millis(20));
                            continue;
                        }

                        let target_bits = header.target_bits;
                        let target = compact_to_target(target_bits);
                        if target == [0u8; 32] {
                            std::thread::sleep(Duration::from_millis(50));
                            continue;
                        }

                        // Evaluate a chunk of nonces
                        const CHUNK_SIZE: u64 = 250;
                        let mut found = false;
                        for _ in 0..CHUNK_SIZE {
                            preimage_buf[buf_len - 8..].copy_from_slice(&nonce.to_le_bytes());
                            let hash = hasher.hash(&preimage_buf);

                            if check_pow_hash_meets_target(&hash, target_bits) {
                                let mut solved = header.clone();
                                solved.nonce = nonce;
                                solved.pow_hash = Some(hash);
                                let _ = solution_tx_clone.send(solved);
                                found = true;
                                nonce = nonce.wrapping_add(stride);
                                break;
                            }
                            nonce = nonce.wrapping_add(stride);
                        }

                        stats_clone
                            .total_hashes
                            .fetch_add(CHUNK_SIZE, Ordering::Relaxed);

                        if found {
                            // After finding a solution, pause briefly to let poller refresh
                            std::thread::sleep(Duration::from_millis(50));
                        }
                    }
                })
                .context("failed to spawn mining worker thread")?;

            worker_handles.push(handle);
        }

        println!("============================================================");
        println!("       Kovanica Standalone Solo CPU Miner (Chain 5)         ");
        println!("============================================================");
        println!(" Chain ID:        {}", self.chain_id);
        println!(" RPC Endpoint:    {}", self.client.url());
        println!(" Payout Address:  {}", self.payout_address);
        println!(" Worker Threads:  {}", self.threads);
        println!(" Poll Interval:   {} ms", self.poll_interval.as_millis());
        println!(" PoW Hash:        KovanicaHash (BLAKE3 + 2 MiB mix)");
        println!(" Press Ctrl-C to stop mining gracefully.");
        println!("============================================================");

        let mut last_display = Instant::now();
        let mut last_hashes = 0u64;
        let mut current_target_bits = 0u32;
        let mut current_round = 0u64;

        // Polling & event loop
        loop {
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }

            // 1. Check for incoming solutions from worker threads
            while let Ok(solved_header) = solution_rx.try_recv() {
                self.stats.blocks_found.fetch_add(1, Ordering::Relaxed);
                println!(
                    "\n[MINER] Solved block! Round: {} Nonce: {} PoW Hash: {}",
                    solved_header.round,
                    solved_header.nonce,
                    solved_header
                        .pow_hash
                        .map(|h| format!("0x{}", hex::encode(h.0)))
                        .unwrap_or_else(|| "none".to_string())
                );

                // Immediately submit via RPC
                match self
                    .client
                    .call("kovanica_submitWork", json!({ "header": solved_header }))
                    .await
                {
                    Ok(resp) => {
                        let accepted = resp
                            .get("accepted")
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        if accepted {
                            self.stats.blocks_accepted.fetch_add(1, Ordering::Relaxed);
                            let digest = resp
                                .get("digest")
                                .and_then(Value::as_str)
                                .unwrap_or("unknown");
                            println!(
                                "[MINER] Block #{} ACCEPTED by node! Digest: {}",
                                solved_header.round, digest
                            );
                        } else {
                            self.stats.blocks_rejected.fetch_add(1, Ordering::Relaxed);
                            println!(
                                "[MINER] Block #{} REJECTED: {:?}",
                                solved_header.round, resp
                            );
                        }
                    }
                    Err(e) => {
                        self.stats.blocks_rejected.fetch_add(1, Ordering::Relaxed);
                        error!(error = %e, round = solved_header.round, "kovanica_submitWork failed");
                        println!(
                            "[MINER] Block #{} SUBMISSION ERROR: {e:#}",
                            solved_header.round
                        );
                    }
                }
            }

            // 2. Poll fresh work template
            match self.fetch_work().await {
                Ok(template) => {
                    let candidate = match template.to_candidate_header() {
                        Ok(h) => h,
                        Err(e) => {
                            warn!(error = %e, "failed to build candidate header from template");
                            continue;
                        }
                    };

                    current_target_bits = candidate.target_bits;
                    current_round = candidate.round;

                    let mut guard = active_work.write().unwrap();
                    let need_update = match guard.as_ref() {
                        None => true,
                        Some(current) => {
                            current.header.round != candidate.round
                                || current.header.parents != candidate.parents
                                || current.header.target_bits != candidate.target_bits
                                || current.header.merkle_root != candidate.merkle_root
                        }
                    };

                    if need_update {
                        let next_version = work_version.fetch_add(1, Ordering::Relaxed) + 1;
                        *guard = Some(ActiveWork {
                            version: next_version,
                            header: candidate,
                        });
                    }
                }
                Err(e) => {
                    warn!(error = %e, "kovanica_getWork RPC failed");
                }
            }

            // 3. Live terminal metrics observability update (every 1s)
            let now = Instant::now();
            let elapsed_since_display = now.duration_since(last_display).as_secs_f64();
            if elapsed_since_display >= 1.0 {
                let total_h = self.stats.total_hashes();
                let delta_h = total_h.saturating_sub(last_hashes);
                let current_rate = format_hashrate(delta_h, elapsed_since_display);

                let found = self.stats.blocks_found();
                let acc = self.stats.blocks_accepted();
                let rej = self.stats.blocks_rejected();

                print!(
                    "\r[MINER] Rate: {:>10} | Hashes: {:>9} | Blocks: {} ({} acc / {} rej) | Round: {:>3} | Target: 0x{:08x}",
                    current_rate,
                    total_h,
                    found,
                    acc,
                    rej,
                    current_round,
                    current_target_bits
                );
                use std::io::Write;
                let _ = std::io::stdout().flush();

                last_display = now;
                last_hashes = total_h;
            }

            // Sleep for poll interval or until shutdown
            tokio::select! {
                _ = tokio::time::sleep(self.poll_interval) => {}
                _ = self.shutdown_notify.notified() => {
                    break;
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Ctrl-C received, shutting down solo miner");
                    println!("\n[MINER] Shutdown requested; stopping worker threads...");
                    self.stop();
                    break;
                }
            }
        }

        // Wait for workers to finish
        self.shutdown.store(true, Ordering::SeqCst);
        for handle in worker_handles {
            let _ = handle.join();
        }

        let total_duration = start_time.elapsed().as_secs_f64();
        let total_h = self.stats.total_hashes();
        let avg_rate = format_hashrate(total_h, total_duration);

        println!("\n============================================================");
        println!("             Kovanica Solo Miner Final Statistics           ");
        println!("============================================================");
        println!(" Payout Address:   {}", self.payout_address);
        println!(" Worker Threads:   {}", self.threads);
        println!(" Total Duration:   {:.2} s", total_duration);
        println!(" Total Hashes:     {}", total_h);
        println!(" Average Hashrate: {}", avg_rate);
        println!(" Blocks Found:     {}", self.stats.blocks_found());
        println!(" Blocks Accepted:  {}", self.stats.blocks_accepted());
        println!(" Blocks Rejected:  {}", self.stats.blocks_rejected());
        println!("============================================================");

        Ok(())
    }

    /// Fetch candidate work template via `kovanica_getWork`.
    pub async fn fetch_work(&self) -> Result<WorkTemplate> {
        let res = self
            .client
            .call("kovanica_getWork", json!([self.payout_address]))
            .await
            .context("failed to query kovanica_getWork")?;

        serde_json::from_value(res).context("failed to deserialize WorkTemplate")
    }
}

/// Entry point called by `kovanica miner run`.
pub async fn run_miner(
    client: &RpcClient,
    _rpc_url: &str,
    args: MinerRunArgs,
    chain_id_opt: Option<u64>,
) -> Result<()> {
    let chain_id = chain_id_opt.unwrap_or(HYBRID_TESTNET);
    let _address = validate_payout_address(&args.address, chain_id)?;

    let threads = args.threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    });

    let miner = SoloMiner::new(
        client.clone(),
        args.address,
        chain_id,
        threads,
        args.poll_interval_ms,
    );

    miner.run().await
}

/// Summary metrics of a KovanicaHash PoW benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MinerBenchResult {
    /// Name and specification of the PoW algorithm.
    pub algorithm: String,
    /// Number of worker threads utilized.
    pub threads: usize,
    /// Requested duration in seconds.
    pub target_duration_secs: u64,
    /// Actual measured elapsed time in seconds.
    pub elapsed_secs: f64,
    /// Total KovanicaHash evaluations computed across all threads.
    pub total_hashes: u64,
    /// Aggregate hashrate in hashes per second.
    pub hashrate_hps: f64,
    /// Human-readable formatted hashrate string.
    pub hashrate_formatted: String,
    /// Average hashrate per worker thread in hashes per second.
    pub per_thread_hps: f64,
    /// Hashes computed by each individual worker thread.
    pub thread_hashes: Vec<u64>,
}

/// Execute a multithreaded KovanicaHash benchmark for the specified duration.
///
/// Each worker thread initializes an independent `KovanicaHasher` (allocating a 2 MiB scratchpad)
/// and hashes preimages constructed from a realistic block header with unique nonces/strides.
/// Timing begins after all threads have initialized and synchronized at a barrier.
pub fn benchmark_kovanica_hash(threads: usize, duration: Duration) -> Result<MinerBenchResult> {
    if duration.is_zero() {
        anyhow::bail!("benchmark duration must be greater than zero");
    }
    let threads = if threads == 0 {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    } else {
        threads
    };
    if threads > 256 {
        anyhow::bail!("requested thread count ({threads}) exceeds maximum allowed (256)");
    }

    // Realistic canonical block header template (Chain ID 5, round 1, 1 parent)
    let dummy_header = BlockHeader::new(
        HYBRID_TESTNET,
        1,
        0,
        vec![BlockReference {
            author: 0,
            round: 0,
            digest: Hash([0x42; 32]),
        }],
        Hash([0x37; 32]),
        1_700_000_000,
        0x207fffff,
        0,
    );
    let base_preimage = dummy_header.preimage();
    if base_preimage.len() < 8 {
        anyhow::bail!("block header preimage too short for nonce injection");
    }

    let barrier = Arc::new(Barrier::new(threads + 1));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let total_hashes = Arc::new(AtomicU64::new(0));

    let mut thread_counters = Vec::with_capacity(threads);
    let mut handles = Vec::with_capacity(threads);

    for thread_idx in 0..threads {
        let thread_hashes = Arc::new(AtomicU64::new(0));
        thread_counters.push(thread_hashes.clone());

        let barrier_clone = barrier.clone();
        let stop_clone = stop_flag.clone();
        let total_hashes_clone = total_hashes.clone();
        let mut preimage = base_preimage.clone();
        let buf_len = preimage.len();
        let stride = threads as u64;

        let handle = std::thread::Builder::new()
            .name(format!("kovanica-bench-{thread_idx}"))
            .spawn(move || {
                // Initialize hasher and allocate 2 MiB scratchpad BEFORE starting timer
                let mut hasher = KovanicaHasher::new();
                let mut nonce = thread_idx as u64;

                // Synchronize with coordinator and other workers
                barrier_clone.wait();

                while !stop_clone.load(Ordering::Relaxed) {
                    preimage[buf_len - 8..].copy_from_slice(&nonce.to_le_bytes());
                    let hash = hasher.hash(&preimage);
                    std::hint::black_box(hash);

                    thread_hashes.fetch_add(1, Ordering::Relaxed);
                    total_hashes_clone.fetch_add(1, Ordering::Relaxed);

                    nonce = nonce.wrapping_add(stride);
                }
            })
            .context("failed to spawn benchmarking worker thread")?;

        handles.push(handle);
    }

    // Wait for all workers to complete scratchpad allocation and reach barrier
    barrier.wait();
    let start = Instant::now();

    // Sleep for target benchmark duration
    std::thread::sleep(duration);
    stop_flag.store(true, Ordering::SeqCst);

    // Wait for all workers to finish and join
    for handle in handles {
        handle
            .join()
            .map_err(|_| anyhow::anyhow!("benchmarking worker thread panicked"))?;
    }

    let elapsed_secs = start.elapsed().as_secs_f64();
    let total_h = total_hashes.load(Ordering::Relaxed);
    let per_thread_counts: Vec<u64> = thread_counters
        .iter()
        .map(|c| c.load(Ordering::Relaxed))
        .collect();

    let hashrate_hps = if elapsed_secs > 0.0 {
        (total_h as f64) / elapsed_secs
    } else {
        0.0
    };
    let per_thread_hps = if threads > 0 {
        hashrate_hps / (threads as f64)
    } else {
        0.0
    };
    let hashrate_formatted = format_hashrate(total_h, elapsed_secs);

    Ok(MinerBenchResult {
        algorithm: "KovanicaHash (BLAKE3 + 2 MiB mix)".to_string(),
        threads,
        target_duration_secs: duration.as_secs(),
        elapsed_secs,
        total_hashes: total_h,
        hashrate_hps,
        hashrate_formatted,
        per_thread_hps,
        thread_hashes: per_thread_counts,
    })
}

/// Entry point called by `kovanica miner bench`.
///
/// Benchmarks local CPU KovanicaHash throughput standalone and offline without requiring RPC.
pub fn run_miner_bench(args: MinerBenchArgs, json: bool) -> Result<MinerBenchResult> {
    if args.duration == 0 {
        anyhow::bail!("benchmark duration must be at least 1 second");
    }
    let threads = args.threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    });
    if threads == 0 {
        anyhow::bail!("thread count must be at least 1");
    }
    if threads > 256 {
        anyhow::bail!("requested thread count ({threads}) exceeds maximum allowed (256)");
    }

    let duration = Duration::from_secs(args.duration);
    let result = benchmark_kovanica_hash(threads, duration)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("============================================================");
        println!("            KovanicaHash PoW Benchmark Results             ");
        println!("============================================================");
        println!(" Algorithm:          {}", result.algorithm);
        println!(" Worker Threads:     {}", result.threads);
        println!(" Target Duration:    {} s", result.target_duration_secs);
        println!(" Elapsed Time:       {:.3} s", result.elapsed_secs);
        println!(" Total Hashes:       {}", result.total_hashes);
        println!(
            " Total Hashrate:     {} ({:.2} H/s)",
            result.hashrate_formatted, result.hashrate_hps
        );
        println!(" Per-Thread Rate:    {:.2} H/s", result.per_thread_hps);
        println!("============================================================");
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::signing::chain_id;

    #[test]
    fn validate_payout_address_accepts_valid_base32() {
        let raw = Address([0x11; 32]);
        let encoded = raw.encode();
        assert_eq!(encoded.len(), 64);
        assert!(encoded.starts_with("kvnc"));
        assert!(encoded.ends_with("dag"));

        let res = validate_payout_address(&encoded, chain_id::HYBRID_TESTNET);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), raw);
    }

    #[test]
    fn validate_payout_address_accepts_legacy_hex_on_devnet() {
        let raw = Address([0x22; 32]);
        let hex_bare = raw.to_hex();
        let hex_0x = format!("0x{}", hex_bare);

        let res1 = validate_payout_address(&hex_bare, chain_id::DEVNET);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap(), raw);

        let res2 = validate_payout_address(&hex_0x, chain_id::DEVNET);
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap(), raw);
    }

    #[test]
    fn validate_payout_address_rejects_legacy_hex_on_mainnet_and_testnet() {
        let raw = Address([0x33; 32]);
        let hex_bare = raw.to_hex();

        // Chain 1 (MAINNET) and Chain 2 (TESTNET) strictly reject legacy hex addresses
        let res_main = validate_payout_address(&hex_bare, chain_id::MAINNET);
        assert!(res_main.is_err());
        let res_test = validate_payout_address(&hex_bare, chain_id::TESTNET);
        assert!(res_test.is_err());
    }

    #[test]
    fn validate_payout_address_rejects_malformed_address() {
        assert!(validate_payout_address("not_an_address", chain_id::HYBRID_TESTNET).is_err());
        assert!(validate_payout_address("kvnc12345dag", chain_id::HYBRID_TESTNET).is_err());
    }

    #[test]
    fn format_hashrate_formats_all_ranges() {
        assert_eq!(format_hashrate(500, 1.0), "500.00 H/s");
        assert_eq!(format_hashrate(15_000, 1.0), "15.00 kH/s");
        assert_eq!(format_hashrate(2_500_000, 1.0), "2.50 MH/s");
        assert_eq!(format_hashrate(4_000_000_000, 1.0), "4.00 GH/s");
        assert_eq!(format_hashrate(0, 0.0), "0.00 H/s");
    }

    #[test]
    fn work_template_parses_and_converts_to_candidate_header() {
        let template = WorkTemplate {
            chain_id: 5,
            round: 10,
            author: 3,
            parents: vec![format!("0x{}", hex::encode([0xaa; 32]))],
            parent_refs: vec![],
            merkle_root: format!("0x{}", hex::encode([0xbb; 32])),
            timestamp: 1_700_000_000_000,
            target_bits: json!("0x207fffff"),
            target: None,
            preimage: None,
            header: None,
        };

        let header = template.to_candidate_header().expect("should build header");
        assert_eq!(header.chain_id, 5);
        assert_eq!(header.round, 10);
        assert_eq!(header.author, 3);
        assert_eq!(header.target_bits, 0x207fffff);
        assert_eq!(header.merkle_root.0, [0xbb; 32]);
        assert_eq!(header.parents.len(), 1);
        assert_eq!(header.parents[0].digest.0, [0xaa; 32]);
    }

    #[test]
    fn preimage_buffer_last_8_bytes_matches_header_preimage_with_nonce() {
        let header = BlockHeader::new(
            5,
            1,
            0,
            vec![],
            kovanica_types::Hash([0x55; 32]),
            1_700_000_000,
            0x207fffff,
            0,
        );

        let mut buf = header.preimage();
        let len = buf.len();
        assert!(len >= 8);

        for test_nonce in [0u64, 1, 42, 999_999, u64::MAX] {
            buf[len - 8..].copy_from_slice(&test_nonce.to_le_bytes());

            let mut header_with_nonce = header.clone();
            header_with_nonce.nonce = test_nonce;
            assert_eq!(buf, header_with_nonce.preimage());
        }
    }

    #[test]
    fn worker_thread_finds_winning_nonce_on_easy_target() {
        let candidate = BlockHeader::new(
            5,
            1,
            0,
            vec![],
            kovanica_types::Hash([0x33; 32]),
            1_700_000_000,
            0x207fffff, // easy target
            0,
        );

        let active_work = Arc::new(RwLock::new(Some(ActiveWork {
            version: 1,
            header: candidate,
        })));
        let work_version = Arc::new(AtomicU64::new(1));
        let shutdown = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(MiningStats::new());
        let (solution_tx, mut solution_rx) = mpsc::unbounded_channel::<BlockHeader>();

        let active_work_clone = active_work.clone();
        let work_version_clone = work_version.clone();
        let shutdown_clone = shutdown.clone();
        let stats_clone = stats.clone();

        let handle = std::thread::spawn(move || {
            let mut hasher = KovanicaHasher::new();
            let mut local_version = 0u64;
            let mut current_header: Option<BlockHeader> = None;
            let mut preimage_buf: Vec<u8> = Vec::new();
            let mut nonce: u64 = 0;

            while !shutdown_clone.load(Ordering::Relaxed) {
                let gv = work_version_clone.load(Ordering::Relaxed);
                if gv != local_version {
                    let guard = active_work_clone.read().unwrap();
                    if let Some(w) = guard.as_ref() {
                        current_header = Some(w.header.clone());
                        local_version = gv;
                        preimage_buf = w.header.preimage();
                    }
                }

                let Some(header) = &current_header else {
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                };

                let buf_len = preimage_buf.len();
                preimage_buf[buf_len - 8..].copy_from_slice(&nonce.to_le_bytes());
                let hash = hasher.hash(&preimage_buf);

                if check_pow_hash_meets_target(&hash, header.target_bits) {
                    let mut solved = header.clone();
                    solved.nonce = nonce;
                    solved.pow_hash = Some(hash);
                    let _ = solution_tx.send(solved);
                    stats_clone.total_hashes.fetch_add(1, Ordering::Relaxed);
                    break;
                }

                stats_clone.total_hashes.fetch_add(1, Ordering::Relaxed);
                nonce = nonce.wrapping_add(1);
            }
        });

        // Solution must arrive quickly
        let solved = solution_rx.blocking_recv().expect("should find solution");
        assert!(solved.verify_pow().is_ok());
        assert!(solved.pow_hash.is_some());

        shutdown.store(true, Ordering::SeqCst);
        handle.join().unwrap();
        assert!(stats.total_hashes() > 0);
    }

    #[test]
    fn worker_thread_lifecycle_detects_stale_work_and_switches() {
        let initial_header = BlockHeader::new(
            5,
            1,
            0,
            vec![],
            kovanica_types::Hash([0x11; 32]),
            1_700_000_000,
            0x00000001, // extremely hard target so it won't solve
            0,
        );

        let active_work = Arc::new(RwLock::new(Some(ActiveWork {
            version: 1,
            header: initial_header,
        })));
        let work_version = Arc::new(AtomicU64::new(1));
        let shutdown = Arc::new(AtomicBool::new(false));
        let (solution_tx, mut solution_rx) = mpsc::unbounded_channel::<BlockHeader>();

        let active_work_clone = active_work.clone();
        let work_version_clone = work_version.clone();
        let shutdown_clone = shutdown.clone();

        let handle = std::thread::spawn(move || {
            let mut hasher = KovanicaHasher::new();
            let mut local_version = 0u64;
            let mut current_header: Option<BlockHeader> = None;
            let mut preimage_buf: Vec<u8> = Vec::new();
            let mut nonce: u64 = 0;

            while !shutdown_clone.load(Ordering::Relaxed) {
                let gv = work_version_clone.load(Ordering::Relaxed);
                if gv != local_version {
                    let guard = active_work_clone.read().unwrap();
                    if let Some(w) = guard.as_ref() {
                        current_header = Some(w.header.clone());
                        local_version = gv;
                        preimage_buf = w.header.preimage();
                        nonce = 0;
                    }
                }

                let Some(header) = &current_header else {
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                };

                let buf_len = preimage_buf.len();
                preimage_buf[buf_len - 8..].copy_from_slice(&nonce.to_le_bytes());
                let hash = hasher.hash(&preimage_buf);

                if check_pow_hash_meets_target(&hash, header.target_bits) {
                    let mut solved = header.clone();
                    solved.nonce = nonce;
                    solved.pow_hash = Some(hash);
                    let _ = solution_tx.send(solved);
                    break;
                }
                nonce = nonce.wrapping_add(1);
            }
        });

        // Let worker run on hard template for a bit
        std::thread::sleep(Duration::from_millis(50));
        assert!(solution_rx.try_recv().is_err());

        // Now advertise NEW template with round 2 and easy target
        let new_header = BlockHeader::new(
            5,
            2,
            0,
            vec![],
            kovanica_types::Hash([0x22; 32]),
            1_700_000_100,
            0x207fffff, // easy target
            0,
        );

        {
            let mut guard = active_work.write().unwrap();
            *guard = Some(ActiveWork {
                version: 2,
                header: new_header,
            });
        }
        work_version.store(2, Ordering::SeqCst);

        // Worker should detect stale work, switch to version 2, and solve it!
        let solved = solution_rx
            .blocking_recv()
            .expect("should solve new template");
        assert_eq!(solved.round, 2);
        assert!(solved.verify_pow().is_ok());

        shutdown.store(true, Ordering::SeqCst);
        handle.join().unwrap();
    }

    #[test]
    fn miner_bench_args_parse_defaults_and_custom() {
        use clap::Parser;
        #[derive(Parser, Debug)]
        struct BenchWrapper {
            #[command(flatten)]
            bench: MinerBenchArgs,
        }

        // Defaults
        let wrapper = BenchWrapper::try_parse_from(["test"]).unwrap();
        assert_eq!(wrapper.bench.threads, None);
        assert_eq!(wrapper.bench.duration, 5);

        // Custom threads and duration
        let wrapper =
            BenchWrapper::try_parse_from(["test", "--threads", "4", "--duration", "10"]).unwrap();
        assert_eq!(wrapper.bench.threads, Some(4));
        assert_eq!(wrapper.bench.duration, 10);

        // Short flags
        let wrapper = BenchWrapper::try_parse_from(["test", "-t", "8", "-d", "3"]).unwrap();
        assert_eq!(wrapper.bench.threads, Some(8));
        assert_eq!(wrapper.bench.duration, 3);
    }

    #[test]
    fn miner_bench_rejects_zero_duration() {
        let args = MinerBenchArgs {
            threads: Some(1),
            duration: 0,
        };
        let err = run_miner_bench(args, false).unwrap_err().to_string();
        assert!(err.contains("at least 1 second"), "{err}");
    }

    #[test]
    fn miner_bench_rejects_excessive_threads() {
        let args = MinerBenchArgs {
            threads: Some(512),
            duration: 1,
        };
        let err = run_miner_bench(args, false).unwrap_err().to_string();
        assert!(err.contains("exceeds maximum allowed"), "{err}");
    }

    #[test]
    fn miner_bench_execution_short() {
        let result = benchmark_kovanica_hash(1, Duration::from_millis(150))
            .expect("benchmark should succeed");
        assert_eq!(result.algorithm, "KovanicaHash (BLAKE3 + 2 MiB mix)");
        assert_eq!(result.threads, 1);
        assert!(result.total_hashes > 0);
        assert!(result.elapsed_secs > 0.0);
        assert!(result.hashrate_hps > 0.0);
        assert_eq!(result.thread_hashes.len(), 1);
        assert_eq!(result.thread_hashes[0], result.total_hashes);
        assert!(!result.hashrate_formatted.is_empty());
    }

    #[test]
    fn miner_bench_execution_multithreaded() {
        let result = benchmark_kovanica_hash(2, Duration::from_millis(150))
            .expect("benchmark should succeed");
        assert_eq!(result.threads, 2);
        assert!(result.total_hashes > 0);
        assert_eq!(result.thread_hashes.len(), 2);
        let sum: u64 = result.thread_hashes.iter().sum();
        assert_eq!(sum, result.total_hashes);
    }
}
