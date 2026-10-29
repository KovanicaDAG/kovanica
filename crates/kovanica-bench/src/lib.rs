//! Load generation and performance benchmark helpers for Kovanica.
//!
//! Optional crate, safe to ship unused: no node/consensus dependency, talks only
//! over public JSON-RPC.

#![deny(unsafe_code)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use kovanica_crypto::generate_keypair;
use kovanica_sdk::rpc::RpcClient;
use kovanica_sdk::transfer as sdk_transfer;
use kovanica_types::{Address, SigningContext, SigningKey};
use serde_json::json;
use tokio::time::sleep;
use tracing::{debug, warn};

pub const DEFAULT_RPC_URL: &str = "http://127.0.0.1:8545";
pub const DEFAULT_CHAIN_ID: u64 = 1337;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    Err,
}

#[derive(Clone, Debug, Default)]
pub struct RunStats {
    pub total: u64,
    pub ok: u64,
    pub err: u64,
    pub latencies_ms: Vec<f64>,
    pub first_error: Option<String>,
}

impl RunStats {
    pub fn record(&mut self, lat_ms: f64, outcome: Outcome, error: Option<String>) {
        self.total += 1;
        match outcome {
            Outcome::Ok => self.ok += 1,
            Outcome::Err => {
                self.err += 1;
                if self.first_error.is_none() {
                    self.first_error = error;
                }
            }
        }
        self.latencies_ms.push(lat_ms);
    }

    pub fn mean_ms(&self) -> f64 {
        if self.latencies_ms.is_empty() {
            return 0.0;
        }
        self.latencies_ms.iter().sum::<f64>() / self.latencies_ms.len() as f64
    }

    pub fn err_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.err as f64 / self.total as f64
    }

    pub fn tps(&self, elapsed: Duration) -> f64 {
        let s = elapsed.as_secs_f64();
        if s <= 0.0 {
            return 0.0;
        }
        self.ok as f64 / s
    }

    pub fn csv_line(&self, elapsed: Duration) -> String {
        format!(
            "{},{},{},{:.3},{:.3},{:.3},{:.3},{:.4},{:.2}",
            self.ok,
            self.err,
            self.total,
            self.mean_ms(),
            self.p50_ms(),
            self.p95_ms(),
            self.p99_ms(),
            self.err_rate(),
            self.tps(elapsed),
        )
    }

    pub fn p50_ms(&self) -> f64 {
        percentile(&self.latencies_ms, 0.50)
    }
    pub fn p95_ms(&self) -> f64 {
        percentile(&self.latencies_ms, 0.95)
    }
    pub fn p99_ms(&self) -> f64 {
        percentile(&self.latencies_ms, 0.99)
    }
}

fn percentile(values: &[f64], q: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = q * (v.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        return v[lo];
    }
    let f = rank - lo as f64;
    v[lo] * (1.0 - f) + v[hi] * f
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    interval: Option<Duration>,
    next_allow: Instant,
}

impl RateLimiter {
    pub fn unbounded() -> Self {
        Self {
            interval: None,
            next_allow: Instant::now(),
        }
    }
    pub fn with_rps(rps: f64) -> Self {
        let interval = if rps > 0.0 {
            Some(Duration::from_secs_f64(1.0 / rps))
        } else {
            None
        };
        Self {
            interval,
            next_allow: Instant::now(),
        }
    }
    pub async fn permit(&mut self) {
        if let Some(interval) = self.interval {
            let now = Instant::now();
            if now < self.next_allow {
                sleep(self.next_allow - now).await;
            }
            self.next_allow = self.next_allow.max(now) + interval;
        }
    }
}

pub struct TransferWorker {
    client: Arc<RpcClient>,
    ctx: SigningContext,
    key: SigningKey,
    sender: Address,
    to: Address,
    amount: u64,
    fee: u64,
    starting_nonce: u64,
    count: u64,
    limiter: RateLimiter,
}

impl TransferWorker {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client: Arc<RpcClient>,
        ctx: SigningContext,
        key: SigningKey,
        to: Address,
        amount: u64,
        fee: u64,
        starting_nonce: u64,
        count: u64,
        limiter: RateLimiter,
    ) -> Self {
        let sender = Address(key.verifying_key().to_bytes());
        Self {
            client,
            ctx,
            key,
            sender,
            to,
            amount,
            fee,
            starting_nonce,
            count,
            limiter,
        }
    }

    pub fn sender(&self) -> Address {
        self.sender
    }
    pub fn starting_nonce(&self) -> u64 {
        self.starting_nonce
    }
    pub fn count(&self) -> u64 {
        self.count
    }

    pub async fn run(&mut self) -> RunStats {
        let mut stats = RunStats::default();
        for offset in 0..self.count {
            self.limiter.permit().await;
            let nonce = self.starting_nonce + offset;
            let started = Instant::now();
            let tx = sdk_transfer(&self.key, &self.ctx, self.to, self.amount, nonce, self.fee);
            let raw = match bincode::serialize(&tx) {
                Ok(b) => b,
                Err(e) => {
                    let lat = started.elapsed().as_secs_f64() * 1000.0;
                    stats.record(lat, Outcome::Err, Some(format!("serialize: {e}")));
                    continue;
                }
            };
            let result = self
                .client
                .call::<serde_json::Value>("kovanica_sendRawTransaction", json!([raw]))
                .await;
            let lat = started.elapsed().as_secs_f64() * 1000.0;
            match result {
                Ok(_) => stats.record(lat, Outcome::Ok, None),
                Err(e) => {
                    debug!(target: "kovanica-bench", "submit failed: {e}");
                    stats.record(lat, Outcome::Err, Some(e.to_string()));
                }
            }
        }
        stats
    }
}
pub struct BenchConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub epoch: u64,
    pub concurrency: usize,
    pub duration: Duration,
    pub rate_rps: Option<f64>,
    pub amount: u64,
    pub fee: u64,
    pub dest: Option<Address>,
}

