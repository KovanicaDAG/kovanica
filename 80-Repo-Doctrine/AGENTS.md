---
title: "AGENTS.md — Monorepo Conventions"
category: 80-Repo-Doctrine
source: AGENTS.md
synced: 2026-09-26
---
# AGENTS.md

Guidance for AI assistants (and humans) working in the **Kovanica** monorepo.

> **Status**: Active development. Component repos are consolidated and tracked
> directly on `main` (no more per-component clones; see `dev.sh`). Two repos stay
> OUTSIDE the workspace at `~/`: `kovanica-agent`, `kovanica-brain-vault`.
> **Deep protocol context lives in `protocol/AGENTS.md`** (consensus invariants,
> RFC implementation notes, "hard-won lessons") — read it before touching
> `kovanica-dag` / `kovanica-state`.
>
> > **Consensus decision (ratified 2026-09-25): Kovanica is PoA-only.**
> > Proof-of-Work is being removed from the protocol. See
> > `protocol/docs/RFC-POA-Migration.md` §0 (canonical). Items marked `[TARGET]`
> > are ratified but not yet implemented; `[CURRENT]` items describe shipped code.
> >
> > `[TARGET]` PoW is **deleted**, not merely off by default: `Dag::set_proof_of_work`,
> > `set_difficulty`, the `pow` and `difficulty` modules, and `KOVANICA_POW` all
> > go away, and `KOVANICA_CONSENSUS=pow` is transitional only. PoA needs
> > `KOVANICA_CONSENSUS`, `KOVANICA_AUTHORITIES`, `KOVANICA_AUTHORITY_THRESHOLD`,
> > `KOVANICA_SLOT_DURATION`. `[CURRENT]` PoW is still in the tree and still
> > reachable, so **do not write a `[TARGET]` claim as though it were shipped.**
> >
> > **DECIDED 2026-09-25 — hybrid / staked-VRF admission is dropped entirely**
> > (§0.7.1). PoA is the only admission path; the "PoA + staked-VRF secondary
> > tier" option was considered and rejected. The stake registry retires with
> > it. **RFC-005 vault/CSV and the treasury vaults are unaffected** (verified:
> > `vault.rs` has zero stake references). A non-authority can never produce a
> > block.
> >
> > `[OPEN]` — still not settled, do not implement on the strength of §0:
> > **mainnet authority-set governance** — the rotation *mechanism* is settled
> > (genesis-fixed set, on-chain M-of-N `AuthorityUpdateTx` only); the residual
> > *inputs* are not: initial set choice, eligibility, authority-key ceremony,
> > threshold `t`, expansion, and dissolution/recovery (§0.7.2).
> >
> > **RFC-006 supply math is unaffected.** The emission curve is height-indexed
> > and `cumulative_minted` is hard-capped at `MAX_SUPPLY` in `apply_block`, so
> > neither depends on who produced a block. MAX_SUPPLY **90.2M KVNC**, s₀
> > **10 KVNC/block**, era **2,000,000 blocks**, α **3/4**, maturity **100
> > blocks**, fee split **75% burned / 25% producer**, GHOSTDAG **k=3**, UTXO,
> > Ed25519, **1 KVNC = 100_000_000 atoms** — all unchanged. What changes is the
> > *pace* (fixed `SLOT_DURATION_MS`, default 3000 ms, no retarget, no gap-fill),
> > never the cap.

---

## 1. Repository Layout

- `protocol/` — Core consensus + ledger (Rust workspace). `crates/kovanica-dag`
  (BlockDAG/GHOSTDAG/VRF), `crates/kovanica-state` (UTXO, stake registry),
  `docs/` (RFCs, KVP, LEGIT-BOARD). **Consensus logic is authored here.**
- `node/` — Thin packaging surface for the runnable node + explorer HTTP API
  (Rust workspace). Contains a single `kovanica-node-bin` crate that depends on
  the protocol crates via path deps and builds the `kovanica-node` binary.
  Cargo.toml repo key is `KovanicaDAG/kovanica-node`. **The VPS builds from here.**
