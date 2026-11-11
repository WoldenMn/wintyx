//! Sauvegardes de cles registre (.reg) : nommage, lots et structure d'inventaire.

use serde::Serialize;

use crate::utils::batch::BatchResult;
use crate::utils::errors::CommandError;

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RegistryBackupInfo {
    pub file_path: String,
    pub hive_name: String,
    pub backup_time: String,
    pub file_size_bytes: u64,
}

const MAX_SAFE_NAME_CHARS: usize = 60;

/// Retire `_<hash 8 hex>_<ms 3 chiffres>_<YYYYMMDD>_<HHMMSS>` ajoute par `export_file_name` :
/// la date est deja affichee a part, le hash n'a de sens que pour distinguer les fichiers.
fn strip_export_suffix(stem: &str) -> &str {
    let parts: Vec<&str> = stem.rsplitn(5, '_').collect();
    if let [time, date, millis, hash, rest] = parts[..] {
        let digits = |s: &str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
        if digits(time, 6) && digits(date, 8) && digits(millis, 3) && hash.len() == 8 && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return rest;
        }
    }
    stem
}

pub(crate) fn hive_name_from_stem(name: &str) -> String {
    if name.starts_with("reg_") {
        name.split('_').nth(1).unwrap_or("Inconnu").to_string()
    } else if let Some(stripped) = name.strip_prefix("regkey_") {
        format!("Clé : {}", strip_export_suffix(stripped))
    } else {
        "Inconnu".to_string()
    }
}

/// Horodatage d'un lot au format `YYYYMMDD_HHMMSS`, lu par `snapshot_diff` pour
/// regrouper les fichiers d'un meme snapshot : un lot n'en calcule qu'un seul.
pub(super) fn batch_timestamp(now: chrono::DateTime<chrono::Local>) -> String {
    now.format("%Y%m%d_%H%M%S").to_string()
}

/// `regkey_<chemin tronque>_<hash du chemin complet>_<ms>_<horodatage du lot>.reg`.
/// Le nom tronque reste lisible ; le hash du chemin complet distingue deux cles qui
/// partagent leurs 60 premiers caracteres ; les millisecondes distinguent deux
/// exports de la meme cle dans la meme seconde. Le suffixe `YYYYMMDD_HHMMSS` reste
/// en fin de nom, ou `snapshot_diff::extract_timestamp` le lit.
pub(super) fn export_file_name(key_path: &str, batch_timestamp: &str, millis: u32) -> String {
    let safe_name: String = key_path
        .replace(['\\', '/'], "_")
        .chars()
        .take(MAX_SAFE_NAME_CHARS)
        .collect();
    format!(
        "regkey_{}_{}_{:03}_{}.reg",
        safe_name,
        crate::utils::registry::path_hash(key_path),
        millis % 1000,
        batch_timestamp
    )
}

/// Exporte un lot de cles avec un horodatage commun : un lot ne peut pas etre
/// coupe en deux groupes de snapshot quand il franchit une seconde.
pub(super) fn export_keys<F>(keys: &[&str], timestamp: &str, mut export: F) -> BatchResult
where
    F: FnMut(&str, &str) -> Result<String, CommandError>,
{
    let mut result = BatchResult::new();
    for key in keys {
        result.record(*key, export(key, timestamp));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hive_name_from_reg_prefix() {
        assert_eq!(hive_name_from_stem("reg_HKLM_20260417"), "HKLM");
        assert_eq!(hive_name_from_stem("reg_HKCU_20260417"), "HKCU");
    }

    #[test]
    fn hive_name_from_regkey_prefix() {
        assert_eq!(
            hive_name_from_stem("regkey_HKLM_SOFTWARE_20260417"),
            "Clé : HKLM_SOFTWARE_20260417"
        );
    }

    #[test]
    fn hive_name_drops_the_hash_millis_and_batch_timestamp_suffix() {
        assert_eq!(
            hive_name_from_stem("regkey_HKCU_SOFTWARE_Microsoft_Windows_CurrentVersion_Run_189a0a72_586_20261003_082305"),
            "Clé : HKCU_SOFTWARE_Microsoft_Windows_CurrentVersion_Run"
        );
    }

    #[test]
    fn hive_name_keeps_a_name_whose_tail_is_not_the_export_suffix() {
        assert_eq!(hive_name_from_stem("regkey_HKLM_SOFTWARE_Run_12_34"), "Clé : HKLM_SOFTWARE_Run_12_34");
    }

    #[test]
    fn hive_name_unknown_pattern() {
        assert_eq!(hive_name_from_stem("random_file"), "Inconnu");
        assert_eq!(hive_name_from_stem(""), "Inconnu");
    }

    #[test]
    fn export_file_name_distinguishes_keys_sharing_a_long_prefix() {
        let prefix = r"HKLM\SOFTWARE\Policies\Microsoft\Windows\SomeVeryLongKeyNameBlock";
        let a = export_file_name(&format!(r"{}\Alpha", prefix), "20260901_101500", 7);
        let b = export_file_name(&format!(r"{}\Beta", prefix), "20260901_101500", 7);
        assert_ne!(a, b);
    }

    #[test]
    fn export_file_name_distinguishes_same_key_by_millisecond() {
        let a = export_file_name(r"HKLM\SOFTWARE\A", "20260901_101500", 7);
        let b = export_file_name(r"HKLM\SOFTWARE\A", "20260901_101500", 8);
        assert_ne!(a, b);
    }

    #[test]
    fn export_file_name_keeps_the_batch_timestamp_as_suffix() {
        let name = export_file_name(r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run", "20260901_101500", 42);
        assert!(name.starts_with("regkey_HKCU_SOFTWARE_Microsoft_Windows_CurrentVersion_Run_"));
        assert!(name.ends_with("_042_20260901_101500.reg"), "{}", name);
        assert!(!name.contains(['\\', '/']));
        assert_eq!(
            crate::commands::snapshot_diff::extract_timestamp_for_test(&name).as_deref(),
            Some("20260901_101500")
        );
    }

    #[test]
    fn export_keys_gives_every_key_the_same_timestamp() {
        let keys = [r"HKLM\SOFTWARE\A", r"HKLM\SOFTWARE\B", r"HKLM\SOFTWARE\C"];
        let mut seen = Vec::new();
        let result = export_keys(&keys, "20260901_101559", |key, ts| {
            seen.push((key.to_string(), ts.to_string()));
            Ok(String::new())
        });
        assert_eq!(result.success, 3);
        assert!(seen.iter().all(|(_, ts)| ts == "20260901_101559"));
    }

    #[test]
    fn export_keys_records_failures_without_stopping() {
        let keys = [r"HKLM\SOFTWARE\A", r"HKLM\SOFTWARE\B"];
        let result = export_keys(&keys, "20260901_101559", |key, _| {
            if key.ends_with('A') {
                Err(CommandError::Generic("boom".into()))
            } else {
                Ok(String::new())
            }
        });
        assert_eq!((result.total, result.success, result.failed), (2, 1, 1));
    }

    #[test]
    fn batch_timestamp_has_the_snapshot_format() {
        let ts = batch_timestamp(chrono::Local::now());
        assert_eq!(ts.len(), 15);
        assert_eq!(ts.as_bytes()[8], b'_');
    }
}
