# Kovanica — ROADMAP

> Tracking doc. Authoritative working documents: `kovanica-protocol/docs/LEGIT-BOARD.md`
> (P0/P1/P2 checklist), `kovanica-protocol/docs/MAINNET-CRITERIA.md` (exit criteria),
> `kovanica-protocol/docs/TESTNET-SOAK.md`, and `kovanica-protocol/AGENTS.md` (stage
> history). This file is a view over those, synced on vault updates.

---

## Status Summary (2026-09-16)

- **Protocol stages 0–3: shipped** (testnet deployed since Stage 0).
- **RFCs 001–005: all on `main`, activated at blue score 0** — multisig, native
  tokens, stealth + script v2, HTLC/atomic swap, vault/CSV.
- **Restructure: complete** — monorepo split via `git filter-repo` into sibling
  repos under `/root/kovanica` (see `myObsidianVaultDAG.md`); `kovanica-ledger-app`
  and `trezor-coin-def` are new, untracked.
- **Active next:** testnet soak & parameter tuning; seed2 deployment
  (`TODO/seed2-deploy.md`); P0/P1 legit-board items (public releases, asset_id
  HTTP surface, run-a-node guide, status page); mainnet track (audit Q1 2027,
  reproducible builds, bounty).

---

## Shipped Stages (from kovanica-protocol AGENTS.md)

### Stage 0 — BlockDAG testnet ✅
DAG+GHOSTDAG consensus · UTXO ledger in linearized order · ed25519 spend auth ·
structural + stateful insert validation · recursive GHOSTDAG linearization ·
per-block UTXO state · finality-depth pruning + re-orgs · replay-log snapshots ·
reachability oracle (interval-tree + future-covering sets, incremental
maintenance) · append-only `LedgerStore` · node binary + line RPC · mempool +
block production · multi-node gossip + one-shot TCP sync · `p2p::Mesh`
discovery/relay/tx dissemination · long-lived TCP relay + WebSocket · difficulty
(algorithm + consensus enforcement) · wall-clock future-time bound (2h, node
policy) · real opt-in PoW (Nakamoto `H*work < 2^256`) · halving schedule · TX
size limits · WS explorer · human `kvnc…dag` addresses · framed bidirectional
TCP sync · multi-input transfers · CI gate + dual-stack P2P.

### Stage 1 — Operations hardening ✅
Auto-deploy armed (VPS secrets + `DEPLOY_ENABLED`) · `OPERATIONS.md` runbook ·
server-side upstream proxy (no CORS problem) · `kvnc…dag` in web wallet.

### Stage 2 — Scale & persistence ✅
Headers-first sync · DAG-level payload pruning behind the reachability oracle
(`Option<Vec<u8>>` payloads, prune depth) · finality checkpointing (UTXO set at
finality boundary + tip segment, format v2) · reachability interval-reindex
amortisation tuning (`CHILD_RESERVE`).

### Stage 3 — Protocol evolution ✅ (workspace v0.2.0)
VRF leader selection/beacon (ECVRF Ristretto255) · P2P hardening (rate limits,
duplicate suppression, scoring/banning) · mempool v2 (orphan pool, fee-based
eviction, capacity) · stake registry (bond/unbond tags `KVB1`/`KVU1`,
`UNBOND_MATURITY` 100) · hybrid PoW + VRF-staked admission · `kovanica-ffi`
UniFFI bindings (slices 3–8: custody/unbond, SPV/filters `KVLS`v1, packaging +
CI drift guard, wallet UX/history) · Android LightNode app 9a–9d landed.

---

## RFCs (all on `main`, activation blue score 0)

