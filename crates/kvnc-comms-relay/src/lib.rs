//! Kovanica comms relay (M1) — store-and-forward for opaque envelopes.
//!
//! Spec: `docs/COMMS-M1-INTERFACES.md` §3 (envelope wire format v1) and §4
//! (relay HTTP API v1). Layer: **client-only infrastructure** — the relay never
//! receives plaintext, a key, or a seed. It validates shape, `id` consistency,
//! size and TTL, then stores the opaque blob until the recipient `ack`s it.
//!
//! Modules:
//! - [`envelope`] — §3.1 struct, §3.3 signing bytes, §3.4 id, seal+sign helper.
//! - [`payload`]  — §3.2 compact-JSON zero padding / trimming.
//! - [`store`]    — in-memory inbox: dedup, ordering, TTL clamp, quota, ack.
//! - [`server`]   — §4 axum router + [`RelayServer`].
//! - [`client`]   — reqwest [`RelayClient`].

#![deny(unsafe_code)]

pub mod client;
pub mod envelope;
pub mod payload;
pub mod server;
pub mod store;

pub use client::{ClientError, HealthResponse, RelayClient};
pub use envelope::{
    comms_id_bytes, comms_id_from_hex, comms_id_hex, Envelope, ENVELOPE_VERSION, ENV_ID_DOMAIN,
    ENV_SIG_DOMAIN, MAX_CT, MAX_ENVELOPE_JSON,
};
pub use payload::{pad_payload, trim_payload, MAX_PAYLOAD, PAYLOAD_BUCKETS};
pub use server::{router, RelayServer};
pub use store::{
    InsertOutcome, Store, StoreError, DEFAULT_INBOX_LIMIT, DEFAULT_TTL_MS, MAX_INBOX_LIMIT,
    MAX_PER_RECIPIENT, MAX_TTL_MS,
};

/// Algorithm id string for E2E crypto v1 (§2), re-exported from the crypto lane
/// so the envelope `alg` field and the cipher can never drift apart.
pub use kvnc_comms_crypto::ALG_V1;

use thiserror::Error;

/// Errors raised while encoding, decoding, validating or signing an envelope.
#[derive(Debug, Error)]
pub enum EnvelopeError {
    /// The JSON document could not be parsed or produced.
    #[error("invalid envelope json: {0}")]
    Json(#[from] serde_json::Error),
    /// A base64 field is not valid standard base64.
    #[error("invalid base64: {0}")]
    Base64(#[from] base64::DecodeError),
    /// A comms-id hex field is not valid hex.
    #[error("invalid hex: {0}")]
    Hex(#[from] hex::FromHexError),
    /// A comms id is not 64 lowercase hex characters.
    #[error("{0}")]
    BadHex(String),
    /// A fixed-width field has the wrong length.
    #[error("invalid length for {field}: expected {expected}, got {got}")]
    BadLength {
        field: &'static str,
        expected: usize,
        got: usize,
    },
    /// The envelope declares a version other than [`ENVELOPE_VERSION`].
    #[error("unsupported envelope version {0}")]
    UnsupportedVersion(u8),
    /// The envelope `alg` is not [`ALG_V1`].
    #[error("unsupported alg {0}")]
    AlgMismatch(String),
    /// The stored `id` does not match [`Envelope::compute_id`].
    #[error("envelope id does not match its contents")]
    IdMismatch,
    /// The `from` comms id does not match the supplied sender public key.
    #[error("sender comms id does not match the supplied public key")]
    SenderMismatch,
    /// The Ed25519 signature did not verify.
    #[error("invalid signature")]
    InvalidSignature,
    /// The payload is larger than the largest padding bucket.
    #[error("payload too large")]
    PayloadTooLarge,
    /// The padded payload could not be interpreted as compact JSON.
    #[error("invalid payload")]
    InvalidPayload,
    /// The E2E cipher failed.
    #[error("crypto error: {0}")]
    Crypto(#[from] kvnc_comms_crypto::CryptoError),
}

/// Wall-clock milliseconds since the Unix epoch.
///
/// Returns `0` if the system clock is before the epoch (never panics).
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
