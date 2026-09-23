#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# Kovanica Rust + CLI Complete Setup
# ============================================================

echo "=== 1. Instalacija osnovnih alata ==="

# Rust
if ! command -v rustup &>/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
fi

rustup component add rustfmt clippy rust-analyzer
rustup default stable

# Brzi cargo alati
cargo install --locked \
  cargo-edit cargo-watch cargo-nextest bacon \
  cargo-expand cargo-outdated cargo-audit cargo-deny \
  just sccache 2>/dev/null || true

# direnv
if ! command -v direnv &>/dev/null; then
  if command -v apt &>/dev/null; then
    sudo apt update && sudo apt install -y direnv
  elif command -v pacman &>/dev/null; then
    sudo pacman -S --noconfirm direnv
  else
    echo "Instaliraj direnv ručno: https://direnv.net"
  fi
fi

# mold + clang (brži linker)
if command -v apt &>/dev/null; then
  sudo apt install -y mold clang 2>/dev/null || true
elif command -v pacman &>/dev/null; then
  sudo pacman -S --noconfirm mold clang 2>/dev/null || true
fi

echo "=== 2. ~/.cargo/config.toml ==="
mkdir -p "$HOME/.cargo"
cat > "$HOME/.cargo/config.toml" << 'EOF'
[build]
rustc-wrapper = "sccache"

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[alias]
b = "build --release"
t = "nextest run"
c = "clippy --all-targets --all-features -- -D warnings"
f = "fmt --all"
w = "watch -x check -x test"
x = "expand"
EOF

echo "=== 3. Shell konfiguracija ==="

SHELL_RC=""
if [ -n "${ZSH_VERSION:-}" ] || [ -f "$HOME/.zshrc" ]; then
  SHELL_RC="$HOME/.zshrc"
elif [ -f "$HOME/.bashrc" ]; then
  SHELL_RC="$HOME/.bashrc"
else
  SHELL_RC="$HOME/.bashrc"
fi

# Ukloni staru sekciju ako postoji
sed -i '/# >>> Kovanica Dev Setup >>>/,/# <<< Kovanica Dev Setup <<</d' "$SHELL_RC" 2>/dev/null || true

cat >> "$SHELL_RC" << 'EOF'

# >>> Kovanica Dev Setup >>>
export CARGO_HOME="$HOME/.cargo"
export RUSTUP_HOME="$HOME/.rustup"
export PATH="$CARGO_HOME/bin:$PATH"
export RUSTC_WRAPPER=sccache
export SCCACHE_DIR="$HOME/.cache/sccache"

# Kovanica defaults
export KOVANICA_DATA="${KOVANICA_DATA:-$HOME/kovanica-data}"
export KOVANICA_PEERS="seed.kovanica.online:9000"
export KOVANICA_LISTEN="0.0.0.0:9000"

# --- Node aliases ---
alias knode='./target/release/kovanica-node'
alias knode-dev='cargo run --release --bin kovanica-node --'

alias kn-participant='KOVANICA_POW=1 KOVANICA_MINE=0 KOVANICA_FAUCET=0 KOVANICA_ALLOW_RESET=0 KOVANICA_OPERATOR=0 KOVANICA_LISTEN=0.0.0.0:9000 KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000 KOVANICA_DATA=$KOVANICA_DATA knode explorer 127.0.0.1:8080'
alias kn-explorer='KOVANICA_POW=1 KOVANICA_MINE=0 KOVANICA_FAUCET=0 KOVANICA_ALLOW_RESET=0 KOVANICA_OPERATOR=0 KOVANICA_LISTEN=0.0.0.0:9000 KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000 KOVANICA_DATA=$KOVANICA_DATA knode explorer 0.0.0.0:8080'
alias kn-miner='KOVANICA_POW=1 KOVANICA_MINE=1 KOVANICA_MINE_SECS=30 KOVANICA_FAUCET=0 KOVANICA_ALLOW_RESET=0 KOVANICA_OPERATOR=0 KOVANICA_LISTEN=0.0.0.0:9000 KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000 KOVANICA_DATA=$KOVANICA_DATA knode explorer 127.0.0.1:8080'

