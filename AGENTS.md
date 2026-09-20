# AGENTS.md

Guidance for AI assistants (and humans) working in the **Kovanica** monorepo.

> **Status**: Active development. Monorepo layout (migration in progress on `chore/monorepo-migration`).
> **Component repos**: `protocol` (core), `node` (node binary), `web` (explorer/wallet), `wallet` (mobile/FFI), `mobile` (Android app).
> **Separate repos** (NOT in the workspace): `kovanica-agent` → `~/kovanica-agent`, `kovanica-brain-vault` → `~/kovanica-brain-vault`.

---

## 1. Repository Structure

```
/root/kovanica/
├── protocol/              # Core consensus + ledger (Rust workspace)
│   ├── crates/kovanica-dag/       # BlockDAG + GHOSTDAG + VRF
│   ├── crates/kovanica-state/     # UTXO ledger, stake registry, hybrid PoW/VRF
│   └── docs/                      # RFCs, KVP specs, LEGIT-BOARD.md
├── node/                  # Runnable node binary + explorer HTTP API
│   ├── crates/kovanica-node/      # Node, mempool, RPC, P2P, explorer
│   └── crates/kovanica-ffi/       # UniFFI bindings for mobile
├── web/                   # Explorer + wallet frontend (TanStack Router, Vite, Nitro)
├── wallet/                # Mobile wallet (UniFFI + Kotlin/Swift)
├── mobile/                # Android app (Jetpack Compose, light node)
├── cli/                   # Rust CLI (was kovanica-cli)
├── installer/             # Install scripts (was kovanica-installer)
├── ledger-app/            # Hardware wallet app (was kovanica-ledger-app, local-only, unversioned)
├── data/                  # Runtime data (NOT versioned; placeholder in git)
└── NETWORK.md             # Testnet seed info, ports, genesis
```

---

## 2. Key Conventions

### Rust Workspace (`protocol/`, `node/`)
- **Edition**: 2021, `rust-version` 1.75+
- **Forbidden**: `unsafe_code` (`#![forbid(unsafe_code)]`)
- **Linting**: `cargo clippy --all-targets` (warning-clean)
- **Formatting**: `cargo fmt --check`
- **Tests**: `cargo test` (unit + integration + doctests)
- **Determinism**: Critical — consensus must be pure function of DAG

### Web (`web/`)
- **Framework**: TanStack Router, Vite, Nitro (SSR)
- **Styling**: Tailwind CSS, custom design system
- **State**: Zustand stores, TanStack Query
- **Build**: `npm run build:vps` (NITRO_PRESET=node-server)
- **Deploy**: PM2 on VPS, Cloudflare in front

### Mobile/FFI (`wallet/`, `node/crates/kovanica-ffi/`)
- **UniFFI**: Generates Kotlin/Swift bindings
- **Bindings committed**: `crates/kovanica-ffi/bindings/{kotlin,swift}/`
- **Drift guard**: CI regenerates and fails on diff (`.github/workflows/bindings.yml`)

---

## 3. Build & Test Commands

| Repo | Build | Test | Lint/Format |
|------|-------|------|-------------|
| `protocol/` | `cargo build` | `cargo test` | `cargo fmt --check && cargo clippy --all-targets` |
| `node/` | `cargo build --workspace` | `cargo test --workspace` | Same as protocol |
| `web/` | `npm run build:vps` | `npm run test` | `npm run lint && npm run format:check` |
| `wallet/` | `./gradlew assembleDebug` | `./gradlew test` | `ktlint` |

---

## 4. Git Workflow

- **Never commit to `main` directly**. Feature branch → draft PR.
- **Branch naming**: `consensus/...`, `dag/...`, `ledger/...`, `web/...`, `mobile/...`
- **Commits**: Imperative subject, clear *why*. Run lint/test before push.
- **Rebase**: Keep history linear. `git pull --rebase origin main` before push.

### 4.1 Multi-Agent Safety (CRITICAL)

This repo is worked on by multiple AI agents/models/providers concurrently. Silent overwrites (one agent's push erasing another agent's work) are the #1 source of lost work. Every agent MUST follow these rules without exception:

- **One agent, one branch.** Never let two agents work on the same branch concurrently. Each agent session gets its own branch (e.g. `consensus/vrf-fix-agentA`, `web/wallet-view-agentB`). If you must share a branch, coordinate explicitly first — never assume you're alone on it.
- **Never `git push --force`.** Force push is banned outright. If history needs rewriting after a rebase, use:
  ```bash
  git push --force-with-lease
  ```
  `--force-with-lease` refuses the push if the remote has commits you haven't seen — this is what actually prevents overwriting another agent's work. Plain `--force` does not check this and WILL silently destroy others' commits.