| RFC | Feature | Spec | Status |
|-----|---------|------|--------|
| RFC-001 | Multisig M-of-N P2SH (`0x01`) | docs/RFC-001-Multisig.md | ✅ shipped |
| RFC-002 | Native tokens / multi-asset (KVP-102; `asset_id`, format bump → testnet reset) | docs/RFC-002-NativeTokens.md | ✅ shipped |
| RFC-003 | Stealth addresses (`0x03`) + script v2 (`0x02`, bounded stack machine) | docs/RFC-003-ScriptV2-and-Stealth.md | ✅ shipped |
| RFC-004 | HTLC (`0x04`) + Tier Nolan atomic swap (CLTV fix included; no format bump) | docs/RFC-004-Htlc.md | ✅ shipped (2026-09-08, PR #88) |
| RFC-005 | Vault / CSV (`0x05`, real CSV + per-UTXO creation height; checkpoint v6) | docs/RFC-005-Vault.md | ✅ shipped |

KVP specs: `KVP.md` (index/status), `KVP-102-NativeTokens.md`.

---

## Upgrade Phases (cross-repo; from protocol AGENTS.md)

| Phase | Status |
|-------|--------|
| 1 — Foundation & consensus infra | ✅ |
| 2 — Consensus evolution (beacon, past-set pruning, undo log) | ✅ |
| 3 — Performance & scalability | ✅ |
| 4 — Mobile light-node | ✅ |
| 5 — Wallet & security (multisig node + FFI) | ✅ |
| 6 — Operations & reliability | ✅ |
| 7 — P2 polish (incl. fee market & RBF) | ✅ |
| 8 — Stealth + script v2 (RFC-003) | ✅ |
| 9 — HTLC / atomic swap (RFC-004) | ✅ |

---

## Legit Board (P0/P1/P2) — progress view

Source of truth: `kovanica-protocol/docs/LEGIT-BOARD.md`. Baseline intro + RFC-004:
**P1.1 KVP-104 (HTLC) ✅ done 2026-09-08** (PR #88, merged `fb13741`).
Drafted in-repo (P0.3 one-pager, P0.4 tokenomics). Everything else open:

- **P0 (0–2 wks):** public source/release link · GitHub Releases +
  SHA256SUMS + APK · one-pager on web · public tokenomics numbers ·
  testnet reset policy · reliable web deploy · disclaimer.
- **P1 (2–6 wks):** node HTTP `asset_id` (P1.2, KVP-102 e2e) · multiple
  public seeds (P1.3; seed3 live, seed2 pending) · run-a-node guide (P1.4) ·
  public status surface (P1.5) · security threat-model note (P1.6) · open
  issue tracker (P1.7) · spec index (P1.8).
- **P2 (1–3 mo):** audit plan (Q1 2027 target, scope dag+state+RPC) ·
  reproducible builds · bug bounty · mainnet exit criteria ·
  entity/legal · community home · KVP-102 issuance policy · ops hardening ·
  optional polish (hardware wallet on real device, deep-links).

**Definition of “legit v1”:** all P0 + P1.1, P1.2, P1.3, P1.4, P1.7.

---

## Mainnet Track

`docs/MAINNET-CRITERIA.md` (draft, P2.4): date follows criteria, not the reverse.
Key gates: all P0 complete · P1.1–P1.4 complete · audit in progress with no
unfixed Critical/High · ≥3 independent seeds ≥30 days · fork rate < 0.1% ·
block propagation < 2s p95 · monitoring armed · reproducible release
(`v1.0.0-mainnet`) · legal sign-off. Post-launch: 90-day ops/audit/bounty cadence.

---

## Post-Stage 3 (shipped ✅ / active ◀)

1. Light clients / SPV proofs ✅
2. Multi-seed discovery ✅ (code shipped; wiring tracked in TODO.md)
3. Observability & reliability ✅ (Prometheus, tracing, fuzz)
4. **Testnet soak & parameter tuning** ◀ **ACTIVE NEXT** — 24/7 multi-seed;
   measure orphan rate, propagation latency, fork rate, disk growth; tune
   `k`, finality depth, payload pruning depth, difficulty window. seed3 live
   since 2026-08-24; **seed2 deployment pending** (`TODO/seed2-deploy.md`).
5. Wallet & explorer polish ✅

---

## Open / Pending (verified 2026-09-16)

- Seed2 VPS deployment + soak checklist (`/root/kovanica/TODO/seed2-deploy.md`).
- P0.1: public clone — gitlinks exist on GitHub; release/mirror wiring pending.
- `kovanica-ledger-app` + `trezor-coin-def`: **untracked** on disk, not pushed.
- Governance/legality: `ENTITY-LEGAL.md`, `BUG-BOUNTY.md`, `AUDIT-PLAN.md` exist
  as docs; engagement/bookings pending.
- Android unit tests for `Format.kt`, KVLS header parsing, address derivation —
  pending (no local SDK; run in CI).

---

*Never invent roadmap items. Update this file only from verifiable source docs.*