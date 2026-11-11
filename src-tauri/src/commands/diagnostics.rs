use serde::Serialize;
use crate::utils::constants;
use crate::utils::errors::CommandError;
use crate::utils::{admin, paths, process};
use crate::utils::wmi_helpers::with_wmi_conn;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticStatus {
    Ok,
    Warning,
    Failed,
}

#[derive(Serialize, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticCheck {
    pub id: String,
    pub label: String,
    pub status: DiagnosticStatus,
    pub detail: String,
    pub affected_pages: Vec<String>,
}

#[derive(Serialize, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AppDiagnostics {
    pub checks: Vec<DiagnosticCheck>,
    pub has_critical_failure: bool,
}

/// Classe l'espace disque libre. `id` du check "storage" : bas -> Warning.
pub(crate) fn classify_disk_space(free_bytes: u64, low_threshold: u64) -> DiagnosticStatus {
    if free_bytes < low_threshold {
        DiagnosticStatus::Warning
    } else {
        DiagnosticStatus::Ok
    }
}

/// Extrait une version depuis la sortie de `$PSVersionTable.PSVersion` (ex "5.1.26200.1").
/// None si la sortie ne contient pas au moins "<major>.<minor>".
pub(crate) fn parse_ps_version(stdout: &str) -> Option<String> {
    let line = stdout.lines().map(str::trim).find(|l| !l.is_empty())?;
    let mut parts = line.split('.');
    let major = parts.next()?.trim();
    let minor = parts.next()?.trim();
    if major.chars().all(|c| c.is_ascii_digit())
        && !major.is_empty()
        && minor.chars().all(|c| c.is_ascii_digit())
        && !minor.is_empty()
    {
        Some(line.to_string())
    } else {
        None
    }
}

/// Un echec DUR sur wmi / powershell / storage declenche la banniere.
/// admin en Warning ne compte jamais comme critique.
pub(crate) fn has_critical_failure(checks: &[DiagnosticCheck]) -> bool {
    checks
        .iter()
        .any(|c| c.status == DiagnosticStatus::Failed
            && matches!(c.id.as_str(), "wmi" | "powershell" | "storage"))
}

fn check_wmi() -> DiagnosticCheck {
    let pages = vec![
        "Système".into(), "Mémoire".into(), "Disques".into(),
        "Pilotes".into(), "Santé système".into(), "Monitoring live".into(),
    ];
    // Requete triviale : si la connexion + une lecture Win32_OperatingSystem
    // aboutissent, WMI est joignable.
    let probe = with_wmi_conn(|wmi| {
        let _rows: Vec<std::collections::HashMap<String, wmi::Variant>> =
            wmi.raw_query("SELECT Caption FROM Win32_OperatingSystem")
                .map_err(|e| CommandError::WmiUnavailable(e.to_string()))?;
        Ok(())
    });
    match probe {
        Ok(()) => DiagnosticCheck {
            id: "wmi".into(),
            label: "WMI joignable".into(),
            status: DiagnosticStatus::Ok,
            detail: "Le service Windows Management Instrumentation répond.".into(),
            affected_pages: vec![],
        },
        Err(e) => DiagnosticCheck {
            id: "wmi".into(),
            label: "WMI joignable".into(),
            status: DiagnosticStatus::Failed,
            detail: format!("WMI injoignable : {}. Les pages système sont limitées.", e.friendly_message()),
            affected_pages: pages,
        },
    }
}

fn check_powershell() -> DiagnosticCheck {
    let pages = vec![
        "Réseau".into(), "Scanner de menaces".into(), "Tâches planifiées".into(),
        "Mises à jour".into(), "Bloatware".into(),
    ];
    let result = process::run(
        "powershell",
        "-NoProfile -Command \"$PSVersionTable.PSVersion.ToString()\"",
        constants::TIMEOUT_SHORT,
    );
    match parse_ps_version(&result.stdout) {
        Some(v) if result.success => DiagnosticCheck {
            id: "powershell".into(),
            label: "PowerShell disponible".into(),
            status: DiagnosticStatus::Ok,
            detail: format!("PowerShell {} répond.", v),
            affected_pages: vec![],
        },
        _ => DiagnosticCheck {
            id: "powershell".into(),
            label: "PowerShell disponible".into(),
            status: DiagnosticStatus::Failed,
            detail: "PowerShell ne répond pas. Réseau, Defender, Tâches et Màj sont limités.".into(),
            affected_pages: pages,
        },
    }
}

fn check_admin() -> DiagnosticCheck {
    if admin::is_admin() {
        DiagnosticCheck {
            id: "admin".into(),
            label: "Droits administrateur".into(),
            status: DiagnosticStatus::Ok,
            detail: "Wintyx est élevé : toutes les actions sont disponibles.".into(),
            affected_pages: vec![],
        }
    } else {
        DiagnosticCheck {
            id: "admin".into(),
            label: "Droits administrateur".into(),
            status: DiagnosticStatus::Warning,
            detail: "Session non élevée : les actions d'écriture nécessitent « Relancer en admin ».".into(),
            affected_pages: vec![
                "Durcissement".into(), "Services".into(), "Protection données".into(),
            ],
        }
    }
}

