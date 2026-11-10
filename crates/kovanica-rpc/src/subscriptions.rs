//! WebSocket subscription support for the Kovanica JSON-RPC server.
//!
//! The RPC server exposes a WebSocket endpoint (`GET /ws`) alongside the
//! existing HTTP `POST /rpc` endpoint. Clients open a socket, issue
//! `subscribe` / `unsubscribe` JSON-RPC requests and receive push
//! notifications for node events.
//!
//! # Supported subscriptions
//!
//! | Method                     | Emitted when                                            |
//! |----------------------------|---------------------------------------------------------|
//! | `newHeads`                 | A new block is produced or accepted by the node.        |
//! | `newCommittedLeader`       | Consensus commits a leader sub-DAG.                     |
//! | `pendingTransactions`      | A transaction enters the mempool.                       |
//! | `logs`                     | Transaction execution logs (receipts) emitted per committed sub-DAG. |
//! | `newSealedHeader`          | A committed-leader header was stored, or a matching seal arrived. |
//!
//! Transaction execution logs (`logs`) are emitted for each committed sub-DAG
//! and contain the transaction receipts (status, gas used, logs, etc.).
//!
//! # Wire protocol
//!
//! Subscribe request (params may be a bare string or a single-element array):
//!
//! ```json
//! {"jsonrpc":"2.0","id":1,"method":"subscribe","params":["newHeads"]}
//! ```
//!
//! Success response (`result` is a hex quantity string):
//!
//! ```json
//! {"jsonrpc":"2.0","id":1,"result":"0x1"}
//! ```
//!
//! Notifications use the standard subscription envelope:
//!
//! ```json
//! {"jsonrpc":"2.0","method":"kovanica_subscription",
//!  "params":{"subscription":"0x1","result":{ ... }}}
//! ```
//!
//! Unsubscribe accepts the id returned by `subscribe` (hex string or number):
//!
//! ```json
//! {"jsonrpc":"2.0","id":2,"method":"unsubscribe","params":["0x1"]}
//! ```
//!
//! Subscription ids are scoped to a single WebSocket connection.

use std::collections::HashMap;

use serde_json::{json, Value};
use tokio::sync::broadcast;

use crate::chain_methods::{block_to_json, transaction_to_json};
use crate::{JsonRpcErrorObject, JsonRpcRequest, JsonRpcResponse, RpcError};
use kovanica_execution::TransactionReceipt;
use kovanica_types::{
    Address, CommitSeal, CommittedSubDag, LeaderHeader, StatementBlock, Transaction,
};

/// Capacity of each broadcast channel. Slow consumers that fall further than
/// this behind simply drop missed notifications (a `Lagged` receive error is
/// treated as "no event") rather than stalling the publisher.
const CHANNEL_CAPACITY: usize = 1024;

/// The set of events a client can subscribe to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SubscriptionKind {
    /// A new block was produced or accepted into the DAG.
    NewHeads,
    /// Consensus committed a leader sub-DAG.
    NewCommittedLeader,
    /// A transaction entered the mempool.
    PendingTransactions,
    /// Transaction execution logs (receipts) emitted per committed sub-DAG.
    Logs,
    /// A locally executed leader header, with the seals stored for it.
    NewSealedHeader,
}

impl SubscriptionKind {
    /// Canonical wire name.
    pub fn as_str(self) -> &'static str {
        match self {
            SubscriptionKind::NewHeads => "newHeads",
            SubscriptionKind::NewCommittedLeader => "newCommittedLeader",
            SubscriptionKind::PendingTransactions => "pendingTransactions",
            SubscriptionKind::Logs => "logs",
            SubscriptionKind::NewSealedHeader => "newSealedHeader",
        }
    }

    /// Parse a subscription name, accepting a few common snake_case aliases.
    pub fn parse(name: &str) -> Result<Self, String> {
        match name {
            "newHeads" | "new_heads" => Ok(SubscriptionKind::NewHeads),
            "newCommittedLeader" | "new_committed_leader" => {
                Ok(SubscriptionKind::NewCommittedLeader)
            }
            "pendingTransactions" | "pending_transactions" => {
                Ok(SubscriptionKind::PendingTransactions)
            }
            "logs" => Ok(SubscriptionKind::Logs),
            "newSealedHeader" | "new_sealed_header" => Ok(SubscriptionKind::NewSealedHeader),
            other => Err(format!("unsupported subscription: {other}")),
        }
    }
}

