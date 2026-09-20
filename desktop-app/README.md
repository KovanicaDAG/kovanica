# kovanica-desktop — Kovanica Desktop Node App

Cross-platform (Windows / Linux / macOS) desktop application that runs an
embedded Kovanica node and provides a full UI for every protocol feature:
wallet, transfer, mining, staking (hybrid PoW + VRF), multisig, native tokens,
stealth, script v2, HTLC/atomic swap, vault, DAG explorer, SPV light mode.

The master plan lives in the [meta repo plans](../../../plans/desktop-node-app.md)
and, once implementation is underway, its own `docs/plans/desktop-node-app.md`.
This crate is the **Rust core**; a Tauri shell (UI) is added in later slices.

## Status — Slice A (gate verified ✅)

- [x] `desktop-app/` standalone Cargo crate (path deps on `kovanica-node` /
      `kovanica-state`; deliberately **not** a root-workspace member, mirroring
      `android-light-node/`, so the future Tauri/system-webview deps never
      disturb the protocol workspace gates).
- [x] `NetworkProfile` — the **verified live** genesis parameters (testnet) plus
      a dormant RFC-006-era mainnet placeholder.
- [x] `NodeService` — synchronous embedded `Node` booted with the **exact**
      genesis path that reproduces the live chain (`genesis_with_finality`,
      `treasury: None`).
- [x] **Genesis-parity gate** — the embedded node reproduces the live genesis
      `3beecbeb…b74056e` byte-for-byte:
  - offline regression test `tests/genesis_parity.rs` (fixtures captured from
    the live network 2026-09-17);
  - live spike `examples/genesis_parity_live.rs`: `cargo run --example
    genesis_parity_live` (PASS against the live network on 2026-09-17);
  - construction probe `examples/probe_genesis.rs` (documents *why* these
    parameters are the live ones).
- [ ] Tauri shell scaffold + minimal node-status screen (next step).
- [ ] Slice B: node lifecycle (worker thread, data dir + network markers,
      snapshot/checkpoint persistence, event stream).

## Run the gates

```sh
cd desktop-app
cargo fmt --check
cargo clippy --all-targets
cargo test
cargo run --example probe_genesis          # offline: shows which params match live
cargo run --example genesis_parity_live    # network-dependent; prints PASS/FAIL
```

## Which genesis parameters are live? (important)

The deployed testnet is a **pre-RFC-006-era chain** running under RFC-006-era
code. The parity gate proved (2026-09-17) that the live genesis
`3beecbeb…b74056e` is reproduced **only** by:

| Parameter | Live value |
|---|---|
| `k` | 3 |
| genesis subsidy | `200 * ATOM` (200 KVNC/block) |
| founder premine | `200 * ATOM` (200 KVNC) |
| founder seed | 1 |
| treasury | **none** |
| finality / payload pruning depth | 100 / 1000 |

The RFC-006-era values the protocol `main` describes (10 KVNC/block, 200,000
KVNC premine, treasury vaults) produce a **different** genesis (`9565fc20…`) and
are **not** live on the network. This matches `AGENTS.md` (Slice 9a), which
already pins `LightConfig { k:3, subsidy:200*ATOM, founder_amount:200*ATOM,
founder_seed:1 }` as the byte-for-byte live reproducer.

⚠️ **Do not "upgrade" `NetworkProfile::testnet()` to `RFC006_*` constants** —
`live_subsidy_and_premine_are_the_old_era_values` guards this. When the network
is deliberately reset to RFC-006-era parameters, update
`NetworkProfile::testnet()`, `tests/genesis_parity.rs`, and
`examples/probe_genesis.rs` **together**.

## Refreshing the parity fixture

The offline test bakes the live genesis id. It only changes on a deliberate
protocol reset. Refresh procedure:

1. `curl https://explorer.kovanica.online/api/bootstrap` and `/api/head`.
2. Update `LIVE_GENESIS` and the constants at the top of
   `tests/genesis_parity.rs`, plus `NetworkProfile::testnet()`.
3. Re-run `examples/probe_genesis.rs` and confirm exactly one candidate prints
   `MATCH` for the new genesis id.
4. Confirm the profile still matches the deployed chain's running subsidy from
   `/api/head → subsidy` (the chain, not `/api/bootstrap → light_config`).

## Operational findings (2026-09-17)

- The live testnet genesis is **`3beecbebb6103ee24d1617fd87e920c949d613febbbcf6ca1453f3a4bf74056e`**.
  The `9565fc20…` hash in the protocol repo's `OPERATIONS.md` is the
  **RFC-006-era-with-treasury** genesis, not the live chain's — the deployed
  seed was **not** reset to RFC-006-era parameters.
- The deployed seed exposes an internal inconsistency: `/api/bootstrap →
  light_config` reports RFC-006-era numbers (`subsidy: 10 KVNC`, `premine:
  200,000 KVNC`), while `/api/head → subsidy` reports the **running schedule**
  (`200 KVNC`) and the chain has minted ~`blocks × 200 KVNC`. The seed is
  RFC-006-era code serving a pre-reset (old-era) chain. This is why the
  protocol repo's `RFC006_*` constants cannot be used as the parity fixture.
- `/api/head` also returns extra fields (`native_minted`, `total`,
  `circulating`, `burned`, `max_supply`) not produced by this checkout's
  `explorer.rs` handler.