//! Kovanica indexer: event/log indexing + WebSocket push.

#![deny(unsafe_code)]
#![allow(clippy::result_large_err)]

pub mod config;
pub mod http;
pub mod indexer;
pub mod node_reader;
pub mod store;
pub mod types;

pub use config::Config;
pub use indexer::Indexer;
pub use node_reader::{NodeReader, StorageNodeReader};
pub use store::IndexerStore;
pub use types::{IndexedEvent, IndexerStatus, LogFilter};
