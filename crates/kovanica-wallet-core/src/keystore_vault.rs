#![deny(unsafe_code)]

//! Keystore vault supporting HD 12/24-word entropy envelope and legacy ADR-0002 raw seeds.
//!
//! Uses Argon2id (m=65536 KiB, t=3, p=4) and XChaCha20-Poly1305 with Keystore v2 compatibility.
//! The envelope plaintext:
//! `0x02 || word_count (12 or 24) || entropy (16 or 32)`
//! Legacy 32-byte plaintexts decrypt cleanly and are classified as [`VaultPayload::LegacySeed`].

use argon2::{Algorithm, Argon2, Params, Version};
use bip39::{Language, Mnemonic};
use chacha20poly1305::{aead::Aead, KeyInit, XChaCha20Poly1305, XNonce};
use rand::{rngs::OsRng, RngCore};
use zeroize::{Zeroize, Zeroizing};

use kovanica_keystore::{Keystore, KEYSTORE_VERSION};
use kovanica_types::{Address, PublicKey, SigningKey};

use crate::account::WalletAccount;
use crate::error::WalletError;
use crate::hd::Slip10Node;

pub const VAULT_ENVELOPE_MAGIC: u8 = 0x02;

const KDF_MEMORY_KIB: u32 = 64 * 1024;
const KDF_ITERATIONS: u32 = 3;
const KDF_PARALLELISM: u32 = 4;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;

/// Decrypted vault payload containing either an HD entropy envelope or a legacy seed.
pub enum VaultPayload {
    /// Modern SLIP-0010 HD wallet with BIP-39 mnemonic entropy.
    Hd {
        word_count: u8,
        entropy: Zeroizing<Vec<u8>>,
    },
    /// Legacy ADR-0002 wallet whose 24 words represent a direct 32-byte Ed25519 seed.
    LegacySeed { seed: Zeroizing<[u8; 32]> },
}

impl Drop for VaultPayload {
    fn drop(&mut self) {
        match self {
            VaultPayload::Hd { entropy, .. } => {
                entropy.zeroize();
            }
            VaultPayload::LegacySeed { seed } => {
                seed.zeroize();
            }
        }
    }
}

impl VaultPayload {
    /// Returns true if this keystore payload represents a legacy ADR-0002 raw seed.
    pub fn is_legacy(&self) -> bool {
        matches!(self, VaultPayload::LegacySeed { .. })
    }

    /// Reconstruct the BIP-39 English mnemonic phrase from the stored entropy or legacy seed.
    pub fn mnemonic_phrase(&self) -> Result<String, WalletError> {
        match self {
            VaultPayload::Hd { entropy, .. } => {
                let m = Mnemonic::from_entropy_in(Language::English, entropy)
                    .map_err(|e| WalletError::Keystore(format!("invalid entropy in vault: {e}")))?;
                Ok(m.to_string())
            }
            VaultPayload::LegacySeed { seed } => {
                let m =
                    Mnemonic::from_entropy_in(Language::English, seed.as_slice()).map_err(|e| {
                        WalletError::Keystore(format!("invalid legacy seed entropy: {e}"))
                    })?;
                Ok(m.to_string())
            }
        }
    }

    /// Derive an account from the decrypted payload.
    ///
    /// For HD wallets, derives at `m/44'/3007'/<account_index>'/0'/0'` using the BIP-39 seed
    /// with an optional BIP-39 passphrase.
    ///
    /// For legacy wallets, ignores `account_index` and `bip39_passphrase` and returns the
    /// single keypair derived directly from the 32-byte seed.
    pub fn to_account(
        &self,
        bip39_passphrase: &str,
        account_index: u32,
    ) -> Result<WalletAccount, WalletError> {
        match self {
            VaultPayload::Hd { entropy, .. } => {
                let m = Mnemonic::from_entropy_in(Language::English, entropy)
                    .map_err(|e| WalletError::Keystore(format!("invalid entropy: {e}")))?;
                let seed_64 = m.to_seed(bip39_passphrase);
                let master = Slip10Node::master(&seed_64)?;
                let child = master.derive_bip44(account_index, 0, 0)?;
                Ok(WalletAccount::from_seed(&child.secret_key))
            }
            VaultPayload::LegacySeed { seed } => Ok(WalletAccount::from_seed(seed)),
        }
    }

