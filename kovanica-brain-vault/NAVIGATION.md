# NAVIGATION — Kovanica Vault Index

Vault root: `/root/Obsidian-Vault`. The Kovanica project snapshot lives under
`KovanicaDAG/`. Authoritative source: `/root/kovanica` (see
`KovanicaDAG/myObsidianVaultDAG.md`).

---

## KovanicaDAG/ — project snapshot

| Note | Purpose |
|------|---------|
| [myObsidianVaultDAG.md](myObsidianVaultDAG.md) | **Start here** — project overview, repo map, network, build/test |
| [ROADMAP.md](ROADMAP.md) | Stage/RFC progress, legit board P0/P1/P2, mainnet track, open items |
| [CODE_INDEX.md](CODE_INDEX.md) | Topic → source `file://` links (crates, docs, clients, ops) |
| [AGENTS.md](AGENTS.md) | Operating guide, synced from `/root/kovanica/AGENTS.md` |
| [NETWORK.md](NETWORK.md) | Testnet/mainnet network map, seeds, genesis, synced from source |

### Snapshots (read-only mirrors of repo top-level docs)

| Folder | Mirrors | Contents |
|--------|---------|----------|
| `kovanica-protocol/` | `/root/kovanica/kovanica-protocol` | AGENTS, README, OPERATIONS, TESTNET(-RFC006), HOWTO_*, Restructure*, docs/ (RFC 001–005, KVP, TOKENOMICS, LEGIT-BOARD, MAINNET-CRITERIA, plans, api) |
| `kovanica-node/` | `/root/kovanica/kovanica-node` | README, JOIN, TESTNET (operator-facing release repo) |
| `kovanica-web/` | `/root/kovanica/kovanica-web` | README + site/ README, DEPLOY, UI-PARITY-CHECKLIST |
| `kovanica-wallet/` | `/root/kovanica/kovanica-wallet` | README (Android/iOS/extension) |
| `kovanica-mobile/` | `/root/kovanica/kovanica-mobile` | README (light-node clients) |
| `kovanica-agent/` | `/root/kovanica/kovanica-agent` | README + docs/ |

> Re-sync snapshots from source; never edit in place expecting propagation.

---

## Related Notes Outside KovanicaDAG/

- `/root/kovanica/kovanica-brain-vault/` — the ecosystem knowledge base
  (numbered 00-Meta … 05-Agentic-Operations + skills). Nested copy lives in
  `kovanica-agent/kovanica-brain-vault/`.
- `/root/Obsidian-Vault/kovanica-protocol/` — **obsolete** embedded clone of the
  old single-monorepo layout (kept for reference; superseded by `/root/kovanica`).
- `/root/Obsidian-Vault/Poslovno/KovanicaDAG/` — business-side notes
  (UPGRADE-PHASES.md referenced from protocol AGENTS.md).

---

## Quick Reference

- **Project entry point:** `/root/kovanica/AGENTS.md`
- **Protocol truth:** `/root/kovanica/kovanica-protocol/AGENTS.md` + `docs/RFC-*`
- **Roadmap truth:** `kovanica-protocol/docs/LEGIT-BOARD.md` · `MAINNET-CRITERIA.md`
- **Testnet:** seed / seed3 `.kovanica.online:9000` · explorer.kovanica.online ·
  faucet.kovanica.online · seed2 pending (`/root/kovanica/TODO/seed2-deploy.md`)
- **Vault sync recipe:** edit under `KovanicaDAG/` → `git add -A` →
  `git commit -m "docs: …"` → `git pull --rebase origin main` (if rejected) →
  `git push origin main`