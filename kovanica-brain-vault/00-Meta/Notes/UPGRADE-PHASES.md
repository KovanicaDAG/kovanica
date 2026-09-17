# Kovanica Protocol — Upgrade & Update Phases

> **Execution plan** derived from the oracle upgrade analysis (2026-08-28).
> Constraint: **do NOT start mainnet** — mainnet switch stays **on-sleep-off**
> (dormant/disabled). All mainnet-launch-only actions (live web wiring, genesis
> ceremony, launch checklist, param freeze *for launch*) are excluded; the
> infrastructure that makes mainnet *possible* is built but left dormant.
>
> **Links:** [[ROADMAP]] · [[TODO]] · [[DECISIONS]] · [[AGENTS]]

---

## Scope rules

- **Excluded (mainnet launch only):** A5 (live web wiring), A6 (genesis
  ceremony), I2 (launch checklist), A4's "freeze params for launch" aspect.
- **Mainnet switch on-sleep-off:** the web `source-switch` keeps Mainnet
  disabled ("Launching soon"); the Rust mainnet profile (A1) is built but
  defaults to testnet and is only reachable via an explicit env/flag — never
  activated.
- Every phase ends with validation + an `@oracle` gate + a focused commit on a
  feature branch (draft PR) per `kovanica-protocol/AGENTS.md`.

---

## Phase 1 — Foundation & consensus infra (P0/P1 core)

Items: **A1** (mainnet network profile, dormant) · **A2** (staked-block uplink
on `POST /api/mine/submit`) · **A3** (`GET /api/light_sync` endpoint) · **D1**
(HTTP rate limits + faucet gating) · **H1** (remove dead `mempool.rs`) · **H2**
(AGENTS.md doc sync) · **I1** (RFC-001 multisig spec) · **D2** (cargo audit CI).

Gate rationale: these are the mainnet *prerequisites* and the highest-risk
unknowns (A2 blocks the hybrid production path); they are independent of the
consensus-evolution phases and should land first.

## Phase 2 — Consensus evolution

Items: **B1** (epoch randomness beacon) · **B3** (UTXO undo log) · **B2**
(DAG-level past-set pruning).

Gate rationale: all three are consensus-affecting and need written rationale +
deterministic/adversarial tests per AGENTS.md conventions; they share the
consensus core and should be reviewed together.

## Phase 3 — Performance & scalability

Items: **C1** (incremental on-disk store) · **C2** (incremental sync + API
pagination).

Gate rationale: both address unbounded growth at mainnet scale and build on the
consensus/state work from Phase 2.

## Phase 4 — Mobile light-node (slices 9b–9d)

Items: **E1** (9b wallet UX) · **E2** (9c light sync + persistence) · **E3**
(9d staking uplink — depends on A2).

Gate rationale: the phone-staking production path; E3 is blocked on A2 from
Phase 1.

## Phase 5 — Wallet & security

Items: **B4** (multisig exposure: node + FFI + wallet) · **D4** (web wallet key
custody) · **F1** (multisig in web wallet) · **F2** (wallet-extension build or
remove).

Gate rationale: custody/security surface for real balances; shares the multisig
feature (B4) across node/FFI/web.

## Phase 6 — Operations & reliability

Items: **A8** (seed backup automation + restore drill) · **G1** (soak tuning
review) · **G2** (4th off-box seed + tunnel alerting) · **G3** (web deploy
automation) · **G4** (release pinning).

Gate rationale: operational hardening that does not require mainnet to be live.

## Phase 7 — P2 polish

Items: **B5** (fee market & RBF) · **B6** (BPS/k scaling path) · **D3** (fuzz +
property-test expansion) · **D5** (P2P ban persistence) · **E4–E6** (9e–9f +
FFI multisig/Keystore) · **F3** (explorer detail views) · **H3** (criterion
benchmarks) · **I3** (explorer HTTP API reference).

Gate rationale: lower-priority polish; sequenced last.

---

## Status

| Phase | Status | PR / commit |
|---|---|---|
| 1 — Foundation & consensus infra | ✅ completed | dormant mainnet profile, staked uplink, light_sync, rate limits, dead `mempool.rs` removed, cargo-audit CI (D2) |
| 2 — Consensus evolution | ✅ completed | B1 epoch beacon (#49) · B2 DAG-level past-set pruning (#36, #51) · B3 UTXO undo log (#50); node/explorer integration landed |
| 3 — Performance & scalability | ✅ completed | merged `c85013a` (#37) |
| 4 — Mobile light-node | ✅ completed | merged `84b6516` (#38); Oracle follow-ups (seed/address mismatch, UI-state cleanup, NodeUrl/lastSyncedBlockId wiring, importWallet dedupe) fixed and merged |
| 5 — Wallet & security | ✅ completed | multisig node layer + FFI bindings: `8a3bec6` (#41) |
| 6 — Operations & reliability | ✅ completed | operations automation: `840e8f1` (#40) |
| 7 — P2 polish | ✅ completed | see breakdown below |

### Phase 7 breakdown

| Item | Status | PR |
|---|---|---|
| Android background sync + Keystore hardening | ✅ completed | merged `f8d261f` (#39) |
| Soak snapshot docs (genesis hash, rate recovery, no retune) | ✅ completed | merged `cee3e98` (#45) |
| Fuzz/property tests + Criterion benchmarks + P2P ban persistence | ✅ completed | merged `9407d24` (#43) |
| Explorer detail views + API docs | ✅ completed | merged `16ff775` (#44) |
| Fee market & RBF | ✅ completed | merged `49dfce0` (#46) |
| Web wallet custody + multisig UI | ✅ closed as superseded | UI merged via `0830c39` (#48); #47 closed |

> **Follow-up note:** All Phase 7 PRs merged to `main`. Android unit tests for `Format.kt`, KVLS header parsing, and address derivation remain pending — no SDK/device available to run them locally.

### Phase 4 follow-ups (tracked in PR #38)

- Fix the BIP39/Ed25519 wallet address vs. u64 actor-address mismatch so wallet funds can actually be bonded/staked (`bond_stake_from_secret` / `unbond_from_secret` FFI variants).
- Clean up UI state: separate `validatorSeedSet`, `hybridEnabled`, `bondedStakeAtoms`; stop setting `isValidatorEnabled` on bond.
- Remove or wire dead code (`NodeUrl.kt`, `lastSyncedBlockId` pref); dedupe `importWallet`/`importMnemonic`.
- Add Android unit tests for `Format.kt`, KVLS header parsing, and address derivation once the host/SDK allows.
