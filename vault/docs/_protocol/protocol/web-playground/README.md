# kovanica-web Playground — Explorer / Wallet / Map (client-only / consensus-safe)

Status: dual-balance & native-null handling preserved; pm2 surfaces live; RFC-006 constants embedded.
Classification: **client-only** — web surfaces never change consensus, UTXO, GHOSTDAG, or Ed25519 rules.

## Build / deploy conventions (per `kovanica-web` / deploy-ops)

```bash
# Install
npm install

# Dev
npm run dev

# Build (respect dual-balance / native-null mapping)
npm run build

# pm2 reload (never restart — reload over restart)
pm2 reload ecosystem.config.js
```

## Dual-balance & native-null rules (do not break)

- Native KVNC and multi-asset tokens (KVP-102) must both render correctly.
- `null` native balance must map cleanly to empty / zero states (no phantom balances).
- Stealth, HTLC, multisig, vault (RFC-003 / RFC-004 / RFC-005) surfaces must preserve script v2 handling.

## RFC-006 mapping (embedded in web surfaces)

- MAX_SUPPLY 90.2M KVNC — shown in supply charts, never editable.
- Maturity 100 blocks — coinbase spend indicators must reflect this, not older values.
- Fee burn 75% / producer 25% — fee breakdown in explorer tx detail.
- Fee floor `max(1, subsidy / 500_000)` — shown in fee estimators.

## Live verification (cache / refresh discipline)

```bash
# Check head from explorer
curl -s https://explorer.kovanica.online/api/head | jq '{height, subsidy, maturity, finality_depth, genesis}'

# Verify bootstrap / seeds
curl -s https://explorer.kovanica.online/api/bootstrap | jq '.seeds'
```

## P2P / seed policy (never point peers at Cloudflare / orange-cloud explorer hostnames for TCP 9000)

- Bootstrap seed: `seed.kovanica.online:9000` (DNS-only / grey-cloud)
- Explorer / API hostnames (`explorer.kovanica.online`, `api.kovanica.online`) are for HTTP, not TCP 9000 peers.

## Key / seed handling

- Private keys and seeds stay strictly client-side (wallet / extension / mobile light-node).
- The explorer and wallet surfaces never receive seeds.
- Transaction flow: `POST /api/prepare` → offline Ed25519 sign (64-byte / 128 hex) → `POST /api/submit`.

## Environment (documented)

- `KOVANICA_DATA`: node data dir (preserve after genesis)
- `KOVANICA_PEERS`: `seed.kovanica.online:9000`
- `KOVANICA_LISTEN`: `0.0.0.0:9000` (plaintext TCP)
- `KOVANICA_ALLOW_RESET`: `0`
- `KOVANICA_FAUCET`: `0`
- `KOVANICA_OPERATOR`: `0` (default participant)
- `KOVANICA_MINE`: `0`

## Risk register (surfaced early)

- P2P seed handling: always DNS name or origin IP; never orange-cloud explorer hostnames.
- Fee floor / supply cap: embedded values must match live `/api/head`; never override with older hardcoded values.
- Dual-balance null mapping: regression in native-null handling breaks wallet UX.
- Web surface deploy: reload over restart (pm2); verify `/api/head` post-deploy.
