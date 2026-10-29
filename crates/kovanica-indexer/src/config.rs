//! Configuration via env + toml.

use clap::Parser;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(name = "kovanica-indexer")]
pub struct Cli {
    #[arg(long)]
    pub config: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default = "default_listen")]
    pub listen: String,
    #[serde(default)]
    pub node_rpc_url: Option<String>,
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    #[serde(default = "default_poll_ms")]
    pub poll_interval_ms: u64,
}

fn default_listen() -> String {
    "0.0.0.0:8090".into()
}
fn default_data_dir() -> String {
    "./data/indexer".into()
}
fn default_poll_ms() -> u64 {
    2000
}

impl Config {
    pub fn load(cli: &Cli) -> Self {
        let mut cfg = if let Some(path) = &cli.config {
            std::fs::read_to_string(path)
                .ok()
                .and_then(|s| toml::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Config::default()
        };
        if let Ok(v) = std::env::var("KOVANICA_INDEXER_LISTEN") {
            cfg.listen = v;
        }
        if let Ok(v) = std::env::var("KOVANICA_INDEXER_NODE_RPC") {
            cfg.node_rpc_url = Some(v);
        }
        if let Ok(v) = std::env::var("KOVANICA_INDEXER_DATA_DIR") {
            cfg.data_dir = v;
        }
        cfg
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:8090".parse().unwrap())
    }
}
