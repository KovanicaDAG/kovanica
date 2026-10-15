# Kovanica (KUNA)

> A Rust-native, DAG-based Layer-1 blockchain implementing Mysticeti-style uncertified consensus, account-based state with staking, deterministic Wasmi contract execution, and an independent light-client / mobile stack.

**Status: pre-alpha.**
Not audited. Do not use with real value. Not for production.
Interfaces, consensus parameters, and tokenomics constants may change without notice.

The chain is **Kovanica**. The native coin ticker is **KUNA** (9 decimals). All crates, binaries, environment variables, and protocol prefixes use the `kvnc` namespace. Addresses use the `kvnc<hex>dag` format with a Blake3 checksum.

---

## Overview

`kvnc` is a Cargo workspace that implements a complete full-node stack:

- **Consensus**: Mysticeti-style uncertified wave DAG (`WAVE_LENGTH = 3`) with leader/vote/decide rounds, quorum direct commits, and path-based indirect commits, plus a **MysticGhost hybrid** that applies GHOSTDAG `k=3` scoped colouring. Non-blue (red) blocks are returned to the mempool rather than pruned.
- **State model**: Account-based (UTXO-free) with balances, nonces, contract code/storage, and sorted key-value Merkle state roots. Snapshots and pruning are supported.
- **Staking & emission**: Proof-of-stake validators (target 15–21 active), delegation, unbonding queue, epoch rotation, fixed-percentage slashing, and a geometric emission schedule with linear treasury vesting.
- **Execution**: Native transactions (transfer, stake, delegate, claim rewards, contract deploy/call) plus a Wasmi-based deterministic WASM runtime with fuel-based gas metering and memory caps.
- **Networking**: real libp2p — TCP transport with **Noise** encryption and **Yamux** multiplexing, plus Gossipsub, Kademlia, Identify, Ping, and block/state request-response. Persistent peer identity, peer scoring, sync, and orphan buffering.
- **Interfaces**: JSON-RPC over HTTP (`POST /rpc`), WebSocket subscriptions (`GET /ws`), `/health` and `/metrics`, an OpenAPI document generated in code, a full-featured CLI wallet, and a TypeScript API client.
- **Light clients & mobile**: a light-client verification crate (`kvnc-light`), a non-consensus light node (`kvnc-lightnode`), UniFFI bindings (`kvnc-light-ffi`), and Android/iOS apps.
- **Communications**: an independent, client-only E2E messaging stack (`kvnc-comms-*`) plus an on-chain name registry contract.

The node is designed to run comfortably on modest hardware (target 2–4 GB RAM with aggressive DAG pruning).

**Testnet status (2026-10-14):** A single canonical seed `testnet.kovanica.online` fronts the testnet mesh; a small validator quorum has been verified for quorum commit, sustained liveness, late-join sync, and VRF-driven leader election (quorum beacon on decide rounds). See `docs/TESTNET-SEED-NODES.md` and `docs/SEED-DNS-AUDIT.md`.

## Features

- **DAG ledger** with causal ordering, stake-filtered parent selection, and a bounded parent round gap.
- **Wave-based consensus** (leader / vote / decide) with quorum (2f+1) direct commits and path-based indirect commits.
- **Stake-weighted leader selection** with **ECVRF-ED25519-SHA512-Elligator2** support (quorum beacon on decide rounds) and deterministic round-robin fallback when no validator VRF keys are present.
- **MysticGhost hybrid colouring** (GHOSTDAG `k=3` mergeset + Mysticeti wave DAG) with non-blue blocks returned to the mempool.
- **Account + staking model** including commission, reward sharing, unbonding, treasury vesting, and a genesis validator allocation (1M KUNA split across the genesis set).
- **Deterministic contract execution** via Wasmi (fuel metering, memory caps, module cache) plus native typed contract APIs (HTLC, Vault, Multisig, Token).
- **Verifiable state** with sorted-KV Merkle roots and snapshot/export support.
- **P2P networking** over Noise-encrypted TCP with gossipsub for blocks, transactions, votes, sync, state-sync, and commit seals; request/response block and state sync; orphan handling; and peer scoring.
- **JSON-RPC & WebSocket** covering chain, transaction, account, staking, mempool, consensus, and light-client reads, plus subscriptions (`newHeads`, `newCommittedLeader`, `pendingTransactions`, `newSealedHeader`, and per-sub-DAG transaction-log receipts).
- **CLI** for key management (encrypted keystore v2 with Argon2id + XChaCha20-Poly1305), transfers, staking operations, and contract interactions, plus an interactive TUI wallet.
- **Light-client & mobile** verification crates and apps (Android, iOS, desktop).
- **Observability**: structured tracing (with per-round trace IDs), Prometheus metrics, and example Grafana dashboards.
- **Docker & Compose**: multi-stage Dockerfile and a seed + 3-validator local testnet compose file.

