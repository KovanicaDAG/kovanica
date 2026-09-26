---
title: "kovanica-wallet"
category: 50-Components
source: wallet/README.md
synced: 2026-09-26
---
# kovanica-wallet

Kovanica wallet — mobile apps + browser extension.

## Structure
- `android/` — Android wallet (Kotlin)
- `ios/` — iOS wallet (Swift)
- `shared/` — Shared logic
- `extension/` — Browser extension (Vite + TypeScript)

## Development
```bash
cd android && ./gradlew build
cd ios && xcodebuild -scheme KovanicaWallet
cd extension && npm install && npm run dev
Related
•	Source of truth: kovanica-protocol
•	Web: kovanica-web
•	Light node mobile: kovanica-mobile
License
MIT OR Apache-2.0