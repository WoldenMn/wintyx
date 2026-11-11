import { useMemo } from "react";
import { useConfirmBatchAction } from "../../hooks/use-confirm-batch-action";
import { RestoreSelectionFooter } from "../../components/ui/RestoreSelectionFooter";
import type { ChangeRequest, SnapshotDiff } from "../../lib/types";

interface SnapshotRestoreFooterProps {
  diff: SnapshotDiff | null;
  selectedChanges: Map<string, ChangeRequest>;
  totalChanges: number;
  onClearAll: () => void;
  onSettled: () => Promise<void> | void;
}

export function SnapshotRestoreFooter({
  diff,
  selectedChanges,
  totalChanges,
  onClearAll,
  onSettled,
}: SnapshotRestoreFooterProps) {
  const args = useMemo(
    () => ({ changes: Array.from(selectedChanges.values()) }),
    [selectedChanges],
  );

  const restoreAction = useConfirmBatchAction({
    command: "restore_selected_changes",
    args,
    enablePreview: true,
    confirmTitle: "Restaurer la sélection ?",
    confirmMessage:
      "Les valeurs cochées vont être restaurées. Un snapshot sera créé automatiquement avant l'opération. Vérifiez la cohérence des modifications.",
    confirmVariant: "warning",
    noun: { singular: "changement", plural: "changements", participle: "restauré" },
    snapshotActionName: `restauration ${selectedChanges.size} changement(s)`,
    onSettled: () => {
      void onSettled();
    },
  });

  if (!diff) return null;

  return (
    <RestoreSelectionFooter
      count={selectedChanges.size}
      total={totalChanges}
      onClearAll={onClearAll}
      onRestore={restoreAction.trigger}
      isRestoring={restoreAction.isRunning || restoreAction.isPreviewing}
    />
  );
}
