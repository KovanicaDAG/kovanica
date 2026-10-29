# PRO-40 — late leader votes: commit walk stall (observed 2026-10-29)

This branch is the **team pickup point** for the uncovered consensus stall:

> fresh branch from `main` (`979adbf`) so the next person can start work on the
> late-votes / WASM stall without pulling stale merged branches.

It supersedes the merged `feat/pro40-late-votes-wasm` (`c357e26`, already merged
as part of the buffer/late-vote + WASM work).

## Symptom

During a quorum cluster run the commit walk stopped making progress, while later
rounds kept reaching quorum:

- the log cluster `target/quorum-logs/1793256534/` shows the commit path
  stalling on a single slot;
- rounds far ahead of it (`45`, `51`, `57`, …) reached quorum and committed, e.g.
  `Leader round 117 now has quorum` while the walk was stuck earlier.

## Evidence

Log files: `target/quorum-logs/1793256534/{node0..node3}.log`.

- node0: `2026-10-29T06:49:42.631 INFO "Leader round 42 timed out (no commit decision)"`
- node0: `2026-10-29T06:49:53.630 WARN ... "leader slot still undecided after T_skip; signed a timeout" ... round: 42`
- node0: `2026-10-29T06:49:53.710 INFO ... "block received over gossip" ... round: 42, digest: f3a685e0…`
- node0: `2026-10-29T06:51:12.631 WARN ... "Not publishing a message that has already been published" ...`
      → `2026-10-29T06:51:12.631 WARN ... "failed to broadcast timeout" ... error: "Gossipsub error: Duplicate"`

The leader for the stuck round proposed right after its slot crossed
`T_skip`; the quorum had already signed timeouts, but the vote quorum and the
skip-certificate quorum could not both be satisfied, so the commit walk had no
way forward and later rounds sailed through.

## Why it stalls (as implemented today)

- `ConsensusConfig::skip_timeout_rounds` (engine.rs) defaults to
  `2 * WAVE_LENGTH` = 2 × 3 = **6 rounds**; `ConsensusEngine::skip_timeout()`
  converts it to a duration via `round_duration_ms` (engine.rs:1403).
- `ConsensusEngine::check_skip_timeout()` (engine.rs:1425, PRO-40 (b)) fires at the
  first undecided slot that stayed unvoted for `T_skip` after its vote round, then
  `time_out_or_rebroadcast()` → `sign_timeout()`.
- A skip requires a quorum certificate built from verified timeouts:
  `verify_skip_cert()` (skip.rs:80) → `TimeoutPool` (skip.rs:120) →
  `Committer::try_skip_and_mark_durable()` (committer.rs:660). The union-quorum
  commit path is `Committer::try_commit_and_mark_durable_with_timeouts()`
  (committer.rs:835) — it receives the timeout stake from the pool.

## The fix that landed in the meantime

`ConsensusEngine::drain_commits()` (engine.rs:2189) — the commit walk now keeps
draining until it stops making progress instead of waiting for the next trigger,
so a round that is already at quorum cannot sit behind an earlier undecided hole.

Related landed fix: the same PR drained restored quorum at startup
(`restore()` + `drain_commits()`).

## Reproduction

```bash
bash scripts/ci-local.sh cluster
```

## Acceptance criteria for the follow-up work

- [ ] Reproduce the stall deterministically (single late leader + full committee).
- [ ] Decide: a late proposal past `T_skip` must either reach skip-cert quorum or
      fall under the union-quorum commit rule with the recorded timeout stake — not
      wedge the walk.
- [ ] Cluster tests green via `bash scripts/ci-local.sh cluster`
      (`live_vote_integration`, `sustained_liveness_integration`, `--test-threads=1`).
