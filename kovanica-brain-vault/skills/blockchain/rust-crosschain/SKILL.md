---
name: rust-crosschain
description: Use when building cross-chain bridges, IBC-style messaging, light client verification of remote chains, message passing between chains, and verifying state from a foreign chain in a Rust-based chain/bridge.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, blockchain, cross-chain, bridge, IBC, light-client, messaging, interoperability, verification]
    related_skills: [rust-merkle-structures, rust-indexer-archive, rust-crypto-primitives, rust-consensus-pos]
---

# Rust Cross-Chain

## Overview

Cross-chain interoperability lets chains communicate: transfer assets, pass messages, verify state of a remote chain. The main approaches are:

- **Bridges (lock-and-mint, burn-and-mint, liquidity-based):** lock tokens on chain A, mint/wrap on chain B.
- **IBC (Inter-Blockchain Communication):** Cosmos-style protocol for packet-based messaging between chains with light client verification.
- **Light client verification:** a chain verifies the consensus/state of a remote chain by running a light client of that chain.

This skill covers the Rust components for cross-chain: light clients, message passing, bridge patterns, and verification.

## When to Use

- Building a bridge between two chains
- Implementing IBC-style packet messaging
- Verifying a remote chain's block header / state in a Rust-based chain
- Building a light client of a foreign chain (e.g., Ethereum light client on Substrate)
- Designing cross-chain message verification and relayer infrastructure

**Don't use for:** single-chain development, or consensus for a single chain. Cross-chain is specifically about the interface between chains.

## Bridge Patterns

### Lock-and-Mint (Token Bridge)

```
1. User locks tokens on Chain A (via bridge contract)
2. Relayer detects the lock event
3. Relayer submits a proof to Chain B
4. Chain B mints wrapped tokens to the user (or a representative)
```

```
User → lock(token, amount) on Chain A
Relayer → verify lock event → call mint(token, amount) on Chain B
```

**Security:** the bridge must verify that the lock on Chain A actually happened. This is done via:
- **Trusted relayer:** a trusted party submits the proof (centralized trust).
- **Light client:** Chain B runs a light client of Chain A and verifies the lock event itself (decentralized).

### Burn-and-Mint (Reverse Direction)

```
User burns wrapped tokens on Chain B
Relayer/light client verifies the burn
Chain A unlocks the original tokens
```

### Liquidity-Based Bridge

```
Chain A and Chain B both have liquidity pools of the asset.
Transfer: debit on A, credit on B (using the pools).
No lock/mint — just liquidity movement.
```

**Tradeoffs:**
- Lock-and-mint: trust in the bridge (or light client).
- Liquidity-based: trust in the liquidity (and the bridge operator if centralized).

## Light Client Verification

A light client verifies a remote chain's consensus without downloading the full chain. It verifies block headers and uses Merkle proofs to verify specific state.

### Light Client of an EVM Chain (e.g., Ethereum) on a Rust Chain

```rust
use ethers_core::{types::{Block, Header, Transaction, Hash}};
use sha2::{Sha256, Digest};

pub struct EthereumLightClient {
    // Trusted checkpoint (block hash + height)
    trusted_checkpoint: Checkpoint,
    // Latest verified header
    latest_header: Option<Header>,
}

impl EthereumLightClient {
    pub fn verify_header(&mut self, header: Header, parent_hash: Hash) -> Result<(), Error> {
        // Verify that header.parent_hash matches the known header
        let parent = self.latest_header.as_ref().ok_or(Error::NoTrustedHeader)?;
        if parent.hash() != parent_hash {
            return Err(Error::ParentMismatch);
        }

        // Verify PoW (for Ethereum PoW — pre-merge, or PoS beacon light client)
        self.verify_pow(&header)?;

        // Verify the merkle root (state root, tx root, receipts root) — format only
        // (full validation requires state proofs)

        self.latest_header = Some(header);
        Ok(())
    }

    pub fn verify_transaction_inclusion(
        &self,
        tx: &Transaction,
        block_hash: Hash,
    ) -> Result<(), Error> {
        // Verify tx is in the block's tx trie via Merkle proof
        // This requires the tx's index in the block and a Merkle proof
        let proof = get_tx_merkle_proof(tx, block_hash)?;
        proof.verify(block_hash)?;
        Ok(())
    }
}
```

