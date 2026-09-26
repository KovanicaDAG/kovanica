---
title: "Root Landing Structure — kovanica.online"
category: 20-Network
source: protocol/docs/upgrades/02-RedesignDomains/02-ROOT-LANDING-STRUCTURE.md
synced: 2026-09-26
---
# Root Landing Structure — kovanica.online

Goal: Pure project / marketing homepage. Zero live network UI.

## Recommended Page Structure

### 1. Header
- Logo + project name
- NetworkBadge (shows current network)
- SourceSwitch (Testnet / Mainnet) — hard redirects
- Simple nav: Docs · GitHub · Status

### 2. Hero
- One strong sentence about the protocol
- Phase badge: **Pre-Mainnet · Testnet Live**
- Primary CTAs (large, clear):
  - Launch Explorer → `https://testnet.kovanica.online`
  - Open Wallet → `https://wallet.kovanica.online`
  - Read Specs → `https://docs.kovanica.online`
- Secondary: Run a Node (link to GitHub / install)

### 3. Protocol Snapshot
- Short description of GHOSTDAG + UTXO + Ed25519
- Key properties (k=3, KVNC, etc.)
- Link to full docs

### 4. Roadmap (high level only)
- KVP-101 … KVP-105 status (Shipped / In progress)
- RFC-006 Tokenomics status
- Mainnet readiness note
- Link to full roadmap / LEGIT-BOARD

### 5. Tokenomics Summary (RFC-006)
- Max supply
- Emission curve (high level)
- Treasury / founder allocation
- Link to full Tokenomics.md / docs

### 6. Network Access
- Clear cards or buttons:
  - Testnet → testnet.kovanica.online
  - Mainnet → mainnet.kovanica.online (Coming soon / Ready)
  - Faucet → faucet.testnet.kovanica.online
  - Status → status.kovanica.online
  - Seeds → seed.kovanica.online:9000

### 7. Footer
- Docs, GitHub, Telegram/Discord
- Explorer, Wallet, API, Status
- Seed information
- Copyright / licence note

## Explicitly Remove from Root
- Live DAG / block explorer
- Faucet form
- Multisig / Stealth / HTLC / Vault interactive views
- Live height / supply gauges
- Any “Connect” that talks to a live node
- Deep protocol playgrounds

## Copy Tone
- Professional, precise, technical but accessible
- No hype language
- Emphasise that the root is the project face and the networks live on their own subdomains
```

