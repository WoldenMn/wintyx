//! Moteur de diff entre deux snapshots registre (ou snapshot vs etat live).
//!
//! Compare les paires nom/valeur, classifie les changements en
//! Added/Removed/Modified, formate la valeur live au format raw .reg
//! pour comparaison directe, et formate la description textuelle.

use std::collections::{BTreeSet, HashMap};

use crate::utils::registry;

use super::{ChangeRequest, ChangeRequestType, KeyValueChange, RegistryDiff, SnapshotDiff};

pub(super) fn format_live_value_dword(v: u32) -> String {
    format!("dword:{:08x}", v)
}

pub(super) fn format_live_value_string(v: &str) -> String {
    format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Lit la valeur live au format raw .reg pour comparaison directe avec
/// la valeur stockee dans le snapshot.
///
/// Le hint `snapshot_raw` (prefix `dword:` ou `"`) determine le type a tenter
/// en priorite, avec fallback inverse si la cle n'existe pas dans ce type.
pub(super) fn read_live_value(key_path: &str, name: &str, snapshot_raw: &str) -> Option<String> {
    if snapshot_raw.starts_with("dword:") {
        registry::get_dword(key_path, name).map(format_live_value_dword)
    } else if snapshot_raw.starts_with('"') {
        registry::get_string(key_path, name).map(|s| format_live_value_string(&s))
    } else {
        registry::get_string(key_path, name)
            .map(|s| format_live_value_string(&s))
            .or_else(|| registry::get_dword(key_path, name).map(format_live_value_dword))
    }
}

/// Formate la description textuelle d'un `ChangeRequest` pour la prevu et le log.
pub(super) fn describe_change(change: &ChangeRequest) -> String {
    let value_label = if change.value_name.is_empty() {
        "(default)".to_string()
    } else {
        change.value_name.clone()
    };
    match change.change_type {
        ChangeRequestType::RestoreModified => format!(
            "{}\\{} : restaurer (vers {})",
            change.key_path,
            value_label,
            change.target_value.as_deref().unwrap_or("?")
        ),
        ChangeRequestType::RestoreRemoved => format!(
            "{}\\{} : recréer (depuis snapshot, vers {})",
            change.key_path,
            value_label,
            change.target_value.as_deref().unwrap_or("?")
        ),
        ChangeRequestType::UndoAdded => format!(
            "{}\\{} : supprimer (ajouté hors snapshot)",
            change.key_path, value_label
        ),
    }
}

/// Compare deux maps `section -> {name -> raw_value}` et retourne le `SnapshotDiff`.
///
/// Pure : pas d'I/O. Utilise par `diff_snapshot_vs_snapshot` et testable en isolation.
pub(super) fn diff_two_maps(
    a: &HashMap<String, HashMap<String, String>>,
    b: &HashMap<String, HashMap<String, String>>,
) -> SnapshotDiff {
    let mut keys: Vec<RegistryDiff> = Vec::new();
    let mut total_changes = 0;

    let all_sections: BTreeSet<&String> = a.keys().chain(b.keys()).collect();

    for section in all_sections {
        let empty: HashMap<String, String> = HashMap::new();
        let a_values = a.get(section).unwrap_or(&empty);
        let b_values = b.get(section).unwrap_or(&empty);

        let mut changes = Vec::new();
        let all_names: BTreeSet<&String> = a_values.keys().chain(b_values.keys()).collect();

        for name in all_names {
            match (a_values.get(name), b_values.get(name)) {
                (Some(av), Some(bv)) if av != bv => {
                    changes.push(KeyValueChange::Modified {
                        name: name.clone(),
                        snapshot: av.clone(),
                        current: bv.clone(),
                    });
                }
                (Some(av), None) => {
                    changes.push(KeyValueChange::Removed {
                        name: name.clone(),
                        snapshot: av.clone(),
                    });
                }
                (None, Some(bv)) => {
                    changes.push(KeyValueChange::Added {
                        name: name.clone(),
                        current: bv.clone(),
                    });
                }
                _ => {}
            }
        }

        if !changes.is_empty() {
            total_changes += changes.len();
            keys.push(RegistryDiff {
                key_path: section.clone(),
                changes,
            });
        }
    }

    SnapshotDiff {
        snapshot_timestamp: String::new(),
        comparison_target: None,
        keys,
        total_changes,
    }
}

/// Compare un snapshot (`section -> {name -> raw}`) avec l'etat REGISTRE LIVE.
///
/// Detecte Removed/Modified (valeurs du snapshot absentes/differentes en live) et
/// Added (valeurs live hors snapshot). Effectue des lectures registre (I/O),
/// contrairement a [`diff_two_maps`]. Symetrie avec `diff_two_maps` pour la
/// commande `diff_snapshot_vs_live` (extrait de mod.rs, debt-rust-05).
pub(super) fn diff_snapshot_against_live(
    snapshot_by_section: HashMap<String, HashMap<String, String>>,
    snapshot_timestamp: String,
) -> SnapshotDiff {
    let mut keys: Vec<RegistryDiff> = Vec::new();
    let mut total_changes = 0;

    for (section, snap_values) in snapshot_by_section {
        let mut changes = Vec::new();
        for (name, snap_raw) in &snap_values {
            match read_live_value(&section, name, snap_raw) {
                None => {
                    changes.push(KeyValueChange::Removed {
                        name: name.clone(),
                        snapshot: snap_raw.clone(),
                    });
                }
                Some(current) if current != *snap_raw => {
                    changes.push(KeyValueChange::Modified {
                        name: name.clone(),
                        snapshot: snap_raw.clone(),
                        current,
                    });
                }
                _ => {}
            }
        }

        // Detection des Added : enumerer les valeurs live de cette section
        // et reporter celles qui ne sont pas dans le snapshot.
        if registry::is_allowed_key_path(&section) {
            for live_name in registry::get_value_names(&section) {
                if snap_values.contains_key(&live_name) {
                    continue;
                }
                // Tente DWORD puis fallback string pour formater au format raw .reg
                let formatted = if let Some(v) = registry::get_dword(&section, &live_name) {
                    format_live_value_dword(v)
                } else if let Some(s) = registry::get_string(&section, &live_name) {
                    format_live_value_string(&s)
                } else {
                    continue;
                };
                changes.push(KeyValueChange::Added {
                    name: live_name,
                    current: formatted,
                });
            }
        }

        if !changes.is_empty() {
            total_changes += changes.len();
            keys.push(RegistryDiff {
                key_path: section,
                changes,
            });
        }
    }

    keys.sort_by(|a, b| a.key_path.cmp(&b.key_path));

    SnapshotDiff {
        snapshot_timestamp,
        comparison_target: None,
        keys,
        total_changes,
    }
}

#[cfg(test)]
mod tests;
