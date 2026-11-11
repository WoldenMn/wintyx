import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../../../components/ui/Toast";
import { ConfirmProvider } from "../../../components/ui/ConfirmDialog";
import { AdminProvider } from "../../../contexts/AdminContext";
import { SnapshotComparePage } from "../SnapshotComparePage";
import { invalidateCache } from "../../../hooks/use-invoke";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const groupsFixture = [
  { timestamp: "20260507_120000", backupFiles: ["a.reg"], keyCount: 7 },
  { timestamp: "20260506_100000", backupFiles: ["b.reg"], keyCount: 7 },
];

function defaultMock(groups: { timestamp: string; backupFiles: string[]; keyCount: number }[] = groupsFixture) {
  mockedInvoke.mockImplementation((cmd: string) => {
    if (cmd === "list_snapshot_groups") return Promise.resolve(groups);
    if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
    return Promise.resolve(null);
  });
}

function renderPage() {
  return render(
    <AdminProvider>
      <ConfirmProvider>
        <ToastProvider>
          <SnapshotComparePage />
        </ToastProvider>
      </ConfirmProvider>
    </AdminProvider>,
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  invalidateCache();
});

describe("SnapshotComparePage", () => {
  it("renders empty state when no snapshots exist", async () => {
    defaultMock([]);
    renderPage();
    await waitFor(() => {
      expect(screen.getByText("Aucun snapshot disponible")).toBeInTheDocument();
    });
  });

  it("shows empty state when no snapshots available", async () => {
    defaultMock([]);
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/Aucun snapshot disponible/i)).toBeInTheDocument();
    });
  });

  it("shows dropdown with snapshots when groups loaded", async () => {
    defaultMock([
      { timestamp: "20260504_092315", backupFiles: ["a.reg", "b.reg"], keyCount: 2 },
      { timestamp: "20260503_181245", backupFiles: ["c.reg"], keyCount: 1 },
    ]);
    renderPage();
    await waitFor(() => {
      expect(screen.getByLabelText("Snapshot")).toBeInTheDocument();
    });
    expect(screen.getByText(/04\/05\/2026 09:23:15/)).toBeInTheDocument();
  });

  it("renders mode toggle buttons", async () => {
    defaultMock();
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Vs état actuel/i })).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: /Vs autre snapshot/i })).toBeInTheDocument();
  });

  it("shows single dropdown in vs-live mode", async () => {
    defaultMock();
    renderPage();
    await waitFor(() => {
      expect(screen.getByLabelText(/^Snapshot$/i)).toBeInTheDocument();
    });
    expect(screen.queryByLabelText(/Snapshot A/i)).not.toBeInTheDocument();
  });

  it("switches to dual selector in vs-snapshot mode", async () => {
    defaultMock();
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Vs autre snapshot/i })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: /Vs autre snapshot/i }));
    expect(screen.getByLabelText(/Snapshot A/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Snapshot B/i)).toBeInTheDocument();
  });

  it("clicking compare without selection shows warning toast", async () => {
    defaultMock([{ timestamp: "20260504_092315", backupFiles: ["a.reg"], keyCount: 1 }]);
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Comparer/i })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: /Comparer/i }));
    await waitFor(() => {
      expect(screen.getByText("Sélectionner un snapshot d'abord")).toBeInTheDocument();
    });
    expect(mockedInvoke).not.toHaveBeenCalledWith("diff_snapshot_vs_live", expect.anything());
  });

  it("clicking a checkbox in DiffView adds the entry to selectedChanges", async () => {
    const diffFixture = {
      snapshotTimestamp: "20260507_120000",
      comparisonTarget: null,
      totalChanges: 1,
      keys: [
        {
          keyPath: "HKCU\\Software\\Test",
          changes: [
            { type: "modified", name: "MyValue", snapshot: "\"old\"", current: "\"new\"" },
          ],
        },
      ],
    };
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_snapshot_groups") return Promise.resolve(groupsFixture);
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
      if (cmd === "diff_snapshot_vs_live") return Promise.resolve(diffFixture);
      return Promise.resolve(null);
    });

    renderPage();

    await waitFor(() => {
      expect(screen.getByLabelText(/^Snapshot$/i)).toBeInTheDocument();
    });

    fireEvent.change(screen.getByLabelText(/^Snapshot$/i), {
      target: { value: "20260507_120000" },
    });
    fireEvent.click(screen.getByRole("button", { name: /Comparer avec état actuel/i }));

    await waitFor(() => {
      expect(screen.getByLabelText(/Sélectionner MyValue/i)).toBeInTheDocument();
    });

    fireEvent.click(screen.getByLabelText(/Sélectionner MyValue/i));

    await waitFor(() => {
      expect(screen.getByText(/1 changement\(s\) sélectionné\(s\)/i)).toBeInTheDocument();
    });
  });

  it("shows the Ajoutés filter button after a diff", async () => {
    const diffFixture = {
      snapshotTimestamp: "20260507_120000",
      comparisonTarget: null,
      totalChanges: 1,
      keys: [
        {
          keyPath: "HKCU\\Software\\Test",
          changes: [
            { type: "modified", name: "MyValue", snapshot: "\"old\"", current: "\"new\"" },
          ],
        },
      ],
    };
    mockedInvoke.mockImplementation((cmd: string) => {
      if (cmd === "list_snapshot_groups") return Promise.resolve(groupsFixture);
      if (cmd === "get_admin_status") return Promise.resolve({ isAdmin: true, canElevate: true });
      if (cmd === "diff_snapshot_vs_live") return Promise.resolve(diffFixture);
      return Promise.resolve(null);
    });

    renderPage();

    await waitFor(() => {
      expect(screen.getByLabelText(/^Snapshot$/i)).toBeInTheDocument();
    });

    fireEvent.change(screen.getByLabelText(/^Snapshot$/i), {
      target: { value: "20260507_120000" },
    });
    fireEvent.click(screen.getByRole("button", { name: /Comparer avec état actuel/i }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Ajoutés/i })).toBeInTheDocument();
    });
  });
});
