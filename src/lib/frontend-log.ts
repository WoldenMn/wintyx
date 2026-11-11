import { invoke } from "@tauri-apps/api/core";
import { describeError } from "./helpers";

export type FrontendLogLevel = "warn" | "error";

const DEDUPE_WINDOW_MS = 1000;

let lastMessage = "";
let lastAt = 0;

export function resetFrontendLogDedupe() {
  lastMessage = "";
  lastAt = 0;
}

function textOf(error: unknown): string {
  return error instanceof Error ? error.message : describeError(error);
}

function logFailure(e: unknown) {
  console.warn("[frontend-log] journalisation impossible :", e);
}

export function reportFrontendError(
  source: string,
  error: unknown,
  level: FrontendLogLevel = "error",
): void {
  const message = `${source} : ${textOf(error)}`;
  const now = Date.now();
  if (message === lastMessage && now - lastAt < DEDUPE_WINDOW_MS) return;
  lastMessage = message;
  lastAt = now;
  try {
    Promise.resolve(invoke("log_frontend_error", { level, message })).catch(logFailure);
  } catch (e) {
    logFailure(e);
  }
}
