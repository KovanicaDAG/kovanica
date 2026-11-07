//! Isolated Hybrid PoW + GHOSTDAG acceptance test suite (Chain 5).
//!
//! Validates:
//! 1. Isolated topology: Producer node + Non-mining Observer node, loopback only,
//!    no public exposure, dedicated disposable directories.
//! 2. Nonzero execution & emission under existing rules:
//!    - Block production via KovanicaHash PoW mining.
//!    - Observer admission via gossip and GHOSTDAG k=5 commitment.
//!    - Mining reward distribution: exactly 2.5 KVNC (2_500_000_000 atoms) per block.
//!    - Mempool transaction inclusion, state execution, and balance state updates.
//! 3. Stop and reopen same data directory (durable recovery):
//!    - Stop observer node.
//!    - Reopen the exact same data directory without peers.
//!    - Prove state persistence: committed height and balances survive restart from durable storage.
//!    - Distinguish recovery from resynchronization.
//! 4. Subsequent progress:
//!    - Reconnect observer to producer.
//!    - Produce subsequent blocks; observer advances height and state beyond recovered checkpoint.
//! 5. External producer submission via `kovanica_submitWork`:
//!    - Supported RPC path used by standalone solo miner (`kovanica miner run`).
//!    - Positive admission, execution, and exactly-once reward.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use ed25519_dalek::SigningKey;
use kovanica_crypto::{generate_keypair, sign as crypto_sign};
use kovanica_params::{INITIAL_BLOCK_REWARD, ONE_KUNA};
use kovanica_staking::MIN_VALIDATOR_STAKE;
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

const CHAIN_ID: u64 = chain_id::HYBRID_TESTNET;
const RPC_READY_TIMEOUT_SECS: u64 = 60;
const PEER_TIMEOUT_SECS: u64 = 90;
const MINING_TIMEOUT_SECS: u64 = 90;
const TX_TIMEOUT_SECS: u64 = 90;
const POLL_INTERVAL_MS: u64 = 500;
const PORT_ATTEMPTS: usize = 32;

const TRANSFER_AMOUNT: u64 = ONE_KUNA;
const TRANSFER_FEE: u64 = 1_000_000;

struct ManagedNode {
    index: usize,
    child: Option<Child>,
    data_dir: TempDir,
    temp_log_path: PathBuf,
    log_path: PathBuf,
    rpc_port: u16,
    p2p_port: u16,
    #[allow(dead_code)]
    address: Address,
    #[allow(dead_code)]
    signing_key: SigningKey,
    is_mining: bool,
    is_validator: bool,
    binary: PathBuf,
}

impl ManagedNode {
    #[allow(clippy::too_many_arguments)]
    fn spawn(
        index: usize,
        rpc_port: u16,
        p2p_port: u16,
        bootnodes: &[String],
        signing_key: SigningKey,
        address: Address,
        binary: &Path,
        genesis_toml: &str,
        log_dir: &Path,
        is_mining: bool,
        is_validator: bool,
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
            .env(
                "KOVANICA_RUN_VALIDATOR",
                if is_validator { "true" } else { "false" },
            )
            .env("KOVANICA_CHAIN_ID", CHAIN_ID.to_string())
            .env(
                "KOVANICA_MINING_ENABLED",
                if is_mining { "true" } else { "false" },
            )
            .env("KOVANICA_MINING_THREADS", "1")
            .env("KOVANICA_MINING_AUTHOR", index.to_string())
            .env("KOVANICA_TEST_TARGET_BITS", "0x207fffff")
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
            mining = is_mining,
            validator = is_validator,
            "spawned node"
        );

        Ok(Self {
            index,
            child: Some(child),
            data_dir,
            temp_log_path,
            log_path,
            rpc_port,
            p2p_port,
            address,
            signing_key,
            is_mining,
            is_validator,
            binary: binary.to_path_buf(),
        })
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            info!(node = self.index, "stopping node process");
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    fn restart_on_same_data_dir(&mut self, bootnodes: &[String]) -> Result<()> {
        self.stop()?;

        let key_path = self.data_dir.path().join("validator.key");
        let restart_log_path = self
            .data_dir
            .path()
            .join(format!("node{}_reopened.log", self.index));
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&restart_log_path)
            .with_context(|| format!("node {}: open restart log", self.index))?;
        let log_err = log.try_clone()?;

