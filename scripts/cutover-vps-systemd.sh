#!/usr/bin/env bash
#
# scripts/cutover-vps-systemd.sh
#
# Automates the live fleet cutover from legacy `kvnc-*` systemd service units
# and `KVNC_*` environment variables to `kovanica-*` and `KOVANICA_*`.
#
# Usage:
#   sudo bash scripts/cutover-vps-systemd.sh [--dry-run | --execute]
#
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." >/dev/null 2>&1 && pwd)"

MODE="dry-run"
if [[ "${1:-}" == "--execute" ]]; then
  MODE="execute"
elif [[ "${1:-}" == "--dry-run" || -z "${1:-}" ]]; then
  MODE="dry-run"
else
  echo "Usage: $0 [--dry-run | --execute]" >&2
  exit 1
fi

echo "======================================================================"
echo "KOVANICA VPS SYSTEMD & ENVIRONMENT CUTOVER TOOL"
echo "======================================================================"
echo "Mode: $MODE"
echo "Repo: $REPO_ROOT"
echo "======================================================================"

LEGACY_UNITS=(
  "kvnc-mainnet-seed1.service"
  "kvnc-node.service"
  "kvnc-testnet-seed1.service"
)

log_action() {
  local msg="$1"
  if [[ "$MODE" == "execute" ]]; then
    echo "[EXEC] $msg"
  else
    echo "[DRY-RUN] $msg"
  fi
}

# 1. Stop and disable legacy kvnc-* units
for unit in "${LEGACY_UNITS[@]}"; do
  unit_path="/etc/systemd/system/$unit"
  if [[ -f "$unit_path" ]]; then
    log_action "Disabling and stopping legacy unit: $unit"
    if [[ "$MODE" == "execute" ]]; then
      systemctl stop "$unit" 2>/dev/null || true
      systemctl disable "$unit" 2>/dev/null || true
      rm -f "$unit_path"
    fi
  else
    echo "[INFO] Legacy unit $unit not present in /etc/systemd/system/."
  fi
done

# 2. Ensure system user & directory structure exist
log_action "Ensuring user 'kovanica' and data directory '/var/lib/kovanica' exist"
if [[ "$MODE" == "execute" ]]; then
  if ! id -u kovanica >/dev/null 2>&1; then
    useradd --system --home-dir /var/lib/kovanica --shell /usr/sbin/nologin kovanica || true
  fi
  install -d -o kovanica -g kovanica -m 0750 /var/lib/kovanica
  install -d -o root -g root -m 0755 /etc/kovanica
fi

# 3. Environment configuration migration (/etc/kvnc -> /etc/kovanica)
if [[ -d "/etc/kvnc" ]]; then
  log_action "Migrating /etc/kvnc files to /etc/kovanica with KOVANICA_* prefixes"
  if [[ "$MODE" == "execute" ]]; then
    for envfile in /etc/kvnc/*.env /etc/kvnc/*.conf; do
      if [[ -f "$envfile" ]]; then
        target="/etc/kovanica/$(basename "$envfile")"
        if [[ ! -f "$target" ]]; then
          sed -e 's/KVNC_/KOVANICA_/g' -e 's/\/root\/Projects\/kvnc/\/root\/Projects\/kovanica/g' "$envfile" > "$target"
          chmod 0600 "$target"
        fi
      fi
    done
  fi
fi

# 4. Install standard kovanica systemd units
log_action "Installing production unit files from ops/systemd/"
if [[ "$MODE" == "execute" ]]; then
  cp "$REPO_ROOT/ops/systemd/kovanica-node.service" /etc/systemd/system/
  cp "$REPO_ROOT/ops/systemd/kovanica-seed@.service" /etc/systemd/system/
  if [[ ! -f "/etc/kovanica/kovanica.env" ]]; then
    cp "$REPO_ROOT/ops/systemd/kovanica.env.example" /etc/kovanica/kovanica.env
    chmod 0644 /etc/kovanica/kovanica.env
  fi
  systemctl daemon-reload
fi

echo "----------------------------------------------------------------------"
if [[ "$MODE" == "execute" ]]; then
  echo "Cutover complete. Next steps:"
  echo "  1. Review /etc/kovanica/kovanica.env"
  echo "  2. Start service: sudo systemctl enable --now kovanica-node"
  echo "  3. Check status:  sudo systemctl status kovanica-node"
else
  echo "Dry-run complete. Re-run with --execute to apply changes."
fi
echo "======================================================================"
