import { CheckCircle2, AlertTriangle, XCircle, RefreshCw } from "lucide-react";
import { useDiagnostics } from "../../contexts/DiagnosticContext";
import { ActionButton } from "../ui/ActionButton";
import type { DiagnosticCheck, DiagnosticStatus } from "../../lib/bindings";

const ICON: Record<DiagnosticStatus, typeof CheckCircle2> = {
  ok: CheckCircle2,
  warning: AlertTriangle,
  failed: XCircle,
};
const COLOR: Record<DiagnosticStatus, string> = {
  ok: "text-success",
  warning: "text-warning",
  failed: "text-danger",
};
const STATUS_LABEL: Record<DiagnosticStatus, string> = {
  ok: "OK",
  warning: "Attention",
  failed: "Échec",
};

function CheckRow({ check }: { check: DiagnosticCheck }) {
  const Icon = ICON[check.status];
  return (
    <div className="flex items-start gap-3 p-4 bg-bg-card border border-border rounded-lg theme-card">
      <Icon size={20} className={`${COLOR[check.status]} shrink-0 mt-0.5`} aria-hidden="true" />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-fg-primary text-sm font-medium">{check.label}</span>
          <span className={`text-xs font-semibold ${COLOR[check.status]}`}>{STATUS_LABEL[check.status]}</span>
        </div>
        <p className="text-fg-dim text-xs mt-1 leading-relaxed">{check.detail}</p>
        {check.affectedPages.length > 0 && (
          <p className="text-fg-muted text-xs mt-1">
            Pages concernées : {check.affectedPages.join(", ")}
          </p>
        )}
      </div>
    </div>
  );
}

export default function DiagnosticPage() {
  const { diagnostics, loaded, loading, error, refresh } = useDiagnostics();

  return (
    <div className="p-7 overflow-y-auto h-full">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-fg-primary text-2xl font-semibold">Diagnostic</h1>
          <p className="text-fg-dim text-sm mt-1">
            Prérequis dont dépendent les pages de Wintyx.
          </p>
        </div>
        <ActionButton variant="neutral" onClick={() => void refresh()} disabled={loading}>
          <RefreshCw size={16} className={loading ? "animate-spin" : ""} aria-hidden="true" />
          Relancer
        </ActionButton>
      </div>

      {!loaded ? (
        <p className="mt-6 text-fg-dim text-sm">Analyse en cours…</p>
      ) : error ? (
        <p className="mt-6 text-danger text-sm" role="alert">
          Le diagnostic n'a pas pu s'exécuter : {error}
        </p>
      ) : (
        <div className="mt-6 grid grid-cols-2 gap-3 items-start">
          {diagnostics?.checks.map((c) => <CheckRow key={c.id} check={c} />)}
        </div>
      )}
    </div>
  );
}
