import { useState, useEffect, useMemo, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageLoader } from "../../components/ui/PageLoader";
import { ActionButton } from "../../components/ui/ActionButton";
import { EmptyState } from "../../components/ui/EmptyState";
import { ErrorState } from "../../components/ui/ErrorState";
import { describeError } from "../../lib/helpers";
import type { LogEntry, LevelFilter } from "../../lib/types";

const SCROLL_EPSILON = 24;

const ACTION_CATEGORIES = new Set([
  "backup",
  "services",
  "hardening",
  "data_protection",
  "performance",
  "bloatware",
  "startup",
  "cleanup",
  "admin",
]);

export function extractCategory(line: string): string | null {
  // Format: [YYYY-MM-DD HH:MM:SS] [LEVEL] [category] message
  const matches = line.match(/\[([^\]]+)\]/g);
  if (!matches || matches.length < 3) return null;
  return matches[2].slice(1, -1).toLowerCase();
}

type ScopeFilter = "all" | "actions";

export function LogsPage() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [hasLoaded, setHasLoaded] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [filter, setFilter] = useState<LevelFilter>("all");
  const [scope, setScope] = useState<ScopeFilter>("all");
  const [isPaused, setIsPaused] = useState(false);
  const [userScrolled, setUserScrolled] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const scrollRef = useRef<HTMLDivElement | null>(null);
  // Sequence de la derniere requete : un changement rapide de filtre peut faire
  // resoudre une requete lente apres une plus recente. On ignore tout resultat
  // qui n'est pas celui de la derniere requete emise (front-04).
  const fetchSeqRef = useRef(0);

  const fetchLogs = useCallback(async () => {
    const seq = ++fetchSeqRef.current;
    try {
      const result = await invoke<LogEntry[]>("get_logs", {
        levelFilter: filter,
        limit: 200,
      });
      if (seq !== fetchSeqRef.current) return;
      setErrorMessage(null);
      setLogs(result);
    } catch (e) {
      if (seq !== fetchSeqRef.current) return;
      setErrorMessage(describeError(e));
      setLogs([]);
    } finally {
      if (seq === fetchSeqRef.current) {
        setIsLoading(false);
        setHasLoaded(true);
      }
    }
  }, [filter]);

  useEffect(() => {
    setIsLoading(true);
    fetchLogs();
  }, [fetchLogs]);

  useEffect(() => {
    if (isPaused) return;
    intervalRef.current = setInterval(fetchLogs, 5000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [fetchLogs, isPaused]);

  useEffect(() => {
    if (userScrolled || !scrollRef.current) return;
    scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
  }, [logs, userScrolled]);

  const handleScroll = () => {
    const el = scrollRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < SCROLL_EPSILON;
    setUserScrolled(!atBottom);
  };

  const scrollToBottom = () => {
    const el = scrollRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
    setUserScrolled(false);
  };

  function getLevelColor(level: string) {
    switch (level.toLowerCase()) {
      case "warn":
      case "warning":
        return "text-warning";
      case "error":
      case "critical":
        return "text-danger";
      default:
        return "text-fg-dim";
    }
  }

  const displayedLogs = useMemo(() => {
    if (scope === "all") return logs;
    return logs.filter((entry) => {
      const cat = extractCategory(entry.line);
      return cat !== null && ACTION_CATEGORIES.has(cat);
    });
  }, [logs, scope]);

  if (isLoading && !hasLoaded) return <PageLoader />;

  const subtitle = isPaused
    ? `${displayedLogs.length} entrées, auto-refresh en pause`
    : `${displayedLogs.length} entrées, auto-refresh toutes les 5 s`;

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Logs"
        subtitle={subtitle}
        actions={
          <>
            <select
              value={scope}
              onChange={(e) => setScope(e.target.value as ScopeFilter)}
              aria-label="Portée des logs"
              className="px-3 py-2 bg-bg-card border border-border rounded-md text-sm text-fg-primary focus:outline-none focus:ring-1 focus:ring-accent focus-visible:ring-2 focus-visible:ring-accent"
            >
              <option value="all">Tous les logs</option>
              <option value="actions">Actions uniquement</option>
            </select>
            <select
              value={filter}
              onChange={(e) => setFilter(e.target.value as LevelFilter)}
              aria-label="Niveau de log"
              className="px-3 py-2 bg-bg-card border border-border rounded-md text-sm text-fg-primary focus:outline-none focus:ring-1 focus:ring-accent focus-visible:ring-2 focus-visible:ring-accent"
            >
              <option value="all">Tous niveaux</option>
              <option value="info">Info</option>
              <option value="warn">Avertissement</option>
              <option value="error">Erreur</option>
            </select>
            <ActionButton
              variant={isPaused ? "success" : "neutral"}
              onClick={() => setIsPaused((p) => !p)}
            >
              {isPaused ? "Reprendre" : "Pause"}
            </ActionButton>
            <ActionButton variant="accent" onClick={fetchLogs}>
              Rafraîchir
            </ActionButton>
          </>
        }
      />

      <div className="bg-bg-card border border-border rounded-lg overflow-hidden theme-card relative">
        <div
          ref={scrollRef}
          onScroll={handleScroll}
          aria-busy={isLoading || undefined}
          className="overflow-y-auto max-h-[calc(100vh-220px)] p-4"
        >
          {errorMessage ? (
            <ErrorState message={errorMessage} onRetry={fetchLogs} />
          ) : displayedLogs.length === 0 ? (
            <EmptyState
              title={
                scope === "actions"
                  ? "Aucune action enregistrée"
                  : "Aucun log disponible"
              }
              hint={
                scope === "actions"
                  ? "Aucune action batch (sauvegarde, modification registre, nettoyage) n'a encore été tracée. Elles apparaissent ici après exécution."
                  : "Les logs apparaissent après chaque action tracée (sauvegarde, modification registre, blocage admin). Ils sont stockés dans %LOCALAPPDATA%\\Wintyx\\Logs\\."
              }
            />
          ) : (
            <div className="font-mono text-xs leading-relaxed space-y-0.5">
              {displayedLogs.map((entry, i) => (
                <div key={`${i}-${entry.level}-${entry.line.slice(0, 24)}`}>
                  {/* La ligne brute contient deja [niveau] : pas de badge separe
                      pour eviter l'affichage en double (run-20). Couleur par niveau. */}
                  <span className={getLevelColor(entry.level)}>{entry.line}</span>
                </div>
              ))}
            </div>
          )}
        </div>

        {userScrolled && (
          <button
            onClick={scrollToBottom}
            className="absolute bottom-4 right-4 px-3 py-1.5 bg-accent text-on-accent text-xs rounded-md hover:bg-accent-hover shadow-lg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent"
          >
            Descendre en bas
          </button>
        )}
      </div>
    </div>
  );
}

export default LogsPage;