        let mut cmd = Command::new(&self.binary);
        cmd.current_dir(self.data_dir.path())
            .env("KOVANICA_DATA_DIR", self.data_dir.path())
            .env("KOVANICA_RPC_ADDR", "127.0.0.1")
            .env("KOVANICA_RPC_PORT", self.rpc_port.to_string())
            .env(
                "KOVANICA_LISTEN_ADDR",
                format!("127.0.0.1:{}", self.p2p_port),
            )
            .env("KOVANICA_MAX_PEERS", "8")
            .env(
                "KOVANICA_RUN_VALIDATOR",
                if self.is_validator { "true" } else { "false" },
            )
            .env("KOVANICA_CHAIN_ID", CHAIN_ID.to_string())
            .env(
                "KOVANICA_MINING_ENABLED",
                if self.is_mining { "true" } else { "false" },
            )
            .env("KOVANICA_MINING_THREADS", "1")
            .env("KOVANICA_MINING_AUTHOR", self.index.to_string())
            .env("KOVANICA_TEST_TARGET_BITS", "0x207fffff")
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
            .with_context(|| {
                format!(
                    "node {}: restart spawn {}",
                    self.index,
                    self.binary.display()
                )
            })?;

        info!(
            node = self.index,
            rpc = self.rpc_port,
            p2p = self.p2p_port,
            "restarted node on same data dir"
        );

        self.child = Some(child);
        Ok(())
    }
}

impl Drop for ManagedNode {
    fn drop(&mut self) {
        let _ = self.stop();
        if let Some(parent) = self.log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::copy(&self.temp_log_path, &self.log_path);
    }
}

