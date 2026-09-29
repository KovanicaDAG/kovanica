# Testnet Reset Credentials — Seed1, Seed2, Seed3

> **Purpose**: Single reference for coordinating a simultaneous testnet reset across all 3 seeds.
> **Run from**: any operator box with the release binary built.
> **⚠️ Testnet is STOPPED (2026-09-28)** — see `protocol/docs/NETWORK-LAUNCH-PLAN.md`.
> **This document was factually wrong until 2026-09-29** and has been corrected. See
> [Corrections](#corrections-2026-09-29) at the bottom for what changed and why the
> old version would have misdirected a launch.

---

## Seed Inventory (verified 2026-09-29)

| Seed | Hostname | IP | Provider / box | SSH user | P2P `:9000` | Unit on that box |
|------|----------|-----|----------------|----------|-------------|-----------------|
| **seed1** | `seed.kovanica.online` | `145.223.116.178` | Hostinger DE (`srv1745734.hstgr.cloud`) | `root` | unreachable | `kovanica-explorer` + `kovanica-seed2` |
| **seed2** | `seed2.kovanica.online` | `76.13.250.65` | Hostinger KVM2 LT (`srv1991525`) | `root` | **OPEN — still serving** | `kovanica-seed2` |
| **seed3** | `seed3.kovanica.online` | `187.7.27.139` | Hostinger (`srv2013143.hstgr.cloud`) | `root` | unreachable (stopped) | `kovanica-testnet.service` |

> **All three seeds are operated by the same owner (Toni).** "seed operator" in any
> table below means the same person, not a separate party. The three boxes are
> still three independent machines, which is what matters for fault tolerance.

### IPv6

| Seed | AAAA | Notes |
|------|------|-------|
| seed1 | `2a02:4780:41:1f43::1` | published, grey-cloud |
| seed2 | *none* | v4 only |
| seed3 | *none* | **has** a real global `2a02:4780:f:602c::1/48` on `eth0`, but it is **not published in DNS** — so the node's v6 P2P listener is unreachable from outside despite the address being routable |

seed3's `/48` is a full routed allocation, not a single host. If it is ever
firewalled, scope rules to the specific address — do not match the whole `/48`.

---

## SSH Access

**No private key for any seed is present on this host.** `/root/.ssh/` holds
only `authorized_keys` + `known_hosts`; `/root/seeds/` does not exist. The paths
historically named here (`/root/seeds/seed2/keykovanica`,
`/root/seeds/seed2/seed2keys`, `/root/seeds/seed3/kovanica-seed3.pem`) are all
absent, and `ssh root@76.13.250.65` returns `Permission denied (publickey,password)`.

| Target | Command | Status |
|--------|---------|--------|
| seed1 | `ssh root@145.223.116.178` | no key here — needs port 2222 per `deploy.yml` |
| seed2 | `ssh -i <key> root@76.13.250.65` | key not on this host |
| seed3 (this host) | already here | ✅ local shell |

To restore programmatic access, place a key on this host and record the path in
the table above. Do not paste key material into this file.

---

## Per-seed facts

### seed1 — `145.223.116.178` (Hostinger DE)
Primary seed. This is `VPS_HOST` in `protocol/.github/workflows/deploy.yml`
(SSH **port 2222**). Carries both `kovanica-explorer` and `kovanica-seed2`.
Has IPv6 published. Both `:9000` and `:9090` are currently unreachable — a dead
process and a firewall are indistinguishable from outside; confirm with shell.

### seed2 — `76.13.250.65` (Hostinger KVM2, `srv1991525`)
**Still serving the retired chain.** `:9000` and `:9090` both OPEN as of
2026-09-29. This is the last live node of the stood-down testnet and the reason
a fresh clone can still join the old chain via `DEFAULT_PEERS`. Teardown:

```bash
# on seed2, as root — back up config + data dir FIRST (its genesis marker has
# not been inspected and may differ from seed3's)
systemctl disable --now kovanica-seed2 kovanica-explorer
```

### seed3 — `187.7.27.139` (this host, `srv2013143`)
Unit `kovanica-testnet.service`, `WorkingDirectory=/var/lib/kovanica-seed3`,
`ExecStart=/usr/local/bin/kovanica-node explorer 127.0.0.1:8080`.

**Stopped and disabled** 2026-09-28; unit file retained so the chain can be
restored. Config `/opt/kovanica/testnet/config/network.env` is `0600` root:root
(it holds `KOVANICA_AUTHORITY_KEY`, so it must never be world-readable).

```bash
systemctl status kovanica-testnet
journalctl -u kovanica-testnet -f
```

Pre-stop backup of data dir + config:
`/var/backups/kovanica/seed3-final-20260928-231354.tar.gz` (`0600`).

Network marker in that data dir: `kovanica-testnet`. `ensure_network()` **wipes
the data dir** on a marker mismatch, so restoring it under a different
`KOVANICA_NETWORK` is destructive.

---

## Deploy

```bash
# Build (from the monorepo, protocol/ is the source of truth)
cd /root/kovanica/protocol
cargo build --release -p kovanica-node
```

> Do **not** use `/root/kovanica/kovanica-node` as the build path — the component
> repos were consolidated into the monorepo on 2026-09-24 and that path is a
> stale v0.2.0 duplicate tree.

Deploy **per seed, to its own unit name** — the units are not uniform:

```bash
# seed3 (local)
sudo install -m755 ./target/release/kovanica-node /usr/local/bin/kovanica-node.new \
  && sudo mv -f /usr/local/bin/kovanica-node{.new,}
sudo systemctl restart kovanica-testnet          # NB: kovanica-testnet, not kovanica-seed2

# seed2 (needs a key restored on this host)
./scripts/deploy-seed-prebuilt.sh root@76.13.250.65 \
  --name seed2 --mine --mine-secs 60 \
  --binary ./target/release/kovanica-node \
  --identity-file <key>
```

> ⚠️ `deploy.yml` also runs `systemctl restart kovanica-seed2 || true` on the VPS
> target. That `|| true` means a missing unit is silent — a deploy can report
> success having touched no seed at all. Verify each host individually.
>
> The deploy is gated on the `DEPLOY_ENABLED` repo variable (unset = off) and the
> `VPS_HOST` / `VPS_USERNAME` / `VPS_PRIVATE_KEY` secrets, which are **currently
> absent from every Kovanica repo** — so nothing has been auto-deploying.

---

## Post-Deploy Verification (each host, individually)

```bash
# Genesis must match on ALL seeds — one command per box, not one local + one ssh.
curl -s http://127.0.0.1:8080/api/head | jq -r .genesis                  # seed3
ssh <...> root@145.223.116.178 "curl -s http://127.0.0.1:8080/api/head | jq -r .genesis"
ssh -i <key> root@76.13.250.65 "curl -s http://127.0.0.1:8080/api/head | jq -r .genesis"

curl -s http://127.0.0.1:8080/api/head | jq .peers                      # expect ≥ 1
watch -n 10 'curl -s http://127.0.0.1:8080/api/head | jq .blocks'
```

A **single** seed showing a matching genesis proves nothing about the others —
that mistake is baked into the pre-correction version of this file, which listed
seed3 as "retired — skip" and therefore only ever checked two of three hosts.

---

## DNS (Cloudflare, grey cloud — DNS only, never proxied)

| Record | Type | Content | Proxy |
|--------|------|---------|-------|
| `seed.kovanica.online` | A | `145.223.116.178` | DNS only |
| `seed.kovanica.online` | AAAA | `2a02:4780:41:1f43::1` | DNS only |
| `seed2.kovanica.online` | A | `76.13.250.65` | DNS only |
| `seed3.kovanica.online` | A | `187.7.27.139` | DNS only |
| `seed3.kovanica.online` | AAAA | — | *absent; add if v6 peers are wanted* |

Never orange-cloud a seed: Cloudflare will proxy TCP :9000 and break P2P.

Leaving these records resolving while the seeds are down is harmless and keeps
relaunch cheap. Pulling them makes the network unreachable and needs provider
credentials that are not present on this host.

---

## Reset Checklist

- [ ] **Build** `cargo build --release -p kovanica-node` in `protocol/`
- [ ] **Back up** each seed's config + data dir *before* touching it
- [ ] **Stop** every seed (`systemctl disable --now <its unit>` — see table)
- [ ] **Confirm all three are down**: `ss -ltn | grep :9000` is empty everywhere
- [ ] **Deploy** to each seed under its own unit name
- [ ] **Genesis match** verified on **all three**, individually
- [ ] **Peers ≥ 2** observed
- [ ] **Block production** advancing
- [ ] **Smoke tests**: faucet (if enabled), transfer, multisig, HTLC, vault
- [ ] **Supply check** against `protocol/docs/RFC-006-EmissionCurve.md`
- [ ] **Light-node sync**: Android LightNode → live genesis
- [ ] **Update** `MASTER-ROADMAP.md`

Supply rules that must hold after any reset — these are consensus, not config:
MAX_SUPPLY **90.2M KVNC**, s₀ **10 KVNC/block**, era **2,000,000 blocks**,
α **3/4**, maturity **100 blocks**, fee split **75% burned / 25% producer**,
GHOSTDAG **k=3**, UTXO, Ed25519, **1 KVNC = 100,000,000 atoms**.

---

## Emergency Rollback

```bash
# seed3 (this host)
systemctl stop kovanica-testnet
sudo install -m755 /root/bin/kovanica-node.prev /usr/local/bin/kovanica-node
systemctl start kovanica-testnet

# restore a pre-stop data dir — ONLY under the same KOVANICA_NETWORK,
# or ensure_network() wipes it on the marker mismatch
tar xzf /var/backups/kovanica/seed3-final-20260928-231354.tar.gz -C /
```

Do not hardcode a passphrase in this file or in shell history. Use a secret store.

---

## Corrections (2026-09-29)

Every line below was wrong before this revision and would have caused a real
failure if followed during a launch.

| Was | Actually | How it was found |
|-----|----------|------------------|
| "this VPS = seed1", unit `kovanica-seed2` | this host is **seed3** (`srv2013143` / `187.7.27.139`), unit `kovanica-testnet.service` | `hostname -I` + `systemctl list-unit-files`; the unit's own `Description` already said "seed3" |
| seed3 = `15.228.170.29`, Hostinger→AWS, "retired" | `15.228.170.29` is a **former** seed3 that was decommissioned; the **current** seed3 is this Hostinger box at `187.7.27.139` | `dig +short A seed3.kovanica.online` → `187.7.27.139`; `15.228.170.29:9000` unreachable |
| "Confirmed live topology 2026-09-20: seed + seed2 only" | seed3 has been in the peer set since (`KOVANICA_PEERS` lists seed + seed2, and seed3 itself ran as a third node) | live `network.env` |
| `CHECKPOINT_VERSION: 7` | **`10`** | `crates/kovanica-state/src/ledger.rs:3699` |
| genesis `9565fc20…` presented as the target | not verifiable from this host — seed3's chain was 1 block, tip == genesis, and no genesis is recorded in the data dir or the backup | inspected `/var/backups/kovanica/seed3-final-*.tar.gz` |
| seed2 key at `/root/seeds/seed2/keykovanica` | **no seed key exists on this host**; `/root/seeds/` is absent | `ls`, `find`, `gh secret list` |
| "seed3 retired — skip" in verification steps | would have checked 2 of 3 hosts and called it a pass | — |
| build path `/root/kovanica/kovanica-node` | stale v0.2.0 duplicate; build in `protocol/` | monorepo consolidation 2026-09-24 |
| `systemctl stop kovanica-seed2` in rollback | stops nothing here — that unit does not exist on seed3 | `systemctl list-unit-files` |

A genesis hash is a **consensus** value: every node must agree on it byte for
byte, or they form separate chains. Treat it as something to read from a running
node's `/api/head`, never as a value to carry forward in a doc.

---

*Last verified against live hosts: 2026-09-29*
*Run `./scripts/deploy-seed-prebuilt.sh --help` for deploy options*
