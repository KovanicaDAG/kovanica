/** Endpoint catalog shared by docs and the api-reference playground. */
export type EndpointRow = {
  method: "GET" | "POST";
  path: string;
  note: string;
  /** Safe to run read-only in the playground. */
  read?: boolean;
};

export const ENDPOINTS: EndpointRow[] = [
  { method: "GET", path: "/api/head", note: "genesis, tip, height", read: true },
  { method: "GET", path: "/api/bootstrap", note: "plus listen, peers, upstream probe", read: true },
  { method: "GET", path: "/api/p2p", note: "TCP listen + bootstrap peers", read: true },
  { method: "GET", path: "/api/blocks", note: "octet-stream dump (clone catch-up)" },
  { method: "GET", path: "/api/state", note: "full DAG + flags", read: true },
  { method: "GET", path: "/api/utxos?address=", note: "spendable outputs", read: true },
  { method: "GET", path: "/api/history?address=", note: "deltas per address", read: true },
  { method: "GET", path: "/api/origins", note: "ISO3 pulses", read: true },
  { method: "GET", path: "/api/spec", note: "full protocol spec, text/plain", read: true },
  { method: "POST", path: "/api/prepare", note: "sighash + fee + change" },
  { method: "POST", path: "/api/submit", note: "queue signed tx" },
  { method: "POST", path: "/api/produce", note: "pack mempool" },
  { method: "POST", path: "/api/mine", note: "mine a coinbase block on the selected node" },
  { method: "POST", path: "/api/faucet", note: "testnet open faucet" },
  { method: "POST", path: "/api/origin", note: "pulse a country" },
];

/** Read-only endpoints available in the playground. */
export const PLAYGROUND_ENDPOINTS = [
  { label: "/api/head", path: "/api/head", needsAddress: false },
  { label: "/api/bootstrap", path: "/api/bootstrap", needsAddress: false },
  { label: "/api/state", path: "/api/state", needsAddress: false },
  { label: "/api/utxos", path: "/api/utxos", needsAddress: true },
  { label: "/api/history", path: "/api/history", needsAddress: true },
  { label: "/api/origins", path: "/api/origins", needsAddress: false },
  { label: "/api/p2p", path: "/api/p2p", needsAddress: false },
] as const;
