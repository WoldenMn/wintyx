//! Snapshot pre-batch : point de restauration + sauvegarde des cles registre
//! critiques avant une action destructive.

use serde::Serialize;

use super::restore_point::RestorePointOutcome;
use crate::utils::batch::BatchResult;
use crate::utils::errors::CommandError;
use crate::utils::logger;

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotInfo {
    /// Vrai seulement si un nouveau point existe reellement apres l'appel.
    pub restore_point_created: bool,
    /// Vrai quand Windows a refuse silencieusement un nouveau point (limite de 24 h).
    pub restore_point_limited: bool,
    pub registry_backups: BatchResult,
    pub timestamp: String,
    pub action_name: String,
}

pub(crate) fn truncate_action_name(input: &str, max_chars: usize) -> String {
    if input.chars().count() > max_chars {
        input.chars().take(max_chars).collect()
    } else {
        input.to_string()
    }
}

/// Retourne `(cree, limite_par_windows)`. Non-admin : aucun appel, rien de cree.
/// Un echec est journalise et laisse le snapshot continuer sur la sauvegarde registre.
pub(super) fn restore_point_state<F>(is_admin: bool, create: F) -> (bool, bool)
where
    F: FnOnce() -> Result<RestorePointOutcome, CommandError>,
{
    if !is_admin {
        logger::info("backup", "Snapshot: non-admin, sauvegarde registre uniquement");
        return (false, false);
    }
    match create() {
        Ok(RestorePointOutcome::Created) => (true, false),
        Ok(RestorePointOutcome::SkippedWindowsLimit) => (false, true),
        Err(e) => {
            logger::warn("backup", &format!("Snapshot: point de restauration ignore ({})", e));
            (false, false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_action_name_short_unchanged() {
        assert_eq!(truncate_action_name("Protect all", 80), "Protect all");
    }

    #[test]
    fn truncate_action_name_long_is_cut() {
        let input = "a".repeat(200);
        let out = truncate_action_name(&input, 80);
        assert_eq!(out.chars().count(), 80);
    }

    #[test]
    fn truncate_action_name_multibyte_safe() {
        let input = "Étendre les protections de confidentialité à tous les comptes utilisateurs du système";
        let out = truncate_action_name(input, 20);
        assert!(out.chars().count() <= 20);
        assert!(out.is_char_boundary(out.len()));
    }

    #[test]
    fn snapshot_info_serializes_camel_case() {
        let info = SnapshotInfo {
            restore_point_created: true,
            restore_point_limited: false,
            registry_backups: BatchResult::default(),
            timestamp: "2026-04-23 10:00:00".to_string(),
            action_name: "Test".to_string(),
        };
        let json = serde_json::to_value(&info).unwrap();
        assert!(json.get("registryBackups").is_some());
        assert!(json.get("timestamp").is_some());
        assert!(json.get("actionName").is_some());
        assert_eq!(json.get("restorePointCreated").unwrap(), &serde_json::json!(true));
        assert_eq!(json.get("restorePointLimited").unwrap(), &serde_json::json!(false));
    }

    #[test]
    fn snapshot_info_includes_batch_result_fields() {
        let info = SnapshotInfo {
            restore_point_created: false,
            restore_point_limited: true,
            registry_backups: BatchResult::default(),
            timestamp: "now".to_string(),
            action_name: "a".to_string(),
        };
        let json = serde_json::to_value(&info).unwrap();
        let backups = json.get("registryBackups").unwrap();
        assert!(backups.get("total").is_some());
        assert!(backups.get("success").is_some());
        assert!(backups.get("failed").is_some());
        assert!(backups.get("errors").is_some());
    }

    #[test]
    fn restore_point_state_created() {
        assert_eq!(restore_point_state(true, || Ok(RestorePointOutcome::Created)), (true, false));
    }

    #[test]
    fn restore_point_state_windows_limit_is_not_created() {
        assert_eq!(
            restore_point_state(true, || Ok(RestorePointOutcome::SkippedWindowsLimit)),
            (false, true)
        );
    }

    #[test]
    fn restore_point_state_error_is_neither() {
        assert_eq!(
            restore_point_state(true, || Err(CommandError::Generic("x".into()))),
            (false, false)
        );
    }

    #[test]
    fn restore_point_state_non_admin_never_calls_create() {
        let mut called = false;
        let state = restore_point_state(false, || {
            called = true;
            Ok(RestorePointOutcome::Created)
        });
        assert_eq!(state, (false, false));
        assert!(!called);
    }
}
