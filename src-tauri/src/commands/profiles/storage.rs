//! Persistance des profils customs dans `%APPDATA%/Wintyx/custom_profiles.json`.
//!
//! Lecture avec validation `MAX_CUSTOM_PROFILES` (fix M3 v2.7.1, anti-bypass JSON
//! edite a la main) ; un JSON illisible est mis de cote par `utils::json_store`.
//! Ecriture atomique via `utils::fs_atomic` (fix H3 v2.7.1, anti-TOCTOU,
//! tempfile unique par ecriture) et section critique `update_custom_profiles`. Validation des bornes nom/description/items via
//! `validate_profile_fields`.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::utils::errors::CommandError;
use crate::utils::json_store;

use super::{Profile, MAX_CUSTOM_PROFILES, MAX_DESCRIPTION_LEN, MAX_NAME_LEN};

pub(super) fn custom_profiles_path() -> PathBuf {
    crate::utils::paths::wintyx_roaming_dir().join("custom_profiles.json")
}

pub(super) fn load_custom_profiles_from(path: &Path) -> Result<Vec<Profile>, CommandError> {
    let profiles: Vec<Profile> = json_store::load_or_quarantine(path, "profiles")?;
    // Defense en profondeur : un JSON edite a la main pourrait depasser
    // MAX_CUSTOM_PROFILES ; on tronque au chargement.
    Ok(profiles
        .into_iter()
        .filter(|p| {
            let valid = !has_control_chars(&p.name) && !has_control_chars(&p.description);
            if !valid {
                crate::utils::logger::warn(
                    "profiles",
                    &format!("profil {} ignoré : caractères de contrôle dans le nom ou la description", p.id),
                );
            }
            valid
        })
        .take(MAX_CUSTOM_PROFILES)
        .collect())
}

pub(super) fn save_custom_profiles_to(path: &Path, profiles: &[Profile]) -> Result<(), CommandError> {
    json_store::write_json_atomic(path, profiles)
}

static PROFILES_LOCK: Mutex<()> = Mutex::new(());

/// Section critique chargement -> modification -> sauvegarde : sans elle, deux
/// commandes concurrentes lisent le meme etat et la derniere ecriture efface l'autre.
pub(super) fn update_custom_profiles_at<R>(
    path: &Path,
    f: impl FnOnce(&mut Vec<Profile>) -> Result<R, CommandError>,
) -> Result<R, CommandError> {
    let _guard = PROFILES_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut profiles = load_custom_profiles_from(path)?;
    let outcome = f(&mut profiles)?;
    save_custom_profiles_to(path, &profiles)?;
    Ok(outcome)
}

pub(super) fn update_custom_profiles<R>(
    f: impl FnOnce(&mut Vec<Profile>) -> Result<R, CommandError>,
) -> Result<R, CommandError> {
    update_custom_profiles_at(&custom_profiles_path(), f)
}

pub(super) fn load_custom_profiles() -> Result<Vec<Profile>, CommandError> {
    load_custom_profiles_from(&custom_profiles_path())
}

fn has_control_chars(text: &str) -> bool {
    text.chars().any(|c| c.is_control())
}

pub(super) fn validate_profile_fields(
    name: &str,
    description: &str,
    items_count: usize,
) -> Result<(), CommandError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_NAME_LEN {
        return Err(CommandError::InvalidInput(format!(
            "Nom invalide (1-{} caracteres)",
            MAX_NAME_LEN
        )));
    }
    if has_control_chars(trimmed) || has_control_chars(description) {
        return Err(CommandError::InvalidInput(
            "Nom ou description : caracteres de controle interdits".into(),
        ));
    }
    if description.len() > MAX_DESCRIPTION_LEN {
        return Err(CommandError::InvalidInput(format!(
            "Description trop longue (max {} caractères)",
            MAX_DESCRIPTION_LEN
        )));
    }
    if items_count == 0 {
        return Err(CommandError::InvalidInput(
            "Profil doit contenir au moins 1 action".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
