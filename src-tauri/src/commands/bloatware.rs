use serde::Serialize;
use crate::utils::batch::BatchResult;
use crate::utils::constants;
use crate::utils::errors::CommandError;
use crate::utils::process;

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BloatwareItem {
    pub package_name: String,
    pub display_name: String,
    pub publisher: String,
    pub category: String,
    pub is_installed: bool,
    pub version: String,
}

struct BloatwareDef {
    package_name: &'static str,
    display_name: &'static str,
    publisher: &'static str,
    category: &'static str,
}

fn known_bloatware() -> Vec<BloatwareDef> {
    vec![
        BloatwareDef { package_name: "Microsoft.3DBuilder", display_name: "3D Builder", publisher: "Microsoft", category: "Entertainment" },
        BloatwareDef { package_name: "Microsoft.BingWeather", display_name: "Météo", publisher: "Microsoft", category: "Utility" },
        BloatwareDef { package_name: "Microsoft.BingNews", display_name: "Actualités", publisher: "Microsoft", category: "Utility" },
        BloatwareDef { package_name: "Microsoft.GetHelp", display_name: "Obtenir de l'aide", publisher: "Microsoft", category: "System" },
        BloatwareDef { package_name: "Microsoft.Getstarted", display_name: "Astuces", publisher: "Microsoft", category: "System" },
        BloatwareDef { package_name: "Microsoft.MicrosoftOfficeHub", display_name: "Hub Office", publisher: "Microsoft", category: "Productivity" },
        BloatwareDef { package_name: "Microsoft.MicrosoftSolitaireCollection", display_name: "Solitaire", publisher: "Microsoft", category: "Entertainment" },
        BloatwareDef { package_name: "Microsoft.MixedReality.Portal", display_name: "Portail de réalité mixte", publisher: "Microsoft", category: "Entertainment" },
        BloatwareDef { package_name: "Microsoft.OneConnect", display_name: "Forfaits mobiles", publisher: "Microsoft", category: "Network" },
        BloatwareDef { package_name: "Microsoft.People", display_name: "Contacts", publisher: "Microsoft", category: "Social" },
        BloatwareDef { package_name: "Microsoft.SkypeApp", display_name: "Skype", publisher: "Microsoft", category: "Communication" },
        BloatwareDef { package_name: "Microsoft.WindowsFeedbackHub", display_name: "Hub de commentaires", publisher: "Microsoft", category: "System" },
        BloatwareDef { package_name: "Microsoft.WindowsMaps", display_name: "Cartes", publisher: "Microsoft", category: "Navigation" },
        BloatwareDef { package_name: "Microsoft.Xbox.TCUI", display_name: "Xbox TCUI", publisher: "Microsoft", category: "Gaming" },
        BloatwareDef { package_name: "Microsoft.XboxApp", display_name: "Compagnon de console Xbox", publisher: "Microsoft", category: "Gaming" },
        BloatwareDef { package_name: "Microsoft.XboxGameOverlay", display_name: "Superposition de jeu Xbox", publisher: "Microsoft", category: "Gaming" },
        BloatwareDef { package_name: "Microsoft.XboxGamingOverlay", display_name: "Xbox Game Bar", publisher: "Microsoft", category: "Gaming" },
        BloatwareDef { package_name: "Microsoft.YourPhone", display_name: "Votre téléphone", publisher: "Microsoft", category: "Utility" },
        BloatwareDef { package_name: "Microsoft.ZuneMusic", display_name: "Groove Musique", publisher: "Microsoft", category: "Entertainment" },
        BloatwareDef { package_name: "Microsoft.ZuneVideo", display_name: "Films et TV", publisher: "Microsoft", category: "Entertainment" },
        BloatwareDef { package_name: "Microsoft.Teams", display_name: "Microsoft Teams (personnel)", publisher: "Microsoft", category: "Communication" },
        BloatwareDef { package_name: "MicrosoftTeams", display_name: "Microsoft Teams", publisher: "Microsoft", category: "Communication" },
        BloatwareDef { package_name: "Microsoft.Windows.Cortana", display_name: "Cortana", publisher: "Microsoft", category: "System" },
        BloatwareDef { package_name: "Microsoft.WindowsAlarms", display_name: "Alarmes et horloge", publisher: "Microsoft", category: "Utility" },
        BloatwareDef { package_name: "Microsoft.WindowsCamera", display_name: "Caméra", publisher: "Microsoft", category: "Utility" },
        BloatwareDef { package_name: "microsoft.windowscommunicationsapps", display_name: "Courrier et Calendrier", publisher: "Microsoft", category: "Communication" },
        BloatwareDef { package_name: "Microsoft.Todos", display_name: "Microsoft To Do", publisher: "Microsoft", category: "Productivity" },
        BloatwareDef { package_name: "Microsoft.PowerAutomateDesktop", display_name: "Power Automate", publisher: "Microsoft", category: "Productivity" },
        BloatwareDef { package_name: "Microsoft.GamingApp", display_name: "Xbox", publisher: "Microsoft", category: "Gaming" },
    ]
}

