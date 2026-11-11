import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../../components/ui/Toast";
import { useConfirmBatchAction } from "../../hooks/use-confirm-batch-action";
import { PageHeader } from "../../components/ui/PageHeader";
import { ActionButton } from "../../components/ui/ActionButton";
import { Badge } from "../../components/ui/Badge";
import { diffPercent, isImprovement, describeError } from "../../lib/helpers";
import { Minus, TrendingDown, TrendingUp } from "lucide-react";

import { estimateLabel } from "../../lib/benchmark";
import type { BatchResult, BenchmarkEntry, BenchmarkResult } from "../../lib/types";
import { BenchmarkHistory, formatThroughput } from "./BenchmarkHistory";

type Phase = "idle" | "benchmarking" | "before" | "optimizing" | "after";

interface MetricDef {
  label: string;
  key: keyof BenchmarkResult;
  unit: string;
  format: (v: number) => string;
  higherIsBetter: boolean;
}

function roundedDiff(percent: number): number {
  return Math.abs(percent) < 0.05 ? 0 : percent;
}

const METRICS: MetricDef[] = [
  { label: "CPU", key: "cpuScore", unit: "M ops/s", format: (v) => v.toFixed(1), higherIsBetter: true },
  { label: "Mémoire", key: "memoryScore", unit: "Mo/s", format: formatThroughput, higherIsBetter: true },
  { label: "Disque lecture", key: "diskReadScore", unit: "Mo/s", format: formatThroughput, higherIsBetter: true },
  { label: "Disque écriture", key: "diskWriteScore", unit: "Mo/s", format: formatThroughput, higherIsBetter: true },
  { label: "Latence réseau", key: "networkLatency", unit: "ms", format: (v) => v.toFixed(1), higherIsBetter: false },
  { label: "Durée du test", key: "durationMs", unit: "s", format: (v) => (v / 1000).toFixed(1), higherIsBetter: false },
];

