---
name: rust-indexer-archive
description: Use when building a blockchain indexer or archive node: block reprocessing, state indexing, event/log indexing, GraphQL/REST APIs for chain data, pruning strategies, and maintaining an indexed view of chain state.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, blockchain, indexer, archive, node, graphql, rest-api, event-indexing, state-indexing, reprocessing]
    related_skills: [rust-block-propagation, rust-account-ledger, rust-utxo-ledger, rust-databases]
---

# Rust Indexer and Archive Nodes

## Overview

An indexer is a node that processes blockchain data and builds queryable indexes on top of it. An archive node is a full node that keeps all historical state (not pruned). Indexers provide APIs (GraphQL, REST, RPC) for querying chain data: transactions, accounts, events, states, etc.

This skill covers building an indexer in Rust: block processing, state indexing, event extraction, API layers, pruning strategies, and the differences between full, archival, and pruned nodes.

## When to Use

- Building a chain indexer for querying transactions, accounts, events
- Building an archive node that keeps full historical state
- Designing an indexing pipeline (block → parse → index → store)
- Exposing GraphQL/REST APIs for chain data
- Handling re-orgs in the index (correcting indexes when the chain reorganizes)
- Deciding what to index (all blocks, specific events, specific accounts)

**Don't use for:** consensus or block validation (the indexer trusts a full node for validity), or wallet key management.

## Indexer Architecture

```
┌─────────────┐    ┌──────────────┐    ┌───────────────┐    ┌─────────────┐
│  Chain Node │───▶│ Block Queue  │───▶│ Block Processor│───▶│  Index Store │
│ (full node) │    │ (new blocks)  │    │ (parse, index)  │    │ (DB, graph)  │
└─────────────┘    └──────────────┘    └───────────────┘    └─────────────┘
                                                  │
                                                  ▼
                                         ┌───────────────┐
                                         │   API Layer   │
                                         │ (GraphQL/REST)│
                                         └───────────────┘
```

**Components:**
- **Chain node:** provides block data (new blocks, re-orgs). Can be a full node, light client, or RPC client to an existing node.
- **Block queue:** buffers new blocks for processing (ordered, handles backpressure).
- **Block processor:** parses blocks, extracts data (txs, events, state diffs), updates indexes.
- **Index store:** persistent storage for indexes (PostgreSQL, SQLite, RocksDB, graph DB).
- **API layer:** exposes the indexes via GraphQL, REST, or RPC.

## Block Processing Pipeline

### Processing a Single Block

```rust
use rustoneye::Block;   // or chain-specific block type
use rustoneye::Transaction;
use tokio::sync::mpsc;

pub struct BlockProcessor {
    db: IndexDatabase,
    chain_client: ChainClient,
}

impl BlockProcessor {
    pub async fn process_block(&mut self, block: Block) -> Result<(), Error> {
        // 1. Extract transactions
        for tx in &block.transactions {
            self.index_transaction(tx, block.height).await?;
        }

        // 2. Extract events/logs (if the chain has events)
        for event in &block.events {
            self.index_event(event, block.height).await?;
        }

        // 3. Update account/state indexes
        for (address, state) in &block.state_diffs {
            self.index_state_update(address, state, block.height).await?;
        }

        // 4. Update block index
        self.db.insert_block(block).await?;

        // 5. Update the "latest" pointer
        self.db.set_latest_height(block.height).await?;

        Ok(())
    }
}
```

### Processing Re-orgs

When the chain reorganizes (a different fork becomes canonical), the indexer must undo the indexes for the old fork and apply the new one.

```rust
pub async fn process_reorg(&mut self, old_tip: BlockHash, new_tip: BlockHash) -> Result<(), Error> {
    // 1. Find the common ancestor
    let ancestor_height = self.find_common_ancestor(old_tip, new_tip).await?;

    // 2. Roll back indexes from old_tip down to ancestor
    for height in (ancestor_height + 1..=old_tip.height).rev() {
        self.rollback_block(height).await?;
    }

    // 3. Apply new fork blocks from ancestor+1 to new_tip
    let new_blocks = self.chain_client.get_blocks(ancestor_height + 1, new_tip.height).await?;
    for block in new_blocks {
        self.process_block(block).await?;
    }

    Ok(())
}
```

**Re-org handling is critical** — without it, the index becomes stale/corrupted during chain reorganizations.

### Event Indexer (Log Indexer)

