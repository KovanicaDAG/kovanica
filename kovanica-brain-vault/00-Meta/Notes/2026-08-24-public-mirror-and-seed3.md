# 2026-08-24 — Public Mirror Pipeline & seed3 Deployment

Session log for [[ROADMAP|Kovanica]] infrastructure work: the public
`kovanica-node` repo became self-updating (mirror + prebuilt releases), and
**seed3** — the first true off-box node — is live on AWS.

## 1. Public mirror pipeline (`sync-public-node`)

One workflow in `kovanica-protocol` now does mirror → build → publish on every
push to `main` touching `crates/**` or workspace manifests (plus manual dispatch):

1. **sync** — rsync `crates/{kovanica-dag,kovanica-state,kovanica-node}` into
   [KovanicaDAG/kovanica-node](https://github.com/KovanicaDAG/kovanica-node)
   (`--delete`, runtime `data/`+`target/` excluded), copies `Cargo.lock`,
   rewrites root manifest (membership filtered — **kovanica-cli stays private**,
   repository URL pointed at the mirror). Public-only files (README, TESTNET.md,
   JOIN.md, `scripts/install.*`, deploy/) are never touched.
2. **build** — release binaries: Linux x86_64 + aarch64 (**static musl**, aarch64
   cross via cargo-zigbuild + zig) and macOS x86_64 + Apple Silicon.
3. **publish** — rolling tag `v<workspace-version>` (currently **v0.1.0**) on the
   public repo, replaced in place; assets carry sha256. Publish skips if any
   build fails — never a partial release.

Auth: `NODE_PUBLIC_TOKEN` secret (classic PAT, repo-scoped). Lessons burned in:
runner credential shadowing (→ `persist-credentials: false` + explicit PAT
remote URL), zig backend missing, raw.githubusercontent 404 on private repos.

## 2. install.sh — prebuilt-first

[kovanica-node#2](https://github.com/KovanicaDAG/kovanica-node/pull/2): the
one-liner now fetches the latest release asset (seconds, no toolchain);
source build remains the fallback path. Smoke-tested: static-pie ELF runs,
REPL responds.

## 3. deploy-seed.sh — Amazon Linux support

PR #15: package-manager detection (`apt-get` vs `dnf`). AL2023 quirk recorded:
it ships `curl-minimal`, so naming `curl` in a dnf transaction conflicts —
the dnf branch installs only gcc/gcc-c++/make/pkgconfig.

## 4. seed3 — first off-box node

| | |
|---|---|
| Instance | AWS EC2 `t3.micro`, **eu-north-1** (Stockholm), Amazon Linux 2023 x86_64, 2 vCPU |
| Address | `seed3.kovanica.online:9000` → A record (DNS-only) → `3.79.148.71` |
| Service | systemd `kovanica-seed3` (`Restart=always`), data `/var/lib/kovanica-seed3` |
| Flags | `--mine` on; explorer/metrics loopback-only per runbook |
| Verify | genesis `76cc019d…` matches testnet; headers-first sync climbing past launch; P2P reachable through DNS |
| Access | permanent ed25519 keys on the ops box (`~/.ssh/aws_seed3`, ssh alias `seed3`) |

Deployment saga worth remembering: instance was first launched as Amazon Linux
with a console-created keypair nobody had on the ops box — resolved via EC2
Instance Connect one-liner installing permanent keys, then relaunching cleanly.
**Open security item:** rotate the AWS keypair whose `.pem` was shared in chat.

## 5. Open follow-ups

- [ ] Add `seed3.kovanica.online:9000` to default `KOVANICA_PEERS`
      (public install.sh + deploy-seed.sh defaults)
- [ ] Rotate the chat-shared AWS keypair pem
- [ ] Decide `kovanica-cli` publication (mirror excludes it by design)
- [ ] Optional Windows release assets for install.ps1
- [ ] seed3 soak watch: peers / block rate / memory over week one

## Links

- PRs #10–#14 (pipeline), #15 (dnf), #16 (seed3 docs), #17 (TODO close-out)
- Release: https://github.com/KovanicaDAG/kovanica-node/releases/tag/v0.1.0
- Runbook updates: [[OPERATIONS]] §3 (DNS table) + roadmap naming line

---

## Nastavak sesije (poslijepodne): Soak kickoff + systemd odluka

**Roadmap:** Post-Stage 3 stavke 1–3 i 5 ✅; stavka 2 (multi-seed) kod gotov,
wiring dovršen danas; **stavka 4 (testnet soak) = ACTIVE NEXT** (PR #19, #20).

### Što je napravljeno
| Stavka | Detalj | PR |
|---|---|---|
| DNS seed list | `seed2` A zapis dodan (Cloudflare); mrtvi hostovi izbačeni iz `DnsSeedConfig::default`, dodan `seed3` | #20 |
| Peers rollout | `seed3.kovanica.online:9000` u defaulte (`install.sh` javni repo #3, `deploy-seed.sh`) | kovanica-node#3 |
| Live mesh | seed↔seed2↔seed3 međusobno; svaki vidi 2 odgovarajuća peera | env fix |
| Peer metrika | `kovanica_peer_count` do sada NIJE bio emitiran; sad broji peerove koji su odgovorili na zadnji sync krug (`live_peers`, tick svakih ~5 s) | #21–#23 |
| Prometheus | na VPS-u, UI `127.0.0.1:19080`; seed direktno :9090, seed3 preko SSH tunela `kovanica-tunnel-seed3` (:19090); 15 alertova + 9 recording pravila; `humanizeBytes` bug popravljen | #21, #27 |
| Baseline | 2026-08-24 16:20 UTC: height 448/447, peers 2/2, mempool 0, orphans 0, blue_score≈height, bez reorga → OPERATIONS.md §5 | #27 |
| **systemd odluka** | pm2 penzioniran za Kovanica procese (port-borba nadzornika); auto-deploy radi atomski swap u `/usr/local/bin/kovanica-node` (ETXTBSY lekcija: seed2 izvodi isti put) + restart oba lokalna unit-a | #25, #26 |
| Hygiene | pem rotacija dovršena; grane počišćene (30 remote + 16 lokalnih); jedini jedinstveni neispaočen patch arhiviran kao tag `archive/geo-origin-node-policy` | — |

### Lekcije (vidi OPERATIONS.md §4)
- Dva supervisora nad istim portovima = kaos: pm2 "siročad" procesi preživljavaju systemd restarte.
- `git push` na već spojenu granu tiho gubi commit (#22 slučaj) — uvijek nova grana.
- In-place `cp` preko izvršnog binaryja koji dijele dva servisa = ETXTBSY; `install`+`mv`.

### Otvoreno
- Tuning review za 1–2 tjedna: `k`, finality depth, payload pruning depth, difficulty window
- Backlog: `kovanica-cli` publikacija, Windows release assets, geo-origin revival (po potrebi)