- `web/site/` — Explorer + wallet frontend. TanStack Start/Router, Vite, Nitro,
  Tailwind v4, Zustand, React Query, better-auth, **pglite** DB + migrations.
  (`web/` itself is just a wrapper; all commands run in `web/site/`.)
- `wallet/` — Mobile wallet (UniFFI + Kotlin/Swift): `android/`, `ios/`,
  `extension/`, `shared/`.
- `mobile/android/` and `android-light-node/` — Android light-node apps
  (Jetpack Compose; `android-light-node` is the actively developed one).
- `cli/`, `installer/`, `ledger-app/` (versioned hardware-wallet harness,
  has `Makefile`), `data/` (runtime data, NOT versioned).
- Root docs to know: `NETWORK.md` (canonical network/domain/ports), `MASTER-ROADMAP.md`.

> **Crate-tree (deduped 2026-09-24):** `protocol/crates/` is the **single source
> of truth** for the 5-crate workspace (`kovanica-dag`, `kovanica-state`,
> `kovanica-node`, `kovanica-cli`, `kovanica-ffi`). The legacy `node/crates/`
> mirror was deleted; `node/` now builds a thin `kovanica-node-bin` wrapper over
> the protocol crates (path deps). Author consensus/ledger changes in
> `protocol/crates/` only. Peer/seed defaults live in
> `protocol/crates/kovanica-node/src/explorer.rs` (`P2P_BOOTSTRAP`,
> `DEFAULT_PEERS`) and `dns_seed.rs`.

---

## 2. Conventions

### Rust (`protocol/`, `node/`)
- Edition 2021, `rust-version` 1.75. `protocol/` pins toolchain **1.98.0** via
  `rust-toolchain.toml` (rustfmt + clippy).
- CI enforces `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`
  (`-D warnings`, not just clean).
- Kept unsafe-free; `ledger-app` enforces `#![forbid(unsafe_code)]`.
- **Determinism is sacred**: consensus must be a pure function of the DAG.
  Node-local policy (e.g. wall-clock block-time bounds) must stay in the node,
  never in `dag`/`state`.

### Web (`web/site/`)
- Commands: `npm run dev` (port **8080**), `npm run typecheck`, `npm run lint`,
  `npm run test` (`node --test`), `npm run format` (prettier, writes).
- `npm run build` = vite build **+ `db:migrate`**; `npm run build:vps` does NOT run
  migrations — run `npm run db:migrate` yourself when you touch `migrations/`.
- `npm run build:vps` sets `NITRO_PRESET=node-server` (needed for PM2 deploy).

### FFI (`crates/kovanica-ffi`)
- Kotlin/Swift bindings are **committed** (do not gitignore them). After any UDL/
  Rust change, regenerate and commit in the same change:
  ```bash
  cargo build --release -p kovanica-ffi
  cargo run -p kovanica-ffi --bin uniffi-bindgen -- generate \
    --library target/release/libkovanica_ffi.so \
    --language kotlin --out-dir crates/kovanica-ffi/bindings/kotlin
  cargo run -p kovanica-ffi --bin uniffi-bindgen -- generate \
    --library target/release/libkovanica_ffi.so \
    --language swift --out-dir crates/kovanica-ffi/bindings/swift
  ```
- Warning: the binding-drift CI (`bindings.yml`) only runs in the standalone
  `kovanica-protocol` repo. Root CI is **only** `.github/workflows/build-android.yml`
  (builds the Android AAR from `protocol/crates/kovanica-ffi`); a monorepo-level
  bindings guard is not yet enabled. Regenerate manually — nothing will catch you.

---

## 3. Build & Test Commands

| Repo | Build | Test | Check |
|------|-------|------|-------|
| `protocol/` | `cargo build` | `cargo test` | `cargo fmt --check && cargo clippy --all-targets -- -D warnings` |
| `node/` | `cargo build --workspace` | `cargo test --workspace` | same, plus `cargo build --release --workspace` (deploy artifact) |
| `web/site/` | `npm run build:vps` | `npm run test` | `npm run typecheck && npm run lint` |
| `wallet/` / `android-light-node/` | `./gradlew assembleDebug` | `./gradlew test` | `ktlint` |

