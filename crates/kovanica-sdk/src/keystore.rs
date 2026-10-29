//! Keystore integration helpers.

#[cfg(feature = "keystore")]
use crate::error::SdkError;
#[cfg(feature = "keystore")]
use kovanica_keystore::Keystore;

#[cfg(feature = "keystore")]
pub fn signing_key_from_keystore(
    ks: &Keystore,
    passphrase: Option<&str>,
) -> Result<kovanica_types::crypto::SigningKey, SdkError> {
    kovanica_keystore::signing_key(ks, passphrase).map_err(|e| SdkError::Signing(e.to_string()))
}
