import { describe, expect, it } from "vitest";

import {
  HYBRID_TESTNET,
  LOCAL,
  MAINNET,
  TESTNET,
  networksFor,
  rejectionMessage,
  resolveNetwork,
} from "./network";

describe("network selection", () => {
  const networks = networksFor(true);

  it("ships testnet and hybrid-testnet in production", () => {
    expect(networks.map((network) => network.id)).toEqual([
      "testnet",
      "mainnet",
      "hybrid-testnet",
    ]);
    expect(networks.find((network) => network.id === "local")).toBeUndefined();
    expect(TESTNET.enabled).toBe(true);
    expect(TESTNET.chainId).toBe(4);
    expect(MAINNET.enabled).toBe(false);
    expect(MAINNET.chainId).toBe(1);
    expect(MAINNET.faucetUrl).toBeNull();
    expect(TESTNET.rpcUrl).toBe("https://rpc-testnet.kovanica.online/rpc");
    expect(HYBRID_TESTNET.id).toBe("hybrid-testnet");
    expect(HYBRID_TESTNET.label).toBe("Hybrid Testnet (Chain 5)");
    expect(HYBRID_TESTNET.chainId).toBe(5);
    expect(HYBRID_TESTNET.enabled).toBe(true);
    expect(HYBRID_TESTNET.rpcUrl).toBe("http://127.0.0.1:8545");
  });

  it("resolves hybrid-testnet when requested", () => {
    expect(resolveNetwork("hybrid-testnet", null, networks).id).toBe("hybrid-testnet");
    expect(resolveNetwork("hybrid-testnet", null, networks).chainId).toBe(5);
    expect(rejectionMessage("hybrid-testnet", networks)).toBeNull();
  });

  it("includes local node in development networks", () => {
    const devNetworks = networksFor(false);
    expect(devNetworks.map((n) => n.id)).toEqual([
      "testnet",
      "mainnet",
      "hybrid-testnet",
      "local",
    ]);
    expect(devNetworks.find((n) => n.id === "local")).toBe(LOCAL);
  });

  it("ignores mainnet from the query string and from storage", () => {
    expect(resolveNetwork("mainnet", "mainnet", networks).id).toBe("testnet");
    expect(resolveNetwork("mainnet", "mainnet", networks).chainId).toBe(4);
    expect(rejectionMessage("mainnet", networks)).toMatch(/not open yet/i);
    expect(rejectionMessage("testnet", networks)).toBeNull();
  });

  it("keeps a stored testnet choice", () => {
    expect(resolveNetwork(null, "testnet", networks).id).toBe("testnet");
  });
});
