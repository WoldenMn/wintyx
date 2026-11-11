import { Badge } from "../../components/ui/Badge";
import type {
  ChangeRequest,
  ChangeRequestType,
  KeyValueChange,
  SnapshotDiff,
} from "../../lib/types";

export type ChangeFilter = "all" | "modified" | "removed" | "added";

export function changeKey(
  keyPath: string,
  valueName: string,
  ct: ChangeRequestType,
): string {
  return `${keyPath}|${valueName}|${ct.type}`;
}

export function changeRequestFromKvc(
  keyPath: string,
  c: KeyValueChange,
): ChangeRequest {
  switch (c.type) {
    case "added":
      return {
        keyPath,
        valueName: c.name,
        changeType: { type: "undoAdded" },
        targetValue: null,
      };
    case "removed":
      return {
        keyPath,
        valueName: c.name,
        changeType: { type: "restoreRemoved" },
        targetValue: c.snapshot,
      };
    case "modified":
      return {
        keyPath,
        valueName: c.name,
        changeType: { type: "restoreModified" },
        targetValue: c.snapshot,
      };
  }
}

export function formatTimestamp(ts: string): string {
  if (ts.length !== 15) return ts;
  const y = ts.slice(0, 4);
  const m = ts.slice(4, 6);
  const d = ts.slice(6, 8);
  const hh = ts.slice(9, 11);
  const mm = ts.slice(11, 13);
  const ss = ts.slice(13, 15);
  return `${d}/${m}/${y} ${hh}:${mm}:${ss}`;
}

interface SnapshotDiffViewProps {
  diff: SnapshotDiff;
  filter: ChangeFilter;
  onFilterChange: (f: ChangeFilter) => void;
  selectedChanges: Map<string, ChangeRequest>;
  onToggleChange: (req: ChangeRequest) => void;
}

