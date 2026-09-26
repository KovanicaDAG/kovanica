---
title: "Mainnet Activation Playbook (Domain side)"
category: 20-Network
source: protocol/docs/upgrades/02-RedesignDomains/07-MAINNET-ACTIVATION.md
synced: 2026-09-26
---
# Mainnet Activation Playbook (Domain side)

When the protocol side is ready to open mainnet:

## 1. Infrastructure
- [ ] Production genesis and binaries deployed
- [ ] Mainnet nodes / seeds running and peered
- [ ] API / explorer / wallet backends pointed at mainnet data

## 2. DNS & Frontend
- [ ] Confirm `mainnet.kovanica.online` resolves and is Proxied
- [ ] Deploy the production web UI to the mainnet origin
- [ ] Verify `detectNetwork()` returns `"mainnet"` on that hostname
- [ ] Confirm no faucet is exposed on mainnet

## 3. Root Landing
- [ ] Change phase badge from “Pre-Mainnet · Testnet Live” → “Mainnet Live”
- [ ] Update CTAs if needed (Explorer / Wallet can stay or become network-aware)
- [ ] Keep testnet clearly available for developers

## 4. Cloudflare
- [ ] Disable or delete temporary Rule 4 (root path → testnet) once traffic has moved
- [ ] Review any other temporary redirects

## 5. Communication
- [ ] Update NETWORK.md and public docs
- [ ] Announce on Telegram / Discord / GitHub
- [ ] Update install scripts and any hard-coded network references

## 6. Post-activation
- [ ] Keep testnet running indefinitely
- [ ] Monitor both networks
- [ ] Watch for users accidentally using testnet addresses on mainnet (and vice versa)
```

