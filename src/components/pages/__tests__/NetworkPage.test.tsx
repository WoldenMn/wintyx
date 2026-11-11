import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { AdminProvider } from "../../../contexts/AdminContext";
import { NetworkPage } from "../NetworkPage";
import { invalidateCache } from "../../../hooks/use-invoke";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

function mockBackend(publicIp: () => Promise<unknown>, firewall: unknown[] = []) {
  mockedInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_network_adapters") return Promise.resolve([]);
    if (cmd === "get_firewall_status") return Promise.resolve(firewall);
    if (cmd === "get_public_ip") return publicIp();
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
    return Promise.resolve(null);
  });
}

function renderPage() {
  return render(
    <AdminProvider>
      <ToastProvider>
        <NetworkPage />
      </ToastProvider>
    </AdminProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  invalidateCache();
});

describe("NetworkPage pare-feu", () => {
  it("distingue un profil inconnu d'un profil desactive", async () => {
    mockBackend(() => Promise.resolve("203.0.113.5"), [
      { name: "Domain", isEnabled: true },
      { name: "Private", isEnabled: false },
      { name: "Public", isEnabled: null },
    ]);
    renderPage();
    expect(await screen.findByText("Partiel")).toBeInTheDocument();
    expect(screen.getByText("Domain : activé")).toBeInTheDocument();
    expect(screen.getByText("Private : désactivé")).toBeInTheDocument();
    expect(screen.getByText("Public : état inconnu")).toBeInTheDocument();
  });

  it("n'annonce pas Actif quand un profil est illisible", async () => {
    mockBackend(() => Promise.resolve("203.0.113.5"), [
      { name: "Domain", isEnabled: true },
      { name: "Private", isEnabled: true },
      { name: "Public", isEnabled: null },
    ]);
    renderPage();
    expect(await screen.findByText("Indéterminé")).toBeInTheDocument();
    expect(screen.queryByText("Actif")).not.toBeInTheDocument();
  });

  it("annonce Actif quand tous les profils sont actives", async () => {
    mockBackend(() => Promise.resolve("203.0.113.5"), [
      { name: "Domain", isEnabled: true },
      { name: "Private", isEnabled: true },
      { name: "Public", isEnabled: true },
    ]);
    renderPage();
    expect(await screen.findByText("Actif")).toBeInTheDocument();
  });
});

describe("NetworkPage IP publique", () => {
  it("ne contacte pas le service d'IP publique a l'ouverture de la page", async () => {
    mockBackend(() => Promise.resolve("203.0.113.5"));
    renderPage();
    expect(await screen.findByRole("button", { name: /afficher l'ip publique/i })).toBeInTheDocument();
    expect(mockedInvoke.mock.calls.some(([cmd]) => cmd === "get_public_ip")).toBe(false);
  });

  it("affiche l'adresse apres un clic sur Afficher", async () => {
    mockBackend(() => Promise.resolve("203.0.113.5"));
    renderPage();
    fireEvent.click(await screen.findByRole("button", { name: /afficher l'ip publique/i }));
    expect(await screen.findByText("203.0.113.5")).toBeInTheDocument();
  });

  it("affiche Indisponible avec la cause au lieu de --, et permet de reessayer", async () => {
    let calls = 0;
    mockBackend(() => {
      calls += 1;
      return calls === 1
        ? Promise.reject({ code: "ProcessFailed", details: { command: "Invoke-WebRequest", detail: "hors ligne" } })
        : Promise.resolve("203.0.113.5");
    });
    renderPage();
    fireEvent.click(await screen.findByRole("button", { name: /afficher l'ip publique/i }));

    expect(await screen.findByText("Indisponible")).toBeInTheDocument();
    expect(screen.getByText(/hors ligne/)).toBeInTheDocument();
    expect(screen.getByText("IP publique").parentElement).not.toHaveTextContent("--");

    fireEvent.click(screen.getByRole("button", { name: /réessayer l'ip publique/i }));
    await waitFor(() => expect(screen.getByText("203.0.113.5")).toBeInTheDocument());
  });
});
