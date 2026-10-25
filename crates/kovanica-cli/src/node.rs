//! Node operations: status, balances, staking and governance.
//!
//! Status and balance are backed by node APIs. Staking and governance RPC
//! methods do not exist on the node yet, so those commands emit a `TODO`
//! describing the intended method name and payload.

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::output::{print_json, print_kv, print_table};
use crate::rpc::{parse_quantity, RpcClient};
use crate::tui::format_kuna;
use crate::wallet::parse_address;

/// Render a JSON value as a display cell (`-` when absent).
fn cell(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => "-".to_string(),
    }
}

fn status_json(rpc: &str, height: u64, members: &[Value], peer_count: Option<usize>) -> Value {
    json!({
        "rpc": rpc,
        "latestBlock": height,
        "committeeSize": members.len(),
        "committee": members,
        // Sync state is unknown: the node exposes no sync-status RPC.
        "syncing": Value::Null,
        "peerCount": peer_count,
    })
}

fn status_rows(
    rpc: &str,
    height: u64,
    committee_size: usize,
    peer_count: Option<usize>,
) -> Vec<(&'static str, String)> {
    vec![
        ("RPC", rpc.to_string()),
        ("Latest block", height.to_string()),
        ("Committee size", committee_size.to_string()),
        (
            "Sync status",
            "unknown (no sync-status RPC available)".to_string(),
        ),
        (
            "Peer count",
            peer_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "unknown (health unavailable)".to_string()),
        ),
    ]
}

/// `kovanica status` — latest block, committee, peer count, and sync status when available.
pub async fn status(client: &RpcClient, json_output: bool) -> Result<()> {
    let height_value = client.call("kovanica_blockNumber", json!([])).await?;
    let height = parse_quantity(&height_value)?;

    let committee_value = client.call("kovanica_getCommittee", json!([])).await?;
    let members = committee_value.as_array().cloned().unwrap_or_default();
    let peer_count = client.health_peer_count().await;

    if json_output {
        let out = status_json(client.url(), height, &members, peer_count);
        return print_json(&out);
    }

    print_kv(&status_rows(
        client.url(),
        height,
        members.len(),
        peer_count,
    ));

    if !members.is_empty() {
        println!();
        let rows: Vec<Vec<String>> = members
            .iter()
            .map(|m| {
                vec![
                    cell(m.get("index")),
                    cell(m.get("address")),
                    cell(m.get("stake")),
                    cell(m.get("network_address")),
                ]
            })
            .collect();
        print_table(&["INDEX", "ADDRESS", "STAKE", "NETWORK"], &rows);
    }

    Ok(())
}

/// `kovanica balance <address>` — native KUNA balance in atoms.
pub async fn balance(
    client: &RpcClient,
    address: &str,
    chain_id: Option<u64>,
    json_output: bool,
) -> Result<()> {
    // No selected chain is not a dev chain: legacy hex is refused. The
    // canonical form still parses.
    let address = parse_address(address, "address", chain_id.unwrap_or(0))?;
    let value = client
        .call("kovanica_getBalance", json!([address.to_string()]))
        .await?;
    let balance = parse_quantity(&value)?;

    if json_output {
        return print_json(&json!({
            "address": address.to_string(),
            "balance": balance,
        }));
    }

    print_kv(&[
        ("Address", address.to_string()),
        ("Balance", format!("{balance} atoms")),
    ]);
    Ok(())
}

