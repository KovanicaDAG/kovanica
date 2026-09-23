# kovanica-sdk

Official Rust (+ WASM) SDK for the **Kovanica** protocol.

**Status:** `0.1.0-alpha.1` — pre-mainnet scaffolding.  
**Address (node-canonical):** `kvnc` + base58(`[version]‖payload32`) + `dag` — **not** bech32.  
P2PK (`version=0x00`) encode/decode is implemented; versions `0x00…0x05` fully
accepted on parse (node parity), legacy 64-hex + 32-byte `kvnc…dag` → P2PK.  
**Sighash / wire format:** BLAKE3 over witness-free encoding → 32 bytes; Ed25519
signs that hash (`verify_strict`). `encode_into` is a **byte-identical port** of
`kovanica-state::tx`, pinned by shared test vectors
(`node/crates/kovanica-state/tests/sighash_vector.rs` ↔ `crates/kovanica-types/tests/sighash_vector.rs`).
`Transaction::decode` is the matching inverse (witness included, trailing-bytes /
truncation safe).  
**Scripts (RFC-001/004/005):** multisig (M-of-N P2SH), HTLC, vault templates are
byte-format ports of `kovanica-state` (`multisig.rs`/`htlc.rs`/`vault.rs`) with
matching validation rules; addresses pinned by
`node/crates/kovanica-state/tests/script_vectors.rs` ↔
`crates/kovanica-keys/tests/script_vectors.rs`.  
**Builders:** `TransferBuilder`, `HtlcBuilder`, `VaultBuilder` (funding txs),
`MultisigSigner` (offline M-of-N signature shares → spend witness), `SignedTx`.  
**RPC:** routes match the node (`GET /api/utxos`, `GET /api/fee_estimate`,
`POST /api/submit_tx` with `{"tx_hex": …}`).  
See `../roadmap-pack/ADDRESS-AND-SIGHASH-SPEC.md` for the original draft.

## Workspace layout

```
crates/
  kovanica-types/   # Amount, Address, UTXO, Transaction, …
  kovanica-keys/    # BIP-39 mnemonic (12/24), Ed25519 keypairs
  kovanica-tx/      # TransferBuilder, HtlcBuilder, VaultBuilder, MultisigSigner
  kovanica-rpc/     # HTTP client for /api/*
  kovanica-fee/     # Tokenomics-aware fee estimation
  kovanica-sdk/     # Facade re-exports + prelude
bindings/
  kovanica-wasm/    # wasm-bindgen surface for browsers
examples/
  generate_wallet.rs
```

## Quick start (Rust)

```bash
# from this directory
cargo test --workspace
cargo run -p kovanica-sdk --example generate_wallet
```

```rust
use kovanica_sdk::prelude::*;

let mnemonic = Mnemonic::generate(WordCount::Words24)?;
let keypair = Keypair::from_mnemonic(&mnemonic, "");
println!("{}", keypair.address());

// Optional live call (requires network)
// let client = Client::testnet()?;
// let head = client.get_head().await?;
```

## Live testnet tests

```bash
cargo test -p kovanica-rpc --features live-testnet -- --nocapture
```

## WASM

```bash
# requires wasm-pack
wasm-pack build bindings/kovanica-wasm --target web
```

## Versioning

- All crates: `0.x` until protocol `1.0.0`
- Breaking changes allowed on minor bumps while in 0.x
- Address format + derivation path will be locked before mainnet genesis

## Security

- Seeds and private keys are zeroized on drop
- Debug impls redact secrets
- Never log or transmit mnemonic / seed material
- Prefer not to persist the seed; if you must, encrypt with a user password

## Related docs

- Master Roadmap & Task Breakdown (planning pack)
- RFC-006 Tokenomics
- KVP-101 … KVP-105 (multisig, multi-asset, stealth, HTLC, vault)

## License

MIT OR Apache-2.0
