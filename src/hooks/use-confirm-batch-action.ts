import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../components/ui/Toast";
import { useConfirm } from "../components/ui/ConfirmDialog";
import type { BatchResult, SnapshotInfo } from "../lib/types";
import type { CommandName } from "../lib/command-names";
import { previewSentence, none, type BatchNoun } from "../lib/batch-text";
import { summarizeBatch, describeError, combineBatchResults, type BatchLevel } from "../lib/helpers";

interface Options {
  command: CommandName | readonly CommandName[];
  confirmTitle: string;
  confirmMessage: string;
  confirmVariant?: "danger" | "warning" | "default";
  noun: BatchNoun;
  args?: Record<string, unknown>;
  /** Appele apres toute reponse (complete, partielle ou en echec), pour rafraichir. */
  onSettled?: (result: BatchResult) => void;
  skipSnapshot?: boolean;
  snapshotActionName?: string;
  enablePreview?: boolean;
}

interface State {
  trigger: () => Promise<void>;
  isRunning: boolean;
  isPreviewing: boolean;
}

const MAX_PREVIEW_LINES = 8;

function formatPreviewMessage(
  baseMessage: string,
  preview: BatchResult,
  noun: BatchNoun,
): string {
  if (preview.total === 0) {
    return `${baseMessage}\n\n${none(noun)} à modifier actuellement.`;
  }
  const items = (preview.previewItems ?? []).slice(0, MAX_PREVIEW_LINES);
  const hidden = (preview.previewItems?.length ?? 0) - items.length;
  const header = `${baseMessage}\n\n${previewSentence(preview.total, noun)} :`;
  const list = items.map((i) => `- ${i}`).join("\n");
  const suffix = hidden > 0 ? `\n... et ${hidden} autres` : "";
  return `${header}\n${list}${suffix}`;
}

export function useConfirmBatchAction(options: Options): State {
  const {
    command,
    confirmTitle,
    confirmMessage,
    confirmVariant,
    noun,
    args,
    onSettled,
    skipSnapshot,
    snapshotActionName,
    enablePreview,
  } = options;
  const confirm = useConfirm();
  const { success, warning, error, info } = useToast();
  const [isRunning, setIsRunning] = useState(false);
  const [isPreviewing, setIsPreviewing] = useState(false);

  const trigger = useCallback(async () => {
    const baseArgs = args ?? {};
    const commands = typeof command === "string" ? [command] : command;
    const runAll = async (commandArgs: Record<string, unknown>) => {
      const results: BatchResult[] = [];
      let failure: { command: string; detail: string } | null = null;
      for (const cmd of commands) {
        try {
          results.push(await invoke<BatchResult>(cmd, commandArgs));
        } catch (e) {
          failure = { command: cmd, detail: describeError(e) };
          break;
        }
      }
      return { result: combineBatchResults(results), failure, completed: results.length };
    };

    let message = confirmMessage;
    if (enablePreview) {
      setIsPreviewing(true);
      try {
        const { result: preview, failure } = await runAll({ ...baseArgs, options: { dryRun: true } });
        if (failure) throw new Error(failure.detail);
        message = formatPreviewMessage(confirmMessage, preview, noun);
      } catch (e) {
        // Preview non-bloquant : on confirme sans le detail, mais on signale que
        // l'apercu est indisponible (fvol-17) au lieu de l'avaler silencieusement.
        console.warn("Apercu (dry-run) indisponible:", e);
        warning("Aperçu indisponible : confirmation sans le détail des éléments.");
      } finally {
        setIsPreviewing(false);
      }
    }

    const ok = await confirm({
      title: confirmTitle,
      message,
      variant: confirmVariant ?? "default",
    });
    if (!ok) return;

    setIsRunning(true);
    try {
      if (!skipSnapshot) {
        let snapshotFailure: string | null = null;
        try {
          const snap = await invoke<SnapshotInfo>("create_pre_batch_snapshot", {
            actionName: snapshotActionName ?? confirmTitle,
          });
          if (snap.restorePointCreated) {
            info("Point de restauration créé");
          } else if (snap.restorePointLimited) {
            warning("Point de restauration non créé : limite Windows de 24 h, un point récent existe déjà.");
          }
          const backups = snap.registryBackups;
          if (backups.total > 0 && backups.success === 0) {
            snapshotFailure = "aucune clé du registre n'a pu être exportée";
          } else if (backups.failed > 0) {
            warning(
              `Sauvegarde partielle du registre : ${backups.failed} clé(s) sur ${backups.total} non exportée(s).`,
            );
          }
        } catch (e) {
          snapshotFailure = describeError(e);
        }
        if (snapshotFailure) {
          const goOn = await confirm({
            title: "Sauvegarde du registre impossible",
            message: `La sauvegarde préalable a échoué (${snapshotFailure}). Continuer sans pouvoir revenir en arrière ?`,
            variant: "danger",
          });
          if (!goOn) {
            warning("Action annulée : aucune modification appliquée.");
            return;
          }
        }
      }

      const { result, failure, completed } = await runAll(baseArgs);
      if (failure && completed === 0) {
        error(`Erreur : ${failure.detail}`);
        return;
      }
      const summary = summarizeBatch(result, noun);
      if (failure) {
        error(`${summary.message}. ${failure.command} a échoué : ${failure.detail}`);
      } else {
        emit(summary.level, summary.message, { success, warning, error, info });
      }
      onSettled?.(result);
    } catch (e) {
      error(`Erreur : ${describeError(e)}`);
    } finally {
      setIsRunning(false);
    }
  }, [
    command,
    confirmTitle,
    confirmMessage,
    confirmVariant,
    noun,
    args,
    onSettled,
    skipSnapshot,
    snapshotActionName,
    enablePreview,
    confirm,
    success,
    warning,
    error,
    info,
  ]);

  return { trigger, isRunning, isPreviewing };
}

function emit(
  level: BatchLevel,
  message: string,
  toasts: {
    success: (m: string) => void;
    warning: (m: string) => void;
    error: (m: string) => void;
    info: (m: string) => void;
  },
) {
  if (level === "success") toasts.success(message);
  else if (level === "warning") toasts.warning(message);
  else if (level === "info") toasts.info(message);
  else toasts.error(message);
}
