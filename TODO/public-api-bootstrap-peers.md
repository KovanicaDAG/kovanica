# Public API — `/api/bootstrap` peer leak & stale `seed3`

**Status**: ⚠️ **Code fix done in repo; VPS deploy + env cleanup still pending (no SSH access from this box)**
**Discovered**: 2026-09-21 · **Last re-verified**: 2026-09-21 (live payload still unfixed)
**Severity**: Low (client-only / API surface + ops hygiene) — no consensus or ledger impact.

---

## Findings

The public bootstrap payload leaked the node's own listen spec into `peers`,
and carried an entry for a seed that no longer exists:

```json
// https://explorer.kovanica.online/api/bootstrap  (2026-09-21, re-verified)
"listen": "0.0.0.0:9001,[::]:9001",
"peers": [
  "seed.kovanica.online:9000",
  "seed3.kovanica.online:9000",   // <-- dead: DNS deleted 2026-09-21
  "0.0.0.0:9001,[::]:9001"        // <-- undialable listen spec leaked into peers
]
```

1. **Code bug (fixed in repo, NOT yet deployed).** `GET /api/bootstrap` chained
   the node's own `listen_addr` into the `peers` array. The listen spec is
   already reported in the separate `listen` field, and `0.0.0.0:9001,[::]:9001`
   is not a dialable peer address. The bug was present on **every** node
   (confirmed on the local seed2 unit too: `peers` contained `0.0.0.0:9000,[::]:9000`).
   - Fixed in `crates/kovanica-node/src/explorer.rs` (both `node/` and
     `protocol/` trees); regression assertion added to
     `test_http_bootstrap_returns_light_config`.
   - **Still to do:** deploy the fixed binary to the explorer host — the live
     payload above is produced by the *old* binary.

2. **Ops config (still to do).** The primary VPS (`145.223.116.178`, the main
   seed / "seed1") still lists the **decommissioned**
   `seed3.kovanica.online:9000` in `KOVANICA_PEERS`.

### What changed on 2026-09-21 (operator)

- **`seed3` is fully decommissioned**: the EC2 instance (AWS, formerly
  `15.228.170.29`, retired 2026-09-17) was stopped **and its DNS record was
  deleted** — `seed3.kovanica.online` is now **NXDOMAIN** (verified from this
  box). The stale `KOVANICA_PEERS` entry is therefore a dead dial, not a
  contactable node.
- **Host identity clarification**: `145.223.116.178` is the **main seed /
  seed1** (the operator's name for the Hostinger VPS `srv1745734` that fronts
  `explorer.kovanica.online`). Do not confuse it with **seed2**
  (`76.13.250.65`, `srv1991525`) — that is the box this session runs on.
  - ⚠️ Port discrepancy to confirm on-host: `OPERATIONS.md` §1 documents seed1
    as P2P `:9002` / HTTP `:28080` and seed2 as P2P `:9001` / HTTP `:18080`,
    but the public `/api/bootstrap` reports `listen: 0.0.0.0:9001,[::]:9001`
    (i.e. the explorer's `/api` backend looks like the `:9001` unit). Resolve
    which unit nginx proxies to before editing the service file — see
    remediation step 0.

---

## Remediation (requires access to `145.223.116.178`)

Access from this box was attempted as `root`, `ubuntu`, `deploy`, `kovanica`,
`admin` — all `Permission denied (publickey,password)`. No SSH key in
`~/.ssh/` is authorised on that host. This step must be run by an operator with
access.

```bash
ssh root@145.223.116.178

# 0. Identify the unit nginx proxies /api/* to, so you edit the right one.
grep -R "127.0.0.1:18080\|127.0.0.1:28080" /etc/nginx/sites-enabled/
systemctl cat kovanica-seed1 kovanica-seed2 2>/dev/null | grep -E 'KOVANICA_PEERS|KOVANICA_LISTEN|ExecStart'

# 1. Drop the decommissioned seed3 from the unit env (keep the live primary
#    seed only; do NOT add seed2.kovanica.online if that resolves to this same
#    box — a node must not peer with itself).
UNIT=kovanica-seed2   # <-- set to whichever unit step 0 identified
sed -i 's|^Environment=KOVANICA_PEERS=.*|Environment=KOVANICA_PEERS=seed.kovanica.online:9000|' \
    "/etc/systemd/system/$UNIT.service"
systemctl daemon-reload
systemctl restart "$UNIT"

# 2. Deploy the /api/bootstrap code fix (build on the box from the canonical tree):
#    cd /root/kovanica/node && cargo build --release --workspace
#    install the binary and restart the units per AGENTS.md §5.

# 3. Verify — no seed3, no listen spec:
curl -s https://explorer.kovanica.online/api/bootstrap | jq '.listen, .peers'
# and confirm seed3 really is gone:
getent hosts seed3.kovanica.online || echo "NXDOMAIN (expected)"
```

Expected after fix:

```json
"listen": "0.0.0.0:9001,[::]:9001",
"peers": ["seed.kovanica.online:9000"]
```

---

## Notes

- `P2P_BOOTSTRAP` / `DEFAULT_PEERS` / `dns_seed.rs` already carry only
  `seed.kovanica.online` + `seed2.kovanica.online`; the `seed3` entry is purely
  stale VPS env, not a repo default.
- The local seed2 box (`76.13.250.65`) is clean: `KOVANICA_PEERS=seed.kovanica.online:9000`.
- With `seed3` DNS deleted, a stale entry is harmless at the DNS level but still
  means needless resolve attempts / retry noise in logs and a misleading public
  API payload — worth cleaning rather than leaving.
