import { test } from "node:test";
import assert from "node:assert/strict";
import { WalletSyncManager, type FetchAdapter } from "../src/lib/wallet/sync";
import type { ApiUtxos } from "../src/lib/api/contract";

function createMockAdapter(overrides?: Partial<FetchAdapter>): {
  adapter: FetchAdapter;
  calls: { utxos: string[]; history: string[]; fee: number; tokens: string[] };
} {
  const calls = {
    utxos: [] as string[],
    history: [] as string[],
    fee: 0,
    tokens: [] as string[],
  };

  const adapter: FetchAdapter = {
    fetchUtxos: async (addr, net) => {
      calls.utxos.push(`${addr}@${net}`);
      if (overrides?.fetchUtxos) return overrides.fetchUtxos(addr, net);
      return { address: addr, balance: 5_000_000_000, utxos: [] };
    },
    fetchHistory: async (addr, net) => {
      calls.history.push(`${addr}@${net}`);
      if (overrides?.fetchHistory) return overrides.fetchHistory(addr, net);
      return { address: addr, balance: 5_000_000_000, txs: [] };
    },
    fetchFeeEstimate: async (net) => {
      calls.fee += 1;
      if (overrides?.fetchFeeEstimate) return overrides.fetchFeeEstimate(net);
      return { fee_rate: 10 };
    },
    fetchTokens: async (addr, net) => {
      calls.tokens.push(`${addr}@${net}`);
      if (overrides?.fetchTokens) return overrides.fetchTokens(addr, net);
      return [];
    },
  };

  return { adapter, calls };
}

test("WalletSync: initial session set triggers fetch and sets fresh status", async () => {
  const { adapter, calls } = createMockAdapter();
  const sync = new WalletSyncManager(adapter, 100);

  sync.setSession("kvnc1testaddr", "testnet");
  // Allow promise to resolve
  await new Promise((r) => setTimeout(r, 10));

  const state = sync.getState();
  assert.equal(state.status, "fresh");
  assert.equal(state.isStale, false);
  assert.equal(state.utxos?.balance, 5_000_000_000);
  assert.equal(state.feeRates?.normal, 10);
  assert.equal(calls.utxos.length, 1);
  assert.equal(calls.utxos[0], "kvnc1testaddr@testnet");

  sync.destroy();
});

test("WalletSync: visible cadence polls at intervalMs", async () => {
  const { adapter, calls } = createMockAdapter();
  const sync = new WalletSyncManager(adapter, 50);

  sync.setSession("kvnc1testaddr", "testnet");
  await new Promise((r) => setTimeout(r, 10));
  assert.equal(calls.utxos.length, 1);

  // Wait for 2 ticks (100ms+)
  await new Promise((r) => setTimeout(r, 120));
  assert.ok(calls.utxos.length >= 2, `Expected at least 2 calls, got ${calls.utxos.length}`);

  sync.destroy();
});

test("WalletSync: hidden visibility pauses polling; visible resume immediately refreshes", async () => {
  const { adapter, calls } = createMockAdapter();
  const sync = new WalletSyncManager(adapter, 50);

  sync.setSession("kvnc1testaddr", "testnet");
  await new Promise((r) => setTimeout(r, 10));
  assert.equal(calls.utxos.length, 1);

  // Pause
  sync.setVisibility(false);
  await new Promise((r) => setTimeout(r, 120));
  // Call count should not have increased while hidden
  const countWhileHidden = calls.utxos.length;
  assert.equal(countWhileHidden, 1, "Cadence must pause when hidden");

  // Resume
  sync.setVisibility(true);
  await new Promise((r) => setTimeout(r, 10));
  assert.equal(calls.utxos.length, 2, "Must immediately refresh upon visibility resume");

  sync.destroy();
});

test("WalletSync: in-flight requests do not overlap", async () => {
  let resolvePromise: (val: ApiUtxos) => void = () => {};
  const slowPromise = new Promise<ApiUtxos>((resolve) => {
    resolvePromise = resolve;
  });

  const { adapter, calls } = createMockAdapter({
    fetchUtxos: async () => slowPromise,
  });

  const sync = new WalletSyncManager(adapter, 100);
  sync.setSession("kvnc1testaddr", "testnet");

  // Call refresh while in-flight
  await sync.refresh();
  await sync.refresh();

  // Exactly 1 fetch should be in flight
  assert.equal(calls.utxos.length, 1, "Concurrent refresh calls must not overlap");

  resolvePromise({ address: "kvnc1testaddr", balance: 1000, utxos: [] });
  await new Promise((r) => setTimeout(r, 10));

  sync.destroy();
});

