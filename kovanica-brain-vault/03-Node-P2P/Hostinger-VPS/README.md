# Hostinger-VPS

Deployment, operations, and autonomous updating for the Hostinger VPS
running **kovanica-protocol**.

## Autonomous updating

Guide for keeping the deployment up to date automatically:
pull → build → restart → verify → rollback.

> Status: template/playbook. Paths, service names, and branches are
> placeholders — fill in the real values before relying on this.
> Facts about the protocol itself live in `/root/kovanica-protocol`.

## 1. Layout on the server

```
/opt/kovanica/                 # deploy root (adjust to your setup)
  kovanica-protocol/           # clone of the repo
  releases/                    # optional: built artifacts per commit
  current -> releases/<sha>    # symlink switched by the updater
  logs/
    update.log
```

## 2. Update script

Create `/opt/kovanica/update.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

REPO=/opt/kovanica/kovanica-protocol
SERVICE=kovanica-node        # systemd unit name (placeholder)
BRANCH=main
LOG=/opt/kovanica/logs/update.log

exec >>"$LOG" 2>&1
echo "=== $(date -Is) update start ==="

cd "$REPO"
OLD_SHA=$(git rev-parse --short HEAD)

git fetch origin "$BRANCH"
if [ "$(git rev-parse HEAD)" = "$(git rev-parse "origin/$BRANCH")" ]; then
  echo "no changes; exiting"
  exit 0
fi

git reset --hard "origin/$BRANCH"
NEW_SHA=$(git rev-parse --short HEAD)
echo "updating $OLD_SHA -> $NEW_SHA"

# Build step — adjust to the project's actual build command
cargo build --release

systemctl restart "$SERVICE"

# Health check: give the service time to start, then verify
sleep 5
if systemctl is-active --quiet "$SERVICE"; then
  echo "update OK at $NEW_SHA"
else
  echo "service failed after update; rolling back to $OLD_SHA"
  git reset --hard "$OLD_SHA"
  cargo build --release
  systemctl restart "$SERVICE"
  echo "rollback complete"
fi
echo "=== $(date -Is) update end ==="
```

Make it executable:

```bash
chmod +x /opt/kovanica/update.sh
mkdir -p /opt/kovanica/logs
```

## 3. Scheduling — pick one

### Option A: cron

```cron
*/15 * * * * /opt/kovanica/update.sh
```

Install with `crontab -e` as the user that owns the deploy.

### Option B: systemd timer (preferred)

`/etc/systemd/system/kovanica-update.service`:

```ini
[Unit]
Description=kovanica autonomous updater

[Service]
Type=oneshot
ExecStart=/opt/kovanica/update.sh
```

`/etc/systemd/system/kovanica-update.timer`:

```ini
[Unit]
Description=Run kovanica updater every 15 minutes

[Timer]
OnBootSec=2min
OnUnitActiveSec=15min

[Install]
WantedBy=timers.target
```

Enable:

```bash
systemctl daemon-reload
systemctl enable --now kovanica-update.timer
```

## 4. Safety notes

- The script pulls from `origin/main` only; it never pushes.
- `git reset --hard` discards any local edits on the server — keep the
  server clone pristine; config belongs in untracked files or env vars.
- Rollback here is code-level only. If migrations/state changed, add an
  explicit downgrade step before enabling automation.
- Cap log growth: add a logrotate rule for `logs/update.log`.

## 5. Verification checklist

- [ ] `update.sh` runs cleanly by hand once
- [ ] timer/cron fires (`journalctl -u kovanica-update` or grep the log)
- [ ] forced-update test: push a trivial commit, watch it land within one cycle
- [ ] rollback test: break a commit intentionally, confirm revert behavior
