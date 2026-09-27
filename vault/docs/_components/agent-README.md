# kovanica-agent

> **RAG Agent Service for Kovanica Protocol** — Python-based retrieval-augmented generation service for developer assistance, protocol queries, and operational support.

---

## Status

🚧 **Work in Progress** — Core retrieval pipeline operational; protocol-specific tooling under development.

---

## Purpose

Provides an intelligent assistant for Kovanica developers and operators that:
- **Answers protocol questions** from RFCs, specs, and codebase
- **Assists with debugging** consensus forks, P2P issues, mempool eviction
- **Generates boilerplate** for RFC implementation, test patterns, CLI commands
- **Validates configurations** against canonical parameters (RFC-006, PoA, network)
- **Monitors network health** via explorer API + metrics

---

## Architecture

```
kovanica-agent/
├── agent/
│   ├── __init__.py
│   ├── core.py              # Main agent loop, tool orchestration
│   ├── retrieval.py         # Vector search + BM25 over knowledge base
│   ├── tools/               # Agent tools (protocol-specific)
│   │   ├── __init__.py
│   │   ├── consensus.py     # GHOSTDAG, GHOSTDAG-k, reachability queries
│   │   ├── ledger.py        # UTXO, tokenomics, RFC-006 calculations
│   │   ├── p2p.py           # Seed policy, peer scoring, eclipse checks
│   │   ├── rpc.py           # Live API queries (/api/head, /api/bootstrap, etc.)
│   │   ├── config.py        # Env var validation, seed linting
│   │   └── codegen.py       # Rust/TypeScript boilerplate generation
│   ├── memory/              # Conversation memory, session management
│   │   ├── __init__.py
│   │   ├── vector.py        # Embeddings + FAISS/Chroma
│   │   └── graph.py         # Knowledge graph (Neo4j/Kuzu)
│   └── llm/                 # LLM provider abstraction
│       ├── __init__.py
│       ├── openai.py
│       ├── anthropic.py
│       └── local.py         # Ollama/Llama.cpp for air-gapped use
├── knowledge/               # Ingested knowledge base (synced from brain-vault)
│   ├── rfc/                 # RFC markdown + extracted specs
│   ├── specs/               # KVP specs, tokenomics tables
│   ├── code/                # Symbol-indexed Rust/TypeScript source
│   ├── ops/                 # Runbooks, incident reports
│   └── network/             # Live parameters, genesis hashes
├── api/                     # REST/gRPC interface for IDE/editors
│   ├── main.py              # FastAPI server
│   ├── routes/
│   └── schemas/
├── tests/
├── requirements.txt
├── pyproject.toml
└── Dockerfile
```

---

## Knowledge Base Sources

| Source | Sync Method | Update Frequency |
|--------|-------------|------------------|
| `kovanica-brain-vault` (Obsidian) | Git pull + markdown parsing | On-demand / webhook |
| `kovanica-protocol` (Rust) | `cargo doc` + syntect highlighting | Per-commit (CI) |
| `kovanica-node` / `kovanica-web` | GitHub API + OpenAPI specs | Daily |
| Live network (`/api/head`, `/api/bootstrap`) | Polling | Every 30s |
| Prometheus metrics (`/metrics`) | Scrape | Every 15s |

---

## Key Capabilities

### Protocol Queries
```
> "What's the current fee floor at height 2,500,000?"
> "Explain GHOSTDAG k=3 mergeset computation"
> "Show me the RFC-006 emission curve formula"
```

### Debugging Assistance
```
> "My node isn't connecting to seed.kovanica.online:9000"
> "Why was my block rejected with DifficultyMismatch?"
> "Mempool evicted my tx — what's the min fee rate?"
```

### Code Generation
```
> "Generate a kovanica-state test for vault absolute+relative lock"
> "Create a CLI command for HTLC refund"
> "Scaffold an RFC-007 NFT minting transaction builder"
```

### Configuration Validation
```
> "Lint this KOVANICA_PEERS value"
> "Validate my authority set for PoA mainnet"
> "Check if my fee estimation matches RFC-006 floor"
```

---

## Quick Start

```bash
# Install dependencies
pip install -r requirements.txt

# Sync knowledge base (requires brain-vault access)
python -m agent.ingest --source brain-vault

# Run API server
python -m api.main --host 0.0.0.0 --port 8081

# Run CLI agent
python -m agent.core --interactive
```

---

## Configuration

```yaml
# config.yaml
llm:
  provider: anthropic  # openai, anthropic, local
  model: claude-3-5-sonnet-20241022
  temperature: 0.1

retrieval:
  vector_db: chroma
  embedding_model: text-embedding-3-large
  top_k: 10
  bm25_weight: 0.3

knowledge:
  brain_vault_path: ~/kovanica-brain-vault
  protocol_repo_path: ~/kovanica-protocol
  auto_sync: true

network:
  default_explorer: https://explorer.kovanica.online
  bootstrap_seeds:
    - seed.kovanica.online:9000
    - seed2.kovanica.online:9000
```

---

## Deployment

### Docker (Recommended)

```bash
docker build -t kovanica-agent .
docker run -d \
  -p 8081:8081 \
  -v ~/kovanica-brain-vault:/knowledge/brain-vault:ro \
  -v ~/kovanica-protocol:/knowledge/protocol:ro \
  -e ANTHROPIC_API_KEY=... \
  kovanica-agent
```

### Systemd

```ini
# /etc/systemd/system/kovanica-agent.service
[Unit]
Description=Kovanica RAG Agent
After=network.target

[Service]
Type=simple
User=kovanica
WorkingDirectory=/opt/kovanica-agent
ExecStart=/opt/kovanica-agent/.venv/bin/python -m api.main
Restart=on-failure
EnvironmentFile=/opt/kovanica-agent/.env

[Install]
WantedBy=multi-user.target
```

---

## Security

- **No private keys/seeds** — Agent never handles key material
- **Read-only network access** — Only GET `/api/*` and `/metrics`
- **Air-gapped mode** — Local LLM (Ollama) + offline knowledge base
- **Audit logging** — All queries + tool calls logged (no sensitive data)

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-brain-vault](https://github.com/KovanicaDAG/kovanica-brain-vault) | Canonical knowledge base (source of truth) |
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger |
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Node binary (live API source) |
| [kovanica-web](https://github.com/KovanicaDAG/kovanica-web) | Web explorer (metrics source) |

---

## License

**MIT OR Apache-2.0**