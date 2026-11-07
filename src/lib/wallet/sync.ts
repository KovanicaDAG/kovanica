/**
 * Wallet balance synchronization and freshness manager.
 *
 * Requirements:
 * - Refreshes balances every 10 seconds while a wallet session is active and page is visible.
 * - Reuses existing fetch logic without competing intervals or overlapping requests.
 * - Refreshes immediately after visibility resumes and after a confirmed transaction.
 * - Rejects/ignores stale responses on address/network change, lock and unmount.
 * - Preserves last successful balances with explicit stale/error status on failure (never turns failure to 0).
 * - Clears values belonging to previous address/network.
 * - Preserves watch-only and hardware flows without requiring private keys or unlocks.
 */
import { api } from "@/lib/api/client";
import type { ApiHistory, ApiUtxos } from "@/lib/api/contract";
import type { ApiToken } from "@/lib/tokens/codec";

export type WalletSyncStatus = "idle" | "loading" | "fresh" | "stale" | "error";

export interface FeeRates {
  slow: number;
  normal: number;
  fast: number;
}

export interface WalletSyncSnapshot {
  utxos: ApiUtxos | null;
  history: ApiHistory | null;
  tokens: ApiToken[];
  feeRates: FeeRates | null;
  status: WalletSyncStatus;
  lastRefreshedAt: number | null;
  error: string | null;
  isStale: boolean;
}

export interface FetchAdapter {
  fetchUtxos: (address: string, network: string) => Promise<ApiUtxos>;
  fetchHistory: (address: string, network: string) => Promise<ApiHistory>;
  fetchFeeEstimate: (network: string) => Promise<{ fee_rate?: number }>;
  fetchTokens: (address: string, network: string) => Promise<ApiToken[]>;
}

export const defaultFetchAdapter: FetchAdapter = {
  fetchUtxos: (addr) => api<ApiUtxos>(`/api/utxos?address=${addr}`),
  fetchHistory: (addr) => api<ApiHistory>(`/api/history?address=${addr}`),
  fetchFeeEstimate: () =>
    api<{ fee_rate: number; unit: string; mempool: number; bytes: number }>(
      "/api/fee_estimate",
      "GET",
    ),
  fetchTokens: (addr) => api<ApiToken[]>(`/api/tokens?address=${addr}`),
};

export class WalletSyncManager {
  private address: string | null = null;
  private network: string = "testnet";
  private isVisible: boolean = true;
  private intervalMs: number = 10000;
  private fetchAdapter: FetchAdapter;

  private timerId: ReturnType<typeof setTimeout> | null = null;
  private inFlightGen: number | null = null;
  private currentGen: number = 0;

  private state: WalletSyncSnapshot = {
    utxos: null,
    history: null,
    tokens: [],
    feeRates: null,
    status: "idle",
    lastRefreshedAt: null,
    error: null,
    isStale: false,
  };

  private listeners = new Set<(s: WalletSyncSnapshot) => void>();

  constructor(
    fetchAdapter: FetchAdapter = defaultFetchAdapter,
    intervalMs = 10000,
  ) {
    this.fetchAdapter = fetchAdapter;
    this.intervalMs = intervalMs;
  }

  public getState(): WalletSyncSnapshot {
    return this.state;
  }

  public subscribe(fn: (s: WalletSyncSnapshot) => void): () => void {
    this.listeners.add(fn);
    return () => {
      this.listeners.delete(fn);
    };
  }

  private notify() {
    for (const fn of this.listeners) {
      fn(this.state);
    }
  }

  public setSession(address: string | null, network: string) {
    const addressChanged = this.address !== address;
    const networkChanged = this.network !== network;

    if (!addressChanged && !networkChanged) return;

    // Invalidate in-flight requests from prior session
    this.currentGen += 1;
    this.clearTimer();

    this.address = address;
    this.network = network;

    // Immediately clear state from prior session/address/network
    this.state = {
      utxos: null,
      history: null,
      tokens: [],
      feeRates: null,
      status: address ? "loading" : "idle",
      lastRefreshedAt: null,
      error: null,
      isStale: false,
    };
    this.notify();

    if (this.address) {
      void this.executeRefresh();
    }
  }

