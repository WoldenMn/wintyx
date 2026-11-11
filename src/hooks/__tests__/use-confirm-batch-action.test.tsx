import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, act, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../components/ui/Toast";
import { ConfirmProvider } from "../../components/ui/ConfirmDialog";
import type { CommandName } from "../../lib/command-names";
import { useConfirmBatchAction } from "../use-confirm-batch-action";
import type { BatchResult, SnapshotInfo } from "../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const DEFAULT_SNAPSHOT: SnapshotInfo = {
  restorePointCreated: false,
  restorePointLimited: false,
  registryBackups: { total: 7, success: 7, failed: 0, errors: [], dryRun: false, previewItems: [] },
  timestamp: "2026-04-23 10:00:00",
  actionName: "test",
};

function setupInvokeMock(
  commandResult: BatchResult | Error,
  snapshot: SnapshotInfo | Error = DEFAULT_SNAPSHOT,
) {
  mockedInvoke.mockImplementation(async (cmd: string) => {
    if (cmd === "create_pre_batch_snapshot") {
      if (snapshot instanceof Error) throw snapshot;
      return snapshot;
    }
    if (commandResult instanceof Error) throw commandResult;
    return commandResult;
  });
}

function Wrapper({ children }: { children: ReactNode }) {
  return (
    <ToastProvider>
      <ConfirmProvider>{children}</ConfirmProvider>
    </ToastProvider>
  );
}

