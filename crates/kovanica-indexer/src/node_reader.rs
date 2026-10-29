//! Traits for reading committed data from a full node.

use crate::types::IndexedEvent;
use async_trait::async_trait;
use kovanica_execution::TransactionReceipt;
use kovanica_storage::Storage;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeReaderError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("execution error: {0}")]
    Execution(String),
    #[error("not available: {0}")]
    Unavailable(String),
    #[error("store error: {0}")]
    Store(String),
}

#[async_trait]
pub trait NodeReader: Send + Sync {
    async fn latest_height(&self) -> Result<u64, NodeReaderError>;
    async fn receipts_in_range(
        &self,
        from: u64,
        to: u64,
    ) -> Result<Vec<(u64, Vec<TransactionReceipt>)>, NodeReaderError>;

    /// Convert receipts into indexed events (default implementation).
    fn receipts_to_events(
        &self,
        height: u64,
        receipts: &[TransactionReceipt],
    ) -> Vec<IndexedEvent> {
        let mut out = Vec::new();
        for (tx_index, receipt) in receipts.iter().enumerate() {
            for (log_index, ev) in receipt.events.iter().enumerate() {
                out.push(IndexedEvent {
                    id: 0,
                    height,
                    tx_hash: receipt.tx_hash.0,
                    tx_index: tx_index as u64,
                    log_index: log_index as u64,
                    address: ev.address,
                    topic: ev.topic.clone(),
                    data: ev.data.clone(),
                    success: receipt.success,
                });
            }
        }
        out
    }
}

/// Storage-backed reader (in-process full node).
pub struct StorageNodeReader {
    pub storage: Arc<Storage>,
}

#[async_trait]
impl NodeReader for StorageNodeReader {
    async fn latest_height(&self) -> Result<u64, NodeReaderError> {
        let txn = self
            .storage
            .begin_read()
            .map_err(|e| NodeReaderError::Storage(e.to_string()))?;
        self.storage
            .consensus()
            .get_committed_leader_height(&txn)
            .map_err(|e| NodeReaderError::Storage(e.to_string()))
    }

    async fn receipts_in_range(
        &self,
        from: u64,
        to: u64,
    ) -> Result<Vec<(u64, Vec<TransactionReceipt>)>, NodeReaderError> {
        kovanica_execution::receipts_in_height_range(&self.storage, from, to)
            .map_err(|e| NodeReaderError::Execution(e.to_string()))
    }
}
