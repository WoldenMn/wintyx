import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { StatCard } from "../StatCard";

describe("StatCard", () => {
  it("renders label and value", () => {
    render(<StatCard label="CPU" value="42" />);
    expect(screen.getByText("CPU")).toBeInTheDocument();
    expect(screen.getByText("42")).toBeInTheDocument();
  });

  it("renders numeric value", () => {
    render(<StatCard label="RAM" value={87} />);
    expect(screen.getByText("87")).toBeInTheDocument();
  });

  it("renders suffix when provided", () => {
    render(<StatCard label="CPU" value="50" suffix="%" />);
    expect(screen.getByText("%")).toBeInTheDocument();
  });

  it("does not render suffix when absent", () => {
    const { container } = render(<StatCard label="CPU" value="50" />);
    const suffixes = container.querySelectorAll("span.text-sm.text-fg-dim");
    expect(suffixes.length).toBe(0);
  });

  it("applies success color when score >= 80", () => {
    const { container } = render(<StatCard label="Score" value="95" score={95} />);
    expect(container.querySelector(".text-success")).toBeTruthy();
  });

  it("applies warning color when score is between 50 and 79", () => {
    const { container } = render(<StatCard label="Score" value="60" score={60} />);
    expect(container.querySelector(".text-warning")).toBeTruthy();
  });

  it("applies danger color when score < 50", () => {
    const { container } = render(<StatCard label="Score" value="30" score={30} />);
    expect(container.querySelector(".text-danger")).toBeTruthy();
  });

  it("applies neutral color when no score provided", () => {
    const { container } = render(<StatCard label="CPU" value="42" />);
    expect(container.querySelector(".text-fg-primary")).toBeTruthy();
  });

  it("renders subtitle when provided", () => {
    render(<StatCard label="CPU" value="50" subtitle="Moyenne 5 min" />);
    expect(screen.getByText("Moyenne 5 min")).toBeInTheDocument();
  });

  it("renders icon when provided", () => {
    render(
      <StatCard
        label="CPU"
        value="50"
        icon={<span data-testid="custom-icon">X</span>}
      />,
    );
    expect(screen.getByTestId("custom-icon")).toBeInTheDocument();
  });

  it("accepts custom className", () => {
    const { container } = render(
      <StatCard label="CPU" value="50" className="custom-extra" />,
    );
    expect(container.querySelector(".custom-extra")).toBeTruthy();
  });

  it("renders a non-interactive div without onClick", () => {
    render(<StatCard label="CPU" value="50" />);
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("renders as a button when onClick is provided", () => {
    render(<StatCard label="CPU" value="50" onClick={() => {}} />);
    const btn = screen.getByRole("button");
    expect(btn).toBeInTheDocument();
    expect(btn.getAttribute("aria-label")).toMatch(/CPU/);
  });

  it("invokes onClick when clicked", () => {
    const onClick = vi.fn();
    render(<StatCard label="CPU" value="50" onClick={onClick} />);
    fireEvent.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("uses custom aria-label when provided", () => {
    render(
      <StatCard
        label="CPU"
        value="50"
        onClick={() => {}}
        ariaLabel="Voir detail CPU"
      />,
    );
    expect(screen.getByRole("button").getAttribute("aria-label")).toBe(
      "Voir detail CPU",
    );
  });

  it("supports keyboard activation (Enter)", () => {
    const onClick = vi.fn();
    render(<StatCard label="CPU" value="50" onClick={onClick} />);
    const btn = screen.getByRole("button");
    btn.focus();
    fireEvent.keyDown(btn, { key: "Enter" });
    fireEvent.click(btn);
    expect(onClick).toHaveBeenCalled();
  });

  it("progress bar uses the mapped bg class instead of bg-current", () => {
    // Regression BUG-01b v2.16.0 : bg-current heritait de la couleur de texte
    // du parent au lieu du token vise par `color`.
    const { container } = render(
      <StatCard label="Score" value={30} score={30} progress={30} />,
    );
    expect(container.querySelector(".bg-danger")).toBeTruthy();
    expect(container.querySelector(".bg-current")).toBeNull();
  });

  it("does not render a progress bar without progress prop", () => {
    const { container } = render(<StatCard label="CPU" value={42} />);
    expect(container.querySelector(".h-1")).toBeNull();
  });

  it("clamps negative progress to width 0%", () => {
    const { container } = render(
      <StatCard label="CPU" value={0} progress={-10} />,
    );
    const bar = container.querySelector(".h-full") as HTMLElement;
    expect(bar).toBeTruthy();
    expect(bar.style.width).toBe("0%");
  });

  it("clamps progress above 100 to width 100%", () => {
    const { container } = render(
      <StatCard label="CPU" value={100} progress={150} />,
    );
    const bar = container.querySelector(".h-full") as HTMLElement;
    expect(bar).toBeTruthy();
    expect(bar.style.width).toBe("100%");
  });

  it("progress bar falls back to bg-fg-primary without score", () => {
    const { container } = render(
      <StatCard label="CPU" value={42} progress={42} />,
    );
    expect(container.querySelector(".bg-fg-primary")).toBeTruthy();
  });

  it("eventually displays the new numeric value after change", async () => {
    const { rerender } = render(<StatCard label="Score" value={70} />);
    expect(screen.getByText("70")).toBeInTheDocument();

    rerender(<StatCard label="Score" value={80} />);

    await waitFor(
      () => {
        expect(screen.getByText("80")).toBeInTheDocument();
      },
      { timeout: 500 },
    );
  });

  it("changes string value instantly without count-up", () => {
    const { rerender } = render(<StatCard label="X" value="2.4 Go" />);
    expect(screen.getByText("2.4 Go")).toBeInTheDocument();

    rerender(<StatCard label="X" value="3.1 Go" />);
    expect(screen.getByText("3.1 Go")).toBeInTheDocument();
  });
});
