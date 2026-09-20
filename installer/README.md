# Kovanica Protocol — 30+ Ways to Install & Run a Node
#
# This directory contains install scripts, deployment configs, and
# platform-specific guides for running a Kovanica BlockDAG node.

## Quick Links

| Method | Time to Run | Requirements |
|--------|-------------|--------------|
| [Universal Installer](install.sh) | 2 min | Linux/macOS, curl |
| [Windows](windows/install.ps1) | 2 min | Windows 10/11 |
| [Docker](docker/) | 1 min | Docker |
| [WSL2](wsl2/) | 2 min | Windows + WSL2 |
| [Termux/Android](termux/) | 2 min | Android + Termux |
| [Raspberry Pi](raspberry-pi/) | 5 min | RPi 3/4/5 |

## All Installation Methods

### Desktop / Laptop

1. **Universal installer** (`install.sh`) — Auto-detects OS, downloads binary or builds from source
2. **Windows PowerShell** (`windows/install.ps1`) — Native Windows, WSL2, or MSYS2 modes
3. **Windows batch** (`windows/install.bat`) — Double-click installer
4. **macOS Homebrew** (`macos/install.sh --homebrew`) — Via Homebrew tap
5. **macOS direct** (`macos/install.sh`) — Pre-built binary for Apple Silicon / Intel
6. **Ubuntu/Debian** (`linux/ubuntu/install.sh`) — apt-based, systemd service
7. **Fedora/RHEL/CentOS** (`linux/fedora/install.sh`) — dnf/yum-based
8. **Arch Linux** (`linux/arch/install.sh`) — pacman, AUR-compatible
9. **Alpine Linux** (`linux/alpine/install.sh`) — Musl, tiny containers
10. **Gentoo** (`linux/gentoo/install.sh`) — From source with ebuild
11. **Void Linux** (`linux/void/install.sh`) — xbps package manager

### Single-Board Computers

12. **Raspberry Pi** (`raspberry-pi/install.sh`) — RPi 3/4/5, ARM, swap setup, GPU optimization
13. **BeagleBone** — Use the ARM Linux script (`linux/ubuntu/install.sh`)
14. **Pine64** — Use the ARM Linux script
15. **ODROID** — Use the ARM Linux script
16. **ESP32** (`esp32/flash.sh`) — Light/SPV node only (headers + filters, BLE wallet)

### Containers & Virtualization

17. **Docker** (`docker/Dockerfile`) — Multi-stage build, health checks
18. **Docker Compose** (`docker/docker-compose.yml`) — Node + Prometheus + Grafana
19. **Kubernetes** (`kubernetes/deploy.sh`) — StatefulSet + Service + Ingress
20. **Nix/NixOS** (`nix/flake.nix`) — Flake with NixOS module for declarative config
21. **NixOS module** — `services.kovanica-node.enable = true;` in configuration.nix
22. **LXC/LXD** — Use Ubuntu container template + `linux/ubuntu/install.sh`
23. **Proxmox** — LXC container or VM + `linux/ubuntu/install.sh`
24. **Vagrant** — `vagrant init ubuntu/jammy64 && vagrant up` + install script

### Cloud Providers

25. **AWS EC2** (`cloud/aws/deploy.sh`) — Auto-detects ARM Graviton, user-data bootstrap
26. **AWS Lambda** — Not applicable (long-running process needed)
27. **GCP Compute Engine** (`cloud/gcp/deploy.sh`) — Debian-based, startup script
28. **Azure VM** (`cloud/azure/deploy.sh`) — Ubuntu, ARM/x86
29. **DigitalOcean** (`cloud/digitalocean/deploy.sh`) — Droplet with user-data
30. **Fly.io** (`cloud/flyio/deploy.sh`) — Container-based, free tier available
31. **Railway** — Push Docker image, auto-deploy
32. **Render** — Blueprint from Docker image
33. **Hetzner Cloud** — Use `cloud/digitalocean/deploy.sh` (similar API)
34. **Oracle Cloud Free Tier** — ARM Ampere instances, use `linux/ubuntu/install.sh`

### Plug-and-Play / Dedicated

35. **USB stick** (`usb-play/usb-builder.sh --target portable`) — Copy to any USB drive
36. **Raspberry Pi SD card** (`usb-play/usb-builder.sh --target pi`) — Boot-and-go image
37. **Pre-built SD image** — Flash RPi OS, run one install command

