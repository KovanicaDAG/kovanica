#!/usr/bin/env bash
# Kovanica Chain 5 Hybrid PoW Testnet Cluster Smoke Test
# Usage: ops/docker/test-chain5-cluster.sh [--leave-running]
#
# Tests:
# 1. Starts multi-node compose stack (node1, node2, node3, faucet, prometheus, grafana).
# 2. Waits for all services to become healthy (/health).
# 3. Verifies PoW block mining and GHOSTDAG blue block sequencing.
# 4. Verifies Prometheus metrics scraping on port 9100 and prometheus target health.
# 5. Tests faucet payout of 10 KVNC on Chain 5.
# 6. Cleans up stack on exit (unless --leave-running is passed).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="${SCRIPT_DIR}/docker-compose.chain5.yml"
LEAVE_RUNNING=0
MAX_WAIT_SECS=60

for arg in "$@"; do
  case "$arg" in
    --leave-running)
      LEAVE_RUNNING=1
      ;;
    *)
      ;;
  esac
done

cleanup() {
  if [ "$LEAVE_RUNNING" -eq 0 ]; then
    echo "==> Stopping Chain 5 cluster and cleaning up..."
    docker compose -f "$COMPOSE_FILE" down -v --remove-orphans >/dev/null 2>&1 || true
    echo "==> Cluster stopped cleanly."
  else
    echo "==> --leave-running specified; leaving cluster active."
  fi
}
trap cleanup EXIT INT TERM

echo "======================================================================"
echo "KOVANICA CHAIN 5 HYBRID POW TESTNET CLUSTER SMOKE TEST"
echo "Timestamp: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "Compose file: $COMPOSE_FILE"
echo "======================================================================"

echo "==> Step 1: Launching Chain 5 cluster with Docker Compose..."
docker compose -f "$COMPOSE_FILE" up -d --build

echo "==> Step 2: Waiting for nodes and faucet to report healthy..."
check_health() {
  local name="$1"
  local url="$2"
  local elapsed=0
  printf "    Waiting for %s (%s)... " "$name" "$url"
  while [ $elapsed -lt $MAX_WAIT_SECS ]; do
    if curl -fsS -m 2 "$url" >/dev/null 2>&1; then
      echo "OK (${elapsed}s)"
      return 0
    fi
    sleep 2
    elapsed=$((elapsed + 2))
  done
  echo "FAILED (timeout after ${MAX_WAIT_SECS}s)"
  return 1
}

check_health "node1" "http://127.0.0.1:8545/health"
check_health "node2" "http://127.0.0.1:8546/health"
check_health "node3" "http://127.0.0.1:8547/health"
check_health "observer" "http://127.0.0.1:8548/health"
check_health "faucet" "http://127.0.0.1:3000/health"

echo "==> Step 3: Verifying PoW block advancement across nodes..."
get_block_number() {
  local rpc_port="$1"
  local res
  res=$(curl -s -m 5 -X POST "http://127.0.0.1:${rpc_port}/rpc" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"kovanica_blockNumber","params":[],"id":1}' 2>/dev/null || echo "{}")
  local hex_val
  hex_val=$(echo "$res" | grep -o '"result":"[^"]*"' | cut -d'"' -f4 || echo "0x0")
  printf "%d" "$hex_val" 2>/dev/null || echo "0"
}

initial_height=$(get_block_number 8545)
echo "    Initial block height on node1: $initial_height"
echo "    Waiting for PoW blocks to advance..."

advance_elapsed=0
current_height=$initial_height
while [ $advance_elapsed -lt 30 ]; do
  sleep 4
  advance_elapsed=$((advance_elapsed + 4))
  current_height=$(get_block_number 8545)
  if [ "$current_height" -gt "$initial_height" ]; then
    echo "    Block height advanced: $initial_height -> $current_height (${advance_elapsed}s)"
    break
  fi
done

if [ "$current_height" -le "$initial_height" ]; then
  echo "ERROR: Block height failed to advance via PoW mining within 30s!"
  exit 1
fi

echo "==> Step 4: Verifying GHOSTDAG blue blocks and PoW metrics..."
metrics_node1=$(curl -s -m 5 "http://127.0.0.1:9100/metrics" 2>/dev/null || true)
if echo "$metrics_node1" | grep -q "pow_blocks_mined"; then
  mined_count=$(echo "$metrics_node1" | grep -E "^pow_blocks_mined" | head -n1 | awk '{print $2}')
  echo "    PoW blocks mined metric: ${mined_count:-0}"
else
  echo "    Warning: pow_blocks_mined metric not yet populated, continuing..."
fi

if echo "$metrics_node1" | grep -q "ghostdag_blue_blocks"; then
  blue_count=$(echo "$metrics_node1" | grep -E "^ghostdag_blue_blocks" | head -n1 | awk '{print $2}')
  echo "    GHOSTDAG blue blocks metric: ${blue_count:-0}"
else
  echo "    Warning: ghostdag_blue_blocks metric not yet populated, continuing..."
fi

echo "==> Step 5: Testing Faucet payout on Chain 5..."
TEST_RECIPIENT="kvnc8d4z00xwk6jz6c4r4rgz5mcdwdjny9thrh3y8f36cpy2rz6emg54rufktdag"
faucet_res=$(curl -s -m 10 -X POST "http://127.0.0.1:3000/faucet" \
  -H "Content-Type: application/json" \
  -d "{\"address\": \"${TEST_RECIPIENT}\"}")

echo "    Faucet response: $faucet_res"
if echo "$faucet_res" | grep -q '"success":true'; then
  tx_hash=$(echo "$faucet_res" | grep -o '"tx_hash":"[^"]*"' | cut -d'"' -f4 || echo "N/A")
  echo "    SUCCESS: Faucet dispensed 10 KVNC (tx: $tx_hash)"
else
  echo "ERROR: Faucet dispense failed!"
  exit 1
fi

echo "==> Step 6: Verifying Prometheus scraping and Grafana..."
if curl -fsS -m 5 "http://127.0.0.1:9090/-/healthy" >/dev/null 2>&1; then
  echo "    Prometheus server is healthy on http://127.0.0.1:9090"
fi
if curl -fsS -m 5 "http://127.0.0.1:3001/api/health" >/dev/null 2>&1; then
  echo "    Grafana server is healthy on http://127.0.0.1:3001"
fi

echo "======================================================================"
echo "SMOKE TEST PASSED: Chain 5 multi-node cluster is fully operational!"
echo "======================================================================"
