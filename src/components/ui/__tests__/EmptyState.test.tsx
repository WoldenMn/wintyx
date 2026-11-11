import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { EmptyState } from "../EmptyState";

describe("EmptyState", () => {
  it("renders title", () => {
    render(<EmptyState title="Aucun element" />);
    expect(screen.getByText("Aucun element")).toBeInTheDocument();
  });

  it("expose role status avec aria-live polite (A11Y-07)", () => {
    render(<EmptyState title="Aucun element" />);
    const region = screen.getByRole("status");
    expect(region).toHaveAttribute("aria-live", "polite");
  });

  it("renders description when provided", () => {
    render(<EmptyState title="Vide" description="Essayer autre chose" />);
    expect(screen.getByText("Essayer autre chose")).toBeInTheDocument();
  });

  it("renders hint when provided", () => {
    render(
      <EmptyState
        title="Vide"
        hint="Wintyx scanne via WMI. Les machines virtualisees peuvent ne rien retourner."
      />,
    );
    expect(screen.getByText(/WMI/i)).toBeInTheDocument();
  });

  it("does not render hint paragraph when absent", () => {
    const { container } = render(<EmptyState title="Vide" />);
    // Only title should be present, no hint paragraph with leading-relaxed
    expect(container.querySelector(".leading-relaxed")).toBeNull();
  });

  it("renders icon when provided", () => {
    render(
      <EmptyState
        title="Vide"
        icon={<span data-testid="custom-icon">Icon</span>}
      />,
    );
    expect(screen.getByTestId("custom-icon")).toBeInTheDocument();
  });

  it("accepts hint as ReactNode (JSX)", () => {
    render(
      <EmptyState
        title="Vide"
        hint={
          <>
            Appuyer sur <kbd>Ctrl+K</kbd> pour la palette.
          </>
        }
      />,
    );
    expect(screen.getByText("Ctrl+K")).toBeInTheDocument();
  });
});
