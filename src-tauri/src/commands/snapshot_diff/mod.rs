//! Comparateur de snapshots registre + restauration selective.
//!
//! Module decompose en 3 sous-modules par responsabilite :
//! - `parser` : decode des fichiers .reg UTF-16 LE + extraction timestamp + symlink filter
//! - `diff_engine` : comparaison de deux maps (sections, valeurs) + formattage
//! - `restore_engine` : parse target_value + application des changements registre
//!
//! Ce fichier expose les 4 commandes Tauri publiques (`diff_snapshot_vs_snapshot`,
//! `restore_selected_changes`, `list_snapshot_groups`, `diff_snapshot_vs_live`)
//! ainsi que les structs serialises (`SnapshotGroup`, `SnapshotDiff`, `RegistryDiff`,
//! `KeyValueChange`, `ChangeRequest`, `ChangeRequestType`).

mod diff_engine;
mod parser;
mod provenance;
mod restore_engine;
mod types;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::utils::batch::{BatchOptions, BatchResult};
use crate::utils::errors::CommandError;
use crate::utils::paths::backup_folder;
use crate::utils::{admin, registry};

const TIMESTAMP_LEN: usize = 15; // YYYYMMDD_HHMMSS
const MAX_CHANGES_PER_BATCH: usize = 50;

pub use types::{ChangeRequest, ChangeRequestType, KeyValueChange, RegistryDiff, SnapshotDiff, SnapshotGroup};

#[cfg(test)]
pub(crate) fn extract_timestamp_for_test(filename: &str) -> Option<String> {
    parser::extract_timestamp(filename)
}

#[tauri::command]
#[specta::specta]
pub async fn diff_snapshot_vs_snapshot(
    snapshot_a_timestamp: String,
    snapshot_b_timestamp: String,
) -> Result<SnapshotDiff, CommandError> {
    crate::utils::blocking::read_result(move || diff_snapshot_vs_snapshot_sync(snapshot_a_timestamp, snapshot_b_timestamp)).await
}

pub fn diff_snapshot_vs_snapshot_sync(
    snapshot_a_timestamp: String,
    snapshot_b_timestamp: String,
) -> Result<SnapshotDiff, CommandError> {
    crate::perf_timed!("diff_snapshot_vs_snapshot", {
        if snapshot_a_timestamp == snapshot_b_timestamp {
            return Err(CommandError::InvalidInput(
                "Les deux snapshots sont identiques".into(),
            ));
        }

        let folder = backup_folder();
        if !folder.exists() {
            return Err(CommandError::NotFound(format!(
                "Snapshot folder introuvable : {}",
                folder.display()
            )));
        }

        let load_snapshot = |timestamp: &str| -> HashMap<String, HashMap<String, String>> {
            let mut by_section: HashMap<String, HashMap<String, String>> = HashMap::new();
            for (path, ts) in parser::iter_reg_files(&folder) {
                if ts != timestamp {
                    continue;
                }
                let Some(content) = parser::decode_reg_file(&path) else {
                    continue;
                };
                for (section, name, raw_value) in parser::parse_reg_content(&content) {
                    by_section.entry(section).or_default().insert(name, raw_value);
                }
            }
            by_section
        };

        let a = load_snapshot(&snapshot_a_timestamp);
        let b = load_snapshot(&snapshot_b_timestamp);

        if a.is_empty() {
            return Err(CommandError::NotFound(format!(
                "Aucun fichier .reg pour timestamp A {}",
                snapshot_a_timestamp
            )));
        }
        if b.is_empty() {
            return Err(CommandError::NotFound(format!(
                "Aucun fichier .reg pour timestamp B {}",
                snapshot_b_timestamp
            )));
        }

        let mut diff = diff_engine::diff_two_maps(&a, &b);
        diff.snapshot_timestamp = snapshot_a_timestamp;
        diff.comparison_target = Some(snapshot_b_timestamp);
        diff.keys.sort_by(|x, y| x.key_path.cmp(&y.key_path));
        Ok(diff)
    })
}

#[tauri::command]
#[specta::specta]
pub async fn restore_selected_changes(
    changes: Vec<ChangeRequest>,
    options: Option<BatchOptions>,
) -> Result<BatchResult, CommandError> {
    crate::utils::blocking::write_result(move || restore_selected_changes_sync(changes, options)).await
}

