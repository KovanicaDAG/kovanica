import { afterEach, describe, expect, it, vi } from "vitest";

import {
  ChainId,
  HYBRID_TESTNET_CHAIN_ID,
  KovanicaRpcClient,
  type BlockHeaderSubmission,
  type DagTipsResponse,
  type MiningInfo,
  type SubmitWorkResponse,
  type WorkTemplate,
} from "./index.js";

afterEach(() => {
  vi.unstubAllGlobals();
});

function jsonResponse(result: unknown, status = 200): Response {
  return new Response(JSON.stringify({ jsonrpc: "2.0", id: 1, result }), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

describe("ChainId and HYBRID_TESTNET_CHAIN_ID constants", () => {
  it("exports correct chain IDs including HYBRID_TESTNET", () => {
    expect(ChainId.MAINNET).toBe(1);
    expect(ChainId.TESTNET).toBe(2);
    expect(ChainId.DEVNET).toBe(3);
    expect(ChainId.HYBRID_TESTNET).toBe(2);
    expect(ChainId.LOCAL).toBe(1337);
    expect(HYBRID_TESTNET_CHAIN_ID).toBe(2);
  });
});

describe("KovanicaRpcClient hybrid PoW and GHOSTDAG RPC methods", () => {
  it("calls kovanica_getMiningInfo", async () => {
    const bodies: unknown[] = [];
    const mockMiningInfo: MiningInfo = {
      chain_id: 2,
      mining_enabled: true,
      mining_threads: 4,
      target_bits: "0x1e00ffff",
      target: "0x0000ffff00000000000000000000000000000000000000000000000000000000",
      difficulty: "1000",
      target_block_time_ms: 4000,
      ghostdag_k: 5,
      network_hashrate_estimate: 250,
      dag_tips_count: 2,
    };

    vi.stubGlobal(
      "fetch",
      vi.fn(async (_url: string, init: RequestInit) => {
        bodies.push(JSON.parse(String(init.body)));
        return jsonResponse(mockMiningInfo);
      }),
    );

    const client = new KovanicaRpcClient("http://127.0.0.1:8545/rpc");
    const res = await client.getMiningInfo();

    expect(bodies[0]).toMatchObject({
      method: "kovanica_getMiningInfo",
    });
    expect(res).toEqual(mockMiningInfo);
  });

  it("calls kovanica_getDagTips", async () => {
    const bodies: unknown[] = [];
    const mockDagTips: DagTipsResponse = {
      chain_id: 2,
      tips: [
        "0x1111111111111111111111111111111111111111111111111111111111111111",
        "0x2222222222222222222222222222222222222222222222222222222222222222",
      ],
      count: 2,
      max_round: 42,
      ghostdag_k: 5,
    };

    vi.stubGlobal(
      "fetch",
      vi.fn(async (_url: string, init: RequestInit) => {
        bodies.push(JSON.parse(String(init.body)));
        return jsonResponse(mockDagTips);
      }),
    );

    const client = new KovanicaRpcClient("http://127.0.0.1:8545/rpc");
    const res = await client.getDagTips();

    expect(bodies[0]).toMatchObject({
      method: "kovanica_getDagTips",
    });
    expect(res).toEqual(mockDagTips);
  });

  it("calls kovanica_getWork with miner address", async () => {
    const bodies: unknown[] = [];
    const mockWorkTemplate: WorkTemplate = {
      chain_id: 2,
      round: 43,
      author: 1,
      parents: ["0x1111111111111111111111111111111111111111111111111111111111111111"],
      parent_refs: [{ author: 1, round: 42, digest: "0x1111" }],
      merkle_root: "0x0000000000000000000000000000000000000000000000000000000000000000",
      timestamp: 1700000000000,
      target_bits: "0x1e00ffff",
      target: "0x0000ffff",
      preimage: "0xaabbcc",
      transactions_count: 0,
      header: {},
    };

    vi.stubGlobal(
      "fetch",
      vi.fn(async (_url: string, init: RequestInit) => {
        bodies.push(JSON.parse(String(init.body)));
        return jsonResponse(mockWorkTemplate);
      }),
    );

    const minerAddr = "kvnc000000000000000000000000000000000000000000000000000000000dag";
    const client = new KovanicaRpcClient("http://127.0.0.1:8545/rpc");
    const res = await client.getWork(minerAddr);

    expect(bodies[0]).toMatchObject({
      method: "kovanica_getWork",
      params: [minerAddr],
    });
    expect(res).toEqual(mockWorkTemplate);
  });

  it("calls kovanica_submitWork with block header", async () => {
    const bodies: unknown[] = [];
    const mockSubmitRes: SubmitWorkResponse = {
      accepted: true,
      digest: "0x3333333333333333333333333333333333333333333333333333333333333333",
      pow_hash: "0x0000000000111111111111111111111111111111111111111111111111111111",
    };

    vi.stubGlobal(
      "fetch",
      vi.fn(async (_url: string, init: RequestInit) => {
        bodies.push(JSON.parse(String(init.body)));
        return jsonResponse(mockSubmitRes);
      }),
    );

    const submission: BlockHeaderSubmission = {
      chain_id: 2,
      round: 43,
      author: 1,
      parents: [],
      merkle_root: "0x00",
      timestamp: 1700000000000,
      target_bits: 0x1e00ffff,
      nonce: 12345,
    };

    const client = new KovanicaRpcClient("http://127.0.0.1:8545/rpc");
    const res = await client.submitWork(submission);

    expect(bodies[0]).toMatchObject({
      method: "kovanica_submitWork",
      params: submission,
    });
    expect(res).toEqual(mockSubmitRes);
  });
});
