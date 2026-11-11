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

function batch(partial: Partial<BatchResult>): BatchResult {
  return { total: 0, success: 0, failed: 0, errors: [], dryRun: false, previewItems: [], ...partial };
}

function snapshot(backups: Partial<BatchResult>): SnapshotInfo {
  return {
    restorePointCreated: false,
    restorePointLimited: false,
    registryBackups: batch(backups),
    timestamp: "2026-09-13 10:00:00",
    actionName: "test",
  };
}

function Wrapper({ children }: { children: ReactNode }) {
  return (
    <ToastProvider>
      <ConfirmProvider>{children}</ConfirmProvider>
    </ToastProvider>
  );
}

function Harness({
  command,
  onResult,
}: {
  command: CommandName | readonly CommandName[];
  onResult?: (r: BatchResult) => void;
}) {
  const { trigger } = useConfirmBatchAction({
    command,
    confirmTitle: "Confirmer",
    confirmMessage: "Message",
    noun: { singular: "element", plural: "elements" },
    onSettled: onResult,
  });
  return <button onClick={() => void trigger()}>Lancer</button>;
}

async function launchAndConfirm() {
  await act(async () => {
    screen.getByText("Lancer").click();
  });
  const confirm = await screen.findByRole("button", { name: /^confirmer$/i });
  await act(async () => {
    confirm.click();
  });
}

function commandsCalled(): string[] {
  return mockedInvoke.mock.calls.map(([cmd]) => cmd);
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("useConfirmBatchAction — pre-batch snapshot", () => {
  it("asks before going on when no registry key could be saved, and stops if refused", async () => {
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "create_pre_batch_snapshot") return snapshot({ total: 7, success: 0, failed: 7 });
      return batch({ total: 1, success: 1 });
    });
    render(
      <Wrapper>
        <Harness command="get_services" />
      </Wrapper>,
    );

    await launchAndConfirm();

    expect(await screen.findByText(/sauvegarde du registre impossible/i)).toBeTruthy();
    const refuse = await screen.findByRole("button", { name: /annuler/i });
    await act(async () => {
      refuse.click();
    });

    await waitFor(() => {
      expect(screen.queryByText(/sauvegarde du registre impossible/i)).toBeNull();
    });
    expect(commandsCalled()).toEqual(["create_pre_batch_snapshot"]);
  });

  it("runs the command when the user accepts to go on without a snapshot", async () => {
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "create_pre_batch_snapshot") throw new Error("export impossible");
      return batch({ total: 1, success: 1 });
    });
    render(
      <Wrapper>
        <Harness command="get_services" />
      </Wrapper>,
    );

    await launchAndConfirm();

    expect(await screen.findByText(/sauvegarde du registre impossible/i)).toBeTruthy();
    const goOn = await screen.findByRole("button", { name: /^confirmer$/i });
    await act(async () => {
      goOn.click();
    });

    await waitFor(() => {
      expect(commandsCalled()).toEqual(["create_pre_batch_snapshot", "get_services"]);
    });
  });

  it("warns about a partial registry backup and still runs the command", async () => {
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "create_pre_batch_snapshot") return snapshot({ total: 7, success: 5, failed: 2 });
      return batch({ total: 1, success: 1 });
    });
    render(
      <Wrapper>
        <Harness command="get_services" />
      </Wrapper>,
    );

    await launchAndConfirm();

    await waitFor(() => {
      expect(commandsCalled()).toEqual(["create_pre_batch_snapshot", "get_services"]);
    });
    await waitFor(() => {
      const toasts = screen.queryAllByRole("status");
      expect(toasts.some((t) => /sauvegarde partielle/i.test(t.textContent ?? ""))).toBe(true);
    });
  });

  it("runs several commands in order after a single snapshot and reports the combined result", async () => {
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "create_pre_batch_snapshot") return snapshot({ total: 7, success: 7 });
      if (cmd === "get_services") return batch({ total: 2, success: 2 });
      return batch({ total: 3, success: 2, failed: 1, errors: ["boom"] });
    });
    const onResult = vi.fn();
    render(
      <Wrapper>
        <Harness command={["get_services", "get_startup_items"]} onResult={onResult} />
      </Wrapper>,
    );

    await launchAndConfirm();

    await waitFor(() => {
      expect(onResult).toHaveBeenCalledWith(batch({ total: 5, success: 4, failed: 1, errors: ["boom"] }));
    });
    expect(commandsCalled()).toEqual(["create_pre_batch_snapshot", "get_services", "get_startup_items"]);
  });
});
