import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { Contracts } from "./Contracts";
import { TESTNET } from "../network";
import { SessionProvider } from "../session";

class FakeWebSocket {
  static OPEN = 1;
  static CLOSED = 3;
  readyState = 0;
  onopen: (() => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  constructor(public url: string) {}
  send(_data: string) {}
  close() {
    this.readyState = FakeWebSocket.CLOSED;
  }
}

afterEach(() => cleanup());

beforeEach(() => {
  localStorage.clear();
  vi.stubGlobal("WebSocket", FakeWebSocket);
  vi.stubGlobal("fetch", vi.fn(async () => new Response("{}", { status: 503 })));
});

function renderContracts() {
  return render(
    <MemoryRouter>
      <SessionProvider>
        <Contracts />
      </SessionProvider>
    </MemoryRouter>,
  );
}

describe("Contracts page", () => {
  it("renders the contracts heading", () => {
    renderContracts();
    expect(screen.getByText("Contracts")).toBeTruthy();
  });

  it("disables the events section when no indexer is configured", () => {
    renderContracts();
    const loadButton = screen.getByRole("button", { name: /Load events/i });
    expect((loadButton as HTMLButtonElement).disabled).toBe(true);
  });

  it("shows method-specific argument hints", () => {
    renderContracts();
    expect(screen.getAllByText(/Args:/i).length).toBeGreaterThan(0);
  });
});