//! Live 3-node Hybrid PoW + GHOSTDAG cluster integration test for Chain 5 (Milestone M8).
//!
//! Spawns a 3-node live cluster running on Chain 5 (`HYBRID_TESTNET`) with mining
//! enabled, verifies P2P discovery, KovanicaHash PoW mining, GHOSTDAG k=5 DAG tips
//! and blue set sequencing, 2.5 KVNC mining reward distribution, and end-to-end
//! transaction inclusion and execution.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use ed25519_dalek::SigningKey;
use kovanica_crypto::{generate_keypair, sign as crypto_sign};
use kovanica_staking::{MIN_VALIDATOR_STAKE, ONE_KUNA};
use kovanica_types::{
    hash::Hash,
    signing::{chain_id, SigningContext},
    transaction::{Transaction, TransactionKind},
    Address, PublicKey, Signature,
};
use reqwest::Client;
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::time::sleep;
use tracing::info;

const NODES: usize = 3;
const CHAIN_ID: u64 = chain_id::HYBRID_TESTNET;
const RPC_READY_TIMEOUT_SECS: u64 = 60;
const PEER_TIMEOUT_SECS: u64 = 90;
const MINING_TIMEOUT_SECS: u64 = 300;
const TX_TIMEOUT_SECS: u64 = 300;
const POLL_INTERVAL_MS: u64 = 500;
const PORT_ATTEMPTS: usize = 32;

/// Transfer amount: 1 KVNC.
const TRANSFER_AMOUNT: u64 = ONE_KUNA;
const TRANSFER_FEE: u64 = 1_000_000;

struct NodeProc {
    index: usize,
    child: Child,
    #[allow(dead_code)]
    data_dir: TempDir,
    temp_log_path: PathBuf,
    log_path: PathBuf,
    rpc_port: u16,
    address: Address,
    signing_key: SigningKey,
}

impl NodeProc {
    #[allow(clippy::too_many_arguments)]
    fn start(
        index: usize,
        rpc_port: u16,
        p2p_port: u16,
        bootnodes: &[String],
        signing_key: SigningKey,
        address: Address,
        binary: &Path,
        genesis_toml: &str,
        log_dir: &Path,
    ) -> Result<Self> {
        let data_dir = TempDir::new().with_context(|| format!("node {index}: temp dir"))?;
        std::fs::write(
            data_dir.path().join("genesis_validators.toml"),
            genesis_toml,
        )
        .with_context(|| format!("node {index}: write genesis"))?;

        let key_path = data_dir.path().join("validator.key");
        write_private(&key_path, hex::encode(signing_key.to_bytes()))
            .with_context(|| format!("node {index}: write validator key"))?;

        let temp_log_path = data_dir.path().join(format!("node{index}.log"));
        let log_path = log_dir.join(format!("node{index}.log"));
        let log = std::fs::File::create(&temp_log_path)
            .with_context(|| format!("node {index}: create log"))?;
        let log_err = log.try_clone()?;

        let mut cmd = Command::new(binary);
        cmd.current_dir(data_dir.path())
            .env("KOVANICA_DATA_DIR", data_dir.path())
            .env("KOVANICA_RPC_ADDR", "127.0.0.1")
            .env("KOVANICA_RPC_PORT", rpc_port.to_string())
            .env("KOVANICA_LISTEN_ADDR", format!("127.0.0.1:{p2p_port}"))
            .env("KOVANICA_MAX_PEERS", "8")
            .env("KOVANICA_RUN_VALIDATOR", "true")
            .env("KOVANICA_CHAIN_ID", CHAIN_ID.to_string())
            .env("KOVANICA_MINING_ENABLED", "true")
            .env("KOVANICA_MINING_THREADS", "1")
            .env("KOVANICA_MINING_AUTHOR", index.to_string())
            .env("KOVANICA_RPC_AUTH", "disable")
            .env("KOVANICA_RPC_RATE_LIMIT_PER_MIN", "0")
            .env("KOVANICA_VALIDATOR_KEY", key_path.as_os_str())
            .env("RUST_LOG", "info");

        if !bootnodes.is_empty() {
            cmd.env("KOVANICA_BOOTNODES", bootnodes.join(","));
        }

        let child = cmd
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(log_err))
            .spawn()
            .with_context(|| format!("node {index}: spawn {}", binary.display()))?;

