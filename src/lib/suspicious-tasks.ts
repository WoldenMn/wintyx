export const SUSPICIOUS_TASKS_CHANGED_EVENT = "wintyx:suspicious-tasks-changed";

export function notifySuspiciousTasksChanged() {
  window.dispatchEvent(new Event(SUSPICIOUS_TASKS_CHANGED_EVENT));
}
