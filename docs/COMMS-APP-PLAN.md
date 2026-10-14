# COMMS-APP-PLAN — Decentralized calls / video / messaging on Kovanica

> Status: DRAFT (scaffold) · Scope owner: Kovanica project owner · Repo: `kvnc` · Target branch: `VPS3`
> Chain: Kovanica · Native token: KUNA · Crate prefix: `kvnc-*`
> **Decisions locked (§8.1–8.2):** first client = **Tauri desktop** (Rust core); messaging transport =
> **minimal store-and-forward relay + OpenMLS (RFC 9420)**.

---

## 1. Goal & non-goals

### Goal
A communications app (1:1 and small-group voice calls, video calls, text messaging) whose
**identity, discovery, payments, spam resistance, and relay incentives** are anchored on the
Kovanica L1, while **all real-time traffic and message content stays off-chain and end-to-end
encrypted**.

### Non-goals (things that will NEVER go on-chain)
- Message payloads and media (audio/video, RTP).
- Contact lists, presence, typing indicators, read receipts (metadata).
- Group encryption keys / ratchet state.
- Any bulk or streaming data.

Kovanica is a **control plane**, never a **data plane**. A blockchain is the wrong transport for
real-time media: per-unit fees, ~12 s committed-leader cadence, no bandwidth, and permanent public
storage. Media and messages flow over WebRTC + an E2E messaging transport; the chain decides
**who you are, what you pay, and who gets paid**.

---

## 2. Architecture

