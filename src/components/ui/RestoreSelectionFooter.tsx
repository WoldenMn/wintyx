import { ActionButton } from "./ActionButton";

interface Props {
  count: number;
  total: number;
  onClearAll: () => void;
  onRestore: () => void;
  isRestoring: boolean;
}

export function RestoreSelectionFooter({ count, total, onClearAll, onRestore, isRestoring }: Props) {
  if (count === 0) return null;
  return (
    <div
      role="region"
      aria-label="Sélection de restauration"
      className="sticky bottom-0 z-10 -mx-7 -mb-7 mt-6 bg-bg-card border-t border-border px-7 py-3 shadow-lg"
    >
      <div className="flex items-center gap-3 flex-wrap">
        <span className="text-sm text-fg-primary">
          {count} changement(s) sélectionné(s) sur {total} total
        </span>
        <div className="flex gap-2 ml-auto">
          <ActionButton variant="neutral" onClick={onClearAll} disabled={isRestoring}>
            Tout désélectionner
          </ActionButton>
          <ActionButton
            variant="warning"
            onClick={onRestore}
            loading={isRestoring}
            disabled={isRestoring}
          >
            Restaurer la sélection ({count})
          </ActionButton>
        </div>
      </div>
    </div>
  );
}
