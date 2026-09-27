# Kovanica Protocol — Technical Backlog

**Playground · BIP-39 backup · SDK**  
**Date:** September 2026 · Pre-Mainnet  
**Revised:** 2026-09-23 (consistency pass; monorepo import)

---

## 1. Overview & Priority

This backlog covers the highest-leverage developer-experience items for the path to mainnet and the first 90 days after. All work stays on the **0.x** version line until the protocol reaches a stable **1.0.0** release.

### Priority Matrix

| Workstream              | Priority      | Target           | Depends On                    | Status      |
|-------------------------|---------------|------------------|-------------------------------|-------------|
| Address + sighash lock  | P0 — Critical | Before broadcast | Node vectors + encode_into    | Partial     |
| 12/24 BIP-39 backup     | P0 — Critical | Before mainnet   | Address/derivation lock       | **Done**    |
| kovanica-sdk scaffold   | P0            | Pre-mainnet      | —                             | **Done**    |
| kovanica-sdk core       | P0 — Critical | Mainnet + 30d    | Address lock, stable RPC      | In progress |
| Playground              | P1 — High     | Mainnet + 60d    | SDK + faucet + testnet        | Todo        |

**Canonical address:** `kvnc` + base58(versioned bytes) + `dag` (NOT bech32).  
P2PK encode/decode implemented in SDK; see `ADDRESS-AND-SIGHASH-SPEC.md`.

---

## 2. BIP-39 Backup Support (12 / 24 words)

### Requirements

- BIP-39 English wordlist (2048 words) — first language
- User-selectable entropy: 128-bit (12 words) or 256-bit (24 words)
- Optional BIP-39 passphrase ("25th word")
- Checksum validation on import; clear, non-technical error messages
- Secure generation: WebCrypto (browser) / getrandom (CLI)
- Key material never leaves the client; never logged or sent to any node
- Deterministic derivation paths compatible with existing Ed25519 key scheme
- **Addresses displayed must use the same `kvnc…` encoding as the node**

### Implementation Notes (Rust)

- Prefer `kovanica-keys` from the SDK workspace (already uses `bip39`)
- CLI: `kovanica-cli wallet new --words 12|24 [--passphrase]`
- Web wallet: toggle + confirm screen that forces user to write the backup down
- Unit tests for known vectors + property tests for checksum
- Version: ship in 0.x line; freeze derivation once mainnet is live

### Acceptance Criteria

- User can generate, display, confirm, and restore both 12- and 24-word backups
- Wrong checksum is rejected with helpful message
- Same backup produces identical addresses on CLI and web wallet **and matches node**
- No key material appears in network traces or logs

---

## 3. kovanica-sdk

### Scope

- Rust library: core types, transaction builders, signing, RPC client
- TypeScript + WASM package for browser and Node.js
- High-level helpers that map 1:1 to KVP standards (102 assets, 104 HTLC, 101 multisig, 105 vaults)
- Typed client for every public `/api` endpoint
- Examples and cookbook published on docs.kovanica.online

### Key modules (crate names use hyphens; Rust paths use underscores)

- `kovanica-types` — Block, Tx, UTXO, AssetId, Address, Signature
- `kovanica-keys` — Backup phrase → derived key → Ed25519 keypair + address
- `kovanica-tx` — Builders for native transfer, asset mint/transfer, HTLC, multisig, vault
- `kovanica-rpc` — Async HTTP client matching explorer / node API
- `kovanica-fee` — Fee estimation helpers (size-based + current subsidy)

### Current skeleton status (`sdk/`, monorepo)

| Crate            | State                                      |
|------------------|--------------------------------------------|
| Workspace / S-01 | **Done**                                   |
| `kovanica-keys`  | **Done**: SLIP-0010 frozen path, `kvnc…dag` P2PK codec, known-answer vectors |
| `kovanica-fee`   | **Done**: fee-floor-aware estimate (`estimate_with_min`) |
| types / rpc / tx stubs | Present (partial)                    |
| Sighash          | BLAKE3 algorithm locked — `encode_into` parity pending (S-03c) |
| Examples         | `generate_wallet.rs` only (S-10 remaining) |

### Versioning & Distribution

- crates.io + npm under `@kovanica/*` namespace
- Strict SemVer: 0.x until protocol 1.0.0; then major aligned with node
- Reproducible builds + `cargo-auditable` for the Rust crate

---

## 4. Playground / Console

### Goal

Zero-install environment to experiment with Kovanica: create assets, build HTLCs, simulate fees, deploy to testnet, copy production-ready SDK snippets.

### Feature List (MVP → v1)

- Hosted at `playground.kovanica.online` (or `console.kovanica.online`)
- Default network: testnet; later mainnet read-only or gated
- In-browser key generation (WebCrypto) — keys never leave the tab
- Panels: Asset Creator (KVP-102), HTLC, Multisig, Vault, Fee Simulator
- One-click "Send to testnet" with faucet
- Shareable state; "Copy as SDK call"
- Desktop-first, mobile-usable

### Architecture Sketch

- Frontend: React / Svelte + WASM bindings from `kovanica-sdk`
- No backend for key material; optional lightweight backend for short-codes / faucet proxy
- CSP hardened against accidental key leakage

---

## 5. Suggested Work Order

1. **ASAP:** Finish S-03b/S-03c parity (node test vectors + exact `encode_into`)  
2. **Done:** BIP-39 backup in CLI + web wallet (M-01 → M-11)  
3. **Week 2–5:** Finish SDK Rust core (S-02 → S-08)  
4. **Week 4–8:** WASM, examples, Playground MVP  
5. **Post-mainnet:** publish, grants, mainnet toggle  

---

*Living backlog. Consistency details: `CONSISTENCY-FIXES.md`.*