# --- API helpers ---
alias khead='curl -s https://api.kovanica.online/api/head | jq'
alias khead-local='curl -s http://127.0.0.1:8080/api/head | jq'
alias ktip='curl -s https://api.kovanica.online/api/head | jq -r ".tip // .hash // .blocks"'
alias ksupply='curl -s https://api.kovanica.online/api/head | jq "{native_minted, circulating, burned, total, max_supply, subsidy}"'
alias ksync='echo "=== LOCAL ===" && khead-local && echo -e "\n=== PUBLIC ===" && khead'

# --- Cargo quality ---
alias c='cargo'
alias cb='cargo build --release'
alias ct='cargo nextest run'
alias cc='cargo clippy --all-targets --all-features -- -D warnings'
alias cf='cargo fmt --all'
alias ccheck='cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo nextest run'
alias be='bacon'
alias cw='cargo watch -x check -x "nextest run"'

# --- Monorepo shortcuts ---
alias kn-build='cargo build --release -p kovanica-node'
alias kn-test='cargo nextest run -p kovanica-node -p kovanica-dag -p kovanica-state'
alias kn-clippy='cargo clippy -p kovanica-node -p kovanica-dag -p kovanica-state --all-features -- -D warnings'

# direnv
eval "$(direnv hook bash)" 2>/dev/null || eval "$(direnv hook zsh)" 2>/dev/null || true
# <<< Kovanica Dev Setup <<<
EOF

echo "=== 4. Globalni justfile predložak ==="
mkdir -p "$HOME/.config/just"
cat > "$HOME/.config/just/justfile" << 'EOF'
# Kovanica global justfile (poziva se s `just --justfile ~/.config/just/justfile ...`)

default:
    @just --list

build:
    cargo build --release -p kovanica-node

test:
    cargo nextest run -p kovanica-node -p kovanica-dag -p kovanica-state

clippy:
    cargo clippy -p kovanica-node -p kovanica-dag -p kovanica-state --all-features -- -D warnings

fmt:
    cargo fmt --all

check: fmt clippy test

participant:
    #!/usr/bin/env bash
    export KOVANICA_POW=1 KOVANICA_MINE=0 KOVANICA_FAUCET=0
    export KOVANICA_ALLOW_RESET=0 KOVANICA_OPERATOR=0
    export KOVANICA_LISTEN=0.0.0.0:9000
    export KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000
    export KOVANICA_DATA=${KOVANICA_DATA:-$HOME/kovanica-data}
    ./target/release/kovanica-node explorer 127.0.0.1:8080

explorer:
    #!/usr/bin/env bash
    export KOVANICA_POW=1 KOVANICA_MINE=0 KOVANICA_FAUCET=0
    export KOVANICA_ALLOW_RESET=0 KOVANICA_OPERATOR=0
    export KOVANICA_LISTEN=0.0.0.0:9000
    export KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000
    export KOVANICA_DATA=${KOVANICA_DATA:-$HOME/kovanica-data}
    ./target/release/kovanica-node explorer 0.0.0.0:8080

head:
    curl -s https://api.kovanica.online/api/head | jq

head-local:
    curl -s http://127.0.0.1:8080/api/head | jq

sync:
    @echo "=== LOCAL ==="
    @just head-local
    @echo ""
    @echo "=== PUBLIC ==="
    @just head
EOF

echo "=== 5. Primjer .envrc za Kovanica projekte ==="
cat > "$HOME/kovanica-envrc-template" << 'EOF'
# Kopiraj ovo kao .envrc u root monorepo-a i pokreni: direnv allow

export KOVANICA_DATA="$PWD/data"
export KOVANICA_PEERS="seed.kovanica.online:9000"
export KOVANICA_LISTEN="0.0.0.0:9000"
export KOVANICA_POW=1
export KOVANICA_MINE=0
export KOVANICA_FAUCET=0
export KOVANICA_ALLOW_RESET=0
export KOVANICA_OPERATOR=0

# Brži build
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1
EOF

echo ""
echo "============================================================"
echo " Setup gotov!"
echo "============================================================"
echo ""
echo "1. Ponovo učitaj shell:"
echo "   source $SHELL_RC"
echo ""
echo "2. U rootu Kovanica monorepo-a:"
echo "   cp ~/kovanica-envrc-template .envrc"
echo "   direnv allow"
echo ""
echo "3. Korisne naredbe:"
echo "   kn-build && kn-participant"
echo "   ksync"
echo "   ksupply"
echo "   ccheck"
echo "   be          # bacon"
echo "   just --justfile ~/.config/just/justfile check"
echo ""
echo "Sretno kodiranje."