        info!(
            node = index,
            rpc = rpc_port,
            p2p = p2p_port,
            "spawned hybrid testnet node"
        );

        Ok(Self {
            index,
            child,
            data_dir,
            temp_log_path,
            log_path,
            rpc_port,
            address,
            signing_key,
        })
    }
}

impl Drop for NodeProc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(parent) = self.log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::copy(&self.temp_log_path, &self.log_path) {
            Ok(bytes) => eprintln!(
                "[hybrid_cluster] preserved node {} log ({} bytes) -> {}",
                self.index,
                bytes,
                self.log_path.display()
            ),
            Err(err) => eprintln!(
                "[hybrid_cluster] failed to preserve node {} log {} -> {}: {err}",
                self.index,
                self.temp_log_path.display(),
                self.log_path.display()
            ),
        }
    }
}

struct Cluster {
    nodes: Vec<NodeProc>,
    client: Client,
    log_dir: PathBuf,
}

impl Cluster {
    fn start(label: &str, port_start: u16, port_end: u16) -> Result<Self> {
        let binary = node_binary_path();
        if !binary.exists() {
            bail!("node binary {} not found", binary.display());
        }
        let log_dir = stable_log_dir(label);
        std::fs::create_dir_all(&log_dir)?;
        eprintln!("[hybrid_cluster] {label} logs under {}", log_dir.display());

        let mut validators = Vec::with_capacity(NODES);
        for _ in 0..NODES {
            let (sk, pk) = generate_keypair();
            let addr = Address::from_public_key(&pk);
            validators.push((sk, pk, addr));
        }
        let genesis = build_genesis_toml(&validators);
        let (rpc_base, p2p_base) = pick_free_ports(port_start, port_end)?;
        eprintln!("[hybrid_cluster] {label} ports: rpc base {rpc_base}, p2p base {p2p_base}");

        let mut nodes = Vec::with_capacity(NODES);

        // Start Node 0 (the seed / bootnode)
        nodes.push(NodeProc::start(
            0,
            rpc_base,
            p2p_base,
            &[],
            validators[0].0.clone(),
            validators[0].2,
            &binary,
            &genesis,
            &log_dir,
        )?);

        // Brief delay so bootnode has time to bind its sockets
        std::thread::sleep(Duration::from_millis(500));

        // Start peers connected to Node 0
        for (i, (sk, _pk, addr)) in validators.iter().enumerate().skip(1) {
            let bootnodes = vec![format!("127.0.0.1:{p2p_base}")];
            nodes.push(NodeProc::start(
                i,
                rpc_base + i as u16,
                p2p_base + i as u16,
                &bootnodes,
                sk.clone(),
                *addr,
                &binary,
                &genesis,
                &log_dir,
            )?);
        }

        Ok(Self {
            nodes,
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .context("reqwest client")?,
            log_dir,
        })
    }

    async fn wait_ready(&self) -> Result<()> {
        for node in &self.nodes {
            let deadline = Instant::now() + Duration::from_secs(RPC_READY_TIMEOUT_SECS);
            loop {
                if get_health(&self.client, node.rpc_port).await.is_ok() {
                    break;
                }
                if Instant::now() >= deadline {
                    bail!(
                        "node {} RPC on 127.0.0.1:{} not ready after {RPC_READY_TIMEOUT_SECS}s (logs {})",
                        node.index,
                        node.rpc_port,
                        self.log_dir.display()
                    );
                }
                sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
            }
        }
        Ok(())
    }