## Tokenomics

Canonical constants (implemented in `crates/kvnc-staking` and documented in `docs/TOKENOMICS.md`):

| Parameter | Value |
|-----------|-------|
| Total supply | 90 200 000 KUNA |
| Decimals | 9 (1 KUNA = 1 000 000 000 atoms) |
| Founder premine | 200 000 KUNA |
| Genesis validator allocation | 1 000 000 KUNA (split across 5–20 genesis validators; remainder distributed deterministically) |
| Treasury | 8 000 000 KUNA (1 M per year × 8 years, linear vesting) |
| Mining subsidy budget | 81 000 000 KUNA (derived: 90.2M − 0.2M − 8M − 1M) |
| Initial block reward | 2.5 KUNA (paid to the author of a committed leader block) |
| Subsidy era length | 8 100 000 committed leaders |
| Reward decay | × 3/4 per era |
| Minimum validator stake | 50 000 KUNA |
| Active validator range | 15–21 |
| Unbonding period | 100 000 rounds |
| Epoch rotation length | 7 200 rounds (`EPOCH_ROUNDS`; deliberately decoupled from the subsidy era) |
| Slashing | 500 bps (5%) fixed-percentage double-sign slash |
| Light-node reward share | 20% (2 000 bps) of the block reward, escrowed for light nodes |
| Minimum light-node bond | 100 KUNA |
| Design round / leader interval | `round_duration_ms` default 4 000 ms → 12 s per committed leader (wave length 3) |

Rewards, treasury vesting, and circulating-supply accounting are applied atomically inside the execution of each committed sub-DAG.

**Not yet implemented:** output/coinbase maturity rules, fee-burn/fee-split accounting (burn vs producer), and a dynamic fee floor are **not** present anywhere in the codebase. Do not assume these behaviors.

## Repository Layout

| Path | Purpose |
|------|---------|
| `crates/` | All 27 Rust crates (see below) |
| `packages/` | TypeScript packages: API client, signer, comms glue, console dapp, explorer web |
| `apps/` | Desktop (Tauri), Android, and iOS applications |
| `docs/` | Architecture, tokenomics, contracts, signature format, comms specs, RFCs, ADRs, and operational notes (31 markdown files) |
| `mainnet/` | Mainnet `genesis.json`, `validators/validators.json` (3 validators at 50 000 KUNA each), `genesis_validators.toml`, validator key files, and `run-seed1..3.sh` |
| `testnet/` | Testnet `genesis/genesis.json` and `validators.json` |
| `ops/` | Docker, genesis ceremony, Grafana, Prometheus, benchmark and soak scripts |
| `packaging/` | systemd unit and Windows service install scripts |
| `fuzz/` | `cargo-fuzz` targets (block, consensus, tx, vote) |
| `brand-assets/` | Logos |
| `config.example.toml` | Example node configuration with environment-variable mappings |
| `Dockerfile` / `docker-compose.yml` | Container build and local testnet compose |
| `TASKLIST.md` | Phased implementation status (code-verified) |
| `AUDIT_REPORT.md` / `AUDIT_FIRMS.md` / `AUTOMATED_SECURITY_CHECKS.md` | Internal audit template, audit-firm shortlist, and automated check notes |
| `.github/workflows/` | CI: fmt, clippy, tests, `cargo audit`, docs, MSRV 1.90, release check, foundation-crate matrix, faucet/explorer deploy |
| `.opencode/` | Internal agent tooling and skills used by the project |
| `run-seed1.sh`, `opencode.json` | Local seed-node launcher and OpenCode project config |

