import { useInvoke } from "../../hooks/use-invoke";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageLoader } from "../../components/ui/PageLoader";
import { ErrorState } from "../../components/ui/ErrorState";
import { Badge } from "../../components/ui/Badge";
import { EmptyState } from "../../components/ui/EmptyState";
import { UnavailableValue } from "../../components/ui/UnavailableValue";
import type { UpdateItem } from "../../lib/types";

export function UpdatesPage() {
  const { data: updates, isLoading: updatesLoading, error: updatesError, refresh: refreshUpdates } =
    useInvoke<UpdateItem[]>("get_update_history");
  const { data: build, isLoading: buildLoading, error: buildError, refresh: refreshBuild } = useInvoke<string>("get_windows_build");
  const { data: lastUpdate, isLoading: lastUpdateLoading, error: lastUpdateError, refresh: refreshLastUpdate } = useInvoke<string>("get_last_update_date");

  const isLoading = updatesLoading || buildLoading || lastUpdateLoading;

  if (isLoading) return <PageLoader />;
  if (updatesError)
    return (
      <div className="p-7">
        <ErrorState message={updatesError} onRetry={refreshUpdates} />
      </div>
    );

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader title="Mises à jour" subtitle="Historique des mises à jour Windows" />

      <div className="grid grid-cols-2 gap-4 mb-6">
        <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
          <p className="text-xs text-fg-muted mb-1">Build Windows</p>
          {buildError ? (
            <UnavailableValue message={buildError} onRetry={refreshBuild} retryAriaLabel="Réessayer la version de Windows" />
          ) : (
            <p className="text-2xl font-bold text-accent">{build}</p>
          )}
        </div>
        <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
          <p className="text-xs text-fg-muted mb-1">Dernière mise à jour</p>
          {lastUpdateError ? (
            <UnavailableValue message={lastUpdateError} onRetry={refreshLastUpdate} retryAriaLabel="Réessayer la dernière mise à jour" />
          ) : (
            <p className="text-2xl font-bold text-fg-primary">{lastUpdate || "Inconnue"}</p>
          )}
        </div>
      </div>

      <div>
        <h2 className="text-lg font-medium text-fg-primary mb-3">Historique</h2>
        {updates && updates.length > 0 ? (
          <div className="space-y-2">
            {updates.map((update, i) => (
              <div key={`${update.kbArticleId || update.title}-${i}`} className="bg-bg-card border border-border rounded-lg p-5 theme-card">
                <div className="flex items-center justify-between mb-2 gap-3 flex-wrap">
                  <p className="text-sm font-medium text-fg-primary">{update.title}</p>
                  <Badge variant={update.isImportant ? "info" : "neutral"}>
                    {update.category}
                  </Badge>
                </div>
                <div className="flex items-center gap-4">
                  <p className="text-xs text-fg-dim">{update.installedOn}</p>
                  {update.kbArticleId && (
                    <p className="text-xs text-fg-muted">{update.kbArticleId}</p>
                  )}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <EmptyState
            title="Aucune mise à jour trouvée"
            hint="Aucun KB installé détecté dans l'historique récent. Vérifier Windows Update dans les paramètres Windows si le système n'est pas à jour."
          />
        )}
      </div>
    </div>
  );
}

export default UpdatesPage;
