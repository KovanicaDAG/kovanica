# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview
> Requested team: Full multi-agent team

Implement Hardware Wallet integration (Ledger and Trezor) for the Kovanica web wallet simultaneously using an abstract hardware provider interface. Include the UI for connecting devices and signing transactions, and implement a Mock Hardware Wallet provider so the CI pipeline can automatically test the flows without physical USB devices.

Working directory: /root/kovanica-protocol/web
Integrity mode: development

## Requirements

### R1. Abstract Hardware Provider & Mock
Create a standardized interface for hardware wallets (connect, export public key, sign transaction). Implement a `MockHardwareProvider` that conforms to this interface to enable end-to-end testing without physical devices.

### R2. Ledger & Trezor Implementations
Implement the interface for Ledger (via `@ledgerhq/hw-transport-webusb`) and Trezor (via `@trezor/connect-web`).

### R3. Wallet UI Integration
Update the wallet interface to allow users to connect a hardware wallet. Modify the transaction send flow so that it delegates the signing process to the connected hardware device, displaying appropriate "waiting for confirmation" states.

## Acceptance Criteria

### CI & Type Safety
- [ ] `npm run typecheck` (or equivalent TypeScript compiler check) passes successfully across the `web` directory without errors.

### Programmatic Verification
- [ ] An automated test or Node script demonstrates the `MockHardwareProvider` successfully exporting a public key and signing a mock transaction payload.

---
*Next: when approved → delegate via invoke_subagent (see Delegation Protocol)*
