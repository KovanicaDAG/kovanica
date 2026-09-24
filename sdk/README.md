# kovanica-sdk

Official Rust (+ WASM) SDK for the **Kovanica** protocol.

**Status:** `0.1.0-alpha.1` — pre-mainnet scaffolding.  
**Address (node-canonical):** `kvnc` + base58(`[version]‖payload32`) + `dag` — **not** bech32.  
P2PK (`version=0x00`) encode/decode is implemented; versions `0x00…0x05` fully
accepted on parse (node parity), legacy 64-hex + 32-byte `kvnc…dag` → P2PK.  
**Sighash / wire format:** BLAKE3 over witness-free encoding → 32 bytes; Ed25519
signs that hash (`verify_strict`). `encode_into` is a **byte-identical port** of
`kovanica-state::tx`, pinned by shared test vectors
(`protocol/crates/kovanica-state/tests/sighash_vector.rs` ↔ `crates/kovanica-types/tests/sighash_vector.rs`).
`Transaction::decode` is the matching inverse (witness included, trailing-bytes /
truncation safe).  
**Scripts (RFC-001/004/005):** multisig (M-of-N P2SH), HTLC, vault templates are
byte-format ports of `kovanica-state` (`multisig.rs`/`htlc.rs`/`vault.rs`) with
matching validation rules; addresses pinned by
`protocol/crates/kovanica-state/tests/script_vectors.rs` ↔
`crates/kovanica-keys/tests/script_vectors.rs`.  
**Builders:** `TransferBuilder`, `HtlcBuilder`, `VaultBuilder` (funding txs),
`MultisigSigner` (offline M-of-N signature shares → spend witness), `SignedTx`.  
**RPC:** routes match the node (`GET /api/utxos`, `GET /api/fee_estimate`,
`POST /api/submit_tx` with `{"tx_hex": …}`).  
See `../roadmap-pack/ADDRESS-AND-SIGHASH-SPEC.md` for the original draft.

**Docs:** [`COOKBOOK.md`](COOKBOOK.md) — practical recipes (build → estimate →
sign → submit, multi-asset, HTLC, vaults, multisig, live API, safety).
[`RELEASE.md`](RELEASE.md) — crates.io/npm publish runbook (S-12),
go/no-go gate, rollback policy.

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
  htlc_swap.rs
  create_asset.rs
  transfer_asset.rs
```

## Quick start (Rust)

```bash
# from this directory
cargo test --workspace
cargo run -p kovanica-sdk --example generate_wallet
```

## Examples

All examples are offline (no keys leave the process) unless noted:

| Example | Shows |
|---|---|
| `generate_wallet` | Mnemonic → keypair → address |
| `htlc_swap` | RFC-004 atomic swap: lock / redeem / refund |
| `transfer_asset` | KVP-102 multi-asset transfer with per-asset change + conservation checks |
| `create_asset` | Client-side asset identity derivation; live per-asset balance listing when `KOVANICA_API` is set |

```bash
cargo run -p kovanica-sdk --example transfer_asset
KOVANICA_API=https://api.kovanica.online cargo run -p kovanica-sdk --example create_asset
```

## Example: KVP-102 multi-asset transfer

```rust
use kovanica_sdk::prelude::*;

let asset = AssetId(Hash32(*blake3::hash(b"my-token").as_bytes()));
let tx = TransferBuilder::new()
    .network(NetworkId::Testnet)
    .add_input(asset_utxo)                 // Utxo { asset_id: asset, .. }
    .add_output(bob, Amount::from_atoms(1200), asset)
    .set_change(alice.address())
    .build()?;                             // change returned in the same asset
```

Non-native assets must balance exactly (KVP-102 conservation); fees only ever
apply to native KVNC. `UtxoItem::into_domain(&address)` converts a node
`/api/utxos` row into a builder-ready `Utxo`.

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

Read-only integration tests against the public API, gated behind the
`live-testnet` feature (off by default — `cargo test --workspace` stays offline):

```bash
cargo test -p kovanica-rpc --features live-testnet -- --nocapture
```

What they verify (checked against a live `/api/head` + `/api/bootstrap` before
being written):

- `GET /api/head` — network identity, `atom = 1e8`, RFC-006 fee floor
  (`min_fee = max(1, subsidy/500_000)` atoms/byte; 2000 in era 0).
- `GET /api/bootstrap` — GHOSTDAG `k = 3`, `token = "KVNC"`,
  `max_supply = 9_020_000_000_000_000` (90.2M KVNC at 1e8 atoms),
  supply invariants (`native_minted ≤ max_supply`, `total == native_minted`,
  `circulating ≤ total`), and P2P seed policy (DNS seed names on TCP 9000,
  never an orange-cloud explorer host).
- `GET /api/utxos` — fresh address returns an empty, zero-balance page.
- `GET /api/fee_estimate` — shape and unit (`atoms/byte`).
- `POST /api/submit_tx` — a deliberately empty transaction must be rejected.
  No keys, no funded spends, no faucet usage.

You can point the tests at any node/explorer (e.g. your local participant node
on `127.0.0.1:8080`):

```bash
KOVANICA_API=http://127.0.0.1:8080 \
  cargo test -p kovanica-rpc --features live-testnet -- --nocapture
```

`KOVANICA_API` defaults to `https://api.kovanica.online`.

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
