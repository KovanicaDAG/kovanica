import { useEffect, useMemo, useRef, useState } from "react";
import { WalletSyncManager, type WalletSyncSnapshot, type FetchAdapter } from "./sync";

export function useWalletSync(
  address: string | null | undefined,
  network: string,
  fetchAdapter?: FetchAdapter,
  intervalMs = 10000,
) {
  const manager = useMemo(() => new WalletSyncManager(fetchAdapter, intervalMs), [fetchAdapter, intervalMs]);
  const [snapshot, setSnapshot] = useState<WalletSyncSnapshot>(() => manager.getState());
  const managerRef = useRef(manager);
  managerRef.current = manager;

  // Sync session changes (address, network)
  useEffect(() => {
    const mgr = managerRef.current;
    mgr.setSession(address ?? null, network);
  }, [address, network]);

  // Subscribe to state changes
  useEffect(() => {
    const mgr = managerRef.current;
    const unsub = mgr.subscribe(setSnapshot);
    return () => {
      unsub();
      mgr.destroy();
    };
  }, []);

  // Listen to visibility changes
  useEffect(() => {
    if (typeof document === "undefined") return;

    const handleVisibilityChange = () => {
      const visible = document.visibilityState === "visible";
      managerRef.current.setVisibility(visible);
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    // Initialize current visibility state
    managerRef.current.setVisibility(document.visibilityState === "visible");

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);

  return {
    ...snapshot,
    refresh: () => managerRef.current.refresh(),
    notifyConfirmedTransaction: (txHash?: string) => managerRef.current.notifyConfirmed(txHash),
  };
}
