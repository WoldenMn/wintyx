import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ProfileItemSelector } from "../ProfileItemSelector";
import type { AvailableItem } from "../../../lib/types";

describe("ProfileItemSelector", () => {
  it("affiche la catégorie en français", () => {
    const items = [
      { id: "a", label: "Collecte", category: "Telemetrie" },
      { id: "b", label: "Annonces", category: "Publicite" },
    ] as AvailableItem[];
    render(<ProfileItemSelector title="Protection" items={items} selectedIds={new Set()} onToggle={vi.fn()} />);
    expect(screen.getByText("Télémétrie")).toBeInTheDocument();
    expect(screen.getByText("Publicité")).toBeInTheDocument();
    expect(screen.queryByText("Telemetrie")).not.toBeInTheDocument();
  });
});
