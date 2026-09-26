# kovanica-testnet

Public BlockDAG testnet. Native token **KVNC** (8 decimals).

> **The testnet parameters table lives in
> [`protocol/TESTNET.md`](../protocol/TESTNET.md).** It is the single canonical
> source for genesis, premine, subsidy, max supply, maturity, fee floor and fee
> split, plus the seed topology. This file used to carry its own copy and it
> drifted: it still advertised pre-RFC-006 economics (50 KVNC premine, 50 KVNC
> subsidy halving every 1000 blocks, 0.0001 KVNC min fee). Duplicated tables
> rot, so the copy is gone rather than patched.

Quick reference (details and the reasoning live in the canonical file):

| | |
| --- | --- |
| Explorer | https://explorer.kovanica.online |
| Wallet | https://wallet.kovanica.online |
| Network | `kovanica-testnet` |
| Subsidy | **10 KVNC / block** at genesis, geometric decay ×3/4 every **2 000 000** blocks |
| Max supply | **90.2M KVNC** hard cap |
| Coinbase maturity | **100 blocks** |
| k | 3 (GHOSTDAG) |
| P2P | **TCP only** `KOVANICA_LISTEN` (default `0.0.0.0:9000`) |
| Bootstrap | DNS-only `seed.kovanica.online:9000` (not the Cloudflare hostname) |

`explorer.kovanica.online` is orange-cloud, so TCP 9000 never reaches the seed
through that name. Dial grey-cloud `seed.kovanica.online` / `seed2.kovanica.online`,
or the origin IP.

See also: [`protocol/docs/RFC-006-EmissionCurve.md`](../protocol/docs/RFC-006-EmissionCurve.md)
(full tokenomics spec) and [`protocol/TESTNET-RFC006.md`](../protocol/TESTNET-RFC006.md)
(activation record).

Join a clone: [JOIN.md](./JOIN.md) (one-click install, Windows/Linux/macOS, USB stick).

## Run

See [README.md](./README.md).
