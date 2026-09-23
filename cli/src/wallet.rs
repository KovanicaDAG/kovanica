//! Local wallet: an Ed25519 key stored as a 32-byte seed on disk.
//!
//! Address encoding and spend signing are delegated to `kovanica-state`
//! (the node's own crate) — the CLI stays byte-compatible with the ledger.
//!
//! Supports raw 32-byte seeds (hex) and BIP39 mnemonics (12 or 24 words).
//! Mnemonic key material follows the **frozen** SLIP-0010 ed25519 path
//! `m/44'/3007'/0'/0'/0'` — see `docs/backlog/DERIVATION.md` (SDK
//! `kovanica-keys` and the web wallet implement the same path).

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bip39::Mnemonic;
use kovanica_state::{Address, KeyPair};

/// Frozen SLIP-0010 coin type (matches `kovanica-keys::SLIP44_COIN_TYPE`).
const SLIP10_COIN_TYPE: u32 = 3007;

/// A loaded wallet: the raw Ed25519 seed plus its derived keypair.
/// Optionally stores the BIP39 mnemonic for human-readable backup.
pub struct Wallet {
    seed: [u8; 32],
    mnemonic: Option<String>,
}

impl Wallet {
    /// Generate a fresh wallet from operating-system randomness (raw 32-byte seed).
    pub fn generate() -> Result<Self> {
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed)
            .map_err(|e| anyhow::anyhow!("failed to read OS randomness for key generation: {e}"))?;
        Ok(Self {
            seed,
            mnemonic: None,
        })
    }

    /// Generate a fresh wallet with a BIP39 mnemonic (24 words, 256 bits entropy).
    pub fn generate_with_mnemonic() -> Result<Self> {
        Self::generate_with_mnemonic_words(24, "")
    }

    /// Generate a fresh wallet with a BIP39 mnemonic of `words` words
    /// (12 or 24) and an optional BIP-39 passphrase ("25th word").
    pub fn generate_with_mnemonic_words(words: usize, passphrase: &str) -> Result<Self> {
        if words != 12 && words != 24 {
            bail!("words must be 12 or 24, got {words}");
        }
        let mnemonic = Mnemonic::generate(words)?;
        let seed_bytes = derive_seed_from_mnemonic(&mnemonic, passphrase)?;
        Ok(Self {
            seed: seed_bytes,
            mnemonic: Some(mnemonic.to_string()),
        })
    }

    /// Reconstruct a wallet from a stored 32-byte seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            seed,
            mnemonic: None,
        }
    }

    /// Reconstruct a wallet from a BIP39 mnemonic phrase (empty passphrase).
    pub fn from_mnemonic(mnemonic: &str) -> Result<Self> {
        Self::from_mnemonic_with_passphrase(mnemonic, "")
    }

    /// Reconstruct a wallet from a BIP39 mnemonic phrase + optional passphrase.
    pub fn from_mnemonic_with_passphrase(mnemonic: &str, passphrase: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse(mnemonic)?;
        let seed_bytes = derive_seed_from_mnemonic(&mnemonic, passphrase)?;
        Ok(Self {
            seed: seed_bytes,
            mnemonic: Some(mnemonic.to_string()),
        })
    }

    /// The Ed25519 keypair, used for signing.
    pub fn keypair(&self) -> KeyPair {
        KeyPair::from_seed(self.seed)
    }

    /// This wallet's address.
    pub fn address(&self) -> Address {
        self.keypair().address()
    }

    /// The raw 32-byte Ed25519 public key (watch-only export, "xpub").
    pub fn public_key(&self) -> [u8; 32] {
        self.keypair().public_key()
    }

    /// Returns the BIP39 mnemonic if available.
    pub fn mnemonic(&self) -> Option<&str> {
        self.mnemonic.as_deref()
    }

    /// Returns the raw 32-byte seed.
    pub fn seed(&self) -> [u8; 32] {
        self.seed
    }

    /// Load a wallet from a key file.
    /// Supports two formats:
    /// 1. 64 hex chars (32-byte raw seed) — legacy format
    /// 2. BIP39 mnemonic phrase (12 or 24 words, space-separated)
    pub fn load(path: &Path) -> Result<Self> {
        Self::load_with_passphrase(path, "")
    }

    /// Load a wallet from a key file, deriving mnemonic keys with `passphrase`.
    pub fn load_with_passphrase(path: &Path, passphrase: &str) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("cannot read key file {}", path.display()))?;
        let trimmed = text.trim();

        // Try to parse as BIP39 mnemonic first (space-separated words)
        if trimmed.split_whitespace().count() >= 12 {
            if let Ok(mnemonic) = Mnemonic::parse(trimmed) {
                let seed_bytes = derive_seed_from_mnemonic(&mnemonic, passphrase)?;
                return Ok(Self {
                    seed: seed_bytes,
                    mnemonic: Some(mnemonic.to_string()),
                });
            }
        }

        // Hex-encoded seeds ignore the passphrase; flag a likely mistake.
        if !passphrase.is_empty() {
            bail!(
                "key file {} holds a raw seed (no passphrase expected); passphrase only applies to mnemonic wallets",
                path.display()
            );
        }
        // Fall back to hex-encoded 32-byte seed
        let raw = hex::decode(trimmed)
            .with_context(|| format!("key file {} is not valid hex or mnemonic", path.display()))?;
        let seed: [u8; 32] = raw
            .try_into()
            .map_err(|_| anyhow::anyhow!("key file {} must hold a 32-byte seed", path.display()))?;
        Ok(Self::from_seed(seed))
    }

    /// Save this wallet to `path` with owner-only (0600) permissions.
    /// If the wallet has a mnemonic, saves the mnemonic (human-readable).
    /// Otherwise saves the raw seed as hex.
    /// Refuses to overwrite an existing file unless `force` is set.
    pub fn save(&self, path: &Path, force: bool) -> Result<()> {
        if path.exists() && !force {
            bail!(
                "{} already exists; refusing to overwrite (use --force)",
                path.display()
            );
        }
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("cannot create directory {}", parent.display()))?;
            }
        }
        let content = if let Some(mnemonic) = &self.mnemonic {
            format!("{}\n", mnemonic)
        } else {
            format!("{}\n", hex::encode(self.seed))
        };
        fs::write(path, content)
            .with_context(|| format!("cannot write key file {}", path.display()))?;
        set_owner_only(path)?;
        Ok(())
    }

    /// Save the mnemonic to a separate file (for backup).
    pub fn save_mnemonic(&self, path: &Path, force: bool) -> Result<()> {
        let Some(mnemonic) = &self.mnemonic else {
            bail!("wallet has no mnemonic to save");
        };
        if path.exists() && !force {
            bail!(
                "{} already exists; refusing to overwrite (use --force)",
                path.display()
            );
        }
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("cannot create directory {}", parent.display()))?;
            }
        }
        fs::write(path, format!("{}\n", mnemonic))
            .with_context(|| format!("cannot write mnemonic file {}", path.display()))?;
        set_owner_only(path)?;
        Ok(())
    }
}

