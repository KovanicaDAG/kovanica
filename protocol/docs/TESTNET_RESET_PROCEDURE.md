# Testnet Reset Procedure (PoA Transition)

**Prerequisite:** Phase 0 complete (PoA-only code on `main`), Phase 1 steps 1.1–1.4 complete.

---

## 1. Pre-Reset Checklist

- [ ] All core crates (`kovanica-dag`, `kovanica-state`, `kovanica-node`, `kovanica-ffi`) on `main` with PoA-only code
- [ ] Real authority keys generated and distributed to 3 operators (see `TESTNET_AUTHORITY_KEYS.md`)
- [ ] Each operator has their `KOVANICA_AUTHORITY_KEY` secret and all have `KOVANICA_AUTHORITIES` public keys
- [ ] `koinca-cli` supports `genesis_poa` and `authority_key` commands
- [ ] Explorer/API updated to show authority set and slot production

---

## 2. Coordinator Actions (Single Point of Execution)

### 2.1 Stop All Nodes
```bash
# On seed2, seed3, seed4, and any other testnet peers
systemctl stop kovanica-seed2
systemctl stop kovanica-seed3
systemctl stop kovanica-seed4
```

### 2.2 Clear Data Directories
```bash
# On each node
rm -rf /var/lib/kovanica-seed2/*
rm -rf /var/lib/kovanica-seed3/*
rm -rf /var/lib/kovanica-seed4/*
```

### 2.3 Update Systemd Units with Real Keys

**seed2 (authority 1):**
```ini
Environment=KOVANICA_AUTHORITIES=b288d2c6b7c5b270bc579ec07e263d43acf02c28534952dcd0a3374a2b458995,e2094f3434e2d1d69c39e7e3f998ccc29dc4d5ce368961619595f039a3ad272e,7d19d93019cd1c9d2fb22a0f46ee8e3233533868cb258250a5296d4b051b1d61
Environment=KOVANICA_AUTHORITY_THRESHOLD=2
Environment=KOVANICA_SLOT_DURATION=3000
Environment=KOVANICA_AUTHORITY_KEY=48fdbcfc774a2bd90190b30895bfea4d6059a9b5649fb51baceda38fec586b55
```

**seed3 (authority 2):**
```ini
Environment=KOVANICA_AUTHORITIES=b288d2c6b7c5b270bc579ec07e263d43acf02c28534952dcd0a3374a2b458995,e2094f3434e2d1d69c39e7e3f998ccc29dc4d5ce368961619595f039a3ad272e,7d19d93019cd1c9d2fb22a0f46ee8e3233533868cb258250a5296d4b051b1d61
Environment=KOVANICA_AUTHORITY_THRESHOLD=2
Environment=KOVANICA_SLOT_DURATION=3000
Environment=KOVANICA_AUTHORITY_KEY=0b9227ebc0e249de5c7d8dcb633dc9e3c463e250ed07ef6a2ca5647ffd7c1c1f
```

**seed4 (authority 3):**
```ini
Environment=KOVANICA_AUTHORITIES=b288d2c6b7c5b270bc579ec07e263d43acf02c28534952dcd0a3374a2b458995,e2094f3434e2d1d69c39e7e3f998ccc29dc4d5ce368961619595f039a3ad272e,7d19d93019cd1c9d2fb22a0f46ee8e3233533868cb258250a5296d4b051b1d61
Environment=KOVANICA_AUTHORITY_THRESHOLD=2
Environment=KOVANICA_SLOT_DURATION=3000
Environment=KOVANICA_AUTHORITY_KEY=8c44b6b3f643c729626ac50ebc4cdb7c83dd196567c41b890e277dd4c27bb99e
```

### 2.4 Start Nodes Sequentially (Coordinator controls order)

```bash
# 1. Start seed2 (authority 1) — creates genesis
systemctl start kovanica-seed2

# Wait for genesis block, verify on explorer
curl http://seed2:8080/api/head | jq '.authority_set'

# 2. Start seed3 (authority 2) — syncs from seed2
systemctl start kovanica-seed3

# 3. Start seed4 (authority 3) — syncs from seed2/seed3
systemctl start kovanica-seed4
```

---

## 3. Post-Reset Verification

### 3.1 Authority Set Verification
```bash
# All nodes should report same authority set
for host in seed2 seed3 seed4; do
  echo "=== $host ==="
  curl -s http://$host:8080/api/head | jq '{authority_set: .authority_set, tip: .selected_tip, height: .blue_score}'
done
```

### 3.2 Slot Production Verification
```bash
# Watch for blocks being produced every ~3s by the scheduled authority
# authority 1 at slots 0, 3, 6...; authority 2 at 1, 4, 7...; authority 3 at 2, 5, 8...
watch -n 3 'curl -s http://seed2:8080/api/head | jq .blue_score'
```

### 3.3 P2P Mesh Verification
```bash
# Each node should have 2 peers
curl -s http://seed2:8080/api/peers | jq '.peers | length'
# Should return 2
```

### 3.4 Light Client / FFI Verification
```bash
# Run FFI test suite against the new testnet
cd protocol && cargo test -p kovanica-ffi --test live_sync_spike
```

---

## 4. Explorer/API Updates

- Update `/api/head` to include `authority_set` and `current_slot`
- Update `/api/bootstrap` with new genesis hash
- Deploy web explorer with PoA status page

---

## 5. Announce

- Post new genesis hash in #testnet channel
- Update docs.kovanica.online with new testnet parameters
- Notify any external operators
