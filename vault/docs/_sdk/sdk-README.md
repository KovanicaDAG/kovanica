# kovanica-sdk

> **Official Rust (+ WASM) SDK for the Kovanica Protocol** — Types, keys, transaction builders, RPC client, and fee estimation.

---

## Status

**`0.1.0-alpha.1`** — Pre-mainnet scaffolding.

---

## Canonical Specifications

| Spec | Value |
|------|-------|
| **Address Format** | `kvnc` + base58(`[version]‖payload32`) + `dag` — **not bech32** |
| **P2PK** | Version `0x00` (encode/decode implemented) |
| **Versions Accepted** | `0x00…0x05` (node parity), legacy 64-hex + 32-byte `kvnc…dag` → P2PK |
| **Sighash** | BLAKE3 over witness-free encoding → 32 bytes; Ed25519 signs that hash (`verify_strict`) |
| **Encoding** | `encode_into` = **byte-identical port** of `kovanica-state::tx` (pinned by shared test vectors) |
| **Decoding** | `Transaction::decode` = matching inverse (witness included, trailing-bytes/truncation safe) |

---

## Workspace Layout

```
kovanica-sdk/
├── crates/
│   ├── kovanica-types/   # Amount, Address, UTXO, Transaction, AssetId, …
│   ├── kovanica-keys/    # BIP-39 mnemonic (12/24), Ed25519 keypairs
│   ├── kovanica-tx/      # TransferBuilder, HtlcBuilder, VaultBuilder, MultisigSigner
│   ├── kovanica-rpc/     # HTTP client for /api/*
│   ├── kovanica-fee/     # Tokenomics-aware fee estimation
│   └── kovanica-sdk/     # Facade re-exports + prelude
├── bindings/
│   └── kovanica-wasm/    # wasm-bindgen surface for browsers
├── examples/
│   ├── generate_wallet.rs
│   ├── htlc_swap.rs
│   ├── create_asset.rs
│   └── transfer_asset.rs
├── COOKBOOK.md           # Practical recipes
├── RELEASE.md            # crates.io/npm publish runbook
└── Cargo.toml
```

---

## Quick Start (Rust)

```bash
# From this directory
cargo test --workspace
cargo run -p kovanica-sdk --example generate_wallet
```

---

## Examples

All examples are **offline** (no keys leave the process) unless noted:

| Example | Shows |
|---------|-------|
| `generate_wallet` | Mnemonic → keypair → address |
| `htlc_swap` | RFC-004 atomic swap: lock / redeem / refund |
| `transfer_asset` | KVP-102 multi-asset transfer with per-asset change + conservation |
| `create_asset` | Client-side asset identity derivation; live per-asset balance listing when `KOVANICA_API` set |

```bash
cargo run -p kovanica-sdk --example transfer_asset
KOVANICA_API=https://api.kovanica.online cargo run -p kovanica-sdk --example create_asset
```

---

## Example: KVP-102 Multi-Asset Transfer

```rust
use kovanica_sdk::prelude::*;

let asset = AssetId(Hash32(*blake3::hash(b"my-token").as_bytes()));
let tx = TransferBuilder::new()
    .network(NetworkId::Testnet)
    .add_input(asset_utxo)                 // Utxo { asset_id: asset, .. }
    .add_output(bob, Amount::from_atoms(1200), asset)
    .set_change(alice.address())
    .build()?;                             // Change returned in same asset
```

- **Non-native assets** must balance exactly (KVP-102 conservation)
- **Fees** only ever apply to native KVNC
- `UtxoItem::into_domain(&address)` converts node `/api/utxos` row → builder-ready `Utxo`

---

## Example: Generate Wallet

```rust
use kovanica_sdk::prelude::*;

let mnemonic = Mnemonic::generate(WordCount::Words24)?;
let keypair = Keypair::from_mnemonic(&mnemonic, "");
println!("{}", keypair.address());

// Optional live call (requires network)
// let client = Client::testnet()?;
// let head = client.get_head().await?;
```

---

## Live Testnet Tests

Read-only integration tests against the public API, gated behind `live-testnet` feature (off by default — `cargo test --workspace` stays offline):

```bash
cargo test -p kovanica-rpc --features live-testnet -- --nocapture
```

**Point at any node/explorer** (e.g., your local participant node):

```bash
KOVANICA_API=http://127.0.0.1:8080 \
  cargo test -p kovanica-rpc --features live-testnet -- --nocapture
```

### What They Verify (Checked Against Live `/api/head` + `/api/bootstrap`)

- `GET /api/head` — network identity, `atom = 1e8`, RFC-006 fee floor (`min_fee = max(1, subsidy/500_000)` atoms/byte; 2000 in era 0)
- `GET /api/bootstrap` — GHOSTDAG `k = 3`, `token = "KVNC"`, `max_supply = 9_020_000_000_000_000` (90.2M KVNC), supply invariants, P2P seed policy (DNS seed names on TCP 9000, never orange-cloud explorer host)
- `GET /api/utxos` — Fresh address returns empty, zero-balance page
- `GET /api/fee_estimate` — Shape and unit (`atoms/byte`)
- `POST /api/submit_tx` — Deliberately empty transaction rejected

---

## WASM Build

```bash
# Requires wasm-pack
wasm-pack build bindings/kovanica-wasm --target web
```

---

## Versioning

- All crates: `0.x` until protocol `1.0.0`
- Breaking changes allowed on minor bumps while in 0.x
- Address format + derivation path will be locked before mainnet genesis

---

## Security

- Seeds and private keys are **zeroized on drop**
- `Debug` impls redact secrets
- **Never** log or transmit mnemonic/seed material
- Prefer not to persist the seed; if you must, encrypt with a user password

---

## Related Documentation

| Document | Purpose |
|----------|---------|
| [COOKBOOK.md](COOKBOOK.md) | Practical recipes (build → estimate → sign → submit, multi-asset, HTLC, vaults, multisig, live API, safety) |
| [RELEASE.md](RELEASE.md) | crates.io/npm publish runbook (S-12), go/no-go gate, rollback policy |
| [RFC-006 Tokenomics](https://github.com/KovanicaDAG/kovanica-protocol/blob/main/docs/RFC-006-EmissionCurve.md) | Emission curve, supply cap, maturity, fee burn |
| [KVP-101…105](https://github.com/KovanicaDAG/kovanica-protocol/tree/main/docs) | Multisig, multi-asset, stealth, HTLC, vault specs |

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger (source of truth) |
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Node binary (serves API) |
| [kovanica-cli](https://github.com/KovanicaDAG/kovanica-cli) | CLI wallet (uses same `kovanica-state` crate) |
| [kovanica-web](https://github.com/KovanicaDAG/kovanica-web) | Web wallet/explorer (uses WASM bindings) |

---

## License

**MIT OR Apache-2.0**