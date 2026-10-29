use axum::{routing::post, Json, Router};
use kovanica_sdk::RpcClient;
use std::result::Result;
use serde_json::{json, Value};
use tokio::net::TcpListener;

#[tokio::test]
async fn mock_rpc_call() {
    let app = Router::new().route(
        "/",
        post(|Json(_req): Json<Value>| async {
            Json(json!({"jsonrpc":"2.0","id":1,"result":"0x10"}))
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = RpcClient::new(format!("http://{}", addr));
    let n: u64 = client.block_number().await.unwrap();
    assert_eq!(n, 16);
}

#[tokio::test]
async fn mock_rpc_error_path() {
    let app = Router::new().route(
        "/",
        post(|Json(_req): Json<Value>| async {
            Json(json!({"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"not found"}}))
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });

    let client = RpcClient::new(format!("http://{}", addr));
    let res: Result<u64, _> = client.block_number().await;
    assert!(res.is_err());
}