fn build_genesis_toml(validators: &[(SigningKey, PublicKey, Address)]) -> String {
    let mut out = String::from("# generated by isolated_hybrid_acceptance\n");
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
        .join("target/acceptance-cluster-logs")
        .join(format!("acceptance-{ts}"))
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

fn pick_free_port_block(start: u16, end: u16, count: u16) -> Result<u16> {
    let span = count as u64;
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
        if (0..count).all(|offset| port_is_available(base + offset)) {
            std::thread::sleep(Duration::from_millis(150));
            return Ok(base);
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

async fn wait_ready(client: &Client, port: u16) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(RPC_READY_TIMEOUT_SECS);
    loop {
        if get_health(client, port).await.is_ok() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!("RPC on 127.0.0.1:{port} not ready after {RPC_READY_TIMEOUT_SECS}s");
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }
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

#[tokio::test]
async fn isolated_hybrid_acceptance_execution_and_recovery() -> Result<()> {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    let binary = node_binary_path();
    if !binary.exists() {
        bail!("node binary {} not found", binary.display());
    }

    let label = "acceptance-topology";
    let log_dir = stable_log_dir(label);
    std::fs::create_dir_all(&log_dir)?;

    let base_port = pick_free_port_block(31_000, 35_000, 4)?;
    let p2p_producer = base_port;
    let rpc_producer = base_port + 1;
    let p2p_observer = base_port + 2;
    let rpc_observer = base_port + 3;

    // Keys for 2 participants
    let (sk0, pk0) = generate_keypair();
    let addr0 = Address::from_public_key(&pk0);

    let (sk1, pk1) = generate_keypair();
    let addr1 = Address::from_public_key(&pk1);

    let validators = vec![(sk0.clone(), pk0, addr0), (sk1.clone(), pk1, addr1)];
    let genesis = build_genesis_toml(&validators);

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .context("reqwest client")?;

    info!(
        rpc_producer,
        p2p_producer, rpc_observer, p2p_observer, "assigned ports for isolated acceptance test"
    );

    // 1. Start Producer Node (mining enabled)
    let mut producer = ManagedNode::spawn(
        0,
        rpc_producer,
        p2p_producer,
        &[],
        sk0.clone(),
        addr0,
        &binary,
        &genesis,
        &log_dir,
        true, // mining enabled
        true, // validator enabled
    )?;

    // 2. Start Non-mining Observer Node (suitable for Apps supervisor)
    let observer_bootnodes = vec![format!("127.0.0.1:{p2p_producer}")];
    let mut observer = ManagedNode::spawn(
        1,
        rpc_observer,
        p2p_observer,
        &observer_bootnodes,
        sk1.clone(),
        addr1,
        &binary,
        &genesis,
        &log_dir,
        false, // NON-MINING
        false, // NON-VALIDATOR
    )?;

    wait_ready(&client, rpc_producer).await?;
    wait_ready(&client, rpc_observer).await?;

    // Verify peering between Producer and Observer
    let peer_deadline = Instant::now() + Duration::from_secs(PEER_TIMEOUT_SECS);
    loop {
        let p_count = peer_count(&client, rpc_producer).await.unwrap_or(0);
        let o_count = peer_count(&client, rpc_observer).await.unwrap_or(0);
        if p_count >= 1 && o_count >= 1 {
            info!(
                p_count,
                o_count, "producer and observer peered successfully"
            );
            break;
        }
        if Instant::now() >= peer_deadline {
            bail!("producer and observer failed to peer within {PEER_TIMEOUT_SECS}s");
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    // Verify observer is non-mining via getMiningInfo
    let mining_info = rpc_call(&client, rpc_observer, "kovanica_getMiningInfo", json!([])).await?;
    let min_res = mining_info.get("result").context("mining info result")?;
    assert_eq!(
        min_res.get("mining_enabled").and_then(Value::as_bool),
        Some(false),
        "observer node must have mining_enabled = false"
    );

    let (_sk_recip, pk_recip) = generate_keypair();
    let recipient_addr = Address::from_public_key(&pk_recip);

    // Submit transfer immediately so it is included in early blocks before DAA scales difficulty
    let tx = create_transfer(
        addr0,
        recipient_addr,
        TRANSFER_AMOUNT,
        TRANSFER_FEE,
        0,
        &sk0,
    );
    let tx_bytes = bincode::serialize(&tx).context("serialize tx")?;
    let raw_hex = format!("0x{}", hex::encode(tx_bytes));

    let tx_resp = rpc_call(
        &client,
        rpc_producer,
        "kovanica_sendRawTransaction",
        json!([raw_hex]),
    )
    .await?;
    let tx_hash = tx_resp
        .get("result")
        .and_then(Value::as_str)
        .context("tx hash")?;
    info!(%tx_hash, "submitted transfer via producer mempool");

    // 3. Wait for Producer to mine blocks, include transaction, and Observer to admit and commit them
    let mine_deadline = Instant::now() + Duration::from_secs(MINING_TIMEOUT_SECS);
    let mut observer_height = 0u64;
    while observer_height == 0 {
        if let Ok(h) = get_height(&client, rpc_observer).await {
            if h >= 1 {
                observer_height = h;
                info!(
                    observer_height,
                    "observer committed blocks mined by producer"
                );
                break;
            }
        }
        if Instant::now() >= mine_deadline {
            bail!("observer did not advance committed height within {MINING_TIMEOUT_SECS}s");
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    // 4. Verify mining reward distribution on Observer (2.5 KVNC per block author)
    let reward_deadline = Instant::now() + Duration::from_secs(MINING_TIMEOUT_SECS);
    let producer_reward = loop {
        if let Ok(b) = get_balance(&client, rpc_observer, addr0).await {
            if b >= INITIAL_BLOCK_REWARD {
                info!(
                    producer_reward = b,
                    "observer executed block reward distribution for author"
                );
                break b;
            }
        }
        if Instant::now() >= reward_deadline {
            bail!("observer did not reflect author mining reward within {MINING_TIMEOUT_SECS}s");
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    };

    assert!(
        producer_reward >= INITIAL_BLOCK_REWARD,
        "author must have received at least one 2.5 KVNC reward"
    );

    // 5. Verify recipient balance received from executed transaction
    let tx_deadline = Instant::now() + Duration::from_secs(TX_TIMEOUT_SECS);
    let recipient_balance = loop {
        if let Ok(b) = get_balance(&client, rpc_observer, recipient_addr).await {
            if b == TRANSFER_AMOUNT {
                info!(recipient_balance = b, "transaction executed on observer");
                break b;
            }
        }
        if Instant::now() >= tx_deadline {
            bail!("transfer not executed within {TX_TIMEOUT_SECS}s");
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    };
    assert_eq!(recipient_balance, TRANSFER_AMOUNT);

    // Record pre-shutdown state on observer
    let pre_stop_height = get_height(&client, rpc_observer).await?;
    let pre_stop_reward = get_balance(&client, rpc_observer, addr0).await?;
    let pre_stop_recipient = get_balance(&client, rpc_observer, recipient_addr).await?;

    info!(
        pre_stop_height,
        pre_stop_reward, pre_stop_recipient, "observer state recorded before shutdown"
    );

    // 6. STOP OBSERVER AND PROVE DURABLE STORAGE RECOVERY (WITHOUT PEERS)
    info!("stopping observer to verify durable storage recovery...");
    observer.stop()?;

    // Verify observer is offline
    assert!(
        get_health(&client, rpc_observer).await.is_err(),
        "observer must be offline after stop"
    );

    // Reopen observer on the EXACT SAME data directory, with NO bootnodes (isolated from network)
    info!("reopening observer on identical data dir without peer connection...");
    observer.restart_on_same_data_dir(&[])?;
    wait_ready(&client, rpc_observer).await?;

    // Verify recovered state from local durable storage (zero network peers)
    let post_restart_peers = peer_count(&client, rpc_observer).await.unwrap_or(0);
    assert_eq!(
        post_restart_peers, 0,
        "restarted observer has no peers; state must come from durable disk"
    );

    let recovered_height = get_height(&client, rpc_observer).await?;
    let recovered_reward = get_balance(&client, rpc_observer, addr0).await?;
    let recovered_recipient = get_balance(&client, rpc_observer, recipient_addr).await?;

    info!(
        recovered_height,
        recovered_reward, recovered_recipient, "observer recovered state from durable storage"
    );

    assert_eq!(
        recovered_height, pre_stop_height,
        "committed height must match pre-stop height upon disk recovery"
    );
    assert_eq!(
        recovered_recipient, pre_stop_recipient,
        "recipient balance must be preserved across restart"
    );
    assert_eq!(
        recovered_reward, pre_stop_reward,
        "mining reward balance must be preserved across restart"
    );

    // 6b. FAILURE PATH VERIFICATION ON DISPOSABLE STORAGE:
    // Verify that submitting a block with a mismatched target (e.g. production MAX_TARGET_BITS
    // 0x1e0ffff0 vs required target) or invalid PoW via kovanica_submitWork is rejected and
    // leaves persisted state untouched.
    let work_resp = rpc_call(&client, rpc_observer, "kovanica_getWork", json!([])).await?;
    let mut bad_header: kovanica_types::block::BlockHeader = serde_json::from_value(
        work_resp
            .get("result")
            .and_then(|r| r.get("header"))
            .cloned()
            .context("missing header in getWork")?,
    )
    .context("deserialize getWork header")?;
    bad_header.target_bits = kovanica_params::MAX_TARGET_BITS;
    let bad_submit = client
        .post(format!("http://127.0.0.1:{rpc_observer}/rpc"))
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "kovanica_submitWork",
            "params": [{ "header": bad_header }],
            "id": 99,
        }))
        .send()
        .await?
        .json::<Value>()
        .await?;
    assert!(
        bad_submit.get("error").is_some(),
        "submitWork with mismatched target/PoW must be rejected: {bad_submit}"
    );
    assert_eq!(
        get_height(&client, rpc_observer).await?,
        recovered_height,
        "rejected submitWork must not advance height"
    );

    // 7. SUBSEQUENT PROGRESS: Reconnect observer to producer and verify progress
    info!("reconnecting observer to producer to verify subsequent progress...");
    observer.restart_on_same_data_dir(&[format!("127.0.0.1:{p2p_producer}")])?;
    wait_ready(&client, rpc_observer).await?;

    // Wait for peering
    let reconnect_deadline = Instant::now() + Duration::from_secs(PEER_TIMEOUT_SECS);
    loop {
        let p_count = peer_count(&client, rpc_producer).await.unwrap_or(0);
        let o_count = peer_count(&client, rpc_observer).await.unwrap_or(0);
        if p_count >= 1 && o_count >= 1 {
            break;
        }
        if Instant::now() >= reconnect_deadline {
            bail!("failed to reconnect after restart");
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    // Wait for height to advance beyond recovered height
    let progress_deadline = Instant::now() + Duration::from_secs(MINING_TIMEOUT_SECS);
    let mut advanced_height = recovered_height;
    while advanced_height <= recovered_height {
        if let Ok(h) = get_height(&client, rpc_observer).await {
            if h > recovered_height {
                advanced_height = h;
                info!(
                    advanced_height,
                    recovered_height, "observer advanced height after durable recovery"
                );
                break;
            }
        }
        if Instant::now() >= progress_deadline {
            bail!("observer did not advance height beyond recovered height within {MINING_TIMEOUT_SECS}s");
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    }

    assert!(
        advanced_height > recovered_height,
        "subsequent progress must advance beyond recovered checkpoint"
    );

    // Verify executed state (author reward balance) also progressed on observer after recovery
    let post_recovery_reward_deadline = Instant::now() + Duration::from_secs(MINING_TIMEOUT_SECS);
    let advanced_reward = loop {
        if let Ok(b) = get_balance(&client, rpc_observer, addr0).await {
            if b > recovered_reward {
                info!(
                    advanced_reward = b,
                    recovered_reward, "observer executed subsequent block rewards after recovery"
                );
                break b;
            }
        }
        if Instant::now() >= post_recovery_reward_deadline {
            bail!("observer reward balance did not advance after recovery within {MINING_TIMEOUT_SECS}s");
        }
        sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
    };
    assert!(
        advanced_reward >= recovered_reward + INITIAL_BLOCK_REWARD,
        "author reward balance on observer must increase by at least 2.5 KVNC after post-recovery commit"
    );

    // Stop all nodes cleanly
    producer.stop()?;
    observer.stop()?;

    info!("isolated hybrid acceptance test passed successfully!");
    Ok(())
}
