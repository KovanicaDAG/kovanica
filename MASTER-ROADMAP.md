# Kovanica — Master Roadmap (merged checklist)

> **Purpose:** single merged note for all upgrade / plan / todo lists in the monorepo.
> Built 2026-09-17 from every source listed under [Sources](#-sources). Duplicate plan
> docs are collapsed into one row each; the authoritative copy lives in
> `protocol/docs/` and is mirrored byte-identical into the Obsidian vault.
>
> **Grouping scheme:** `A` protocol & consensus · `B` public visibility · `C` public
> testnet maturity · `D` mainnet track · `E` clients & UI · `F` process & infra.
> Sub-items numbered `A1, A2, …`. Legacy ids in source files (LEGIT-BOARD `P0.x/P1.x/P2.x`,
> `UPGRADE-PHASES` `A1–A3/B1–…`, 6-point plan `1.x–6.x`) are **remapped** here, not repeated.
>
> **Status key:** ✅ done · 🟡 partial · ❌ missing · 🔒 shipped in core but node API not
> exposed · ⏳ pending / planned · 🔴 **ACTIVE**.

> **Consensus decision (ratified 2026-09-25): Kovanica is PoA-only.**
> Proof-of-Work is being **removed**, not merely disabled. Items marked
> `[TARGET]` are ratified but not yet implemented; `[CURRENT]` items describe
> shipped code. Canonical policy, the removal inventory, the operator
> replacements and the open decisions are in
> [`protocol/docs/RFC-POA-Migration.md` §0](protocol/docs/RFC-POA-Migration.md)
> — this roadmap cites it and does not restate it.
>
> **Roadmap-level consequences** (annotations below name the affected rows):
> - **A13 + A6 / C12 (testnet reset) is superseded in scope.** The reset is now
>   **mandatory** for the PoA transition, not optional (RFC-POA-Migration §0.6),
>   and it must additionally re-seed the authority set.
> - **A6 (RFC-006 tokenomics) is unaffected** — MAX_SUPPLY 90.2M KVNC, s₀
>   10 KVNC/block, era 2 000 000, α 3/4, maturity 100, fee 75% burned /
>   25% producer. Do not re-derive these for the PoA work.
> - **A10 (stake registry / sortition) is CANCELLED, not `[OPEN]`.** Hybrid
>   was dropped entirely (§0.7.1, decided 2026-09-25) and the stake registry
>   retires with it. There is no DeFi 5.3 sortition primitive to extend —
>   `StakeState` asset support is moot unless a *different* staking design is
>   proposed. **RFC-005 vault/CSV and the treasury vaults are unaffected** and
>   remain the live multi-asset custody story.
> - **C6 (security notes) changes class.** `docs/SECURITY.md` must be written
>   against a *permissioned* PoA threat model, not a hash-power one — and it
>   must own the adversarial-coverage gap left by deleting the `challenger_*`
>   suites (§0.7.3).
> - **C9 tuning review drops "difficulty"** from its scope; it gains authority
>   slot pacing.
> - **Dormant: no mainnet** is unchanged — the mainnet authority-set
>   governance question is `[OPEN]` (§0.7.2), which is another reason mainnet
>   stays parked.

---

## A — Protocol & consensus upgrades

*Sources: `docs/RFC-00{1..5}*.md`, `docs/LEGIT-BOARD.md` P1.1, `plans/protocol-evolution-6-points.md`,
`00-Meta/Notes/ROADMAP.md`, `UPGRADE-PHASES.md`, `02-State-UTXO/current-branch-state.md`*

| ID | Item | Status | Notes / source |
|----|------|--------|----------------|
| A1 | **RFC-001 Multisig** (KVP-101, M-of-N P2SH) | ✅ | Blue score 0; 35-test adversarial suite `multisig_consensus.rs` |
| A2 | **RFC-002 Native tokens** (KVP-102, multi-asset) | ✅ | 1-byte asset flag = **format bump → testnet reset at activation**; 29-test suite; HTTP `asset_id` surfacing tracked in C2 |
| A3 | **RFC-003 Stealth + Script v2** (KVP-103) | ✅ | Address `0x02`/`0x03`; checkpoint v5; one-time-key ECDH, CLTV/CSV ops; 25-test suite |
| A4 | **RFC-004 HTLC / atomic swap** (KVP-104) | ✅ | PR #88 merged 2026-09-08 (`fb13741`); no format bump; Tier Nolan `atomic_swap.rs`; CLTV fix included |
| A5 | **RFC-005 Vault / CSV** (KVP-105) | ✅ | Real CSV + per-UTXO creation height; checkpoint v6, no format bump; **⏳ FFI surface deferred** (HTLC FFI slice is the model) |
| A6 | **RFC-006 Tokenomics** (emission curve) | ✅ **CORE DONE** | Steps 1–4 **landed** on `tokenomics/rfc-006-emission-curve` (785 tests): smooth α=¾ emission, MAX_SUPPLY 90.2M, coinbase maturity 100, fee burn 75/25. **Steps 5–6 done** (treasury genesis + mainnet profile merged via fix/rfc006-critical-issues), **Step 7 done** (supply accounting on /api/head). Activation = **consensus fork → testnet reset, checkpoint v7**. `TOKENOMICS_ACTIVATION_SCORE=0` active at genesis on main. **Unaffected by the PoA-only decision** — the curve is height-indexed, not work-indexed, and `cumulative_minted` is capped in `apply_block`, so not one number here changes. |
| A7 | **Epoch randomness beacon** | ✅ | `UPGRADE-PHASES` B1, merged via PR #49 (Phase 2) |
| A8 | **DAG-level past-set pruning** | ✅ | `UPGRADE-PHASES` B2, PRs #36 / #51 |
| A9 | **UTXO undo log** | ✅ | `UPGRADE-PHASES` B3, PR #50; enables ledger per-block state pruning (6-point plan 2.3) |
| A10 | **Token staking / sortition** (DeFi 5.3) | ❌ **CANCELLED** | Hybrid dropped entirely (§0.7.1, decided 2026-09-25); stake registry retires with it; a non-authority can never produce a block. Extending `StakeState` to assets is moot. **RFC-005 vault/CSV + treasury vaults unaffected** and remain the live custody path |
| A11 | **Research / docs-only gates** (bridge RFC 4.x, VM decision 5.5, CT 6.3, DEX design 5.4) | ⏳ | All deferred; no implementation in scope; VM explicitly **not EVM** |
| A12 | **CoinJoin batching** (6.2) | ⏳ | Node-level, non-consensus, `prepare_transfer` pattern |
| A13 | **Codebase fix + testnet reset** | ✅ **CODE DONE** | Operator/founder wallet genesis implemented; all code changes + tests pass. **Awaiting testnet reset execution** (A13.13–A13.24). |

---

## B — Public visibility (was LEGIT-BOARD P0)

*Source: `protocol/docs/LEGIT-BOARD.md` §P0, `protocol/TODO.md`*

| ID | Item | Status | Notes |
|----|------|--------|-------|
| B1 | **Public source code** | ✅ | Public mirror + `sync-public-node` pipeline (PRs #10–#14); mirror-only manifests, secrets stay private; **link from kovanica.online ⏳** |
| B2 | **GitHub Releases + checksums** | 🟡 | Rolling `v0.1.0` binaries with sha256 (linux x86_64/aarch64, macos, windows); **⏳ APK on the same release**; web “Native wallet” card → Releases ⏳ |
| B3 | **One-pager (What is Kovanica)** | 🟡 | Draft `WHAT-IS-KOVANICA.md` ✅; web surface + footer link ❌ |
| B4 | **Emission & tokenomics (public)** | 🟡 | `TOKENOMICS.md` draft ✅; web docs + “KVNC vs KVP-102” wording ⏳ (finalize after RFC-006) |
| B5 | **Testnet reset policy** | 🟡 | Resets only on format bumps / safety incidents (de-facto); written policy + epoch tags ❌ |
| B6 | **Reliable web deploy** | 🟡 | Build + VPS + PM2 working; `web-deploy.yml` CI gate + smoke checks ⏳; fallback path documented |
| B7 | **Disclaimer** | ❌ | Footer/docs: testnet software, no investment advice; maintainer contact ⏳ (ENTITY-LEGAL exists, D5) |

---

## A13 Detail — Codebase fix + testnet reset (tracking)

> **Purpose:** Single checklist for any change that requires a **testnet reset** (genesis hash change → all nodes resync from zero).
> **PoA overlay (2026-09-25):** this checklist is now the *de facto* procedure for
> the mandatory PoA reset (RFC-POA-Migration §0.6) as well as A13/A6. Two steps
> must be added before use: configure the authority set on every seed
> (`KOVANICA_CONSENSUS=poa`, `KOVANICA_AUTHORITIES`,
> `KOVANICA_AUTHORITY_THRESHOLD`, `KOVANICA_SLOT_DURATION`) and replace
> A13.19's "verify block production (mining / staking)" with "verify authority
> slot production". Until then, treat the A6/A13 numbers as `[CURRENT]` and the
> PoA additions as `[TARGET]`.

### 1. Define the fix scope
- [x] **A13.1** Root cause / motivation: No main node wallet address — node operator has no dedicated wallet to receive mining rewards; currently defaults to founder (seed=1) address
- [x] **A13.2** Files to change (list paths):
    - `node/src/node.rs` — Added `operator_wallet`, `founder_wallet` fields; `genesis_with_finality()` generates/saves both
    - `protocol/crates/kovanica-cli/src/wallet.rs` — BIP39 mnemonic wallet support (complete rewrite)
    - `protocol/crates/kovanica-cli/Cargo.toml` — Added `bip39` dep, lib+bin structure
    - `node/Cargo.toml` — Added `kovanica-cli` dependency
    - `protocol/Cargo.toml` — Added `bip39` workspace dependency
    - `node/src/explorer.rs` — Added `operator_wallet_address` to `/api/bootstrap` response
- [x] **A13.3** Type of change:
    - [ ] Wire format bump (block/tx encoding)
    - [x] Consensus logic (validation, ordering, fork rule)
    - [ ] Genesis params (subsidy, premine, founder_seed, k, finality)
    - [x] New RFC activation (checkpoint version bump)
    - [ ] Other: _______________

### 2. Code changes
- [x] **A13.4** Implement fix in `protocol/` (core crates)
    - [x] Reuse `Wallet` struct from `kovanica-cli` for BIP39 mnemonic generation
    - [x] Add `operator_wallet_path` config to `Node` / `LightConfig`
- [x] **A13.5** Update `node/` (explorer, RPC, miner)
    - [x] `node.rs`: Add `operator_wallet: Option<Wallet>` field; on `genesis()`, generate/load wallet, set `self.miner = Some(operator_wallet.address())`
    - [x] `main.rs` (binary): On first run (`serve`/`explorer`), create wallet if missing, save to `$KOVANICA_DATA/operator-wallet.key` (BIP39 mnemonic + hex seed)
    - [x] `explorer.rs`: Persist/load operator wallet alongside node state (log/snapshot)
    - [x] CLI: Add `--operator-wallet-path` flag, `--generate-operator-wallet` flag
    - [x] **Founder wallet**: On `genesis()`, generate founder wallet with BIP39 mnemonic, set as coinbase recipient for 200K KVNC premine, save to `$KOVANICA_DATA/founder-wallet.key`
- [ ] **A13.6** Update `web/` (address format, API types) if needed — *not needed: operator wallet is genesis-only, address exposed via /api/bootstrap*
- [ ] **A13.7** Update `wallet/` / `mobile/` (FFI bindings) if needed — *not needed: no FFI surface change; operator wallet is node-internal*
- [x] **A13.8** Run `cargo test` (all green) + `cargo fmt --check` + `cargo clippy --all-targets`

### 3. Genesis / activation
- [x] **A13.9** New genesis hash (compute: `cargo run -p kovanica-node -- genesis --dry-run` with new operator wallet) → `256c87faf1803ad75c855de95b7cee317f52c1f3fa311ca5479083174e56c31b` — ⚠️ *CORRECTION: `256c87fa…` is the treasury-LESS genesis (RPC `genesis` path); the live network's treasury-included genesis (explorer `genesis_node()`) is `9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97`*
- [x] **A13.10** Update `NETWORK.md` (genesis hash, seeds, params, operator wallet note)
- [x] **A13.11** Update `/api/bootstrap` response (genesis, subsidy, premine, founder_seed, operator_wallet_address)
- [x] **A13.12** If RFC activation: set activation score, bump `CHECKPOINT_VERSION` (v7 for RFC-006 + operator wallet) — already 0 (active at genesis) + v7

### 4. Testnet reset procedure
- [ ] **A13.13** Stop all seeds (`systemctl stop kovanica-*`)
- [ ] **A13.14** Wipe data dirs (`/var/lib/kovanica-*`, `/root/kovanica-data`)
- [ ] **A13.15** Rebuild binaries (`cargo build --release -p kovanica-node`)
- [ ] **A13.16** Deploy new binaries to seeds (`deploy-seed.sh` / `deploy-seed2.sh`)
- [ ] **A13.17** Start seeds, verify genesis match (`/api/head` on all)
- [ ] **A13.18** Verify peer connectivity (mesh, DHT, DNS seeds)
- [ ] **A13.19** Verify block production (mining / staking) `[TARGET]`→ verify authority slot production
- [ ] **A13.20** Update explorer / web (if API changed)

### 5. Post-reset validation
- [ ] **A13.21** Smoke test: faucet, transfer, multisig, HTLC, vault
- [ ] **A13.22** Light-node sync test (Android `LightNode` → live genesis)
- [ ] **A13.23** Metrics baseline (orphan rate, propagation, fork rate, disk growth)
- [ ] **A13.24** Update `MASTER-ROADMAP.md` status → ✅

---

## C — Public testnet maturity (was LEGIT-BOARD P1 + soak + ops)

*Sources: `docs/LEGIT-BOARD.md` §P1, `docs/TESTNET-SOAK.md`, `TODO/seed2-deploy.md`,
`TODO.md` §Next session, `00-Meta/Notes/ROADMAP.md`, 6-point plan Tačka 1*

| ID | Item | Status | Notes |
|----|------|--------|-------|
| C1 | **HTLC on testnet** (was P1.1) | ✅ | Done 2026-09-08; KVP-104 → Shipped |
| C2 | **Node HTTP `asset_id`** (was P1.2) | 🔒 | `/api/utxos|history|prepare` asset fields; web AssetPicker + wallet + explorer wiring done ✅; live multi-asset on testnet blocked on node API |
| C3 | **Multiple public seeds** (was P1.3) | ✅ | seed (Hostinger VPS, `kovanica-explorer`) ✅ + seed2 (Hostinger KVM2 VPS `76.13.250.65`, `srv1991525`) ✅ — org-distinct; seed3 (AWS) **fully decommissioned 2026-09-21** (instance stopped + DNS deleted, NXDOMAIN); `/api/bootstrap` peer leak fixed (commit `7bd9aab`), both VPSes deploy fixed binary with clean `KOVANICA_PEERS` |
| C4 | **Run-a-node guide** (was P1.4) | 🟡 | `install.sh` prebuilt-first + `OPERATIONS.md` runbook; single operator guide + “tip == explorer” smoke ⏳ |
| C5 | **Public status surface** (was P1.5) | 🟡 | `/network` page shipped (PR #89) ✅; uptime history ❌ |
| C6 | **Security notes / threat model** (was P1.6) | ❌ | `docs/SECURITY.md` (PoW+GHOSTDAG guarantees, key handling, finality/reorg expectations) not written — **and its premise is now wrong**: the guarantee set is permissioned-PoA + GHOSTDAG, not hash-power + GHOSTDAG. Must be rewritten against the PoA threat model (RFC-POA-Migration §0.5) before it ships |
| C7 | **Open issue tracker** (was P1.7) | 🟡 | Public Issues on mirrored repos; bug/feature templates + security contact (GH advisories / `security@kovanica.online`) ⏳ |
| C8 | **Spec index** (was P1.8) | ❌ | Docs index of KVP-101…105 + RFC links + roadmap-sync ⏳ (skills `rfcs-index` exists in vault) |
| C9 | **Testnet soak** (roadmap item 4 / 6-point 1.1) | 🔴 **ACTIVE** | Baseline 2026-08-24 (h448/447, 2 peers, no reorgs) + public-API snapshot 2026-08-31 (~1.18 min/block recovered, **no retune**) — both `[HISTORICAL — PoW era]`, must be re-captured post-transition. Next: **⏳ VPS Prometheus scrape** (orphan/propagation/fork/disk, `/metrics` not public) and **⏳ tuning review** (k, finality, pruning — **difficulty is dropped**, there is nothing to retune after the removal; authority slot pacing is added) |
| C10 | **Ops hardening** (6-point 1.2 / Phase 6) | 🟡 | Backup automation ✅ (#40), systemd unified ✅, atomic binary swap ✅; restore drill + tunnel alerting + release pinning ✔️ in OPS-HARDENING doc, drills ⏳ |
| C11 | **Security hardening** (6-point 1.3) | 🟡 | `cargo-audit` CI ✅ (D2); extended net-boundary fuzz + ed25519/VRF crypto review ⏳ |
| C12 | **Testnet reset execution** (for A13 + A6) | 🟡 **READY** | A13.1–A13.12 complete; A6 core done on main; use A13.13–A13.24 checklist + TESTNET-RESET-PROCEDURE.md; coordinate with seed operators |

---

## D — Mainnet track (was LEGIT-BOARD P2)

*Sources: `docs/LEGIT-BOARD.md` §P2, `docs/{AUDIT-PLAN,BUG-BOUNTY,REPRODUCIBLE-BUILDS,MAINNET-CRITERIA,OPS-HARDENING,ENTITY-LEGAL,COMMUNITY-DISCORD,PRODUCT-POLISH}.md`,
`brain-vault …/6-Business/Compliance/MiCA/MASTER_PLAN.md`*

> **Constraint:** mainnet switch stays **dormant / on-sleep-off**. Excluded from scope:
> live web wiring, genesis ceremony, launch checklist, param freeze. 6-point plan: “mainnet
> NE pokrećemo”.

| ID | Item | Status | Notes |
|----|------|--------|-------|
| D1 | **Audit plan** (was P2.1) | 🟡 | `AUDIT-PLAN.md` draft ✅ — target Q1 2027, $75k–$150k; scope dag (~4.5k LoC, Critical) + state (~8.5k, Critical) + node RPC (~2k, High); FFI/mobile/cli/web/infra out of scope. Firm selected/booked ⏳ |
| D2 | **Reproducible builds** (was P2.2) | 🟡 | `REPRODUCIBLE-BUILDS.md` draft ✅ (Rust 1.82.0, node 22.12.0, `Cargo.lock` committed, `SOURCE_DATE_EPOCH`, CI workflow). Third-party rebuild matches release SHA256 ⏳ |
| D3 | **Bug bounty** (was P2.3) | 🟡 | `BUG-BOUNTY.md` draft ✅ — effective Q4 2026, $10k–$50k severity tiers, in/out-of-scope lists, safe harbor. Live bounty page ⏳ |
| D4 | **Mainnet exit criteria** (was P2.4) | 🟡 | `MAINNET-CRITERIA.md` draft ✅ + `skills/mainnet-checklist` (RFC-006 locked, ≥2 wk soak, ≥3 independent seeds ≥30 days, geo/org/ASN-distinct, public `/metrics`, no single operator >50%, audit + reproducible genesis by ≥2 parties). **Not yet met** |
| D5 | **Entity & legal blurb** (was P2.5) | 🟡 | `ENTITY-LEGAL.md` in-repo ✅; site integration + disclaimers ⏳ |
| D6 | **Community home** (was P2.6) | 🟡 | `COMMUNITY-DISCORD.md` server structure/moderation ✅; live channel + link ⏳ |
| D7 | **KVP-102 issuance policy** (was P2.7) | ❌ | Rule “coinbase mint only” is code + KVP-102 enforced; public policy wording on TOKENOMICS/landing ⏳ |
| D8 | **Ops hardening (mainnet)** (was P2.8) | 🟡 | `OPS-HARDENING.md` draft ✅ (hourly 48h / daily 30d / weekly 90d off-site, systemd `kovanica-backup.service|timer`, RTO<4h, RPO<24h); restore drill run ⏳ |
| D9 | **Product polish** (was P2.9) | 🟡 | `PRODUCT-POLISH.md` draft ✅ — Ledger (WebHID + custom `kovanica-ledger-app`, BIP-44 `m/44'/3007'/0'/0/0`) and Trezor (WebUSB, `KOVANICA_TESTNET/MAINNET`) planned High, Keystone/Coldcard later; fee estimation ✅ shipped; hardware verified on device ⏳; deep-links ⏳ |
| D10 | **Business / compliance (MiCA CASP)** | ⏳ | `6-Business/Compliance/MiCA/MASTER_PLAN.md` — Kovanica Payments d.o.o. (Croatia, HANFA, €150k capital, 8 CASP services), policy inventory “Planned”. **Out of protocol-upgrade merge scope; tracked here as business-track reference only** |

---

## E — Clients & UI

*Sources: `brain-vault/04-Clients-LightNodes/web/site/UI-PARITY-CHECKLIST.md`,
`docs/plans/mobile-light-node.md`, `docs/plans/android-light-node-app.md`,
`00-Meta/Notes/ROADMAP.md`, `mobile/README.md`*

| ID | Item | Status | Notes |
|----|------|--------|-------|
| E1 | **Web ↔ protocol parity** | 🟡 | Core flows ✅ (DAG explorer, wallet, multisig `/multisig`, `/network`); RFC-002 contract types/AssetPicker/wallet+explorer wiring ✅ (PR #89); ❌ hybrid staking UI, ❌ SPV in browser, 🟡 mobile download strip, ❌ metrics dashboard |
| E2 | **Mobile light-node slices 4–8** | ✅ | Landed 2026-08-25 (workspace **v0.2.0**): custody/unbond FFI, SPV `KVLS`v1 + merkle proofs + Golomb-Rice filters, android/apple packaging + CI drift guard (`bindings.yml`), `history_of` UX layer, docs |
| E3 | **Android LightNode app (slices 9a–9d)** | 🟡 | 9a genesis gate ✅ (live-sync spike + fixtures), 9b wallet UX ✅ (Keystore AES/GCM, Compose/Material3), 9c light sync ✅ (`LightNodeRepository` + KVLS persistence), 9d staking uplink ✅ (`export_block` → `POST /api/mine/submit`); **9e WorkManager periodic sync ⏳, 9f release ⏳**; APK builds must run in GH Actions (no local SDK); `/api/bootstrap` must expose subsidy/premine/seed before mainnet |
| E4 | **iOS light node** | ⏳ | `kovanica-mobile` README: Android light node exists; iOS planned |
| E5 | **Vault FFI surface** | ⏳ | RFC-005 `vault.rs` FFI/bindings deferred (see A5) |
| E6 | **`kovanica-cli` publication** | ✅ | Decision made: included in public mirror workspace + release assets (re-synced 2026-08-24; was “decide” → now fixed) |

---

## F — Process & infra

*Sources: `00-Meta/Notes/ROADMAP.md`, `00-Meta/Notes/MASTER-STATUS.md`, `03-Node-P2P/upgrade-progress-2026-09-12.md`,
`02-State-UTXO/skills-project-planning.md`, root `AGENTS.md` §5, `TODO.md`*

| ID | Item | Status | Notes |
|----|------|--------|-------|
| F1 | **Project-planning playbook** | ✅ | `skills/project-planning`: layered decomposition (consensus-safe / ledger-safe / client-only), phase sequencing (0 stability → 0.5 RFC-006 → 1 API/tooling → 2 wallet/UX → 3 advanced), risk register |
| F2 | **Skill-pack split** | ✅ | 2026-09-12: monolithic SKILL.md (1374 lines) → 19 per-skill dirs + `scripts/`; vault structure (`markdown-vault/`) created; original kept as pointer |
| F3 | **Vault snapshots & dedup** | ✅ | `brain-vault-sync-obsidian` branch active; 7 P2 docs + 5 plans + soak-snapshot verified **byte-identical** between `protocol/docs/` and `brain-vault/…`; `protocol-evolution-6-points.md` unique to vault; `upgrade-progress-2026-09-12.md` archived point-in-time (agent tooling) |
| F4 | **Observability & reliability** | ✅ | metrics 0.22 unified recorder, `/metrics` (explorer + standalone 9090), `alerting_rules.yml` (15 alerts + 9 recording rules loaded on seeds), structured JSON tracing, fuzz targets + proptest |
| F5 | **Multi-seed discovery (DHT + DNS)** | ✅ | `dns_seed.rs` + `dht.rs` Kademlia + relay tags 0x20–0x23 + Tier 1–5 tests green; default DNS list = the three live hosts |
| F6 | **Release pinning** | ✅ | Rolling release `v0.1.0` replaced-in-place, sha256 per asset, publish skips on any build failure; Windows target added |

---

## Sequenced next actions (top-N, cross-group)

1. **C12**: Execute testnet reset (A13 + A6 combined) — run A13.13–A13.24 checklist + TESTNET-RESET-PROCEDURE.md; deploy to seed1 (primary) + seed2; verify genesis match, peer connectivity, block production, smoke tests, light-node sync. `[TARGET]` — also serves as the mandatory PoA reset, so fold in the authority-set config and slot-production check (RFC-POA-Migration §0.6).
2. **C9**: VPS Prometheus scrape (orphan rate, propagation, fork/reorg, disk) and post-~2-week tuning review (k, finality, pruning; **difficulty removed**). **Dormant: no mainnet.**
3. **C3 follow-up**: provision a third-provider/continent/ASN seed for geo diversity (seed2 = Hostinger KVM2 VPS ✅, **seed3 = AWS fully decommissioned 2026-09-21**); e.g. `deploy-seed.sh`, post-deploy checks (DNS, grey-cloud :9000, `/api/head`, bootstrap list). `/api/bootstrap` peer leak fixed on both VPSes (2026-09-21).
4. **E3**: finish Android 9e (WorkManager periodic sync) + 9f; add subsidy/premine/seed to `/api/bootstrap`.
5. **C4 / C6 / C7 / C8**: run-a-node operator guide, `SECURITY.md`, issue templates + security contact, spec index — the "legit v1" bundle (all B + C1/C2/C3/C4/C7).
6. **B4 / D7**: freeze final tokenomics numbers on web after RFC-006; publish KVP-102 issuance policy wording.

## ✅ Completed this session (2026-09-21)

| Item | Commit | Notes |
|------|--------|-------|
| Git remote PAT security | `d6e6173` | PAT moved to `~/.config/kovanica/git-credentials` (0600); remote URL clean |
| W-3 relay decoder hardening | `d84a991` | Structurally panic-free (bounds-checked `Cursor`, checked arithmetic, hostile-count guards); 2 regression tests |
| `/api/bootstrap` peer leak + seed3 cleanup | `7bd9aab` + deploy | Code fix in repo; deployed to **both VPSes** (seed1 `145.223.116.178` + seed2 `76.13.250.65`); `KOVANICA_PEERS` cleaned; public API verified clean |
| Tokenomics activation gate removal | `106e476` | Vestigial `tokenomics_activation_score` dropped from `ledger.rs` (both trees); 117 state tests pass |

**Public API now**: `"peers":["seed.kovanica.online:9000"]` — no seed3, no listen leak.
**Roadmap updated**: C3 status reflects seed3 fully decommissioned + bootstrap fix deployed.

---

## Sources

All read/verified 2026-09-17 unless noted:

- `protocol/docs/LEGIT-BOARD.md` — P0/P1/P2 grouping and horizons
- `protocol/docs/RFC-00{1..5}*.md` + `docs/KVP*.md` — RFC/KVP statuses
- `protocol/docs/{MAINNET-CRITERIA,TESTNET-SOAK,AUDIT-PLAN,BUG-BOUNTY,REPRODUCIBLE-BUILDS,PRODUCT-POLISH,OPS-HARDENING,ENTITY-LEGAL,COMMUNITY-DISCORD,WHAT-IS-KOVANICA,TOKENOMICS}.md`
- `protocol/docs/plans/{mobile-light-node,android-light-node-app,htlc-atomic-swap,stealth-script-v2-rfc-003,vault-time-lock}.md` (mirrored in `brain-vault/01-Consensus-DAG/plans/`)
- `brain-vault/01-Consensus-DAG/plans/protocol-evolution-6-points.md` (unique; Serbian, order 3→6→5, DeFi without VM)
- `protocol/TODO.md`, `protocol/TODO/seed2-deploy.md`
- `brain-vault/00-Meta/Notes/{ROADMAP,MASTER-STATUS}.md`, `brain-vault/00-Meta/Notes/UPGRADE-PHASES.md`
- `brain-vault/02-State-UTXO/{skills-mainnet-checklist,skills-project-planning,current-branch-state}.md`
- `brain-vault/03-Node-P2P/TESTNET-RFC006.md` + `upgrade-progress-2026-09-12.md` (archived snapshot)
- `brain-vault/04-Clients-LightNodes/web/site/UI-PARITY-CHECKLIST.md`
- `brain-vault/06-Business/Compliance/MiCA/MASTER_PLAN.md` (business track)
- `mobile/README.md` (iOS planned)
- root `AGENTS.md` §5 (network), §4 (git/multi-agent safety)

*This file is the merged view. Sources above remain authoritative per-item; update them and here in the same PR.*

---

## ➕ Easy insertion template (copy-paste for new items)

### For Protocol & Consensus (Section A)
```markdown
| A{N} | **Short title** (RFC/KVP/feature) | ⏳ | One-liner: what, test status, blockers |
```
Then add detail block after Section A table:
```markdown
## A{N} Detail — Short title (tracking)
- [ ] A{N}.1 Root cause / motivation
- [ ] A{N}.2 Files to change
- [ ] A{N}.3 Type: [ ] format bump  [ ] consensus  [ ] genesis  [ ] RFC activation  [ ] other
- [ ] A{N}.4 Implement in protocol crates
- [ ] A{N}.5 Update node/explorer if needed
- [ ] A{N}.6 Update web/mobile/FFI if needed
- [ ] A{N}.7 Tests pass (cargo test + fmt + clippy)
- [ ] A{N}.8 Genesis/activation params
- [ ] A{N}.9 Reset procedure (if needed) → links to C-item
```

### For Testnet Maturity (Section C)
```markdown
| C{N} | **Short title** | ⏳ | One-liner: dependency, owner, ETA |
```

### For Mainnet Track (Section D)
```markdown
| D{N} | **Short title** | ⏳ | One-liner: criteria, blocker, owner |
```

### For Clients & UI (Section E)
```markdown
| E{N} | **Short title** | ⏳ | One-liner: platform, feature, blocker |
```

### For Process & Infra (Section F)
```markdown
| F{N} | **Short title** | ⏳ | One-liner: tooling, automation, docs |
```

### Numbering rules
- **Next ID** = highest existing in section + 1 (A13, C12, D10, E6, F6)
- Keep tables sorted by ID
- Status: ✅ 🟡 ❌ 🔒 ⏳ 🔴
- Cross-ref: link dependent items (e.g., "Depends on A13.9 complete")