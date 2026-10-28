/**
 * Thin wrapper over `@kovanica/api-client` for the comms app (desktop webview).
 *
 * Chain head:
 *  - `kovanica_blockNumber` returns the latest committed leader height — see
 *    {@link fetchChainHead}.
 *  - the `newHeads` WebSocket subscription pushes a block per accepted block —
 *    see {@link subscribeNewHeads}.
 *
 * RPC methods actually exposed by `@kovanica/api-client` (do not invent others):
 *   `kovanica_blockNumber`, `kovanica_getBlockByHash`, `kovanica_getBlockByNumber`,
 *   `kovanica_getBalance`, `kovanica_getValidators`, `kovanica_sendRawTransaction`,
 *   `kovanica_getPendingTransactions`, `kovanica_estimateFee`, `kovanica_getNonce`,
 *   `kovanica_getStake`, `kovanica_getRewards`, `kovanica_getTransactionByHash`,
 *   `kovanica_getTransactionReceipt`, `kovanica_getLeaderSchedule`, `kovanica_getCommittee`
 *   and `GET /health`.
 *
 * WebSocket subscriptions accepted by `kovanica-rpc/src/subscriptions.rs`:
 *   `newHeads`, `newCommittedLeader`, `pendingTransactions`, `logs`,
 *   `newSealedHeader`.
 */

import { KovanicaRpcClient } from "@kovanica/api-client";

export {
  KovanicaRpcClient,
  subscribeNewHeads,
  subscriptionHead,
} from "@kovanica/api-client";
export type { BlockResponse, HealthStatus } from "@kovanica/api-client";

/** Loopback node default. The URL must include `/rpc`. */
export const LOCAL_RPC_URL = "http://127.0.0.1:8545/rpc";

/** Live public testnet (chain 4). The URL must include `/rpc`. */
export const TESTNET_RPC_URL = "https://rpc-testnet.kovanica.online/rpc";

/** One selectable endpoint. `chainId` is what the network signs with. */
export interface NetworkPreset {
  id: "testnet" | "local";
  label: string;
  chainId: number;
  rpcUrl: string;
}

/** Built-in endpoints, live testnet first (mirrors `packages/console/src/network.ts`). */
export const NETWORK_PRESETS: readonly NetworkPreset[] = [
  { id: "testnet", label: "Testnet (chain 4)", chainId: 4, rpcUrl: TESTNET_RPC_URL },
  { id: "local", label: "Local", chainId: 1337, rpcUrl: LOCAL_RPC_URL },
];

/** Subscription names accepted by the node (see `subscriptions.rs`). */
export const SUBSCRIPTION_METHODS = [
  "newHeads",
  "newCommittedLeader",
  "pendingTransactions",
  "logs",
  "newSealedHeader",
] as const;

export type SubscriptionMethod = (typeof SUBSCRIPTION_METHODS)[number];

/** Parsed chain head. */
export interface ChainHead {
  /** Latest committed leader height from `kovanica_blockNumber`. */
  height: bigint;
  /** `height` as a `0x`-prefixed hex quantity, for display. */
  heightHex: string;
}

/** Build a client for a JSON-RPC endpoint. Defaults to a loopback node. */
export function createRpcClient(url: string = LOCAL_RPC_URL): KovanicaRpcClient {
  return new KovanicaRpcClient(url);
}

/** Read the latest committed leader height (`kovanica_blockNumber`). */
export async function fetchChainHead(client: KovanicaRpcClient): Promise<ChainHead> {
  const height = await client.blockNumber();
  return { height, heightHex: `0x${height.toString(16)}` };
}