### Mobile

38. **Android (Termux)** (`termux/install.sh`) — Full node on your phone
39. **Android (Light node app)** — Built-in, via FFI (`android-light-node/`)
40. **iOS (Swift app)** — Via FFI bindings (`kovanica-ffi/build-apple.sh`)

### Enterprise / Ops

41. **Ansible** (`ansible/deploy.yml`) — Multi-server playbook with inventory
42. **Terraform** — Use provider-specific scripts + cloud-init
43. **Puppet/Chef/Salt** — Use the systemd service template

### Embedded / IoT

44. **ESP32** (`esp32/flash.sh`) — SPV light client with BLE wallet
45. **ESP32-S3** — Same as above, more RAM for larger filter cache
46. **Arduino Nano 33 IoT** — Would need significant porting effort (not yet supported)

### Experimental / Fun

47. **Live CD/USB boot** — Build a custom Linux ISO with kovanica-node auto-start
48. **PXE network boot** — Network-boot a headless kovanica node
49. **Retro computing** — Port the SPV client to DOS/Amiga (extreme porting project)
50. **Satellite relay** — Iridium/Satellite link to seed node (theoretical)

## Choosing a Method

```
                    ┌─────────────────┐
                    │  What's your    │
                    │  platform?      │
                    └────────┬────────┘
              ┌──────────────┼──────────────┐
              │              │              │
        ┌─────▼─────┐ ┌─────▼─────┐ ┌─────▼─────┐
        │  Desktop   │ │  Server   │ │  Mobile   │
        │  / Laptop  │ │  / VPS    │ │  / IoT    │
        └─────┬─────┘ └─────┬─────┘ └─────┬─────┘
              │              │              │
     ┌────────┼────────┐     │       ┌──────┼──────┐
     │        │        │     │       │      │      │
  Windows  macOS   Linux  Cloud    Android  iOS  ESP32
  install  install  install deploy  Termux  FFI  SPV
  .ps1     .sh      .sh    .sh     install .sh  flash
```

## System Requirements

### Full Node
- **CPU**: 2+ cores (ARM or x86_64)
- **RAM**: 1GB minimum, 2GB+ recommended
- **Disk**: 10GB+ (grows with chain)
- **Network**: 1Mbps+ (P2P + block gossip)
- **OS**: Linux (any), macOS, Windows (WSL2)

### Light / SPV Node
- **CPU**: 1 core
- **RAM**: 256MB
- **Disk**: 100MB (headers + filters only)
- **Network**: 100kbps
- **OS**: ESP32, Android, iOS, any

### Raspberry Pi
- **Model**: RPi 3B+ / 4 / 5
- **RAM**: 2GB+ (4GB recommended)
- **SD card**: 16GB+ (Class 10 / A2)
- **Power**: Official USB-C adapter (3A for RPi 4)
- **Case**: With heatsink or active cooling recommended

## Configuration Reference

All scripts respect these environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `KOVANICA_DATA` | `~/.kovanica-data` | Chain data directory |
| `KOVANICA_P2P_PORT` | `9000` | P2P listen port |
| `KOVANICA_HTTP_PORT` | `8080` | HTTP API port |
| `KOVANICA_PEERS` | `seed.kovanica.online:9000,...` | Bootstrap peers |
| `KOVANICA_MINE` | `0` | Enable auto-mining |
| `KOVANICA_MINE_SECS` | `60` | Block interval (seconds) |
| `KOVANICA_EXPLORER` | `0` | Enable web explorer |

## Quick Start (Universal)

```bash
# One-liner install
curl -fsSL https://raw.githubusercontent.com/KovanicaDAG/kovanica-protocol/main/kovanica-install/install.sh | bash

# Or build from source
curl -fsSL https://raw.githubusercontent.com/KovanicaDAG/kovanica-protocol/main/kovanica-install/install.sh | bash -s -- --build

# Run
kovanica-node demo    # Test it works
kovanica-node serve   # Start the node
```

## Links

- **Repository**: https://github.com/KovanicaDAG/kovanica-protocol
- **Testnet explorer**: https://explorer.kovanica.online
- **Seed nodes**: seed.kovanica.online:9000, seed3.kovanica.online:9000
- **Docs**: https://github.com/KovanicaDAG/kovanica-protocol#readme
