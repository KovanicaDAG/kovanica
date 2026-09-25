# kovanica-testnet

Public BlockDAG testnet. Native token **KVNC** (8 decimals).

> > **Consensus decision (ratified 2026-09-25): Kovanica is PoA-only.**
> > Proof-of-Work is being removed from the protocol. See
> > `protocol/docs/RFC-POA-Migration.md` §0 (canonical). Items marked `[TARGET]`
> > are ratified but not yet implemented; `[CURRENT]` items describe shipped code.
> >
> > ⚠️ **This table is stale on tokenomics, independently of the PoA decision.**
> > The `Premine`, `Subsidy cap` and `Min fee` rows below are pre-RFC-006 values
> > and do **not** match the code. `protocol/TESTNET.md` and
> > `protocol/TESTNET-RFC006.md` carry the canonical values. See
> > "Stale rows" below.
> >
> > **The canonical RFC-006 values are unchanged by the PoA-only decision**: the
> > curve is height-indexed and `cumulative_minted` is hard-capped at
> > `MAX_SUPPLY` in `apply_block`, so supply math does not depend on admission.
> > Only the wall-clock *pace* changes (fixed 3000 ms slots, no retarget, no
> > gap-fill); the 90.2M cap does not.

| | |
| --- | --- |
| Explorer | https://explorer.kovanica.online |
| Wallet | https://wallet.kovanica.online |
| Node source | https://github.com/KovanicaDAG/kovanica-node |
| Network | `kovanica-testnet` |
| Premine | ~~50 KVNC (founder)~~ **STALE** — see below |
| Subsidy cap | ~~50 KVNC / block, halves every 1000 blocks~~ **STALE** — see below |
| Min fee | ~~0.0001 KVNC at genesis~~ **STALE** — see below |
| k | 3 (GHOSTDAG) |
| PoW | `[CURRENT]` on (`KOVANICA_POW=1`) — pre-reset chain / **`[TARGET]`-removed** |
| PoA | `[CURRENT]` available, **default when `KOVANICA_CONSENSUS` is unset** · `[TARGET]` the only admission model |
| Slot duration | `[CURRENT]` `KOVANICA_SLOT_DURATION`, default 3000 ms · no gap-fill |
| P2P | **TCP only** `KOVANICA_LISTEN` (default `0.0.0.0:9000`) |
| Bootstrap | DNS-only `seed.kovanica.online:9000` (not the Cloudflare hostname) |

### Stale rows (pre-RFC-006 — do not use)

The three struck-through rows above predate RFC-006 and contradict
`kovanica-state/src/ledger.rs`. The canonical values, which the PoA-only
decision leaves **untouched**, are:

| Parameter | Canonical value | Source |
| --- | --- | --- |
| Premine | **0.2M KVNC** (founder), plus **10M KVNC** in 10 × 1M RFC-005 treasury vaults | `RFC006_PREMINE`, `RFC006_TREASURY_TOTAL` |
| Genesis subsidy s₀ | **10 KVNC / block** | `RFC006_GENESIS_SUBSIDY` |
| Decay | geometric ×3/4 every **2 000 000** blocks (`RFC006_ERA_LENGTH`) — **not** a halving every 1000 blocks | `subsidy_at()` |
| Max supply | **90.2M KVNC** hard cap | `MAX_SUPPLY` |
| Coinbase maturity | **100 blocks** | `COINBASE_MATURITY` |
| Fee | floor `max(1, subsidy/500_000)` atoms/byte; **75% burned / 25% producer** | `FEE_PRODUCER_NUM`/`FEE_PRODUCER_DEN` |
| Precision | **1 KVNC = 100_000_000 atoms** | `ATOM` |

This file was left in place rather than rewritten so the discrepancy stays
visible; it should be reconciled with `protocol/TESTNET.md` by the `node-ops`
role.

Live genesis and tip: `GET https://explorer.kovanica.online/api/head`  
P2P status on a running node: `GET /api/p2p`  
Block dump (same bytes a clone pulls over TCP): `GET /api/blocks`  
Bootstrap blob: `GET https://explorer.kovanica.online/api/bootstrap`

There is no second network path. libp2p / 30333 was removed: it bound a port
and never gossiped blocks.

`explorer.kovanica.online` is orange-cloud. TCP 9000 never reaches the seed
through that name. Grey-cloud `seed.kovanica.online` (or the origin IP) is the
peer address clones should dial. The primary seed dials its sibling
(`KOVANICA_PEERS=seed2.kovanica.online:9000`; `seed3` retired 2026-09-17).
On connect the seed **serves** its dump then **reads** the clone's dump, so extra
blocks on a clone can land on the seed. The open faucet
(`POST https://explorer.kovanica.online/api/faucet`) pays 1 KVNC from the
operator's funds — the old TAP micro-faucet (0.01 KVNC, 40/day) was removed
project-wide on 2026-08-24.

## Tokenomics

- 1 KVNC = 10^8 atoms.
- New coins only from coinbase (issuance + fees to the miner).
- The public primary seed **mines** ~1 block/min (`KOVANICA_MINE=1 KOVANICA_MINE_SECS=60`); the open faucet is **on**.
- Empty blocks are minted on the `KOVANICA_MINE_SECS` interval (60s on the primary).
- Wallet `prepare` / `submit` stays open: you sign in the browser; the node never sees the seed.

> **Note:** this file predates the RFC-006 tokenomics. For the current tokenomics
> (genesis, subsidy, caps), see [`protocol/TESTNET.md`](../protocol/TESTNET.md) —
> `Premine`/`Subsidy cap` rows above are pre-RFC-006 and kept only as historic reference.

Join a clone: [JOIN.md](./JOIN.md) (one-click install, Windows/Linux/macOS, USB stick).

## Run

See [README.md](./README.md).
