# kovanica-hw-signer

Stub interface for external/hardware signers.

## How real devices plug in

Implement the `Signer` trait:

```rust
pub trait Signer {
    fn pubkey(&self) -> [u8; 32];
    fn sign_hash(&self, hash: &[u8; 32]) -> Result<Vec<u8>, SignerError>;
}
```

Adapters for Ledger/Trezor/etc. should be separate crates that depend on this
interface. This crate ships only a software `SoftSigner` for testing; it is
**not** a secure signer and must not be used for real funds.

Wallet-core can enable support via the `hw-signer` feature.