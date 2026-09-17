# FACTS — Atomic Quick Reference

> **Links:** [[myObsidianVaultDAG]] · [[OPERATIONS]] · [[TESTNET]] · [[CODE_INDEX]] · [[DECISIONS]]

One-line answers to the questions asked most often. Every fact carries its
source — if reality disagrees with this file, **reality wins**: fix the source,
then this file.

---

## Network Identity

| Fact | Value | Source |
|---|---|---|
| Network name | `kovanica-testnet` | `TESTNET.md` |
| Native token | **KVNC**, 8 decimals; 1 KVNC = 10^8 atoms | `TESTNET.md` |
| Genesis hash | `76cc019de947cb9f6b2abe9428dc120bbf6f3ee3c3f0be89efa83a4e3af3c140` | `OPERATIONS.md` §1 |
| Premine | 50 KVNC (founder) | `TESTNET.md` |
| Subsidy | cap 50 KVNC/block, halves every 1000 blocks | `TESTNET.md` |
| Min fee | 0.0001 KVNC at genesis | `TESTNET.md` |
| GHOSTDAG `k` | 3 | `TESTNET.md` |
| PoW | opt-in (`KOVANICA_POW=1`), Nakamoto `H * work < 2^256` | `TESTNET.md`, protocol AGENTS §Roadmap |
| Address format | `kvnc…dag` human encoding | `myObsidianVaultDAG.md` |
| Consensus paper | GHOSTDAG (Sompolinsky, Wyborski & Zohar) | `myObsidianVaultDAG.md` |

## Hosts & Services (all seeds per `OPERATIONS.md` §1)

| Thing | Where | Notes |
|---|---|---|
| VPS | `srv1745734` = `145.223.116.178` | SSH on **:2222** (not 22) |
| seed1 (primary) | pm2 `kovanica-explorer` | P2P `:9000`, HTTP loopback `:8080`, mines 1 blk/min |
| seed2 (validation) | systemd `kovanica-seed2` | P2P `:9001`, HTTP loopback `:18080` |
| seed3 (off-box) | AWS EC2 t3.micro, eu-north-1 = `3.79.148.71` | systemd `kovanica-seed3`, mining on |
| web | pm2 `kovanica-web` → `127.0.0.1:3010` | built `npm run build:vps` → `/root/kovanica-web/.output` |
| nginx | `/etc/nginx/sites-enabled/explorer.kovanica.online` | `/api/*`→`:8080`, pages→`:3010`, `/download/*`→`/var/www/kovanica-dist/` |
| Public binaries | `/var/www/kovanica-dist/{kovanica-node-linux-x64,-arm64,install.sh}` | served under `explorer…/download/` |
| Chain data | `/root/kovanica-data` (`KOVANICA_DATA`) | outside any git tree — see [[DECISIONS]] |
| Soak logs | `/root/kovanica-data/soak/` | `testnet-measure.py`, 24h runs |

## DNS (Cloudflare zone `kovanica.online`, full table in `OPERATIONS.md` §3)

| Name | Type | Points | Proxy |
|---|---|---|---|
| `seed.kovanica.online` | A + AAAA | `145.223.116.178`, `2a02:4780:41:1f43::1` | **DNS only** |
| `seed3.kovanica.online` | A | `3.79.148.71` | **DNS only** |
| `explorer`/`www`/`app`/`wallet`/… | A | `145.223.116.178` | proxied (orange) |

Rule of thumb: **seeds must be grey cloud** (proxying breaks raw TCP :9000);
Cloudflare API needs `curl -4` from the ops box.

## Node Environment Variables (verified in `crates/`)

| Var | Meaning | Seen in |
|---|---|---|
| `KOVANICA_DATA` | chain data directory | node |
| `KOVANICA_LISTEN` | P2P listen addr (default `0.0.0.0:9000`, TCP only) | node/explorer |
| `KOVANICA_PEERS` | static peers (or `off`); bootstrap is DNS-first | node/explorer |
| `KOVANICA_POW` | real PoW mining switch | node |
| `KOVANICA_MINE` / `KOVANICA_MINE_SECS` | auto-mine toggle / interval (seed1: `=1`, `=60`) | node |
| `KOVANICA_FAUCET` / `KOVANICA_TAP` | faucet minting / micro-tap switches (public explorer: off) | explorer |
| `KOVANICA_ALLOW_RESET` | destructive reset guard | node/web |
| `KOVANICA_OPERATOR` | operator-mode guard | node/web |
| `KOVANICA_COIN_TYPE` | SLIP-44 coin type for wallet derivation | state/cli |
| `KOVANICA_KEY` | CLI wallet key file (default `kovanica.key`) | cli |
| `KOVANICA_API` | explorer base URL for CLI wallet | cli |
| `KOVANICA_DEMO_MESH` | demo-mode mesh simulation flag | explorer |

Web build flags: `VITE_AUTH_ENABLED`, `VITE_STUN_URLS` (`web/src`).

## Node HTTP Surface (from `crates/kovanica-node/src/explorer.rs`)

`GET /api/head` · `/api/blocks` · `/api/bootstrap` · `/api/history` ·
`/api/origins` · `/api/state` · `/api/utxos` · `/metrics` · WS `/ws`

Live checks: head `https://explorer.kovanica.online/api/head`, P2P status on a
running node `/api/p2p`. Shared contract types: [`web/src/lib/api/contract.ts`](file:///root/kovanica-protocol/web/src/lib/api/contract.ts).

## Wire Constants

| Constant | Value | Where |
|---|---|---|
| DHT wire tags | `0x20` Ping, `0x21` Pong, `0x22` FindNode, `0x23` Nodes | `TODO.md` Architecture Notes, `relay.rs` |
| DHT buckets | 256 k-buckets, capacity k=8 (configurable to 20), α=3 lookups | `TODO.md` Architecture Notes |
| VRF domain tag | `"KOVANICA_VRF_INPUT_v1"` hashed into VRF input | `crates/kovanica-dag/src/dag.rs` |

## Build / Deploy One-Liners (details: `myObsidianVaultDAG.md`, `OPERATIONS.md` §2)

```bash
cargo test && cargo clippy --all-targets          # CI gate (fmt --check too)
cargo run -p kovanica-node -- demo                # scripted end-to-end scenario
./scripts/deploy-seed.sh root@<host> --name seedN --mine --peers seed.kovanica.online:9000
cd web && npm run build:vps && rsync -a --delete .output/ /root/kovanica-web/.output/ && pm2 restart kovanica-web
```
