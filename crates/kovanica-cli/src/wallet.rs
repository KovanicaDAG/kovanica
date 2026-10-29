//! Keystore operations.
//!
//! The on-disk format and the Argon2id parameters live in `kovanica-keystore`.
//! This module re-exports that crate so existing `kovanica_cli::wallet` callers
//! keep the same paths. The format is unchanged.

use anyhow::Result;
use zeroize::Zeroizing;

pub use kovanica_keystore::*;

/// Prompt for the keystore passphrase when the file is encrypted.
pub fn prompt_keystore_password(keystore: &Keystore) -> Result<Option<Zeroizing<String>>> {
    prompt_password_if_encrypted(keystore.encrypted)
}

/// Prompt for a passphrase for an encrypted keystore.
///
/// `KOVANICA_KEYSTORE_PASSWORD` is honoured for non-interactive use (scripts,
/// CI), otherwise the passphrase is read from the terminal without echo.
pub fn prompt_password_if_encrypted(encrypted: bool) -> Result<Option<Zeroizing<String>>> {
    if !encrypted {
        return Ok(None);
    }
    if let Ok(pass) = std::env::var("KOVANICA_KEYSTORE_PASSWORD") {
        return Ok(Some(Zeroizing::new(pass)));
    }
    Ok(Some(Zeroizing::new(rpassword::prompt_password(
        "Keystore passphrase: ",
    )?)))
}
