/**
 * Typed client for the standalone `kovanica-indexer` process.
 *
 * The indexer is an optional, separate service. These helpers never throw on
 * a missing indexer at construction time; callers are expected to catch
 * network errors and fall back to node RPC (`kovanica_getLogs`) where
 * possible.
 *
 * Quantities on the wire are JSON numbers/byte arrays, matching the
 * indexer's serde-derived shape (`IndexedEvent`, `IndexerStatus`).
 */

export interface IndexedEvent {
  id: number;
  height: number;
  tx_hash: number[];
  tx_index: number;
  log_index: number;
  address: number[];
  topic: number[];
  data: number[];
  success: boolean;
}

export interface IndexerStatus {
  latest_indexed_height: number;
  latest_node_height: number;
  event_count: number;
}

export interface IndexerLogFilter {
  fromHeight?: number;
  toHeight?: number;
  address?: string;
  topic?: string;
  /** Alias for `topic`, mirroring the common `topic0` naming. */
  topic0?: string;
  limit?: number;
}

export interface IndexerWsFilter {
  address?: string;
  topics?: string[];
}

export class KovanicaIndexerClient {
  constructor(private baseUrl: string) {
    if (!baseUrl || baseUrl.length === 0) throw new Error("Indexer URL is required");
  }

  private endpoint(path: string): string {
    return `${this.baseUrl.replace(/\/+$/, "")}${path}`;
  }

  private buildQuery(filter?: IndexerLogFilter): string {
    const params = new URLSearchParams();
    if (filter?.fromHeight != null) params.set("from_height", String(filter.fromHeight));
    if (filter?.toHeight != null) params.set("to_height", String(filter.toHeight));
    if (filter?.address) params.set("address", filter.address);
    const topic = filter?.topic ?? filter?.topic0;
    if (topic) params.set("topic", topic);
    if (filter?.limit != null) params.set("limit", String(filter.limit));
    const qs = params.toString();
    return qs ? `?${qs}` : "";
  }

  async getStatus(): Promise<IndexerStatus> {
    const res = await fetch(this.endpoint("/status"));
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return (await res.json()) as IndexerStatus;
  }

  async getLogs(filter?: IndexerLogFilter): Promise<IndexedEvent[]> {
    const res = await fetch(`${this.endpoint("/logs")}${this.buildQuery(filter)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return (await res.json()) as IndexedEvent[];
  }

  async getEvents(filter?: IndexerLogFilter): Promise<IndexedEvent[]> {
    const res = await fetch(`${this.endpoint("/events")}${this.buildQuery(filter)}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return (await res.json()) as IndexedEvent[];
  }
}

export class IndexerWebSocketClient {
  private socket: WebSocket | null = null;
  private pendingFilter: IndexerWsFilter | null = null;
  private readonly eventHandlers = new Set<(event: IndexedEvent) => void>();
  private readonly errorHandlers = new Set<(error: Event) => void>();
  private readonly closeHandlers = new Set<() => void>();
  private readonly openHandlers = new Set<() => void>();

  constructor(private url: string) {
    if (!url || url.length === 0) throw new Error("Indexer WS URL is required");
  }

  connect(): void {
    if (this.socket) return;
    const ws = new WebSocket(this.url);
    this.socket = ws;
    ws.onopen = () => {
      if (this.pendingFilter) {
        try {
          ws.send(JSON.stringify({ subscribe: this.pendingFilter }));
        } catch {
          /* ignore send errors on a closing socket */
        }
      }
      for (const handler of this.openHandlers) handler();
    };
    ws.onmessage = (event: MessageEvent) => {
      let parsed: unknown;
      try {
        parsed = JSON.parse(String(event.data));
      } catch {
        return;
      }
      if (parsed && typeof parsed === "object" && "height" in (parsed as object) && "tx_hash" in (parsed as object)) {
        for (const handler of this.eventHandlers) handler(parsed as IndexedEvent);
      }
    };
    ws.onerror = (event: Event) => {
      for (const handler of this.errorHandlers) handler(event);
    };
    ws.onclose = () => {
      for (const handler of this.closeHandlers) handler();
    };
  }

  subscribe(filter?: IndexerWsFilter): void {
    this.pendingFilter = filter ?? {};
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      try {
        this.socket.send(JSON.stringify({ subscribe: this.pendingFilter }));
      } catch {
        /* ignore */
      }
    }
  }

  onEvent(handler: (event: IndexedEvent) => void): () => void {
    this.eventHandlers.add(handler);
    return () => this.eventHandlers.delete(handler);
  }

  onError(handler: (error: Event) => void): () => void {
    this.errorHandlers.add(handler);
    return () => this.errorHandlers.delete(handler);
  }

  onClose(handler: () => void): () => void {
    this.closeHandlers.add(handler);
    return () => this.closeHandlers.delete(handler);
  }

  onOpen(handler: () => void): () => void {
    this.openHandlers.add(handler);
    return () => this.openHandlers.delete(handler);
  }

  close(): void {
    if (this.socket) {
      try {
        this.socket.close();
      } catch {
        /* ignore */
      }
      this.socket = null;
    }
  }

  get readyState(): number {
    return this.socket?.readyState ?? 3;
  }
}

export interface IndexerWebSocketHandlers {
  onEvent?: (event: IndexedEvent) => void;
  onError?: (error: Event) => void;
  onClose?: () => void;
  filter?: IndexerWsFilter;
}

export function openIndexerWebSocket(
  url: string,
  handlers: IndexerWebSocketHandlers = {},
): IndexerWebSocketClient {
  const client = new IndexerWebSocketClient(url);
  if (handlers.onEvent) client.onEvent(handlers.onEvent);
  if (handlers.onError) client.onError(handlers.onError);
  if (handlers.onClose) client.onClose(handlers.onClose);
  if (handlers.filter) client.subscribe(handlers.filter);
  client.connect();
  return client;
}

export async function* indexerEvents(
  url: string,
  filter?: IndexerWsFilter,
): AsyncGenerator<IndexedEvent> {
  const client = new IndexerWebSocketClient(url);
  const queue: IndexedEvent[] = [];
  let resolve: ((value: IndexedEvent | null) => void) | null = null;
  let done = false;

  client.onEvent((event) => {
    if (resolve) {
      resolve(event);
      resolve = null;
    } else {
      queue.push(event);
    }
  });
  client.onClose(() => {
    done = true;
    if (resolve) {
      resolve(null);
      resolve = null;
    }
  });

  client.connect();
  if (filter) client.subscribe(filter);

  try {
    while (!done) {
      if (queue.length > 0) {
        yield queue.shift() as IndexedEvent;
      } else {
        const event = await new Promise<IndexedEvent | null>((res) => {
          resolve = res;
        });
        if (event === null) break;
        if (event) yield event;
      }
    }
  } finally {
    client.close();
  }
}