```
            ┌──────────────────────────── CONTROL PLANE (on Kovanica) ───────────────────────────┐
            │                                                                                    │
            │  Identity / DID        Name registry        Payment & escrow      Relay registry    │
            │  (wallet pubkey →      (name → pubkeys,      (HTLC prepaid,         (staked node      │
            │   self-certifying id)   relay endpoints)      per-msg fee/stake)    operators)        │
            │                                                                                     │
            │  Reused: kvnc-htlc            Reused: kvnc-token (plans)   Reused: kvnc-multisig,    │
            │          kvnc-rpc (JSON-RPC + WS subs)                              kvnc-vault         │
            │  NEW (proposed): name-registry contract, comms-escrow contract (see §3.3)             │
            └────────────────────────────────────────────────────────────────────────────────────┘
                       ▲ RPC/WS (read state, submit tx)                 ▲ RPC/WS (settlement)
                       │                                                  │
            ┌──────────┴──────────────────────────────────────────────────┴──────────────────────┐
            │                   CLIENT — Tauri desktop (Rust core)   [DECIDED]                     │
            │  Wallet keys via kvnc-keystore / kvnc-light-ffi (native OS keystore; seeds stay on-box)│
            │  ┌─────────────┐   ┌──────────────────┐   ┌───────────────┐   ┌──────────────────┐   │
            │  │ Identity    │   │ Messaging        │   │ Calling       │   │ Payments         │   │
            │  │ module      │   │ E2E: OpenMLS     │   │ (WebRTC in    │   │ (HTLC prepaid,   │   │
            │  │             │   │  (RFC 9420)      │   │  webview)     │   │  plan tokens)    │   │
            │  └─────────────┘   └──────────────────┘   └───────────────┘   └──────────────────┘   │
            └────────────────────────────────────────────────────────────────────────────────────┘
                       │ signed ciphertext envelopes (off-chain)   │ media (SRTP, off-chain)
                       ▼                                            ▼
            ┌────────────────────────────────────────────────────────────────────────────────────┐
            │              DATA PLANE  (off-chain, E2E-encrypted, never stored on-chain)          │
            │   Store-and-forward relays (Nostr-style opaque envelopes)  WebRTC SFU / TURN relays │
            │   [DECIDED] minimal relay protocol, HTLC-paid              (paid via HTLC)           │
            └────────────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Component inventory — REUSE vs NEW

| Component | Reuse | New (proposed) | Notes |
|---|---|---|---|
| Wallet / signing / keys | `@kvnc/signer` (`ed25519.ts`, `keys.ts`, `tx.ts`, `address.ts`) | — | Client-side only; seeds never leave device |
| Desktop client shell | `kvnc-lightnode`, `kvnc-keystore`, `kvnc-light-ffi` | **NEW** Tauri app (`apps/desktop`) | **[DECIDED §8.1]** Rust core; native OS keystore |
| Chain access | `@kvnc/api-client` (`index.ts`) | — | Plus WS subscriptions from `kvnc-rpc/src/subscriptions.rs` |
| RPC spec | `kvnc-rpc/src/openapi.rs` (OpenAPI 3.1) | — | Keep client bindings generated from spec |
| Prepaid / pay-for-service | `crates/kvnc-htlc` (escrow, `hash_lock`, claim/refund) | — | Native-balance escrow to relay/escrow addresses |
| Plans / entitlements | `crates/kvnc-token` (fungible multi-asset) | — | One token instance per data plan |
| Escrow / treasury rules | `crates/kvnc-multisig`, `crates/kvnc-vault` | — | Relay pools, discretionary payouts |
| Mobile SDK | `crates/kvnc-light-ffi` (UniFFI) + `kvnc-lightnode` | — | ⚠ Argon2id 64 MiB runs on calling thread → never on UI thread |
| E2E messaging crypto | — | **NEW** OpenMLS (RFC 9420, Rust) | **[DECIDED §8.2]** covers 1:1 + groups uniformly |
| Name registry | — | **NEW contract** `kvnc-name-registry` (WASM) | ledger-safe; name → pubkey/relay endpoints |
| Comms escrow (per-msg / streaming) | HTLC covers most | **NEW contract** `kvnc-comms-escrow` (proposed) | Only if HTLC proves insufficient; justify first |
| Client comms package | — | **NEW** `packages/comms` (TS glue) + Rust core | Messaging + calling + payments glue |
| Messaging relays | — | **NEW infra**: store-and-forward relay (Nostr-style, opaque signed envelopes) | Off-chain; HTLC pay-to-publish/relay |

> **New-crate justification rule (AGENTS.md #7):** each NEW crate above must have a one-paragraph
> justification in its PR. Prefer extending `kvnc-token`/`kvnc-htlc` patterns before adding contracts.

---

## 3. Layer classification

| Feature | Classification | Rationale |
|---|---|---|
| Comms identity (wallet pubkey → self-certifying id) | **client-only** | Derived locally; no state |
| Message transport + E2E crypto | **client-only** | Off-chain, E2E encrypted |
| Name registry contract | **ledger-safe** | New state, no consensus-rule change |
| Relays / TURN / SFU | **client-only / infra** | Off-chain, no chain coupling |
| WebRTC signaling | **client-only** | Transported over messaging channel |
| Per-message fee / stake-to-send | **ledger-safe** | New escrow/stake tx flow |
| Prepaid HTLC top-up | **ledger-safe** | Reuses `kvnc-htlc` |
| Data-plan entitlement token | **ledger-safe** | Reuses `kvnc-token` |
| Relay registry + bonding | **ledger-safe** | New registry state |
| Relay payment settlement | **ledger-safe** | HTLC settlement |
| Group key management (MLS) | **client-only** | Local ratchet state |
| Mobile FFI packaging | **client-only** | SDK surface |
| eSIM / top-up provisioning (§M7) | **ledger-safe payment leg + external** | Chain pays; SM-DP+ issues profile off-chain |

**Nothing here is consensus-critical.** If any proposal requires a consensus-rule change, stop and
escalate — it is out of scope for this app.

---

## 4. Phased milestones

Effort: S ≤ 1 wk · M ≈ 2–4 wk · L ≥ 1 month (one dev, part-time assumed).

### M0 — Foundation & identity (client-only)
- **Scope:** Tauri desktop shell + headless CLI harness, wallet identity, chain connectivity.
- **Deliverables:**
  - `apps/desktop` Tauri shell (Rust core via `kvnc-lightnode` / `kvnc-keystore` / `kvnc-light-ffi`)
  - Headless CLI harness (extend `kvnc-cli`) to exercise identity/transport/payments in CI, no GUI
  - `packages/comms/src/{index,identity,rpc}.ts` (TS glue for the webview)
  - Identity: `comms_id = hash(pubkey)` (BLAKE3, matches `kvnc_common::hash` domain style)
  - RPC wrapper over `@kvnc/api-client` + WS subscriptions
  - `packages/comms/src/identity.test.ts`
- **Exit criteria:** load wallet via native keystore, derive comms id, read chain head over RPC, tests pass;
  CLI harness exercises the same path headlessly.
- **Deps:** none · **Effort:** S

### M1 — 1:1 E2E text MVP (client-only + ledger-safe registry)
- **Scope:** E2E 1:1 messaging over an off-chain store-and-forward relay; on-chain name resolution.
- **Deliverables:**
  - Store-and-forward **relay** (Rust): accepts opaque, signed ciphertext envelopes keyed by `comms_id`,
    delivers on reconnect, expires old envelopes. **[DECIDED §8.2]**
  - E2E crypto module: **OpenMLS (RFC 9420)** integration (`crates/kvnc-comms-crypto` proposed) — one
    scheme for 1:1 and groups.
  - `packages/comms/src/{messaging,relay,envelope}.ts` (envelope framing + relay client)
  - `crates/kvnc-name-registry` (WASM contract): `register(name, pubkey, relay_endpoints)`, `resolve(name)`
  - TS binding + `packages/comms/src/registry.ts`
- **Exit criteria:** two clients exchange E2E messages via relay (offline recipient receives on reconnect);
  resolve a peer by name from chain; ciphertext only on the wire; integration round-trip test.
- **Deps:** M0 · **Effort:** M

### M2 — Payments & spam resistance (ledger-safe)
- **Scope:** pay-to-send / stake-to-send; prepaid balance; plan entitlements.
- **Deliverables:**
  - `packages/comms/src/payments.ts` (HTLC create/claim/refund flow using `@kvnc/signer`)
  - Prepaid path reusing `crates/kvnc-htlc`; plan tokens via `crates/kvnc-token`
  - Anti-spam policy module (min-fee / stake threshold)
- **Exit criteria:** unpaid flood rejected; paid message accepted; HTLC settlement + refund tests.
- **Deps:** M1 · **Effort:** M

### M3 — 1:1 voice calls (client-only)
- **Scope:** WebRTC audio inside the Tauri webview; signaling over the M1 relay channel; paid TURN.
- **Deliverables:** `packages/comms/src/rtc/{signal,peer,turn}.ts`; TURN relay with HTLC-funded access.
- **Exit criteria:** 1:1 audio call across NAT via paid TURN; **zero media on-chain**.
- **Deps:** M1 (+ M2 for paid TURN) · **Effort:** M

### M4 — Video + small group (client-only)
- **Scope:** video calls; ≤ 4-party mesh or SFU; group keys via the M1 OpenMLS module.
- **Deliverables:** `packages/comms/src/rtc/sfu.ts`; `packages/comms/src/group/mls.ts` (thin wrapper on OpenMLS).
- **Exit criteria:** 4-party video; group key rotates on join/leave; rekey test.
- **Deps:** M3 · **Effort:** L

### M5 — Relay incentives & decentralization (ledger-safe)
- **Scope:** staked/bonded store-and-forward relay operators; HTLC streaming payment; reputation.
- **Deliverables:** relay registry contract + `packages/comms/src/rewards.ts`.
- **Exit criteria:** an independent relay earns via HTLC; app works with no single operator dependency.
- **Deps:** M2, M3 · **Effort:** L

### M6 — Mobile packaging & hardening (client-only)
- **Scope:** iOS/Android via `kvnc-light-ffi` (UniFFI).
- **Deliverables:** mobile bindings; off-UI-thread key ops (Argon2id 64 MiB constraint); push notifications.
- **Exit criteria:** mobile build runs; no seed leaves device; key ops off UI thread.
- **Deps:** M1–M4 · **Effort:** L

### M7 — Top-up / eSIM integration (ledger-safe + external)
- **Scope:** on-chain payment (HTLC) → external certified SM-DP+ releases eSIM activation code.
- **Deliverables:** `packages/comms/src/topup.ts` + provider integration.
- **Exit criteria:** end-to-end demo with a certified provider; profile issuance off-chain.
- **Deps:** M2; external partner · **Effort:** M (+ partner timeline)

---

## 5. Dependency graph

```
M0 ──► M1 ──► M2 ──┬──► M3 ──► M4 ──┐
                   │                 ├──► M6 (mobile)
                   └──► M5 ──────────┘
                    M2 ──► M7 (eSIM/top-up)