/// Params object for `kovanica_getTransactionHistory`.
///
/// The address is parsed with [`parse_address`] and sent in canonical
/// `kvnc…dag` form. `--sent` and `--received` are exclusive; neither means
/// the node default (`all`). A blank cursor is rejected.
pub fn history_params(
    address: &str,
    chain_id: Option<u64>,
    limit: Option<u64>,
    cursor: Option<&str>,
    sent: bool,
    received: bool,
) -> Result<Value> {
    if sent && received {
        return Err(anyhow!("--sent and --received cannot be used together"));
    }
    // No selected chain is not a dev chain: legacy hex is refused. The
    // canonical form still parses.
    let address = parse_address(address, "address", chain_id.unwrap_or(0))?;
    let mut params = serde_json::Map::new();
    params.insert("address".into(), json!(address.to_string()));
    if let Some(limit) = limit {
        params.insert("limit".into(), json!(limit));
    }
    if let Some(cursor) = cursor {
        let cursor = cursor.trim();
        if cursor.is_empty() {
            return Err(anyhow!("cursor: empty"));
        }
        params.insert("cursor".into(), json!(cursor));
    }
    if sent {
        params.insert("direction".into(), json!("sent"));
    } else if received {
        params.insert("direction".into(), json!("received"));
    }
    Ok(Value::Object(params))
}

/// JSON and table rows for one history page.
///
/// JSON keeps the node's hex quantities and adds `amountAtoms` /
/// `amountKuna` and `feeAtoms` / `feeKuna` (9 decimal places).
pub fn history_view(page: &Value) -> Result<HistoryView> {
    let entries = page
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("history: entries must be an array"))?;
    let next_cursor = match page.get("next_cursor") {
        None | Some(Value::Null) => None,
        Some(Value::String(cursor)) => Some(cursor.clone()),
        Some(other) => {
            return Err(anyhow!(
                "history: next_cursor must be a string or null, got {other}"
            ))
        }
    };

    let mut json_entries = Vec::with_capacity(entries.len());
    let mut rows = Vec::with_capacity(entries.len());
    for entry in entries {
        let amount = parse_quantity(
            entry
                .get("amount")
                .ok_or_else(|| anyhow!("history: entry missing amount"))?,
        )?;
        let fee = parse_quantity(
            entry
                .get("fee")
                .ok_or_else(|| anyhow!("history: entry missing fee"))?,
        )?;
        let amount_kuna = format_kuna(amount);
        let fee_kuna = format_kuna(fee);
        let height = entry
            .get("height")
            .ok_or_else(|| anyhow!("history: entry missing height"))?;
        let height_text = parse_quantity(height)?.to_string();
        let timestamp_text = match entry.get("timestamp") {
            None | Some(Value::Null) => "-".to_string(),
            Some(value) => parse_quantity(value)?.to_string(),
        };
        let kind = json_string(entry, "kind")?;
        let status = json_string(entry, "status")?;
        let from = json_string(entry, "from")?;
        let to = match entry.get("to") {
            None | Some(Value::Null) => "-".to_string(),
            Some(Value::String(value)) => value.clone(),
            Some(other) => {
                return Err(anyhow!("history: to must be a string or null, got {other}"))
            }
        };
        let hash = json_string(entry, "hash")?;
        let mut enriched = entry.clone();
        if let Some(object) = enriched.as_object_mut() {
            object.insert("amountAtoms".into(), json!(amount));
            object.insert("amountKuna".into(), json!(amount_kuna.clone()));
            object.insert("feeAtoms".into(), json!(fee));
            object.insert("feeKuna".into(), json!(fee_kuna.clone()));
        }
        json_entries.push(enriched);
        rows.push(vec![
            height_text,
            timestamp_text,
            kind,
            status,
            from,
            to,
            amount_kuna,
            fee_kuna,
            hash,
        ]);
    }

    Ok(HistoryView {
        json: json!({
            "entries": json_entries,
            "next_cursor": next_cursor,
        }),
        rows,
        next_cursor,
    })
}

fn json_string(entry: &Value, field: &str) -> Result<String> {
    entry
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("history: entry missing {field}"))
}

/// Rendered history page.
pub struct HistoryView {
    /// JSON object with atom counts and KUNA strings added to each entry.
    pub json: Value,
    /// Human table cells: height, timestamp, kind, status, from, to, amount, fee, hash.
    pub rows: Vec<Vec<String>>,
    /// Cursor for the next page, when the node returned one.
    pub next_cursor: Option<String>,
}

