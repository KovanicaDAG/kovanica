# kovanica-installer

This repository contains the Kovanica Protocol installer scripts for easy deployment on Ubuntu/Debian systems.

## Overview

The `kovanica-installer` provides a one-command solution to install and configure a full Kovanica stack, including:
- Node.js runtime
- PostgreSQL database
- nginx web server (as reverse proxy)
- PM2 process manager
- Kovanica web application
- SSL certificates (via Let's Encrypt)
- Firewall configuration (ufw)

## Main Install Script

### `install-kovanica.sh`

The primary installation script that orchestrates the entire setup process.

#### Features
- Interactive and automated modes
- Comprehensive logging to `/var/log/kovanica-installer.log`
- Support for Ubuntu 22.04/24.04 and Debian 12
- Dry-run mode to preview changes
- Force reinstall option
- Complete uninstall capability
- Modular design (though currently all logic resides in the main script)

#### Usage

```bash
# Download and run (recommended)
curl -fsSL https://raw.githubusercontent.com/KovanicaDAG/kovanica-installer/main/install-kovanica.sh | sudo bash

# Or run locally after cloning
sudo ./install-kovanica.sh [options]
```

#### Options

| Option | Description |
|--------|-------------|
| `--domain DOMAIN` | Domain name for SSL (e.g., kovanica.example.com) |
| `--email EMAIL` | Email for Let's Encrypt expiration notices |
| `--app-port PORT` | Backend API port (default: 5000) |
| `--frontend-port PORT` | Frontend development port (default: 3000, loopback only) |
| `--skip-node` | Skip Node.js installation |
| `--skip-postgres` | Skip PostgreSQL installation |
| `--skip-nginx` | Skip nginx installation/configuration |
| `--skip-pm2` | Skip PM2 installation/configuration |
| `--skip-app` | Skip Kovanica application clone/build |
| `--skip-ssl` | Skip SSL certificate setup |
| `--skip-firewall` | Skip firewall configuration |
| `--force` | Force reinstall (overwrite existing installation) |
| `--uninstall` | Uninstall Kovanica completely |
| `--dry-run` | Show what would be done without executing changes |
| `-h, --help` | Show help message |
| `-v, --version` | Show script version |

#### Examples

**Interactive installation:**
```bash
sudo ./install-kovanica.sh
```

**Automated with SSL:**
```bash
sudo ./install-kovanica.sh --domain kovanica.example.com --email admin@example.com
```

**Minimal install (assumes dependencies already installed):**
```bash
sudo ./install-kovanica.sh --skip-node --skip-postgres --skip-nginx --skip-ssl
```

**Dry run to preview changes:**
```bash
./install-kovanica.sh --dry-run
```

**Complete uninstall:**
```bash
sudo ./install-kovanica.sh --uninstall
```

#### Directories Used

- `/opt/kovanica` - Base installation directory
- `/opt/kovanica/app` - Kovanica application code
- `/opt/kovanica/data` - Application data (database uploads, etc.)
- `/opt/kovanica/backups` - Automatic backups
- `/var/log/kovanica-installer.log` - Installation log

#### Services Configured

- `kovanica-backend` (managed by PM2) - Node.js API server
- `nginx` - Reverse proxy serving the application
- `postgresql` - Database service
- `ufw` - Firewall (if not skipped)

#### Environment Configuration

The installer creates a `.env.production` file in the app directory containing:
- Database connection strings
- JWT secrets
- Backup passphrase (critical for restoring backups!)
- Other application settings

**⚠️ IMPORTANT:** The backup passphrase must be stored securely. Without it, encrypted backups cannot be restored.

## Modules (Reference)

The installer is designed with a modular structure, though the current version keeps all logic in the main script for simplicity. The `scripts/modules/` directory is intended for future separation of concerns:

- `01-system-prep.sh` - System updates, packages, user creation
- `02-nodejs.sh` - Node.js installation and configuration
- `03-postgresql.sh` - PostgreSQL setup and database creation
- `04-nginx.sh` - nginx installation, configuration, and site setup
- `05-pm2.sh` - PM2 installation and process configuration
- `06-kovanica-app.sh` - Application cloning, building, and environment setup
- `07-ssl.sh` - Let's Encrypt SSL certificate acquisition and renewal
- `08-firewall.sh` - Firewall (ufw) configuration
- `09-finalize.sh` - Final steps, service starts, and summary

## Requirements

- **Operating System:** Ubuntu 22.04/24.04 or Debian 12
- **Access:** Root or sudo privileges
- **Resources:** Minimum 2GB RAM, 20GB disk space
- **Network:** Ports 80, 443, and the chosen app port must be available
- **Dependencies:** Internet connection for package downloads

## Contributing

Feel free to submit issues or pull requests to improve the installer. Please ensure:
- Backward compatibility is maintained
- Changes are tested on supported OS versions
- Documentation is updated accordingly

## License

See the main Kovanica Protocol license for details.