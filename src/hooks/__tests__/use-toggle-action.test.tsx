import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, screen, fireEvent, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../components/ui/Toast";
import { ConfirmProvider } from "../../components/ui/ConfirmDialog";
import { useToggleAction } from "../use-toggle-action";
import type { CommandName } from "../../lib/command-names";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

interface TestItem {
  id: string;
  label: string;
}

function Wrapper({ children }: { children: ReactNode }) {
  return (
    <ToastProvider>
      <ConfirmProvider>{children}</ConfirmProvider>
    </ToastProvider>
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  mockedInvoke.mockResolvedValue(null);
});

describe("useToggleAction", () => {
  it("exposes trigger function + busy=null initially", () => {
    const { result } = renderHook(
      () =>
        useToggleAction<TestItem>({
          doCommand: "get_services",
          getId: (i) => i.id,
          confirmTitle: "Title",
          confirmMessage: () => "Message",
          successMessage: () => "OK",
        }),
      { wrapper: Wrapper },
    );
    expect(typeof result.current.trigger).toBe("function");
    expect(result.current.busy).toBeNull();
  });

  it("affiche un toast warning sur succes partiel via evaluateResult (degr-16)", async () => {
    mockedInvoke.mockResolvedValue({ errors: ["a", "b"] });
    const { result } = renderHook(
      () =>
        useToggleAction<TestItem>({
          doCommand: "clean_target" as CommandName,
          getId: (i) => i.id,
          confirmTitle: "T",
          confirmMessage: () => "M",
          successMessage: () => "Succes complet",
          evaluateResult: (res) => {
            const r = res as { errors: string[] };
            return r.errors.length > 0
              ? { level: "warning", message: "2 element(s) non supprime(s)" }
              : null;
          },
        }),
      { wrapper: Wrapper },
    );

    act(() => {
      void result.current.trigger({ id: "x", label: "X" });
    });
    const confirmBtn = await screen.findByRole("button", { name: "Confirmer" });
    await act(async () => {
      fireEvent.click(confirmBtn);
    });

    await waitFor(() => {
      expect(screen.getByText("2 element(s) non supprime(s)")).toBeInTheDocument();
    });
    // Pas de faux "succes complet" quand il y a des erreurs.
    expect(screen.queryByText("Succes complet")).not.toBeInTheDocument();
  });

  it("trigger does nothing if user cancels confirm", async () => {
    // Le ConfirmProvider attend un clic sur un bouton du dialog.
    // Si on n'interagit pas, la promise ne resout jamais — on la race contre un timeout
    // pour verifier que busy reste null pendant ce temps.
    const { result } = renderHook(
      () =>
        useToggleAction<TestItem>({
          doCommand: "get_services",
          getId: (i) => i.id,
          confirmTitle: "T",
          confirmMessage: () => "M",
          successMessage: () => "OK",
        }),
      { wrapper: Wrapper },
    );

    // Declencher sans attendre — busy doit rester null tant que confirm pas repondu
    act(() => {
      void result.current.trigger({ id: "x", label: "X" });
    });
    expect(result.current.busy).toBeNull();
  });

  it("exposes recentId=null initially", () => {
    const { result } = renderHook(
      () =>
        useToggleAction<TestItem>({
          doCommand: "get_services",
          getId: (i) => i.id,
          confirmTitle: "T",
          confirmMessage: () => "M",
          successMessage: () => "OK",
        }),
      { wrapper: Wrapper },
    );
    expect(result.current.recentId).toBeNull();
  });

  it("sets recentId to item id after successful invoke", async () => {
    const { result } = renderHook(
      () =>
        useToggleAction<TestItem>({
          doCommand: "get_services",
          getId: (i) => i.id,
          confirmTitle: "T",
          confirmMessage: () => "M",
          successMessage: () => "OK",
        }),
      { wrapper: Wrapper },
    );

    act(() => {
      void result.current.trigger({ id: "test-id", label: "X" });
    });

    // Attendre que le dialog apparaisse et cliquer "Confirmer"
    const confirmBtn = await screen.findByRole("button", { name: "Confirmer" });
    await act(async () => {
      fireEvent.click(confirmBtn);
    });

    await waitFor(() => {
      expect(result.current.recentId).toBe("test-id");
    });
  });

  it(
    "clears recentId to null after 800ms",
    async () => {
      // Sans fake timers : on attend 850ms reels, suffisamment pour que le clear soit declenche.
      // vi.useFakeTimers interfere avec ConfirmProvider (qui utilise des promises internes).
      const { result } = renderHook(
        () =>
          useToggleAction<TestItem>({
            doCommand: "get_services",
            getId: (i) => i.id,
            confirmTitle: "T",
            confirmMessage: () => "M",
            successMessage: () => "OK",
          }),
        { wrapper: Wrapper },
      );

      act(() => {
        void result.current.trigger({ id: "test-id", label: "X" });
      });

      const confirmBtn = await screen.findByRole("button", { name: "Confirmer" });
      await act(async () => {
        fireEvent.click(confirmBtn);
      });

      await waitFor(() => expect(result.current.recentId).toBe("test-id"));
      await new Promise((r) => setTimeout(r, 850));
      expect(result.current.recentId).toBeNull();
    },
    2000,
  );

  it("keeps recentId null when invoke fails", async () => {
    mockedInvoke.mockRejectedValueOnce(new Error("backend error"));
    const { result } = renderHook(
      () =>
        useToggleAction<TestItem>({
          doCommand: "get_services",
          getId: (i) => i.id,
          confirmTitle: "T",
          confirmMessage: () => "M",
          successMessage: () => "OK",
        }),
      { wrapper: Wrapper },
    );

    act(() => {
      void result.current.trigger({ id: "test-id", label: "X" });
    });

    const confirmBtn = await screen.findByRole("button", { name: "Confirmer" });
    await act(async () => {
      fireEvent.click(confirmBtn);
    });

    // Laisser le temps a l'invoke de rejecter et au catch de s'executer
    await waitFor(() => {
      expect(result.current.busy).toBeNull();
    });
    expect(result.current.recentId).toBeNull();
  });

  it("reports a BatchResult with failures as an error toast, not a success", async () => {
    mockedInvoke.mockResolvedValue({ total: 1, success: 0, failed: 1, errors: ["Pilote en cours d'utilisation"], dryRun: false, previewItems: [] });
    const onSettled = vi.fn();
    const { result } = renderHook(
      () =>
        useToggleAction<TestItem>({
          doCommand: "uninstall_driver",
          getId: (i) => i.id,
          confirmTitle: "T",
          confirmMessage: () => "M",
          successMessage: () => "Pilote desinstalle",
          onSettled,
        }),
      { wrapper: Wrapper },
    );

    act(() => {
      void result.current.trigger({ id: "x", label: "X" });
    });
    const confirmBtn = await screen.findByRole("button", { name: "Confirmer" });
    await act(async () => {
      fireEvent.click(confirmBtn);
    });

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Pilote en cours d'utilisation");
    expect(screen.queryByText("Pilote desinstalle")).not.toBeInTheDocument();
    expect(result.current.recentId).toBeNull();
    expect(onSettled).toHaveBeenCalledTimes(1);
  });

  it("opens a single confirmation when triggered twice for the same item", async () => {
    const { result } = renderHook(
      () =>
        useToggleAction<TestItem>({
          doCommand: "uninstall_driver",
          getId: (i) => i.id,
          confirmTitle: "T",
          confirmMessage: () => "M",
          successMessage: () => "OK",
        }),
      { wrapper: Wrapper },
    );

    act(() => {
      void result.current.trigger({ id: "x", label: "X" });
      void result.current.trigger({ id: "x", label: "X" });
    });
    const confirmBtn = await screen.findByRole("button", { name: "Confirmer" });
    await act(async () => {
      fireEvent.click(confirmBtn);
    });
    await waitFor(() => expect(result.current.busy).toBeNull());

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(mockedInvoke.mock.calls.filter(([c]) => c === "uninstall_driver")).toHaveLength(1);
  });
});
