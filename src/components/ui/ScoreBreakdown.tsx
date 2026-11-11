import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Badge } from "./Badge";
import { describeError, groupBy } from "../../lib/helpers";
import { reportFrontendError } from "../../lib/frontend-log";
import {
  getDomainConfig,
  type BreakdownItem,
  type ScoreDomain,
} from "../../lib/scoreBreakdownNormalizers";

// Re-export retro-compatible : certains imports referencent ScoreDomain via ce module.
export type { ScoreDomain };

interface ScoreBreakdownProps {
  domain: ScoreDomain | null;
  onClose: () => void;
}

export function ScoreBreakdown({ domain, onClose }: ScoreBreakdownProps) {
  const [items, setItems] = useState<BreakdownItem[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [attempt, setAttempt] = useState(0);
  const dialogRef = useRef<HTMLDivElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);
  const onCloseRef = useRef(onClose);

  useEffect(() => {
    onCloseRef.current = onClose;
  }, [onClose]);

  const config = useMemo(() => (domain ? getDomainConfig(domain) : null), [domain]);

  useEffect(() => {
    if (!config) {
      setItems(null);
      setError(null);
      return;
    }
    let cancelled = false;
    setItems(null);
    setError(null);
    invoke<unknown>(config.command)
      .then((data) => {
        if (cancelled) return;
        setItems(config.normalize(data));
      })
      .catch((e) => {
        if (cancelled) return;
        reportFrontendError(`ScoreBreakdown:${domain}`, e);
        setError(describeError(e));
      });
    return () => {
      cancelled = true;
    };
  }, [config, domain, attempt]);

  useEffect(() => {
    if (!domain) return;
    previousFocusRef.current = document.activeElement as HTMLElement | null;
    closeButtonRef.current?.focus();

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onCloseRef.current();
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
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      previousFocusRef.current?.focus();
    };
  }, [domain]);

  const grouped = useMemo(() => {
    if (!items) return null;
    return groupBy(items, "category");
  }, [items]);

  if (!domain || !config) return null;

  const total = items?.length ?? 0;
  const ok = items?.filter((i) => i.isOk).length ?? 0;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      role="presentation"
    >
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={onClose}
        aria-hidden="true"
      />
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="breakdown-title"
        aria-describedby="breakdown-summary"
        className="relative bg-bg-card border border-border rounded-xl shadow-2xl max-w-2xl w-full mx-4 max-h-[80vh] flex flex-col"
      >
        <div className="flex items-start justify-between p-5 border-b border-border">
          <div>
            <h3
              id="breakdown-title"
              className="text-lg font-semibold text-fg-primary"
            >
              {config.title}
            </h3>
            <p id="breakdown-summary" className="text-xs text-fg-dim mt-1">
              {items
                ? `${ok}/${total} items au bon état`
                : error
                  ? "Erreur de chargement"
                  : "Chargement..."}
            </p>
          </div>
          <button
            type="button"
            ref={closeButtonRef}
            onClick={onClose}
            aria-label="Fermer"
            className="text-fg-dim hover:text-fg-primary rounded-md px-2 py-1 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
          >
            Fermer
          </button>
        </div>

        <div className="overflow-y-auto p-5">
          {error && (
            <div className="flex items-center justify-between gap-3" role="alert">
              <p className="text-sm text-danger">{error}</p>
              <button
                type="button"
                onClick={() => setAttempt((n) => n + 1)}
                className="shrink-0 text-sm text-accent hover:underline rounded-md px-2 py-1 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
              >
                Réessayer
              </button>
            </div>
          )}
          {!error && !items && (
            <p className="text-sm text-fg-dim">Chargement des items...</p>
          )}
          {!error && items && items.length === 0 && (
            <p className="text-sm text-fg-dim">Aucun élément.</p>
          )}
          {grouped && (
            <div className="space-y-5">
              {Object.entries(grouped).map(([category, categoryItems]) => (
                <section key={category}>
                  <h4 className="text-xs font-medium text-fg-muted uppercase tracking-wider mb-2">
                    {category}{" "}
                    <span className="text-fg-dim font-normal">
                      ({categoryItems.filter((i) => i.isOk).length}/
                      {categoryItems.length})
                    </span>
                  </h4>
                  <ul className="space-y-2">
                    {categoryItems.map((item) => (
                      <li
                        key={item.id}
                        className="flex items-start justify-between gap-3 bg-bg-secondary rounded-md px-3 py-2"
                      >
                        <div className="min-w-0 flex-1">
                          <p className="text-sm text-fg-primary">{item.name}</p>
                          {item.description && (
                            <p className="text-xs text-fg-dim mt-0.5 break-words">
                              {item.description}
                            </p>
                          )}
                        </div>
                        <div className="shrink-0 flex flex-col items-end gap-1">
                          <Badge variant={item.isOk ? "success" : "warning"}>
                            {item.statusLabel}
                          </Badge>
                          {item.weightLabel && (
                            <span className="text-xs text-fg-dim">
                              {item.weightLabel}
                            </span>
                          )}
                        </div>
                      </li>
                    ))}
                  </ul>
                </section>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default ScoreBreakdown;
