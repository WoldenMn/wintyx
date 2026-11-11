//! Generation du rapport systeme HTML (commande Tauri `generate_report`).
//! Les helpers de presentation et la feuille de style sont dans [`html`].

use std::fs;
use crate::utils::errors::CommandError;
use crate::utils::{known_folders, logger};
use std::os::windows::process::CommandExt;
use crate::commands::{
    system_info, data_protection, services, hardening, startup, network, drivers, tasks, updates, disks,
    memory, bloatware, backup, benchmark,
};

mod html;
mod labels;
mod sections;
use html::{escape, firewall_table, row, score_row, section, REPORT_CSS};
use sections::{bloatware_section, render_or_unavailable, suspicious_tasks_section};

#[tauri::command]
#[specta::specta]
pub async fn generate_report() -> Result<String, CommandError> {
    crate::utils::blocking::read_result(generate_report_sync).await
}

pub fn generate_report_sync() -> Result<String, CommandError> {
    let desktop_path = known_folders::desktop_dir();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let output_path = desktop_path.join(format!("Wintyx_Report_{}.html", timestamp));

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"fr\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("<title>Rapport système Wintyx</title>\n");
    html.push_str("<style>\n");
    html.push_str(REPORT_CSS);
    html.push_str("\n</style>\n</head>\n<body>\n");

    html.push_str("<div class=\"header\">\n");
    html.push_str("<h1>Rapport système Wintyx</h1>\n");
    html.push_str(&format!("<p>Généré le {}</p>\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    let computer = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Inconnu".to_string());
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "Inconnu".to_string());
    html.push_str(&format!("<p>Ordinateur : {} | Utilisateur : {}</p>\n", escape(&computer), escape(&user)));
    html.push_str("</div>\n");

    html.push_str(&render_or_unavailable("Informations système", system_info::get_system_info_sync(), |info| {
        section("Informations système", &format!(
            "<table>\
            {}{}{}{}{}{}{}</table>",
            row("Système d'exploitation", &escape(&info.os_name)),
            row("Processeur", &escape(&info.cpu_name)),
            row("RAM totale", &format!("{} Go", info.total_ram_gb)),
            row("RAM utilisée", &format!("{} Go", info.used_ram_gb)),
            row("Utilisation du disque C:", &format!("{}%", info.disk_used_percent)),
            row("Antivirus", &escape(&info.antivirus_name)),
            row("Utilisation CPU", &format!("{}%", info.cpu_usage)),
        ))
    }));

    let dp_score = data_protection::get_data_protection_score();
    let svc_score = services::get_services_score_sync();
    let hard_score = hardening::get_hardening_score_sync();
    html.push_str(&section("Scores de sécurité", &format!(
        "<table><tr><th>Catégorie</th><th>Score</th><th>Statut</th></tr>\
        {}{}{}</table>",
        score_row("Protection des données", dp_score),
        score_row("Services", svc_score),
        score_row("Durcissement", hard_score),
    )));

    let dp_items = data_protection::get_data_protection_items();
    let mut dp_html = String::from("<table><tr><th>Catégorie</th><th>Nom</th><th>Statut</th></tr>");
    for item in &dp_items {
        let (css, status) = if item.is_protected { ("good", "Protégé") } else { ("bad", "EXPOSÉ") };
        dp_html.push_str(&format!("<tr><td>{}</td><td>{}</td><td class=\"{}\">{}</td></tr>",
            escape(labels::category(&item.category)), escape(&item.name), css, status));
    }
    dp_html.push_str("</table>");
    html.push_str(&section("Protection des données", &dp_html));

    let svc_items = services::get_services_sync();
    let mut svc_html = String::from("<table><tr><th>Service</th><th>Nom affiché</th><th>Risque</th><th>Statut</th></tr>");
    for item in &svc_items {
        let css = match item.status.as_str() {
            "Stopped" | "Absent" => "good",
            _ => "bad",
        };
        svc_html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td class=\"{}\">{}</td></tr>",
            escape(&item.name), escape(&item.display_name), labels::risk(&item.risk), css, labels::service_status(&item.status)));
    }
    svc_html.push_str("</table>");
    html.push_str(&section("Services", &svc_html));

    let hard_items = hardening::get_hardening_checks_sync();
    let mut hard_html = String::from("<table><tr><th>Catégorie</th><th>Vérification</th><th>Niveau</th><th>Statut</th></tr>");
    for item in &hard_items {
        let (css, status) = if item.is_secure { ("good", "Sécurisé") } else if item.is_unknown { ("warn", "Inconnu") } else { ("bad", "VULNÉRABLE") };
        hard_html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td class=\"{}\">{}</td></tr>",
            escape(labels::category(&item.category)), escape(&item.name), labels::hardening_level(&item.risk_level), css, status));
    }
    hard_html.push_str("</table>");
    html.push_str(&section("Vérifications de durcissement", &hard_html));

    let start_items = startup::get_startup_items();
    if !start_items.is_empty() {
        let mut start_html = String::from("<table><tr><th>Nom</th><th>Commande</th><th>Emplacement</th></tr>");
        for item in &start_items {
            let cmd = if item.command.len() > 80 { format!("{}...", item.command.chars().take(80).collect::<String>()) } else { item.command.clone() };
            start_html.push_str(&format!("<tr><td>{}</td><td><code>{}</code></td><td>{}</td></tr>",
                escape(&item.name), escape(&cmd), escape(&item.location_display)));
        }
        start_html.push_str("</table>");
        html.push_str(&section("Programmes au démarrage", &start_html));
    }

    html.push_str(&section("État du pare-feu", &firewall_table(&network::get_firewall_status())));

    html.push_str(&render_or_unavailable("Disques", disks::get_disks_sync(), |disk_list| {
        let mut disk_html = String::from("<table><tr><th>Lecteur</th><th>Étiquette</th><th>Utilisé</th><th>Total</th><th>%</th></tr>");
        for d in &disk_list {
            let css = if d.used_percent > 90 { "bad" } else if d.used_percent > 75 { "warn" } else { "good" };
            disk_html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{:.1} Go</td><td>{:.1} Go</td><td class=\"{}\">{:.0}%</td></tr>",
                escape(&d.name), escape(&d.volume_name), d.used_gb, d.total_gb, css, d.used_percent));
        }
        disk_html.push_str("</table>");
        section("Disques", &disk_html)
    }));

    html.push_str(&render_or_unavailable("Résumé des pilotes", drivers::get_driver_summary_sync(), |summary| {
        let mut drv_html = String::from("<table>");
        drv_html.push_str(&row("Pilotes au total", &summary.total.to_string()));
        drv_html.push_str(&row("Pilotes non signés", &summary.unsigned.to_string()));
        drv_html.push_str(&row("Périphériques en erreur", &summary.problems.to_string()));
        drv_html.push_str("</table>");
        section("Résumé des pilotes", &drv_html)
    }));

    // Un seul scan des taches planifiees (la commande PowerShell la plus lente du
    // rapport) : suspicious est derive du resultat au lieu d'un 2e scan (perf-rust-02).
    let tasks_result = tasks::get_scheduled_tasks_sync();
    html.push_str(&suspicious_tasks_section(&tasks_result));

    let build = updates::get_windows_build_sync().unwrap_or_else(|_| "Inconnue".to_string());
    html.push_str(&section("Version de Windows", &format!("<p>{}</p>", escape(&build))));

    html.push_str(&render_or_unavailable("Résumé de la mémoire", memory::get_memory_summary_sync(), |mem_summary| {
        let css = if mem_summary.used_percent > 90 { "bad" }
            else if mem_summary.used_percent > 75 { "warn" } else { "good" };
        let mut mem_html = String::from("<table>");
        mem_html.push_str(&row("Totale", &format!("{:.1} Go", mem_summary.total_gb)));
        mem_html.push_str(&row("Utilisée", &format!("{:.1} Go", mem_summary.used_gb)));
        mem_html.push_str(&row("Disponible", &format!("{:.1} Go", mem_summary.available_gb)));
        mem_html.push_str(&format!("<tr><td class=\"label\">Utilisation</td><td class=\"{}\">{}%</td></tr>", css, mem_summary.used_percent));
        mem_html.push_str(&row("Emplacements", &format!("{} / {}", mem_summary.slots_filled, mem_summary.total_slots)));
        mem_html.push_str(&row("Vitesse max", &format!("{} MHz", mem_summary.max_speed_mhz)));
        mem_html.push_str("</table>");
        section("Résumé de la mémoire", &mem_html)
    }));

    html.push_str(&render_or_unavailable("Adaptateurs réseau", network::get_network_adapters_sync(), |adapters| {
        if adapters.is_empty() {
            return String::new();
        }
        let mut net_html = String::from("<table><tr><th>Nom</th><th>Type</th><th>Statut</th><th>IP</th><th>MAC</th></tr>");
        for a in &adapters {
            let css = if a.is_connected { "good" } else { "bad" };
            let ip = if a.ip_address.is_empty() { "—" } else { a.ip_address.as_str() };
            let mac = if a.mac_address.is_empty() { "—" } else { a.mac_address.as_str() };
            net_html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td class=\"{}\">{}</td><td>{}</td><td>{}</td></tr>",
                escape(&a.name), escape(&a.adapter_type), css, escape(&a.status), escape(ip), escape(mac),
            ));
        }
        net_html.push_str("</table>");
        section("Adaptateurs réseau", &net_html)
    }));

    html.push_str(&bloatware_section(&bloatware::get_bloatware_items_sync()));

    if let Some(all_tasks) = tasks_result.as_ref().ok().filter(|all| !all.is_empty()) {
        let suspicious_count = all_tasks.iter().filter(|t| t.is_suspicious).count();
        let total = all_tasks.len();
        let enabled = all_tasks.iter().filter(|t| t.state.eq_ignore_ascii_case("Ready") || t.state.eq_ignore_ascii_case("Running")).count();
        let mut tk_html = String::from("<table>");
        tk_html.push_str(&row("Tâches au total", &total.to_string()));
        tk_html.push_str(&row("Tâches actives", &enabled.to_string()));
        tk_html.push_str(&row("Tâches suspectes", &suspicious_count.to_string()));
        tk_html.push_str("</table>");
        html.push_str(&section("Vue d'ensemble des tâches planifiées", &tk_html));
    }

    let reg_backups = backup::get_registry_backups_sync();
    let (protection_css, protection_label) = match backup::system_protection_policy() {
        Ok(state @ backup::PolicyState::NotDisabled) => ("", state.label()),
        Ok(state) => ("warn", state.label()),
        Err(_) => ("warn", "Indéterminée"),
    };
    let mut backup_html = String::from("<table>");
    backup_html.push_str(&format!(
        "<tr><td class=\"label\">Protection du système</td><td class=\"{}\">{}</td></tr>",
        protection_css, protection_label
    ));
    backup_html.push_str(&row("Sauvegardes de registre stockées", &reg_backups.len().to_string()));
    backup_html.push_str("</table>");
    if !reg_backups.is_empty() {
        backup_html.push_str("<table><tr><th>Ruche</th><th>Date</th><th>Taille</th></tr>");
        for b in reg_backups.iter().take(10) {
            backup_html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{:.1} Ko</td></tr>",
                escape(&b.hive_name), escape(&b.backup_time), b.file_size_bytes as f64 / 1024.0
            ));
        }
        backup_html.push_str("</table>");
    }
    html.push_str(&section("Sauvegardes", &backup_html));

    match benchmark::get_benchmark_history() {
        Ok(history) => {
            if let Some(latest) = history.first() {
                html.push_str(&section("Dernier benchmark", &benchmark_html(latest)));
            }
        }
        Err(e) => {
            logger::warn("report", &format!("Historique benchmark indisponible: {}", e.friendly_message()));
            html.push_str(&section(
                "Dernier benchmark",
                &format!("<p class=\"bad\">Historique indisponible : {}</p>", escape(&e.friendly_message())),
            ));
        }
    }

    html.push_str("<div class=\"footer\">\n<p>Wintyx v2 - Rapport de diagnostic système</p>\n</div>\n");
    html.push_str("</body>\n</html>");

    fs::write(&output_path, &html).map_err(|e| e.to_string())?;

    let file_path = output_path.to_string_lossy().to_string();
    match crate::utils::process::resolve_system_program("explorer.exe") {
        Some(explorer) => {
            if let Err(e) = std::process::Command::new(explorer)
                .arg(&output_path)
                .creation_flags(crate::utils::constants::CREATE_NO_WINDOW)
                .spawn()
            {
                logger::warn("report", &format!("Ouverture du rapport impossible: {}", e));
            }
        }
        None => logger::warn("report", "Ouverture du rapport impossible: explorateur Windows introuvable"),
    }

    Ok(file_path)
}

