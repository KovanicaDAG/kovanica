# kovanica-ledger-app

> **Hardware wallet harness for Kovanica** — Ledger (WebHID) and Trezor (WebUSB) support for the Kovanica Protocol. Versioned harness with Makefile-driven builds.

---

## Status

🚧 **Work in Progress** — App loading, BIP-44 path, and Kovanica-specific APDU commands under development.

---

## Target Hardware

| Device | Transport | Status |
|--------|-----------|--------|
| **Ledger Nano S / S+ / X / Stax** | WebHID (browser) / Speculos (dev) | 🚧 Dev |
| **Trezor Model One / Safe 3 / Safe 5** | WebUSB (browser) / Trezor Emulator | 🚧 Dev |

---

## Architecture

```
ledger-app/
├── app/                    # Ledger app (C, BOLOS SDK)
│   ├── src/
│   │   ├── main.c          # Entry point, APDU dispatch
│   │   ├── kovanica.c      # Kovanica-specific commands
│   │   ├── bip32.c         # BIP-44 path derivation (m/44'/11111'/0'/0/0)
│   │   ├── ed25519.c       # Ed25519 signing (via cx_ecfp)
│   │   └── ui.c            # Device screen flows (review address, sign tx)
│   ├── Makefile            # Build for Nano S/S+/X/Stax
│   └── icon.gif            # App icon
├── trezor/                 # Trezor app (Python, trezor-firmware)
│   ├── kovanica/           # Coin definition
│   │   ├── __init__.py
│   │   ├── messages.proto  # Kovanica-specific protobuf messages
│   │   ├── client.py       # Python client helper
│   │   └── signing.py      # Transaction signing logic
│   └── Makefile
├── web/                    # Browser integration (WebHID/WebUSB)
│   ├── ledger.ts           # Ledger transport + APDU helpers
│   ├── trezor.ts           # Trezor transport + protobuf helpers
│   └── kovanica.ts         # High-level Kovanica API (derive, sign, getAddress)
├── Makefile                # Top-level build orchestration
└── tests/                  # Integration tests (Speculos, Trezor Emulator)
```

---

## BIP-44 Path

```
m / 44' / 11111' / account' / change / address_index
```

| Component | Value | Note |
|-----------|-------|------|
| Purpose | `44'` | BIP-44 |
| Coin Type | `11111'` | Registered for Kovanica (KVNC) |
| Account | `0'` | First account |
| Change | `0` | External (receive) |
| Address Index | `0` | First address |

> **Coin type 11111** — Registered in [SLIP-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md) for Kovanica.

---

## APDU Commands (Ledger)

| CLA | INS | P1 | P2 | Name | Description |
|-----|-----|----|----|------|-------------|
| `0xE0` | `0x01` | `0x00` | `0x00` | `GET_VERSION` | Returns app version + Kovanica protocol version |
| `0xE0` | `0x02` | `0x00` | `0x00` | `GET_ADDRESS` | Derive + return `kvnc…dag` address (P1=0x01: confirm on device) |
| `0xE0` | `0x03` | `0x00` | `0x00` | `SIGN_TX` | Sign transaction (sighash provided by host) |
| `0xE0` | `0x04` | `0x00` | `0x00` | `GET_PUBLIC_KEY` | Return Ed25519 public key for path |

---

## Build (Ledger)

```bash
# Requires: BOLOS SDK, clang, ledgerblue (for loading)

# Build for Nano S
make -C app TARGET=NANOS

# Build for Nano S+
make -C app TARGET=NANOSP

# Build for Nano X
make -C app TARGET=NANOX

# Build for Stax
make -C app TARGET=STAX

# Load to device (requires ledgerblue)
make -C app load TARGET=NANOS
```

### Speculos (Emulator)

```bash
# Run Speculos Docker
docker run --rm -it -p 5000:5000 --cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
  ghcr.io/ledgerhq/speculos:latest --model nanos --display headless --api-port 5000 ./app/bin/app.elf

# Test with Python
python -m ledgerwallet --apdu --host 127.0.0.1 --port 5000 get_version
```

---

## Build (Trezor)

```bash
# Requires: trezor-firmware repo, Python 3.10+, protobuf compiler

cd trezor
make build          # Build firmware with Kovanica coin
make test           # Run pytest tests
make emulate        # Start Trezor Emulator
```

---

## Web Integration (Browser)

```typescript
import { LedgerKovanica } from './web/ledger';
import { TrezorKovanica } from './web/trezor';

// Ledger (WebHID)
const ledger = new LedgerKovanica();
await ledger.connect();                    // Request device permission
const address = await ledger.getAddress(); // Derive m/44'/11111'/0'/0/0
const signature = await ledger.signTx(sighash); // Sign 32-byte sighash

// Trezor (WebUSB)
const trezor = new TrezorKovanica();
await trezor.connect();
const address = await trezor.getAddress();
const signature = await trezor.signTx(sighash);
```

---

## Transaction Signing Flow

1. **Host** (wallet/node) builds unsigned transaction
2. **Host** computes sighash (BLAKE3 of witness-free encoding)
3. **Host** sends sighash to hardware device via APDU/WebHID/WebUSB
4. **Device** displays transaction details (amount, recipient, fee) for user confirmation
5. **User** confirms on device
6. **Device** signs sighash with Ed25519 private key (never leaves device)
7. **Device** returns 64-byte signature to host
8. **Host** submits signed transaction to node via `/api/submit`

---

## Security

- **Private keys never leave the device** — Ed25519 signing happens inside Secure Element (Ledger) or STM32 (Trezor)
- **Transaction details verified on device screen** — user confirms amount, recipient, fee
- **No blind signing** — full transaction parsed and displayed
- **BOLOS isolation** — Ledger apps run in isolated memory regions
- **`#![forbid(unsafe_code)]`** enforced in Rust host libraries

---

## Testing

```bash
# Ledger: Speculos emulator
make test-ledger

# Trezor: Emulator
make test-trezor

# Web: Playwright E2E (requires physical device or CI with USB passthrough)
make test-web
```

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger (sighash spec) |
| [kovanica-web](https://github.com/KovanicaDAG/kovanica-web) | Web wallet (integrates hardware wallet flow) |
| [kovanica-sdk](https://github.com/KovanicaDAG/kovanica-sdk) | SDK (transaction building, sighash) |
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Node (verifies hardware-signed transactions) |

---

## License

**MIT OR Apache-2.0** — Ledger app: **Apache-2.0** (BOLOS SDK requirement); Trezor app: **MIT**; Web integration: **MIT OR Apache-2.0**.