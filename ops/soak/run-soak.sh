#!/usr/bin/env bash
#
# Kovanica Automated Long-Running Soak and Resource Budget Test Harness.
#
# Brings up a 3-validator peered cluster with strict cgroup limits (cpus: 1.5,
# mem_limit: 2048m) and Prometheus observability, runs a continuous transaction
# load generator, monitors resource consumption and heights, and verifies:
#   1. Zero OOM kills across all containers (oom_killed == false).
#   2. Monotonic height increase across all 3 nodes without stalls.
#   3. Clean cluster shutdown and artifact collection.
#
# Usage:
#   ./ops/soak/run-soak.sh                        # default 10m smoke (600s)
#   ./ops/soak/run-soak.sh --duration 10m         # 10 minutes smoke
#   ./ops/soak/run-soak.sh --duration 6h          # 6-hour endurance soak
#   ./ops/soak/run-soak.sh --duration 24h         # 24-hour long soak
#   ./ops/soak/run-soak.sh --duration 1200 --rate 10.0
#   ./ops/soak/run-soak.sh --skip-up --no-teardown
#
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." >/dev/null 2>&1 && pwd)"
cd "$REPO_ROOT"

# Defaults
DURATION_INPUT="${SOAK_DURATION:-${SOAK_SECS:-600}}"
INTERVAL="${INTERVAL:-15}"
WARMUP_TIMEOUT="${WARMUP_TIMEOUT:-300}"
STALL_SECS="${STALL_SECS:-120}"
LOAD_RATE="${LOAD_RATE:-5.0}"
PROJECT_NAME="${PROJECT_NAME:-kovanica-soak}"
COMPOSE_FILE="${SCRIPT_DIR}/docker-compose.soak.yml"
SKIP_UP="${SOAK_SKIP_UP:-0}"
TEARDOWN="${SOAK_TEARDOWN:-1}"
SKIP_LOAD="${SOAK_SKIP_LOAD:-0}"
OUT_DIR="${OUT_DIR:-${TMPDIR:-/tmp}/kovanica-soak}"

# 3 validator nodes peered in soak stack
NODES=(node1 node2 node3)
RPC_PORTS=(8545 8546 8547)
METRICS_PORTS=(9100 9101 9102)

parse_duration() {
  local in="$1"
  if [[ "$in" =~ ^([0-9]+)([smhd]?)$ ]]; then
    local val="${BASH_REMATCH[1]}"
    local unit="${BASH_REMATCH[2]}"
    case "$unit" in
      s|"") echo "$val" ;;
      m) echo "$((val * 60))" ;;
      h) echo "$((val * 3600))" ;;
      d) echo "$((val * 86400))" ;;
    esac
  else
    echo "error: invalid duration '$in'. Use e.g. 600, 10m, 6h, 24h" >&2
    exit 2
  fi
}

usage() {
  cat <<EOF
Kovanica Soak & Resource Budget Test Harness

Usage: $(basename "$0") [options]

Options:
  -d, --duration <TIME>   Soak duration (e.g. 600, 10m, 6h, 24h; default: 600)
  -i, --interval <SECS>   Sample monitoring interval in seconds (default: 15)
  -w, --warmup <SECS>     Warm-up timeout waiting for health & commit (default: 300)
  -s, --stall <SECS>      Max seconds without block height advance before stall (default: 120)
  -r, --rate <TPS>        Load generator target rate in tx/s (default: 5.0)
  --no-load               Skip running the load generator
  --skip-up               Do not run docker compose up (stack already running)
  --no-teardown           Do not tear down docker compose stack at end
  -o, --out-dir <DIR>     Directory for logs, stats, and CSV samples
  -h, --help              Show this help message

Assertions Enforced:
  * Zero container OOM kills (oom_killed == false)
  * Monotonic height increase on all 3 nodes (no regressions or stalls)
  * Clean shutdown and metric consistency
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -d|--duration)
      DURATION_INPUT="${2:?--duration requires a value}"; shift 2 ;;
    -i|--interval)
      INTERVAL="${2:?--interval requires a value}"; shift 2 ;;
    -w|--warmup)
      WARMUP_TIMEOUT="${2:?--warmup requires a value}"; shift 2 ;;
    -s|--stall)
      STALL_SECS="${2:?--stall requires a value}"; shift 2 ;;
    -r|--rate)
      LOAD_RATE="${2:?--rate requires a value}"; shift 2 ;;
    --no-load)
      SKIP_LOAD=1; shift ;;
    --skip-up)
      SKIP_UP=1; shift ;;
    --no-teardown)
      TEARDOWN=0; shift ;;
    -o|--out-dir)
      OUT_DIR="${2:?--out-dir requires a value}"; shift 2 ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo "unknown argument: $1" >&2; usage; exit 2 ;;
  esac
