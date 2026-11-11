//! 6 scanners de menaces sur le systeme Windows, regroupes par famille de source.
//!
//! Chaque fonction `scan_*` retourne (poids_score, Vec<ThreatItem>). Le poids
//! contribue au `total_items_scanned` global. Les ThreatItem alimentent le
//! risk_score via `calculate_risk_score` dans `mod.rs` (parent).
//!
//! - [`process_based`] : sources via `process::run` (taches planifiees, services)
//! - [`tasks`] : taches planifiees, via l'analyse de la page Taches
//! - [`ports`] : ports en ecoute, jugés sur l'exécutable qui les tient
//! - [`registry_based`] : sources via `registry` (demarrage, autorun, pare-feu)
//!
//! Les helpers `extract_exe_path` / `expand_env_vars` sont partages ici.

mod ports;
mod process_based;
mod registry_based;
mod tasks;

pub(super) use ports::scan_open_ports;
pub(super) use process_based::scan_unsigned_services;
pub(super) use tasks::scan_scheduled_tasks;
pub(super) use registry_based::{scan_autorun_registry, scan_firewall, scan_startup_entries};

pub(super) fn extract_exe_path(value: &str) -> String {
    let trimmed = value.trim();
    // Chemin entre guillemets : "C:\...\app.exe" -arg
    if let Some(after_quote) = trimmed.strip_prefix('"') {
        if let Some(end) = after_quote.find('"') {
            return after_quote[..end].to_string();
        }
    }
    // Chemin non quote : couper a la fin de la 1re extension executable situee a
    // une frontiere de mot, MEME si le chemin contient des espaces (ex:
    // "C:\Program Files\App\x.exe --flag" -> "C:\Program Files\App\x.exe").
    // Sans ca, un chemin non quote avec espaces tombe en fallback sur la ligne
    // entiere (args inclus) -> faux positif "Exécutable introuvable" (run-08 :
    // NVIDIA Broadcast, OpenVPN, GogGalaxy). `to_ascii_lowercase` preserve les
    // positions d'octets, donc le slicing sur `trimmed` reste sur.
    let lower = trimmed.to_ascii_lowercase();
    for ext in [".exe", ".bat", ".cmd", ".com", ".scr"] {
        for (pos, _) in lower.match_indices(ext) {
            let end = pos + ext.len();
            if trimmed[end..].chars().next().is_none_or(|c| c == ' ') {
                return trimmed[..end].to_string();
            }
        }
    }
    trimmed.to_string()
}

/// Expanse les variables d'environnement Windows `%VAR%` d'un chemin.
///
/// `registry::get_string` lit les valeurs REG_EXPAND_SZ sans les developper : un
/// chemin `%ProgramFiles%\App\app.exe` resterait litteral et serait declare
/// "introuvable" a tort (bug-rust-03). Les variables inconnues sont conservees
/// (`%VAR%`) pour que l'appelant evite un faux positif.
pub(super) fn expand_env_vars(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find('%') {
        result.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(0) => {
                // "%%" -> "%" litteral
                result.push('%');
                rest = &after[1..];
            }
            Some(end) => {
                let var_name = &after[..end];
                if let Ok(val) = std::env::var(var_name) {
                    result.push_str(&val);
                } else {
                    result.push('%');
                    result.push_str(var_name);
                    result.push('%');
                }
                rest = &after[end + 1..];
            }
            None => {
                // Pas de % fermant : conserver le reste tel quel.
                result.push('%');
                result.push_str(after);
                return result;
            }
        }
    }
    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_exe_quoted_path() {
        assert_eq!(
            extract_exe_path("\"C:\\Program Files\\app.exe\" -arg"),
            "C:\\Program Files\\app.exe"
        );
    }

    #[test]
    fn extract_exe_unquoted() {
        assert_eq!(extract_exe_path("C:\\app.exe -flag"), "C:\\app.exe");
    }

    #[test]
    fn extract_exe_unquoted_path_with_spaces() {
        // run-08 : chemin non quote avec espaces dans le dossier + arguments ne doit
        // PAS etre flague "Exécutable introuvable" (NVIDIA Broadcast, OpenVPN...).
        assert_eq!(
            extract_exe_path(
                "C:\\Program Files\\NVIDIA Broadcast\\NVIDIA Broadcast.exe --process-start-args \"--launch-hidden\""
            ),
            "C:\\Program Files\\NVIDIA Broadcast\\NVIDIA Broadcast.exe"
        );
    }

    #[test]
    fn extract_exe_empty() {
        assert_eq!(extract_exe_path(""), "");
    }

    #[test]
    fn extract_exe_no_exe_token_returns_full_line() {
        // tests-be-05 : token non-quote ne finissant pas par .exe -> ligne entiere.
        assert_eq!(extract_exe_path("rundll32 arg,Entry"), "rundll32 arg,Entry");
    }

    #[test]
    fn extract_exe_unterminated_quote_returns_full() {
        // tests-be-05 : guillemet ouvrant sans fermant -> fallback ligne entiere.
        assert_eq!(extract_exe_path("\"C:\\unterminated"), "\"C:\\unterminated");
    }

    #[test]
    fn expand_env_vars_expands_known() {
        // Regression bug-rust-03 : une variable d'env connue doit etre developpee
        // avant le test d'existence sur disque.
        std::env::set_var("WINTYX_TEST_EXPAND", "C:\\TestDir");
        assert_eq!(
            expand_env_vars("%WINTYX_TEST_EXPAND%\\app.exe"),
            "C:\\TestDir\\app.exe"
        );
        std::env::remove_var("WINTYX_TEST_EXPAND");
    }

    #[test]
    fn expand_env_vars_keeps_unknown() {
        // Variable inconnue conservee litteralement (% present) -> pas de faux positif.
        let r = expand_env_vars("%WINTYX_NOPE_XYZ_123%\\app.exe");
        assert!(r.contains('%'), "variable inconnue doit rester litterale: {}", r);
    }

    #[test]
    fn expand_env_vars_no_vars() {
        assert_eq!(expand_env_vars("C:\\plain\\path.exe"), "C:\\plain\\path.exe");
    }
}
