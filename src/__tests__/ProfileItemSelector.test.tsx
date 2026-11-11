import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ProfileItemSelector } from "../components/ui/ProfileItemSelector";

const items = [
  { id: "dp_0", label: "Telemetrie", category: "Privacy" },
  { id: "dp_1", label: "Cortana", category: "Privacy" },
  { id: "dp_2", label: "Localisation", category: "Tracking" },
];

describe("ProfileItemSelector", () => {
  it("renders title and counter", () => {
    render(
      <ProfileItemSelector
        title="Data Protection"
        items={items}
        selectedIds={new Set(["dp_0"])}
        onToggle={() => {}}
      />,
    );
    expect(screen.getByText(/Data Protection/i)).toBeInTheDocument();
    expect(screen.getByText(/1 \/ 3/)).toBeInTheDocument();
  });

  it("renders all items as checkboxes", () => {
    render(
      <ProfileItemSelector
        title="Test"
        items={items}
        selectedIds={new Set()}
        onToggle={() => {}}
      />,
    );
    const checkboxes = screen.getAllByRole("checkbox");
    expect(checkboxes.length).toBe(3);
  });

  it("calls onToggle when checkbox clicked", () => {
    const onToggle = vi.fn();
    render(
      <ProfileItemSelector
        title="Test"
        items={items}
        selectedIds={new Set()}
        onToggle={onToggle}
      />,
    );
    fireEvent.click(screen.getByLabelText(/Telemetrie/i));
    expect(onToggle).toHaveBeenCalledWith("dp_0");
  });

  it("checkbox is checked when id is in selectedIds", () => {
    render(
      <ProfileItemSelector
        title="Test"
        items={items}
        selectedIds={new Set(["dp_1"])}
        onToggle={() => {}}
      />,
    );
    const cortana = screen.getByLabelText(/Cortana/i) as HTMLInputElement;
    expect(cortana.checked).toBe(true);
  });
});
