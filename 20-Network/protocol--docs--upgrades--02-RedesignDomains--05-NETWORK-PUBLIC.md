---
title: "Kovanica Network & Domains (Public)"
category: 20-Network
source: protocol/docs/upgrades/02-RedesignDomains/05-NETWORK-PUBLIC.md
synced: 2026-09-26
---
# Kovanica Network & Domains (Public)

> Pre-Mainnet phase — RFC-006 activated on testnet

## Domain Map

| Hostname                            | Purpose                          |
|-------------------------------------|----------------------------------|
| **kovanica.online**                 | Project homepage                 |
| **testnet.kovanica.online**         | Full Testnet                     |
| **mainnet.kovanica.online**         | Mainnet surface (ready)          |
| **faucet.testnet.kovanica.online**  | Testnet faucet                   |
| **api.kovanica.online**             | Public HTTP API                  |
| **explorer.kovanica.online**        | Block explorer                   |
| **wallet.kovanica.online**          | Web wallet                       |
| **docs.kovanica.online**            | Specifications & RFCs            |
| **status.kovanica.online**          | Network status                   |
| **seed.kovanica.online**            | P2P seed (port 9000)             |

## Networks

|                | Testnet                | Mainnet              |
|----------------|------------------------|----------------------|
| Network ID     | `kovanica-testnet`     | `kovanica`           |
| Hostname       | testnet.kovanica.online| mainnet.kovanica.online |
| Faucet         | Yes                    | No                   |
| Status         | Live                   | Not yet open         |

## For Developers

Detect the network from the hostname and use the matching configuration.
A hard redirect (changing hostname) is preferred when switching networks so that storage and API clients stay isolated.

Canonical client helper lives in the web repository as `src/lib/network.ts`.

Full internal details: see `NETWORK.md` in the monorepo.
```