### Core Crates

Workspace crate versions default to `0.2.0`; the table records the version each crate currently declares.

| Crate | Version | Responsibility |
|-------|---------|----------------|
| `kvnc-types` | 0.0.31 | Core types (blocks, transactions, addresses `kvnc…dag`, hashes, rounds, votes, committee, signing context) |
| `kvnc-crypto` | 0.0.32 | Ed25519 sign/verify, batch verification, key generation, ECVRF VRF |
| `kvnc-keystore` | 0.1.0 | Encrypted keystore v2 (Argon2id + XChaCha20-Poly1305) |
| `kvnc-storage` | 0.0.31 | redb-backed block/state/consensus stores, Merkle roots, pruning, snapshots |
| `kvnc-dag` | 0.2.0 | DAG store, causal ordering, block manager, mergesets, pruning |
| `kvnc-consensus` | 0.0.31 | Mysticeti-style wave DAG, committer, linearizer, MysticGhost / GHOSTDAG colouring, metrics |
| `kvnc-mempool` | 0.2.0 | Fee-rate priority pool, admission, eviction, rebroadcast |
| `kvnc-network` | 0.2.0 | libp2p service (TCP/Noise/Yamux), protocols, gossipsub topics, scoring, identity |
| `kvnc-runtime` | 0.2.0 | Wasmi WASM engine and fuel-based gas metering |
| `kvnc-execution` | 0.2.0 | State-transition function, reward distribution, contract host |
| `kvnc-staking` | 0.0.31 | Emission schedule, treasury, validator set, delegation, slashing, light-node rewards |
| `kvnc-node` | 0.0.31 | Full-node binary (`kvnc-node`), incl. the `genesis` subcommand |
| `kvnc-cli` | 0.0.31 | CLI wallet and tooling (bins `kvnc`, `generate-validators`, `kvnc-bench`) |
| `kvnc-rpc` | 0.2.0 | JSON-RPC server, WebSocket subscriptions, rate limiting, OpenAPI generation |
| `kvnc-common` | 0.2.0 | Shared contract types, WASM helpers, memory utilities, Host trait |
| `kvnc-htlc` | 0.2.0 | HTLC contract (native + WASM) |
| `kvnc-vault` | 0.2.0 | Time-lock vault contract (native + WASM) |
| `kvnc-multisig` | 0.2.0 | M-of-N multisig contract (native + WASM) |
| `kvnc-token` | 0.2.0 | Native multi-asset token contract (native + WASM) |
| `kvnc-faucet` | 0.1.0 | Rate-limited testnet faucet service |
| `kvnc-light` | 0.1.0 | Light-client verification (headers, seals, inclusion proofs) |
| `kvnc-lightnode` | 0.1.0 | Light node: seal-follow and transfers, no consensus loop |
| `kvnc-light-ffi` | 0.1.0 | UniFFI bindings for the light client (Android/Kotlin, iOS/Swift) |
| `kvnc-comms-core` | 0.2.0 | Canonical comms identity (`BLAKE3("kvnc/comms/id" ‖ pubkey)`) |
| `kvnc-comms-crypto` | 0.2.0 | 1:1 E2E sealing (X25519 + XChaCha20-Poly1305), client-only |
| `kvnc-comms-relay` | 0.2.0 | Store-and-forward relay for opaque comms envelopes (M1 + M2) |
| `kvnc-name-registry` | 0.2.0 | On-chain comms name registry contract (RFC-008, ledger-safe) |

