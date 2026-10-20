//! Local wallet file for Kovanica Console.
//!
//! The file is the v2 keystore JSON produced by `kovanica-light-ffi`, the same
//! document the Android wallet stores. `kovanica_keystore::parse` checks the
//! parameters. This module does not derive keys itself.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde::Serialize;
use zeroize::{Zeroize, Zeroizing};

/// File name inside the app data directory.
pub const KEYSTORE_FILE: &str = "wallet.keystore.json";

/// Lock the session after this long with no activity.
pub const LOCK_AFTER: Duration = Duration::from_secs(5 * 60);

/// How long the idle thread sleeps when no session is open.
const IDLE_PARK: Duration = Duration::from_secs(60);

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
    /// Idle limit for this store. Production uses [`LOCK_AFTER`].
    lock_after: Duration,
}

/// Session plus the condvar the idle thread waits on.
///
/// The condvar is paired with [`WalletShared::store`]. Notify it after every
/// change to the session so the idle thread recomputes its deadline.
pub struct WalletShared {
    pub store: Mutex<WalletStore>,
    pub wake: Condvar,
}

impl WalletShared {
    /// Empty session. The idle thread is started separately.
    pub fn new() -> Self {
        Self {
            store: Mutex::new(WalletStore::new()),
            wake: Condvar::new(),
        }
    }

    /// Lock the session. A poisoned mutex still yields the store.
    pub fn lock_store(&self) -> MutexGuard<'_, WalletStore> {
        self.store.lock().unwrap_or_else(|err| err.into_inner())
    }
}

/// Background lock. The session is cleared after [`LOCK_AFTER`] even when the
/// webview never calls status again.
pub struct IdleLockThread {
    shared: Arc<WalletShared>,
    stop: Arc<AtomicBool>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl IdleLockThread {
    /// Start the idle thread. Drop joins it.
    pub fn spawn(shared: Arc<WalletShared>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_shared = Arc::clone(&shared);
        let thread_stop = Arc::clone(&stop);
        let join = std::thread::Builder::new()
            .name("kovanica-wallet-idle".into())
            .spawn(move || idle_lock_loop(&thread_shared, &thread_stop))
            .expect("wallet idle lock thread");
        Self {
            shared,
            stop,
            join: Mutex::new(Some(join)),
        }
    }
}

impl Drop for IdleLockThread {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.shared.wake.notify_one();
        if let Some(join) = self
            .join
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .take()
        {
            let _ = join.join();
        }
    }
}

/// Sleep until the open session is idle, then clear it. A notify recomputes the wait.
fn idle_lock_loop(shared: &WalletShared, stop: &AtomicBool) {
    let mut store = shared.lock_store();
    while !stop.load(Ordering::SeqCst) {
        let now = Instant::now();
        store.lock_if_idle(now);
        let wait = match store.idle_remaining(now) {
            Some(wait) if !wait.is_zero() => wait,
            Some(_) => Duration::from_millis(1),
            None => IDLE_PARK,
        };
        let waited = shared.wake.wait_timeout(store, wait);
        store = match waited {
            Ok((guard, _)) => guard,
            Err(poisoned) => poisoned.into_inner().0,
        };
    }
}

