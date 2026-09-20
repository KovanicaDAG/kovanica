//! Local wallet: an Ed25519 key stored as a 32-byte seed on disk.
//!
//! Key generation, the `kvnc…dag` address encoding, and signing are all
//! delegated to `kovanica-state` (the node's own crate) so the CLI can never
//! disagree with the ledger about what an address is or how a spend is signed.
//!
//! Supports both raw 32-byte seeds (hex) and BIP39 mnemonics (24 words).

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bip39::Mnemonic;
use kovanica_state::{Address, KeyPair};

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
        Ok(Self { seed, mnemonic: None })
    }

    /// Generate a fresh wallet with a BIP39 mnemonic (24 words, 256 bits entropy).
    pub fn generate_with_mnemonic() -> Result<Self> {
        let mnemonic = Mnemonic::generate(24)?;
        let seed_bytes = derive_seed_from_mnemonic(&mnemonic)?;
        Ok(Self {
            seed: seed_bytes,
            mnemonic: Some(mnemonic.to_string()),
        })
    }

    /// Reconstruct a wallet from a stored 32-byte seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self { seed, mnemonic: None }
    }

    /// Reconstruct a wallet from a BIP39 mnemonic phrase.
    pub fn from_mnemonic(mnemonic: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse(mnemonic)?;
        let seed_bytes = derive_seed_from_mnemonic(&mnemonic)?;
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
    /// 2. BIP39 mnemonic phrase (24 words, space-separated)
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("cannot read key file {}", path.display()))?;
        let trimmed = text.trim();

        // Try to parse as BIP39 mnemonic first (space-separated words)
        if trimmed.split_whitespace().count() >= 12 {
            if let Ok(mnemonic) = Mnemonic::parse(trimmed) {
                let seed_bytes = derive_seed_from_mnemonic(&mnemonic)?;
                return Ok(Self {
                    seed: seed_bytes,
                    mnemonic: Some(mnemonic.to_string()),
                });
            }
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

/// Derive a 32-byte Ed25519 seed from a BIP39 mnemonic using standard BIP39 → BIP32 derivation.
/// Uses PBKDF2-HMAC-SHA512 with 2048 iterations, salt = "mnemonic" + passphrase (empty).
fn derive_seed_from_mnemonic(mnemonic: &Mnemonic) -> Result<[u8; 32]> {
    // Get 64-byte BIP39 seed with empty passphrase
    let seed_64 = mnemonic.to_seed_normalized("");
    // Take first 32 bytes for Ed25519
    let mut ed25519_seed = [0u8; 32];
    ed25519_seed.copy_from_slice(&seed_64[..32]);
    Ok(ed25519_seed)
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
    fn mnemonic_save_and_load() {
        let wallet = Wallet::generate_with_mnemonic().unwrap();
        let addr = wallet.address();
        let mnemonic = wallet.mnemonic().unwrap().to_string();

        let path = std::env::temp_dir().join(format!("kvnc-mnemonic-test-{}.key", std::process::id()));
        wallet.save(&path, true).unwrap();

        let loaded = Wallet::load(&path).unwrap();
        assert_eq!(loaded.address(), addr);
        assert_eq!(loaded.mnemonic(), Some(mnemonic.as_str()));

        let _ = fs::remove_file(&path);
    }
}