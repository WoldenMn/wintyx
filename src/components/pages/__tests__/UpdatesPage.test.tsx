import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { UpdatesPage } from "../UpdatesPage";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

function mockBackend(lastUpdate: string | Error | object) {
  mockedInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_update_history") return Promise.resolve([]);
    if (cmd === "get_windows_build") return Promise.resolve("24H2 (Build 26100.1)");
    if (cmd === "get_last_update_date") {
      return typeof lastUpdate === "string" ? Promise.resolve(lastUpdate) : Promise.reject(lastUpdate);
    }
    return Promise.resolve(null);
  });
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("UpdatesPage", () => {
  it("shows the last update date when the backend provides one", async () => {
    mockBackend("2026-09-23");
    render(<UpdatesPage />);
    expect(await screen.findByText("2026-09-23")).toBeInTheDocument();
  });

  it("affiche Indisponible avec la cause quand la date de derniere mise a jour echoue", async () => {
    mockBackend({ code: "ProcessFailed", details: { command: "Get-HotFix", detail: "service arrete" } });
    render(<UpdatesPage />);
    expect(await screen.findByText("Indisponible")).toBeInTheDocument();
    expect(screen.getByText(/service arrete/)).toBeInTheDocument();
    expect(screen.queryByText("Inconnue")).not.toBeInTheDocument();
  });

  it("affiche Indisponible avec la cause quand la version de Windows est illisible", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_update_history") return Promise.resolve([]);
      if (cmd === "get_windows_build") return Promise.reject({ code: "Registry", details: "version illisible" });
      if (cmd === "get_last_update_date") return Promise.resolve("2026-09-23");
      return Promise.resolve(null);
    });
    render(<UpdatesPage />);
    expect(await screen.findByText("Indisponible")).toBeInTheDocument();
    expect(screen.getByText(/version illisible/)).toBeInTheDocument();
    expect(screen.queryByText("Unknown")).not.toBeInTheDocument();
  });

  it("falls back to Inconnue when the backend returns an empty string", async () => {
    mockBackend("");
    render(<UpdatesPage />);
    expect(await screen.findByText("Inconnue")).toBeInTheDocument();
  });
});
