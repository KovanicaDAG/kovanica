# Cloudflare Pages Deployment Config for KVNC Web

## Project Settings

**Project Name:** `kvnc-explorer`
**Production Branch:** `main`
**Build Command:** `cd packages/web && npm run build`
**Build Output Directory:** `packages/web/dist`
**Root Directory:** `/` (monorepo root)
**Node Version:** `20`

## Environment Variables

### Production (explorer.kovanica.online)

| Variable | Value | Type |
|----------|-------|------|
| `VITE_RPC_URL` | `https://rpc.kovanica.online` | Secret |
| `VITE_CHAIN_ID` | `1` | Plain |

### Preview (auto-generated *.pages.dev)

| Variable | Value | Type |
|----------|-------|------|
| `VITE_RPC_URL` | `https://rpc-testnet.kovanica.online` | Secret |
| `VITE_CHAIN_ID` | `2` | Plain |

## Kovanica Console

**Project Name:** `kvnc-console`
**Domain:** `console.kovanica.online` (proxied CNAME to the Pages hostname)
**Root Directory:** `packages/console`
**Build Command:** `npm run build`
**Build Output Directory:** `dist`
**Node Version:** `20`

One production build contains both networks. Testnet (`chain_id` 2, `https://rpc-testnet.kovanica.online/rpc`) is the live target. Mainnet (`chain_id` 1) is shown in the header switch and labeled Soon. It does not become the signing network until `MAINNET.enabled` is set to `true` in `packages/console/src/network.ts`. There is no RPC URL secret: the endpoints are public and pinned in that file. Preview deployments use the same table.

`packages/console/public/_headers` is the Content-Security-Policy. `connect-src` allows only the testnet RPC, the mainnet RPC, their WebSocket endpoints, and the testnet faucet. `public/_redirects` sends client routes to `index.html`.

The local node (`chain_id` 1337) is compiled out of production builds.

## Custom Domain

**Domain:** `explorer.kovanica.online`
**DNS Record:** CNAME to `<project>.pages.dev`
**Cloudflare Proxy:** Enabled (orange-cloud)
**SSL:** Automatic (Cloudflare Universal SSL)

## Build Configuration

```json
{
  "buildCommand": "cd packages/web && npm run build",
  "destinationDir": "packages/web/dist",
  "rootDir": "",
  "nodeVersion": "20"
}
```

## GitHub Actions Workflow (Optional - for additional testing)

```yaml
# .github/workflows/web-deploy.yml
name: Deploy Web to Cloudflare Pages

on:
  push:
    branches: [main]
    paths:
      - 'packages/web/**'
  workflow_dispatch:

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: packages/web/package-lock.json

      - name: Install dependencies
        run: cd packages/web && npm ci

      - name: Build
        run: cd packages/web && npm run build
        env:
          VITE_RPC_URL: ${{ secrets.VITE_RPC_URL }}
          VITE_CHAIN_ID: ${{ secrets.VITE_CHAIN_ID }}

      - name: Deploy to Cloudflare Pages
        uses: cloudflare/pages-action@v1
        with:
          apiToken: ${{ secrets.CLOUDFLARE_API_TOKEN }}
          accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
          projectName: kvnc-explorer
          directory: packages/web/dist
          branch: main
```

## DNS Records (Cloudflare Dashboard)

| Type | Name | Target | Proxy Status | TTL |
|------|------|--------|--------------|-----|
| CNAME | explorer | kvnc-explorer.pages.dev | Proxied (Orange) | Auto |
| CNAME | explorer | kvnc-explorer.pages.dev | Proxied | Auto |

## WAF Custom Rules

```
# Rate limit RPC calls from explorer (if any)
(http.request.uri.path contains "/rpc") and (http.request.method eq "POST")
  -> Rate Limit: 60 requests/minute per IP

# Allow all traffic to static assets
(http.request.uri.path matches "^/assets/")
  -> Allow
```

## Page Rules

| Pattern | Setting | Value |
|---------|---------|-------|
| explorer.kovanica.online/* | Cache Level | Standard |
| explorer.kovanica.online/* | Edge Cache TTL | 30 seconds |
| explorer.kovanica.online/* | Always Online | On |
| explorer.kovanica.online/api/* | Cache Level | Bypass |

## SSL/TLS Settings

- **SSL Mode:** Full (Strict)
- **Always Use HTTPS:** On
- **Minimum TLS Version:** 1.2
- **Automatic HTTPS Rewrites:** On
- **Opportunistic Encryption:** On
- **TLS 1.3:** On

## Analytics & Performance

- **Web Analytics:** Enabled
- **Speed Insights:** Enabled
- **Real User Monitoring:** Enabled

## Security Headers (via Cloudflare Workers or _headers file)

Create `packages/web/public/_headers`:

```
/*
  X-Frame-Options: DENY
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
  Permissions-Policy: camera=(), microphone=(), geolocation=()
  Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' https://rpc.kovanica.online wss://rpc.kovanica.online;
```

## Monitoring & Alerts

- **Uptime Monitor:** 1-minute interval, alert on 3 consecutive failures
- **SSL Certificate Expiry:** Alert 30 days before expiry
- **Page Load Time:** Alert if p95 > 3s
- **Error Rate:** Alert if 5xx > 1%

## Rollback Procedure

1. Go to Cloudflare Pages dashboard → Deployments
2. Click "..." on previous successful deployment → "Promote to production"
3. DNS propagates instantly (CNAME to same zone)