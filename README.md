# kovanica

Meta-directory — Kovanica DAG protocol ecosystem. 
**Not a git repository itself.** Each subdirectory is an independent git repo.

## Repositories

|||| Repo                | Purpose                                      | Remote                          ||
||||---------------------|----------------------------------------------|---------------------------------||
|||| kovanica-protocol   | Internal monorepo (dag/state/node/cli/ffi)   | https://github.com/KovanicaDAG/kovanica-protocol   ||
|||| kovanica-node       | Public release snapshot for operators        | https://github.com/KovanicaDAG/kovanica-node       ||
|||| kovanica-wallet     | Wallet apps + browser extension              | https://github.com/KovanicaDAG/kovanica-wallet     ||
|||| kovanica-web        | Web app + deploy                             | https://github.com/KovanicaDAG/kovanica-web        ||
|||| kovanica-mobile     | Light-node mobile clients                    | https://github.com/KovanicaDAG/kovanica-mobile     ||
|||| kovanica-agent      | RAG agent service (Python)                   | https://github.com/KovanicaDAG/kovanica-agent      ||
|||| kovanica-installer  | Installer scripts                            | https://github.com/KovanicaDAG/kovanica-installer  ||
|||| kovanica-data       | Runtime data (not versioned)                 | —                               ||

## Getting Started

For users and developers looking to interact with the Kovanica Protocol, we provide detailed how-to guides:

- [How to Mine KVNC](kovanica-protocol/HOWTO_MINE.md) - CPU mining instructions for testnet and mainnet preparation
- [How to Run a Light Node](kovanica-protocol/HOWTO_LIGHT_NODE.md) - Run a resource-efficient node for wallet usage and network participation
- [How to Get KVNC Tokens](kovanica-protocol/HOWTO_GET_KVNC.md) - Acquire testnet KVNC via faucet, mining, or peer-to-peer transfer

These guides are located in the `kovanica-protocol` repository and cover:
- Testnet operations (faucet, mining, transactions)
- Node operation (full node, light node, mining setup)
- Wallet usage and address management
- Network configuration and troubleshooting

## Release Flow

`kovanica-node` is periodically synced from `kovanica-protocol` crates when a new public version is tagged. 
**Not auto-synced** — always diff before push.

## Contributing

Each repository maintains its own contribution guidelines. Please check the respective README files in each repository for specific instructions on contributing to that component.

For protocol-level changes, refer to the `kovanica-protocol` repository and its `AGENTS.md` file for development conventions and workflow.