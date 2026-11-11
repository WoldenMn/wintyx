import { DEFAULT_PARTICIPLE, agree, labelFor, none, previewSentence, type BatchNoun } from "./batch-text";
import type { BatchResult, CommandError } from "./types";
import type { ScoreSnapshot } from "./bindings";
import type { BadgeVariant } from "../components/ui/Badge";

export function getScoreColor(score: number): string {
  if (score >= 80) return "text-success";
  if (score >= 50) return "text-warning";
  return "text-danger";
}

export function getRiskScoreColor(score: number): string {
  if (score >= 70) return "text-danger";
  if (score >= 40) return "text-warning";
  return "text-success";
}

export function getScoreBg(score: number): string {
  if (score >= 80) return "bg-success/10";
  if (score >= 50) return "bg-warning/10";
  return "bg-danger/10";
}

export function getUsageColor(value: number): string {
  if (value >= 90) return "text-danger";
  if (value >= 70) return "text-warning";
  return "text-success";
}

export function getUsageBarColor(value: number): string {
  if (value >= 90) return "bg-danger";
  if (value >= 70) return "bg-warning";
  return "bg-success";
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} Ko`;
  if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} Mo`;
  return `${(bytes / 1073741824).toFixed(2)} Go`;
}

export function formatUptime(seconds: number): string {
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (d > 0) return `${d}j ${h}h ${m}m`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export function groupBy<T>(items: T[], key: keyof T): Record<string, T[]> {
  return items.reduce<Record<string, T[]>>((acc, item) => {
    const group = String(item[key]);
    if (!acc[group]) acc[group] = [];
    acc[group].push(item);
    return acc;
  }, {});
}

export interface BatchSummary {
  total: number;
  success: number;
  failed: number;
  errors: string[];
  dryRun?: boolean;
  previewItems?: string[];
}

export type BatchLevel = "success" | "warning" | "error" | "info";

export function summarizeBatch(
  result: BatchSummary,
  noun: BatchNoun,
): { message: string; level: BatchLevel } {
  const participle = noun.participle ?? DEFAULT_PARTICIPLE;
  if (result.dryRun) {
    const level = "info";
    if (result.total === 0) return { message: `${none(noun)} à modifier`, level };
    return { message: previewSentence(result.total, noun), level };
  }
  if (result.total === 0) {
    return { message: `${none(noun)} à traiter`, level: "success" };
  }
  if (result.failed === 0) {
    const text = `${result.success} ${labelFor(result.success, noun)} ${agree(participle, result.success, noun.feminine)}`;
    return { message: text, level: "success" };
  }
  if (result.success === 0) {
    return { message: `${result.failed} ${labelFor(result.failed, noun)} en échec`, level: "error" };
  }
  return {
    message: `${result.success}/${result.total} ${labelFor(result.total, noun)} ${agree(participle, result.total, noun.feminine)}, ${result.failed} en échec`,
    level: "warning",
  };
}

export interface ActionVerdict {
  level: "success" | "warning" | "error";
  message: string;
}

export function evaluateBatchResult(result: unknown): ActionVerdict | null {
  if (typeof result !== "object" || result === null) return null;
  const { failed, success, errors } = result as Partial<BatchSummary>;
  if (typeof failed !== "number" || failed <= 0) return null;
  const detail =
    Array.isArray(errors) && errors.length > 0 ? errors.slice(0, 3).join(" ; ") : `${failed} échec(s)`;
  return {
    level: success === 0 ? "error" : "warning",
    message: success === 0 ? `Échec : ${detail}` : `Succès partiel : ${detail}`,
  };
}

export function combineBatchResults(results: BatchResult[]): BatchResult {
  return {
    total: results.reduce((n, r) => n + r.total, 0),
    success: results.reduce((n, r) => n + r.success, 0),
    failed: results.reduce((n, r) => n + r.failed, 0),
    errors: results.flatMap((r) => r.errors),
    dryRun: results.length > 0 && results.every((r) => r.dryRun),
    previewItems: results.flatMap((r) => r.previewItems),
  };
}

export function diffPercent(before: number, after: number): number {
  if (before === 0) return 0;
  return ((after - before) / before) * 100;
}

export function isImprovement(
  before: number,
  after: number,
  higherIsBetter: boolean,
): boolean {
  const d = diffPercent(before, after);
  return higherIsBetter ? d > 0 : d < 0;
}

/**
 * Mappe un niveau de risque vers une variante de Badge.
 * Centralise pour eviter les redefinitions dans HardeningPage/ServicesPage/TasksPage.
 * Accepte string pour tolerer les niveaux inattendus (fallback vers "success").
 */
export function getRiskBadgeVariant(level: string): BadgeVariant {
  const normalized = level.toLowerCase();
  if (normalized === "critical" || normalized === "high" || normalized === "essential") return "danger";
  if (normalized === "medium" || normalized === "recommended") return "warning";
  if (normalized === "low") return "info";
  return "success";
}

/**
 * Mappe une taille en octets vers une variante de Badge.
 * Centralise pour CleanupPage (sizeBadgeVariant inline avant v2.9.0).
 */
export function getSizeBadgeVariant(bytes: number): BadgeVariant {
  const mb = bytes / 1048576;
  if (mb > 100) return "danger";
  if (mb > 10) return "warning";
  return "neutral";
}

export function isCommandError(e: unknown): e is CommandError {
  return (
    typeof e === "object" &&
    e !== null &&
    "code" in e &&
    "details" in e &&
    typeof (e as { code: unknown }).code === "string"
  );
}

/**
 * Delta du score global entre le dernier snapshot et celui d'il y a ~`days` jours.
 * Choisit le snapshot le plus recent dont la date est <= (derniere date - days) ;
 * si aucun (historique plus court), retombe sur le plus ancien. null si < 2 points.
 * L'historique est suppose en ordre chronologique croissant.
 */
export function computeScoreDelta(history: ScoreSnapshot[], days: number): number | null {
  if (history.length < 2) return null;
  const latest = history[history.length - 1];
  const targetMs = Date.parse(latest.date) - days * 86_400_000;
  let chosen = history[0];
  for (const s of history) {
    if (Date.parse(s.date) <= targetMs) chosen = s;
    else break;
  }
  return latest.globalScore - chosen.globalScore;
}

export function describeError(e: unknown): string {
  if (isCommandError(e)) {
    switch (e.code) {
      case "NotAdmin":
        return `${e.details} nécessite des privilèges administrateur. Relancer Wintyx en mode admin.`;
      case "WmiUnavailable":
        return `Service WMI indisponible : ${e.details}. Vérifier que le service Winmgmt est démarré.`;
      case "Timeout":
        return `La commande '${e.details.command}' a dépassé ${e.details.timeoutMs} ms. Système peut-être surchargé.`;
      case "NotFound":
        return `Introuvable : ${e.details}`;
      case "PermissionDenied":
        return `Accès refusé : ${e.details}`;
      case "InvalidInput":
        return `Entrée invalide : ${e.details}`;
      case "Registry":
        return `Erreur registre : ${e.details}`;
      case "ProcessFailed":
        return `Échec de '${e.details.command}' : ${e.details.detail}`;
      case "Generic":
        return e.details;
    }
  }
  return typeof e === "string" ? e : String(e);
}
