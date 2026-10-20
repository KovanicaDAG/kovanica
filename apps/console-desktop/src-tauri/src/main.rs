//! Kovanica Console desktop shell.
//!
//! The webview only displays what this process returns. RPC calls, chain-id
//! checks, and key handling stay on this side of the bridge.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod amount;
mod connection;
mod history;
mod transfer;
mod wallet;

use amount::{format_kuna_atoms, parse_kuna_atoms};

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use connection::{
    endpoint_label, fetch_status, read_saved_url, resolve_rpc_url, write_saved_url, HttpRpc,
    NodeStatus, RPC_URL_ENV,
};
use serde::Serialize;
use std::time::Instant;
use tauri::Emitter;
use tauri::Manager;
use tauri::State;
use zeroize::Zeroizing;

/// Process-local wallet session. The keystore file lives in the app data directory.
///
/// `idle` is read so the lock thread stays alive for the process. Drop joins it.
struct WalletApp {
    idle: wallet::IdleLockThread,
    shared: Arc<wallet::WalletShared>,
}

/// Endpoint the screen should show and edit.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionConfig {
    rpc_url: String,
    endpoint_label: String,
}

fn endpoint_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|_| "Could not open the app data directory.".to_string())?;
    std::fs::create_dir_all(&dir)
        .map_err(|_| "Could not open the app data directory.".to_string())?;
    Ok(dir.join("rpc-endpoint.json"))
}

fn current_url(app: &tauri::AppHandle) -> Result<String, String> {
    let saved = read_saved_url(&endpoint_path(app)?).map_err(|err| err.to_string())?;
    let env_value = env::var(RPC_URL_ENV).ok();
    Ok(resolve_rpc_url(saved.as_deref(), env_value.as_deref()))
}

fn config_for(rpc_url: String) -> ConnectionConfig {
    let endpoint_label = endpoint_label(&rpc_url);
    ConnectionConfig {
        rpc_url,
        endpoint_label,
    }
}

/// Saved endpoint, or the environment, or the public testnet DNS name.
#[tauri::command]
fn connection_config(app: tauri::AppHandle) -> Result<ConnectionConfig, String> {
    current_url(&app).map(config_for)
}

/// Store an endpoint in the app data directory and use it from now on.
#[tauri::command]
fn save_connection(app: tauri::AppHandle, rpc_url: String) -> Result<ConnectionConfig, String> {
    let saved = write_saved_url(&endpoint_path(&app)?, &rpc_url).map_err(|err| err.to_string())?;
    Ok(config_for(saved))
}

/// Forget the saved endpoint and return to the environment or the DNS default.
#[tauri::command]
fn reset_connection(app: tauri::AppHandle) -> Result<ConnectionConfig, String> {
    let path = endpoint_path(&app)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|_| "Could not reset the RPC setting.".to_string())?;
    }
    current_url(&app).map(config_for)
}

/// Parse a KUNA amount. The result is a decimal atom string.
#[tauri::command]
fn parse_kuna(amount: String) -> Result<String, String> {
    parse_kuna_atoms(&amount).map(|atoms| atoms.to_string())
}

/// Format an atom string as KUNA. The input is decimal, never a float.
#[tauri::command]
fn format_kuna(atoms: String) -> Result<String, String> {
    let atoms = parse_atom_string(&atoms)?;
    Ok(format_kuna_atoms(atoms))
}

fn wallet_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|_| "Could not open the app data directory.".to_string())?;
    std::fs::create_dir_all(&dir)
        .map_err(|_| "Could not open the app data directory.".to_string())?;
    Ok(dir.join(wallet::KEYSTORE_FILE))
}

fn with_wallet<T>(
    state: &State<'_, WalletApp>,
    body: impl FnOnce(&mut wallet::WalletStore) -> Result<T, String>,
) -> Result<T, String> {
    let _idle = &state.idle;
    let mut store = state.shared.lock_store();
    let result = body(&mut store);
    drop(store);
    state.shared.wake.notify_one();
    result
}