impl WalletStore {
    /// No session and no file handle.
    pub fn new() -> Self {
        Self {
            open: None,
            lock_after: LOCK_AFTER,
        }
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

    /// Move the idle deadline of an open session.
    ///
    /// This is the only extension. Create, import, and unlock start a new
    /// deadline. Status, quote, and send leave it where it is.
    pub fn touch(&mut self, now: Instant) {
        if let Some(open) = self.open.as_mut() {
            open.last_active = now;
        }
    }

    /// Lock when `now` is at least [`LOCK_AFTER`] after the last activity.
    pub fn lock_if_idle(&mut self, now: Instant) {
        let limit = self.lock_after;
        let idle = self
            .open
            .as_ref()
            .is_some_and(|open| now.saturating_duration_since(open.last_active) >= limit);
        if idle {
            self.open.take();
        }
    }

    /// Time left before [`lock_if_idle`] clears the session.
    pub fn idle_remaining(&self, now: Instant) -> Option<Duration> {
        let open = self.open.as_ref()?;
        let elapsed = now.saturating_duration_since(open.last_active);
        Some(self.lock_after.saturating_sub(elapsed))
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

    /// Shorten the idle limit. Production keeps [`LOCK_AFTER`].
    #[cfg(test)]
    pub(crate) fn set_lock_after_for_test(&mut self, lock_after: Duration) {
        self.lock_after = lock_after;
    }

    /// True when a session is open. Does not apply the idle limit.
    #[cfg(test)]
    pub(crate) fn is_open_for_test(&self) -> bool {
        self.open.is_some()
    }

    /// Deadline anchor, when a session is open.
    #[cfg(test)]
    pub(crate) fn last_active_for_test(&self) -> Option<Instant> {
        self.open.as_ref().map(|open| open.last_active)
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

/// Accept a keystore only when `kovanica_keystore::parse` does.
fn accept_keystore_json(json: &str) -> Result<String, String> {
    let parsed = kovanica_keystore::parse(json.as_bytes())
        .map_err(|_| "Could not save the wallet.".to_string())?;
    kovanica_keystore::address(&parsed)
        .map(|address| address.to_string())
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
    use serde_json::Value;

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
    fn status_does_not_extend_idle_and_only_touch_does() {
        let path = Path::new("missing-idle-wallet");
        let mut store = WalletStore::new();
        let now = Instant::now();
        store.open_for_test("kvnc-idle".into(), now);
        let later = now + Duration::from_secs(30);
        let status = must(store.status(path, later));
        assert_eq!(status.phase, PHASE_OPEN);
        assert_eq!(store.last_active_for_test(), Some(now));
        store.touch(later);
        assert_eq!(store.last_active_for_test(), Some(later));
        let still = must(store.status(path, later + LOCK_AFTER - Duration::from_secs(1)));
        assert_eq!(still.phase, PHASE_OPEN);
        assert!(store.session(later + LOCK_AFTER).is_err());
        assert!(!store.is_open_for_test());
        store.touch(later + LOCK_AFTER + Duration::from_secs(1));
        assert!(store.last_active_for_test().is_none());
        assert!(!store.is_open_for_test());
    }

    #[test]
    fn idle_thread_clears_the_session_without_a_status_poll() {
        let shared = Arc::new(WalletShared::new());
        {
            let mut store = shared.lock_store();
            store.set_lock_after_for_test(Duration::from_millis(40));
            store.open_for_test("kvnc-timer".into(), Instant::now());
        }
        let idle = IdleLockThread::spawn(Arc::clone(&shared));
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut cleared = false;
        while Instant::now() < deadline {
            if !shared.lock_store().is_open_for_test() {
                cleared = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        drop(idle);
        assert!(cleared, "idle thread left the session open");
        shared.lock_store().touch(Instant::now());
        assert!(!shared.lock_store().is_open_for_test());
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

    /// Produced by `kovanica_keystore::from_seed(&[0x5A; 32], "cross-check")`.
    /// No Android device fixture is checked into this repo, so this vector
    /// comes from the same keystore code the Android wallet reaches.
    const CROSS_KEYSTORE_JSON: &str = r#"{"version":2,"address":"kvnc0d7550754e0800a5d237eef5826035766b9b3e5a15868a940ab289958788e3b07bc9f7e9dag","public_key":"0d7550754e0800a5d237eef5826035766b9b3e5a15868a940ab289958788e3b0","encrypted":true,"secret_key":"21406ec83c99222e1a7262d80534b3cf334d5b213654c65efe8b5fb5ea70f8d2c78403778f0d09902a7c9a2b21c4b055","cipher":"xchacha20poly1305","kdf":"argon2id","kdf_memory_kib":65536,"kdf_iterations":3,"kdf_parallelism":4,"salt":"88e37fe15da91bf57bfa24181fc595d9","nonce":"1d6b4b4a95293d5a4bf31f16059f032a4acf0c9cf2674e42"}"#;

    #[test]
    fn a_fixed_keystore_vector_uses_keystore_validation() {
        let body = include_str!("wallet.rs").split("mod tests").next().unwrap();
        assert!(!body.contains("kdf_memory_kib"));
        assert!(!body.contains("65_536"));
        assert!(!body.contains("kdf_iterations"));

        let address = must(accept_keystore_json(CROSS_KEYSTORE_JSON));
        let parsed = kovanica_keystore::parse(CROSS_KEYSTORE_JSON.as_bytes()).unwrap();
        assert_eq!(
            kovanica_keystore::address(&parsed).unwrap().to_string(),
            address
        );
        assert_eq!(
            kovanica_keystore::secret_seed(&parsed, Some("cross-check"))
                .unwrap()
                .as_ref(),
            &[0x5A; 32]
        );

        let mut tampered: Value = serde_json::from_str(CROSS_KEYSTORE_JSON).unwrap();
        tampered["kdf_iterations"] = Value::from(1);
        let bad = serde_json::to_string(&tampered).unwrap();
        assert!(accept_keystore_json(&bad).is_err());
        assert!(kovanica_keystore::parse(bad.as_bytes()).is_err());
    }

    #[test]
    fn jarvis_cross_keystore_core_to_desktop_and_back() {
        let dir = temp_dir("jarvis-cross");
        let path = keystore_in(&dir);
        let ks = kovanica_keystore::from_seed(&[0x22; 32], "core-password").unwrap();
        let addr = kovanica_keystore::address(&ks).unwrap().to_string();
        let json = serde_json::to_string(&ks).unwrap();
        fs::write(&path, json).unwrap();
        let mut store = WalletStore::new();
        let st = must(store.unlock(&path, "core-password", Instant::now()));
        assert_eq!(st.address.as_deref(), Some(addr.as_str()));
        assert!(store.unlock(&path, "nope", Instant::now()).is_err());
        let _ = fs::remove_dir_all(&dir);

        let dir = temp_dir("jarvis-cross2");
        let path = keystore_in(&dir);
        let mut store = WalletStore::new();
        let created = must(store.create(&path, "desk-password", Instant::now()));
        let parsed = kovanica_keystore::parse(&fs::read(&path).unwrap()).unwrap();
        let sk = kovanica_keystore::signing_key(&parsed, Some("desk-password")).unwrap();
        let seed = kovanica_keystore::seed_from_mnemonic(&created.mnemonic).unwrap();
        assert_eq!(sk.to_bytes(), *seed);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn jarvis_idle_lock_drops_session_and_quote_path() {
        let mut store = WalletStore::new();
        let now = Instant::now();
        store.open_for_test("x".into(), now);
        assert!(store
            .session(now + LOCK_AFTER - Duration::from_millis(1))
            .is_ok());
        assert!(store.session(now + LOCK_AFTER).is_err());
        assert!(store.open.is_none());
        // touching after lock must not reopen
        store.touch(now + LOCK_AFTER + Duration::from_secs(1));
        assert!(store.open.is_none());
    }
}