/// Fan-out bus connecting node event sources to WebSocket subscribers.
///
/// Cheap to clone; all clones share the same channels.
#[derive(Clone)]
pub struct EventBus {
    new_heads: broadcast::Sender<Value>,
    committed_leaders: broadcast::Sender<Value>,
    pending_transactions: broadcast::Sender<Value>,
    logs: broadcast::Sender<Value>,
    sealed_headers: broadcast::Sender<Value>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Create a new bus with empty channels.
    pub fn new() -> Self {
        let (new_heads, _) = broadcast::channel(CHANNEL_CAPACITY);
        let (committed_leaders, _) = broadcast::channel(CHANNEL_CAPACITY);
        let (pending_transactions, _) = broadcast::channel(CHANNEL_CAPACITY);
        let (logs, _) = broadcast::channel(CHANNEL_CAPACITY);
        let (sealed_headers, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self {
            new_heads,
            committed_leaders,
            pending_transactions,
            logs,
            sealed_headers,
        }
    }

    /// Subscribe to the raw event stream for `kind`.
    pub fn subscribe_receiver(&self, kind: SubscriptionKind) -> broadcast::Receiver<Value> {
        match kind {
            SubscriptionKind::NewHeads => self.new_heads.subscribe(),
            SubscriptionKind::NewCommittedLeader => self.committed_leaders.subscribe(),
            SubscriptionKind::PendingTransactions => self.pending_transactions.subscribe(),
            SubscriptionKind::Logs => self.logs.subscribe(),
            SubscriptionKind::NewSealedHeader => self.sealed_headers.subscribe(),
        }
    }

    /// Broadcast a newly produced/accepted block. No-op if nobody listens.
    pub fn publish_new_head(&self, block: &StatementBlock) {
        let _ = self.new_heads.send(block_to_json(block));
    }

    /// Broadcast a committed leader sub-DAG. No-op if nobody listens.
    pub fn publish_committed_leader(&self, subdag: &CommittedSubDag) {
        let payload = json!({
            "leader": block_to_json(&subdag.leader),
            "leaderRound": subdag.leader_round,
            "leaderAuthor": subdag.leader_author,
            "blocks": subdag
                .blocks
                .iter()
                .map(block_to_json)
                .collect::<Vec<Value>>(),
        });
        let _ = self.committed_leaders.send(payload);
    }

    /// Broadcast a mempool admission. No-op if nobody listens.
    pub fn publish_pending_transaction(&self, tx: &Transaction) {
        let _ = self.pending_transactions.send(transaction_to_json(tx));
    }

    /// Broadcast a transaction execution log. No-op if nobody listens.
    pub fn publish_logs(&self, receipts: &[TransactionReceipt]) {
        let _ = self.logs.send(json!(receipts));
    }

    /// Broadcast a leader header and the seals stored for its height.
    ///
    /// No-op if nobody listens. Consensus does not wait on this event.
    pub fn publish_sealed_header(&self, header: &LeaderHeader, seals: &[CommitSeal]) {
        let _ = self
            .sealed_headers
            .send(crate::chain_methods::sealed_header_json(header, seals));
    }
}

/// Optional filter for `logs` subscriptions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogFilter {
    /// Addresses to filter by (or None for any address).
    pub addresses: Option<Vec<Address>>,
    /// Topics to filter by (per-position alternatives, per Ethereum / kovanica spec).
    pub topics: Vec<Option<Vec<Vec<u8>>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ActiveSubscription {
    kind: SubscriptionKind,
    is_eth: bool,
    log_filter: Option<LogFilter>,
}

/// Per-connection subscription registry.
struct Connection {
    chain_id: u64,
    next_id: u64,
    subscriptions: HashMap<u64, ActiveSubscription>,
}

impl Default for Connection {
    fn default() -> Self {
        Self::new(1337)
    }
}

impl Connection {
    fn new(chain_id: u64) -> Self {
        Self {
            chain_id,
            next_id: 1,
            subscriptions: HashMap::new(),
        }
    }

