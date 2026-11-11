//! Exact-value assertions on the process-global consensus gauges.
//!
//! The metrics registry — and every gauge in it — is a process-global
//! singleton: a `record_*` write is visible to every thread in the binary. That
//! is the right shape for a node, and it is why these two checks live in a test
//! binary of their own.
//!
//! Next to the unit tests in `kovanica-consensus` they are racy by
//! construction. `engine` tests call the same recorders while they run, and
//! `alert_backed_metrics_are_exported` asserts an exact value that a concurrent
//! engine test overwrites: `record_commit_latency` stores `ms as i64`, so a
//! sub-millisecond commit writes `commit_latency 0` and replaces the sentinel
//! between the write and the scrape. That turned CI red on a test that had
//! nothing to do with the change under test. No lock a unit test can take would
//! also be taken by a production caller, so the only isolation available is a
//! process of its own — nothing here starts an engine.
//!
//! The two checks write disjoint series (`rss_proxy`, `commit_latency`,
//! `peer_count`, `mempool_size` versus `block_height`, `current_round`), so they
//! do not have to serialize against each other.

use kovanica_consensus::metrics::{
    metrics_text, record_block_height, record_commit_latency, record_current_round,
    record_mempool_size, record_peer_count, record_rss_proxy,
};

/// The alert rules read `rss_proxy` and `commit_latency`; a recorder that is
/// never called leaves the series at 0 and the alert can never fire.
#[test]
fn alert_backed_metrics_are_exported() {
    record_rss_proxy(1234);
    record_commit_latency(321.0);
    record_peer_count(4);
    record_mempool_size(7);

    let text = metrics_text();
    for expected in [
        "rss_proxy 1234",
        "commit_latency 321",
        "peer_count 4",
        "mempool_size 7",
    ] {
        assert!(text.contains(expected), "missing {expected} in:\n{text}");
    }
}

/// `block_height` and `current_round` must be separate series. They used to be
/// the same gauge, fed the local round by the ticker and the committed leader
/// round by the commit path.
#[test]
fn block_height_and_current_round_are_independent() {
    record_current_round(41);
    record_block_height(13);

    let text = metrics_text();
    assert!(
        text.contains("block_height 13"),
        "block_height should be the committed height, got:\n{text}"
    );
    assert!(
        text.contains("current_round 41"),
        "current_round should be the local round, got:\n{text}"
    );
}
