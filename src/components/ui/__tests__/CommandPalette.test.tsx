import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { CommandPalette } from "../CommandPalette";

function open() {
  render(<CommandPalette open onClose={vi.fn()} onNavigate={vi.fn()} />);
  return screen.getByLabelText("Rechercher une page");
}

async function typeQuery(query: string) {
  fireEvent.change(open(), { target: { value: query } });
  await waitFor(() => expect(screen.queryAllByRole("option").length).toBeGreaterThan(0));
}

describe("CommandPalette recherche", () => {
  it("trouve Mémoire avec « mem » (accent ignoré)", async () => {
    await typeQuery("mem");
    expect(screen.getByRole("option", { name: /Mémoire/ })).toBeInTheDocument();
  });

  it("trouve À propos avec « a propos » (majuscule et accent ignorés)", async () => {
    await typeQuery("a propos");
    expect(screen.getByRole("option", { name: /À propos/ })).toBeInTheDocument();
  });

  it("accepte aussi la saisie accentuée et la casse libre", async () => {
    await typeQuery("MÉMOIRE");
    expect(screen.getByRole("option", { name: /Mémoire/ })).toBeInTheDocument();
  });

  it("affiche Aucun résultat pour une requête sans correspondance", async () => {
    fireEvent.change(open(), { target: { value: "zzzzzz" } });
    await waitFor(() => expect(screen.getByText("Aucun résultat")).toBeInTheDocument());
  });
});

describe("CommandPalette clavier", () => {
  it("garde le focus dans la palette quand on presse Tab", () => {
    const input = open();
    input.focus();
    expect(fireEvent.keyDown(input, { key: "Tab" })).toBe(false);
    expect(fireEvent.keyDown(input, { key: "Tab", shiftKey: true })).toBe(false);
  });

  it("annonce l'option active a la fleche bas", () => {
    const input = open();
    const options = screen.getAllByRole("option");
    expect(input).toHaveAttribute("aria-activedescendant", options[0].id);
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(input).toHaveAttribute("aria-activedescendant", options[1].id);
    expect(options[1]).toHaveAttribute("aria-selected", "true");
  });

  it("rend le focus a l'element d'origine a la fermeture", async () => {
    const origin = document.createElement("button");
    document.body.appendChild(origin);
    origin.focus();
    const { rerender } = render(<CommandPalette open onClose={vi.fn()} onNavigate={vi.fn()} />);
    await waitFor(() => expect(document.activeElement).toBe(screen.getByLabelText("Rechercher une page")));
    rerender(<CommandPalette open={false} onClose={vi.fn()} onNavigate={vi.fn()} />);
    expect(document.activeElement).toBe(origin);
    origin.remove();
  });
});
