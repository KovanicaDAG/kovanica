# KNOWN-TRAPS — traps and false alarms

### Root Project Kit can look complete while failing validation

- **Looks like:** the root already has the familiar 9 documentation files.
- **Actually:** v3 validation requires 14 surfaces plus `.agent/tasks`.
- **So:** run `./scripts/validate-project-kit.sh .`, not a visual file-count guess.

### `MARKDOWN.god` is generated

- **Looks like:** the compiled file is the easiest place to edit doctrine.
- **Actually:** it is an output artifact and freshness checks compare it with its sources.
- **So:** edit `core/`, `memory/`, or explicit layers, then rebuild.

### A Kovanica layer is not the same as a Kovanica source checkout

- **Looks like:** `layers/KovanicaDAG.md` contains protocol-specific rules and paths.
- **Actually:** the external protocol repository is not present in this archive.
- **So:** treat layer claims as project-layer evidence until the referenced repository is available for direct inspection.

### A zip archive is not a Git repository

- **Looks like:** filenames and session notes can imply commit state.
- **Actually:** the supplied archive has no `.git/` directory.
- **So:** never claim a commit, branch, remote, or push was performed from this artifact.

### `SECRETS.local.md` is not safe project context

- **Looks like:** it is a useful local operations note.
- **Actually:** the input archive contained plaintext tokens and infrastructure references.
- **So:** redact/exclude it from shipped artifacts and rotate credentials that were exposed.

### Root docs must not silently become universal doctrine

- **Looks like:** concrete Kovanica details make the agent smarter.
- **Actually:** core doctrine is deliberately project-agnostic.
- **So:** put concrete downstream knowledge in `layers/` or the downstream project's own kit.
