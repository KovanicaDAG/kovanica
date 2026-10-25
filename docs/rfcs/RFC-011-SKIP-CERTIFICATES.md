# RFC-011: Leader-slot timeouts and skip certificates (PRO-40 (b))

Status: implemented for the chain-4 reset (wire change). Approved by Toni on 2026-10-23 as PRO-40 PR 3/4.

## Problem

Since PRO-35, the commit walk stops at the first undecided leader slot. A leader whose block never reaches a vote quorum therefore halts the chain for good. That happens when the leader is down or restarting, or when its block arrives after every late-vote window.

## Rule

1. **Timeout.** After its vote round, a validator waits `T_skip` for the first undecided slot `R` (`skip_timeout_rounds`, default 6 rounds ≈ 24 s). If it has not voted for `R` by then, it signs a timeout:
   `KUNA/timeout/v1 ‖ chain_id ‖ genesis_hash ‖ epoch(R) ‖ voter ‖ R`
   That is 74 bytes, with integers in little-endian. The slot author is not signed, so nodes that disagree on the schedule still certify the same skip.
2. **Vote xor timeout.** A validator signs a vote **or** a timeout for a slot, never both.
   - The own timeout is written to the `timeouts` table **before** it is broadcast. If that write fails, it is not sent.
   - Votes are refused for timed-out slots, timeouts are refused for voted slots, and both are restored at start-up.
3. **Skip certificate.** A skip certificate is a set of timeouts for `R` from signers that hold at least the **vote quorum** of stake (`2f+1`; 4 of 5 equal validators). Using the same threshold is what makes it safe: a vote quorum and a skip quorum intersect in an honest validator, and that validator signed only one of the two.
4. **Decision.** The committer applies a certificate only to the first undecided slot.
   - It writes the slot to the decided-round index as a decision with **no leaders** (`mark_round_skipped`), then moves the cursor.
   - A slot that has its own vote quorum is committed, never skipped.
   - Replay commits nothing for a skipped round.
   - The skipped leader block, if any, stays in the DAG and is ordered into the next committed leader's sub-DAG (scoped GHOSTDAG, deterministic because every node skips the same set).
5. **Acceptance.**
   - Timeouts and certificates are accepted only for slots in `(cursor, current + max(lookahead, 3)]`, one per (signer, slot).
   - Each is verified against the committee, `chain_id` and the genesis config digest.
   - A vote and a timeout from the same signer for one slot is counted as equivocation (`kovanica_vote_timeout_equivocations_total`, log only; slashing later).
6. **Gossip and recovery.**
   - Gossip uses topic `timeouts` (`TimeoutMessage::{Timeout, SkipCert}`), peer-scored like votes. The network edge checks membership and signatures; consensus checks range and quorum.
   - A node that forms a certificate gossips it.
   - While the slot stays stalled, a node rebroadcasts its own timeout (or its own vote) every `T_skip`.
   - A node that receives a timeout for a slot it already skipped answers with the stored certificate, at most once per `T_skip`. This is how a restarted or lagging validator gets past a slot the others skipped.

## Known limits

- **Split (accepted for testnet; Toni, PRO-40 proposal).** If between `n − q + 1` and `q − 1` validators voted, the rest can only time out. Then neither quorum forms and the chain **stops safely**, without a fork. This happens, for example, with a 3/2 partition, or a byzantine leader that sends its block to exactly two honest nodes. Closing it needs a 2-phase lock or Mysticeti-style DAG votes before mainnet (separate RFC).
- **Committee per epoch.** Like votes, timeouts are verified against the engine's committee. Verification with `EPOCH_COMMITTEE[epoch(R)]` comes with the epoch-committee work.
- **Block sync of certificates.** Block sync does not yet serve certificates (RFC-009 style). A node more than the pruning window behind is P1-a (commit-seal / state-sync anchoring).
- **Timeout retention.** Timeouts are not yet pruned with the vote retention pass.

## Metrics

- `kovanica_skipped_leaders_total{reason="certified"}`
- `kovanica_timeouts_signed_total`
- `kovanica_vote_timeout_equivocations_total`
