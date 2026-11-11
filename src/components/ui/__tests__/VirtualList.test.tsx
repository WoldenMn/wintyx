import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { VirtualList } from "../VirtualList";

function makeItems(n: number): Array<{ id: number; label: string }> {
  return Array.from({ length: n }, (_, i) => ({ id: i, label: `Item ${i}` }));
}

describe("VirtualList", () => {
  it("renders all items below threshold (naive path)", () => {
    const items = makeItems(10);
    render(
      <VirtualList
        items={items}
        itemHeight={40}
        threshold={80}
        render={(item) => <div>{item.label}</div>}
      />,
    );
    expect(screen.getByText("Item 0")).toBeInTheDocument();
    expect(screen.getByText("Item 9")).toBeInTheDocument();
  });

  it("applies ariaLabel to list container", () => {
    render(
      <VirtualList
        items={makeItems(5)}
        itemHeight={40}
        ariaLabel="Liste test"
        render={(item) => <div>{item.label}</div>}
      />,
    );
    expect(screen.getByRole("list", { name: "Liste test" })).toBeInTheDocument();
  });

  it("uses getKey for unique keys", () => {
    const items = makeItems(5);
    const { container } = render(
      <VirtualList
        items={items}
        itemHeight={40}
        getKey={(item) => `custom-${item.id}`}
        render={(item) => <div>{item.label}</div>}
      />,
    );
    expect(container.querySelectorAll('[role="listitem"]').length).toBe(5);
  });

  it("virtualizes above threshold (renders subset)", () => {
    const items = makeItems(200);
    render(
      <VirtualList
        items={items}
        itemHeight={40}
        threshold={80}
        overscan={4}
        render={(item) => <div>{item.label}</div>}
      />,
    );
    const rendered = screen.queryAllByText(/^Item \d+$/);
    // In jsdom clientHeight is 0 so only overscan items are rendered.
    // Verify we do NOT render all 200 items.
    expect(rendered.length).toBeLessThan(200);
    expect(rendered.length).toBeGreaterThan(0);
  });

  it("sets aria-rowcount on virtualized container", () => {
    render(
      <VirtualList
        items={makeItems(150)}
        itemHeight={40}
        threshold={80}
        render={(item) => <div>{item.label}</div>}
      />,
    );
    const list = screen.getByRole("list");
    expect(list.getAttribute("aria-rowcount")).toBe("150");
  });

  it("updates visible window on scroll", () => {
    const items = makeItems(200);
    render(
      <VirtualList
        items={items}
        itemHeight={40}
        threshold={80}
        overscan={2}
        render={(item) => <div>{item.label}</div>}
      />,
    );
    const list = screen.getByRole("list");
    fireEvent.scroll(list, { target: { scrollTop: 4000 } });
    // Apres scroll a 4000px (itemHeight 40 -> index 100), on devrait voir Item 100
    // avec un petit overscan.
    const visible = screen.queryAllByText(/^Item \d+$/);
    const visibleTexts = visible.map((el) => el.textContent ?? "");
    expect(visibleTexts.some((t) => t.startsWith("Item 10"))).toBe(true);
  });

  it("renders empty list without error", () => {
    const { container } = render(
      <VirtualList
        items={[]}
        itemHeight={40}
        render={(item) => <div>{(item as { label: string }).label}</div>}
      />,
    );
    expect(container.querySelectorAll('[role="listitem"]').length).toBe(0);
  });
});
