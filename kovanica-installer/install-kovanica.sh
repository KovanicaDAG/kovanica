#!/usr/bin/env bash
# =============================================================================
# Kovanica Installer - One-command installation for miners and developers
# =============================================================================
# Installs: Node.js, PostgreSQL, nginx, PM2, Kovanica app, SSL, firewall
# Target: Ubuntu 22.04/24.04, Debian 12
# Usage: curl -fsSL https://raw.githubusercontent.com/kovanica/kovanica-installer/main/install-kovanica.sh | bash
#        or: ./install-kovanica.sh [options]
# =============================================================================

set -euo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# Configuration & Constants
# ─────────────────────────────────────────────────────────────────────────────

readonly SCRIPT_VERSION="1.0.0"
readonly SCRIPT_NAME="$(basename "$0")"
readonly INSTALLER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly LOG_FILE="/var/log/kovanica-installer.log"
readonly KOVANICA_USER="kovanica"
readonly KOVANICA_HOME="/opt/kovanica"
readonly KOVANICA_APP_DIR="${KOVANICA_HOME}/app"
readonly KOVANICA_DATA_DIR="${KOVANICA_HOME}/data"
readonly KOVANICA_BACKUP_DIR="${KOVANICA_HOME}/backups"
readonly NODE_VERSION="20"
readonly POSTGRES_VERSION="16"
readonly KOVANICA_REPO="https://github.com/kovanica/kovanica-app.git"
readonly KOVANICA_BRANCH="main"

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Default options
DRY_RUN=false
SKIP_NODE=false
SKIP_POSTGRES=false
SKIP_NGINX=false
SKIP_PM2=false
SKIP_APP=false
SKIP_SSL=false
SKIP_FIREWALL=false
FORCE=false
UNINSTALL=false
DOMAIN=""
EMAIL=""
APP_PORT=5000
FRONTEND_PORT=3000

# ─────────────────────────────────────────────────────────────────────────────
# Logging & Output Functions
# ─────────────────────────────────────────────────────────────────────────────

log() {
    local level="$1"
    shift
    local msg="$*"
    local timestamp
    timestamp="$(date '+%Y-%m-%d %H:%M:%S')"
    echo -e "${timestamp} [${level}] ${msg}" | tee -a "${LOG_FILE}"
}

info() { log "INFO" "${BLUE}$*${NC}"; }
warn() { log "WARN" "${YELLOW}$*${NC}"; }
error() { log "ERROR" "${RED}$*${NC}"; }
success() { log "SUCCESS" "${GREEN}$*${NC}"; }

die() {
    error "$*"
    exit 1
}

# ─────────────────────────────────────────────────────────────────────────────
# Help & Argument Parsing
# ─────────────────────────────────────────────────────────────────────────────