export function SnapshotDiffView({
  diff,
  filter,
  onFilterChange,
  selectedChanges,
  onToggleChange,
}: SnapshotDiffViewProps) {
  const visibleKeys = diff.keys
    .map((k) => ({
      ...k,
      changes: k.changes.filter((c) => matchesFilter(c, filter)),
    }))
    .filter((k) => k.changes.length > 0);

  const visibleCount = visibleKeys.reduce((sum, k) => sum + k.changes.length, 0);
  const headerLabel = diff.comparisonTarget
    ? `Snapshot ${formatTimestamp(diff.snapshotTimestamp)} ▸ Snapshot ${formatTimestamp(diff.comparisonTarget)}`
    : `Snapshot ${formatTimestamp(diff.snapshotTimestamp)} ▸ état actuel`;

  return (
    <div>
      <div className="flex items-center gap-3 mb-4 flex-wrap">
        <span className="text-sm text-fg-dim">
          {headerLabel} : {diff.totalChanges} modification(s)
        </span>
        <div className="flex gap-2 ml-auto">
          <FilterButton current={filter} value="all" onChange={onFilterChange}>
            Tous ({diff.totalChanges})
          </FilterButton>
          <FilterButton current={filter} value="modified" onChange={onFilterChange}>
            Modifiés
          </FilterButton>
          <FilterButton current={filter} value="removed" onChange={onFilterChange}>
            Supprimés
          </FilterButton>
          <FilterButton current={filter} value="added" onChange={onFilterChange}>
            Ajoutés
          </FilterButton>
        </div>
      </div>

      {visibleCount === 0 ? (
        <p className="text-sm text-fg-dim">Aucun changement correspondant au filtre.</p>
      ) : (
        <div className="space-y-4">
          {visibleKeys.map((k) => (
            <div key={k.keyPath} className="bg-bg-card border border-border rounded-lg p-4 theme-card">
              <h3 className="text-xs font-medium text-fg-muted uppercase tracking-wider mb-2 break-all">
                {k.keyPath}
              </h3>
              <table className="w-full text-sm">
                <caption className="sr-only">
                  Changements détectés sur {k.keyPath} avec type, nom, valeur snapshot et valeur actuelle
                </caption>
                <thead>
                  <tr className="text-left text-xs text-fg-muted">
                    <th className="px-2 py-1 font-normal w-8" scope="col">
                      <span className="sr-only">Sélection</span>
                    </th>
                    <th className="px-2 py-1 font-normal" scope="col">Type</th>
                    <th className="px-2 py-1 font-normal" scope="col">Nom</th>
                    <th className="px-2 py-1 font-normal" scope="col">Snapshot</th>
                    <th className="px-2 py-1 font-normal" scope="col">Actuel</th>
                  </tr>
                </thead>
                <tbody>
                  {k.changes.map((c) => {
                    const req = changeRequestFromKvc(k.keyPath, c);
                    const id = changeKey(req.keyPath, req.valueName, req.changeType);
                    const isChecked = selectedChanges.has(id);
                    return (
                      <ChangeRow
                        key={id}
                        change={c}
                        checked={isChecked}
                        onToggle={() => onToggleChange(req)}
                      />
                    );
                  })}
                </tbody>
              </table>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

interface ChangeRowProps {
  change: KeyValueChange;
  checked: boolean;
  onToggle: () => void;
}

function ChangeRow({ change, checked, onToggle }: ChangeRowProps) {
  const checkboxCell = (
    <td className="px-2 py-1.5 align-middle">
      <input
        type="checkbox"
        checked={checked}
        onChange={onToggle}
        aria-label={`Sélectionner ${change.name || "(par défaut)"}`}
      />
    </td>
  );
  switch (change.type) {
    case "added":
      return (
        <tr className="border-t border-border/50">
          {checkboxCell}
          <td className="px-2 py-1.5"><Badge variant="success">Ajouté</Badge></td>
          <td className="px-2 py-1.5 font-mono text-xs">{change.name || "(par défaut)"}</td>
          <td className="px-2 py-1.5 text-fg-dim">—</td>
          <td className="px-2 py-1.5 font-mono text-xs break-all">{change.current}</td>
        </tr>
      );
    case "removed":
      return (
        <tr className="border-t border-border/50">
          {checkboxCell}
          <td className="px-2 py-1.5"><Badge variant="danger">Supprimé</Badge></td>
          <td className="px-2 py-1.5 font-mono text-xs">{change.name || "(par défaut)"}</td>
          <td className="px-2 py-1.5 font-mono text-xs break-all">{change.snapshot}</td>
          <td className="px-2 py-1.5 text-fg-dim">—</td>
        </tr>
      );
    case "modified":
      return (
        <tr className="border-t border-border/50">
          {checkboxCell}
          <td className="px-2 py-1.5"><Badge variant="warning">Modifié</Badge></td>
          <td className="px-2 py-1.5 font-mono text-xs">{change.name || "(par défaut)"}</td>
          <td className="px-2 py-1.5 font-mono text-xs break-all">{change.snapshot}</td>
          <td className="px-2 py-1.5 font-mono text-xs break-all">{change.current}</td>
        </tr>
      );
  }
}

interface FilterButtonProps {
  current: ChangeFilter;
  value: ChangeFilter;
  onChange: (v: ChangeFilter) => void;
  children: React.ReactNode;
}

function FilterButton({ current, value, onChange, children }: FilterButtonProps) {
  const isActive = current === value;
  return (
    <button
      type="button"
      onClick={() => onChange(value)}
      aria-pressed={isActive}
      className={`px-3 py-1 text-xs rounded-md transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent ${
        isActive
          ? "bg-accent text-on-accent"
          : "bg-bg-elevated border border-border text-fg-dim hover:text-fg-primary"
      }`}
    >
      {children}
    </button>
  );
}

function matchesFilter(change: KeyValueChange, filter: ChangeFilter): boolean {
  if (filter === "all") return true;
  if (filter === "modified") return change.type === "modified";
  if (filter === "removed") return change.type === "removed";
  if (filter === "added") return change.type === "added";
  return false;
}
