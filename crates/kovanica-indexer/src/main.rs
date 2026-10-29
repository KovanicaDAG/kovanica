use clap::Parser;
use kovanica_indexer::config::{Cli, Config};
use kovanica_indexer::http;
use kovanica_indexer::indexer::Indexer;
use kovanica_indexer::node_reader::StorageNodeReader;
use kovanica_indexer::store::IndexerStore;
use kovanica_storage::Storage;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let cfg = Config::load(&cli);

    std::fs::create_dir_all(&cfg.data_dir)?;
    let store = Arc::new(IndexerStore::new(format!("{}/indexer.redb", cfg.data_dir))?);

    // Optional storage-backed reader if data dir has node storage? For MVP, try open storage at data_dir/node
    let node_storage_path = format!("{}/node", cfg.data_dir);
    let node: Arc<dyn kovanica_indexer::node_reader::NodeReader> =
        if std::path::Path::new(&node_storage_path).exists() {
            let storage = Arc::new(Storage::new(&node_storage_path)?);
            Arc::new(StorageNodeReader { storage })
        } else {
            // No node storage; create an empty in-memory-ish storage for demo
            let storage = Arc::new(Storage::new(format!("{}/node", cfg.data_dir))?);
            Arc::new(StorageNodeReader { storage })
        };

    let indexer = Arc::new(Indexer::new(store, node));

    // sync loop
    let idx_clone = indexer.clone();
    let poll = cfg.poll_interval_ms;
    tokio::spawn(async move {
        loop {
            let _ = idx_clone.sync_once().await;
            tokio::time::sleep(std::time::Duration::from_millis(poll)).await;
        }
    });

    let app = http::router(indexer);
    let addr = cfg.listen_addr();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("kovanica-indexer listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