export function BenchmarkPage() {
  const [phase, setPhase] = useState<Phase>("idle");
  const [beforeResult, setBeforeResult] = useState<BenchmarkResult | null>(null);
  const [afterResult, setAfterResult] = useState<BenchmarkResult | null>(null);
  const [optimization, setOptimization] = useState<BatchResult | null>(null);
  const [history, setHistory] = useState<BenchmarkEntry[]>([]);
  const [historyError, setHistoryError] = useState<string | null>(null);
  const toast = useToast();

  const loadHistory = useCallback(async () => {
    try {
      const data = await invoke<BenchmarkEntry[]>("get_benchmark_history");
      setHistory(data);
      setHistoryError(null);
    } catch (e) {
      setHistoryError(describeError(e));
    }
  }, []);

  useEffect(() => {
    void loadHistory();
  }, [loadHistory]);

  const runBenchmark = async () => {
    setPhase("benchmarking");
    try {
      const data = await invoke<BenchmarkResult>("run_benchmark");
      setBeforeResult(data);
      setAfterResult(null);
      setOptimization(null);
      setPhase("before");
      toast.success("Benchmark terminé");
      void loadHistory();
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
      setPhase("idle");
    }
  };

  const retest = async () => {
    setPhase("optimizing");
    try {
      const data = await invoke<BenchmarkResult>("run_benchmark");
      setAfterResult(data);
      setPhase("after");
      toast.success("Benchmark de comparaison terminé");
      void loadHistory();
    } catch (e) {
      toast.error(`Erreur : ${describeError(e)}`);
      setPhase("before");
    }
  };

  const optimize = useConfirmBatchAction({
    command: "optimize_all",
    confirmTitle: "Optimiser et re-tester",
    confirmMessage:
      "Cette action va appliquer les optimisations de performance puis relancer le benchmark. Continuer ?",
    confirmVariant: "warning",
    noun: { singular: "optimisation", plural: "optimisations", feminine: true },
    onSettled: (result) => {
      if (result.success === 0) return;
      setOptimization(result);
      void retest();
    },
  });


  return (
    <div className="p-7 overflow-y-auto h-full">
      <PageHeader title="Benchmark" subtitle="Mesurez et comparez les performances" />

      {phase === "idle" && (
        <>
          <div className="bg-bg-card border border-border rounded-lg p-12 flex flex-col items-center justify-center text-center theme-card">
            <h2 className="text-xl font-semibold text-fg-primary mb-2">Testez votre système</h2>
            <p className="text-sm text-fg-dim mb-6 max-w-md">
              Lancez un benchmark complet pour mesurer les performances CPU, mémoire, disque et réseau.
            </p>
            <ActionButton variant="accent" onClick={runBenchmark}>
              Lancer le benchmark
            </ActionButton>
          </div>

        </>
      )}

      {(phase === "benchmarking" || phase === "optimizing") && (
        <div className="bg-bg-card border border-border rounded-lg p-12 flex flex-col items-center justify-center text-center theme-card">
          <div className="w-10 h-10 border-2 border-accent/30 border-t-accent rounded-full animate-spin mb-4" />
          <h2 className="text-xl font-semibold text-fg-primary mb-2">
            {phase === "optimizing" ? "Optimisation en cours..." : "Test en cours..."}
          </h2>
          <p className="text-sm text-fg-dim">
            {phase === "optimizing" ? "Application des optimisations et re-test" : estimateLabel(history)}
          </p>
          <div className="mt-4 w-64 h-1.5 bg-bg-elevated rounded-full overflow-hidden">
            <div className="h-full bg-accent rounded-full animate-pulse w-full" />
          </div>
        </div>
      )}

      {(phase === "before" || phase === "after") && beforeResult && (
        <>
          <div className="bg-bg-card border border-border rounded-lg p-6 mb-6 text-center theme-card">
            <p className="text-xs text-fg-dim uppercase tracking-wider mb-2">Score global</p>
            <p className="text-5xl font-bold text-accent">
              {beforeResult.overallScore.toFixed(0)}<span className="text-lg text-fg-dim">/100</span>
            </p>
          </div>

          <div className="grid grid-cols-3 gap-3 mb-6">
            {METRICS.map((m) => (
              <div key={m.key} className="bg-bg-card border border-border rounded-lg p-5 min-w-[180px] theme-card">
                <p className="text-xs text-fg-dim uppercase tracking-wider mb-1">{m.label}</p>
                <p className="text-2xl font-bold text-fg-primary">
                  {m.format(beforeResult[m.key])} <span className="text-sm text-fg-dim font-normal">{m.unit}</span>
                </p>
              </div>
            ))}
          </div>

          {phase === "before" && (
            <div className="text-center">
              <ActionButton variant="accent" requiresAdmin loading={optimize.isRunning} onClick={() => void optimize.trigger()}>
                Optimiser & Re-tester
              </ActionButton>
            </div>
          )}

          {phase === "after" && afterResult && (
            <>
              <h2 className="text-lg font-semibold text-fg-primary mb-3">Comparaison Avant / Après</h2>
              {optimization && optimization.failed > 0 && (
                <p role="alert" className="bg-warning/10 text-warning border border-warning/30 rounded-lg p-3 mb-4 text-sm">
                  Optimisations appliquées : {optimization.success} sur {optimization.total}. L'écart mesuré ne reflète qu'une partie des changements.
                </p>
              )}
              <div className="grid grid-cols-2 gap-3 mb-6">
                {METRICS.map((m) => {
                  const bVal = beforeResult[m.key];
                  const aVal = afterResult[m.key];
                  const diff = roundedDiff(diffPercent(bVal, aVal));
                  const improved = isImprovement(bVal, aVal, m.higherIsBetter);
                  const badgeVariant = diff === 0 ? "neutral" : improved ? "success" : "danger";
                  return (
                    <div key={m.key} className="bg-bg-card border border-border rounded-lg p-4 theme-card">
                      <p className="text-xs text-fg-dim uppercase tracking-wider mb-2">{m.label}</p>
                      <div className="flex items-center gap-3 flex-wrap">
                        <span className="text-fg-dim text-sm">{m.format(bVal)} {m.unit}</span>
                        <span className="text-fg-muted">→</span>
                        <span className="text-fg-primary text-sm font-medium">{m.format(aVal)} {m.unit}</span>
                        <Badge variant={badgeVariant}>
                          {diff > 0 ? "+" : ""}{diff.toFixed(1)}%
                        </Badge>
                      </div>
                    </div>
                  );
                })}
              </div>

              <div className="bg-bg-card border border-border rounded-lg p-6 text-center theme-card">
                <p className="text-xs text-fg-dim uppercase tracking-wider mb-2">Amélioration globale</p>
                {(() => {
                  const d = roundedDiff(diffPercent(beforeResult.overallScore, afterResult.overallScore));
                  const Trend = d > 0 ? TrendingUp : d < 0 ? TrendingDown : Minus;
                  const tone = d > 0 ? "text-success" : d < 0 ? "text-danger" : "text-fg-dim";
                  return (
                    <p className={`inline-flex items-center gap-2 text-3xl font-bold ${tone}`}>
                      <Trend size={28} aria-hidden="true" />
                      {d > 0 ? "+" : ""}{d.toFixed(1)}%
                    </p>
                  );
                })()}
                <p className="text-sm text-fg-dim mt-1">
                  {beforeResult.overallScore.toFixed(0)} → {afterResult.overallScore.toFixed(0)}
                </p>
              </div>

              <div className="text-center mt-6">
                <ActionButton variant="neutral" onClick={runBenchmark}>
                  Nouveau benchmark
                </ActionButton>
              </div>
            </>
          )}
        </>
      )}

      {phase !== "benchmarking" && phase !== "optimizing" && (
        <BenchmarkHistory history={history} error={historyError} onRetry={() => void loadHistory()} />
      )}
    </div>
  );
}

export default BenchmarkPage;
