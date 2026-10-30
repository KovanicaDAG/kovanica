//! HTTP + WebSocket server for the indexer.

use crate::indexer::Indexer;
use crate::types::{IndexedEvent, LogFilter, WsFilter};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, Default)]
pub struct HttpFilter {
    #[serde(alias = "fromBlock", alias = "from_height", alias = "from")]
    pub from_height: Option<u64>,
    #[serde(alias = "toBlock", alias = "to_height", alias = "to")]
    pub to_height: Option<u64>,
    pub address: Option<String>,
    pub topic: Option<String>,
    pub limit: Option<usize>,
}

impl From<HttpFilter> for LogFilter {
    fn from(f: HttpFilter) -> Self {
        LogFilter {
            from_height: f.from_height,
            to_height: f.to_height,
            address: f.address,
            topic: f.topic,
            limit: f.limit,
        }
    }
}

pub fn router(indexer: Arc<Indexer>) -> Router {
    Router::new()
        .route("/logs", get(handle_logs))
        .route("/events", get(handle_logs))
        .route("/contracts/:address/logs", get(handle_contract_logs))
        .route("/contracts/:address/events", get(handle_contract_logs))
        .route("/status", get(handle_status))
        .route("/ws", get(handle_ws))
        .with_state(indexer)
}

async fn handle_logs(
    State(idx): State<Arc<Indexer>>,
    Query(f): Query<HttpFilter>,
) -> Result<Json<Vec<IndexedEvent>>, StatusCode> {
    let filter: LogFilter = f.into();
    let events = idx
        .store
        .query(&filter)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(events))
}

async fn handle_contract_logs(
    State(idx): State<Arc<Indexer>>,
    Path(address): Path<String>,
    Query(f): Query<HttpFilter>,
) -> Result<Json<Vec<IndexedEvent>>, StatusCode> {
    let mut filter: LogFilter = f.into();
    filter.address = Some(address);
    let events = idx
        .store
        .query(&filter)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(events))
}

async fn handle_status(State(idx): State<Arc<Indexer>>) -> Json<crate::types::IndexerStatus> {
    Json(idx.status().await)
}

async fn handle_ws(ws: WebSocketUpgrade, State(idx): State<Arc<Indexer>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_loop(socket, idx))
}

async fn ws_loop(mut socket: WebSocket, idx: Arc<Indexer>) {
    let mut rx = idx.tx.subscribe();
    let mut filter: WsFilter = WsFilter::default();

    // read initial filter (optional) with timeout
    // We'll just loop handling incoming messages and broadcast.
    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(sub) = v.get("subscribe") {
                                if let Ok(f) = serde_json::from_value::<WsFilter>(sub.clone()) {
                                    filter = f;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            ev = rx.recv() => {
                match ev {
                    Ok(event) => {
                        if !matches_filter(&filter, &event) { continue; }
                        let json = serde_json::to_string(&event).unwrap_or_default();
                        if socket.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
        }
    }
}

fn matches_filter(filter: &WsFilter, event: &IndexedEvent) -> bool {
    if let Some(addr) = &filter.address {
        if addr != &event.address.to_string() && addr != &hex::encode(event.address.0) {
            return false;
        }
    }
    if let Some(topics) = &filter.topics {
        let topic_str = String::from_utf8_lossy(&event.topic).to_string();
        let topic_hex = format!("0x{}", hex::encode(&event.topic));
        let ok = topics
            .iter()
            .any(|t| t == &topic_str || t == &topic_hex || t.as_bytes() == event.topic);
        if !ok {
            return false;
        }
    }
    true
}
