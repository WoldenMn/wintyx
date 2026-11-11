//! Sauvegarde / restauration : commandes Tauri (centralisees ici, pattern projet)
//! orchestrant 4 sous-domaines decomposes par responsabilite :
//! - [`restore_point`] : points de restauration systeme (lecture, creation verifiee)
//! - [`registry_backup`] : nommage et lots d'export de cles registre (.reg)
//! - [`snapshot`] : snapshot pre-batch (restore point + backup registre)
//! - system protection / dossier de sauvegarde (helpers ci-dessous)

use std::fs;
use std::path::PathBuf;
use crate::utils::batch::BatchResult;
use crate::utils::errors::CommandError;
use crate::utils::paths::backup_folder;
use crate::utils::{admin, constants, logger, process};

mod registry_backup;
mod reg_guard;
mod registry_export;
mod restore_point;
mod snapshot;
mod system_protection;

use registry_backup::{batch_timestamp, hive_name_from_stem, RegistryBackupInfo};
use registry_export::export_critical_keys;
use restore_point::{RestorePointInfo, RestorePointOutcome};
use snapshot::{restore_point_state, truncate_action_name, SnapshotInfo};

pub use system_protection::PolicyState;

fn ensure_backup_folder() -> Result<(), String> {
    let folder = backup_folder();
    if !folder.exists() {
        fs::create_dir_all(&folder).map_err(|e| format!("Création du dossier de sauvegarde impossible : {}", e))?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_restore_points() -> Result<Vec<RestorePointInfo>, CommandError> {
    crate::utils::blocking::read_result(get_restore_points_sync).await
}

pub fn get_restore_points_sync() -> Result<Vec<RestorePointInfo>, CommandError> {
    let result = process::run(
        "powershell.exe",
        "-NoProfile -Command \"Get-ComputerRestorePoint | Select-Object Description, CreationTime, SequenceNumber, RestorePointType | ConvertTo-Json\"",
        constants::TIMEOUT_VERY_LONG,
    );

    if !result.success {
        logger::warn(
            "backup.lecture",
            &format!("get_restore_points: échec Get-ComputerRestorePoint (exit {}): {}", result.exit_code, result.stderr),
        );
        return Err(CommandError::ProcessFailed {
            command: "Get-ComputerRestorePoint".into(),
            detail: if result.stderr.trim().is_empty() {
                format!("code de sortie {}", result.exit_code)
            } else {
                result.stderr.trim().to_string()
            },
        });
    }

    restore_point::parse_restore_points(&result.stdout)
}

#[tauri::command]
#[specta::specta]
pub async fn create_restore_point(description: String) -> Result<RestorePointOutcome, CommandError> {
    crate::utils::blocking::write_result(move || create_restore_point_sync(description)).await
}

pub fn create_restore_point_sync(description: String) -> Result<RestorePointOutcome, CommandError> {
    admin::require_admin("Créer un point de restauration")?;
    restore_point::create(&description)
}

#[tauri::command]
#[specta::specta]
pub async fn backup_critical_keys() -> BatchResult {
    crate::utils::blocking::write_value(backup_critical_keys_sync).await
}

pub fn backup_critical_keys_sync() -> BatchResult {
    export_critical_keys(&batch_timestamp(chrono::Local::now()))
}

#[tauri::command]
#[specta::specta]
pub async fn create_pre_batch_snapshot(action_name: String) -> SnapshotInfo {
    crate::utils::blocking::write_value(move || create_pre_batch_snapshot_sync(action_name)).await
}

pub fn create_pre_batch_snapshot_sync(action_name: String) -> SnapshotInfo {
    let safe_action = truncate_action_name(&action_name, 80);
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let (restore_point_created, restore_point_limited) = restore_point_state(admin::is_admin(), || {
        restore_point::create(&format!("Wintyx - avant {}", safe_action))
    });

    let registry_backups = export_critical_keys(&batch_timestamp(now));

    logger::info(
        "backup",
        &format!(
            "Snapshot '{}': restore_point={}, limite_windows={}, backups={}/{}",
            safe_action,
            restore_point_created,
            restore_point_limited,
            registry_backups.success,
            registry_backups.total
        ),
    );

    SnapshotInfo {
        restore_point_created,
        restore_point_limited,
        registry_backups,
        timestamp,
        action_name: safe_action,
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_registry_backups() -> Vec<RegistryBackupInfo> {
    crate::utils::blocking::read_value(get_registry_backups_sync).await
}

pub fn get_registry_backups_sync() -> Vec<RegistryBackupInfo> {
    let folder = backup_folder();
    if !folder.exists() {
        return Vec::new();
    }

    let mut backups = Vec::new();
    if let Ok(entries) = fs::read_dir(&folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "reg").unwrap_or(false) {
                if let Ok(meta) = entry.metadata() {
                    let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                    let hive_name = hive_name_from_stem(&name);

                    let backup_time = meta.created()
                        .ok()
                        .map(|t| {
                            let dt: chrono::DateTime<chrono::Local> = t.into();
                            dt.format("%Y-%m-%d %H:%M:%S").to_string()
                        })
                        .unwrap_or_default();

                    backups.push(RegistryBackupInfo {
                        file_path: path.to_string_lossy().to_string(),
                        hive_name,
                        backup_time,
                        file_size_bytes: meta.len(),
                    });
                }
            }
        }
    }

    backups.sort_by(|a, b| b.backup_time.cmp(&a.backup_time));
    backups
}

#[tauri::command]
#[specta::specta]
pub async fn restore_registry_backup(backup_path: String) -> Result<bool, CommandError> {
    crate::utils::blocking::write_result(move || restore_registry_backup_sync(backup_path)).await
}

pub fn restore_registry_backup_sync(backup_path: String) -> Result<bool, CommandError> {
    admin::require_admin("Restaurer une sauvegarde registre")?;
    let path = PathBuf::from(&backup_path);
    if !path.exists() {
        return Err(CommandError::NotFound(format!("Fichier de sauvegarde : {}", backup_path)));
    }

    let metadata = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err(CommandError::InvalidInput("Les symlinks ne sont pas autorisés".into()));
    }

    let canonical = std::fs::canonicalize(&path).map_err(|e| e.to_string())?;
    let canonical_backup_dir = std::fs::canonicalize(backup_folder()).map_err(|e| e.to_string())?;
    if !canonical.starts_with(&canonical_backup_dir) {
        return Err(CommandError::InvalidInput(
            "Chemin en dehors du dossier de sauvegarde".into(),
        ));
    }
    if canonical.extension().and_then(|e| e.to_str()) != Some("reg") {
        return Err(CommandError::InvalidInput(
            "Le fichier doit être un .reg".into(),
        ));
    }

    let _verified = reg_guard::open_verified(&canonical)?;

    let result = process::run(
        "reg.exe",
        &format!("import \"{}\"", canonical.to_string_lossy()),
        constants::TIMEOUT_EXTRA,
    );
    if !result.success {
        logger::error(
            "backup",
            &format!(
                "restore_registry_backup: échec reg import (exit {}): {}",
                result.exit_code, result.stderr
            ),
        );
        return Err(CommandError::ProcessFailed {
            command: "reg.exe import".into(),
            detail: if result.stderr.trim().is_empty() {
                format!("code de sortie {}", result.exit_code)
            } else {
                result.stderr.trim().to_string()
            },
        });
    }
    logger::info(
        "backup",
        &format!("Backup registre restaure: {}", canonical.to_string_lossy()),
    );
    Ok(true)
}

#[tauri::command]
#[specta::specta]
pub fn is_system_protection_enabled() -> Result<bool, CommandError> {
    system_protection::is_enabled()
}

pub fn system_protection_policy() -> Result<PolicyState, CommandError> {
    system_protection::policy_state()
}

#[tauri::command]
#[specta::specta]
pub fn get_backup_folder() -> Result<String, CommandError> {
    ensure_backup_folder()?;
    Ok(backup_folder().to_string_lossy().to_string())
}

#[tauri::command]
#[specta::specta]
pub fn open_backup_folder() -> Result<(), CommandError> {
    use std::os::windows::process::CommandExt;
    ensure_backup_folder()?;
    let folder = backup_folder();
    let explorer = process::resolve_system_program("explorer.exe")
        .ok_or_else(|| "Explorateur Windows introuvable".to_string())?;
    std::process::Command::new(explorer)
        .arg(&folder)
        .creation_flags(constants::CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Impossible d'ouvrir le dossier: {}", e))?;
    Ok(())
}
