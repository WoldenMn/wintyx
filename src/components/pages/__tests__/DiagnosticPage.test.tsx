import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import DiagnosticPage from "../DiagnosticPage";
import { DiagnosticProvider } from "../../../contexts/DiagnosticContext";
import type { AppDiagnostics } from "../../../lib/bindings";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const DATA: AppDiagnostics = {
  checks: [
    { id: "wmi", label: "WMI joignable", status: "ok", detail: "répond", affectedPages: [] },
    { id: "admin", label: "Droits administrateur", status: "warning", detail: "non élevé", affectedPages: ["Durcissement"] },
  ],
  hasCriticalFailure: false,
};

beforeEach(() => vi.clearAllMocks());

describe("DiagnosticPage", () => {
  it("liste les checks avec leur statut", async () => {
    mockedInvoke.mockResolvedValue(DATA);
    render(<DiagnosticProvider><DiagnosticPage /></DiagnosticProvider>);
    await waitFor(() => expect(screen.getByText("WMI joignable")).toBeInTheDocument());
    expect(screen.getByText("Droits administrateur")).toBeInTheDocument();
    expect(screen.getByText(/non élevé/)).toBeInTheDocument();
  });

  it("relance le diagnostic au clic sur Relancer", async () => {
    mockedInvoke.mockResolvedValue(DATA);
    render(<DiagnosticProvider><DiagnosticPage /></DiagnosticProvider>);
    await waitFor(() => expect(screen.getByText("WMI joignable")).toBeInTheDocument());
    mockedInvoke.mockClear();
    fireEvent.click(screen.getByRole("button", { name: /relancer/i }));
    await waitFor(() => expect(mockedInvoke).toHaveBeenCalledWith("get_app_diagnostics"));
  });
});
