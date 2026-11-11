import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Badge } from "../../components/ui/Badge";
import { ActionButton } from "../../components/ui/ActionButton";
import { CrudListPage, type CrudApi } from "../../components/ui/CrudListPage";
import { useToggleAction } from "../../hooks/use-toggle-action";
import { useToast } from "../../components/ui/Toast";
import { useConfirm } from "../../components/ui/ConfirmDialog";
import { formatBytes, getSizeBadgeVariant, describeError } from "../../lib/helpers";
import { describeCleanAll, isCleanable } from "../../lib/cleanup";
import type { CleanupTarget, CleanResult } from "../../lib/types";

export function CleanupPage() {
  const cleanItem = useToggleAction<CleanupTarget>({
    doCommand: "clean_target",
    getId: (t) => t.id,
    confirmTitle: "Nettoyer",
    confirmMessage: (t) => `Nettoyer ${t.name} ?`,
    confirmVariant: "danger",
    successMessage: (t) => `${t.name} : nettoyage effectué`,
    evaluateResult: (result, t) => {
      const r = result as CleanResult | null;
      if (!r) return null;
      const freed = formatBytes(r.freedBytes);
      if (r.errors.length > 0) {
        return {
          level: "warning",
          message: `${t.name} : ${freed} libéré, ${r.errors.length} élément(s) non supprimé(s) (voir Logs)`,
        };
      }
      if (r.freedBytes === 0) {
        return { level: "success", message: `${t.name} : rien à libérer` };
      }
      return { level: "success", message: `${t.name} : ${freed} libéré` };
    },
  });

  return (
    <CrudListPage<CleanupTarget>
      title="Nettoyage"
      fetchCommand="get_cleanup_targets"
      getItemId={(t) => t.id}
      subtitleCompute={(items) => {
        const total = items.reduce((s, t) => s + t.sizeBytes, 0);
        const blocked = items.filter((t) => !t.accessible).length;
        return (
          <>
            {items.length} cibles détectées, {formatBytes(total)} récupérables
            {blocked > 0 ? ` (${blocked} inaccessible(s))` : ""}
          </>
        );
      }}
      emptyState={{
        title: "Aucune cible de nettoyage détectée",
        hint: "Les 6 dossiers scannés (Temp, Prefetch, cache Windows Update, corbeille, thumbnails) sont tous vides ou inaccessibles.",
      }}
      headerActions={(api) => <CleanAllButton api={api} />}
      renderItem={(item, api) => {
        const recently = cleanItem.recentId === item.id;
        return (
          <div
            data-recently-changed={recently ? "true" : undefined}
            className="bg-bg-card border border-border rounded-lg p-5 flex items-center justify-between theme-card transition-all duration-700 data-[recently-changed=true]:ring-2 data-[recently-changed=true]:ring-success/40"
          >
            <div className="flex-1 min-w-0 mr-4">
              <div className="flex items-center gap-3 mb-1 flex-wrap">
                <p className="text-sm font-medium text-fg-primary">{item.name}</p>
                <Badge variant={item.accessible ? getSizeBadgeVariant(item.sizeBytes) : "warning"}>
                  {item.sizeDisplay}
                </Badge>
              </div>
              <p className="text-xs text-fg-muted truncate">{item.description}</p>
            </div>
            <ActionButton
              variant="accent"
              loading={cleanItem.busy === item.id}
              requiresAdmin={item.isSystem}
              disabled={!item.accessible}
              onClick={() => cleanItem.trigger(item).then(() => api.refresh())}
              className="shrink-0"
            >
              Nettoyer
            </ActionButton>
          </div>
        );
      }}
    />
  );
}

function CleanAllButton({ api }: { api: CrudApi<CleanupTarget> }) {
  const toast = useToast();
  const confirm = useConfirm();
  const [running, setRunning] = useState(false);
  const cleanableCount = api.items.filter(isCleanable).length;

  async function handleCleanAll() {
    if (cleanableCount === 0) return;
    const ok = await confirm({
      title: "Tout nettoyer",
      message: describeCleanAll(api.items, formatBytes),
      variant: "danger",
    });
    if (!ok) return;
    setRunning(true);
    try {
      const result = await invoke<CleanResult>("clean_all");
      const freed = formatBytes(result.freedBytes);
      if (result.errors.length > 0) {
        toast.warning(`Nettoyage : ${freed} libéré, ${result.errors.length} erreur(s)`);
      } else {
        toast.success(`Nettoyage terminé : ${freed} libéré`);
      }
      api.refresh();
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    }
    setRunning(false);
  }

  return (
    <ActionButton
      variant="danger"
      loading={running}
      disabled={cleanableCount === 0}
      onClick={handleCleanAll}
    >
      {running ? "Nettoyage..." : "Tout nettoyer"}
    </ActionButton>
  );
}

export default CleanupPage;
