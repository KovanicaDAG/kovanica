# kovanica-web

> **Explorer + Wallet + Map frontend** — TanStack Start/Router, Vite, Nitro, Tailwind v4, Zustand, React Query, better-auth, **pglite** DB + migrations.

---

## Live Services

| Host | Application | Source Path |
|------|-------------|-------------|
| [kovanica.online](https://kovanica.online) | Landing page | `/` |
| [wallet.kovanica.online](https://wallet.kovanica.online) | Browser wallet | `/wallet` |
| [map.kovanica.online](http://map.kovanica.online) | Network origin map | `/map` |
| [explorer.kovanica.online](https://explorer.kovanica.online) | **Rust node explorer** — do not point at this app | `kovanica-node` binary |

> **Preview vs Live**: Header toggle switches between in-process demo DAG (different genesis) and live proxy to `https://explorer.kovanica.online` (`kovanica-testnet`).

---

## Repository Structure

```
kovanica-web/
├── site/          # Vite + TanStack web application (all commands run here)
└── deploy/        # Deploy configuration / output
```

---

## Development

```bash
cd site
npm ci
npm run dev              # Dev server on 127.0.0.1:8080 (binds 0.0.0.0 for Grok preview)
```

**Port discipline**: When running locally on the shared server, **bind to `127.0.0.1` and avoid port 3000** — that port is forwarded to the internet by Cloudflare tunnel.

```bash
# Safe local dev
npx vite dev --host 127.0.0.1 --port 8080
```

---

## Available Routes

| Path | Description |
|------|-------------|
| `/` | Landing page |
| `/explorer` | BlockDAG graph visualizer |
| `/wallet` | Create/import/send/encrypt seed |
| `/multisig` | M-of-N P2SH multisig (create, spend, combine) |
| `/map` | Origin choropleth map |
| `/docs` | HTTP API contract reference |
| `/api/*` | Same contract as Rust node; `?source=live` proxies to explorer |

---

## Build & Deploy

```bash
# Production build for VPS (Node 22 to build, Node 20 to run)
npm run build:vps

# Deploy (from VPS)
rsync -av .output/ user@vps:/path/to/kovanica-web/
pm2 restart kovanica-web
```

> **Important**: `npm run build:vps` does **NOT** run migrations. Run `npm run db:migrate` yourself when you touch `migrations/`.

See [DEPLOY.md](site/DEPLOY.md) for canonical VPS layout, seed env, and systemd config.

---

## Tech Stack

- **Framework**: TanStack Start (file-based routing, SSR)
- **Build**: Vite 5 + Nitro
- **Styling**: Tailwind CSS v4
- **State**: Zustand + React Query (TanStack Query)
- **Auth**: better-auth
- **Database**: pglite (WASM PostgreSQL) + migrations
- **Language**: TypeScript (strict)

---

## Quality Gates

```bash
npm run typecheck    # TypeScript strict check
npm run lint         # Oxlint
npm run format       # Prettier (writes)
npm run test         # node --test
```

---

## Canonical Source

> **Active development happens in `kovanica-protocol/web`**. This repo mirrors it; sync changes from there.

- Protocol repo: https://github.com/KovanicaDAG/kovanica-protocol/tree/main/web
- This repo: https://github.com/KovanicaDAG/kovanica-web

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger (source of truth) |
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Rust node binary (serves explorer API) |
| [kovanica-wallet](https://github.com/KovanicaDAG/kovanica-wallet) | Mobile wallet apps + browser extension |
| [kovanica-sdk](https://github.com/KovanicaDAG/kovanica-sdk) | Rust/WASM SDK |

---

## License

**MIT OR Apache-2.0**