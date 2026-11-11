mod commands;
mod utils;

#[cfg(debug_assertions)]
use specta_typescript::{BigIntExportBehavior, Typescript};
use tauri_specta::{collect_commands, collect_events, Builder};

/// Construit le Builder specta pour la generation TS.
/// Centralise pour pouvoir etre appele depuis un test (export bindings.ts)
/// en parallele du runtime Tauri (run()).
fn build_specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
        commands::system_info::get_system_info,
        commands::data_protection::get_data_protection_items,
        commands::data_protection::protect_item,
        commands::data_protection::protect_all,
        commands::services::get_services,
        commands::services::stop_service,
        commands::services::start_service,
        commands::hardening::get_hardening_checks,
        commands::hardening::apply_hardening_fix,
        commands::hardening::apply_hardening_essential,
        commands::hardening::apply_hardening_all,
        commands::performance::get_performance_items,
        commands::performance::optimize_item,
        commands::performance::optimize_all,
        commands::cleanup::get_cleanup_targets,
        commands::cleanup::clean_target,
        commands::cleanup::clean_all,
        commands::cleanup::flush_dns,
        commands::bloatware::get_bloatware_items,
        commands::bloatware::remove_bloatware,
        commands::bloatware::remove_all_bloatware,
        commands::startup::get_startup_items,
        commands::startup::disable_startup_item,
        commands::startup::enable_startup_item,
        commands::network::get_network_adapters,
        commands::network::get_firewall_status,
        commands::network::ping_host,
        commands::network::test_connectivity,
        commands::network::get_public_ip,
        commands::drivers::get_driver_audit,
        commands::drivers::get_driver_audit_summary,
        commands::drivers::uninstall_driver,
        commands::drivers::remove_phantom_device,
        commands::drivers::remove_all_phantom_devices,
        commands::memory::get_memory_sticks,
        commands::memory::get_memory_summary,
        commands::memory::get_memory_consumers,
        commands::disks::get_disks,
        commands::updates::get_update_history,
        commands::updates::get_windows_build,
        commands::updates::get_last_update_date,
        commands::tasks::get_scheduled_tasks,
        commands::tasks::get_suspicious_tasks,
        commands::tasks::disable_scheduled_task,
        commands::tasks::get_suspicious_task_count,
        commands::backup::get_restore_points,
        commands::backup::create_restore_point,
        commands::backup::backup_critical_keys,
        commands::backup::get_registry_backups,
        commands::backup::restore_registry_backup,
        commands::backup::is_system_protection_enabled,
        commands::backup::get_backup_folder,
        commands::backup::open_backup_folder,
        commands::backup::create_pre_batch_snapshot,
        commands::report::generate_report,
        commands::logs::get_logs,
        commands::monitor::start_monitoring,
        commands::monitor::stop_monitoring,
        commands::monitor::set_monitor_interval,
        commands::threat_scanner::run_threat_scan,
        commands::benchmark::run_benchmark,
        commands::benchmark::get_benchmark_history,
        commands::scores::get_all_scores,
        commands::scores_history::record_score_snapshot,
        commands::scores_history::get_score_history,
        commands::defender::get_defender_status,
        commands::diagnostics::get_app_diagnostics,
        commands::profiles::get_profiles,
        commands::profiles::apply_profile,
        commands::profiles::get_available_items,
        commands::profiles::create_custom_profile,
        commands::profiles::update_custom_profile,
        commands::profiles::delete_custom_profile,
        commands::snapshot_diff::list_snapshot_groups,
        commands::snapshot_diff::diff_snapshot_vs_live,
        commands::snapshot_diff::diff_snapshot_vs_snapshot,
        commands::snapshot_diff::restore_selected_changes,
        utils::admin::get_admin_status,
        utils::admin::relaunch_as_admin,
        commands::perf::perf_dump_frontend,
        commands::frontend_log::log_frontend_error,
        ])
        // Events emis via app.emit(...) cote backend et ecoutes via listen<T>(name)
        // cote frontend. Les declarer ici (collect_events!, types-event) les inclut
        // dans la codegen bindings.ts : plus de redefinition manuelle dans types.ts.
        // Les noms de canaux sont preserves via #[tauri_specta(event_name = "...")].
        .events(collect_events![
            commands::monitor::SystemMetrics,
            commands::threat_scanner::ThreatScanProgress,
        ])
}