/// Derive the 32-byte Ed25519 key material from a BIP39 mnemonic using the
/// **frozen** SLIP-0010 path `m/44'/3007'/0'/0'/0'` (all segments hardened).
///
/// Mirrors `kovanica-keys::slip10::derive_ed25519` — see
/// `docs/backlog/DERIVATION.md`. Must not drift from the SDK/web derivation.
fn derive_seed_from_mnemonic(mnemonic: &Mnemonic, passphrase: &str) -> Result<[u8; 32]> {
    // 64-byte BIP39 seed: PBKDF2-HMAC-SHA512, 2048 iters, salt "mnemonic"+phrase
    let seed_64 = mnemonic.to_seed_normalized(passphrase);
    Ok(slip10_ed25519(&seed_64, 0))
}

/// SLIP-0010 ed25519 (hardened-only): master + path derivation.
fn slip10_ed25519(input: &[u8; 64], index: u32) -> [u8; 32] {
    use hmac::{Hmac, Mac};
    type HmacSha512 = Hmac<sha2::Sha512>;

    fn hmac_sha512(key: &[u8], data: &[u8]) -> [u8; 64] {
        let mut mac = <HmacSha512 as Mac>::new_from_slice(key).expect("HMAC accepts any key size");
        mac.update(data);
        let out = mac.finalize().into_bytes();
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&out);
        arr
    }

    fn hardened(sk: &[u8; 32], chain: &[u8; 32], n: u32) -> ([u8; 32], [u8; 32]) {
        let mut data = [0u8; 1 + 32 + 4];
        data[1..33].copy_from_slice(sk);
        data[33..].copy_from_slice(&(n | 0x8000_0000).to_be_bytes());
        let i = hmac_sha512(chain, &data);
        (i[..32].try_into().unwrap(), i[32..].try_into().unwrap())
    }

    let i = hmac_sha512(b"ed25519 seed", input);
    let (mut sk, mut chain) = (i[..32].try_into().unwrap(), i[32..].try_into().unwrap());
    for step in [44u32, SLIP10_COIN_TYPE, 0, 0, index] {
        let (nsk, nchain) = hardened(&sk, &chain, step);
        sk = nsk;
        chain = nchain;
    }
    sk
}

#[cfg(unix)]
fn set_owner_only(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("cannot set 0600 permissions on {}", path.display()))
}

