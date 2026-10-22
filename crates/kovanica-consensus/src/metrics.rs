//! Prometheus metrics for MysticGhost consensus.

use once_cell::sync::Lazy;
use prometheus_client::encoding::text::{encode, encode_eof};
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::{Registry, Unit};
use std::sync::Mutex;

/// Global metrics registry.
static REGISTRY: Lazy<Mutex<Registry>> = Lazy::new(|| Mutex::new(Registry::default()));

/// Get the global metrics registry.
pub fn registry() -> &'static Mutex<Registry> {
    &REGISTRY
}

/// Labels for MysticGhost metrics.
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct MysticGhostLabels {
    pub result: &'static str, // "success", "fallback", "error"
}

/// MysticGhost mergeset size (number of blocks in mergeset before colouring).
static MYSTICGHOST_MERGESET_SIZE: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register(
        "mysticghost_mergeset_size",
        "Number of blocks in the mergeset before GHOSTDAG colouring",
        gauge.clone(),
    );
    gauge
});

/// MysticGhost colouring duration in milliseconds.
static MYSTICGHOST_COLOURING_DURATION_MS: Lazy<Family<MysticGhostLabels, Histogram>> =
    Lazy::new(|| {
        let family = Family::<MysticGhostLabels, Histogram>::new_with_constructor(|| {
            let buckets: Vec<f64> =
                prometheus_client::metrics::histogram::exponential_buckets(1.0, 2.0, 20).collect();
            Histogram::new(buckets)
        });
        let mut registry = REGISTRY.lock().unwrap();
        registry.register(
            "mysticghost_colouring_duration_ms",
            "Duration of GHOSTDAG colouring in milliseconds",
            family.clone(),
        );
        family
    });

/// Total number of non-blue blocks pruned.
static MYSTICGHOST_PRUNED_BLOCKS_TOTAL: Lazy<Counter> = Lazy::new(|| {
    let counter = Counter::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register(
        "mysticghost_pruned_blocks",
        "Total number of DAG blocks removed by wave pruning after commit",
        counter.clone(),
    );
    counter
});

/// Total number of waves pruned.
static MYSTICGHOST_PRUNED_WAVES_TOTAL: Lazy<Counter> = Lazy::new(|| {
    let counter = Counter::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register(
        "mysticghost_pruned_waves",
        "Total number of waves pruned (all blocks in old waves)",
        counter.clone(),
    );
    counter
});

/// Record mergeset size.
pub fn record_mergeset_size(size: usize) {
    MYSTICGHOST_MERGESET_SIZE.set(size as i64);
}

/// Record colouring duration.
pub fn record_colouring_duration_ms(duration_ms: f64, result: &'static str) {
    let labels = MysticGhostLabels { result };
    MYSTICGHOST_COLOURING_DURATION_MS
        .get_or_create(&labels)
        .observe(duration_ms);
}

/// Record pruned blocks count.
pub fn record_pruned_blocks(count: u64) {
    MYSTICGHOST_PRUNED_BLOCKS_TOTAL.inc_by(count);
}

/// Record pruned waves count.
pub fn record_pruned_waves(count: u64) {
    MYSTICGHOST_PRUNED_WAVES_TOTAL.inc_by(count);
}

/// Current block height.
static BLOCK_HEIGHT: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register("block_height", "Current chain block height", gauge.clone());
    gauge
});

/// Number of connected peers.
static PEER_COUNT: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register("peer_count", "Number of connected P2P peers", gauge.clone());
    gauge
});

/// Size of the mempool (pending transactions).
static MEMPOOL_SIZE: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register(
        "mempool_size",
        "Number of pending transactions in mempool",
        gauge.clone(),
    );
    gauge
});

/// Commit latency in milliseconds.
static COMMIT_LATENCY: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register(
        "commit_latency",
        "Commit latency in milliseconds (proxy)",
        gauge.clone(),
    );
    gauge
});

/// Mergeset size.
static MERGESET_SIZE: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register(
        "mergeset_size",
        "Number of blocks in the mergeset (proxy)",
        gauge.clone(),
    );
    gauge
});

