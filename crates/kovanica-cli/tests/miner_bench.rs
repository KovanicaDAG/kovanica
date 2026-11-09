//! Integration tests for the `kovanica miner bench` CLI subcommand.
//!
//! Verifies CLI option parsing, duration and thread flags, human-readable
//! and machine-readable JSON outputs, offline operation, and validation error handling.

use std::process::Command;
use std::time::Duration;

use kovanica_cli::miner::{benchmark_kovanica_hash, format_hashrate, MinerBenchResult};

fn kovanica_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kovanica"))
}

#[test]
fn miner_bench_help_displays_flags_and_descriptions() {
    let output = kovanica_bin()
        .args(["miner", "bench", "--help"])
        .output()
        .expect("failed to execute kovanica miner bench --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--threads"), "{stdout}");
    assert!(stdout.contains("--duration"), "{stdout}");
    assert!(stdout.contains("KovanicaHash"), "{stdout}");
}

#[test]
fn miner_bench_runs_standalone_offline_human_output() {
    let output = kovanica_bin()
        .args(["miner", "bench", "--threads", "1", "--duration", "1"])
        .output()
        .expect("failed to execute miner bench");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("KovanicaHash PoW Benchmark Results"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Algorithm:          KovanicaHash (BLAKE3 + 2 MiB mix)"),
        "{stdout}"
    );
    assert!(stdout.contains("Worker Threads:     1"), "{stdout}");
    assert!(stdout.contains("Target Duration:    1 s"), "{stdout}");
    assert!(stdout.contains("Elapsed Time:"), "{stdout}");
    assert!(stdout.contains("Total Hashes:"), "{stdout}");
    assert!(stdout.contains("Total Hashrate:"), "{stdout}");
    assert!(stdout.contains("Per-Thread Rate:"), "{stdout}");
}

#[test]
fn miner_bench_runs_with_global_json_flag() {
    let output = kovanica_bin()
        .args([
            "--json",
            "miner",
            "bench",
            "--threads",
            "2",
            "--duration",
            "1",
        ])
        .output()
        .expect("failed to execute miner bench with --json");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    let parsed: MinerBenchResult = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("failed to parse JSON output: {e}\nRaw output: {stdout}"));

    assert_eq!(parsed.algorithm, "KovanicaHash (BLAKE3 + 2 MiB mix)");
    assert_eq!(parsed.threads, 2);
    assert_eq!(parsed.target_duration_secs, 1);
    assert!(
        parsed.elapsed_secs >= 0.5,
        "elapsed: {}",
        parsed.elapsed_secs
    );
    assert!(
        parsed.total_hashes > 0,
        "total hashes: {}",
        parsed.total_hashes
    );
    assert!(parsed.hashrate_hps > 0.0, "hps: {}", parsed.hashrate_hps);
    assert!(
        parsed.per_thread_hps > 0.0,
        "per_thread: {}",
        parsed.per_thread_hps
    );
    assert_eq!(parsed.thread_hashes.len(), 2);
    let sum: u64 = parsed.thread_hashes.iter().sum();
    assert_eq!(sum, parsed.total_hashes);
    assert!(
        parsed.hashrate_formatted.ends_with("H/s")
            || parsed.hashrate_formatted.ends_with("kH/s")
            || parsed.hashrate_formatted.ends_with("MH/s")
            || parsed.hashrate_formatted.ends_with("GH/s"),
        "unexpected formatted rate: {}",
        parsed.hashrate_formatted
    );
}

#[test]
fn miner_bench_runs_with_trailing_json_flag() {
    let output = kovanica_bin()
        .args([
            "miner",
            "bench",
            "--threads",
            "1",
            "--duration",
            "1",
            "--json",
        ])
        .output()
        .expect("failed to execute miner bench with trailing --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("failed to parse JSON: {e}\nRaw: {stdout}"));

    assert_eq!(val["algorithm"], "KovanicaHash (BLAKE3 + 2 MiB mix)");
    assert_eq!(val["threads"], 1);
    assert_eq!(val["target_duration_secs"], 1);
    assert!(val["total_hashes"].as_u64().unwrap() > 0);
}

#[test]
fn mining_bench_alias_executes_identically() {
    let output = kovanica_bin()
        .args(["mining", "bench", "--threads", "1", "--duration", "1"])
        .output()
        .expect("failed to execute mining bench alias");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("KovanicaHash PoW Benchmark Results"),
        "{stdout}"
    );
    assert!(stdout.contains("Worker Threads:     1"), "{stdout}");
}

#[test]
fn miner_bench_rejects_zero_duration() {
    let output = kovanica_bin()
        .args(["miner", "bench", "--duration", "0"])
        .output()
        .expect("failed to run miner bench with 0 duration");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("at least 1 second"),
        "expected duration error, got: {stderr}"
    );
}

#[test]
fn miner_bench_rejects_excessive_thread_count() {
    let output = kovanica_bin()
        .args(["miner", "bench", "--threads", "500", "--duration", "1"])
        .output()
        .expect("failed to run miner bench with 500 threads");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("exceeds maximum allowed"),
        "expected thread cap error, got: {stderr}"
    );
}

#[test]
fn hashrate_formatting_precision_and_scaling() {
    assert_eq!(format_hashrate(0, 0.0), "0.00 H/s");
    assert_eq!(format_hashrate(0, 5.0), "0.00 H/s");
    assert_eq!(format_hashrate(750, 1.0), "750.00 H/s");
    assert_eq!(format_hashrate(1_500, 1.0), "1.50 kH/s");
    assert_eq!(format_hashrate(12_340, 2.0), "6.17 kH/s");
    assert_eq!(format_hashrate(2_500_000, 1.0), "2.50 MH/s");
    assert_eq!(format_hashrate(3_750_000_000, 1.0), "3.75 GH/s");
}

#[test]
fn direct_benchmark_kovanica_hash_api_returns_valid_metrics() {
    let result = benchmark_kovanica_hash(1, Duration::from_millis(150))
        .expect("direct benchmark should succeed");

    assert_eq!(result.algorithm, "KovanicaHash (BLAKE3 + 2 MiB mix)");
    assert_eq!(result.threads, 1);
    assert!(result.total_hashes > 0);
    assert!(result.elapsed_secs > 0.0);
    assert!(result.hashrate_hps > 0.0);
    assert_eq!(result.thread_hashes.len(), 1);
    assert_eq!(result.thread_hashes[0], result.total_hashes);
}
