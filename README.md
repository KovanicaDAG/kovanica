# Kovanica (kvnc)

> A Rust-native DAG-based Layer-1 blockchain implementing Mysticeti-style uncertified consensus, account-based state with staking, and deterministic Wasmi contract execution.

**Status: pre-alpha.**  
Not audited. Not for production use.  
Interfaces, consensus parameters, and tokenomics constants may change without notice.

The chain is **Kovanica**. The native coin ticker is **KUNA** (9 decimals). All crates, binaries, environment variables, and protocol prefixes use the `kvnc` namespace.

---

## Overview

`kvnc` is a Cargo workspace that implements a complete full-node stack:

- **Consensus**: Mysticeti-style uncertified DAG (wave length = 3) with direct/indirect commit rules, linearization, and an optional MysticGhost hybrid that applies GHOSTDAG \(k=3\) scoped colouring. Non-blue (red) blocks are returned to the mempool rather than pruned.
- **State model**: Account-based (UTXO-free) with balances, nonces, contract code/storage, and a sorted key-value Merkle state root. Snapshots and pruning are supported.
- **Staking & emission**: Proof-of-stake validators (target 15–21 active), delegation, unbonding queue, epoch rotation, fixed-percentage slashing, and a geometric emission schedule with treasury vesting.
- **Execution**: Native transactions (transfer, stake, delegate, claim rewards, contract deploy/call) plus a Wasmi-based deterministic WASM runtime with gas metering and memory limits.
- **Networking**: libp2p (TCP + Noise + Yamux, Gossipsub, Kademlia, Identify, Ping). Persistent peer identity, peer scoring, block/state sync, and orphan buffering.
- **Interfaces**: JSON-RPC + WebSocket subscriptions, a full-featured CLI wallet, OpenAPI specification, and a TypeScript API client.

The node is designed to run comfortably on modest hardware (target 2–4 GB RAM with aggressive DAG pruning).

## Features

- **DAG ledger** with causal ordering, stake-filtered parent selection, and bounded parent round gap.
- **Wave-based consensus** (leader / vote / decide) with quorum (2f+1) direct commits and path-based indirect commits.
- **Stake-weighted leader selection** with optional ECVRF-ED25519-SHA512-Elligator2 VRF support and deterministic fallback.
- **Account + staking model** including commission, reward sharing, unbonding, and treasury vesting.
- **Deterministic contract execution** via Wasmi (fuel metering, memory caps, module cache) plus native typed contract APIs (HTLC, Vault, Multisig, Token).
- **Verifiable state** with sorted-KV Merkle roots and snapshot/export support.
- **P2P networking** with gossip for blocks, transactions, and votes; request/response sync; and orphan handling.
- **JSON-RPC & WebSocket** covering chain, account, staking, mempool, consensus, and contract methods, plus subscriptions (newHeads, logs, pending transactions).
- **CLI** for key management (encrypted keystore v2 with Argon2id + XChaCha20-Poly1305), transfers, staking operations, and contract interactions.
- **Observability**: structured tracing (with per-round trace IDs), Prometheus metrics, and example Grafana dashboards.
- **Docker & Compose**: multi-stage Dockerfile and a 4-node local quorum setup.

## Tokenomics

Canonical constants (locked in `kvnc-staking` and documented in `docs/TOKENOMICS.md`):

| Parameter | Value |
|-----------|-------|
| Maximum supply | 90 200 000 KUNA |
| Decimals | 9 (1 KUNA = 1 000 000 000 atoms) |
| Founder premine | 200 000 KUNA |
| Genesis validator allocation | 1 000 000 KUNA (split evenly across genesis validators; remainder distributed deterministically) |
| Treasury | 8 000 000 KUNA (1 M per year × 8 years, linear vesting by committed-leader height) |
| Mining subsidy budget | 81 000 000 KUNA |
| Initial block reward | 2.5 KUNA (paid to the author of a committed leader block) |
| Subsidy era length | 8 100 000 committed leaders |
| Reward decay | × 3/4 per era |
| Minimum validator stake | 50 000 KUNA |
| Active validator range | 15–21 |
| Unbonding period | 100 000 rounds |
| Epoch length | 7 200 committed leaders (≈ 1 day at the design interval) |
| Design round / leader interval | configurable (`round_duration_ms`); tokenomics comments assume 4 s rounds → 12 s per committed leader |

Rewards, treasury vesting, and circulating-supply accounting are applied atomically inside the execution of each committed sub-DAG.

## Repository Layout

| Path | Purpose |
|------|---------|
| `crates/` | All Rust crates (see below) |
| `crates/kvnc-node` | Full-node binary (`kvnc-node`) |
| `crates/kvnc-cli` | CLI wallet and tooling (`kvnc-cli`) |
| `docs/` | Architecture, tokenomics, contracts, signature format, ADRs, and operational notes |
| `ops/` | Docker, Grafana, Prometheus, soak scripts |
| `packages/` | TypeScript API client and web/explorer assets |
| `brand-assets/` | Logos |
| `config.example.toml` | Example node configuration |
| `Dockerfile` / `docker-compose.yml` | Container build and local multi-node setup |
| `TASKLIST.md` | Phased implementation status (code-verified) |
| `.github/workflows/` | CI (fmt, clippy, test, audit, MSRV 1.90, docs) |

### Core Crates

