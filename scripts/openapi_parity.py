#!/usr/bin/env python3
"""Verify that every method in KNOWN_RPC_METHODS in crates/kovanica-rpc/src/lib.rs is documented in crates/kovanica-rpc/src/openapi.rs."""
import os
import re
import sys

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
lib_rs = os.path.join(root, "crates", "kovanica-rpc", "src", "lib.rs")
openapi_rs = os.path.join(root, "crates", "kovanica-rpc", "src", "openapi.rs")

with open(lib_rs, encoding="utf-8") as f:
    lib_src = f.read()

m = re.search(r"const\s+KNOWN_RPC_METHODS\s*:\s*&\[&str\]\s*=\s*&\[(.*?)\];", lib_src, re.S)
if not m:
    print("error: could not locate KNOWN_RPC_METHODS in crates/kovanica-rpc/src/lib.rs", file=sys.stderr)
    sys.exit(1)

known_methods = re.findall(r'"([^"]+)"', m.group(1))

with open(openapi_rs, encoding="utf-8") as f:
    openapi_src = f.read()

missing = [method for method in known_methods if f'"{method}"' not in openapi_src]
if missing:
    print(f"openapi parity FAILED — missing from openapi.rs: {', '.join(missing)}", file=sys.stderr)
    sys.exit(1)

print(f"openapi parity: OK ({len(known_methods)} methods verified)")
