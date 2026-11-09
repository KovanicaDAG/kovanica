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
if git -C "$SCRIPT_DIR" rev-parse --show-toplevel >/dev/null 2>&1; then
  REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
else
  REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." >/dev/null 2>&1 && pwd)"
fi

MODE="dry-run"

usage() {
  cat <<EOF
Kovanica VPS Systemd & Environment Cutover Tool

Usage: $(basename "$0") [options]

Modes:
  --dry-run       Validate paths, permissions, systemd prerequisites, and migration
                  state without altering system state (default mode)
  --execute       Apply changes: stop legacy units, create user/directories,
                  migrate environment configurations, install systemd units

Options:
  -h, --help      Show this help message and exit
EOF
}

if [[ $# -eq 0 ]]; then
  MODE="dry-run"
elif [[ "${1:-}" == "--execute" ]]; then
  MODE="execute"
elif [[ "${1:-}" == "--dry-run" || "${1:-}" == "-n" ]]; then
  MODE="dry-run"
elif [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
else
  echo "Error: Unknown argument '$1'" >&2
  usage >&2
  exit 1
fi

echo "======================================================================"
echo "KOVANICA VPS SYSTEMD & ENVIRONMENT CUTOVER TOOL"
echo "======================================================================"
echo "Mode:      $MODE"
echo "Repo root: $REPO_ROOT"
echo "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "User:      $(id -un) (EUID=$EUID)"
echo "======================================================================"

LEGACY_UNITS=(
  "kvnc-mainnet-seed1.service"
  "kvnc-node.service"
  "kvnc-testnet-seed1.service"
)

PREFLIGHT_ERRORS=0
PREFLIGHT_WARNINGS=0
PREFLIGHT_PASSED=0

log_pass() {
  echo "[PASS] $1"
  PREFLIGHT_PASSED=$((PREFLIGHT_PASSED + 1))
}

log_info() {
  echo "[INFO] $1"
}

log_warn() {
  echo "[WARN] $1"
  PREFLIGHT_WARNINGS=$((PREFLIGHT_WARNINGS + 1))
}

log_fail() {
  echo "[FAIL] $1"
  PREFLIGHT_ERRORS=$((PREFLIGHT_ERRORS + 1))
}

log_action() {
  local msg="$1"
  if [[ "$MODE" == "execute" ]]; then
    echo "[EXEC] $msg"
  else
    echo "[DRY-RUN] $msg"
  fi
}

# -----------------------------------------------------------------------------
# PRE-FLIGHT VALIDATION (Always runs in dry-run; runs sanity checks in execute)
# -----------------------------------------------------------------------------
echo "--- 1. Privilege and Environment Check ---"
if [[ $EUID -ne 0 ]]; then
  if [[ "$MODE" == "execute" ]]; then
    echo "[FAIL] Root privileges required for cutover execution. Re-run with sudo." >&2
    exit 1
  else
    log_warn "Not running as root (EUID=$EUID). Pre-flight checks will run in read-only mode, but --execute requires sudo."
  fi
else
  log_pass "Running with root privileges (EUID=0)."
fi

echo "--- 2. Systemd and Service Manager Check ---"
if command -v systemctl >/dev/null 2>&1; then
  log_pass "systemctl binary located at $(command -v systemctl)."
  if [[ -d /run/systemd/system ]]; then
    log_pass "systemd is active and booted as init system."
  else
    log_warn "systemd init directory /run/systemd/system not detected (containerized or chroot environment)."
  fi
else
  log_fail "systemctl binary not found in PATH. systemd is required."
fi

if [[ -d /etc/systemd/system ]]; then
  log_pass "Target systemd directory /etc/systemd/system exists."
else
  log_fail "Target systemd directory /etc/systemd/system does not exist."
fi

echo "--- 3. Repository and Source Unit Files Check ---"
SRC_NODE_UNIT="$REPO_ROOT/ops/systemd/kovanica-node.service"
SRC_SEED_UNIT="$REPO_ROOT/ops/systemd/kovanica-seed@.service"
SRC_ENV_TPL="$REPO_ROOT/ops/systemd/kovanica.env.example"

if [[ -f "$SRC_NODE_UNIT" ]]; then
  log_pass "Found node service unit: $SRC_NODE_UNIT"
else
  log_fail "Missing node service unit: $SRC_NODE_UNIT"
fi

if [[ -f "$SRC_SEED_UNIT" ]]; then
  log_pass "Found seed template unit: $SRC_SEED_UNIT"
else
  log_fail "Missing seed template unit: $SRC_SEED_UNIT"
fi

if [[ -f "$SRC_ENV_TPL" ]]; then
  log_pass "Found environment template: $SRC_ENV_TPL"
else
  log_fail "Missing environment template: $SRC_ENV_TPL"
fi

if command -v systemd-analyze >/dev/null 2>&1 && [[ -f "$SRC_NODE_UNIT" ]] && [[ -f "$SRC_SEED_UNIT" ]]; then
  if systemd-analyze verify "$SRC_NODE_UNIT" "$SRC_SEED_UNIT" >/dev/null 2>&1; then
    log_pass "systemd-analyze verify passed for production unit files."
  else
    log_warn "systemd-analyze verify reported warnings on production unit files."
  fi
fi

echo "--- 4. Node Daemon Binary Check ---"
TARGET_BIN="/usr/local/bin/kovanica-node"
if [[ -x "$TARGET_BIN" ]]; then
  log_pass "Node daemon executable exists at $TARGET_BIN."
else
  if [[ -f "$TARGET_BIN" ]]; then
    log_warn "File exists at $TARGET_BIN but is not executable."
  else
    # Check if compiled binary exists in repo target directory
    if [[ -x "$REPO_ROOT/target/release/kovanica-node" ]]; then
      log_warn "Binary not installed at $TARGET_BIN, but found built binary at $REPO_ROOT/target/release/kovanica-node."
    elif [[ -x "$REPO_ROOT/target/debug/kovanica-node" ]]; then
      log_warn "Binary not installed at $TARGET_BIN, but found debug binary at $REPO_ROOT/target/debug/kovanica-node."
    else
      log_warn "Node daemon binary not found at $TARGET_BIN. Build with 'cargo build --release' and install before starting service."
    fi
  fi
fi

echo "--- 5. User and Directory Prerequisites ---"
if id -u kovanica >/dev/null 2>&1; then
  kovanica_uid=$(id -u kovanica)
  kovanica_gid=$(id -g kovanica)
  log_pass "System user 'kovanica' exists (UID=$kovanica_uid, GID=$kovanica_gid)."
else
  log_info "User 'kovanica' does not exist yet (will be created in --execute mode)."
fi

DATA_DIR="/var/lib/kovanica"
if [[ -d "$DATA_DIR" ]]; then
  data_perms=$(stat -c "%a %U:%G" "$DATA_DIR" 2>/dev/null || stat -f "%p %u:%g" "$DATA_DIR" 2>/dev/null || echo "unknown")
  log_pass "Data directory $DATA_DIR exists (permissions: $data_perms)."
else
  log_info "Data directory $DATA_DIR does not exist yet (will be created in --execute mode)."
fi

CONF_DIR="/etc/kovanica"
if [[ -d "$CONF_DIR" ]]; then
  conf_perms=$(stat -c "%a %U:%G" "$CONF_DIR" 2>/dev/null || stat -f "%p %u:%g" "$CONF_DIR" 2>/dev/null || echo "unknown")
  log_pass "Config directory $CONF_DIR exists (permissions: $conf_perms)."
else
  log_info "Config directory $CONF_DIR does not exist yet (will be created in --execute mode)."
fi

CONF_FILE="/etc/kovanica/kovanica.env"
if [[ -f "$CONF_FILE" ]]; then
  log_info "Existing config file detected at $CONF_FILE (will be preserved)."
else
  log_info "Config file $CONF_FILE not present (will be initialized from template)."
fi

echo "--- 6. Legacy Services & Config Detection ---"
ACTIVE_LEGACY_UNITS=0
for unit in "${LEGACY_UNITS[@]}"; do
  unit_path="/etc/systemd/system/$unit"
  if [[ -f "$unit_path" ]]; then
    is_active="inactive"
    if command -v systemctl >/dev/null 2>&1; then
      if systemctl is-active --quiet "$unit" 2>/dev/null; then
        is_active="active (running)"
        ACTIVE_LEGACY_UNITS=$((ACTIVE_LEGACY_UNITS + 1))
      fi
    fi
    log_info "Detected legacy unit: $unit at $unit_path (state: $is_active)."
  else
    log_pass "Legacy unit $unit is not present."
  fi
done

if [[ -d "/etc/kvnc" ]]; then
  shopt -s nullglob
  legacy_configs=(/etc/kvnc/*.env /etc/kvnc/*.conf)
  shopt -u nullglob
  if [[ ${#legacy_configs[@]} -gt 0 ]]; then
    log_info "Legacy configuration files detected in /etc/kvnc: ${legacy_configs[*]}"
  else
    log_info "Legacy directory /etc/kvnc exists but contains no .env or .conf files."
  fi
else
  log_pass "No legacy /etc/kvnc directory present."
fi

# -----------------------------------------------------------------------------
# DRY-RUN SUMMARY & EXIT
# -----------------------------------------------------------------------------
if [[ "$MODE" == "dry-run" ]]; then
  echo "======================================================================"
  echo "DRY-RUN PRE-FLIGHT VALIDATION SUMMARY"
  echo "======================================================================"
  echo "Passed checks:    $PREFLIGHT_PASSED"
  echo "Warnings:         $PREFLIGHT_WARNINGS"
  echo "Fatal errors:     $PREFLIGHT_ERRORS"
  echo "Legacy units:     $ACTIVE_LEGACY_UNITS active"
  echo "----------------------------------------------------------------------"
  if [[ $PREFLIGHT_ERRORS -gt 0 ]]; then
    echo "STATUS: PRE-FLIGHT FAILED. Correct errors above before executing cutover."
    exit 1
  else
    echo "STATUS: PRE-FLIGHT PASSED. Safe to proceed with --execute."
    echo "To apply changes, run: sudo bash $0 --execute"
    exit 0
  fi
fi

# -----------------------------------------------------------------------------
# EXECUTE MODE (Alters system state)
# -----------------------------------------------------------------------------
echo "======================================================================"
echo "APPLYING CUTOVER CHANGES"
echo "======================================================================"

# 1. Stop and disable legacy kvnc-* units
for unit in "${LEGACY_UNITS[@]}"; do
  unit_path="/etc/systemd/system/$unit"
  if [[ -f "$unit_path" ]]; then
    log_action "Disabling and stopping legacy unit: $unit"
    systemctl stop "$unit" 2>/dev/null || true
    systemctl disable "$unit" 2>/dev/null || true
    rm -f "$unit_path"
  fi
done

# 2. Ensure system user & directory structure exist
log_action "Ensuring user 'kovanica' and data directory '/var/lib/kovanica' exist"
if ! id -u kovanica >/dev/null 2>&1; then
  useradd --system --home-dir /var/lib/kovanica --shell /usr/sbin/nologin kovanica || true
fi
install -d -o kovanica -g kovanica -m 0750 /var/lib/kovanica
install -d -o root -g root -m 0755 /etc/kovanica

# 3. Environment configuration migration (/etc/kvnc -> /etc/kovanica)
if [[ -d "/etc/kvnc" ]]; then
  log_action "Migrating /etc/kvnc files to /etc/kovanica with KOVANICA_* prefixes"
  shopt -s nullglob
  for envfile in /etc/kvnc/*.env /etc/kvnc/*.conf; do
    target="/etc/kovanica/$(basename "$envfile")"
    if [[ ! -f "$target" ]]; then
      sed -e 's/KVNC_/KOVANICA_/g' -e 's/\/root\/Projects\/kvnc/\/root\/Projects\/kovanica/g' "$envfile" > "$target"
      chmod 0600 "$target"
    fi
  done
  shopt -u nullglob
fi

# 4. Install standard kovanica systemd units
log_action "Installing production unit files from ops/systemd/"
cp "$SRC_NODE_UNIT" /etc/systemd/system/
cp "$SRC_SEED_UNIT" /etc/systemd/system/
if [[ ! -f "/etc/kovanica/kovanica.env" ]]; then
  cp "$SRC_ENV_TPL" /etc/kovanica/kovanica.env
  chmod 0644 /etc/kovanica/kovanica.env
fi
systemctl daemon-reload

echo "----------------------------------------------------------------------"
echo "Cutover complete. Next steps:"
echo "  1. Review /etc/kovanica/kovanica.env"
echo "  2. Start service: sudo systemctl enable --now kovanica-node"
echo "  3. Check status:  sudo systemctl status kovanica-node"
echo "======================================================================"