### Packages

| Package | Version | Purpose |
|---------|---------|---------|
| `@kvnc/api-client` | 0.0.31 | Typed fetch wrapper for `kvnc-rpc` |
| `@kvnc/signer` | 0.0.1 | Browser transaction signer (signature format v1) |
| `@kvnc/comms` | 0.0.1 | TypeScript glue for the comms app: comms identity + RPC access |
| `@kvnc/console` | 0.0.1 | React/Vite dapp: Overview, Account, Transfer, Stake, Activity, Faucet |
| `@kvnc/web` | 0.0.1 | React/Vite block explorer (deployed to Cloudflare Pages as `kvnc-explorer`) |

### Applications

| App | Identifier | Stack |
|-----|------------|-------|
| `apps/desktop` | `@kvnc/desktop` 0.1.0 | Tauri v2 + React comms client |
| `apps/android` | `io.kovanica.light` (versionCode 5 / versionName 0.5.0, minSdk 24) | Kotlin + UniFFI light client |
| `apps/ios` | `KovanicaLight` | SwiftUI light client |

### Documentation

Key documents under `docs/` (all 31 markdown files are the source of truth over this README):

- `docs/ARCHITECTURE.md`, `docs/TOKENOMICS.md`, `docs/CONTRACTS.md`, `docs/SIGNATURE_FORMAT.md`, `docs/GAS_SCHEDULE.md`, `docs/EXAMPLES.md`
- `docs/VALIDATOR-ONBOARDING.md`, `docs/TESTNET-SEED-NODES.md`, `docs/SEED-DNS-AUDIT.md`, `docs/TESTNET-RESET.md`
- `docs/GENESIS-CEREMONY.md` (draft), `docs/MAINNET-FREEZE-CHECKLIST.md` (draft), `docs/GOVERNANCE.md` (Phase-25 draft — **not implemented**), `docs/AUDIT-SCOPE.md`
- `docs/COMMS-APP-PLAN.md`, `docs/COMMS-M1-INTERFACES.md`, `docs/COMMS-M2-INTERFACES.md`, `docs/COMMS-THREAT-MODEL.md`, `docs/COMMS-ANDROID-INTERFACES.md`, `docs/RFC-008-NAME-REGISTRY.md`
- `docs/rfcs/RFC-B1-B2-VRF-AND-MIN-DIGEST.md`, `docs/adr/0001-mysticeti-c-weighted-consensus.md`, `docs/adr/0002-cli-wallet-keystore-v2.md`, `docs/design/epoch-rotation.md`, `docs/design/multi-proposer.md`

> Note: `docs/ARCHITECTURE.md` references a `docs/RPC.md` reference document that does not exist in this tree. The OpenAPI document is generated in code (`crates/kvnc-rpc/src/openapi.rs`).

## Quick Start

### Requirements

- Rust stable and Cargo. **MSRV is 1.90**, but there is no `rust-version` key in `Cargo.toml` — the floor is enforced by CI (`dtolnay/rust-toolchain@1.90`) and the Docker builder (`rust:1.90-bookworm`).
- (Optional) Docker and Docker Compose for the local testnet.
- (Optional) the `wasm32-unknown-unknown` target for full contract test coverage.

### Build and Test

```bash
git clone https://github.com/KovanicaDAG/kvnc.git
cd kvnc
cargo build --workspace
cargo check --workspace --all-targets
cargo test --workspace
```

CI runs `cargo fmt --check`, `clippy -D warnings`, `cargo test --all-targets`, doctests, `cargo audit`, documentation builds, an MSRV (1.90) job, and a foundation-crate matrix.

### Run a Node

```bash
cp config.example.toml config.toml
# Edit config.toml or override via environment variables.
cargo run -p kvnc-node -- --config config.toml
```

Configuration keys (see `config.example.toml` for the full list and the environment-variable mapping):

