/**
 * Thin wrapper over `@kvnc/api-client` for the comms app (desktop webview).
 *
 * Chain head:
 *  - `kvnc_blockNumber` returns the latest committed leader height — see
 *    {@link fetchChainHead}.
 *  - the `newHeads` WebSocket subscription pushes a block per accepted block —
 *    see {@link subscribeNewHeads}.
 *
 * RPC methods actually exposed by `@kvnc/api-client` (do not invent others):
 *   `kvnc_blockNumber`, `kvnc_getBlockByHash`, `kvnc_getBlockByNumber`,
 *   `kvnc_getBalance`, `kvnc_getValidators`, `kvnc_sendRawTransaction`,
 *   `kvnc_getPendingTransactions`, `kvnc_estimateFee`, `kvnc_getNonce`,
 *   `kvnc_getStake`, `kvnc_getRewards`, `kvnc_getTransactionByHash`,
 *   `kvnc_getTransactionReceipt`, `kvnc_getLeaderSchedule`, `kvnc_getCommittee`
 *   and `GET /health`.
 *
 * WebSocket subscriptions accepted by `kvnc-rpc/src/subscriptions.rs`:
 *   `newHeads`, `newCommittedLeader`, `pendingTransactions`, `logs`,
 *   `newSealedHeader`.
 */

import { KvncRpcClient } from "@kvnc/api-client";

export {
  KvncRpcClient,
  subscribeNewHeads,
  subscriptionHead,
} from "@kvnc/api-client";
export type { BlockResponse, HealthStatus } from "@kvnc/api-client";

/** Loopback node default. The URL must include `/rpc`. */
export const LOCAL_RPC_URL = "http://127.0.0.1:8545/rpc";

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
  /** Latest committed leader height from `kvnc_blockNumber`. */
  height: bigint;
  /** `height` as a `0x`-prefixed hex quantity, for display. */
  heightHex: string;
}

/** Build a client for a JSON-RPC endpoint. Defaults to a loopback node. */
export function createRpcClient(url: string = LOCAL_RPC_URL): KvncRpcClient {
  return new KvncRpcClient(url);
}

/** Read the latest committed leader height (`kvnc_blockNumber`). */
export async function fetchChainHead(client: KvncRpcClient): Promise<ChainHead> {
  const height = await client.blockNumber();
  return { height, heightHex: `0x${height.toString(16)}` };
}
