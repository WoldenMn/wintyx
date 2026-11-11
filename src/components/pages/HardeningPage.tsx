import { useInvoke } from "../../hooks/use-invoke";
import { useConfirmBatchAction } from "../../hooks/use-confirm-batch-action";
import { useToggleAction } from "../../hooks/use-toggle-action";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageSkeleton } from "../../components/ui/Skeleton";
import { ErrorState } from "../../components/ui/ErrorState";
import { Badge } from "../../components/ui/Badge";
import { ActionButton } from "../../components/ui/ActionButton";
import { InfoTooltip } from "../../components/ui/InfoTooltip";
import { useScores } from "../../contexts/ScoresContext";
import { getScoreColor, getRiskBadgeVariant } from "../../lib/helpers";
import { riskRank, getCategoryLabel, getHardeningLevelLabel } from "../../lib/constants";
import type { HardeningItem } from "../../lib/types";

const CORRECTION_NOUN = { singular: "correction", plural: "corrections", feminine: true };

export function HardeningPage() {
  const { data: items, isLoading, error, refresh } = useInvoke<HardeningItem[]>("get_hardening_checks");
  const { scores, refreshScores } = useScores();

  const pending = (items ?? []).filter((i) => !i.isSecure && !i.isUnknown && i.isFixable);
  const afterBatch = () => {
    refresh();
    void refreshScores();
  };
  const fix = useToggleAction<HardeningItem>({
    doCommand: "apply_hardening_fix",
    getId: (item) => item.id,
    buildArgs: (item) => ({ id: item.id }),
    confirmTitle: "Appliquer la correction",
    confirmMessage: (item) => `Appliquer "${item.name}" ? Cette modification affecte le registre.`,
    confirmVariant: "warning",
    successMessage: () => "Correction appliquée",
    onSettled: afterBatch,
  });
  const applyEssentials = useConfirmBatchAction({
    command: "apply_hardening_essential",
    confirmTitle: "Appliquer essentiels",
    confirmMessage: "Appliquer les corrections essentielles ?",
    confirmVariant: "warning",
    noun: CORRECTION_NOUN,
    enablePreview: true,
    onSettled: afterBatch,
  });
  const applyAll = useConfirmBatchAction({
    command: "apply_hardening_all",
    confirmTitle: "Appliquer tout",
    confirmMessage: `${pending.length} corrections vont être appliquées :\n${pending
      .map((i) => `- ${i.name} (${getHardeningLevelLabel(i.riskLevel)})`)
      .join("\n")}`,
    confirmVariant: "danger",
    noun: CORRECTION_NOUN,
    onSettled: afterBatch,
  });

  if (isLoading) return <PageSkeleton kind="list" count={8} />;
  if (error) return <div className="p-7"><ErrorState message={error} onRetry={refresh} /></div>;

  const grouped = (items ?? []).reduce<Record<string, HardeningItem[]>>((acc, item) => {
    const level = item.riskLevel;
    if (!acc[level]) acc[level] = [];
    acc[level].push(item);
    return acc;
  }, {});

  const sortedGroups = Object.entries(grouped).sort(
    ([a], [b]) => riskRank(a) - riskRank(b)
  );

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Durcissement"
        subtitle={
          <>Score : <span className={`font-bold ${getScoreColor(scores.hardening)}`}>{scores.hardening}%</span></>
        }
        actions={
          <>
            <ActionButton requiresAdmin loading={applyEssentials.isRunning} onClick={() => void applyEssentials.trigger()}>
              Appliquer essentiels
            </ActionButton>
            <ActionButton
              requiresAdmin
              variant="accent"
              loading={applyAll.isRunning}
              onClick={() => {
                if (pending.length > 0) void applyAll.trigger();
              }}
            >
              Appliquer tout
            </ActionButton>
          </>
        }
      />

      {sortedGroups.map(([riskLevel, levelItems]) => (
        <div key={riskLevel} className="mb-6">
          <h2 className="text-sm font-medium text-fg-muted uppercase tracking-wider mb-3">
            {getHardeningLevelLabel(riskLevel)}
          </h2>
          <div className="space-y-3">
            {levelItems.map((item) => (
              <div key={item.id} className="bg-bg-card border border-border rounded-lg p-5 flex items-center justify-between theme-card">
                <div className="flex-1 mr-4 min-w-0">
                  <div className="flex items-center gap-3 mb-1 flex-wrap">
                    <p className="text-sm font-medium text-fg-primary">{item.name}</p>
                    <InfoTooltip textId={item.id.toLowerCase()} ariaLabel={`Aide sur ${item.name}`} />
                    <Badge variant={getRiskBadgeVariant(item.riskLevel)}>
                      {getHardeningLevelLabel(item.riskLevel)}
                    </Badge>
                    <Badge>{getCategoryLabel(item.category)}</Badge>
                    {item.isSecure ? (
                      <Badge variant="success">Appliqué</Badge>
                    ) : item.isUnknown ? (
                      <Badge variant="warning">Inconnu</Badge>
                    ) : (
                      <Badge variant="danger">Non appliqué</Badge>
                    )}
                    {!item.isFixable && !item.isSecure && <Badge>Configuration manuelle</Badge>}
                  </div>
                  <p className="text-xs text-fg-dim">{item.description}</p>
                </div>
                {!item.isSecure && !item.isUnknown && item.isFixable && (
                  <ActionButton
                    requiresAdmin
                    variant="accent"
                    loading={fix.busy === item.id}
                    onClick={() => void fix.trigger(item)}
                    className="shrink-0"
                  >
                    Corriger
                  </ActionButton>
                )}
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

export default HardeningPage;
