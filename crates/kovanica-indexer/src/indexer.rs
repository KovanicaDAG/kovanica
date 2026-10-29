//! Indexer orchestration: sync loop + broadcast.

use crate::node_reader::{NodeReader, NodeReaderError};
use crate::store::IndexerStore;
use crate::types::{IndexedEvent, IndexerStatus};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::info;

pub struct Indexer {
    pub store: Arc<IndexerStore>,
    pub node: Arc<dyn NodeReader>,
    pub tx: broadcast::Sender<IndexedEvent>,
    pub latest_node_height: Arc<RwLock<u64>>,
}

impl Indexer {
    pub fn new(store: Arc<IndexerStore>, node: Arc<dyn NodeReader>) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            store,
            node,
            tx,
            latest_node_height: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn sync_once(&self) -> Result<usize, NodeReaderError> {
        let latest = self.node.latest_height().await?;
        *self.latest_node_height.write().await = latest;
        let current = self.store.latest_indexed_height();
        if latest <= current {
            return Ok(0);
        }
        let from = current + 1;
        let to = latest;
        let batches = self.node.receipts_in_range(from, to).await?;
        let mut total = 0;
        for (height, receipts) in batches {
            let mut events = self.node.receipts_to_events(height, &receipts);
            if events.is_empty() {
                // still advance height
                self.store.set_latest_indexed_height(height).ok();
                continue;
            }
            self.store
                .insert_events(&mut events)
                .map_err(|e| NodeReaderError::Store(e.to_string()))?;
            total += events.len();
            for ev in events {
                let _ = self.tx.send(ev);
            }
            self.store.set_latest_indexed_height(height).ok();
        }
        // ensure latest height advanced even if no receipts rows
        self.store.set_latest_indexed_height(to).ok();
        info!("indexed {} events up to height {}", total, to);
        Ok(total)
    }

    pub async fn status(&self) -> IndexerStatus {
        let node_h = *self.latest_node_height.read().await;
        IndexerStatus {
            latest_indexed_height: self.store.latest_indexed_height(),
            latest_node_height: node_h,
            event_count: self.store.event_count(),
        }
    }
}
