import type { SnapshotGroup } from "../../lib/types";

interface Props {
  groups: SnapshotGroup[];
  valueA: string;
  valueB: string;
  onChangeA: (timestamp: string) => void;
  onChangeB: (timestamp: string) => void;
}

function formatTimestamp(ts: string): string {
  if (ts.length !== 15) return ts;
  return `${ts.slice(6, 8)}/${ts.slice(4, 6)}/${ts.slice(0, 4)} ${ts.slice(9, 11)}:${ts.slice(11, 13)}:${ts.slice(13, 15)}`;
}

export function SnapshotDualSelector({ groups, valueA, valueB, onChangeA, onChangeB }: Props) {
  return (
    <div className="flex gap-3 flex-wrap">
      <div className="flex-1 min-w-[240px]">
        <label htmlFor="snapshot-a" className="block text-xs text-fg-muted mb-1">
          Snapshot A (référence)
        </label>
        <select
          id="snapshot-a"
          aria-label="Snapshot A"
          value={valueA}
          onChange={(e) => onChangeA(e.target.value)}
          className="w-full px-3 py-2 bg-bg-input border border-border rounded-md text-sm text-fg-primary focus:outline-none focus:border-accent"
        >
          <option value="">-- Choisir le snapshot de base --</option>
          {groups.map((g) => (
            <option key={g.timestamp} value={g.timestamp}>
              {formatTimestamp(g.timestamp)} ({g.keyCount} clés)
            </option>
          ))}
        </select>
      </div>
      <div className="flex-1 min-w-[240px]">
        <label htmlFor="snapshot-b" className="block text-xs text-fg-muted mb-1">
          Snapshot B (comparé)
        </label>
        <select
          id="snapshot-b"
          aria-label="Snapshot B"
          value={valueB}
          onChange={(e) => onChangeB(e.target.value)}
          className="w-full px-3 py-2 bg-bg-input border border-border rounded-md text-sm text-fg-primary focus:outline-none focus:border-accent"
        >
          <option value="">-- Choisir le snapshot à comparer --</option>
          {groups.map((g) => (
            <option key={g.timestamp} value={g.timestamp}>
              {formatTimestamp(g.timestamp)} ({g.keyCount} clés)
            </option>
          ))}
        </select>
      </div>
    </div>
  );
}
