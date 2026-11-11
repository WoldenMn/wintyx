import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useToast } from "../../components/ui/Toast";
import { PageHeader } from "../../components/ui/PageHeader";
import { ActionButton } from "../../components/ui/ActionButton";
import { getRiskScoreColor, describeError } from "../../lib/helpers";
import { SEVERITY_STYLES, RISK_ORDER, getSeverityLabel } from "../../lib/constants";

import type { ThreatScanResult, ThreatScanProgress } from "../../lib/types";

const NONE_STYLE = "bg-success/10 text-success";

export function ThreatScanPage() {
  const [phase, setPhase] = useState<"idle" | "scanning" | "done">("idle");
  const [result, setResult] = useState<ThreatScanResult | null>(null);
  const [progress, setProgress] = useState<ThreatScanProgress | null>(null);
  const [progressUnavailable, setProgressUnavailable] = useState(false);
  const [scanError, setScanError] = useState<string | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);
  const toast = useToast();

  useEffect(() => {
    let mounted = true;
    listen<ThreatScanProgress>("threat-scan-progress", (event) => {
      if (!mounted) return;
      setProgress(event.payload);
    })
      .then((fn) => {
        if (!mounted) {
          fn();
          return;
        }
        unlistenRef.current = fn;
      })
      .catch((e) => {
        // Important : ne pas silently swallow — sinon un UnhandledPromiseRejection
        // pollue la console et masque des bugs futurs.
        console.error("[ThreatScanPage] listen() failed:", e);
        if (mounted) setProgressUnavailable(true);
      });
    return () => {
      mounted = false;
      unlistenRef.current?.();
    };
  }, []);

  const runScan = async () => {
    setPhase("scanning");
    setProgress(null);
    setScanError(null);
    try {
      const data = await invoke<ThreatScanResult>("run_threat_scan");
      setResult(data);
      setPhase("done");
      const failedCount = data.categories.filter((c) => c.failed).length;
      if (failedCount > 0) {
        toast.warning(`Scan terminé : ${failedCount} vérification(s) non analysée(s)`);
      } else {
        toast.success("Scan terminé");
      }
    } catch (e) {
      const message = describeError(e);
      toast.error(`Erreur : ${message}`);
      setScanError(message);
      setPhase(result ? "done" : "idle");
    }
  };

  const failedCategories = result ? result.categories.filter((c) => c.failed) : [];
  const allCategoriesFailed = result !== null && result.categories.length > 0 && failedCategories.length === result.categories.length;

  const sortedThreats = result
    ? [...result.threats].sort((a, b) => (RISK_ORDER[a.severity] ?? 9) - (RISK_ORDER[b.severity] ?? 9))
    : [];

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Scanner de menaces"
        subtitle="Analysez votre système pour détecter les vulnérabilités"
      />

      {scanError && (
        <div
          data-testid="scan-error"
          role="alert"
          className="bg-danger/10 text-danger border border-danger/30 rounded-lg p-4 mb-6 text-sm"
        >
          Le scan a échoué : {scanError}
          {result && " Les résultats affichés datent du scan précédent."}
        </div>
      )}

      {phase === "idle" && !result && (
        <div className="bg-bg-card border border-border rounded-lg p-12 flex flex-col items-center justify-center text-center theme-card">
          <h2 className="text-xl font-semibold text-fg-primary mb-2">Analysez votre système</h2>
          <p className="text-sm text-fg-dim mb-6 max-w-md">
            Lancez un scan complet pour détecter les menaces, vulnérabilités et configurations à risque.
          </p>
          <ActionButton variant="accent" onClick={runScan}>
            Lancer le scan
          </ActionButton>
        </div>
      )}

      {phase === "scanning" && (
        <div
          className="bg-bg-card border border-border rounded-lg p-12 flex flex-col items-center justify-center text-center theme-card"
          role="status"
          aria-live="polite"
        >
          <div className="w-10 h-10 border-2 border-accent/30 border-t-accent rounded-full animate-spin mb-4" />
          <h2 className="text-xl font-semibold text-fg-primary mb-2">Analyse en cours...</h2>
          <p className="text-sm text-fg-dim min-h-[1.25rem]">
            {progress
              ? `${progress.phaseIndex + 1}/${progress.phaseCount} vérifications terminées (${progress.phaseName})`
              : progressUnavailable
                ? "Analyse en cours (progression indisponible)"
                : "Préparation..."}
          </p>
          <p className="text-xs text-fg-muted mt-1 min-h-[1rem]">
            {progress ? `${progress.scannedSoFar} éléments scannés` : ""}
          </p>
          <div className="mt-4 w-64 h-1.5 bg-bg-elevated rounded-full overflow-hidden">
            <div
              className="h-full bg-accent rounded-full transition-all duration-300"
              style={{
                width: progress
                  ? `${Math.min(100, Math.round(((progress.phaseIndex + 1) / progress.phaseCount) * 100))}%`
                  : "5%",
              }}
            />
          </div>
        </div>
      )}

      {phase === "done" && result && (
        <>
          <div className="bg-bg-card border border-border rounded-lg p-6 mb-6 text-center theme-card">
            <p className="text-xs text-fg-dim uppercase tracking-wider mb-2">Indice de risque estimé</p>
            {allCategoriesFailed ? (
              <p className="text-2xl font-bold text-warning">Aucune catégorie n'a pu être analysée</p>
            ) : (
              <>
                <p
                  className={`text-5xl font-bold ${failedCategories.length > 0 ? "text-fg-primary" : getRiskScoreColor(result.riskScore)}`}
                >
                  {result.riskScore}<span className="text-lg text-fg-dim">/100</span>
                </p>
                <p className="text-sm text-fg-dim mt-2">
                  {result.totalItemsScanned} éléments analysés, {result.threats.length} menaces détectées
                </p>
              </>
            )}
          </div>

          {failedCategories.length > 0 && (
            <div role="alert" className="bg-warning/10 text-warning border border-warning/30 rounded-lg p-4 mb-6 text-sm">
              Analyse incomplète, l'indice de risque est partiel. Non analysé : {failedCategories.map((c) => c.name).join(", ")}.
            </div>
          )}

          <div className="grid grid-cols-3 gap-3 mb-6">
            {result.categories.map((cat) => (
              <div key={cat.name} className="bg-bg-card border border-border rounded-lg p-4 theme-card">
                <p className="text-sm font-medium text-fg-primary mb-1">{cat.name}</p>
                <p className="text-xs text-fg-dim">{cat.scanned} scannés</p>
                <div className="flex items-center gap-2 mt-2 flex-wrap">
                  {cat.failed ? (
                    <span className="px-3 py-1 rounded-full text-xs bg-warning/10 text-warning">Non analysé</span>
                  ) : (
                    <>
                      <span className={`px-3 py-1 rounded-full text-xs ${SEVERITY_STYLES[cat.riskLevel] ?? NONE_STYLE}`}>
                        {cat.threatsFound} menaces
                      </span>
                      <span className={`px-3 py-1 rounded-full text-xs ${SEVERITY_STYLES[cat.riskLevel] ?? NONE_STYLE}`}>
                        {getSeverityLabel(cat.riskLevel)}
                      </span>
                    </>
                  )}
                </div>
              </div>
            ))}
          </div>

          {sortedThreats.length > 0 && (
            <div className="mb-6">
              <h2 className="text-lg font-semibold text-fg-primary mb-3">Menaces détectées</h2>
              <div className="space-y-3">
                {sortedThreats.map((threat, i) => (
                  <div key={`${threat.path}-${threat.name}-${i}`} className="bg-bg-card border border-border rounded-lg p-5 theme-card">
                    <div className="flex items-start justify-between gap-3">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1 flex-wrap">
                          <span className={`px-3 py-1 rounded-full text-xs ${SEVERITY_STYLES[threat.severity] ?? SEVERITY_STYLES.low}`}>
                            {getSeverityLabel(threat.severity)}
                          </span>
                          <span className="text-sm font-medium text-fg-primary">{threat.name}</span>
                        </div>
                        <p className="text-sm text-fg-dim mt-1">{threat.description}</p>
                        <p className="text-xs text-fg-muted mt-1 font-mono break-all">{threat.path}</p>
                      </div>
                      <span className="px-3 py-1.5 rounded-md text-xs bg-warning/10 text-warning shrink-0">
                        Correction manuelle requise
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="text-center">
            <ActionButton variant="neutral" onClick={runScan}>
              Relancer le scan
            </ActionButton>
          </div>
        </>
      )}
    </div>
  );
}

export default ThreatScanPage;
