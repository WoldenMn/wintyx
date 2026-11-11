// Couche de normalisation des donnees par domaine de score (logique pure,
// testable sans React). Separee du composant modal ScoreBreakdown.tsx.

import type {
  BloatwareItem,
  DataProtectionItem,
  HardeningItem,
  PerformanceItem,
  ServiceItem,
} from "./types";
import { getCategoryLabel, getHardeningLevelLabel } from "./constants";
import { SCORE_DOMAINS, type ScoreDomainId } from "./scoreDomains";

const SERVICE_STATUS_LABELS: Record<string, string> = {
  Running: "Actif",
  Stopped: "Arrêté",
  Absent: "Non installé",
  Unknown: "Inconnu",
};

export type ScoreDomain = ScoreDomainId;

export interface BreakdownItem {
  id: string;
  name: string;
  category: string;
  isOk: boolean;
  statusLabel: string;
  description?: string;
  weightLabel?: string;
}

type Normalizer = (data: unknown) => BreakdownItem[];

function describeRisk(risk: string): string {
  switch (risk.toLowerCase()) {
    case "high":
      return "Risque élevé";
    case "medium":
      return "Risque moyen";
    case "low":
      return "Risque faible";
    default:
      return risk;
  }
}

const NORMALIZERS: Record<ScoreDomain, Normalizer> = {
  dataProtection: (data) =>
    (data as DataProtectionItem[]).filter((i) => i.recommended).map((i) => ({
      id: i.id,
      name: i.name,
      category: getCategoryLabel(i.category),
      isOk: i.isProtected,
      statusLabel: i.isProtected ? "Protégé" : "Non protégé",
      description: i.description,
    })),
  services: (data) =>
    (data as ServiceItem[]).map((i) => ({
      id: i.name,
      name: i.displayName,
      category: describeRisk(i.risk),
      isOk: i.status === "Stopped" || i.status === "Absent",
      statusLabel: SERVICE_STATUS_LABELS[i.status] ?? "Inconnu",
      description: i.description,
      weightLabel: i.startType,
    })),
  hardening: (data) =>
    (data as HardeningItem[]).map((i) => ({
      id: i.id,
      name: i.name,
      category: getCategoryLabel(i.category),
      isOk: i.isSecure,
      statusLabel: i.isSecure ? "Appliqué" : i.isUnknown ? "Inconnu" : "Non appliqué",
      description: i.description,
      weightLabel: getHardeningLevelLabel(i.riskLevel),
    })),
  performance: (data) =>
    (data as PerformanceItem[]).map((i) => ({
      id: i.id,
      name: i.name,
      category: getCategoryLabel(i.category),
      isOk: i.isOptimized,
      statusLabel: i.isOptimized ? "Optimisé" : "À optimiser",
      description: i.description,
    })),
  bloatware: (data) =>
    (data as BloatwareItem[]).map((i) => ({
      id: i.packageName,
      name: i.displayName || i.packageName,
      category: getCategoryLabel(i.category || "Autres"),
      isOk: !i.isInstalled,
      statusLabel: i.isInstalled ? "Installé" : "Absent",
      description: i.publisher,
    })),
};

export function getDomainConfig(domain: ScoreDomain) {
  const def = SCORE_DOMAINS.find((d) => d.id === domain);
  if (!def) throw new Error(`Unknown score domain: ${domain}`);
  return {
    title: def.label,
    command: def.breakdownCommand,
    normalize: NORMALIZERS[domain],
  };
}
