import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { Sparkline } from "../Sparkline";

describe("Sparkline", () => {
  it("renders svg with role=img and aria-label including current value", () => {
    const { container } = render(
      <Sparkline data={[10, 20, 30, 42]} ariaLabel="CPU" unit="%" />,
    );
    const svg = container.querySelector("svg");
    expect(svg).not.toBeNull();
    expect(svg).toHaveAttribute("role", "img");
    expect(svg?.getAttribute("aria-label")).toContain("CPU");
    expect(svg?.getAttribute("aria-label")).toContain("42");
  });

  it("includes <title> element for screen reader fallback", () => {
    const { container } = render(
      <Sparkline data={[5, 10]} ariaLabel="RAM" />,
    );
    const title = container.querySelector("svg title");
    expect(title).not.toBeNull();
    expect(title?.textContent).toContain("RAM");
  });

  it("handles empty data gracefully with aucune donnee label", () => {
    const { container } = render(<Sparkline data={[]} ariaLabel="Empty" />);
    const svg = container.querySelector("svg");
    expect(svg?.getAttribute("aria-label")).toContain("aucune donnée");
  });
});
