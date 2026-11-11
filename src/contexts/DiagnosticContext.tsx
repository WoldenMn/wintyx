import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { describeError } from "../lib/helpers";
import type { AppDiagnostics } from "../lib/bindings";

interface DiagnosticContextType {
  diagnostics: AppDiagnostics | null;
  loaded: boolean;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

const DiagnosticContext = createContext<DiagnosticContextType | null>(null);

export function DiagnosticProvider({ children }: { children: ReactNode }) {
  const [diagnostics, setDiagnostics] = useState<AppDiagnostics | null>(null);
  const [loaded, setLoaded] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<AppDiagnostics>("get_app_diagnostics");
      setDiagnostics(result);
      setError(null);
    } catch (e) {
      setError(describeError(e));
    } finally {
      setLoaded(true);
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const value = useMemo<DiagnosticContextType>(
    () => ({ diagnostics, loaded, loading, error, refresh }),
    [diagnostics, loaded, loading, error, refresh],
  );

  return <DiagnosticContext.Provider value={value}>{children}</DiagnosticContext.Provider>;
}

export function useDiagnostics() {
  const ctx = useContext(DiagnosticContext);
  if (!ctx) throw new Error("useDiagnostics must be used within DiagnosticProvider");
  return ctx;
}

const FALLBACK: DiagnosticContextType = {
  diagnostics: null,
  loaded: true,
  loading: false,
  error: null,
  refresh: async () => {},
};

/**
 * Variante tolerante : defauts safe hors DiagnosticProvider (tests isoles).
 * loaded=true pour qu'un gate de chargement reste transparent sans provider.
 */
export function useDiagnosticsOptional(): DiagnosticContextType {
  return useContext(DiagnosticContext) ?? FALLBACK;
}
