---
title: "kovanica-mobile"
category: 50-Components
source: mobile/README.md
synced: 2026-09-26
---
# kovanica-mobile

Kovanica light-node mobile clients (Android + future iOS).  
**Distinct from kovanica-wallet** — this is the light node, not the full wallet.

## Structure
- `android/` — Android light node client (Kotlin)
- `ios/` — iOS light node (planned)

## Development
```bash
cd android && ./gradlew build
Related
•	Protocol: kovanica-protocol
•	Wallet: kovanica-wallet
•	Web: kovanica-web
License
MIT OR Apache-2.0