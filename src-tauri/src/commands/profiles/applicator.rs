//! Orchestration de l'application d'un profil sur les 6 domaines Wintyx.
//!
//! Découple la commande Tauri `apply_profile` (mod.rs) de la logique d'application
//! par domaine. Fix H1 audit v2.7.9 : avant ce module, `apply_profile` couplait
//! directement les 6 commandes Tauri (`protect_item`, `apply_hardening_fix`, etc.),
//! créant une dette de découplage non visible par le système de types.
//!
//! Helper `apply_section` factorise le pattern `dry-run preview / execute & record`.

use crate::commands::{bloatware, data_protection, hardening, performance, services, startup};
use crate::utils::batch::BatchResult;

use super::presets;
use super::Profile;

/// Applique les items d'un domaine au resultat batch.
///
/// `op` : closure qui execute l'operation pour 1 item.
/// `label` : prefix du message preview en mode dry-run (ex: "Protection", "Service").
fn apply_section<F>(
    ids: &[String],
    label: &str,
    dry: bool,
    result: &mut BatchResult,
    mut op: F,
) where
    F: FnMut(&str) -> Result<(), String>,
{
    for id in ids {
        if dry {
            result.record_preview(format!("{} : {}", label, id));
        } else {
            match op(id) {
                Ok(_) => result.record_success(),
                Err(e) => result.record_failure(format!("{}: {}", id, e)),
            }
        }
    }
}

/// Applique un profil sur les 6 domaines (data_protection, hardening, performance,
/// services, bloatware, startup) et retourne le `BatchResult` consolide.
///
/// `dry` : si vrai, n'execute aucune commande (preview only).
pub(super) fn apply_profile_items(profile: &Profile, dry: bool) -> BatchResult {
    let mut result = if dry {
        BatchResult::new_dry_run()
    } else {
        BatchResult::new()
    };

    apply_section(&profile.data_protection_ids, "Protection", dry, &mut result, |id| {
        data_protection::protect_item_sync(id.to_string()).map_err(|e| e.to_string())
    });

    apply_section(&profile.hardening_ids, "Durcissement", dry, &mut result, |id| {
        hardening::apply_hardening_fix_sync(id.to_string()).map_err(|e| e.to_string())
    });

    apply_section(&profile.performance_ids, "Performance", dry, &mut result, |id| {
        performance::optimize_item_sync(id.to_string()).map_err(|e| e.to_string())
    });

    apply_section(&profile.services_ids, "Service", dry, &mut result, |id| {
        services::stop_service_sync(id.to_string()).map_err(|e| e.to_string())
    });

    apply_section(&profile.bloatware_ids, "Bloatware", dry, &mut result, |id| {
        bloatware::remove_bloatware_sync(id.to_string()).map_err(|e| e.to_string())
    });

    apply_section(&profile.startup_ids, "Démarrage", dry, &mut result, |id| {
        let (name, location) = presets::parse_startup_id(id);
        startup::disable_startup_item_sync(name, location).map_err(|e| e.to_string())
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::batch::BatchResult;

    #[test]
    fn apply_section_dry_run_records_preview_without_invoking_op() {
        let ids = vec!["item1".to_string(), "item2".to_string()];
        let mut result = BatchResult::new_dry_run();
        let mut invoked_ids: Vec<String> = Vec::new();

        apply_section(&ids, "Test", true, &mut result, |id| {
            invoked_ids.push(id.to_string());
            Ok(())
        });

        // En dry-run, op n'est jamais invoque
        assert!(invoked_ids.is_empty(), "op must not be called in dry-run mode");
        // Et preview_items contient les labels formates
        assert_eq!(result.preview_items.len(), 2);
        assert!(result.preview_items[0].contains("item1"));
        assert!(result.preview_items[1].contains("item2"));
        assert!(result.preview_items[0].contains("Test"));
    }

    #[test]
    fn apply_section_execute_continues_on_error_and_collects() {
        let ids = vec![
            "ok1".to_string(),
            "fail".to_string(),
            "ok2".to_string(),
        ];
        let mut result = BatchResult::new();

        apply_section(&ids, "Test", false, &mut result, |id| {
            if id == "fail" {
                Err("simulated error".to_string())
            } else {
                Ok(())
            }
        });

        // BatchResult v2.2.0 utilise `success` (pas `success_count`)
        // et `errors` (Vec<String>).
        assert_eq!(result.success, 2);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("fail"));
        assert!(result.errors[0].contains("simulated"));
    }
}

