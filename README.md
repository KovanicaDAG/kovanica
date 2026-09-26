# Kovanica Protocol Ecosystem

> **Single meta-monorepo** — This repository (`kovanica.git`) **IS the source-of-truth** for the entire Kovanica ecosystem. The `protocol/` directory contains the canonical Rust workspace (5 crates: `kovanica-dag`, `kovanica-state`, `kovanica-node`, `kovanica-cli`, `kovanica-ffi`). All consensus-critical changes go here.

---

## Repository Architecture

```
kovanica/                          ← THIS REPO (source-of-truth)
├── protocol/                      # Core consensus + ledger (Rust workspace)
│   ├── crates/
│   │   ├── kovanica-dag/          # GHOSTDAG consensus core
│   │   ├── kovanica-state/        # UTXO ledger + scripts
│   │   ├── kovanica-node/         # Node library (RPC, mempool, P2P)
│   │   ├── kovanica-cli/          # CLI wallet
│   │   └── kovanica-ffi/          # LightNode UniFFI bindings
│   ├── docs/                      # RFCs, KVP specs, LEGIT-BOARD
│   └── desktop-app/               # Embedded node desktop app (Tauri)
├── node/                          # Thin binary wrapper (builds kovanica-node)
├── web/                           # Explorer + wallet + map (TanStack Start)
├── wallet/                        # Mobile wallet apps + browser extension
├── mobile/                        # Light-node mobile clients (FFI-based)
├── android-light-node/            # Actively developed Android light node
├── cli/                           # CLI wallet (standalone crate)
├── sdk/                           # Rust/WASM SDK
├── installer/                     # One-click installers (30+ platforms)
├── ledger-app/                    # Hardware wallet harness
├── agent/                         # RAG agent service (Python)
├── data/                          # Runtime data (NOT versioned)
├── brain-vault/                   # Private knowledge base (Obsidian)
└── docs/                          # Cross-cutting docs (NETWORK.md, MASTER-ROADMAP.md)
```

> **External repos** (live outside this workspace):
> - `kovanica-agent` → `~/kovanica-agent` (Python RAG service)
> - `kovanica-brain-vault` → `~/kovanica-brain-vault` (private Obsidian vault)

---

## Quick Start

```bash
# Clone the meta-monorepo (THIS IS THE SOURCE OF TRUTH)
git clone https://github.com/KovanicaDAG/kovanica.git
cd kovanica

# Provision everything (toolchains, deps, doctor report)
./dev.sh

# Or just sync and report status
./dev.sh --status
```

---

## Component Map

| Directory | Purpose | Build / Run |
|-----------|---------|-------------|
| **`protocol/`** | **Consensus + ledger crates** (source-of-truth) | `cargo build --workspace` |
| **`node/`** | `kovanica-node` binary packaging | `cargo build --release --workspace` |
| **`web/`** | Explorer, wallet, map frontend | `cd web/site && npm run dev` |
| **`wallet/`** | Android/iOS wallets + browser extension | `./gradlew assembleDebug` / `xcodebuild` |
| **`mobile/`** | Light-node mobile clients | `cd mobile/android && ./gradlew assembleDebug` |
| **`android-light-node/`** | Active Android light node (FFI) | `./gradlew assembleDebug` |
| **`cli/`** | Command-line wallet | `cargo build --release` |
| **`sdk/`** | Rust/WASM SDK | `cargo build --workspace` |
| **`installer/`** | Cross-platform installers | Used by `kovanica-node` releases |
| **`ledger-app/`** | Ledger/Trezor hardware wallet support | `make -C ledger-app/app` |
| **`agent/`** | Python RAG agent | `python -m agent.core` |

---

## Live Network (Testnet)

| Property | Value |
|----------|-------|
| **Network** | `kovanica-testnet` |
| **Explorer** | https://explorer.kovanica.online |
| **Wallet** | https://wallet.kovanica.online |
| **Map** | http://map.kovanica.online |
| **API** | https://api.kovanica.online |
| **P2P** | TCP **9000** only (no libp2p) |
| **Bootstrap Seed** | `seed.kovanica.online:9000` (DNS-only, grey-cloud) |
| **Secondary Seed** | `seed2.kovanica.online:9000` |
| **Token** | **KVNC** (8 decimals, 1 KVNC = 100,000,000 atoms) |

---

## Protocol Highlights

