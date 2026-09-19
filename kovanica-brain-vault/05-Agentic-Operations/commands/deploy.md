---
description: Deploy to testnet (requires DEPLOY_ENABLED)
agent: release-engineer
---

Deploy kovanica-protocol to testnet.

Prerequisites:
- `DEPLOY_ENABLED=true` in GitHub repo variables
- `VPS_HOST`, `VPS_USERNAME`, `VPS_PRIVATE_KEY` secrets set
- All tests passing on main branch

Manual trigger:
```bash
gh workflow run deploy.yml -R KovanicaDAG/kovanica-protocol
```

Or merge to main — auto-deploy runs via `.github/workflows/deploy.yml`.

Post-deploy verification:
- `curl https://explorer.kovanica.online/api/head`
- Check dual-stack listeners (`0.0.0.0:P` + `[::]:P`)
- Verify peer exchange (seed ↔ peers)
- Test WebSocket `/ws` and TAP faucet `/api/tap`

$ARGUMENTS — optional: "verify" to run post-deploy checks only

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.