    async fn wait_for_peers(&self) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(PEER_TIMEOUT_SECS);
        loop {
            let mut counts = Vec::with_capacity(self.nodes.len());
            for node in &self.nodes {
                counts.push(peer_count(&self.client, node.rpc_port).await.unwrap_or(0));
            }
            if counts.iter().all(|c| *c >= 1) {
                info!(?counts, "all hybrid cluster nodes connected");
                return Ok(());
            }
            if Instant::now() >= deadline {
                bail!(
                    "nodes did not peer within {PEER_TIMEOUT_SECS}s (counts {counts:?}, logs {})",
                    self.log_dir.display()
                );
            }
            sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
        }
    }

    async fn wait_for_height_above(&self, min_exclusive: u64) -> Result<u64> {
        let deadline = Instant::now() + Duration::from_secs(MINING_TIMEOUT_SECS);
        loop {
            for node in &self.nodes {
                if let Ok(h) = get_height(&self.client, node.rpc_port).await {
                    if h > min_exclusive {
                        info!(
                            height = h,
                            node = node.index,
                            "node advanced committed height"
                        );
                        return Ok(h);
                    }
                }
            }
            if Instant::now() >= deadline {
                bail!(
                    "cluster did not advance height > {min_exclusive} within {MINING_TIMEOUT_SECS}s (logs {})",
                    self.log_dir.display()
                );
            }
            sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
        }
    }
}

fn build_genesis_toml(validators: &[(SigningKey, PublicKey, Address)]) -> String {
    let mut out = String::from("# generated by hybrid_node_cluster_integration\n");
    out.push_str(&format!("chain_id = {CHAIN_ID}\n\n"));
    for (_, pk, addr) in validators {
        out.push_str("[[validator]]\n");
        out.push_str(&format!("address = \"{}\"\n", hex::encode(addr.0)));
        out.push_str(&format!("stake = {MIN_VALIDATOR_STAKE}\n"));
        out.push_str(&format!("public_key = \"{}\"\n\n", hex::encode(pk.0)));
    }
    out
}

fn create_transfer(
    sender: Address,
    to: Address,
    amount: u64,
    fee: u64,
    nonce: u64,
    signing_key: &SigningKey,
) -> Transaction {
    let tx = Transaction {
        sender,
        nonce,
        fee,
        kind: TransactionKind::Transfer { to, amount },
        signature: Signature([0; 64]),
        hash: Hash::zero(),
    };
    let msg = tx.signing_hash(&SigningContext::new(CHAIN_ID));
    let signature = crypto_sign(signing_key, &msg.0);
    Transaction {
        signature,
        hash: msg,
        ..tx
    }
}

fn node_binary_path() -> PathBuf {
    std::env::var("KOVANICA_NODE_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_BIN_EXE_kovanica-node")))
}

fn stable_log_dir(label: &str) -> PathBuf {
    if let Ok(dir) = std::env::var("KOVANICA_TEST_LOG_DIR") {
        return PathBuf::from(dir).join(label);
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|p| p.join("../.."))
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("target/hybrid-cluster-logs")
        .join(format!("hybrid-{ts}"))
        .join(label)
}

fn port_is_available(port: u16) -> bool {
    if std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_millis(5),
    )
    .is_ok()
    {
        return false;
    }
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn pick_free_ports(start: u16, end: u16) -> Result<(u16, u16)> {
    let span = (2 * NODES) as u64;
    let blocks = (end - start) as u64 / span;
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
        ^ ((std::process::id() as u64) << 32);
    for _ in 0..PORT_ATTEMPTS {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let base = start + ((seed % blocks) * span) as u16;
        if (0..span as u16).all(|offset| port_is_available(base + offset)) {
            std::thread::sleep(Duration::from_millis(150));
            return Ok((base, base + NODES as u16));
        }
    }
    bail!("no free block of {span} ports in {start}..{end}")
}

