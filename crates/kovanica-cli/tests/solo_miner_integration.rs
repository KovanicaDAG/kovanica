//! Integration test for the standalone solo CPU miner CLI tooling (`kovanica miner run`).
//!
//! Spawns a mock Kovanica node JSON-RPC endpoint serving candidate work templates,
//! runs the `SoloMiner` with 2 worker threads, computes KovanicaHash PoW solutions,
//! submits them via `kovanica_submitWork`, and verifies real-time acceptance and graceful shutdown.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use kovanica_cli::miner::SoloMiner;
use kovanica_cli::rpc::RpcClient;
use kovanica_types::block::BlockHeader;
use kovanica_types::signing::chain_id::HYBRID_TESTNET;
use kovanica_types::{Address, Hash};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Spawn a mock Kovanica node JSON-RPC HTTP server.
/// Returns the RPC URL and submission counter.
async fn spawn_mock_rpc_server(
    server_shutdown: Arc<AtomicBool>,
) -> (String, Arc<AtomicU64>, Arc<AtomicU64>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let rpc_url = format!("http://127.0.0.1:{port}/rpc");

    let get_work_count = Arc::new(AtomicU64::new(0));
    let submit_work_count = Arc::new(AtomicU64::new(0));

    let gw_count = get_work_count.clone();
    let sw_count = submit_work_count.clone();

    tokio::spawn(async move {
        while !server_shutdown.load(Ordering::Relaxed) {
            let accept_res = tokio::select! {
                res = listener.accept() => res,
                _ = tokio::time::sleep(Duration::from_millis(50)) => continue,
            };

            let Ok((mut socket, _)) = accept_res else {
                continue;
            };

            let gw = gw_count.clone();
            let sw = sw_count.clone();

            tokio::spawn(async move {
                let mut buffer = vec![0u8; 8192];
                let mut total_read = 0;

                // Read until we get header terminator
                loop {
                    let n = match socket.read(&mut buffer[total_read..]).await {
                        Ok(0) => return,
                        Ok(n) => n,
                        Err(_) => return,
                    };
                    total_read += n;

                    if let Some(pos) = buffer[..total_read]
                        .windows(4)
                        .position(|w| w == b"\r\n\r\n")
                    {
                        let header_bytes = &buffer[..pos];
                        let header_str = String::from_utf8_lossy(header_bytes);

                        let mut content_length = 0usize;
                        for line in header_str.lines() {
                            if line.to_ascii_lowercase().starts_with("content-length:") {
                                if let Some(val) = line.split(':').nth(1) {
                                    content_length = val.trim().parse().unwrap_or(0);
                                }
                            }
                        }

                        let body_start = pos + 4;
                        while total_read - body_start < content_length {
                            let n = match socket.read(&mut buffer[total_read..]).await {
                                Ok(0) => break,
                                Ok(n) => n,
                                Err(_) => break,
                            };
                            total_read += n;
                        }

                        let body_bytes = &buffer[body_start..body_start + content_length];
                        if let Ok(req) = serde_json::from_slice::<Value>(body_bytes) {
                            let method = req.get("method").and_then(Value::as_str).unwrap_or("");
                            let id = req.get("id").cloned().unwrap_or(json!(1));

                            let response_json = match method {
                                "kovanica_getWork" => {
                                    gw.fetch_add(1, Ordering::Relaxed);
                                    let template_header = BlockHeader::new(
                                        HYBRID_TESTNET,
                                        1,
                                        0,
                                        vec![],
                                        Hash([0x44; 32]),
                                        1_700_000_000_000,
                                        0x207fffff, // easy difficulty target (~50% per nonce)
                                        0,
                                    );
                                    json!({
                                        "jsonrpc": "2.0",
                                        "result": {
                                            "chain_id": HYBRID_TESTNET,
                                            "round": 1,
                                            "author": 0,
                                            "parents": [],
                                            "parent_refs": [],
                                            "merkle_root": format!("0x{}", hex::encode([0x44; 32])),
                                            "timestamp": 1_700_000_000_000u64,
                                            "target_bits": "0x207fffff",
                                            "target": "0x7fffff0000000000000000000000000000000000000000000000000000000000",
                                            "preimage": format!("0x{}", hex::encode(template_header.preimage())),
                                            "transactions_count": 0,
                                            "header": template_header
                                        },
                                        "id": id
                                    })
                                }
                                "kovanica_submitWork" => {
                                    sw.fetch_add(1, Ordering::Relaxed);
                                    let params = req.get("params").cloned().unwrap_or(Value::Null);
                                    let header_val = match params {
                                        Value::Object(ref m) => m.get("header").cloned(),
                                        Value::Array(ref a) if !a.is_empty() => Some(a[0].clone()),
                                        _ => None,
                                    };

                                    let verified = if let Some(hv) = header_val {
                                        if let Ok(hdr) = serde_json::from_value::<BlockHeader>(hv) {
                                            hdr.verify_pow().is_ok()
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    };

                                    json!({
                                        "jsonrpc": "2.0",
                                        "result": {
                                            "accepted": verified,
                                            "digest": "0x1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff",
                                            "pow_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                                        },
                                        "id": id
                                    })
                                }
                                _ => json!({
                                    "jsonrpc": "2.0",
                                    "error": { "code": -32601, "message": "Method not found" },
                                    "id": id
                                }),
                            };

                            let resp_body = serde_json::to_vec(&response_json).unwrap();
                            let resp_headers = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                resp_body.len()
                            );
                            let _ = socket.write_all(resp_headers.as_bytes()).await;
                            let _ = socket.write_all(&resp_body).await;
                            let _ = socket.flush().await;
                        }
                        return;
                    }
                }
            });
        }
    });

    (rpc_url, get_work_count, submit_work_count)
}

#[tokio::test]
async fn solo_miner_fetches_work_mines_and_submits_solution() {
    let server_shutdown = Arc::new(AtomicBool::new(false));
    let (rpc_url, gw_count, sw_count) = spawn_mock_rpc_server(server_shutdown.clone()).await;

    // Payout address: canonical base32 address
    let payout_raw = Address([0x99; 32]);
    let payout_addr = payout_raw.encode();

    // 2 worker threads, 50ms poll interval for rapid testing
    let miner = SoloMiner::new(RpcClient::new(&rpc_url), payout_addr, HYBRID_TESTNET, 2, 50);

    let stats = miner.stats();
    assert_eq!(stats.total_hashes(), 0);
    assert_eq!(stats.blocks_found(), 0);
    assert_eq!(stats.blocks_accepted(), 0);

    // Run miner in background task
    let miner_handle = tokio::spawn({
        let miner_ref = Arc::new(miner);
        let m = miner_ref.clone();
        async move { m.run().await }
    });

    // Wait until at least 1 block is accepted, with a 10s deadline
    let start = tokio::time::Instant::now();
    let timeout = Duration::from_secs(10);
    while stats.blocks_accepted() == 0 {
        if start.elapsed() > timeout {
            panic!(
                "timeout waiting for solo miner to solve and submit block (hashes: {}, found: {}, accepted: {})",
                stats.total_hashes(),
                stats.blocks_found(),
                stats.blocks_accepted()
            );
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Verify stats
    assert!(stats.total_hashes() > 0);
    assert!(stats.blocks_found() >= 1);
    assert!(stats.blocks_accepted() >= 1);
    assert_eq!(stats.blocks_rejected(), 0);

    assert!(gw_count.load(Ordering::Relaxed) >= 1);
    assert!(sw_count.load(Ordering::Relaxed) >= 1);

    // Stop miner and shutdown mock server
    server_shutdown.store(true, Ordering::SeqCst);
    miner_handle.abort();
}
