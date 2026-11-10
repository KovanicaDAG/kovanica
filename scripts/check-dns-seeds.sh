#!/usr/bin/env bash
#
# scripts/check-dns-seeds.sh
#
# Automated DNS Seed Audit & Reachability Probe for Kovanica Networks.
# Verifies DNS-only (grey-cloud) resolution, absence of Cloudflare HTTP proxying,
# and TCP port 8000 P2P reachability according to docs/SEED-DNS-AUDIT.md.
#
# Usage:
#   ./scripts/check-dns-seeds.sh [options]
#
# Options:
#   -n, --network <NET>     Target network: 'testnet' (default) or 'mainnet'
#   -s, --seeds <HOSTS>     Comma-separated list of hostnames to override defaults
#   -p, --port <PORT>       P2P port to probe (default: 8000)
#   -t, --timeout <SECS>    TCP connect timeout in seconds (default: 5)
#   --json                  Output results in JSON format
#   -v, --verbose           Verbose output
#   -h, --help              Show this help message
#
set -euo pipefail

NETWORK="testnet"
PORT="8000"
TIMEOUT="5"
JSON_OUTPUT=0
VERBOSE=0
CUSTOM_SEEDS=""

TESTNET_DEFAULT_SEEDS=(
  "testnet.kovanica.online"
  "seed1.kovanica.online"
  "seed3.kovanica.online"
)

MAINNET_DEFAULT_SEEDS=(
  "mainnet.kovanica.online"
  "seed1.kovanica.online"
  "seed3.kovanica.online"
)

usage() {
  cat <<EOF
Kovanica DNS Seed Audit Tool

Usage: $(basename "$0") [options]

Options:
  -n, --network <NET>     Network preset: 'testnet' (default) or 'mainnet'
  -s, --seeds <HOSTS>     Comma-separated hostnames (e.g. seed1.example.com,seed2.example.com)
  -p, --port <PORT>       TCP P2P port to check (default: 8000)
  -t, --timeout <SECS>    Connection timeout in seconds (default: 5)
  --json                  Output results as structured JSON
  -v, --verbose           Print verbose diagnostic logs
  -h, --help              Show this help text
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -n|--network)
      NETWORK="${2:?--network requires value}"; shift 2 ;;
    -s|--seeds)
      CUSTOM_SEEDS="${2:?--seeds requires value}"; shift 2 ;;
    -p|--port)
      PORT="${2:?--port requires value}"; shift 2 ;;
    -t|--timeout)
      TIMEOUT="${2:?--timeout requires value}"; shift 2 ;;
    --json)
      JSON_OUTPUT=1; shift ;;
    -v|--verbose)
      VERBOSE=1; shift ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo "Unknown option: $1" >&2; usage; exit 2 ;;
  esac
done

# Known Cloudflare IPv4 CIDR blocks (P2P cannot route through Cloudflare proxy)
CF_CIDRS=(
  "173.245.48.0/20"
  "103.21.244.0/22"
  "103.22.200.0/22"
  "103.31.4.0/22"
  "141.101.64.0/18"
  "108.162.192.0/18"
  "190.93.240.0/20"
  "188.114.96.0/20"
  "197.234.240.0/22"
  "198.41.128.0/17"
  "162.158.0.0/15"
  "104.16.0.0/13"
  "104.24.0.0/14"
  "172.64.0.0/13"
  "131.0.72.0/22"
)

# Convert IP to integer
ip_to_int() {
  local a b c d
  IFS=. read -r a b c d <<< "$1"
  echo "$(( (a << 24) + (b << 16) + (c << 8) + d ))"
}

is_cloudflare_ip() {
  local ip="$1"
  local ip_int
  if ! ip_int=$(ip_to_int "$ip" 2>/dev/null); then
    return 1
  fi

  for cidr in "${CF_CIDRS[@]}"; do
    local net_addr="${cidr%%/*}"
    local prefix="${cidr##*/}"
    local net_int
    net_int=$(ip_to_int "$net_addr")
    local mask=$(( 0xFFFFFFFF << (32 - prefix) & 0xFFFFFFFF ))
    if (( (ip_int & mask) == (net_int & mask) )); then
      return 0
    fi
  done
  return 1
}

# Determine seed list
SEEDS=()
if [[ -n "$CUSTOM_SEEDS" ]]; then
  IFS=',' read -ra SEEDS <<< "$CUSTOM_SEEDS"
elif [[ "$NETWORK" == "mainnet" ]]; then
  SEEDS=("${MAINNET_DEFAULT_SEEDS[@]}")
else
  SEEDS=("${TESTNET_DEFAULT_SEEDS[@]}")
fi

