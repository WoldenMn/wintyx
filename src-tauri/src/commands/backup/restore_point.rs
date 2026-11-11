//! Points de restauration systeme : lecture, creation et verification de la creation.
//!
//! `Checkpoint-Computer` sort en succes sans rien creer quand Windows a deja pris un
//! point dans les dernieres 24 h (`SystemRestorePointCreationFrequency`, 1440 min par
//! defaut) : il ne leve qu'un avertissement. Le code de sortie ne prouve donc rien, la
//! creation se verifie en comparant le plus grand `SequenceNumber` avant et apres.

use serde::Serialize;

use crate::utils::errors::CommandError;
use crate::utils::{constants, logger, process};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RestorePointInfo {
    pub description: String,
    pub creation_time: String,
    pub sequence_number: u32,
    pub restore_type: String,
}

/// Resultat d'une demande de point de restauration qui a abouti cote PowerShell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum RestorePointOutcome {
    Created,
    SkippedWindowsLimit,
}

pub(crate) fn restore_type_label(type_code: u64) -> &'static str {
    match type_code {
        0 => "Installation d'application",
        1 => "Désinstallation d'application",
        6 => "Restauration",
        7 => "Point de contrôle",
        10 => "Installation de pilote",
        12 => "Modification de paramètres",
        _ => "Autre",
    }
}

pub(super) fn parse_restore_points(stdout: &str) -> Result<Vec<RestorePointInfo>, CommandError> {
    let stdout = stdout.trim();
    if stdout.is_empty() {
        return Ok(Vec::new());
    }
    let json = serde_json::from_str::<serde_json::Value>(stdout).map_err(|e| {
        logger::warn("backup.lecture", &format!("get_restore_points: JSON invalide: {}", e));
        CommandError::Generic(format!("Réponse Get-ComputerRestorePoint illisible: {}", e))
    })?;
    let items = match json {
        serde_json::Value::Array(arr) => arr,
        obj @ serde_json::Value::Object(_) => vec![obj],
        _ => vec![],
    };
    Ok(items
        .iter()
        .map(|item| RestorePointInfo {
            description: item.get("Description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            creation_time: item.get("CreationTime").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            sequence_number: item.get("SequenceNumber").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            restore_type: restore_type_label(
                item.get("RestorePointType").and_then(|v| v.as_u64()).unwrap_or(0),
            )
            .to_string(),
        })
        .collect())
}

const LAST_SEQUENCE: &str =
    "[int](Get-ComputerRestorePoint | Measure-Object SequenceNumber -Maximum).Maximum";

/// Script PowerShell : releve le dernier `SequenceNumber`, cree le point, le releve
/// de nouveau et imprime `SEQ:<avant>:<apres>`. La description est inseree entre
/// single-quotes, contexte pour lequel `escape_powershell` est concue.
pub(super) fn build_create_script(description: &str) -> String {
    format!(
        "$ErrorActionPreference='Stop'; $before={seq}; \
         Checkpoint-Computer -Description '{desc}' -RestorePointType 'MODIFY_SETTINGS'; \
         $after={seq}; Write-Output ('SEQ:'+$before+':'+$after)",
        seq = LAST_SEQUENCE,
        desc = process::escape_powershell(description),
    )
}

pub(super) fn parse_sequence_pair(stdout: &str) -> Option<(u64, u64)> {
    stdout.lines().find_map(|line| {
        let rest = line.trim().strip_prefix("SEQ:")?;
        let (before, after) = rest.split_once(':')?;
        Some((before.trim().parse().ok()?, after.trim().parse().ok()?))
    })
}

pub(super) fn outcome_from_sequences(before: u64, after: u64) -> RestorePointOutcome {
    if after > before {
        RestorePointOutcome::Created
    } else {
        RestorePointOutcome::SkippedWindowsLimit
    }
}

pub(super) fn create(description: &str) -> Result<RestorePointOutcome, CommandError> {
    let script = build_create_script(description);
    let result = process::run(
        "powershell.exe",
        &format!("-NoProfile -Command \"{}\"", script),
        constants::TIMEOUT_RESTORE,
    );
    if !result.success {
        let detail = if !result.stderr.trim().is_empty() {
            result.stderr.trim().to_string()
        } else if !result.stdout.trim().is_empty() {
            result.stdout.trim().to_string()
        } else {
            "échec non documenté".to_string()
        };
        logger::error("backup", &format!("create_restore_point echec: {}", detail));
        return Err(CommandError::ProcessFailed {
            command: "Checkpoint-Computer".to_string(),
            detail,
        });
    }
    let Some((before, after)) = parse_sequence_pair(&result.stdout) else {
        logger::error(
            "backup",
            &format!("create_restore_point: sortie inexploitable: {}", result.stdout.trim()),
        );
        return Err(CommandError::Generic(
            "Création du point de restauration non vérifiable : sortie PowerShell illisible".to_string(),
        ));
    };
    let outcome = outcome_from_sequences(before, after);
    match outcome {
        RestorePointOutcome::Created => {
            logger::info("backup", &format!("Point de restauration cree (sequence {})", after));
        }
        RestorePointOutcome::SkippedWindowsLimit => {
            logger::warn(
                "backup",
                &format!("Point de restauration non créé : séquence inchangée ({}), limite Windows de 24 h", after),
            );
        }
    }
    Ok(outcome)
}

#[cfg(test)]
mod tests;