- **Before ANY commit**: run `git status` and `git log origin/<branch> -1` to check whether the branch has moved upstream since you last synced.
- **Before ANY push**: run `git fetch origin` then `git diff HEAD origin/<branch>` and actually review the diff. Never push blind. If the diff shows commits you don't recognize, STOP and investigate before proceeding — do not overwrite them.
- **Rebase, don't merge, and do it often**: `git pull --rebase origin main` before starting new work and again immediately before pushing.
- **Never edit files another agent is actively touching** without first pulling their latest commit. If unsure who touched a file last: `git log -1 --format='%an %ar' -- <file>`.
- **Never delete or rewrite commits you didn't author** without explicit human confirmation — even if they look like "leftover" or "broken" work. Ask Toni first.
- **PR-only merges to main**: main is protected (see 4.2) — this is the real backstop if an agent violates any rule above.

### 4.2 Required GitHub Branch Protection (main)

Set once per repo, in GitHub → Settings → Branches → `main`:

- ✅ Require a pull request before merging
- ✅ Require branches to be up to date before merging
- ✅ Require status checks to pass before merging (CI: lint + test)
- ✅ Require linear history
- ❌ **Do not allow force pushes** (Restrict force pushes)
- ❌ **Do not allow deletions**
- ✅ (Optional but recommended) Require signed commits

### 4.3 Local Git Safety Config

Run once per machine/agent environment:

```bash
git config --global push.default simple
git config --global pull.rebase true
git config --global branch.autosetuprebase always
git config --global push.useForceIfIncludes true   # extra safety with --force-with-lease
```

### 4.4 Pre-push Hook (install in every clone)

Save as `.git/hooks/pre-push`, then `chmod +x .git/hooks/pre-push`:

```bash
#!/bin/bash
# Refuses push if local branch is not rebased on latest origin/main
set -e
BRANCH=$(git rev-parse --abbrev-ref HEAD)

git fetch origin main --quiet

BASE=$(git merge-base HEAD origin/main)
REMOTE=$(git rev-parse origin/main)

if [ "$BASE" != "$REMOTE" ]; then
  echo "❌ Branch '$BRANCH' is not rebased on origin/main."
  echo "   Run: git pull --rebase origin main"
  exit 1
fi

# Extra guard: block accidental --force (not --force-with-lease) on shared branches
if [ "$BRANCH" = "main" ]; then
  echo "❌ Direct push to main is forbidden. Open a PR instead."
  exit 1
fi

echo "✅ Pre-push checks passed for branch '$BRANCH'."
exit 0
```

Since hooks aren't committed by git, distribute it via a setup script (e.g. `scripts/install-hooks.sh`) that every agent/human runs once after cloning:

```bash
#!/bin/bash
# scripts/install-hooks.sh
HOOK_DIR="$(git rev-parse --git-dir)/hooks"
cp AGENTS/pre-push "$HOOK_DIR/pre-push"
chmod +x "$HOOK_DIR/pre-push"
echo "✅ Git hooks installed."
```

---

## 5. Network & Deployment

### Testnet (Current)
- **Seed** (primary): `seed.kovanica.online:9000` (Hostinger VPS; live systemd unit `kovanica-explorer`, HTTP `127.0.0.1:8080`)
- **Seed2** (secondary): `seed2.kovanica.online:9000` (AWS EC2 `76.13.250.65`, re-keyed from `seed3` 2026-09-17)
- **VPS seed units**: `kovanica-seed1` (P2P `:9002`, HTTP `127.0.0.1:28080`) + `kovanica-seed2` (P2P `:9001`, HTTP `127.0.0.1:18080`, nginx `/api` backend)
- **Explorer**: `https://explorer.kovanica.online` (PM2 web on `:3000` + nginx → `:18080` for API)
- **Faucet**: `https://faucet.kovanica.online` (1 tKVNC, rate-limited)
- **Genesis**: `kovanica-testnet` (k=3, subsidy 200*ATOM, founder seed=1)

### VPS Deploy (kovanica-web)
```bash
cd /root/kovanica/web/site
npm run build:vps    # NITRO_PRESET=node-server
pm2 restart kovanica-web
```

### VPS Deploy (kovanica-node)
```bash
cd /root/kovanica/node
cargo build --release --workspace
systemctl restart kovanica-explorer kovanica-seed1 kovanica-seed2
```

---

## 5. Consensus Architecture (Critical)

### GHOSTDAG (kovanica-dag)
- **k-parameter**: 3 (testnet)
- **Ordering**: Recursive `order(B) = order(sp) ++ mergeset ++ [B]`
- **Reachability**: Interval-tree + future-covering sets (incremental, Kaspa-style)
- **Blue score / work**: Drives chain selection

### Ledger (kovanica-state)
- **UTXO model**: Ed25519 spend auth, per-asset conservation (KVP-102)
- **Per-block state**: Incremental from selected parent + mergeset
- **Finality pruning**: Depth-based, folds deltas into children
- **Stake registry**: Bond/unbond via tags (`KVB1||vrf_pk`, `KVU1`), maturity 100 blocks

### Hybrid Admission (kovanica-node)
- **PoW path**: Hash target `H*work < 2^256`, difficulty retargeting
- **Staked-VRF path**: VRF over epoch beacon, eligibility threshold
- **Sibling guard**: One staked block per `(vrf_pk, selected_parent)`

