import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ToastProvider } from "../Toast";
import { AdminProvider } from "../../../contexts/AdminContext";
import { CrudListPage } from "../CrudListPage";
import { invalidateCache } from "../../../hooks/use-invoke";
import type { CommandName } from "../../../lib/command-names";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

// Commande factice (invoke mocke) cast en CommandName (DX-01).
const TEST_COMMAND = "get_test_items" as string as CommandName;

interface TestItem {
  id: string;
  name: string;
  category: string;
}

const ITEMS: TestItem[] = [
  { id: "1", name: "Alpha", category: "A" },
  { id: "2", name: "Beta", category: "B" },
  { id: "3", name: "Gamma", category: "A" },
];

function Wrapper({ children }: { children: ReactNode }) {
  return (
    <AdminProvider>
      <ToastProvider>{children}</ToastProvider>
    </AdminProvider>
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  invalidateCache(); // reset cache SWR entre tests
  mockedInvoke.mockImplementation(async (cmd: string) => {
    if (cmd === "get_test_items") return ITEMS;
    if (cmd === "get_admin_status") return { isAdmin: true, canElevate: true };
    return null;
  });
});

describe("CrudListPage", () => {
  it("fetches items via fetchCommand on mount", async () => {
    render(
      <Wrapper>
        <CrudListPage<TestItem>
          title="Test"
          fetchCommand={TEST_COMMAND}
          renderItem={(item) => <span>{item.name}</span>}
        />
      </Wrapper>,
    );
    await waitFor(() => expect(screen.getByText("Alpha")).toBeInTheDocument());
    expect(screen.getByText("Beta")).toBeInTheDocument();
    expect(screen.getByText("Gamma")).toBeInTheDocument();
  });

  it("affiche ErrorState (pas EmptyState) quand le fetch echoue (env-013)", async () => {
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_admin_status") return { isAdmin: true, canElevate: true };
      if (cmd === "get_test_items")
        throw { code: "WmiUnavailable", details: "Winmgmt" };
      return null;
    });
    render(
      <Wrapper>
        <CrudListPage<TestItem>
          title="Test"
          fetchCommand={TEST_COMMAND}
          renderItem={(item) => <span>{item.name}</span>}
          emptyState={{ title: "Aucun élément" }}
        />
      </Wrapper>,
    );
    await waitFor(() => expect(screen.getByRole("alert")).toBeInTheDocument());
    expect(screen.getByText(/WMI indisponible/i)).toBeInTheDocument();
    // Un echec ne doit jamais etre rendu comme une liste vide.
    expect(screen.queryByText("Aucun élément")).not.toBeInTheDocument();
  });

  it("renders title in header after load", async () => {
    render(
      <Wrapper>
        <CrudListPage<TestItem>
          title="Mon titre"
          fetchCommand={TEST_COMMAND}
          renderItem={(item) => <span>{item.name}</span>}
        />
      </Wrapper>,
    );
    // Attendre que le skeleton disparaisse et le header apparaisse
    await waitFor(() => expect(screen.getByText("Mon titre")).toBeInTheDocument());
  });

  it("filters items when filter prop is provided", async () => {
    render(
      <Wrapper>
        <CrudListPage<TestItem>
          title="Test"
          fetchCommand={TEST_COMMAND}
          filter={(item, q) => item.name.toLowerCase().includes(q.toLowerCase())}
          renderItem={(item) => <span>{item.name}</span>}
        />
      </Wrapper>,
    );
    await waitFor(() => expect(screen.getByText("Alpha")).toBeInTheDocument());

    const search = screen.getByRole("searchbox");
    fireEvent.change(search, { target: { value: "Bet" } });
    await waitFor(() => {
      expect(screen.queryByText("Alpha")).toBeNull();
      expect(screen.getByText("Beta")).toBeInTheDocument();
    });
  });

  it("groups items when groupBy is provided", async () => {
    render(
      <Wrapper>
        <CrudListPage<TestItem>
          title="Test"
          fetchCommand={TEST_COMMAND}
          groupBy={(i) => i.category}
          renderItem={(item) => <span>{item.name}</span>}
        />
      </Wrapper>,
    );
    await waitFor(() => expect(screen.getByText("Alpha")).toBeInTheDocument());
    const headings = screen.getAllByRole("heading", { level: 2 });
    expect(headings.length).toBe(2);
  });

  it("shows empty state when no items", async () => {
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_test_items") return [];
      if (cmd === "get_admin_status") return { isAdmin: true, canElevate: true };
      return null;
    });
    render(
      <Wrapper>
        <CrudListPage<TestItem>
          title="Test"
          fetchCommand={TEST_COMMAND}
          renderItem={(item) => <span>{item.name}</span>}
          emptyState={{ title: "Vide", hint: "Rien ici" }}
        />
      </Wrapper>,
    );
    await waitFor(() => expect(screen.getByText("Vide")).toBeInTheDocument());
    expect(screen.getByText("Rien ici")).toBeInTheDocument();
  });

  it("uses getItemId for stable keys when provided", async () => {
    // Premier rendu : liste complete
    const { unmount } = render(
      <Wrapper>
        <CrudListPage<TestItem>
          title="Test"
          fetchCommand={TEST_COMMAND}
          getItemId={(item) => item.id}
          renderItem={(item) => <span data-testid={`item-${item.id}`}>{item.name}</span>}
        />
      </Wrapper>,
    );

    await waitFor(() => expect(screen.getByText("Alpha")).toBeInTheDocument());
    unmount();

    // Second rendu : liste filtree (Alpha retire) — verifie que les keys
    // utilisent getItemId (item.id) et non l'index, en s'assurant qu'aucun
    // attribut React reactKey ne reste base sur l'index.
    invalidateCache();
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_test_items") return [ITEMS[1], ITEMS[2]]; // Beta, Gamma
      if (cmd === "get_admin_status") return { isAdmin: true, canElevate: true };
      return null;
    });

    render(
      <Wrapper>
        <CrudListPage<TestItem>
          title="Test"
          fetchCommand={TEST_COMMAND}
          getItemId={(item) => item.id}
          renderItem={(item) => <span data-testid={`item-${item.id}`}>{item.name}</span>}
        />
      </Wrapper>,
    );

    await waitFor(() => expect(screen.getByTestId("item-2")).toBeInTheDocument());
    expect(screen.queryByText("Alpha")).toBeNull();
    expect(screen.getByTestId("item-3")).toBeInTheDocument();
  });

  it("uses subtitleCompute when provided to render dynamic subtitle from items", async () => {
    type Item = { id: string; size: number };
    const items: Item[] = [
      { id: "a", size: 100 },
      { id: "b", size: 250 },
      { id: "c", size: 50 },
    ];
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_test_items") return items;
      if (cmd === "get_admin_status") return { isAdmin: true, canElevate: true };
      return null;
    });

    render(
      <Wrapper>
        <CrudListPage<Item>
          title="Test"
          fetchCommand={TEST_COMMAND}
          renderItem={(item) => <div>{item.id}</div>}
          subtitleCompute={(items) =>
            `Total: ${items.reduce((s, i) => s + i.size, 0)}`
          }
        />
      </Wrapper>,
    );

    await waitFor(() => {
      expect(screen.getByText("Total: 400")).toBeInTheDocument();
    });
  });
});
