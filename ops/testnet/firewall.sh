#!/usr/bin/env bash
# ufw rules for a Kovanica seed host: 8000 P2P public, 8545 RPC loopback only
# (public RPC goes through nginx 443 with limit_req).
set -euo pipefail
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 8000/tcp comment 'kovanica p2p'
ufw deny 8545/tcp comment 'rpc via nginx only'
ufw --force enable
