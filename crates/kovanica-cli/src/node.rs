//! Node operations: status, balances, staking and governance.
//!
//! Status, balance, stake, and rewards read the node JSON-RPC. Governance
//! proposal commands are still a TODO: the node has no proposal RPC.

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

use crate::amount::{format_kuna_decimal, format_kuna_fixed};
use crate::output::{print_json, print_kv, print_table};
use crate::rpc::{parse_quantity, RpcClient};
use crate::wallet::parse_address;
use kovanica_types::Address;

/// Render a JSON value as a display cell (`-` when absent).
fn cell(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => "-".to_string(),
    }
}

/// One connected peer from `kovanica_syncStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerView {
    /// libp2p peer id. This is a transport identity, not a validator key.
    pub peer_id: String,
    /// Last DAG tip round the peer advertised.
    pub reported_tip_round: Option<u64>,
}

/// Fields read from `kovanica_syncStatus`.
///
/// Counters are plain JSON integers on the wire. Hex quantities are accepted
/// too, because other chain methods use that encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncSnapshot {
    /// Committed leader height, falling back to `localHeight`.
    pub height: Option<u64>,
    /// Local DAG tip round.
    pub round: Option<u64>,
    /// `None` when the payload did not say whether the node is catching up.
    pub catching_up: Option<bool>,
    pub behind_by: Option<u64>,
    pub target: Option<u64>,
    pub peer_count: Option<usize>,
    pub peers: Vec<PeerView>,
}

/// How far the node is from the peers it can see.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncState {
    /// `kovanica_syncStatus` reported `catchingUp: false`.
    InSync,
    /// `kovanica_syncStatus` reported `catchingUp: true`.
    CatchingUp {
        behind_by: Option<u64>,
        target: Option<u64>,
    },
    /// The sync method was missing or did not include `catchingUp`.
    Unknown,
}

/// Operator view of `kovanica status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainStatus {
    pub rpc: String,
    /// `--chain-id` or `KOVANICA_CHAIN_ID`. The node does not report this.
    pub signing_chain_id: Option<u64>,
    pub height: Option<u64>,
    pub round: Option<u64>,
    pub epoch: Option<u64>,
    pub committee: Vec<Value>,
    pub peer_count: Option<usize>,
    pub peers: Vec<PeerView>,
    pub sync: SyncState,
}

