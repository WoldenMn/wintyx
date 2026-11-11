use serde::Serialize;
use std::time::Duration;
use crate::utils::{admin, constants, errors::CommandError, logger, process, registry, snapshot_cache::SnapshotCache};

mod sc_state;
use sc_state::*;

const SERVICE_BACKUP_KEY: &str = r"HKCU\SOFTWARE\Wintyx\ServiceBackup";

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ServiceItem {
    pub display_name: String,
    pub name: String,
    pub risk: String,
    pub status: String,
    pub description: String,
    pub start_type: String,
}

struct ServiceDef {
    name: &'static str,
    display_name: &'static str,
    description: &'static str,
    risk: &'static str,
}

fn build_services() -> Vec<ServiceDef> {
    vec![
        ServiceDef { name: "DiagTrack", display_name: "Expériences des utilisateurs connectés et télémétrie", description: "Service de collecte de télémétrie Microsoft", risk: "Low" },
        ServiceDef { name: "dmwappushservice", display_name: "Routage des messages Push WAP", description: "Achemine les messages push WAP pour la télémétrie", risk: "Low" },
        ServiceDef { name: "SysMain", display_name: "SysMain (Superfetch)", description: "Précharge les applications en mémoire, sollicite le disque", risk: "Low" },
        ServiceDef { name: "WSearch", display_name: "Recherche Windows", description: "Service d'indexation pour la recherche de fichiers", risk: "Medium" },
        ServiceDef { name: "RemoteRegistry", display_name: "Registre à distance", description: "Autorise la modification du registre à distance", risk: "Low" },
        ServiceDef { name: "Fax", display_name: "Télécopie", description: "Ancien service de télécopie", risk: "Low" },
        ServiceDef { name: "lfsvc", display_name: "Service de géolocalisation", description: "Suit la position de l'appareil", risk: "Low" },
        ServiceDef { name: "MapsBroker", display_name: "Gestionnaire de cartes téléchargées", description: "Gère les cartes hors connexion", risk: "Low" },
        ServiceDef { name: "RetailDemo", display_name: "Service de démonstration en magasin", description: "Mode démonstration pour les présentoirs", risk: "Low" },
        ServiceDef { name: "WMPNetworkSvc", display_name: "Partage réseau de Windows Media Player", description: "Partage de médias sur le réseau", risk: "Low" },
        ServiceDef { name: "XblAuthManager", display_name: "Gestionnaire d'authentification Xbox Live", description: "Service d'authentification Xbox Live", risk: "Low" },
        ServiceDef { name: "XblGameSave", display_name: "Sauvegarde de jeu Xbox Live", description: "Synchronisation des sauvegardes de jeu Xbox", risk: "Low" },
        ServiceDef { name: "XboxNetApiSvc", display_name: "Service de mise en réseau Xbox Live", description: "Prise en charge réseau Xbox Live", risk: "Medium" },
        ServiceDef { name: "WbioSrvc", display_name: "Service de biométrie Windows", description: "Prise en charge des empreintes digitales et de la biométrie", risk: "High" },
    ]
}

#[tauri::command]
#[specta::specta]
pub async fn get_services() -> Vec<ServiceItem> {
    crate::utils::blocking::read_value(get_services_sync).await
}

/// `get_all_scores`, le prefetch et la page Services lisent les mêmes 14 services
/// dans la même seconde : une collecte partagée évite de relancer 14 `sc.exe`.
static SERVICES_SNAPSHOT: SnapshotCache<Vec<ServiceItem>> = SnapshotCache::new();

struct StaleAfterWrite;

impl Drop for StaleAfterWrite {
    fn drop(&mut self) {
        SERVICES_SNAPSHOT.invalidate();
    }
}

pub fn get_services_sync() -> Vec<ServiceItem> {
    SERVICES_SNAPSHOT.get_or_collect(Duration::from_secs(constants::SERVICES_SNAPSHOT_TTL_SECS), collect_services)
}

fn collect_services() -> Vec<ServiceItem> {
    crate::perf_timed!("get_services", {
        let defs = build_services();
        defs.into_iter()
            .map(|def| {
                let (status, start_type) = query_service_status(def.name);
                ServiceItem {
                    display_name: def.display_name.to_string(),
                    name: def.name.to_string(),
                    risk: def.risk.to_string(),
                    status,
                    description: def.description.to_string(),
                    start_type,
                }
            })
            .collect()
    })
}

fn backup_original_start(safe_name: &str, reg_path: &str) {
    let backup_path = format!(r"{}\{}", SERVICE_BACKUP_KEY, safe_name);
    backup_original_start_to(&backup_path, registry::get_dword(reg_path, "Start"));
}

/// Ne memorise le demarrage d'origine qu'une seule fois : un second arret ne doit pas
/// ecraser la valeur initiale par `4` (Desactive), sinon la reactivation la perd.
fn backup_original_start_to(backup_path: &str, current: Option<u32>) {
    let Some(current) = current else { return };
    if registry::get_dword(backup_path, "OriginalStart").is_some() {
        return;
    }
    if let Err(e) = registry::set_dword(backup_path, "OriginalStart", current) {
        logger::warn("services", &format!("Echec sauvegarde Start ({}): {}", backup_path, e));
    }
}

