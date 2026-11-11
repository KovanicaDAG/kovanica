#!/bin/sh
# Kovanica node installer.
#
# Installs a prebuilt kovanica-node binary, the genesis bundle for one network
# and a ready-to-run configuration, then checks that the local network tag
# matches the published one before anything starts. It ships no network data of
# its own: the bundle and its tag come from the release that network publishes.
#
#   curl -fsSL https://kovanica.online/download/install.sh | sh
#
# The default network is the canonical hybrid public testnet (chain_id 2).
# Mainnet (1), devnet (3) and local (1337) are selectable with --chain-id, but
# only a network whose bundle is published for that release can pass the tag
# check.
#
# Options (also accepted as arguments to a saved copy):
#   --version <v>     node release version          (default 0.2.0)
#   --chain-id <id>   network to install for        (default 2, testnet)
#   --bundle-asset <name>  genesis asset in the release
#                          (default kovanica-chain<chain-id>-genesis.tar.gz)
#   --expected-tag <t>     published tag <chain-id>-<16 hex>
#                          (default: the `expected-tag` file inside the bundle)
#   --prefix <dir>    install prefix                (default /usr/local, root)
#   --home <dir>      config directory              (default /etc/kovanica, root)
#   --data-dir <dir>  chain data directory          (default /var/lib/kovanica, root)
#   --no-service      do not create/enable a systemd unit
#   --skip-tag-check  do not compare the network tag with the published one
#                     (the node still prints it; only the comparison is skipped)
#   --uninstall       remove binary, unit and config (chain data is kept)
#   --help
#
# Environment:
#   KOVANICA_VERSION, KOVANICA_CHAIN_ID, KOVANICA_BUNDLE_ASSET,
#   KOVANICA_EXPECTED_TAG, KOVANICA_PREFIX, KOVANICA_HOME, KOVANICA_DATA_DIR,
#   KOVANICA_RELEASE_BASE (override the download base, e.g. a mirror),
#   KOVANICA_BOOTNODES (comma-separated, overrides the built-in seed list).
#
# The installer never handles validator keys. A node installed this way is a
# non-validating full node: it syncs, verifies and serves RPC on loopback.
#
# This script is deliberately plain POSIX sh with no dependencies beyond
# curl-or-wget, tar and sha256sum.

set -eu

VERSION="${KOVANICA_VERSION:-0.2.0}"
REPO="KovanicaDAG/kovanica"
RELEASE_BASE="${KOVANICA_RELEASE_BASE:-https://github.com/${REPO}/releases/download/v${VERSION}-node}"

# Default network: the canonical hybrid public testnet. Nothing chain-specific
# is baked into this script beyond that default - the bundle and the tag it
# expects both come from the release.
CHAIN_ID="${KOVANICA_CHAIN_ID:-2}"
BUNDLE_ASSET="${KOVANICA_BUNDLE_ASSET:-}"
EXPECTED_TAG="${KOVANICA_EXPECTED_TAG:-}"

SERVICE_NAME="kovanica-node"
SERVICE_USER="kovanica"

AS_ROOT=0
if [ "$(id -u)" = "0" ]; then AS_ROOT=1; fi
if [ "$AS_ROOT" = "1" ]; then
    PREFIX="${KOVANICA_PREFIX:-/usr/local}"
    CONF_DIR="${KOVANICA_HOME:-/etc/kovanica}"
    DATA_DIR="${KOVANICA_DATA_DIR:-/var/lib/kovanica}"
else
    PREFIX="${KOVANICA_PREFIX:-${HOME}/.local}"
    CONF_DIR="${KOVANICA_HOME:-${HOME}/.kovanica}"
    DATA_DIR="${KOVANICA_DATA_DIR:-${HOME}/.kovanica/data}"
fi

WANT_SERVICE=1
TAG_CHECK=1
UNINSTALL=0
DRY_RUN=0

BOOTNODES="${KOVANICA_BOOTNODES:-seed1.kovanica.online:8000,seed3.kovanica.online:8000}"

die() {
    printf 'install: error: %s\n' "$*" >&2
    exit 1
}

say() {
    printf 'install: %s\n' "$*"
}

usage() {
    sed -n '3,42p' "$0" 2>/dev/null | sed 's/^# \{0,1\}//' || printf 'See the header of install.sh\n'
}