pub async fn run_transfer_load(config: &BenchConfig) -> Result<RunStats> {
    use tokio::task::JoinSet;

    let client = Arc::new(RpcClient::new(config.rpc_url.clone()));
    let ctx = SigningContext {
        chain_id: config.chain_id,
        epoch: config.epoch,
    };
    let dest = config.dest.unwrap_or(Address([1u8; 32]));
    let concurrency = config.concurrency.max(1);
    let per_worker = if config.duration.as_secs() > 0 {
        let rps = config.rate_rps.unwrap_or(100.0).max(1.0);
        ((rps * config.duration.as_secs_f64()) as u64).max(1) / concurrency as u64
    } else {
        1
    };
    let amount = config.amount;
    let fee = config.fee;
    let rate_rps = config.rate_rps.unwrap_or(0.0);

    let mut set = JoinSet::new();
    for i in 0..concurrency {
        let key = generate_keypair().0;
        let client = Arc::clone(&client);
        let limiter = RateLimiter::with_rps(rate_rps);
        set.spawn(async move {
            let mut w = TransferWorker::new(
                client, ctx, key, dest, amount, fee, i as u64, per_worker, limiter,
            );
            w.run().await
        });
    }

    let mut agg = RunStats::default();
    while let Some(res) = set.join_next().await {
        match res {
            Ok(stats) => {
                agg.total += stats.total;
                agg.ok += stats.ok;
                agg.err += stats.err;
                agg.latencies_ms.extend(stats.latencies_ms);
                if agg.first_error.is_none() {
                    agg.first_error = stats.first_error;
                }
            }
            Err(e) => warn!(target: "kovanica-bench", "worker join error: {e}"),
        }
    }
    Ok(agg)
}

pub async fn run_health_poll(rpc_url: &str, rate_rps: f64, duration: Duration) -> Result<RunStats> {
    let client = Arc::new(RpcClient::new(rpc_url.to_string()));
    let mut limiter = RateLimiter::with_rps(rate_rps);
    let deadline = Instant::now() + duration;
    let mut stats = RunStats::default();
    while Instant::now() < deadline {
        limiter.permit().await;
        let started = Instant::now();
        let result = client
            .call::<serde_json::Value>("kovanica_syncStatus", json!([]))
            .await;
        let lat = started.elapsed().as_secs_f64() * 1000.0;
        match result {
            Ok(_) => stats.record(lat, Outcome::Ok, None),
            Err(e) => stats.record(lat, Outcome::Err, Some(e.to_string())),
        }
    }
    Ok(stats)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_aggregation_is_deterministic() {
        let mut s = RunStats::default();
        s.record(10.0, Outcome::Ok, None);
        s.record(20.0, Outcome::Err, Some("boom".into()));
        s.record(30.0, Outcome::Ok, None);
        assert_eq!(s.total, 3);
        assert_eq!(s.ok, 2);
        assert_eq!(s.err, 1);
        assert_eq!(s.first_error.as_deref(), Some("boom"));
        assert!((s.mean_ms() - 20.0).abs() < 1e-6);
        assert!((s.err_rate() - (1.0 / 3.0)).abs() < 1e-6);
    }

    #[test]
    fn percentile_handles_empty_and_sorted() {
        assert_eq!(percentile(&[], 0.5), 0.0);
        let v = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(percentile(&v, 0.5), 30.0);
        assert_eq!(percentile(&v, 1.0), 50.0);
        assert_eq!(percentile(&v, 0.0), 10.0);
    }

    #[test]
    fn csv_line_is_greppable() {
        let mut s = RunStats::default();
        s.record(5.0, Outcome::Ok, None);
        s.record(15.0, Outcome::Err, Some("x".into()));
        let line = s.csv_line(Duration::from_secs(2));
        let parts: Vec<&str> = line.split(',').collect();
        assert_eq!(parts.len(), 9);
        assert_eq!(parts[0], "1");
        assert_eq!(parts[1], "1");
        assert_eq!(parts[2], "2");
    }
}
