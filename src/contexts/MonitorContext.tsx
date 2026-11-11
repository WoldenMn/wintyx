import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { describeError } from "../lib/helpers";
import type { SystemMetrics } from "../lib/types";

const HISTORY_MAX = 60;
const DEFAULT_INTERVAL_MS = 1500;
// Une collecte dure ~400 ms (classes WMI brutes) : en dessous de 1 s l'intervalle demande ne serait pas tenu.
const MIN_INTERVAL_MS = 1000;
const MEASURE_WINDOW = 5;
const MAX_INTERVAL_MS = 10000;

interface MonitorContextValue {
  metrics: SystemMetrics | null;
  cpuHistory: number[];
  ramHistory: number[];
  isRunning: boolean;
  /** Non-null si start/stop monitoring a echoue. */
  error: string | null;
  intervalMs: number;
  /** Moyenne des derniers ecarts reels entre deux mesures recues, null tant qu'il n'y en a pas. */
  measuredIntervalMs: number | null;
  intervalError: string | null;
  setInterval: (intervalMs: number) => Promise<void>;
  pause: () => Promise<void>;
  resume: () => Promise<void>;
  subscribe: () => void;
  unsubscribe: () => void;
}

const MonitorContext = createContext<MonitorContextValue | null>(null);

export function MonitorProvider({ children }: { children: ReactNode }) {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [cpuHistory, setCpuHistory] = useState<number[]>([]);
  const [ramHistory, setRamHistory] = useState<number[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [listenError, setListenError] = useState<string | null>(null);
  const [intervalMs, setIntervalMsState] = useState(DEFAULT_INTERVAL_MS);
  const [intervalError, setIntervalError] = useState<string | null>(null);
  const [measuredIntervalMs, setMeasuredIntervalMs] = useState<number | null>(null);
  const lastEventAtRef = useRef<number | null>(null);
  const deltasRef = useRef<number[]>([]);
  const refcount = useRef(0);
  const startRequestedRef = useRef(false);
  const intervalRef = useRef(DEFAULT_INTERVAL_MS);
  // Intention utilisateur (bouton Arreter). Distinct du refcount : le monitoring
  // tourne si (un consommateur est monte) ET (pas en pause manuelle). Evite la
  // desync bouton/etat reel (run-10).
  const pausedRef = useRef(false);

  useEffect(() => {
    // FE-01 : listen() est async — si le cleanup s'execute avant la resolution
    // de la promesse, le listener Tauri ne serait jamais desabonne. Le flag
    // cancelled couvre ce cas : la promesse resolue apres cleanup appelle fn()
    // immediatement au lieu de stocker un unlisten orphelin.
    let cancelled = false;
    let unlistenFn: UnlistenFn | null = null;
    listen<SystemMetrics>("system-metrics", (event) => {
      if (cancelled) return;
      const m = event.payload;
      const at = performance.now();
      if (lastEventAtRef.current !== null) {
        deltasRef.current = [...deltasRef.current.slice(-(MEASURE_WINDOW - 1)), at - lastEventAtRef.current];
        const sum = deltasRef.current.reduce((a, b) => a + b, 0);
        setMeasuredIntervalMs(Math.round(sum / deltasRef.current.length));
      }
      lastEventAtRef.current = at;
      setMetrics(m);
      setCpuHistory((prev) => [...prev.slice(-(HISTORY_MAX - 1)), m.cpuUsage]);
      setRamHistory((prev) => [...prev.slice(-(HISTORY_MAX - 1)), m.ramPercent]);
      // run-10 : si des metriques arrivent et que l'utilisateur n'a pas mis en
      // pause, le monitoring tourne -> resynchronise le bouton ("Arreter").
      if (!pausedRef.current) setIsRunning(true);
    })
      .then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlistenFn = fn;
        }
      })
      .catch((e) => {
        if (!cancelled) setListenError(describeError(e));
      });
    return () => {
      cancelled = true;
      unlistenFn?.();
      startRequestedRef.current = false;
      invoke("stop_monitoring").catch(() => {});
    };
  }, []);

  const startBackend = useCallback(async () => {
    // Dedoublonne seulement un start deja demande sans stop entre-temps : le double
    // montage StrictMode envoie start, stop, start et le dernier doit passer.
    if (startRequestedRef.current) return;
    startRequestedRef.current = true;
    try {
      await invoke("start_monitoring", { intervalMs: intervalRef.current });
      setIsRunning(true);
      setError(null);
    } catch (e) {
      startRequestedRef.current = false;
      setError(describeError(e));
      setIsRunning(false);
    }
  }, []);

  const stopBackend = useCallback(async () => {
    startRequestedRef.current = false;
    try {
      await invoke("stop_monitoring");
      setIsRunning(false);
      setError(null);
    } catch (e) {
      setError(describeError(e));
    }
  }, []);

  const subscribe = useCallback(() => {
    refcount.current += 1;
    if (refcount.current === 1 && !pausedRef.current) {
      startBackend();
    }
  }, [startBackend]);

  const unsubscribe = useCallback(() => {
    refcount.current = Math.max(0, refcount.current - 1);
    if (refcount.current === 0) {
      stopBackend();
    }
  }, [stopBackend]);

  const resetMeasure = useCallback(() => {
    lastEventAtRef.current = null;
    deltasRef.current = [];
    setMeasuredIntervalMs(null);
  }, []);

  const pause = useCallback(async () => {
    pausedRef.current = true;
    resetMeasure();
    await stopBackend();
  }, [stopBackend, resetMeasure]);

  const resume = useCallback(async () => {
    pausedRef.current = false;
    await startBackend();
  }, [startBackend]);

  const setInterval = useCallback(async (value: number) => {
    const clamped = Math.min(MAX_INTERVAL_MS, Math.max(MIN_INTERVAL_MS, Math.round(value)));
    const previous = intervalRef.current;
    intervalRef.current = clamped;
    setIntervalMsState(clamped);
    setIntervalError(null);
    resetMeasure();
    try {
      await invoke<number>("set_monitor_interval", { intervalMs: clamped });
    } catch (e) {
      // Rollback du state local si le backend a echoue — evite un etat incoherent
      // (UI affiche X mais le backend tourne toujours a Y).
      intervalRef.current = previous;
      setIntervalMsState(previous);
      setIntervalError(describeError(e));
    }
  }, [resetMeasure]);

  const value = useMemo(
    () => ({ metrics, cpuHistory, ramHistory, isRunning, error: error ?? listenError, intervalMs, measuredIntervalMs, intervalError, setInterval, pause, resume, subscribe, unsubscribe }),
    [metrics, cpuHistory, ramHistory, isRunning, error, listenError, intervalMs, measuredIntervalMs, intervalError, setInterval, pause, resume, subscribe, unsubscribe],
  );

  return <MonitorContext.Provider value={value}>{children}</MonitorContext.Provider>;
}

export function useMonitor() {
  const ctx = useContext(MonitorContext);
  if (!ctx) throw new Error("useMonitor must be used within MonitorProvider");

  const { subscribe, unsubscribe } = ctx;
  useEffect(() => {
    subscribe();
    return () => unsubscribe();
  }, [subscribe, unsubscribe]);

  return ctx;
}
