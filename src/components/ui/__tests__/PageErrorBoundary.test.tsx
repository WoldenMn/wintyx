import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, act } from "@testing-library/react";
import { useState } from "react";
import { PageErrorBoundary } from "../PageErrorBoundary";
import { reportFrontendError } from "../../../lib/frontend-log";

vi.mock("../../../lib/frontend-log", () => ({
  reportFrontendError: vi.fn(),
}));

function Boom({ shouldThrow }: { shouldThrow: boolean }) {
  if (shouldThrow) throw new Error("kaboom");
  return <div>contenu sain</div>;
}

let errorSpy: ReturnType<typeof vi.spyOn>;

beforeEach(() => {
  errorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
});

afterEach(() => {
  errorSpy.mockRestore();
});

describe("PageErrorBoundary", () => {
  it("renders children when no error", () => {
    render(
      <PageErrorBoundary pageName="Test">
        <Boom shouldThrow={false} />
      </PageErrorBoundary>,
    );
    expect(screen.getByText("contenu sain")).toBeInTheDocument();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("catches render errors and shows fallback with alert role", () => {
    render(
      <PageErrorBoundary pageName="BuggyPage">
        <Boom shouldThrow={true} />
      </PageErrorBoundary>,
    );
    const alert = screen.getByRole("alert");
    expect(alert).toBeInTheDocument();
    expect(screen.getByText(/Cette section n'a pas pu charger/i)).toBeInTheDocument();
    expect(screen.getByText("BuggyPage")).toBeInTheDocument();
    expect(screen.getByText(/kaboom/)).toBeInTheDocument();
  });

  it("calls componentDidCatch logging the error", () => {
    render(
      <PageErrorBoundary pageName="Test">
        <Boom shouldThrow={true} />
      </PageErrorBoundary>,
    );
    const calls = errorSpy.mock.calls.map((c: unknown[]) => String(c[0]));
    const hasBoundaryLog = calls.some((c: string) =>
      c.includes("[PageErrorBoundary:Test]"),
    );
    expect(hasBoundaryLog).toBe(true);
  });

  it("journalise l'erreur de rendu avec le nom de la page", () => {
    render(
      <PageErrorBoundary pageName="BuggyPage">
        <Boom shouldThrow={true} />
      </PageErrorBoundary>,
    );
    expect(reportFrontendError).toHaveBeenCalledWith(
      "PageErrorBoundary:BuggyPage",
      expect.objectContaining({ message: "kaboom" }),
    );
  });

  it("reset button clears error and re-renders children", () => {
    function Wrapper() {
      const [throwIt, setThrowIt] = useState(true);
      return (
        <PageErrorBoundary
          pageName="Test"
          onReset={() => setThrowIt(false)}
        >
          <Boom shouldThrow={throwIt} />
        </PageErrorBoundary>
      );
    }
    render(<Wrapper />);
    expect(screen.getByRole("alert")).toBeInTheDocument();

    act(() => {
      screen.getByRole("button", { name: /réessayer/i }).click();
    });

    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByText("contenu sain")).toBeInTheDocument();
  });

  it("resets error state when parent forces remount via key", () => {
    // App.tsx utilise `key={currentPage}` sur PageErrorBoundary, ce qui force
    // React a unmount/remount le composant a chaque changement de page.
    // Le state est donc reset naturellement via le lifecycle React.
    const { rerender } = render(
      <PageErrorBoundary key="PageA" pageName="PageA">
        <Boom shouldThrow={true} />
      </PageErrorBoundary>,
    );
    expect(screen.getByRole("alert")).toBeInTheDocument();

    // Changer la key force le remount (simule une navigation dans App.tsx)
    rerender(
      <PageErrorBoundary key="PageB" pageName="PageB">
        <Boom shouldThrow={false} />
      </PageErrorBoundary>,
    );

    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByText("contenu sain")).toBeInTheDocument();
  });
});
