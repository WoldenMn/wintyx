import { createContext, useContext, useRef, useState, useCallback, useEffect, type ReactNode, type KeyboardEvent as ReactKeyboardEvent } from "react";

interface ConfirmOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: "danger" | "warning" | "default";
}

interface PendingConfirm {
  options: ConfirmOptions;
  resolve: (v: boolean) => void;
}

interface ConfirmContextType {
  confirm: (options: ConfirmOptions) => Promise<boolean>;
}

const ConfirmContext = createContext<ConfirmContextType | null>(null);

export function ConfirmProvider({ children }: { children: ReactNode }) {
  const [queue, setQueue] = useState<PendingConfirm[]>([]);
  const dialogRef = useRef<HTMLDivElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  const confirm = useCallback((options: ConfirmOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      setQueue((prev) => [...prev, { options, resolve }]);
    });
  }, []);

  const current = queue[0];

  const handleClose = useCallback((result: boolean) => {
    setQueue((prev) => {
      if (prev.length === 0) return prev;
      const [first, ...rest] = prev;
      first.resolve(result);
      return rest;
    });
  }, []);

  useEffect(() => {
    if (!current) return;
    previousFocusRef.current = document.activeElement as HTMLElement | null;
    return () => {
      previousFocusRef.current?.focus();
    };
  }, [current]);

  const handleKeyDown = (e: ReactKeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Escape") {
      e.preventDefault();
      handleClose(false);
      return;
    }
    if (e.key === "Enter") {
      if (e.repeat) {
        e.preventDefault();
        return;
      }
      if (e.target instanceof HTMLButtonElement) {
        e.preventDefault();
        e.target.click();
      }
      return;
    }
    if (e.key === "Tab") {
      const focusables = dialogRef.current?.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      );
      if (!focusables || focusables.length === 0) return;
      const first = focusables[0];
      const last = focusables[focusables.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  };

  const isDanger = current?.options.variant === "danger";

  const btnClass =
    current?.options.variant === "danger"
      ? "bg-danger text-on-semantic hover:bg-danger/80"
      : current?.options.variant === "warning"
        ? "bg-warning text-on-semantic hover:bg-warning/80"
        : "bg-accent text-on-accent hover:bg-accent-hover";

  return (
    <ConfirmContext.Provider value={{ confirm }}>
      {children}
      {current && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div
            className="absolute inset-0 bg-black/60 backdrop-blur-sm"
            onClick={() => handleClose(false)}
            aria-hidden="true"
          />
          <div
            ref={dialogRef}
            role="dialog"
            aria-modal="true"
            aria-labelledby="confirm-title"
            aria-describedby="confirm-desc"
            tabIndex={-1}
            onKeyDown={handleKeyDown}
            className="relative bg-bg-card border border-border rounded-xl p-6 max-w-md w-full mx-4 shadow-2xl focus:outline-none"
          >
            <h3 id="confirm-title" className="text-lg font-semibold text-fg-primary mb-2">
              {current.options.title}
            </h3>
            <p id="confirm-desc" className="text-sm text-fg-dim mb-6 whitespace-pre-line">
              {current.options.message}
            </p>
            <div className="flex justify-end gap-3">
              <button
                autoFocus={isDanger}
                className="px-4 py-2 text-sm bg-bg-elevated border border-border text-fg-primary rounded-md hover:bg-bg-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
                onClick={() => handleClose(false)}
              >
                {current.options.cancelLabel ?? "Annuler"}
              </button>
              <button
                autoFocus={!isDanger}
                className={`px-4 py-2 text-sm rounded-md focus:outline-none focus-visible:ring-2 focus-visible:ring-accent ${btnClass}`}
                onClick={() => handleClose(true)}
              >
                {current.options.confirmLabel ?? "Confirmer"}
              </button>
            </div>
          </div>
        </div>
      )}
    </ConfirmContext.Provider>
  );
}

export function useConfirm() {
  const ctx = useContext(ConfirmContext);
  if (!ctx) throw new Error("useConfirm must be used within ConfirmProvider");
  return ctx.confirm;
}
