# Apps moved

Client apps previously under `apps/*` now live in:

**https://github.com/KovanicaDAG/kovanica-apps**

Migrated from this monorepo tip `e98b5e008331056d6bea55014982d4077e745b77` (flatten: each former `apps/<name>` is a top-level directory there).

## Still in L1

- `crates/kovanica-light-ffi` (UniFFI bindings) — **do not move**
- Consensus, genesis, light client crates, seed nginx — stay here

Build FFI / `kovanica-node` from this repo, then consume the artifacts from `kovanica-apps`.
