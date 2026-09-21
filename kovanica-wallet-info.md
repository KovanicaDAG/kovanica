# Kovanica Wallet Information

> ⚠️ **SECURITY WARNING**: This file contains private key material. Keep it secure and never share it.

---

## Your Personal Wallet (BIP39 Mnemonic)

**Mnemonic (24 words):**
```
file snack upgrade pulse mesh stage cabbage reflect gym panic defense carry
```

**Derived Address:**
- **kvnc format:** `kvnc1A4XLkrefPBsXLwRH7kcRutGm3pgzrC7zJvAf8uiLLHqgdag`
- **hex format:** `0086a03b23e1fb208533bb4d8eed54be8fca6254fea97ec470895eb618ef57a78f`

**Derivation:**
- BIP39 seed (64 bytes): `b238e35ebbbd5893c310b5d24d7aeb36e3163b1f4cd93934624c7cca9259776ad85d07180f72a91d0d5a43e9d5a89ea22b7ecc69255eb1ed551231913d8da64c`
- Ed25519 private key (first 32 bytes): `b238e35ebbbd5893c310b5d24d7aeb36e3163b1f4cd93934624c7cca9259776a`
- Address = `0x00 || Ed25519 public key` (base58 encoded with `kvnc` prefix, `dag` suffix)

---

## Founder / Seed1 Wallet (Deterministic u64=1)

**Seed:** `1` (u64, little-endian: `0100000000000000` + 24 zero bytes)

**Address:**
- **kvnc format:** `kvnc1EvFUfisEScFuZSqDXagC17m3bpP32B74dseMHtzQ5TNbdag`
- **hex format:** `00cecc1507dc1ddd7295951c290888f095adb9044d1b73d696e6df065d683bd4fc`

**Note:** This is the genesis founder address that received the 200,000 KVNC premine and receives mining rewards on seed1.

---

## Network Info

- **Testnet:** `kovanica-testnet`
- **Explorer:** https://explorer.kovanica.online
- **Faucet:** `curl -X POST https://explorer.kovanica.online/api/faucet -H "Content-Type: application/json" -d '{"address": "kvnc1A4XLkrefPBsXLwRH7kcRutGm3pgzrC7zJvAf8uiLLHqgdag"}'`
- **Genesis hash:** `9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97`

---

## Current Balance

Your wallet (`kvnc1A4XLkrefPBsXLwRH7kcRutGm3pgzrC7zJvAf8uiLLHqgdag`): **0 KVNC** (needs funding via faucet)

---
*Generated: 2026-09-17*