### Light Client of a Cosmos Chain (IBC)

IBC light clients are defined in the IBC protocol. A chain runs a light client of the remote chain and verifies headers.

```rust
// IBC light client (conceptual)
pub trait IbcLightClient {
    fn verify_header(&self, header: &RemoteHeader) -> Result<(), Error>;
    fn verify_packet_commitment(&self, packet: &Packet, commitment: &CommitmentProof) -> Result<(), Error>;
    fn verify_packet_acknowledgement(&self, ack: &Ack, proof: &Proof) -> Result<(), Error>;
}
```

**IBC flow:**
1. Chain A sends a packet (via IBC) to Chain B.
2. The packet is committed to Chain A's state (a Merkle proof of the packet exists).
3. A relayer submits the packet + commitment proof to Chain B.
4. Chain B's light client of Chain A verifies the proof.
5. Chain B processes the packet.

## IBC-Style Packet Messaging

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IbcPacket {
    pub source_port: String,
    pub source_channel: String,
    pub destination_port: String,
    pub destination_channel: String,
    pub sender: Address,
    pub recipient: Address,
    pub sequence: u64,
    pub timeout_height: Option<Height>,
    pub timeout_timestamp: Option<u64>,
    pub data: Vec<u8>,   // the actual message (proto-encoded, JSON, etc.)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IbcAcknowledgement {
    pub packet_sequence: u64,
    pub success: bool,
    pub data: Vec<u8>,   // response data (if any)
}
```

### Sending a Packet

```rust
pub fn send_packet(packet: IbcPacket) -> Result<(), Error> {
    // 1. Store the packet commitment on the local chain
    let commitment = hash_packet(&packet);
    store_packet_commitment(packet.sequence, &commitment)?;

    // 2. Emit an event / publish the commitment for relayers
    emit_packet_event(packet)?;

    Ok(())
}
```

### Receiving a Packet (on the destination chain)

```rust
pub fn receive_packet(
    packet: IbcPacket,
    commitment_proof: CommitmentProof,
) -> Result<IbcAcknowledgement, Error> {
    // 1. Verify the packet commitment on the source chain via light client
    let commitment = light_client.verify_packet_commitment(&packet, &commitment_proof)?;

    // 2. Verify the sequence (no replay)
    if commitment.sequence != packet.sequence {
        return Err(Error::SequenceMismatch);
    }

    // 3. Process the packet (deliver the message)
    let result = process_packet_message(&packet)?;

    // 4. Create acknowledgement
    let ack = if result.is_ok() {
        IbcAcknowledgement { packet_sequence: packet.sequence, success: true, data: result.unwrap() }
    } else {
        IbcAcknowledgement { packet_sequence: packet.sequence, success: false, data: result.unwrap_err() }
    };

    // 5. Store the acknowledgement commitment (for the source chain to verify)
    store_ack_commitment(ack.clone())?;

    Ok(ack)
}
```

## Relayer

The relayer moves packets between chains. It watches for packets on the source chain, and submits them (with proofs) to the destination chain.

```rust
pub struct Relayer {
    source_client: ChainClient,
    dest_client: ChainClient,
    light_client: LightClient,   // light client of source chain on dest chain
}

