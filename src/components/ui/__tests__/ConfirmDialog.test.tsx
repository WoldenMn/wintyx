import { describe, it, expect } from "vitest";
import { render, act, fireEvent } from "@testing-library/react";
import { ConfirmProvider, useConfirm } from "../ConfirmDialog";

function Trigger({ onDone }: { onDone: (v: boolean) => void }) {
  const confirm = useConfirm();
  return (
    <button
      onClick={() => {
        void confirm({ title: "Supprimer", message: "Confirmer ?" }).then(onDone);
      }}
    >
      open
    </button>
  );
}

function DangerTrigger({ onDone }: { onDone: (v: boolean) => void }) {
  const confirm = useConfirm();
  return (
    <button
      onClick={() => {
        void confirm({ title: "Supprimer", message: "Confirmer ?", variant: "danger" }).then(onDone);
      }}
    >
      open
    </button>
  );
}

describe("ConfirmDialog", () => {
  it("renders with role=dialog aria-modal aria-labelledby aria-describedby", () => {
    const { getByText, getByRole } = render(
      <ConfirmProvider>
        <Trigger onDone={() => {}} />
      </ConfirmProvider>,
    );

    act(() => {
      fireEvent.click(getByText("open"));
    });

    const dialog = getByRole("dialog");
    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(dialog.getAttribute("aria-labelledby")).toBe("confirm-title");
    expect(dialog.getAttribute("aria-describedby")).toBe("confirm-desc");
  });

  it("closes with Escape pressed in the dialog and resolves false", async () => {
    let resolvedWith: boolean | null = null;
    const { getByText } = render(
      <ConfirmProvider>
        <Trigger onDone={(v) => { resolvedWith = v; }} />
      </ConfirmProvider>,
    );

    act(() => {
      fireEvent.click(getByText("open"));
    });

    await act(async () => {
      fireEvent.keyDown(document.activeElement as Element, { key: "Escape" });
      await Promise.resolve();
    });

    expect(resolvedWith).toBe(false);
  });

  it("Enter activates the focused button : Confirmer by default", async () => {
    let resolvedWith: boolean | null = null;
    const { getByText, getByRole } = render(
      <ConfirmProvider>
        <Trigger onDone={(v) => { resolvedWith = v; }} />
      </ConfirmProvider>,
    );

    act(() => {
      fireEvent.click(getByText("open"));
    });
    expect(document.activeElement).toBe(getByRole("button", { name: "Confirmer" }));

    await act(async () => {
      fireEvent.keyDown(document.activeElement as Element, { key: "Enter" });
      await Promise.resolve();
    });

    expect(resolvedWith).toBe(true);
  });

  it("ignores a held Enter key that repeats into the dialog", async () => {
    let resolvedWith: boolean | null = null;
    const { getByText, getByRole } = render(
      <ConfirmProvider>
        <Trigger onDone={(v) => { resolvedWith = v; }} />
      </ConfirmProvider>,
    );

    act(() => {
      fireEvent.click(getByText("open"));
    });

    await act(async () => {
      fireEvent.keyDown(document.activeElement as Element, { key: "Enter", repeat: true });
      await Promise.resolve();
    });

    expect(resolvedWith).toBeNull();
    expect(getByRole("dialog")).toBeInTheDocument();
  });

  it("Enter on Annuler cancels instead of confirming", async () => {
    let resolvedWith: boolean | null = null;
    const { getByText, getByRole } = render(
      <ConfirmProvider>
        <Trigger onDone={(v) => { resolvedWith = v; }} />
      </ConfirmProvider>,
    );

    act(() => {
      fireEvent.click(getByText("open"));
    });
    const cancel = getByRole("button", { name: "Annuler" });
    act(() => cancel.focus());

    await act(async () => {
      fireEvent.keyDown(cancel, { key: "Enter" });
      await Promise.resolve();
    });

    expect(resolvedWith).toBe(false);
  });

  it("danger dialogs focus Annuler by default and Enter cancels", async () => {
    let resolvedWith: boolean | null = null;
    const { getByText, getByRole } = render(
      <ConfirmProvider>
        <DangerTrigger onDone={(v) => { resolvedWith = v; }} />
      </ConfirmProvider>,
    );

    act(() => {
      fireEvent.click(getByText("open"));
    });
    expect(document.activeElement).toBe(getByRole("button", { name: "Annuler" }));

    await act(async () => {
      fireEvent.keyDown(document.activeElement as Element, { key: "Enter" });
      await Promise.resolve();
    });

    expect(resolvedWith).toBe(false);
  });

  it("ignores Enter and Escape pressed outside the dialog (command palette)", async () => {
    let resolvedWith: boolean | null = null;
    const { getByText, getByRole } = render(
      <ConfirmProvider>
        <Trigger onDone={(v) => { resolvedWith = v; }} />
        <input aria-label="palette" />
      </ConfirmProvider>,
    );

    act(() => {
      fireEvent.click(getByText("open"));
    });
    const outside = getByRole("textbox", { name: "palette" });

    await act(async () => {
      fireEvent.keyDown(outside, { key: "Enter" });
      fireEvent.keyDown(outside, { key: "Escape" });
      fireEvent.keyDown(window, { key: "Enter" });
      await Promise.resolve();
    });

    expect(resolvedWith).toBeNull();
    expect(getByRole("dialog")).toBeInTheDocument();
  });

  it("traps Tab forward : cycles back to first focusable", () => {
    const { getByText, getByRole } = render(
      <ConfirmProvider>
        <Trigger onDone={() => {}} />
      </ConfirmProvider>,
    );

    act(() => {
      fireEvent.click(getByText("open"));
    });

    const dialog = getByRole("dialog");
    const focusables = dialog.querySelectorAll<HTMLButtonElement>("button");
    expect(focusables.length).toBeGreaterThanOrEqual(2);
    const first = focusables[0];
    const last = focusables[focusables.length - 1];

    last.focus();
    expect(document.activeElement).toBe(last);
    act(() => {
      fireEvent.keyDown(last, { key: "Tab" });
    });
    expect(document.activeElement).toBe(first);
  });
});
