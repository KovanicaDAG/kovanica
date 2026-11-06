import { useCallback, useEffect, useRef, useState } from "react";
import { api, useApiSource } from "./client";
import type { ApiState } from "./contract";

export const MIN_POLL_MS = 2000;
export const MAX_BACKOFF_MS = 30000;

/** Bounded exponential retry backoff helper (e.g. 2s -> 4s -> 8s -> 16s -> 30s). */
export function nextBackoffMs(currentMs: number, maxMs = MAX_BACKOFF_MS): number {
  return Math.min(Math.max(currentMs * 2, MIN_POLL_MS), maxMs);
}

export function useNode(pollMs = MIN_POLL_MS) {
  const source = useApiSource();
  const [state, setState] = useState<ApiState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const inFlightRef = useRef(false);
  const currentDelayRef = useRef(pollMs);
  const timerRef = useRef<number | null>(null);
  const abortCtrlRef = useRef<AbortController | null>(null);
  const currentSourceRef = useRef(source);
  currentSourceRef.current = source;

  const clearTimer = () => {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  };

  const scheduleNext = (delayMs: number) => {
    clearTimer();
    if (typeof document !== "undefined" && document.hidden) {
      return; // Pause when document is hidden
    }
    timerRef.current = window.setTimeout(() => {
      void executePoll();
    }, delayMs);
  };

  const executePoll = async () => {
    if (inFlightRef.current) return;
    if (typeof document !== "undefined" && document.hidden) return;

    inFlightRef.current = true;
    const reqSource = currentSourceRef.current;
    const ctrl = new AbortController();
    abortCtrlRef.current = ctrl;

    try {
      const next = await api<ApiState>("/api/state", "GET", ctrl.signal);
      // Guard against responses for stale source or unmounted
      if (reqSource === currentSourceRef.current) {
        setState(next);
        setError(null);
        currentDelayRef.current = pollMs; // Reset backoff on success
      }
    } catch (e: any) {
      if (e?.name === "AbortError") return;
      if (reqSource === currentSourceRef.current) {
        setError(e instanceof Error ? e.message : "offline");
        currentDelayRef.current = nextBackoffMs(currentDelayRef.current, MAX_BACKOFF_MS);
      }
    } finally {
      inFlightRef.current = false;
      if (reqSource === currentSourceRef.current) {
        scheduleNext(currentDelayRef.current);
      }
    }
  };

  const refresh = useCallback(async () => {
    clearTimer();
    currentDelayRef.current = pollMs;
    await executePoll();
  }, [pollMs]);

  useEffect(() => {
    // Reset delay when source changes and execute poll
    currentDelayRef.current = pollMs;
    void executePoll();

    const handleVisibilityChange = () => {
      if (!document.hidden) {
        // Immediate refresh on visible and resume schedule
        clearTimer();
        currentDelayRef.current = pollMs;
        void executePoll();
      } else {
        clearTimer();
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      clearTimer();
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      if (abortCtrlRef.current) {
        abortCtrlRef.current.abort();
      }
    };
  }, [source, pollMs]);

  const act = useCallback(
    async (path: string) => {
      setBusy(true);
      try {
        await api(path, "POST");
        await refresh();
      } finally {
        setBusy(false);
      }
    },
    [refresh],
  );

  return { state, error, busy, refresh, act, source };
}