while [ $# -gt 0 ]; do
    case "$1" in
        --version)     [ $# -ge 2 ] || die "--version needs a value"; VERSION="$2"; shift 2 ;;
        --chain-id)    [ $# -ge 2 ] || die "--chain-id needs a value"; CHAIN_ID="$2"; shift 2 ;;
        --bundle-asset) [ $# -ge 2 ] || die "--bundle-asset needs a value"; BUNDLE_ASSET="$2"; shift 2 ;;
        --expected-tag) [ $# -ge 2 ] || die "--expected-tag needs a value"; EXPECTED_TAG="$2"; shift 2 ;;
        --prefix)      [ $# -ge 2 ] || die "--prefix needs a value"; PREFIX="$2"; shift 2 ;;
        --home)        [ $# -ge 2 ] || die "--home needs a value"; CONF_DIR="$2"; shift 2 ;;
        --data-dir)    [ $# -ge 2 ] || die "--data-dir needs a value"; DATA_DIR="$2"; shift 2 ;;
        --no-service)  WANT_SERVICE=0; shift ;;
        --skip-tag-check) TAG_CHECK=0; shift ;;
        --uninstall)   UNINSTALL=1; shift ;;
        --dry-run)     DRY_RUN=1; shift ;;
        -h|--help)     usage; exit 0 ;;
        *)             die "unknown option: $1 (try --help)" ;;
    esac
done

# --version / --release-base may come after the initial assignment, so the base
# is recomputed here rather than at the top of the file.
if [ "${KOVANICA_RELEASE_BASE:-}" ]; then
    RELEASE_BASE="$KOVANICA_RELEASE_BASE"
else
    RELEASE_BASE="https://github.com/${REPO}/releases/download/v${VERSION}-node"
fi

case "$CHAIN_ID" in
    ''|*[!0-9]*) die "--chain-id must be a number (got '${CHAIN_ID}'); registered ids are 1 mainnet, 2 testnet, 3 devnet, 1337 local" ;;
esac

case "$CHAIN_ID" in
    1)    NETWORK_LABEL="mainnet (chain 1)" ;;
    2)    NETWORK_LABEL="testnet (chain 2)" ;;
    3)    NETWORK_LABEL="devnet (chain 3)" ;;
    1337) NETWORK_LABEL="local devnet (chain 1337)" ;;
    *)    NETWORK_LABEL="chain ${CHAIN_ID}" ;;
esac

# Default asset name for the network's genesis bundle. A network that publishes
# a different name can say so with --bundle-asset.
if [ -z "$BUNDLE_ASSET" ]; then
    BUNDLE_ASSET="kovanica-chain${CHAIN_ID}-genesis.tar.gz"
fi

# A tag is `<chain-id>-<16 lowercase hex>` and is derived from the chain id and
# the genesis digest. Validate the shape early so a typo cannot silently skip
# the comparison later.
tag_shape_ok() {
    case "$1" in
        *-*) ;;
        *) return 1 ;;
    esac
    case "${1%%-*}" in
        ''|*[!0-9]*) return 1 ;;
    esac
    tag_hex="${1#*-}"
    [ "${#tag_hex}" -eq 16 ] || return 1
    case "$tag_hex" in
        *[!0-9a-f]*) return 1 ;;
    esac
    return 0
}

if [ -n "$EXPECTED_TAG" ]; then
    tag_shape_ok "$EXPECTED_TAG" \
        || die "--expected-tag '${EXPECTED_TAG}' is not of the form <chain-id>-<16 lowercase hex>"
    [ "${EXPECTED_TAG%%-*}" = "$CHAIN_ID" ] \
        || die "--expected-tag ${EXPECTED_TAG} belongs to chain ${EXPECTED_TAG%%-*}, not chain ${CHAIN_ID}"
fi

BIN_PATH="${PREFIX}/bin/kovanica-node"
CONF_PATH="${CONF_DIR}/kovanica.toml"
UNIT_PATH="/etc/systemd/system/${SERVICE_NAME}.service"

run() {
    if [ "$DRY_RUN" = "1" ]; then
        printf 'install: [dry-run] %s\n' "$*"
    else
        "$@"
    fi
}

fetch() {
    url="$1"
    out="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --retry 3 --connect-timeout 20 -o "$out" "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$out" "$url"
    else
        die "need curl or wget"
    fi
}

sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

