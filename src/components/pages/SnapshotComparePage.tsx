import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../../hooks/use-invoke";
import { useToast } from "../../components/ui/Toast";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageSkeleton } from "../../components/ui/Skeleton";
import { ErrorState } from "../../components/ui/ErrorState";
import { ActionButton } from "../../components/ui/ActionButton";
import { EmptyState } from "../../components/ui/EmptyState";
import { SnapshotDualSelector } from "../../components/ui/SnapshotDualSelector";
import { describeError } from "../../lib/helpers";
import type {
  SnapshotGroup,
  SnapshotDiff,
  ChangeRequest,
} from "../../lib/types";
import {
  SnapshotDiffView,
  changeKey,
  formatTimestamp,
  type ChangeFilter,
} from "./SnapshotDiffView";
import { SnapshotRestoreFooter } from "./SnapshotRestoreFooter";

type CompareMode = "vs-live" | "vs-snapshot";

export function SnapshotComparePage() {
  const { data: groups, isLoading, error, refresh } = useInvoke<SnapshotGroup[]>("list_snapshot_groups");
  const [mode, setMode] = useState<CompareMode>("vs-live");
  const [selectedTimestamp, setSelectedTimestamp] = useState<string>("");
  const [snapshotA, setSnapshotA] = useState<string>("");
  const [snapshotB, setSnapshotB] = useState<string>("");
  const [diff, setDiff] = useState<SnapshotDiff | null>(null);
  const [isComparing, setIsComparing] = useState(false);
  const [filter, setFilter] = useState<ChangeFilter>("all");
  const [selectedChanges, setSelectedChanges] = useState<Map<string, ChangeRequest>>(new Map());
  const toast = useToast();

  if (isLoading) return <PageSkeleton kind="list" count={3} />;
  if (error) return <div className="p-7"><ErrorState message={error} onRetry={refresh} /></div>;

  async function handleCompare() {
    if (mode === "vs-live") {
      if (!selectedTimestamp) {
        toast.warning("Sélectionner un snapshot d'abord");
        return;
      }
    } else {
      if (!snapshotA || !snapshotB) {
        toast.warning("Sélectionner les deux snapshots");
        return;
      }
      if (snapshotA === snapshotB) {
        toast.warning("Les deux snapshots sont identiques");
        return;
      }
    }

    setIsComparing(true);
    setSelectedChanges(new Map());
    try {
      const result =
        mode === "vs-live"
          ? await invoke<SnapshotDiff>("diff_snapshot_vs_live", {
              snapshotTimestamp: selectedTimestamp,
            })
          : await invoke<SnapshotDiff>("diff_snapshot_vs_snapshot", {
              snapshotATimestamp: snapshotA,
              snapshotBTimestamp: snapshotB,
            });
      setDiff(result);
      if (result.totalChanges === 0) {
        toast.info("Aucune modification détectée");
      } else {
        toast.success(`${result.totalChanges} modification(s) détectée(s)`);
      }
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    }
    setIsComparing(false);
  }

  if (!groups || groups.length === 0) {
    return (
      <div className="p-7 overflow-y-auto h-full">
        <PageHeader
          title="Comparer snapshots"
          subtitle="Voir ce qui a changé depuis un snapshot."
        />
        <EmptyState
          title="Aucun snapshot disponible"
          hint="Les snapshots sont créés automatiquement avant chaque batch (apply hardening, optimize all, profil, etc.). Lancez une action pour créer votre premier snapshot."
        />
      </div>
    );
  }

  const totalChanges = diff?.totalChanges ?? 0;

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Comparer snapshots"
        subtitle="Comparez avec l'état actuel ou avec un autre snapshot, puis restaurez les changements voulus."
      />

      <div className="bg-bg-card border border-border rounded-lg p-5 theme-card mb-6">
        <div className="flex gap-2 mb-4">
          <ModeButton current={mode} value="vs-live" onChange={setMode}>
            Vs état actuel
          </ModeButton>
          <ModeButton current={mode} value="vs-snapshot" onChange={setMode}>
            Vs autre snapshot
          </ModeButton>
        </div>

        {mode === "vs-live" ? (
          <div className="flex items-end gap-3 flex-wrap">
            <div className="flex-1 min-w-[280px]">
              <label htmlFor="snapshot-select" className="block text-xs text-fg-muted mb-1">
                Snapshot
              </label>
              <select
                id="snapshot-select"
                value={selectedTimestamp}
                onChange={(e) => setSelectedTimestamp(e.target.value)}
                className="w-full px-3 py-2 bg-bg-input border border-border rounded-md text-sm text-fg-primary focus:outline-none focus-visible:ring-2 focus-visible:ring-accent"
              >
                <option value="">-- Choisir un snapshot --</option>
                {groups.map((g) => (
                  <option key={g.timestamp} value={g.timestamp}>
                    {formatTimestamp(g.timestamp)} ({g.keyCount} clés)
                  </option>
                ))}
              </select>
            </div>
            <ActionButton variant="accent" loading={isComparing} onClick={handleCompare}>
              Comparer avec état actuel
            </ActionButton>
          </div>
        ) : (
          <div className="space-y-3">
            <SnapshotDualSelector
              groups={groups}
              valueA={snapshotA}
              valueB={snapshotB}
              onChangeA={setSnapshotA}
              onChangeB={setSnapshotB}
            />
            <ActionButton variant="accent" loading={isComparing} onClick={handleCompare}>
              Comparer les deux snapshots
            </ActionButton>
          </div>
        )}
      </div>

      {diff && (
        <SnapshotDiffView
          diff={diff}
          filter={filter}
          onFilterChange={setFilter}
          selectedChanges={selectedChanges}
          onToggleChange={(req) => {
            const k = changeKey(req.keyPath, req.valueName, req.changeType);
            setSelectedChanges((prev) => {
              const next = new Map(prev);
              if (next.has(k)) next.delete(k);
              else next.set(k, req);
              return next;
            });
          }}
        />
      )}

      <SnapshotRestoreFooter
        diff={diff}
        selectedChanges={selectedChanges}
        totalChanges={totalChanges}
        onClearAll={() => setSelectedChanges(new Map())}
        onSettled={async () => {
          setSelectedChanges(new Map());
          await handleCompare();
        }}
      />
    </div>
  );
}

interface ModeButtonProps {
  current: CompareMode;
  value: CompareMode;
  onChange: (v: CompareMode) => void;
  children: React.ReactNode;
}

function ModeButton({ current, value, onChange, children }: ModeButtonProps) {
  const isActive = current === value;
  return (
    <button
      type="button"
      onClick={() => onChange(value)}
      aria-pressed={isActive}
      className={`px-4 py-1.5 text-sm rounded-md transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-accent ${
        isActive
          ? "bg-accent text-on-accent"
          : "bg-bg-elevated border border-border text-fg-dim hover:text-fg-primary"
      }`}
    >
      {children}
    </button>
  );
}

export default SnapshotComparePage;