CI runs: root `build-android.yml` only. Per-repo workflows under `*/\.github/`
are leftovers from the standalone repos and do **not** run in the monorepo.

---

## 4. Git Workflow

- **Never commit to `main` directly.** Branch → PR. Branch naming:
  `consensus/…`, `dag/…`, `ledger/…`, `web/…`, `mobile/…`.
- Linear history: `git pull --rebase origin main` before starting and before push.

### 4.1 Multi-Agent Safety (CRITICAL)

Multiple AI agents/models work this repo concurrently; silent overwrites are the
#1 cause of lost work. Every agent MUST:

- **One agent, one branch.** Never share a branch without explicit coordination.
- **Never plain `git push --force`** — it bypasses the safety check and destroys
  others' commits. If history must be rewritten: `git push --force-with-lease`.
- **Before commit**: `git status` + `git log origin/<branch> -1` — branch moved
  upstream since you synced?
- **Before push**: `git fetch origin` then `git diff HEAD origin/<branch>` and
  review. If you see commits you don't recognize, STOP — do not overwrite.
- **Never edit or delete commits/files another agent owns** without pulling their
  latest commit first. Who touched it last: `git log -1 --format='%an %ar' -- <file>`.
- `main` is protected (PR + linear history + branch-up-to-date + no force push);
  that protection is the backstop — not a reason to skip the rules above.
- Helper: `git fetch origin main --quiet && git diff HEAD origin/main --stat`
  before pushing (mirrors the pre-push hook behavior; hook not yet installed here).

---

## 5. Network & Deployment

Canonical sources (read before trusting anything else):
`NETWORK.md` (domains/ports/seeds), `protocol/TESTNET-RFC006.md` +
`protocol/docs/TOKENOMICS.md` (economy), `web/site/DEPLOY.md` (VPS layout),
`protocol/OPERATIONS.md`.

High-signal facts that are easy to get wrong:
- **RFC-006 tokenomics is LIVE on testnet** — activated as a consensus fork that
  **wiped all pre-RFC-006 balances**. Subsidy 10 KVNC/block (geometric decay ×¾
  every 2,000,000 blocks), hard cap 90.2M, coinbase maturity 100 blocks, 75% fee
  burned. Current truth is in `protocol/TESTNET-RFC006.md` — the older "200×ATOM /
  founder seed=1" genesis notes are obsolete. **These numbers are unaffected by
  the PoA-only decision** (height-indexed curve + `MAX_SUPPLY` cap in
  `apply_block`) — only the wall-clock *pace* changes.
- P2P is **TCP 9000 only** (libp2p removed). Seeds: `seed.kovanica.online:9000`
  (primary, IP behind grey-cloud DNS) and `seed2.kovanica.online:9000`; `seed3`
  retired. **Never dial `explorer.kovanica.online:9000`** — it is Cloudflare
  orange-cloud, which proxies TCP; only grey-cloud DNS names reach the seed.
- Run a node (from `protocol/`):
  `KOVANICA_POW=1 KOVANICA_LISTEN=0.0.0.0:9000 KOVANICA_PEERS=seed.kovanica.ons…`
  … exact env from `protocol/TESTNET.md`.
  `[TARGET]` `KOVANICA_POW=1` drops out of this line; the PoA equivalent is
  `KOVANICA_CONSENSUS=poa` (already the default when unset) plus, for an
  authority operator, their signing key via `set_authority_signing_key`.

(Deployment steps: web `cd web/site && npm run build:vps && pm2 restart kovanica-web`;
node `cd node && cargo build --release --workspace` then restart systemd units —
details drift; rely on `DEPLOY.md`/`OPERATIONS.md`.)

---

## 6. Consensus Overview & RFC Index

Deep detail + invariants: `protocol/AGENTS.md`. This monorepo-level summary:

- **GHOSTDAG** (k=3 on testnet): order = `order(sp) ++ mergeset ++ [B]`;
  incremental interval-tree + future-covering reachability; blue work drives chain
  selection.
- **Ledger**: UTXO, Ed25519 spend auth, per-asset conservation (KVP-102); per-block
  state incremental from selected parent + mergeset; finality pruning folds deltas
  into children; stake registry bonds via `KVB1||vrf_pk` / `KVU1` tags, maturity 100.