function Harness({
  onResult,
  command = "get_services",
  skipSnapshot,
}: {
  onResult?: (r: BatchResult) => void;
  command?: CommandName | readonly CommandName[];
  skipSnapshot?: boolean;
}) {
  const { trigger, isRunning } = useConfirmBatchAction({
    command,
    confirmTitle: "Confirmer",
    confirmMessage: "Message",
    noun: { singular: "element", plural: "elements" },
    onSettled: onResult,
    skipSnapshot,
  });
  return (
    <div>
      <button onClick={() => void trigger()}>Lancer</button>
      <span data-testid="running">{String(isRunning)}</span>
    </div>
  );
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("useConfirmBatchAction reentrance", () => {
  it("n'ouvre qu'une confirmation pour deux clics rapides", async () => {
    setupInvokeMock({ total: 1, success: 1, failed: 0, errors: [], dryRun: false, previewItems: [] });
    render(
      <Wrapper>
        <Harness />
      </Wrapper>,
    );
    await act(async () => {
      screen.getByText("Lancer").click();
      screen.getByText("Lancer").click();
    });
    expect(screen.getAllByRole("dialog")).toHaveLength(1);
    await act(async () => {
      screen.getByRole("button", { name: "Confirmer" }).click();
    });
    await waitFor(() => expect(screen.getByTestId("running").textContent).toBe("false"));
    expect(mockedInvoke.mock.calls.filter(([cmd]) => cmd === "get_services")).toHaveLength(1);
  });
});

describe("useConfirmBatchAction", () => {
  it("does not invoke command if user cancels", async () => {
    setupInvokeMock({ total: 0, success: 0, failed: 0, errors: [], dryRun: false, previewItems: [] });
    render(
      <Wrapper>
        <Harness />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const cancel = await screen.findByRole("button", { name: /annuler/i });
    await act(async () => {
      cancel.click();
    });

    expect(mockedInvoke).not.toHaveBeenCalled();
  });

  it("invokes command on confirm + triggers success toast when all succeed", async () => {
    const result: BatchResult = { total: 3, success: 3, failed: 0, errors: [], dryRun: false, previewItems: [] };
    setupInvokeMock(result);
    const onResult = vi.fn();

    render(
      <Wrapper>
        <Harness onResult={onResult} />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      confirm.click();
    });

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_services", {});
    });
    await waitFor(() => {
      expect(onResult).toHaveBeenCalledWith(result);
    });
  });

  it("shows warning toast on partial success", async () => {
    const result: BatchResult = { total: 5, success: 3, failed: 2, errors: ["a", "b"], dryRun: false, previewItems: [] };
    setupInvokeMock(result);

    render(
      <Wrapper>
        <Harness />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      confirm.click();
    });

    await waitFor(() => {
      const toasts = screen.queryAllByRole("status");
      const hasPartial = toasts.some((t) =>
        /(3\/5|2 en echec)/i.test(t.textContent ?? ""),
      );
      expect(hasPartial).toBe(true);
    });
  });

  it("shows error toast on total failure", async () => {
    const result: BatchResult = { total: 2, success: 0, failed: 2, errors: ["x", "y"], dryRun: false, previewItems: [] };
    setupInvokeMock(result);

    render(
      <Wrapper>
        <Harness />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      confirm.click();
    });

    await waitFor(() => {
      const alerts = screen.queryAllByRole("alert");
      const hasError = alerts.some((a) => /échec/i.test(a.textContent ?? ""));
      expect(hasError).toBe(true);
    });
  });

  it("propagates invoke error as toast", async () => {
    setupInvokeMock(new Error("boom"));

    render(
      <Wrapper>
        <Harness />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      confirm.click();
    });

    await waitFor(() => {
      const alerts = screen.queryAllByRole("alert");
      const hasError = alerts.some((a) => /erreur/i.test(a.textContent ?? ""));
      expect(hasError).toBe(true);
    });
  });

  it("calls create_pre_batch_snapshot before the main command", async () => {
    const result: BatchResult = { total: 1, success: 1, failed: 0, errors: [], dryRun: false, previewItems: [] };
    const callOrder: string[] = [];
    mockedInvoke.mockImplementation(async (cmd: string) => {
      callOrder.push(cmd);
      if (cmd === "create_pre_batch_snapshot") return DEFAULT_SNAPSHOT;
      return result;
    });

    render(
      <Wrapper>
        <Harness />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      confirm.click();
    });

    await waitFor(() => {
      expect(callOrder).toEqual(["create_pre_batch_snapshot", "get_services"]);
    });
  });

  it("skips snapshot when skipSnapshot=true", async () => {
    const result: BatchResult = { total: 1, success: 1, failed: 0, errors: [], dryRun: false, previewItems: [] };
    setupInvokeMock(result);

    render(
      <Wrapper>
        <Harness skipSnapshot />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      confirm.click();
    });

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_services", {});
    });
    const snapshotCalls = mockedInvoke.mock.calls.filter(
      ([cmd]) => cmd === "create_pre_batch_snapshot",
    );
    expect(snapshotCalls).toHaveLength(0);
  });

  it("shows restore-point toast when admin snapshot succeeds", async () => {
    const result: BatchResult = { total: 1, success: 1, failed: 0, errors: [], dryRun: false, previewItems: [] };
    const snapshot: SnapshotInfo = { ...DEFAULT_SNAPSHOT, restorePointCreated: true };
    setupInvokeMock(result, snapshot);

    render(
      <Wrapper>
        <Harness />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      confirm.click();
    });

    await waitFor(() => {
      const toasts = screen.queryAllByRole("status");
      const hasRestore = toasts.some((t) =>
        /point de restauration/i.test(t.textContent ?? ""),
      );
      expect(hasRestore).toBe(true);
    });
  });

  it("warns instead of claiming success when Windows skipped the restore point (24 h limit)", async () => {
    const result: BatchResult = { total: 1, success: 1, failed: 0, errors: [], dryRun: false, previewItems: [] };
    const snapshot: SnapshotInfo = { ...DEFAULT_SNAPSHOT, restorePointCreated: false, restorePointLimited: true };
    setupInvokeMock(result, snapshot);

    render(
      <Wrapper>
        <Harness />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      confirm.click();
    });

    await waitFor(() => {
      const texts = screen.queryAllByRole("status").map((t) => t.textContent ?? "");
      expect(texts.some((t) => /point de restauration non créé.*24 h/i.test(t))).toBe(true);
      expect(texts.some((t) => /point de restauration créé/i.test(t))).toBe(false);
    });
  });

  it("continues with main command when snapshot fails and the user accepts to go on", async () => {
    const result: BatchResult = { total: 1, success: 1, failed: 0, errors: [], dryRun: false, previewItems: [] };
    setupInvokeMock(result, new Error("snapshot failed"));

    render(
      <Wrapper>
        <Harness />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      confirm.click();
    });

    await screen.findByText(/sauvegarde du registre impossible/i);
    const goOn = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      goOn.click();
    });

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_services", {});
    });
  });

  it("with enablePreview: runs dry-run first then enriches confirm message", async () => {
    const preview: BatchResult = {
      total: 2,
      success: 0,
      failed: 0,
      errors: [],
      dryRun: true,
      previewItems: ["SMBv1", "RDP"],
    };
    const finalResult: BatchResult = {
      total: 2,
      success: 2,
      failed: 0,
      errors: [],
      dryRun: false,
      previewItems: [],
    };
    const calls: Array<{ cmd: string; args: unknown }> = [];
    mockedInvoke.mockImplementation(async (cmd: string, args) => {
      calls.push({ cmd, args });
      if (cmd === "create_pre_batch_snapshot") return DEFAULT_SNAPSHOT;
      if (cmd === "get_services" && (args as { options?: { dryRun?: boolean } })?.options?.dryRun) {
        return preview;
      }
      return finalResult;
    });

    function PreviewHarness() {
      const { trigger } = useConfirmBatchAction({
        command: "get_services",
        confirmTitle: "Confirmer",
        confirmMessage: "Message de base",
        noun: { singular: "element", plural: "elements" },
        enablePreview: true,
      });
      return <button onClick={() => void trigger()}>Lancer</button>;
    }

    render(
      <Wrapper>
        <PreviewHarness />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    const desc = document.getElementById("confirm-desc");
    expect(desc?.textContent).toContain("Aperçu");
    expect(desc?.textContent).toContain("SMBv1");
    expect(desc?.textContent).toContain("RDP");

    await act(async () => {
      confirm.click();
    });

    await waitFor(() => {
      const mainCall = calls.find(
        (c) =>
          c.cmd === "get_services" && !(c.args as { options?: { dryRun?: boolean } })?.options?.dryRun,
      );
      expect(mainCall).toBeDefined();
    });
    const dryCall = calls.find(
      (c) => c.cmd === "get_services" && (c.args as { options?: { dryRun?: boolean } })?.options?.dryRun === true,
    );
    expect(dryCall).toBeDefined();
  });

  it("with enablePreview: truncates preview list beyond 8 items (... et N autres)", async () => {
    // tests-fe-09 : formatPreviewMessage (prive) tronque a 8 et ajoute le suffixe.
    const preview: BatchResult = {
      total: 10,
      success: 0,
      failed: 0,
      errors: [],
      dryRun: true,
      previewItems: ["i1", "i2", "i3", "i4", "i5", "i6", "i7", "i8", "i9", "i10"],
    };
    const finalResult: BatchResult = {
      total: 10, success: 10, failed: 0, errors: [], dryRun: false, previewItems: [],
    };
    mockedInvoke.mockImplementation(async (cmd: string, args) => {
      if (cmd === "create_pre_batch_snapshot") return DEFAULT_SNAPSHOT;
      if (cmd === "get_services" && (args as { options?: { dryRun?: boolean } })?.options?.dryRun) {
        return preview;
      }
      return finalResult;
    });

    function PreviewHarness() {
      const { trigger } = useConfirmBatchAction({
        command: "get_services",
        confirmTitle: "Confirmer",
        confirmMessage: "Message de base",
        noun: { singular: "element", plural: "elements" },
        enablePreview: true,
      });
      return <button onClick={() => void trigger()}>Lancer</button>;
    }

    render(
      <Wrapper>
        <PreviewHarness />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    await screen.findByRole("button", { name: /^confirmer$/i });
    const desc = document.getElementById("confirm-desc");
    expect(desc?.textContent).toContain("i8");
    expect(desc?.textContent).not.toContain("i9");
    expect(desc?.textContent).toContain("... et 2 autres");
  });

  it("shows info toast with dry-run summary when result has dryRun=true", async () => {
    const dryResult: BatchResult = {
      total: 3,
      success: 0,
      failed: 0,
      errors: [],
      dryRun: true,
      previewItems: ["A", "B", "C"],
    };
    setupInvokeMock(dryResult);

    render(
      <Wrapper>
        <Harness />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });

    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      confirm.click();
    });

    await waitFor(() => {
      const toasts = screen.queryAllByRole("status");
      const hasPreview = toasts.some((t) => /Aperçu/i.test(t.textContent ?? ""));
      expect(hasPreview).toBe(true);
    });
  });

  it("keeps the partial results when a later command rejects, shows them and still calls onSettled", async () => {
    const first: BatchResult = { total: 2, success: 2, failed: 0, errors: [], dryRun: false, previewItems: [] };
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "create_pre_batch_snapshot") return DEFAULT_SNAPSHOT;
      if (cmd === "get_services") return first;
      throw new Error("deuxieme commande en echec");
    });
    const onResult = vi.fn();

    render(
      <Wrapper>
        <Harness onResult={onResult} command={["get_services", "get_startup_items"]} />
      </Wrapper>,
    );

    await act(async () => {
      screen.getByText("Lancer").click();
    });
    const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      confirm.click();
    });

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("2");
    expect(alert).toHaveTextContent("deuxieme commande en echec");
    expect(onResult).toHaveBeenCalledTimes(1);
    expect(onResult.mock.calls[0][0]).toMatchObject({ total: 2, success: 2 });
  });
});
