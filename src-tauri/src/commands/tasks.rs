use serde::Serialize;
use crate::utils::admin;
use crate::utils::constants;
use crate::utils::errors::CommandError;
use crate::utils::process;

mod analysis;
pub(crate) use analysis::{analyze_suspicious, suspicious_severity};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTaskInfo {
    pub name: String,
    pub path: String,
    pub state: String,
    pub author: String,
    pub description: String,
    pub last_run_time: String,
    pub next_run_time: String,
    pub action_path: String,
    pub action_arguments: String,
    /// Indice fort : technique d'exécution ou de téléchargement détournée (commande encodée,
    /// mshta, certutil, URL...). Seul ce niveau compte dans le badge de la sidebar.
    pub is_suspicious: bool,
    /// Indice faible, informatif : auteur absent ou inconnu, PowerShell ou cmd, dossier
    /// inscriptible. Décrit presque toute tâche créée par un utilisateur ou un logiciel tiers.
    pub needs_review: bool,
    /// Motif de l'un ou l'autre niveau.
    pub suspicious_reason: String,
}

#[tauri::command]
#[specta::specta]
pub async fn get_scheduled_tasks() -> Result<Vec<ScheduledTaskInfo>, CommandError> {
    crate::utils::blocking::read_result(get_scheduled_tasks_sync).await
}

pub fn get_scheduled_tasks_sync() -> Result<Vec<ScheduledTaskInfo>, CommandError> {
    crate::perf_timed!("get_scheduled_tasks", {
        let result = process::run(
            "powershell.exe",
            r#"-NoProfile -Command "Get-ScheduledTask | Where-Object { $_.State -ne 'Disabled' } | ForEach-Object { $action = $_.Actions | Select-Object -First 1; $info = $_ | Get-ScheduledTaskInfo -ErrorAction SilentlyContinue; $lr = if ($info -and $info.LastRunTime -and $info.LastRunTime.Year -gt 1) { $info.LastRunTime.ToString('yyyy-MM-dd HH:mm:ss') } else { '' }; $nr = if ($info -and $info.NextRunTime -and $info.NextRunTime.Year -gt 1) { $info.NextRunTime.ToString('yyyy-MM-dd HH:mm:ss') } else { '' }; $exe = if ($action) { $action.Execute } else { '' }; $args = if ($action) { $action.Arguments } else { '' }; [PSCustomObject]@{ Name=$_.TaskName; Path=$_.TaskPath; State=$_.State.ToString(); Author=$_.Author; Description=$_.Description; ActionExe=$exe; ActionArgs=$args; LastRun=$lr; NextRun=$nr } } | ConvertTo-Json -Depth 3""#,
            constants::TIMEOUT_EXTRA,
        );

        if !result.success {
            crate::utils::logger::warn(
                "tasks",
                &format!(
                    "get_scheduled_tasks: échec PowerShell (exit {}): {}",
                    result.exit_code, result.stderr
                ),
            );
            return Err(CommandError::ProcessFailed {
                command: "Get-ScheduledTask".into(),
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

        let json = serde_json::from_str::<serde_json::Value>(stdout).map_err(|e| {
            crate::utils::logger::warn(
                "tasks",
                &format!("get_scheduled_tasks: JSON ScheduledTask invalide: {}", e),
            );
            CommandError::Generic(format!("Réponse Get-ScheduledTask illisible: {}", e))
        })?;

        let mut tasks = Vec::new();
        {
            let items = match &json {
                serde_json::Value::Array(arr) => arr.clone(),
                obj @ serde_json::Value::Object(_) => vec![obj.clone()],
                _ => vec![],
            };

            for item in &items {
                let mut task = ScheduledTaskInfo {
                    name: item.get("Name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    path: item.get("Path").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    state: item.get("State").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    author: item.get("Author").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    description: item.get("Description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    last_run_time: item.get("LastRun").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    next_run_time: item.get("NextRun").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    action_path: item.get("ActionExe").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    action_arguments: item.get("ActionArgs").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    is_suspicious: false,
                    needs_review: false,
                    suspicious_reason: String::new(),
                };
                analyze_suspicious(&mut task);
                tasks.push(task);
            }
        }

        Ok(tasks)
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_suspicious_tasks() -> Result<Vec<ScheduledTaskInfo>, CommandError> {
    crate::utils::blocking::read_result(get_suspicious_tasks_sync).await
}

pub fn get_suspicious_tasks_sync() -> Result<Vec<ScheduledTaskInfo>, CommandError> {
    crate::perf_timed!("get_suspicious_tasks", {
        Ok(get_scheduled_tasks_sync()?.into_iter().filter(|t| t.is_suspicious).collect())
    })
}

/// Valide un nom/chemin de tache planifiee avant interpolation dans schtasks.exe.
///
/// Allowlist (defense-in-depth, coherent avec registry::sanitize_name) : les
/// chemins de taches Windows contiennent alphanumerique + `\ - _ . espace`.
/// Rejette l'Unicode (confusables) et tout metacaractere shell (`" & | ; $ ` < >`),
/// remplacant l'ancienne blocklist par une allowlist stricte.
fn sanitize_task_name(name: &str) -> Result<String, CommandError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CommandError::InvalidInput("Nom de tâche vide".into()));
    }
    if trimmed.len() > 256 {
        return Err(CommandError::InvalidInput("Nom de tâche trop long".into()));
    }
    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || " \\-_.".contains(c))
    {
        Ok(trimmed.to_string())
    } else {
        Err(CommandError::InvalidInput(format!(
            "Nom de tâche invalide : {}",
            trimmed
        )))
    }
}

#[tauri::command]
#[specta::specta]
pub async fn disable_scheduled_task(task_name: String) -> Result<(), CommandError> {
    crate::utils::blocking::write_result(move || disable_scheduled_task_sync(task_name)).await
}

pub fn disable_scheduled_task_sync(task_name: String) -> Result<(), CommandError> {
    admin::require_admin("Désactiver une tâche planifiée")?;
    let safe_name = sanitize_task_name(&task_name)?;

    let cmd = format!("/Change /TN \"{}\" /DISABLE", safe_name);
    let result = process::run("schtasks.exe", &cmd, constants::TIMEOUT_MEDIUM);
    if result.success {
        Ok(())
    } else {
        Err(CommandError::ProcessFailed {
            command: format!("schtasks /Change /TN {} /DISABLE", safe_name),
            detail: result.stderr.trim().to_string(),
        })
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_suspicious_task_count() -> Result<u32, CommandError> {
    crate::utils::blocking::read_result(get_suspicious_task_count_sync).await
}

pub fn get_suspicious_task_count_sync() -> Result<u32, CommandError> {
    suspicious_count(get_suspicious_tasks_sync())
}

fn suspicious_count(tasks: Result<Vec<ScheduledTaskInfo>, CommandError>) -> Result<u32, CommandError> {
    tasks.map(|t| t.len() as u32)
}

#[cfg(test)]
mod tests;
