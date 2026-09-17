#!/usr/bin/env sh
# kovanica — multi-repo workspace bootstrap.
#
# Run from the kovanica meta-repo root on any device. It:
#   1. clones / pulls every independent Kovanica repo
#   2. provisions toolchains (Rust, Node, Android, Docker detect)
#   3. installs per-repo deps and machine-local env
#   4. prints a doctor report (repo state, tool versions, ready commands)
#
# Every repo pulls automatically. Repos with a dirty tree are reported,
# never reset. Missing tools are skipped, never fatal.
#
# Usage:
#   ./dev.sh            full bootstrap (reconcile + toolchains + deps + doctor)
#   ./dev.sh --status   reconcile + doctor only (no toolchain/deps work)
#   ./dev.sh --skip-deps    same as --status
#   ./dev.sh --quiet    suppress per-step notes
#
# Env overrides:
#   KOVANICA_BRAIN_VAULT_URL   remote for kovanica-brain-vault (default GitHub)

set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
cd "$ROOT"

LOG="${DEV_LOG:-$ROOT/.dev.log}"
QUIET=0
DO_DEPS=1

for arg in "$@"; do
    case "$arg" in
        --status|--skip-deps) DO_DEPS=0 ;;
        --quiet|-q) QUIET=1 ;;
        --help|-h) sed -n '2,20p' "$0"; exit 0 ;;
        *) printf 'unknown flag: %s\n' "$arg" >&2; exit 2 ;;
    esac
done

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

log() { printf '[%s] %s\n' "$(date +%FT%T)" "$*" >>"$LOG"; }
note() { log "$*"; if [ "$QUIET" -eq 0 ]; then printf '  %s\n' "$*"; fi; }
warn() { log "WARN: $*"; printf '  WARN: %s\n' "$*" >&2; }
headline() { printf '\n== %s ==\n' "$*"; }

