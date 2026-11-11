import { ErrorState } from "../../components/ui/ErrorState";
import { getScoreColor } from "../../lib/helpers";
import { isPreviousVersionEntry } from "../../lib/benchmark";
import type { BenchmarkEntry } from "../../lib/types";

// Formate les debits (Mo/s) avec separateurs de milliers : un score memoire eleve
// (ou une vieille entree d'historique a une autre echelle) reste lisible plutot que
// "100000000" colle (run-22). Coherent entre cartes live et tableau d'historique.
export const formatThroughput = (v: number): string => Math.round(v).toLocaleString("fr-FR");

const RECENT_COUNT = 5;

function formatTimestamp(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString("fr-FR", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

interface Props {
  history: BenchmarkEntry[];
  error: string | null;
  onRetry: () => void;
}

export function BenchmarkHistory({ history, error, onRetry }: Props) {
  const recent = history.slice(0, RECENT_COUNT);

  if (recent.length === 0) {
    return error ? (
      <div className="mt-6">
        <ErrorState message={error} onRetry={onRetry} title="Historique indisponible" />
      </div>
    ) : null;
  }

  return (
    <div className="mt-6">
      <h2 className="text-lg font-semibold text-fg-primary mb-3">Historique récent</h2>
      <div className="bg-bg-card border border-border rounded-lg overflow-hidden theme-card">
        <table className="w-full text-sm">
          <caption className="sr-only">Historique des benchmarks récents avec scores CPU, mémoire et disque</caption>
          <thead>
            <tr className="border-b border-border text-fg-dim text-left">
              <th scope="col" className="px-4 py-2 font-medium">Date</th>
              <th scope="col" className="px-4 py-2 font-medium text-right">Score</th>
              <th scope="col" className="px-4 py-2 font-medium text-right">CPU</th>
              <th scope="col" className="px-4 py-2 font-medium text-right">Mémoire</th>
              <th scope="col" className="px-4 py-2 font-medium text-right">Disque</th>
            </tr>
          </thead>
          <tbody>
            {recent.map((entry) => (
              <tr key={entry.timestamp} className="border-b border-border/50 last:border-0">
                <td className="px-4 py-2 text-fg-primary">
                  {formatTimestamp(entry.timestamp)}
                  {isPreviousVersionEntry(entry) && (
                    <span className="block text-xs text-fg-muted italic">
                      Mesure d&apos;une version précédente, non comparable
                    </span>
                  )}
                </td>
                {entry.plausible !== false ? (
                  <>
                    <td className={`px-4 py-2 text-right font-semibold ${getScoreColor(entry.result.overallScore)}`}>
                      {entry.result.overallScore.toFixed(0)}
                    </td>
                    <td className="px-4 py-2 text-right text-fg-dim">{entry.result.cpuScore.toFixed(1)}</td>
                    <td className="px-4 py-2 text-right text-fg-dim">{formatThroughput(entry.result.memoryScore)}</td>
                    <td className="px-4 py-2 text-right text-fg-dim">{formatThroughput(entry.result.diskReadScore)}</td>
                  </>
                ) : (
                  <td colSpan={4} className="px-4 py-2 text-right text-fg-muted italic">
                    Mesure invalide (ancienne version)
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
