import { AlertTriangle } from "lucide-react";
import type { AppDiagnostics } from "../../lib/bindings";

interface DiagnosticBannerProps {
  diagnostics: AppDiagnostics | null;
  onDiagnose: () => void;
}

/**
 * Banniere globale : visible uniquement quand un prerequis DUR (WMI / PowerShell /
 * stockage) est en echec. Un non-admin (Warning) ne la declenche pas.
 */
export function DiagnosticBanner({ diagnostics, onDiagnose }: DiagnosticBannerProps) {
  if (!diagnostics?.hasCriticalFailure) return null;

  const failed = diagnostics.checks.filter((c) => c.status === "failed");
  const pages = [...new Set(failed.flatMap((c) => c.affectedPages))];
  const labels = failed.map((c) => c.label).join(", ");

  return (
    <div
      role="alert"
      className="flex items-center gap-3 px-4 py-2 bg-danger text-on-semantic text-sm"
    >
      <AlertTriangle size={16} className="shrink-0" aria-hidden="true" />
      <span className="flex-1">
        Prérequis indisponible : {labels}.
        {pages.length > 0 && ` Pages limitées : ${pages.join(", ")}.`}
      </span>
      <button
        onClick={onDiagnose}
        className="shrink-0 underline font-medium hover:no-underline focus:outline-none focus-visible:ring-2 focus-visible:ring-on-semantic rounded px-1"
      >
        Diagnostiquer
      </button>
    </div>
  );
}

export default DiagnosticBanner;