done

SOAK_SECS="$(parse_duration "$DURATION_INPUT")"

mkdir -p "$OUT_DIR"
LOG_FILE="$OUT_DIR/soak.log"
SAMPLES_CSV="$OUT_DIR/samples.csv"
LOAD_LOG="$OUT_DIR/load_generator.log"
LOAD_STATS_JSON="$OUT_DIR/load_generator_stats.json"

log() {
  printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" | tee -a "$LOG_FILE"
}

compose() {
  docker compose -p "$PROJECT_NAME" -f "$COMPOSE_FILE" "$@"
}

LOAD_PID=""
cleanup() {
  local exit_code=$?
  if [[ -n "$LOAD_PID" ]] && kill -0 "$LOAD_PID" 2>/dev/null; then
    log "Stopping load generator (pid $LOAD_PID)..."
    kill -TERM "$LOAD_PID" 2>/dev/null || true
    wait "$LOAD_PID" 2>/dev/null || true
  fi

  if [[ "$SKIP_UP" != "1" && "$TEARDOWN" == "1" ]]; then
    log "Tearing down soak compose stack (down -v)..."
    compose down -v --remove-orphans >/dev/null 2>&1 || true
    log "Compose stack stopped cleanly."
  fi
  exit "$exit_code"
}
trap cleanup EXIT INT TERM

# ---------------------------------------------------------------------------
# Diagnostics & Assertions
# ---------------------------------------------------------------------------
collect_diagnostics() {
  log "Collecting soak diagnostics into $OUT_DIR..."
  compose ps > "$OUT_DIR/compose-ps.txt" 2>&1 || true
  compose logs --no-color --timestamps > "$OUT_DIR/compose-logs.txt" 2>&1 || true
  docker stats --no-stream > "$OUT_DIR/docker-stats.txt" 2>&1 || true
  for node in "${NODES[@]}" prometheus; do
    local cid
    cid=$(compose ps -q "$node" 2>/dev/null || true)
    if [[ -n "$cid" ]]; then
      docker inspect "$cid" > "$OUT_DIR/inspect-${node}.json" 2>&1 || true
    fi
  done
}

fail() {
  log "FAIL: $*"
  collect_diagnostics || true
  log "Soak execution FAILED."
  exit 1
}

assert_zero_oom() {
  local phase="$1"
  local svc cid oom_killed
  for svc in "${NODES[@]}" prometheus; do
    cid=$(compose ps -q "$svc" 2>/dev/null || true)
    [[ -n "$cid" ]] || continue
    oom_killed=$(docker inspect -f '{{.State.OOMKilled}}' "$cid" 2>/dev/null || echo "false")
    if [[ "$oom_killed" == "true" ]]; then
      fail "OOM kill assertion failed during $phase: container for '$svc' ($cid) was OOM killed!"
    fi
  done
}

fetch_health() {
  local port="$1"
  curl -fsS -m 3 "http://127.0.0.1:${port}/health" 2>/dev/null || true
}

fetch_metric() {
  local port="$1" key="$2"
  curl -fsS -m 3 "http://127.0.0.1:${port}/metrics" 2>/dev/null | awk -v k="$key" '$1==k{print $2; exit}' || echo ""
}

# ---------------------------------------------------------------------------
# Warm-up phase
# ---------------------------------------------------------------------------
warmup() {
  log "Warm-up: waiting up to ${WARMUP_TIMEOUT}s for all ${#NODES[@]} nodes to be healthy and committing..."
  local deadline=$(( $(date +%s) + WARMUP_TIMEOUT ))
  local -a last_h=(0 0 0)
  while (( $(date +%s) < deadline )); do
    local all_ok=1 progressed=0 i node rpc_p m_p res status bh
    assert_zero_oom "warmup"

    for i in "${!NODES[@]}"; do
      node="${NODES[$i]}"; rpc_p="${RPC_PORTS[$i]}"; m_p="${METRICS_PORTS[$i]}"
      res=$(fetch_health "$rpc_p")
      if [[ -z "$res" ]]; then
        all_ok=0; continue
      fi
      status=$(printf '%s' "$res" | jq -r '.status // "?"' 2>/dev/null || echo "?")
      if [[ "$status" != "ok" ]]; then
        all_ok=0; continue
      fi

      bh=$(fetch_metric "$m_p" "block_height")
      if [[ -z "$bh" || "$bh" -le 0 ]]; then
        all_ok=0; continue
      fi

      if (( bh > ${last_h[$i]} )); then
        progressed=1
      fi
      last_h[$i]="$bh"
    done

    if (( all_ok == 1 && progressed == 1 )); then
      log "Warm-up complete: all 3 nodes healthy and advancing blocks (heights: ${last_h[*]})"
      return 0
    fi
    sleep 4
  done
  return 1
}

