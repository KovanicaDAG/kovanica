#!/usr/bin/env bash
# VPS status snapshot setup
# Run this ONCE as root on your VPS (145.223.116.178).
# It creates a tiny script + systemd timer that writes a JSON status
# snapshot into your nginx web root every 5 minutes, at an
# unguessable URL. No new ports are opened; it rides on your
# existing nginx (port 80).
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "Please run as root (e.g. sudo bash setup_vps_status.sh)"
  exit 1
fi

TOKEN="$(openssl rand -hex 16)"

# Try to find nginx's actual docroot; fall back to common defaults.
DOCROOT="$(grep -rhoP '(?<=root\s)[^;]+' /etc/nginx/sites-enabled/ /etc/nginx/conf.d/ 2>/dev/null | head -1 | tr -d ' ')"
if [ -z "${DOCROOT:-}" ]; then
  DOCROOT="$(grep -rhoP '(?<=root\s)[^;]+' /etc/nginx/nginx.conf 2>/dev/null | head -1 | tr -d ' ')"
fi
if [ -z "${DOCROOT:-}" ]; then
  for candidate in /var/www/html /usr/share/nginx/html /var/www; do
    if [ -d "$candidate" ]; then
      DOCROOT="$candidate"
      break
    fi
  done
fi
if [ -z "${DOCROOT:-}" ]; then
  echo "Could not auto-detect your nginx web root."
  echo "Edit this script and set DOCROOT manually near the top, then re-run."
  exit 1
fi

if [ ! -w "$DOCROOT" ]; then
  echo "Web root '$DOCROOT' is not writable. Aborting."
  exit 1
fi

STATUS_FILE="$DOCROOT/status-$TOKEN.json"

mkdir -p /usr/local/bin
cat > /usr/local/bin/vps_status_snapshot.py << 'PYEOF'
import json, subprocess, time, sys, os

def sh(cmd):
    try:
        return subprocess.check_output(cmd, shell=True, stderr=subprocess.DEVNULL, text=True).strip()
    except Exception:
        return None

def cpu_percent():
    try:
        def read():
            with open('/proc/stat') as f:
                parts = f.readline().split()[1:8]
            return [int(x) for x in parts]
        a = read(); time.sleep(1); b = read()
        idle_a = a[3] + a[4]; idle_b = b[3] + b[4]
        total_a = sum(a); total_b = sum(b)
        totald = total_b - total_a; idled = idle_b - idle_a
        return round((totald - idled) / totald * 100, 1) if totald else None
    except Exception:
        return None

def mem():
    try:
        info = {}
        with open('/proc/meminfo') as f:
            for line in f:
                k, v = line.split(':')
                info[k.strip()] = int(v.strip().split()[0])
        total = info['MemTotal']; avail = info.get('MemAvailable', info['MemFree'])
        used = total - avail
        return {"total_mb": round(total/1024), "used_mb": round(used/1024), "percent": round(used/total*100, 1)}
    except Exception:
        return None

def disk():
    try:
        out = sh("df -Pk /")
        line = out.splitlines()[1].split()
        total = int(line[1]); used = int(line[2])
        return {"total_gb": round(total/1024/1024, 1), "used_gb": round(used/1024/1024, 1), "percent": round(used/total*100, 1)}
    except Exception:
        return None

def load_avg():
    try:
        with open('/proc/loadavg') as f:
            return [float(x) for x in f.read().split()[:3]]
    except Exception:
        return None

def uptime():
    return sh("uptime -p")

def top_processes():
    out = sh("ps -eo pid,comm,%mem,%cpu --sort=-%mem --no-headers | head -5")
    if not out:
        return None
    procs = []
    for line in out.splitlines():
        parts = line.split(None, 3)
        if len(parts) == 4:
            procs.append({"pid": parts[0], "name": parts[1], "mem_pct": parts[2], "cpu_pct": parts[3]})
    return procs

def failed_units():
    out = sh("systemctl --failed --no-legend")
    return len(out.splitlines()) if out else 0

def failed_ssh_logins():
    out = sh('journalctl -u ssh -u sshd --since "-24 hours" 2>/dev/null | grep -c "Failed password"')
    if out is None or out == "":
        out = sh('grep -c "Failed password" /var/log/auth.log 2>/dev/null')
    try:
        return int(out)
    except Exception:
        return None

def pending_updates():
    total = sh("apt list --upgradable 2>/dev/null | tail -n +2 | wc -l")
    sec = sh("apt-get -s upgrade 2>/dev/null | grep -ic security")
    try:
        return {"total": int(total) if total else 0, "security": int(sec) if sec else 0}
    except Exception:
        return None

data = {
    "generated_at": sh("date -u +%Y-%m-%dT%H:%M:%SZ"),
    "hostname": sh("hostname"),
    "uptime": uptime(),
    "load_avg": load_avg(),
    "cpu_percent": cpu_percent(),
    "memory": mem(),
    "disk_root": disk(),
    "top_processes": top_processes(),
    "failed_systemd_units": failed_units(),
    "failed_ssh_logins_24h": failed_ssh_logins(),
    "pending_updates": pending_updates(),
}

out_path = sys.argv[1] if len(sys.argv) > 1 else "/tmp/status.json"
tmp_path = out_path + ".tmp"
with open(tmp_path, "w") as f:
    json.dump(data, f, indent=2)
os.replace(tmp_path, out_path)
PYEOF

chmod +x /usr/local/bin/vps_status_snapshot.py

# Generate one immediately so the file exists right away.
python3 /usr/local/bin/vps_status_snapshot.py "$STATUS_FILE"

cat > /etc/systemd/system/vps-status.service << EOF
[Unit]
Description=Generate VPS status snapshot JSON

[Service]
Type=oneshot
ExecStart=/usr/bin/python3 /usr/local/bin/vps_status_snapshot.py $STATUS_FILE
EOF

cat > /etc/systemd/system/vps-status.timer << 'EOF'
[Unit]
Description=Run vps-status every 5 minutes

[Timer]
OnBootSec=1min
OnUnitActiveSec=5min

[Install]
WantedBy=timers.target
EOF

systemctl daemon-reload
systemctl enable --now vps-status.timer

echo ""
echo "=== SETUP COMPLETE ==="
echo "Status URL : http://145.223.116.178/status-$TOKEN.json"
echo "File path  : $STATUS_FILE"
echo "Token      : $TOKEN"
echo ""
echo "Send that URL back to Claude so it can start polling it for your daily updates."
echo "(It's a long random token, but note it IS served over plain http, so treat the"
echo "URL itself as semi-secret. To remove this later: systemctl disable --now vps-status.timer"
echo "&& rm /etc/systemd/system/vps-status.* /usr/local/bin/vps_status_snapshot.py '$STATUS_FILE')"
