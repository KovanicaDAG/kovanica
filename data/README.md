# kovanica-data

> **Runtime data directory** — Chain state, wallets, snapshots, checkpoints, and logs. **NOT versioned** (gitignored in parent repo).

---

## Purpose

This directory is created at runtime by `kovanica-node` and related tools. It contains:

```
kovanica-data/
├── dag/                    # DAG snapshot (blocks in topological order)
├── ledger/                 # UTXO set snapshots + finality checkpoints
├── snapshots/              # Full state snapshots (Dag + Ledger)
├── checkpoints/            # Finality checkpoints (UTXO set at finality boundary)
├── wallets/                # User wallet data (encrypted seeds, address books)
├── logs/                   # Structured logs (JSON, rotation)
├── metrics/                # Prometheus metrics snapshots
├── p2p/                    # Peer database (DHT routing table, peer scores)
└── network-marker          # File indicating network/genesis (prevents cross-network corruption)
```

---

## Location

| Environment | Default Path |
|-------------|--------------|
| **Linux/macOS (installer)** | `~/kovanica-node/data/` |
| **Linux/macOS (manual)** | `$KOVANICA_DATA` or `./data/` |
| **Windows (installer)** | `%USERPROFILE%\kovanica-node\data\` |
| **Windows (manual)** | `%KOVANICA_DATA%` or `.\data\` |
| **Docker** | `/data/` (volume mount) |
| **VPS (seed/explorer)** | `/root/kovanica-data/` |

---

## Critical Rules

1. **Never commit this directory** — Added to `.gitignore` in all repos
2. **Preserve after first genesis write** — `KOVANICA_DATA` must persist across restarts
3. **One network per directory** — The `network-marker` file prevents accidental cross-network corruption
4. **Backup before upgrades** — `tar -czf backup-$(date +%F).tar.gz kovanica-data/`
5. **Do not manually edit files** — All state managed by `kovanica-node` (replay-log persistence)

---

## Environment Variable

```bash
export KOVANICA_DATA="/path/to/kovanica-data"
```

Used by:
- `kovanica-node` (binary)
- `kovanica-cli` (for local node queries)
- `desktop-app` (embedded node)
- `android-light-node` (not used — light node stores blob in app private dir)

---

## Backup & Restore

```bash
# Backup (node must be stopped)
systemctl --user stop kovanica-node  # if systemd
tar -czf kovanica-data-backup-$(date +%F).tar.gz kovanica-data/

# Restore
tar -xzf kovanica-data-backup-2026-09-26.tar.gz
# Ensure KOVANICA_DATA points to restored directory
```

---

## Disk Usage (Testnet, ~1M blocks)

| Component | Approximate Size |
|-----------|------------------|
| DAG snapshot | ~200 MB |
| Ledger snapshot | ~150 MB |
| Checkpoints | ~50 MB |
| P2P peer DB | ~10 MB |
| Logs (7 days) | ~50 MB |
| **Total** | **~460 MB** |

> With payload pruning (default depth 1000): ~60% reduction

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Creates/manages this data |
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger |
| [kovanica-installer](https://github.com/KovanicaDAG/kovanica-installer) | Sets up default data dir |

---

## License

**MIT OR Apache-2.0** — This directory contains generated data; license applies to the tooling that creates it.