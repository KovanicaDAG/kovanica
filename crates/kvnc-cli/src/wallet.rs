//! Keystore operations.
//!
//! The on-disk format and the Argon2id parameters live in `kvnc-keystore`.
//! This module re-exports that crate so existing `kvnc_cli::wallet` callers
//! keep the same paths. The format is unchanged.

pub use kvnc_keystore::*;
