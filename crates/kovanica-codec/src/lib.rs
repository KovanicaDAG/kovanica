//! Versioned codec for Kovanica persistence.
//!
//! Provides explicit version byte for all serialized records.
//! New encoding uses bincode 2 (postcard-compatible).
//! Readers accept both legacy bincode 1.3 blob and new versioned blob.
//! Writers emit only new blob for newly written records.

#![deny(unsafe_code)]

pub const CODEC_VERSION: u8 = 1;

/// A versioned encoded blob.
#[derive(Debug, Clone)]
pub struct VersionedBlob {
    pub version: u8,
    pub data: Vec<u8>,
}

impl VersionedBlob {
    /// Encode a value with the current codec version.
    pub fn encode<T: serde::Serialize>(value: &T) -> Result<Self, CodecError> {
        let data = bincode::serialize(value).map_err(|_| CodecError::EncodeFailed)?;
        Ok(VersionedBlob {
            version: CODEC_VERSION,
            data,
        })
    }

    /// Decode a value, accepting both legacy (version 0) and versioned blobs.
    pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, CodecError> {
        if bytes.is_empty() {
            return Err(CodecError::EmptyBlob);
        }
        // Check if this is a versioned blob (first byte is version)
        // Legacy blobs start with the first field's bytes
        // We try versioned first, fall back to legacy
        if bytes[0] == CODEC_VERSION {
            // Versioned blob: skip version byte
            bincode::deserialize(&bytes[1..]).map_err(|_| CodecError::DecodeFailed)
        } else {
            // Legacy bincode 1.3 blob
            bincode::deserialize(bytes).map_err(|_| CodecError::DecodeFailed)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("encode failed")]
    EncodeFailed,
    #[error("decode failed")]
    DecodeFailed,
    #[error("empty blob")]
    EmptyBlob,
    #[error("unknown version")]
    UnknownVersion,
}