  public setVisibility(visible: boolean) {
    if (this.isVisible === visible) return;
    this.isVisible = visible;

    if (visible) {
      // Immediately refresh when visibility resumes and reset the 10s timer
      if (this.address) {
        this.clearTimer();
        void this.executeRefresh();
      }
    } else {
      // Pause cadence when hidden
      this.clearTimer();
    }
  }

  public async refresh(): Promise<void> {
    if (!this.address) return;
    this.clearTimer();
    await this.executeRefresh();
  }

  public async notifyConfirmed(_txHash?: string): Promise<void> {
    if (!this.address) return;
    // Transaction confirmation refresh: immediately refreshes balances and resets cadence
    this.clearTimer();
    await this.executeRefresh();
  }

  private clearTimer() {
    if (this.timerId !== null) {
      clearTimeout(this.timerId);
      this.timerId = null;
    }
  }

  private scheduleNext() {
    this.clearTimer();
    if (!this.address || !this.isVisible) return;

    this.timerId = setTimeout(() => {
      void this.executeRefresh();
    }, this.intervalMs);
  }

  private async executeRefresh(): Promise<void> {
    if (!this.address) return;
    // Prevent overlapping requests for the same session generation
    if (this.inFlightGen === this.currentGen) return;

    const targetAddress = this.address;
    const targetNetwork = this.network;
    const gen = this.currentGen;
    this.inFlightGen = gen;

    // If we have no prior data, show loading status
    if (!this.state.utxos) {
      this.state = { ...this.state, status: "loading" };
      this.notify();
    }

    try {
      const [u, h, f] = await Promise.all([
        this.fetchAdapter.fetchUtxos(targetAddress, targetNetwork),
        this.fetchAdapter.fetchHistory(targetAddress, targetNetwork),
        this.fetchAdapter.fetchFeeEstimate(targetNetwork),
      ]);

      if (
        !u ||
        typeof u !== "object" ||
        typeof u.balance !== "number" ||
        !Array.isArray(u.utxos) ||
        !h ||
        typeof h !== "object" ||
        !Array.isArray(h.txs)
      ) {
        throw new Error("Malformed wallet sync response");
      }

      // Check if stale before applying
      if (
        gen !== this.currentGen ||
        targetAddress !== this.address ||
        targetNetwork !== this.network
      ) {
        return;
      }

      let nextFeeRates = this.state.feeRates;
      if (typeof f?.fee_rate === "number") {
        nextFeeRates = {
          slow: f.fee_rate,
          normal: f.fee_rate,
          fast: f.fee_rate,
        };
      }

      this.state = {
        ...this.state,
        utxos: u,
        history: h,
        feeRates: nextFeeRates,
        status: "fresh",
        isStale: false,
        error: null,
        lastRefreshedAt: Date.now(),
      };
      this.notify();

      // Async token scan - does not hold up balance
      void this.fetchAdapter
        .fetchTokens(targetAddress, targetNetwork)
        .then((tokens) => {
          if (
            Array.isArray(tokens) &&
            gen === this.currentGen &&
            targetAddress === this.address &&
            targetNetwork === this.network
          ) {
            this.state = { ...this.state, tokens };
            this.notify();
          }
        })
        .catch(() => {
          // Token discovery error does not invalidate balance
        });
    } catch (err) {
      if (
        gen !== this.currentGen ||
        targetAddress !== this.address ||
        targetNetwork !== this.network
      ) {
        return;
      }

      const errMsg = err instanceof Error ? err.message : "Refresh failed";

      // If we had successful data, preserve it with explicit stale status
      if (this.state.utxos !== null) {
        this.state = {
          ...this.state,
          status: "stale",
          isStale: true,
          error: errMsg,
        };
      } else {
        this.state = {
          ...this.state,
          status: "error",
          isStale: true,
          error: errMsg,
        };
      }
      this.notify();
    } finally {
      if (this.inFlightGen === gen) {
        this.inFlightGen = null;
      }
      this.scheduleNext();
    }
  }

  public destroy() {
    this.currentGen += 1;
    this.inFlightGen = null;
    this.clearTimer();
    this.listeners.clear();
    this.address = null;
  }
}
