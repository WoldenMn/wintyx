//! Profils de configuration Wintyx (presets + customs).
//!
//! Module decompose en sous-modules par responsabilite :
//! - `presets` : profils predefinis hardcodes (Bureau, Discrétion max, Gaming, Performance)
//! - `storage` : persistance JSON locale + validation des bornes
//!
//! Ce fichier expose les 6 commandes Tauri publiques et les structs serialises.

mod applicator;
mod presets;
mod storage;

use serde::{Deserialize, Serialize};

use crate::utils::batch::{BatchOptions, BatchResult};
use crate::utils::errors::CommandError;

const MAX_CUSTOM_PROFILES: usize = 20;
const MAX_NAME_LEN: usize = 80;
const MAX_DESCRIPTION_LEN: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub data_protection_ids: Vec<String>,
    #[serde(default)]
    pub hardening_ids: Vec<String>,
    #[serde(default)]
    pub performance_ids: Vec<String>,
    #[serde(default)]
    pub is_custom: bool,
    #[serde(default)]
    pub services_ids: Vec<String>,
    #[serde(default)]
    pub bloatware_ids: Vec<String>,
    #[serde(default)]
    pub startup_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AvailableItem {
    pub id: String,
    pub label: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AvailableItems {
    pub data_protection: Vec<AvailableItem>,
    pub hardening: Vec<AvailableItem>,
    pub performance: Vec<AvailableItem>,
    pub services: Vec<AvailableItem>,
    pub bloatware: Vec<AvailableItem>,
    pub startup: Vec<AvailableItem>,
}

#[tauri::command]
#[specta::specta]
pub fn get_profiles() -> Result<Vec<Profile>, CommandError> {
    let mut all = presets::presets();
    for mut c in storage::load_custom_profiles()? {
        c.is_custom = true;
        all.push(c);
    }
    Ok(all)
}

#[tauri::command]
#[specta::specta]
pub async fn apply_profile(
    id: String,
    options: Option<BatchOptions>,
) -> Result<BatchResult, CommandError> {
    crate::utils::blocking::write_result(move || apply_profile_sync(id, options)).await
}

pub fn apply_profile_sync(
    id: String,
    options: Option<BatchOptions>,
) -> Result<BatchResult, CommandError> {
    crate::perf_timed!("apply_profile", {
        // Fix v2.7.9 H1 : decouplage. La logique d'application des 6 domaines est
        // deportee dans `applicator::apply_profile_items` (helper factorise via
        // `apply_section`). Cette commande Tauri se concentre sur :
        // - Lookup du profil (presets + customs)
        // - Validation options (dry-run)
        // - Delegation a l'applicator
        let opts = options.unwrap_or_default();

        let profile = get_profiles()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| CommandError::NotFound(format!("Profil : {}", id)))?;

        Ok(applicator::apply_profile_items(&profile, opts.dry_run))
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_available_items() -> AvailableItems {
    crate::utils::blocking::read_value(get_available_items_sync).await
}

pub fn get_available_items_sync() -> AvailableItems {
    let dp = super::data_protection::get_data_protection_items()
        .into_iter()
        .map(|item| AvailableItem {
            id: item.id,
            label: item.name,
            category: item.category,
        })
        .collect();

    let hard = super::hardening::get_hardening_checks_sync()
        .into_iter()
        .filter(|item| item.is_fixable)
        .map(|item| AvailableItem {
            id: item.id,
            label: item.name,
            category: item.category,
        })
        .collect();

    let perf = super::performance::get_performance_items_sync()
        .into_iter()
        .map(|item| AvailableItem {
            id: item.id,
            label: item.name,
            category: item.category,
        })
        .collect();

    let services = super::services::get_services_sync()
        .into_iter()
        .map(|s| AvailableItem {
            id: s.name,
            label: s.display_name,
            category: s.risk,
        })
        .collect();

    let bloatware = super::bloatware::get_bloatware_items_sync()
        .unwrap_or_else(|e| {
            crate::utils::logger::warn("profiles", &format!("get_available_items: bloatware indisponible: {}", e));
            Vec::new()
        })
        .into_iter()
        .map(|b| AvailableItem {
            id: b.package_name,
            label: b.display_name,
            category: b.category,
        })
        .collect();

    let startup = super::startup::get_startup_items()
        .into_iter()
        .map(|s| AvailableItem {
            id: format!("{}|{}", s.location, s.name),
            label: s.name.clone(),
            category: s.location_display,
        })
        .collect();

    AvailableItems {
        data_protection: dp,
        hardening: hard,
        performance: perf,
        services,
        bloatware,
        startup,
    }
}

#[tauri::command]
#[specta::specta]
pub fn create_custom_profile(
    source_preset_id: String,
    name: String,
) -> Result<Profile, CommandError> {
    let trimmed_name = name.trim().to_string();
    storage::validate_profile_fields(&trimmed_name, "", 1)?;

    let preset = presets::presets()
        .into_iter()
        .find(|p| p.id == source_preset_id)
        .ok_or_else(|| CommandError::NotFound(format!("Preset : {}", source_preset_id)))?;

    storage::update_custom_profiles(|customs| {
        if customs.len() >= MAX_CUSTOM_PROFILES {
            return Err(CommandError::InvalidInput(format!(
                "Limite {} profils personnalisés atteinte",
                MAX_CUSTOM_PROFILES
            )));
        }

        let new_id = format!("custom_{}", chrono::Local::now().timestamp_millis());
        let new_profile = Profile {
            id: new_id,
            name: trimmed_name,
            description: format!("Cloné depuis {}", preset.name),
            data_protection_ids: preset.data_protection_ids.clone(),
            hardening_ids: preset.hardening_ids.clone(),
            performance_ids: preset.performance_ids.clone(),
            is_custom: true,
            services_ids: preset.services_ids.clone(),
            bloatware_ids: preset.bloatware_ids.clone(),
            startup_ids: preset.startup_ids.clone(),
        };
        customs.push(new_profile.clone());
        Ok(new_profile)
    })
}

#[tauri::command]
#[specta::specta]
pub fn update_custom_profile(profile: Profile) -> Result<Profile, CommandError> {
    if !profile.id.starts_with("custom_") {
        return Err(CommandError::InvalidInput(
            "Impossible de modifier un profil prédéfini".into(),
        ));
    }

    let total_items = profile.data_protection_ids.len()
        + profile.hardening_ids.len()
        + profile.performance_ids.len()
        + profile.services_ids.len()
        + profile.bloatware_ids.len()
        + profile.startup_ids.len();
    storage::validate_profile_fields(&profile.name, &profile.description, total_items)?;

    storage::update_custom_profiles(|customs| {
        let pos = customs
            .iter()
            .position(|p| p.id == profile.id)
            .ok_or_else(|| CommandError::NotFound(format!("Profil personnalisé : {}", profile.id)))?;

        let mut updated = profile;
        updated.name = updated.name.trim().to_string();
        updated.is_custom = true;
        customs[pos] = updated.clone();
        Ok(updated)
    })
}

#[tauri::command]
#[specta::specta]
pub fn delete_custom_profile(id: String) -> Result<(), CommandError> {
    if !id.starts_with("custom_") {
        return Err(CommandError::InvalidInput(
            "Impossible de supprimer un profil prédéfini".into(),
        ));
    }

    storage::update_custom_profiles(|customs| {
        let initial_len = customs.len();
        customs.retain(|p| p.id != id);
        if customs.len() == initial_len {
            return Err(CommandError::NotFound(format!("Profil personnalisé : {}", id)));
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests;
