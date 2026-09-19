---
name: rust-zk-proofs
description: Use when working with zero-knowledge proofs in Rust: ZK-SNARKs/STARKs concepts, halo2 for ZK circuits, SP1/zkvm for ZK-verified execution, proof generation/verification, circuit design, and common blockchain ZK use cases.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, blockchain, zk, zkp, snark, stark, halo2, sp1, zkvm, circuit, proof, verification]
    related_skills: [rust-crypto, rust-merkle-structures, rust-crypto-primitives, blockchain-fundamentals]
---

# Rust ZK Proofs

## Overview

Zero-knowledge proofs (ZKPs) allow one party (the prover) to prove to another (the verifier) that a statement is true without revealing the underlying data. In blockchain, ZKPs are used for privacy (hide transaction details), scaling (rollups — prove computation was done correctly), and identity (prove membership without revealing identity).

This skill covers the high-level concepts and the Rust tooling landscape. ZK is a deep field — the goal here is to know what's available and when to reach for each tool, not to implement a ZK protocol from scratch.

## When to Use

- Designing a privacy-preserving transaction (e.g., confidential amounts, stealth addresses)
- Building a ZK rollup (prove state transitions)
- Verifying a computation on-chain without re-executing it
- Building a ZK identity / credential system
- Using an existing ZK framework (halo2, SP1, etc.) to build a circuit

**Don't use for:** consensus rules that don't need ZK, or simple commitments (a hash commitment doesn't need ZK — it's just a hash). Use ZK when you need to prove a statement without revealing the data.

## ZK Proof Systems (Conceptual)

### SNARKs (Succinct Non-Interactive Arguments of Knowledge)

- Small, fast-to-verify proofs.
- Require a trusted setup (or universal setup with some systems).
- Used in: zk-SNARK-based privacy protocols, some rollups (zkSync, Scroll).

### STARKs (Scalable Transparent Arguments of Knowledge)

- No trusted setup (transparent).
- Larger proofs than SNARKs, but fast proving and verification.
- Quantum-resistant (hash-based).
- Used in: StarkNet, some rollups.

### Common Properties

| Property | SNARK | STARK |
|---|---|---|
| Proof size | small (hundreds of bytes) | larger (tens of KB) |
| Verification time | fast | fast |
| Prover time | moderate | fast (parallel) |
| Trusted setup | sometimes required | no |
| Post-quantum | no (elliptic curve-based) | yes (hash-based) |

## halo2 — ZK Circuit Development

```toml
[dependencies]
halo2_proofs = "0.3"
halo2_gadgets = "0.3"
```

halo2 is a ZK-SNARK framework by Zcash, used for building ZK circuits in Rust. It's used in Zcash, Polygon zkEVM, and other projects. halo2 provides a DSL for defining circuits, manages the polynomial commitments, and generates proofs.

### Circuit Concept

A halo2 circuit defines:
- **Cells** — the basic unit of a circuit, holding a value.
- **Gates** — constraints on cells (e.g., `a + b = c`, `a * b = c`, range checks).
- **Regions** — layouts of cells.
- **Constraints** — the polynomial constraints that define the circuit's behavior.

### Minimal Circuit Concept (halo2)

```rust
use halo2_proofs::{circuit::Region, layout::ConstraintSystem, plonk:: synthesize, Circuit, Error, Value};

// Define a simple circuit: prove you know x such that x * x = y (without revealing x)
struct SquareCircuit {
    x: Option<u64>,   // private input (witness)
    y: u64,           // public input (output)
}

impl Circuit for SquareCircuit {
    type Config = MyConfig;   // gate configuration
    type FloorPlanner = SimpleFloorPlanner;

    fn synthesize(&self, config: Self::Config, mut layout: Region<'_, '_>) -> Result<(), Error> {
        // Allocate a private cell for x
        let x = layout.query_cell(|| "x", self.x)?;
        // Allocate a public cell for y
        let y = layout.query_cell(|| "y", Value::known(self.y))?;

        // Constrain: x * x = y
        layout.constrain_equal(x.clone() * x.clone(), y)?;

        Ok(())
    }
}
```

