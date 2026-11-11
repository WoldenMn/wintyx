import { useRef } from "react";
import { Badge } from "../../components/ui/Badge";
import { ActionButton } from "../../components/ui/ActionButton";
import { CrudListPage } from "../../components/ui/CrudListPage";
import { useToggleAction } from "../../hooks/use-toggle-action";
import { useConfirmBatchAction } from "../../hooks/use-confirm-batch-action";
import { useScores } from "../../contexts/ScoresContext";
import { getScoreColor } from "../../lib/helpers";
import type { PerformanceItem } from "../../lib/types";

export function PerformancePage() {
  const { scores, refreshScores } = useScores();
  const listRefresh = useRef<(() => void) | null>(null);

  const optimize = useToggleAction<PerformanceItem>({
    doCommand: "optimize_item",
    getId: (item) => item.id,
    confirmTitle: "Appliquer l'optimisation",
    confirmMessage: (item) => `Appliquer "${item.name}" ?`,
    confirmVariant: "warning",
    successMessage: (item) => `${item.name} optimisé`,
    onSettled: () => {
      listRefresh.current?.();
      return refreshScores();
    },
  });

  return (
    <CrudListPage<PerformanceItem>
      title="Performance"
      fetchCommand="get_performance_items"
      refreshRef={listRefresh}
      getItemId={(item) => item.id}
      subtitle={
        <>Score : <span className={`font-bold ${getScoreColor(scores.performance)}`}>{scores.performance}%</span></>
      }
      headerActions={(api) => (
        <OptimizeAllButton
          onSettled={() => {
            api.refresh();
            void refreshScores();
          }}
        />
      )}
      emptyState={{
        title: "Aucune optimisation disponible",
        hint: "Toutes les optimisations sont déjà appliquées. Le système est configuré pour les performances.",
      }}
      renderItem={(item) => {
        const recently = optimize.recentId === item.id;
        return (
          <div
            data-recently-changed={recently ? "true" : undefined}
            className="bg-bg-card border border-border rounded-lg p-5 flex items-center justify-between theme-card transition-all duration-700 data-[recently-changed=true]:ring-2 data-[recently-changed=true]:ring-success/40"
          >
            <div className="flex-1 mr-4 min-w-0">
              <div className="flex items-center gap-3 mb-1 flex-wrap">
                <p className="text-sm font-medium text-fg-primary">{item.name}</p>
                {item.isOptimized ? (
                  <Badge variant="success">Optimisé</Badge>
                ) : (
                  <Badge variant="warning">Non optimisé</Badge>
                )}
              </div>
              <p className="text-xs text-fg-dim">{item.description}</p>
            </div>
            {!item.isOptimized && (
              <ActionButton
                variant="accent"
                loading={optimize.busy === item.id}
                onClick={() => optimize.trigger(item)}
                className="shrink-0"
              >
                Optimiser
              </ActionButton>
            )}
          </div>
        );
      }}
    />
  );
}

function OptimizeAllButton({ onSettled }: { onSettled: () => void }) {
  const batch = useConfirmBatchAction({
    command: "optimize_all",
    confirmTitle: "Tout optimiser",
    confirmMessage: "Toutes les optimisations disponibles vont être appliquées.",
    confirmVariant: "warning",
    noun: { singular: "optimisation", plural: "optimisations", feminine: true },
    snapshotActionName: "Optimisations performance",
    enablePreview: true,
    onSettled,
  });

  return (
    <ActionButton variant="accent" requiresAdmin loading={batch.isRunning || batch.isPreviewing} onClick={batch.trigger}>
      Tout optimiser
    </ActionButton>
  );
}

export default PerformancePage;
