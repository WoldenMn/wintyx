import type { DriverIssue, IssueKind } from "../../lib/types";

export type AuditFilter = "all" | "unsigned" | "duplicate" | "phantom" | "obsolete" | "thirdPartyOem";

export const FILTER_LABELS: Record<AuditFilter, string> = {
  all: "Tous",
  unsigned: "Non signés",
  duplicate: "Doublons",
  phantom: "Fantômes",
  obsolete: "Obsolètes",
  thirdPartyOem: "OEM tiers",
};

export function severityVariant(s: string) {
  if (s === "high") return "danger" as const;
  if (s === "medium") return "warning" as const;
  return "info" as const;
}

export function severityLabel(s: string) {
  if (s === "high") return "Élevé";
  if (s === "medium") return "Moyen";
  return "Faible";
}

export function kindLabel(kind: IssueKind["kind"]) {
  switch (kind) {
    case "unsigned": return "Non signé";
    case "duplicate": return "Doublon";
    case "phantom": return "Fantôme";
    case "obsolete": return "Obsolète";
    case "thirdPartyOem": return "OEM tiers";
  }
}

export function kindHelpId(kind: IssueKind["kind"]) {
  switch (kind) {
    case "duplicate": return "driver_duplicate";
    case "phantom": return "driver_phantom";
    case "obsolete": return "driver_obsolete";
    case "thirdPartyOem": return "driver_third_party";
    case "unsigned": return "driver_unsigned";
  }
}

export function canUninstall(issue: DriverIssue): boolean {
  const k = issue.kind.kind;
  return (k === "duplicate" || k === "obsolete" || k === "thirdPartyOem" || k === "unsigned")
    && issue.infName.startsWith("oem");
}

export function canRemovePhantom(issue: DriverIssue): boolean {
  return issue.kind.kind === "phantom" && issue.instanceId !== null;
}