/// Whether a wallet file exists, and whether the session is open.
#[tauri::command]
fn wallet_status(
    app: tauri::AppHandle,
    state: State<'_, WalletApp>,
) -> Result<wallet::WalletStatus, String> {
    let path = wallet_file(&app)?;
    with_wallet(&state, |store| store.status(&path, Instant::now()))
}

/// Create a key. The phrase is returned once.
#[tauri::command]
fn wallet_create(
    app: tauri::AppHandle,
    state: State<'_, WalletApp>,
    passphrase: String,
) -> Result<wallet::NewWallet, String> {
    let path = wallet_file(&app)?;
    let passphrase = Zeroizing::new(passphrase);
    with_wallet(&state, |store| {
        store.create(&path, passphrase.as_str(), Instant::now())
    })
}

/// Import a 24-word seed. The phrase is not returned.
#[tauri::command]
fn wallet_import(
    app: tauri::AppHandle,
    state: State<'_, WalletApp>,
    mnemonic: String,
    passphrase: String,
) -> Result<wallet::WalletStatus, String> {
    let path = wallet_file(&app)?;
    let mnemonic = Zeroizing::new(mnemonic);
    let passphrase = Zeroizing::new(passphrase);
    with_wallet(&state, |store| {
        store.import(
            &path,
            mnemonic.as_str(),
            passphrase.as_str(),
            Instant::now(),
        )
    })
}

/// Open an existing wallet. A wrong password does not reveal the phrase.
#[tauri::command]
fn wallet_unlock(
    app: tauri::AppHandle,
    state: State<'_, WalletApp>,
    passphrase: String,
) -> Result<wallet::WalletStatus, String> {
    let path = wallet_file(&app)?;
    let passphrase = Zeroizing::new(passphrase);
    with_wallet(&state, |store| {
        store.unlock(&path, passphrase.as_str(), Instant::now())
    })
}

/// Drop the session. The webview hears [`wallet::WALLET_LOCKED_EVENT`] at once.
#[tauri::command]
fn wallet_lock(
    app: tauri::AppHandle,
    state: State<'_, WalletApp>,
) -> Result<wallet::WalletStatus, String> {
    let path = wallet_file(&app)?;
    let (was_open, status) = with_wallet(&state, |store| {
        let was_open = store.lock();
        let status = store.status(&path, Instant::now())?;
        Ok((was_open, status))
    })?;
    if was_open {
        let _ = app.emit(wallet::WALLET_LOCKED_EVENT, ());
    }
    Ok(status)
}

/// Note that the user is still at the keyboard.
///
/// This is the only IPC that moves the idle deadline of an open session.
#[tauri::command]
fn wallet_touch(state: State<'_, WalletApp>) -> Result<(), String> {
    with_wallet(&state, |store| {
        store.touch(Instant::now());
        Ok(())
    })
}

/// Quote a transfer. The keystore is not decrypted.
#[tauri::command]
async fn wallet_quote(
    app: tauri::AppHandle,
    shown_chain_id: Option<String>,
    recipient: String,
    amount: String,
) -> Result<transfer::TransferQuote, String> {
    let url = current_url(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let rpc = HttpRpc::new(url.clone()).map_err(|err| err.to_string())?;
        let wallet = app.state::<WalletApp>();
        with_wallet(&wallet, |store| {
            store.quote_on(
                &rpc,
                &url,
                transfer::TransferDraft {
                    shown_chain_id: shown_chain_id.as_deref(),
                    confirm_mainnet: false,
                    recipient: &recipient,
                    amount: &amount,
                },
                Instant::now(),
            )
        })
    })
    .await
    .map_err(|_| "The quote was interrupted.".to_string())?
}

