import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ScoreRing } from "../ScoreRing";

describe("ScoreRing", () => {
  it("affiche la valeur arrondie", () => {
    render(<ScoreRing value={81.6} />);
    expect(screen.getByText("82")).toBeInTheDocument();
  });

  it("expose un role img avec aria-label contenant la valeur", () => {
    render(<ScoreRing value={82} ariaLabel="Score de santé : 82 sur 100" />);
    const el = screen.getByRole("img");
    expect(el.getAttribute("aria-label")).toMatch(/82/);
  });

  it("applique la couleur de score success quand value >= 80", () => {
    const { container } = render(<ScoreRing value={90} />);
    expect(container.querySelector(".text-success")).toBeTruthy();
  });

  it("applique la couleur de score danger quand value < 50", () => {
    const { container } = render(<ScoreRing value={20} />);
    expect(container.querySelector(".text-danger")).toBeTruthy();
  });

  it("respecte colorClass quand fourni", () => {
    const { container } = render(<ScoreRing value={95} colorClass="text-info" />);
    expect(container.querySelector(".text-info")).toBeTruthy();
    expect(container.querySelector(".text-success")).toBeNull();
  });

  it("affiche le label central quand fourni", () => {
    render(<ScoreRing value={50} label="CPU" />);
    expect(screen.getByText("CPU")).toBeInTheDocument();
  });

  it("affiche 0 avec la couleur danger pour value=0", () => {
    const { container } = render(<ScoreRing value={0} />);
    expect(screen.getByText("0")).toBeInTheDocument();
    expect(container.querySelector(".text-danger")).toBeTruthy();
  });

  it("affiche 100 avec la couleur success pour value=100", () => {
    const { container } = render(<ScoreRing value={100} />);
    expect(screen.getByText("100")).toBeInTheDocument();
    expect(container.querySelector(".text-success")).toBeTruthy();
  });

  it("borne une valeur negative a 0", () => {
    render(<ScoreRing value={-25} />);
    expect(screen.getByText("0")).toBeInTheDocument();
    expect(screen.getByRole("img").getAttribute("aria-label")).toBe("0 sur 100");
  });

  it("borne une valeur superieure a 100 a 100", () => {
    render(<ScoreRing value={150} />);
    expect(screen.getByText("100")).toBeInTheDocument();
    expect(screen.getByRole("img").getAttribute("aria-label")).toBe("100 sur 100");
  });
});
