use serde::Serialize;
use crate::utils::{constants, process, registry};
use crate::utils::errors::CommandError;

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WindowsUpdate {
    pub title: String,
    pub kb_article_id: String,
    pub installed_on: String,
    pub description: String,
    pub category: String,
    pub is_important: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn get_update_history() -> Result<Vec<WindowsUpdate>, CommandError> {
    crate::utils::blocking::read_result(get_update_history_sync).await
}

pub fn get_update_history_sync() -> Result<Vec<WindowsUpdate>, CommandError> {
    let result = process::run(
        "powershell.exe",
        "-NoProfile -Command \"Get-HotFix | Sort-Object InstalledOn -Descending | Select-Object HotFixID, Description, @{N='InstalledOn';E={if($_.InstalledOn){$_.InstalledOn.ToString('yyyy-MM-dd')}else{''}}} | ConvertTo-Json\"",
        constants::TIMEOUT_VERY_LONG,
    );

    if !result.success {
        crate::utils::logger::warn(
            "updates",
            &format!("get_update_history: echec Get-HotFix (exit {}): {}", result.exit_code, result.stderr),
        );
        return Err(CommandError::ProcessFailed {
            command: "Get-HotFix".into(),
            detail: if result.stderr.trim().is_empty() {
                format!("code de sortie {}", result.exit_code)
            } else {
                result.stderr.trim().to_string()
            },
        });
    }

    let stdout = result.stdout.trim();
    if stdout.is_empty() {
        return Ok(Vec::new());
    }

    parse_hotfixes(stdout)
}

fn parse_hotfixes(stdout: &str) -> Result<Vec<WindowsUpdate>, CommandError> {
    let json = serde_json::from_str::<serde_json::Value>(stdout).map_err(|e| {
        crate::utils::logger::warn("updates", &format!("get_update_history: JSON Get-HotFix invalide: {}", e));
        CommandError::Generic(format!("Reponse Get-HotFix illisible: {}", e))
    })?;
    let items = match &json {
        serde_json::Value::Array(arr) => arr.clone(),
        obj @ serde_json::Value::Object(_) => vec![obj.clone()],
        _ => vec![],
    };

    let mut updates = Vec::new();
    for item in &items {
        let text = |key: &str| item.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let hotfix_id = text("HotFixID");
        let description = text("Description");
        let category = classify_update(&description);
        updates.push(WindowsUpdate {
            title: format!("{} ({})", description, hotfix_id),
            kb_article_id: hotfix_id,
            installed_on: text("InstalledOn"),
            is_important: category == "Sécurité",
            category: category.to_string(),
            description,
        });
    }
    Ok(updates)
}

#[tauri::command]
#[specta::specta]
pub async fn get_windows_build() -> Result<String, CommandError> {
    crate::utils::blocking::read_result(get_windows_build_sync).await
}

const NT_VERSION_KEY: &str = r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion";

fn format_windows_build(display_version: &str, build_number: &str, ubr: Option<u32>) -> Option<String> {
    if display_version.is_empty() || build_number.is_empty() {
        return None;
    }
    let ubr_str = ubr.map(|v| format!(".{}", v)).unwrap_or_default();
    Some(format!("{} (Build {}{})", display_version, build_number, ubr_str))
}

pub fn get_windows_build_sync() -> Result<String, CommandError> {
    let display_version = registry::get_string(NT_VERSION_KEY, "DisplayVersion").unwrap_or_default();
    let build_number = registry::get_string(NT_VERSION_KEY, "CurrentBuildNumber").unwrap_or_default();
    let ubr = registry::get_dword(NT_VERSION_KEY, "UBR");
    format_windows_build(&display_version, &build_number, ubr).ok_or_else(|| {
        crate::utils::logger::warn("updates", "get_windows_build: DisplayVersion ou CurrentBuildNumber illisible");
        CommandError::Registry(format!("{} : version de Windows illisible", NT_VERSION_KEY))
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_last_update_date() -> Result<String, CommandError> {
    crate::utils::blocking::read_result(get_last_update_date_sync).await
}

fn latest_install_date(updates: &[WindowsUpdate]) -> String {
    updates
        .iter()
        .map(|u| u.installed_on.as_str())
        .find(|date| !date.is_empty())
        .unwrap_or_default()
        .to_string()
}

pub fn get_last_update_date_sync() -> Result<String, CommandError> {
    get_update_history_sync().map(|updates| latest_install_date(&updates))
}

fn classify_update(description: &str) -> &'static str {
    let lower = description.to_lowercase();
    if lower.contains("security") {
        "Sécurité"
    } else if lower.contains("update") {
        "Mise à jour"
    } else {
        "Autre"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_security() {
        assert_eq!(classify_update("Security Update for Windows"), "Sécurité");
        assert_eq!(classify_update("SECURITY PATCH"), "Sécurité");
    }

    #[test]
    fn classify_regular_update() {
        assert_eq!(classify_update("Cumulative Update KB1234"), "Mise à jour");
    }

    #[test]
    fn classify_other() {
        assert_eq!(classify_update("Hotfix"), "Autre");
        assert_eq!(classify_update(""), "Autre");
    }

    #[test]
    fn live_history_has_iso_installation_dates() {
        let updates = get_update_history_sync().expect("Get-HotFix");
        if updates.is_empty() {
            return;
        }
        assert!(updates.iter().any(|u| !u.installed_on.is_empty()), "aucune date lue");
        for u in updates.iter().filter(|u| !u.installed_on.is_empty()) {
            assert!(
                chrono::NaiveDate::parse_from_str(&u.installed_on, "%Y-%m-%d").is_ok(),
                "date non ISO : {:?}",
                u.installed_on
            );
        }
    }

    #[test]
    fn parses_projected_hotfixes_with_blank_date() {
        let json = r#"[{"HotFixID":"KB1","Description":"Security Update","InstalledOn":"2026-09-23"},{"HotFixID":"KB2","Description":"Update","InstalledOn":""}]"#;
        let updates = parse_hotfixes(json).unwrap();
        assert_eq!(updates[0].installed_on, "2026-09-23");
        assert_eq!(updates[1].installed_on, "");
        assert!(updates[0].is_important);
    }

    fn update(date: &str) -> WindowsUpdate {
        WindowsUpdate {
            title: String::new(),
            kb_article_id: String::new(),
            installed_on: date.to_string(),
            description: String::new(),
            category: String::new(),
            is_important: false,
        }
    }

    #[test]
    fn build_needs_both_version_and_number() {
        assert_eq!(format_windows_build("24H2", "26100", Some(1)).as_deref(), Some("24H2 (Build 26100.1)"));
        assert_eq!(format_windows_build("24H2", "26100", None).as_deref(), Some("24H2 (Build 26100)"));
        assert_eq!(format_windows_build("", "26100", Some(1)), None);
        assert_eq!(format_windows_build("24H2", "", Some(1)), None);
    }

    #[test]
    fn last_update_date_skips_blank_dates() {
        assert_eq!(latest_install_date(&[update(""), update("2026-09-23"), update("2026-09-01")]), "2026-09-23");
    }

    #[test]
    fn last_update_date_is_empty_without_any_date() {
        assert_eq!(latest_install_date(&[update("")]), "");
        assert_eq!(latest_install_date(&[]), "");
    }
}
