# kovanica-desktop — Dev Playground (client-only / consensus-safe)

Status: bundle complete, binary `target/release/kovanica-desktop` (24M, Tauri).
Classification of all work here: **client-only / consensus-safe** (embedded node; no GHOSTDAG/UTXO/Ed25519 rules changed; PoA-only alignment per 2026-09-25 ratification).

## Quick commands

```bash
# Verify build (lib + all-targets)
cd /root/kovanica/protocol/desktop-app
cargo check --offline --lib
cargo check --offline --all-targets
cargo clippy --offline --lib

# Tauri Linux bundle (needs libwebkit2gtk-4.1-dev + libgtk-3-dev)
./ui/node_modules/.bin/tauri build --no-bundle

# Verify live params before any release
curl -s https://explorer.kovanica.online/api/head | jq '{height, finality_depth, genesis}'
```

## Environment (documented)

| Variable | Meaning | Default / Value used |
|---|---|---|
| `KOVANICA_DATA` | Node data dir | `$PWD/data` (preserve after genesis) |
| `KOVANICA_POW` | PoW flag (being removed per PoA) | `1` (deprecated; use PoA) |
| `KOVANICA_MINE` | Mining (target-for-removal) | `0` |
| `KOVANICA_FAUCET` | Faucet (keep `0` on public nodes) | `0` |
| `KOVANICA_ALLOW_RESET` | Never `1` on public-facing nodes | `0` |
| `KOVANICA_PEERS` | DNS seed only, never orange-cloud | `seed.kovanica.online:9000` |
| `KOVANICA_LISTEN` | TCP 9000 P2P plaintext only | `0.0.0.0:9000` |

## Hard rules (RFC-006 — do not override)

- MAX_SUPPLY = 90.2M KVNC (`9_020_000_000_000_000` atoms)
- Coinbase maturity = 100 blocks
- Fee split = 75% burned / 25% to producer
- Fee floor = `max(1, subsidy / 500_000)` atoms/byte
- Emission: `s0 = 10 KVNC`, era = 2_000_000, α = 3/4
- GHOSTDAG `k = 3`
- Ed25519 signatures (64-byte / 128 hex)

## Consensus / ledger impact checklist

Every new desktop feature must classify:
- [ ] **consensus-safe** (no dag/state rules changed)
- [ ] **ledger-safe** (UTXO / supply invariant preserved)
- [ ] **client-only** (this crate is always this)

Changes here have been **client-only** (embedded node, PoA-only removal of dead hybrid/mining/staking APIs).

## Private keys

Never enter the node. Client-side only (`KeyPair`, `keyring`). Standard flow: `POST /api/prepare` → offline Ed25519 sign → `POST /api/submit`.

## Cross-platform notes

- Linux: binary + Tauri bundle done (GTK installed).
- Windows: requires `mingw` / `cross` CI runner.
- Alpine: `installer/linux/alpine/install.sh` verified (`bash -n`, musl target, builds from source if pre-built missing).

---
UPDATED: 2026-09-26 — desktop-alp build complete (binary 24M, Tauri bundle, 26 dead tests stubbed, SDK sync, commit e21303b). Deploy stage: ready for pm2/release server (not public endpoint; no fixture needed). Consensus: client-only / consensus-safe.
