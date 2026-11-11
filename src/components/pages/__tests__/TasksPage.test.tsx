import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { TasksPage } from "../TasksPage";
import { invalidateCache } from "../../../hooks/use-invoke";
import { SUSPICIOUS_TASKS_CHANGED_EVENT } from "../../../lib/suspicious-tasks";
import type { TaskItem } from "../../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const TASK: TaskItem = {
  name: "Updater",
  path: "\\Vendor\\",
  state: "Ready",
  author: "x",
  description: "",
  lastRunTime: "",
  nextRunTime: "",
  actionPath: "powershell.exe",
  actionArguments: "-enc AAAA",
  isSuspicious: true,
  needsReview: false,
  suspiciousReason: "Arguments encodes",
};

function renderPage() {
  return render(
    <ToastProvider>
      <ConfirmProvider>
        <TasksPage />
      </ConfirmProvider>
    </ToastProvider>,
  );
}

let changed: () => void;

beforeEach(() => {
  vi.clearAllMocks();
  invalidateCache();
  changed = vi.fn();
  window.addEventListener(SUSPICIOUS_TASKS_CHANGED_EVENT, changed);
});

afterEach(() => {
  window.removeEventListener(SUSPICIOUS_TASKS_CHANGED_EVENT, changed);
});

async function disableTask() {
  fireEvent.click(await screen.findByRole("button", { name: "Désactiver" }));
  const confirm = await screen.findByRole("button", { name: "Confirmer" });
  await act(async () => {
    fireEvent.click(confirm);
  });
}

describe("TasksPage badge de la barre laterale", () => {
  it("demande le rafraichissement du compteur apres une desactivation reussie", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_suspicious_tasks" || cmd === "get_scheduled_tasks") return Promise.resolve([TASK]);
      return Promise.resolve(null);
    });
    renderPage();
    await disableTask();

    await screen.findByText("Updater désactivé");
    expect(changed).toHaveBeenCalledTimes(1);
  });

  it("ne le demande pas quand la desactivation echoue", async () => {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_suspicious_tasks" || cmd === "get_scheduled_tasks") return Promise.resolve([TASK]);
      if (cmd === "disable_scheduled_task") return Promise.reject({ code: "Generic", details: "refuse" });
      return Promise.resolve(null);
    });
    renderPage();
    await disableTask();

    await screen.findByText(/refuse/);
    expect(changed).not.toHaveBeenCalled();
  });
});

describe("TasksPage niveaux de signalement", () => {
  const USER_TASK: TaskItem = {
    ...TASK,
    name: "outil-perso",
    author: "",
    actionArguments: "-File run.ps1",
    isSuspicious: false,
    needsReview: true,
    suspiciousReason: "Aucun auteur déclaré",
  };

  function mockTasks() {
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_suspicious_tasks") return Promise.resolve([TASK]);
      if (cmd === "get_scheduled_tasks") return Promise.resolve([TASK, USER_TASK]);
      return Promise.resolve(null);
    });
  }

  it("compte séparément les tâches suspectes et celles à vérifier", async () => {
    mockTasks();
    renderPage();
    expect(await screen.findByRole("button", { name: "Suspectes (1)" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "À vérifier (1)" })).toBeInTheDocument();
  });

  it("présente une tâche d'utilisateur comme à vérifier, pas comme suspecte", async () => {
    mockTasks();
    renderPage();
    fireEvent.click(await screen.findByRole("button", { name: "À vérifier (1)" }));
    expect(await screen.findByText("outil-perso")).toBeInTheDocument();
    expect(screen.getByText("À vérifier", { selector: "span" })).toBeInTheDocument();
    expect(screen.queryByText("Suspecte")).not.toBeInTheDocument();
  });

  it("distingue une recherche sans résultat d'un planificateur vide", async () => {
    mockTasks();
    renderPage();
    fireEvent.click(await screen.findByRole("button", { name: "Toutes (2)" }));
    fireEvent.change(screen.getByPlaceholderText("Rechercher une tâche..."), { target: { value: "zzzzz" } });
    expect(await screen.findByText("Aucune tâche ne correspond à la recherche")).toBeInTheDocument();
    expect(screen.queryByText(/semble vide ou inaccessible/)).not.toBeInTheDocument();
  });

  it("garde le message d'inaccessibilité quand le planificateur est réellement vide", async () => {
    mockedInvoke.mockImplementation(() => Promise.resolve([]));
    renderPage();
    fireEvent.click(await screen.findByRole("button", { name: "Toutes (0)" }));
    expect(await screen.findByText(/semble vide ou inaccessible/)).toBeInTheDocument();
  });

  it("cherche sans tenir compte des accents", async () => {
    mockedInvoke.mockImplementation((cmd: string) =>
      Promise.resolve(cmd === "get_scheduled_tasks" || cmd === "get_suspicious_tasks" ? [{ ...TASK, name: "Mise à jour" }] : null),
    );
    renderPage();
    fireEvent.click(await screen.findByRole("button", { name: "Toutes (1)" }));
    fireEvent.change(screen.getByPlaceholderText("Rechercher une tâche..."), { target: { value: "mise a jour" } });
    expect(await screen.findByText("Mise à jour")).toBeInTheDocument();
  });
});
