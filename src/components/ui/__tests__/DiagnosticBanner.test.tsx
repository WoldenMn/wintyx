import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { DiagnosticBanner } from "../DiagnosticBanner";
import type { AppDiagnostics } from "../../../lib/bindings";

const CRIT: AppDiagnostics = {
  checks: [{ id: "wmi", label: "WMI joignable", status: "failed", detail: "x", affectedPages: ["Système", "Mémoire"] }],
  hasCriticalFailure: true,
};
const OK: AppDiagnostics = { checks: [], hasCriticalFailure: false };

describe("DiagnosticBanner", () => {
  it("s'affiche sur echec critique et navigue au clic", () => {
    const nav = vi.fn();
    render(<DiagnosticBanner diagnostics={CRIT} onDiagnose={nav} />);
    expect(screen.getByRole("alert")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /diagnostiquer/i }));
    expect(nav).toHaveBeenCalled();
  });

  it("ne rend rien sans echec critique", () => {
    const { container } = render(<DiagnosticBanner diagnostics={OK} onDiagnose={() => {}} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("ne rend rien si diagnostics est null (non charge)", () => {
    const { container } = render(<DiagnosticBanner diagnostics={null} onDiagnose={() => {}} />);
    expect(container).toBeEmptyDOMElement();
  });
});
