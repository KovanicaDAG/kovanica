---
title: "kovanica-desktop — Kovanica Desktop Node App"
category: 50-Components
source: protocol/desktop-app/README.md
synced: 2026-09-26
---
# kovanica-desktop — Kovanica Desktop Node App

Cross-platform (Windows / Linux / macOS) desktop application that runs an
embedded Kovanica node and provides a full UI for every protocol feature:
wallet, transfer, mining, staking (hybrid PoW + VRF), multisig, native tokens,
stealth, script v2, HTLC/atomic swap, vault, DAG explorer, SPV light mode.

The master plan lives in the [[60-Planning/plans--desktop-node-app|meta repo plans]]
and its own working plan [docs/plans/desktop-node-app.md](../../docs/plans/desktop-node-app.md).
This crate is the **Rust core**; the Tauri shell (UI) is built up in later slices.
Slices 1–4 are landed; **Slice 5 (staking & mining cadence) is in progress**.

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
- [x] Tauri shell scaffold + minimal node-status screen (node-status dashboard
      with block/tip/mempool/peer counters and a live event stream; Tauri
      `invoke` handlers for status, block production, tx submission,
      snapshot/checkpoint, shutdown, wallet create/unlock/lock/addresses/
      send/balance/history, and P2P start/stop; wallet panel wired
      create/unlock/lock/addresses/balance/history/send via the handlers;
      operations panel driving whole-file snapshots (`Node::save`), finality
      checkpoints and a graceful worker shutdown with app exit).
- [x] Slice B: node lifecycle (worker thread with an mpsc command channel and
      broadcast event stream, data dir + network markers,
      snapshot/checkpoint persistence, tip/block-change events).