impl Relayer {
    pub async fn relay_packets(&mut self) -> Result<(), Error> {
        // 1. Fetch unrelayed packets from source
        let packets = self.source_client.get_unrelayed_packets().await?;

        for packet in packets {
            // 2. Get the commitment proof from source
            let proof = self.source_client.get_packet_commitment_proof(packet.sequence).await?;

            // 3. Submit to destination chain
            self.dest_client.submit_packet(packet, proof).await?;
        }

        Ok(())
    }
}
```

**Relayer trust:** the relayer doesn't need to be trusted for the packet's content — the light client on the destination chain verifies the proof. The relayer just moves data. However, the relayer can be untrusted and still perform its function (it's just a data mover, not a verifier).

## Message Verification

The core of cross-chain security: verifying that a message (packet, event, state) actually happened on the source chain.

### Verification with a Light Client

```rust
pub fn verify_remote_event(
    event: RemoteEvent,
    proof: MerkleProof,
    light_client: &LightClient,
) -> Result<(), Error> {
    // 1. Verify the event's commitment on the remote chain
    let commitment = hash_event(&event);
    light_client.verify_commitment(&commitment, &proof)?;

    // 2. Verify the light client's latest header is valid (trusted checkpoint)
    light_client.verify_latest_header()?;

    Ok(())
}
```

### Verification with a Trusted Relayer (less secure)

```rust
pub fn verify_with_trusted_relayer(event: RemoteEvent, relayer_signature: Signature) -> Result<(), Error> {
    // Verify the relayer's signature on the event
    trusted_relayer.pubkey().verify(&event.hash(), &relayer_signature)?;
    Ok(())
}
```

**Trusted relayer approach:** simpler but requires trust in the relayer. If the relayer is dishonest, it can forge events.

## Cross-Chain State Verification

To verify a specific piece of state on a remote chain (e.g., "does this account have a balance on Chain A?"), the light client verifies a Merkle proof of that state against a verified remote header.

```rust
pub fn verify_remote_balance(
    address: Address,
    balance: u64,
    state_proof: StateProof,
    light_client: &LightClient,
) -> Result<(), Error> {
    // 1. Verify the state proof against the remote chain's state root
    light_client.verify_state_proof(&state_proof)?;

    // 2. Check the state root is from a verified header
    let header = light_client.latest_verified_header()?;
    if state_proof.state_root != header.state_root {
        return Err(Error::StateRootMismatch);
    }

    // 3. Verify the balance in the proof
    let (proof_address, proof_balance) = state_proof.read();
    if proof_address != address || proof_balance != balance {
        return Err(Error::BalanceMismatch);
    }

    Ok(())
}
```

## Bridge Security Considerations

- **Trust model:** light client (decentralized, trustless verification) vs trusted relayer (centralized trust) vs multi-sig (trusted committee).
- **Replay protection:** packets/messages must have unique sequence numbers to prevent replay.
- **Timeout:** packets should have timeouts (height or timestamp) so they don't hang forever if the destination chain doesn't process them.
- **Slashing / accountability:** if the bridge uses a trusted committee, there should be a way to hold them accountable (slashing, governance removal).
- **Light client correctness:** the light client must correctly verify the remote chain's consensus. If the light client has a bug, the bridge can be fooled.

## Verification Checklist

- [ ] Can describe lock-and-mint vs burn-and-mint vs liquidity bridge patterns
- [ ] Can describe the role of a relayer in IBC-style messaging
- [ ] Can describe what a light client verifies (headers, Merkle proofs, state roots)
- [ ] Can implement a simple packet with sequence, timeout, and data
- [ ] Can implement packet sending (store commitment) and receiving (verify commitment, process)
- [ ] Can describe the trust model of a bridge (light client vs trusted relayer vs multi-sig)
- [ ] Can implement replay protection (sequence numbers)
- [ ] Can implement timeouts for packets

## Common Pitfalls

1. **Trusting the relayer without verification.** A relayer can submit false data if the destination chain doesn't verify it via a light client. Always verify.

2. **Not implementing replay protection.** Without sequence numbers or nonces, a packet can be replayed.

3. **Not setting timeouts on packets.** A packet that never gets processed (destination chain down, relayer stalled) can leave the sender waiting indefinitely.

4. **Not verifying the light client's trusted checkpoint.** The light client must have a trusted starting point (a known valid header). If the trusted checkpoint is wrong, the light client can be fooled.

5. **Assuming cross-chain state proofs are simple.** Cross-chain state proofs require a light client of the remote chain, a Merkle proof of the specific state, and verification that the state root is from a verified header. Each step has pitfalls.

6. **Not handling packet acknowledgement.** The source chain needs to know if the packet was processed successfully. Acknowledgement mechanics are part of IBC.

7. **Building a bridge without considering the attack surface.** Bridges are high-value targets. Design security carefully (light client, multi-sig with slashing, audits).

8. **Not considering what happens on a bridge hack.** If the bridge is compromised, funds on both chains are at risk. Design for shutdown/safety (pause mechanism, governance).

9. **Ignoring the relayer's role in latency.** The relayer moves packets. If the relayer is slow or down, packets are delayed. The system should be resilient to relayer downtime.

10. **Not testing with a real light client.** Light client verification is subtle. Test with a real or simulated remote chain to verify the light client works correctly.