# --------------------------------------------------------------------------
# Uninstall
# --------------------------------------------------------------------------
if [ "$UNINSTALL" = "1" ]; then
    say "removing ${SERVICE_NAME} (chain data in ${DATA_DIR} is kept)"
    if [ "$AS_ROOT" = "1" ] && command -v systemctl >/dev/null 2>&1; then
        run systemctl disable --now "${SERVICE_NAME}.service" || true
    fi
    run rm -f "$UNIT_PATH"
    if [ "$AS_ROOT" = "1" ] && command -v systemctl >/dev/null 2>&1; then
        run systemctl daemon-reload || true
    fi
    run rm -f "$BIN_PATH"
    run rm -rf "$CONF_DIR"
    say "done. delete ${DATA_DIR} yourself if you also want the chain data gone."
    exit 0
fi

# --------------------------------------------------------------------------
# Platform checks
# --------------------------------------------------------------------------
[ "$(uname -s)" = "Linux" ] || die "prebuilt binaries are Linux only; build from source on this OS: cargo build --release -p kovanica-node"

case "$(uname -m)" in
    x86_64|amd64)  ARCH=x86_64 ;;
    aarch64|arm64) ARCH=aarch64 ;;
    *) die "unsupported architecture $(uname -m); supported: x86_64, aarch64" ;;
esac

command -v tar >/dev/null 2>&1 || die "tar is required"

ARCHIVE="kovanica-node-${ARCH}-linux.tar.gz"
say "Kovanica node ${VERSION} for linux-${ARCH} (${NETWORK_LABEL})"

CONF_TMP=""
TMPDIR_INSTALL="$(mktemp -d)"
cleanup() {
    rm -rf "$TMPDIR_INSTALL"
    [ -z "$CONF_TMP" ] || rm -f "$CONF_TMP"
}
trap cleanup EXIT INT TERM

# --------------------------------------------------------------------------
# Checksums
# --------------------------------------------------------------------------
say "fetching ${RELEASE_BASE}/SHA256SUMS"
fetch "${RELEASE_BASE}/SHA256SUMS" "${TMPDIR_INSTALL}/SHA256SUMS" \
    || die "cannot download SHA256SUMS for v${VERSION}-node (is that release published?)"

expected_sum() {
    # Accept both `name` and the `./name` form that `sha256sum ./*` emits, and
    # the `*name` binary marker some tools use.
    awk -v want="$1" '{
        n = $2
        sub(/^\*/, "", n)
        sub(/^\.\//, "", n)
        if (n == want) { print $1; exit }
    }' "${TMPDIR_INSTALL}/SHA256SUMS"
}

verify() {
    file="$1"
    name="$2"
    want="$(expected_sum "$name")"
    [ -n "$want" ] || die "${name} is not listed in SHA256SUMS"
    got="$(sha256_of "$file")"
    [ "$want" = "$got" ] || die "checksum mismatch for ${name} (expected ${want}, got ${got})"
    say "verified ${name} (sha256 ${got})"
}

# --------------------------------------------------------------------------
# Binary
# --------------------------------------------------------------------------
say "downloading ${ARCHIVE}"
fetch "${RELEASE_BASE}/${ARCHIVE}" "${TMPDIR_INSTALL}/${ARCHIVE}" || die "cannot download ${ARCHIVE}"
verify "${TMPDIR_INSTALL}/${ARCHIVE}" "${ARCHIVE}"

mkdir -p "${TMPDIR_INSTALL}/extract"
tar -xzf "${TMPDIR_INSTALL}/${ARCHIVE}" -C "${TMPDIR_INSTALL}/extract"
[ -f "${TMPDIR_INSTALL}/extract/kovanica-node" ] || die "${ARCHIVE} does not contain kovanica-node"
run install -d "${PREFIX}/bin"
run install -m 0755 "${TMPDIR_INSTALL}/extract/kovanica-node" "$BIN_PATH"
say "installed ${BIN_PATH}"

# --------------------------------------------------------------------------
# Genesis bundle (byte-identical to the live mesh)
# --------------------------------------------------------------------------
say "downloading chain-${CHAIN_ID} genesis bundle"
fetch "${RELEASE_BASE}/${BUNDLE_ASSET}" "${TMPDIR_INSTALL}/${BUNDLE_ASSET}" || die "cannot download ${BUNDLE_ASSET}"
verify "${TMPDIR_INSTALL}/${BUNDLE_ASSET}" "${BUNDLE_ASSET}"

mkdir -p "${TMPDIR_INSTALL}/bundle"
tar -xzf "${TMPDIR_INSTALL}/${BUNDLE_ASSET}" -C "${TMPDIR_INSTALL}/bundle"
BUNDLE_DIR="${TMPDIR_INSTALL}/bundle"
[ -f "${BUNDLE_DIR}/genesis_validators.toml" ] || die "bundle has no genesis_validators.toml"

