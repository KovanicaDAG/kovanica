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
    let addr = Address([7u8; 32]);
    let ev = kovanica_execution::ContractEvent {
        address: addr,
        topic: b"contract".to_vec(),
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
async fn contract_logs_route_filters_by_address() {
    let store = Arc::new(
        IndexerStore::new(
            std::env::temp_dir().join(format!("idx_contract_{}.redb", std::process::id())),
        )
        .unwrap(),
    );
    let reader = Arc::new(FakeReader {
        receipts: synthetic_receipts(),
    });
    let reader_dyn: Arc<dyn NodeReader> = reader;
    let indexer = Arc::new(Indexer::new(store.clone(), reader_dyn));

    let app = kovanica_indexer::http::router(indexer.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    indexer.sync_once().await.unwrap();

    let contract_addr = Address([7u8; 32]);
    let url = format!("http://{}/contracts/{}/logs", addr, contract_addr);
    let resp = reqwest::get(&url).await.unwrap();
    assert!(resp.status().is_success());
    let logs: Vec<IndexedEvent> = resp.json().await.unwrap();
    assert_eq!(logs.len(), 1);

    // non-matching address returns empty
    let other = Address([8u8; 32]);
    let url2 = format!("http://{}/contracts/{}/logs", addr, other);
    let resp2 = reqwest::get(&url2).await.unwrap();
    let logs2: Vec<IndexedEvent> = resp2.json().await.unwrap();
    assert!(logs2.is_empty());
}