- `data_dir` — chain data (separate consensus and state redb files) → `KVNC_DATA_DIR`
- `listen_addr` — P2P listen address, default `0.0.0.0:8000` (host:port or a full libp2p multiaddr) → `KVNC_LISTEN_ADDR`
- `rpc_addr` / `rpc_port` — JSON-RPC bind, default `127.0.0.1:8545` → `KVNC_RPC_ADDR`, `KVNC_RPC_PORT`
- `bootnodes` — list of multiaddrs or DNS seeds; **empty by default**, no hosts are hard-coded → `KVNC_BOOTNODES`
- `round_duration_ms` — target consensus round duration → `KVNC_ROUND_DURATION_MS`
- `max_peers` — maximum connected peers (example default 50) → `KVNC_MAX_PEERS`
- `validator_key` — path to a key file. Required while `run_validator` is true (the default); a read-only node may omit it → `KVNC_VALIDATOR_KEY`, `KVNC_RUN_VALIDATOR`
- `rpc_rate_limit_per_min` — per-IP token-bucket limit, burst derived as `max(1, rate/6)` → `KVNC_RPC_RATE_LIMIT_PER_MIN`
- `use_mysticghost` — MysticGhost hybrid colouring (default `true`) → `KVNC_MYSTICGHOST`
- `use_vrf_leader` — VRF-driven leader election (default `true`) → `KVNC_VRF_LEADER`
- `treasury_address` — genesis treasury address (32 bytes hex); only read on fresh genesis → `KVNC_TREASURY_ADDRESS`

Additional environment variables read by the node:

- `KVNC_CHAIN_ID` — chain id committed to by signatures (see the registry below)
- `KVNC_RPC_AUTH` — set to `disable` to skip bearer-token auth on write methods
- `KVNC_RPC_WRITE_TOKENS` — comma-separated bearer tokens accepted for RPC write methods
- `KVNC_MEMPOOL_REBROADCAST_INTERVAL_SECS`, `KVNC_MEMPOOL_IN_FLIGHT_TIMEOUT_SECS`, `KVNC_MEMPOOL_MIN_FEE_RATE`, `KVNC_MEMPOOL_MAX_TXS_PER_SENDER` — mempool tuning

Chain-id registry: **mainnet = 1, testnet = 2, devnet = 3, local = 1337**.

The `kvnc-node` binary exposes a `genesis` subcommand that builds a genesis block and staking state from a validators list:

```bash
kvnc-node genesis \
  --validators validators.json \
  --treasury-address <64 hex> \
  --founder-address <64 hex> \
  --output genesis.json \
  [--validator-keys-out <dir>] \
  [--force]
```

### Docker

```bash
docker build -t kvnc-node:local .
docker compose up   # seed + 3 validators, KVNC_CHAIN_ID=2
```

The image is a multi-stage build that runs as a non-root user (uid/gid 10001), persists chain data on a `VOLUME` at `/var/lib/kvnc`, and includes a healthcheck against `/health`. The node listens on P2P **8000** and RPC **8545** by default. See `ops/docker/` and `docker-compose.yml` (`kvnc-testnet`: a `seed` service plus `val1`–`val3`, bootnode `testnet.kovanica.online:8000`) for entrypoint and multi-node details.

### CLI Highlights

```bash
# Primary CLI binary (from the kvnc-cli crate):
cargo run -p kvnc-cli --bin kvnc -- --help
# wallet: keygen, import, export, address, sign, tui
# chain: balance, status, transfer
# staking: stake, unstake, delegate, claim-rewards
# contracts: HTLC, Vault, Multisig, Token subcommands

# Additional binaries from the same crate:
cargo run -p kvnc-cli --bin generate-validators -- --help  # genesis validator key generation
cargo run -p kvnc-cli --bin kvnc-bench -- --help           # benchmarking tool

# Light node (kvnc-lightnode crate): transfer / bond (register VRF) / exit / witness
cargo run -p kvnc-lightnode -- --help
```

Keys are stored in an encrypted keystore (Argon2id + XChaCha20-Poly1305). The node never receives private seeds. `kvnc tui` opens an interactive wallet for balances, transfers, and staking against that same local keystore.

