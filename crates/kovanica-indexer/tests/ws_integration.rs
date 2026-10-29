use async_trait::async_trait;
use kovanica_execution::TransactionReceipt;
use kovanica_indexer::indexer::Indexer;
use kovanica_indexer::node_reader::{NodeReader, NodeReaderError};
use kovanica_indexer::store::IndexerStore;
use kovanica_indexer::types::IndexedEvent;
use kovanica_types::{Address, Hash};
use std::sync::Arc;
use tokio::net::TcpListener;

struct FakeReader {
    receipts: Vec<(u64, Vec<TransactionReceipt>)>,
}

#[async_trait]
impl NodeReader for FakeReader {
    async fn latest_height(&self) -> Result<u64, NodeReaderError> {
        Ok(self.receipts.last().map(|r| r.0).unwrap_or(0))
    }
    async fn receipts_in_range(
        &self,
        from: u64,
        to: u64,
    ) -> Result<Vec<(u64, Vec<TransactionReceipt>)>, NodeReaderError> {
        Ok(self
            .receipts
            .iter()
            .filter(|(h, _)| *h >= from && *h <= to)
            .cloned()
            .collect())
    }
}

fn synthetic_receipts() -> Vec<(u64, Vec<TransactionReceipt>)> {
    let ev = kovanica_execution::ContractEvent {
        address: Address([1u8; 32]),
        topic: b"transfer".to_vec(),
        data: vec![1, 2, 3],
    };
    let receipt = TransactionReceipt {
        tx_hash: Hash([9u8; 32]),
        success: true,
        gas_used: 0,
        error: None,
        events: vec![ev],
    };
    vec![(1, vec![receipt])]
}

#[tokio::test]
async fn indexes_and_serves_over_ws() {
    let store = Arc::new(
        IndexerStore::new(
            std::env::temp_dir().join(format!("idx_test_{}.redb", std::process::id())),
        )
        .unwrap(),
    );
    let reader = Arc::new(FakeReader {
        receipts: synthetic_receipts(),
    });
    // Need Arc<dyn NodeReader>
    let reader_dyn: Arc<dyn NodeReader> = reader;
    let indexer = Arc::new(Indexer::new(store.clone(), reader_dyn));

    let app = kovanica_indexer::http::router(indexer.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // WS connect first, then sync to exercise push.
    use tokio_tungstenite::connect_async;
    let url = format!("ws://{}/ws", addr);
    let (mut ws, _) = connect_async(&url).await.unwrap();
    use futures::StreamExt;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    indexer.sync_once().await.unwrap();

    // receive pushed event
    let msg = tokio::time::timeout(std::time::Duration::from_secs(2), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let text = msg.into_text().unwrap();
    let ev: IndexedEvent = serde_json::from_str(&text).unwrap();
    assert_eq!(ev.topic, b"transfer");

    // REST check
    let resp = reqwest::get(format!("http://{}/logs", addr)).await.unwrap();
    let logs: Vec<IndexedEvent> = resp.json().await.unwrap();
    assert_eq!(logs.len(), 1);

    let _ = ws.close(None).await;
}
