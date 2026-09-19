# ROADMAP — stages and checklists

Direction of travel. Current state is verified where marked; proposed work is
not represented as already shipped.

## Stage 1 — Harden the brain **(done)**

- [x] Fix documentation drift in the repo's own kit.
- [x] Resolve the AGENTS.md core-edit authority contradiction.
- [x] Make scripts path-portable.
- [x] Make compiler file enumeration space-safe.
- [x] Add compiled-brain freshness protection.
- [x] Add no-shell/mobile session handling.
- [x] Scaffold a Project Kit into this repository.
- [x] Dogfood all 14 Project Kit surfaces with repository-specific data.

## Stage 2 — Prove it across real projects **(current)**

- [ ] Run the completed Project Kit against the actual KovanicaDAG repository.
- [ ] Populate KovanicaDAG `VERIFIED-FACTS` and `CODE-MAP` from source, not only from the layer snapshot.
- [ ] Build a KovanicaDAG-specific brain using `layers/KovanicaDAG.md` and verify that project-specific knowledge remains outside core doctrine.
- [ ] Apply the kit to the cyberdeck build.
- [ ] Apply the kit to the Obsidian↔Claude integration.
- [ ] After 2–3 real project sessions, evaluate lessons for memory/core promotion.

## Stage 3 — Wire more surfaces

- [ ] Extend `sync-brain.sh` to additional agent/tool surfaces if needed.
- [ ] Decide whether mobile-only sessions need a lighter entry point.
- [ ] Add version/hash signaling to make stale compiled brains obvious outside the repository.

## Backlog

- [ ] Automate Git hook installation.
- [ ] Improve project-layer discovery and validation.
- [ ] Add stronger cross-checks between root Project Kit facts and actual filesystem state.
