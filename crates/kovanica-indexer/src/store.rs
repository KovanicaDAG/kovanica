//! Persistent redb store for indexed events.

use crate::types::IndexedEvent;
use redb::{Database, ReadableTable, TableDefinition};
use std::path::Path;
use thiserror::Error;

const T_EVENTS: TableDefinition<u64, Vec<u8>> = TableDefinition::new("idx_events");
const T_HEIGHTS: TableDefinition<u64, Vec<u8>> = TableDefinition::new("idx_heights");
const T_BY_ADDR: TableDefinition<[u8; 32], Vec<u8>> = TableDefinition::new("idx_by_addr");
const T_BY_TOPIC: TableDefinition<[u8; 32], Vec<u8>> = TableDefinition::new("idx_by_topic");
const T_META: TableDefinition<&'static str, u64> = TableDefinition::new("idx_meta");

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Database(#[from] redb::DatabaseError),
    #[error("transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),
    #[error("table error: {0}")]
    Table(#[from] redb::TableError),
    #[error("commit error: {0}")]
    Commit(#[from] redb::CommitError),
    #[error("storage error: {0}")]
    Storage(#[from] redb::StorageError),
    #[error("serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("{0}")]
    Other(String),
}

pub struct IndexerStore {
    db: Database,
}

impl IndexerStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let db = Database::create(path).map_err(StoreError::Database)?;
        let write = db.begin_write().map_err(StoreError::Transaction)?;
        {
            let _ = write.open_table(T_EVENTS)?;
            let _ = write.open_table(T_HEIGHTS)?;
            let _ = write.open_table(T_BY_ADDR)?;
            let _ = write.open_table(T_BY_TOPIC)?;
            let _ = write.open_table(T_META)?;
        }
        write.commit().map_err(StoreError::Commit)?;
        Ok(Self { db })
    }

    fn hash_topic(topic: &[u8]) -> [u8; 32] {
        let mut out = [0u8; 32];
        let h = blake3::hash(topic);
        out.copy_from_slice(h.as_bytes());
        out
    }

    pub fn latest_indexed_height(&self) -> u64 {
        let txn = match self.db.begin_read() {
            Ok(t) => t,
            Err(_) => return 0,
        };
        let table = match txn.open_table(T_META) {
            Ok(t) => t,
            Err(_) => return 0,
        };
        table
            .get("latest_height")
            .ok()
            .flatten()
            .map(|v| v.value())
            .unwrap_or(0)
    }

    pub fn set_latest_indexed_height(&self, height: u64) -> Result<(), StoreError> {
        let txn = self.db.begin_write().map_err(StoreError::Transaction)?;
        {
            let mut table = txn
                .open_table(T_META)
                .map_err(|e| StoreError::Other(e.to_string()))?;
            table
                .insert("latest_height", &height)
                .map_err(|e| StoreError::Other(e.to_string()))?;
        }
        txn.commit().map_err(StoreError::Commit)?;
        Ok(())
    }

    pub fn event_count(&self) -> u64 {
        let txn = match self.db.begin_read() {
            Ok(t) => t,
            Err(_) => return 0,
        };
        let table = match txn.open_table(T_EVENTS) {
            Ok(t) => t,
            Err(_) => return 0,
        };
        {
            let mut c = 0u64;
            if let Ok(iter) = table.iter() {
                for _ in iter {
                    c += 1;
                }
            }
            c
        }
    }

    /// Insert a batch of events for one height. Assigns ids.
    pub fn insert_events(&self, events: &mut [IndexedEvent]) -> Result<(), StoreError> {
        let txn = self.db.begin_write().map_err(StoreError::Transaction)?;
        // next id
        let mut next_id: u64 = {
            let table = txn
                .open_table(T_META)
                .map_err(|e| StoreError::Other(e.to_string()))?;
            table
                .get("next_id")
                .ok()
                .flatten()
                .map(|v| v.value())
                .unwrap_or(1)
        };
        {
            let mut events_table = txn
                .open_table(T_EVENTS)
                .map_err(|e| StoreError::Other(e.to_string()))?;
            let mut heights_table = txn
                .open_table(T_HEIGHTS)
                .map_err(|e| StoreError::Other(e.to_string()))?;
            let mut by_addr = txn
                .open_table(T_BY_ADDR)
                .map_err(|e| StoreError::Other(e.to_string()))?;
            let mut by_topic = txn
                .open_table(T_BY_TOPIC)
                .map_err(|e| StoreError::Other(e.to_string()))?;

            // group ids by height for heights index (all same height usually)
            let mut height_ids: std::collections::BTreeMap<u64, Vec<u64>> =
                std::collections::BTreeMap::new();

            for ev in events.iter_mut() {
                let id = next_id;
                next_id += 1;
                ev.id = id;
                let bytes = bincode::serialize(ev).map_err(StoreError::Serialization)?;
                events_table
                    .insert(id, bytes)
                    .map_err(|e| StoreError::Other(e.to_string()))?;

                height_ids.entry(ev.height).or_default().push(id);

                // address index
                let addr_key = ev.address.0;
                let mut ids: Vec<u64> = match by_addr
                    .get(addr_key)
                    .map_err(|e| StoreError::Other(e.to_string()))?
                {
                    Some(v) => bincode::deserialize(&v.value()).unwrap_or_default(),
                    None => Vec::new(),
                };
                ids.push(id);
                let b = bincode::serialize(&ids).map_err(StoreError::Serialization)?;
                by_addr
                    .insert(addr_key, b)
                    .map_err(|e| StoreError::Other(e.to_string()))?;

                // topic index
                let topic_key = Self::hash_topic(&ev.topic);
                let mut ids: Vec<u64> = match by_topic
                    .get(topic_key)
                    .map_err(|e| StoreError::Other(e.to_string()))?
                {
                    Some(v) => bincode::deserialize(&v.value()).unwrap_or_default(),
                    None => Vec::new(),
                };
                ids.push(id);
                let b = bincode::serialize(&ids).map_err(StoreError::Serialization)?;
                by_topic
                    .insert(topic_key, b)
                    .map_err(|e| StoreError::Other(e.to_string()))?;
            }

            for (h, ids) in height_ids {
                let mut existing: Vec<u64> = match heights_table
                    .get(h)
                    .map_err(|e| StoreError::Other(e.to_string()))?
                {
                    Some(v) => bincode::deserialize(&v.value()).unwrap_or_default(),
                    None => Vec::new(),
                };
                existing.extend(ids);
                let b = bincode::serialize(&existing).map_err(StoreError::Serialization)?;
                heights_table
                    .insert(h, b)
                    .map_err(|e| StoreError::Other(e.to_string()))?;
            }
        }
        {
            let mut table = txn
                .open_table(T_META)
                .map_err(|e| StoreError::Other(e.to_string()))?;
            table
                .insert("next_id", &next_id)
                .map_err(|e| StoreError::Other(e.to_string()))?;
        }
        txn.commit().map_err(StoreError::Commit)?;
        Ok(())
    }

    pub fn get_event(&self, id: u64) -> Result<IndexedEvent, StoreError> {
        let txn = self.db.begin_read().map_err(StoreError::Transaction)?;
        let table = txn
            .open_table(T_EVENTS)
            .map_err(|e| StoreError::Other(e.to_string()))?;
        let v = table
            .get(id)
            .map_err(|e| StoreError::Other(e.to_string()))?
            .ok_or_else(|| StoreError::NotFound(format!("event {id}")))?;
        let ev: IndexedEvent =
            bincode::deserialize(&v.value()).map_err(StoreError::Serialization)?;
        Ok(ev)
    }

    /// Query events with filters.
    pub fn query(&self, filter: &crate::types::LogFilter) -> Result<Vec<IndexedEvent>, StoreError> {
        let txn = self.db.begin_read().map_err(StoreError::Transaction)?;
        let events_table = txn
            .open_table(T_EVENTS)
            .map_err(|e| StoreError::Other(e.to_string()))?;
        let heights_table = txn
            .open_table(T_HEIGHTS)
            .map_err(|e| StoreError::Other(e.to_string()))?;
        let by_addr = txn
            .open_table(T_BY_ADDR)
            .map_err(|e| StoreError::Other(e.to_string()))?;
        let by_topic = txn
            .open_table(T_BY_TOPIC)
            .map_err(|e| StoreError::Other(e.to_string()))?;

        // Determine candidate ids
        // height range
        let from_h = filter.from_height.unwrap_or(0);
        let to_h = filter.to_height.unwrap_or(u64::MAX);
        // collect ids from heights in range
        let mut height_ids: Vec<u64> = Vec::new();
        for row in heights_table
            .range(from_h..=to_h)
            .map_err(|e| StoreError::Other(e.to_string()))?
        {
            let (_, v) = row.map_err(|e| StoreError::Other(e.to_string()))?;
            let ids: Vec<u64> = bincode::deserialize(&v.value()).unwrap_or_default();
            height_ids.extend(ids);
        }
        let mut set: std::collections::BTreeSet<u64> = height_ids.into_iter().collect();
        // if no heights rows, but to_h == u64::MAX and from 0, maybe all events? heights index covers all. ok.

        // address filter
        if let Some(addr_str) = &filter.address {
            if let Ok(addr) = addr_str.parse::<kovanica_types::Address>() {
                if let Some(v) = by_addr
                    .get(addr.0)
                    .map_err(|e| StoreError::Other(e.to_string()))?
                {
                    let ids: Vec<u64> = bincode::deserialize(&v.value()).unwrap_or_default();
                    let s: std::collections::BTreeSet<u64> = ids.into_iter().collect();
                    set = set.intersection(&s).copied().collect();
                } else {
                    set.clear();
                }
            }
        }

        // topic filter
        if let Some(topic_str) = &filter.topic {
            let topic_bytes = if let Some(hex_str) = topic_str.strip_prefix("0x") {
                hex::decode(hex_str).unwrap_or_else(|_| topic_str.as_bytes().to_vec())
            } else {
                topic_str.as_bytes().to_vec()
            };
            let key = Self::hash_topic(&topic_bytes);
            if let Some(v) = by_topic
                .get(key)
                .map_err(|e| StoreError::Other(e.to_string()))?
            {
                let ids: Vec<u64> = bincode::deserialize(&v.value()).unwrap_or_default();
                let s: std::collections::BTreeSet<u64> = ids.into_iter().collect();
                set = set.intersection(&s).copied().collect();
            } else {
                set.clear();
            }
        }

        let limit = filter.limit.unwrap_or(2000).min(2000);
        let mut out = Vec::new();
        for id in set.into_iter().take(limit) {
            if let Some(v) = events_table
                .get(id)
                .map_err(|e| StoreError::Other(e.to_string()))?
            {
                let ev: IndexedEvent =
                    bincode::deserialize(&v.value()).map_err(StoreError::Serialization)?;
                out.push(ev);
            }
        }
        // sort by height then id
        out.sort_by_key(|e| (e.height, e.id));
        Ok(out)
    }
}
