# Web UI — KVP-102 Multi-asset + domain map

## What this package adds

1. **`/multi-asset` surface (KVP-102 / RFC-002)**  
   - Balances tab: `GET /api/utxos` → prefer `balances` map, show per-UTXO `asset_id`  
   - Transfer tab: AssetPicker + `POST /api/prepare` with explicit `asset_id` (fees always KVNC)  
   - History tab: `GET /api/history` entries with `asset_id`  
   - Helpers in `src/lib/protocol/multi-asset.ts` (normalize, short labels, picker sort)

2. **Domain-aware network detection** (`src/lib/network.ts`)  
   - `testnet.kovanica.online` → testnet  
   - `mainnet.kovanica.online` → mainnet  
   - Hard redirect on switch so storage stays isolated  
   - Constants: `API_PUBLIC` = `https://api.kovanica.online`, `DOCS_PUBLIC` = `https://docs.kovanica.online`

3. **Source switch**  
   - On network hosts: redirects between `testnet.` ↔ `mainnet.`  
   - On shared hosts: in-app `setApiSource` as before

4. **Nav + landing**  
   - Shell: **Assets** (`/multi-asset`) next to Multisig  
   - Home product card for Multi-asset

## Files to copy into `kovanica-protocol/web/`

```
src/lib/network.ts                              ← new
src/lib/protocol/multi-asset.ts                 ← new
src/components/protocol/multi-asset-view.tsx    ← new
src/routes/multi-asset.tsx                      ← new
src/components/layout/shell.tsx                 ← replace
src/components/layout/source-switch.tsx         ← replace
src/components/landing/home.tsx                 ← replace
```

TanStack Router file-based routing picks up `/multi-asset` automatically.

## Apply (from monorepo root)

```bash
cd kovanica-protocol/web

mkdir -p src/lib/protocol src/components/protocol

cp /path/to/web-ui-kvp102/src/lib/network.ts                    src/lib/
cp /path/to/web-ui-kvp102/src/lib/protocol/multi-asset.ts       src/lib/protocol/
cp /path/to/web-ui-kvp102/src/components/protocol/multi-asset-view.tsx src/components/protocol/
cp /path/to/web-ui-kvp102/src/routes/multi-asset.tsx            src/routes/
cp /path/to/web-ui-kvp102/src/components/layout/shell.tsx       src/components/layout/
cp /path/to/web-ui-kvp102/src/components/layout/source-switch.tsx src/components/layout/
cp /path/to/web-ui-kvp102/src/components/landing/home.tsx       src/components/landing/

npm run build   # or pnpm / bun
```

## Prerequisites (node HTTP)

Full live UX needs the node HTTP `asset_id` surface (PR / branch `api/kvp-102-asset-id`):

- `GET /api/utxos` → `balances` map + per-UTXO `asset_id`
- `GET /api/history` → per-entry `asset_id`
- `POST /api/prepare` accepts `asset_id` (default `KVNC`)

Until the public explorer node is on that build, the UI still works: scalar `balance` is treated as KVNC-only and prepare omits failure modes gracefully via toast.

Deploy guide: `artifacts/DEPLOY-KVP102.md`  
Gap plan: `artifacts/KVP-102-HTTP-asset_id-gap.md`

## Domain map (your additions)

| Host | Role |
|------|------|
| `mainnet.kovanica.online` | Mainnet interactive surface |
| `testnet.kovanica.online` | Testnet interactive surface |
| `api.kovanica.online` | Shared public HTTP API entry |
| `docs.kovanica.online` | Specs / RFCs / KVP docs |

Point Cloudflare / DNS so `testnet.` and `mainnet.` serve the same web dist (network chosen by hostname). API clients should prefer `https://api.kovanica.online` (or path-proxy through the network host).

## Smoke after deploy

```bash
# Balances (expect balances map + asset_id once node is updated)
curl -sS "https://api.kovanica.online/api/utxos?address=<addr>" | jq '.balances, .utxos[0]'

# Prepare native
curl -sS -X POST "https://api.kovanica.online/api/prepare" \
  -H 'content-type: application/json' \
  -d '{"from":"<a>","to":"<b>","amount":1000,"asset_id":"KVNC"}' | jq .

# UI
open https://testnet.kovanica.online/multi-asset
```
