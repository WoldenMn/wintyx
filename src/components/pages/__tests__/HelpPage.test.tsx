import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { HelpPage } from "../HelpPage";

describe("HelpPage", () => {
  it("renders title and subtitle", () => {
    render(<HelpPage />);
    expect(screen.getByText("Aide")).toBeInTheDocument();
    expect(screen.getByText(/Guide utilisateur Wintyx/i)).toBeInTheDocument();
  });

  it("renders all section headers", () => {
    render(<HelpPage />);
    expect(screen.getByRole("button", { name: /Comprendre les scores/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Mode administrateur/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Raccourcis clavier/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Thèmes visuels/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Actions destructives/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Dépannage/i })).toBeInTheDocument();
  });

  it("first section is expanded by default", () => {
    render(<HelpPage />);
    // "Protection des données" n'apparait que dans le contenu de la 1ere section
    expect(screen.getByText(/Protection des données/i)).toBeInTheDocument();
  });

  it("clicking a section toggles its content", () => {
    render(<HelpPage />);
    const adminHeader = screen.getByRole("button", { name: /Mode administrateur/i });
    expect(adminHeader.getAttribute("aria-expanded")).toBe("false");
    fireEvent.click(adminHeader);
    expect(adminHeader.getAttribute("aria-expanded")).toBe("true");
  });

  it("clicking active section collapses it", () => {
    render(<HelpPage />);
    const scoresHeader = screen.getByRole("button", { name: /Comprendre les scores/i });
    expect(scoresHeader.getAttribute("aria-expanded")).toBe("true");
    fireEvent.click(scoresHeader);
    expect(scoresHeader.getAttribute("aria-expanded")).toBe("false");
  });

  it("only one section expanded at a time", () => {
    render(<HelpPage />);
    const adminHeader = screen.getByRole("button", { name: /Mode administrateur/i });
    fireEvent.click(adminHeader);
    const scoresHeader = screen.getByRole("button", { name: /Comprendre les scores/i });
    expect(scoresHeader.getAttribute("aria-expanded")).toBe("false");
    expect(adminHeader.getAttribute("aria-expanded")).toBe("true");
  });
});