fn check_storage() -> DiagnosticCheck {
    let dir = paths::wintyx_local_dir();
    // Test d'ecriture : cree le dossier au besoin, ecrit puis supprime un fichier sonde.
    if std::fs::create_dir_all(&dir).is_err() {
        return DiagnosticCheck {
            id: "storage".into(),
            label: "Stockage %LOCALAPPDATA%".into(),
            status: DiagnosticStatus::Failed,
            detail: format!("Dossier de données inaccessible : {}", dir.display()),
            affected_pages: vec!["Sauvegarde".into(), "Logs".into()],
        };
    }
    let probe = dir.join(".diag_write_test");
    let writable = std::fs::write(&probe, b"ok").is_ok();
    let _ = std::fs::remove_file(&probe);
    if !writable {
        return DiagnosticCheck {
            id: "storage".into(),
            label: "Stockage %LOCALAPPDATA%".into(),
            status: DiagnosticStatus::Failed,
            detail: "Écriture impossible : snapshots, logs et historique seraient perdus.".into(),
            affected_pages: vec!["Sauvegarde".into(), "Logs".into()],
        };
    }
    let status = match free_space_bytes(&dir) {
        Some(free) => classify_disk_space(free, constants::DIAG_DISK_LOW_BYTES),
        None => DiagnosticStatus::Ok, // espace indeterminable : ne pas alarmer
    };
    let detail = if status == DiagnosticStatus::Warning {
        "Espace disque bas sur le volume de données Wintyx.".into()
    } else {
        "Dossier de données accessible en écriture.".into()
    };
    DiagnosticCheck {
        id: "storage".into(),
        label: "Stockage %LOCALAPPDATA%".into(),
        status,
        detail,
        affected_pages: if status == DiagnosticStatus::Warning {
            vec!["Sauvegarde".into(), "Historique des scores".into()]
        } else {
            vec![]
        },
    }
}

/// Espace libre (octets) du volume contenant `dir`, via `GetDiskFreeSpaceExW`.
/// None si l'appel echoue.
fn free_space_bytes(dir: &std::path::Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = dir.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let mut free_avail: u64 = 0;
    // SAFETY: pointeurs valides, chaine terminee par nul, les 2 autres OUT ignores.
    let ok = unsafe {
        winapi_get_disk_free_space_ex(wide.as_ptr(), &mut free_avail)
    };
    if ok { Some(free_avail) } else { None }
}

// Binding minimal a GetDiskFreeSpaceExW (kernel32). Evite d'ajouter une dependance :
// le projet cible Windows uniquement.
#[link(name = "kernel32")]
extern "system" {
    #[link_name = "GetDiskFreeSpaceExW"]
    fn get_disk_free_space_ex_w(
        lp_directory_name: *const u16,
        lp_free_bytes_available_to_caller: *mut u64,
        lp_total_number_of_bytes: *mut u64,
        lp_total_number_of_free_bytes: *mut u64,
    ) -> i32;
}

/// # Safety
///
/// `dir` doit pointer une chaine UTF-16 terminee par nul et `free_avail` un `u64`
/// ecrivable, tous deux valides pendant toute la duree de l'appel.
unsafe fn winapi_get_disk_free_space_ex(dir: *const u16, free_avail: *mut u64) -> bool {
    let mut total: u64 = 0;
    let mut total_free: u64 = 0;
    get_disk_free_space_ex_w(dir, free_avail, &mut total, &mut total_free) != 0
}

#[tauri::command]
#[specta::specta]
pub async fn get_app_diagnostics() -> Result<AppDiagnostics, CommandError> {
    crate::utils::blocking::read_result(get_app_diagnostics_sync).await
}

pub fn get_app_diagnostics_sync() -> Result<AppDiagnostics, CommandError> {
    let checks = vec![
        check_wmi(),
        check_powershell(),
        check_admin(),
        check_storage(),
    ];
    let has_critical_failure = has_critical_failure(&checks);
    Ok(AppDiagnostics { checks, has_critical_failure })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(id: &str, status: DiagnosticStatus) -> DiagnosticCheck {
        DiagnosticCheck {
            id: id.to_string(),
            label: String::new(),
            status,
            detail: String::new(),
            affected_pages: vec![],
        }
    }

    #[test]
    fn disk_below_threshold_warns() {
        assert_eq!(classify_disk_space(100, 500), DiagnosticStatus::Warning);
        assert_eq!(classify_disk_space(500, 500), DiagnosticStatus::Ok);
        assert_eq!(classify_disk_space(999, 500), DiagnosticStatus::Ok);
    }

    #[test]
    fn parses_ps_version() {
        assert_eq!(parse_ps_version("5.1.26200.1").as_deref(), Some("5.1.26200.1"));
        assert_eq!(parse_ps_version("  7.4.1  ").as_deref(), Some("7.4.1"));
        assert_eq!(parse_ps_version("").as_deref(), None);
        assert_eq!(parse_ps_version("not a version").as_deref(), None);
        assert_eq!(parse_ps_version("5").as_deref(), None);
    }

    #[test]
    fn critical_only_on_hard_infra_failure() {
        assert!(has_critical_failure(&[check("wmi", DiagnosticStatus::Failed)]));
        assert!(has_critical_failure(&[check("powershell", DiagnosticStatus::Failed)]));
        assert!(has_critical_failure(&[check("storage", DiagnosticStatus::Failed)]));
        // admin Failed n'existe pas (Warning max), mais meme Failed ne serait pas critique :
        assert!(!has_critical_failure(&[check("admin", DiagnosticStatus::Failed)]));
        assert!(!has_critical_failure(&[check("wmi", DiagnosticStatus::Warning)]));
        assert!(!has_critical_failure(&[check("admin", DiagnosticStatus::Warning)]));
    }
}