# Mainnet carries no funded accounts: a bundle that ships them is not mainnet.
if [ "$CHAIN_ID" = "1" ] && [ -f "${BUNDLE_DIR}/funded_accounts.toml" ]; then
    die "${BUNDLE_ASSET} ships funded_accounts.toml, which chain 1 (mainnet) forbids; refusing it"
fi

# The expected tag comes from the command line, the environment, or a file
# inside the bundle - in that order. A bundle that names its own tag is what
# lets a release add a network without touching this script.
if [ -z "$EXPECTED_TAG" ] && [ -f "${BUNDLE_DIR}/expected-tag" ]; then
    EXPECTED_TAG="$(tr -d '[:space:]' < "${BUNDLE_DIR}/expected-tag")"
    tag_shape_ok "$EXPECTED_TAG" \
        || die "the bundle's expected-tag '${EXPECTED_TAG}' is not of the form <chain-id>-<16 lowercase hex>"
    say "expected tag ${EXPECTED_TAG} (from the bundle)"
fi

if [ -n "$EXPECTED_TAG" ]; then
    [ "${EXPECTED_TAG%%-*}" = "$CHAIN_ID" ] \
        || die "expected tag ${EXPECTED_TAG} is for chain ${EXPECTED_TAG%%-*}, not chain ${CHAIN_ID}; pass the matching --chain-id and --bundle-asset"
fi

# Fail closed: an unpinned install cannot detect a forked or wrong-network
# bundle, so it must be an explicit choice.
if [ "$TAG_CHECK" = "1" ] && [ -z "$EXPECTED_TAG" ]; then
    die "no expected network tag for chain ${CHAIN_ID}: this release publishes no expected-tag in ${BUNDLE_ASSET}. Pass --expected-tag <chain-id>-<16 hex>, or --skip-tag-check to install without a pin (the node will still print its tag)"
fi

if [ -d "$DATA_DIR" ] && [ -f "${DATA_DIR}/consensus.redb" ]; then
    die "${DATA_DIR} already holds chain data; refusing to overwrite its genesis bundle (pass --data-dir for a fresh directory)"
fi

