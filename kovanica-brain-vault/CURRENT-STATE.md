# CURRENT-STATE — living snapshot

Updated: 2026-08-25

## Now (one line)

MARKDOWN.god v3 is a self-dogfooding doctrine repository: 7 core docs + 1 memory ledger compile into `MARKDOWN.god`, with a real KovanicaDAG project layer and a 14-surface Project Kit at root.

## In progress

- Complete root Project Kit validation after replacing the five missing scaffold surfaces.
- Keep the universal core project-agnostic while proving the layer mechanism against real downstream projects.

## Verified today

- 7 core markdown documents.
- 1 memory ledger file.
- 9 operational shell scripts.
- 14 Project Kit templates.
- `layers/KovanicaDAG.md` present.
- `bash -n scripts/*.sh` passes.
- `./scripts/build-god-brain.sh --check` passes.
- The previous root kit was incomplete by exactly 5 validator requirements: `EVIDENCE.md`, `AGENT-STATE.md`, `SCOPE.md`, `IMPACT-MAP.md`, and `.agent/tasks`.

## Known broken / debt

- The supplied archive has no `.git/`, so commit/branch/remote state cannot be verified or shipped from this artifact alone.
- The KovanicaDAG protocol source referenced by the project layer is not present in this archive; its claims remain layer data, not independently verified external source truth.
- `sync-brain.sh` only wires the tool surfaces implemented in its current source; broader integrations remain roadmap work.
- Any credentials that were present in the original `SECRETS.local.md` require rotation/revocation and must not be redistributed.

## Next steps (ordered)

1. Run `./scripts/validate-project-kit.sh .` and `./scripts/doctor.sh .` after the root kit completion.
2. Verify the compiled brain remains fresh.
3. Apply the Project Kit to the actual KovanicaDAG repository, where protocol facts can be verified against source rather than the layer snapshot.
4. After 2–3 real project sessions, evaluate which lessons belong in `memory/` versus human-approved `core/` promotion.

## WIP branches & stashes

- Unknown — no `.git/` metadata is present in this archive.
