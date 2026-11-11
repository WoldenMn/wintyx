import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import {
  Skeleton,
  CardSkeleton,
  ListSkeleton,
  PageSkeleton,
} from "../Skeleton";

describe("Skeleton", () => {
  it("renders with default dimensions", () => {
    const { container } = render(<Skeleton />);
    const el = container.firstChild as HTMLElement;
    expect(el.style.width).toBe("100%");
    expect(el.style.height).toBe("1rem");
  });

  it("accepts custom dimensions", () => {
    const { container } = render(<Skeleton width="50%" height="2rem" />);
    const el = container.firstChild as HTMLElement;
    expect(el.style.width).toBe("50%");
    expect(el.style.height).toBe("2rem");
  });

  it("applies rounded-full when rounded=true", () => {
    const { container } = render(<Skeleton rounded />);
    expect(container.querySelector(".rounded-full")).toBeTruthy();
  });

  it("has aria-hidden for accessibility", () => {
    const { container } = render(<Skeleton />);
    expect(container.firstChild).toHaveAttribute("aria-hidden", "true");
  });
});

describe("CardSkeleton", () => {
  it("renders with 2 lines by default", () => {
    const { container } = render(<CardSkeleton />);
    // 1 title skeleton + 2 line skeletons = 3
    const skeletons = container.querySelectorAll(".animate-pulse");
    expect(skeletons.length).toBe(3);
  });

  it("renders custom number of lines", () => {
    const { container } = render(<CardSkeleton lines={4} />);
    const skeletons = container.querySelectorAll(".animate-pulse");
    expect(skeletons.length).toBe(5);
  });
});

describe("ListSkeleton", () => {
  it("renders 5 items by default", () => {
    const { container } = render(<ListSkeleton />);
    const items = container.querySelectorAll(".theme-card");
    expect(items.length).toBe(5);
  });

  it("renders custom count", () => {
    const { container } = render(<ListSkeleton count={3} />);
    expect(container.querySelectorAll(".theme-card").length).toBe(3);
  });

  it("has aria-busy for screen readers", () => {
    const { container } = render(<ListSkeleton />);
    expect(container.querySelector('[aria-busy="true"]')).toBeTruthy();
  });
});

describe("PageSkeleton", () => {
  it("renders list variant by default", () => {
    const { container } = render(<PageSkeleton />);
    expect(container.querySelectorAll(".theme-card").length).toBe(6);
  });

  it("renders grid variant", () => {
    const { container } = render(<PageSkeleton kind="grid" count={3} />);
    expect(container.querySelector(".grid-cols-3")).toBeTruthy();
  });

  it("renders dashboard variant", () => {
    const { container } = render(<PageSkeleton kind="dashboard" />);
    expect(container.querySelector(".grid-cols-5")).toBeTruthy();
    expect(container.querySelector(".grid-cols-2")).toBeTruthy();
  });
});
