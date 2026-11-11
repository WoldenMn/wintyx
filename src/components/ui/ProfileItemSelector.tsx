import { getCategoryLabel } from "../../lib/constants";
import type { AvailableItem } from "../../lib/types";

interface Props {
  title: string;
  items: AvailableItem[];
  selectedIds: Set<string>;
  onToggle: (id: string) => void;
}

export function ProfileItemSelector({ title, items, selectedIds, onToggle }: Props) {
  const selectedCount = items.filter((i) => selectedIds.has(i.id)).length;

  return (
    <details className="bg-bg-card border border-border rounded-lg theme-card mb-4" open>
      <summary className="cursor-pointer px-5 py-3 flex items-center justify-between">
        <span className="text-sm font-medium text-fg-primary">{title}</span>
        <span className="text-xs text-fg-dim">{selectedCount} / {items.length}</span>
      </summary>
      <div className="px-5 pb-3 pt-1 space-y-1">
        {items.map((item) => (
          <label
            key={item.id}
            className="flex items-start gap-2 py-1 cursor-pointer hover:bg-bg-elevated rounded px-2"
          >
            <input
              type="checkbox"
              checked={selectedIds.has(item.id)}
              onChange={() => onToggle(item.id)}
              className="mt-1 focus-visible:ring-2 focus-visible:ring-accent rounded"
            />
            <div className="flex-1 min-w-0">
              <div className="text-sm text-fg-primary">{item.label}</div>
              <div className="text-xs text-fg-muted">{getCategoryLabel(item.category)}</div>
            </div>
          </label>
        ))}
      </div>
    </details>
  );
}