repo_list() {
    cat <<EOF
kovanica-protocol	https://github.com/KovanicaDAG/kovanica-protocol.git
kovanica-node	https://github.com/KovanicaDAG/kovanica-node.git
kovanica-web	https://github.com/KovanicaDAG/kovanica-web.git
kovanica-wallet	https://github.com/KovanicaDAG/kovanica-wallet.git
kovanica-mobile	https://github.com/KovanicaDAG/kovanica-mobile.git
kovanica-agent	https://github.com/KovanicaDAG/kovanica-agent.git
kovanica-installer	https://github.com/KovanicaDAG/kovanica-installer.git
kovanica-brain-vault	${KOVANICA_BRAIN_VAULT_URL:-https://github.com/KovanicaDAG/kovanica-brain-vault.git}
EOF
}

# ---------------------------------------------------------------- reconcile

sync_repo() {
    name="$1"
    url="$2"

    if [ -d "$name/.git" ]; then
        cur="$(git -C "$name" remote get-url origin 2>/dev/null || true)"
        if [ -z "$cur" ]; then
            git -C "$name" remote add origin "$url"
            note "$name: added origin"
        elif [ "$cur" != "$url" ] && [ "$cur" != "${url%.git}" ]; then
            warn "$name: origin differs ($cur) — leaving as-is"
        fi

        br="$(git -C "$name" rev-parse --abbrev-ref HEAD 2>/dev/null || echo detached)"
        if [ "$br" != "main" ]; then
            note "$name: on '$br' -> checking out main"
            git -C "$name" checkout -B main 2>/dev/null || git -C "$name" checkout main
        fi

        if ! git -C "$name" fetch --prune origin >/dev/null 2>&1; then
            warn "$name: fetch failed (offline? auth?)"
            head="$(git -C "$name" rev-parse --short HEAD 2>/dev/null || echo -)"
            echo "$name	$br	unknown	0	0	$head" >>"$WORK/repos.txt"
            return 0
        fi

        if git -C "$name" merge --ff-only origin/main >/dev/null 2>&1; then
            dirty=clean
        else
            changed="$(git -C "$name" status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
            warn "$name: local tree has $changed changed file(s) — not stashed, not reset"
            dirty=dirty
        fi

        ab="$(git -C "$name" rev-list --left-right --count origin/main...HEAD 2>/dev/null || echo '0 0')"
        behind="$(printf '%s' "$ab" | awk '{print $1}')"
        ahead="$(printf '%s' "$ab" | awk '{print $2}')"
        head="$(git -C "$name" rev-parse --short HEAD 2>/dev/null || echo -)"
        note "$name: $dirty (${behind:-0} behind, ${ahead:-0} ahead)"
        echo "$name	$br	$dirty	$ahead	$behind	$head" >>"$WORK/repos.txt"

    elif [ -d "$name" ]; then
        warn "$name: plain dir (no .git) — initializing from origin"
        if git init -q -b main "$name" \
            && git -C "$name" remote add origin "$url" \
            && git -C "$name" fetch -q origin \
            && git -C "$name" checkout -q -B main origin/main; then
            note "$name: initialized from origin"
            echo "$name	main	clean	0	0	$(git -C "$name" rev-parse --short HEAD)" >>"$WORK/repos.txt"
        else
            warn "$name: could not init from origin — left untouched"
            echo "$name	-	dirty	-	-	-" >>"$WORK/repos.txt"
        fi

    else
        if git clone -q --branch main --single-branch "$url" "$name"; then
            note "$name: cloned"
            echo "$name	main	clean	0	0	$(git -C "$name" rev-parse --short HEAD)" >>"$WORK/repos.txt"
        else
            warn "$name: clone failed (private repo / offline?) — git clone $url $name"
            echo "$name	-	miss	-	-	-" >>"$WORK/repos.txt"
        fi
    fi
}

ensure_data() {
    if [ ! -d kovanica-data ]; then
        mkdir -p kovanica-data
        printf '%s\n' "# kovanica-data" "" "Runtime data for the Kovanica ecosystem (not versioned)." >kovanica-data/README.md
        note "kovanica-data: created (runtime data, unversioned)"
    elif [ ! -f kovanica-data/README.md ]; then
        printf '%s\n' "# kovanica-data" "" "Runtime data for the Kovanica ecosystem (not versioned)." >kovanica-data/README.md
    fi
}

# --------------------------------------------------------------- toolchains

CARGO_OK=unset
NODE_OK=unset
ANDROID_SDK=unset
DOCKER_OK=unset
PYTHON_OK=unset
JAVA_OK=unset

ensure_rust() {
    if [ "$CARGO_OK" != unset ]; then return 0; fi
    if command -v cargo >/dev/null 2>&1; then
        CARGO_OK=yes
        note "cargo: $(cargo --version 2>/dev/null)"
        if ! command -v cc >/dev/null 2>&1 && ! command -v gcc >/dev/null 2>&1 && ! command -v clang >/dev/null 2>&1; then
            warn "no C compiler (cc/gcc/clang) found — cargo build-scripts and linking will fail. Linux: install build-essential; macOS: xcode-select --install"
        fi
        if command -v rustup >/dev/null 2>&1; then
            pin=
            if [ -f "$ROOT/kovanica-protocol/rust-toolchain.toml" ]; then
                pin="$(sed -n 's/^channel = "\(.*\)"/\1/p' "$ROOT/kovanica-protocol/rust-toolchain.toml")"
            fi
            if [ -n "$pin" ] && ! rustup toolchain list | grep -q "^$pin"; then
                note "rust: installing pinned toolchain $pin (rustfmt, clippy)"
                rustup toolchain install "$pin" --profile minimal --component rustfmt,clippy
            fi
        else
            warn "rustup not found — relying on system toolchain"
        fi
    else
        CARGO_OK=skip
        warn "cargo not found — protocol/node build & check skipped (install via rustup.rs)"
    fi
}

ensure_node() {
    if [ "$NODE_OK" != unset ]; then return 0; fi
    if command -v node >/dev/null 2>&1; then
        maj="$(node -v 2>/dev/null | sed 's/^v//; s/\..*//')"
        if [ "${maj:-0}" -ge 20 ]; then
            NODE_OK=yes
            note "node: $(node -v) ($(command -v node))"
            return 0
        fi
        warn "node: v$(node -v) too old (>=20 required for kovanica-web)"
    fi

    nvm_sh="${NVM_DIR:-$HOME/.nvm}/nvm.sh"
    if [ -f "$nvm_sh" ]; then
        bin_dir="$(bash -c '. "$1" >/dev/null 2>&1 && nvm install 22 >/dev/null 2>&1 && nvm which 22' _ "$nvm_sh" 2>/dev/null || true)"
        if [ -n "$bin_dir" ]; then
            export PATH="$(dirname "$bin_dir"):$PATH"
            NODE_OK=yes
            note "node: $(node -v) via nvm"
            return 0
        fi
        warn "nvm present but could not provision node 22"
    fi
    NODE_OK=skip
    warn "node >= 20 not available — web/wallet deps skipped"
}

ensure_android() {
    if command -v sdkmanager >/dev/null 2>&1; then
        if ! command -v java >/dev/null 2>&1 && [ -z "${JAVA_HOME:-}" ]; then
            ANDROID_SDK=needs-java
            warn "android: SDK present but no java/JAVA_HOME — package install skipped (install JDK 17 first)"
            return 0
        fi
        note "android: installing SDK/NDK packages (android-36, build-tools 36, ndk 27)"
        yes | sdkmanager --licenses >/dev/null 2>&1 || true
        if sdkmanager "platforms;android-36" "build-tools;36.0.0" "ndk;27.0.12077973" >/dev/null 2>&1; then
            ANDROID_SDK=present
        else
            ANDROID_SDK=partial
            warn "android: SDK/NDK package install failed — check sdkmanager licenses & network"
        fi
        if command -v cargo >/dev/null 2>&1 && ! command -v cargo-ndk >/dev/null 2>&1; then
            note "android: installing cargo-ndk"
            cargo install cargo-ndk --locked >/dev/null 2>&1 || warn "android: cargo-ndk install failed"
        fi
    else
        ANDROID_SDK=skip
        warn "android: sdkmanager not found — mobile/wallet android builds skipped"
    fi
}

probe_procs() {
    if command -v docker >/dev/null 2>&1; then DOCKER_OK=yes; else DOCKER_OK=skip; warn "docker not found — kovanica-agent stack (compose) requires it"; fi
    if command -v python3 >/dev/null 2>&1; then PYTHON_OK=yes; else PYTHON_OK=skip; warn "python3 not found — kovanica-agent venv skipped"; fi
    if command -v java >/dev/null 2>&1; then JAVA_OK=yes; else JAVA_OK=skip; fi
}

# --------------------------------------------------------------------- deps

cargo_check() {
    if [ "$DO_DEPS" -eq 0 ]; then return 0; fi
    ensure_rust
    dir="$1"
    shift
    if [ "$CARGO_OK" = skip ]; then warn "skip cargo check in $dir (no cargo)"; return 0; fi
    note "cargo check: $dir $*"
    if ! (cd "$ROOT/$dir" && cargo check "$@" --locked) >/dev/null 2>&1; then
        warn "$dir: cargo check failed — run manually: cd $dir && cargo check $* --locked"
    fi
}

npm_deps() {
    if [ "$DO_DEPS" -eq 0 ]; then return 0; fi
    ensure_node
    if [ "$NODE_OK" = skip ]; then return 0; fi
    dir="$1"; sub="$2"
    [ -d "$ROOT/$dir/$sub" ] || { warn "$dir/$sub: missing"; return 0; }
    if [ ! -d "$ROOT/$dir/$sub/node_modules" ]; then
        note "npm install: $dir/$sub"
        if ! (cd "$ROOT/$dir/$sub" && npm ci --no-fund --no-audit) >/dev/null 2>&1; then
            warn "$dir/$sub: npm ci failed — falling back to npm install"
            (cd "$ROOT/$dir/$sub" && npm install --no-fund --no-audit) >/dev/null 2>&1 \
                || warn "$dir/$sub: npm install failed"
        fi
    else
        note "$dir/$sub: node_modules present"
    fi
}

setup_agent() {
    if [ "$DO_DEPS" -eq 0 ]; then return 0; fi
    d="$ROOT/kovanica-agent"
    [ -d "$d" ] || { warn "kovanica-agent missing"; return 0; }

    if [ "$PYTHON_OK" = yes ] && [ ! -d "$d/venv" ]; then
        note "agent: creating python venv"
        python3 -m venv "$d/venv" || warn "agent: venv creation failed"
    fi
    if [ -x "$d/venv/bin/pip" ] && [ -f "$d/agent/requirements.txt" ]; then
        if ! "$d/venv/bin/pip" install -q -r "$d/agent/requirements.txt" >/dev/null 2>&1; then
            warn "agent: pip install of agent/requirements.txt failed"
        else
            note "agent: python deps installed"
        fi
    fi

    if [ ! -f "$d/.env" ] && [ -f "$d/.env.example" ]; then
        cp "$d/.env.example" "$d/.env"
        note "agent: created .env from .env.example"
    fi
    if [ -f "$d/.env" ] && ! grep -q '^AUTH_DEV_TOKEN=.\+' "$d/.env"; then
        tok="$(openssl rand -hex 24 2>/dev/null || true)"
        if [ -n "$tok" ]; then
            tmp="$(mktemp)"
            awk -v t="$tok" 'BEGIN{FS=OFS="="} /^AUTH_DEV_TOKEN=/{print "AUTH_DEV_TOKEN=" t; next} {print}' "$d/.env" >"$tmp" \
                && mv "$tmp" "$d/.env"
            warn "agent: generated new AUTH_DEV_TOKEN — keep it safe, it grants the dev role"
        fi
    fi

    if [ -f "$d/kovanica" ]; then
        first="$(head -n 1 "$d/kovanica" 2>/dev/null || true)"
        case "$first" in
            '#!/'*)
                tgt="$(printf '%s' "$first" | cut -c3-)"
                if [ ! -x "$tgt" ] && [ ! -f "$d/kovanica.local" ]; then
                    tmp="$(mktemp)"
                    { printf '#!%s\n' "$d/venv/bin/python3"; tail -n +2 "$d/kovanica"; } >"$tmp" \
                        && mv "$tmp" "$d/kovanica.local" \
                        && chmod +x "$d/kovanica.local"
                    note "agent: wrote portable launcher kovanica.local (tracked ./kovanica has a VPS-specific shebang)"
                fi
                ;;
        esac
    fi
}