Many chains have events/logs emitted by transactions (Ethereum logs, Substrate events, Solana logs). Indexing these lets you query "all Transfer events for token X" or "all votes for proposal Y."

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexedEvent {
    pub event_id: u64,         // auto-increment
    pub block_height: u64,
    pub tx_index: u32,
    pub log_index: u32,
    pub contract_address: Address,
    pub event_name: String,    // e.g., "Transfer"
    pub topic0: Topic,         // first topic (event signature hash)
    pub topics: Vec<Topic>,
    pub data: Vec<u8>,
    pub decoded_params: Option<serde_json::Value>,   // if we decode the event
}
```

**Decoding events:** event data is usually binary. To make it queryable, decode it using the contract's ABI (for EVM chains) or event schema (for Substrate/Cosmos). Store both raw and decoded.

## Archive Node vs Pruned Node

| Type | Description | Storage | Use case |
|---|---|---|---|
| **Full node** | Validates all blocks, keeps recent state | Recent state + block data (pruned after some depth) | General participation |
| **Archive node** | Keeps all historical state (no pruning) | All blocks + all historical state | Historical queries, indexers |
| **Pruned node** | Prunes old block data and/or state | Configurable retention | Low-storage nodes |
| **Indexer node** | Builds indexes on top of chain data | Index store (DB) + chain data (may be full or light) | Query APIs |

For a chain indexer, you typically need:
- Access to all blocks (full node or archive node or RPC to one).
- An index store that's separate from the chain's state DB.
- Re-org handling (the chain node notifies you of re-orgs, or you detect them).

## Storage for Indexes

### PostgreSQL (relational, good for complex queries)

```rust
use sqlx::PgPool;

pub struct PostgresIndexer {
    pool: PgPool,
}

