//! Local wallet file for Kovanica Console.
//!
//! The file is the v2 keystore JSON produced by `kovanica-light-ffi`, the same
//! document the Android wallet stores. This module does not parse addresses
//! or derive keys itself.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;
use zeroize::{Zeroize, Zeroizing};

/// File name inside the app data directory.
pub const KEYSTORE_FILE: &str = "wallet.keystore.json";

/// Lock the session after this long with no activity.
pub const LOCK_AFTER: Duration = Duration::from_secs(5 * 60);

const PHASE_ABSENT: &str = "absent";
const PHASE_LOCKED: &str = "locked";
const PHASE_OPEN: &str = "open";

const MAX_KEYSTORE_BYTES: u64 = 64 * 1024;

/// What the screen should show. The passphrase is never included.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletStatus {
    /// `absent`, `locked`, or `open`.
    pub phase: String,
    /// Canonical address from Rust, when a wallet file exists.
    pub address: Option<String>,
}

/// A newly created wallet. The phrase is returned once and is not stored.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewWallet {
    /// Canonical address from `kovanica-light-ffi`.
    pub address: String,
    /// 24-word seed. The caller shows it once.
    pub mnemonic: String,
}

pub(crate) struct OpenWallet {
    pub(crate) passphrase: Zeroizing<String>,
    pub(crate) address: String,
    pub(crate) last_active: Instant,
    pub(crate) quote: Option<crate::transfer::TransferQuote>,
}

impl Drop for OpenWallet {
    fn drop(&mut self) {
        self.passphrase.zeroize();
    }
}

/// In-memory session. The keystore file is the only copy on disk.
pub struct WalletStore {
    open: Option<OpenWallet>,
}

impl WalletStore {
    /// No session and no file handle.
    pub fn new() -> Self {
        Self { open: None }
    }

    /// Create a key. Refuses to replace or follow a file that is already there.
    pub fn create(
        &mut self,
        path: &Path,
        passphrase: &str,
        now: Instant,
    ) -> Result<NewWallet, String> {
        if passphrase.is_empty() {
            return Err("Enter a password.".to_string());
        }
        refuse_existing(path)?;
        let mut created = kovanica_light_ffi::create_wallet(passphrase.to_string())
            .map_err(|err| map_ffi_error(&err))?;
        let saved = (|| {
            let address = accept_keystore_json(&created.keystore_json)?;
            if address != created.address {
                return Err("The wallet could not be opened.".to_string());
            }
            write_new_keystore(path, created.keystore_json.as_bytes())?;
            Ok(address)
        })();
        let address = match saved {
            Ok(address) => address,
            Err(err) => {
                created.mnemonic.zeroize();
                created.keystore_json.zeroize();
                return Err(err);
            }
        };
        self.open = Some(OpenWallet {
            passphrase: Zeroizing::new(passphrase.to_string()),
            address: address.clone(),
            last_active: now,
            quote: None,
        });
        let mnemonic = std::mem::take(&mut created.mnemonic);
        created.keystore_json.zeroize();
        Ok(NewWallet { address, mnemonic })
    }

    /// Import a 24-word seed. The phrase is not returned.
    pub fn import(
        &mut self,
        path: &Path,
        mnemonic: &str,
        passphrase: &str,
        now: Instant,
    ) -> Result<WalletStatus, String> {
        if passphrase.is_empty() {
            return Err("Enter a password.".to_string());
        }
        if mnemonic.trim().is_empty() {
            return Err("That recovery phrase is not a 24-word Kovanica seed.".to_string());
        }
        refuse_existing(path)?;
        let mut imported =
            kovanica_light_ffi::import_wallet(mnemonic.to_string(), passphrase.to_string())
                .map_err(|err| map_ffi_error(&err))?;
        let saved = (|| {
            let address = accept_keystore_json(&imported.keystore_json)?;
            if address != imported.address {
                return Err("The wallet could not be opened.".to_string());
            }
            write_new_keystore(path, imported.keystore_json.as_bytes())?;
            Ok(address)
        })();
        let address = match saved {
            Ok(address) => address,
            Err(err) => {
                imported.keystore_json.zeroize();
                return Err(err);
            }
        };
        self.open = Some(OpenWallet {
            passphrase: Zeroizing::new(passphrase.to_string()),
            address: address.clone(),
            last_active: now,
            quote: None,
        });
        imported.keystore_json.zeroize();
        Ok(WalletStatus {
            phase: PHASE_OPEN.to_string(),
            address: Some(address),
        })
    }

