#!/usr/bin/env bash
# KVNC TPS Benchmark — Phase 12.4
# Uses kvnc-bench binary for proper nonce management, funding, and committed counting.
# Target: >= 100 committed transactions/sec on 4-node devnet (2GB RAM target)

set -euo pipefail

# Configuration
DUR=${DUR:-60}                              # benchmark duration (seconds)
RPC_URL=${RPC_URL:-http://127.0.0.1:8545}   # node1 RPC endpoint
CHAIN_ID=${CHAIN_ID:-1337}                  # local chain ID
FUNDER_KEYS=${FUNDER_KEYS:-}                # space-separated funder key files
SENDERS=${SENDERS:-8}                       # number of derived bench senders
TXS_PER_SENDER=${TXS_PER_SENDER:-16}        # transactions per sender (clamped to 64)
CONCURRENCY=${CONCURRENCY:-64}              # max concurrent submissions
FUND_AMOUNT=${FUND_AMOUNT:-1000000}         # atoms per funding tx (1 KUNA = 1e9 atoms)
TX_AMOUNT=${TX_AMOUNT:-1}                   # transfer amount per tx (atoms)
TX_FEE=${TX_FEE:-1}                         # fee per tx (atoms)
WAIT_SECS=${WAIT_SECS:-120}                 # wait for commits (seconds)
BENCH_BIN=${BENCH_BIN:-./target/release/kvnc-bench}

# Derived funder keys from docker compose (if not provided)
if [ -z "$FUNDER_KEYS" ]; then
    FUNDER_KEYS=("/tmp/opencode/kvncbench/k1" "/tmp/opencode/kvncbench/k2" "/tmp/opencode/kvncbench/k3" "/tmp/opencode/kvncbench/k4")
else
    # Convert space-separated string to array
    read -ra FUNDER_KEYS <<< "$FUNDER_KEYS"
fi

set -x

echo "=== KVNC TPS Benchmark ==="
echo "duration=${DUR}s  rpc=${RPC_URL}  chain=${CHAIN_ID}"
echo "funders=($FUNDER_KEYS)"
echo "senders=${SENDERS}  txs_per_sender=${TXS_PER_SENDER}  concurrency=${CONCURRENCY}"
echo "fund_amount=${FUND_AMOUNT}  tx_amount=${TX_AMOUNT}  tx_fee=${TX_FEE}"
echo "wait=${WAIT_SECS}s"
echo "bench_binary=${BENCH_BIN}"

# Verify bench binary exists
if [ ! -x "$BENCH_BIN" ]; then
    echo "ERROR: Bench binary not found at $BENCH_BIN"
    echo "Build with: cargo build --release -p kvnc-cli --bin kvnc-bench"
    exit 1
fi

# Run benchmark
OUT_FILE="/tmp/tps-$(date +%s).log"
echo "=== KVNC TPS Benchmark $(date -Iseconds) ===" | tee "$OUT_FILE"
echo "config: dur=${DUR} rpc=${RPC_URL} chain=${CHAIN_ID} senders=${SENDERS} txs_per_sender=${TXS_PER_SENDER} concurrency=${CONCURRENCY} fund_amount=${FUND_AMOUNT} tx_amount=${TX_AMOUNT} tx_fee=${TX_FEE} wait=${WAIT_SECS}" >> "$OUT_FILE"

# Run the bench binary
FUNDER_ARGS=()
for key in "${FUNDER_KEYS[@]}"; do
    FUNDER_ARGS+=(--funder-key-file "$key")
done

"$BENCH_BIN" \
    --rpc-url "$RPC_URL" \
    --chain-id "$CHAIN_ID" \
    "${FUNDER_ARGS[@]}" \
    --senders "$SENDERS" \
    --txs-per-sender "$TXS_PER_SENDER" \
    --fund-amount "$FUND_AMOUNT" \
    --fund-wait-secs "$WAIT_SECS" \
    --concurrency "$CONCURRENCY" \
    --amount "$TX_AMOUNT" \
    --fee "$TX_FEE" \
    --wait-secs "$WAIT_SECS" \
    2>&1 | tee -a "$OUT_FILE"

BENCH_EXIT=${PIPESTATUS[0]}

# Extract TPS from output
BENCH_LINE=$(grep "^BENCH " "$OUT_FILE" | tail -1 || true)
if [ -n "$BENCH_LINE" ]; then
    TPS=$(echo "$BENCH_LINE" | sed 's/.* tps=\([0-9.]*\)$/\1/')
    COMMITTED=$(echo "$BENCH_LINE" | sed 's/.* committed=\([0-9]*\).*/\1/')
    echo ""
    echo "=== RESULT ==="
    echo "Committed transactions: $COMMITTED"
    echo "TPS: $TPS"
    echo "Target (>=100): $([ "$(echo "$TPS >= 100" | bc -l 2>/dev/null || echo 0)" = "1" ] && echo "PASS" || echo "FAIL")"
    echo "Log: $OUT_FILE"
else
    echo "ERROR: No BENCH output line found"
    BENCH_EXIT=1
fi

exit $BENCH_EXIT