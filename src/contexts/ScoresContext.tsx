import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SCORE_DOMAINS } from "../lib/scoreDomains";
import type { AllScores, DomainScores } from "../lib/bindings";
import { describeError } from "../lib/helpers";
import { PERF_ENABLED } from "../lib/perf-collector";

interface ScoresContextType {
  scores: AllScores;
  globalScore: number;
  loaded: boolean;
  measured: boolean;
  refreshScores: () => Promise<void>;
  /** Non-null si le dernier refresh a echoue : les scores affiches sont peut-etre obsoletes. */
  scoresError: string | null;
}

const EMPTY_SCORES: AllScores = SCORE_DOMAINS.reduce(
  (acc, d) => ({ ...acc, [d.id]: 0 }),
  {} as AllScores,
);

const ScoresContext = createContext<ScoresContextType | null>(null);

export function ScoresProvider({ children }: { children: ReactNode }) {
  const [scores, setScores] = useState<AllScores>(EMPTY_SCORES);
  const [loaded, setLoaded] = useState(false);
  const [measured, setMeasured] = useState(false);
  const [scoresError, setScoresError] = useState<string | null>(null);
  // Promesse en vol : un appel concurrent retourne la MEME promesse au lieu de
  // resoudre immediatement, pour que `await refreshScores()` apres une action
  // reflete reellement les scores a jour (bug-frontend-02).
  const inFlightRef = useRef<Promise<void> | null>(null);

  const refreshScores = useCallback(() => {
    if (inFlightRef.current) return inFlightRef.current;
    const run = (async () => {
      const tPerf = performance.now();
      try {
        // PERF-01 v2.16.0 : 1 seul aller-retour IPC (get_all_scores, parallele
        // cote backend) au lieu de 5 invokes Promise.all. Les cles du struct
        // AllScores backend correspondent aux ids du registre SCORE_DOMAINS.
        const all = await invoke<AllScores>("get_all_scores");
        const next: AllScores = SCORE_DOMAINS.reduce(
          (acc, d) => ({ ...acc, [d.id]: all[d.id] ?? null }),
          {} as AllScores,
        );
        setScores(next);
        setMeasured(true);
        setScoresError(null);
        // Persistance fire-and-forget de l'historique (1 point/jour cote backend).
        // N'altere jamais le refresh : une erreur est seulement journalisee.
        if (SCORE_DOMAINS.every((d) => next[d.id] !== null)) {
          invoke("record_score_snapshot", { scores: next as DomainScores }).catch((e) =>
            console.warn("[scores-history] record echoue:", describeError(e)),
          );
        }
      } catch (e) {
        setScoresError(describeError(e));
      } finally {
        setLoaded(true);
        inFlightRef.current = null;
        if (PERF_ENABLED) {
          console.info(`[perf] T2 ScoresContext refreshScores (1 invoke get_all_scores) = ${Math.round(performance.now() - tPerf)}ms`);
        }
      }
    })();
    inFlightRef.current = run;
    return run;
  }, []);

  useEffect(() => {
    void refreshScores();
  }, [refreshScores]);

  const globalScore = useMemo(() => {
    const read = SCORE_DOMAINS.map((d) => scores[d.id]).filter((v): v is number => v !== null);
    return read.length === 0 ? 0 : Math.round(read.reduce((sum, v) => sum + v, 0) / read.length);
  }, [scores]);

  const value = useMemo(
    () => ({ scores, globalScore, loaded, measured, refreshScores, scoresError }),
    [scores, globalScore, loaded, measured, refreshScores, scoresError],
  );

  return <ScoresContext.Provider value={value}>{children}</ScoresContext.Provider>;
}

export function useScores() {
  const ctx = useContext(ScoresContext);
  if (!ctx) throw new Error("useScores must be used within ScoresProvider");
  return ctx;
}