fn get_installed_packages() -> Result<Vec<(String, String)>, CommandError> {
    let result = process::run(
        "powershell.exe",
        "-NoProfile -Command \"Get-AppxPackage | ForEach-Object { \\\"$($_.Name)|$($_.Version)\\\" }\"",
        constants::TIMEOUT_VERY_LONG,
    );
    if !result.success {
        crate::utils::logger::warn(
            "bloatware",
            &format!("get_installed_packages: échec Get-AppxPackage (exit {}): {}", result.exit_code, result.stderr),
        );
        return Err(CommandError::ProcessFailed {
            command: "Get-AppxPackage".into(),
            detail: if result.stderr.trim().is_empty() {
                format!("code de sortie {}", result.exit_code)
            } else {
                result.stderr.trim().to_string()
            },
        });
    }
    Ok(result.stdout
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            match process::parse_pipe_fields(line, 2) {
                Some(parts) => Some((parts[0].to_string(), parts[1].to_string())),
                None => Some((line.to_string(), String::new())),
            }
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub async fn get_bloatware_items() -> Result<Vec<BloatwareItem>, CommandError> {
    crate::utils::blocking::read_result(get_bloatware_items_sync).await
}

pub fn get_bloatware_items_sync() -> Result<Vec<BloatwareItem>, CommandError> {
    crate::perf_timed!("get_bloatware_items", {
        let installed = get_installed_packages()?;
        let defs = known_bloatware();

        Ok(defs.into_iter()
            .map(|def| {
                let pkg_lower = def.package_name.to_lowercase();
                let matched = installed.iter().find(|(name, _)| {
                    name.to_lowercase().starts_with(&pkg_lower)
                });
                let (is_installed, version) = match matched {
                    Some((_, ver)) => (true, ver.clone()),
                    None => (false, String::new()),
                };
                BloatwareItem {
                    package_name: def.package_name.to_string(),
                    display_name: def.display_name.to_string(),
                    publisher: def.publisher.to_string(),
                    category: def.category.to_string(),
                    is_installed,
                    version,
                }
            })
            .collect())
    })
}

#[tauri::command]
#[specta::specta]
pub async fn remove_bloatware(package_name: String) -> Result<(), CommandError> {
    crate::utils::blocking::write_result(move || remove_bloatware_sync(package_name)).await
}

pub fn remove_bloatware_sync(package_name: String) -> Result<(), CommandError> {
    let known = known_bloatware();
    if !known.iter().any(|b| b.package_name == package_name) {
        return Err(CommandError::NotFound(format!("Package bloatware : {}", package_name)));
    }

    let escaped = process::escape_powershell(&package_name);
    let cmd = format!(
        "-NoProfile -Command \"Get-AppxPackage -Name '*{}*' | Remove-AppxPackage -ErrorAction SilentlyContinue\"",
        escaped
    );
    let result = process::run("powershell.exe", &cmd, constants::TIMEOUT_EXTRA);
    if !result.success {
        return Err(CommandError::ProcessFailed {
            command: format!("Remove-AppxPackage {}", package_name),
            detail: result.stderr.trim().to_string(),
        });
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn remove_all_bloatware(options: Option<crate::utils::batch::BatchOptions>) -> BatchResult {
    crate::utils::blocking::write_value(move || remove_all_bloatware_sync(options)).await
}

pub fn remove_all_bloatware_sync(options: Option<crate::utils::batch::BatchOptions>) -> BatchResult {
    crate::perf_timed!("remove_all_bloatware", {
        let opts = options.unwrap_or_default();
        let dry = opts.dry_run;
        let mut result = if dry {
            BatchResult::new_dry_run()
        } else {
            BatchResult::new()
        };
        let items = match get_bloatware_items_sync() {
            Ok(items) => items,
            Err(e) => {
                result.record_failure(format!("Impossible de lister les applications : {}", e));
                return result;
            }
        };
        for item in items.into_iter().filter(|i| i.is_installed) {
            if dry {
                result.record_preview(item.display_name);
            } else {
                result.record(
                    &item.display_name,
                    remove_bloatware_sync(item.package_name.clone()),
                );
            }
        }
        result
    })
}

pub(crate) fn ratio_score(removed: usize, total: usize) -> u32 {
    if total == 0 {
        return 0;
    }
    ((removed as f64 / total as f64) * 100.0).round() as u32
}

pub fn get_bloatware_score_sync() -> Option<u32> {
    score_from_list(get_bloatware_items_sync())
}

pub(crate) fn score_from_list<E: std::fmt::Display>(items: Result<Vec<BloatwareItem>, E>) -> Option<u32> {
    match items {
        Ok(items) => {
            let removed = items.iter().filter(|i| !i.is_installed).count();
            Some(ratio_score(removed, items.len()))
        }
        Err(e) => {
            crate::utils::logger::warn("bloatware.lecture", &format!("get_bloatware_score: liste indisponible: {}", e));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listed(is_installed: bool) -> BloatwareItem {
        BloatwareItem {
            package_name: "Microsoft.Test".into(),
            display_name: "Test".into(),
            category: "Entertainment".into(),
            publisher: String::new(),
            is_installed,
            version: String::new(),
        }
    }

    #[test]
    fn an_unreadable_list_gives_no_score_instead_of_zero() {
        assert_eq!(score_from_list::<String>(Err("PowerShell bloque".into())), None);
    }

    #[test]
    fn a_readable_list_gives_the_removed_ratio() {
        assert_eq!(score_from_list::<String>(Ok(vec![listed(false), listed(true)])), Some(50));
    }

    #[test]
    fn ratio_score_empty_returns_zero() {
        // Coherence avec les 4 autres domaines (data_protection, services, hardening, performance)
        // qui retournent 0 si total=0. Un ratio sur zero element n'a pas de sens defini,
        // on choisit 0 uniformement.
        assert_eq!(ratio_score(0, 0), 0);
    }

    #[test]
    fn ratio_all_removed_is_100() {
        assert_eq!(ratio_score(20, 20), 100);
    }

    #[test]
    fn ratio_none_removed_is_0() {
        assert_eq!(ratio_score(0, 20), 0);
    }

    #[test]
    fn ratio_half_is_50() {
        assert_eq!(ratio_score(10, 20), 50);
    }

    #[test]
    fn known_bloatware_has_items() {
        let items = known_bloatware();
        assert!(items.len() >= 20, "attendu >=20 bloatware, trouve {}", items.len());
    }

    #[test]
    fn known_bloatware_ids_unique() {
        let items = known_bloatware();
        let names: std::collections::HashSet<&str> =
            items.iter().map(|i| i.package_name).collect();
        assert_eq!(names.len(), items.len(), "package_name dupliques detectes");
    }

    #[test]
    fn known_bloatware_categories_not_empty() {
        let items = known_bloatware();
        for item in &items {
            assert!(!item.category.is_empty(), "categorie vide pour {}", item.package_name);
        }
    }
}
