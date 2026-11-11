import { useRef } from "react";
import { Badge } from "../../components/ui/Badge";
import { ActionButton } from "../../components/ui/ActionButton";
import { CrudListPage } from "../../components/ui/CrudListPage";
import { useToggleAction } from "../../hooks/use-toggle-action";
import { useConfirmBatchAction } from "../../hooks/use-confirm-batch-action";
import { useScores } from "../../contexts/ScoresContext";
import { getScoreColor } from "../../lib/helpers";
import { getCategoryLabel } from "../../lib/constants";
import type { DataProtectionItem } from "../../lib/types";

export function DataProtectionPage() {
  const { scores, refreshScores } = useScores();
  const listRefresh = useRef<(() => void) | null>(null);

  const protectAction = useToggleAction<DataProtectionItem>({
    doCommand: "protect_item",
    getId: (item) => item.id,
    confirmTitle: "Appliquer la protection",
    confirmMessage: (item) => `Activer "${item.name}" ?`,
    confirmVariant: "warning",
    successMessage: (item) => `${item.name} protégé`,
    onSettled: () => {
      listRefresh.current?.();
      return refreshScores();
    },
  });

  return (
    <CrudListPage<DataProtectionItem>
      title="Protection des données"
      fetchCommand="get_data_protection_items"
      refreshRef={listRefresh}
      getItemId={(item) => item.id}
      subtitle={
        <>Score : <span className={`font-bold ${getScoreColor(scores.dataProtection)}`}>{scores.dataProtection}%</span></>
      }
      groupBy={(item) => item.category}
      groupLabel={getCategoryLabel}
      headerActions={(api) => (
        <ProtectAllButton
          onSettled={() => {
            api.refresh();
            void refreshScores();
          }}
        />
      )}
      renderItem={(item) => {
        const recently = protectAction.recentId === item.id;
        return (
          <div
            data-recently-changed={recently ? "true" : undefined}
            className="bg-bg-card border border-border rounded-lg p-5 flex items-center justify-between theme-card transition-all duration-700 data-[recently-changed=true]:ring-2 data-[recently-changed=true]:ring-success/40"
          >
            <div className="flex-1 mr-4 min-w-0">
              <div className="flex items-center gap-3 mb-1 flex-wrap">
                <p className="text-sm font-medium text-fg-primary">{item.name}</p>
                {item.isProtected ? (
                  <Badge variant="success">Protégé</Badge>
                ) : item.recommended ? (
                  <Badge variant="danger">Non protégé</Badge>
                ) : (
                  <Badge>Facultatif</Badge>
                )}
              </div>
              <p className="text-xs text-fg-dim">{item.description}</p>
            </div>
            {!item.isProtected && (
              <ActionButton
                variant="accent"
                loading={protectAction.busy === item.id}
                onClick={() => protectAction.trigger(item)}
                className="shrink-0"
              >
                Protéger
              </ActionButton>
            )}
          </div>
        );
      }}
    />
  );
}

function ProtectAllButton({ onSettled }: { onSettled: () => void }) {
  const batch = useConfirmBatchAction({
    command: "protect_all",
    confirmTitle: "Tout protéger",
    confirmMessage: "Toutes les protections disponibles vont être appliquées.",
    confirmVariant: "warning",
    noun: { singular: "protection", plural: "protections", feminine: true },
    snapshotActionName: "Protections données",
    enablePreview: true,
    onSettled,
  });

  return (
    <ActionButton variant="accent" requiresAdmin loading={batch.isRunning || batch.isPreviewing} onClick={batch.trigger}>
      Tout protéger
    </ActionButton>
  );
}

export default DataProtectionPage;
