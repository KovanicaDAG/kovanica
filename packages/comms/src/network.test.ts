import { describe, expect, it } from "vitest";

import { LOCAL_RPC_URL, NETWORK_PRESETS, TESTNET_RPC_URL } from "./index.js";

describe("comms network presets", () => {
  it("lists the live chain-4 testnet first", () => {
    expect(NETWORK_PRESETS[0]).toEqual({
      id: "testnet",
      label: "Testnet (chain 4)",
      chainId: 4,
      rpcUrl: TESTNET_RPC_URL,
    });
    expect(TESTNET_RPC_URL).toBe("https://rpc-testnet.kovanica.online/rpc");
  });

  it("keeps the loopback node for development", () => {
    const local = NETWORK_PRESETS.find((preset) => preset.id === "local");
    expect(local?.chainId).toBe(1337);
    expect(local?.rpcUrl).toBe(LOCAL_RPC_URL);
  });

  it("uses chain ids the node accepts", () => {
    expect(NETWORK_PRESETS.map((preset) => preset.chainId)).toEqual([4, 1337]);
  });
});