/// `kovanica history <address>` — newest-first committed transactions.
#[allow(clippy::too_many_arguments)]
pub async fn history(
    client: &RpcClient,
    address: &str,
    chain_id: Option<u64>,
    limit: Option<u64>,
    cursor: Option<&str>,
    sent: bool,
    received: bool,
    json_output: bool,
) -> Result<()> {
    let address_parsed = parse_address(address, "address", chain_id.unwrap_or(0))?;
    let params = history_params(address, chain_id, limit, cursor, sent, received)?;
    let page = client
        .call("kovanica_getTransactionHistory", params)
        .await?;
    let view = history_view(&page)?;
    if json_output {
        return print_json(&view.json);
    }
    print_kv(&[("Address", address_parsed.to_string())]);
    if view.rows.is_empty() {
        println!("No transactions.");
    } else {
        println!();
        print_table(
            &[
                "HEIGHT",
                "TIMESTAMP",
                "KIND",
                "STATUS",
                "FROM",
                "TO",
                "AMOUNT",
                "FEE",
                "HASH",
            ],
            &view.rows,
        );
    }
    println!();
    match &view.next_cursor {
        Some(cursor) => println!("Next cursor: {cursor}"),
        None => println!("Next cursor: none"),
    }
    Ok(())
}

/// Print a `TODO` for an operation whose RPC method is not implemented yet.
pub fn not_implemented(
    operation: &str,
    intended_rpc: &str,
    params: Value,
    json_output: bool,
) -> Result<()> {
    if json_output {
        return print_json(&json!({
            "status": "not_implemented",
            "operation": operation,
            "intendedRpc": intended_rpc,
            "params": params,
        }));
    }

    println!("TODO: `{operation}` is not implemented yet.");
    println!("      Intended RPC method: `{intended_rpc}`");
    println!("      Intended params:     {params}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_json_and_human_rows_show_zero_and_nonzero_peer_counts() {
        let members = vec![];
        for count in [0, 4] {
            let output = status_json("http://localhost:8545/rpc", 12, &members, Some(count));
            assert_eq!(output["peerCount"], json!(count));
            let rows = status_rows("http://localhost:8545/rpc", 12, 0, Some(count));
            assert_eq!(
                rows.iter().find(|(key, _)| *key == "Peer count").unwrap().1,
                count.to_string()
            );
            assert_eq!(output["syncing"], Value::Null);
        }
    }

    #[test]
    fn missing_peer_count_stays_unknown_in_json_and_human_rows() {
        let output = status_json("http://localhost:8545/rpc", 12, &[], None);
        assert_eq!(output["peerCount"], Value::Null);
        assert_eq!(output["syncing"], Value::Null);
        let rows = status_rows("http://localhost:8545/rpc", 12, 0, None);
        assert_eq!(
            rows.iter().find(|(key, _)| *key == "Peer count").unwrap().1,
            "unknown (health unavailable)"
        );
    }

    #[test]
    fn history_params_canonicalizes_address_and_sets_direction() {
        let hex = "11".repeat(32);
        let canonical = kovanica_types::Address([0x11; 32]).to_string();
        let cursor = " ab ".to_string();
        let params = history_params(
            &hex,
            Some(kovanica_types::signing::chain_id::LOCAL),
            Some(5),
            Some(cursor.as_str()),
            true,
            false,
        )
        .unwrap();
        assert_eq!(params["address"], json!(canonical));
        assert!(canonical.starts_with("kvnc") && canonical.ends_with("dag"));
        assert_eq!(params["limit"], json!(5));
        assert_eq!(params["cursor"], json!("ab"));
        assert_eq!(params["direction"], json!("sent"));

        // Legacy hex is refused when chain_id is None (defaults to 0) or mainnet (1).
        assert!(history_params(&hex, None, None, None, false, false).is_err());
        assert!(history_params(
            &hex,
            Some(kovanica_types::signing::chain_id::MAINNET),
            None,
            None,
            false,
            false,
        )
        .is_err());

        let all = history_params(&canonical, None, None, None, false, false).unwrap();
        assert!(all.get("limit").is_none());
        assert!(all.get("cursor").is_none());
        assert!(all.get("direction").is_none());
        assert_eq!(all["address"], json!(canonical));

        let received = history_params(&canonical, None, None, None, false, true).unwrap();
        assert_eq!(received["direction"], json!("received"));

        assert!(history_params(&canonical, None, None, None, true, true).is_err());
        assert!(history_params("not-an-address", None, None, None, false, false).is_err());
        assert!(history_params(&canonical, None, None, Some("  "), false, false).is_err());
        let bad = format!("kvnc{}dag", "00".repeat(36));
        assert!(history_params(&bad, None, None, None, false, false).is_err());
    }

    #[test]
    fn history_view_formats_kuna_with_nine_decimals_and_keeps_atoms() {
        let page = json!({
            "entries": [
                {
                    "hash": "0xab",
                    "height": "0x2",
                    "timestamp": null,
                    "from": "kvnc-from",
                    "to": "kvnc-to",
                    "amount": "0x3b9aca00",
                    "fee": "0x1",
                    "kind": "transfer",
                    "status": "success"
                },
                {
                    "hash": "0xcd",
                    "height": "0x1",
                    "timestamp": "0x2a",
                    "from": "kvnc-from",
                    "to": null,
                    "amount": "0x0",
                    "fee": "0x5f5e100",
                    "kind": "stake",
                    "status": "failed"
                }
            ],
            "next_cursor": "aa"
        });
        let view = history_view(&page).unwrap();
        assert_eq!(view.next_cursor.as_deref(), Some("aa"));
        assert_eq!(
            view.json["entries"][0]["amountAtoms"],
            json!(1_000_000_000u64)
        );
        assert_eq!(view.json["entries"][0]["amountKuna"], json!("1 KUNA"));
        assert_eq!(view.json["entries"][0]["feeAtoms"], json!(1));
        assert_eq!(
            view.json["entries"][0]["feeKuna"],
            json!("0.000000001 KUNA")
        );
        assert_eq!(view.json["entries"][0]["amount"], json!("0x3b9aca00"));
        assert_eq!(view.json["entries"][1]["amountKuna"], json!("0 KUNA"));
        assert_eq!(view.json["entries"][1]["feeAtoms"], json!(100_000_000u64));
        assert_eq!(view.json["entries"][1]["feeKuna"], json!("0.1 KUNA"));
        assert_eq!(view.rows[0][0], "2");
        assert_eq!(view.rows[0][1], "-");
        assert_eq!(view.rows[0][6], "1 KUNA");
        assert_eq!(view.rows[0][7], "0.000000001 KUNA");
        assert_eq!(view.rows[1][1], "42");
        assert_eq!(view.rows[1][5], "-");
        assert_eq!(view.rows[1][2], "stake");
        assert_eq!(view.rows.len(), 2);

        let whole = format_kuna(1_500_000_000);
        assert_eq!(whole, "1.5 KUNA");
        let atom = format_kuna(1);
        assert_eq!(atom, "0.000000001 KUNA");

        let last = json!({"entries": [], "next_cursor": null});
        let empty = history_view(&last).unwrap();
        assert!(empty.rows.is_empty());
        assert!(empty.next_cursor.is_none());
        assert!(history_view(&json!({"entries": "no"})).is_err());
        assert!(history_view(&json!({
            "entries": [{"hash": "0x1", "height": "0x1", "from": "a", "kind": "transfer", "status": "success"}],
            "next_cursor": null
        }))
        .is_err());
    }
}
