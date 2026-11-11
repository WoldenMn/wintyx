import { useCallback, useEffect, useState, useRef } from "react";
import { useInvoke } from "../../hooks/use-invoke";
import { useConfirmBatchAction } from "../../hooks/use-confirm-batch-action";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../../components/ui/Toast";
import { PageHeader } from "../../components/ui/PageHeader";
import { ActionButton } from "../../components/ui/ActionButton";
import { StatCard } from "../../components/ui/StatCard";
import { ScoreBreakdown, type ScoreDomain } from "../../components/ui/ScoreBreakdown";
import { InfoTooltip } from "../../components/ui/InfoTooltip";
import { ErrorState } from "../../components/ui/ErrorState";
import { useScores } from "../../contexts/ScoresContext";
import { useDiagnosticsOptional } from "../../contexts/DiagnosticContext";
import { SCORE_DOMAINS } from "../../lib/scoreDomains";
import { getScoreBg, describeError } from "../../lib/helpers";
import { ScoreRing } from "../../components/ui/ScoreRing";
import { ScoreTrendCard } from "../../components/ui/ScoreTrendCard";
import { RefreshCw, FileText, ShieldCheck } from "lucide-react";
import type { SystemInfoData } from "../../lib/types";
import type { ScoreSnapshot, DefenderStatus } from "../../lib/bindings";
import { DefenderCard } from "../../components/ui/DefenderCard";
import { BrandMark } from "../../components/ui/BrandMark";
import { PERF_ENABLED } from "../../lib/perf-collector";

const PROTECT_ALL_COMMANDS = ["protect_all", "apply_hardening_essential"] as const;