- [x] Slice B extension — network, assets and SPV in the worker:
  - **P2P integration**: `start_p2p`/`stop_p2p` now drive real TCP sync
    (mirrors the explorer's `tick_p2p`/`sync_peers`): non-blocking inbound
    listeners serving headers-first (with legacy full-dump fallback) and a
    per-tick outbound headers-first sync round per configured peer; the status
    Peers counter reflects this round's live peers and peer joins/drops surface
    as `PeerConnected`/`PeerDisconnected` events. Defaults: listen
    `0.0.0.0:9000`, bootstrap `seed.kovanica.online:9000`.
  - **Native-token balances**: `get_asset_balances` lists every asset an
    address holds (`balances_map_of` + the asset registry), each with its
    `Fungible`/`NonFungible` kind; the wallet query column shows the per-asset
    breakdown and wallet history now surfaces non-native `asset_id`s.
  - **SPV light mode**: `spv_sync` fetches the KVLS v1 light-sync blob from any
    node's `/api/light_sync` and verifies the whole header chain through a
    `SpvClient` (`require_pow = false`, checkpoint = the blob's first header);
    `spv_matches` answers which light-synced blocks' Golomb-Rice filters MIGHT
    contain a watch address; `spv_verify` pulls `/api/light_proof` and checks
    the Merkle proof against the synced header. Verified end-to-end against the
    live explorer blob (4,953/4,953 headers). KVLS/proof parsing is byte-compatible
    with `kovanica-ffi`, guarded by unit tests.
- [ ] **Slice 5 — staking & mining cadence (worker, Tauri handlers and UI)**:
  - **Validator identity**: `set_validator_seed` parses 32-byte seeds and sets
    the node's VRF validator key (`ValidatorReady` event + surfaced pk).
  - **Hybrid sortition**: `enable_hybrid` mirrors the FFI's
    `HybridConfig { rate_num, rate_den, stake_nominal_work: 1,
    use_epoch_beacon: true, retarget }` (zero rates rejected), so the worker
    can win slots by stake-weighted VRF draw instead of PoW.
  - **Bonding**: `bond_stake` mirrors the FFI's two-step bond — auto-splits an
    oversized unfrozen coin via a mined block, then bonds the requested amount
    via a `KVB1||vrf_pk`-tagged tx (mined/`produce_block`-sealed), freezing it
    into the stake registry with a `ValidatorReady`-class lifecycle.
  - **Unbonding**: `unbond_stake` delegates to `Node::unbond_with` (FIFO over
    matured frozen coins, release auto-sealed in its own block) and surfaces
    `InsufficientStake` before maturity.
  - **Mining cadence**: `start_mining`/`stop_mining` run a live
    `MissedTickBehavior::Skip` interval that produces a block every N seconds
    (staked draw first, PoW fallback); the status Peers/counters clean up.
  - **`get_staking`** reports validator pk, hybrid config, total/my stake,
    chain height, era issuance, pending-unbond height and mining state for the
    Staking/Mining panel.
  - **Staking/Mining UI panel**: validator seed + hybrid controls, bond/unbond
    (KVNC), mining cadence, and a live staking-state readout, all through the
    new `tauri_main.rs` handlers; bonding requires an unlocked wallet.
  - Unit tests: seed parsing, FFI-parity hybrid config, bond-source
    selection (exact coin → split → shortfall), and a full
    bond → maturity → unbond lifecycle on an embedded node (10 lib tests).

## Run the gates

```sh
cd desktop-app
cargo fmt --check
cargo clippy --all-targets
cargo test
cargo run --example probe_genesis          # offline: shows which params match live
cargo run --example genesis_parity_live    # network-dependent; prints PASS/FAIL
```

## Run the app

The Tauri shell needs the System WebView (webkit2gtk-4.1 + GTK3 on Linux),
Node ≥ 18, and the `ui/` dependencies:

```sh
cd desktop-app/ui && npm install

# Dev (Vite HMR on :5173) — from desktop-app:
node ui/node_modules/.bin/tauri dev

# Production build + binary (builds ui/dist first) — from desktop-app:
node ui/node_modules/.bin/tauri build --no-bundle
```

The worker is embedded and spawns on app startup (`NodeHandle::spawn(
NetworkProfile::testnet())`), booting the live genesis under the parity gate.
The frontend subscribes to the `node-event` broadcast and drives everything
through the `invoke` handlers in `tauri_main.rs`; it never touches the `Node`
directly.

## Which genesis parameters are live? (important)

**RFC-006 is LIVE on testnet** (activated 2026-09-20 as a consensus fork that
wiped all pre-RFC-006 balances). The live genesis is now:

| Parameter | Live value (RFC-006) |
|---|---|
| `k` | 3 |
| genesis subsidy | `10 * ATOM` (10 KVNC/block) |
| founder premine | `200_000 * ATOM` (200,000 KVNC = 0.2M KVNC) |
| treasury | 10 × 1M KVNC vaults (placeholder keys) |
| founder seed | 1 |
| finality / payload pruning depth | 100 / 1000 |
| max supply | 90.2M KVNC |
| coinbase maturity | 100 blocks |
| fee split | 75% burned / 25% to producer |

The live genesis hash is **`9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97`**
(per `NETWORK.md` and `/api/bootstrap`).

The old pre-RFC-006 genesis (`3beecbeb…`) is **no longer valid** — the activation
fork reset the chain. `NetworkProfile::testnet()` now uses RFC-006 constants.

## Refreshing the parity fixture

The offline test bakes the live genesis id. It only changes on a deliberate
protocol reset. Refresh procedure:

1. `curl https://explorer.kovanica.online/api/bootstrap` and `/api/head`.
2. Update `LIVE_GENESIS` and the constants at the top of
   `tests/genesis_parity.rs`, plus `NetworkProfile::testnet()`.
3. Re-run `examples/probe_genesis.rs` and confirm exactly one candidate prints
   `MATCH` for the new genesis id.
4. Confirm the profile matches the deployed chain's running subsidy from
   `/api/head → subsidy` (should be 10 KVNC at genesis, decaying geometrically).

## Operational findings (2026-09-21)

- RFC-006 activation fork (2026-09-20) wiped all pre-RFC-006 balances and
  reset the chain to the new genesis `9565fc20…` with subsidy 10 KVNC/block,
  founder premine 0.2M KVNC, treasury 10M KVNC in vaults, max supply 90.2M KVNC.
- The old genesis `3beecbeb…` (subsidy 200 KVNC, founder 200 KVNC) is obsolete.
- `/api/bootstrap` now correctly reports RFC-006-era numbers including
  `subsidy`, `founder_amount`, `founder_seed`, `max_supply`, `native_minted`,
  `total`, `circulating`, `burned`.
- `/api/head` returns `subsidy` (current, decaying), `native_minted`, `total`,
  `circulating`, `burned`, `max_supply` — all matching RFC-006 tokenomics.