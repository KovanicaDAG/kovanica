# kovanica-installer

> **One-click installers for Kovanica Node** — 30+ platforms (Linux, macOS, Windows, USB). Used by `kovanica-node` release pipeline.

---

## What It Does

Generates platform-specific installers that:
- Download prebuilt binary from GitHub Releases (or build from source)
- Install to `~/kovanica-node/` (Linux/macOS) or `%USERPROFILE%\kovanica-node\` (Windows)
- Create `run.sh` / `run.cmd` launchers with sensible defaults
- Optionally install systemd user unit (Linux) or LaunchAgent (macOS) for auto-start

---

## Supported Platforms

| Platform | Method | Output |
|----------|--------|--------|
| **Linux (x86_64)** | Shell script | `install.sh` |
| **Linux (aarch64)** | Shell script | `install.sh` |
| **macOS (x86_64)** | Shell script | `install.sh` |
| **macOS (Apple Silicon)** | Shell script | `install.sh` |
| **Windows (x86_64)** | PowerShell | `install.ps1` |
| **USB Stick** | FAT32 + scripts | `scripts/usb/` |

---

## Quick Start (End Users)

### Linux / macOS

```bash
# Interactive install
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash

# With systemd (auto-start on login)
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash -s -- --systemd
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.ps1 | iex
```

### USB Stick (Offline-Friendly)

1. Copy `scripts/usb/` to a FAT32 USB stick
2. On target machine, run `install.sh` (or `install.ps1` on Windows) from the stick
3. Network still required for first build/binary download

---

## For Maintainers

### Release Pipeline

1. Tag a release in `kovanica-node` (e.g., `v0.3.0`)
2. GitHub Actions builds binaries for all targets
3. Attach tarballs to the release:
   - `kovanica-node-x86_64-linux.tar.gz`
   - `kovanica-node-aarch64-linux.tar.gz`
   - `kovanica-node-x86_64-macos.tar.gz`
   - `kovanica-node-aarch64-macos.tar.gz`
4. Installer scripts auto-detect latest release and download matching tarball

### Scripts Structure

```
installer/
├── scripts/
│   ├── install.sh          # Linux/macOS (bash)
│   ├── install.ps1         # Windows (PowerShell)
│   ├── common.sh           # Shared logic (version detection, checksums)
│   └── usb/                # USB stick variant
├── pkg/                    # Packaging helpers (deb, rpm, pkg, msi - future)
└── test/                   # Integration tests (Docker-based)
```

---

## Development

```bash
# Test Linux installer locally
cd scripts
./install.sh --dry-run --version v0.3.0

# Test Windows installer (needs PowerShell 7+)
pwsh install.ps1 -DryRun -Version v0.3.0
```

---

## Environment Variables (Installer)

| Variable | Purpose |
|----------|---------|
| `KOVANICA_INSTALL_DIR` | Override install directory (default: `~/kovanica-node`) |
| `KOVANICA_SKIP_BINARY` | Skip binary download (for testing) |
| `KOVANICA_NO_SYSTEMD` | Skip systemd unit creation |

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Node binary (consumes installer scripts) |
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger |

---

## License

**MIT OR Apache-2.0**