import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { CloneProfileDialog } from "../components/ui/CloneProfileDialog";

const presets = [
  { id: "bureau", name: "Bureau", description: "", dataProtectionIds: [], hardeningIds: [], performanceIds: [], isCustom: false },
  { id: "gaming", name: "Gaming", description: "", dataProtectionIds: [], hardeningIds: [], performanceIds: [], isCustom: false },
];

describe("CloneProfileDialog", () => {
  it("renders preset selector and name input", () => {
    render(
      <CloneProfileDialog
        presets={presets}
        onClone={() => {}}
        onCancel={() => {}}
      />,
    );
    expect(screen.getByLabelText(/Source/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Nom/i)).toBeInTheDocument();
  });

  it("clone button disabled when name is empty", () => {
    render(
      <CloneProfileDialog
        presets={presets}
        onClone={() => {}}
        onCancel={() => {}}
      />,
    );
    expect(screen.getByRole("button", { name: /Cloner/i })).toBeDisabled();
  });

  it("calls onClone with sourceId and name", () => {
    const onClone = vi.fn();
    render(
      <CloneProfileDialog
        presets={presets}
        onClone={onClone}
        onCancel={() => {}}
      />,
    );
    fireEvent.change(screen.getByLabelText(/Source/i), { target: { value: "bureau" } });
    fireEvent.change(screen.getByLabelText(/Nom/i), { target: { value: "Mon profil" } });
    fireEvent.click(screen.getByRole("button", { name: /Cloner/i }));
    expect(onClone).toHaveBeenCalledWith("bureau", "Mon profil");
  });

  it("calls onClone once when the button is double-clicked while the clone is pending", async () => {
    let release: () => void = () => {};
    const onClone = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          release = resolve;
        }),
    );
    render(
      <CloneProfileDialog
        presets={presets}
        onClone={onClone}
        onCancel={() => {}}
      />,
    );
    fireEvent.change(screen.getByLabelText(/Source/i), { target: { value: "bureau" } });
    fireEvent.change(screen.getByLabelText(/Nom/i), { target: { value: "Mon profil" } });
    const button = screen.getByRole("button", { name: /Cloner/i });
    fireEvent.click(button);
    fireEvent.click(button);
    expect(onClone).toHaveBeenCalledTimes(1);
    expect(button).toBeDisabled();
    release();
    await waitFor(() => expect(button).not.toBeDisabled());
  });

  it("calls onCancel when cancel clicked", () => {
    const onCancel = vi.fn();
    render(
      <CloneProfileDialog
        presets={presets}
        onClone={() => {}}
        onCancel={onCancel}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /Annuler/i }));
    expect(onCancel).toHaveBeenCalled();
  });

  it("calls onCancel when Escape key pressed", () => {
    const onCancel = vi.fn();
    render(
      <CloneProfileDialog
        presets={presets}
        onClone={() => {}}
        onCancel={onCancel}
      />,
    );
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onCancel).toHaveBeenCalled();
  });
});
