# kovanica-protocol — Core Consensus + Ledger (Source-of-Truth)

> **Canonical protocol workspace** — This directory contains the **actual Rust workspace** with the 5 consensus/ledger crates. All consensus-critical changes are authored here.

---

## Architecture Overview

```
protocol/
├── crates/
│   ├── kovanica-dag/      # DAG + GHOSTDAG consensus core (reachability, colouring, linearization)
│   ├── kovanica-state/    # UTXO ledger applied in GHOSTDAG order (ed25519, multi-asset, scripts)
│   ├── kovanica-node/     # Runnable node: RPC, mempool, P2P mesh, block production, explorer
│   ├── kovanica-cli/      # CLI wallet (embedded in node binary)
│   └── kovanica-ffi/      # LightNode — UniFFI bindings for Kotlin/Swift mobile light nodes
├── docs/                  # RFCs, KVP specs, LEGIT-BOARD, plans
└── desktop-app/           # Embedded node desktop app (Tauri, separate crate)
```

---

## Consensus & Ledger (Canonical)

| Property | Value |
|----------|-------|
| **Consensus** | GHOSTDAG BlockDAG (k=3) |
| **Ledger** | Pure UTXO |
| **Signatures** | Ed25519 (64-byte / 128 hex) |
| **Native Token** | KVNC (1 KVNC = 100,000,000 atoms) |
| **Admission** | **PoA-only** (ratified 2026-09-25) — fixed authority set, 3s slots |
| **P2P** | Plaintext TCP **9000** only (no libp2p) |
| **Bootstrap** | `seed.kovanica.online:9000` (DNS-only, grey-cloud) |

### RFC-006 Tokenomics (Live on Testnet)

| Parameter | Value |
|-----------|-------|
| **Max Supply** | 90.2M KVNC (`9,020,000,000,000,000` atoms) |
| **Genesis Subsidy (s₀)** | 10 KVNC/block |
| **Era Length** | 2,000,000 blocks |
| **Decay (α)** | 3/4 per era (geometric, not halving) |
| **Coinbase Maturity** | 100 blocks |
| **Fee Split** | 75% burned / 25% to producer |
| **Fee Floor** | `max(1, subsidy / 500,000)` atoms/byte |
| **Founder Premine** | 0.2M KVNC |
| **Treasury** | 10M KVNC (10 × 1M RFC-005 vaults) |

> **PoA Migration**: Proof-of-Work and hybrid (PoW + VRF-staked) admission are `[TARGET]`-for-removal. See [RFC-POA-Migration.md](docs/RFC-POA-Migration.md) §0. The stake registry retires with hybrid. RFC-006 supply math is **unaffected** — only wall-clock pace changes.

---

## Shipped RFCs / KVP Standards

| RFC | KVP | Feature | Status |
|-----|-----|---------|--------|
| 001 | KVP-101 | Multisig (M-of-N P2SH) | ✅ Main |
| 002 | KVP-102 | Native Multi-Asset Tokens | ✅ Main |
| 003 | KVP-103 | Stealth Addresses + Script v2 | ✅ Main |
| 004 | KVP-104 | HTLC / Atomic Swaps | ✅ Main |
| 005 | KVP-105 | Time-Lock Vault + CSV | ✅ Main |
| 006 | — | Tokenomics (Emission, Cap, Fee Burn) | ✅ Live on testnet |
| — | KVP-106 | NFT (RFC-007 draft) | 🚧 Draft |
| — | — | DeFi (HTLC-first DEX) | 🚧 Design |
| — | — | RWA Integration | 🚧 Design |

---

## Quick Start

### Build & Test

```bash
# From protocol/ root
cargo build                 # Build all 5 crates
cargo test                  # Unit + integration + doctests
cargo clippy --all-targets  # Keep warning-clean
cargo fmt --check           # CI gate

# Run the node (scripted demo)
cargo run -p kovanica-node -- demo

# Run REPL (try `help`)
cargo run -p kovanica-node
```

### Run a Light Node (Kotlin/Android)

```bash
# Build native library + AAR (min SDK 24)
./crates/kovanica-ffi/build-android.sh

# Add crates/kovanica-ffi/android/ to your Gradle project
# Runtime dep: net.java.dev.jna:jna:5.14.0@aar
# Bindings package: uniffi.kovanica
```

### Run a Light Node (Swift/iOS)

```bash
# Build xcframework (macOS host)
./crates/kovanica-ffi/build-apple.sh

# Add target/kovanica.xcframework to Xcode project
# Compile bindings/swift/kovanica.swift into app target
```

---

## Development Conventions

- **Rust Edition**: 2021, `rust-version` 1.75 (toolchain pinned to **1.98.0** via `rust-toolchain.toml`)
- **CI Gates**: `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings`
- **No `unsafe`** — forbidden crate-wide
- **Determinism is sacred** — consensus must be a pure function of the DAG
- **Branch naming**: `consensus/…`, `dag/…`, `ledger/…`, `claude/<topic>`
- **Never commit to `main`** — feature branch + draft PR

