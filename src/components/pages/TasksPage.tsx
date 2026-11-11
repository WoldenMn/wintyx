import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../../hooks/use-invoke";
import { useToast } from "../../components/ui/Toast";
import { useConfirm } from "../../components/ui/ConfirmDialog";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageLoader } from "../../components/ui/PageLoader";
import { ErrorState } from "../../components/ui/ErrorState";
import { Badge } from "../../components/ui/Badge";
import { ActionButton } from "../../components/ui/ActionButton";
import { EmptyState } from "../../components/ui/EmptyState";
import type { TaskItem } from "../../lib/types";
import { describeError } from "../../lib/helpers";
import { notifySuspiciousTasksChanged } from "../../lib/suspicious-tasks";
import { getTaskStateLabel } from "../../lib/labels";
import { normalizeSearch } from "../../lib/search";

type TaskFilter = "suspicious" | "review" | "all";

export function TasksPage() {
  const { data: suspiciousTasks, isLoading: suspiciousLoading, error: suspiciousError, refresh: refreshSuspicious } =
    useInvoke<TaskItem[]>("get_suspicious_tasks");
  const { data: allTasks, isLoading: allLoading, error: allError, refresh: refreshAll } =
    useInvoke<TaskItem[]>("get_scheduled_tasks");
  const [filter, setFilter] = useState<TaskFilter>("suspicious");
  const [search, setSearch] = useState("");
  const [disabling, setDisabling] = useState<string | null>(null);
  const toast = useToast();
  const confirm = useConfirm();

  const isLoading = suspiciousLoading || allLoading;
  const error = suspiciousError || allError;
  if (isLoading) return <PageLoader />;
  if (error)
    return (
      <div className="p-7">
        <ErrorState
          message={error}
          onRetry={() => {
            refreshSuspicious();
            refreshAll();
          }}
        />
      </div>
    );

  async function handleDisable(task: TaskItem) {
    const fullPath = task.path.endsWith("\\") ? `${task.path}${task.name}` : `${task.path}\\${task.name}`;
    const ok = await confirm({ title: "Désactiver", message: `Désactiver la tâche ${task.name} ?`, variant: "warning" });
    if (!ok) return;
    setDisabling(fullPath);
    try {
      await invoke("disable_scheduled_task", { taskName: fullPath });
      toast.success(`${task.name} désactivé`);
      refreshSuspicious();
      refreshAll();
      notifySuspiciousTasksChanged();
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    }
    setDisabling(null);
  }

  const reviewTasks = (allTasks ?? []).filter((t) => t.needsReview);
  const source = filter === "all" ? (allTasks ?? []) : filter === "review" ? reviewTasks : (suspiciousTasks ?? []);
  const query = normalizeSearch(search.trim());
  const filtered = source.filter(
    (t) => normalizeSearch(t.name).includes(query) || normalizeSearch(t.path).includes(query),
  );
  const noMatch = query !== "" && source.length > 0 && filtered.length === 0;

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Tâches planifiées"
        subtitle={
          filter === "suspicious"
            ? "Tâches présentant un indice fort de détournement"
            : filter === "review"
              ? "Tâches d'auteur inconnu ou lançant un interpréteur : informatif, souvent créées par vos logiciels"
              : "Toutes les tâches planifiées"
        }
      />

      <div className="mb-4 flex flex-wrap gap-2 items-center">
        <div className="inline-flex rounded-lg border border-border bg-bg-card overflow-hidden shrink-0">
          <button
            type="button"
            onClick={() => setFilter("suspicious")}
            aria-pressed={filter === "suspicious"}
            className={`px-3 py-2 text-xs font-medium transition-colors ${filter === "suspicious" ? "bg-danger text-on-semantic" : "text-fg-dim hover:text-fg-primary"}`}
          >
            Suspectes ({suspiciousTasks?.length ?? 0})
          </button>
          <button
            type="button"
            onClick={() => setFilter("review")}
            aria-pressed={filter === "review"}
            className={`px-3 py-2 text-xs font-medium transition-colors ${filter === "review" ? "bg-warning text-on-semantic" : "text-fg-dim hover:text-fg-primary"}`}
          >
            À vérifier ({reviewTasks.length})
          </button>
          <button
            type="button"
            onClick={() => setFilter("all")}
            aria-pressed={filter === "all"}
            className={`px-3 py-2 text-xs font-medium transition-colors ${filter === "all" ? "bg-accent text-on-accent" : "text-fg-dim hover:text-fg-primary"}`}
          >
            Toutes ({allTasks?.length ?? 0})
          </button>
        </div>
        <input
          type="search"
          aria-label="Rechercher une tâche"
          placeholder="Rechercher une tâche..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="flex-1 min-w-[200px] px-4 py-2 bg-bg-card border border-border rounded-md text-sm text-fg-primary placeholder:text-fg-dim focus:outline-none focus:border-accent focus-visible:ring-2 focus-visible:ring-accent"
        />
      </div>

      {filtered.length > 0 ? (
        <div className="space-y-2">
          {filtered.map((task) => {
            const fullPath = task.path.endsWith("\\") ? `${task.path}${task.name}` : `${task.path}\\${task.name}`;
            return (
              <div key={fullPath} className="bg-bg-card border border-border rounded-lg p-5 theme-card">
                <div className="flex items-center justify-between mb-2 gap-3 flex-wrap min-w-0">
                  <div className="flex items-center gap-3 flex-wrap min-w-0">
                    <p className="text-sm font-medium text-fg-primary break-words min-w-0">{task.name}</p>
                    <Badge variant={task.isSuspicious ? "danger" : task.needsReview ? "warning" : "success"}>
                      {task.isSuspicious ? "Suspecte" : task.needsReview ? "À vérifier" : "OK"}
                    </Badge>
                    {task.state && <Badge>{getTaskStateLabel(task.state)}</Badge>}
                  </div>
                  <ActionButton
                    variant="warning"
                    loading={disabling === fullPath}
                    onClick={() => handleDisable(task)}
                  >
                    Désactiver
                  </ActionButton>
                </div>
                {task.suspiciousReason && (
                  <p className="text-xs text-fg-dim mb-1 break-words">{task.suspiciousReason}</p>
                )}
                <div className="flex items-center gap-4 flex-wrap">
                  <p className="text-xs text-fg-muted break-all">{task.path}</p>
                  {task.lastRunTime && <p className="text-xs text-fg-muted">Dernier : {task.lastRunTime}</p>}
                  {task.nextRunTime && <p className="text-xs text-fg-muted">Prochain : {task.nextRunTime}</p>}
                </div>
              </div>
            );
          })}
        </div>
      ) : (
        <EmptyState
          title={
            noMatch
              ? "Aucune tâche ne correspond à la recherche"
              : filter === "suspicious"
              ? "Aucune tâche suspecte trouvée"
              : filter === "review"
                ? "Aucune tâche à vérifier"
                : "Aucune tâche trouvée"
          }
          hint={
            noMatch
              ? "Modifiez ou effacez la recherche pour revoir la liste."
              : filter === "suspicious"
              ? "Wintyx signale comme suspectes les tâches qui lancent une commande encodée, mshta, certutil, bitsadmin, wscript ou une URL. Les tâches d'auteur inconnu sont listées dans « À vérifier »."
              : filter === "review"
                ? "Aucune tâche d'auteur inconnu ou lançant PowerShell ou cmd."
                : "Le planificateur de tâches semble vide ou inaccessible. Vérifier les droits d'accès au service 'Task Scheduler'."
          }
        />
      )}
    </div>
  );
}

export default TasksPage;
