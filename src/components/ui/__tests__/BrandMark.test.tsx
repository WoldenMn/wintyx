import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { BrandMark } from "../BrandMark";

describe("BrandMark", () => {
  it("is decorative for assistive technologies", () => {
    const { container } = render(<BrandMark />);
    const svg = container.querySelector("svg");
    expect(svg?.getAttribute("aria-hidden")).toBe("true");
    expect(svg?.getAttribute("focusable")).toBe("false");
  });

  it("gives each instance its own gradient id", () => {
    const { container } = render(
      <>
        <BrandMark />
        <BrandMark />
      </>,
    );
    const ids = Array.from(container.querySelectorAll("linearGradient")).map((g) => g.id);
    expect(ids).toHaveLength(2);
    expect(new Set(ids).size).toBe(2);
    for (const id of ids) {
      expect(id).toMatch(/^[a-zA-Z0-9_-]+$/);
      expect(container.querySelector(`polyline[stroke="url(#${id})"]`)).not.toBeNull();
    }
  });

  it("keeps the 720 by 520 proportions of the mark", () => {
    const { container } = render(<BrandMark size={72} />);
    const svg = container.querySelector("svg");
    expect(svg?.getAttribute("width")).toBe("72");
    expect(svg?.getAttribute("height")).toBe("52");
  });
});
