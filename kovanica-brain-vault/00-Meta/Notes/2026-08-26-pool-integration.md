# 2026-08-26 — Pool Dashboard and React App Integration

> **Links:** [[ROADMAP]] · [[TODO]] · [Agent Session](conversation://fe7514dd-54b0-4771-9ee0-8f01a2a393be)

## Shipped

- [x] **Go Stratum Pool Upgrade**: Completely overhauled the legacy Node.js NOMP pool and replaced it with a modern, high-performance Go-based Stratum mining pool implementing Blake3 hashing.
- [x] **Cloudflare DNS Automation**: Extracted the Cloudflare API token from the vault/secrets, automated the zone lookup, and provisioned a DNS-only A record for `pool.kovanica.online` pointing to the server IP. Allowed TCP port 3333 in `ufw`.
- [x] **JSON Backend API**: Implemented a lightweight `/api/stats` JSON endpoint with CORS directly inside the Go pool (`main.go` on port 8081) to serve real-time pool metrics (active workers, shares).
- [x] **Nginx Proxying and SSL**: Secured `pool.kovanica.online` with Certbot/Let's Encrypt. Configured Nginx to proxy `/api/stats` to the Go backend, and mapped `/` to redirect to the main UI.
- [x] **React App Integration (kovanica-web)**: Added a native `/pool` route to the Tanstack React router application. Removed `@tanstack/react-query` dependency to avoid `QueryClient` initialization errors, instead implementing robust polling via `useEffect`. Rebuilt and deployed the React app seamlessly via PM2.
- [x] **Vault Sync**: Recognized the deprecation of the TAP micro-faucet and aligned the frontend architecture to current specifications.

## Lessons burned in

- `@tanstack/react-query` requires a `QueryClientProvider` at the root of the app. Without it, `useQuery` crashes the component tree. Always verify the root providers before dropping in complex query hooks.
- Cloudflare standard proxying breaks raw TCP stratum connections. Always configure DNS records as "DNS-only" (proxied=false) when routing traffic for ports like 3333.
- Always check the Obsidian vault (`/root/Antonio/Obsidian-Vault/`) for the latest architecture notes and deprecated features (e.g., TAP faucet removal) before making frontend assumptions!

## Open follow-ups

> [!WARNING] **HANDOFF NOTICE FOR NEXT AGENT**
> A `teamwork_preview` multi-agent squad is currently executing in the background to implement `GET /api/mine/template` and `POST /api/mine/submit` in the `kovanica-node` Rust codebase.
> Start your session by checking their output in `/root/kovanica-protocol`.

- [ ] Verify the background teamwork agents successfully implemented the Rust mining APIs in `kovanica-node`.
- [ ] Implement Kovanica node HTTP JSON polling inside the Go pool (`main.go`) to replace the currently broken Bitcoin `btcd` RPC implementation, so miners fetch and submit real Kovanica blocks.
- [ ] Connect the Go pool's `/api/stats` endpoint with real hashrate calculations rather than the current mock implementation.
