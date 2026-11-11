import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, act } from "@testing-library/react";
import { ToastProvider, useToast } from "../Toast";
import { reportFrontendError } from "../../../lib/frontend-log";

vi.mock("../../../lib/frontend-log", () => ({
  reportFrontendError: vi.fn(),
}));

function TriggerButton({ type }: { type: "info" | "error" | "success" | "warning" }) {
  const { info, error, success, warning } = useToast();
  const trigger = () => {
    if (type === "info") info("Message info");
    if (type === "error") error("Message erreur");
    if (type === "success") success("Message succes");
    if (type === "warning") warning("Message warning");
  };
  return <button onClick={trigger}>Trigger {type}</button>;
}

beforeEach(() => {
  vi.mocked(reportFrontendError).mockClear();
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("ToastProvider", () => {
  it("error toast uses role=alert and aria-live=assertive", async () => {
    render(
      <ToastProvider>
        <TriggerButton type="error" />
      </ToastProvider>,
    );

    await act(async () => {
      screen.getByText("Trigger error").click();
    });

    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent("Message erreur");
    expect(alert).toHaveAttribute("aria-live", "assertive");
  });

  it("journalise les toasts d'erreur et seulement eux", async () => {
    render(
      <ToastProvider>
        <TriggerButton type="error" />
        <TriggerButton type="warning" />
        <TriggerButton type="success" />
      </ToastProvider>,
    );

    await act(async () => {
      screen.getByText("Trigger warning").click();
      screen.getByText("Trigger success").click();
    });
    expect(reportFrontendError).not.toHaveBeenCalled();

    await act(async () => {
      screen.getByText("Trigger error").click();
    });
    expect(reportFrontendError).toHaveBeenCalledWith("toast", "Message erreur");
  });

  it("info toast uses role=status and aria-live=polite", async () => {
    render(
      <ToastProvider>
        <TriggerButton type="info" />
      </ToastProvider>,
    );

    await act(async () => {
      screen.getByText("Trigger info").click();
    });

    const status = screen.getByRole("status");
    expect(status).toHaveTextContent("Message info");
    expect(status).toHaveAttribute("aria-live", "polite");
  });

  it("notification container has aria-label and role=region", async () => {
    render(
      <ToastProvider>
        <TriggerButton type="success" />
      </ToastProvider>,
    );

    await act(async () => {
      screen.getByText("Trigger success").click();
    });

    const region = screen.getByRole("region", { name: /notifications/i });
    expect(region).toBeInTheDocument();
  });

  it("auto-dismisses after 3500ms", async () => {
    render(
      <ToastProvider>
        <TriggerButton type="info" />
      </ToastProvider>,
    );

    await act(async () => {
      screen.getByText("Trigger info").click();
    });

    expect(screen.getByText("Message info")).toBeInTheDocument();

    await act(async () => {
      vi.advanceTimersByTime(3500);
    });

    expect(screen.queryByText("Message info")).not.toBeInTheDocument();
  });

  it("renders action button when action provided", async () => {
    function Harness() {
      const { success } = useToast();
      return (
        <button
          onClick={() =>
            success("Service arrete", {
              action: { label: "Annuler", onClick: () => {} },
            })
          }
        >
          Lancer
        </button>
      );
    }
    render(
      <ToastProvider>
        <Harness />
      </ToastProvider>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    expect(screen.getByRole("button", { name: "Annuler" })).toBeInTheDocument();
  });

  it("calls action onClick and dismisses toast", async () => {
    const onClick = vi.fn();
    function Harness() {
      const { success } = useToast();
      return (
        <button
          onClick={() => success("Done", { action: { label: "Annuler", onClick } })}
        >
          Lancer
        </button>
      );
    }
    render(
      <ToastProvider>
        <Harness />
      </ToastProvider>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    await act(async () => {
      screen.getByRole("button", { name: "Annuler" }).click();
    });

    expect(onClick).toHaveBeenCalledTimes(1);
    expect(screen.queryByText("Done")).not.toBeInTheDocument();
  });

  it("ignores a second click on the action while the first one is running", async () => {
    let release: () => void = () => {};
    const onClick = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          release = resolve;
        }),
    );
    function Trigger() {
      const { success } = useToast();
      return (
        <button onClick={() => success("Done", { action: { label: "Annuler", onClick } })}>
          fire
        </button>
      );
    }
    render(
      <ToastProvider>
        <Trigger />
      </ToastProvider>,
    );
    await act(async () => {
      screen.getByText("fire").click();
    });
    const action = screen.getByRole("button", { name: "Annuler" });
    await act(async () => {
      action.click();
      action.click();
    });
    expect(onClick).toHaveBeenCalledTimes(1);
    await act(async () => {
      release();
    });
  });

  it("toast with action auto-dismisses after 6000ms (extended)", async () => {
    function Harness() {
      const { success } = useToast();
      return (
        <button
          onClick={() =>
            success("Done", { action: { label: "Annuler", onClick: () => {} } })
          }
        >
          Lancer
        </button>
      );
    }
    render(
      <ToastProvider>
        <Harness />
      </ToastProvider>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    expect(screen.getByText("Done")).toBeInTheDocument();

    await act(async () => {
      vi.advanceTimersByTime(3500);
    });
    expect(screen.getByText("Done")).toBeInTheDocument();

    await act(async () => {
      vi.advanceTimersByTime(3000);
    });
    expect(screen.queryByText("Done")).not.toBeInTheDocument();
  });

  it("durationMs option overrides default", async () => {
    function Harness() {
      const { info } = useToast();
      return (
        <button onClick={() => info("Quick", { durationMs: 500 })}>
          Lancer
        </button>
      );
    }
    render(
      <ToastProvider>
        <Harness />
      </ToastProvider>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    expect(screen.getByText("Quick")).toBeInTheDocument();

    await act(async () => {
      vi.advanceTimersByTime(600);
    });

    expect(screen.queryByText("Quick")).not.toBeInTheDocument();
  });

  it("clears pending auto-dismiss timers on unmount (bug-frontend-03)", async () => {
    const { unmount } = render(
      <ToastProvider>
        <TriggerButton type="info" />
      </ToastProvider>,
    );

    await act(async () => {
      screen.getByText("Trigger info").click();
    });

    const before = vi.getTimerCount();
    expect(before).toBeGreaterThan(0);

    unmount();

    // Le timer d'auto-dismiss en attente doit avoir ete clear au unmount.
    expect(vi.getTimerCount()).toBeLessThan(before);
  });
});