    /// Handle a single inbound JSON-RPC frame, returning the response text.
    fn handle_request(&mut self, text: &str) -> String {
        let request: JsonRpcRequest = match serde_json::from_str(text) {
            Ok(request) => request,
            Err(e) => return error_response(Value::Null, RpcError::ParseError(e.to_string())),
        };

        if request.jsonrpc != "2.0" {
            return error_response(
                request.id.unwrap_or(Value::Null),
                RpcError::InvalidRequest("Invalid JSON-RPC version".into()),
            );
        }

        let id = request.id.clone().unwrap_or(Value::Null);

        match request.method.as_str() {
            "subscribe" | "kovanica_subscribe" => match self.subscribe(request.params, false) {
                Ok(subscription_id) => success_response(id, Value::String(subscription_id)),
                Err(e) => error_response(id, e),
            },
            "eth_subscribe" => match self.subscribe(request.params, true) {
                Ok(subscription_id) => success_response(id, Value::String(subscription_id)),
                Err(e) => error_response(id, e),
            },
            "unsubscribe" | "kovanica_unsubscribe" | "eth_unsubscribe" => {
                match self.unsubscribe(request.params) {
                    Ok(removed) => success_response(id, Value::Bool(removed)),
                    Err(e) => error_response(id, e),
                }
            }
            other => error_response(id, RpcError::MethodNotFound(other.to_string())),
        }
    }

