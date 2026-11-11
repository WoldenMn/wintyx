import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../../hooks/use-invoke";
import { useToast } from "../../components/ui/Toast";
import { useConfirm } from "../../components/ui/ConfirmDialog";
import { PageHeader } from "../../components/ui/PageHeader";
import { PageLoader } from "../../components/ui/PageLoader";
import { ErrorState } from "../../components/ui/ErrorState";
import { ActionButton } from "../../components/ui/ActionButton";
import { Badge } from "../../components/ui/Badge";
import { EmptyState } from "../../components/ui/EmptyState";
import { RestorePointsSection } from "./RestorePointsSection";
import { formatBytes, summarizeBatch, describeError } from "../../lib/helpers";
import type { BatchResult, RegistryBackup, RestorePointOutcome } from "../../lib/types";

export function BackupPage() {
  const { data: regBackups, isLoading, error: backupsError, refresh: refreshBackups } = useInvoke<RegistryBackup[]>("get_registry_backups");
  const { data: protectionEnabled, isLoading: protectionLoading, error: protectionError } = useInvoke<boolean>("is_system_protection_enabled");
  const { data: backupFolder, isLoading: folderLoading, error: folderError, refresh: refreshFolder } = useInvoke<string>("get_backup_folder");
  const [busy, setBusy] = useState<"restore" | "backup" | null>(null);
  const [restoreRefresh, setRestoreRefresh] = useState(0);
  const [restoringPath, setRestoringPath] = useState<string | null>(null);
  const toast = useToast();
  const confirm = useConfirm();

  const error = backupsError || folderError;

  if (isLoading || protectionLoading || folderLoading) return <PageLoader />;
  if (error)
    return (
      <div className="p-7">
        <ErrorState
          message={error}
          onRetry={() => {
            refreshBackups();
            refreshFolder();
          }}
        />
      </div>
    );

  async function handleOpenFolder() {
    if (!backupFolder) return;
    try {
      await invoke("open_backup_folder");
    } catch (e) {
      toast.error(`Impossible d'ouvrir: ${describeError(e)}`);
    }
  }

  async function handleCreateRestore() {
    setBusy("restore");
    try {
      const outcome = await invoke<RestorePointOutcome>("create_restore_point", {
        description: "Wintyx - Point manuel",
      });
      if (outcome === "created") {
        toast.success("Point de restauration créé");
        setRestoreRefresh((n) => n + 1);
      } else {
        toast.warning("Point de restauration non créé : limite Windows de 24 h, un point récent existe déjà.");
      }
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    }
    setBusy(null);
  }

  async function handleBackupCriticalKeys() {
    setBusy("backup");
    try {
      const result = await invoke<BatchResult>("backup_critical_keys");
      const { message, level } = summarizeBatch(result, { singular: "clé", plural: "clés", feminine: true, participle: "sauvegardé" });
      toast[level](message);
      refreshBackups();
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    }
    setBusy(null);
  }

  async function handleRestoreBackup(backup: RegistryBackup) {
    const ok = await confirm({
      title: "Restaurer",
      message: `Restaurer le backup "${backup.hiveName}" du ${backup.backupTime} ?`,
      variant: "danger",
    });
    if (!ok) return;
    setRestoringPath(backup.filePath);
    try {
      await invoke("restore_registry_backup", { backupPath: backup.filePath });
      toast.success("Sauvegarde du registre restaurée");
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    }
    setRestoringPath(null);
  }

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader title="Sauvegarde" subtitle="Points de restauration et configuration du registre" />

      <div className="grid grid-cols-2 gap-4 mb-6">
        <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
          <p className="text-xs text-fg-muted mb-1">Protection système Windows</p>
          <div className="flex items-center gap-2">
            <Badge variant={protectionError || !protectionEnabled ? "warning" : "info"}>
              {protectionError ? "Indéterminée" : protectionEnabled ? "Non désactivée" : "Désactivée par stratégie"}
            </Badge>
          </div>
          <p className="text-xs text-fg-dim mt-2" role={protectionError ? "alert" : undefined}>
            {protectionError
              ? `État non lisible : ${protectionError}`
              : protectionEnabled
                ? "Aucune stratégie ne la désactive. L'état de chaque disque se lit dans Paramètres > Système > Protection du système."
                : "Une stratégie de groupe désactive la protection système : Windows refusera les points de restauration."}
          </p>
        </div>
        <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
          <p className="text-xs text-fg-muted mb-1">Dossier de sauvegarde</p>
          <p className="text-xs text-fg-dim break-all font-mono mb-2">{backupFolder ?? "—"}</p>
          <ActionButton
            variant="neutral"
            onClick={handleOpenFolder}
            disabled={!backupFolder}
            className="px-3 py-1.5 text-xs"
          >
            Ouvrir
          </ActionButton>
        </div>
      </div>

      <div className="bg-bg-card border border-border rounded-lg p-5 mb-6 theme-card">
        <h2 className="text-lg font-medium text-fg-primary mb-4">Actions</h2>
        <div className="flex items-center gap-3 flex-wrap">
          <ActionButton
            requiresAdmin
            variant="accent"
            loading={busy === "restore"}
            onClick={handleCreateRestore}
            disabled={!protectionError && !protectionEnabled}
          >
            Créer un point de restauration
          </ActionButton>
          <ActionButton
            requiresAdmin
            variant="neutral"
            loading={busy === "backup"}
            onClick={handleBackupCriticalKeys}
          >
            Sauvegarder les clés critiques
          </ActionButton>
        </div>
      </div>

      <RestorePointsSection refreshKey={restoreRefresh} />

      <div>
        <h2 className="text-lg font-medium text-fg-primary mb-3">Sauvegardes du registre</h2>
        {regBackups && regBackups.length > 0 ? (
          <div className="space-y-2">
            {regBackups.map((backup) => (
              <div key={backup.filePath} className="bg-bg-card border border-border rounded-lg p-4 flex items-center justify-between theme-card gap-3">
                <div className="flex-1 min-w-0 mr-4">
                  <p className="text-sm font-medium text-fg-primary truncate">{backup.hiveName}</p>
                  <div className="flex items-center gap-3 mt-1">
                    <p className="text-xs text-fg-dim">{backup.backupTime}</p>
                    <p className="text-xs text-fg-muted">{formatBytes(backup.fileSizeBytes)}</p>
                  </div>
                </div>
                <ActionButton
                  requiresAdmin
                  variant="warning"
                  loading={restoringPath === backup.filePath}
                  onClick={() => handleRestoreBackup(backup)}
                  className="shrink-0"
                >
                  Restaurer
                </ActionButton>
              </div>
            ))}
          </div>
        ) : (
          <EmptyState
            title="Aucun backup de registre"
            hint="Les sauvegardes apparaissent après une action qui modifie le registre (Wintyx crée automatiquement un snapshot avant chaque action destructive batch)."
          />
        )}
      </div>
    </div>
  );
}

export default BackupPage;