| Crate | Responsibility |
|-------|----------------|
| `kvnc-types` | Core types (blocks, transactions, addresses `kvnc…dag`, hashes, votes, committee, signing context) |
| `kvnc-crypto` | Ed25519, batch verification, VRF, keystore primitives |
| `kvnc-storage` | redb-backed state, consensus, and account stores; Merkle roots; snapshots |
| `kvnc-dag` | DAG store, block manager, mergesets, pruning |
| `kvnc-consensus` | Committer, linearizer, engine, MysticGhost / GHOSTDAG colouring, metrics |
| `kvnc-mempool` | Fee-rate priority pool, admission, eviction, rebroadcast |
| `kvnc-network` | libp2p service, protocols, scoring, identity |
| `kvnc-runtime` | Wasmi WASM engine and gas metering |
| `kvnc-execution` | State-transition function, reward distribution, contract host |
| `kvnc-staking` | Emission schedule, treasury, validator set, delegation, slashing |
| `kvnc-rpc` | JSON-RPC server, WebSocket subscriptions, rate limiting |
| `kvnc-common` | Shared events, WASM helpers, memory utilities |
| `kvnc-htlc` / `kvnc-vault` / `kvnc-multisig` / `kvnc-token` | Native + WASM contract implementations |
| `kvnc-faucet` | Rate-limited testnet faucet service |
| `kvnc-cli` | Command-line wallet, staking, and contract tools |

## Quick Start

### Requirements

- Rust stable (MSRV 1.90) and Cargo
- (Optional) Docker and Docker Compose for the multi-node devnet
- (Optional) `wasm32-unknown-unknown` target for full contract test coverage

### Build and Test

```bash
git clone https://github.com/KovanicaDAG/kvnc.git
cd kvnc
cargo build --workspace
cargo test --workspace
```

CI runs `cargo fmt --check`, `clippy -D warnings`, full tests (including integration), `cargo audit`, documentation builds, and an MSRV check.

### Run a Node

```bash
cp config.example.toml config.toml
# Edit config.toml or override via environment variables (KVNC_DATA_DIR, KVNC_LISTEN_ADDR, KVNC_RPC_PORT, KVNC_BOOTNODES, KVNC_VALIDATOR_KEY, …)
cargo run -p kvnc-node -- --config config.toml
```

Key configuration points (see `config.example.toml` for the full list and environment-variable mappings):

- `data_dir` — chain data (separate consensus and state redb files)
- `listen_addr` — P2P listen address (default `0.0.0.0:8000`)
- `rpc_addr` / `rpc_port` — JSON-RPC bind (default `127.0.0.1:8545`)
- `bootnodes` — list of multiaddrs or DNS seeds (no hard-coded hosts; supply 3+ kvnc-native seeds)
- `round_duration_ms` — target consensus round duration
- `validator_key` — path to a 32-byte hex seed file (mode `0600`). Required when `run_validator` is true, which is the default. A read-only node may omit it.
- `rpc_rate_limit_per_min` — per-IP token-bucket limit (burst derived as `max(1, rate/6)`)

A `genesis` subcommand is available to produce a genesis JSON and optional validator key files from a validators list, treasury address, and founder address.

### Docker

```bash
docker build -t kvnc-node:local .
# See ops/docker/ and docker-compose.yml for multi-node and entrypoint details.
```

The image runs as a non-root user, exposes ports 8000 (P2P) and 8545 (RPC), and includes a healthcheck against `/health`.

### CLI Highlights

```bash
cargo run -p kvnc-cli -- --help
# wallet: keygen, import, export, address, sign, tui
# chain: balance, status, transfer
# staking: stake, unstake, delegate, claim-rewards
# contracts: HTLC, Vault, Multisig, Token subcommands
```

Keys are stored in an encrypted keystore (Argon2id + XChaCha20-Poly1305). The node never receives private seeds. `kvnc tui` opens an interactive wallet for balances, transfers, and staking on that same local keystore.

## Architecture (High Level)

```
Network (libp2p)
    │
    ▼
Mempool ──────────────────────────────┐
    │                                 │
    ▼                                 │
Block Producer (per validator)        │
    │                                 │
    ▼                                 │
Consensus (Mysticeti DAG + optional   │
           MysticGhost colouring)     │
    │ produces CommittedSubDag        │
    ▼                                 │
Execution Layer ◄──────────────────────┘
    │  native txs + WASM contracts
    │  leader reward + treasury vesting
    ▼
State (accounts, staking, contracts)
    │
    ▼
Storage (redb, pruning, snapshots)
```

See `docs/ARCHITECTURE.md`, `docs/TOKENOMICS.md`, `docs/CONTRACTS.md`, and `docs/SIGNATURE_FORMAT.md` for deeper detail. Signature domains use the `KUNA/…` tags; addresses follow the `kvnc<hex>dag` format with a Blake3 checksum.

## Development Status

Most foundation phases (types, crypto, storage, DAG, consensus core, execution, staking, node wiring, RPC, CLI wallet) are implemented and covered by unit and multi-node integration tests. Multi-node quorum, late-join, and sustained-liveness scenarios pass.

Remaining work includes longer soak tests, light-client endpoints, full governance hooks, production seed DNS registration, and external audit. Consult `TASKLIST.md` for the authoritative, code-verified phase status.

## Security Notes

- Pre-alpha software. Do not use with real value.
- Not audited.
- Consensus currently rejects same-round / same-author forks (first-valid wins); lexicographic min-digest selection is a known future improvement.
- Client keys stay in the CLI/wallet; the node only ever sees public keys and signatures.
- Domain-separated signing hashes and strict nonce checks mitigate replay.
- Fixed-percentage double-sign slashing is implemented.

## License

Apache-2.0

## Contributing

Contributions are welcome. Please open an issue or pull request against the `main` branch. Ensure `cargo fmt`, `clippy -D warnings`, and tests pass. See `AGENTS.md` and `.opencode/` for internal agent guidance used by the project.

---

*Generated from a full inspection of the `KovanicaDAG/kvnc` codebase. Ignore any prior README content.*
