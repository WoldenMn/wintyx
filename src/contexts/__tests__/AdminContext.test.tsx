import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, renderHook, waitFor, act } from "@testing-library/react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AdminProvider, useAdmin, useAdminOptional } from "../AdminContext";
import type { AdminStatus } from "../../lib/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

const STATUS_ADMIN: AdminStatus = { isAdmin: true, canElevate: true };
const STATUS_USER: AdminStatus = { isAdmin: false, canElevate: true };

function mockStatus(status: AdminStatus) {
  mockedInvoke.mockImplementation(async (cmd: string) => {
    if (cmd === "get_admin_status") return status;
    return null;
  });
}

function Wrapper({ children }: { children: ReactNode }) {
  return <AdminProvider>{children}</AdminProvider>;
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("AdminContext", () => {
  it("fetches admin status on mount and exposes it", async () => {
    mockStatus(STATUS_ADMIN);
    const { result } = renderHook(() => useAdmin(), { wrapper: Wrapper });

    await waitFor(() => {
      expect(result.current.loaded).toBe(true);
    });
    expect(result.current.isAdmin).toBe(true);
    expect(result.current.canElevate).toBe(true);
    expect(mockedInvoke).toHaveBeenCalledWith("get_admin_status");
  });

  it("exposes isAdmin=false when running as user", async () => {
    mockStatus(STATUS_USER);
    const { result } = renderHook(() => useAdmin(), { wrapper: Wrapper });

    await waitFor(() => {
      expect(result.current.loaded).toBe(true);
    });
    expect(result.current.isAdmin).toBe(false);
  });

  it("useAdmin throws outside AdminProvider", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    expect(() => renderHook(() => useAdmin())).toThrow(/AdminProvider/);
    spy.mockRestore();
  });

  it("useAdminOptional returns safe defaults without provider", () => {
    const { result } = renderHook(() => useAdminOptional());
    expect(result.current.loaded).toBe(false);
    expect(result.current.isAdmin).toBe(false);
    expect(result.current.canElevate).toBe(false);
  });

  it("relaunch invokes relaunch_as_admin", async () => {
    mockStatus(STATUS_USER);
    const { result } = renderHook(() => useAdmin(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.loaded).toBe(true));

    mockedInvoke.mockResolvedValueOnce(undefined);
    await act(async () => {
      await result.current.relaunch();
    });
    expect(mockedInvoke).toHaveBeenCalledWith("relaunch_as_admin");
  });

  it("initial render shows loaded=false before first fetch resolves", () => {
    let resolveStatus: ((v: AdminStatus) => void) | undefined;
    mockedInvoke.mockImplementation(() => {
      return new Promise<AdminStatus>((r) => {
        resolveStatus = r;
      });
    });
    const { result } = renderHook(() => useAdmin(), { wrapper: Wrapper });
    expect(result.current.loaded).toBe(false);
    act(() => {
      resolveStatus?.(STATUS_ADMIN);
    });
  });

  it("render integrates without error", () => {
    mockStatus(STATUS_USER);
    const { container } = render(
      <AdminProvider>
        <span>child</span>
      </AdminProvider>,
    );
    expect(container.textContent).toBe("child");
  });
});
