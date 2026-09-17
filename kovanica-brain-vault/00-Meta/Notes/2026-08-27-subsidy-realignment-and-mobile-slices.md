# 2026-08-27 — Subsidy Realignment, Mining Endpoints & Mobile Light-Node Landing

> **Links:** [[ROADMAP]] · [[TODO]] · [[DECISIONS]] · [[FACTS]] · [[TESTNET]]

---

## Shipped

| Item | Status | Notes |
|------|--------|-------|
| Subsidy raised to **200 KVNC/block** (was 50) | ✅ Done | `kovanica-dag/src/params.rs`; `alpha.snap` updated |
| Halving era changed to **500,000 blocks** (was 1,000) | ✅ Done | First halving now at ~500k blocks instead of 1k |
| TAP micro-faucet removed project-wide | ✅ Done | Endpoint, `data/taps.txt`, `KOVANICA_TAP` env all gone; open faucet (`POST /api/faucet`) remains |
| Mining template + submit endpoints | ✅ Done | `GET /api/mine/template` (block template), `POST /api/mine/submit` (submit mined blocks) |
| Dynamic fee estimation | ✅ Done | `POST /api/fee_estimate` — DAG tip congestion based |
| Hardware wallet integration | ✅ Done | WebHID/Trezor in web explorer (commit `1271a75`, `93f100e`) |
| Mobile light-node slices 4–8 landed | ✅ Done | Workspace bumped to **v0.2.0**; commits `d06962f` → `51c4520` |
| `kovanica-hash` + `wallet-extension` consolidated | ✅ Done | Merged into protocol; stale vault mirrors removed (commits `e98a31a`, `b03aee1`) |
| seed3 in default P2P_BOOTSTRAP | ✅ Done | `seed3.kovanica.online:9000` added to defaults (commit `5cfb597`) |
| Web frontend sync | ✅ Done | Hardware wallet UI, pool page, credit system, updated components |
| Tuning review script | ✅ Done | VPS soak analysis script for parameter tuning (commit `7e80977`) |
| kovanica-cli + Windows assets | ✅ Done | CLI in public mirror + release; Windows x86_64 target (PR #30) |

---

## Lessons burned in

- Subsidy/halving parameters are a **protocol-level constant** — changing them requires a snapshot regeneration (`alpha.snap`) and coordinated redeploy across all seeds.
- TAP removal was clean because it was an isolated feature (endpoint + env + rate-limit store). Open faucet is independent and remains.
- Hardware wallet integration lives in the web frontend layer, not the node — it uses WebHID for direct USB communication with Trezor.
- Repo consolidation (`kovanica-hash`, `wallet-extension`) means the protocol repo is now the single source of truth for all client-side code.

---

## Open follow-ups

- [ ] Tuning review after 1–2 weeks of soak data (`k`, finality depth, payload pruning depth, difficulty window)
- [ ] Verify all three seeds mine with new 200 KVNC subsidy (check `alpha.snap` alignment)
- [ ] Pool integration testing with new `mine/template` + `mine/submit` endpoints
- [ ] Document fee estimation algorithm in OPERATIONS.md or FACTS.md