    fn subscribe(&mut self, params: Option<Value>, is_eth: bool) -> Result<String, RpcError> {
        let (name, filter_obj) = parse_subscribe_params(params)?;

        let kind = SubscriptionKind::parse(&name)
            .map_err(|e| RpcError::InvalidParams(format!("subscribe: {e}")))?;

        let log_filter = if kind == SubscriptionKind::Logs {
            if let Some(ref obj) = filter_obj {
                if !obj.is_null() && obj.is_object() {
                    let addresses = crate::chain_methods::parse_address_filter(
                        obj.get("address"),
                        self.chain_id,
                    )?;
                    let topics = crate::chain_methods::parse_topics_filter(obj.get("topics"))?;
                    Some(LogFilter { addresses, topics })
                } else if !obj.is_null() {
                    return Err(RpcError::InvalidParams(
                        "subscribe: logs filter must be an object".into(),
                    ));
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let subscription_id = self.next_id;
        self.next_id += 1;
        self.subscriptions.insert(
            subscription_id,
            ActiveSubscription {
                kind,
                is_eth,
                log_filter,
            },
        );

        Ok(format!("0x{subscription_id:x}"))
    }

    fn unsubscribe(&mut self, params: Option<Value>) -> Result<bool, RpcError> {
        let subscription_id = first_param(params)
            .and_then(parse_subscription_id)
            .ok_or_else(|| {
                RpcError::InvalidParams("unsubscribe: expected a subscription id".into())
            })?;

        Ok(self.subscriptions.remove(&subscription_id).is_some())
    }
}

fn parse_subscribe_params(params: Option<Value>) -> Result<(String, Option<Value>), RpcError> {
    match params {
        Some(Value::Array(mut values)) => {
            if values.is_empty() {
                return Err(RpcError::InvalidParams(
                    "subscribe: expected a subscription name".into(),
                ));
            }
            let name_val = values.remove(0);
            let name = name_val
                .as_str()
                .ok_or_else(|| {
                    RpcError::InvalidParams("subscribe: expected a subscription name string".into())
                })?
                .to_string();
            let filter = values.into_iter().next();
            Ok((name, filter))
        }
        Some(Value::String(name)) => Ok((name, None)),
        _ => Err(RpcError::InvalidParams(
            "subscribe: expected a subscription name".into(),
        )),
    }
}

/// Extract the first parameter, flattening a single-element array.
fn first_param(params: Option<Value>) -> Option<Value> {
    match params {
        Some(Value::Array(values)) => values.into_iter().next(),
        Some(Value::Null) | None => None,
        Some(value) => Some(value),
    }
}

/// Parse a subscription id given as a hex string, decimal string or number.
fn parse_subscription_id(value: Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(raw) => {
            let raw = raw.trim();
            match raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
                Some(hex) => u64::from_str_radix(hex, 16).ok(),
                None => raw.parse::<u64>().ok(),
            }
        }
        _ => None,
    }
}

fn success_response(id: Value, result: Value) -> String {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id: Some(id),
    };
    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

fn error_response(id: Value, error: RpcError) -> String {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(JsonRpcErrorObject {
            code: error.code(),
            message: error.to_string(),
            data: None,
        }),
        id: Some(id),
    };
    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

/// Filter a logs event (array of TransactionReceipts) according to a LogFilter.
/// Returns None if no receipts match.
fn filter_logs_event(event: &Value, filter: &LogFilter, chain_id: u64) -> Option<Value> {
    let receipts = event.as_array()?;
    let mut matching_receipts = Vec::new();

    for receipt in receipts {
        let events = receipt.get("events").and_then(|e| e.as_array());
        let mut matched = false;

        if let Some(events_arr) = events {
            for ev in events_arr {
                // Address filter
                if let Some(ref addrs) = filter.addresses {
                    let ev_addr = if let Some(arr) = ev.get("address").and_then(|a| a.as_array()) {
                        if arr.len() == 32 {
                            let mut bytes = [0u8; 32];
                            for (i, b) in arr.iter().enumerate() {
                                bytes[i] = b.as_u64().unwrap_or(0) as u8;
                            }
                            Some(Address(bytes))
                        } else {
                            None
                        }
                    } else if let Some(a_str) = ev.get("address").and_then(|a| a.as_str()) {
                        Address::parse_for_chain(a_str, chain_id).ok().or_else(|| {
                            addrs
                                .iter()
                                .find(|a| {
                                    a.to_string() == a_str
                                        || a.encode() == a_str
                                        || a.to_hex() == a_str
                                })
                                .copied()
                        })
                    } else {
                        None
                    };

                    match ev_addr {
                        Some(addr) if addrs.contains(&addr) => {}
                        _ => continue,
                    }
                }

                // Topic filter
                let topic_bytes: Vec<u8> =
                    if let Some(arr) = ev.get("topic").and_then(|t| t.as_array()) {
                        arr.iter()
                            .filter_map(|v| v.as_u64().map(|b| b as u8))
                            .collect()
                    } else if let Some(s) = ev.get("topic").and_then(|t| t.as_str()) {
                        let clean = s
                            .strip_prefix("0x")
                            .or_else(|| s.strip_prefix("0X"))
                            .unwrap_or(s);
                        hex::decode(clean).unwrap_or_else(|_| s.as_bytes().to_vec())
                    } else {
                        Vec::new()
                    };

                if crate::chain_methods::topics_match(&filter.topics, &topic_bytes) {
                    matched = true;
                    break;
                }
            }
        }

        if matched {
            matching_receipts.push(receipt.clone());
        }
    }

    if matching_receipts.is_empty() {
        None
    } else {
        Some(Value::Array(matching_receipts))
    }
}

/// All notification frames owed to `connection` for an event of `kind`.
///
/// Returns an empty vector when the receiver lagged / closed or when the
/// connection has no matching subscriptions.
fn notifications_for(
    connection: &Connection,
    kind: SubscriptionKind,
    received: Result<Value, broadcast::error::RecvError>,
) -> Vec<Value> {
    let Ok(event) = received else {
        return Vec::new();
    };

    let mut ids: Vec<u64> = connection
        .subscriptions
        .iter()
        .filter(|(_, sub)| sub.kind == kind)
        .map(|(id, _)| *id)
        .collect();
    ids.sort_unstable();

    ids.into_iter()
        .filter_map(|id| {
            let sub = connection.subscriptions.get(&id)?;
            let method = if sub.is_eth {
                "eth_subscription"
            } else {
                "kovanica_subscription"
            };

            let payload = if kind == SubscriptionKind::Logs {
                if let Some(ref filter) = sub.log_filter {
                    filter_logs_event(&event, filter, connection.chain_id)?
                } else {
                    event.clone()
                }
            } else {
                event.clone()
            };

            Some(json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": {
                    "subscription": format!("0x{id:x}"),
                    "result": payload,
                }
            }))
        })
        .collect()
}