pub fn restore_selected_changes_sync(
    changes: Vec<ChangeRequest>,
    options: Option<BatchOptions>,
) -> Result<BatchResult, CommandError> {
    if changes.is_empty() {
        return Err(CommandError::InvalidInput(
            "Aucun changement à restaurer".into(),
        ));
    }
    if changes.len() > MAX_CHANGES_PER_BATCH {
        return Err(CommandError::InvalidInput(format!(
            "Max {} changements par batch (reçu : {})",
            MAX_CHANGES_PER_BATCH,
            changes.len()
        )));
    }

    for change in &changes {
        if !registry::is_allowed_key_path(&change.key_path) {
            return Err(CommandError::InvalidInput(format!(
                "Chemin registre non autorisé : {}",
                change.key_path
            )));
        }
        registry::sanitize_value_name(&change.value_name)
            .map_err(CommandError::InvalidInput)?;
        match change.change_type {
            ChangeRequestType::RestoreModified | ChangeRequestType::RestoreRemoved => {
                if change.target_value.is_none() {
                    return Err(CommandError::InvalidInput(
                        "target_value requis pour ce type de changement".into(),
                    ));
                }
            }
            ChangeRequestType::UndoAdded => {}
        }
    }

    let snapshots = provenance::load_snapshots(&backup_folder());
    restore_verified(changes, options, &snapshots)
}

fn restore_verified(
    changes: Vec<ChangeRequest>,
    options: Option<BatchOptions>,
    snapshots: &[provenance::RecordedValues],
) -> Result<BatchResult, CommandError> {
    provenance::verify_changes(&changes, snapshots)?;

    let opts = options.unwrap_or_default();
    let dry_run = opts.dry_run;

    // Choix produit assume (cf. SECURITY.md, finding accepte "Dry-run bypass de
    // require_admin") : le dry-run s'execute volontairement AVANT le gate admin
    // pour qu'un utilisateur non eleve puisse previsualiser les changements.
    // Il ne retourne que des descriptions (describe_change) de requetes deja
    // validees par l'allowlist + sanitize ci-dessus, et n'ecrit jamais.
    if dry_run {
        let mut result = BatchResult::new_dry_run();
        for change in &changes {
            result.record_preview(diff_engine::describe_change(change));
        }
        return Ok(result);
    }

    // Admin gate : nécessaire si au moins un change touche HKLM.
    // Comparaison case-insensitive (cf. registry::is_hklm_path) car le
    // registre Windows est case-insensitive et un attaquant ne doit pas
    // pouvoir bypasser ce gate via une variation de casse.
    let touches_hklm = changes.iter().any(|c| registry::is_hklm_path(&c.key_path));
    if touches_hklm {
        admin::require_admin("Restaurer des modifications registre")?;
    }

    let mut result = BatchResult::new();
    for change in &changes {
        let label = diff_engine::describe_change(change);
        result.record(label, restore_engine::apply_single_change(change));
    }
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn list_snapshot_groups() -> Vec<SnapshotGroup> {
    crate::utils::blocking::read_value(list_snapshot_groups_sync).await
}

pub fn list_snapshot_groups_sync() -> Vec<SnapshotGroup> {
    let folder = backup_folder();
    if !folder.exists() {
        return Vec::new();
    }

    let mut by_timestamp: HashMap<String, Vec<String>> = HashMap::new();
    for (path, ts) in parser::iter_reg_files(&folder) {
        by_timestamp
            .entry(ts)
            .or_default()
            .push(path.to_string_lossy().to_string());
    }

    let mut groups: Vec<SnapshotGroup> = by_timestamp
        .into_iter()
        .map(|(timestamp, files)| SnapshotGroup {
            timestamp,
            key_count: files.len(),
            backup_files: files,
        })
        .collect();

    groups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    groups
}

#[tauri::command]
#[specta::specta]
pub async fn diff_snapshot_vs_live(snapshot_timestamp: String) -> Result<SnapshotDiff, CommandError> {
    crate::utils::blocking::read_result(move || diff_snapshot_vs_live_sync(snapshot_timestamp)).await
}

pub fn diff_snapshot_vs_live_sync(snapshot_timestamp: String) -> Result<SnapshotDiff, CommandError> {
    crate::perf_timed!("diff_snapshot_vs_live", {
        let folder = backup_folder();
        if !folder.exists() {
            return Err(CommandError::NotFound(format!(
                "Snapshot folder introuvable : {}",
                folder.display()
            )));
        }

        let matching_files: Vec<PathBuf> = parser::iter_reg_files(&folder)
            .into_iter()
            .filter(|(_, ts)| ts == &snapshot_timestamp)
            .map(|(path, _)| path)
            .collect();

        if matching_files.is_empty() {
            return Err(CommandError::NotFound(format!(
                "Aucun fichier .reg pour timestamp {}",
                snapshot_timestamp
            )));
        }

        let mut snapshot_by_section: HashMap<String, HashMap<String, String>> = HashMap::new();
        for file in &matching_files {
            let Some(content) = parser::decode_reg_file(file) else {
                continue;
            };
            for (section, name, raw_value) in parser::parse_reg_content(&content) {
                snapshot_by_section
                    .entry(section)
                    .or_default()
                    .insert(name, raw_value);
            }
        }

        Ok(diff_engine::diff_snapshot_against_live(
            snapshot_by_section,
            snapshot_timestamp,
        ))
    })
}

#[cfg(test)]
mod tests;
