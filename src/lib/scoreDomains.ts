// Les scores par domaine sont recuperes en un seul invoke batch `get_all_scores`
// (PERF-01 v2.16.0) : les ids ci-dessous correspondent aux cles du struct
// AllScores backend. Plus de `scoreCommand` par domaine.
import type { CommandName } from "./command-names";

export interface ScoreDomainDef {
  id: string;
  label: string;
  breakdownCommand: CommandName;
  subtitle?: string;
}

export const SCORE_DOMAINS = [
  {
    id: "dataProtection",
    label: "Protection des données",
    breakdownCommand: "get_data_protection_items",
    subtitle: "Télémétrie, suivi, publicité",
  },
  {
    id: "services",
    label: "Services",
    breakdownCommand: "get_services",
    subtitle: "Services Windows à risque",
  },
  {
    id: "hardening",
    label: "Durcissement",
    breakdownCommand: "get_hardening_checks",
    subtitle: "SMBv1, UAC, WDigest, LLMNR",
  },
  {
    id: "performance",
    label: "Performance",
    breakdownCommand: "get_performance_items",
    subtitle: "Effets visuels, prefetch, plans énergie",
  },
  {
    id: "bloatware",
    label: "Bloatware",
    breakdownCommand: "get_bloatware_items",
    subtitle: "Applications préinstallées Microsoft",
  },
] as const satisfies readonly ScoreDomainDef[];

export type ScoreDomainId = typeof SCORE_DOMAINS[number]["id"];
