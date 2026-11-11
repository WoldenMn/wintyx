//! Parser de fichiers .reg (UTF-16 LE BOM) exportes par reg.exe.
//!
//! Decode le contenu binaire, normalise les noms de hives Windows
//! (HKEY_LOCAL_MACHINE -> HKLM, etc.) et parse les sections avec
//! leurs paires nom/valeur. Egalement responsable de l'extraction
//! du timestamp depuis les noms de fichiers de backup, et du rejet
//! des symlinks dans le dossier de backups.

use std::fs;
use std::path::{Path, PathBuf};

use super::TIMESTAMP_LEN;

/// Rejette les symlinks dans le dossier de backups (coherence avec
/// `backup::restore_registry_backup`). Defense en profondeur contre la
/// lecture indirecte de fichiers hors `%LOCALAPPDATA%\Wintyx\Backups`.
pub(super) fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .ok()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

/// Enumere les fichiers `.reg` valides du dossier de backup.
///
/// Filtre cumulativement :
/// - les symlinks (defense en profondeur, fix M2 v2.7.1)
/// - les fichiers sans extension `.reg`
/// - les fichiers dont le timestamp ne peut etre extrait
///
/// Retourne (PathBuf, timestamp) pour chaque fichier valide. Centralise les
/// 3 boucles `fs::read_dir` qui faisaient le meme filtrage dans
/// `list_snapshot_groups`, `diff_snapshot_vs_snapshot` et `diff_snapshot_vs_live`.
///
/// Cout d'allocation Vec : negligeable (typiquement <50 fichiers dans
/// `%LOCALAPPDATA%\Wintyx\Backups`).
pub(super) fn iter_reg_files(folder: &Path) -> Vec<(PathBuf, String)> {
    let mut result = Vec::new();
    let Ok(entries) = fs::read_dir(folder) else {
        return result;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if is_symlink(&path) {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("reg") {
            continue;
        }
        let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if let Some(ts) = extract_timestamp(filename) {
            result.push((path, ts));
        }
    }
    result
}

/// Decode un fichier .reg UTF-16 LE (BOM optionnel) en String.
///
/// Validation UTF-16 stricte (fix M1 v2.7.1) : rejette les paires substitut
/// orphelines plutot que de les remplacer par U+FFFD silencieusement.
pub(super) fn decode_reg_file(path: &PathBuf) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    if bytes.len() < 2 {
        return None;
    }

    let payload = if bytes[0] == 0xFF && bytes[1] == 0xFE {
        &bytes[2..]
    } else {
        &bytes[..]
    };

    if payload.len() % 2 != 0 {
        return None;
    }

    let u16_buf: Vec<u16> = payload
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect();

    match String::from_utf16(&u16_buf) {
        Ok(s) => Some(s),
        Err(_) => {
            crate::utils::logger::warn(
                "snapshot_diff",
                &format!("Sequence UTF-16 invalide dans {}", path.display()),
            );
            None
        }
    }
}

fn normalize_hive(full: &str) -> String {
    full.replace("HKEY_LOCAL_MACHINE", "HKLM")
        .replace("HKEY_CURRENT_USER", "HKCU")
        .replace("HKEY_CLASSES_ROOT", "HKCR")
}

/// Parse le contenu d'un fichier .reg textuel en tuples (section, name, raw_value).
///
/// `raw_value` est conserve sous sa forme .reg brute (`dword:00000001`,
/// `"hello"`, etc.) pour permettre une comparaison directe entre snapshots
/// sans re-encodage. `name` est vide pour la valeur par defaut `@`.
pub fn parse_reg_content(content: &str) -> Vec<(String, String, String)> {
    let mut result = Vec::new();
    let mut current_section: Option<String> = None;
    let mut lines = content.lines();

    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with("Windows Registry Editor") {
            continue;
        }

        if let Some(stripped) = line.strip_prefix('[') {
            if let Some(section) = stripped.strip_suffix(']') {
                current_section = Some(normalize_hive(section));
            }
            continue;
        }

        if let Some(section) = &current_section {
            if let Some((raw_name, raw_value)) = line.split_once('=') {
                let name = if raw_name.trim() == "@" {
                    String::new()
                } else if let Some(stripped) = raw_name.trim().strip_prefix('"') {
                    stripped.strip_suffix('"').unwrap_or(stripped).to_string()
                } else {
                    continue;
                };
                let mut value = raw_value.trim().to_string();
                // reg.exe coupe les valeurs binaires longues par un antislash en fin de
                // ligne, repris sur la ligne suivante : on les recolle pour qu'un meme
                // contenu donne la meme valeur quel que soit son decoupage.
                if value.starts_with("hex") {
                    while value.ends_with('\\') {
                        value.pop();
                        match lines.next() {
                            Some(next) => value.push_str(next.trim()),
                            None => break,
                        }
                    }
                }
                result.push((section.clone(), name, value));
            }
        }
    }

    result
}

/// Extrait le timestamp (format YYYYMMDD_HHMMSS) du nom de fichier de backup.
///
/// Format attendu : `<prefix>_<timestamp>.reg`. Retourne `None` si le format
/// ne correspond pas (filename trop court, separateur manquant, caracteres
/// non numeriques).
pub(super) fn extract_timestamp(filename: &str) -> Option<String> {
    let stem = filename.strip_suffix(".reg")?;
    if stem.len() < TIMESTAMP_LEN {
        return None;
    }
    let candidate = stem.get(stem.len() - TIMESTAMP_LEN..)?;
    let bytes = candidate.as_bytes();
    if bytes[8] == b'_'
        && bytes[..8].iter().all(u8::is_ascii_digit)
        && bytes[9..].iter().all(u8::is_ascii_digit)
    {
        Some(candidate.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