See **[AGENTS.md](AGENTS.md)** for deep conventions, invariants, and workflow.

---

## Key Crates

### `kovanica-dag` — Consensus Core
- `block.rs` — Block (multi-parent vertex), `BlockId` (BLAKE3)
- `dag.rs` — Insert/validate, oracle-backed reachability, mergeset, tips
- `ghostdag.rs` — Selected parent, k-cluster blue/red colouring
- `ordering.rs` — Recursive GHOSTDAG linearization
- `reachability.rs` — Interval-tree + future-covering sets
- `difficulty.rs` / `pow.rs` — `[TARGET]`-for-removal
- `vrf.rs` — ECVRF over Ristretto255 (IRTF CFRG draft)

### `kovanica-state` — UTXO Ledger
- `keys.rs` — Address, KeyPair, `kvnc…dag` base58 encoding
- `tx.rs` — Transaction, TxId, OutPoint, TxInput, TxOutput, sighash
- `ledger.rs` — `apply_block`/`apply_dag`, per-block state, snapshots, finality pruning
- `multisig.rs` — RFC-001 M-of-N P2SH
- `script_v2.rs` — RFC-003 bounded stack machine
- `htlc.rs` — RFC-004 HTLC template (100 bytes)
- `vault.rs` — RFC-005 time-lock vault (40 bytes)
- `stake.rs` — `[TARGET]`-retiring with hybrid

### `kovanica-node` — Runnable Node
- `node.rs` — Ledger + Mempool, genesis, produce, gossip, multi-input transfers
- `mempool_v2.rs` — Orphan pool, fee-based eviction, capacity limits
- `p2p.rs` / `p2p_hardening.rs` — Mesh, discovery, rate limits, peer scoring
- `dht.rs` / `dns_seed.rs` — Kademlia DHT + DNS multi-seed resolver
- `explorer.rs` — Self-hosted explorer (JSON API + WebSocket + static UI)
- `spv.rs` — SPV light client wire sync + proof verification

### `kovanica-ffi` — Mobile Bindings
- `LightNode` — `Mutex<Node>` wrapper, full surface: genesis, validator, bond, produce, transfer, blob sync, SPV, history
- **Bindings committed** — regenerate after UDL/Rust changes:
  ```bash
  cargo build --release -p kovanica-ffi
  cargo run -p kovanica-ffi --bin uniffi-bindgen -- generate \
    --library target/release/libkovanica_ffi.so \
    --language kotlin --out-dir crates/kovanica-ffi/bindings/kotlin
  cargo run -p kovanica-ffi --bin uniffi-bindgen -- generate \
    --library target/release/libkovanica_ffi.so \
    --language swift --out-dir crates/kovanica-ffi/bindings/swift
  ```

---

## Testing

```bash
# All tests
cargo test

# Specific test suites
cargo test -p kovanica-dag adversarial_wide_fork
cargo test -p kovanica-state tokenomics
cargo test -p kovanica-node network

# FFI tests (offline)
cargo test -p kovanica-ffi
```

---

## Documentation

| Document | Path |
|----------|------|
| **Agent Conventions** | [AGENTS.md](AGENTS.md) |
| **RFC Index** | [docs/](docs/) |
| **RFC-006 Tokenomics** | [docs/RFC-006-EmissionCurve.md](docs/RFC-006-EmissionCurve.md) |
| **PoA Migration** | [docs/RFC-POA-Migration.md](docs/RFC-POA-Migration.md) |
| **LEGIT-BOARD** | [docs/LEGIT-BOARD.md](docs/LEGIT-BOARD.md) |
| **Testnet Params** | [TESTNET-RFC006.md](TESTNET-RFC006.md) |
| **Operations** | [OPERATIONS.md](OPERATIONS.md) |

---

## Related Repositories (in the meta-monorepo)

| Component | Directory | Purpose |
|-----------|-----------|---------|
| Node Binary | `../node/` | Thin wrapper building `kovanica-node` |
| Web Frontend | `../web/` | Explorer + wallet + map |
| Mobile Wallet | `../wallet/` | Android/iOS + browser extension |
| Light Node Mobile | `../mobile/` | FFI-based light clients |
| Android Light Node | `../android-light-node/` | Active Android light node |
| CLI Wallet | `../cli/` | Command-line wallet |
| SDK | `../sdk/` | Rust/WASM SDK |
| Installer | `../installer/` | One-click installers |
| Hardware Wallet | `../ledger-app/` | Ledger/Trezor support |

---

## License

**MIT OR Apache-2.0** — See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

---

## Security

- Private keys and seeds **never** leave the client
- Node policy never weakens consensus determinism
- Report vulnerabilities: GitHub Security Advisories or `security@kovanica.online`