    /// Check the password, then keep the session open. The phrase is wiped.
    pub fn unlock(
        &mut self,
        path: &Path,
        passphrase: &str,
        now: Instant,
    ) -> Result<WalletStatus, String> {
        if passphrase.is_empty() {
            return Err("Enter a password.".to_string());
        }
        let json = read_keystore(path)?;
        let mut phrase =
            kovanica_light_ffi::reveal_mnemonic_raw(json.clone(), passphrase.to_string())
                .map_err(|err| map_ffi_error(&err))?;
        phrase.zeroize();
        let address = kovanica_light_ffi::wallet_address(json)
            .map_err(|_| "Could not read the wallet.".to_string())?;
        self.open = Some(OpenWallet {
            passphrase: Zeroizing::new(passphrase.to_string()),
            address: address.clone(),
            last_active: now,
            quote: None,
        });
        Ok(WalletStatus {
            phase: PHASE_OPEN.to_string(),
            address: Some(address),
        })
    }

    /// The open session, after an idle lock.
    pub(crate) fn session(&mut self, now: Instant) -> Result<&mut OpenWallet, String> {
        self.lock_if_idle(now);
        self.open
            .as_mut()
            .ok_or_else(|| "The wallet is locked.".to_string())
    }

    /// Drop the session. The file stays on disk.
    pub fn lock(&mut self) {
        self.open.take();
    }

    /// Record activity while the session is open.
    pub fn touch(&mut self, now: Instant) {
        if let Some(open) = self.open.as_mut() {
            open.last_active = now;
        }
    }

    /// Lock when `now` is at least [`LOCK_AFTER`] after the last activity.
    pub fn lock_if_idle(&mut self, now: Instant) {
        let idle = self
            .open
            .as_ref()
            .is_some_and(|open| now.saturating_duration_since(open.last_active) >= LOCK_AFTER);
        if idle {
            self.open.take();
        }
    }

    /// Phase and address. An idle session is locked first.
    pub fn status(&mut self, path: &Path, now: Instant) -> Result<WalletStatus, String> {
        self.lock_if_idle(now);
        if let Some(open) = &self.open {
            return Ok(WalletStatus {
                phase: PHASE_OPEN.to_string(),
                address: Some(open.address.clone()),
            });
        }
        if !path_exists(path)? {
            return Ok(WalletStatus {
                phase: PHASE_ABSENT.to_string(),
                address: None,
            });
        }
        let json = read_keystore(path)?;
        let address = kovanica_light_ffi::wallet_address(json)
            .map_err(|_| "Could not read the wallet.".to_string())?;
        Ok(WalletStatus {
            phase: PHASE_LOCKED.to_string(),
            address: Some(address),
        })
    }

    /// Open a session without a keystore. Tests use this for quotes that must not decrypt.
    #[cfg(test)]
    pub(crate) fn open_for_test(&mut self, address: String, now: Instant) {
        self.open = Some(OpenWallet {
            passphrase: Zeroizing::new("unused-passphrase".into()),
            address,
            last_active: now,
            quote: None,
        });
    }

    /// Whether the open session is holding a quote.
    #[cfg(test)]
    pub(crate) fn has_quote(&self) -> bool {
        self.open.as_ref().is_some_and(|open| open.quote.is_some())
    }
}

impl Default for WalletStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Map a light-ffi failure onto a fixed sentence. The raw text is never returned.
pub(crate) fn map_ffi_error(err: &kovanica_light_ffi::LightFfiError) -> String {
    let lower = err.to_string().to_ascii_lowercase();
    if lower.contains("passphrase is empty") {
        "Enter a password.".to_string()
    } else if lower.contains("mnemonic") || lower.contains("bip-39") || lower.contains("bip39") {
        "That recovery phrase is not a 24-word Kovanica seed.".to_string()
    } else if lower.contains("authentication")
        || lower.contains("decrypt")
        || lower.contains("passphrase")
    {
        "That password does not open this wallet.".to_string()
    } else {
        "The wallet could not be opened.".to_string()
    }
}

fn refuse_existing(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            Err("Refusing to write the wallet through a symlink.".to_string())
        }
        Ok(_) => Err("A wallet already exists on this device.".to_string()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Could not save the wallet.".to_string()),
    }
}

fn path_exists(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            Err("Refusing to read the wallet through a symlink.".to_string())
        }
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err("Could not read the wallet.".to_string()),
    }
}

