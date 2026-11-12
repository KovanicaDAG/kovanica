#!/bin/bash
# Tests for node/scripts/install.sh.
#
#   bash node/scripts/install.test.sh
#
# Everything runs against a fake release served over file://, in a throwaway
# directory: no network, no root, no systemd. The point is the failure modes,
# because the installer is served to strangers and a wrong network or a wrong
# bundle must stop the install instead of starting a node nobody can peer with.
set -uo pipefail

SCRIPT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/install.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

failures=0
pass() { printf 'PASS %s\n' "$1"; }
fail() { printf 'FAIL %s\n' "$1"; failures=$((failures + 1)); }

sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

# A throwaway PRO-21 address, never the official treasury: chains 1 and 2 now
# require the recorded treasury because it is part of the network tag.
TREASURY="kvncqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdag"
OTHER_TREASURY="kvncpppppppppppppppppppppppppppppppppppppppppppppppppppppppppdag"

# build_release <dir> <bundle asset> <tag file content> <binary tag printed> [extra bundle file]
build_release() {
    dir="$TMP/$1"; asset="$2"; tag_file="$3"; bin_tag="$4"; extra="${5:-}"
    mkdir -p "$dir/node" "$dir/bundle"
    cat >"$dir/node/kovanica-node" <<EOF
#!/bin/sh
case " \$* " in
  *" --check-config "*) exit 0 ;;
  *" network-tag "*) printf '%s\n' "$bin_tag"; exit 0 ;;
esac
exit 0
EOF
    chmod +x "$dir/node/kovanica-node"
    tar -C "$dir/node" -czf "$dir/kovanica-node-x86_64-linux.tar.gz" kovanica-node
    printf 'chain_id = 2\ntreasury_address = "%s"\n' "$TREASURY" >"$dir/bundle/genesis_validators.toml"
    printf 'aa\n' >"$dir/bundle/founder_premine.hex"
    [ -n "$extra" ] && printf 'address = "x"\n' >"$dir/bundle/$extra"
    [ -n "$tag_file" ] && printf '%s\n' "$tag_file" >"$dir/bundle/expected-tag"
    tar -C "$dir/bundle" -czf "$dir/$asset" .
    {
        printf '%s  %s\n' "$(sha256 "$dir/kovanica-node-x86_64-linux.tar.gz")" kovanica-node-x86_64-linux.tar.gz
        printf '%s  %s\n' "$(sha256 "$dir/$asset")" "$asset"
    } >"$dir/SHA256SUMS"
}

# reseal_bundle <dir> <bundle asset> <genesis_validators.toml content>
# Rebuilds the archive and checksums after a test rewrites the bundle's file.
reseal_bundle() {
    dir="$TMP/$1"; asset="$2"; content="$3"
    printf '%s\n' "$content" >"$dir/bundle/genesis_validators.toml"
    tar -C "$dir/bundle" -czf "$dir/$asset" .
    {
        printf '%s  %s\n' "$(sha256 "$dir/kovanica-node-x86_64-linux.tar.gz")" kovanica-node-x86_64-linux.tar.gz
        printf '%s  %s\n' "$(sha256 "$dir/$asset")" "$asset"
    } >"$dir/SHA256SUMS"
}

# run <release dir> <name> [installer args...] -> sets $out, $status
run() {
    dir="$TMP/$1"; name="$2"; shift 2
    home="$TMP/run-$name"
    out="$(KOVANICA_RELEASE_BASE="file://$dir" \
           KOVANICA_HOME="$home/home" \
           KOVANICA_DATA_DIR="$home/data" \
           KOVANICA_PREFIX="$home/prefix" \
           sh "$SCRIPT" --no-service "$@" 2>&1)"
    status=$?
}

expect() { # <description> <pattern>
    if printf '%s' "$out" | grep -qF -- "$2"; then
        pass "$1"
    else
        fail "$1 (expected to see: $2)"
        printf '%s\n' "$out" | sed 's/^/    | /'
    fi
}

# --- a healthy chain-2 install -------------------------------------------------
build_release chain2 kovanica-chain2-genesis.tar.gz 2-0123456789abcdef 2-0123456789abcdef
run chain2 ok
expect "installs and verifies the tag" "matches the published testnet (chain 2) tag"

if [ "$status" -eq 0 ] && [ -x "$TMP/run-ok/prefix/bin/kovanica-node" ] \
   && [ -f "$TMP/run-ok/home/kovanica.toml" ]; then
    pass "binary and config are in place"
else
    fail "binary and config are in place (status $status)"
fi

conf="$TMP/run-ok/home/kovanica.toml"
if grep -q '^chain_id = 2$' "$conf" \
   && grep -q "^treasury_address = \"$TREASURY\"\$" "$conf" \
   && grep -q '^bootnodes = \["seed1.kovanica.online:8000", "seed3.kovanica.online:8000"\]$' "$conf"; then
    pass "config targets chain 2 with the recorded treasury and the current seeds"
