//! Mining and DAG inspection CLI operations.

use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::output::{print_json, print_kv, print_table};
use crate::rpc::RpcClient;

/// Operator view of `kovanica mining info`.
pub async fn info(client: &RpcClient, json_mode: bool) -> Result<()> {
    let res = client
        .call("kovanica_getMiningInfo", json!([]))
        .await
        .context("failed to query mining info")?;

    if json_mode {
        return print_json(&res);
    }

    let obj = res
        .as_object()
        .context("expected JSON object in response")?;

    let chain_id = obj
        .get("chain_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    let mining_enabled = obj
        .get("mining_enabled")
        .and_then(Value::as_bool)
        .map(|b| if b { "true" } else { "false" })
        .unwrap_or("false")
        .to_string();
    let mining_threads = obj
        .get("mining_threads")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0".to_string());
    let target_bits = obj
        .get("target_bits")
        .and_then(Value::as_str)
        .unwrap_or("-")
        .to_string();
    let difficulty = obj
        .get("difficulty")
        .and_then(Value::as_str)
        .unwrap_or("-")
        .to_string();
    let target_block_time_ms = obj
        .get("target_block_time_ms")
        .map(|v| format!("{v} ms"))
        .unwrap_or_else(|| "-".to_string());
    let ghostdag_k = obj
        .get("ghostdag_k")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    let hashrate = obj
        .get("network_hashrate_estimate")
        .map(|v| format!("{v} H/s"))
        .unwrap_or_else(|| "-".to_string());
    let tips_count = obj
        .get("dag_tips_count")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0".to_string());

    let rows = vec![
        ("Chain ID", chain_id),
        ("Mining Enabled", mining_enabled),
        ("Mining Threads", mining_threads),
        ("Target Bits", target_bits),
        ("Difficulty", difficulty),
        ("Target Block Time", target_block_time_ms),
        ("GHOSTDAG K", ghostdag_k),
        ("Est. Network Hashrate", hashrate),
        ("Active DAG Tips", tips_count),
    ];

    print_kv(&rows);
    Ok(())
}

/// Operator view of `kovanica mining status`.
pub async fn status(client: &RpcClient, json_mode: bool) -> Result<()> {
    let res = client
        .call("kovanica_getMiningInfo", json!([]))
        .await
        .context("failed to query mining info")?;

    if json_mode {
        return print_json(&res);
    }

    let obj = res
        .as_object()
        .context("expected JSON object in response")?;

    let chain_id = obj
        .get("chain_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    let enabled = obj
        .get("mining_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mining_status = if enabled {
        "Running (Active)"
    } else {
        "Disabled"
    }
    .to_string();
    let threads = obj
        .get("mining_threads")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0".to_string());
    let target_bits = obj
        .get("target_bits")
        .and_then(Value::as_str)
        .unwrap_or("-")
        .to_string();
    let hashrate = obj
        .get("network_hashrate_estimate")
        .map(|v| format!("{v} H/s"))
        .unwrap_or_else(|| "-".to_string());

    let rows = vec![
        ("Chain ID", chain_id),
        ("Mining Status", mining_status),
        ("Worker Threads", threads),
        ("Target Bits", target_bits),
        ("Est. Network Hashrate", hashrate),
    ];

    print_kv(&rows);
    Ok(())
}

/// Operator view of `kovanica dag tips`.
pub async fn dag_tips(client: &RpcClient, json_mode: bool) -> Result<()> {
    let res = client
        .call("kovanica_getDagTips", json!([]))
        .await
        .context("failed to query DAG tips")?;

    if json_mode {
        return print_json(&res);
    }

    let obj = res
        .as_object()
        .context("expected JSON object in response")?;
    let chain_id = obj
        .get("chain_id")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    let count = obj
        .get("count")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0".to_string());
    let max_round = obj
        .get("max_round")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "0".to_string());
    let ghostdag_k = obj
        .get("ghostdag_k")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());

    println!("Chain ID: {chain_id} | Max Round: {max_round} | GHOSTDAG K: {ghostdag_k} | Total Tips: {count}");

    let tips = obj
        .get("tips")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if tips.is_empty() {
        println!("No active tips.");
        return Ok(());
    }

    let headers = ["Index", "Tip Hash"];
    let rows: Vec<Vec<String>> = tips
        .iter()
        .enumerate()
        .map(|(idx, val)| {
            vec![
                (idx + 1).to_string(),
                val.as_str().unwrap_or("-").to_string(),
            ]
        })
        .collect();

    print_table(&headers, &rows);
    Ok(())
}