usage() {
    cat <<EOF
${SCRIPT_NAME} v${SCRIPT_VERSION} - Kovanica One-Command Installer

USAGE:
    ${SCRIPT_NAME} [OPTIONS]

OPTIONS:
    --domain DOMAIN         Domain name for SSL (e.g., kovanica.example.com)
    --email EMAIL           Email for Let's Encrypt notifications
    --app-port PORT         Backend API port (default: 5000)
    --frontend-port PORT    Frontend dev port (default: 3000, not exposed publicly)
    --skip-node             Skip Node.js installation
    --skip-postgres         Skip PostgreSQL installation
    --skip-nginx            Skip nginx installation/configuration
    --skip-pm2              Skip PM2 installation/configuration
    --skip-app              Skip Kovanica app clone/build
    --skip-ssl              Skip SSL certificate setup
    --skip-firewall         Skip firewall configuration
    --force                 Force reinstall (overwrite existing)
    --uninstall             Uninstall Kovanica completely
    --dry-run               Show what would be done without executing
    -h, --help              Show this help
    -v, --version           Show version

EXAMPLES:
    # Full interactive install
    sudo ${SCRIPT_NAME}

    # Automated with domain for SSL
    sudo ${SCRIPT_NAME} --domain kovanica.example.com --email admin@example.com

    # Minimal install (app only, assumes dependencies exist)
    sudo ${SCRIPT_NAME} --skip-node --skip-postgres --skip-nginx --skip-ssl

    # Dry run to preview
    ${SCRIPT_NAME} --dry-run

    # Uninstall
    sudo ${SCRIPT_NAME} --uninstall

REQUIREMENTS:
    - Ubuntu 22.04/24.04 or Debian 12
    - Root/sudo access
    - Internet connection
    - Port 80, 443 available for nginx/SSL

EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --domain) DOMAIN="$2"; shift 2 ;;
            --email) EMAIL="$2"; shift 2 ;;
            --app-port) APP_PORT="$2"; shift 2 ;;
            --frontend-port) FRONTEND_PORT="$2"; shift 2 ;;
            --skip-node) SKIP_NODE=true; shift ;;
            --skip-postgres) SKIP_POSTGRES=true; shift ;;
            --skip-nginx) SKIP_NGINX=true; shift ;;
            --skip-pm2) SKIP_PM2=true; shift ;;
            --skip-app) SKIP_APP=true; shift ;;
            --skip-ssl) SKIP_SSL=true; shift ;;
            --skip-firewall) SKIP_FIREWALL=true; shift ;;
            --force) FORCE=true; shift ;;
            --uninstall) UNINSTALL=true; shift ;;
            --dry-run) DRY_RUN=true; shift ;;
            -h|--help) usage; exit 0 ;;
            -v|--version) echo "${SCRIPT_VERSION}"; exit 0 ;;
            *) die "Unknown option: $1. Use --help for usage." ;;
        esac
    done
}

# ─────────────────────────────────────────────────────────────────────────────
# System Checks
# ─────────────────────────────────────────────────────────────────────────────

check_root() {
    if [[ $EUID -ne 0 ]]; then
        die "This script must be run as root (use sudo)"
    fi
}

check_os() {
    if [[ ! -f /etc/os-release ]]; then
        die "Cannot detect OS. /etc/os-release not found."
    fi
    source /etc/os-release
    case "${ID}" in
        ubuntu)
            if [[ "${VERSION_ID}" != "22.04" && "${VERSION_ID}" != "24.04" ]]; then
                warn "Ubuntu ${VERSION_ID} not officially supported. Tested on 22.04/24.04."
            fi
            ;;
        debian)
            if [[ "${VERSION_ID}" != "12" ]]; then
                warn "Debian ${VERSION_ID} not officially supported. Tested on 12."
            fi
            ;;
        *)
            die "Unsupported OS: ${PRETTY_NAME}. Supported: Ubuntu 22.04/24.04, Debian 12"
            ;;
    esac
    info "OS detected: ${PRETTY_NAME}"
}

check_internet() {
    if ! ping -c 1 -W 2 8.8.8.8 &>/dev/null; then
        die "No internet connection. Required for package downloads."
    fi
}

check_ports() {
    local ports=(80 443 "${APP_PORT}")
    for port in "${ports[@]}"; do
        if ss -tuln | grep -q ":${port} "; then
            warn "Port ${port} is already in use. This may cause conflicts."
        fi
    done
}