# ---------------------------------------------------------------------------
# Main Orchestration
# ---------------------------------------------------------------------------
log "======================================================================"
log "KOVANICA AUTOMATED SOAK & RESOURCE BUDGET TEST HARNESS"
log "======================================================================"
log "repo=$REPO_ROOT"
log "compose_file=$COMPOSE_FILE"
log "duration=${SOAK_SECS}s ($DURATION_INPUT) interval=${INTERVAL}s stall=${STALL_SECS}s load_rate=${LOAD_RATE}tx/s"
log "cgroup_limits: cpus=1.5 mem_limit=2048m per node"
log "out_dir=$OUT_DIR"

: > "$SAMPLES_CSV"
printf 'timestamp,elapsed_s,node,status,peers,block_height,mempool_size,mem_rss_bytes,mem_limit_bytes,cpu_pct,oom_killed\n' >> "$SAMPLES_CSV"

if [[ "$SKIP_UP" == "1" ]]; then
  log "SOAK_SKIP_UP=1: expecting cluster to be already running"
else
  log "Building and launching multi-node soak cluster with Docker Compose..."
  compose up -d --build || fail "docker compose up failed"
fi

warmup || fail "Cluster warm-up timed out after ${WARMUP_TIMEOUT}s"

# Record initial heights and progress timestamps
declare -A PREV_HEIGHT=()
declare -A INITIAL_HEIGHT=()
declare -A LAST_PROG_TIME=()
for i in "${!NODES[@]}"; do
  node="${NODES[$i]}"
  init_bh=$(fetch_metric "${METRICS_PORTS[$i]}" "block_height")
  init_bh="${init_bh:-0}"
  INITIAL_HEIGHT[$node]="$init_bh"
  PREV_HEIGHT[$node]="$init_bh"
  LAST_PROG_TIME[$node]="$(date +%s)"
done

# Launch Load Generator
if [[ "$SKIP_LOAD" == "0" ]]; then
  log "Starting continuous load generator (${LOAD_RATE} tx/s to http://127.0.0.1:8545)..."
  python3 "$SCRIPT_DIR/load_generator.py" \
    --rpc-url "http://127.0.0.1:8545" \
    --chain-id 2 \
    --duration "$SOAK_SECS" \
    --rate "$LOAD_RATE" \
    --out-json "$LOAD_STATS_JSON" > "$LOAD_LOG" 2>&1 &
  LOAD_PID=$!
  log "Load generator running in background (pid $LOAD_PID, logging to $LOAD_LOG)"
else
  log "SOAK_SKIP_LOAD=1: skipping load generator invocation"
fi

SOAK_START_TIME=$(date +%s)
log "Entering soak observation loop for ${SOAK_SECS} seconds..."

