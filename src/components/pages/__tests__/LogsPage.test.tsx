import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { LogsPage, extractCategory } from "../LogsPage";
import type { LogEntry } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const FAKE_LOGS: LogEntry[] = [
  { level: "info", line: "[2026-04-24 10:00:00] [INFO] [services] Service stopped" },
  { level: "warn", line: "[2026-04-24 10:00:01] [WARN] [monitor] Slow WMI query" },
  { level: "error", line: "[2026-04-24 10:00:02] [ERROR] [backup] Failed to export" },
  { level: "info", line: "[2026-04-24 10:00:03] [INFO] [tauri] App started" },
  { level: "info", line: "[2026-04-24 10:00:04] [INFO] [hardening] Applied SMBv1" },
];

beforeEach(() => {
  vi.clearAllMocks();
  mockedInvoke.mockResolvedValue(FAKE_LOGS);
});

describe("extractCategory", () => {
  it("extracts category from standard format", () => {
    expect(
      extractCategory("[2026-04-24 10:00:00] [INFO] [services] Service stopped"),
    ).toBe("services");
  });

  it("returns null when format does not match", () => {
    expect(extractCategory("plain text line")).toBeNull();
  });

  it("lowercases category", () => {
    expect(
      extractCategory("[2026-04-24 10:00:00] [INFO] [BACKUP] msg"),
    ).toBe("backup");
  });
});

describe("LogsPage", () => {
  it("initial fetch uses levelFilter=all", async () => {
    render(<LogsPage />);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_logs", {
        levelFilter: "all",
        limit: 200,
      });
    });
  });

  it("exposes all / info / warn / error options in level select", async () => {
    render(<LogsPage />);

    await waitFor(() => {
      expect(screen.getAllByText(/\[INFO\]|\[WARN\]|\[ERR /)[0]).toBeInTheDocument();
    });

    const levelSelect = screen.getByLabelText("Niveau de log") as HTMLSelectElement;
    const values = Array.from(levelSelect.options).map((o) => o.value);
    expect(values).toEqual(["all", "info", "warn", "error"]);
  });

  it("exposes all / actions options in scope select", async () => {
    render(<LogsPage />);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalled();
    });

    const scopeSelect = screen.getByLabelText("Portée des logs") as HTMLSelectElement;
    const values = Array.from(scopeSelect.options).map((o) => o.value);
    expect(values).toEqual(["all", "actions"]);
  });

  it("garde la page et le filtre a l'ecran pendant le rechargement", async () => {
    render(<LogsPage />);
    const levelSelect = await screen.findByLabelText("Niveau de log");

    mockedInvoke.mockReturnValue(new Promise(() => {}));
    fireEvent.change(levelSelect, { target: { value: "warn" } });

    expect(screen.getByLabelText("Niveau de log")).toBe(levelSelect);
    expect(levelSelect).toHaveValue("warn");
  });

  it("refetches with new level filter when user changes selection", async () => {
    render(<LogsPage />);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_logs", {
        levelFilter: "all",
        limit: 200,
      });
    });

    const levelSelect = screen.getByLabelText("Niveau de log");
    fireEvent.change(levelSelect, { target: { value: "error" } });

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_logs", {
        levelFilter: "error",
        limit: 200,
      });
    });
  });

  it("actions scope filters out non-action categories", async () => {
    render(<LogsPage />);

    await waitFor(() => {
      expect(screen.getByText(/Service stopped/)).toBeInTheDocument();
    });
    expect(screen.getByText(/Slow WMI query/)).toBeInTheDocument();
    expect(screen.getByText(/App started/)).toBeInTheDocument();

    const scopeSelect = screen.getByLabelText("Portée des logs");
    fireEvent.change(scopeSelect, { target: { value: "actions" } });

    await waitFor(() => {
      expect(screen.queryByText(/Slow WMI query/)).toBeNull();
    });
    expect(screen.queryByText(/App started/)).toBeNull();
    expect(screen.getByText(/Service stopped/)).toBeInTheDocument();
    expect(screen.getByText(/Failed to export/)).toBeInTheDocument();
    expect(screen.getByText(/Applied SMBv1/)).toBeInTheDocument();
  });

  it("actions scope drops reads logged under a .lecture category", async () => {
    mockedInvoke.mockResolvedValue([
      { level: "warn", line: "[2026-04-24 10:00:00] [WARN ] [cleanup.lecture] get_folder_size: lecture impossible" },
      { level: "info", line: "[2026-04-24 10:00:01] [INFO ] [cleanup] Nettoyage termine" },
    ] satisfies LogEntry[]);
    render(<LogsPage />);

    await waitFor(() => {
      expect(screen.getByText(/lecture impossible/)).toBeInTheDocument();
    });
    fireEvent.change(screen.getByLabelText("Portée des logs"), { target: { value: "actions" } });

    await waitFor(() => {
      expect(screen.queryByText(/lecture impossible/)).toBeNull();
    });
    expect(screen.getByText(/Nettoyage termine/)).toBeInTheDocument();
  });
});