fn restore_original_start(safe_name: &str) -> u32 {
    let backup_path = format!(r"{}\{}", SERVICE_BACKUP_KEY, safe_name);
    let value = registry::get_dword(&backup_path, "OriginalStart").unwrap_or(3);
    if !(2..=4).contains(&value) {
        return 3;
    }
    value
}

#[tauri::command]
#[specta::specta]
pub async fn stop_service(name: String) -> Result<(), CommandError> {
    crate::utils::blocking::write_result(move || stop_service_sync(name)).await
}

pub fn stop_service_sync(name: String) -> Result<(), CommandError> {
    admin::require_admin("Arrêter un service")?;
    let _stale = StaleAfterWrite;
    let safe_name = registry::sanitize_name(&name).map_err(CommandError::InvalidInput)?;
    let known = build_services();
    if !known.iter().any(|d| d.name == safe_name) {
        return Err(CommandError::NotFound(format!("Service : {}", safe_name)));
    }
    let reg_path = format!(r"{}\{}", constants::REG_SERVICES, safe_name);
    backup_original_start(&safe_name, &reg_path);
    let result = process::run("sc.exe", &format!("stop {}", safe_name), constants::TIMEOUT_LONG);
    match classify_stop(result.success, result.exit_code, &result.stdout) {
        StopOutcome::Stopped => {}
        StopOutcome::AlreadyStopped => {
            logger::info("services", &format!("Service deja arrete: {}", safe_name));
        }
        StopOutcome::Absent => {
            logger::info("services", &format!("Service non installe, rien a arreter: {}", safe_name));
            return Ok(());
        }
        StopOutcome::Failed => {
            return Err(CommandError::ProcessFailed {
                command: format!("sc.exe stop {}", safe_name),
                detail: if result.stderr.trim().is_empty() {
                    result.stdout.trim().to_string()
                } else {
                    result.stderr.trim().to_string()
                },
            });
        }
    }
    registry::set_dword(&reg_path, "Start", 4).map_err(CommandError::Registry)?;
    logger::info("services", &format!("Service arrete: {}", safe_name));
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn start_service(name: String) -> Result<(), CommandError> {
    crate::utils::blocking::write_result(move || start_service_sync(name)).await
}

pub fn start_service_sync(name: String) -> Result<(), CommandError> {
    admin::require_admin("Demarrer un service")?;
    let _stale = StaleAfterWrite;
    let safe_name = registry::sanitize_name(&name).map_err(CommandError::InvalidInput)?;
    let known = build_services();
    if !known.iter().any(|d| d.name == safe_name) {
        return Err(CommandError::NotFound(format!("Service : {}", safe_name)));
    }
    let reg_path = format!(r"{}\{}", constants::REG_SERVICES, safe_name);
    match query_service_state(&safe_name) {
        ServiceState::Absent => {
            return Err(CommandError::NotFound(format!("Service non installé : {}", safe_name)));
        }
        ServiceState::Unknown => {
            return Err(CommandError::ProcessFailed {
                command: format!("sc.exe query {}", safe_name),
                detail: "état du service illisible".to_string(),
            });
        }
        ServiceState::Running | ServiceState::Stopped => {}
    }
    let original = restore_original_start(&safe_name);
    registry::set_dword(&reg_path, "Start", original).map_err(CommandError::Registry)?;
    let result = process::run("sc.exe", &format!("start {}", safe_name), constants::TIMEOUT_LONG);
    if !result.success {
        return Err(CommandError::ProcessFailed {
            command: format!("sc.exe start {}", safe_name),
            detail: result.stderr.trim().to_string(),
        });
    }
    logger::info("services", &format!("Service demarre: {} (Start={})", safe_name, original));
    Ok(())
}

pub(crate) fn risk_weight(risk: &str) -> u32 {
    match risk.to_lowercase().as_str() {
        "high" => 3,
        "medium" => 2,
        _ => 1,
    }
}

pub(crate) fn compute_score(items: &[(String, String)]) -> u32 {
    if items.is_empty() {
        return 0;
    }
    let max_points: u32 = items.iter().map(|(risk, _)| risk_weight(risk)).sum();
    if max_points == 0 {
        // Garde defensive contre division par zero ligne suivante. Inatteignable
        // en pratique (risk_weight >= 1 toujours, donc max_points == 0 <=> items vide,
        // deja capture ci-dessus). Retourne 0 pour coherence avec les 4 autres domaines.
        return 0;
    }
    let earned_points: u32 = items
        .iter()
        .filter(|(_, status)| status == "Stopped" || status == "Absent")
        .map(|(risk, _)| risk_weight(risk))
        .sum();
    ((earned_points as f64 / max_points as f64) * 100.0).round() as u32
}

pub fn get_services_score_sync() -> u32 {
    let items = get_services_sync();
    let pairs: Vec<(String, String)> = items
        .into_iter()
        .map(|i| (i.risk, i.status))
        .collect();
    compute_score(&pairs)
}

#[cfg(test)]
mod tests;