    /// Generate a fresh HD wallet with 12 words (128-bit entropy) encrypted under `passphrase`.
    /// Returns the encrypted [`Keystore`] and the initial 12-word mnemonic phrase.
    pub fn generate_hd_12(passphrase: &str) -> Result<(Keystore, String), WalletError> {
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);
        Self::create_hd_store(&entropy, 12, passphrase)
    }

    /// Generate a fresh HD wallet with 24 words (256-bit entropy) encrypted under `passphrase`.
    /// Returns the encrypted [`Keystore`] and the initial 24-word mnemonic phrase.
    pub fn generate_hd_24(passphrase: &str) -> Result<(Keystore, String), WalletError> {
        let mut entropy = [0u8; 32];
        OsRng.fill_bytes(&mut entropy);
        Self::create_hd_store(&entropy, 24, passphrase)
    }

    /// Import an existing 12- or 24-word BIP-39 mnemonic into a new encrypted HD keystore.
    /// Returns the encrypted [`Keystore`] and the verified canonical phrase.
    pub fn import_hd_mnemonic(
        phrase: &str,
        passphrase: &str,
    ) -> Result<(Keystore, String), WalletError> {
        let m = Mnemonic::parse_in(Language::English, phrase)
            .map_err(|e| WalletError::Keystore(format!("invalid BIP-39 mnemonic: {e}")))?;
        let word_count = phrase.split_whitespace().count() as u8;
        if word_count != 12 && word_count != 24 {
            return Err(WalletError::Keystore(format!(
                "unsupported mnemonic word count: {word_count}; must be 12 or 24"
            )));
        }
        let entropy = m.to_entropy();
        Self::create_hd_store(&entropy, word_count, passphrase)
    }

    fn create_hd_store(
        entropy: &[u8],
        word_count: u8,
        passphrase: &str,
    ) -> Result<(Keystore, String), WalletError> {
        let m = Mnemonic::from_entropy_in(Language::English, entropy)
            .map_err(|e| WalletError::Keystore(format!("failed to encode mnemonic: {e}")))?;
        let phrase = m.to_string();

        // Account 0 at m/44'/3007'/0'/0'/0' without BIP-39 passphrase establishes primary address
        let seed_64 = m.to_seed("");
        let master = Slip10Node::master(&seed_64)?;
        let primary_child = master.derive_bip44(0, 0, 0)?;
        let primary_sk = SigningKey::from_bytes(&primary_child.secret_key);
        let primary_pk = PublicKey::from_bytes(primary_sk.verifying_key().to_bytes());
        let primary_addr = Address::from_public_key(&primary_pk);

        // Format plaintext envelope: 0x02 || word_count || entropy
        let mut plaintext = Vec::with_capacity(2 + entropy.len());
        plaintext.push(VAULT_ENVELOPE_MAGIC);
        plaintext.push(word_count);
        plaintext.extend_from_slice(entropy);

        let keystore = encrypt_vault_payload(&plaintext, primary_addr, primary_pk, passphrase)?;
        Ok((keystore, phrase))
    }

    /// Decrypt a v2 keystore into its [`VaultPayload`].
    pub fn decrypt(keystore: &Keystore, passphrase: &str) -> Result<VaultPayload, WalletError> {
        if keystore.version != KEYSTORE_VERSION {
            return Err(WalletError::Keystore("unsupported keystore version".into()));
        }

        let ciphertext = hex::decode(&keystore.secret_key)
            .map_err(|e| WalletError::Keystore(format!("invalid ciphertext hex: {e}")))?;
        let salt = hex::decode(keystore.salt.as_deref().unwrap_or_default())
            .map_err(|e| WalletError::Keystore(format!("invalid salt hex: {e}")))?;
        let nonce = hex::decode(keystore.nonce.as_deref().unwrap_or_default())
            .map_err(|e| WalletError::Keystore(format!("invalid nonce hex: {e}")))?;

        if salt.len() != SALT_LEN || nonce.len() != NONCE_LEN {
            return Err(WalletError::Keystore(
                "malformed salt or nonce length".into(),
            ));
        }

        let salt_arr: [u8; SALT_LEN] = salt.try_into().unwrap();
        let nonce_arr: [u8; NONCE_LEN] = nonce.try_into().unwrap();

        let key = derive_argon2id_key(passphrase, &salt_arr)?;
        let addr = kovanica_keystore::address(keystore)
            .map_err(|e| WalletError::Keystore(format!("invalid stored address: {e}")))?;
        let pk_bytes = hex::decode(&keystore.public_key)
            .map_err(|e| WalletError::Keystore(format!("invalid stored public key hex: {e}")))?;
        if pk_bytes.len() != 32 {
            return Err(WalletError::Keystore(
                "stored public key must be 32 bytes".into(),
            ));
        }
        let mut pk_arr = [0u8; 32];
        pk_arr.copy_from_slice(&pk_bytes);

        let aad = build_associated_data(&addr.0, &pk_arr, &salt_arr, &nonce_arr);
        let cipher = XChaCha20Poly1305::new_from_slice(&key[..])
            .map_err(|e| WalletError::Keystore(format!("cipher init error: {e}")))?;

        let plaintext = Zeroizing::new(
            cipher
                .decrypt(
                    XNonce::from_slice(&nonce_arr),
                    chacha20poly1305::aead::Payload {
                        msg: &ciphertext,
                        aad: &aad,
                    },
                )
                .map_err(|_| {
                    WalletError::Keystore(
                        "keystore authentication failed (wrong passphrase)".into(),
                    )
                })?,
        );

        if plaintext.len() == 32 {
            // Legacy ADR-0002 32-byte seed
            let mut seed = [0u8; 32];
            seed.copy_from_slice(&plaintext);
            Ok(VaultPayload::LegacySeed {
                seed: Zeroizing::new(seed),
            })
        } else if plaintext.len() == 18
            && plaintext[0] == VAULT_ENVELOPE_MAGIC
            && plaintext[1] == 12
        {
            // 12-word HD envelope
            Ok(VaultPayload::Hd {
                word_count: 12,
                entropy: Zeroizing::new(plaintext[2..18].to_vec()),
            })
        } else if plaintext.len() == 34
            && plaintext[0] == VAULT_ENVELOPE_MAGIC
            && plaintext[1] == 24
        {
            // 24-word HD envelope
            Ok(VaultPayload::Hd {
                word_count: 24,
                entropy: Zeroizing::new(plaintext[2..34].to_vec()),
            })
        } else {
            Err(WalletError::Keystore(format!(
                "unrecognized vault plaintext format (length {})",
                plaintext.len()
            )))
        }
    }
}

