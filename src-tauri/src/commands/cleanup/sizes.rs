use std::fs;
use std::path::PathBuf;
use crate::utils::{cleanup_helpers, constants, logger, process};

pub(super) fn get_folder_size(path: &str, pattern: &str) -> Option<u64> {
    let dir = PathBuf::from(path);
    if !dir.exists() {
        return Some(0);
    }

    let mut total: u64 = 0;
    match fs::read_dir(&dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if pattern == "*" {
                    total += cleanup_helpers::entry_size(&entry);
                } else {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if cleanup_helpers::matches_pattern(&name, pattern) {
                        if let Ok(meta) = entry.metadata() {
                            total += meta.len();
                        }
                    }
                }
            }
        }
        Err(e) => {
            logger::warn(
                "cleanup.lecture",
                &format!("get_folder_size: lecture impossible de {} (cible marquée inaccessible): {}", path, e),
            );
            return None;
        }
    }
    Some(total)
}

pub(super) fn get_recycle_bin_size() -> Option<u64> {
    let result = process::run(
        "powershell.exe",
        "-NoProfile -Command \"(New-Object -ComObject Shell.Application).NameSpace(10).Items() | ForEach-Object { $_.Size } | Measure-Object -Sum | Select-Object -ExpandProperty Sum\"",
        constants::TIMEOUT_MEDIUM,
    );
    if result.success {
        let trimmed = result.stdout.trim();
        if trimmed.is_empty() {
            return Some(0); // corbeille vide : Measure-Object Sum est vide, etat normal
        }
        trimmed.parse().ok().or_else(|| {
            logger::warn("cleanup.lecture", &format!("get_recycle_bin_size: taille non numerique '{}'", trimmed));
            None
        })
    } else {
        logger::warn(
            "cleanup.lecture",
            &format!("get_recycle_bin_size: échec requête corbeille (exit {}): {}", result.exit_code, result.stderr),
        );
        None
    }
}
