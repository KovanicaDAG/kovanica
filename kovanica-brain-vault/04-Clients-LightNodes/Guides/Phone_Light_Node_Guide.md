# Kovanica on Mobile: The Staked Light Node

Welcome to the true decentralized vision of Kovanica! You can actively secure the network and earn rewards directly from your smartphone—without draining your battery or turning your phone into a space heater.

## The Myth of Phone Mining
**Phones do NOT PoW mine on Kovanica.** 

Traditional Proof-of-Work (PoW) mining requires heavy industrial computation. Trying to hash cryptographic algorithms billions of times per second on a mobile processor will rapidly degrade the battery, overheat the device, and yield virtually zero rewards compared to an ASIC.

Instead, Kovanica uses a revolutionary **Hybrid PoW + VRF-Staked** architecture.

## How it Works: The Algorand/Praos Mechanism
Kovanica separates the concept of "chain selection" from "block admission."
- **Industrial Miners** use PoW to decide *which* chain of blocks is the heaviest and most valid (GHOSTDAG chain-selection).
- **Your Phone** uses its "Stake" (the KOV coins you hold) to win a sortition lottery that decides *who is allowed to produce the next block*.

### The VRF Lottery (Step-by-Step)
1. **The Heartbeat:** Every time a new network tip is announced (multiple times a second), your Kovanica Light Node app wakes up for a fraction of a millisecond.
2. **The Signature:** The app uses your wallet's private key to sign a **Verifiable Random Function (VRF)** over the current network tip data. This is just a standard cryptographic signature—it takes virtually zero computational effort.
3. **The Threshold:** The VRF outputs a pseudo-random hash. The app checks if this hash is below a specific mathematical threshold. This threshold is strictly proportional to how much KOV you have staked. 
   - *If you hold 1% of the staked supply, your threshold is set so that you have a 1% chance of winning the lottery on any given heartbeat.*
4. **The Block:** If your VRF output beats the threshold, congratulations! Your phone instantly bundles pending transactions into a block, attaches the winning VRF proof, and broadcasts the "staked block" to the network. You earn the block reward.
5. **Sleep:** If you don't win, the app immediately goes back to sleep until the next heartbeat.

## Why This Matters
- **Zero Battery Drain:** Signing one signature per heartbeat is the computational equivalent of sending a WhatsApp message. Your battery will not notice.
- **51% Attack Resistance:** Because block admission is tied to decentralized mobile stake, a malicious actor cannot simply buy massive mining farms to take over the network. They would also need to acquire a massive percentage of the staked KOV supply, which is distributed across thousands of user phones.

## Getting Started
1. **Download:** Get the official Kovanica Mobile Wallet & Light Node app from the App Store or Google Play.
2. **Fund:** Deposit KOV into your wallet. This acts as your "Stake". The more you hold, the higher your chances of winning the block lottery.
3. **Activate:** Toggle the "Light Node Staking" button to **ON**.
4. **Relax:** Leave the app running in the background. Ensure your phone's OS doesn't force-close the app to save memory (you may need to adjust your phone's battery optimization settings for the Kovanica app).

By running a light node, you are actively decentralizing the Kovanica network and earning passive yields on your holdings!
