import { useState } from "react";
import { useInvoke } from "../../hooks/use-invoke";
import { useToggleAction } from "../../hooks/use-toggle-action";
import { useConfirmBatchAction } from "../../hooks/use-confirm-batch-action";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageSkeleton } from "../../components/ui/Skeleton";
import { ErrorState } from "../../components/ui/ErrorState";
import { StatCard } from "../../components/ui/StatCard";
import { ActionButton } from "../../components/ui/ActionButton";
import { EmptyState } from "../../components/ui/EmptyState";
import { DriverIssueCard } from "./DriverIssueCard";
import type { DriverIssue, DriverAuditSummary } from "../../lib/types";
import { type AuditFilter, FILTER_LABELS } from "./DriverAuditHelpers";

export function DriversPage() {
  // get_driver_audit(_summary) renvoient Result<T, CommandError> cote Rust :
  // invoke() resout T directement (data) ou rejette (error). L'ancien typage
  // { status: "ok" | "error" } ne correspondait a rien -> summary/issues
  // toujours vides et erreur perdue (fvol-19).
  const { data: summary, isLoading: summaryLoading, error: summaryError, refresh: refreshSummary } =
    useInvoke<DriverAuditSummary>("get_driver_audit_summary");
  const { data: auditData, isLoading: auditLoading, error: auditError, refresh } =
    useInvoke<DriverIssue[]>("get_driver_audit");

  const [filter, setFilter] = useState<AuditFilter>("all");
  const [search, setSearch] = useState("");

  const issues: DriverIssue[] = auditData ?? [];
  const phantomCount = summary?.phantoms ?? 0;

  const refreshAll = () => {
    refreshSummary();
    refresh();
  };

  const uninstall = useToggleAction<DriverIssue>({
    doCommand: "uninstall_driver",
    getId: (issue) => issue.id,
    buildArgs: (issue) => ({ infName: issue.infName, options: null }),
    confirmTitle: "Désinstaller le pilote",
    confirmMessage: (issue) => `Désinstaller le pilote ${issue.infName} (${issue.deviceName}) ?`,
    confirmVariant: "danger",
    successMessage: (issue) => `Pilote ${issue.infName} désinstallé`,
    onSettled: refreshAll,
  });

  const removePhantom = useToggleAction<DriverIssue>({
    doCommand: "remove_phantom_device",
    getId: (issue) => issue.id,
    buildArgs: (issue) => ({ instanceId: issue.instanceId, options: null }),
    confirmTitle: "Supprimer le périphérique fantôme",
    confirmMessage: (issue) => `Supprimer le périphérique fantôme ${issue.deviceName} ?`,
    confirmVariant: "warning",
    successMessage: (issue) => `Périphérique fantôme ${issue.deviceName} supprimé`,
    onSettled: refreshAll,
  });

  const cleanPhantoms = useConfirmBatchAction({
    command: "remove_all_phantom_devices",
    confirmTitle: "Nettoyer les fantômes",
    confirmMessage: `Supprimer tous les ${phantomCount} périphériques fantômes détectés ?`,
    confirmVariant: "warning",
    noun: { singular: "fantôme", plural: "fantômes", participle: "supprimé" },
    onSettled: refreshAll,
    snapshotActionName: "Nettoyage fantômes",
    enablePreview: true,
  });

  if (summaryLoading || auditLoading) return <PageSkeleton kind="list" count={6} />;
  if (summaryError || auditError)
    return (
      <div className="p-7">
        <ErrorState
          message={summaryError || auditError || ""}
          onRetry={() => {
            refreshSummary();
            refresh();
          }}
        />
      </div>
    );

  const statCards: { filter: AuditFilter; label: string; value: number }[] = [
    { filter: "all", label: "Total", value: summary?.total ?? 0 },
    { filter: "unsigned", label: FILTER_LABELS.unsigned, value: summary?.unsigned ?? 0 },
    { filter: "duplicate", label: FILTER_LABELS.duplicate, value: summary?.duplicates ?? 0 },
    { filter: "phantom", label: FILTER_LABELS.phantom, value: summary?.phantoms ?? 0 },
    { filter: "obsolete", label: FILTER_LABELS.obsolete, value: summary?.obsolete ?? 0 },
    { filter: "thirdPartyOem", label: FILTER_LABELS.thirdPartyOem, value: summary?.thirdParty ?? 0 },
  ];

  const filtered = issues.filter((issue) => {
    if (filter !== "all" && issue.kind.kind !== filter) return false;
    if (!search) return true;
    const term = search.toLowerCase();
    return (
      issue.deviceName.toLowerCase().includes(term) ||
      issue.manufacturer.toLowerCase().includes(term) ||
      issue.driverVersion.toLowerCase().includes(term) ||
      issue.reason.toLowerCase().includes(term)
    );
  });

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Pilotes"
        subtitle="Audit des pilotes : doublons, fantômes, non signés et obsolètes"
        actions={
          phantomCount > 0 ? (
            <ActionButton
              variant="warning"
              requiresAdmin
              loading={cleanPhantoms.isRunning || cleanPhantoms.isPreviewing}
              onClick={cleanPhantoms.trigger}
            >
              Nettoyer fantômes ({phantomCount})
            </ActionButton>
          ) : undefined
        }
      />

      <div className="grid grid-cols-3 gap-3 mb-6">
        {statCards.map(({ filter: f, label, value }) => (
          <StatCard
            key={f}
            label={label}
            value={value}
            onClick={() => setFilter(f)}
            ariaLabel={`${label} : ${value}, cliquer pour filtrer`}
            className={filter === f ? "ring-2 ring-accent/60" : ""}
          />
        ))}
      </div>

      <div className="flex flex-wrap gap-2 mb-4 items-center">
        <div className="inline-flex rounded-lg border border-border bg-bg-card overflow-hidden shrink-0">
          {(Object.keys(FILTER_LABELS) as AuditFilter[]).map((f) => (
            <button
              key={f}
              type="button"
              aria-pressed={filter === f}
              onClick={() => setFilter(f)}
              className={`px-3 py-2 text-xs font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent ${
                filter === f ? "bg-accent text-on-accent" : "text-fg-dim hover:text-fg-primary"
              }`}
            >
              {FILTER_LABELS[f]}
            </button>
          ))}
        </div>
        <input
          type="text"
          placeholder="Rechercher un pilote..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          aria-label="Rechercher dans les pilotes"
          className="flex-1 min-w-[200px] px-4 py-2.5 bg-bg-input border border-border rounded-lg text-sm text-fg-primary placeholder:text-fg-muted focus:outline-none focus:border-accent focus-visible:ring-2 focus-visible:ring-accent"
        />
      </div>

      {filtered.length === 0 ? (
        <EmptyState
          title={search || filter !== "all" ? "Aucun problème correspondant" : "Aucun problème détecté"}
          hint={
            search || filter !== "all"
              ? "Modifier la recherche ou changer de filtre."
              : "Wintyx n'a détecté aucun pilote problématique. Votre système de pilotes est en bon état."
          }
        />
      ) : (
        <div className="space-y-3">
          {filtered.map((issue) => (
            <DriverIssueCard
              key={issue.id}
              issue={issue}
              recentlyChanged={uninstall.recentId === issue.id || removePhantom.recentId === issue.id}
              uninstallBusy={uninstall.busy}
              removePhantomBusy={removePhantom.busy}
              onUninstall={uninstall.trigger}
              onRemovePhantom={removePhantom.trigger}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export default DriversPage;