/// RSS memory proxy (MB).
static RSS_PROXY: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register(
        "rss_proxy",
        "Approximate RSS memory footprint in MB (proxy)",
        gauge.clone(),
    );
    gauge
});

/// Current consensus round.
///
/// Separate from `block_height` on purpose: rounds advance every round interval
/// (4 s) while a leader commits every wave (12 s), so writing the round into
/// `block_height` made that gauge jump backwards on every commit.
static CURRENT_ROUND: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register(
        "current_round",
        "Current consensus round (advances every round interval)",
        gauge.clone(),
    );
    gauge
});

/// Record block height.
///
/// This is the *committed leader* height. Do not feed it the local round number
/// — use [`record_current_round`] for that.
pub fn record_block_height(height: i64) {
    BLOCK_HEIGHT.set(height);
}

/// Record the current consensus round.
pub fn record_current_round(round: i64) {
    CURRENT_ROUND.set(round);
}

/// Record peer count.
pub fn record_peer_count(count: i64) {
    PEER_COUNT.set(count);
}

/// Record mempool size.
pub fn record_mempool_size(size: i64) {
    MEMPOOL_SIZE.set(size);
}

/// Record commit latency (ms).
pub fn record_commit_latency(ms: f64) {
    COMMIT_LATENCY.set(ms as i64);
}

/// Record mergeset size.
pub fn record_mergeset_size_metric(size: usize) {
    MERGESET_SIZE.set(size as i64);
}

/// Record RSS proxy (MB).
pub fn record_rss_proxy(mb: i64) {
    RSS_PROXY.set(mb);
}

/// Encode the metrics registry to the Prometheus text format.
pub fn metrics_text() -> String {
    let mut buffer = String::new();
    let reg = REGISTRY.lock().unwrap();
    let _ = encode(&mut buffer, &reg);
    buffer
}

/// Encode registry content (without EOF) for streaming responses.
pub fn metrics_text_stream(buffer: &mut String) {
    let reg = REGISTRY.lock().unwrap();
    let _ = encode(buffer, &reg);
}

// ---------------------------------------------------------------------------
// JSON-RPC server metrics
// ---------------------------------------------------------------------------

/// Labels for RPC-level metrics.
///
/// `method` is always a bounded, compile-time method name taken from the RPC
/// server's own registry (`"other"` for anything unrecognised). Nothing
/// caller-supplied is ever used verbatim, so the number of series an RPC
/// client can create is fixed at the number of registered methods — a request
/// for an arbitrary method name cannot grow the cardinality.
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct RpcMethodLabels {
    pub method: &'static str,
}

/// Total JSON-RPC requests handled, by method.
static RPC_REQUESTS_TOTAL: Lazy<Family<RpcMethodLabels, Counter>> = Lazy::new(|| {
    let family = Family::<RpcMethodLabels, Counter>::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register(
        "rpc_requests",
        "Total number of JSON-RPC requests handled, by method",
        family.clone(),
    );
    family
});

/// Total JSON-RPC requests answered with a JSON-RPC error, by method.
static RPC_ERRORS_TOTAL: Lazy<Family<RpcMethodLabels, Counter>> = Lazy::new(|| {
    let family = Family::<RpcMethodLabels, Counter>::default();
    let mut registry = REGISTRY.lock().unwrap();
    registry.register(
        "rpc_errors",
        "Total number of JSON-RPC requests answered with an error, by method",
        family.clone(),
    );
    family
});

/// JSON-RPC request latency in seconds, by method.
static RPC_REQUEST_DURATION_SECONDS: Lazy<Family<RpcMethodLabels, Histogram>> = Lazy::new(|| {
    let family = Family::<RpcMethodLabels, Histogram>::new_with_constructor(|| {
        // ~1 ms .. ~16 s, doubling: covers the whole call range up to the
        // 10 s request timeout without a bucket per millisecond.
        let buckets: Vec<f64> =
            prometheus_client::metrics::histogram::exponential_buckets(0.001, 2.0, 15).collect();
        Histogram::new(buckets)
    });
    let mut registry = REGISTRY.lock().unwrap();
    registry.register_with_unit(
        "rpc_request_duration",
        "JSON-RPC request latency in seconds, by method",
        Unit::Seconds,
        family.clone(),
    );
    family
});

