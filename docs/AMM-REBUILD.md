# AMM (KUNA ↔ token pool) — what must be rebuilt

The pool shipped in #134/#135 (#61eab76…c10980d) is **contract calls, not RPC
methods**: `token_init_pool`, `token_buy`, `token_sell` live in
`crates/kovanica-token` and are reached through a signed `TransactionKind::Call`,
the same path as `token_transfer`. The node's `token_*` RPC methods
(`crates/kovanica-rpc/src/lib.rs`) deliberately do not mirror them, so no node
RPC surface changes.

What changes is therefore the **artifacts that embed or describe the contract**.

## Artifact of record

| Item | Value |
|---|---|
| File | `contracts/kovanica-token.wasm` |
| sha256 | `a6988f5e1742b1ac0a069a3ef274d7590a54266dee57435438664e23036dc49b` |
| Size | 91,797 bytes (was 79,565 before the pool) |
| Rebuilt in | #135 (`94fdf88`) — do not rebuild unless the Rust source changes |
| Entry points | 19 in `crates/kovanica-execution/src/contracts.rs` (`ENTRY_POINTS`), the pool adds the last three |

## Surface checklist

| # | Surface | Path | Affected by the AMM? | Action |
|---|---|---|---|---|
| 1 | Contract artifact | `contracts/kovanica-token.wasm` | **yes** — new entry points + `kovanica_call_value` | already rebuilt; keep the sha table in `contracts/README.md` in sync if it changes |
| 2 | Android app | `apps/android/app/src/main/assets/kovanica_token.wasm` | **yes** — byte-identical copy of the artifact | rebuild the APK/AAB and republish; an installed app keeps whichever asset it shipped |
| 3 | Android wallet | `apps/android-wallet/app/src/main/assets/kovanica_token.wasm` | **yes** — same copy | rebuild + republish |
| 4 | API client | `packages/api-client/src/index.ts` (`ENTRY_POINTS`) | **yes** — TypeScript list still has 16 names; the pool's three are missing | add `token_init_pool`, `token_buy`, `token_sell`, bump the package version, republish |
| 5 | Console | `packages/console` | **yes**, indirectly — depends on `@kovanica/api-client` + `@kovanica/signer` | rebuild/redistribute after 4 |
| 6 | Comms package | `packages/comms` | **yes**, indirectly — depends on `@kovanica/api-client` + `@kovanica/signer` | rebuild after 4 |
| 7 | Desktop app | `apps/desktop` (Tauri, `frontendDist: ../dist`) | **yes**, indirectly — imports `@kovanica/comms`/`@kovanica/signer` | rebuild the installer after 4/6 |
| 8 | Web | separate repo `KovanicaDAG/kovanica-web` (deployed site), not `packages/web` | **yes, read path** — `/api/pools` decodes the pool slot and `/pool` renders reserves, fee, LP supply and an exact quote | rebuild + `pm2 reload kovanica-web`; `packages/web` itself still has no pool UI |
| 9 | Signer | `packages/signer` | **no** — no token entry points; the AMM needs a signed `Call.value`, which the CLI builds in Rust | no change |
| 10 | Android comms / iOS | `apps/android-comms`, `apps/ios` | **no** — no wasm asset, no token references | no rebuild |
| 11 | Docs | `contracts/README.md`, `docs/CONTRACTS.md`, `docs/EXAMPLES.md`, `docs/GAS_SCHEDULE.md` | **yes** — updated in #134/#135 | only re-check if the artifact or the gas table changes |

## Verification

```bash
# the three copies must stay byte-identical, and match the README table
sha256sum contracts/kovanica-token.wasm \
  apps/android/app/src/main/assets/kovanica_token.wasm \
  apps/android-wallet/app/src/main/assets/kovanica_token.wasm

# Rust side: does the current source still expose the pool entry points (builds
# the crate for wasm32 inside the test and runs it under Wasmi, so this checks
# the source, not the committed bytes above)
cargo test -p kovanica-execution wasmi_e2e_kovanica_token -- --nocapture

# is anything in the packages/apps actually calling the pool today?
git grep -n -E "token_init_pool|token_buy|token_sell" -- packages apps
```

At the time of writing the last command matches only the wasm binaries: no
TypeScript or app code calls the pool yet, so rows 2 and 3 are the only
mandatory rebuilds and row 4 is the only code change.

## Web surface (`KovanicaDAG/kovanica-web`)

The deployed site is the separate `kovanica-web` repo, not `packages/web`. It now:

- reads pools via `GET /api/pools` (`src/lib/api/tokens.server.ts` → `listPools`,
  decoding `blake3("kvnc/v1/pool")` with the shared 51-byte bincode layout);
- renders `/pool` (`src/components/protocol/pool-view.tsx`) with reserves, fee,
  LP supply and a quote computed by the same constant-product formula as
  `kovanica_token::quote`;
- sends any KVP-102 token from the wallet through `POST /api/token_prepare` →
  offline Ed25519 signature → `POST /api/token_submit`, which encodes a
  `TransactionKind::Call { method: "token_transfer" }` exactly like the CLI.

Executing a swap (`token_buy` / `token_sell`) is still CLI/Android only; the web
page quotes, it does not sign swaps.
