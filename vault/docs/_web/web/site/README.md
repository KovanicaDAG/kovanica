# Kovanica Web Application

> **TypeScript UI only** — Protocol (GHOSTDAG, UTXO, Ed25519, PoA) lives in [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol).

---

## Live Deployment

| Host | App | Notes |
|------|-----|-------|
| [kovanica.online](https://kovanica.online) | Landing | `/` |
| [wallet.kovanica.online](https://wallet.kovanica.online) | Wallet | `/wallet` — create/import/send/encrypt seed |
| [map.kovanica.online](http://map.kovanica.online) | Network Map | `/map` — origin choropleth |
| [explorer.kovanica.online](https://explorer.kovanica.online) | **Rust Node Explorer** | Do **not** point this app at that URL — it's a separate binary |

**VPS**: `pm2 kovanica-web` on `127.0.0.1:3000`  
**Build**: Node 22 → runs on Node 20

---

## Route Map

| Path | Feature |
|------|---------|
| `/` | Landing page |
| `/explorer` | BlockDAG graph visualizer |
| `/wallet` | Browser wallet (create, import, send, encrypt seed) |
| `/multisig` | M-of-N P2SH multisig (create, spend, combine signatures) |
| `/map` | Geographic origin choropleth |
| `/docs` | HTTP API contract reference |
| `/api/*` | Node API contract; `?source=live` proxies to Rust explorer |

**Header Toggle**: Preview (in-process demo DAG, different genesis) ↔ Live (proxies to `https://explorer.kovanica.online`)

---

## Development

```bash
npm ci
npm run dev              # Binds 0.0.0.0:8080 for Grok preview
```

**Local dev on shared server** — bind to localhost, avoid port 3000 (Cloudflare tunnel):

```bash
npx vite dev --host 127.0.0.1 --port 8080
```

---

## Commands

| Command | Description |
|---------|-------------|
| `npm run dev` | Development server |
| `npm run build` | Vite build + `db:migrate` |
| `npm run build:vps` | Production build (no migrations, `NITRO_PRESET=node-server`) |
| `npm run db:migrate` | Run pglite migrations |
| `npm run typecheck` | TypeScript strict check |
| `npm run lint` | Oxlint |
| `npm run format` | Prettier (writes) |
| `npm run test` | `node --test` |

---

## Deploy to VPS

```bash
# On development machine
npm run build:vps

# On VPS
rsync -av .output/ user@vps:/path/to/kovanica-web/
pm2 restart kovanica-web
```

See [DEPLOY.md](DEPLOY.md) for canonical VPS layout, seed env, and systemd config.

---

## Architecture Notes

- **Dual-balance handling**: Native KVNC (`asset_id = null`) + multi-asset (KVP-102) balances displayed separately
- **Native-null mapping**: `asset_id: null` in API → `AssetId::native()` in SDK → "KVNC" in UI
- **Address format**: `kvnc…dag` (base58) everywhere; 64-hex accepted on input
- **Signing**: Ed25519 offline in browser; node only verifies (`POST /api/prepare` → sign → `POST /api/submit`)

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Framework | TanStack Start (file routing, SSR) |
| Build | Vite 5 + Nitro |
| Styling | Tailwind CSS v4 |
| State | Zustand + React Query (TanStack Query) |
| Auth | better-auth |
| Database | pglite (WASM PostgreSQL) + migrations |
| Language | TypeScript (strict) |

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger (source of truth) |
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Rust node binary (serves `/api/*`) |
| [kovanica-sdk](https://github.com/KovanicaDAG/kovanica-sdk) | Rust/WASM SDK (types, keys, tx builders, RPC) |
| [kovanica-wallet](https://github.com/KovanicaDAG/kovanica-wallet) | Mobile wallet apps + browser extension |

---

## License

**MIT OR Apache-2.0**