/// Record one handled JSON-RPC request: increments the per-method request
/// counter and observes its latency in seconds.
///
/// `method` must be a bounded label (see [`RpcMethodLabels`]); callers pass the
/// handler's registered name, or `"other"` for an unknown method.
pub fn observe_rpc_request(method: &'static str, duration_secs: f64) {
    let labels = RpcMethodLabels { method };
    RPC_REQUESTS_TOTAL.get_or_create(&labels).inc();
    RPC_REQUEST_DURATION_SECONDS
        .get_or_create(&labels)
        .observe(duration_secs);
}

/// Record one JSON-RPC request that was answered with an error (bad version,
/// unauthorized, unknown method, handler error or timeout).
pub fn record_rpc_error(method: &'static str) {
    RPC_ERRORS_TOTAL
        .get_or_create(&RpcMethodLabels { method })
        .inc();
}

/// Read the per-method request counter (diagnostics and tests).
pub fn rpc_request_count(method: &'static str) -> u64 {
    RPC_REQUESTS_TOTAL
        .get_or_create(&RpcMethodLabels { method })
        .get()
}

/// Read the per-method error counter (diagnostics and tests).
pub fn rpc_error_count(method: &'static str) -> u64 {
    RPC_ERRORS_TOTAL
        .get_or_create(&RpcMethodLabels { method })
        .get()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The registry is process-global, but tests run on parallel threads: the
    /// tests that assert on a concrete gauge value must not overlap, or one
    /// test overwrites another's value between the write and the scrape.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn serial() -> std::sync::MutexGuard<'static, ()> {
        // A panicking test poisons the lock; the value assertions are still
        // meaningful afterwards.
        SERIAL.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn test_metrics_registration() {
        let _guard = serial();
        // Just verify metrics can be registered and used
        record_mergeset_size(100);
        record_colouring_duration_ms(10.5, "success");
        record_colouring_duration_ms(5.2, "fallback");
        record_pruned_blocks(50);
        record_pruned_waves(2);
    }

    /// `block_height` and `current_round` must be separate series. They used to
    /// be the same gauge, fed the local round by the ticker and the committed
    /// leader round by the commit path.
    #[test]
    fn block_height_and_current_round_are_independent() {
        let _guard = serial();
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

    /// Pull the `expr:` values out of the alert rules (including `expr: |`
    /// block scalars).
    fn alert_expressions(yaml: &str) -> Vec<String> {
        let lines: Vec<&str> = yaml.lines().collect();
        let mut out = Vec::new();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("expr:") {
                let rest = rest.trim();
                if rest.is_empty() || rest == "|" || rest == ">" {
                    let base_indent = line.len() - trimmed.len();
                    i += 1;
                    let mut block = String::new();
                    while i < lines.len() {
                        let cur = lines[i];
                        let cur_trimmed = cur.trim();
                        if cur_trimmed.is_empty() {
                            i += 1;
                            continue;
                        }
                        let indent = cur.len() - cur.trim_start().len();
                        if indent <= base_indent {
                            break;
                        }
                        block.push_str(cur_trimmed);
                        block.push(' ');
                        i += 1;
                    }
                    out.push(block);
                    continue;
                }
                out.push(rest.to_string());
            }
            i += 1;
        }
        out
    }

    /// Identifiers in a PromQL expression, ignoring quoted label values and
    /// unit suffixes (`5m` must not yield `m`).
    fn promql_identifiers(expr: &str) -> Vec<String> {
        // Drop quoted strings (label values), keeping a separator.
        let mut cleaned = String::new();
        let mut in_quotes = false;
        for ch in expr.chars() {
            if ch == '"' {
                in_quotes = !in_quotes;
                cleaned.push(' ');
                continue;
            }
            if !in_quotes {
                cleaned.push(ch);
            }
        }

        let mut out = Vec::new();
        let mut current = String::new();
        let mut prev_alnum = false;
        for ch in cleaned.chars() {
            let is_alnum = ch.is_ascii_alphanumeric() || ch == '_';
            let starts = (ch.is_ascii_alphabetic() || ch == '_') && !prev_alnum;
            if starts {
                current.clear();
                current.push(ch);
            } else if is_alnum && !current.is_empty() {
                current.push(ch);
            } else if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            prev_alnum = is_alnum;
        }
        if !current.is_empty() {
            out.push(current);
        }
        out
    }

    /// Every series the shipped Prometheus rules alert on must exist in this
    /// registry.
    ///
    /// This is the check that was missing: `rss_proxy` and `commit_latency` were
    /// registered but never updated, so `KovanicaMemoryHigh` and
    /// `KovanicaCommitLatencyHigh` could never fire, and nothing noticed because
    /// each side was valid on its own.
    #[test]
    fn alerts_reference_registered_metrics() {
        let _guard = serial();
        // Force every lazily-registered family into the registry.
        record_peer_count(0);
        record_mempool_size(0);
        record_commit_latency(0.0);
        record_rss_proxy(0);
        record_block_height(0);
        record_current_round(0);
        record_mergeset_size(0);
        record_mergeset_size_metric(0);
        record_colouring_duration_ms(0.0, "fallback");
        record_pruned_blocks(0);
        record_pruned_waves(0);
        observe_rpc_request("other", 0.0);
        record_rpc_error("other");

        let text = metrics_text();
        let mut names: Vec<String> = text
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
            .filter_map(|line| {
                let series = line.split_whitespace().next()?;
                // Drop the label set: we compare family names.
                Some(series.split('{').next().unwrap_or(series).to_string())
            })
            .collect();
        names.sort();
        names.dedup();
        assert!(
            names.iter().any(|n| n == "block_height"),
            "registry did not populate:\n{text}"
        );
        // prometheus_client appends `_total`/the unit itself; registering a name
        // that already carries it exported `…_total_total` and
        // `…_seconds_seconds`, which no query would match.
        for name in &names {
            assert!(
                !name.ends_with("_total_total") && !name.ends_with("_seconds_seconds"),
                "metric name has a doubled suffix: {name}"
            );
        }
        // The encoder appends `_total` to counters and `_bucket`/`_sum`/`_count`
        // to histograms, so those exact suffixes are accepted — an arbitrary
        // prefix is not, or a rename would slip through.
        let registered = |name: &str| {
            names.iter().any(|n| {
                n == name
                    || ["_total", "_bucket", "_sum", "_count"]
                        .iter()
                        .any(|suffix| n.strip_prefix(name) == Some(*suffix))
            })
        };

        for expected in [
            "rpc_requests_total",
            "rpc_errors_total",
            "rpc_request_duration_seconds_bucket",
            "mysticghost_pruned_blocks_total",
            "mysticghost_pruned_waves_total",
        ] {
            assert!(
                registered(expected),
                "expected exported series {expected}; registered: {names:?}"
            );
        }

        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../ops/prometheus/alerts.yml"
        );
        let yaml =
            std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
        let exprs = alert_expressions(&yaml);
        assert!(!exprs.is_empty(), "no expr: values parsed from {path}");

        // PromQL keywords/functions and Prometheus' own series, not ours.
        const NOT_OURS: &[&str] = &[
            "and",
            "avg",
            "by",
            "changes",
            "count",
            "count_values",
            "delta",
            "group_left",
            "group_right",
            "ignoring",
            "increase",
            "instance",
            "job",
            "max",
            "min",
            "on",
            "or",
            "rate",
            "result",
            "severity",
            "sum",
            "unless",
            "up",
            "without",
        ];

        let mut missing: Vec<String> = Vec::new();
        for expr in &exprs {
            for token in promql_identifiers(expr) {
                if NOT_OURS.contains(&token.as_str()) || registered(&token) {
                    continue;
                }
                missing.push(token);
            }
        }
        missing.sort();
        missing.dedup();
        assert!(
            missing.is_empty(),
            "alerts.yml alerts on series this registry never exports: {missing:?}\n\
             registered: {names:?}"
        );
    }

    /// The alert rules read `rss_proxy` and `commit_latency`; a recorder that is
    /// never called leaves the series at 0 and the alert can never fire.
    #[test]
    fn alert_backed_metrics_are_exported() {
        let _guard = serial();
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
}
