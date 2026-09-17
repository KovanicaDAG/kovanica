# Kovanica DAG — Obsidian Vault Overview

> **Authoritative source:** `/root/kovanica` (meta-directory / whole project)
> **Vault sync:** This note describes the *current* restructured layout. The old
> single-monorepo copy that used to live at `/root/kovanica-protocol` (and was
> embedded in this vault as `kovanica-protocol/`) is **obsolete** — the project
> was split via `git filter-repo` into independent sibling repos under
> `/root/kovanica`. Prefer this note and `/root/kovanica/AGENTS.md` when anything
> disagrees.

---

## What is Kovanica?

**Kovanica** is a **DAG-based distributed ledger protocol** — a high-throughput,
parallel-block cryptocurrency/ledger built on a **Directed Acyclic Graph**
(BlockDAG) rather than a single linear chain. The name *kovanica* is
Serbo-Croatian for "coin / mint."

- **Consensus:** GHOSTDAG (Sompolinsky, Wyborski & Zohar — the protocol behind Kaspa)
- **Blocks reference multiple parents** → parallel block production, later merged → high BPS
- **Core primitives:** DAG + GHOSTDAG ordering, UTXO ledger, ed25519 signatures,
  recursive linearization, finality-depth pruning, replay-log persistence,
  incremental reachability oracle (interval-tree + future-covering sets)
- **Hybrid admission:** PoW *and* VRF-staked block production (`H*work < 2^256`
  PoW path + stake-weighted VRF sortition path)
- **Activated RFCs (all on `main`, activation at blue score 0):**
  RFC-001 Multisig (M-of-N P2SH) · RFC-002 Native tokens (KVP-102) ·
  RFC-003 Stealth + Script v2 · RFC-004 HTLC / Atomic swaps · RFC-005 Vault / CSV

---

## Project Layout — `/root/kovanica` (meta-repo)

`/root/kovanica` is a **git meta-repo** (branch `master`, remote
`https://github.com/KovanicaDAG/kovanica.git`) that tracks the 6 core component
repos as **gitlinks** and the rest as plain files.

| Component | Type | Purpose | Remote |
|-----------|------|---------|--------|
| `kovanica-protocol/` | gitlink | **Rust workspace — source of truth** (dag/state/node/cli/ffi crates + RFCs + scripts/deploy) | KovanicaDAG/kovanica-protocol |
| `kovanica-node/` | gitlink | **Public release snapshot** for node operators (mirrors protocol crates on tags; not auto-synced) | KovanicaDAG/kovanica-node |
| `kovanica-wallet/` | gitlink | Wallet apps: `android/` (Kotlin/Compose), `ios/` (SwiftUI), `extension/` (browser ext), `shared/` (logo) | KovanicaDAG/kovanica-wallet |
| `kovanica-web/` | gitlink | Web app: `site/` (Vite 8 + TanStack Router + React 19 + Nitro SSR) + `deploy/` | KovanicaDAG/kovanica-web |
| `kovanica-mobile/` | gitlink | Light-node mobile clients: `android/` (ex-`android-light-node`), `ios/` placeholder | KovanicaDAG/kovanica-mobile |
| `kovanica-agent/` | gitlink | Python RAG engineering-agent service (FastAPI + LangGraph + Qdrant, sandboxed cargo) | KovanicaDAG/kovanica-agent |
| `kovanica-brain-vault/` | plain files | Knowledge base: numbered zettelkasten `00-Meta`…`05-Agentic-Operations`, RFCs, skills | (top-level repo) |
| `kovanica-installer/` | plain files | `install-kovanica.sh` + env/nginx/pm2/systemd templates | (top-level repo) |
| `kovanica-data/` | plain files | Placeholder — runtime data never versioned | — |
| `kovanica-ledger-app/` | **untracked** | Ledger hardware-wallet app: Rust `no_std` core + C SDK, APDU 0x01–0x05, BIP-44 coin type 11111 | (not yet pushed) |
| `trezor-coin-def/` | **untracked** | Trezor coin definition PR for KVNC (Ed25519, base58 `kvnc…dag`, 8 decimals) | (not yet pushed) |
| `TODO/` | untracked | `seed2-deploy.md` — pending seed2 VPS + testnet soak checklist | — |
| `AGENTS.md` | file | Operating guide for the monorepo (repo map, conventions, commands) — **entry point** | — |
| `NETWORK.md` | file | Network & domain architecture, testnet/mainnet config, seed hosts | — |
| `README.md` | file | Meta-directory index + getting-started | — |

> **Caveats found while mapping (2026-09-16):**
> - Root `README.md` still says "Not a git repository itself" — outdated, `.git` exists.
> - `kovanica-protocol` gitlink remote points at `KovanicaDAG/kovanica.git`
>   while README claims `kovanica-protocol` — remote name discrepancy to check.
> - `kovanica-node/kovanica-node/` is a **nested self-clone (~6 GB, includes
>   `target/` and a tagged zip)** — potential backup size hazard.
> - `kovanica-wallet/kovanica-wallet/` is a leftover README-only dir.
> - `kovanica-ledger-app` + `trezor-coin-def` are untracked/unpushed.

---

## Rust Workspace — `kovanica-protocol/crates/`

Workspace manifest: `kovanica-protocol/Cargo.toml` (resolver 2, v0.2.0, edition
2021, rust-version 1.75, `#![forbid(unsafe_code)]`).

