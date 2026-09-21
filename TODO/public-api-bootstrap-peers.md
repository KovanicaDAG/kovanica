# Public API — `/api/bootstrap` peer leak & stale `seed3`

**Status**: ⚠️ **Code fix done; VPS config change BLOCKED (no SSH access)**
**Discovered**: 2026-09-21
**Severity**: Low (client-only / API surface + ops hygiene) — no consensus or ledger impact.

---

## Findings

Two separate problems were visible in the public bootstrap payload:

```json
// https://explorer.kovanica.online/api/bootstrap  (2026-09-21)
"listen": "0.0.0.0:9001,[::]:9001",
"peers": [
  "seed.kovanica.online:9000",
  "seed3.kovanica.online:9000",
  "0.0.0.0:9001,[::]:9001"     // <-- undialable listen spec leaked into peers
]
```

1. **Code bug (fixed in repo).** `GET /api/bootstrap` chained the node's own
   `listen_addr` into the `peers` array. The listen spec is already reported in
   the separate `listen` field, and `0.0.0.0:9001,[::]:9001` is not a dialable
   peer address. The bug was present on **every** node (confirmed on the local
   seed2 unit too: `peers` contained `0.0.0.0:9000,[::]:9000`).
   - Fixed in `crates/kovanica-node/src/explorer.rs` (both `node/` and
     `protocol/` trees); regression assertion added to
     `test_http_bootstrap_returns_light_config`.

2. **Ops config (still to do).** The primary VPS (`145.223.116.178`, which
   fronts `explorer.kovanica.online` and runs its local `kovanica-seed2` unit,
   P2P `:9001`, HTTP `127.0.0.1:18080`) still lists the **retired**
   `seed3.kovanica.online:9000` in `KOVANICA_PEERS`. `seed3` (AWS
   `15.228.170.29`) was retired 2026-09-17 — see `OPERATIONS.md` §1.

---

## Remediation (requires access to `145.223.116.178`)

Access from this box was attempted as `root`, `ubuntu`, `deploy`, `kovanica`,
`admin` — all `Permission denied (publickey,password)`. No SSH key in
`~/.ssh/` is authorised on that host. This step must be run by an operator
with access.

```bash
# 1. Drop the retired seed3 from the unit env (keep the live primary seed only;
#    do NOT list seed2.kovanica.online here — that resolves to this same box).
ssh root@145.223.116.178
sed -i 's|^Environment=KOVANICA_PEERS=.*|Environment=KOVANICA_PEERS=seed.kovanica.online:9000|' \
    /etc/systemd/system/kovanica-seed2.service
systemctl daemon-reload
systemctl restart kovanica-seed2

# 2. Deploy the /api/bootstrap code fix (build on the box from the canonical tree):
#    cd /root/kovanica/node && cargo build --release --workspace
#    install the binary and restart the units per AGENTS.md §5.

# 3. Verify — no seed3, no listen spec:
curl -s https://explorer.kovanica.online/api/bootstrap | jq '.listen, .peers'
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