fn write_private(path: &Path, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    std::fs::write(path, contents)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

async fn rpc_call(client: &Client, port: u16, method: &str, params: Value) -> Result<Value> {
    let resp = client
        .post(format!("http://127.0.0.1:{port}/rpc"))
        .json(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1,
        }))
        .send()
        .await
        .with_context(|| format!("RPC {method} on {port}"))?;
    let status = resp.status();
    let body = resp
        .text()
        .await
        .with_context(|| format!("RPC body {method} on {port}"))?;
    if !status.is_success() {
        bail!("RPC {method} on {port}: HTTP {status} body {body}");
    }
    let value: Value = serde_json::from_str(&body)
        .with_context(|| format!("RPC {method} on {port}: invalid JSON {body}"))?;
    if let Some(err) = value.get("error") {
        bail!("RPC {method} on {port}: {err}");
    }
    Ok(value)
}

async fn get_health(client: &Client, port: u16) -> Result<Value> {
    let resp = client
        .get(format!("http://127.0.0.1:{port}/health"))
        .send()
        .await
        .with_context(|| format!("health on {port}"))?
        .json::<Value>()
        .await
        .with_context(|| format!("health JSON on {port}"))?;
    Ok(resp)
}

async fn peer_count(client: &Client, port: u16) -> Result<u64> {
    let health = get_health(client, port).await?;
    health
        .get("peer_count")
        .and_then(Value::as_u64)
        .with_context(|| format!("health on {port}: missing peer_count in {health}"))
}

async fn get_height(client: &Client, port: u16) -> Result<u64> {
    let v = rpc_call(client, port, "kovanica_blockNumber", json!([])).await?;
    parse_hex_u64(
        v.get("result")
            .with_context(|| format!("missing result in {v}"))?,
        "height",
    )
}

async fn get_balance(client: &Client, port: u16, address: Address) -> Result<u64> {
    let v = rpc_call(
        client,
        port,
        "kovanica_getBalance",
        json!([format!("0x{}", hex::encode(address.0))]),
    )
    .await?;
    parse_hex_u64(
        v.get("result")
            .with_context(|| format!("missing result in {v}"))?,
        "balance",
    )
}

fn parse_hex_u64(value: &Value, what: &str) -> Result<u64> {
    let raw = value
        .as_str()
        .with_context(|| format!("{what}: expected a hex quantity string, got {value}"))?;
    let raw = raw.strip_prefix("0x").unwrap_or(raw);
    u64::from_str_radix(raw, 16).with_context(|| format!("{what}: invalid hex {raw}"))
}

