// Timeout cote frontend pour invoke() : evite un PageLoader infini si le backend
// hang (deadlock WMI, PowerShell bloque). Au-dela, l'erreur Timeout est affichee
// (describeError) avec possibilite de reessayer (env-017).
//
// Un delai front inferieur au delai backend affiche "Timeout" alors que la
// commande aboutit encore : les commandes longues ont donc un delai >= celui du
// backend (WMI 60 s = TIMEOUT_WMI / TIMEOUT_EXTRA, restauration 120 s =
// TIMEOUT_RESTORE, cf. src-tauri/src/utils/constants.rs), avec une marge.
export const INVOKE_TIMEOUT_MS = 30_000;
export const INVOKE_TIMEOUT_BACKEND_60S_MS = 90_000;
export const INVOKE_TIMEOUT_BACKEND_120S_MS = 180_000;

const INVOKE_TIMEOUT_OVERRIDES_MS: Readonly<Record<string, number>> = {
  get_system_info: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_memory_sticks: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_memory_summary: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_memory_consumers: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_disks: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_network_adapters: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_driver_audit: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_driver_audit_summary: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_drivers: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_unsigned_drivers: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_problem_devices: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_driver_summary: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_app_diagnostics: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_scheduled_tasks: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_suspicious_tasks: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_update_history: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_restore_points: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_available_items: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_cleanup_targets: INVOKE_TIMEOUT_BACKEND_60S_MS,
  get_bloatware_items: INVOKE_TIMEOUT_BACKEND_60S_MS,
  run_threat_scan: INVOKE_TIMEOUT_BACKEND_120S_MS,
  run_benchmark: INVOKE_TIMEOUT_BACKEND_120S_MS,
  generate_report: INVOKE_TIMEOUT_BACKEND_120S_MS,
  create_pre_batch_snapshot: INVOKE_TIMEOUT_BACKEND_120S_MS,
};

export function invokeTimeoutFor(command: string): number {
  return INVOKE_TIMEOUT_OVERRIDES_MS[command] ?? INVOKE_TIMEOUT_MS;
}

// Seuils d'alerte des badges sidebar (UX-04 v2.16.0) : en dessous, un badge
// "!" signale le domaine sur son entree de navigation. Durcissement < 50 =
// warning ; Services < 30 = danger (plus rare, plus grave).
export const SIDEBAR_BADGE_HARDENING_THRESHOLD = 50;
export const SIDEBAR_BADGE_SERVICES_THRESHOLD = 30;

export const RISK_ORDER: Record<string, number> = {
  essential: 0,
  recommended: 1,
  critical: 0,
  high: 1,
  medium: 2,
  low: 3,
  safe: 4,
};

export function riskRank(level: string): number {
  return RISK_ORDER[level.toLowerCase()] ?? 99;
}

export const SEVERITY_STYLES: Record<string, string> = {
  critical: "bg-danger/10 text-danger",
  high: "bg-danger/20 text-danger",
  medium: "bg-warning/10 text-warning",
  low: "bg-fg-dim/10 text-fg-dim",
};

export const SEVERITY_LABELS_FR: Record<string, string> = {
  critical: "Critique",
  high: "Élevé",
  medium: "Moyen",
  low: "Faible",
  safe: "Sûr",
};

export function getSeverityLabel(severity: string): string {
  return SEVERITY_LABELS_FR[severity.toLowerCase()] ?? severity;
}

export const CATEGORY_STYLES: Record<string, string> = {
  gaming: "bg-danger/10 text-danger",
  social: "bg-info/10 text-info",
  media: "bg-warning/10 text-warning",
  microsoft: "bg-accent/10 text-accent",
};

// Libelles FR des categories backend (Bloatware + Durcissement). Les valeurs
// backend restent en anglais (cles stables, couplees a CATEGORY_STYLES et a la
// logique) ; seul l'affichage est francise (meme pattern que SEVERITY_LABELS_FR).
export const CATEGORY_LABELS_FR: Record<string, string> = {
  // Bloatware
  Entertainment: "Divertissement",
  Utility: "Utilitaire",
  System: "Système",
  Productivity: "Productivité",
  Network: "Réseau",
  Social: "Social",
  Communication: "Communication",
  Navigation: "Navigation",
  Gaming: "Jeux",
  // Durcissement
  "Remote Access": "Accès distant",
  Media: "Média",
  "Access Control": "Contrôle d'accès",
  Credentials: "Identifiants",
  Scripting: "Scripts",
  Auditing: "Audit",
  Services: "Services",
  "File Sharing": "Partage de fichiers",
  Accounts: "Comptes",
  // Protection des donnees (categories backend en francais non-accentue)
  Telemetrie: "Télémétrie",
  Publicite: "Publicité",
  Activite: "Activité",
  Reseau: "Réseau",
  Securite: "Sécurité",
  Energie: "Énergie",
  Demarrage: "Démarrage",
};

export function getCategoryLabel(category: string): string {
  return CATEGORY_LABELS_FR[category] ?? category;
}

// Niveau de durcissement backend ("Essential"/"Recommended" — cle couplee a la
// logique apply_hardening_essential et aux tests) -> libelle FR affiche.
export const HARDENING_LEVEL_LABELS_FR: Record<string, string> = {
  Essential: "Essentiel",
  Recommended: "Recommandé",
};

export function getHardeningLevelLabel(level: string): string {
  return HARDENING_LEVEL_LABELS_FR[level] ?? level;
}