```

MVP cut line = **M0 + M1 + minimal M2** (§7). Everything from M3 up is post-MVP.

---

## 6. Risk register

| # | Risk | L | I | Mitigation | Owner lane |
|---|---|---|---|---|---|
| R1 | Sybil / spam without cost | H | H | Min per-msg fee or stake-to-send; relay-side rate limits; proof-of-work fallback for non-paying senders | client-wallet |
| R2 | Per-message payment UX friction | H | M | Prepaid balance (HTLC deposit); batch settlement; free tier between mutual contacts | client-wallet |
| R3 | NAT traversal / relay cost | H | H | Paid TURN; ICE fallback chain; measure relay $/call | node-ops / infra |
| R4 | Group E2E key management (MLS) complexity | M | H | Adopt MLS (RFC 9420) rather than roll-our-own; audit | security-audit |
| R5 | Relay incentivization economics | M | H | HTLC streaming pay; bonded relays; reputation; treasury bootstrap | project-plan |
| R6 | Key/seed handling on mobile | M | H | `kvnc-light-ffi`; Argon2id 64 MiB off UI thread; zeroize; no seed export | security-audit |
| R7 | Regulatory / telecom / eSIM (M7) | M | H | Partner with certified SM-DP+; chain = payment leg only | project-plan |
| R8 | Chain throughput/latency misuse | M | M | Keep chain to control plane; never per-media-tick tx; 12 s cadence is fine for settlement | reviewer |
| R9 | Mobile app-store crypto-payment policies | M | M | Separate "buy plan" flow; external purchase where required | project-plan |
| R10 | Metadata privacy (who-talks-to-whom) | H | H | Sealed sender, no plaintext contact graph on-chain, relay blind to identities | security-audit |
| R11 | On-chain discovery enumeration | M | M | Names via registry, pubkeys discoverable by design; avoid publishing contact graph | client-wallet |

---

## 7. MVP scope vs later

**MVP (ship first):**
- **Tauri desktop client** (§8.1) + a headless CLI harness.
- 1:1 **text only**, E2E encrypted via **OpenMLS** over the store-and-forward relay (§8.2).
- Identity from wallet keys; name resolution from chain.
- Minimal pay-to-send / prepaid to prove spam resistance.

**Later:** voice (M3) → video/group (M4) → relay incentives (M5) → mobile (M6) → eSIM/top-up (M7).

---

## 8. Decisions & open questions

### Locked
1. **First client form factor → Tauri desktop** (Rust core, native OS keystore), plus a headless CLI
   harness for CI. *Rationale:* reuses `kvnc-lightnode` / `kvnc-keystore` / `kvnc-light-ffi` for
   identity + signing, gives real key custody, WebRTC works in the webview for M3+, and the same core
   is reused for mobile via UniFFI (M6) — while avoiding app-store friction early (R9).
2. **Messaging transport → minimal store-and-forward relay + OpenMLS (RFC 9420)** for E2E (1:1 and
   groups). *Rationale:* opaque relays are HTLC-incentivizable, `comms_id = hash(wallet pubkey)` is
   wallet-native, offline delivery works, and one E2E scheme covers 1:1 + groups.
   **Avoid** raw libp2p/gossipsub as the primary transport (no offline store-and-forward).
   **Fallback:** Matrix (`matrix-rust-sdk`) only if off-the-shelf E2E maturity outweighs relay
   incentives and the heavier federated homeserver ops.

### Locked (continued)
3. **§8.3 Launch relay model → self-hosted first.** Only our relays in phase 1; open incentivized
   third-party relays at M5.
4. **§8.4 Plan/entitlement model → native KUNA only for MVP.** Add a KVP-102 token later only if needed.
5. **§8.5 Name resolution → on-chain ledger-safe contract** (`kvnc-name-registry`): name → pubkey /
   relay endpoints.
6. **§8.6 Anti-spam / fee model → prepaid balance + free tier between mutual contacts.** HTLC-funded
   prepaid balance; micro-fee per message beyond the free tier.
7. **§8.7 Telecom / eSIM → deferred to post-MVP.** M7 stays in the plan; no action until M2/M5 lands.
8. **§8.8 Consensus changes: none contemplated.** (Reaffirmed.)

### M0 build decisions (locked)
- **Layout:** `apps/desktop` (Tauri shell, React + Vite) + `crates/kvnc-comms-core` (shared Rust core)
  + `packages/comms` (TS glue). New top-level `apps/` directory.
- **Webview framework:** React 18 + Vite 5 + TypeScript (matches `@kvnc/web`, `@kvnc/console`).
- **Headless harness:** `comms` subcommand added to `kvnc-cli`.
- **Dev/test target:** local throwaway node (never the running `kvnc-node seed3`).
- **Identity:** `comms_id = blake3(b"kvnc/comms/id" || pubkey)` — domain-separated, matches the
  `kvnc-htlc`/`kvnc-token` prefix convention.

---

## 9. TASKLIST — concrete first tasks

- [x] First client form factor decided: **Tauri desktop** + CLI harness (§8.1).
- [x] Messaging transport decided: **store-and-forward relay + OpenMLS** (§8.2).
- [x] Scaffold `apps/desktop` (Tauri v2 + React 18 + Vite 5) and the shared `crates/kvnc-comms-core`.
- [x] Extend `kvnc-cli` with the headless M0 harness: `kvnc comms id --keystore <f> [--json]`.
- [x] Scaffold `packages/comms` (TS glue) with `src/index.ts`, wired to `@kvnc/api-client`.
- [x] Implement `packages/comms/src/identity.ts` + `kvnc-comms-core` (same BLAKE3 `comms_id`,
      pinned by a shared known-answer vector) with tests on both sides.
- [x] Wire the WS subscription helper from `kvnc-rpc/src/subscriptions.rs` into `packages/comms/src/rpc.ts`
      (re-exported; live use lands with M1).
- [ ] Prototype store-and-forward relay + OpenMLS envelope exchange (throwaway) to prove 1:1 transport before touching chain.
- [ ] Draft `crates/kvnc-name-registry` contract spec (entry points, storage keys, error codes) matching `kvnc-token`/`kvnc-htlc` conventions.
- [ ] Write HTLC top-up flow (`payments.ts`) against `kvnc-htlc` API; unit-test claim/refund.
- [ ] Produce a threat model doc (R1, R4, R6, R10) before any voice/video work.
- [ ] Confirm branch policy: all of the above committed to `VPS3`.

---

## Appendix A — Assumptions

- This repo (`/root/kvnc`) is **development only**; a `kvnc-node seed3` is running on this machine and
  is not to be disturbed. All work commits to branch `VPS3`.
- Independent codebase: no code/design imported from `kovanica-protocol` or other chains.
- Tokenomics untouched. 2–4 GB RAM footprint target honored on client and node.
- `kvnc` contracts (`kvnc-htlc`, `kvnc-token`) are WASM entry-point modules with bincode args/results;
  new contracts follow the same ABI and error-packing conventions.