else
    fail "config targets chain 2 with the current seeds"
    sed 's/^/    | /' "$conf"
fi

if grep -q 'use_vrf_leader\|use_mysticghost\|round_duration_ms' "$conf"; then
    fail "config carries no dead Mysticeti knobs"
    grep -n 'use_vrf_leader\|use_mysticghost\|round_duration_ms' "$conf"
else
    pass "config carries no dead Mysticeti knobs"
fi

if [ ! -f "$TMP/run-ok/data/expected-tag" ]; then
    pass "expected-tag is not copied into the data dir"
else
    fail "expected-tag is not copied into the data dir"
fi

# --- the tag is not optional ---------------------------------------------------
run chain2 skip --skip-tag-check
expect "--skip-tag-check is an explicit opt-out" "tag check skipped"

build_release untreasured kovanica-chain2-genesis.tar.gz 2-0123456789abcdef 2-0123456789abcdef
reseal_bundle untreasured kovanica-chain2-genesis.tar.gz 'chain_id = 2'
run untreasured no-treasury
expect "a chain-2 bundle without a treasury is refused" "records no treasury_address"
if [ "$status" -ne 0 ] && [ ! -f "$TMP/run-no-treasury/home/kovanica.toml" ]; then
    pass "a refused treasury leaves no config behind"
else
    fail "a refused treasury leaves no config behind (status $status)"
fi

build_release untagged kovanica-chain2-genesis.tar.gz "" 2-0123456789abcdef
run untagged no-tag
expect "fails closed when nothing pins the tag" "no expected network tag for chain 2"
if [ "$status" -ne 0 ] && [ ! -f "$TMP/run-no-tag/home/kovanica.toml" ]; then
    pass "a failed tag check leaves no config behind"
else
    fail "a failed tag check leaves no config behind (status $status)"
fi

# --- a wrong pin, a wrong chain, a wrong bundle --------------------------------
run chain2 wrong-chain-id --chain-id 4
expect "the chain-4 bundle is not reachable by default" "cannot download kovanica-chain4-genesis.tar.gz"

run chain2 pin-other-chain --expected-tag 4-0123456789abcdef
expect "a tag for another chain is rejected" "belongs to chain 4, not chain 2"

run chain2 bad-tag-shape --expected-tag 2-nothex
expect "a malformed tag is rejected" "is not of the form"

run chain2 non-numeric --chain-id two
expect "a non-numeric chain id is rejected" "must be a number"

build_release tagged-wrong-tag kovanica-chain2-genesis.tar.gz 2-0123456789abcdef 2-ffffffffffffffff
run tagged-wrong-tag mismatch
expect "a binary on the wrong set is refused" "does not match the published testnet (chain 2) tag"

# --- mainnet rules -------------------------------------------------------------
build_release mainnet kovanica-chain1-genesis.tar.gz 1-0123456789abcdef 1-0123456789abcdef funded_accounts.toml
run mainnet funded --chain-id 1
expect "mainnet refuses a bundle with funded_accounts.toml" "chain 1 (mainnet) forbids"

# --- an existing config is never rewritten ------------------------------------
build_release chain2c kovanica-chain2-genesis.tar.gz 2-0123456789abcdef 2-0123456789abcdef
mkdir -p "$TMP/run-treasury-mismatch/home"
printf 'chain_id = 2\ntreasury_address = "%s"\n' "$OTHER_TREASURY" >"$TMP/run-treasury-mismatch/home/kovanica.toml"
run chain2c treasury-mismatch
expect "an existing config with another treasury is refused" "but the bundle binds"
if [ "$status" -ne 0 ] \
   && grep -q "^treasury_address = \"$OTHER_TREASURY\"\$" "$TMP/run-treasury-mismatch/home/kovanica.toml"; then
    pass "the refused existing config is left untouched"
else
    fail "the refused existing config is left untouched (status $status)"
    sed 's/^/    | /' "$TMP/run-treasury-mismatch/home/kovanica.toml"
fi

build_release chain2d kovanica-chain2-genesis.tar.gz 2-0123456789abcdef 2-0123456789abcdef
mkdir -p "$TMP/run-treasury-match/home"
printf 'chain_id = 2\ntreasury_address = "%s"\n' "$TREASURY" >"$TMP/run-treasury-match/home/kovanica.toml"
run chain2d treasury-match
expect "an existing config with the bundle treasury is kept" "existing config treasury matches the bundle"

# --- an existing data dir is never overwritten --------------------------------
build_release chain2b kovanica-chain2-genesis.tar.gz 2-0123456789abcdef 2-0123456789abcdef
mkdir -p "$TMP/run-busy/data"
: >"$TMP/run-busy/data/consensus.redb"
run chain2b busy
expect "an existing data dir is protected" "already holds chain data"

if [ "$failures" -ne 0 ]; then
    printf '\n%s test(s) failed\n' "$failures"
    exit 1
fi
printf '\nall installer tests passed\n'
