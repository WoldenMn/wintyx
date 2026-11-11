import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { NavigationProvider, useNavigateOptional } from "../NavigationContext";

function Consumer() {
  const navigate = useNavigateOptional();
  return <button onClick={() => navigate("diagnostic")}>go</button>;
}

describe("NavigationContext", () => {
  it("appelle navigate fourni par le provider", () => {
    const spy = vi.fn();
    render(<NavigationProvider navigate={spy}><Consumer /></NavigationProvider>);
    fireEvent.click(screen.getByText("go"));
    expect(spy).toHaveBeenCalledWith("diagnostic");
  });

  it("fournit un no-op hors provider (pas de crash)", () => {
    render(<Consumer />);
    fireEvent.click(screen.getByText("go")); // ne doit pas lever
    expect(screen.getByText("go")).toBeInTheDocument();
  });
});