export function DashboardPage() {
  const { data: sysInfo, isLoading: sysLoading, error: sysError, refresh: refreshSys } = useInvoke<SystemInfoData>("get_system_info");
  const { data: scoreHistory, error: historyError, refresh: refreshHistory } = useInvoke<ScoreSnapshot[]>("get_score_history");
  const { data: defender, isLoading: defenderLoading, error: defenderError } = useInvoke<DefenderStatus>("get_defender_status");
  const { scores, globalScore, loaded: scoresLoaded, measured: scoresMeasured, refreshScores, scoresError } = useScores();
  const { loaded: diagLoaded } = useDiagnosticsOptional();

  const [scanPhase, setScanPhase] = useState<"scanning" | "done">("scanning");
  const [scanProgress, setScanProgress] = useState(0);
  const [breakdownDomain, setBreakdownDomain] = useState<ScoreDomain | null>(null);
  const [reportBusy, setReportBusy] = useState(false);
  const scanIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const toast = useToast();
  const protectAll = useConfirmBatchAction({
    command: PROTECT_ALL_COMMANDS,
    confirmTitle: "Protéger tout",
    confirmMessage: "Appliquer les protections essentielles au système ?",
    confirmVariant: "danger",
    noun: { singular: "protection", plural: "protections", feminine: true },
    enablePreview: true,
    onSettled: () => void refreshScores(),
  });
  const isLoading = sysLoading || !scoresLoaded || !diagLoaded;
  const closeBreakdown = useCallback(() => setBreakdownDomain(null), []);

  function skipScan() {
    if (scanIntervalRef.current) clearInterval(scanIntervalRef.current);
    setScanProgress(100);
  }

  async function handleRefresh() {
    refreshSys();
    await refreshScores();
  }

  useEffect(() => {
    // Perf : T3 = isLoading bascule a false (system_info + scores tous resolus).
    // Compare avec T0 (main.tsx) pour avoir le temps total avant interactif reel.
    if (!isLoading) {
      try {
        performance.mark("wintyx-dashboard-ready");
        performance.measure("wintyx-time-to-interactive", "wintyx-app-init", "wintyx-dashboard-ready");
        const entries = performance.getEntriesByName("wintyx-time-to-interactive");
        if (entries[0] && PERF_ENABLED) {
          console.info(`[perf] T3 Time-to-interactive (Dashboard data ready) = ${Math.round(entries[0].duration)}ms`);
        }
      } catch {
        // ignore
      }
    }
    // H7 perf : si data deja chargee (cache SWR hit au remount), skipper l'animation
    // de 3s et passer directement en phase "done". Au 1er launch `isLoading=true`, anim joue.
    if (!isLoading) {
      setScanProgress(100);
      setScanPhase("done");
      return;
    }
    // Anim scan : 100% atteint en ~750ms (14-15 ticks de 50ms a +7).
    // Avant v2.7.7 : +3 par tick = ~1700ms artificiel. Si invokes <1700ms,
    // l'utilisateur attendait pour rien. -55% de delai au premier launch.
    scanIntervalRef.current = setInterval(() => {
      setScanProgress((prev) => {
        if (prev >= 100) {
          if (scanIntervalRef.current) clearInterval(scanIntervalRef.current);
          return 100;
        }
        return Math.min(prev + 7, 100);
      });
    }, 50);
    return () => {
      if (scanIntervalRef.current) clearInterval(scanIntervalRef.current);
    };
  }, [isLoading]);

  useEffect(() => {
    if (!isLoading && scanProgress >= 100) {
      setScanPhase("done");
    }
  }, [isLoading, scanProgress]);

  if (scanPhase === "scanning") {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-6">
        <BrandMark size={64} />
        <p className="text-fg-dim text-sm">Analyse du système...</p>
        <div
          role="progressbar"
          aria-label="Analyse du système"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={scanProgress}
          className="w-64 h-1.5 bg-bg-elevated rounded-full overflow-hidden"
        >
          <div className="h-full bg-accent rounded-full transition-all duration-100" style={{ width: `${scanProgress}%` }} />
        </div>
        <p className="text-fg-muted text-xs">{scanProgress}%</p>
        {scanProgress >= 100 ? (
          <p role="status" className="text-xs text-fg-dim">En attente des données système...</p>
        ) : (
          <button
            onClick={skipScan}
            className="text-xs text-fg-dim hover:text-fg-primary underline underline-offset-4 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent rounded px-2 py-1"
          >
            Passer
          </button>
        )}
      </div>
    );
  }

  async function handleGenerateReport() {
    if (reportBusy) return;
    setReportBusy(true);
    try {
      await invoke("generate_report");
      toast.success("Rapport généré et ouvert");
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
    } finally {
      setReportBusy(false);
    }
  }

  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader
        title="Tableau de bord"
        subtitle="Vue d'ensemble de la protection du système"
        actions={
          <>
            <ActionButton onClick={handleRefresh}><RefreshCw size={16} aria-hidden="true" />Rafraîchir</ActionButton>
            <ActionButton loading={reportBusy} onClick={handleGenerateReport}><FileText size={16} aria-hidden="true" />Rapport</ActionButton>
            <ActionButton variant="accent" requiresAdmin loading={protectAll.isRunning} onClick={() => void protectAll.trigger()}><ShieldCheck size={16} aria-hidden="true" />Protéger tout</ActionButton>
          </>
        }
      />

      {!scoresMeasured && scoresError ? (
        <div role="alert" className="mb-6 bg-warning/10 border border-warning/30 rounded-lg p-6 text-sm text-warning">
          <p className="font-medium">Scores indisponibles : la lecture a échoué ({scoresError}).</p>
          <ActionButton onClick={handleRefresh} className="mt-3 px-3 py-1.5 text-xs">Réessayer</ActionButton>
        </div>
      ) : (
        <>
          <div
            className={`bg-bg-card border border-border rounded-lg p-6 mb-6 flex items-center gap-6 theme-card ${getScoreBg(globalScore)}`}
            title="% de recommandations Wintyx appliquées sur 5 domaines (Protection, Services, Durcissement, Performance, Bloatware). 0% = configuration Windows par défaut, pas un danger."
          >
            <ScoreRing value={globalScore} size={104} ariaLabel={`Recommandations Wintyx appliquées : ${globalScore} sur 100`} />
            <div className="flex-1">
              <p className="text-lg font-medium text-fg-primary flex items-center gap-2">
                Recommandations Wintyx appliquées
                <InfoTooltip textId="score_global" ariaLabel="Comment est calculé le score global" size="md" />
              </p>
              <p className="text-sm text-fg-dim">% appliqué sur 5 domaines (protection, services, durcissement, performance, bloatware), 0% correspondant à la config Windows par défaut</p>
              {SCORE_DOMAINS.some((d) => scores[d.id] === null) && (
                <p className="text-xs text-warning mt-1">
                  Calculé sans : {SCORE_DOMAINS.filter((d) => scores[d.id] === null).map((d) => d.label).join(", ")} (lecture impossible)
                </p>
              )}
            </div>
          </div>

          {scoresError && (
            <div role="alert" className="mb-6 bg-warning/10 border border-warning/30 rounded-lg px-4 py-3 text-xs text-warning">
              Scores peut-être obsolètes : le dernier rafraîchissement a échoué ({scoresError}).
            </div>
          )}

          <div className="grid grid-cols-5 gap-4 mb-6">
            {SCORE_DOMAINS.map((d) => {
              const score = scores[d.id];
              if (score === null) {
                return (
                  <StatCard
                    key={d.id}
                    label={d.label}
                    value="--"
                    subtitle="Lecture impossible"
                    onClick={() => setBreakdownDomain(d.id)}
                  />
                );
              }
              return (
                <StatCard
                  key={d.id}
                  label={d.label}
                  value={score}
                  suffix="%"
                  score={score}
                  progress={score}
                  subtitle="Cliquer pour voir le détail"
                  onClick={() => setBreakdownDomain(d.id)}
                />
              );
            })}
          </div>
        </>
      )}

      <ScoreBreakdown domain={breakdownDomain} onClose={closeBreakdown} />

      <ScoreTrendCard history={scoreHistory ?? []} error={historyError} onRetry={refreshHistory} />
      <DefenderCard
        status={defender ?? undefined}
        activeAntivirus={sysInfo?.antivirusName}
        loading={defenderLoading}
        error={defenderError}
      />

      {sysError ? (
        <ErrorState message={sysError} onRetry={refreshSys} title="Infos système indisponibles" />
      ) : sysInfo && (
        <div className="grid grid-cols-2 gap-4">
          <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
            <p className="text-xs text-fg-muted mb-1">Système</p>
            <p className="text-sm font-medium text-fg-primary">{sysInfo.osName}</p>
            <p className="text-xs text-fg-dim">Antivirus : {sysInfo.antivirusName || "Inconnu"}</p>
          </div>
          <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
            <p className="text-xs text-fg-muted mb-1">Processeur</p>
            <p className="text-sm font-medium text-fg-primary">{sysInfo.cpuName}</p>
            <p className="text-xs text-fg-dim">{sysInfo.cpuCores} cœurs à {sysInfo.cpuClockMhz} MHz</p>
          </div>
          <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
            <p className="text-xs text-fg-muted mb-1">GPU</p>
            <p className="text-sm font-medium text-fg-primary">{sysInfo.gpuName || "Non détecté"}</p>
          </div>
          <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
            <p className="text-xs text-fg-muted mb-1">Mémoire</p>
            <p className="text-sm font-medium text-fg-primary">{sysInfo.usedRamGb.toFixed(1)} / {sysInfo.totalRamGb.toFixed(1)} Go</p>
          </div>
          <div className="bg-bg-card border border-border rounded-lg p-5 theme-card">
            <p className="text-xs text-fg-muted mb-1">Machine</p>
            <p className="text-sm font-medium text-fg-primary">{sysInfo.computerName}</p>
            <p className="text-xs text-fg-dim">Disque : {sysInfo.diskUsedPercent}% utilisé</p>
          </div>
        </div>
      )}
    </div>
  );
}

export default DashboardPage;
