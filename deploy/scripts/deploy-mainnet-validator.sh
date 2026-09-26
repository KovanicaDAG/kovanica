#!/usr/bin/env bash
# deploy-mainnet-validator.sh — Provision a mainnet validator (PoA bootstrap phase).
# Run FROM operator machine that can SSH to target.
#
# Usage:
#   ./scripts/deploy-mainnet-validator.sh <validator_num> <user>@<host> [options]
#
#   validator_num: 1-7 (validator1 through validator7)
#
# Options:
#   --peers <host:port>   Bootstrap peers (comma-separated, default: all 3 seeds)
#   --p2p <port>          P2P listen port (default: 9000)
#   --keep-build          Keep the source tree after building
#   --authority-key <path> Local path to authority-N.env (will be copied to target)
#
# Example:
#   ./scripts/deploy-mainnet-validator.sh 1 root@192.0.2.10 --authority-key /secure/authority-1.env


usage() { grep '^#' "$0" | cut -c4-; exit 0; }

set -euo pipefail

VALIDATOR_NUM="${1:-}"
[[ -z "$VALIDATOR_NUM" || ! "$VALIDATOR_NUM" =~ ^[1-7]$ ]] && { echo "Error: validator_num must be 1-7"; usage; }
shift

NAME="validator${VALIDATOR_NUM}"

AUTHORITIES="<VALIDATOR_PUBKEY_1>,<VALIDATOR_PUBKEY_2>,<VALIDATOR_PUBKEY_3>,<VALIDATOR_PUBKEY_4>,<VALIDATOR_PUBKEY_5>,<VALIDATOR_PUBKEY_6>,<VALIDATOR_PUBKEY_7>"
THRESHOLD=5
SLOT_DURATION=120000

PEERS="seed1.mainnet.kovanica.online:9000,seed2.mainnet.kovanica.online:9000,seed3.mainnet.kovanica.online:9000"
P2P_PORT=9000
KEEP_BUILD=0
METRICS_PORT=9090
AUTHORITY_KEY_PATH=""

TARGET="${1:-}"
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help) usage ;;
        --peers) PEERS="$2"; shift 2 ;;
        --p2p) P2P_PORT="$2"; shift 2 ;;
        --keep-build) KEEP_BUILD=1; shift ;;
        --authority-key) AUTHORITY_KEY_PATH="$2"; shift 2 ;;
        *)
            if [[ -z "$TARGET" ]]; then TARGET="$1"; else echo "Unknown argument: $1"; usage; fi
            shift ;;
    esac
done

[[ -z "$TARGET" ]] && { echo "Error: <user>@<host> required"; usage; }

