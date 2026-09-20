# Kovanica Ledger App

Ledger hardware wallet application for Kovanica Protocol (KVP-102 multi-asset, Ed25519, BIP-44).

## Architecture

- **Rust** (no_std): Core logic, BIP-32/44, transaction parsing, Ed25519 signing
- **C** (Ledger SDK): UI, APDU dispatch, device communication
- **Build**: `cargo build --target thumbv6m-none-eabi` + `make`

## Requirements

- Ledger Nano S / S Plus / X / Nano S Plus
- Rust `thumbv6m-none-eabi` target
- Ledger SDK (BOLOS)
- `cargo-embed` for flashing

## Build

```bash
# Install Rust target
rustup target add thumbv6m-none-eabi

# Build app
make

# Load to device (dev mode)
make load

# Delete app
make delete
```

## APDU Commands

| CLA | INS | Description |
|-----|-----|-------------|
| 0xE0 | 0x01 | Get version |
| 0xE0 | 0x02 | Get public key (BIP-44 path) |
| 0xE0 | 0x03 | Sign transaction |
| 0xE0 | 0x04 | Get app configuration |
| 0xE0 | 0x05 | Sign message (BIP-137 style) |

## BIP-44 Path

```
m / 44' / 11111' / account' / change / address_index
          ↑
     Coin type (unregistered, 11111 = Kovanica)
```

## Transaction Format

Kovanica transactions are signed as Ed25519 over the sighash (BLAKE3 of witness-free encoding).

The app receives:
1. Transaction sighash (32 bytes)
2. BIP-44 derivation path
3. Optional: full transaction for display verification

Returns: 64-byte Ed25519 signature

## Project Structure

```
src/
├── rust/
│   ├── src/
│   │   ├── lib.rs           # Entry point
│   │   ├── bip32.rs         # BIP-32/44 derivation
│   │   ├── tx.rs            # Transaction parsing
│   │   ├── ed25519.rs       # Ed25519 signing
│   │   └── apdu.rs          # APDU command handlers
│   └── Cargo.toml
├── c/
│   ├── src/
│   │   ├── main.c           # Entry, APDU dispatch
│   │   ├── ui.c             # Ledger UI flows
│   │   ├── kovanica.c       # Kovanica-specific logic
│   │   └── crypto.c         # Ed25519 (from Ledger SDK)
│   └── include/
│       ├── kovanica.h
│       └── ui.h
├── tests/
│   ├── test_vectors.json
│   └── integration_test.py
├── Makefile
├── Cargo.toml
└── README.md
```