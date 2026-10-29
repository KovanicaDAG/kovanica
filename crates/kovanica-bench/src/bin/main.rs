//! `kovanica-bench` binary: load generation against a running Kovanica node.
//!
//! Example:
//!   # Health poll at 50 rps for 30s
//!   kovanica-bench health --rpc-url http://127.0.0.1:8545 --rate 50 --duration 30s
//!   # Transfer load: 8 workers, ~100 rps total, 60s
//!   kovanica-bench transfer-load --rpc-url http://127.0.0.1:8545 --workers 8 --rate 100 --duration 60s --amount 1 --fee 1

use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use kovanica_bench::{run_health_poll, run_transfer_load, BenchConfig};

#[derive(Parser, Debug)]
#[command(
    name = "kovanica-bench",
    about = "Load generation for Kovanica soak testing"
)]
enum Cli {
    Health(HealthArgs),
    TransferLoad(TransferLoadArgs),
}

#[derive(Parser, Debug)]
struct HealthArgs {
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc_url: String,
    #[arg(long, default_value_t = 10.0)]
    rate: f64,
    #[arg(long, value_parser = parse_duration, default_value = "30s")]
    duration: Duration,
}

#[derive(Parser, Debug)]
struct TransferLoadArgs {
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc_url: String,
    #[arg(long, default_value_t = 1337)]
    chain_id: u64,
    #[arg(long, default_value_t = 0)]
    epoch: u64,
    #[arg(long, default_value_t = 8)]
    workers: usize,
    #[arg(long, default_value_t = 100.0)]
    rate: f64,
    #[arg(long, value_parser = parse_duration, default_value = "60s")]
    duration: Duration,
    #[arg(long, default_value_t = 1)]
    amount: u64,
    #[arg(long, default_value_t = 1)]
    fee: u64,
}

fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    if let Some(secs) = s.strip_suffix('s') {
        Ok(Duration::from_secs(secs.parse()?))
    } else if let Some(ms) = s.strip_suffix("ms") {
        Ok(Duration::from_millis(ms.parse()?))
    } else {
        Ok(Duration::from_secs(s.parse()?))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    match Cli::parse() {
        Cli::Health(args) => {
            let stats = run_health_poll(&args.rpc_url, args.rate, args.duration).await?;
            println!(
                "health ok={} err={} total={} mean_ms={:.2} p50_ms={:.2} p95_ms={:.2} err_rate={:.4}",
                stats.ok,
                stats.err,
                stats.total,
                stats.mean_ms(),
                stats.p50_ms(),
                stats.p95_ms(),
                stats.err_rate()
            );
        }
        Cli::TransferLoad(args) => {
            let config = BenchConfig {
                rpc_url: args.rpc_url,
                chain_id: args.chain_id,
                epoch: args.epoch,
                concurrency: args.workers,
                duration: args.duration,
                rate_rps: Some(args.rate),
                amount: args.amount,
                fee: args.fee,
                dest: None,
            };
            let stats = run_transfer_load(&config).await?;
            println!(
                "transfer-load ok={} err={} total={} mean_ms={:.2} p50_ms={:.2} p95_ms={:.2} err_rate={:.4} tps={:.2}\nCSV: {}",
                stats.ok,
                stats.err,
                stats.total,
                stats.mean_ms(),
                stats.p50_ms(),
                stats.p95_ms(),
                stats.err_rate(),
                stats.tps(args.duration),
                stats.csv_line(args.duration)
            );
        }
    }
    Ok(())
}
