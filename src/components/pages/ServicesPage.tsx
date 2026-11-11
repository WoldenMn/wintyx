import { useRef } from "react";
import { Badge } from "../../components/ui/Badge";
import { ActionButton } from "../../components/ui/ActionButton";
import { CrudListPage } from "../../components/ui/CrudListPage";
import { useToggleAction } from "../../hooks/use-toggle-action";
import { useScores } from "../../contexts/ScoresContext";
import { getScoreColor } from "../../lib/helpers";
import { riskRank } from "../../lib/constants";
import type { ServiceItem } from "../../lib/types";

function riskLabel(risk: string): string {
  switch (risk.toLowerCase()) {
    case "high": return "Risque élevé";
    case "medium": return "Risque moyen";
    case "low": return "Risque faible";
    default: return "Sans risque";
  }
}

export function ServicesPage() {
  const { scores, refreshScores } = useScores();
  const listRefresh = useRef<(() => void) | null>(null);
  const afterAction = () => {
    listRefresh.current?.();
    return refreshScores();
  };

  const stop = useToggleAction<ServiceItem>({
    doCommand: "stop_service",
    undoCommand: "start_service",
    getId: (s) => s.name,
    buildArgs: (s) => ({ name: s.name }),
    confirmTitle: "Arrêter le service",
    confirmMessage: (s) => `Arrêter ${s.displayName} ?`,
    confirmVariant: "warning",
    successMessage: (s) => `${s.displayName} arrêté`,
    undoMessage: (s) => `${s.displayName} redémarré`,
    onSettled: afterAction,
  });

  const start = useToggleAction<ServiceItem>({
    doCommand: "start_service",
    undoCommand: "stop_service",
    getId: (s) => s.name,
    buildArgs: (s) => ({ name: s.name }),
    confirmTitle: "Démarrer le service",
    confirmMessage: (s) => `Démarrer ${s.displayName} ?`,
    confirmVariant: "default",
    successMessage: (s) => `${s.displayName} démarré`,
    undoMessage: (s) => `${s.displayName} arrêté`,
    onSettled: afterAction,
  });

  return (
    <CrudListPage<ServiceItem>
      title="Services"
      subtitle={
        <>
          Score : <span className={`font-bold ${getScoreColor(scores.services)}`}>{scores.services}%</span>
        </>
      }
      fetchCommand="get_services"
      refreshRef={listRefresh}
      getItemId={(item) => item.name}
      searchPlaceholder="Rechercher un service..."
      filter={(s, q) =>
        s.displayName.toLowerCase().includes(q.toLowerCase()) ||
        s.name.toLowerCase().includes(q.toLowerCase())
      }
      groupBy={(s) => s.risk}
      sortGroups={(keys) =>
        keys.sort((a, b) => riskRank(a) - riskRank(b))
      }
      groupLabel={(risk, count) => `${riskLabel(risk)} (${count})`}
      emptyState={{ title: "Aucun service trouvé" }}
      renderItem={(item) => {
        const recently =
          stop.recentId === item.name || start.recentId === item.name;
        return (
          <div
            data-recently-changed={recently ? "true" : undefined}
            className="bg-bg-card border border-border rounded-lg p-5 flex items-center justify-between theme-card transition-all duration-700 data-[recently-changed=true]:ring-2 data-[recently-changed=true]:ring-success/40"
          >
            <div className="flex-1 mr-4 min-w-0">
              <div className="flex items-center gap-3 mb-1 flex-wrap">
                <p className="text-sm font-medium text-fg-primary">{item.displayName}</p>
                {item.status === "Running" ? (
                  <Badge variant="info">Actif</Badge>
                ) : item.status === "Absent" ? (
                  <Badge>Non installé</Badge>
                ) : item.status === "Unknown" ? (
                  <Badge>Inconnu</Badge>
                ) : (
                  <Badge>Arrêté</Badge>
                )}
                {item.status !== "Absent" && item.status !== "Unknown" && <Badge>{item.startType}</Badge>}
              </div>
              <p className="text-xs text-fg-dim">{item.description}</p>
              <p className="text-xs text-fg-muted mt-1">{item.name}</p>
            </div>
            {item.status === "Running" ? (
              <ActionButton
                requiresAdmin
                variant="danger"
                loading={stop.busy === item.name}
                onClick={() => stop.trigger(item)}
                className="shrink-0"
              >
                Arrêter
              </ActionButton>
            ) : item.status === "Absent" || item.status === "Unknown" ? null : (
              <ActionButton
                requiresAdmin
                variant="success"
                loading={start.busy === item.name}
                onClick={() => start.trigger(item)}
                className="shrink-0"
              >
                Démarrer
              </ActionButton>
            )}
          </div>
        );
      }}
    />
  );
}

export default ServicesPage;