TOTAL_SEEDS=${#SEEDS[@]}
DNS_FAIL_COUNT=0
CF_PROXY_COUNT=0
TCP_FAIL_COUNT=0
HEALTHY_COUNT=0

RESULTS_JSON=()

if [[ "$JSON_OUTPUT" -eq 0 ]]; then
  echo "======================================================================"
  echo "KOVANICA SEED DNS AUDIT & CONNECTIVITY PROBE"
  echo "======================================================================"
  echo "Network:    $NETWORK"
  echo "P2P Port:   $PORT"
  echo "Timeout:    ${TIMEOUT}s"
  echo "Seeds:      ${SEEDS[*]}"
  echo "======================================================================"
fi

for seed in "${SEEDS[@]}"; do
  seed="$(echo "$seed" | xargs)"
  [[ -z "$seed" ]] && continue

  resolved_ips=()
  dns_status="ok"
  cf_proxied="no"
  tcp_status="failed"
  error_msg=""

  # 1. DNS Resolution
  if command -v dig >/dev/null 2>&1; then
    while read -r line; do
      [[ -n "$line" ]] && resolved_ips+=("$line")
    done < <(dig +short +time="$TIMEOUT" A "$seed" 2>/dev/null | grep -E '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$' || true)
  elif command -v getent >/dev/null 2>&1; then
    while read -r line; do
      [[ -n "$line" ]] && resolved_ips+=("$line")
    done < <(getent ahostsv4 "$seed" 2>/dev/null | awk '{print $1}' | sort -u || true)
  elif command -v host >/dev/null 2>&1; then
    while read -r line; do
      [[ -n "$line" ]] && resolved_ips+=("$line")
    done < <(host -t A "$seed" 2>/dev/null | awk '/has address/ {print $NF}' || true)
  fi

  if [[ ${#resolved_ips[@]} -eq 0 ]]; then
    dns_status="unresolved"
    error_msg="DNS query returned no A records"
    DNS_FAIL_COUNT=$((DNS_FAIL_COUNT + 1))
  else
    # 2. Check for Cloudflare Orange-Cloud Proxy
    for ip in "${resolved_ips[@]}"; do
      if is_cloudflare_ip "$ip"; then
        cf_proxied="yes"
        CF_PROXY_COUNT=$((CF_PROXY_COUNT + 1))
        error_msg="Resolves to Cloudflare proxy IP ($ip); must be grey-cloud DNS-only"
        break
      fi
    done

    # 3. Probe TCP P2P port
    primary_ip="${resolved_ips[0]}"
    if command -v nc >/dev/null 2>&1; then
      if nc -z -w "$TIMEOUT" "$primary_ip" "$PORT" >/dev/null 2>&1; then
        tcp_status="open"
      else
        tcp_status="unreachable"
      fi
    elif timeout "$TIMEOUT" bash -c "</dev/tcp/$primary_ip/$PORT" >/dev/null 2>&1; then
      tcp_status="open"
    else
      tcp_status="unreachable"
    fi

    if [[ "$tcp_status" != "open" && -z "$error_msg" ]]; then
      TCP_FAIL_COUNT=$((TCP_FAIL_COUNT + 1))
      error_msg="TCP port $PORT unreachable on $primary_ip"
    fi
  fi

  is_healthy="false"
  if [[ "$dns_status" == "ok" && "$cf_proxied" == "no" && "$tcp_status" == "open" ]]; then
    is_healthy="true"
    HEALTHY_COUNT=$((HEALTHY_COUNT + 1))
  fi

  # Record JSON entry
  ips_str=$(printf '"%s",' "${resolved_ips[@]}")
  ips_str="[${ips_str%,}]"
  RESULTS_JSON+=("{\"hostname\":\"$seed\",\"dns\":\"$dns_status\",\"ips\":$ips_str,\"cf_proxied\":\"$cf_proxied\",\"tcp_port_$PORT\":\"$tcp_status\",\"healthy\":$is_healthy,\"error\":\"$error_msg\"}")

  if [[ "$JSON_OUTPUT" -eq 0 ]]; then
    status_icon="[FAIL]"
    if [[ "$is_healthy" == "true" ]]; then
      status_icon="[PASS]"
    fi
    printf "%-7s %-30s -> IPs: %-25s | CF-Proxy: %-3s | TCP %-4s: %-11s" \
      "$status_icon" "$seed" "${resolved_ips[*]:-NONE}" "$cf_proxied" "$PORT" "$tcp_status"
    if [[ -n "$error_msg" ]]; then
      printf " (%s)" "$error_msg"
    fi
    echo ""
  fi
done

if [[ "$JSON_OUTPUT" -eq 1 ]]; then
  json_body=$(IFS=,; echo "${RESULTS_JSON[*]}")
  cat <<EOF
{
  "network": "$NETWORK",
  "port": $PORT,
  "total": $TOTAL_SEEDS,
  "healthy": $HEALTHY_COUNT,
  "dns_failures": $DNS_FAIL_COUNT,
  "cf_proxy_violations": $CF_PROXY_COUNT,
  "tcp_failures": $TCP_FAIL_COUNT,
  "results": [$json_body]
}
EOF
else
  echo "----------------------------------------------------------------------"
  echo "Summary: $HEALTHY_COUNT/$TOTAL_SEEDS seeds healthy (DNS failures: $DNS_FAIL_COUNT, CF proxy violations: $CF_PROXY_COUNT, TCP unreachable: $TCP_FAIL_COUNT)"
fi

if [[ "$HEALTHY_COUNT" -eq 0 && "$TOTAL_SEEDS" -gt 0 ]]; then
  exit 1
fi

exit 0
