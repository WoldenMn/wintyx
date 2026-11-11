import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { DiagnosticProvider, useDiagnostics, useDiagnosticsOptional } from "../DiagnosticContext";
import type { AppDiagnostics } from "../../lib/bindings";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const OK: AppDiagnostics = { checks: [], hasCriticalFailure: false };
const CRIT: AppDiagnostics = {
  checks: [{ id: "wmi", label: "WMI joignable", status: "failed", detail: "x", affectedPages: ["Système"] }],
  hasCriticalFailure: true,
};

function Probe() {
  const { diagnostics, loaded, error } = useDiagnostics();
  return (
    <div>
      <span data-testid="loaded">{String(loaded)}</span>
      <span data-testid="crit">{String(diagnostics?.hasCriticalFailure ?? "null")}</span>
      <span data-testid="error">{error ?? "none"}</span>
    </div>
  );
}

beforeEach(() => vi.clearAllMocks());

describe("DiagnosticContext", () => {
  it("expose les diagnostics apres resolution", async () => {
    mockedInvoke.mockResolvedValue(OK);
    render(<DiagnosticProvider><Probe /></DiagnosticProvider>);
    await waitFor(() => expect(screen.getByTestId("loaded")).toHaveTextContent("true"));
    expect(screen.getByTestId("crit")).toHaveTextContent("false");
    expect(mockedInvoke).toHaveBeenCalledWith("get_app_diagnostics");
  });

  it("expose hasCriticalFailure sur echec dur", async () => {
    mockedInvoke.mockResolvedValue(CRIT);
    render(<DiagnosticProvider><Probe /></DiagnosticProvider>);
    await waitFor(() => expect(screen.getByTestId("crit")).toHaveTextContent("true"));
  });

  it("expose error et reste non critique si la commande echoue", async () => {
    mockedInvoke.mockRejectedValue({ code: "Generic", details: "boom" });
    render(<DiagnosticProvider><Probe /></DiagnosticProvider>);
    await waitFor(() => expect(screen.getByTestId("loaded")).toHaveTextContent("true"));
    expect(screen.getByTestId("error")).not.toHaveTextContent("none");
    expect(screen.getByTestId("crit")).toHaveTextContent("null");
  });

  it("useDiagnosticsOptional retourne loaded=true hors provider", () => {
    function Probe2() {
      const { loaded, diagnostics } = useDiagnosticsOptional();
      return <span data-testid="opt">{String(loaded)}-{String(diagnostics)}</span>;
    }
    render(<Probe2 />);
    expect(screen.getByTestId("opt")).toHaveTextContent("true-null");
  });
});