check_existing_install() {
    if [[ -d "${KOVANICA_APP_DIR}" ]] && [[ "${FORCE}" != "true" ]]; then
        warn "Existing installation found at ${KOVANICA_APP_DIR}"
        read -rp "Overwrite? [y/N] " -n 1
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            die "Installation aborted. Use --force to overwrite."
        fi
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# Module Loading
# ─────────────────────────────────────────────────────────────────────────────

load_modules() {
    source "${INSTALLER_DIR}/scripts/lib/common.sh"
    source "${INSTALLER_DIR}/scripts/modules/01-system-prep.sh"
    source "${INSTALLER_DIR}/scripts/modules/02-nodejs.sh"
    source "${INSTALLER_DIR}/scripts/modules/03-postgresql.sh"
    source "${INSTALLER_DIR}/scripts/modules/04-nginx.sh"
    source "${INSTALLER_DIR}/scripts/modules/05-pm2.sh"
    source "${INSTALLER_DIR}/scripts/modules/06-kovanica-app.sh"
    source "${INSTALLER_DIR}/scripts/modules/07-ssl.sh"
    source "${INSTALLER_DIR}/scripts/modules/08-firewall.sh"
    source "${INSTALLER_DIR}/scripts/modules/09-finalize.sh"
}

# ─────────────────────────────────────────────────────────────────────────────
# Main Installation Flow
# ─────────────────────────────────────────────────────────────────────────────

run_install() {
    info "Starting Kovanica installation v${SCRIPT_VERSION}"
    info "Log file: ${LOG_FILE}"

    # Pre-flight checks
    check_root
    check_os
    check_internet
    check_ports
    check_existing_install

    if [[ "${DRY_RUN}" == "true" ]]; then
        info "DRY RUN MODE - No changes will be made"
    fi

    # Load all modules
    load_modules

    # Execute installation steps
    system_prep

    [[ "${SKIP_NODE}" != "true" ]] && install_nodejs
    [[ "${SKIP_POSTGRES}" != "true" ]] && install_postgresql
    [[ "${SKIP_NGINX}" != "true" ]] && install_nginx
    [[ "${SKIP_PM2}" != "true" ]] && install_pm2
    [[ "${SKIP_APP}" != "true" ]] && install_kovanica_app
    [[ "${SKIP_SSL}" != "true" && -n "${DOMAIN}" ]] && setup_ssl
    [[ "${SKIP_FIREWALL}" != "true" ]] && configure_firewall

    finalize_installation

    success "Kovanica installation completed successfully!"
    print_summary
}

run_uninstall() {
    info "Starting Kovanica uninstallation..."
    check_root
    load_modules
    uninstall_kovanica
    success "Kovanica uninstalled completely."
}

print_summary() {
    cat <<EOF

${GREEN}═══════════════════════════════════════════════════════════════════════${NC}
${GREEN}  Kovanica Installation Complete!${NC}
${GREEN}═══════════════════════════════════════════════════════════════════════${NC}

${BLUE}Installation Details:${NC}
  • App directory:     ${KOVANICA_APP_DIR}
  • Data directory:    ${KOVANICA_DATA_DIR}
  • Backup directory:  ${KOVANICA_BACKUP_DIR}
  • Service user:      ${KOVANICA_USER}
  • Backend API:       http://localhost:${APP_PORT}
  • Frontend (dev):    http://localhost:${FRONTEND_PORT} (loopback only)

${BLUE}Management Commands:${NC}
  • View logs:         pm2 logs kovanica-backend
  • Restart app:       pm2 restart kovanica-backend
  • Stop app:          pm2 stop kovanica-backend
  • App status:        pm2 status
  • Nginx status:      systemctl status nginx
  • PostgreSQL:        systemctl status postgresql

${BLUE}Next Steps:${NC}
EOF

    if [[ -n "${DOMAIN}" ]]; then
        cat <<EOF
  1. Configure DNS: Point ${DOMAIN} to this server's IP
  2. SSL certificate will auto-renew via certbot timer
  3. Access your app at: https://${DOMAIN}
EOF
    else
        cat <<EOF
  1. Configure your domain DNS to point to this server
  2. Run SSL setup: sudo ${INSTALLER_DIR}/scripts/modules/07-ssl.sh --domain your.domain.com --email your@email.com
  3. Access via HTTP on port 80 (not recommended for production)
EOF
    fi

    cat <<EOF

${YELLOW}IMPORTANT:${NC}
  • Backup passphrase is in ${KOVANICA_APP_DIR}/.env.production (BACKUP_PASSPHRASE)
  • Store it securely - backups cannot be restored without it!
  • Database: kovanica_app (production), kovanica_app_dev (development)
  • Check logs: tail -f ${LOG_FILE}

${GREEN}═══════════════════════════════════════════════════════════════════════${NC}
EOF
}

# ─────────────────────────────────────────────────────────────────────────────
# Entry Point
# ─────────────────────────────────────────────────────────────────────────────

main() {
    parse_args "$@"

    # Create log file
    touch "${LOG_FILE}"
    chmod 644 "${LOG_FILE}"

    if [[ "${UNINSTALL}" == "true" ]]; then
        run_uninstall
    else
        run_install
    fi
}

main "$@"