//! Tauri application entry point for the Kovanica Desktop Node App.

use crate::{NetworkProfile, NodeHandle};
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Spawn the node worker on startup
            tauri::async_runtime::spawn(async move {
                match NodeHandle::spawn(NetworkProfile::testnet()).await {
                    Ok(node_handle) => {
                        // Store the handle in app state for commands
                        handle.manage(node_handle.clone());

                        // Forward node events to the frontend
                        let mut event_rx = node_handle.events();
                        while let Ok(event) = event_rx.recv().await {
                            let _ = handle.emit(
                                "node-event",
                                serde_json::to_value(event).unwrap_or_default(),
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to spawn node worker: {e}");
                        let _ = handle.emit(
                            "node-event",
                            serde_json::json!({
                                "type": "Error",
                                "data": { "message": format!("Node startup failed: {e}") }
                            }),
                        );
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            produce_block,
            submit_tx,
            save_snapshot,
            save_checkpoint,
            shutdown_node
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_status(
    handle: tauri::State<'_, NodeHandle>,
) -> Result<crate::NodeStatus, String> {
    match handle.send(crate::WorkerCmd::GetStatus).await {
        Ok(crate::WorkerResp::Status(s)) => Ok(s),
        Ok(_) => Err("unexpected response".into()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn produce_block(handle: tauri::State<'_, NodeHandle>) -> Result<Option<String>, String> {
    match handle.send(crate::WorkerCmd::ProduceBlock).await {
        Ok(crate::WorkerResp::ProduceBlock(r)) => {
            r.map(|b| b.map(|id| id.to_string())).map_err(|e| e.to_string())
        }
        Ok(_) => Err("unexpected response".into()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn submit_tx(_handle: tauri::State<'_, NodeHandle>, _tx_hex: String) -> Result<String, String> {
    // Parse hex transaction (placeholder - real impl would decode)
    // For now just return not implemented
    Err("submit_tx not yet implemented".into())
}

#[tauri::command]
async fn save_snapshot(handle: tauri::State<'_, NodeHandle>) -> Result<(), String> {
    match handle.send(crate::WorkerCmd::SaveSnapshot).await {
        Ok(crate::WorkerResp::Ok) => Ok(()),
        Ok(_) => Err("unexpected response".into()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn save_checkpoint(handle: tauri::State<'_, NodeHandle>) -> Result<(), String> {
    match handle
        .send(crate::WorkerCmd::SaveCheckpoint)
        .await
    {
        Ok(crate::WorkerResp::Ok) => Ok(()),
        Ok(_) => Err("unexpected response".into()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn shutdown_node(handle: tauri::State<'_, NodeHandle>) -> Result<(), String> {
    handle.shutdown().await.map_err(|e| e.to_string())
}
