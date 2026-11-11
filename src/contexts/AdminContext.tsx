import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { describeError } from "../lib/helpers";
import type { AdminStatus } from "../lib/types";

interface AdminContextType {
  isAdmin: boolean;
  canElevate: boolean;
  loaded: boolean;
  relaunch: () => Promise<void>;
  refresh: () => Promise<void>;
  /** Non-null si get_admin_status a echoue : statut admin inconnu (defaut non-admin). */
  error: string | null;
}

const DEFAULT_STATUS: AdminStatus = {
  isAdmin: false,
  canElevate: false,
};

const AdminContext = createContext<AdminContextType | null>(null);

export function AdminProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<AdminStatus>(DEFAULT_STATUS);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const result = await invoke<AdminStatus>("get_admin_status");
      setStatus(result);
      setError(null);
    } catch (e) {
      setError(describeError(e));
    } finally {
      setLoaded(true);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const relaunch = useCallback(async () => {
    await invoke("relaunch_as_admin");
  }, []);

  const value = useMemo<AdminContextType>(
    () => ({
      isAdmin: status.isAdmin,
      canElevate: status.canElevate,
      loaded,
      relaunch,
      refresh,
      error,
    }),
    [status.isAdmin, status.canElevate, loaded, relaunch, refresh, error],
  );

  return <AdminContext.Provider value={value}>{children}</AdminContext.Provider>;
}

export function useAdmin() {
  const ctx = useContext(AdminContext);
  if (!ctx) throw new Error("useAdmin must be used within AdminProvider");
  return ctx;
}

const FALLBACK: AdminContextType = {
  isAdmin: false,
  canElevate: false,
  loaded: false,
  relaunch: async () => {},
  refresh: async () => {},
  error: null,
};

/**
 * Variante tolerante de useAdmin : retourne des defauts safe quand aucun
 * AdminProvider n'est present dans l'arbre (tests isoles, stories).
 * En production, App.tsx wrap toujours dans AdminProvider.
 */
export function useAdminOptional(): AdminContextType {
  return useContext(AdminContext) ?? FALLBACK;
}
