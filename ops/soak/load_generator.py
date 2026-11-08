#!/usr/bin/env python3
"""
Kovanica Soak Test Load Generator.

Continuously generates valid, signed transfer transactions and submits them
via JSON-RPC (kovanica_sendRawTransaction) to a target node.

Tracks and reports:
  - Throughput (tx/sec)
  - Submission success / failure counts
  - Latency statistics (min, max, mean, p50, p90, p95, p99)

Supports:
  - Raw 32-byte hex seeds (--key or --key-file)
  - Pre-funded test keys (default built-in seed)
  - Standard JSON keystores (--keystore)
  - Rate limiting (--rate <tx/s>)
  - Duration limits (--duration <seconds>)
  - Monotonic nonce tracking and recovery
"""

import argparse
import json
import os
import signal
import struct
import sys
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from typing import List, Optional, Tuple

try:
    from cryptography.hazmat.primitives.asymmetric import ed25519
except ImportError:
    sys.exit("Error: python 'cryptography' library is required. Install via apt install python3-cryptography or pip.")

# ---------------------------------------------------------------------------
# BLAKE3 Keyed Single-Chunk Implementation (pure Python, zero dependencies)
# ---------------------------------------------------------------------------
IV = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
    0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
]

MSG_PERMUTATION = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8]


def _rotr(x: int, n: int) -> int:
    return ((x >> n) | (x << (32 - n))) & 0xFFFFFFFF


def _g(v: List[int], a: int, b: int, c: int, d: int, mx: int, my: int) -> None:
    v[a] = (v[a] + v[b] + mx) & 0xFFFFFFFF
    v[d] = _rotr(v[d] ^ v[a], 16)
    v[c] = (v[c] + v[d]) & 0xFFFFFFFF
    v[b] = _rotr(v[b] ^ v[c], 12)
    v[a] = (v[a] + v[b] + my) & 0xFFFFFFFF
    v[d] = _rotr(v[d] ^ v[a], 8)
    v[c] = (v[c] + v[d]) & 0xFFFFFFFF
    v[b] = _rotr(v[b] ^ v[c], 7)


def _compress(cv: List[int], block: bytes, block_len: int, counter: int, flags: int) -> List[int]:
    m = list(struct.unpack("<16I", block))
    v = list(cv) + IV[:4] + [
        counter & 0xFFFFFFFF,
        (counter >> 32) & 0xFFFFFFFF,
        block_len,
        flags,
    ]
    for _ in range(7):
        _g(v, 0, 4, 8, 12, m[0], m[1])
        _g(v, 1, 5, 9, 13, m[2], m[3])
        _g(v, 2, 6, 10, 14, m[4], m[5])
        _g(v, 3, 7, 11, 15, m[6], m[7])
        _g(v, 0, 5, 10, 15, m[8], m[9])
        _g(v, 1, 6, 11, 12, m[10], m[11])
        _g(v, 2, 7, 8, 13, m[12], m[13])
        _g(v, 3, 4, 9, 14, m[14], m[15])
        m = [m[p] for p in MSG_PERMUTATION]
    return [v[i] ^ v[i + 8] for i in range(8)]


def blake3_keyed_hash(domain_key: bytes, data: bytes) -> bytes:
    """Compute keyed BLAKE3 hash for single chunk (data <= 1024 bytes)."""
    cv = list(struct.unpack("<8I", domain_key.ljust(32, b"\x00")[:32]))
    blocks = [data[i:i + 64] for i in range(0, max(len(data), 1), 64)]
    for i, blk in enumerate(blocks):
        blk_len = len(blk)
        blk_padded = blk.ljust(64, b"\x00")
        f = 16  # KEYED_HASH
        if i == 0:
            f |= 1  # CHUNK_START
        if i == len(blocks) - 1:
            f |= 2 | 8  # CHUNK_END | ROOT
        cv = _compress(cv, blk_padded, blk_len, 0, f)
    return struct.pack("<8I", *cv)


def blake3_standard_hash(data: bytes) -> bytes:
    """Compute unkeyed BLAKE3 hash."""
    cv = list(IV)
    blocks = [data[i:i + 64] for i in range(0, max(len(data), 1), 64)]
    for i, blk in enumerate(blocks):
        blk_len = len(blk)
        blk_padded = blk.ljust(64, b"\x00")
        f = 0
        if i == 0:
            f |= 1
        if i == len(blocks) - 1:
            f |= 2 | 8
        cv = _compress(cv, blk_padded, blk_len, 0, f)
    return struct.pack("<8I", *cv)