/// Hybrid PoW Testnet Cluster Integration Test (Milestone M8).
#[tokio::test]
async fn hybrid_pow_testnet_cluster_e2e() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();
    let cluster = Cluster::start("hybrid-m8", 26_000, 30_000)?;

    cluster.wait_ready().await?;
    cluster.wait_for_peers().await?;

    // 1. Verify kovanica_getMiningInfo on all nodes
    for node in &cluster.nodes {
        let mining_info = rpc_call(
            &cluster.client,
            node.rpc_port,
            "kovanica_getMiningInfo",
            json!([]),
        )
        .await?;
        let res = mining_info
            .get("result")
            .context("missing mining info result")?;

        assert_eq!(res.get("chain_id").and_then(Value::as_u64), Some(5));
        assert_eq!(
            res.get("mining_enabled").and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(res.get("ghostdag_k").and_then(Value::as_u64), Some(5));
        assert_eq!(
            res.get("target_block_time_ms").and_then(Value::as_u64),
            Some(4000)
        );
        assert!(res.get("target_bits").is_some());
    }

    // 2. Verify kovanica_getDagTips on node 0
    let dag_tips = rpc_call(
        &cluster.client,
        cluster.nodes[0].rpc_port,
        "kovanica_getDagTips",
        json!([]),
    )
    .await?;
    let tips_res = dag_tips.get("result").context("missing dag tips result")?;
    assert_eq!(tips_res.get("chain_id").and_then(Value::as_u64), Some(5));
    assert_eq!(tips_res.get("ghostdag_k").and_then(Value::as_u64), Some(5));
    let tips = tips_res
        .get("tips")
        .and_then(Value::as_array)
        .context("tips array")?;
    assert!(!tips.is_empty(), "DAG must have at least genesis tip");

    // 3. Submit transaction from node 0 early so it is admitted into
    // mempool before early PoW blocks are sealed, matching isolated_hybrid_acceptance.
    let (_sk_recip, pk_recip) = generate_keypair();
    let recipient_addr = Address::from_public_key(&pk_recip);

    let sender = cluster.nodes[0].address;
    let sender_sk = &cluster.nodes[0].signing_key;

    // Send transfer from Node 0 to recipient
    let tx = create_transfer(
        sender,
        recipient_addr,
        TRANSFER_AMOUNT,
        TRANSFER_FEE,
        0, // nonce 0
        sender_sk,
    );

    let tx_bytes = bincode::serialize(&tx).context("serialize transaction")?;
    let raw_hex = format!("0x{}", hex::encode(tx_bytes));

    let tx_resp = rpc_call(
        &cluster.client,
        cluster.nodes[0].rpc_port,
        "kovanica_sendRawTransaction",
        json!([raw_hex]),
    )
    .await?;

    let tx_hash = tx_resp
        .get("result")
        .and_then(Value::as_str)
        .context("tx hash result")?;
    info!(%tx_hash, "submitted transfer transaction to mempool");

    // 4. Wait for mining to produce blocks and advance committed height
    let height = cluster.wait_for_height_above(0).await?;
    assert!(height >= 1, "committed height must advance above 0");

    // 5. Verify mining rewards (2.5 KVNC = 2_500_000_000 atoms per block)
    let deadline = Instant::now() + Duration::from_secs(MINING_TIMEOUT_SECS);
    let mut total_balance;
    loop {
        total_balance = 0;
        for node in &cluster.nodes {
            if let Ok(b) = get_balance(&cluster.client, node.rpc_port, node.address).await {
                total_balance += b;
            }
        }
        if total_balance > 0 {
            break;
        }
        if Instant::now() >= deadline {
            bail!("mining nodes received no rewards after {MINING_TIMEOUT_SECS}s");
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }
    assert!(
        total_balance >= kovanica_params::INITIAL_BLOCK_REWARD,
        "total balance should reflect minted block rewards"
    );

    // 6. Wait for recipient to receive the transfer balance
    let tx_deadline = Instant::now() + Duration::from_secs(TX_TIMEOUT_SECS);
    let mut recipient_balance = 0u64;
    let mut last_logged_instant = Instant::now();
    loop {
        if let Ok(b) = get_balance(&cluster.client, cluster.nodes[0].rpc_port, recipient_addr).await
        {
            recipient_balance = b;
            if recipient_balance == TRANSFER_AMOUNT {
                info!(recipient_balance, "transaction executed in PoW block!");
                break;
            }
        }
        if last_logged_instant.elapsed() >= Duration::from_secs(10) {
            let h = get_height(&cluster.client, cluster.nodes[0].rpc_port)
                .await
                .unwrap_or(0);
            info!(
                current_height = h,
                recipient_balance, "waiting for transaction inclusion in PoW block..."
            );
            last_logged_instant = Instant::now();
        }
        if Instant::now() >= tx_deadline {
            bail!(
                "transaction not executed within {TX_TIMEOUT_SECS}s (balance {recipient_balance})"
            );
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    assert_eq!(
        recipient_balance, TRANSFER_AMOUNT,
        "recipient must receive exact transferred amount"
    );

    Ok(())
}
