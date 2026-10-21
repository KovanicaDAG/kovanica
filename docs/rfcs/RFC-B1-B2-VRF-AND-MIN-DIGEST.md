# RFC draft: B1 VRF leader election and B2 min-digest fork resolution

Status: **draft / not activated**. Default consensus behaviour is unchanged.

This document records the subset that can ship without a wire-format break.
It is not a network upgrade.

> **Update 2026-10-21 (TASKLIST item 15).** The VRF implementation is the
> `ecvrf` 0.4.3 crate (ristretto255 + SHA3) — **not** an RFC 9381 ciphersuite.
> The proof is **96 bytes** (`Gamma(32) ‖ c(32) ‖ s(32)`), carried in
> `StatementBlock::statements` behind the `b"KVNC-VRF1"` magic (137 bytes total)
> and covered by `compute_digest_statements`. The RFC 9381 80-byte proof length
> does **not** apply. Suite, byte layout and the full file:line map:
> `docs/VRF-PROOF-LENGTH.md`.
>
> This note supersedes the "Not activated" wording below for the *statements
> framing* only: the digest-covered `b"KVNC-VRF1"` VRF blob does exist in the
> tree today. The VRF **leader-election** activation items (1-4 below) are
> unchanged — `use_vrf_leader` still defaults to false.

## B1 — stake-weighted VRF leader election

### Already in tree

- `kovanica-crypto` implements the VRF (`vrf_prove` / `vrf_verify`) on top of the
  `ecvrf` 0.4.3 crate — ristretto255 + SHA3, **not** an RFC 9381 ciphersuite
  (proof layout and the 96 B vs 80 B question: `docs/VRF-PROOF-LENGTH.md`).
- `Authority` / `AuthorityInfo` already carry an optional `vrf_public_key`.
- `Committee::leader_with_vrf` selects the lowest stake-weighted VRF hash when
  every authority in the map has an output, and otherwise falls back to
  round-robin.

### Not activated

`StatementBlock` has no VRF proof field. `compute_digest` does not cover one.
Adding a required field, or treating an optional field as consensus-critical
without a coordinated upgrade, splits the network.

Until a block carries a verifiable VRF output:

- `ConsensusEngine::scheduled_leader_for_round` stays round-robin
  (`CommitteeInfo::leader`), unless an explicit `leader_schedule` override is set.
- `ConsensusConfig::use_vrf_leader` exists and **defaults to false**.
- When the flag is true, `scheduled_leader_for_round` ignores the
  round-robin prefetch (`leader_schedule` for rounds 0..1000) and calls
  `CommitteeInfo::leader_with_vrf` with `record_verified_vrf_output` entries
  for that round. An empty map is round-robin. The node must not record a
  hash it has not verified. Live blocks still carry no proof, so the
  default-off flag does not change the scheduled leader.

### Activation requirements (not in this change)

1. Optional, digest-covered VRF proof on leader blocks, versioned so old
   blocks still hash the same way, **or** a hard-fork height after which the
   field is required.
2. Every authority publishes a VRF public key in the committee.
3. Peers verify the proof against
   `domain || chain_id || epoch || round || author` before the output may
   enter `leader_with_vrf`.
4. Missing or invalid proofs must not silently fall back to a different
   leader than honest peers. Fallback is only legal when **no** authority
   has a VRF key (pre-upgrade committees).

## B2 — min-digest fork resolution

### Live default

For a scheduled leader slot, the first block that validates is canonical.
A later block from the same author and round with a different digest is
rejected as equivocation. Tests lock this
(`fork_first_valid_digest_wins_deterministic`).

The committer's `update_leader` already prefers the lexicographically smaller
digest in its in-memory leader map. That map is not the DAG. Preferring a
hash the store does not contain would commit a block that was never accepted.

### Gated path

`ConsensusConfig::use_min_digest_fork` defaults to **false**.

When it is true, `process_block` may replace the stored leader block only if
`DagStoreTrait::replace_block` succeeds. Replacement must:

- remove the previous digest from the store and from parent/child indexes
- refuse if either digest is already a parent of another stored block
- refuse if the round is already decided

`DagStore::replace_block` implements that swap in one write transaction and
refuses a decided round, a missing existing digest, an already-stored
incoming digest, or an existing digest that already has children. The node
adapter calls it. Other `DagStoreTrait` impls keep the default refusal.

If replacement is unsupported or refused, the new digest is rejected. The
engine must not update leader state to a digest the DAG does not hold.

Transaction re-execution is out of scope. Replacement is a pre-commit store
swap only. A decided round is final.

## What this change does not do

- It does not alter `StatementBlock` or `compute_digest`.
- It does not change default leader selection.
- It does not change default equivocation handling.
- It does not re-execute transactions on a fork.