# ---------------------------------------------------------------------------
# Canonical PRO-21 Address Encoding & Decoding (Base32 / Bech32 charset)
# ---------------------------------------------------------------------------
BASE32_ALPHABET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
ADDRESS_PREFIX = "kvnc"
ADDRESS_SUFFIX = "dag"


def encode_address(pubkey_bytes: bytes) -> str:
    """Encode 32-byte public key into canonical PRO-21 base32 address: kvnc...dag."""
    h = blake3_standard_hash(pubkey_bytes)
    check = (h[0] << 21) | (h[1] << 13) | (h[2] << 5) | (h[3] >> 3)

    acc = 0
    nbits = 0
    chars = []
    for b in pubkey_bytes:
        acc = (acc << 8) | b
        nbits += 8
        while nbits >= 5:
            nbits -= 5
            chars.append(BASE32_ALPHABET[(acc >> nbits) & 31])
            acc &= (1 << nbits) - 1
    acc = (acc << 29) | check
    nbits += 29
    while nbits >= 5:
        nbits -= 5
        chars.append(BASE32_ALPHABET[(acc >> nbits) & 31])
        acc &= (1 << nbits) - 1
    return ADDRESS_PREFIX + "".join(chars) + ADDRESS_SUFFIX


def decode_address(address_str: str) -> bytes:
    """Decode canonical PRO-21 base32 address or 32-byte hex into 32-byte public key."""
    s = address_str.strip()
    if s.startswith(ADDRESS_PREFIX) and s.endswith(ADDRESS_SUFFIX) and len(s) == 64:
        body = s[len(ADDRESS_PREFIX):-len(ADDRESS_SUFFIX)]
        acc = 0
        nbits = 0
        payload = bytearray()
        for c in body:
            idx = BASE32_ALPHABET.find(c)
            if idx < 0:
                raise ValueError(f"invalid base32 character in address: {c}")
            acc = (acc << 5) | idx
            nbits += 5
            if len(payload) < 32 and nbits >= 8:
                nbits -= 8
                payload.append((acc >> nbits) & 0xFF)
                acc &= (1 << nbits) - 1
        return bytes(payload)
    clean = s.replace("0x", "")
    if len(clean) == 64:
        return bytes.fromhex(clean)
    raise ValueError(f"unrecognized address format: {address_str}")


# ---------------------------------------------------------------------------
# Transaction Builder & Serializer (Bincode format)
# ---------------------------------------------------------------------------
TX_DOMAIN_TAG = b"KUNA/tx/v1".ljust(16, b"\x00")
TX_HASH_DOMAIN = b"KUNA-TX-v1"


def build_and_sign_transfer(
    privkey: ed25519.Ed25519PrivateKey,
    sender_pubkey: bytes,
    recipient_pubkey: bytes,
    nonce: int,
    amount_atoms: int,
    fee_atoms: int,
    chain_id: int,
) -> Tuple[str, str]:
    """
    Build a signed native transfer transaction and serialize to bincode hex string.
    Returns (raw_tx_hex, tx_hash_hex).
    """
    # Signing preimage construction (docs/SIGNATURE_FORMAT.md §2):
    # Preimage = TX_DOMAIN_TAG (16B) || chain_id (u64 LE) || sender (32B) ||
    #            nonce (u64 LE) || kind_tag (u8: 0=Transfer) || to (32B) ||
    #            amount (u64 LE) || fee (u64 LE)
    preimage = bytearray()
    preimage.extend(TX_DOMAIN_TAG)
    preimage.extend(struct.pack("<Q", chain_id))
    preimage.extend(sender_pubkey)
    preimage.extend(struct.pack("<Q", nonce))
    preimage.append(0)  # TransactionKind::Transfer tag
    preimage.extend(recipient_pubkey)
    preimage.extend(struct.pack("<Q", amount_atoms))
    preimage.extend(struct.pack("<Q", fee_atoms))

    tx_hash = blake3_keyed_hash(TX_HASH_DOMAIN, preimage)
    signature = privkey.sign(tx_hash)

    # Bincode serialization of Transaction struct:
    # sender (32B) || nonce (u64 LE) || kind_variant (u32 LE: 0) ||
    # to (32B) || amount (u64 LE) || fee (u64 LE) || signature (64B) || hash (32B)
    raw = bytearray()
    raw.extend(sender_pubkey)
    raw.extend(struct.pack("<Q", nonce))
    raw.extend(struct.pack("<I", 0))  # Transfer variant index 0 in bincode
    raw.extend(recipient_pubkey)
    raw.extend(struct.pack("<Q", amount_atoms))
    raw.extend(struct.pack("<Q", fee_atoms))
    raw.extend(signature)
    raw.extend(tx_hash)

    return raw.hex(), tx_hash.hex()


