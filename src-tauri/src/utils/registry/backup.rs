//! Sauvegarde du contenu precedent d'une valeur avant ecriture (`RegistryBackups`).

use super::{get_dword, get_string};

pub(super) fn backup_before_write(path: &str, name: &str) {
    let old_val = match get_string(path, name) {
        Some(v) => v,
        None => match get_dword(path, name) {
            Some(v) => v.to_string(),
            None => return,
        },
    };
    if old_val.is_empty() { return; }

    let backup_dir = crate::utils::paths::wintyx_local_dir().join("RegistryBackups").to_string_lossy().to_string();
    if let Err(e) = std::fs::create_dir_all(&backup_dir) {
        crate::utils::logger::warn("registry", &format!("Création du dossier de sauvegarde impossible {}: {}", backup_dir, e));
        return;
    }

    let now = chrono::Local::now();
    let content = format!("Path: {}\nName: {}\nOldValue: {}\nTimestamp: {}\n", path, name, old_val, now.format("%Y%m%d_%H%M%S"));
    let file_name = backup_file_name(path, name, now);
    if let Err(e) = write_new_backup(std::path::Path::new(&backup_dir), &file_name, &content) {
        crate::utils::logger::warn("registry", &format!("Écriture de la sauvegarde impossible {}/{}: {}", backup_dir, file_name, e));
    }
}

fn write_new_backup(dir: &std::path::Path, file_name: &str, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    let stem = file_name.trim_end_matches(".txt");
    for attempt in 0..100u32 {
        let candidate = if attempt == 0 {
            dir.join(file_name)
        } else {
            dir.join(format!("{}_{}.txt", stem, attempt))
        };
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&candidate) {
            Ok(mut file) => return file.write_all(content.as_bytes()),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "trop de sauvegardes homonymes"))
}

/// Hash FNV-1a stable (8 hex) d'un chemin registre, insensible a la casse.
pub fn path_hash(text: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.to_ascii_lowercase().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", hash as u32)
}

pub(super) fn backup_file_name(path: &str, name: &str, now: chrono::DateTime<chrono::Local>) -> String {
    let safe_name = name.replace(['\\', '/'], "_");
    let key_hash = path_hash(path);
    format!("{}_{}_{}.txt", safe_name, key_hash, now.format("%Y%m%d_%H%M%S_%3f"))
}