- **Hybrid admission** `[CURRENT]`, **`[TARGET]`-removed entirely**: PoW
  (`H*work < 2^256`, retargeted) and staked-VRF over an epoch beacon; one
  staked block per `(vrf_pk, selected_parent)`. **Both halves are being
  removed** (RFC-POA-Migration §0.7.1, decided 2026-09-25) — the staked-VRF
  half was not left to ride along with PoW. The stake registry
  (`kovanica-state/src/stake.rs`) retires with it. Do not build on it, and do
  not port it to a "secondary tier": that option was rejected.
- **PoA admission** `[CURRENT]`: fixed authority set (`KVA1` UTXO), slot
  round-robin at `SLOT_DURATION_MS` (default 3000), Ed25519 authority signature
  per block, `work` pinned to `POA_NOMINAL_WORK = 1`. `[TARGET]` PoA is the
  *only* admission model once PoW and hybrid are removed.

| RFC | Feature | Status |
|-----|---------|--------|
| 001 | Multisig (M-of-N P2SH) | main |
| 002 | Native tokens (KVP-102) | main |
| 003 | Stealth + Script v2 | main |
| 004 | HTLC / Atomic swaps | main |
| 005 | Vault / CSV | main |
| 006 | Tokenomics (emission, cap, fee burn) | **live on testnet** (activation fork wiped balances) |
| RFC-POA / KVP-201 | PoA-only consensus (authority set + slots) | **Draft**; §0 ratified, PoW removal `[TARGET]` |

---

## 7. Key Files

| File | Purpose |
|------|---------|
| `protocol/AGENTS.md` | **Deep consensus/RFC invariants — read before dag/state work** |
| `protocol/docs/LEGIT-BOARD.md` | P0/P1/P2 roadmap checklist |
| `MASTER-ROADMAP.md` | Cross-component roadmap (root) |
| `NETWORK.md` | Canonical domains, seeds, ports, genesis |
| `protocol/TESTNET-RFC006.md` | Current testnet economy + run env |
| `web/site/DEPLOY.md` | VPS deploy canonical (seed env, systemd) |
| `protocol/docs/TOKENOMICS.md` | RFC-006 full spec |
| `protocol/crates/kovanica-state/src/stake.rs` | Multi-asset stake registry |

---

## 8. Common Tasks

### Consensus feature
1. RFC in `protocol/docs/RFC-XXX-*.md`; implement in `kovanica-dag`/`kovanica-state`.
2. Adversarial tests in `crates/kovanica-dag/tests/` / `crates/kovanica-state/tests/`.
3. Update activation-score constant + `Ledger::set_*_activation_score`; then
   `LEGIT-BOARD.md`.

### Web feature
Routes `web/site/src/routes/`, components `src/components/`, API `src/lib/api/`.
If it touches `migrations/`, also `npm run db:migrate`. Deploy:
`npm run build:vps && pm2 restart kovanica-web`.

### FFI addition
Add method to `LightNode` in `crates/kovanica-ffi/src/light_node.rs`, regenerate
bindings (§2), commit generated Kotlin/Swift — **no monorepo CI enforces this yet**.

---

## 9. Debugging

| Issue | Look here |
|-------|-----------|
| Consensus fork | `kovanica-dag/tests/consensus.rs` — `adversarial_wide_fork` |
| Mempool eviction / fee est | `protocol/crates/kovanica-node/src/mempool_v2.rs` |
| P2P not connecting | `protocol/crates/kovanica-node/src/p2p_hardening.rs` |
| Wallet balance wrong | `web/site/src/components/wallet/wallet-view.tsx` |
| Map black on mobile | `web/site/src/components/map/dashboard.tsx` (`h-[46vh] min-h-[300px]`) |

---

## 10. Security

- No secrets in the repo — `.env`/`.env.*` are gitignored; never commit one.
- Node policy must never weaken consensus determinism (see §2 Rust).
- Security reports: GitHub Security Advisories or `security@kovanica.online`.

---

*File owned by maintainers; update when conventions change. Keep succinct — the
deep version lives in `protocol/AGENTS.md`.*