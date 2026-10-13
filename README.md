# Kovanica (kvnc)

Kovanica is a Rust workspace for a DAG-based Layer-1 blockchain prototype. The project is designed around a Mysticeti-style uncertified DAG, a Wasmi-based execution layer, staking and validator logic, and a libp2p networking stack. The chain name is Kovanica and the native token is KUNA with 9 decimals.

Status: prototype / pre-alpha. This repository is not production-ready, not audited, and not intended for a live mainnet deployment.

## Scope and current state

This repository contains a real Rust workspace and multiple crates, but it is still a development codebase with ongoing architecture and milestone work. The code and docs in the repo consistently describe the following:

- Consensus: Mysticeti-style, uncertified DAG with wave = 3
- Execution: Wasmi deterministic WASM execution
- State model: account-based runtime + staking + validator lifecycle
- Networking: libp2p with gossipsub, Kademlia, identify, ping, sync modules
- API: JSON-RPC server, WebSocket support, CLI wallet and staking commands
- Contracts: HTLC, Vault, Multisig, Token, plus host infrastructure

The repository includes both Rust crates and a small TypeScript package surface (`packages/*`).

## Repository layout

At the root, the project is structured like this:

- `Cargo.toml` — workspace manifest
- `config.example.toml` — sample node configuration
- `docker-compose.yml` — multi-node local integration setup
- `Dockerfile` — container build definition
- `docs/` — design, architecture, tokenomics, and audit-related docs
- `crates/` — workspace Rust crates
- `packages/` — package-based frontend/client code
- `ops/` — operations and deployment scripts
- `brand-assets/` — branding assets
- `fuzz/` — fuzzing targets
- `AGENTS.md` — repo-specific agent instructions
- `TASKLIST.md` — project roadmap / task tracking

## Workspace crates

The workspace manifest currently includes the following crates:

- `kvnc-types`
- `kvnc-crypto`
- `kvnc-storage`
- `kvnc-dag`
- `kvnc-consensus`
- `kvnc-mempool`
- `kvnc-network`
- `kvnc-runtime`
- `kvnc-execution`
- `kvnc-staking`
- `kvnc-node`
- `kvnc-cli`
- `kvnc-rpc`
- `kvnc-common`
- `kvnc-htlc`
- `kvnc-vault`
- `kvnc-multisig`
- `kvnc-token`
- `kvnc-faucet`

The repo also contains generated / package directories such as:

- `packages/api-client`
- `packages/web`

## Main components

### 1) Consensus and DAG

The consensus layer is implemented in `kvnc-consensus` and `kvnc-dag`.

The repository docs and code describe the system as a Mysticeti-style DAG with:

- leader blocks
- committed sub-DAG logic
- linearization / commit processing
- vote verification and validation
- state sync and block sync flows

The network crate explicitly contains libp2p features including gossipsub, Kademlia, identify, ping, and custom sync protocols.

### 2) Execution and runtime

The runtime and execution layer uses Wasmi for deterministic WASM execution.

Relevant components:

- `kvnc-runtime` — Wasmi runtime integration
- `kvnc-execution` — execution context, contract execution, state transitions
- `kvnc-common` — shared helpers and runtime glue

The repository also includes contract crates:

- `kvnc-htlc`
- `kvnc-vault`
- `kvnc-multisig`
- `kvnc-token`

These are present as workspace crates, and the docs describe them as contract implementations with host trait support.

### 3) Staking and token economics

The staking crate is central to validator selection, rewards, treasury logic, and emission. The project documentation explicitly locks the tokenomics and states that they should not be modified without approval.

The current locked parameters documented in the repo are:

- Total supply: 90,200,000 KUNA
- Decimals: 9
- Founder premine: 200,000 KUNA
- Treasury: 8,000,000 KUNA (1,000,000 per year for 8 years, linear)
- Genesis validator allocation: 1,000,000 KUNA
- Mining subsidy budget: 81,000,000 KUNA
- Initial block reward: 2.5 KUNA
- Subsidy era length: 8,100,000 committed leader blocks
- Reward decay: × 3/4 each era
- Round / leader interval: 4 s round, 12 s leader commit interval
- Active validators: 15–21
- Minimum validator stake: 50,000 KUNA

The repo documents that new rewards are paid only on committed leader blocks and that the mining schedule is not intended to be changed casually.

### 4) Network and node

The node binary is `kvnc-node`, and the project includes a `kvnc-rpc` crate and a `kvnc-network` crate.

Notable implementation details from the current repo:

- `kvnc-network` uses libp2p
- `kvnc-rpc` includes axum-based RPC endpoints and WebSocket support
- `kvnc-node` wires together storage, consensus, network, execution, staking, and RPC
- `config.example.toml` defines how a node is configured and how environment variables override config values

### 5) CLI and tooling

The CLI is `kvnc` and is shipped via `crates/kvnc-cli`.

Features present in this repo include:

- CLI wallet functions
- staking operations
- RPC client calls
- contract commands
- benchmarking helper binary
- validator generation example

The repo also contains `kvnc-faucet`, a rate-limited faucet service intended for testnet usage.

## Build and test

The repository is configured as a Cargo workspace.

Typical commands:

```bash
cargo build --workspace
cargo test --workspace
```

To run the node:

```bash
cp config.example.toml config.toml
cargo run -p kvnc-node -- --config config.toml
```

The repository also contains a Docker setup:

```bash
docker compose up --build
```

## Documentation status

The repo includes several docs, including:

- `docs/ARCHITECTURE.md`
- `docs/TOKENOMICS.md`
- `docs/CONTRACTS.md`
- `docs/GAS_SCHEDULE.md`
- `docs/AUDIT-CHECKLIST.md`
- `docs/SEED-DNS-AUDIT.md`
- `docs/SIGNATURE_FORMAT.md`
- `docs/design/...`
- `docs/adr/...`

These docs describe the intended architecture and current milestones. Some content still reflects future or phased roadmap items, so the project should be treated as an active prototype, not a finalized blockchain platform.

## Important truths to know

This repository is not a copy or fork of another chain. The project explicitly states a requirement to remain independent and avoid mixing in code or design from `kovanica-protocol` or older Kovanica repositories.

The repo is intentionally designed toward a low-resource target and mentions comfortable operation on a 2–4 GB RAM VPS, but it is still a development codebase.

The project currently includes no root `LICENSE` file at the time of inspection, and the README should not claim a finalized licensing state unless the repository adds one.

## Risks and limitations

Before using or depending on this code:

- it is not formally audited
- it is not a production blockchain
- consensus and execution details are still evolving
- some roadmap items remain open
- some implementation details are phase-based and may change

## Summary

This repository is a real Rust workspace for a Kovanica blockchain prototype with a DAG-based consensus model, Wasmi execution, staking, validators, libp2p networking, CLI tools, and contract crates. It is a serious engineering prototype, but it is not yet a live public network or a fully audited production system.

For precise implementation details, the repository's canonical documents are:

- `AGENTS.md`
- `docs/ARCHITECTURE.md`
- `docs/TOKENOMICS.md`
- `Cargo.toml`
- `TASKLIST.md`

This README reflects the actual repository state as it exists in the checked-out code and docs.

