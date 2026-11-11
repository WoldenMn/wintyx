import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { InfoTooltip } from "../InfoTooltip";

describe("InfoTooltip", () => {
  it("renders nothing when no text or textId", () => {
    const { container } = render(<InfoTooltip />);
    expect(container.firstChild).toBeNull();
  });

  it("resolves textId from help.ts", () => {
    render(<InfoTooltip textId="smbv1" />);
    const btn = screen.getByRole("button", { name: /aide/i });
    expect(btn).toBeInTheDocument();
  });

  it("falls back to text prop when textId not found", () => {
    render(<InfoTooltip textId="inexistant" text="Texte direct" />);
    const btn = screen.getByRole("button");
    fireEvent.mouseEnter(btn);
    expect(screen.getByText("Texte direct")).toBeInTheDocument();
  });

  it("returns null when textId not found and no text", () => {
    const { container } = render(<InfoTooltip textId="inexistant" />);
    expect(container.firstChild).toBeNull();
  });

  it("shows tooltip on mouseEnter and hides on mouseLeave", () => {
    render(<InfoTooltip textId="smbv1" />);
    const btn = screen.getByRole("button");
    fireEvent.mouseEnter(btn);
    expect(screen.getByRole("tooltip")).toBeInTheDocument();
    fireEvent.mouseLeave(btn);
    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("shows tooltip on focus and hides on blur", () => {
    render(<InfoTooltip textId="smbv1" />);
    const btn = screen.getByRole("button");
    fireEvent.focus(btn);
    expect(screen.getByRole("tooltip")).toBeInTheDocument();
    fireEvent.blur(btn);
    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("toggles tooltip on click", () => {
    render(<InfoTooltip textId="smbv1" />);
    const btn = screen.getByRole("button");
    fireEvent.click(btn);
    expect(screen.getByRole("tooltip")).toBeInTheDocument();
    fireEvent.click(btn);
    expect(screen.queryByRole("tooltip")).toBeNull();
  });

  it("uses custom ariaLabel when provided", () => {
    render(<InfoTooltip textId="smbv1" ariaLabel="Aide SMB" />);
    expect(screen.getByRole("button", { name: "Aide SMB" })).toBeInTheDocument();
  });

  it("aria-describedby links to tooltip id when open", () => {
    render(<InfoTooltip textId="smbv1" />);
    const btn = screen.getByRole("button");
    fireEvent.click(btn);
    const tip = screen.getByRole("tooltip");
    expect(btn.getAttribute("aria-describedby")).toBe(tip.id);
  });
});
