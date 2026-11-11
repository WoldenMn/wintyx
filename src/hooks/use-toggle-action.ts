import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../components/ui/Toast";
import { useConfirm } from "../components/ui/ConfirmDialog";
import { describeError, evaluateBatchResult, type ActionVerdict } from "../lib/helpers";
import type { CommandName } from "../lib/command-names";

export interface ToggleActionOptions<T> {
  /** Commande Tauri a invoquer pour l'action principale. */
  doCommand: CommandName;
  /** Commande Tauri pour annuler l'action (bouton "Annuler" dans le toast). Si absent, pas d'undo. */
  undoCommand?: CommandName;
  /** Deriver un id stable depuis l'item (utilise pour tracker busy). */
  getId: (item: T) => string;
  /** Titre du dialog de confirmation. */
  confirmTitle: string;
  /** Message du dialog, calcule depuis l'item. */
  confirmMessage: (item: T) => string;
  /** Variante de confirmation. Defaut "warning". */
  confirmVariant?: "danger" | "warning" | "default";
  /** Message toast de succes, calcule depuis l'item. */
  successMessage: (item: T) => string;
  /** Message toast apres undo. Defaut : "<success> annule". */
  undoMessage?: (item: T) => string;
  /** Callback apres toute reponse de la commande, succes ou echec (ex: refresh, refreshScores). */
  onSettled?: (item: T) => void | Promise<void>;
  /** Args additionnels a passer a invoke (en plus de `id`). */
  extraArgs?: (item: T) => Record<string, unknown>;
  /**
   * Si fourni, remplace totalement les args passes a invoke (pas de `{ id }` auto).
   * Utilise quand le backend attend un nom de param different (ex: `{ name }` pour
   * `stop_service`, ou `{ packageName }` pour `remove_bloatware`).
   */
  buildArgs?: (item: T) => Record<string, unknown>;
  /**
   * Si fourni, inspecte le resultat de la commande pour decider du toast :
   * permet de signaler un succes PARTIEL (ex: CleanResult.errors non vide) au lieu
   * d'un faux "succes" (politique mode degrade v2.12.0). Retourne null = succes normal.
   * Sans verdict, un BatchResult avec `failed > 0` est traite comme un echec.
   */
  evaluateResult?: (result: unknown, item: T) => ActionVerdict | null;
}

export interface ToggleActionResult<T> {
  trigger: (item: T) => Promise<void>;
  busy: string | null;
  /** Id du dernier item modifie avec succes. Auto-clear apres 800ms. */
  recentId: string | null;
}

/**
 * Hook generique pour le pattern `confirm + invoke + toast + undo`.
 *
 * Elimine la duplication dans ServicesPage, StartupPage, BloatwarePage, HardeningPage,
 * PerformancePage, DataProtectionPage, TasksPage. Chaque page reimplementait le meme
 * squelette (setBusy, try/catch, toast success/error, undo optionnel).
 *
 * Note : pour un TypeScript sain, specifier `<T>` explicitement au site d'appel.
 */
export function useToggleAction<T>(options: ToggleActionOptions<T>): ToggleActionResult<T> {
  const confirm = useConfirm();
  const toast = useToast();
  const [busy, setBusy] = useState<string | null>(null);
  const [recentId, setRecentId] = useState<string | null>(null);
  const recentTimerRef = useRef<number | null>(null);
  const inFlightRef = useRef<Set<string>>(new Set());

  useEffect(() => {
    return () => {
      if (recentTimerRef.current !== null) {
        window.clearTimeout(recentTimerRef.current);
      }
    };
  }, []);

  const trigger = useCallback(
    async (item: T) => {
      const id = options.getId(item);
      if (inFlightRef.current.has(id)) return;
      inFlightRef.current.add(id);
      try {
        const ok = await confirm({
          title: options.confirmTitle,
          message: options.confirmMessage(item),
          variant: options.confirmVariant ?? "warning",
        });
        if (!ok) return;

        setBusy(id);
        try {
          const args = options.buildArgs
            ? options.buildArgs(item)
            : { id, ...(options.extraArgs?.(item) ?? {}) };
          const result = await invoke(options.doCommand, args);
          const verdict = options.evaluateResult?.(result, item) ?? evaluateBatchResult(result);

          if (verdict?.level === "error") {
            toast.error(verdict.message);
            await options.onSettled?.(item);
            return;
          }

          setRecentId(id);
          if (recentTimerRef.current !== null) {
            window.clearTimeout(recentTimerRef.current);
          }
          recentTimerRef.current = window.setTimeout(() => {
            setRecentId(null);
            recentTimerRef.current = null;
          }, 800);

          const successMsg = options.successMessage(item);
          if (verdict?.level === "warning") {
            toast.warning(verdict.message);
          } else if (options.undoCommand) {
            const undoCmd = options.undoCommand;
            toast.success(verdict?.message ?? successMsg, {
              action: {
                label: "Annuler",
                onClick: async () => {
                  try {
                    await invoke(undoCmd, args);
                    toast.info(options.undoMessage?.(item) ?? `${successMsg} : annulé`);
                  } catch (e) {
                    toast.error(`Annulation impossible : ${describeError(e)}`);
                  }
                  await options.onSettled?.(item);
                },
              },
            });
          } else {
            toast.success(verdict?.message ?? successMsg);
          }

          await options.onSettled?.(item);
        } catch (e) {
          toast.error(`Erreur : ${describeError(e)}`);
        } finally {
          setBusy(null);
        }
      } finally {
        inFlightRef.current.delete(id);
      }
    },
    [confirm, toast, options],
  );

  return { trigger, busy, recentId };
}
