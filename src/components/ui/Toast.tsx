import { createContext, useContext, useEffect, useMemo, useRef, useState, useCallback, type ReactNode } from "react";
import { CheckCircle2, XCircle, AlertTriangle, Info } from "lucide-react";
import { reportFrontendError } from "../../lib/frontend-log";
import { describeError } from "../../lib/helpers";

type ToastType = "success" | "error" | "warning" | "info";

const ICON_FOR: Record<ToastType, ReactNode> = {
  success: <CheckCircle2 size={16} className="shrink-0 mt-0.5" aria-hidden="true" />,
  error: <XCircle size={16} className="shrink-0 mt-0.5" aria-hidden="true" />,
  warning: <AlertTriangle size={16} className="shrink-0 mt-0.5" aria-hidden="true" />,
  info: <Info size={16} className="shrink-0 mt-0.5" aria-hidden="true" />,
};

export interface ToastAction {
  label: string;
  onClick: () => void | Promise<void>;
}

export interface ToastOptions {
  action?: ToastAction;
  durationMs?: number;
}

interface Toast {
  id: string;
  message: string;
  type: ToastType;
  action?: ToastAction;
}

interface ToastContextType {
  toast: (message: string, type?: ToastType, options?: ToastOptions) => void;
  success: (message: string, options?: ToastOptions) => void;
  error: (message: string, options?: ToastOptions) => void;
  warning: (message: string, options?: ToastOptions) => void;
  info: (message: string, options?: ToastOptions) => void;
}

const ToastContext = createContext<ToastContextType | null>(null);

const DEFAULT_DURATION_MS = 3500;
const DURATION_WITH_ACTION_MS = 6000;

function generateId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `toast-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const mountedRef = useRef(true);
  const timersRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());
  const runningActionsRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    mountedRef.current = true;
    const timers = timersRef.current;
    return () => {
      mountedRef.current = false;
      // Nettoyer les timers d'auto-dismiss encore en attente (bug-frontend-03).
      timers.forEach((t) => clearTimeout(t));
      timers.clear();
    };
  }, []);

  const dismiss = useCallback((id: string) => {
    const timer = timersRef.current.get(id);
    if (timer !== undefined) {
      clearTimeout(timer);
      timersRef.current.delete(id);
    }
    if (!mountedRef.current) return;
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const addToast = useCallback(
    (message: string, type: ToastType = "info", options?: ToastOptions) => {
      const id = generateId();
      if (type === "error") reportFrontendError("toast", message);
      setToasts((prev) => [...prev, { id, message, type, action: options?.action }]);
      const duration =
        options?.durationMs ??
        (options?.action ? DURATION_WITH_ACTION_MS : DEFAULT_DURATION_MS);
      timersRef.current.set(id, setTimeout(() => dismiss(id), duration));
    },
    [dismiss],
  );

  const ctx = useMemo<ToastContextType>(
    () => ({
      toast: addToast,
      success: (m, opts) => addToast(m, "success", opts),
      error: (m, opts) => addToast(m, "error", opts),
      warning: (m, opts) => addToast(m, "warning", opts),
      info: (m, opts) => addToast(m, "info", opts),
    }),
    [addToast],
  );

  const colors: Record<ToastType, string> = {
    success: "border-success/40 bg-success/10 text-success",
    error: "border-danger/40 bg-danger/10 text-danger",
    warning: "border-warning/40 bg-warning/10 text-warning",
    info: "border-info/40 bg-info/10 text-info",
  };

  async function handleAction(t: Toast) {
    if (!t.action || runningActionsRef.current.has(t.id)) return;
    runningActionsRef.current.add(t.id);
    try {
      await t.action.onClick();
    } catch (e) {
      reportFrontendError(`action de toast ${t.action.label}`, e);
      addToast(`Action impossible : ${describeError(e)}`, "error");
    } finally {
      runningActionsRef.current.delete(t.id);
      // Guard : ne pas dismiss si le provider a ete unmount pendant l'action
      if (mountedRef.current) {
        dismiss(t.id);
      }
    }
  }

  return (
    <ToastContext.Provider value={ctx}>
      {children}
      <div
        role="region"
        aria-label="Notifications"
        className="fixed bottom-12 right-4 z-50 flex flex-col gap-2 pointer-events-none"
      >
        {toasts.map((t) => (
          <div
            key={t.id}
            role={t.type === "error" ? "alert" : "status"}
            aria-live={t.type === "error" ? "assertive" : "polite"}
            aria-atomic="true"
            className={`pointer-events-auto border rounded-lg px-4 py-3 text-sm shadow-lg backdrop-blur-sm animate-slide-in min-w-[280px] max-w-[400px] flex items-start gap-3 ${colors[t.type]}`}
          >
            {ICON_FOR[t.type]}
            <span className="text-fg-primary flex-1">{t.message}</span>
            {t.action && (
              <button
                type="button"
                onClick={() => void handleAction(t)}
                className="shrink-0 underline underline-offset-2 text-fg-primary hover:opacity-80 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent rounded px-1"
              >
                {t.action.label}
              </button>
            )}
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast() {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error("useToast must be used within ToastProvider");
  return ctx;
}