### Upgraded RFCs (All on `main`)
| RFC | Feature | Activation |
|-----|---------|------------|
| 001 | Multisig (M-of-N P2SH) | Blue score 0 |
| 002 | Native tokens (KVP-102) | Blue score 0 |
| 003 | Stealth + Script v2 | Blue score 0 |
| 004 | HTLC / Atomic swaps | Blue score 0 |
| 005 | Vault / CSV | Blue score 0 |

---

## 6. Important Files to Know

| File | Purpose |
|------|---------|
| `protocol/docs/LEGIT-BOARD.md` | P0/P1/P2 roadmap checklist |
| `protocol/docs/AUDIT-PLAN.md` | Audit scope, firms, timeline |
| `protocol/docs/REPRODUCIBLE-BUILDS.md` | Toolchain pinning, CI verification |
| `protocol/docs/BUG-BOUNTY.md` | Severity tiers, payouts, safe harbor |
| `protocol/docs/MAINNET-CRITERIA.md` | Exit checklist for mainnet launch |
| `protocol/docs/TESTNET-SOAK.md` | 30-day soak plan, metrics |
| `protocol/docs/ENTITY-LEGAL.md` | Maintainer identity, disclaimers |
| `protocol/docs/COMMUNITY-DISCORD.md` | Server structure, moderation |
| `protocol/docs/OPS-HARDENING.md` | Backups, alerts, seed hardening |
| `protocol/docs/PRODUCT-POLISH.md` | Hardware wallet, deep-links, fee est |
| `NETWORK.md` | Seed peers, ports, genesis params |
| `protocol/crates/kovanica-state/src/stake.rs` | Multi-asset stake registry |

---

## 7. Common Tasks

### Add Consensus Feature
1. Write RFC in `protocol/docs/RFC-XXX-*.md`
2. Implement in `kovanica-dag` or `kovanica-state`
2. Add adversarial tests (`tests/consensus.rs`, `tests/*_consensus.rs`)
3. Update activation score constant + `Ledger::set_*_activation_score`
4. Update `LEGIT-BOARD.md` status

### Web Feature
1. Routes in `web/site/src/routes/`
2. Components in `web/site/src/components/`
3. API in `web/site/src/lib/api/`
4. Deploy: `npm run build:vps && pm2 restart kovanica-web`

### FFI Addition
1. Add method to `LightNode` in `kovanica-ffi/src/light_node.rs`
2. Regenerate bindings: `cargo run -p kovanica-ffi --bin uniffi-bindgen generate ...`
3. Commit generated Kotlin/Swift under `bindings/`
4. CI drift guard will verify

---

## 8. Testing Standards

- **Consensus**: Adversarial + property tests (Byzantine parents, wide forks >k, tie-breaks)
- **Ledger**: Double-spend across parallel blocks, order-independence, snapshot roundtrip
- **P2P**: Multi-node convergence, conflict resolution, mempool eviction
- **FFI**: Lifecycle, seed validation, bond/split/freeze, blob sync convergence
- **Web**: SSR hydration, wallet flows, asset picker, fee estimation

---

## 9. Debugging Tips

| Issue | Where to Look |
|-------|---------------|
| Consensus fork | `kovanica-dag/tests/consensus.rs` — `adversarial_wide_fork` |
| Mempool eviction | `node/crates/kovanica-node/src/mempool_v2.rs` |
| P2P not connecting | `node/crates/kovanica-node/src/p2p_hardening.rs` |
| Fee estimation | `node/crates/kovanica-node/src/mempool_v2.rs:fee_estimate` |
| Wallet balance wrong | `web/site/src/components/wallet/wallet-view.tsx` |
| Map black on mobile | `web/site/src/components/map/dashboard.tsx` (h-[46vh] min-h-[300px]) |

---

## 10. Security

- **No secrets in repo** — VPS keys, `.env`, private workflows stay private
- **Security reports**: GitHub Security Advisories or `security@kovanica.online`
- **Bug bounty**: See `protocol/docs/BUG-BOUNTY.md`
- **Audit**: Target Q1 2027, scope = dag + state + RPC

---

## 11. Quick Reference

```bash
# Full test suite (protocol)
cd /root/kovanica/protocol && cargo test

# Full test suite (node)
cd /root/kovanica/node && cargo test --workspace

# Web build + deploy
cd /root/kovanica/web/site && npm run build:vps && pm2 restart kovanica-web

# Check all hosts healthy
for h in explorer.kovanica.online testnet.kovanica.online kovanica.online mainnet.kovanica.online api.kovanica.online docs.kovanica.online faucet.kovanica.online; do echo "=== $h ==="; curl -s -4 -o /dev/null -w "%{http_code}" "https://$h/" && echo; done

# Verify branch is safe to push (matches pre-push hook logic)
git fetch origin main --quiet && git diff HEAD origin/main --stat
```

---

*This file is the primary entry point for agents. Update it when conventions change.*