/// Configuration TS partagee : tolere les u64/i64 en serialisant en `number`.
/// Les valeurs Wintyx (timeouts, sizes WMI, timestamps) restent dans la safe range JS.
/// Header avec `// @ts-nocheck` + `eslint-disable` car le fichier est genere
/// et contient des imports/declarations conditionnels que tsc/ESLint ne savent
/// pas evaluer (TAURI_CHANNEL, __makeEvents__).
/// Reservee aux builds debug : utilisee uniquement par les deux call-sites
/// d'export TS (`run` en debug et `export_bindings` en debug).
#[cfg(debug_assertions)]
fn ts_config() -> Typescript {
    Typescript::default()
        .bigint(BigIntExportBehavior::Number)
        .header("// @ts-nocheck\n/* eslint-disable */\n")
}

#[cfg(debug_assertions)]
fn bindings_path(manifest_dir: &str) -> std::path::PathBuf {
    std::path::Path::new(manifest_dir)
        .parent()
        .expect("Le manifeste Cargo n'a pas de dossier parent")
        .join("src")
        .join("lib")
        .join("bindings.ts")
}

pub fn run() {
    utils::logger::init();

    let specta_builder = build_specta_builder();

    #[cfg(debug_assertions)]
    specta_builder
        .export(ts_config(), bindings_path(env!("CARGO_MANIFEST_DIR")))
        .expect("Failed to export bindings");

    let result = tauri::Builder::default()
        .invoke_handler(specta_builder.invoke_handler())
        .run(tauri::generate_context!());

    if let Err(e) = result {
        utils::logger::error("tauri", &format!("Echec du runtime Tauri: {}", e));
        eprintln!("Erreur fatale Tauri: {}", e);
        std::process::exit(1);
    }
}

/// Genere `src/lib/bindings.ts` sans demarrer le runtime Tauri.
/// Appele depuis `main.rs` quand l'argument `--export-bindings` est passe.
/// Reservee aux builds debug pour eviter l'introspection de la surface API
/// depuis un binaire release distribue.
#[cfg(debug_assertions)]
pub fn export_bindings() {
    build_specta_builder()
        .export(ts_config(), bindings_path(env!("CARGO_MANIFEST_DIR")))
        .expect("Failed to export bindings");
    eprintln!("bindings.ts exporte avec succes");
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::bindings_path;
    use std::path::Path;

    #[test]
    fn bindings_path_remonte_au_dossier_parent_du_manifeste() {
        let manifeste = Path::new("racine").join("Wintyx").join("src-tauri");
        let attendu = Path::new("racine")
            .join("Wintyx")
            .join("src")
            .join("lib")
            .join("bindings.ts");
        assert_eq!(bindings_path(manifeste.to_str().unwrap()), attendu);
    }

    #[test]
    fn bindings_path_du_manifeste_reel_est_absolu_et_dans_le_projet() {
        let chemin = bindings_path(env!("CARGO_MANIFEST_DIR"));
        assert!(
            chemin.is_absolute(),
            "chemin relatif au repertoire courant : {chemin:?}"
        );
        assert!(
            chemin.parent().is_some_and(Path::is_dir),
            "dossier absent : {chemin:?}"
        );
        assert!(
            chemin.ends_with(Path::new("src").join("lib").join("bindings.ts")),
            "chemin inattendu : {chemin:?}"
        );
        let projet = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        assert!(chemin.starts_with(projet), "hors du projet : {chemin:?}");
        assert!(
            projet.join("package.json").is_file(),
            "pas la racine Wintyx : {projet:?}"
        );
    }
}
