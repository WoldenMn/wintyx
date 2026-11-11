import { useEffect, useRef } from "react";
import { useInvoke } from "../../hooks/use-invoke";
import { useAdminOptional } from "../../contexts/AdminContext";
import { EmptyState } from "../../components/ui/EmptyState";
import { ErrorState } from "../../components/ui/ErrorState";
import type { RestorePointInfo } from "../../lib/types";

function RestorePointsList({ refreshKey }: { refreshKey: number }) {
  const { data, isLoading, error, refresh } = useInvoke<RestorePointInfo[]>("get_restore_points");
  const lastKey = useRef(refreshKey);

  useEffect(() => {
    if (refreshKey !== lastKey.current) {
      lastKey.current = refreshKey;
      refresh();
    }
  }, [refreshKey, refresh]);

  if (isLoading) return <p className="text-sm text-fg-dim">Lecture des points de restauration...</p>;
  if (error) return <ErrorState message={error} onRetry={refresh} />;
  if (!data || data.length === 0) {
    return (
      <EmptyState
        title="Aucun point de restauration"
        hint="Créez un point avec le bouton ci-dessus avant une modification importante."
      />
    );
  }
  return (
    <div className="space-y-2">
      {data.map((point) => (
        <div
          key={point.sequenceNumber}
          className="bg-bg-card border border-border rounded-lg p-4 theme-card flex items-center justify-between gap-3"
        >
          <p className="text-sm font-medium text-fg-primary truncate">{point.description}</p>
          <p className="text-xs text-fg-dim shrink-0">{point.creationTime}</p>
        </div>
      ))}
    </div>
  );
}

export function RestorePointsSection({ refreshKey }: { refreshKey: number }) {
  const { isAdmin, loaded } = useAdminOptional();

  return (
    <div className="mb-6">
      <h2 className="text-lg font-medium text-fg-primary mb-3">Points de restauration</h2>
      {!loaded ? (
        <p className="text-sm text-fg-dim">Vérification des droits...</p>
      ) : !isAdmin ? (
        <p className="text-sm text-fg-dim bg-bg-card border border-border rounded-lg p-4 theme-card">
          Droits administrateur requis pour lister les points de restauration Windows.
        </p>
      ) : (
        <RestorePointsList refreshKey={refreshKey} />
      )}
    </div>
  );
}