impl PostgresIndexer {
    pub async fn index_transaction(&self, tx: &Transaction, height: u64) -> Result<(), Error> {
        sqlx::query!(
            "INSERT INTO transactions (txid, block_height, tx_index, from, to, value, fee, data)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            tx.txid.as_slice(),
            height,
            tx.index,
            tx.from.as_slice(),
            tx.to.as_slice(),
            tx.value,
            tx.fee,
            &tx.data,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```

### RocksDB / Key-Value (fast, good for state snapshots, Merkle proofs)

For state indexes that are keyed by address/hash and need fast lookups, a key-value store works well.

```rust
use rocksdb::{DB, Options};

let mut opts = Options::default();
opts.create_if_missing(true);
let db = DB::open(&opts, "/path/to/index").unwrap();

// Store state by address
db.put(address.as_bytes(), state_serialization).unwrap();

// Get state by address
let state = db.get(address.as_bytes()).unwrap();
```

### GraphQL (query layer)

```toml
[dependencies]
async-graphql = "7"
```

```rust
use async_graphql::{Object, Result};

#[derive(Object)]
pub struct QueryRoot {
    // Query transactions by address
    async fn transactions(
        &self,
        ctx: &Context<'_>,
        address: Address,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Transaction>> {
        let pool = ctx.data::<PgPool>()?;
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query_as!(
            TransactionRow,
            r#"
            SELECT txid, block_height, tx_index, from, to, value, fee
            FROM transactions
            WHERE from = $1 OR to = $1
            ORDER BY block_height DESC, tx_index DESC
            LIMIT $2 OFFSET $3
            "#,
            address.as_slice(),
            limit as i64,
            offset as i64,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(Transaction::from).collect())
    }
}
```

## Indexing Strategies

### Full Indexing (every block, every tx, every event)

- Most flexible, most storage.
- Can answer any historical query.
- Heavy to maintain (re-orgs, large data volume).

### Selective Indexing (specific contracts, addresses, event types)

- Lighter, faster to build.
- Only answers specific queries.
- Common for DeFi dashboards (index specific tokens, pools, protocols).

```rust
pub struct SelectiveIndexer {
    watched_addresses: HashSet<Address>,
    watched_contracts: HashSet<Address>,
    watched_events: HashSet<EventSignature>,
}

impl SelectiveIndexer {
    pub async fn process_block(&mut self, block: Block) -> Result<(), Error> {
        for tx in &block.transactions {
            // Only index if relevant
            if self.watched_addresses.contains(&tx.from)
                || self.watched_addresses.contains(&tx.to)
                || self.watched_contracts.contains(&tx.contract_address)
            {
                self.index_transaction(tx, block.height).await?;
            }
        }

        for event in &block.events {
            if self.watched_events.contains(&event.signature)
                || self.watched_addresses.contains(&event.sender)
            {
                self.index_event(event, block.height).await?;
            }
        }
    }
}
```

### Event-Driven Indexing (substrate/polkadot style)

Some chains (Substrate) emit events per block that are easy to index. The indexer subscribes to events and indexes them.

```rust
// Substrate-style: query events from a block
let events = chain_client.get_block_events(block_hash).await?;
for event in events {
    match event {
        SubstrateEvent::Transfer(transfer) => {
            self.index_transfer(&transfer, block.height).await?;
        }
        SubstrateEvent::Staking(staking) => {
            self.index_staking(&staking, block.height).await?;
        }
    }
}
```

## API Design

### REST API

```rust
use axum::{routing::get, Router, extract::Path};

let app = Router::new()
    .route("/tx/:txid", get(get_transaction))
    .route("/address/:address/txs", get(address_transactions))
    .route("/block/:height", get(get_block));

async fn get_transaction(Path(txid): Path<String>) -> Result<Json<Transaction>, Error> {
    let tx = db.get_transaction(&txid).await?;
    Ok(Json(tx))
}

async fn address_transactions(
    Path(address): Path<String>,
    Query(params): Query<AddressQuery>,
) -> Result<Json<Vec<Transaction>>, Error> {
    let txs = db.get_address_transactions(&address, params.limit, params.offset).await?;
    Ok(Json(txs))
}
```

### GraphQL (flexible queries)

```rust
#[derive(Object)]
pub struct Query {
    async fn transaction(&self, ctx: &Context<'_>, txid: String) -> Result<Option<Transaction>> {
        let db = ctx.data::<IndexerDb>()?;
        Ok(db.get_transaction(&txid).await?)
    }

    async fn transactions_by_address(
        &self,
        ctx: &Context<'_>,
        address: String,
        limit: Option<u32>,
    ) -> Result<Vec<Transaction>> {
        let db = ctx.data::<IndexerDb>()?;
        Ok(db.get_address_transactions(&address, limit.unwrap_or(100) as usize).await?)
    }

    async fn block(&self, ctx: &Context<'_>, height: u64) -> Result<Option<Block>> {
        let db = ctx.data::<IndexerDb>()?;
        Ok(db.get_block(height).await?)
    }
}
```

## Pruning the Index

Over time, indexes grow. Strategies:

- **Prune old blocks:** remove index entries for blocks below a certain height (if historical queries aren't needed).
- **Compress old data:** move old data to cold storage (slower queries, less disk).
- **Separate hot/cold indexes:** recent data in fast storage, old data in cheaper storage.
- **Keep only what's needed:** if the indexer only indexes specific events, don't store full blocks.

```rust
pub async fn prune_before(&self, height: u64) -> Result<(), Error> {
    // Remove transaction indexes below height
    sqlx::query!("DELETE FROM transactions WHERE block_height < $1", height)
        .execute(&self.pool)
        .await?;

    // Update the "earliest indexed height" metadata
    self.db.set_earliest_height(height).await?;
}
```

## Verification Checklist

- [ ] Can describe the indexer architecture (chain node → block queue → processor → index store → API)
- [ ] Can process a block and extract transactions, events, state diffs
- [ ] Can handle re-orgs (rollback old fork, apply new fork)
- [ ] Can design an event indexer with raw + decoded event storage
- [ ] Can choose between PostgreSQL, RocksDB, or other storage for indexes
- [ ] Can set up a GraphQL query layer for chain data
- [ ] Can set up a REST API for common queries (tx by id, txs by address, block by height)
- [ ] Can implement selective indexing (only watch specific addresses/contracts/events)
- [ ] Can prune old indexes when needed
- [ ] Understands the difference between archive, full, and pruned nodes for indexing needs

## Common Pitfalls

1. **Not handling re-orgs.** Without re-org handling, the index diverges from the chain during reorganizations. Design for re-orgs from the start.

2. **Indexing without a re-org notification mechanism.** The chain node must notify the indexer of re-orgs, or the indexer must detect them (by checking the chain tip). Plan for this.

3. **Storing only decoded events without raw data.** If the decoding schema changes (e.g., a contract upgrades), old decoded data may be wrong. Store raw data too, so you can re-decode.

4. **Not indexing txs by txid.** Txid is the primary key for transactions. Index by txid for fast lookups.

5. **Not indexing blocks by height and hash.** Blocks are queryable by height (sequential) and by hash (random). Index both.

6. **Assuming the chain is always available.** The chain node may go down, or RPC may fail. Handle connection failures, queue blocks for later processing.

7. **Indexing everything without a pruning strategy.** Full indexes grow unbounded. Plan for pruning, compression, or selective indexing.

8. **Not testing re-org handling.** Re-orgs are rare but critical. Test with simulated re-orgs.

9. **Not separating the indexer from the chain node.** The indexer should be a separate process/service. Don't couple it to the chain node's internal state.

10. **Not rate-limiting API queries.** Public APIs can be abused. Rate-limit, paginate, and cache queries.

## Quick Reference

| Concern | Pattern |
|---|---|
| Block processing | Fetch block → parse → index → store |
| Re-org handling | Detect → rollback old fork → apply new fork |
| Event indexing | Extract events → decode → store raw + decoded |
| API layer | GraphQL (flexible) or REST (simple) |
| Storage | PostgreSQL (relational) + RocksDB (key-value) |
| Pruning | Delete old data, set earliest_height, compress cold data |
| Selective indexing | Watch specific addresses/contracts/events |
| Backpressure | Queue blocks, process at a rate the DB can handle |
