//! Relay HTTP API v1 — `docs/COMMS-M1-INTERFACES.md` §4.
//!
//! axum 0.7 server exposing exactly:
//!
//! | Method | Path | Success | Errors |
//! |---|---|---|---|
//! | POST | `/v1/envelope` | `201 {"ok":true,"id":"<hex>"}` | `400`,`413`,`429` |
//! | GET  | `/v1/inbox/{comms_id}` | `200 {"envelopes":[…]}` | `400` |
//! | POST | `/v1/ack` | `200 {"ok":true,"deleted":<n>}` | `400` |
//! | GET  | `/v1/health` | `200 {"ok":true,"version":…,"envelopes":<n>}` | — |
//!
//! Errors are `{"error":{"code":"<slug>","message":"…"}}` with slugs
//! `bad_request`, `too_large`, `not_found`, `rate_limited`, `internal`.
//! **GET never deletes** — deletion is only via `/v1/ack`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::envelope::{Envelope, MAX_CT, MAX_ENVELOPE_JSON};
use crate::store::{Store, StoreError, DEFAULT_INBOX_LIMIT, MAX_INBOX_LIMIT};
use crate::now_ms;

/// How often the background task sweeps expired envelopes.
pub const SWEEP_INTERVAL: Duration = Duration::from_secs(60);

/// Error slugs (`error.code`) used by §4.
pub mod slug {
    pub const BAD_REQUEST: &str = "bad_request";
    pub const TOO_LARGE: &str = "too_large";
    pub const NOT_FOUND: &str = "not_found";
    pub const RATE_LIMITED: &str = "rate_limited";
    pub const INTERNAL: &str = "internal";
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

fn error_response(status: StatusCode, code: &'static str, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorEnvelope {
            error: ErrorBody {
                code,
                message: message.into(),
            },
        }),
    )
        .into_response()
}

fn bad_request(message: impl Into<String>) -> Response {
    error_response(StatusCode::BAD_REQUEST, slug::BAD_REQUEST, message)
}

fn too_large(message: impl Into<String>) -> Response {
    error_response(StatusCode::PAYLOAD_TOO_LARGE, slug::TOO_LARGE, message)
}

fn rate_limited() -> Response {
    error_response(
        StatusCode::TOO_MANY_REQUESTS,
        slug::RATE_LIMITED,
        "recipient quota exceeded",
    )
}

#[derive(Debug, Serialize)]
struct PostEnvelopeResponse {
    ok: bool,
    id: String,
}

/// `POST /v1/envelope` — validate and store an opaque envelope.
async fn post_envelope(State(store): State<Arc<Store>>, body: Bytes) -> Response {
    if body.len() > MAX_ENVELOPE_JSON {
        return too_large(format!(
            "envelope json exceeds {MAX_ENVELOPE_JSON} bytes"
        ));
    }

    let envelope = match Envelope::from_json(&body) {
        Ok(envelope) => envelope,
        Err(e) => return bad_request(format!("malformed envelope: {e}")),
    };
    if let Err(e) = envelope.validate_shape() {
        return bad_request(format!("invalid envelope: {e}"));
    }
    if envelope.ct.len() > MAX_CT {
        return too_large(format!("ct exceeds {MAX_CT} bytes"));
    }
    match envelope.compute_id() {
        Ok(id) if id == envelope.id => {}
        Ok(_) => return bad_request("envelope id does not match its contents"),
        Err(e) => return bad_request(format!("invalid envelope: {e}")),
    }

    let id = envelope.id.clone();
    match store.insert(envelope, now_ms()) {
        Ok(_outcome) => (
            StatusCode::CREATED,
            Json(PostEnvelopeResponse { ok: true, id }),
        )
            .into_response(),
        Err(StoreError::RateLimited) => rate_limited(),
    }
}

#[derive(Debug, Deserialize)]
struct InboxQuery {
    limit: Option<usize>,
    since: Option<u64>,
}

#[derive(Debug, Serialize)]
struct InboxResponse {
    envelopes: Vec<Envelope>,
}