This is conceptual — actual halo2 circuits require more setup (gadgets, column allocation, etc.).

### When to Use halo2

- Building a custom ZK circuit with fine-grained control.
- When you need a SNARK with a specific tradeoff (proof size, verification time).
- When you're building on a chain that uses halo2-based proofs.

### Alternatives to halo2

- **circom** (not Rust, but widely used) — popular for SNARK circuit development, generates R1CS.
- **Noir** (Aztec) — Rust-like DSL for ZK circuits, compiles to various backends.
- **Cairo** (StarkWare) — STARK-based, different model (CPU-like).
- **gnark** (Go) — SNARK circuit framework.

## SP1 / zkVM — ZK-Verified Execution

```toml
[dependencies]
sp1-sdk = "0.3"
```

SP1 (by Succinct) is a zkVM (ZK virtual machine) that lets you write Rust code, compile it to a circuit, and generate a proof that the code was executed correctly. The proof can be verified on-chain or off-chain.

### Concept

```rust
// In your zkVM program (the "guest")
#![no_std]
#![no_main]

sp1_zkvm::declare_main!

fn main() {
    let input = sp1_zkvm::read_vec();
    let result = do_computation(&input);
    sp1_zkvm::write_vec(&result);
}
```

```rust
// In the host (proof generation)
use sp1_sdk::SP1;

let elf = SP1::load_elf("program").unwrap();
let proof = SP1::prove(elf, &inputs).unwrap();
let verified = SP1::verify(proof).unwrap();
```

