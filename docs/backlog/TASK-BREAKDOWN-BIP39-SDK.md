# Kovanica Protocol — Detailed Task Breakdown

**BIP-39 Backup · kovanica-sdk**  
**Date:** September 2026 · Pre-Mainnet · Sprint-ready  
**Revised:** 2026-09-23 (consistency pass; monorepo import)

---

## 1. Scope & Rules

All work ships on the **0.x** line. No breaking derivation or address format after mainnet genesis. Key material stays purely client-side; the SDK never receives raw keys over the wire.

- Versioning: crates stay `0.y.z` until protocol `1.0.0`
- Security: key material never logged, never sent to node/API
- Tests: known BIP-39 vectors + property tests mandatory
- Docs: every public API has a doc comment + cookbook entry
- **Address:** `kvnc` + base58(version‖payload) + `dag` (node format; NOT bech32).
- **Sighash:** BLAKE3(witness-free encoding) → 32B; Ed25519 over that hash (`verify_strict`).

---

## 2. Workstream A — 12 / 24-word BIP-39 backup

### 2.1 Goals

- User can generate 12-word (128-bit) or 24-word (256-bit) BIP-39 backup
- Optional passphrase (25th word) supported
- Identical derivation in CLI and web wallet
- Clear UX that forces the user to write the backup down
- Addresses produced must match node encoding (`kvnc…`)

### 2.2 Task List

| ID   | Task                                                      | Est.  | Owner | Status |
|------|-----------------------------------------------------------|-------|-------|--------|
| M-01 | Choose bip39 crate + pin version (RustCrypto preferred)   | 0.5d  | Core  | **Done**   |
| M-02 | CLI: `wallet new --words 12\|24 [--passphrase]`           | 1d    | Core  | **Done**   |
| M-03 | CLI: `wallet restore` (interactive + file + phrase flag)  | 1d    | Core  | **Done**   |
| M-04 | CLI: show address / export public key (no key print by default)| 0.5d | Core | **Done** |
| M-05 | Web: generate flow with 12/24 toggle + confirm screens    | 2d    | Web   | In progress (web branch) |
| M-06 | Web: restore flow with paste / word-by-word + checksum UX | 1.5d  | Web   | In progress (web branch) |
| M-07 | Shared derivation path constants (document & freeze)      | 0.5d  | Core  | **Done** — `m/44'/917'/0'/0'/i'`, `DERIVATION.md` |
| M-08 | Unit tests: known vectors + checksum failure cases        | 1d    | Core  | **Done** — `slip10_vectors.rs` |
| M-09 | Property tests: round-trip generate → restore → same keys | 0.5d  | Core  | **Done** |
| M-10 | Security review: no key material in logs, network, analytics | 0.5d | Core | **Done** — see review summary |
| M-11 | Docs: user guide + developer note                         | 0.5d  | Docs  | **Done** — `DERIVATION.md` + backlog |

**Rough total:** ~9.5 developer-days  
**Depends on:** S-03b (address codec) for M-04 / web address display to be consensus-correct.

### 2.3 Acceptance Criteria

- Same backup + passphrase produces identical Ed25519 keypair on CLI and web
- Invalid checksum is rejected with non-technical message
- User cannot skip the "I wrote it down" confirmation on first generation
- No key material appears in browser console, network tab, or server logs
- CLI help and man-page updated
- Displayed addresses use the same `kvnc…` encoding as the node

### 2.4 Implementation Notes

- Crate: `bip39 = "2"` (RustCrypto) — pinned in SDK and CLI
- Entropy: 16 bytes → 12 words, 32 bytes → 24 words
- Derivation: SLIP-0010 ed25519 `m/44'/917'/0'/0'/i'` (all hardened), documented and **frozen** — `docs/backlog/DERIVATION.md`
- Web: use `WebCrypto.getRandomValues`; never `Math.random`
- Storage: prefer not to persist the backup at all; if needed, encrypt with user password
- Prefer reusing `kovanica-keys` from the SDK rather than duplicating BIP-39 logic in the wallet
- CLI keeps its own copy of the derivation (workspace isolation — `cli/` cannot depend on `sdk/`)
- `KeyPair::public_key()` added to `kovanica-state` keys.rs (both protocol + node copies) for watch-only export

### 2.5 M-10 Security Review Summary (Done, 2026-09-23)

Scope: key material handling in CLI wallet, SDK keys crate, and WASM binding.

Findings — all pass:

- **No logging of key material.** No `log::`/`dbg!` calls touch seeds or phrases. The only prints are user-facing and intentional: `wallet new` shows the phrase once on the terminal (required UX), `wallet show --show-seed` prints the seed hex only when explicitly requested.
- **File permissions:** key files written 0600 (owner-only) on unix; `save` and `save_mnemonic` refuse to overwrite without `--force`.
- **Passphrase never persisted.** The optional passphrase exists only in memory for derivation; files store the phrase/seed only.
- **No network exposure.** The node never receives key material; the tx flow is prepare → offline sign → submit (signature only). WASM binding exposes only `address_from_mnemonic(phrase, index)` which returns an address and never transmits the phrase.
- **Zeroize:** seeds remain in process memory for wallet lifetime (accepted: local wallet, not shared memory); the earlier `key.zeroize(); Ok(key)` bug pattern was removed.