| Crate | Role |
|-------|------|
| `kovanica-dag/` | BlockDAG + GHOSTDAG core: `block.rs`, `dag.rs`, `ghostdag.rs`, `ordering.rs`, `reachability.rs`, `pow.rs`, `difficulty.rs`, `vrf.rs`, `snapshot.rs`, `validation.rs` |
| `kovanica-state/` | UTXO ledger in GHOSTDAG order: `ledger.rs`, `utxo.rs`, `tx.rs`, `keys.rs`, `stake.rs`, `multisig.rs`, `htlc.rs`, `script_v2.rs`, `vault.rs`, `spv.rs`, `store.rs`, `validation.rs` |
| `kovanica-node/` | Runnable node + explorer: `node.rs`, `mempool.rs`, `mempool_v2.rs`, `net.rs`, `p2p.rs`, `p2p_hardening.rs`, `dht.rs`, `dns_seed.rs`, `relay.rs`, `spv.rs`, `rpc.rs`, `explorer.rs`, `metrics.rs`, `atomic_swap.rs`, `fuzz.rs` |
| `kovanica-cli/` | CLI wallet binary `kovanica`: `main.rs` (clap), `api.rs`, `wallet.rs` |
| `kovanica-ffi/` | UniFFI bindings: `light_node.rs`, `bin/uniffi-bindgen.rs`, committed `bindings/{kotlin,swift}`, `android/`, `build-android.sh` / `build-apple.sh` |

---

## Network & Testnet (from `NETWORK.md` + `AGENTS.md`)

- **Seed:** `seed.kovanica.online:9000` (Hostinger VPS)
- **Seed3:** `seed3.kovanica.online:9000` (AWS eu-north-1, live since 2026-08-24)
- **Seed2:** pending deployment — see `TODO/seed2-deploy.md`
- **Explorer:** `https://explorer.kovanica.online` · **Faucet:** `https://faucet.kovanica.online` (1 tKVNC, rate-limited)
- **Genesis:** `kovanica-testnet` — k=3, subsidy `200*ATOM`, founder seed=1
- **Hosts:** kovanica.online / testnet / mainnet / api / docs / faucet subdomains via Cloudflare

---

## Build & Test

| Repo | Build | Test | Lint/Format |
|------|-------|------|-------------|
| `kovanica-protocol/` | `cargo build` | `cargo test` | `cargo fmt --check && cargo clippy --all-targets` |
| `kovanica-node/` | `cargo build --workspace` | `cargo test --workspace` | same as protocol |
| `kovanica-web/site/` | `npm run build:vps` | `npm run test` | `npm run lint && npm run format:check` |
| `kovanica-wallet/android/` | `./gradlew assembleDebug` | `./gradlew test` | `ktlint` |

---

## Vault Snapshot Layout (this directory)

```
KovanicaDAG/
├── myObsidianVaultDAG.md      ← this file — project overview
├── NAVIGATION.md              ← vault navigation index
├── CODE_INDEX.md              ← topic → source file:// links
├── ROADMAP.md                 ← stage/RFC progress tracking
├── AGENTS.md                  ← synced from /root/kovanica/AGENTS.md
├── NETWORK.md                 ← synced from /root/kovanica/NETWORK.md
├── kovanica-protocol/         ← snapshot: top-level docs + docs/ (RFCs, KVP, plans)
├── kovanica-node/             ← snapshot: README, JOIN, TESTNET
├── kovanica-web/              ← snapshot: README + site/{README,DEPLOY,UI-PARITY-CHECKLIST}
├── kovanica-wallet/           ← snapshot: README
├── kovanica-mobile/           ← snapshot: README
└── kovanica-agent/            ← snapshot: README + docs/
```

The authoritative **knowledge base** for the ecosystem also exists separately at
`/root/kovanica/kovanica-brain-vault/` (numbered 00–05 + skills) — the agent
service keeps a nested copy at `kovanica-agent/kovanica-brain-vault/`.

---

## Authority Map

| Topic | Authoritative Source |
|-------|---------------------|
| Project layout & conventions | `/root/kovanica/AGENTS.md` |
| Protocol/code design, build, test | `/root/kovanica/kovanica-protocol` — read its `AGENTS.md` |
| Consensus features & activation | `kovanica-protocol/docs/RFC-*.md`, `KVP.md`, `KVP-102-NativeTokens.md` |
| Roadmap checklist (P0/P1/P2) | `kovanica-protocol/docs/LEGIT-BOARD.md` |
| Deployed testnet ops | `NETWORK.md`, `kovanica-protocol/OPERATIONS.md` / `TESTNET.md` |
| Mainnet readiness | `kovanica-protocol/docs/MAINNET-CRITERIA.md`, `TESTNET-SOAK.md` |
| Project overview as presented in Obsidian | `KovanicaDAG/myObsidianVaultDAG.md` |

---

## Working Notes

- **Do not invent** APIs, paths, commands, or roadmap items. If a fact isn't
  verifiable in `/root/kovanica` or these docs, say so.
- The vault snapshots under `KovanicaDAG/kovanica-*/` are **read-only mirrors**
  of top-level docs from each repo — re-sync them from source, never edit in place
  expecting the change to propagate.
- `kovanica-node` is a **release mirror** of `kovanica-protocol` crates (synced on
  tags, not auto). Prefer `kovanica-protocol` for code truth.
- Verify before claiming: run commands, don't assume output.

---

## Never Commit

- `.obsidian/`, `.claudian/`, `.trash/` — ignored (app state, session data)
- `KovanicaDAG/KovanicaDAG/` — embedded stale copies with their own `.git`
- Never convert `kovanica-*` doc folders into submodules