#[cfg(not(unix))]
fn set_owner_only(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_roundtrips_through_a_file() {
        let seed = [7u8; 32];
        let wallet = Wallet::from_seed(seed);
        let addr = wallet.address();

        let path = std::env::temp_dir().join(format!("kvnc-test-{}.key", std::process::id()));
        wallet.save(&path, true).unwrap();
        let loaded = Wallet::load(&path).unwrap();
        assert_eq!(loaded.address(), addr);

        // Refuses to clobber without force.
        assert!(Wallet::from_seed([9u8; 32]).save(&path, false).is_err());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn generated_wallets_differ() {
        let a = Wallet::generate().unwrap();
        let b = Wallet::generate().unwrap();
        assert_ne!(a.address(), b.address());
    }

    #[test]
    fn mnemonic_generation_and_recovery() {
        let wallet = Wallet::generate_with_mnemonic().unwrap();
        let addr1 = wallet.address();
        let mnemonic = wallet.mnemonic().unwrap();

        // Recover from mnemonic
        let recovered = Wallet::from_mnemonic(mnemonic).unwrap();
        let addr2 = recovered.address();
        assert_eq!(addr1, addr2);
        assert_eq!(recovered.mnemonic(), Some(mnemonic));
    }

    #[test]
    fn twelve_word_mnemonic_works() {
        let wallet = Wallet::generate_with_mnemonic_words(12, "").unwrap();
        let words: Vec<&str> = wallet.mnemonic().unwrap().split(' ').collect();
        assert_eq!(words.len(), 12);
        let recovered = Wallet::from_mnemonic(wallet.mnemonic().unwrap()).unwrap();
        assert_eq!(recovered.address(), wallet.address());
    }

    #[test]
    fn invalid_word_count_rejected() {
        assert!(Wallet::generate_with_mnemonic_words(13, "").is_err());
        assert!(Wallet::generate_with_mnemonic_words(0, "").is_err());
    }

    #[test]
    fn passphrase_changes_the_derived_key() {
        let wallet = Wallet::generate_with_mnemonic_words(12, "hunter2").unwrap();
        let mnemonic = wallet.mnemonic().unwrap().to_string();
        // Same phrase, empty passphrase must NOT give the same key.
        let no_pass = Wallet::from_mnemonic(&mnemonic).unwrap();
        assert_ne!(no_pass.address(), wallet.address());
        // Same phrase + same passphrase must recover the key.
        let recovered = Wallet::from_mnemonic_with_passphrase(&mnemonic, "hunter2").unwrap();
        assert_eq!(recovered.address(), wallet.address());
        // A wrong passphrase must not match either.
        let wrong = Wallet::from_mnemonic_with_passphrase(&mnemonic, "nope").unwrap();
        assert_ne!(wrong.address(), wallet.address());
    }

    #[test]
    fn load_with_passphrase_flags_raw_seed_files() {
        let seed = [3u8; 32];
        let path = std::env::temp_dir().join(format!("kvnc-pass-test-{}.key", std::process::id()));
        Wallet::from_seed(seed).save(&path, true).unwrap();
        // A passphrase on a raw-seed file is a likely mistake: reject it.
        assert!(Wallet::load_with_passphrase(&path, "hunter2").is_err());
        // Plain load still works.
        assert_eq!(
            Wallet::load(&path).unwrap().address(),
            Wallet::from_seed(seed).address()
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn slip10_frozen_vector_index_zero() {
        // Canonical zero-entropy 128-bit phrase (built from entropy bytes in
        // code so no mnemonic-like string appears here), empty passphrase.
        // Expected 32-byte derived key at m/44'/3007'/0'/0'/0' MUST match the
        // SDK `kovanica-keys` known-answer vector (see
        // sdk/crates/kovanica-keys/tests/slip10_vectors.rs).
        let mnemonic = Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 16]).unwrap();
        let key = derive_seed_from_mnemonic(&mnemonic, "").unwrap();
        let expected: [u8; 32] = [
            0x99, 0xd5, 0xe3, 0xa2, 0xa1, 0x67, 0xff, 0xae, 0x44, 0x07, 0xe9, 0x48, 0x51, 0x05,
            0xf3, 0x01, 0xab, 0x88, 0xd4, 0x9e, 0xc9, 0x95, 0x1f, 0x00, 0x8e, 0xb7, 0x84, 0x0f,
            0xfd, 0xed, 0x80, 0x4d,
        ];
        assert_eq!(key, expected);
    }

    #[test]
    fn public_key_export_matches_address() {
        let wallet = Wallet::generate_with_mnemonic().unwrap();
        let pk = wallet.public_key();
        // A P2PK address embeds the raw pubkey directly: 0x00 || pubkey.
        let addr = wallet.address();
        assert_eq!(addr.payload(), &pk);
    }

    #[test]
    fn mnemonic_save_and_load() {
        let wallet = Wallet::generate_with_mnemonic().unwrap();
        let addr = wallet.address();
        let mnemonic = wallet.mnemonic().unwrap().to_string();

        let path =
            std::env::temp_dir().join(format!("kvnc-mnemonic-test-{}.key", std::process::id()));
        wallet.save(&path, true).unwrap();

        let loaded = Wallet::load(&path).unwrap();
        assert_eq!(loaded.address(), addr);
        assert_eq!(loaded.mnemonic(), Some(mnemonic.as_str()));

        let _ = fs::remove_file(&path);
    }
}