gradle_warm() {
    if [ "$DO_DEPS" -eq 0 ]; then return 0; fi
    if [ "$JAVA_OK" != yes ]; then return 0; fi
    for d in "$ROOT/kovanica-wallet/android" "$ROOT/kovanica-mobile/android"; do
        if [ -x "$d/gradlew" ]; then
            (cd "$d" && ./gradlew --version >/dev/null 2>&1) \
                && note "gradle: warmed $d" \
                || warn "gradle warm-up failed in $d"
        fi
    done
}

check_vault_drift() {
    a="$ROOT/kovanica-brain-vault"
    b="$ROOT/kovanica-agent/kovanica-brain-vault"
    if [ -d "$a" ] && [ -d "$b" ]; then
        diffs="$(diff -rq "$a" "$b" 2>/dev/null | wc -l | tr -d ' ')"
        if [ -n "${diffs:-}" ] && [ "$diffs" -gt 0 ]; then
            warn "brain-vault drift: top-level vault differs from agent-shipped copy (${diffs} diff lines) — agent repo owns its copy; sync when intended"
        fi
    fi
}

meta_hygiene() {
    tracked="$(git ls-files 2>/dev/null | grep '/' || true)"
    if [ -n "$tracked" ]; then
        warn "meta repo tracks subdirectory files (beyond README/NETWORK/.gitignore) — run: git rm --cached -r <subdir>"
    fi
    if git ls-files -s 2>/dev/null | grep -q ':160000'; then
        warn "meta repo still records gitlinks — run: git rm --cached -r <subrepo>"
    fi
}

