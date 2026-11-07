import { useSyncExternalStore } from "react";
import { isPublicSource, type ApiSource } from "./contract";

const KEY = "kovanica.source";
const listeners = new Set<() => void>();

function read(): ApiSource {
  // Preview was removed; the app always talks to the public testnet proxy by
  // default. A stored override (or operator "local" mode) wins if present.
  const stored = typeof window === "undefined" ? null : window.localStorage.getItem(KEY);
  if (stored === "local" || stored === "mainnet" || stored === "testnet") return stored;
  return "testnet";
}

export function getApiSource(): ApiSource {
  return read();
}

export function setApiSource(next: ApiSource) {
  window.localStorage.setItem(KEY, next);
  listeners.forEach((l) => l());
}

/** Is a public (proxied testnet/mainnet) network selected — i.e. not local? */
export function isPublic(source: ApiSource): boolean {
  return isPublicSource(source);
}

export function useApiSource(): ApiSource {
  return useSyncExternalStore(
    (cb) => {
      listeners.add(cb);
      return () => listeners.delete(cb);
    },
    read,
    () => "local" as const,
  );
}

export class ApiError extends Error {
  status: number;
  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

export class HttpError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = "HttpError";
    this.status = status;
  }
}

export function withSource(path: string, source: ApiSource): string {
  if (path.startsWith("http://") || path.startsWith("https://")) {
    const u = new URL(path);
    if (!u.searchParams.has("source")) u.searchParams.set("source", source);
    return u.toString();
  }
  const base =
    typeof window !== "undefined"
      ? window.location.origin
      : `http://127.0.0.1:${process.env.PORT || process.env.NITRO_PORT || 3010}`;
  const u = new URL(path, base);
  if (!u.searchParams.has("source")) u.searchParams.set("source", source);
  return typeof window !== "undefined" ? `${u.pathname}${u.search}` : u.toString();
}

export async function api<T = unknown>(
  path: string,
  method: "GET" | "POST" = "GET",
  signal?: AbortSignal,
): Promise<T> {
  const source = getApiSource();
  const r = await fetch(withSource(path, source), { method, signal });
  if (!r.ok) {
    const text = await r.text().catch(() => "");
    throw new ApiError(text || `HTTP ${r.status}`, r.status);
  }
  const ct = r.headers.get("content-type") ?? "";
  if (ct.includes("json")) return r.json() as Promise<T>;
  return (await r.text()) as T;
}

/** POST a JSON body to the configured API source. */
export async function apiPostJson<T = unknown>(path: string, body: unknown): Promise<T> {
  const source = getApiSource();
  const r = await fetch(withSource(path, source), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!r.ok) {
    const text = await r.text().catch(() => "");
    throw new ApiError(text || `HTTP ${r.status}`, r.status);
  }
  return r.json() as Promise<T>;
}
