# Teamwork Project Prompt — Draft

> Status: Ready for launch — awaiting user approval
> Goal: Craft prompt → get user approval → delegate to teamwork_preview
> Requested team: Small focused team

This is a single self-contained fix; keep it small and focused. Fix the DNS-seed list and P2P routing bugs in the node codebase, and update the WHAT-WE-LEARNED.md ledger according to the Reflection Protocol.

Working directory: /root
Integrity mode: development

## Requirements

### R1. Resolve DNS-Seed and Routing Bugs
Investigate and resolve any remaining DNS-seed list issues and P2P routing bugs in the `kovanica-node` networking stack.

### R2. Update the Learning Ledger
Update the `WHAT-WE-LEARNED.md` ledger in the `MARKDOWN.god` (or `Obsidian-Vault`) repository, following the Reflection Protocol to persist recent session insights.

## Acceptance Criteria

### DNS & P2P Stability
- [ ] `cargo test -p kovanica-node` executes successfully with 0 failures, ensuring the DHT and networking tests remain green.

### Doctrine Integrity
- [ ] `WHAT-WE-LEARNED.md` contains the newly persisted insight entries.
- [ ] `scripts/build-god-brain.sh` executes successfully, cleanly recompiling the agent brain file.

---
*Next: when approved → delegate via invoke_subagent (see Delegation Protocol)*
