import { memo, useState, useEffect } from "react";
import { X } from "lucide-react";
import { useAdmin } from "../../contexts/AdminContext";
import { useToast } from "../ui/Toast";
import { describeError } from "../../lib/helpers";

interface StatusBarProps {
  /** null tant que les scores ne sont pas charges : etat neutre, jamais un 0 non mesure. */
  healthScore?: number | null;
  /** Le dernier calcul des scores a echoue : la valeur affichee serait trompeuse. */
  scoreError?: boolean;
  statusMessage?: string;
}

const CMDK_HINT_KEY = "wintyx-hint-cmdk-shown";
const CMDK_HINT_DURATION_MS = 8000;

const Clock = memo(function Clock() {
  const [time, setTime] = useState(new Date());

  useEffect(() => {
    const interval = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <span className="text-fg-muted tabular-nums">
      {time.toLocaleTimeString("fr-FR", {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      })}
    </span>
  );
});

function readHintShown(): boolean {
  try {
    return localStorage.getItem(CMDK_HINT_KEY) === "1";
  } catch {
    return true;
  }
}

function setHintShown() {
  try {
    localStorage.setItem(CMDK_HINT_KEY, "1");
  } catch {
    // Ignorer (mode privé ou storage indisponible)
  }
}

export default function StatusBar({
  healthScore = null,
  scoreError = false,
  statusMessage = "Prêt",
}: StatusBarProps) {
  const { isAdmin, canElevate, loaded, relaunch } = useAdmin();
  const toast = useToast();
  const [relaunching, setRelaunching] = useState(false);
  const [showCmdKHint, setShowCmdKHint] = useState(false);

  useEffect(() => {
    if (readHintShown()) return;
    setShowCmdKHint(true);
    const timer = setTimeout(() => {
      setShowCmdKHint(false);
      setHintShown();
    }, CMDK_HINT_DURATION_MS);
    return () => clearTimeout(timer);
  }, []);

  function dismissHint() {
    setShowCmdKHint(false);
    setHintShown();
  }

  let scoreColor = "text-fg-muted";
  let scoreText = "Analyse…";
  if (scoreError) {
    scoreColor = "text-warning";
    scoreText = "inconnu";
  } else if (healthScore !== null) {
    scoreText = `${healthScore}/100`;
    scoreColor =
      healthScore >= 75
        ? "text-success"
        : healthScore >= 50
          ? "text-warning"
          : "text-danger";
  }

  async function handleRelaunch() {
    if (relaunching) return;
    setRelaunching(true);
    try {
      await relaunch();
    } catch (e) {
      toast.error(`Relance impossible : ${describeError(e)}`);
      setRelaunching(false);
    }
  }

  const showAdminBanner = loaded && !isAdmin;

  return (
    <div className="flex items-center justify-between h-7 px-4 bg-bg-sidebar border-t border-border text-xs">
      <div className="flex items-center gap-3 min-w-0">
        <span data-testid="status-score" className={`font-semibold ${scoreColor} shrink-0`}>
          {scoreText}
        </span>
        <span className="text-fg-muted truncate max-w-[40ch]" title={statusMessage}>
          {statusMessage}
        </span>
      </div>
      <div className="flex items-center gap-3">
        {showCmdKHint && (
          <span
            role="status"
            className="inline-flex items-center gap-2 text-accent"
            aria-label="Astuce clavier"
          >
            <span>
              Astuce :{" "}
              <kbd className="px-1.5 py-0.5 bg-bg-elevated border border-border rounded text-xs font-mono">
                Ctrl+K
              </kbd>{" "}
              ouvre la palette de commandes
            </span>
            <button
              type="button"
              onClick={dismissHint}
              aria-label="Fermer l'astuce"
              className="inline-flex items-center text-fg-dim hover:text-fg-primary rounded px-1 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
            >
              <X size={12} aria-hidden="true" />
            </button>
          </span>
        )}
        {showAdminBanner && (
          <span
            role="status"
            className="inline-flex items-center gap-2 text-warning"
            aria-label="Mode non-administrateur, fonctionnalités limitées"
          >
            <span className="inline-block w-1.5 h-1.5 rounded-full bg-warning" aria-hidden="true" />
            <span>Mode non-admin</span>
            {canElevate && (
              <button
                type="button"
                onClick={handleRelaunch}
                disabled={relaunching}
                className="underline underline-offset-2 hover:text-warning/80 focus:outline-none focus-visible:ring-2 focus-visible:ring-warning rounded px-1 disabled:opacity-50"
                aria-label="Relancer Wintyx en mode administrateur"
              >
                {relaunching ? "Relance..." : "Relancer en admin"}
              </button>
            )}
          </span>
        )}
        <Clock />
      </div>
    </div>
  );
}
