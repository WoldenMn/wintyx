import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ItemErrorBoundary } from "../ItemErrorBoundary";

describe("ItemErrorBoundary", () => {
  it("rend le contenu quand render ne throw pas", () => {
    render(<ItemErrorBoundary render={() => <span>Contenu OK</span>} />);
    expect(screen.getByText("Contenu OK")).toBeInTheDocument();
  });

  it("affiche un fallback discret quand render throw, sans propager (fvol-12)", () => {
    const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    render(
      <ItemErrorBoundary
        render={() => {
          throw new Error("carte buggee");
        }}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Cet élément n'a pas pu s'afficher",
    );
    errSpy.mockRestore();
  });
});