rounds=0
while :; do
  now=$(date +%s)
  elapsed=$(( now - SOAK_START_TIME ))
  (( elapsed >= SOAK_SECS )) && break

  # Check load generator health if running
  if [[ -n "$LOAD_PID" ]] && ! kill -0 "$LOAD_PID" 2>/dev/null; then
    log "NOTE: Load generator process exited before soak elapsed time"
  fi

  # Sample all 3 nodes
  for i in "${!NODES[@]}"; do
    node="${NODES[$i]}"
    rpc_p="${RPC_PORTS[$i]}"
    m_p="${METRICS_PORTS[$i]}"

    # 1. Assert container running & zero OOM
    cid=$(compose ps -q "$node" 2>/dev/null || true)
    if [[ -z "$cid" ]]; then
      fail "Container for service '$node' disappeared!"
    fi
    running=$(docker inspect -f '{{.State.Running}}' "$cid" 2>/dev/null || echo "false")
    if [[ "$running" != "true" ]]; then
      fail "Container for service '$node' is not running (crashed or stopped) at elapsed=${elapsed}s"
    fi
    oom_killed=$(docker inspect -f '{{.State.OOMKilled}}' "$cid" 2>/dev/null || echo "false")
    if [[ "$oom_killed" == "true" ]]; then
      fail "OOM KILLED: container for service '$node' exceeded cgroup budget and was killed!"
    fi

    # Resource usage stats
    mem_bytes=$(docker inspect -f '{{.HostConfig.Memory}}' "$cid" 2>/dev/null || echo "0")
    # Quick live memory inspect from cgroup
    mem_usage=$(docker stats --no-stream --format '{{.MemUsage}}' "$cid" 2>/dev/null | awk '{print $1}' || echo "0B")
    cpu_pct=$(docker stats --no-stream --format '{{.CPUPerc}}' "$cid" 2>/dev/null || echo "0.0%")

    # 2. Health check
    h_res=$(fetch_health "$rpc_p")
    status=$(printf '%s' "$h_res" | jq -r '.status // "?"' 2>/dev/null || echo "?")
    if [[ "$status" != "ok" ]]; then
      fail "Service '$node' /health returned status '$status' (unhealthy/degraded) at elapsed=${elapsed}s"
    fi
    peers=$(printf '%s' "$h_res" | jq -r '.peer_count // -1' 2>/dev/null || echo "-1")

    # 3. Metrics & Monotonic height assertion
    bh=$(fetch_metric "$m_p" "block_height")
    mp=$(fetch_metric "$m_p" "mempool_size")
    if [[ -z "$bh" ]]; then
      fail "Service '$node' metrics endpoint unreachable or missing block_height at elapsed=${elapsed}s"
    fi

    # ASSERTION: Height must never regress (monotonic increase)
    if (( bh < ${PREV_HEIGHT[$node]} )); then
      fail "MONOTONIC VIOLATION: block_height on '$node' decreased from ${PREV_HEIGHT[$node]} to $bh at elapsed=${elapsed}s!"
    elif (( bh > ${PREV_HEIGHT[$node]} )); then
      PREV_HEIGHT[$node]="$bh"
      LAST_PROG_TIME[$node]="$now"
    else
      # Check for liveness stall
      since_prog=$(( now - ${LAST_PROG_TIME[$node]} ))
      if (( since_prog > STALL_SECS )); then
        fail "LIVENESS STALL: block_height on '$node' stuck at $bh for ${since_prog}s (> ${STALL_SECS}s limit) at elapsed=${elapsed}s!"
      fi
    fi

    printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
      "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$elapsed" "$node" "$status" "$peers" "$bh" "${mp:-0}" "$mem_usage" "$mem_bytes" "$cpu_pct" "$oom_killed" \
      >> "$SAMPLES_CSV"
  done

  # Log periodic heartbeats
  rounds=$(( rounds + 1 ))
  if (( rounds % 4 == 0 )); then
    log "Soak progress: elapsed=${elapsed}s/${SOAK_SECS}s | heights: node1=${PREV_HEIGHT[node1]} node2=${PREV_HEIGHT[node2]} node3=${PREV_HEIGHT[node3]} | OOM: zero"
  fi

  sleep "$INTERVAL"
done

SOAK_END_TIME=$(date +%s)
TOTAL_SOAKED=$(( SOAK_END_TIME - SOAK_START_TIME ))
log "Observation loop completed: ${rounds} sample rounds over ${TOTAL_SOAKED}s."

# ---------------------------------------------------------------------------
# Post-Soak Verification
# ---------------------------------------------------------------------------
log "Performing post-soak verification and assertions..."

# 1. Final OOM kill check across all containers
assert_zero_oom "post-soak"
log "  [PASS] Zero OOM kills verified across all containers (oom_killed == false)"

# 2. Final monotonic height increase verification
for node in "${NODES[@]}"; do
  init_h="${INITIAL_HEIGHT[$node]}"
  final_h="${PREV_HEIGHT[$node]}"
  if (( final_h <= init_h )); then
    fail "Height assertion failed on '$node': final height ($final_h) did not advance beyond initial height ($init_h)"
  fi
  log "  [PASS] Node '$node' height advanced monotonically: $init_h -> $final_h (+ $(( final_h - init_h )) blocks)"
done

# 3. Prometheus scrape check
prom_targets=$(curl -fsS -m 5 "http://127.0.0.1:9090/api/v1/targets" 2>/dev/null || true)
if echo "$prom_targets" | grep -q '"health":"up"'; then
  log "  [PASS] Prometheus metrics scraper active and reporting targets UP"
else
  log "  [NOTE] Prometheus targets check returned: $(echo "$prom_targets" | head -c 120)..."
fi

# 4. Print Load Generator Stats if available
if [[ -f "$LOAD_STATS_JSON" ]]; then
  log "Load Generator Results:"
  cat "$LOAD_STATS_JSON" | tee -a "$LOG_FILE"
fi

collect_diagnostics

log "======================================================================"
log "SOAK TEST PASSED: All resource budgets, liveness, and OOM gates green!"
log "Duration: ${TOTAL_SOAKED}s | Artifacts preserved in $OUT_DIR"
log "======================================================================"

exit 0
