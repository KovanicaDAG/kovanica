# Welcome to Kovanica: The General User Guide

**Kovanica is a next-generation BlockDAG network designed for speed, true decentralization, and inclusivity.** 

Whether you are here to transact, mine, or just hold, this guide will help you understand and navigate the Kovanica ecosystem.

---

## 1. What makes Kovanica different?

### The BlockDAG vs. The Blockchain
Traditional blockchains (like Bitcoin or Ethereum) process one block at a time. If two blocks are mined simultaneously, they create a "fork," and the network must orphan one of them, wasting time and energy. This forces blockchains to enforce slow block times (10 minutes for Bitcoin) to prevent chaos.

Kovanica uses a **BlockDAG (Directed Acyclic Graph)** powered by the **GHOSTDAG** protocol. Instead of a single straight chain, a BlockDAG looks like a braided web. When multiple blocks are created simultaneously, they don't orphan each other—they reference each other. 
- **The Result:** Multiple blocks can be mined every second without conflict. This allows for near-instant transaction finality and massive transaction throughput (TPS).

### Hybrid Consensus
Security is maintained by two entirely different groups working together:
- **Industrial Miners:** Using Proof-of-Work to secure the history of the DAG.
- **Mobile Stakers:** Using their phones to cryptographically admit new blocks to the DAG without draining their batteries.

---

## 2. Setting Up Your Wallet

To interact with the network, you need a Kovanica Wallet. Your wallet is secured by a **Seed Phrase** (usually 12 or 24 words). *Never share this phrase with anyone. It is the master key to your funds.*

- **Web Wallet:** Fast, browser-based access for quick transactions and hardware wallet integration.
- **Mobile App (Recommended):** Store your coins securely and run a background Light Node to earn passive staking rewards simply by keeping the app open.
- **CLI/Desktop Wallet:** For advanced users, developers, and node operators who want full control over their UTXOs and node RPC connections.

---

## 3. How to Obtain KOV

There is no pre-mine or ICO in true decentralized fashion. You can acquire KOV in three ways:

1. **Earn via Mobile Staking:** Hold KOV in your mobile wallet and turn on the Light Node feature to win block rewards via the VRF lottery.
2. **Earn via Mining:** 
   - **Native Hardware (GPU/CPU):** Connect your PC to our native Blake3 pool.
   - **Alien Hardware (ASICs):** Connect your old Bitcoin/Litecoin ASICs to our Smart Proxy to earn KOV by hashing for the Kovanica Treasury. *(See the Miner Guide for details).*
3. **Exchanges:** Buy, sell, or trade KOV on supported cryptocurrency exchanges.

---

## 4. Transacting on the DAG

### The Transaction Lifecycle
1. **Broadcast:** You hit "Send" in your wallet. The transaction is instantly broadcast to the Kovanica P2P network.
2. **Mempool:** Nodes verify your signature and hold the transaction in memory.
3. **Inclusion:** Within a fraction of a second, a miner or a mobile light node scoops up your transaction and includes it in a new block.
4. **Finality:** Because Kovanica creates multiple blocks per second, your block is rapidly buried under newer blocks that reference it (adding to its "Blue Work"). Within roughly 10 seconds, the transaction is cryptographically finalized and irreversible.

### Viewing Transactions
You can track your transactions on the **Kovanica Block Explorer**. Because it is a DAG, you might notice your transaction is referenced by multiple subsequent blocks simultaneously. This high connectivity is exactly what makes the network so fast and secure.

---
*Welcome to the future of decentralized consensus. Welcome to Kovanica.*
