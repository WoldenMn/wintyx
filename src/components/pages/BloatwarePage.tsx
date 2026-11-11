import { useRef } from "react";
import { Badge } from "../../components/ui/Badge";
import { ActionButton } from "../../components/ui/ActionButton";
import { CrudListPage } from "../../components/ui/CrudListPage";
import { useToggleAction } from "../../hooks/use-toggle-action";
import { useConfirmBatchAction } from "../../hooks/use-confirm-batch-action";
import { useScores } from "../../contexts/ScoresContext";
import { CATEGORY_STYLES, getCategoryLabel } from "../../lib/constants";
import type { BloatwareItem } from "../../lib/types";

export function BloatwarePage() {
  const { refreshScores } = useScores();
  const listRefresh = useRef<(() => void) | null>(null);

  const remove = useToggleAction<BloatwareItem>({
    doCommand: "remove_bloatware",
    getId: (item) => item.packageName,
    buildArgs: (item) => ({ packageName: item.packageName }),
    confirmTitle: "Supprimer",
    confirmMessage: (item) => `Supprimer ${item.displayName} ?`,
    confirmVariant: "danger",
    successMessage: (item) => `${item.displayName} supprimé`,
    onSettled: () => {
      listRefresh.current?.();
      return refreshScores();
    },
  });

  return (
    <CrudListPage<BloatwareItem>
      title="Bloatware"
      fetchCommand="get_bloatware_items"
      refreshRef={listRefresh}
      getItemId={(item) => item.packageName}
      subtitle={<BloatwareSubtitle />}
      headerActions={(api) => (
        <RemoveAllButton
          installedCount={api.items.filter((i) => i.isInstalled).length}
          onSettled={() => {
            api.refresh();
            void refreshScores();
          }}
        />
      )}
      emptyState={{
        title: "Aucun bloatware détecté",
        hint: "Wintyx recherche 29 applications préinstallées Microsoft couramment non désirées. Aucune n'est présente sur ce système.",
      }}
      renderItem={(item) => {
        const recently = remove.recentId === item.packageName;
        return (
          <div
            data-recently-changed={recently ? "true" : undefined}
            className="bg-bg-card border border-border rounded-lg p-5 flex items-center justify-between theme-card transition-all duration-700 data-[recently-changed=true]:ring-2 data-[recently-changed=true]:ring-success/40"
          >
            <div className="flex-1 min-w-0 mr-4">
              <div className="flex items-center gap-3 mb-1 flex-wrap">
                <p className="text-sm font-medium text-fg-primary">{item.displayName}</p>
                <span
                  className={`inline-flex items-center px-3 py-1 rounded-full text-xs ${CATEGORY_STYLES[item.category.toLowerCase()] ?? "bg-bg-hover text-fg-dim"}`}
                >
                  {getCategoryLabel(item.category)}
                </span>
              </div>
              <p className="text-xs text-fg-muted">{item.publisher}</p>
              {item.version && (
                <p className="text-xs text-fg-dim">{item.packageName}, v{item.version}</p>
              )}
            </div>
            {!item.isInstalled ? (
              <Badge variant="success">Supprimé</Badge>
            ) : (
              <ActionButton
                variant="danger"
                loading={remove.busy === item.packageName}
                onClick={() => remove.trigger(item)}
                className="shrink-0"
              >
                Supprimer
              </ActionButton>
            )}
          </div>
        );
      }}
    />
  );
}

function BloatwareSubtitle() {
  return <span>Supprimez les applications préinstallées inutiles</span>;
}

function RemoveAllButton({
  installedCount,
  onSettled,
}: {
  installedCount: number;
  onSettled: () => void;
}) {
  const batch = useConfirmBatchAction({
    command: "remove_all_bloatware",
    confirmTitle: "Tout supprimer",
    confirmMessage: "Tous les bloatwares installés vont être supprimés.",
    confirmVariant: "danger",
    noun: { singular: "bloatware", plural: "bloatwares", participle: "supprimé" },
    snapshotActionName: "Suppression bloatware",
    enablePreview: true,
    onSettled,
  });

  return (
    <ActionButton
      variant="danger"
      loading={batch.isRunning || batch.isPreviewing}
      disabled={installedCount === 0}
      onClick={batch.trigger}
    >
      {batch.isRunning ? "Suppression..." : "Tout supprimer"}
    </ActionButton>
  );
}

export default BloatwarePage;
