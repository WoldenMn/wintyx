use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use crate::utils::{admin, cleanup_helpers, constants, errors::CommandError, logger, process};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CleanupTarget {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub size_bytes: u64,
    pub size_display: String,
    pub is_system: bool,
    pub accessible: bool,
}

mod sizes;
use sizes::{get_folder_size, get_recycle_bin_size};

const INACCESSIBLE_LABEL: &str = "Inaccessible sans droits administrateur";

/// Cibles que `clean_all` traite, en simulation comme en reel : accessibles et non vides.
fn is_cleanable(target: &CleanupTarget) -> bool {
    target.accessible && target.size_bytes > 0
}

fn ensure_rights(target: &CleanupTarget, is_admin: bool) -> Result<(), CommandError> {
    if target.is_system && !is_admin {
        return Err(CommandError::NotAdmin(format!("Nettoyer {}", target.name)));
    }
    Ok(())
}

fn select_cleanable(
    targets: &[(CleanupTarget, String)],
    is_admin: bool,
) -> (Vec<&CleanupTarget>, Vec<String>) {
    let mut selected = Vec::new();
    let mut skipped = Vec::new();
    for (target, _) in targets.iter().filter(|(t, _)| is_cleanable(t)) {
        if target.is_system && !is_admin {
            skipped.push(format!("{} : droits administrateur requis, cible ignorée", target.name));
        } else {
            selected.push(target);
        }
    }
    (selected, skipped)
}