run install -d "$DATA_DIR"
for f in "${BUNDLE_DIR}"/*; do
    [ -f "$f" ] || continue
    case "$(basename "$f")" in
        SHA256SUMS|expected-tag) continue ;;
    esac
    run install -m 0644 "$f" "${DATA_DIR}/$(basename "$f")"
done
say "genesis bundle installed in ${DATA_DIR}"

# --------------------------------------------------------------------------
# Config
# --------------------------------------------------------------------------
# A new config is written to a temporary file and only moved into place after
# the config and the network tag have been checked, so a failed install never
# leaves a config for the wrong network behind.
CONF_CHECK=""
if [ -e "$CONF_PATH" ]; then
    say "keeping existing ${CONF_PATH}"
    CONF_CHECK="$CONF_PATH"
else
    boot_list=""
    old_ifs="$IFS"
    IFS=','
    for b in $BOOTNODES; do
        [ -n "$b" ] || continue
        if [ -n "$boot_list" ]; then boot_list="${boot_list}, "; fi
        boot_list="${boot_list}\"${b}\""
    done
    IFS="$old_ifs"

    run mkdir -p "$CONF_DIR"
    if [ "$DRY_RUN" = "1" ]; then
        say "[dry-run] would write ${CONF_PATH} (chain_id ${CHAIN_ID})"
    else
        CONF_TMP="${CONF_PATH}.tmp.$$"
        cat >"$CONF_TMP" <<EOF
# Kovanica node configuration - ${NETWORK_LABEL}.
# Written by install.sh for node ${VERSION}. Edit freely; re-running the
# installer keeps this file.
#
# Consensus parameters (target block time, GHOSTDAG K, PoW hash) are chain-wide
# and must not be overridden with KOVANICA_* variables on chain ${CHAIN_ID}.

chain_id = ${CHAIN_ID}
data_dir = "${DATA_DIR}"
listen_addr = "0.0.0.0:8000"
rpc_addr = "127.0.0.1"
rpc_port = 8545

# Non-validating full node: no signing key, no votes, RPC on loopback only.
run_validator = false

# Seed nodes. Plaintext TCP 8000; do not point these at a proxied hostname.
bootnodes = [${boot_list}]

max_peers = 50
rpc_rate_limit_per_min = 60
EOF
        chmod 0644 "$CONF_TMP"
        CONF_CHECK="$CONF_TMP"
    fi
fi

# --------------------------------------------------------------------------
# Validate config and network tag before starting anything
# --------------------------------------------------------------------------
if [ "$DRY_RUN" = "0" ] && [ -n "$CONF_CHECK" ]; then
    "$BIN_PATH" --check-config --config "$CONF_CHECK" >/dev/null \
        || die "config for chain ${CHAIN_ID} failed validation (unregistered chain id, bad address, missing data_dir?)"
    say "config is valid"

    tag_line="$("$BIN_PATH" network-tag --config "$CONF_CHECK" 2>&1 || true)"
    local_tag="$(printf '%s\n' "$tag_line" | awk '{ for (i = 1; i <= NF; i++) if ($i ~ /^[0-9]+-[0-9a-f]{16}$/) { print $i; exit } }')"
    [ -n "$local_tag" ] || die "could not read a network tag from the node (output: ${tag_line})"

    if [ "$TAG_CHECK" = "1" ]; then
        [ "$local_tag" = "$EXPECTED_TAG" ] || die "network tag ${local_tag} does not match the published ${NETWORK_LABEL} tag (${EXPECTED_TAG}); this binary or genesis bundle is not the published chain-${CHAIN_ID} set. Do not start the node."
        say "network tag ${local_tag} matches the published ${NETWORK_LABEL} tag"
    else
        say "network tag ${local_tag} (tag check skipped)"
    fi

    if [ "$CONF_CHECK" != "$CONF_PATH" ]; then
        mv "$CONF_CHECK" "$CONF_PATH"
        CONF_TMP=""
        say "wrote ${CONF_PATH}"
    fi
fi

# --------------------------------------------------------------------------
# systemd unit
# --------------------------------------------------------------------------
if [ "$WANT_SERVICE" = "1" ] && [ "$AS_ROOT" = "1" ] && command -v systemctl >/dev/null 2>&1; then
    if ! id -u "$SERVICE_USER" >/dev/null 2>&1; then
        run useradd --system --home-dir "$DATA_DIR" --shell /usr/sbin/nologin "$SERVICE_USER" \
            || say "could not create user ${SERVICE_USER}; continuing"
    fi
    run chown -R "${SERVICE_USER}:${SERVICE_USER}" "$DATA_DIR"

    if [ "$DRY_RUN" = "1" ]; then
        say "[dry-run] would write ${UNIT_PATH}"
    else
        cat >"$UNIT_PATH" <<EOF
[Unit]
Description=Kovanica full node (${NETWORK_LABEL})
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_USER}
ExecStart=${BIN_PATH} --config ${CONF_PATH}
Restart=on-failure
RestartSec=10
LimitNOFILE=65536

# 2-4 GB VPS target: the node is expected to stay well inside this.
MemoryMax=3G
CPUQuota=200%
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
ReadWritePaths=${DATA_DIR}

[Install]
WantedBy=multi-user.target
EOF
        chmod 0644 "$UNIT_PATH"
        say "wrote ${UNIT_PATH}"
    fi

    run systemctl daemon-reload
    run systemctl enable --now "${SERVICE_NAME}.service"
    say "started ${SERVICE_NAME}.service"
elif [ "$WANT_SERVICE" = "0" ]; then
    say "systemd unit not installed (--no-service)"
else
    say "not running as root: run the node yourself with"
    say "  ${BIN_PATH} --config ${CONF_PATH}"
fi

# --------------------------------------------------------------------------
# Summary
# --------------------------------------------------------------------------
cat <<EOF

Kovanica node ${VERSION} is installed.

  binary      ${BIN_PATH}
  config      ${CONF_PATH}
  chain data  ${DATA_DIR}
  network     ${NETWORK_LABEL}
  network tag ${EXPECTED_TAG:-(not pinned; --skip-tag-check)}

Next:
  curl -s http://127.0.0.1:8545/health
  curl -s -X POST http://127.0.0.1:8545/rpc -H 'Content-Type: application/json' \\
    -d '{"jsonrpc":"2.0","id":1,"method":"kovanica_syncStatus","params":[]}'

  Explorer  https://explorer.kovanica.online
  Faucet    https://kovanica.online/faucet
  Docs      https://docs.kovanica.online

Logs (systemd): journalctl -u ${SERVICE_NAME} -f
EOF
