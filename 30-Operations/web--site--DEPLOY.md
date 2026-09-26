---
title: "VPS (`srv1745734`)"
category: 30-Operations
source: web/site/DEPLOY.md
synced: 2026-09-26
---
# VPS (`srv1745734`)

`/root/kovanica-web` and `/root/kovanica-ledger` are **not git repos.** Clone
sidecars. Do not `git -C` them.

Kovanica owns `127.0.0.1:3000` (web), `127.0.0.1:8080` (explorer HTTP,
primary seed unit `kovanica-explorer`), `0.0.0.0:9000` (P2P). Leave dashboard / trader / postgres / docker alone.

---

## Seed

Explorer listens on TCP 9000. Live values on the VPS (systemd `kovanica-explorer`,
2026-09-20 — the node dials seed2, mines, and runs the faucet; the old
"peers=off / mine=0 / faucet=0" block is obsolete):

```
KOVANICA_LISTEN=0.0.0.0:9000
KOVANICA_PEERS=seed2.kovanica.online:9000
KOVANICA_MINE=1
KOVANICA_MINE_SECS=60
KOVANICA_FAUCET=1
KOVANICA_ALLOW_RESET=0
KOVANICA_OPERATOR=1
KOVANICA_POW=1
KOVANICA_DATA=/root/kovanica-data
```

ufw `9000/tcp` is open. **That is not enough:** `explorer.kovanica.online` is
Cloudflare-proxied (orange cloud). A clone that dials that hostname:9000 hits
Cloudflare, not this box.

Publish a **DNS-only** (grey cloud) A record:

```
seed.kovanica.online  →  $(curl -s ifconfig.me)   # DNS only, proxy OFF
```

Clones:

```
KOVANICA_PEERS=seed.kovanica.online:9000
```

Until that record exists, use the origin IP: `KOVANICA_PEERS=<ip>:9000`.

---

## Ship UI (Telegram links gone)

Do this **inside tmux** so an SSH drop does not kill the build. Reuse the
existing `/tmp` clone — do **not** `npm ci` again (that is what dropped SSH).

```sh
tmux new -s kv || tmux attach -s kv
cd /tmp/kovanica-web-build
git fetch origin
git reset --hard origin/main
npm run build:vps
test -f .output/server/index.mjs
mkdir -p /root/kovanica-web/.output
rsync -a --delete .output/ /root/kovanica-web/.output/
pm2 delete kovanica-web
cd /root/kovanica-web
HOST=127.0.0.1 PORT=3000 pm2 start .output/server/index.mjs --name kovanica-web
pm2 save
```

If `/tmp/kovanica-web-build` is missing, clone once then `npm ci` **inside tmux**.

Confirm no `t.me` on `https://kovanica.online`. Purge Cloudflare cache if needed.