test("WalletSync: address switch clears previous state and drops stale responses", async () => {
  let resolveFirst: (val: ApiUtxos) => void = () => {};
  const firstPromise = new Promise<ApiUtxos>((resolve) => {
    resolveFirst = resolve;
  });

  const { adapter } = createMockAdapter({
    fetchUtxos: async (addr) => {
      if (addr === "addr1") return firstPromise;
      return { address: "addr2", balance: 9_000_000, utxos: [] };
    },
  });

  const sync = new WalletSyncManager(adapter, 100);
  sync.setSession("addr1", "testnet");

  // Switch to addr2 before addr1 resolves
  sync.setSession("addr2", "testnet");
  assert.equal(sync.getState().utxos, null, "State must be immediately cleared on address switch");

  // Wait for addr2 to resolve
  await new Promise((r) => setTimeout(r, 20));
  assert.equal(sync.getState().utxos?.address, "addr2");
  assert.equal(sync.getState().utxos?.balance, 9_000_000);

  // Now resolve the late addr1 response
  resolveFirst({ address: "addr1", balance: 1_000_000, utxos: [] });
  await new Promise((r) => setTimeout(r, 20));

  // addr2 state must remain unchanged; stale addr1 response must be ignored
  assert.equal(sync.getState().utxos?.address, "addr2");
  assert.equal(sync.getState().utxos?.balance, 9_000_000);

  sync.destroy();
});

test("WalletSync: network switch clears previous state and fetches for new network", async () => {
  const { adapter, calls } = createMockAdapter();
  const sync = new WalletSyncManager(adapter, 100);

  sync.setSession("myaddr", "testnet");
  await new Promise((r) => setTimeout(r, 10));
  assert.equal(calls.utxos[0], "myaddr@testnet");

  sync.setSession("myaddr", "devnet");
  assert.equal(sync.getState().utxos, null, "State must clear on network switch");
  await new Promise((r) => setTimeout(r, 10));
  assert.equal(calls.utxos[1], "myaddr@devnet");

  sync.destroy();
});

test("WalletSync: failure preserves last successful balance with status='stale'", async () => {
  let shouldFail = false;
  const { adapter } = createMockAdapter({
    fetchUtxos: async (addr) => {
      if (shouldFail) throw new Error("Connection refused (node offline)");
      return { address: addr, balance: 42_000_000_000, utxos: [] };
    },
  });

  const sync = new WalletSyncManager(adapter, 100);
  sync.setSession("myaddr", "testnet");
  await new Promise((r) => setTimeout(r, 10));

  assert.equal(sync.getState().status, "fresh");
  assert.equal(sync.getState().utxos?.balance, 42_000_000_000);

  // Trigger failing refresh
  shouldFail = true;
  await sync.refresh();

  const state = sync.getState();
  assert.equal(state.status, "stale");
  assert.equal(state.isStale, true);
  assert.equal(state.utxos?.balance, 42_000_000_000, "Must NOT zero out balance on failure");
  assert.ok(state.error?.includes("Connection refused"));

  sync.destroy();
});

test("WalletSync: notifyConfirmed immediately refreshes and does not create second polling loop", async () => {
  const { adapter, calls } = createMockAdapter();
  const sync = new WalletSyncManager(adapter, 60);

  sync.setSession("myaddr", "testnet");
  await new Promise((r) => setTimeout(r, 10));
  assert.equal(calls.utxos.length, 1);

  // Notify confirmed transaction
  await sync.notifyConfirmed("tx12345");
  assert.equal(calls.utxos.length, 2, "Must immediately refresh upon confirmation");

  // Wait 90ms (less than 2 full intervals)
  await new Promise((r) => setTimeout(r, 90));

  // Should have had at most 1 additional cadence poll, NOT multiple runaway loops
  assert.ok(calls.utxos.length <= 4, `Calls count ${calls.utxos.length} indicates no competing loops`);

  sync.destroy();
});

test("WalletSync: destroy clears timers and stops state emission", async () => {
  const { adapter, calls } = createMockAdapter();
  const sync = new WalletSyncManager(adapter, 30);

  sync.setSession("myaddr", "testnet");
  await new Promise((r) => setTimeout(r, 10));
  assert.equal(calls.utxos.length, 1);

  sync.destroy();
  const countAtDestroy = calls.utxos.length;

  await new Promise((r) => setTimeout(r, 80));
  assert.equal(calls.utxos.length, countAtDestroy, "Destroyed manager must never poll again");
});