No blocking findings. Residual risk is end-user hygiene (terminal scrollback after printing a phrase) — documented in the user guide.

---

## 3. Workstream B — kovanica-sdk

### 3.1 Goals

- Single Rust workspace + TypeScript/WASM package that any developer can use
- Cover core types, keys, tx builders, RPC client, fee estimation
- 1:1 mapping to existing KVP standards (101–105 + multi-asset)
- Ship examples that compile and run against testnet **with consensus-valid addresses/signatures**

### 3.2 Task List

| ID    | Task                                                       | Est.  | Owner | Status      |
|-------|------------------------------------------------------------|-------|-------|-------------|
| S-01  | Cargo workspace layout + crate skeleton                    | 0.5d  | Core  | **Done**    |
| S-02  | `kovanica-types`: Block, Tx, UTXO, AssetId, Address, Sig   | 2d    | Core  | Partial     |
| S-03  | `kovanica-keys`: backup → seed → Ed25519 + address         | 1.5d  | Core  | **Done**    |
| S-03b | **Lock address codec to node** (`kvnc`+base58+`dag`)       | 1d    | Core  | **Done** (P2PK); stealth/HTLC/vault helpers pending |
| S-03c | **Lock sighash to node** (BLAKE3 witness-free)             | 1d    | Core  | Partial     |
| S-04  | `kovanica-tx`: native transfer + asset mint/transfer       | 2d    | Core  | Partial     |
| S-05  | `kovanica-tx`: HTLC (KVP-104) + Multisig (KVP-101)         | 2d    | Core  | Todo        |
| S-06  | `kovanica-tx`: Vault / CSV (KVP-105) builder               | 1d    | Core  | Todo        |
| S-07  | `kovanica-rpc`: typed HTTP client for `/api/*`             | 2d    | Core  | Partial     |
| S-08  | `kovanica-fee`: size-based + subsidy-aware fee estimate    | 1d    | Core  | **Done** — `estimate_with_min` honors RFC-006 floor |
| S-09  | WASM / TypeScript bindings                                 | 2.5d  | Core  | Partial (pkg regenerated) |
| S-10  | Examples: transfer, create-asset, htlc-swap (testnet)      | 1.5d  | Core  | Todo        |
| S-11  | Integration tests against live testnet (feature-gated)     | 1d    | Core  | Todo        |
| S-12  | crates.io + npm publish pipeline (0.x)                     | 1d    | Core  | Todo        |
| S-13  | Cookbook pages on docs.kovanica.online                     | 1.5d  | Docs  | Todo        |

**Notes on status**

- **S-01 Done:** `sdk/` workspace exists (keys, fee, sdk facade, wasm binding, `generate_wallet` example).
- **S-03 Done:** SLIP-0010 frozen derivation, `kvnc…dag` P2PK codec, known-answer vectors in `slip10_vectors.rs` (verified against the official SLIP-0010 test vectors).
- **Partial:** stubs compile but advanced builders return "not yet implemented".
- **S-03b / S-03c:** P2PK address encode/decode + BLAKE3 sighash algorithm in place. Still need byte-identical `encode_into` + node test vectors before production broadcast.

### 3.3 Acceptance Criteria

- `cargo test --workspace` passes with no network by default
- Optional `--features live-testnet` runs against public testnet
- TypeScript package can build a signed tx and broadcast it (**requires S-03b + S-03c**)
- Every public builder has at least one example in `/examples`
- README shows a 5-minute "first transfer" path
- Addresses round-trip with node (`kvnc…` encode/decode)

### 3.4 Suggested Dependencies

- `ed25519-dalek` / existing node crypto
- `bip39`, `sha2`, `hmac`, `pbkdf2`
- `serde` + `serde_json`, `thiserror`, `anyhow`
- `reqwest` (async) or `ureq` (sync) for RPC
- `wasm-bindgen` + `js-sys` for browser target
- `bs58`, `blake3` (aligned to node; **not** bech32)

---

## 4. Combined Sprint Suggestion

| Sprint   | Window     | Focus                                              |
|----------|------------|----------------------------------------------------|
| Sprint 0 | ASAP       | S-03b + S-03c (address + sighash lock to node)     |
| Sprint 1 | Week 1–2   | M-01 → M-11 (backup support complete) — **Done**   |
| Sprint 2 | Week 2–4   | S-02 → S-08 finish (Rust core against locked codec)|
| Sprint 3 | Week 4–6   | S-09 → S-13 + Playground MVP kickoff               |

**Total rough effort (one strong Rust engineer):** ~20–24 developer-days including address/sighash lock.

---

*Source of truth: monorepo `/docs/backlog/`. See also `CONSISTENCY-FIXES.md`.*