## Architecture (High Level)

```
Network (libp2p: TCP + Noise + Yamux)
    │
    ▼
Mempool ──────────────────────────────┐
    │                                 │
    ▼                                 │
Block Producer (per validator)        │
    │                                 │
    ▼                                 │
Consensus (Mysticeti wave DAG +       │
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

Consensus parameters of note: `WAVE_LENGTH = 3`, `round_duration_ms` default 4 000 (→ 12 s per committed leader), MysticGhost = scoped GHOSTDAG `k=3`, and `EPOCH_ROUNDS = 7 200`. Leader election uses **ECVRF-ED25519-SHA512-Elligator2** with a quorum beacon on decide rounds, falling back to stake-weighted round-robin when the VRF set is empty.

### RPC & Interfaces

- HTTP router: `POST /rpc`, `GET /ws`, `GET /health`, `GET /metrics`. `/rpc` and `/ws` are rate-limited per client IP; default bind is `127.0.0.1:8545`.
- **23 implemented `kvnc_*` methods** span chain, transaction, account, staking, mempool, consensus, and light-client reads (e.g. `kvnc_blockNumber`, `kvnc_sendRawTransaction`, `kvnc_getBalance`, `kvnc_getValidators`, `kvnc_getStake`, `kvnc_getCommitSeals`, `kvnc_getLightSet`, `kvnc_lightClientCheckpoint`), plus `subscribe` / `unsubscribe`.
- **15 contract methods are registered but not wired** — `htlc_*` (3), `vault_*` (3), `multisig_*` (4), `token_*` (5). All return `NotImplemented` (**-32004**) until a real contract host is attached. Clients must submit contract calls through `kvnc_sendRawTransaction` instead.
- WebSocket subscription kinds (5): `newHeads`, `newCommittedLeader`, `pendingTransactions`, `logs` (transaction execution receipts, emitted per committed sub-DAG), and `newSealedHeader`.
- `kvnc_lightClientCheckpoint` is registered but is a **minimal read-only stub** that reports `verified = false`; state-backed checkpoints are not implemented yet.
- OpenAPI is generated in code; there is no committed spec file.

Faucet (`kvnc-faucet`): default listen `0.0.0.0:8080`, `--chain-id` is required (no default), dispenses **10 KUNA** per request with a **3 600 s window / 3 requests per IP** rate limit.

## Light Client & Mobile

The light stack lets clients follow committed leader headers and seals without running consensus:

- `kvnc-light` — verification of headers, commit seals, and inclusion proofs.
- `kvnc-lightnode` — a light node that follows seals and submits transfers, with no consensus loop. Its CLI supports `transfer`, `bond` (bond KUNA and register the keystore's VRF key, default 100 KUNA), `exit`, and hourly witness submission. It is **not** a validator.
- `kvnc-light-ffi` — UniFFI bindings consumed by the Android (Kotlin) and iOS (Swift) apps.
- Apps: `apps/android` (`io.kovanica.light`, versionCode 5 / versionName 0.5.0, minSdk 24) and `apps/ios` (SwiftUI `KovanicaLight`), both driven by the generated UniFFI bindings.

Light nodes earn a **20% share** of the block reward (`LIGHT_REWARD_BPS = 2 000`), escrowed per epoch across `LIGHT_WITNESS_SLOTS = 32` winning witness slots, capped at `MAX_LIGHT_NODES = 4 096` identities, with a minimum bond of `MIN_LIGHT_BOND = 100 KUNA`. The light node reads its keystore passphrase from `KVNC_LIGHT_PASSPHRASE`; a systemd unit is provided in `packaging/systemd/kvnc-lightnode.service`.

The only light-client RPC surface today is `kvnc_lightClientCheckpoint` (stub, `verified = false`) and `kvnc_getLightSet` — see the RPC notes above.

## Communications

The comms stack is an independent, **client-only / ledger-safe** messaging layer (no consensus changes). It is specified under `docs/COMMS-*.md` and `docs/RFC-008-NAME-REGISTRY.md`:

- `kvnc-comms-core` — canonical comms identity, `BLAKE3("kvnc/comms/id" ‖ Ed25519 pubkey)`.
- `kvnc-comms-crypto` — 1:1 E2E sealed envelopes (raw-key X25519 + XChaCha20-Poly1305).
- `kvnc-comms-relay` — store-and-forward relay for opaque envelopes, covering the frozen M1 (messaging) and M2 (payments/anti-spam) designs.
- `kvnc-name-registry` — on-chain name → owner / wallet key / relay-endpoints registry contract (RFC-008; ledger-safe, new contract state only).
- `packages/comms` — TypeScript glue (comms identity + RPC).
- `apps/desktop` — Tauri v2 + React desktop client.

Status: specs are frozen at M1 and M2, but the client apps are not shipped.

## Development Status

Most foundation phases (types, crypto, storage, DAG, consensus core, execution, staking, node wiring, RPC, CLI wallet) are implemented and covered by unit and multi-node integration tests. Multi-node quorum, late-join, sustained liveness, live vote→commit→execute, and VRF-driven leader election (quorum beacon on decide rounds) have all been exercised on the testnet mesh.

**Verified locally at `f4e5082` (branch `main`, 2026-10-15):**
- `cargo check --workspace --all-targets` → completes (2 dead-code warnings in a `kvnc-types` test-only target).
- `cargo test --workspace --lib` → every crate passes **except** one known-failing fixture test: `kvnc-cli::tx::tests::console_signer_wire_fixtures_match_the_node` (`unexpected fixture call-value-zero`). The `@kvnc/signer` fixture `packages/signer/fixtures/wire.json` gained `call-value-zero` / `call-value-500` cases that the `kvnc-cli` cross-check test does not yet know about.

Remaining work includes: fixing the CLI/signer fixture drift above, longer soak tests (6h/24h), state-backed light-client checkpoints, governance hooks, production seed DNS registration, and an external audit. Consult `TASKLIST.md` for the authoritative, code-verified phase status.

## Security Notes

- Pre-alpha software. Do not use with real value.
- Not audited. `AUDIT_REPORT.md` is an internal template; no external audit has been performed (`AUDIT_FIRMS.md` tracks candidate firms).
- P2P is libp2p over **Noise-encrypted** TCP with Yamux; it is not plaintext. Seeds must be DNS-only (grey-cloud) A records — never Cloudflare-proxied hostnames — because raw TCP does not traverse the HTTP proxy.
- Consensus uses `use_min_digest_fork: true` by default (lexicographic min-digest tie-break on same-round/same-author forks).
- VRF leader election defaults ON; with no validator VRF keys it falls back to stake-weighted round-robin.
- MysticGhost hybrid colouring defaults ON; non-blue blocks are returned to the mempool (not pruned).
- Client keys stay in the CLI/wallet; the node only ever sees public keys and signatures.
- Domain-separated signing hashes (`KUNA/tx/v1`, `KUNA/vote/v1`, `KUNA/commit/v1`) and strict nonce checks mitigate replay.
- Fixed-percentage double-sign slashing is implemented (500 bps).
- RPC write methods require bearer-token auth by default; `KVNC_RPC_AUTH=disable` turns this off (for isolated dev only). An open faucet or an operator node should never face the public internet without explicit isolation.

## License

Apache-2.0 (see `LICENSE`). Some packages under `packages/` are MIT-licensed as noted in their `package.json`.

## Contributing

Contributions are welcome. Please open an issue or pull request against the `main` branch. Ensure `cargo fmt`, `clippy -D warnings`, and tests pass before submitting. See `AGENTS.md` and `.opencode/` for internal agent guidance used by the project.

---

*Refresh of the root README against the `kvnc` tree at `f4e5082` (branch `main`). Where this document and `docs/` disagree, `docs/` wins.*
