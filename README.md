# kovanica

Meta-directory — Kovanica DAG protocol ecosystem. 
**Not a git repository itself (tracked files only).** Each subdirectory is an independent git repo. `./dev.sh` clones, pulls, provisions toolchains and installs deps on whatever device you're on.

## Quick start on a new device

```sh
git clone https://github.com/KovanicaDAG/kovanica.git
cd kovanica
./dev.sh              # clone/pull all repos, toolchains, deps, doctor report
./dev.sh --status     # sync + report only, no installs
```

## Repositories

|||| Repo                | Purpose                                      | Remote                          ||
||||---------------------|----------------------------------------------|---------------------------------||
|||| kovanica-protocol   | Internal monorepo (dag/state/node/cli/ffi)   | https://github.com/KovanicaDAG/kovanica-protocol   ||
|||| kovanica-node       | Public release snapshot for operators        | https://github.com/KovanicaDAG/kovanica-node       ||
|||| kovanica-wallet     | Wallet apps + browser extension              | https://github.com/KovanicaDAG/kovanica-wallet     ||
|||| kovanica-web        | Web app + deploy                             | https://github.com/KovanicaDAG/kovanica-web        ||
|||| kovanica-mobile     | Light-node mobile clients                    | https://github.com/KovanicaDAG/kovanica-mobile     ||
|||| kovanica-agent      | RAG agent service (Python)                   | https://github.com/KovanicaDAG/kovanica-agent      ||
|||| kovanica-installer  | Installer scripts (30+ platforms)            | https://github.com/KovanicaDAG/kovanica-installer  ||
|||| kovanica-brain-vault| Canonical knowledge base (Obsidian, layers 00-06) | https://github.com/KovanicaDAG/kovanica-brain-vault ||
|||| kovanica-data       | Runtime data (not versioned)                 | —                               ||

## Getting Started

For users and developers looking to interact with the Kovanica Protocol, we provide detailed how-to guides:

- [How to Mine KVNC](protocol/HOWTO_MINE.md) - CPU mining instructions for testnet and mainnet preparation
- [How to Run a Light Node](protocol/HOWTO_LIGHT_NODE.md) - Run a resource-efficient node for wallet usage and network participation
- [How to Get KVNC Tokens](protocol/HOWTO_GET_KVNC.md) - Acquire testnet KVNC via faucet, mining, or peer-to-peer transfer

These guides are located in the `protocol/` directory (component repos consolidated in the monorepo) and cover:
- Testnet operations (faucet, mining, transactions)
- Node operation (full node, light node, mining setup)
- Wallet usage and address management
- Network configuration and troubleshooting

## Release Flow

`node` is periodically synced from `protocol` crates when a new public version is tagged. 
**Not auto-synced** — always diff before push.

## Contributing

Each component maintains its own contribution guidelines. Please check the respective README files in each component for specific instructions on contributing to that component.

For protocol-level changes, refer to `protocol/` and its `AGENTS.md` file for development conventions and workflow.