/// Accept only the Android v2 document: Argon2id and XChaCha20-Poly1305.
fn accept_keystore_json(json: &str) -> Result<String, String> {
    let value: Value =
        serde_json::from_str(json).map_err(|_| "Could not save the wallet.".to_string())?;
    let ok = value.get("version").and_then(Value::as_u64) == Some(2)
        && value.get("encrypted").and_then(Value::as_bool) == Some(true)
        && value.get("cipher").and_then(Value::as_str) == Some("xchacha20poly1305")
        && value.get("kdf").and_then(Value::as_str) == Some("argon2id")
        && value.get("kdf_memory_kib").and_then(Value::as_u64) == Some(65_536)
        && value.get("kdf_iterations").and_then(Value::as_u64) == Some(3)
        && value.get("kdf_parallelism").and_then(Value::as_u64) == Some(4);
    if !ok {
        return Err("Could not save the wallet.".to_string());
    }
    kovanica_light_ffi::wallet_address(json.to_string())
        .map_err(|_| "Could not save the wallet.".to_string())
}

fn write_new_keystore(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() as u64 > MAX_KEYSTORE_BYTES {
        return Err("Could not save the wallet.".to_string());
    }
    refuse_existing(path)?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|_| "Could not open the app data directory.".to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|_| "Could not save the wallet.".to_string())?;
    }
    let tmp_name = format!(
        ".{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("wallet")
    );
    let tmp = parent.join(tmp_name);
    if fs::symlink_metadata(&tmp).is_ok() {
        return Err("Could not save the wallet.".to_string());
    }
    {
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&tmp)
            .map_err(|_| "Could not save the wallet.".to_string())?;
        file.write_all(bytes)
            .map_err(|_| "Could not save the wallet.".to_string())?;
        file.sync_all()
            .map_err(|_| "Could not save the wallet.".to_string())?;
    }
    fs::rename(&tmp, path).map_err(|_| {
        let _ = fs::remove_file(&tmp);
        "Could not save the wallet.".to_string()
    })?;
    Ok(())
}

