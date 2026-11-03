# NOTED — PRO-40 late vote / commit-walk stall

Living note for pickup. Original pickup doc moved from `docs/PRO-40-LATE-VOTE-STALL.md` on 2026-10-31.
Open acceptance criteria at the bottom remain open work unless closed in code + TASKLIST.

---


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

## Resolution (PRO-40 follow-up)

- **Decision (union-quorum with non-overlapping timeout stake above the skip-complement floor)**:
  - `BaseCommitter::union_quorum_decides_commit` (`crates/kovanica-consensus/src/committer.rs`) now commits a leader block when `votes + timeout_stake >= quorum_threshold` and `votes > total_stake - quorum_threshold` (the skip-complement floor).
  - `TimeoutPool::stake_by_round_excluding` (`crates/kovanica-consensus/src/skip.rs`), called from `ConsensusEngine::try_commit_and_deliver` (`crates/kovanica-consensus/src/engine.rs`), excludes any authority that also voted for the slot so a Byzantine equivocator cannot contribute both vote stake and timeout stake to the union quorum.
  - Because honest validators enforce durable vote-xor-timeout (`vote_for_leader` / `sign_timeout`), every full-committee outcome past `T_skip` is partitioned with no dead zone:
    - When `votes <= total_stake - quorum_threshold` (e.g. a single late leader voting `1/4` or `1/5` while the remaining validators time out), the non-voting validators hold `total_stake - votes >= quorum_threshold` stake and form a verified `SkipCert`, while the union-quorum rule stays `Undecided` so `try_skip_and_mark_durable` skips the slot cleanly.
    - When `votes > total_stake - quorum_threshold` (e.g. the `2/4` votes + `2/4` timeouts split from `1793256534`, or `2/5` votes + `3/5` timeouts on a 5-validator committee), the non-voting validators hold strictly less than `quorum_threshold` stake (`total_stake - votes < quorum_threshold`) so no honest `SkipCert` can form, and `try_commit_and_mark_durable_with_timeouts` commits the slot under the union-quorum rule and drains subsequent quorum slots.

## Acceptance criteria for the follow-up work

- [x] Reproduce the stall deterministically (single late leader + full committee): `committer::tests::late_leader_split_past_t_skip_either_skips_or_union_commits` and `engine::tests::late_leader_past_t_skip_either_skips_or_union_commits_without_wedging`.
- [x] Decide: a late proposal past `T_skip` must either reach skip-cert quorum or
      fall under the union-quorum commit rule with the recorded timeout stake — not
      wedge the walk.
- [x] Cluster tests green via `bash scripts/ci-local.sh cluster`
      (`live_vote_integration`, `sustained_liveness_integration`, `--test-threads=1`).
