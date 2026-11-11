import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ErrorState } from "../ErrorState";
import { NavigationProvider } from "../../../contexts/NavigationContext";

// ErrorState utilise ActionButton -> useAdminOptional, qui retombe sur des
// defauts safe sans AdminProvider : aucun provider ni mock invoke necessaire.

describe("ErrorState", () => {
  it("affiche le message d'erreur avec role alert", () => {
    render(<ErrorState message="Service WMI indisponible" />);
    expect(screen.getByRole("alert")).toBeInTheDocument();
    expect(screen.getByText("Service WMI indisponible")).toBeInTheDocument();
  });

  it("appelle onRetry au clic sur Reessayer", () => {
    const onRetry = vi.fn();
    render(<ErrorState message="Echec" onRetry={onRetry} />);
    fireEvent.click(screen.getByRole("button", { name: "Réessayer" }));
    expect(onRetry).toHaveBeenCalledOnce();
  });

  it("n'affiche aucun bouton sans onRetry", () => {
    render(<ErrorState message="Echec" />);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});

describe("ErrorState — lien Diagnostiquer", () => {
  it("navigue vers diagnostic au clic quand un NavigationProvider est present", () => {
    const nav = vi.fn();
    render(
      <NavigationProvider navigate={nav}>
        <ErrorState message="WMI indisponible" onRetry={() => {}} />
      </NavigationProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: /diagnostiquer/i }));
    expect(nav).toHaveBeenCalledWith("diagnostic");
  });

  it("n'affiche pas le lien Diagnostiquer hors NavigationProvider", () => {
    render(<ErrorState message="x" />);
    expect(screen.queryByRole("button", { name: /diagnostiquer/i })).toBeNull();
  });
});
