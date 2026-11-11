//! Helpers purs pour le nettoyage (format taille, pattern matching, taille recursive).
//!
//! Extraits de `commands/cleanup.rs` en v2.9.0 (decouplage logique pure / I/O metier).
//! Ces helpers ne dependent que de `std::fs` + `super::constants`.

use std::fs::{self, DirEntry};
use std::path::Path;

use super::constants;

/// Formate une taille en octets en chaine lisible (o / Ko / Mo / Go).
pub(crate) fn format_size(bytes: u64) -> String {
    if bytes < constants::BYTES_KB {
        format!("{} o", bytes)
    } else if bytes < constants::BYTES_MB {
        format!("{:.1} Ko", bytes as f64 / constants::KB)
    } else if bytes < constants::GB as u64 {
        format!("{:.1} Mo", bytes as f64 / constants::MB)
    } else {
        format!("{:.2} Go", bytes as f64 / constants::GB)
    }
}

/// Vrai si `name` matche `pattern` (wildcards limites : `*`, `*.ext`, `prefix*suffix`).
///
/// `pattern` est mis en minuscules. `name` doit l'etre aussi (responsabilite du caller).
pub(crate) fn matches_pattern(name: &str, pattern: &str) -> bool {
    let pattern = pattern.to_lowercase();
    if pattern == "*" {
        return true;
    }
    if let Some(ext) = pattern.strip_prefix("*.") {
        return name.ends_with(&format!(".{}", ext));
    }
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return name.starts_with(parts[0]) && name.ends_with(parts[1]);
        }
    }
    name == pattern
}

/// Taille totale d'un dossier (recursive). Retourne 0 si le dossier n'existe pas ou est inaccessible.
pub(crate) fn dir_size_recursive(path: &Path) -> u64 {
    let mut total: u64 = 0;
    let mut pending = vec![path.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                } else if meta.is_dir() {
                    pending.push(entry.path());
                }
            }
        }
    }
    total
}

/// Taille d'une entree (fichier ou dossier). Retourne 0 si metadata inaccessible.
pub(crate) fn entry_size(entry: &DirEntry) -> u64 {
    match entry.metadata() {
        Ok(meta) => {
            if meta.is_file() {
                meta.len()
            } else if meta.is_dir() {
                dir_size_recursive(&entry.path())
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(0), "0 o");
        assert_eq!(format_size(512), "512 o");
        assert_eq!(format_size(1023), "1023 o");
    }

    #[test]
    fn format_size_kb() {
        assert_eq!(format_size(1024), "1.0 Ko");
        assert_eq!(format_size(2048), "2.0 Ko");
    }

    #[test]
    fn format_size_mb_gb() {
        assert_eq!(format_size(1024 * 1024), "1.0 Mo");
        assert_eq!(format_size(1024u64.pow(3)), "1.00 Go");
    }

    #[test]
    fn matches_pattern_wildcard() {
        assert!(matches_pattern("anything.log", "*"));
        assert!(matches_pattern("", "*"));
    }

    #[test]
    fn matches_pattern_extension() {
        assert!(matches_pattern("data.log", "*.log"));
        assert!(!matches_pattern("data.txt", "*.log"));
    }

    #[test]
    fn matches_pattern_prefix_suffix() {
        assert!(matches_pattern("thumbcache_42.db", "thumbcache_*.db"));
        assert!(!matches_pattern("other_42.db", "thumbcache_*.db"));
    }

    #[test]
    fn matches_pattern_exact() {
        assert!(matches_pattern("desktop.ini", "desktop.ini"));
        assert!(!matches_pattern("other.ini", "desktop.ini"));
    }

    #[test]
    fn matches_pattern_case_insensitive_pattern() {
        assert!(matches_pattern("data.log", "*.LOG"));
    }

    #[test]
    fn dir_size_recursive_empty_dir() {
        let tmp = std::env::temp_dir().join(format!("wintyx_test_empty_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        assert_eq!(dir_size_recursive(&tmp), 0);
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn dir_size_recursive_nonexistent_returns_zero() {
        let nonexistent = std::env::temp_dir().join("wintyx_does_not_exist_xyz_123");
        let _ = fs::remove_dir_all(&nonexistent);
        assert_eq!(dir_size_recursive(&nonexistent), 0);
    }

    #[test]
    fn dir_size_recursive_counts_nested_folders() {
        let tmp = std::env::temp_dir().join(format!("wintyx_test_nested_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let deep = tmp.join("a").join("b").join("c");
        fs::create_dir_all(&deep).unwrap();
        fs::write(tmp.join("racine.txt"), b"123").unwrap();
        fs::write(tmp.join("a").join("un.txt"), b"4567").unwrap();
        fs::write(deep.join("profond.txt"), b"89").unwrap();
        let size = dir_size_recursive(&tmp);
        fs::remove_dir_all(&tmp).ok();
        assert_eq!(size, 9);
    }

    #[test]
    fn dir_size_recursive_sums_files() {
        let tmp = std::env::temp_dir().join(format!(
            "wintyx_test_sum_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        assert!(tmp.exists(), "tmp dir should exist after create_dir_all");
        {
            let mut f = File::create(tmp.join("a.txt")).unwrap();
            f.write_all(b"hello").unwrap();
            f.sync_all().unwrap();
        }
        {
            let mut f2 = File::create(tmp.join("b.txt")).unwrap();
            f2.write_all(b"world!").unwrap();
            f2.sync_all().unwrap();
        }
        let size = dir_size_recursive(&tmp);
        fs::remove_dir_all(&tmp).ok();
        assert_eq!(size, 5 + 6);
    }
}