# ------------------------------------------------------------------ doctor

doctor() {
    headline "Repos"
    if [ -f "$WORK/repos.txt" ]; then
        printf '  %-22s %-8s %-7s %6s %6s  %s\n' NAME BRANCH STATE AHEAD BEHIND HEAD
        while IFS='	' read -r name br state ahead behind head; do
            printf '  %-22s %-8s %-7s %6s %6s  %s\n' \
                "$name" "$br" "$state" "${ahead:-0}" "${behind:-0}" "${head:--}"
        done <"$WORK/repos.txt"
    fi
    if [ -d kovanica-data ]; then printf '  %-22s (runtime data, unversioned)\n' kovanica-data; fi

    headline "Toolchain"
    probe() {
        name="$1"; shift
        if command -v "$name" >/dev/null 2>&1; then
            printf '  %-12s %s\n' "$name" "$("$@")"
        else
            printf '  %-12s %s\n' "$name" "missing"
        fi
    }
    probe cargo cargo --version
    if command -v rustup >/dev/null 2>&1; then
        printf '  %-12s %s\n' rustup "$(rustup toolchain list 2>/dev/null | awk '/\(active/ {print $1; exit}')"
    else
        printf '  %-12s %s\n' rustup "n/a (rust via $(command -v cargo))"
    fi
    probe node node -v
    probe python3 python3 --version
    probe docker docker --version
    if command -v java >/dev/null 2>&1; then
        printf '  %-12s %s\n' java "$(java -version 2>&1 | head -n 1)"
    else
        printf '  %-12s %s\n' java missing
    fi
    case "$ANDROID_SDK" in
        present) a="SDK ready (android-36 / ndk 27)" ;;
        partial) a="SDK present — package install incomplete" ;;
        needs-java) a="SDK present — install JDK 17 first" ;;
        *) a="skipped (no sdkmanager)" ;;
    esac
    printf '  %-12s %s\n' android "$a"

    headline "Deps"
    if [ -d kovanica-protocol/target ]; then
        printf '  %-22s target/ built\n' kovanica-protocol
    else
        printf '  %-22s target/ not built (first cargo check builds it)\n' kovanica-protocol
    fi
    [ -d kovanica-node/target ] && printf '  %-22s target/ built\n' kovanica-node || \
        printf '  %-22s target/ not built\n' kovanica-node
    [ -d kovanica-web/site/node_modules ] && printf '  %-22s node_modules/\n' kovanica-web/site || \
        printf '  %-22s not installed (npm ci)\n' kovanica-web/site
    [ -d kovanica-wallet/extension/node_modules ] && printf '  %-22s node_modules/\n' kovanica-wallet/extension || \
        printf '  %-22s not installed (npm ci)\n' kovanica-wallet/extension
    [ -x kovanica-agent/venv/bin/python ] && printf '  %-22s venv/\n' kovanica-agent || \
        printf '  %-22s no venv yet\n' kovanica-agent

    headline "Run"
    cat <<RUN
  web dev (all interfaces):   cd kovanica-web/site && npm run dev          -> http://localhost:8080
  web dev (local only):       npx vite dev --host 127.0.0.1 --port 8080
  node (solo, cargo):         cd kovanica-node && KOVANICA_POW=1 KOVANICA_MINE=0 KOVANICA_DATA=\$PWD/data \\
                              cargo run --release -p kovanica-node -- explorer 127.0.0.1:8080
  node (public testnet):      see kovanica-node/README.md (KOVANICA_PEERS=seed.kovanica.online:9000)
  agent REPL:                 (cd kovanica-agent && ./kovanica.local repl)   # portable launcher written by this script
  agent stack (needs docker): docker compose -f kovanica-agent/docker-compose.yml up -d qdrant vllm sandbox-runner agent-api
  wallet extension:           cd kovanica-wallet/extension && npm run dev
RUN
}

# ------------------------------------------------------------------- main

headline "Reconcile"
repo_list | while IFS='	' read -r name url; do
    sync_repo "$name" "$url"
done
ensure_data

headline "Toolchains & deps"
probe_procs
ensure_rust
ensure_node
ensure_android
gradle_warm
check_vault_drift

if [ "$DO_DEPS" -eq 1 ]; then
    cargo_check kovanica-protocol --workspace
    cargo_check kovanica-node -p kovanica-node
    npm_deps kovanica-web site
    npm_deps kovanica-wallet extension
    setup_agent
else
    note "deps step skipped (--status)"
fi

meta_hygiene
doctor
log "done"