fn get_targets_internal() -> Vec<(CleanupTarget, String)> {
    let temp_path = std::env::temp_dir().to_string_lossy().to_string();
    let win_dir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".to_string());
    let win_temp = format!(r"{}\Temp", win_dir);
    let prefetch = format!(r"{}\Prefetch", win_dir);
    let software_dist = format!(r"{}\SoftwareDistribution\Download", win_dir);
    let recycle_bin = PathBuf::from(std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string())).join("$Recycle.Bin").to_string_lossy().to_string();
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let thumb_cache = format!(r"{}\Microsoft\Windows\Explorer", local_app_data);

    let defs: Vec<(&str, &str, &str, String, &str, bool)> = vec![
        ("UserTemp", "Fichiers temporaires utilisateur", "Fichiers temporaires du profil utilisateur", temp_path.clone(), "*", false),
        ("WinTemp", "Fichiers temporaires Windows", "Fichiers temporaires système", win_temp, "*", true),
        ("Prefetch", "Données Prefetch", "Fichiers de cache Prefetch des applications", prefetch, "*.pf", true),
        ("SoftwareDist", "Cache Windows Update", "Fichiers Windows Update téléchargés", software_dist, "*", true),
        ("RecycleBin", "Corbeille", "Fichiers supprimés de la corbeille", recycle_bin, "*", false),
        ("ThumbCache", "Cache des miniatures", "Base de données des miniatures de l'Explorateur", thumb_cache, "thumbcache_*.db", false),
    ];

    defs.into_iter()
        .map(|(id, name, desc, path, pattern, is_system)| {
            let measured = if id == "RecycleBin" {
                get_recycle_bin_size()
            } else {
                get_folder_size(&path, pattern)
            };
            let accessible = measured.is_some();
            let size = measured.unwrap_or(0);
            let target = CleanupTarget {
                id: id.to_string(),
                name: name.to_string(),
                description: desc.to_string(),
                path: path.clone(),
                size_bytes: size,
                size_display: if accessible {
                    cleanup_helpers::format_size(size)
                } else {
                    INACCESSIBLE_LABEL.to_string()
                },
                is_system,
                accessible,
            };
            (target, pattern.to_string())
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub async fn get_cleanup_targets() -> Vec<CleanupTarget> {
    crate::utils::blocking::read_value(get_cleanup_targets_sync).await
}

pub fn get_cleanup_targets_sync() -> Vec<CleanupTarget> {
    let mut targets: Vec<CleanupTarget> = get_targets_internal().into_iter().map(|(t, _)| t).collect();
    targets.sort_by_key(|t| std::cmp::Reverse(t.size_bytes));
    targets
}

#[derive(Debug, Clone, Serialize, specta::Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub freed_bytes: u64,
    pub errors: Vec<String>,
    // Champs requis (toujours produits par le backend via ..Default::default()) :
    // alignes sur BatchResult, sans #[serde(default)] -> typage TS strict.
    pub dry_run: bool,
    pub preview_items: Vec<String>,
}

#[tauri::command]
#[specta::specta]
pub async fn clean_target(id: String) -> Result<CleanResult, CommandError> {
    crate::utils::blocking::write_result(move || clean_target_sync(id)).await
}

pub fn clean_target_sync(id: String) -> Result<CleanResult, CommandError> {
    let targets = get_targets_internal();
    let (target, pattern) = match targets.iter().find(|(t, _)| t.id == id) {
        Some((t, p)) => (t.clone(), p.clone()),
        None => return Err(CommandError::NotFound(format!("Cible nettoyage : {}", id))),
    };

    ensure_rights(&target, admin::is_admin())?;

    if target.id == "RecycleBin" {
        let mut errors: Vec<String> = Vec::new();
        let before = get_recycle_bin_size().unwrap_or(0);
        if !clean_recycle_bin() {
            logger::warn("cleanup", "clean_target RecycleBin: echec Clear-RecycleBin");
            errors.push("Échec Clear-RecycleBin".to_string());
        }
        let after = get_recycle_bin_size().unwrap_or(before);
        return Ok(CleanResult {
            freed_bytes: before.saturating_sub(after),
            errors,
            ..Default::default()
        });
    }

    let path = PathBuf::from(&target.path);
    if !path.exists() {
        return Ok(CleanResult::default());
    }

    let before = get_folder_size(&target.path, &pattern).unwrap_or(0);

    let errors = match clean_directory(&path, &pattern) {
        Ok(failures) => failures,
        Err(e) => {
            logger::warn(
                "cleanup",
                &format!("clean_target {}: lecture impossible de {}: {}", target.id, target.path, e),
            );
            return Err(read_dir_error(&target.name, &e));
        }
    };

    let after = get_folder_size(&target.path, &pattern).unwrap_or(before);
    if !errors.is_empty() {
        logger::warn(
            "cleanup",
            &format!(
                "clean_target {}: {} élément(s) non supprimé(s), ex: {}",
                target.id,
                errors.len(),
                errors.first().map(|s| s.as_str()).unwrap_or("")
            ),
        );
    }
    Ok(CleanResult {
        freed_bytes: before.saturating_sub(after),
        errors,
        ..Default::default()
    })
}

fn read_dir_error(name: &str, e: &std::io::Error) -> CommandError {
    if e.kind() == std::io::ErrorKind::PermissionDenied {
        CommandError::PermissionDenied(format!("{} : lecture du dossier refusée", name))
    } else {
        CommandError::Generic(format!("{} : lecture du dossier impossible ({})", name, e))
    }
}

fn clean_directory(path: &std::path::Path, pattern: &str) -> std::io::Result<Vec<String>> {
    let mut errors = Vec::new();
    for entry in fs::read_dir(path)?.flatten() {
        let entry_path = entry.path();
        if pattern == "*" {
            let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
            let result = if is_dir {
                fs::remove_dir_all(&entry_path)
            } else {
                fs::remove_file(&entry_path)
            };
            if let Err(e) = result {
                errors.push(format!("{}: {}", entry_path.display(), e));
            }
        } else {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if cleanup_helpers::matches_pattern(&name, pattern) {
                if let Err(e) = fs::remove_file(&entry_path) {
                    errors.push(format!("{}: {}", entry_path.display(), e));
                }
            }
        }
    }
    Ok(errors)
}

fn clean_recycle_bin() -> bool {
    process::run_silent(
        "powershell.exe",
        "-NoProfile -Command \"Clear-RecycleBin -Force -ErrorAction SilentlyContinue\"",
        constants::TIMEOUT_LONG,
    )
}

#[tauri::command]
#[specta::specta]
pub async fn clean_all(options: Option<crate::utils::batch::BatchOptions>) -> Result<CleanResult, CommandError> {
    crate::utils::blocking::write_result(move || clean_all_sync(options)).await
}

pub fn clean_all_sync(options: Option<crate::utils::batch::BatchOptions>) -> Result<CleanResult, CommandError> {
    crate::perf_timed!("clean_all", {
        let targets = get_targets_internal();
        let opts = options.unwrap_or_default();
        let dry = opts.dry_run;
        let (selected, skipped) = select_cleanable(&targets, admin::is_admin());
        if dry {
            let mut total = CleanResult {
                dry_run: true,
                errors: skipped,
                ..Default::default()
            };
            for target in selected {
                total.freed_bytes = total.freed_bytes.saturating_add(target.size_bytes);
                total
                    .preview_items
                    .push(format!("{} ({})", target.name, target.size_display));
            }
            return Ok(total);
        }
        let mut total = CleanResult {
            errors: skipped,
            ..Default::default()
        };
        for target in selected {
            match clean_target_sync(target.id.clone()) {
                Ok(r) => {
                    total.freed_bytes = total.freed_bytes.saturating_add(r.freed_bytes);
                    total.errors.extend(r.errors);
                }
                Err(e) => {
                    logger::warn("cleanup", &format!("Echec nettoyage {}: {}", target.id, e));
                    total.errors.push(format!("{}: {}", target.name, e));
                }
            }
        }
        Ok(total)
    })
}

#[tauri::command]
#[specta::specta]
pub async fn flush_dns() -> Result<(), CommandError> {
    crate::utils::blocking::write_result(flush_dns_sync).await
}

pub fn flush_dns_sync() -> Result<(), CommandError> {
    if !process::run_silent("ipconfig", "/flushdns", constants::TIMEOUT_MEDIUM) {
        logger::warn("cleanup", "flush_dns: echec ipconfig /flushdns");
        return Err(CommandError::ProcessFailed {
            command: "ipconfig /flushdns".into(),
            detail: "La commande ipconfig /flushdns a échoué".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
