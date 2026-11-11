import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { RootErrorBoundary } from "../RootErrorBoundary";
import { reportFrontendError } from "../../../lib/frontend-log";

vi.mock("../../../lib/frontend-log", () => ({
  reportFrontendError: vi.fn(),
}));

function Boom(): never {
  throw new Error("crash racine");
}

let errorSpy: ReturnType<typeof vi.spyOn>;

beforeEach(() => {
  vi.clearAllMocks();
  errorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
});

afterEach(() => {
  errorSpy.mockRestore();
});

describe("RootErrorBoundary", () => {
  it("renders its children when nothing throws", () => {
    render(
      <RootErrorBoundary>
        <p>ok</p>
      </RootErrorBoundary>,
    );
    expect(screen.getByText("ok")).toBeInTheDocument();
    expect(reportFrontendError).not.toHaveBeenCalled();
  });

  it("shows the message and journals the crash", () => {
    render(
      <RootErrorBoundary>
        <Boom />
      </RootErrorBoundary>,
    );
    expect(screen.getByText("Erreur Wintyx")).toBeInTheDocument();
    expect(screen.getAllByText(/crash racine/).length).toBeGreaterThan(0);
    expect(reportFrontendError).toHaveBeenCalledWith(
      "RootErrorBoundary",
      expect.objectContaining({ message: "crash racine" }),
    );
  });
});
