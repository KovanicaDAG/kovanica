# Kovanica Hash Infrastructure Setup

This document records the automated deployment of the dual-system mining infrastructure for the Kovanica network, enabling both native Blake3 mining and foreign ASIC hashing.

## Overview
The infrastructure consists of two main components running in `/root/kovanica-hash/`:
1. **Native Blake3 Pool (`/root/kovanica-hash/pool`)**
2. **Private Exchange Proxy (`/root/kovanica-hash/proxy`)**

---

## 1. Native Blake3 Pool (Go-based)
A modern, high-performance Go-based Stratum mining pool designed natively for the Blake3 algorithm to aggregate hashrate for CPU/GPU miners.

**Tech Stack:**
- Go (Golang 1.20+)
- `lukechampine/blake3` library for hashing
- PM2 / Systemd for process management

**Configuration:**
- **Source Code:** `/root/kovanica-hash/pool/go-pool/`
- **Build Command:** `go build -o kovanica-pool main.go`
- **Start Command:** `./kovanica-pool` (or via PM2)

---

## 2. Private Exchange Proxy (Hash-for-Kovanica)
A custom "Smart Pool" Stratum proxy to allow incompatible ASICs (SHA-256 / Scrypt) to indirectly mine for the network by routing their hashrate to a public pool, and paying the miners in Kovanica coins via the node RPC.

**Tech Stack:**
- Python 3 with `asyncio` for the Stratum proxy
- SQLite (`shares.db`) for lightweight share accounting
- `requests` library for JSON-RPC communication

**Scripts:**
- **`proxy.py`**: Listens on port `3333`. Forwards hashrate to a designated upstream pool (default: `stratum.antpool.com:3333`). Intercepts `mining.submit` events and records them in `shares.db`.
- **`payout.py`**: Connects to the local Kovanica node daemon (RPC `127.0.0.1:8332`). Tallies unpaid shares per worker (worker name = Kovanica address), calculates the payout (default 0.1 KOV per share), executes the `sendtoaddress` RPC command, and clears the processed shares.

**Next Steps / Maintenance:**
- Update `UPSTREAM_HOST` and `UPSTREAM_PORT` in `proxy.py` if changing the public pool destination.
- Run `payout.py` on a cron schedule to automate the payouts.
- Adjust `PAYOUT_RATE` in `payout.py` to reflect actual profitability vs Kovanica emission.
