# Infrastructure surfaces — DNS, nginx and hosts

Inventory of the public Kovanica surfaces, which host serves what, and which
files/records have been retired. Verified **2026-10-16**.

Scope: the public web/API/P2P surface. Chain parameters and tokenomics live in
`TOKENOMICS.md`; this file is about hostnames, vhosts and certificates.

---

## 1. Hosts

| Host | Address | Role | Runs |
|------|---------|------|------|
| `srv1745734` (VPS#1) | `seed1.kovanica.online` | **web / nginx** + testnet **seed1** | nginx :80/:443, pm2 `kovanica-web` on `127.0.0.1:3010`, docker node exposing JSON-RPC via `docker-proxy` on `127.0.0.1:8545`, `kovanica-prometheus@1.service` on `127.0.0.1:9091`, `kovanica-soak-collect.timer`, active `cloudflared` tunnel |
| `srv1991525` (VPS#2) | `seed2.kovanica.online` | testnet **seed2** | node |
| `srv2013143` (VPS#3) | `seed3.kovanica.online` | testnet **seed3** + dev workstation | `/root/kvnc` (main repo, `main`), `/root/Projects/kvnc` (worktree on `fix/leader-rotation-and-liveness`) |

Notes:

- Only VPS#1 has nginx, pm2 and `cloudflared`. VPS#3 has no nginx and no pm2.
- **Grafana is not running** on VPS#1. Prometheus *is* (`127.0.0.1:9091`), which is
  why the `monitor.kovanica.online` vhost is currently pointless (see §5).
- The node's HTTP surface is **`/rpc` (JSON-RPC) and `/health` only**. There is no
  `/api/head` or `/api/bootstrap` — see §5.4.

---

## 2. DNS records (Cloudflare)

### Present — proxied (orange cloud, resolves to `104.21.87.177` / `172.67.170.125`)

| Name | Served by |
|------|-----------|
| `kovanica.online` | nginx → web app |
| `www.kovanica.online` | nginx → web app |
| `explorer.kovanica.online` | nginx → web app (also hosts `wallet.kovanica.online`) |
| `wallet.kovanica.online` | nginx → 302 to `testnet.kovanica.online/wallet` |
| `docs.kovanica.online` | nginx → web app |
| `dash.kovanica.online` | nginx → web app |
| `playground.kovanica.online` | nginx → web app (403 on the root) |
| `map.kovanica.online` | nginx → 302 to `testnet.kovanica.online/map` |
| `mainnet.kovanica.online` | nginx → web app ⚠️ see §5.3 |
| `monitor.kovanica.online` | **nothing** — no enabled vhost ⚠️ see §5.1 |
| `console.kovanica.online` | Cloudflare Pages (not on these hosts) |
| `status.kovanica.online` | Cloudflare Pages (not on these hosts) |
| `faucet-testnet.kovanica.online` | nginx → `127.0.0.1:3010` (same web app as `kovanica.online`; usable while `faucet.kovanica.online` is being retired) |

### Present — DNS-only (grey cloud, → `seed1.kovanica.online`)

| Name | Purpose |
|------|---------|
| `testnet.kovanica.online` | testnet web surface |
| `rpc-testnet.kovanica.online` | public testnet JSON-RPC (`/rpc`) |
| `seed1.kovanica.online` | testnet P2P seed, plaintext TCP 8000 |

### Absent (no DNS record — do not re-add without a reason)

`api.kovanica.online`, `opencode.kovanica.online`, `kovanica.kovanica.online`,
`nft.kovanica.online`, `pool.kovanica.online`, `rpc.kovanica.online`.

The only Cloudflare API token lives in the workspace `.env` (line 11); it is
**IP-restricted to VPS#1 over IPv4**, so record changes work from there with
`curl -4` and fail with 401 everywhere else.

---

## 3. nginx vhosts (VPS#1) — current state

Ten enabled vhosts. `sites-available` contains exactly these ten plus the stock
`default`.

| vhost | Behaviour |
|-------|-----------|
| `kovanica.online` (+`www`) | proxy → `127.0.0.1:3010` |
| `explorer.kovanica.online` | proxy → `127.0.0.1:3010`; `location /api/ { return 404; }`; `location /download/` aliases `/var/www/kovanica-dist/`; deep-link 301s (`/b/<64hex>` → `/explorer/block/`, `/tx/<64hex>` → `/explorer/tx/`, `/addr/kvnc…` → `/explorer/address/`, `/out/<64hex>/<n>` → `/explorer/output/`); also carries `server_name wallet.kovanica.online` → 302 to `https://testnet.kovanica.online/wallet$request_uri` |
| `docs.kovanica.online` | proxy → `127.0.0.1:3010` |
| `dash.kovanica.online` | proxy → `127.0.0.1:3010` |
| `playground.kovanica.online` | proxy → `127.0.0.1:3010`, root `return 403` |
| `map.kovanica.online` | `return 302 https://testnet.kovanica.online/map$request_uri` |
| `mainnet.kovanica.online` | proxy → `127.0.0.1:3010` |
| `testnet.kovanica.online` | proxy → `127.0.0.1:3010` |
| `rpc-testnet.kovanica.online` | proxy → `http://127.0.0.1:8545/rpc` |
| `faucet-testnet.kovanica.online` | proxy → `127.0.0.1:3010` (web app; added 2026-10-28 for `/faucet` and `/download/*`) |

The `/api/ { return 404; }` rule on the explorer vhost is **deliberate**: the old
public API service (port 3001) was retired, and the node now exposes JSON-RPC on
`/rpc` only. Do not "fix" that 404 — it is the intended tombstone.

Smoke test (all from VPS#1, `curl -sk --resolve <host>:443:127.0.0.1`):

```
kovanica.online 200   www 200   explorer 200
wallet 302 -> testnet/wallet
docs 307   dash 307   playground 307
map 200    mainnet 200   testnet 200
upstream 127.0.0.1:3010 200
rpc-testnet.kovanica.online/rpc + kovanica_getCommittee -> valid committee JSON
```

---

## 4. Retired files (moved, never deleted)

Everything removed in the 2026-10-16 cleanup was **moved**, so it is recoverable:

- `/root/nginx-retired/20261016/` — the disabled vhosts
- `/root/nginx-retired/20261016/_archive/` — dead `sites-available` backups

### Retired because the service is gone

| File | Why |
|------|-----|
| `sites-enabled/api.kovanica.online` (+ `sites-available` copy, 2 baks) | 503 tombstone; the API service was retired and the DNS record is gone |
| `sites-available/opencode` (+ `.bak`) | never enabled; DNS gone; cert gone; backend `:4096` not listening. ⚠️ contained a plaintext Basic-auth credential — **treat as exposed and rotate** |
| `sites-enabled/kovanica.kovanica.online` (+ file) | no DNS record; only redirected to `faucet.testnet.kovanica.online` |

### Archived dead backups (20 files)

Stale `default.*` backups, old `explorer*` variants, `kovanica-online.conf*`,
`kovanica-web`, `monitor`, `nft-kovanica.bak*`, `pool.kovanica.online`,
`kovanica.kovanica.online`.

Rule applied: archive every `sites-available/*` that is not the target of a live
symlink in `sites-enabled`, always keeping the stock `default`.

After every step `nginx -t` passed and `systemctl reload nginx` succeeded.

---

## 5. Open items

### 5.1 `monitor.kovanica.online` — record without a service

The DNS record exists (proxied) but the vhost is archived and disabled, and its
backends are inconsistent: Prometheus answers on `127.0.0.1:9091` while Grafana on
`127.0.0.1:3000` is not running.

**Resolution chosen:** leave the vhost disabled and archived; **delete the DNS
record** in the Cloudflare dashboard (or bring Grafana back up and restore the
vhost — pick one, don't leave it dangling).

### 5.2 `claudes.online` certificate — unrelated domain

A `claudes.online` certificate (expires 2027-01-09) is installed on VPS#1 for a
domain that has nothing to do with Kovanica. **Flag only** — it was not deleted.

### 5.3 `cloudflared` tunnel is a no-op

The tunnel `75815529-e462-4bc5-977a-5afc406d20aa` is active, but its only ingress
rule is now `service: http_status:404`. It serves nothing. Consider stopping and
disabling it, or give it a real purpose.

### 5.4 Public head/bootstrap endpoints are gone by design

`https://explorer.kovanica.online/api/head` and `/api/bootstrap` return **404**.
This is intentional (the API surface was retired, and the explorer vhost 404s
`/api/` before proxying).

Live chain parameters must instead be read from:

- the RPC method `kovanica_getLeaderHeader` via
  `https://rpc-testnet.kovanica.online/rpc`, or
- a local node.

### 5.5 `mainnet.kovanica.online` must not be the P2P seed

The record is **orange-cloud/proxied**. A P2P seed has to be **grey-cloud/DNS-only**
because P2P is plaintext TCP 8000 and Cloudflare's proxy only forwards HTTP(S).
Fix this before mainnet launch.

### 5.6 P2P seed policy reminder

Point peers at the DNS seed name or the origin IP — **never** at a proxied
Cloudflare hostname.

### 5.7 Pre-existing nginx warning

```
protocol options redefined for [::]:443 in /etc/nginx/sites-enabled/testnet.kovanica.online:19
```

Harmless today, but worth cleaning up.

### 5.8 Certificates

| Certificate | Notes |
|-------------|-------|
| `kovanica.online` (+`www`) | |
| `explorer.kovanica.online` (+ SAN `wallet.kovanica.online`) | |
| `testnet.kovanica.online` | |
| `rpc-testnet.kovanica.online` | |
| `dash.kovanica.online` | |
| `map.kovanica.online` | |
| `claudes.online` (+`www`) | unrelated domain — see §5.2 |

`docs.kovanica.online` has **no dedicated certificate**; the vhost still carries
the comment "the server_name is claimed. Replace with a docs-specific cert when
issued." Issue one when convenient.

---

## 6. Accessing these hosts

VPS#1 is reached with `sshpass` as `root@seed1.kovanica.online`. The credential is kept
in `/tmp/opencode/.vps_cred` (mode 600, `root@seed1.kovanica.online|<password>`) on the
dev box:

```sh
cd /tmp/opencode; PW=$(cut -d'|' -f2 .vps_cred)
sshpass -p "$PW" ssh -o StrictHostKeyChecking=no root@seed1.kovanica.online 'bash -s' <<'EOS'
  nginx -t && systemctl reload nginx
EOS
```

An unrelated `~/.ssh/deploy_kvnc` key on the dev box does **not** authenticate to
VPS#1.
