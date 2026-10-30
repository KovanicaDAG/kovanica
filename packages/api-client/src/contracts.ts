/**
 * Contract SDK helpers for Kovanica (TypeScript).
 */

export const ContractKind = {
  HTLC: "htlc",
  Vault: "vault",
  Multisig: "multisig",
  Token: "token",
} as const;
export type ContractKind = (typeof ContractKind)[keyof typeof ContractKind];

export const HtlcMethod = { CREATE: "create", CLAIM: "claim", REFUND: "refund" } as const;
export const VaultMethod = { CREATE: "create", CLAIM: "claim", CANCEL: "cancel" } as const;
export const MultisigMethod = { PROPOSE: "propose", CONFIRM: "confirm", EXECUTE: "execute" } as const;
export const TokenMethod = {
  TRANSFER: "transfer",
  APPROVE: "approve",
  TRANSFER_FROM: "transfer_from",
  MINT: "mint",
  BURN: "burn",
} as const;

export interface ContractCallSpec {
  method: string;
  args: number[];
}

export function contractCall(method: string, args: Uint8Array | number[] = []): ContractCallSpec {
  const arr = args instanceof Uint8Array ? Array.from(args) : args;
  return { method, args: arr };
}

export const CONTRACT_EVENT_TOPIC = "contract";

export function isContractEvent(event: { topic?: number[] }): boolean {
  return Array.isArray(event.topic) && event.topic.length > 0;
}

export function bytesToHex(bytes: number[] | Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

export function hexToBytes(hex: string): number[] {
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  if (clean.length % 2 !== 0) throw new Error("hex string must have even length");
  const out: number[] = [];
  for (let i = 0; i < clean.length; i += 2) {
    out.push(parseInt(clean.slice(i, i + 2), 16));
  }
  return out;
}

export interface DecodedContractEvent {
  addressHex: string;
  topicHex: string;
  dataHex: string;
  dataText: string | null;
  json: unknown | null;
}

export function decodeContractEvent(event: {
  address: number[];
  topic: number[];
  data: number[];
}): DecodedContractEvent {
  const dataBytes = new Uint8Array(event.data);
  let dataText: string | null = null;
  let json: unknown | null = null;
  try {
    dataText = new TextDecoder("utf-8", { fatal: true }).decode(dataBytes);
  } catch {
    dataText = null;
  }
  if (dataText !== null) {
    try {
      json = JSON.parse(dataText);
    } catch {
      json = null;
    }
  }
  return {
    addressHex: bytesToHex(event.address),
    topicHex: bytesToHex(event.topic),
    dataHex: bytesToHex(event.data),
    dataText,
    json,
  };
}
export function entryPoint(kind: ContractKind, method: string): string {
  return `${kind}_${method}`;
}
