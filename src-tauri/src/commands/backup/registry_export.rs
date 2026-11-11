use crate::utils::batch::BatchResult;
use crate::utils::errors::CommandError;
use crate::utils::paths::backup_folder;
use crate::utils::{admin, constants, logger, process, registry};

use super::ensure_backup_folder;
use super::registry_backup::{export_file_name, export_keys};

pub(super) fn export_registry_key(key_path: &str, timestamp: &str) -> Result<String, CommandError> {
    if registry::is_hklm_path(key_path) {
        admin::require_admin("Exporter une cle HKLM")?;
    }
    if !registry::is_allowed_key_path(key_path) {
        return Err(CommandError::InvalidInput("Chemin registre non autorisé".into()));
    }
    registry::sanitize_key_path(key_path).map_err(CommandError::InvalidInput)?;

    ensure_backup_folder()?;
    let filename = export_file_name(key_path, timestamp, chrono::Local::now().timestamp_subsec_millis());
    let file_path = backup_folder().join(&filename);

    let result = process::run(
        "reg.exe",
        &format!("export \"{}\" \"{}\" /y", key_path, file_path.to_string_lossy()),
        constants::TIMEOUT_EXTRA,
    );

    if result.success && file_path.exists() {
        logger::info("backup", &format!("Registre exporte: {}", key_path));
        Ok(file_path.to_string_lossy().to_string())
    } else {
        logger::error("backup", &format!("Echec export registre: {}", result.stderr));
        Err(CommandError::ProcessFailed {
            command: format!("reg.exe export {}", key_path),
            detail: result.stderr.trim().to_string(),
        })
    }
}

pub(super) const CRITICAL_KEYS: [&str; 7] = [
    r"HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection",
    r"HKLM\SOFTWARE\Policies\Microsoft\Windows\System",
    r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
    r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
    r"HKLM\SYSTEM\CurrentControlSet\Services\LanmanServer\Parameters",
    r"HKLM\SYSTEM\CurrentControlSet\Control\Terminal Server",
    r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System",
];

pub(super) fn export_critical_keys(timestamp: &str) -> BatchResult {
    export_keys(&CRITICAL_KEYS, timestamp, export_registry_key)
}
