import { useMonitor } from "../../contexts/MonitorContext";
import { Sparkline } from "../../components/ui/Sparkline";
import { PageHeader } from "../../components/ui/PageHeader";
import { Badge } from "../../components/ui/Badge";
import { ActionButton } from "../../components/ui/ActionButton";
import { ErrorState } from "../../components/ui/ErrorState";
import { getUsageBarColor } from "../../lib/helpers";

export function MonitorPage() {
  const { metrics, cpuHistory, ramHistory, isRunning, error, intervalMs, measuredIntervalMs, intervalError, setInterval, pause, resume } = useMonitor();

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Monitoring live"
        subtitle="Suivi continu en temps réel (CPU/RAM rafraîchis automatiquement). Pour un état ponctuel, voir Santé du système."
        actions={
          <ActionButton
            variant={isRunning ? "danger" : "accent"}
            onClick={isRunning ? pause : resume}
          >
            {isRunning ? "Arrêter" : "Démarrer"}
          </ActionButton>
        }
      />

      {error && (
        <div className="mb-4">
          <ErrorState title="Monitoring indisponible" message={error} onRetry={() => void resume()} />
        </div>
      )}

      <div className="bg-bg-card border border-border rounded-lg p-4 mb-4 flex items-center gap-4 theme-card">
        <label htmlFor="monitor-interval" className="text-sm text-fg-dim shrink-0">Intervalle</label>
        <input
          id="monitor-interval"
          type="range"
          min={1000}
          max={5000}
          step={250}
          value={intervalMs}
          onChange={(e) => void setInterval(Number(e.target.value))}
          className="flex-1 accent-accent"
        />
        <span className="text-xs text-fg-primary font-medium w-20 text-right tabular-nums">
          {(intervalMs / 1000).toFixed(2)} s
        </span>
        <span className="text-xs text-fg-dim w-44 text-right tabular-nums" data-testid="measured-interval">
          {measuredIntervalMs === null ? "Réel : mesure en cours" : `Réel : ${(measuredIntervalMs / 1000).toFixed(2)} s`}
        </span>
      </div>
      {intervalError && (
        <p role="alert" className="text-xs text-danger -mt-2 mb-4">Intervalle non modifié : {intervalError}</p>
      )}

      <div className="grid grid-cols-2 auto-rows-fr gap-4 mb-6">
        <div className="bg-bg-card border border-border rounded-lg p-5 min-w-[180px] theme-card">
          <p className="text-xs text-fg-dim uppercase tracking-wider mb-1">CPU</p>
          <p className="text-3xl font-bold text-fg-primary">{metrics?.cpuUsage.toFixed(1) ?? "—"}%</p>
          {cpuHistory.length > 1 && (
            <div className="mt-3">
              <Sparkline data={cpuHistory} color="var(--color-accent)" ariaLabel="Historique CPU 60 secondes" unit="%" />
              <p className="text-xs text-fg-muted mt-1">60 dernières mesures</p>
            </div>
          )}
        </div>

        <div className="bg-bg-card border border-border rounded-lg p-5 min-w-[180px] theme-card">
          <p className="text-xs text-fg-dim uppercase tracking-wider mb-1">RAM</p>
          <p className="text-3xl font-bold text-fg-primary">
            {metrics ? `${metrics.ramUsedGb.toFixed(1)}/${metrics.ramTotalGb.toFixed(1)} Go` : "—"}
          </p>
          <p className="text-sm text-fg-dim">{metrics ? `${metrics.ramPercent.toFixed(1)}%` : ""}</p>
          {ramHistory.length > 1 && (
            <div className="mt-3">
              <Sparkline data={ramHistory} color="var(--color-info)" ariaLabel="Historique RAM 60 secondes" unit="%" />
              <p className="text-xs text-fg-muted mt-1">60 dernières mesures</p>
            </div>
          )}
        </div>
      </div>

      {metrics && metrics.cpuPerCore.length > 0 && (
        <div className="mb-6">
          <h2 className="text-lg font-semibold text-fg-primary mb-3">CPU par cœur</h2>
          <div className="grid grid-cols-4 gap-2">
            {metrics.cpuPerCore.map((usage, i) => (
              <div key={`core-${i}`} className="bg-bg-card border border-border rounded-lg p-3 theme-card">
                <div className="flex items-center justify-between mb-1">
                  <span className="text-xs text-fg-dim">Cœur {i}</span>
                  <span className="text-xs text-fg-primary font-medium">{usage.toFixed(0)}%</span>
                </div>
                <div className="h-2 bg-bg-elevated rounded-full overflow-hidden">
                  <div
                    className={`h-full rounded-full transition-all ${getUsageBarColor(usage)}`}
                    style={{ width: `${Math.min(usage, 100)}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {metrics && metrics.topProcesses.length > 0 && (
        <div>
          <h2 className="text-lg font-semibold text-fg-primary mb-3">Top processus</h2>
          <div className="bg-bg-card border border-border rounded-lg overflow-hidden theme-card">
            <table className="w-full text-sm">
              <caption className="sr-only">Top processus par utilisation CPU instantanée</caption>
              <thead>
                <tr className="border-b border-border text-fg-dim text-left">
                  <th scope="col" className="px-4 py-2 font-medium">Nom</th>
                  <th scope="col" className="px-4 py-2 font-medium">PID</th>
                  <th scope="col" className="px-4 py-2 font-medium text-right">CPU %</th>
                  <th scope="col" className="px-4 py-2 font-medium text-right">Mémoire</th>
                </tr>
              </thead>
              <tbody>
                {metrics.topProcesses.map((proc) => (
                  <tr key={proc.pid} className="border-b border-border/50 last:border-0">
                    <td className="px-4 py-2 text-fg-primary">{proc.name}</td>
                    <td className="px-4 py-2 text-fg-dim">{proc.pid}</td>
                    <td className="px-4 py-2 text-right">
                      <Badge variant="accent">{proc.cpuPercent.toFixed(1)}%</Badge>
                    </td>
                    <td className="px-4 py-2 text-right">
                      <Badge variant="info">{proc.memoryMb.toFixed(0)} Mo</Badge>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}

export default MonitorPage;
