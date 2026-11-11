use serde::Serialize;
use crate::utils::batch::BatchResult;
use crate::utils::{admin, constants, process, registry};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub is_optimized: bool,
}

struct PerfDef {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    category: &'static str,
    reg_path: &'static str,
    reg_value: &'static str,
    optimized_value: i32,
    fix_type: &'static str,
    fix_target: &'static str,
    check_command: &'static str,
    check_contains: &'static str,
}

fn build_items() -> Vec<PerfDef> {
    vec![
        PerfDef { id: "visual_effects", name: "Effets visuels", description: "Ajuster pour meilleures performances", category: "Affichage", reg_path: r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects", reg_value: "VisualFXSetting", optimized_value: 2, fix_type: "Registry", fix_target: "", check_command: "", check_contains: "" },
        PerfDef { id: "animations", name: "Animations de fenêtres", description: "Désactiver les animations d'ouverture/fermeture", category: "Affichage", reg_path: r"HKCU\Control Panel\Desktop\WindowMetrics", reg_value: "MinAnimate", optimized_value: 0, fix_type: "RegistryString", fix_target: "", check_command: "", check_contains: "" },
        PerfDef { id: "transparency", name: "Transparence", description: "Désactiver les effets de transparence", category: "Affichage", reg_path: r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize", reg_value: "EnableTransparency", optimized_value: 0, fix_type: "Registry", fix_target: "", check_command: "", check_contains: "" },
        PerfDef { id: "game_mode", name: "Mode Jeu", description: "Activer le mode jeu Windows", category: "Performances", reg_path: r"HKCU\SOFTWARE\Microsoft\GameBar", reg_value: "AllowAutoGameMode", optimized_value: 1, fix_type: "Registry", fix_target: "", check_command: "", check_contains: "" },
        PerfDef { id: "prefetch", name: "Prefetch / Superfetch", description: "Désactiver le préchargement automatique", category: "Performances", reg_path: r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters", reg_value: "EnablePrefetcher", optimized_value: 0, fix_type: "Registry", fix_target: "", check_command: "", check_contains: "" },
        PerfDef { id: "power_throttling", name: "Power Throttling", description: "Désactiver la limitation de puissance", category: "Performances", reg_path: r"HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling", reg_value: "PowerThrottlingOff", optimized_value: 1, fix_type: "Registry", fix_target: "", check_command: "", check_contains: "" },
        PerfDef { id: "nagle", name: "Algorithme de Nagle", description: "Désactiver Nagle pour réduire la latence réseau", category: "Reseau", reg_path: r"HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters", reg_value: "TcpNoDelay", optimized_value: 1, fix_type: "Registry", fix_target: "", check_command: "", check_contains: "" },
        PerfDef { id: "network_throttling", name: "Network Throttling", description: "Désactiver le throttling réseau multimédia", category: "Reseau", reg_path: r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile", reg_value: "NetworkThrottlingIndex", optimized_value: -1, fix_type: "Registry", fix_target: "", check_command: "", check_contains: "" },
        PerfDef { id: "high_perf_power", name: "Plan d'énergie Performances", description: "Activer le plan haute performance", category: "Energie", reg_path: "", reg_value: "", optimized_value: 0, fix_type: "Command", fix_target: "powercfg /setactive 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c", check_command: "/getactivescheme", check_contains: "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c" },
        PerfDef { id: "onedrive_startup", name: "OneDrive au démarrage", description: "Désactiver le lancement automatique de OneDrive", category: "Demarrage", reg_path: r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run", reg_value: "OneDrive", optimized_value: 0, fix_type: "DeleteValue", fix_target: "", check_command: "", check_contains: "" },
    ]
}

fn check_item(def: &PerfDef) -> bool {
    if !def.check_command.is_empty() && !def.check_contains.is_empty() {
        let result = process::run("powercfg", def.check_command, constants::TIMEOUT_SHORT);
        return result.success && result.stdout.to_lowercase().contains(&def.check_contains.to_lowercase());
    }

    match def.fix_type {
        "Registry" => {
            registry::get_dword(def.reg_path, def.reg_value)
                .map(|v| v as i32 == def.optimized_value)
                .unwrap_or(false)
        }
        "RegistryString" => {
            registry::get_string(def.reg_path, def.reg_value)
                .map(|v| v == def.optimized_value.to_string())
                .unwrap_or(false)
        }
        "DeleteValue" => {
            !registry::key_exists(def.reg_path, def.reg_value)
        }
        "Command" => false,
        _ => false,
    }
}

fn needs_admin(def: &PerfDef) -> bool {
    matches!(def.fix_type, "Registry" | "RegistryString" | "DeleteValue") && registry::is_hklm_path(def.reg_path)
}

fn get_items_internal() -> Vec<(PerformanceItem, PerfDef)> {
    let defs = build_items();
    defs.into_iter()
        .map(|def| {
            let is_optimized = check_item(&def);
            let item = PerformanceItem {
                id: def.id.to_string(),
                name: def.name.to_string(),
                description: def.description.to_string(),
                category: def.category.to_string(),
                is_optimized,
            };
            (item, def)
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub async fn get_performance_items() -> Vec<PerformanceItem> {
    crate::utils::blocking::read_value(get_performance_items_sync).await
}

pub fn get_performance_items_sync() -> Vec<PerformanceItem> {
    crate::perf_timed!("get_performance_items", {
        get_items_internal().into_iter().map(|(item, _)| item).collect()
    })
}

#[tauri::command]
#[specta::specta]
pub async fn optimize_item(id: String) -> Result<(), crate::utils::errors::CommandError> {
    crate::utils::blocking::write_result(move || optimize_item_sync(id)).await
}

pub fn optimize_item_sync(id: String) -> Result<(), crate::utils::errors::CommandError> {
    use crate::utils::errors::CommandError;
    let items = get_items_internal();
    let (_, def) = items
        .iter()
        .find(|(item, _)| item.id == id)
        .ok_or_else(|| CommandError::NotFound(format!("Item performance : {}", id)))?;
    if needs_admin(def) {
        admin::require_admin(&format!("Optimiser « {} »", def.name))?;
    }
    apply_optimization(def)?;
    Ok(())
}

fn apply_optimization(def: &PerfDef) -> Result<(), String> {
    match def.fix_type {
        "Registry" => {
            registry::ensure_key(def.reg_path)?;
            registry::set_dword(def.reg_path, def.reg_value, def.optimized_value as u32)
        }
        "RegistryString" => {
            registry::ensure_key(def.reg_path)?;
            registry::set_string(def.reg_path, def.reg_value, &def.optimized_value.to_string())
        }
        "DeleteValue" => {
            registry::delete_value(def.reg_path, def.reg_value)
        }
        "Command" => {
            if !def.fix_target.is_empty() {
                let parts: Vec<&str> = def.fix_target.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let out = process::run(parts[0], parts[1], constants::TIMEOUT_SHORT);
                    if !out.success {
                        crate::utils::logger::warn(
                            "performance",
                            &format!("apply_optimization Command '{}': échec (exit {}): {}", def.fix_target, out.exit_code, out.stderr),
                        );
                        return Err(format!(
                            "Échec de '{}' (code {})",
                            def.fix_target, out.exit_code
                        ));
                    }
                }
            }
            Ok(())
        }
        other => Err(format!("Type de correctif inconnu : {}", other)),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn optimize_all(options: Option<crate::utils::batch::BatchOptions>) -> BatchResult {
    crate::utils::blocking::write_value(move || optimize_all_sync(options)).await
}

fn run_optimize_batch(
    items: Vec<(PerformanceItem, PerfDef)>,
    dry: bool,
    is_admin: bool,
    apply: impl Fn(&PerfDef) -> Result<(), String>,
) -> BatchResult {
    let mut result = if dry {
        BatchResult::new_dry_run()
    } else {
        BatchResult::new()
    };
    let mut needing_admin = 0usize;
    for (item, def) in items {
        if item.is_optimized {
            continue;
        }
        if dry {
            result.record_preview(def.name);
        } else if needs_admin(&def) && !is_admin {
            needing_admin += 1;
        } else {
            result.record(def.id, apply(&def));
        }
    }
    if needing_admin > 0 {
        result.record_failure(
            crate::utils::errors::CommandError::NotAdmin(format!("Optimisation de {} réglage(s) machine", needing_admin)).friendly_message(),
        );
    }
    result
}

pub fn optimize_all_sync(options: Option<crate::utils::batch::BatchOptions>) -> BatchResult {
    crate::perf_timed!("optimize_all", {
        let dry = options.unwrap_or_default().dry_run;
        run_optimize_batch(get_items_internal(), dry, admin::is_admin(), apply_optimization)
    })
}

pub(crate) fn ratio_score(optimized: usize, total: usize) -> u32 {
    if total == 0 {
        return 0;
    }
    ((optimized as f64 / total as f64) * 100.0).round() as u32
}

pub fn get_performance_score_sync() -> u32 {
    let items = get_items_internal();
    let optimized = items.iter().filter(|(item, _)| item.is_optimized).count();
    ratio_score(optimized, items.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_score_zero_total() {
        assert_eq!(ratio_score(0, 0), 0);
    }

    #[test]
    fn ratio_score_all_optimized() {
        assert_eq!(ratio_score(10, 10), 100);
    }

    #[test]
    fn ratio_score_none_optimized() {
        assert_eq!(ratio_score(0, 10), 0);
    }

    #[test]
    fn ratio_score_partial() {
        assert_eq!(ratio_score(3, 10), 30);
        assert_eq!(ratio_score(7, 10), 70);
    }

    #[test]
    fn build_items_has_categories_and_ids() {
        let items = build_items();
        assert!(!items.is_empty());
        for item in &items {
            assert!(!item.id.is_empty(), "id vide pour {}", item.name);
            assert!(!item.category.is_empty(), "category vide pour {}", item.id);
        }
    }

    #[test]
    fn build_items_ids_are_unique() {
        let items = build_items();
        let ids: std::collections::HashSet<&str> = items.iter().map(|i| i.id).collect();
        assert_eq!(ids.len(), items.len(), "ids dupliques detectes");
    }

    fn pending_items() -> Vec<(PerformanceItem, PerfDef)> {
        build_items()
            .into_iter()
            .map(|def| {
                let item = PerformanceItem {
                    id: def.id.to_string(),
                    name: def.name.to_string(),
                    description: def.description.to_string(),
                    category: def.category.to_string(),
                    is_optimized: false,
                };
                (item, def)
            })
            .collect()
    }

    #[test]
    fn only_machine_registry_items_need_admin_rights() {
        let flagged: Vec<&str> = build_items().iter().filter(|d| needs_admin(d)).map(|d| d.id).collect();
        assert_eq!(flagged, ["prefetch", "power_throttling", "nagle", "network_throttling"]);
    }

    #[test]
    fn without_admin_rights_machine_items_are_not_attempted_and_one_not_admin_line_is_recorded() {
        let total = build_items().len();
        let machine = build_items().iter().filter(|d| needs_admin(d)).count();
        let attempted = std::cell::RefCell::new(Vec::new());
        let result = run_optimize_batch(pending_items(), false, false, |def| {
            attempted.borrow_mut().push(def.id);
            Ok(())
        });
        let attempted = attempted.into_inner();
        assert_eq!(attempted.len(), total - machine);
        assert!(!attempted.contains(&"prefetch") && attempted.contains(&"visual_effects"));
        assert_eq!((result.success, result.failed), (total - machine, 1));
        assert!(result.errors[0].contains("administrateur"), "{:?}", result.errors);
    }

    #[test]
    fn with_admin_rights_every_pending_item_is_attempted() {
        let total = build_items().len();
        let result = run_optimize_batch(pending_items(), false, true, |_| Ok(()));
        assert_eq!((result.success, result.failed), (total, 0));
    }

    #[test]
    fn a_dry_run_lists_every_pending_item_and_writes_nothing() {
        let result = run_optimize_batch(pending_items(), true, false, |_| panic!("aucune ecriture en simulation"));
        assert!(result.dry_run);
        assert_eq!(result.preview_items.len(), build_items().len());
    }

    #[test]
    fn optimizing_a_machine_item_without_admin_rights_is_refused_before_any_write() {
        if admin::is_admin() {
            return;
        }
        let err = optimize_item_sync("prefetch".to_string()).unwrap_err();
        assert!(matches!(err, crate::utils::errors::CommandError::NotAdmin(_)), "{err:?}");
    }

    #[test]
    fn an_unknown_fix_type_is_an_error_not_a_silent_success() {
        let def = PerfDef {
            id: "x",
            name: "Inconnu",
            description: "d",
            category: "c",
            reg_path: "",
            reg_value: "",
            optimized_value: 0,
            fix_type: "Bogus",
            fix_target: "",
            check_command: "",
            check_contains: "",
        };
        let err = apply_optimization(&def).unwrap_err();
        assert!(err.contains("Bogus"), "{err}");
    }

    #[test]
    fn build_items_fix_types_are_known() {
        let items = build_items();
        let known = ["Registry", "RegistryString", "DeleteValue", "Command"];
        for item in &items {
            assert!(
                known.contains(&item.fix_type),
                "fix_type inconnu: {} pour {}",
                item.fix_type,
                item.id
            );
        }
    }
}
