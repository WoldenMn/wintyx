pub const KB: f64 = 1024.0;
pub const MB: f64 = 1_048_576.0;
pub const GB: f64 = 1_073_741_824.0;
pub const KB_TO_GB: f64 = 1_048_576.0;

pub const BYTES_KB: u64 = 1024;
pub const BYTES_MB: u64 = 1_048_576;

pub const TIMEOUT_SHORT: u64 = 5_000;
pub const TIMEOUT_MEDIUM: u64 = 10_000;
pub const TIMEOUT_LONG: u64 = 15_000;
pub const TIMEOUT_VERY_LONG: u64 = 30_000;
pub const TIMEOUT_EXTRA: u64 = 60_000;
pub const TIMEOUT_RESTORE: u64 = 120_000;
/// Delai maximum d'une requete WMI avant abandon (Winmgmt fige).
pub const TIMEOUT_WMI: u64 = TIMEOUT_EXTRA;

/// Seuil d'espace disque bas sur %LOCALAPPDATA% (auto-diagnostic v2.17.0).
/// En dessous, le check stockage passe en Warning (snapshots/logs a risque).
pub const DIAG_DISK_LOW_BYTES: u64 = 500 * 1024 * 1024; // 500 Mo

pub const TOP_PROCESSES_CONSUMERS: usize = 15;
pub const TOP_PROCESSES_MONITOR: usize = 10;

pub const MAX_LOG_FILE_BYTES: u64 = 10 * 1_048_576;
pub const LOG_RETENTION_DAYS: i64 = 7;

pub const CREATE_NO_WINDOW: u32 = 0x08000000;

pub const SYSTEM_INFO_CACHE_TTL_SECS: u64 = 60;
pub const SERVICES_SNAPSHOT_TTL_SECS: u64 = 3;

pub const DRIVER_OBSOLETE_THRESHOLD_DAYS: i64 = 365 * 2;

pub const REG_SERVICES: &str = r"HKLM\SYSTEM\CurrentControlSet\Services";
