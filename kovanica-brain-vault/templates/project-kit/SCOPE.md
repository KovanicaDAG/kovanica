# SCOPE — change firewall

Classify project surfaces before non-trivial work. If the task needs a RED
surface, stop and obtain explicit human authorization before changing it.

## GREEN

- Tests
- Documentation
- Isolated implementation inside the task contract

## YELLOW

- Shared interfaces
- Database/storage behavior
- Networking/integration
- Performance-sensitive paths

## RED

- Production deletion or destructive migrations
- Credentials, private keys, secret material
- Consensus/cryptographic primitives
- Irreversible financial or identity effects

## Task-specific overrides

- <surface> — <classification> — <reason>

---

*From MARKDOWN.god: Law 12 — Scope is a boundary, not a suggestion.*