/// Drive a single upgraded WebSocket connection until the peer disconnects.
pub async fn serve_connection(socket: axum::extract::ws::WebSocket, state: crate::RpcState) {
    use axum::extract::ws::Message;
    use futures::{SinkExt, StreamExt};

    let chain_id = state.mempool.signing_context().chain_id;
    let mut connection = Connection::new(chain_id);
    let mut new_heads = state.events.subscribe_receiver(SubscriptionKind::NewHeads);
    let mut committed_leaders = state
        .events
        .subscribe_receiver(SubscriptionKind::NewCommittedLeader);
    let mut pending = state
        .events
        .subscribe_receiver(SubscriptionKind::PendingTransactions);
    let mut logs = state.events.subscribe_receiver(SubscriptionKind::Logs);
    let mut sealed_headers = state
        .events
        .subscribe_receiver(SubscriptionKind::NewSealedHeader);

    let (mut sender, mut receiver) = socket.split();

    loop {
        tokio::select! {
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        let response = connection.handle_request(&text);
                        if sender
                            .send(Message::Text(response))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        if sender
                            .send(Message::Pong(payload))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            event = new_heads.recv() => {
                let frames = notifications_for(&connection, SubscriptionKind::NewHeads, event);
                if send_frames(&mut sender, frames).await.is_err() {
                    break;
                }
            }
            event = committed_leaders.recv() => {
                let frames = notifications_for(&connection, SubscriptionKind::NewCommittedLeader, event);
                if send_frames(&mut sender, frames).await.is_err() {
                    break;
                }
            }
            event = pending.recv() => {
                let frames = notifications_for(&connection, SubscriptionKind::PendingTransactions, event);
                if send_frames(&mut sender, frames).await.is_err() {
                    break;
                }
            }
            event = logs.recv() => {
                let frames = notifications_for(&connection, SubscriptionKind::Logs, event);
                if send_frames(&mut sender, frames).await.is_err() {
                    break;
                }
            }
            event = sealed_headers.recv() => {
                let frames = notifications_for(&connection, SubscriptionKind::NewSealedHeader, event);
                if send_frames(&mut sender, frames).await.is_err() {
                    break;
                }
            }
        }
    }
}

