export { COMMS_ID_DOMAIN, deriveCommsId, deriveCommsIdBytes } from "./identity.js";
export {
  LOCAL_RPC_URL,
  SUBSCRIPTION_METHODS,
  createRpcClient,
  fetchChainHead,
  KvncRpcClient,
  subscribeNewHeads,
  subscriptionHead,
} from "./rpc.js";
export type { ChainHead, SubscriptionMethod } from "./rpc.js";