/// Balance the node reported. It is not a light-client proof.
#[tauri::command]
async fn wallet_balance(app: tauri::AppHandle) -> Result<history::ReportedBalance, String> {
    let url = current_url(&app)?;
    let path = wallet_file(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let rpc = HttpRpc::new(url.clone()).map_err(|err| err.to_string())?;
        let wallet = app.state::<WalletApp>();
        let address = with_wallet(&wallet, |store| {
            store
                .status(&path, Instant::now())?
                .address
                .ok_or_else(|| "Create or import a wallet first.".to_string())
        })?;
        history::reported_balance(&rpc, &address)
    })
    .await
    .map_err(|_| "The balance request was interrupted.".to_string())?
}

/// Transfers the node reported for this wallet. They are not proven.
#[tauri::command]
async fn wallet_history(app: tauri::AppHandle) -> Result<history::HistoryPage, String> {
    let url = current_url(&app)?;
    let path = wallet_file(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let rpc = HttpRpc::new(url.clone()).map_err(|err| err.to_string())?;
        let wallet = app.state::<WalletApp>();
        let address = with_wallet(&wallet, |store| {
            store
                .status(&path, Instant::now())?
                .address
                .ok_or_else(|| "Create or import a wallet first.".to_string())
        })?;
        history::reported_history(&rpc, &address)
    })
    .await
    .map_err(|_| "The history request was interrupted.".to_string())?
}

/// Sign a quoted transfer. Chain 1 is refused until `confirm_mainnet` is set.
#[tauri::command]
async fn wallet_send(
    app: tauri::AppHandle,
    shown_chain_id: Option<String>,
    confirm_mainnet: bool,
    recipient: String,
    amount: String,
) -> Result<transfer::SendReceipt, String> {
    let url = current_url(&app)?;
    let path = wallet_file(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let rpc = HttpRpc::new(url.clone()).map_err(|err| err.to_string())?;
        let status = fetch_status(&rpc, &url).map_err(|err| err.to_string())?;
        let wallet = app.state::<WalletApp>();
        with_wallet(&wallet, |store| {
            store.send_prepared(
                &rpc,
                &path,
                &status,
                transfer::TransferDraft {
                    shown_chain_id: shown_chain_id.as_deref(),
                    confirm_mainnet,
                    recipient: &recipient,
                    amount: &amount,
                },
                Instant::now(),
            )
        })
    })
    .await
    .map_err(|_| "The transfer was interrupted.".to_string())?
}

fn parse_atom_string(text: &str) -> Result<u64, String> {
    let text = text.trim();
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("Enter an atom amount.".to_string());
    }
    text.parse::<u64>()
        .map_err(|_| "That amount is larger than KUNA can send.".to_string())
}

/// Chain id, height, leader round, epoch, and peers. Fails with a clear error
/// when the node cannot be reached.
#[tauri::command]
async fn node_status(app: tauri::AppHandle) -> Result<NodeStatus, String> {
    let url = current_url(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let rpc = HttpRpc::new(url.clone())?;
        fetch_status(&rpc, &url)
    })
    .await
    .map_err(|_| "The status request was interrupted.".to_string())?
    .map_err(|err| err.to_string())
}

/// Forwards a session lock to the webview. The idle thread calls this.
struct EmitSession {
    app: tauri::AppHandle,
}

impl wallet::SessionNotify for EmitSession {
    fn wallet_locked(&self) {
        let _ = self.app.emit(wallet::WALLET_LOCKED_EVENT, ());
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let shared = Arc::new(wallet::WalletShared::new());
            let notify = Arc::new(EmitSession {
                app: app.handle().clone(),
            });
            let idle = wallet::IdleLockThread::spawn_with(Arc::clone(&shared), notify);
            app.manage(WalletApp { idle, shared });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connection_config,
            save_connection,
            reset_connection,
            node_status,
            parse_kuna,
            format_kuna,
            wallet_status,
            wallet_create,
            wallet_import,
            wallet_unlock,
            wallet_lock,
            wallet_touch,
            wallet_quote,
            wallet_send,
            wallet_balance,
            wallet_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running Kovanica Console");
}
