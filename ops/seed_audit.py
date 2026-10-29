#!/usr/bin/env python3
"""Read-only audit of DNS seeds / bootstrap peers. Prints a report."""
import socket, sys

def resolve(host):
    try:
        return socket.gethostbyname(host)
    except Exception as e:
        return f"FAIL: {e}"

def main():
    seeds = sys.argv[1:] or ["seed1.kovanica.example", "seed2.kovanica.example"]
    ok = 0
    for s in seeds:
        ip = resolve(s)
        print(f"{s} -> {ip}")
        if not str(ip).startswith("FAIL"):
            ok += 1
    if ok == 0:
        print("TOTAL FAILURE")
        sys.exit(1)
    print(f"OK {ok}/{len(seeds)}")

if __name__ == "__main__":
    main()
