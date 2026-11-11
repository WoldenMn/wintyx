import { Sparkline } from "./Sparkline";
import { computeScoreDelta } from "../../lib/helpers";
import type { ScoreSnapshot } from "../../lib/bindings";

const WINDOWS = [7, 30];

interface ScoreTrendCardProps {
  history: ScoreSnapshot[];
  error?: string | null;
  onRetry?: () => void;
}

export function ScoreTrendCard({ history, error, onRetry }: ScoreTrendCardProps) {
  if (error) {
    return (
      <div role="alert" className="bg-bg-card border border-danger/30 rounded-lg p-5 mb-6 theme-card">
        <p className="text-sm font-medium text-fg-primary mb-2">Évolution</p>
        <p className="text-xs text-danger">Historique des scores indisponible : {error}</p>
        {onRetry && (
          <button
            type="button"
            onClick={onRetry}
            className="mt-2 text-xs text-accent hover:underline rounded px-1 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
          >
            Réessayer
          </button>
        )}
      </div>
    );
  }

  if (history.length < 2) {
    return (
      <div className="bg-bg-card border border-border rounded-lg p-5 mb-6 theme-card">
        <p className="text-sm font-medium text-fg-primary mb-2">Évolution</p>
        <p className="text-xs text-fg-muted">
          Collecte en cours : revenez demain pour voir la tendance du score.
        </p>
      </div>
    );
  }

  const series = history.slice(-30).map((s) => s.globalScore);

  return (
    <div className="bg-bg-card border border-border rounded-lg p-5 mb-6 theme-card">
      <p className="text-sm font-medium text-fg-primary mb-3">Évolution</p>
      <div className="flex items-center gap-8">
        <Sparkline
          data={series}
          width={260}
          height={48}
          color="var(--color-accent)"
          ariaLabel="Évolution du score global"
          unit=""
        />
        <div className="flex gap-8">
          {WINDOWS.map((d) => {
            const delta = computeScoreDelta(history, d);
            const color =
              delta === null || delta === 0 ? "text-fg-dim" : delta > 0 ? "text-success" : "text-danger";
            const text = delta === null ? "—" : `${delta > 0 ? "+" : ""}${delta}`;
            return (
              <div key={d}>
                <p className="text-xs text-fg-muted">{d}j</p>
                <p className={`text-lg font-semibold tabular-nums ${color}`}>{text}</p>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

export default ScoreTrendCard;
