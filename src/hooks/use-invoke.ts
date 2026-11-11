import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { recordSample, PERF_ENABLED } from "../lib/perf-collector";
import { describeError } from "../lib/helpers";
import { reportFrontendError } from "../lib/frontend-log";
import { invokeTimeoutFor } from "../lib/constants";
import type { CommandError } from "../lib/types";
import type { CommandName } from "../lib/command-names";

const CACHE = new Map<string, unknown>();
const MAX_CACHE_SIZE = 50;

function cacheSet(key: string, value: unknown) {
  if (CACHE.has(key)) CACHE.delete(key);
  CACHE.set(key, value);
  if (CACHE.size > MAX_CACHE_SIZE) {
    const oldest = CACHE.keys().next().value;
    if (oldest !== undefined) CACHE.delete(oldest);
  }
}

export function invalidateCache(commandPattern?: string) {
  if (!commandPattern) {
    CACHE.clear();
    return;
  }
  const prefix = `${commandPattern}:`;
  for (const key of Array.from(CACHE.keys())) {
    if (key.startsWith(prefix)) CACHE.delete(key);
  }
}

export function getCacheSize() {
  return CACHE.size;
}

/**
 * Lance l'invocation d'une commande en tache de fond et place le resultat
 * dans le cache. Sans exception remontee (erreurs ignorees silencieusement).
 *
 * Cas d'usage : au mount de l'App, prefetcher les 2-3 pages les plus visitees
 * pour qu'elles s'affichent instantanement au premier clic utilisateur.
 */
export function prefetchCommand(
  command: CommandName,
  args?: Record<string, unknown>,
): void {
  const cacheKey = `${command}:${args ? JSON.stringify(args) : ""}`;
  if (CACHE.has(cacheKey)) return;
  const start = PERF_ENABLED ? performance.now() : 0;
  invoke(command, args ?? {})
    .then((value) => {
      cacheSet(cacheKey, value);
      if (PERF_ENABLED) {
        recordSample({
          ts: Date.now(),
          cmd: command,
          argsKey: args ? JSON.stringify(args) : "",
          durationMs: performance.now() - start,
          cacheHit: false,
        });
      }
    })
    .catch(() => {
      // Prefetch silencieux : ne rien faire en cas d'echec
    });
}

export function useInvoke<T>(command: CommandName, args?: Record<string, unknown>) {
  // H4 perf : memoise le JSON.stringify pour eviter le recalcul a chaque render
  // sur les pages lourdes qui appellent useInvoke depuis plusieurs composants.
  const argsKey = useMemo(() => (args ? JSON.stringify(args) : ""), [args]);
  const cacheKey = `${command}:${argsKey}`;

  const [data, setData] = useState<T | null>(() => {
    const hit = CACHE.get(cacheKey);
    return hit === undefined ? null : (hit as T);
  });
  const [isLoading, setIsLoading] = useState(() => !CACHE.has(cacheKey));
  const [error, setError] = useState<string | null>(null);

  const argsRef = useRef(args);
  argsRef.current = args;
  const mountedRef = useRef(true);
  // Jeton de sequence : seule la derniere requete emise peut committer son
  // resultat (state + cache), pour qu'une reponse lente obsolete n'ecrase pas
  // une valeur fraiche ni n'empoisonne le cache (bug-frontend-01).
  const seqRef = useRef(0);
  const timeoutIdRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      // Nettoie un timer de timeout encore en vol si on demonte avant la reponse.
      if (timeoutIdRef.current !== undefined) clearTimeout(timeoutIdRef.current);
    };
  }, []);

  const refresh = useCallback(() => {
    setError(null);
    if (!CACHE.has(cacheKey)) setIsLoading(true);
    const reqId = ++seqRef.current;
    const start = PERF_ENABLED ? performance.now() : 0;
    // Timeout cote frontend : si le backend ne repond pas (deadlock WMI,
    // PowerShell bloque), on rejette pour ne pas laisser isLoading=true a vie
    // (env-017). L'erreur Timeout est mise en forme par describeError.
    const timeoutMs = invokeTimeoutFor(command);
    let timeoutId: ReturnType<typeof setTimeout> | undefined;
    const timeoutPromise = new Promise<never>((_, reject) => {
      timeoutId = setTimeout(() => {
        const err: CommandError = {
          code: "Timeout",
          details: { command, timeoutMs },
        };
        reject(err);
      }, timeoutMs);
      timeoutIdRef.current = timeoutId;
    });
    Promise.race([invoke<T>(command, argsRef.current ?? {}), timeoutPromise])
      .then((value) => {
        if (timeoutId !== undefined) clearTimeout(timeoutId);
        // Reponse obsolete (une requete plus recente a ete emise) : ne rien
        // committer, ni dans le state ni dans le cache.
        if (reqId !== seqRef.current) return;
        cacheSet(cacheKey, value as T);
        if (PERF_ENABLED) {
          recordSample({
            ts: Date.now(),
            cmd: command,
            argsKey,
            durationMs: performance.now() - start,
            cacheHit: false,
          });
        }
        if (!mountedRef.current) return;
        setData(value as T);
        setIsLoading(false);
      })
      .catch((e) => {
        if (timeoutId !== undefined) clearTimeout(timeoutId);
        if (reqId !== seqRef.current || !mountedRef.current) return;
        reportFrontendError(`commande ${command}`, e);
        setError(describeError(e));
        setIsLoading(false);
      });
  }, [command, cacheKey, argsKey]);

  useEffect(() => {
    const hit = CACHE.get(cacheKey);
    if (hit !== undefined) {
      setData(hit as T);
      setIsLoading(false);
      if (PERF_ENABLED) {
        recordSample({
          ts: Date.now(),
          cmd: command,
          argsKey,
          durationMs: 0,
          cacheHit: true,
        });
      }
    } else {
      setData(null);
      setIsLoading(true);
    }
    setError(null);
    refresh();
  }, [cacheKey, refresh, command, argsKey]);

  return { data, isLoading, error, refresh };
}
