use kovanica_devnet::node_config;
use std::env;
use std::fs;

fn main() {
    let nodes: usize = env::var("NODES").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
    let out = env::var("OUT").unwrap_or_else(|_| "./devnet".to_string());
    fs::create_dir_all(&out).unwrap();
    for i in 0..nodes {
        let cfg = node_config(i, nodes, 8545, 8000);
        fs::write(format!("{}/node{}.toml", out, i), cfg).unwrap();
    }
    println!("generated {} configs in {}", nodes, out);
}
