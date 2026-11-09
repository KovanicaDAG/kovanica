# Kovanica Systemd Service Units

Production systemd unit definitions and environment templates for Kovanica node and seed services.

## Unit Files

* **`kovanica-node.service`**: Standard full node and mining worker service.
* **`kovanica-seed@.service`**: Multi-instance template for seed node operators (`systemctl start kovanica-seed@1`).
* **`kovanica.env.example`**: Example environment file template for `/etc/kovanica/kovanica.env`.

## Installation

1. Create system user and data directory:
   ```bash
   sudo useradd --system --home-dir /var/lib/kovanica --shell /usr/sbin/nologin kovanica || true
   sudo install -d -o kovanica -g kovanica /var/lib/kovanica
   sudo install -d -o root -g root -m 0755 /etc/kovanica
   ```

2. Copy unit files to `/etc/systemd/system/`:
   ```bash
   sudo cp ops/systemd/kovanica-node.service /etc/systemd/system/
   sudo cp ops/systemd/kovanica-seed@.service /etc/systemd/system/
   sudo cp ops/systemd/kovanica.env.example /etc/kovanica/kovanica.env
   sudo systemctl daemon-reload
   ```

3. Enable and start:
   ```bash
   sudo systemctl enable --now kovanica-node
   ```

## Live Cutover from Legacy `kvnc-*`

To automate the migration and retirement of legacy `kvnc-*` services and configuration paths:
```bash
sudo bash scripts/cutover-vps-systemd.sh --execute
```
