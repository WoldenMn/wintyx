import { getMemorySlotLabel } from "../../lib/labels";
import { useInvoke } from "../../hooks/use-invoke";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageLoader } from "../../components/ui/PageLoader";
import { ErrorState } from "../../components/ui/ErrorState";
import { Badge } from "../../components/ui/Badge";
import { EmptyState } from "../../components/ui/EmptyState";
import type { MemoryStick, MemoryConsumer, MemorySummary } from "../../lib/types";

export function MemoryPage() {
  const { data: sticks, isLoading: sticksLoading, error: sticksError, refresh: refreshSticks } = useInvoke<MemoryStick[]>("get_memory_sticks");
  const { data: consumers, isLoading: consumersLoading, error: consumersError, refresh: refreshConsumers } = useInvoke<MemoryConsumer[]>("get_memory_consumers");
  const { data: memorySummary, isLoading: summaryLoading, error: summaryError, refresh: refreshSummary } = useInvoke<MemorySummary>("get_memory_summary");

  const isLoading = sticksLoading || consumersLoading || summaryLoading;
  const error = sticksError || consumersError || summaryError;

  if (isLoading) return <PageLoader />;
  if (error)
    return (
      <div className="p-7">
        <ErrorState
          message={error}
          onRetry={() => {
            refreshSticks();
            refreshConsumers();
            refreshSummary();
          }}
        />
      </div>
    );

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Mémoire"
        subtitle="Barrettes RAM installées et consommation mémoire"
      />

      {memorySummary && (
        <div className="grid grid-cols-3 gap-4 mb-6">
          <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
            <p className="text-xs text-fg-muted mb-1">Total</p>
            <p className="text-2xl font-bold text-fg-primary">{memorySummary.totalGb.toFixed(1)} Go</p>
          </div>
          <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
            <p className="text-xs text-fg-muted mb-1">Utilisée</p>
            <p className="text-2xl font-bold text-warning">{memorySummary.usedGb.toFixed(1)} Go</p>
            <p className="text-xs text-fg-dim">{memorySummary.usedPercent.toFixed(0)}%</p>
          </div>
          <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
            <p className="text-xs text-fg-muted mb-1">Slots</p>
            <p className="text-2xl font-bold text-accent">{memorySummary.slotsFilled}/{memorySummary.totalSlots}</p>
            <p className="text-xs text-fg-dim">remplis</p>
          </div>
        </div>
      )}

      <div className="mb-8">
        <h2 className="text-lg font-medium text-fg-primary mb-3">Barrettes RAM</h2>
        {sticks && sticks.length > 0 ? (
          <div className="grid grid-cols-2 gap-4">
            {sticks.map((stick, i) => (
              <div key={`${stick.deviceLocator || stick.bankLabel || "slot"}-${i}`} className="bg-bg-card border border-border rounded-lg p-5 theme-card">
                <div className="flex items-center justify-between mb-2">
                  <p className="text-sm font-medium text-fg-primary">{getMemorySlotLabel(stick, i)}</p>
                  <Badge variant="accent">{stick.capacityGb.toFixed(1)} Go</Badge>
                </div>
                <p className="text-xs text-fg-dim">Vitesse : {stick.speedMhz} MHz</p>
                <p className="text-xs text-fg-dim">Type : {stick.memoryType}</p>
                <p className="text-xs text-fg-dim">Fabricant : {stick.manufacturer}</p>
              </div>
            ))}
          </div>
        ) : (
          <EmptyState
            title="Aucune barrette détectée"
            hint="WMI n'a pas pu lister les modules mémoire physiques. Fréquent sur les systèmes virtualisés ou certains OEM."
          />
        )}
      </div>

      <div>
        <h2 className="text-lg font-medium text-fg-primary mb-3">Consommation mémoire</h2>
        {consumers && consumers.length > 0 ? (
          <div className="space-y-2">
            {consumers.map((consumer, i) => (
              <div key={`${consumer.name}-${i}`} className="bg-bg-card border border-border rounded-lg p-5 flex items-center justify-between theme-card">
                <p className="text-sm font-medium text-fg-primary">{consumer.name}</p>
                <Badge variant="warning">{consumer.memoryMb} Mo</Badge>
              </div>
            ))}
          </div>
        ) : (
          <EmptyState
            title="Aucune donnée de consommation"
            hint="Impossible de lister les processus via WMI. Un redémarrage de Wintyx peut résoudre le problème."
          />
        )}
      </div>
    </div>
  );
}

export default MemoryPage;
