//! Scanners dont la source est le registre Windows (entrees de demarrage,
//! cles autorun Winlogon/AppInit, etat du pare-feu).

use crate::utils::{process, registry};

use crate::commands::threat_scanner::{ScanOutcome, ThreatItem};
use super::{expand_env_vars, extract_exe_path};

pub(crate) fn scan_startup_entries() -> ScanOutcome {
    let mut threats = Vec::new();
    let mut scanned = 0u32;

    let run_keys = [
        "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
        "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
    ];

    for key_path in &run_keys {
        let value_names = registry::get_value_names(key_path);
        for name in &value_names {
            scanned += 1;
            if let Some(value) = registry::get_string(key_path, name) {
                let exe_path = expand_env_vars(&extract_exe_path(&value));
                // Un % restant = variable d'env non resolue : impossible de conclure
                // a un executable introuvable, donc on ne flague pas (bug-rust-03).
                if !exe_path.is_empty()
                    && !exe_path.contains('%')
                    && !std::path::Path::new(&exe_path).exists()
                {
                    threats.push(ThreatItem {
                        name: name.clone(),
                        category: "Programmes au démarrage".into(),
                        severity: "high".into(),
                        description: format!(
                            "Exécutable introuvable: {}",
                            exe_path
                        ),
                        path: format!("{}\\{}", key_path, name),
                        can_fix: true,
                        fix_id: format!("startup_{}_{}", key_path.replace('\\', "_"), name.replace(' ', "_")),
                    });
                }
            }
        }
    }

    Ok((scanned, threats))
}

fn normalized_autorun_value(value: &str, system_root: &str) -> String {
    let expanded = value
        .trim()
        .to_lowercase()
        .replace("%systemroot%", &system_root.to_lowercase());
    expanded
        .strip_suffix(',')
        .unwrap_or(&expanded)
        .trim_end()
        .to_string()
}

pub(super) fn winlogon_value_is_default(value: &str, expected: &str, system_root: &str) -> bool {
    normalized_autorun_value(value, system_root) == normalized_autorun_value(expected, system_root)
}

pub(crate) fn scan_autorun_registry() -> ScanOutcome {
    let mut threats = Vec::new();
    let mut scanned = 0u32;

    let system_root = process::system_root().to_string_lossy().trim_end_matches('\\').to_string();
    let default_userinit = format!("{}\\system32\\userinit.exe,", system_root);
    let checks: Vec<(&str, &str, &str)> = vec![
        (
            "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
            "Shell",
            "explorer.exe",
        ),
        (
            "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
            "Userinit",
            default_userinit.as_str(),
        ),
        (
            "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Windows",
            "AppInit_DLLs",
            "",
        ),
        (
            "HKLM\\Software\\Wow6432Node\\Microsoft\\Windows NT\\CurrentVersion\\Windows",
            "AppInit_DLLs",
            "",
        ),
    ];

    for (path, name, expected) in &checks {
        scanned += 1;
        if let Some(value) = registry::get_string(path, name) {
            let value_lower = value.to_lowercase().trim().to_string();

            if *name == "AppInit_DLLs" {
                if !value_lower.is_empty() {
                    threats.push(ThreatItem {
                        name: format!("{}\\{}", path, name),
                        category: "Clés de registre (démarrage auto)".into(),
                        severity: "critical".into(),
                        description: format!(
                            "AppInit_DLLs contient des valeurs: {}",
                            value
                        ),
                        path: path.to_string(),
                        can_fix: true,
                        fix_id: format!(
                            "reg_{}_{}",
                            path.replace('\\', "_"),
                            name
                        ),
                    });
                }
            } else if !expected.is_empty() && !winlogon_value_is_default(&value, expected, &system_root) {
                threats.push(ThreatItem {
                    name: format!("{}\\{}", path, name),
                    category: "Clés de registre (démarrage auto)".into(),
                    severity: "critical".into(),
                    description: format!(
                        "{} modifié: attendu '{}', trouvé '{}'",
                        name, expected, value
                    ),
                    path: path.to_string(),
                    can_fix: false,
                    fix_id: format!(
                        "reg_{}_{}",
                        path.replace('\\', "_"),
                        name
                    ),
                });
            }
        }
    }

    Ok((scanned, threats))
}

pub(crate) fn scan_firewall() -> ScanOutcome {
    let mut threats = Vec::new();
    let mut scanned = 0u32;

    let profiles = [
        ("Domaine", "DomainProfile"),
        ("Privé", "StandardProfile"),
        ("Public", "PublicProfile"),
    ];

    let base_path = "HKLM\\SYSTEM\\CurrentControlSet\\Services\\SharedAccess\\Parameters\\FirewallPolicy";

    for (display_name, profile_key) in &profiles {
        scanned += 1;
        let full_path = format!("{}\\{}", base_path, profile_key);
        let enabled = registry::get_dword(&full_path, "EnableFirewall");

        match enabled {
            Some(0) => {
                threats.push(ThreatItem {
                    name: format!("Pare-feu {} désactivé", display_name),
                    category: "Pare-feu".into(),
                    severity: "critical".into(),
                    description: format!(
                        "Le profil pare-feu '{}' est désactivé",
                        display_name
                    ),
                    path: full_path,
                    can_fix: true,
                    fix_id: format!("fw_{}", profile_key),
                });
            }
            None => {
                threats.push(ThreatItem {
                    name: format!("Pare-feu {} inconnu", display_name),
                    category: "Pare-feu".into(),
                    severity: "high".into(),
                    description: format!(
                        "Impossible de lire l'état du profil '{}'",
                        display_name
                    ),
                    path: full_path,
                    can_fix: false,
                    fix_id: format!("fw_{}", profile_key),
                });
            }
            _ => {}
        }
    }

    Ok((scanned, threats))
}

#[cfg(test)]
mod tests;
