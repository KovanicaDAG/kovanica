# Kovanica Protocol — kovanica-sdk Crate Structure Draft

**Workspace layout · Modules · Public API sketch**  
**Date:** September 2026 · Version 0.1.0-alpha.1 · Pre-Mainnet  
**Address HRP:** `kvnc` (base58 + `dag` suffix, node format — NOT bech32; see `ADDRESS-AND-SIGHASH-SPEC.md`).

---

## 1. Design Goals

- One workspace that produces both native Rust library and WASM/TS bindings
- Zero network by default in tests; optional `live-testnet` feature
- Types and builders mirror on-chain / KVP semantics exactly
- Private key material stays client-side; the SDK never asks for it over RPC
- Semantic versioning `0.x` until protocol `1.0.0`

---

## 2. Suggested Workspace Layout

```
sdk/                                 # Cargo workspace root (monorepo)
├── Cargo.toml                       # workspace members
├── README.md
├── crates/
│   ├── kovanica-types/              # [draft] pure data types + serde
│   ├── kovanica-keys/               # BIP-39 phrase, derivation, signing
│   ├── kovanica-tx/                 # [draft] transaction builders
│   ├── kovanica-rpc/                # [draft] HTTP client for /api/*
│   ├── kovanica-fee/                # fee estimation helpers
│   └── kovanica-sdk/                # facade crate (re-exports)
├── bindings/
│   └── kovanica-wasm/               # wasm-bindgen target (pkg ignored, built on publish)
├── examples/
│   ├── generate_wallet.rs           # shipped with skeleton
│   ├── transfer.rs                  # S-10
│   ├── create_asset.rs              # S-10
│   └── htlc_swap.rs                 # S-10
└── tests/
```

> Current monorepo reality: `kovanica-keys` + `kovanica-fee` implemented and
> tested; `types`/`tx`/`rpc` exist as stubs in the original skeleton.

---

## 3. Crate Responsibilities

### 3.1 kovanica-types *(stub)*

- `Address`, `PublicKey`, `Signature`, `TxHash`, `BlockHash`, `AssetId`
- `Transaction`, `Input`, `Output`, `Coinbase`, `UTXO`
- `NetworkId` (testnet / mainnet), `Amount` (atoms)
- Serde implementations; **no** crypto or network code

### 3.2 kovanica-keys *(implemented)*

- BIP-39 English phrase generation for 12 or 24 words (`WordCount::Words12 | Words24`)
- Phrase parse + optional passphrase ("25th word")
- `Keypair::from_seed` / `from_mnemonic` / `from_mnemonic_at(index)` — see the crate
- `address()` — `kvnc` + base58(0x00 ‖ pubkey) + `dag`, matching the node
- **Frozen derivation:** SLIP-0010 ed25519 `m/44'/3007'/0'/0'/i'` (all hardened) — `DERIVATION.md`
- `sign(message) → Signature`; `verify` (strict)
- Known-answer vectors: `crates/kovanica-keys/tests/slip10_vectors.rs`
  (verified against the official SLIP-0010 test vectors)

### 3.3 kovanica-tx *(stub)*

- `TransferBuilder` — native KVNC
- `AssetMintBuilder` / `AssetTransferBuilder` (KVP-102)
- `HtlcBuilder` (KVP-104) — lock, redeem, refund
- `MultisigBuilder` (KVP-101)
- `VaultBuilder` (KVP-105)
- All builders produce unsigned `Tx` → sign with `Keypair` → `SignedTx`

### 3.4 kovanica-rpc *(stub)*

- `Client::new(base_url)` — default `https://api.kovanica.online`
- `get_head()`, `get_block`, `get_utxos`, `submit_tx`, `estimate_fee`
- Typed responses matching current explorer/node JSON
- Feature flag `live-testnet` for integration tests

### 3.5 kovanica-fee *(implemented)*

- `estimate(size_bytes, subsidy_atoms)` → size-based fee
- `estimate_with_min(size_bytes, subsidy_atoms, min_fee_atoms)` → `max(size-based, floor)`
- Fee floor from RFC-006 tokenomics: `max(1, subsidy/500_000)` atoms/byte

### 3.6 kovanica-sdk (facade)

```rust
pub use kovanica_keys::*;
pub use kovanica_fee::*;
// kovanica_types / kovanica_tx / kovanica_rpc re-exported once stubs land
```

Single entry point for application developers.

---

## 4. Public API Sketch (selected)

### Backup phrase & keys

```rust
let phrase = Phrase::generate(WordCount::Words24)?;
let kp = Keypair::from_mnemonic(&phrase, "optional-passphrase");
let addr = kp.address(); // deterministic, kvnc…dag
```

### Transfer *(S-04, draft)*

```rust
let tx = TransferBuilder::new()
    .add_input(utxo)
    .add_output(recipient, amount)
    .set_change(change_addr)
    .set_fee(fee)
    .build()?;
let signed = tx.sign(&kp)?;
```

### RPC *(S-07, draft)*

```rust
let client = Client::testnet(); // or Client::mainnet()
let head = client.get_head().await?;
client.submit_tx(&signed).await?;
```

---

## 5. Versioning & Publish

- All crates start at `0.1.0-alpha`
- crates.io: `kovanica-sdk`, `kovanica-types`, …
- npm: `@kovanica/sdk` (from the wasm package)
- Lock derivation path and address encoding before `1.0.0` — **derivation already frozen**
- Reproducible builds + `cargo-auditable` recommended

---

## 6. First Milestones

| Milestone | Scope                                              | Status |
|-----------|----------------------------------------------------|--------|
| M1        | types + keys + transfer builder + unit tests       | keys **done**; types/tx partial |
| M2        | RPC client + fee + live-testnet example            | fee **done**; rpc partial |
| M3        | HTLC + multisig + asset builders                   | Todo   |
| M4        | WASM bindings + TypeScript typings + cookbook      | Partial (wasm binding; pkg regenerated) |

---

*This draft is intentionally minimal and can be expanded once the first transfer example works end-to-end on testnet.*