//! Kovanica Console desktop shell.
//!
//! The webview only displays what this process returns. RPC calls, chain-id
//! checks, and later key handling stay on this side of the bridge.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod amount;
mod connection;

use amount::{format_kuna_atoms, parse_kuna_atoms};

use std::env;
use std::path::PathBuf;

use connection::{
    endpoint_label, fetch_status, read_saved_url, resolve_rpc_url, write_saved_url, HttpRpc,
    NodeStatus, RPC_URL_ENV,
};
use serde::Serialize;
use tauri::Manager;

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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            connection_config,
            save_connection,
            reset_connection,
            node_status,
            parse_kuna,
            format_kuna
        ])
        .run(tauri::generate_context!())
        .expect("error while running Kovanica Console");
}
