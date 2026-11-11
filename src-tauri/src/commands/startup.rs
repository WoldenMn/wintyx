use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use crate::utils::{admin, errors::CommandError, registry};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    pub name: String,
    pub command: String,
    pub location: String,
    pub location_display: String,
    pub is_enabled: bool,
}

const HKCU_RUN: &str = r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const HKLM_RUN: &str = r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const HKCU_RUN_DISABLED: &str = r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run_Disabled";
const HKLM_RUN_DISABLED: &str = r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run_Disabled";
const HKCU_RUNONCE: &str = r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce";
const HKLM_RUNONCE: &str = r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce";

fn get_registry_startup_items(
    reg_path: &str,
    location: &str,
    location_display: &str,
    is_enabled: bool,
) -> Vec<StartupItem> {
    let names = registry::get_value_names(reg_path);
    names.into_iter()
        .filter(|n| !n.is_empty())
        .map(|name| {
            let command = registry::get_string(reg_path, &name).unwrap_or_default();
            StartupItem {
                name,
                command,
                location: location.to_string(),
                location_display: location_display.to_string(),
                is_enabled,
            }
        })
        .collect()
}

/// Nom affichable d'un fichier de demarrage : le file_stem, ou "(sans nom)" +
/// log si le chemin n'a pas de composant nom exploitable (env-005), au lieu d'une
/// chaine vide produisant un element fantome dans l'UI.
fn file_stem_or_unnamed(path: &std::path::Path) -> String {
    match path.file_stem() {
        Some(s) => s.to_string_lossy().to_string(),
        None => {
            crate::utils::logger::warn(
                "startup",
                &format!("fichier startup sans nom exploitable: {}", path.display()),
            );
            "(sans nom)".to_string()
        }
    }
}

const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;

fn is_hidden_or_system(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM) != 0
}

/// Elements d'un dossier Demarrage. Un fichier `<x>.lnk.disabled` (pose par
/// `disable_startup_item`) est liste comme desactive sous son nom reel `x`.
fn items_in_startup_dir(dir: &Path, location_display: &str) -> Vec<StartupItem> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|entry| entry.metadata().map(|m| m.is_file() && !is_hidden_or_system(&m)).unwrap_or(false))
        .map(|entry| {
            let file_path = entry.path();
            let disabled = file_path.extension().and_then(|e| e.to_str()) == Some("disabled");
            let original = if disabled { file_path.with_extension("") } else { file_path.clone() };
            StartupItem {
                name: file_stem_or_unnamed(&original),
                command: file_path.to_string_lossy().to_string(),
                location: "StartupFolder".to_string(),
                location_display: location_display.to_string(),
                is_enabled: !disabled,
            }
        })
        .collect()
}

fn get_startup_folder_items() -> Vec<StartupItem> {
    let mut items = Vec::new();
    if let Some(path) = dirs_user_startup() {
        items.extend(items_in_startup_dir(&path, "Dossier Démarrage (utilisateur)"));
    }
    if let Some(path) = dirs_common_startup() {
        items.extend(items_in_startup_dir(&path, "Dossier Démarrage (tous les utilisateurs)"));
    }
    items
}

fn dirs_user_startup() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    Some(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup"))
}

fn dirs_common_startup() -> Option<PathBuf> {
    let pd = std::env::var("PROGRAMDATA").ok()?;
    Some(PathBuf::from(pd).join(r"Microsoft\Windows\Start Menu\Programs\Startup"))
}

/// Valide un nom d'entree de demarrage. Ces noms sont des noms de valeurs
/// registre (espaces legitimes, ex "NVIDIA Share") passes uniquement a
/// registry::get_string/set_string/delete_value — jamais a un shell — d'ou
/// sanitize_value_name et non sanitize_name (qui rejette les espaces).
fn sanitize_startup_name(name: &str) -> Result<String, CommandError> {
    registry::sanitize_value_name(name).map_err(CommandError::InvalidInput)
}

fn startup_item_not_found(name: &str) -> CommandError {
    CommandError::NotFound(format!("Élément de démarrage '{}'", name))
}

fn ensure_registry_item_exists(reg_path: &str, name: &str) -> Result<(), CommandError> {
    if registry::key_exists(reg_path, name) {
        Ok(())
    } else {
        Err(CommandError::NotFound(format!("Élément de démarrage '{}' dans {}", name, reg_path)))
    }
}

/// Cherche dans `dir` un fichier desactive `<original>.disabled` dont le nom
/// original a pour `file_stem` `safe_name`. Gere le cas dominant `<x>.lnk.disabled`
/// que l'ancienne reconstruction `<safe_name>.disabled` ne retrouvait jamais.
fn find_disabled_in_dir(dir: &Path, safe_name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("disabled") {
            continue;
        }
        let original = path.with_extension("");
        let stem = original.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        if stem == safe_name {
            return Some(path);
        }
    }
    None
}

