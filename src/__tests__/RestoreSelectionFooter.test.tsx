import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { RestoreSelectionFooter } from "../components/ui/RestoreSelectionFooter";

describe("RestoreSelectionFooter", () => {
  it("renders nothing when count is 0", () => {
    const { container } = render(
      <RestoreSelectionFooter
        count={0}
        total={10}
        onClearAll={() => {}}
        onRestore={() => {}}
        isRestoring={false}
      />,
    );
    expect(container.firstChild).toBeNull();
  });

  it("displays counter correctly", () => {
    render(
      <RestoreSelectionFooter
        count={3}
        total={10}
        onClearAll={() => {}}
        onRestore={() => {}}
        isRestoring={false}
      />,
    );
    expect(screen.getByText(/3.*10/)).toBeInTheDocument();
  });

  it("calls onClearAll when clear button clicked", () => {
    const onClearAll = vi.fn();
    render(
      <RestoreSelectionFooter
        count={3}
        total={10}
        onClearAll={onClearAll}
        onRestore={() => {}}
        isRestoring={false}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /Tout désélectionner/i }));
    expect(onClearAll).toHaveBeenCalled();
  });

  it("calls onRestore when restore button clicked", () => {
    const onRestore = vi.fn();
    render(
      <RestoreSelectionFooter
        count={3}
        total={10}
        onClearAll={() => {}}
        onRestore={onRestore}
        isRestoring={false}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /Restaurer/i }));
    expect(onRestore).toHaveBeenCalled();
  });

  it("disables restore button when isRestoring is true", () => {
    render(
      <RestoreSelectionFooter
        count={3}
        total={10}
        onClearAll={() => {}}
        onRestore={() => {}}
        isRestoring={true}
      />,
    );
    expect(screen.getByRole("button", { name: /Restaurer/i })).toBeDisabled();
  });
});
