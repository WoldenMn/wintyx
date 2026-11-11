use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use crate::utils::errors::CommandError;

use super::{parser, ChangeRequest, ChangeRequestType};

pub(super) type RecordedValues = HashMap<String, HashMap<String, String>>;

pub(super) fn load_snapshots(folder: &Path) -> Vec<RecordedValues> {
    let mut by_timestamp: BTreeMap<String, RecordedValues> = BTreeMap::new();
    for (path, timestamp) in parser::iter_reg_files(folder) {
        let Some(content) = parser::decode_reg_file(&path) else {
            continue;
        };
        let snapshot = by_timestamp.entry(timestamp).or_default();
        for (section, name, raw) in parser::parse_reg_content(&content) {
            snapshot
                .entry(section.to_lowercase())
                .or_default()
                .insert(name.to_lowercase(), raw);
        }
    }
    by_timestamp.into_values().collect()
}

fn recorded<'a>(snapshot: &'a RecordedValues, section: &str, name: &str) -> Option<&'a String> {
    snapshot.get(section).and_then(|values| values.get(name))
}

fn describe(change: &ChangeRequest) -> String {
    let name = if change.value_name.is_empty() {
        "(default)"
    } else {
        change.value_name.as_str()
    };
    format!("{}\\{}", change.key_path, name)
}

pub(super) fn verify_changes(
    changes: &[ChangeRequest],
    snapshots: &[RecordedValues],
) -> Result<(), CommandError> {
    for change in changes {
        let section = change.key_path.to_lowercase();
        let name = change.value_name.to_lowercase();
        match change.change_type {
            ChangeRequestType::RestoreModified | ChangeRequestType::RestoreRemoved => {
                let target = change.target_value.as_deref().ok_or_else(|| {
                    CommandError::InvalidInput("target_value requis pour ce type de changement".into())
                })?;
                let known: Vec<&String> = snapshots
                    .iter()
                    .filter_map(|snapshot| recorded(snapshot, &section, &name))
                    .collect();
                if known.is_empty() {
                    return Err(CommandError::InvalidInput(format!(
                        "Valeur absente des snapshots, restauration refusée : {}",
                        describe(change)
                    )));
                }
                if !known.iter().any(|raw| raw.as_str() == target) {
                    return Err(CommandError::InvalidInput(format!(
                        "Valeur différente de celle des snapshots, restauration refusée : {}",
                        describe(change)
                    )));
                }
            }
            ChangeRequestType::UndoAdded => {
                if !snapshots.iter().any(|snapshot| snapshot.contains_key(&section)) {
                    return Err(CommandError::InvalidInput(format!(
                        "Aucun snapshot ne couvre cette clé, suppression refusée : {}",
                        describe(change)
                    )));
                }
                if snapshots
                    .iter()
                    .all(|snapshot| recorded(snapshot, &section, &name).is_some()) {
                    return Err(CommandError::InvalidInput(format!(
                        "Valeur présente dans tous les snapshots, suppression refusée : {}",
                        describe(change)
                    )));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