#[tauri::command]
#[specta::specta]
pub fn get_startup_items() -> Vec<StartupItem> {
    crate::perf_timed!("get_startup_items", {
        let mut items = Vec::new();
        items.extend(get_registry_startup_items(HKCU_RUN, "HKCU_Run", "Utilisateur courant (registre)", true));
        items.extend(get_registry_startup_items(HKLM_RUN, "HKLM_Run", "Tous les utilisateurs (registre)", true));
        items.extend(get_registry_startup_items(HKCU_RUN_DISABLED, "HKCU_Run", "Utilisateur courant (registre)", false));
        items.extend(get_registry_startup_items(HKLM_RUN_DISABLED, "HKLM_Run", "Tous les utilisateurs (registre)", false));
        items.extend(get_registry_startup_items(HKCU_RUNONCE, "HKCU_RunOnce", "Utilisateur courant (RunOnce)", true));
        items.extend(get_registry_startup_items(HKLM_RUNONCE, "HKLM_RunOnce", "Tous les utilisateurs (RunOnce)", true));
        items.extend(get_startup_folder_items());
        items
    })
}

#[tauri::command]
#[specta::specta]
pub async fn disable_startup_item(name: String, location: String) -> Result<(), CommandError> {
    crate::utils::blocking::write_result(move || disable_startup_item_sync(name, location)).await
}

pub fn disable_startup_item_sync(name: String, location: String) -> Result<(), CommandError> {
    if location.starts_with("HKLM") {
        admin::require_admin("Désactiver un démarrage machine")?;
    }
    let safe_name = sanitize_startup_name(&name)?;
    match location.as_str() {
        "HKCU_Run" => {
            registry::move_string_value(HKCU_RUN, HKCU_RUN_DISABLED, &safe_name)?;
        }
        "HKLM_Run" => {
            registry::move_string_value(HKLM_RUN, HKLM_RUN_DISABLED, &safe_name)?;
        }
        "HKCU_RunOnce" => {
            ensure_registry_item_exists(HKCU_RUNONCE, &safe_name)?;
            registry::delete_value(HKCU_RUNONCE, &safe_name)?;
        }
        "HKLM_RunOnce" => {
            ensure_registry_item_exists(HKLM_RUNONCE, &safe_name)?;
            registry::delete_value(HKLM_RUNONCE, &safe_name)?;
        }
        "StartupFolder" => {
            let items = get_startup_folder_items();
            let item = items
                .iter()
                .find(|i| i.is_enabled && i.name == safe_name)
                .ok_or_else(|| startup_item_not_found(&safe_name))?;
            let src = PathBuf::from(&item.command);
            if !src.exists() {
                return Err(startup_item_not_found(&safe_name));
            }
            let dest = format!("{}.disabled", item.command);
            fs::rename(&src, &dest).map_err(|e| e.to_string())?;
        }
        _ => return Err(CommandError::InvalidInput(format!("Location inconnue : {}", location))),
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn enable_startup_item(name: String, location: String) -> Result<(), CommandError> {
    crate::utils::blocking::write_result(move || enable_startup_item_sync(name, location)).await
}

pub fn enable_startup_item_sync(name: String, location: String) -> Result<(), CommandError> {
    if location.starts_with("HKLM") {
        admin::require_admin("Réactiver un démarrage machine")?;
    }
    let safe_name = sanitize_startup_name(&name)?;
    match location.as_str() {
        "HKCU_Run" => {
            registry::move_string_value(HKCU_RUN_DISABLED, HKCU_RUN, &safe_name)?;
        }
        "HKLM_Run" => {
            registry::move_string_value(HKLM_RUN_DISABLED, HKLM_RUN, &safe_name)?;
        }
        "StartupFolder" => {
            // Chercher dans les dossiers User ET Common (symetrie avec disable),
            // en matchant par file_stem pour retrouver les `.lnk.disabled`.
            let disabled_path = [dirs_user_startup(), dirs_common_startup()]
                .into_iter()
                .flatten()
                .filter(|dir| dir.exists())
                .find_map(|dir| find_disabled_in_dir(&dir, &safe_name))
                .ok_or_else(|| startup_item_not_found(&safe_name))?;
            let original = disabled_path.with_extension("");
            fs::rename(&disabled_path, &original).map_err(|e| e.to_string())?;
        }
        _ => return Err(CommandError::InvalidInput(format!("Location inconnue : {}", location))),
    }
    Ok(())
}

#[cfg(test)]
mod tests;