fn derive_argon2id_key(
    passphrase: &str,
    salt: &[u8; SALT_LEN],
) -> Result<Zeroizing<[u8; 32]>, WalletError> {
    let params = Params::new(KDF_MEMORY_KIB, KDF_ITERATIONS, KDF_PARALLELISM, Some(32))
        .map_err(|e| WalletError::Keystore(format!("argon2 params error: {e}")))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; 32]);
    argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut *key)
        .map_err(|e| WalletError::Keystore(format!("argon2 derivation error: {e}")))?;
    Ok(key)
}

fn build_associated_data(
    address: &[u8; 32],
    public_key: &[u8; 32],
    salt: &[u8; SALT_LEN],
    nonce: &[u8; NONCE_LEN],
) -> Vec<u8> {
    let mut aad = b"KVNC keystore\0v2\0argon2id\0m=65536,t=3,p=4\0xchacha20poly1305\0".to_vec();
    aad.extend_from_slice(salt);
    aad.extend_from_slice(nonce);
    aad.extend_from_slice(address);
    aad.extend_from_slice(public_key);
    aad
}

fn encrypt_vault_payload(
    plaintext: &[u8],
    address: Address,
    public_key: PublicKey,
    passphrase: &str,
) -> Result<Keystore, WalletError> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);

    let key = derive_argon2id_key(passphrase, &salt)?;
    let aad = build_associated_data(&address.0, public_key.as_bytes(), &salt, &nonce);
    let cipher = XChaCha20Poly1305::new_from_slice(&key[..])
        .map_err(|e| WalletError::Keystore(format!("cipher init error: {e}")))?;

    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            chacha20poly1305::aead::Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|e| WalletError::Keystore(format!("encryption failed: {e}")))?;

    Ok(Keystore {
        version: KEYSTORE_VERSION,
        address: address.encode(),
        public_key: hex::encode(public_key.as_bytes()),
        encrypted: true,
        secret_key: hex::encode(ciphertext),
        cipher: Some("xchacha20poly1305".into()),
        kdf: Some("argon2id".into()),
        kdf_memory_kib: Some(KDF_MEMORY_KIB),
        kdf_iterations: Some(KDF_ITERATIONS),
        kdf_parallelism: Some(KDF_PARALLELISM),
        salt: Some(hex::encode(salt)),
        nonce: Some(hex::encode(nonce)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hd_12_and_24_word_generation_and_decrypt() {
        let (ks12, phrase12) = VaultPayload::generate_hd_12("strong_pass_12").unwrap();
        assert_eq!(phrase12.split_whitespace().count(), 12);
        let decrypted12 = VaultPayload::decrypt(&ks12, "strong_pass_12").unwrap();
        assert!(!decrypted12.is_legacy());
        assert_eq!(decrypted12.mnemonic_phrase().unwrap(), phrase12);

        let (ks24, phrase24) = VaultPayload::generate_hd_24("strong_pass_24").unwrap();
        assert_eq!(phrase24.split_whitespace().count(), 24);
        let decrypted24 = VaultPayload::decrypt(&ks24, "strong_pass_24").unwrap();
        assert!(!decrypted24.is_legacy());
        assert_eq!(decrypted24.mnemonic_phrase().unwrap(), phrase24);
    }

    #[test]
    fn test_legacy_vs_hd_pubkeys_differ_for_same_phrase() {
        // A test must assert the two pubkeys differ for one fixed phrase
        let fixed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

        // 1. HD account derivation at m/44'/3007'/0'/0'/0'
        let hd_account = WalletAccount::from_mnemonic_hd(fixed_phrase, "", 0, 0, 0).unwrap();
        let hd_pubkey = hd_account.public_key;

        // 2. Legacy ADR-0002 derivation (24 words entropy == direct Ed25519 seed)
        let m = Mnemonic::parse_in(Language::English, fixed_phrase).unwrap();
        let entropy = m.to_entropy();
        assert_eq!(entropy.len(), 32);
        let mut legacy_seed = [0u8; 32];
        legacy_seed.copy_from_slice(&entropy);
        let legacy_account = WalletAccount::from_seed(&legacy_seed);
        let legacy_pubkey = legacy_account.public_key;

        assert_ne!(
            hd_pubkey, legacy_pubkey,
            "HD derivation and legacy ADR-0002 derivation MUST NOT produce the same public key"
        );
    }
}
