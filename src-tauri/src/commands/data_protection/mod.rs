use serde::Serialize;
use crate::utils::batch::BatchResult;
use crate::utils::errors::CommandError;
use crate::utils::{admin, registry};

mod items;
use items::{build_items, ItemDef};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DataProtectionItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub is_protected: bool,
    pub recommended: bool,
}

fn get_items_internal() -> Vec<(DataProtectionItem, ItemDef)> {
    let defs = build_items();
    defs.into_iter()
        .enumerate()
        .map(|(i, def)| {
            let current = registry::get_dword(def.reg_path, def.reg_value);
            let is_protected = current.map(|v| v == def.disabled_value).unwrap_or(false);
            let item = DataProtectionItem {
                id: format!("dp_{}", i),
                name: def.name.to_string(),
                description: def.description.to_string(),
                category: def.category.to_string(),
                is_protected,
                recommended: def.in_batch(),
            };
            (item, def)
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub fn get_data_protection_items() -> Vec<DataProtectionItem> {
    crate::perf_timed!("get_data_protection_items", {
        get_items_internal().into_iter().map(|(item, _)| item).collect()
    })
}

#[tauri::command]
#[specta::specta]
pub async fn protect_item(id: String) -> Result<(), CommandError> {
    crate::utils::blocking::write_result(move || protect_item_sync(id)).await
}

pub fn protect_item_sync(id: String) -> Result<(), CommandError> {
    let items = get_items_internal();
    let found = items.iter().find(|(item, _)| item.id == id);
    match found {
        Some((_, def)) => {
            if registry::is_hklm_path(def.reg_path) {
                admin::require_admin(&format!("Protéger « {} »", def.name))?;
            }
            registry::ensure_key(def.reg_path).map_err(CommandError::Registry)?;
            registry::set_dword(def.reg_path, def.reg_value, def.disabled_value)
                .map_err(CommandError::Registry)
        }
        None => Err(CommandError::NotFound(format!("Item data protection : {}", id))),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn protect_all(options: Option<crate::utils::batch::BatchOptions>) -> BatchResult {
    crate::utils::blocking::write_value(move || protect_all_sync(options)).await
}

fn apply_protection(def: &ItemDef) -> Result<(), String> {
    registry::ensure_key(def.reg_path)
        .and_then(|_| registry::set_dword(def.reg_path, def.reg_value, def.disabled_value))
}

fn run_protect_batch(
    items: Vec<(DataProtectionItem, ItemDef)>,
    dry: bool,
    is_admin: bool,
    apply: impl Fn(&ItemDef) -> Result<(), String>,
) -> BatchResult {
    let mut result = if dry {
        BatchResult::new_dry_run()
    } else {
        BatchResult::new()
    };
    let mut needing_admin = 0usize;
    for (item, def) in items {
        if item.is_protected || !def.in_batch() {
            continue;
        }
        if dry {
            result.record_preview(def.name);
        } else if registry::is_hklm_path(def.reg_path) && !is_admin {
            needing_admin += 1;
        } else {
            result.record(def.name, apply(&def));
        }
    }
    if needing_admin > 0 {
        result.record_failure(
            CommandError::NotAdmin(format!("Protection de {} élément(s) machine", needing_admin)).friendly_message(),
        );
    }
    result
}

pub fn protect_all_sync(options: Option<crate::utils::batch::BatchOptions>) -> BatchResult {
    crate::perf_timed!("protect_all", {
        let dry = options.unwrap_or_default().dry_run;
        run_protect_batch(get_items_internal(), dry, admin::is_admin(), apply_protection)
    })
}

pub(crate) fn batch_item_ids() -> Vec<String> {
    get_items_internal()
        .into_iter()
        .filter(|(_, def)| def.in_batch())
        .map(|(item, _)| item.id)
        .collect()
}

pub(crate) fn ratio_score(protected: usize, total: usize) -> u32 {
    if total == 0 {
        return 0;
    }
    ((protected as f64 / total as f64) * 100.0).round() as u32
}

pub(crate) fn recommended_score(items: &[DataProtectionItem]) -> u32 {
    let counted: Vec<&DataProtectionItem> = items.iter().filter(|item| item.recommended).collect();
    let protected = counted.iter().filter(|item| item.is_protected).count();
    ratio_score(protected, counted.len())
}

pub fn get_data_protection_score() -> u32 {
    let items: Vec<DataProtectionItem> = get_items_internal().into_iter().map(|(item, _)| item).collect();
    recommended_score(&items)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unprotected_items() -> Vec<(DataProtectionItem, ItemDef)> {
        build_items()
            .into_iter()
            .enumerate()
            .map(|(i, def)| {
                let item = DataProtectionItem {
                    id: format!("dp_{}", i),
                    name: def.name.to_string(),
                    description: def.description.to_string(),
                    category: def.category.to_string(),
                    is_protected: false,
                    recommended: def.in_batch(),
                };
                (item, def)
            })
            .collect()
    }

    fn batch_split() -> (usize, usize) {
        let defs = build_items();
        let batch: Vec<_> = defs.iter().filter(|d| d.in_batch()).collect();
        let machine = batch.iter().filter(|d| registry::is_hklm_path(d.reg_path)).count();
        (machine, batch.len() - machine)
    }

    #[test]
    fn the_catalogue_mixes_machine_and_user_items() {
        let (machine, user) = batch_split();
        assert!(machine > 0 && user > 0, "{machine} machine, {user} utilisateur");
    }

    #[test]
    fn without_admin_rights_machine_items_are_not_attempted_and_one_not_admin_line_is_recorded() {
        let (machine, user) = batch_split();
        let attempted = std::cell::RefCell::new(Vec::new());
        let result = run_protect_batch(unprotected_items(), false, false, |def| {
            attempted.borrow_mut().push(def.reg_path);
            Ok(())
        });
        let attempted = attempted.into_inner();
        assert_eq!(attempted.len(), user);
        assert!(attempted.iter().all(|path| !registry::is_hklm_path(path)));
        assert_eq!((result.success, result.failed), (user, 1));
        assert!(result.errors[0].contains("administrateur"), "{:?}", result.errors);
        assert!(result.errors[0].contains(&machine.to_string()), "{:?}", result.errors);
    }

    #[test]
    fn with_admin_rights_every_batch_item_is_attempted_and_nothing_is_reported() {
        let (machine, user) = batch_split();
        let result = run_protect_batch(unprotected_items(), false, true, |_| Ok(()));
        assert_eq!((result.success, result.failed), (machine + user, 0));
    }

    #[test]
    fn a_dry_run_lists_everything_without_applying_or_asking_for_rights() {
        let (machine, user) = batch_split();
        let result = run_protect_batch(unprotected_items(), true, false, |_| panic!("aucune ecriture en simulation"));
        assert!(result.dry_run);
        assert_eq!(result.preview_items.len(), machine + user);
        assert_eq!(result.failed, 0);
    }

    #[test]
    fn protecting_a_machine_item_without_admin_rights_is_refused_before_any_write() {
        if admin::is_admin() {
            return;
        }
        let machine_item = unprotected_items()
            .into_iter()
            .find(|(_, def)| registry::is_hklm_path(def.reg_path))
            .map(|(item, _)| item.id)
            .expect("un item machine");
        let err = protect_item_sync(machine_item).unwrap_err();
        assert!(matches!(err, CommandError::NotAdmin(_)), "{err:?}");
    }

    #[test]
    fn batch_ids_skip_smartscreen_and_nothing_else() {
        let all = get_items_internal();
        let ids = batch_item_ids();
        assert_eq!(ids.len(), all.len() - 1);
        let smartscreen = all
            .iter()
            .find(|(_, def)| def.reg_value == "EnableSmartScreen")
            .map(|(item, _)| item.id.clone())
            .expect("SmartScreen reste un item protegeable seul");
        assert!(!ids.contains(&smartscreen));
        assert!(all.iter().all(|(item, _)| item.id == smartscreen || ids.contains(&item.id)));
    }

    #[test]
    fn protect_all_preview_never_lists_smartscreen() {
        let opts = Some(crate::utils::batch::BatchOptions { dry_run: true });
        let preview = protect_all_sync(opts);
        assert!(preview.dry_run);
        assert!(
            preview.preview_items.iter().all(|name| !name.contains("SmartScreen")),
            "{:?}",
            preview.preview_items
        );
    }

    fn item(is_protected: bool, recommended: bool) -> DataProtectionItem {
        DataProtectionItem {
            id: "dp_test".into(),
            name: "test".into(),
            description: String::new(),
            category: "Telemetrie".into(),
            is_protected,
            recommended,
        }
    }

    #[test]
    fn score_ignores_optional_items() {
        let items = [item(true, true), item(false, true), item(false, false)];
        assert_eq!(recommended_score(&items), 50);
    }

    #[test]
    fn smartscreen_is_optional_and_every_other_item_recommended() {
        let optional: Vec<&'static str> = build_items()
            .into_iter()
            .filter(|def| !def.in_batch())
            .map(|def| def.reg_value)
            .collect();
        assert_eq!(optional, vec!["EnableSmartScreen"]);
    }

    #[test]
    fn ratio_score_empty() {
        assert_eq!(ratio_score(0, 0), 0);
    }

    #[test]
    fn ratio_score_all_protected() {
        assert_eq!(ratio_score(15, 15), 100);
    }

    #[test]
    fn ratio_score_none_protected() {
        assert_eq!(ratio_score(0, 15), 0);
    }

    #[test]
    fn ratio_score_partial() {
        assert_eq!(ratio_score(7, 14), 50);
    }
}