/// `GET /v1/inbox/{comms_id}` — undelivered envelopes, oldest first. Never deletes.
async fn get_inbox(
    State(store): State<Arc<Store>>,
    Path(comms_id): Path<String>,
    Query(query): Query<InboxQuery>,
) -> Response {
    if crate::comms_id_from_hex(&comms_id).is_err() {
        return bad_request("invalid comms id");
    }
    let limit = query.limit.unwrap_or(DEFAULT_INBOX_LIMIT).min(MAX_INBOX_LIMIT);
    let envelopes = store.inbox(&comms_id, limit, query.since, now_ms());
    (
        StatusCode::OK,
        Json(InboxResponse { envelopes }),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
struct AckRequest {
    to: String,
    ids: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AckResponse {
    ok: bool,
    deleted: usize,
}

/// `POST /v1/ack` — the only delete path.
async fn post_ack(State(store): State<Arc<Store>>, Json(req): Json<AckRequest>) -> Response {
    if crate::comms_id_from_hex(&req.to).is_err() {
        return bad_request("invalid comms id");
    }
    for id in &req.ids {
        if !is_lower_hex64(id) {
            return bad_request("invalid envelope id");
        }
    }
    let deleted = store.ack(&req.to, &req.ids);
    (
        StatusCode::OK,
        Json(AckResponse { ok: true, deleted }),
    )
        .into_response()
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    ok: bool,
    version: &'static str,
    envelopes: usize,
}

/// `GET /v1/health`.
async fn get_health(State(store): State<Arc<Store>>) -> Response {
    (
        StatusCode::OK,
        Json(HealthResponse {
            ok: true,
            version: env!("CARGO_PKG_VERSION"),
            envelopes: store.len(),
        }),
    )
        .into_response()
}

fn is_lower_hex64(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Build the §4 router around a shared [`Store`].
pub fn router(store: Arc<Store>) -> Router {
    Router::new()
        .route("/v1/envelope", post(post_envelope))
        .route("/v1/inbox/:comms_id", get(get_inbox))
        .route("/v1/ack", post(post_ack))
        .route("/v1/health", get(get_health))
        .with_state(store)
}

/// The relay HTTP server. Mirrors the `kvnc-rpc` [`RpcServer`] shape
/// (`router()` / `start()`).
///
/// [`RpcServer`]: https://docs.rs/kvnc-rpc
#[derive(Clone)]
pub struct RelayServer {
    addr: SocketAddr,
    store: Arc<Store>,
}

impl RelayServer {
    /// Create a server with a fresh, empty store.
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            store: Arc::new(Store::new()),
        }
    }

    /// Create a server sharing an existing store (tests, embedding).
    pub fn with_store(addr: SocketAddr, store: Arc<Store>) -> Self {
        Self { addr, store }
    }

    /// The shared store backing this server.
    pub fn store(&self) -> Arc<Store> {
        self.store.clone()
    }

    /// The configured bind address.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Build the §4 router (exposed for tests and embedded use).
    pub fn router(&self) -> Router {
        router(self.store.clone())
    }

    /// Bind, serve, and return the background task handle.
    ///
    /// Shuts down gracefully on `SIGINT`/`SIGTERM`. Also spawns the periodic
    /// expiry sweep for the lifetime of the store.
    pub async fn start(self) -> Result<JoinHandle<()>, std::io::Error> {
        let app = self.router();
        let listener = TcpListener::bind(self.addr).await?;
        let actual_addr = listener.local_addr()?;
        info!("kvnc-comms-relay listening on http://{}", actual_addr);

        spawn_sweeper(self.store.clone());

        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await
            {
                error!(error = %e, "relay server stopped with an error");
            }
        });

        Ok(handle)
    }
}

/// Spawn the background expiry sweep (drops envelopes past `expiresAt`).
///
/// Returns the task handle; the task runs until the process exits.
pub fn spawn_sweeper(store: Arc<Store>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(SWEEP_INTERVAL);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            let dropped = store.sweep(now_ms());
            if dropped > 0 {
                info!(dropped, "relay sweep dropped expired envelopes");
            }
        }
    })
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    warn!("relay shutdown signal received");
}