fn benchmark_html(latest: &benchmark::BenchmarkEntry) -> String {
    let r = &latest.result;
    let css = if r.overall_score >= 75 { "good" }
        else if r.overall_score >= 40 { "warn" } else { "bad" };
    let mut bm_html = String::from("<table>");
    bm_html.push_str(&row("Dernière exécution", &escape(&latest.timestamp)));
    if !latest.plausible {
        bm_html.push_str(&row("Résultat", "Mesure invalide (ancienne version)"));
        bm_html.push_str("</table>");
        return bm_html;
    }
    bm_html.push_str(&format!("<tr><td class=\"label\">Score global</td><td class=\"{}\">{}/100</td></tr>", css, r.overall_score));
    bm_html.push_str(&row("Score CPU", &format!("{:.1} M ops/s", r.cpu_score)));
    bm_html.push_str(&row("Score mémoire", &format!("{:.0} Mo/s", r.memory_score)));
    bm_html.push_str(&row("Lecture disque", &format!("{:.0} Mo/s", r.disk_read_score)));
    bm_html.push_str(&row("Écriture disque", &format!("{:.0} Mo/s", r.disk_write_score)));
    bm_html.push_str(&row("Latence réseau", &format!("{:.1} ms", r.network_latency)));
    bm_html.push_str("</table>");
    bm_html
}
