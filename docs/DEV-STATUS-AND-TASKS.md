# kovanica — dev status & tasks

Snapshot date: **2026-10-13**  
Branch: **main** @ **98d88cc**  
Source: full codebase inspection, targeted tests, and a partial full-workspace test run.

## What just landed

- `tui` branch merged (`4be2ff4`) — interactive TUI wallet + amount parser + net helpers.
- `console` branch merged (`29ba439`) — Kovanica Console, faucet CORS hardening, signer package, API client improvements, RPC middleware/write-method token checks.
- Prior local edits committed (`98d88cc`):
  - `config.example.toml`: P2P default `8000`, round `4000` ms, validator key requirement documented.
  - `crates/kovanica-execution/src/call_args.rs`: bincode argument encoders for native contract calls (HTLC, Vault, Multisig, Token).
  - Consensus docs/comments: MysticGhost is the `use_mysticghost` flag on `UniversalCommitter`, not the wrapper module.
  - TA/KS accuracy cleanup in `TASKLIST.md`.
  - `live_vote_integration.rs` rewritten as a non-ignored 2-node commit + transfer harness.

## Build / test evidence

- Rust:
  - `cargo check -p kovanica-cli -p kovanica-rpc -p kovanica-faucet` — ok
  - `cargo check -p kovanica-execution -p kovanica-consensus -p kovanica-node` — ok (warnings only)
  - `cargo test -p kovanica-cli` — 48 unit + 7 binary tests pass
  - `cargo test -p kovanica-rpc` — 40 tests pass
  - `cargo test -p kovanica-faucet` — 10 tests pass
  - `cargo test -p kovanica-node --test live_vote_integration` — 2/2 tests pass
  - `cargo test --workspace` — in progress during the last run; **503+ passing** before timeout in `sustained_liveness_integration`
- TypeScript:
  - `packages/console` tests — 6/6 pass; production build passes
  - `packages/api-client` tests — 5/5 pass
  - `packages/signer` tests — 5/5 pass

## Workspace size stats

| Metric | Value |
|--------|-------|
| Crates | 19 |
| Rust files | 113 |
| Rust LOC | ~53k |
| TS source files (excluding node_modules) | 40 across api-client/console/signer/web |
| Git branch | main @ 98d88cc |
| Pushed to origin | yes |

## Network defaults

| Key | Value |
|-----|-------|
| P2P listen | `0.0.0.0:8000` (config) |
| RPC bind | `127.0.0.1:8545` |
| Max peers | 50 |
| Round duration | 4000 ms (`=>` 12 s per committed leader with wave length 3) |
| Bootnodes | supplied by operator; no hard-coded public hosts |

## Backlog / developer tasks

1. Finish and stabilise the full `cargo test --workspace` run (notably `sustained_liveness_integration`) in CI.
2. Regenerate/commit any missing lock files deliberately (root `package-lock.json` was removed as empty/stray; verify CI `npm ci` behaviour per package).
3. Split the large local-edit commit `98d88cc` if review requires smaller PRs; mapping is in the commit message.
4. Decide whether `KOVANICA_MYSTICGHOST` stays default `false` for testnet; current docs/tests assume linearizer path.
5. Update deployment scripts/Docker defaults to port 8000 if any still reference 9000 (grep done: README fixed, audit note remains intentional).
6. Keep `TASKLIST.md` in lockstep with the VRF reality: primitives exist, but leader scheduling uses round-robin `CommitteeInfo::leader`, not `stake_weighted_leader`.
7. Add coverage for `kovanica-execution::call_args` in contract integration tests.
8. Soak/chaos: re-run 4-node quorum after merge and confirm no regressions from console branch.
9. Cleanup warnings: `std::io::Write` unused import in node main, doc comment on statement in main.

## Notes for future agents

- Hard rule: do not import or mix code from `kovanica-protocol` or older Kovanica repos.
- Do not change tokenomics constants without explicit owner approval.
- Environment defaults and docs now assume **KUNA** (9 decimals), **8000** P2P, **4000 ms** rounds.
