import { describe, expect, it } from "vitest";

import {
  ContractKind,
  HtlcMethod,
  VaultMethod,
  MultisigMethod,
  TokenMethod,
  contractCall,
  isContractEvent,
  bytesToHex,
  hexToBytes,
} from "./contracts.js";

describe("contracts helpers", () => {
  it("exposes contract kinds and method constants", () => {
    expect(ContractKind.HTLC).toBe("htlc");
    expect(HtlcMethod.CLAIM).toBe("claim");
    expect(VaultMethod.CANCEL).toBe("cancel");
    expect(MultisigMethod.EXECUTE).toBe("execute");
    expect(TokenMethod.TRANSFER_FROM).toBe("transfer_from");
  });

  it("contractCall normalizes Uint8Array and arrays", () => {
    const fromArr = contractCall("transfer", [1, 2, 3]);
    expect(fromArr).toEqual({ method: "transfer", args: [1, 2, 3] });
    const fromU8 = contractCall("mint", new Uint8Array([4, 5]));
    expect(fromU8).toEqual({ method: "mint", args: [4, 5] });
    const empty = contractCall("burn");
    expect(empty).toEqual({ method: "burn", args: [] });
  });

  it("isContractEvent detects non-empty topics", () => {
    expect(isContractEvent({ topic: [1, 2] })).toBe(true);
    expect(isContractEvent({ topic: [] })).toBe(false);
    expect(isContractEvent({})).toBe(false);
  });

  it("bytesToHex / hexToBytes round-trip", () => {
    const bytes = [0xde, 0xad, 0xbe, 0xef];
    const hex = bytesToHex(bytes);
    expect(hex).toBe("deadbeef");
    expect(hexToBytes(hex)).toEqual(bytes);
    expect(hexToBytes("0xdeadbeef")).toEqual(bytes);
  });

  it("hexToBytes rejects odd length", () => {
    expect(() => hexToBytes("abc")).toThrow();
  });
});