pub(crate) fn read_keystore(path: &Path) -> Result<String, String> {
    path_exists(path)?;
    let file = fs::File::open(path).map_err(|_| "Could not read the wallet.".to_string())?;
    let mut bytes = Vec::new();
    use std::io::Read;
    file.take(MAX_KEYSTORE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Could not read the wallet.".to_string())?;
    if bytes.len() as u64 > MAX_KEYSTORE_BYTES {
        return Err("Could not read the wallet.".to_string());
    }
    String::from_utf8(bytes).map_err(|_| "Could not read the wallet.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("kovanica-console-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn keystore_in(dir: &Path) -> std::path::PathBuf {
        dir.join(KEYSTORE_FILE)
    }

    fn must<T>(result: Result<T, String>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("{err}"),
        }
    }

    fn must_err<T>(result: Result<T, String>) -> String {
        match result {
            Ok(_) => panic!("expected the wallet call to fail"),
            Err(err) => err,
        }
    }

    #[test]
    fn wallet_source_does_not_log_secrets() {
        let src = include_str!("wallet.rs");
        let body = src.split("mod tests").next().unwrap_or(src);
        assert!(!body.contains("print"));
        assert!(!body.contains("dbg!"));
        assert!(!body.contains("tracing"));
        assert!(!body.contains("log::"));
    }

    #[test]
    fn ffi_errors_do_not_echo_the_secret() {
        let word = "zznotawordzz";
        let err = kovanica_light_ffi::LightFfiError::Failed {
            reason: format!("invalid mnemonic word {word}"),
        };
        let mapped = map_ffi_error(&err);
        assert_eq!(
            mapped,
            "That recovery phrase is not a 24-word Kovanica seed."
        );
        assert!(!mapped.contains(word));

        let password = "wrong-horse-marker";
        let err = kovanica_light_ffi::LightFfiError::Failed {
            reason: format!("keystore authentication failed for {password}"),
        };
        let mapped = map_ffi_error(&err);
        assert_eq!(mapped, "That password does not open this wallet.");
        assert!(!mapped.contains(password));
    }

    #[test]
    fn a_bad_phrase_is_rejected_before_the_file_is_written() {
        let dir = temp_dir("bad-phrase");
        let path = keystore_in(&dir);
        let mut store = WalletStore::new();
        let marker = "zznotawordzz";
        let err = must_err(store.import(&path, marker, "import-password", Instant::now()));
        assert_eq!(err, "That recovery phrase is not a 24-word Kovanica seed.");
        assert!(!err.contains(marker));
        assert!(!path.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_empty_password_is_rejected() {
        let dir = temp_dir("empty-password");
        let path = keystore_in(&dir);
        let mut store = WalletStore::new();
        let err = must_err(store.create(&path, "", Instant::now()));
        assert_eq!(err, "Enter a password.");
        assert!(!path.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_is_not_a_wallet_file() {
        use std::os::unix::fs::symlink;
        let dir = temp_dir("symlink");
        let target = dir.join("elsewhere");
        fs::write(&target, b"keep").unwrap();
        let path = keystore_in(&dir);
        symlink(&target, &path).unwrap();
        let mut store = WalletStore::new();
        let err = must_err(store.create(&path, "symlink-password", Instant::now()));
        assert_eq!(err, "Refusing to write the wallet through a symlink.");
        assert_eq!(fs::read(&target).unwrap(), b"keep");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn create_writes_an_android_v2_keystore_and_locks_when_idle() {
        let dir = temp_dir("create");
        let path = keystore_in(&dir);
        let mut store = WalletStore::new();
        let now = Instant::now();
        let created = must(store.create(&path, "create-password", now));
        assert_eq!(created.mnemonic.split_whitespace().count(), 24);

        let bytes = fs::read(&path).unwrap();
        let text = String::from_utf8(bytes.clone()).unwrap();
        assert!(!text.contains(&created.mnemonic));
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["version"], 2);
        assert_eq!(value["encrypted"], true);
        assert_eq!(value["cipher"], "xchacha20poly1305");
        assert_eq!(value["kdf"], "argon2id");
        assert_eq!(value["kdf_memory_kib"], 65_536);
        assert_eq!(value["kdf_iterations"], 3);
        assert_eq!(value["kdf_parallelism"], 4);
        assert_eq!(value["secret_key"].as_str().unwrap().len(), (32 + 16) * 2);
        let parsed = kovanica_keystore::parse(&bytes).unwrap();
        assert_eq!(
            kovanica_keystore::address(&parsed).unwrap().to_string(),
            created.address
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }

        let status = must(store.status(&path, now));
        assert_eq!(status.phase, PHASE_OPEN);
        let status_json = serde_json::to_string(&status).unwrap();
        assert!(!status_json.contains(&created.mnemonic));

        let err = must_err(store.create(&path, "create-password", now));
        assert_eq!(err, "A wallet already exists on this device.");

        let err = must_err(store.unlock(&path, "wrong-horse-marker", now));
        assert_eq!(err, "That password does not open this wallet.");
        assert!(!err.contains("wrong-horse-marker"));

        store.touch(now + Duration::from_secs(200));
        let still_open = must(store.status(
            &path,
            now + Duration::from_secs(200) + LOCK_AFTER - Duration::from_secs(1),
        ));
        assert_eq!(still_open.phase, PHASE_OPEN);

        let locked = must(store.status(&path, now + Duration::from_secs(200) + LOCK_AFTER));
        assert_eq!(locked.phase, PHASE_LOCKED);
        assert_eq!(locked.address.as_deref(), Some(created.address.as_str()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_round_trips_the_same_keystore_parser() {
        let phrase = kovanica_keystore::seed_to_mnemonic(&[0x11; 32]).unwrap();
        let dir = temp_dir("import");
        let path = keystore_in(&dir);
        let mut store = WalletStore::new();
        let now = Instant::now();
        let status = must(store.import(&path, &phrase, "import-password", now));
        assert_eq!(status.phase, PHASE_OPEN);
        let bytes = fs::read(&path).unwrap();
        let text = String::from_utf8(bytes.clone()).unwrap();
        assert!(!text.contains(&phrase));
        let parsed = kovanica_keystore::parse(&bytes).unwrap();
        assert_eq!(
            kovanica_keystore::address(&parsed).unwrap().to_string(),
            status.address.unwrap()
        );
        assert_eq!(parsed.kdf.as_deref(), Some("argon2id"));
        assert_eq!(parsed.cipher.as_deref(), Some("xchacha20poly1305"));
        store.lock();
        let locked = must(store.status(&path, now));
        assert_eq!(locked.phase, PHASE_LOCKED);
        let _ = fs::remove_dir_all(&dir);
    }
}
