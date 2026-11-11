use serde::Serialize;
use std::fs;
use chrono::Local;

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub line: String,
    pub level: String,
}

fn detect_level(line: &str) -> &'static str {
    let upper = line.to_uppercase();
    if upper.contains("[ERROR]") || upper.contains("[ERR]") {
        "error"
    } else if upper.contains("[WARN ]") || upper.contains("[WARN]") || upper.contains("[WARNING]") {
        "warn"
    } else {
        "info"
    }
}

fn matches_filter(level: &str, filter: Option<&str>) -> bool {
    match filter {
        None => true,
        Some("all") | Some("") => true,
        Some(f) => {
            let normalized = match f.to_ascii_lowercase().as_str() {
                "warning" => "warn".to_string(),
                other => other.to_string(),
            };
            normalized == level
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_logs(level_filter: Option<String>, limit: Option<usize>) -> Vec<LogEntry> {
    crate::utils::blocking::read_value(move || get_logs_sync(level_filter, limit)).await
}

pub fn get_logs_sync(level_filter: Option<String>, limit: Option<usize>) -> Vec<LogEntry> {
    let log_file = crate::utils::paths::logs_dir().join(format!("wintyx_{}.log", Local::now().format("%Y%m%d")));

    let content = match fs::read_to_string(&log_file) {
        Ok(c) => c,
        // Fichier de log du jour pas encore cree : etat normal, pas d'erreur.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            crate::utils::logger::warn("logs", &format!("get_logs: lecture impossible de {}: {}", log_file.display(), e));
            String::new()
        }
    };
    let max = limit.unwrap_or(200);
    let filter = level_filter.as_deref();

    parse_entries(&content)
        .into_iter()
        .rev()
        .filter(|entry| matches_filter(&entry.level, filter))
        .take(max)
        .collect()
}

fn is_entry_start(line: &str) -> bool {
    let b = line.as_bytes();
    b.len() > 5 && b[0] == b'[' && b[1..5].iter().all(u8::is_ascii_digit) && b[5] == b'-'
}

/// Les anciens fichiers contiennent des messages multi-lignes : toute ligne qui ne
/// commence pas par un horodatage prolonge l'entrée précédente.
fn parse_entries(content: &str) -> Vec<LogEntry> {
    let mut entries: Vec<LogEntry> = Vec::new();
    for line in content.lines().filter(|l| !l.trim().is_empty()) {
        match entries.last_mut() {
            Some(last) if !is_entry_start(line) => {
                last.line.push_str("\\n");
                last.line.push_str(line.trim_end());
            }
            _ => {
                entries.push(LogEntry { line: line.to_string(), level: detect_level(line).to_string() });
            }
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_error_variants() {
        assert_eq!(detect_level("2026-04-16 10:00:00 [ERROR] boom"), "error");
        assert_eq!(detect_level("2026-04-16 10:00:00 [err] lowercase"), "error");
    }

    #[test]
    fn detects_warn_variants() {
        assert_eq!(detect_level("2026-04-16 10:00:00 [WARN ] padded"), "warn");
        assert_eq!(detect_level("2026-04-16 10:00:00 [WARN] no padding"), "warn");
        assert_eq!(detect_level("2026-04-16 10:00:00 [WARNING] full"), "warn");
        assert_eq!(detect_level("2026-04-16 10:00:00 [warning] lowercase"), "warn");
    }

    #[test]
    fn detects_info_default() {
        assert_eq!(detect_level("2026-04-16 10:00:00 [INFO] ok"), "info");
        assert_eq!(detect_level("2026-04-16 10:00:00 arbitrary line"), "info");
    }

    #[test]
    fn filter_none_matches_all() {
        assert!(matches_filter("info", None));
        assert!(matches_filter("error", None));
    }

    #[test]
    fn filter_all_matches_all() {
        assert!(matches_filter("info", Some("all")));
        assert!(matches_filter("error", Some("all")));
    }

    const LEGACY: &str = "[2026-10-03 07:59:55] [WARN ] [backup] echec\nAu caractere Ligne:1 : 59\n+ CategoryInfo : refus\n[2026-10-03 08:00:01] [INFO ] [app] suite";

    #[test]
    fn legacy_continuation_lines_join_the_previous_entry() {
        let entries = parse_entries(LEGACY);
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0].line,
            "[2026-10-03 07:59:55] [WARN ] [backup] echec\\nAu caractere Ligne:1 : 59\\n+ CategoryInfo : refus"
        );
        assert_eq!(entries[0].level, "warn");
        assert_eq!(entries[1].level, "info");
    }

    #[test]
    fn continuation_text_never_changes_the_level_of_its_entry() {
        let entries = parse_entries("[2026-10-03 08:00:00] [INFO ] [app] debut\nligne avec [ERROR] dedans");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "info");
    }

    #[test]
    fn orphan_leading_lines_form_their_own_entry() {
        let entries = parse_entries("reste d'une rotation\nsuite\n[2026-10-03 08:00:00] [INFO ] [app] ok");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].line, "reste d'une rotation\\nsuite");
    }

    #[test]
    fn entries_stay_chronological_and_blank_lines_are_ignored() {
        let entries = parse_entries("[2026-10-03 08:00:00] [INFO ] [a] un\n\n[2026-10-03 08:00:01] [INFO ] [a] deux\n");
        assert_eq!(entries.len(), 2);
        assert!(entries[0].line.ends_with("un") && entries[1].line.ends_with("deux"));
    }

    #[test]
    fn filter_specific_level() {
        assert!(matches_filter("warn", Some("warn")));
        assert!(matches_filter("warn", Some("WARN")));
        assert!(matches_filter("warn", Some("warning")));
        assert!(!matches_filter("info", Some("warn")));
    }
}
