import { afterEach, describe, expect, it, vi } from "vitest";

import {
  IndexerWebSocketClient,
  KovanicaIndexerClient,
  openIndexerWebSocket,
} from "./indexer.js";

afterEach(() => {
  vi.unstubAllGlobals();
});

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

describe("KovanicaIndexerClient", () => {
  it("getStatus fetches /status", async () => {
    const urls: string[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        urls.push(url);
        return jsonResponse({ latest_indexed_height: 5, latest_node_height: 6, event_count: 2 });
      }),
    );
    const client = new KovanicaIndexerClient("http://127.0.0.1:8787/");
    await expect(client.getStatus()).resolves.toEqual({
      latest_indexed_height: 5,
      latest_node_height: 6,
      event_count: 2,
    });
    expect(urls[0]).toBe("http://127.0.0.1:8787/status");
  });

  it("getLogs builds query params", async () => {
    const urls: string[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        urls.push(url);
        return jsonResponse([]);
      }),
    );
    const client = new KovanicaIndexerClient("http://127.0.0.1:8787");
    await client.getLogs({ fromHeight: 1, toHeight: 2, address: "abc", topic0: "0x1234", limit: 10 });
    expect(urls[0]).toContain("/logs?");
    expect(urls[0]).toContain("from_height=1");
    expect(urls[0]).toContain("to_height=2");
    expect(urls[0]).toContain("address=abc");
    expect(urls[0]).toContain("topic=0x1234");
    expect(urls[0]).toContain("limit=10");
  });

  it("getEvents hits /events", async () => {
    const urls: string[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        urls.push(url);
        return jsonResponse([]);
      }),
    );
    const client = new KovanicaIndexerClient("http://127.0.0.1:8787");
    await client.getEvents({ limit: 5 });
    expect(urls[0]).toBe("http://127.0.0.1:8787/events?limit=5");
  });

  it("surfaces HTTP errors", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response("nope", { status: 500 })),
    );
    const client = new KovanicaIndexerClient("http://127.0.0.1:8787");
    await expect(client.getStatus()).rejects.toThrow("HTTP 500");
  });
});

class MockWebSocket {
  static OPEN = 1;
  static CLOSED = 3;
  readyState = 0;
  onopen: ((e: any) => void) | null = null;
  onmessage: ((e: any) => void) | null = null;
  onerror: ((e: any) => void) | null = null;
  onclose: ((e: any) => void) | null = null;
  sent: string[] = [];
  closed = false;
  constructor(public url: string) {
    MockWebSocket.instances.push(this);
  }
  send(data: string) {
    this.sent.push(data);
  }
  close() {
    this.closed = true;
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.({});
  }
  open() {
    this.readyState = MockWebSocket.OPEN;
    this.onopen?.({});
  }
  message(data: unknown) {
    this.onmessage?.({ data: JSON.stringify(data) });
  }
  static instances: MockWebSocket[] = [];
}

describe("IndexerWebSocketClient", () => {
  afterEach(() => {
    MockWebSocket.instances = [];
  });

  it("sends subscribe on open and emits events", () => {
    vi.stubGlobal("WebSocket", MockWebSocket as any);
    const client = new IndexerWebSocketClient("ws://127.0.0.1:8787/ws");
    const events: any[] = [];
    client.onEvent((e) => events.push(e));
    client.subscribe({ address: "abc" });
    client.connect();
    const ws = MockWebSocket.instances[0];
    ws.open();
    expect(ws.sent.length).toBe(1);
    expect(JSON.parse(ws.sent[0])).toEqual({ subscribe: { address: "abc" } });
    const ev = { id: 1, height: 1, tx_hash: [9], tx_index: 0, log_index: 0, address: [1], topic: [2], data: [3], success: true };
    ws.message(ev);
    expect(events).toHaveLength(1);
    expect(events[0].height).toBe(1);
    client.close();
    expect(ws.closed).toBe(true);
  });

  it("openIndexerWebSocket wires handlers", () => {
    vi.stubGlobal("WebSocket", MockWebSocket as any);
    const events: any[] = [];
    const errors: any[] = [];
    const client = openIndexerWebSocket("ws://127.0.0.1:8787/ws", {
      filter: { topics: ["transfer"] },
      onEvent: (e) => events.push(e),
      onError: (e) => errors.push(e),
    });
    const ws = MockWebSocket.instances[0];
    ws.open();
    expect(ws.sent[0]).toContain("transfer");
    ws.message({ id: 2, height: 2, tx_hash: [1], tx_index: 0, log_index: 0, address: [1], topic: [2], data: [], success: true });
    expect(events).toHaveLength(1);
    client.close();
  });
});
