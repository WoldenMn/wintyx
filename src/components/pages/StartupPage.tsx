import { useRef } from "react";
import { Badge } from "../../components/ui/Badge";
import { ActionButton } from "../../components/ui/ActionButton";
import { CrudListPage } from "../../components/ui/CrudListPage";
import { useToggleAction } from "../../hooks/use-toggle-action";
import type { StartupItem } from "../../lib/types";

function startupId(item: StartupItem): string {
  return `${item.location}-${item.name}`;
}

export function StartupPage() {
  const listRefresh = useRef<(() => void) | null>(null);
  const afterAction = () => listRefresh.current?.();

  const disable = useToggleAction<StartupItem>({
    doCommand: "disable_startup_item",
    undoCommand: "enable_startup_item",
    getId: startupId,
    buildArgs: (item) => ({ name: item.name, location: item.location }),
    confirmTitle: "Désactiver",
    confirmMessage: (item) => `Désactiver ${item.name} au démarrage ?`,
    confirmVariant: "warning",
    successMessage: (item) => `${item.name} désactivé`,
    undoMessage: (item) => `${item.name} réactivé`,
    onSettled: afterAction,
  });

  const enable = useToggleAction<StartupItem>({
    doCommand: "enable_startup_item",
    undoCommand: "disable_startup_item",
    getId: startupId,
    buildArgs: (item) => ({ name: item.name, location: item.location }),
    confirmTitle: "Réactiver",
    confirmMessage: (item) => `Réactiver ${item.name} au démarrage ?`,
    confirmVariant: "default",
    successMessage: (item) => `${item.name} réactivé`,
    undoMessage: (item) => `${item.name} désactivé`,
    onSettled: afterAction,
  });

  return (
    <CrudListPage<StartupItem>
      title="Démarrage"
      fetchCommand="get_startup_items"
      refreshRef={listRefresh}
      getItemId={startupId}
      subtitle="Programmes au démarrage de Windows"
      emptyState={{
        title: "Aucun programme au démarrage",
        hint: "Aucune application ne se lance au démarrage de Windows. Performance au boot optimale.",
      }}
      renderItem={(item) => {
        const id = startupId(item);
        const recently = disable.recentId === id || enable.recentId === id;
        return (
          <div
            data-recently-changed={recently ? "true" : undefined}
            className="bg-bg-card border border-border rounded-lg p-5 flex items-center justify-between theme-card transition-all duration-700 data-[recently-changed=true]:ring-2 data-[recently-changed=true]:ring-success/40"
          >
            <div className="flex-1 min-w-0 mr-4">
              <div className="flex items-center gap-3 mb-1 flex-wrap">
                <p className="text-sm font-medium text-fg-primary">{item.name}</p>
                <Badge>{item.locationDisplay}</Badge>
                <Badge variant={item.isEnabled ? "info" : "neutral"}>{item.isEnabled ? "Actif" : "Inactif"}</Badge>
              </div>
              <p className="text-xs text-fg-muted truncate">{item.command}</p>
            </div>
            {item.isEnabled ? (
              <ActionButton
                variant="warning"
                loading={disable.busy === id}
                onClick={() => disable.trigger(item)}
                className="shrink-0"
              >
                Désactiver
              </ActionButton>
            ) : (
              <ActionButton
                variant="success"
                loading={enable.busy === id}
                onClick={() => enable.trigger(item)}
                className="shrink-0"
              >
                Réactiver
              </ActionButton>
            )}
          </div>
        );
      }}
    />
  );
}

export default StartupPage;