The proof attests that the program was executed with the given inputs and produced the given outputs, without revealing the inputs (if the verifier doesn't have them).

### Use Cases

- **zkRollups:** prove block execution (state transition) without re-executing on L1.
- **Proof of computation:** prove a computation was done (e.g., a light client proof, an oracle result).
- **Private computation:** prove a result without revealing inputs.

## ZK Use Cases in Blockchain

### Privacy (Confidential Transactions)

Prove that a transaction is valid (inputs sum to outputs, signature is valid) without revealing the amounts or addresses.

**Tools:** Zcash (Orchard, halo2), Tornado Cash-style (SNARKs for deposit/withdrawal proofs), Firo, Zerocash.

**Concept:** commitments to amounts (Pedersen commitments), range proofs (prove amount is in a valid range without revealing it), zero-knowledge proof of signature validity.

### ZK Rollups

Prove that a batch of transactions was executed correctly and the new state is correct, without the L1 re-executing all transactions.

**Tools:** SP1 (zkVM-based), halo2 (custom circuits), StarkNet (Cairo/STARK), zkSync (SNARKs).

**Concept:** the rollup operator posts transactions and a proof. L1 verifies the proof (or a fraud proof in optimistic rollups). The proof covers the state transition.

### Identity and Credentials

Prove that you hold a credential (e.g., you're over 18, you're a accredited investor, you're in a whitelist) without revealing your identity.

**Tools:** ZK-SNARKs/STARKs with credential circuits, Semaphore (group membership proofs), ZK identity protocols.

### Light Client Proofs

Prove that a block/header is part of the chain without downloading the whole chain. Used in bridges, light clients, and cross-chain messaging.

**Concept:** a ZK proof that a header is valid according to the consensus rules, or a Merkle proof of inclusion combined with a consensus proof.

## Proof Generation and Verification

### Proof Generation (Prover)

The prover runs the circuit with the witness (private inputs) and generates a proof. This is computationally expensive (polynomial commitment, FFTs, etc.).

```rust
// Conceptual — actual APIs vary by framework
let proof = halo2::prove(circuit, witness).unwrap();
```

### Proof Verification (Verifier)

The verifier checks the proof against the public inputs and the circuit's verification key. This is fast (polynomial evaluation at a few points).

```rust
let valid = halo2::verify(verification_key, public_inputs, &proof).unwrap();
```

### On-Chain Verification

For proofs verified on-chain (e.g., rollup proofs, privacy proofs), the verifier runs in the chain's VM (EVM, Wasm, SVM). This requires:
- A verifier contract/algorithm that fits in the chain's gas/compute budget.
- The proof format must be compatible with the on-chain verifier.

For EVM chains, SNARK verifiers are typically written in Solidity or use a pre-compile. For Substrate/Wasm chains, the verifier can be a Wasm contract or a pallet.

## Circuit Design Considerations

### What to Put in the Circuit

- **Public inputs:** values the verifier knows (e.g., a nullifier, a commitment, a state root).
- **Private inputs (witness):** values the prover knows but doesn't reveal (e.g., a secret key, an amount, a preimage).
- **Constraints:** the rules the circuit enforces (e.g., amounts balance, signature is valid, the value is in range).

### Common Circuit Primitives

- **Hash functions:** SHA-256, Keccak, Poseidon (ZK-friendly).
- **Signature verification:** Ed25519, ECDSA in circuit (expensive but possible).
- **Range proofs:** prove a value is in a range without revealing it.
- **Commitments:** Pedersen commitments (hiding + binding), hash commitments.
- ** Merkle proofs:** prove inclusion in a Merkle tree (in circuit).
- **Encryption / decryption:** in circuit (for privacy protocols).

### Circuit Efficiency

- **ZK-friendly hashes:** SHA-256 is expensive in circuits. Use Poseidon, MiMC, or other ZK-friendly hashes for circuits that need hashing.
- **Minimize constraints:** each constraint adds to proving time. Keep circuits lean.
- **Lookups:** some systems support lookup tables (e.g., halo2 lookups) to reduce constraint count for operations like range checks or bitwise ops.

## Verification Checklist

- [ ] Can explain the difference between SNARKs and STARKs at a high level
- [ ] Can explain when ZK is needed vs a simple hash commitment
- [ ] Can describe a ZK rollup proof (what it proves, who verifies it, where it runs)
- [ ] Can describe a privacy transaction proof (what's hidden, what's proven)
- [ ] Can write a simple halo2 circuit conceptually (private input, public input, constraint)
- [ ] Can explain what SP1/zkvm does (prove Rust program execution)
- [ ] Understands trusted setup vs transparent systems
- [ ] Understands that ZK proof generation is expensive and verification is fast
- [ ] Knows that on-chain verification has a gas/compute cost and that the verifier must be implemented for the target chain

## Common Pitfalls

1. **Using ZK where a simpler primitive would do.** A hash commitment + reveal is not ZK but is often sufficient for "commit now, reveal later." Don't over-engineer.

2. **Assuming ZK makes data private.** ZK proves a statement without revealing data, but the statement itself might leak information. Design the statement carefully.

3. **Not considering the trusted setup.** Some SNARK systems require a trusted setup — if the setup is compromised, proofs can be forged. Understand the setup model.

4. **Building a circuit that's too large.** Large circuits take longer to prove and may not fit in the prover's memory. Start small and optimize.

5. **Using SHA-256 in a ZK circuit without considering alternatives.** SHA-256 is expensive in circuits. Use ZK-friendly hashes when possible.

6. **Not designing the public inputs carefully.** Public inputs are part of the statement — they're visible to the verifier. If a public input leaks too much, the ZK property is weakened.

7. **Assuming ZK proofs are free to verify on-chain.** Verification costs gas/computation. zkEVM rollups, for example, require significant gas for proof verification. Plan for it.

8. **Not separating the prover and verifier roles.** The prover generates the proof (expensive), the verifier checks it (cheap). In a rollup, the operator is the prover, the L1 is the verifier.

9. **Building a custom ZK protocol without expertise.** ZK is subtle. Use established protocols and systems rather than designing your own proof system.

10. **Forgetting that ZK proofs are not encrypted.** A ZK proof proves a statement — it doesn't encrypt the data. If the statement includes a value, the value is not revealed, but the proof itself is not an encrypted blob.