IFS=',' read -ra AUTH_ARRAY <<< "$AUTHORITIES"
if [[ ${#AUTH_ARRAY[@]} -ne 7 ]]; then
    echo "ERROR: AUTHORITIES must contain exactly 7 comma-separated 64-char hex pubkeys"
    echo "Run AUTHORITY_KEY_CEREMONY.md ceremony first, then edit this script"
    exit 1
fi
for key in "${AUTH_ARRAY[@]}"; do
    if [[ ! "$key" =~ ^[0-9a-f]{64}$ ]]; then
        echo "ERROR: Invalid pubkey format: $key (must be 64 hex chars)"
        exit 1
    fi
done

REPO_ROOT="/root/kovanica/protocol"
REMOTE_SRC="/opt/kovanica-src"
REMOTE_DATA="/var/lib/kovanica-mainnet"

echo "=== Kovanica Mainnet ${NAME} deploy -> $TARGET ==="
echo "Authorities: ${#AUTH_ARRAY[@]}, Threshold: $THRESHOLD, Slot: ${SLOT_DURATION}ms"

echo "[1/8] Shipping source tarball..."
TARBALL=$(mktemp /tmp/kovanica-src.XXXX.tar.gz)
git -C "$REPO_ROOT" archive --format=tar.gz -o "$TARBALL" HEAD
ssh "$TARGET" "sudo mkdir -p '$REMOTE_SRC' && sudo rm -rf '$REMOTE_SRC'/*"
scp -q "$TARBALL" "$TARGET:/tmp/kovanica-src.tar.gz"
ssh "$TARGET" "sudo tar -xzf /tmp/kovanica-src.tar.gz -C '$REMOTE_SRC' && sudo chown -R \$(whoami) '$REMOTE_SRC'"
rm -f "$TARBALL"

echo "[2/8] Installing build prerequisites..."
ssh "$TARGET" 'if command -v apt-get >/dev/null; then
    sudo apt-get update -qq && sudo apt-get install -y -qq curl ca-certificates build-essential pkg-config >/dev/null
elif command -v dnf >/dev/null; then
    sudo dnf install -y -q gcc gcc-c++ make pkgconfig 2>/dev/null || sudo dnf install -y -q gcc gcc-c++ make
else
    echo "unsupported distro: need apt-get or dnf" >&2; exit 1
fi'

echo "[3/8] Ensuring swap..."
ssh "$TARGET" 'if [ "$(free -m | awk "/Mem:/{print \$2}")" -lt 2000 ] && ! swapon --show | grep -q .; then
    sudo fallocate -l 2G /swapfile && sudo chmod 600 /swapfile &&
    sudo mkswap /swapfile && sudo swapon /swapfile &&
    echo "/swapfile none swap sw 0 0" | sudo tee -a /etc/fstab >/dev/null;
fi'

echo "[4/8] Installing Rust toolchain..."
ssh "$TARGET" 'if ! command -v cargo >/dev/null; then
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none >/dev/null 2>&1;
fi;
source "$HOME/.cargo/env"'

echo "[5/8] Building release binary..."
ssh "$TARGET" "source \$HOME/.cargo/env && cd '$REMOTE_SRC' && cargo build --release --locked -p kovanica-node"

echo "[6/8] Installing systemd service..."
AUTH_KEY_INSTALL=""
if [[ -n "$AUTHORITY_KEY_PATH" && -f "$AUTHORITY_KEY_PATH" ]]; then
    echo "  Installing authority key..."
    scp -q "$AUTHORITY_KEY_PATH" "$TARGET:/tmp/authority.env"
    ssh "$TARGET" "sudo mkdir -p /etc/kovanica/authority-keys && sudo mv /tmp/authority.env /etc/kovanica/authority-keys/authority-${VALIDATOR_NUM}.env && sudo chmod 600 /etc/kovanica/authority-keys/authority-${VALIDATOR_NUM}.env"
    AUTH_KEY_INSTALL="EnvironmentFile=/etc/kovanica/authority-keys/authority-${VALIDATOR_NUM}.env"
fi

ssh "$TARGET" "sudo mkdir -p '$REMOTE_DATA' && \
sudo cp '$REMOTE_SRC/target/release/kovanica-node' /usr/local/bin/kovanica-node && \
sudo tee /etc/systemd/system/kovanica-$NAME.service >/dev/null <<EOF
[Unit]
Description=Kovanica Mainnet Validator ($NAME)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=$REMOTE_DATA
Environment=KOVANICA_LISTEN=0.0.0.0:$P2P_PORT
Environment=KOVANICA_CONSENSUS=poa
Environment=KOVANICA_AUTHORITIES=$AUTHORITIES
Environment=KOVANICA_AUTHORITY_THRESHOLD=$THRESHOLD
Environment=KOVANICA_SLOT_DURATION=$SLOT_DURATION
Environment=KOVANICA_PEERS=$PEERS
Environment=KOVANICA_OPERATOR=0
Environment=KOVANICA_FAUCET=0
Environment=KOVANICA_ALLOW_RESET=0
Environment=KOVANICA_DATA=$REMOTE_DATA
Environment=KOVANICA_METRICS=0.0.0.0:$METRICS_PORT
Environment=KOVANICA_PRODUCE=1
Environment=KOVANICA_PRODUCE_SECS=120
$AUTH_KEY_INSTALL
ExecStart=/usr/local/bin/kovanica-node
Restart=always
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl daemon-reload && sudo systemctl enable --now kovanica-$NAME"

echo "[7/8] Firewall + fail2ban..."
ssh "$TARGET" "
command -v ufw >/dev/null && sudo ufw allow ${P2P_PORT}/tcp comment 'Kovanica P2P' >/dev/null || true
command -v apt-get >/dev/null && sudo apt-get install -y -qq fail2ban >/dev/null 2>&1 || true
sudo tee /etc/fail2ban/jail.local >/dev/null <<'F2B'
[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 3
bantime = 3600
findtime = 600
[recidive]
enabled = true
filter = recidive
logpath = /var/log/fail2ban.log
bantime = 86400
findtime = 86400
F2B
sudo systemctl enable --now fail2ban || true
" || true

echo "[8/8] Prometheus node-exporter..."
ssh "$TARGET" "
command -v apt-get >/dev/null && sudo apt-get install -y -qq prometheus-node-exporter >/dev/null 2>&1 || true
sudo systemctl enable --now prometheus-node-exporter || true
" || true

if [[ "$KEEP_BUILD" != 1 ]]; then
    echo "Cleaning up source tree..."
    ssh "$TARGET" "sudo rm -rf '$REMOTE_SRC' /tmp/kovanica-src.tar.gz"
fi

echo "Waiting for sync verification..."
sleep 20

echo "Fetching genesis from seed1..."
SEED_GENESIS=$(curl -sS --max-time 10 "http://seed1.mainnet.kovanica.online:8080/api/head" 2>/dev/null | python3 -c "import json,sys; print(json.load(sys.stdin)['genesis'])" 2>/dev/null || echo "unknown")

echo "Checking $NAME genesis..."
NEW_HEAD=$(ssh "$TARGET" "curl -sS --max-time 10 http://127.0.0.1:8080/api/head 2>/dev/null || true")
NEW_GENESIS=$(echo "$NEW_HEAD" | python3 -c "import json,sys; print(json.load(sys.stdin).get('genesis',''))" 2>/dev/null || echo "unknown")

if [[ -n "$NEW_GENESIS" && "$NEW_GENESIS" == "$SEED_GENESIS" ]]; then
    echo "=== OK: $NAME synced to the same network (genesis $NEW_GENESIS) ==="
else
    echo "=== WARNING: could not confirm genesis match (seed: $SEED_GENESIS, got: $NEW_GENESIS) ==="
    echo "Check: ssh $TARGET journalctl -u kovanica-$NAME -n 50"
fi

cat <<EOF

=== $NAME (Mainnet Validator) deployed ===

P2P endpoint      : \${TARGET#*@}:\${P2P_PORT}
Prometheus metrics: loopback :\${METRICS_PORT}   (ssh -L \${METRICS_PORT}:127.0.0.1:\${METRICS_PORT} \${TARGET})

Service: systemctl status kovanica-\${NAME}
Logs:    journalctl -u kovanica-\${NAME} -f
Data:    /var/lib/kovanica-mainnet

Authority key: /etc/kovanica/authority-keys/authority-${VALIDATOR_NUM}.env (mode 600)

Next:
  1. Verify validator produces blocks in its assigned slots
  2. Monitor: journalctl -u kovanica-$NAME -f | grep "produced"
  3. After T+14d: coordinate PoA->PoW transition (GENESIS_PLAYBOOK.md Phase 4)
EOF