- **Consensus**: GHOSTDAG BlockDAG (k=3) — parallel blocks, deterministic linearization
- **Ledger**: Pure UTXO, Ed25519 spend authorization
- **Admission**: **PoA-only** (ratified 2026-09-25) — fixed authority set, 3-second slots, Ed25519 authority signatures. PoW and hybrid admission are `[TARGET]`-for-removal.
- **Tokenomics (RFC-006, live on testnet)**:
  - Max supply: **90.2M KVNC** (hard cap)
  - Emission: 10 KVNC/block genesis, geometric decay (×¾ per 2M blocks)
  - Coinbase maturity: **100 blocks**
  - Fee split: **75% burned / 25% to producer**
  - Fee floor: `max(1, subsidy / 500,000)` atoms/byte
- **Shipped RFCs**: Multisig (KVP-101), Native Tokens (KVP-102), Stealth+Script v2 (KVP-103), HTLC/Atomic Swaps (KVP-104), Vault/CSV (KVP-105)

---

## For Developers

### Protocol Work (Consensus/Ledger) — **Work in `protocol/`**

```bash
cd protocol
cargo build          # Build all 5 crates
cargo test           # Unit + integration + doctests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

> **Read `protocol/AGENTS.md` first** — source of truth for conventions, invariants, and workflow.

### Node Operations

```bash
# One-liner install (Linux/macOS)
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash

# Run locally
~/kovanica-node/run.sh
# Open http://127.0.0.1:8080
```

See **[node/JOIN.md](node/JOIN.md)** and **[node/TESTNET.md](node/TESTNET.md)**.

### Web Frontend

```bash
cd web/site
npm ci
npm run dev          # Dev server on 127.0.0.1:8080
npm run build:vps    # Production build for PM2 deploy
```

See **[web/site/DEPLOY.md](web/site/DEPLOY.md)**.

### Mobile / FFI

```bash
# Build Android AAR (from protocol root)
cd protocol/crates/kovanica-ffi
./build-android.sh

# Build Android light-node app
cd ../../../android-light-node
./gradlew assembleDebug
```

### CLI Wallet

```bash
cd cli
cargo build --release
# Binary at target/release/kovanica
```

---

## Documentation

| Document | Location |
|----------|----------|
| **Network & Domains** | [NETWORK.md](NETWORK.md) |
| **Master Roadmap** | [MASTER-ROADMAP.md](MASTER-ROADMAP.md) |
| **Testnet Parameters (RFC-006)** | [protocol/TESTNET-RFC006.md](protocol/TESTNET-RFC006.md) |
| **Tokenomics Spec** | [protocol/docs/TOKENOMICS.md](protocol/docs/TOKENOMICS.md) |
| **RFC Index** | [protocol/docs/](protocol/docs/) |
| **Operations Runbook** | [protocol/OPERATIONS.md](protocol/OPERATIONS.md) |
| **Web Deploy** | [web/site/DEPLOY.md](web/site/DEPLOY.md) |
| **PoA Migration RFC** | [protocol/docs/RFC-POA-Migration.md](protocol/docs/RFC-POA-Migration.md) |
| **Agent Conventions** | [protocol/AGENTS.md](protocol/AGENTS.md) |

---

## Contributing

1. **Never commit to `main` directly** — use feature branches:
   - `consensus/…`, `dag/…`, `ledger/…` — protocol changes
   - `web/…`, `mobile/…`, `cli/…`, `sdk/…` — component changes
2. **Linear history** — `git pull --rebase origin main` before starting and before push.
3. **Run checks before push**:
   - Rust: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
   - Web: `npm run typecheck && npm run lint`
4. **Private keys/seeds stay client-side** — the node never receives them.
5. **Multi-agent safety**: Multiple AI agents work concurrently. **One agent, one branch.** Never share branches without explicit coordination. Never plain `git push --force` — use `--force-with-lease`.

---

## Security

- **No secrets in the repo** — `.env`/`.env.*` are gitignored.
- **Node policy never weakens consensus determinism**.
- **Report vulnerabilities**: GitHub Security Advisories or `security@kovanica.online`.

---

## License

**MIT OR Apache-2.0** — each component carries dual licensing. See individual directories for `LICENSE-MIT` and `LICENSE-APACHE`.

---

## Links

- **Website**: https://kovanica.online
- **Documentation**: https://docs.kovanica.online
- **GitHub Org**: https://github.com/KovanicaDAG
- **This Repository**: https://github.com/KovanicaDAG/kovanica
- **Explorer API**: https://explorer.kovanica.online/api/head
- **Bootstrap Info**: https://explorer.kovanica.online/api/bootstrap