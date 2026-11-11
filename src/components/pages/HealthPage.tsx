import { useInvoke } from "../../hooks/use-invoke";
import { useMonitor } from "../../contexts/MonitorContext";
import { Sparkline } from "../../components/ui/Sparkline";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageLoader } from "../../components/ui/PageLoader";
import { ErrorState } from "../../components/ui/ErrorState";
import { getUsageColor, formatUptime } from "../../lib/helpers";
import { ScoreRing } from "../../components/ui/ScoreRing";
import type { SystemInfoData } from "../../lib/types";

const HEALTH_HISTORY = 30;

export function HealthPage() {
  const { data: sysInfo, isLoading, error, refresh } = useInvoke<SystemInfoData>("get_system_info");
  const { metrics, cpuHistory, ramHistory, error: monitorError, resume } = useMonitor();

  if (isLoading) return <PageLoader />;
  if (error) return <div className="p-7"><ErrorState message={error} onRetry={refresh} /></div>;

  const cpuSlice = cpuHistory.slice(-HEALTH_HISTORY);
  const ramSlice = ramHistory.slice(-HEALTH_HISTORY);

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Santé du système"
        subtitle="État instantané à l'ouverture : CPU, RAM et informations système. Pour un suivi continu, voir Monitoring live."
      />

      <div className="grid grid-cols-3 gap-4 mb-6">
        {metrics ? (
          <>
            <div className="bg-bg-card border border-border rounded-lg p-5 theme-card flex flex-col items-center gap-3">
              <p className="text-xs text-fg-muted self-start">CPU</p>
              <ScoreRing value={metrics.cpuUsage} colorClass={getUsageColor(metrics.cpuUsage)} ariaLabel={`Utilisation CPU : ${metrics.cpuUsage}%`} label="util." />
              <Sparkline data={cpuSlice} width={160} height={40} color="var(--color-accent)" ariaLabel="Utilisation CPU" unit="%" />
            </div>

            <div className="bg-bg-card border border-border rounded-lg p-5 theme-card flex flex-col items-center gap-2">
              <p className="text-xs text-fg-muted self-start">RAM</p>
              <ScoreRing value={metrics.ramPercent} colorClass={getUsageColor(metrics.ramPercent)} ariaLabel={`Utilisation RAM : ${metrics.ramPercent}%`} label="util." />
              <p className="text-xs text-fg-dim">
                {metrics.ramUsedGb.toFixed(1)} / {metrics.ramTotalGb.toFixed(1)} Go
              </p>
              <Sparkline data={ramSlice} width={160} height={40} color="var(--color-info)" ariaLabel="Utilisation RAM" unit="%" />
            </div>
          </>
        ) : (
          <div className="col-span-2">
            {monitorError ? (
              <ErrorState
                title="Monitoring indisponible"
                message={monitorError}
                onRetry={() => void resume()}
              />
            ) : (
              <div className="bg-bg-card border border-border rounded-lg p-5 theme-card h-full flex items-center justify-center">
                <p className="text-sm text-fg-dim" role="status">Mesure en cours...</p>
              </div>
            )}
          </div>
        )}

        <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
          <p className="text-xs text-fg-muted mb-2">Durée de fonctionnement</p>
          <p className="text-lg font-bold text-fg-primary">{sysInfo ? formatUptime(sysInfo.uptimeSeconds) : "n.d."}</p>
          {sysInfo && (
            <div className="mt-3 space-y-1">
              <p className="text-xs text-fg-dim truncate" title={sysInfo.osName}>{sysInfo.osName}</p>
              <p className="text-xs text-fg-dim truncate" title={sysInfo.cpuName}>{sysInfo.cpuName}</p>
              <p className="text-xs text-fg-dim">{sysInfo.totalRamGb.toFixed(1)} Go</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default HealthPage;
