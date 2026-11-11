// Types generes automatiquement depuis Rust via tauri-specta.
// NE PAS EDITER MANUELLEMENT — re-generer via `bun tauri dev` (mode debug exporte les bindings).

export * from "./bindings";

import type {
  HardeningCheck,
  NetworkAdapterInfo,
  WindowsUpdate,
  ScheduledTaskInfo,
  RegistryBackupInfo,
} from "./bindings";

// Alias retro-compatibles : les bindings Rust utilisent des noms differents
// (ex: HardeningCheck cote Rust, HardeningItem dans l'UI). On preserve les
// anciens noms pour eviter une migration cascade dans toutes les pages.
export type HardeningItem = HardeningCheck;
export type NetworkAdapter = NetworkAdapterInfo;
export type UpdateItem = WindowsUpdate;
export type TaskItem = ScheduledTaskInfo;
export type RegistryBackup = RegistryBackupInfo;

// SystemMetrics, ProcessMetric et ThreatScanProgress sont desormais generes dans
// bindings.ts (re-exportes ci-dessus via `export *`) : ils sont declares cote
// Rust avec #[derive(Event)] + #[tauri_specta(event_name = ...)] et collectes
// par tauri-specta (collect_events!, types-event). Plus de redefinition manuelle
// a synchroniser : la codegen est l'unique source de verite.

export type LevelFilter = "all" | "info" | "warn" | "error";
