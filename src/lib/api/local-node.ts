/**
 * Upstream of the local Rust node's HTTP API (server-only).
 *
 * Configured with `KOVANICA_NODE_API_URL` (see `.env.example`); defaults to the
 * loopback explorer port so a co-located node works without configuration.
 */
export const DEFAULT_NODE_API_URL = "http://127.0.0.1:8080";

/** Normalised base URL (no trailing slash); invalid or non-http(s) values fall back. */
export function nodeApiBase(value: string | undefined = process.env.KOVANICA_NODE_API_URL): string {
  const raw = value?.trim();
  if (!raw) return DEFAULT_NODE_API_URL;
  try {
    const u = new URL(raw);
    if (u.protocol !== "http:" && u.protocol !== "https:") return DEFAULT_NODE_API_URL;
    return `${u.origin}${u.pathname}`.replace(/\/+$/, "");
  } catch {
    return DEFAULT_NODE_API_URL;
  }
}

/** Full upstream URL for an incoming request path + query. */
export function localNodeTarget(pathname: string, search: string, base: string = nodeApiBase()): string {
  return `${base}${pathname}${search}`;
}