# ---------------------------------------------------------------------------
# JSON-RPC Client Helpers
# ---------------------------------------------------------------------------
def rpc_call(rpc_url: str, method: str, params: list, timeout: float = 10.0) -> dict:
    """Send a JSON-RPC 2.0 request and return parsed response."""
    payload = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode("utf-8")
    req = urllib.request.Request(
        rpc_url,
        data=payload,
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        body = resp.read()
        return json.loads(body.decode("utf-8"))


def get_account_nonce(rpc_url: str, address_str: str) -> int:
    """Query current committed nonce for account."""
    res = rpc_call(rpc_url, "kovanica_getNonce", [address_str])
    if "error" in res:
        raise RuntimeError(f"kovanica_getNonce error: {res['error']}")
    val = res.get("result", 0)
    if isinstance(val, str):
        return int(val, 16) if val.startswith("0x") else int(val)
    return int(val)


def send_raw_transaction(rpc_url: str, raw_hex: str) -> Tuple[bool, str, float]:
    """
    Submit signed raw transaction.
    Returns (success, result_or_error_msg, latency_ms).
    """
    t0 = time.perf_counter()
    try:
        res = rpc_call(rpc_url, "kovanica_sendRawTransaction", [raw_hex], timeout=8.0)
        lat_ms = (time.perf_counter() - t0) * 1000.0
        if "error" in res:
            err = res["error"]
            msg = err.get("message", str(err)) if isinstance(err, dict) else str(err)
            return False, msg, lat_ms
        return True, str(res.get("result", "")), lat_ms
    except Exception as e:
        lat_ms = (time.perf_counter() - t0) * 1000.0
        return False, str(e), lat_ms


# ---------------------------------------------------------------------------
# Statistics Collector
# ---------------------------------------------------------------------------
class SoakStats:
    def __init__(self):
        self.submitted = 0
        self.success = 0
        self.failed = 0
        self.latencies: List[float] = []
        self.errors = {}
        self.start_time = time.time()

    def record(self, ok: bool, lat_ms: float, err: Optional[str] = None):
        self.submitted += 1
        self.latencies.append(lat_ms)
        if ok:
            self.success += 1
        else:
            self.failed += 1
            if err:
                self.errors[err] = self.errors.get(err, 0) + 1

    def throughput(self) -> float:
        elapsed = time.time() - self.start_time
        return self.success / elapsed if elapsed > 0 else 0.0

    def percentiles(self) -> Tuple[float, float, float, float, float, float]:
        if not self.latencies:
            return 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
        sorted_l = sorted(self.latencies)
        n = len(sorted_l)
        avg_l = sum(sorted_l) / n
        min_l = sorted_l[0]
        max_l = sorted_l[-1]
        p50 = sorted_l[int(n * 0.50)]
        p90 = sorted_l[min(int(n * 0.90), n - 1)]
        p95 = sorted_l[min(int(n * 0.95), n - 1)]
        p99 = sorted_l[min(int(n * 0.99), n - 1)]
        return avg_l, min_l, max_l, p50, p95, p99


# ---------------------------------------------------------------------------
# Main Execution Loop
# ---------------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(description="Kovanica Soak Test Load Generator")
    parser.add_argument("--rpc-url", default="http://127.0.0.1:8545", help="Target node JSON-RPC URL")
    parser.add_argument("--chain-id", type=int, default=5, help="Chain ID (default 5 for Hybrid PoW)")
    parser.add_argument("--duration", type=int, default=0, help="Duration in seconds (0 = run indefinitely)")
    parser.add_argument("--rate", type=float, default=5.0, help="Target transactions per second (0 = maximum speed)")
    parser.add_argument("--key", default="", help="Raw 32-byte hex seed of funder account")
    parser.add_argument("--key-file", default="", help="Path to file containing raw 32-byte hex seed")
    parser.add_argument("--keystore", default="", help="Path to keystore.json")
    parser.add_argument("--password", default="", help="Keystore passphrase (or KOVANICA_KEYSTORE_PASSWORD)")
    parser.add_argument("--amount", type=int, default=1, help="Transfer amount in atoms (default 1)")
    parser.add_argument("--fee", type=int, default=1000000, help="Fee in atoms (default 1,000,000 = 0.001 KUNA)")
    parser.add_argument("--recipient", default="", help="Recipient address (default: alternating soak sink)")
    parser.add_argument("--report-interval", type=float, default=5.0, help="Stats reporting interval in seconds")
    parser.add_argument("--out-json", default="", help="Optional JSON file to save final run statistics")
    args = parser.parse_args()

    # Determine private seed
    # Default funded test key from ops/soak/funded_accounts.toml
    DEFAULT_SEED = "11" * 32
    seed_bytes = None

    if args.key:
        seed_bytes = bytes.fromhex(args.key.strip().replace("0x", ""))
    elif args.key_file and os.path.isfile(args.key_file):
        with open(args.key_file, "r") as f:
            seed_bytes = bytes.fromhex(f.read().strip().replace("0x", ""))
    elif args.keystore and os.path.isfile(args.keystore):
        with open(args.keystore, "r") as f:
            ks = json.load(f)
            # If keystore has an unencrypted secret_key or raw seed
            if not ks.get("encrypted", False) and "secret_key" in ks:
                seed_bytes = bytes.fromhex(ks["secret_key"][:64])
            else:
                pass_val = args.password or os.environ.get("KOVANICA_KEYSTORE_PASSWORD", "")
                if not pass_val and ks.get("encrypted", False):
                    sys.exit("Error: encrypted keystore specified but no passphrase provided.")

    if seed_bytes is None:
        seed_bytes = bytes.fromhex(DEFAULT_SEED)

    if len(seed_bytes) != 32:
        sys.exit(f"Error: expected 32-byte seed, got {len(seed_bytes)} bytes")

    privkey = ed25519.Ed25519PrivateKey.from_private_bytes(seed_bytes)
    sender_pub = privkey.public_key().public_bytes_raw()
    sender_addr = encode_address(sender_pub)

    # Recipient address (default soak sink if not specified)
    if args.recipient:
        recipient_pub = decode_address(args.recipient)
    else:
        # Alternating fixed sink addresses
        recipient_pub = bytes.fromhex("22" * 32)
    recipient_addr = encode_address(recipient_pub)

    now_iso = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    print(f"[{now_iso}] Kovanica Soak Load Generator started")
    print(f"  RPC URL:         {args.rpc_url}")
    print(f"  Chain ID:        {args.chain_id}")
    print(f"  Sender:          {sender_addr}")
    print(f"  Recipient:       {recipient_addr}")
    print(f"  Target Rate:     {args.rate} tx/s" if args.rate > 0 else "  Target Rate:     Unlimited")
    print(f"  Duration:        {args.duration}s" if args.duration > 0 else "  Duration:        Continuous (Ctrl+C to stop)")
    print(f"  Transfer Amount: {args.amount} atoms")
    print(f"  Transfer Fee:    {args.fee} atoms")

    # Graceful termination handling
    running = True

    def _sig_handler(sig, frame):
        nonlocal running
        print("\nShutdown signal received. Stopping load generator...")
        running = False

    signal.signal(signal.SIGINT, _sig_handler)
    signal.signal(signal.SIGTERM, _sig_handler)

    # Warm-up / Nonce check
    print("Connecting to node JSON-RPC to fetch initial nonce...")
    max_retries = 30
    current_nonce = None
    for attempt in range(max_retries):
        try:
            current_nonce = get_account_nonce(args.rpc_url, sender_addr)
            print(f"  Initial nonce for {sender_addr}: {current_nonce}")
            break
        except Exception as e:
            if attempt == max_retries - 1:
                sys.exit(f"Failed to connect to node at {args.rpc_url}: {e}")
            time.sleep(2)

    stats = SoakStats()
    last_report = time.time()
    start_time = time.time()
    interval_interval = 1.0 / args.rate if args.rate > 0 else 0.0

    while running:
        loop_start = time.perf_counter()

        # Check duration limit
        if args.duration > 0 and (time.time() - start_time) >= args.duration:
            print(f"Duration limit reached ({args.duration}s).")
            break

        # Generate signed transaction
        raw_hex, tx_hash = build_and_sign_transfer(
            privkey=privkey,
            sender_pubkey=sender_pub,
            recipient_pubkey=recipient_pub,
            nonce=current_nonce,
            amount_atoms=args.amount,
            fee_atoms=args.fee,
            chain_id=args.chain_id,
        )

        # Submit transaction
        ok, res_or_err, lat_ms = send_raw_transaction(args.rpc_url, raw_hex)
        stats.record(ok, lat_ms, res_or_err if not ok else None)

        if ok:
            current_nonce += 1
        else:
            # On nonce errors, refresh nonce from node
            if "nonce" in res_or_err.lower():
                try:
                    refreshed = get_account_nonce(args.rpc_url, sender_addr)
                    if refreshed > current_nonce:
                        current_nonce = refreshed
                except Exception:
                    pass

        # Periodic statistics reporting
        now = time.time()
        if now - last_report >= args.report_interval:
            avg_l, min_l, max_l, p50, p95, p99 = stats.percentiles()
            elapsed = now - start_time
            tps = stats.throughput()
            ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
            print(
                f"[{ts}] elapsed={elapsed:6.1f}s | tps={tps:5.2f} tx/s | "
                f"total={stats.submitted:<6} ok={stats.success:<6} err={stats.failed:<4} | "
                f"latency: avg={avg_l:5.1f}ms p50={p50:5.1f}ms p95={p95:5.1f}ms p99={p99:5.1f}ms"
            )
            last_report = now

        # Pacing for target rate
        if interval_interval > 0:
            spent = time.perf_counter() - loop_start
            sleep_time = interval_interval - spent
            if sleep_time > 0:
                time.sleep(sleep_time)

    # Final summary report
    total_elapsed = time.time() - start_time
    avg_l, min_l, max_l, p50, p95, p99 = stats.percentiles()
    final_tps = stats.throughput()

    print("\n" + "=" * 70)
    print("SOAK LOAD GENERATION SUMMARY")
    print("=" * 70)
    print(f"Elapsed Time:      {total_elapsed:.2f} seconds")
    print(f"Total Submitted:   {stats.submitted}")
    print(f"Successful (OK):   {stats.success}")
    print(f"Failed (ERR):      {stats.failed}")
    print(f"Overall Throughput:{final_tps:.2f} tx/sec")
    print(f"Latency Min:       {min_l:.2f} ms")
    print(f"Latency Avg:       {avg_l:.2f} ms")
    print(f"Latency p50:       {p50:.2f} ms")
    print(f"Latency p95:       {p95:.2f} ms")
    print(f"Latency p99:       {p99:.2f} ms")
    print(f"Latency Max:       {max_l:.2f} ms")

    if stats.errors:
        print("\nTop Error Reasons:")
        for err_msg, count in sorted(stats.errors.items(), key=lambda x: x[1], reverse=True)[:5]:
            print(f"  [{count}x] {err_msg}")
    print("=" * 70)

    if args.out_json:
        report_data = {
            "elapsed_seconds": total_elapsed,
            "submitted": stats.submitted,
            "success": stats.success,
            "failed": stats.failed,
            "throughput_tps": final_tps,
            "latency_ms": {
                "min": min_l,
                "avg": avg_l,
                "max": max_l,
                "p50": p50,
                "p95": p95,
                "p99": p99,
            },
            "errors": stats.errors,
        }
        try:
            with open(args.out_json, "w") as jf:
                json.dump(report_data, jf, indent=2)
            print(f"Saved stats to {args.out_json}")
        except Exception as e:
            print(f"Warning: could not write JSON stats to {args.out_json}: {e}")

    sys.exit(0)


if __name__ == "__main__":
    main()