/// Parse `kovanica_syncStatus`.
pub fn parse_sync_status(value: &Value) -> Result<SyncSnapshot> {
    let object = value
        .as_object()
        .context("kovanica_syncStatus: expected an object")?;
    let committed = optional_quantity(object.get("committedHeight"))?;
    let local_height = optional_quantity(object.get("localHeight"))?;
    let peers = object
        .get("peers")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let peer_id = item.get("peerId")?.as_str()?.trim();
                    if peer_id.is_empty() {
                        return None;
                    }
                    let reported_tip_round =
                        optional_quantity(item.get("reportedTipRound")).unwrap_or(None);
                    Some(PeerView {
                        peer_id: peer_id.to_string(),
                        reported_tip_round,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let peer_count = match object.get("peerCount") {
        None | Some(Value::Null) => None,
        Some(count) => Some(
            usize::try_from(parse_quantity(count).context("kovanica_syncStatus peerCount")?)
                .map_err(|_| anyhow!("kovanica_syncStatus peerCount does not fit"))?,
        ),
    };
    Ok(SyncSnapshot {
        height: committed.or(local_height),
        round: optional_quantity(object.get("localTipRound"))?,
        catching_up: match object.get("catchingUp") {
            Some(Value::Bool(flag)) => Some(*flag),
            _ => None,
        },
        behind_by: optional_quantity(object.get("behindBy"))?,
        target: optional_quantity(object.get("catchUpTarget"))?,
        peer_count,
        peers,
    })
}

/// Parse `kovanica_getCommitteeInfo` into `(epoch, members)`.
pub fn parse_committee_info(value: &Value) -> Result<(Option<u64>, Vec<Value>)> {
    let object = value
        .as_object()
        .context("kovanica_getCommitteeInfo: expected an object")?;
    let epoch = optional_quantity(object.get("epoch"))?;
    let members = object
        .get("members")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok((epoch, members))
}

/// Combine the RPC pieces `status` managed to read.
///
/// Height prefers the sync snapshot, then `kovanica_blockNumber`. Peer count
/// prefers the sync snapshot, including zero, then `/health`.
pub fn assemble_status(
    rpc: &str,
    signing_chain_id: Option<u64>,
    block_height: Option<u64>,
    sync: Option<&SyncSnapshot>,
    epoch: Option<u64>,
    committee: Vec<Value>,
    health_peers: Option<usize>,
) -> ChainStatus {
    let (height, round, peer_count, peers, sync_state) = match sync {
        Some(sync) => (
            sync.height.or(block_height),
            sync.round,
            sync.peer_count.or(health_peers),
            sync.peers.clone(),
            match sync.catching_up {
                Some(false) => SyncState::InSync,
                Some(true) => SyncState::CatchingUp {
                    behind_by: sync.behind_by,
                    target: sync.target,
                },
                None => SyncState::Unknown,
            },
        ),
        None => (
            block_height,
            None,
            health_peers,
            Vec::new(),
            SyncState::Unknown,
        ),
    };
    ChainStatus {
        rpc: rpc.to_string(),
        signing_chain_id,
        height,
        round,
        epoch,
        committee,
        peer_count,
        peers,
        sync: sync_state,
    }
}

fn status_json(status: &ChainStatus) -> Value {
    json!({
        "rpc": status.rpc,
        "chainIdReportedByNode": false,
        "signingChainId": status.signing_chain_id,
        "latestBlock": status.height,
        "round": status.round,
        "epoch": status.epoch,
        "committeeSize": status.committee.len(),
        "committee": status.committee,
        "syncing": match &status.sync {
            SyncState::InSync => Value::Bool(false),
            SyncState::CatchingUp { .. } => Value::Bool(true),
            SyncState::Unknown => Value::Null,
        },
        "sync": sync_label(&status.sync),
        "peerCount": status.peer_count,
        "peers": status.peers.iter().map(|peer| json!({
            "peerId": peer.peer_id,
            "reportedTipRound": peer.reported_tip_round,
        })).collect::<Vec<_>>(),
    })
}

fn status_rows(status: &ChainStatus) -> Vec<(&'static str, String)> {
    vec![
        ("RPC", status.rpc.clone()),
        (
            "Signing chain id",
            signing_chain_label(status.signing_chain_id),
        ),
        ("Height", optional_number(status.height)),
        ("Round", optional_number(status.round)),
        ("Epoch", optional_number(status.epoch)),
        ("Sync", sync_label(&status.sync)),
        (
            "Peers",
            optional_number(status.peer_count.map(|count| count as u64)),
        ),
        ("Committee size", status.committee.len().to_string()),
    ]
}

fn signing_chain_label(chain_id: Option<u64>) -> String {
    match chain_id {
        Some(id) => format!("{id} (local flag; the node does not report chain_id)"),
        None => "not reported by the node".to_string(),
    }
}

fn optional_number(value: Option<u64>) -> String {
    value
        .map(|number| number.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn sync_label(sync: &SyncState) -> String {
    match sync {
        SyncState::InSync => "in sync".to_string(),
        SyncState::Unknown => "unknown".to_string(),
        SyncState::CatchingUp {
            behind_by: Some(behind),
            target: Some(target),
        } => format!("catching up (behind {behind} rounds, target {target})"),
        SyncState::CatchingUp {
            behind_by: Some(behind),
            target: None,
        } => format!("catching up (behind {behind} rounds)"),
        SyncState::CatchingUp {
            behind_by: None,
            target: Some(target),
        } => format!("catching up (target {target})"),
        SyncState::CatchingUp {
            behind_by: None,
            target: None,
        } => "catching up".to_string(),
    }
}

fn optional_quantity(value: Option<&Value>) -> Result<Option<u64>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(other) => Ok(Some(parse_quantity(other)?)),
    }
}

fn stake_cell(value: Option<&Value>) -> String {
    value
        .and_then(|stake| parse_quantity(stake).ok())
        .map(format_kuna_fixed)
        .unwrap_or_else(|| cell(value))
}

/// `kovanica status` — height, round, epoch, peers, and sync, as the node reports them.
///
/// `signing_chain_id` is the local `--chain-id` / `KOVANICA_CHAIN_ID` value.
/// No RPC returns the node's chain id, so the command says so instead of
/// inventing one.
pub async fn status(
    client: &RpcClient,
    signing_chain_id: Option<u64>,
    json_output: bool,
) -> Result<()> {
    let mut block_error = None;
    let block_height = match client.call("kovanica_blockNumber", json!([])).await {
        Ok(value) => Some(parse_quantity(&value).context("kovanica_blockNumber")?),
        Err(error) => {
            block_error = Some(error);
            None
        }
    };
    let sync = match client.call("kovanica_syncStatus", json!([])).await {
        Ok(value) => Some(parse_sync_status(&value)?),
        Err(error) => {
            if block_height.is_none() {
                return Err(block_error.unwrap_or(error));
            }
            None
        }
    };
    let (epoch, committee) = match client.call("kovanica_getCommitteeInfo", json!([])).await {
        Ok(value) => parse_committee_info(&value)?,
        Err(_) => match client.call("kovanica_getCommittee", json!([])).await {
            Ok(value) => (None, value.as_array().cloned().unwrap_or_default()),
            Err(_) => (None, Vec::new()),
        },
    };
    let health_peers = if sync.as_ref().and_then(|item| item.peer_count).is_none() {
        client.health_peer_count().await
    } else {
        None
    };
    let status = assemble_status(
        client.url(),
        signing_chain_id,
        block_height,
        sync.as_ref(),
        epoch,
        committee,
        health_peers,
    );

    if json_output {
        return print_json(&status_json(&status));
    }

    print_kv(&status_rows(&status));
    if !status.committee.is_empty() {
        println!();
        let rows: Vec<Vec<String>> = status
            .committee
            .iter()
            .map(|member| {
                vec![
                    cell(member.get("index")),
                    cell(member.get("address")),
                    stake_cell(member.get("stake")),
                    cell(member.get("network_address")),
                ]
            })
            .collect();
        print_table(&["INDEX", "ADDRESS", "STAKE", "NETWORK"], &rows);
    }
    if !status.peers.is_empty() {
        println!();
        let rows: Vec<Vec<String>> = status
            .peers
            .iter()
            .map(|peer| {
                vec![
                    peer.peer_id.clone(),
                    peer.reported_tip_round
                        .map(|round| round.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                ]
            })
            .collect();
        print_table(&["PEER", "TIP ROUND"], &rows);
    }
    Ok(())
}

/// JSON and human rows for a native balance.
///
/// `balance` stays the atom count. `balanceKuna` is the same value with exactly
/// 9 decimal places, so a JSON number is never asked to hold a fractional KUNA.
pub fn balance_report(address: &Address, atoms: u64) -> Value {
    json!({
        "address": address.to_string(),
        "balance": atoms,
        "balanceKuna": format_kuna_decimal(atoms),
    })
}

/// Human-readable balance rows. The amount always shows 9 decimal places.
pub fn balance_rows(address: &Address, atoms: u64) -> Vec<(&'static str, String)> {
    vec![
        ("Address", address.to_string()),
        ("Balance", format_kuna_fixed(atoms)),
    ]
}

/// `kovanica balance <address>` — native balance, displayed in KUNA.
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
    let atoms = parse_quantity(&value)?;

    if json_output {
        return print_json(&balance_report(&address, atoms));
    }

    print_kv(&balance_rows(&address, atoms));
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

    fn sample_sync(catching_up: bool, peers: usize) -> SyncSnapshot {
        parse_sync_status(&json!({
            "localTipRound": 36,
            "committedHeight": 12,
            "localHeight": 12,
            "catchingUp": catching_up,
            "catchUpTarget": if catching_up { Some(40) } else { None::<u64> },
            "behindBy": if catching_up { Some(4) } else { None::<u64> },
            "peerCount": peers,
            "peers": [{ "peerId": "12D3KooWexample", "reportedTipRound": 36 }],
        }))
        .unwrap()
    }

    #[test]
    fn status_shows_height_round_epoch_sync_and_a_zero_peer_count() {
        let sync = sample_sync(false, 0);
        let committee = json!({
            "epoch": "0x2",
            "members": [{
                "index": 0,
                "address": "kvncexampledag",
                "stake": "0x3b9aca00",
                "network_address": "validator.example:8000"
            }]
        });
        let (epoch, members) = parse_committee_info(&committee).unwrap();
        let status = assemble_status(
            "http://127.0.0.1:8545/rpc",
            Some(4),
            Some(9),
            Some(&sync),
            epoch,
            members,
            Some(99),
        );
        assert_eq!(status.height, Some(12));
        assert_eq!(status.round, Some(36));
        assert_eq!(status.epoch, Some(2));
        assert_eq!(status.sync, SyncState::InSync);
        // Zero from the sync method wins over a health fallback.
        assert_eq!(status.peer_count, Some(0));
        let output = status_json(&status);
        assert_eq!(output["chainIdReportedByNode"], json!(false));
        assert_eq!(output["signingChainId"], json!(4));
        assert_eq!(output["syncing"], json!(false));
        assert_eq!(output["latestBlock"], json!(12));
        assert_eq!(output["round"], json!(36));
        assert_eq!(output["epoch"], json!(2));
        let rows = status_rows(&status);
        assert_eq!(row(&rows, "Height"), "12");
        assert_eq!(row(&rows, "Round"), "36");
        assert_eq!(row(&rows, "Epoch"), "2");
        assert_eq!(row(&rows, "Sync"), "in sync");
        assert_eq!(row(&rows, "Peers"), "0");
        assert!(row(&rows, "Signing chain id").contains("local flag"));
        assert_eq!(stake_cell(Some(&json!("0x3b9aca00"))), "1.000000000 KUNA");
    }

    #[test]
    fn status_reports_catch_up_and_leaves_chain_id_unknown_without_a_flag() {
        let sync = sample_sync(true, 1);
        let status = assemble_status(
            "http://127.0.0.1:8545/rpc",
            None,
            None,
            Some(&sync),
            None,
            Vec::new(),
            None,
        );
        assert_eq!(
            status.sync,
            SyncState::CatchingUp {
                behind_by: Some(4),
                target: Some(40),
            }
        );
        let output = status_json(&status);
        assert_eq!(output["syncing"], json!(true));
        assert_eq!(output["signingChainId"], Value::Null);
        assert_eq!(output["peerCount"], json!(1));
        assert_eq!(output["peers"][0]["peerId"], json!("12D3KooWexample"));
        let rows = status_rows(&status);
        assert_eq!(
            row(&rows, "Sync"),
            "catching up (behind 4 rounds, target 40)"
        );
        assert_eq!(row(&rows, "Signing chain id"), "not reported by the node");
        assert_eq!(row(&rows, "Epoch"), "unknown");
    }

    #[test]
    fn missing_sync_status_stays_unknown_and_keeps_a_zero_health_peer_count() {
        let status = assemble_status(
            "http://127.0.0.1:8545/rpc",
            None,
            Some(12),
            None,
            None,
            Vec::new(),
            Some(0),
        );
        assert_eq!(status.sync, SyncState::Unknown);
        assert_eq!(status.height, Some(12));
        assert_eq!(status.peer_count, Some(0));
        assert_eq!(status_json(&status)["syncing"], Value::Null);
        let rows = status_rows(&status);
        assert_eq!(row(&rows, "Sync"), "unknown");
        assert_eq!(row(&rows, "Peers"), "0");
        assert_eq!(row(&rows, "Round"), "unknown");

        let absent = assemble_status(
            "http://127.0.0.1:8545/rpc",
            None,
            Some(12),
            None,
            None,
            Vec::new(),
            None,
        );
        assert_eq!(row(&status_rows(&absent), "Peers"), "unknown");
    }

    fn row<'a>(rows: &'a [(&'static str, String)], key: &str) -> &'a str {
        &rows.iter().find(|(label, _)| *label == key).unwrap().1
    }

    #[test]
    fn balance_is_shown_in_kuna_with_nine_decimals() {
        let address = Address([0x11; 32]);
        let report = balance_report(&address, 1_500_000_000);
        assert_eq!(report["balance"], json!(1_500_000_000));
        assert_eq!(report["balanceKuna"], json!("1.500000000"));
        assert_eq!(report["address"], json!(address.to_string()));
        let rows = balance_rows(&address, 1);
        assert_eq!(rows[1].1, "0.000000001 KUNA");
        assert_eq!(balance_rows(&address, 0)[1].1, "0.000000000 KUNA");
    }
}
