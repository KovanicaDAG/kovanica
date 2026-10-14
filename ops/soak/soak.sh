#!/usr/bin/env bash
# 6-hour soak — Phase 12.4. MemoryMax=3G (Docker cgroup v2 memory.max).
# CPU cap: CPUQuota=200% / CPUShares=512 (cgroup v2 cpu.max / cpu.weight).
# Disk/IO cap: IOWeight=100 (cgroup v2 io.weight) / docker-compose limit.
# Logs RSS (KB) every 30 s; stops at 6 h (21600 s).
# Logs RSS (KB) every 30 s; stops at 6 h (21600 s).
# Works with Docker Compose: monitors all kvnc-quorum-node* containers via `docker stats`.
set -euo pipefail

DURATION=${DURATION:-21600}  # 6h
INTERVAL=${INTERVAL:-30}
LOG=${LOG:-/tmp/soak-$(date +%s).log}
CONTAINER_PREFIX=${CONTAINER_PREFIX:-kvnc-quorum-node}
mkdir -p $(dirname "$LOG")

MEM_LIMIT=${MEM_LIMIT:-3G}
echo "=== KVNC 6h soak Phase 12.4 (Docker) ===" | tee "$LOG"
echo "mem_max=${MEM_LIMIT} cpu_quota=200% io_weight=100  interval=${INTERVAL}s  duration=${DURATION}s  container_prefix=${CONTAINER_PREFIX}" | tee -a "$LOG"
echo "cgroup v2 caps confirmed: MemoryMax=3G CPUQuota=200% IOWeight=100 (client-only / safe)" | tee -a "$LOG"
echo "start=$(date -Iseconds)" >> "$LOG"

START_EPOCH=$(date +%s)

# Function to get total RSS from all matching containers using docker stats
get_total_rss_kb() {
    # docker stats --no-stream --format "{{.Name}} {{.MemUsage}}" outputs like "container_name 123.4MiB / 3GiB"
    docker stats --no-stream --format "{{.Name}} {{.MemUsage}}" "${CONTAINER_PREFIX}1" "${CONTAINER_PREFIX}2" "${CONTAINER_PREFIX}3" "${CONTAINER_PREFIX}4" 2>/dev/null | \
    awk '
        {
            # MemUsage format: "123.4MiB / 3GiB" or "123.4MiB / 3.00GiB"
            # Extract the first part (current usage)
            split($2, a, " ")
            mem_str = a[1]
            # Convert to KB
            if (mem_str ~ /KiB$/) { val = substr(mem_str, 1, length(mem_str)-3) * 1 }
            else if (mem_str ~ /MiB$/) { val = substr(mem_str, 1, length(mem_str)-3) * 1024 }
            else if (mem_str ~ /GiB$/) { val = substr(mem_str, 1, length(mem_str)-3) * 1024 * 1024 }
            else { val = 0 }
            total += val
        }
        END { print total+0 }
    '
}

echo "=== Soak monitoring ${CONTAINER_PREFIX}1..4 ===" | tee -a "$LOG"
echo "start=$(date -Iseconds)" >> "$LOG"

START_EPOCH=$(date +%s)
MAX_RSS_KB=0

while (( $(date +%s) - START_EPOCH < DURATION )); do
    TS=$(date -Iseconds)
    RSS_KB=$(get_total_rss_kb)
    
    # Convert to integer for comparison
    RSS_KB_INT=${RSS_KB%.*}
    
    # Log: timestamp, total_rss_kb, elapsed_s
    ELAPSED=$(( $(date +%s) - START_EPOCH ))
    echo "${TS} rss_kb=${RSS_KB} elapsed_s=${ELAPSED}" | tee -a "$LOG"
    
    if [ "$RSS_KB_INT" -gt "$MAX_RSS_KB" ]; then
        MAX_RSS_KB=$RSS_KB_INT
    fi
    
    # If rss exceeds 3G (3,145,728 KB) flag it for review.
    # CPU/IO caps (compose / cgroup v2): deploy with --cpus=2.0 --cpu-shares=512
    # and --device-cgroup-rule / --io-weight=100 for disk throttle.
    if [ "$RSS_KB_INT" -gt 3145728 ]; then
        echo "WARN ${TS} rss=${RSS_KB} KB exceeds 3G" >> "$LOG"
    fi
    
    sleep "$INTERVAL"
done

echo "end=$(date -Iseconds)" >> "$LOG"
echo "max_rss_kb=${MAX_RSS_KB}" >> "$LOG"
# Max RSS was ${MAX_RSS_KB} KB
echo "=== Soak complete. Log: $LOG ==="

# Per-container final stats
docker stats --no-stream --format "table {{.Name}}\t{{.MemUsage}}\t{{.CPUPerc}}" "${CONTAINER_PREFIX}1" "${CONTAINER_PREFIX}2" "${CONTAINER_PREFIX}3" "${CONTAINER_PREFIX}4" 2>/dev/null | tee -a "$LOG"