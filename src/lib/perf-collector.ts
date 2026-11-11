import { invoke } from "@tauri-apps/api/core";

export type FrontendSample = {
  ts: number;
  cmd: string;
  argsKey: string;
  durationMs: number;
  cacheHit: boolean;
};

const buffer: FrontendSample[] = [];
const FLUSH_INTERVAL_MS = 5000;
const MAX_BUFFER = 500;

export const PERF_ENABLED =
  import.meta.env.DEV ||
  ((import.meta as unknown as { env: Record<string, string> }).env.VITE_PERF === "1");

export function recordSample(s: FrontendSample) {
  buffer.push(s);
  if (buffer.length >= MAX_BUFFER) {
    void flush();
  }
}

let flushInFlight = false;

async function flush() {
  if (flushInFlight) return;
  if (buffer.length === 0) return;
  flushInFlight = true;
  const batch = buffer.slice();
  buffer.length = 0;
  try {
    await invoke("perf_dump_frontend", { samples: batch });
  } catch {
    // Best-effort
  } finally {
    flushInFlight = false;
  }
}

if (PERF_ENABLED && typeof window !== "undefined") {
  const flushTimer = setInterval(flush, FLUSH_INTERVAL_MS);
  window.addEventListener("beforeunload", () => {
    void flush();
  });
  // HMR (dev) : ce bloc module-scope est reexecute a chaque rechargement de
  // module. Sans cleanup, les setInterval s'accumulent (fuite + flushes
  // dupliques). On nettoie le timer au dispose du module (front-01). En build
  // de production, import.meta.hot est undefined : aucun effet.
  if (import.meta.hot) {
    import.meta.hot.dispose(() => clearInterval(flushTimer));
  }
}

// Helpers test-only
export function _bufferSizeForTests() {
  return buffer.length;
}
export async function _flushForTests() {
  await flush();
}
export function _resetBufferForTests() {
  buffer.length = 0;
  flushInFlight = false;
}
export function _flushInFlightForTests() {
  return flushInFlight;
}