async fn send_frames<S>(sender: &mut S, frames: Vec<Value>) -> Result<(), ()>
where
    S: futures::Sink<axum::extract::ws::Message, Error = axum::Error> + Unpin,
{
    use futures::SinkExt;
    for frame in frames {
        sender
            .send(axum::extract::ws::Message::Text(frame.to_string()))
            .await
            .map_err(|_| ())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request(connection: &mut Connection, value: Value) -> Value {
        let raw = serde_json::to_string(&value).expect("serialise request");
        let response = connection.handle_request(&raw);
        serde_json::from_str(&response).expect("parse response")
    }

    #[test]
    fn parses_canonical_and_alias_subscription_names() {
        assert_eq!(
            SubscriptionKind::parse("newHeads").unwrap(),
            SubscriptionKind::NewHeads
        );
        assert_eq!(
            SubscriptionKind::parse("new_committed_leader").unwrap(),
            SubscriptionKind::NewCommittedLeader
        );
        assert_eq!(
            SubscriptionKind::parse("pendingTransactions").unwrap(),
            SubscriptionKind::PendingTransactions
        );
        assert_eq!(
            SubscriptionKind::parse("logs").unwrap(),
            SubscriptionKind::Logs
        );
        assert_eq!(
            SubscriptionKind::parse("newSealedHeader").unwrap(),
            SubscriptionKind::NewSealedHeader
        );
        assert!(SubscriptionKind::parse("blocks").is_err());
    }

    #[test]
    fn subscribe_returns_hex_id_and_unsubscribe_removes_it() {
        let mut connection = Connection::default();
        let response = request(
            &mut connection,
            json!({"jsonrpc":"2.0","id":1,"method":"subscribe","params":["newHeads"]}),
        );
        assert_eq!(response["result"], json!("0x1"));

        let response = request(
            &mut connection,
            json!({"jsonrpc":"2.0","id":2,"method":"unsubscribe","params":["0x1"]}),
        );
        assert_eq!(response["result"], json!(true));

        // Unsubscribing twice reports the subscription no longer exists.
        let response = request(
            &mut connection,
            json!({"jsonrpc":"2.0","id":3,"method":"unsubscribe","params":["0x1"]}),
        );
        assert_eq!(response["result"], json!(false));
    }

    #[test]
    fn subscribe_accepts_bare_string_param_variant() {
        let mut connection = Connection::default();
        let response = request(
            &mut connection,
            json!({"jsonrpc":"2.0","id":1,"method":"subscribe","params":"newCommittedLeader"}),
        );
        assert_eq!(response["result"], json!("0x1"));
    }

    #[test]
    fn unknown_subscription_is_an_invalid_params_error() {
        let mut connection = Connection::default();
        let response = request(
            &mut connection,
            json!({"jsonrpc":"2.0","id":1,"method":"subscribe","params":["nope"]}),
        );
        assert_eq!(response["error"]["code"], json!(-32602));
    }

    #[test]
    fn notifications_only_reach_matching_subscriptions() {
        let mut connection = Connection::default();
        request(
            &mut connection,
            json!({"jsonrpc":"2.0","id":1,"method":"subscribe","params":["newHeads"]}),
        );
        request(
            &mut connection,
            json!({"jsonrpc":"2.0","id":2,"method":"subscribe","params":["pendingTransactions"]}),
        );

        let event = json!({"hash": "0xabc"});
        let frames = notifications_for(&connection, SubscriptionKind::NewHeads, Ok(event.clone()));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0]["method"], json!("kovanica_subscription"));
        assert_eq!(frames[0]["params"]["subscription"], json!("0x1"));
        assert_eq!(frames[0]["params"]["result"], event);

        // A lagged receiver yields no frames rather than an error.
        let frames = notifications_for(
            &connection,
            SubscriptionKind::NewHeads,
            Err(broadcast::error::RecvError::Lagged(3)),
        );
        assert!(frames.is_empty());
    }

    #[test]
    fn eth_subscribe_and_unsubscribe_with_filter() {
        let mut connection = Connection::new(1337);
        let response = request(
            &mut connection,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "eth_subscribe",
                "params": ["logs", {
                    "topics": ["transfer"]
                }]
            }),
        );
        assert_eq!(response["result"], json!("0x1"));

        // Event matching topic "transfer"
        let matching_event = json!([{
            "tx_hash": "0x1111",
            "success": true,
            "gas_used": 21000,
            "error": null,
            "events": [{
                "address": "kvnc1111111111111111111111111111111111111111111111111111111111dag",
                "topic": b"transfer".to_vec(),
                "data": [1, 2, 3]
            }]
        }]);

        let frames = notifications_for(&connection, SubscriptionKind::Logs, Ok(matching_event));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0]["method"], json!("eth_subscription"));
        assert_eq!(frames[0]["params"]["subscription"], json!("0x1"));
        assert_eq!(frames[0]["params"]["result"][0]["gas_used"], json!(21000));

        // Event with non-matching topic "approval"
        let non_matching_event = json!([{
            "tx_hash": "0x2222",
            "success": true,
            "gas_used": 21000,
            "error": null,
            "events": [{
                "address": "kvnc1111111111111111111111111111111111111111111111111111111111dag",
                "topic": b"approval".to_vec(),
                "data": [1, 2, 3]
            }]
        }]);

        let frames = notifications_for(&connection, SubscriptionKind::Logs, Ok(non_matching_event));
        assert!(
            frames.is_empty(),
            "filtered subscriber should not receive non-matching event"
        );

        // eth_unsubscribe
        let response = request(
            &mut connection,
            json!({"jsonrpc": "2.0", "id": 2, "method": "eth_unsubscribe", "params": ["0x1"]}),
        );
        assert_eq!(response["result"], json!(true));
    }
}
