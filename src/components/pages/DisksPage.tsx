import { useInvoke } from "../../hooks/use-invoke";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageLoader } from "../../components/ui/PageLoader";
import { ErrorState } from "../../components/ui/ErrorState";
import { Badge } from "../../components/ui/Badge";
import { EmptyState } from "../../components/ui/EmptyState";
import { getUsageColor, getUsageBarColor } from "../../lib/helpers";
import type { DiskInfo } from "../../lib/types";

export function DisksPage() {
  const { data: disks, isLoading, error, refresh } = useInvoke<DiskInfo[]>("get_disks");

  if (isLoading) return <PageLoader />;
  if (error) return <div className="p-7"><ErrorState message={error} onRetry={refresh} /></div>;

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader title="Disques" subtitle={`${disks?.length ?? 0} disques détectés`} />

      <div className="space-y-3">
        {disks?.map((disk) => (
          <div key={disk.name} className="bg-bg-card border border-border rounded-lg p-5 theme-card">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-3 flex-wrap">
                <p className="text-sm font-medium text-fg-primary">
                  {disk.name} {disk.volumeName && `(${disk.volumeName})`}
                </p>
                <Badge>{disk.diskType}</Badge>
                {disk.filesystem && <Badge variant="accent">{disk.filesystem}</Badge>}
              </div>
              <p className={`text-lg font-bold ${getUsageColor(disk.usedPercent)}`}>
                {disk.usedPercent}%
              </p>
            </div>

            <div className="w-full bg-bg-elevated rounded-full h-2.5 mb-3">
              <div
                className={`h-2.5 rounded-full transition-all ${getUsageBarColor(disk.usedPercent)}`}
                style={{ width: `${Math.min(disk.usedPercent, 100)}%` }}
              />
            </div>

            <div className="flex items-center justify-between text-xs text-fg-muted">
              <span>Utilisé : {disk.usedGb.toFixed(1)} Go</span>
              <span>Libre : {disk.freeGb.toFixed(1)} Go</span>
              <span>Total : {disk.totalGb.toFixed(1)} Go</span>
            </div>
          </div>
        ))}

        {(!disks || disks.length === 0) && (
          <EmptyState
            title="Aucun disque détecté"
            hint="Wintyx scanne les disques locaux via WMI. Les disques amovibles sont inclus s'ils sont connectés au moment du scan."
          />
        )}
      </div>
    </div>
  );
}

export default DisksPage;
