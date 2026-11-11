import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import TitleBar from "../TitleBar";
import { ToastProvider } from "../../ui/Toast";
import { reportFrontendError } from "../../../lib/frontend-log";

vi.mock("../../../lib/frontend-log", () => ({
  reportFrontendError: vi.fn(),
}));

type ResizeHandler = () => void;

const win = {
  minimize: vi.fn(),
  toggleMaximize: vi.fn(),
  isMaximized: vi.fn(),
  close: vi.fn(),
  onResized: vi.fn(),
};
let resizeHandler: ResizeHandler | null = null;
const unlisten = vi.fn();

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => win,
}));

function renderBar() {
  return render(
    <ToastProvider>
      <TitleBar currentPage="Tableau de bord" />
    </ToastProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  resizeHandler = null;
  win.minimize.mockResolvedValue(undefined);
  win.toggleMaximize.mockResolvedValue(undefined);
  win.close.mockResolvedValue(undefined);
  win.isMaximized.mockResolvedValue(false);
  win.onResized.mockImplementation(async (handler: ResizeHandler) => {
    resizeHandler = handler;
    return unlisten;
  });
});

describe("TitleBar", () => {
  it("affiche Agrandir ou Restaurer selon l'etat reel de la fenetre au montage", async () => {
    win.isMaximized.mockResolvedValue(true);
    renderBar();
    expect(await screen.findByRole("button", { name: "Restaurer la fenêtre" })).toBeInTheDocument();
  });

  it("se resynchronise quand la fenetre est maximisee autrement que par le bouton", async () => {
    renderBar();
    expect(await screen.findByRole("button", { name: "Agrandir la fenêtre" })).toBeInTheDocument();

    win.isMaximized.mockResolvedValue(true);
    await act(async () => {
      resizeHandler?.();
    });
    expect(await screen.findByRole("button", { name: "Restaurer la fenêtre" })).toBeInTheDocument();
  });

  it("se desabonne des evenements de redimensionnement au demontage", async () => {
    const { unmount } = renderBar();
    await screen.findByRole("button", { name: "Agrandir la fenêtre" });
    unmount();
    expect(unlisten).toHaveBeenCalledTimes(1);
  });

  it("se desabonne aussi si le composant est demonte avant la resolution de l'abonnement", async () => {
    let resolve: (fn: () => void) => void = () => {};
    win.onResized.mockImplementation(() => new Promise<() => void>((r) => { resolve = r; }));
    const { unmount } = renderBar();
    unmount();
    await act(async () => {
      resolve(unlisten);
    });
    expect(unlisten).toHaveBeenCalledTimes(1);
  });

  it("affiche un toast d'erreur quand Reduire echoue", async () => {
    win.minimize.mockRejectedValue("refuse par le systeme");
    renderBar();
    await act(async () => {
      fireEvent.click(await screen.findByRole("button", { name: "Réduire la fenêtre" }));
    });
    expect(await screen.findByText(/Réduction impossible : refuse par le systeme/)).toBeInTheDocument();
  });

  it("affiche un toast d'erreur quand Agrandir echoue", async () => {
    win.toggleMaximize.mockRejectedValue("refuse");
    renderBar();
    await act(async () => {
      fireEvent.click(await screen.findByRole("button", { name: "Agrandir la fenêtre" }));
    });
    expect(await screen.findByText(/Redimensionnement impossible : refuse/)).toBeInTheDocument();
  });

  it("affiche un toast d'erreur quand Fermer echoue", async () => {
    win.close.mockRejectedValue("refuse");
    renderBar();
    await act(async () => {
      fireEvent.click(await screen.findByRole("button", { name: "Fermer la fenêtre" }));
    });
    expect(await screen.findByText(/Fermeture impossible : refuse/)).toBeInTheDocument();
  });

  it("journalise l'echec de l'abonnement aux redimensionnements", async () => {
    win.onResized.mockRejectedValue("pas de permission");
    renderBar();
    await screen.findByRole("button", { name: "Agrandir la fenêtre" });
    expect(reportFrontendError).toHaveBeenCalledWith("TitleBar", "pas de permission", "warn");
  });

  it("journalise l'echec de lecture de l'etat maximise", async () => {
    win.isMaximized.mockRejectedValue("lecture impossible");
    renderBar();
    await screen.findByRole("button", { name: "Agrandir la fenêtre" });
    expect(reportFrontendError).toHaveBeenCalledWith("TitleBar", "lecture impossible", "warn");
  });
});
