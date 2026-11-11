//! Ecriture atomique d'un fichier : tempfile unique dans le meme dossier puis
//! `fs::rename`, atomique sur NTFS et qui ecrase la cible.
//!
//! Le nom temporaire est propre a chaque appel (pid + compteur) : deux ecritures
//! concurrentes de la meme cible ne se marchent plus dessus.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static WRITE_SEQ: AtomicU64 = AtomicU64::new(0);

fn unique_temp_path(target: &Path) -> PathBuf {
    let seq = WRITE_SEQ.fetch_add(1, Ordering::Relaxed);
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    target.with_file_name(format!("{}.{}.{}.tmp", name, std::process::id(), seq))
}

pub fn write_atomic(target: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = unique_temp_path(target);
    fs::write(&temp, contents)?;
    if let Err(e) = fs::rename(&temp, target) {
        let _ = fs::remove_file(&temp);
        return Err(e);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_target(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("wintyx-fs-atomic-{}", std::process::id()))
            .join(name)
    }

    #[test]
    fn writes_and_overwrites_without_leaving_tempfiles() {
        let target = temp_target("basic.json");
        write_atomic(&target, b"v1").unwrap();
        write_atomic(&target, b"v2").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"v2");
        let orphans: Vec<_> = fs::read_dir(target.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("basic.json."))
            .collect();
        assert!(orphans.is_empty(), "tempfiles orphelins : {:?}", orphans);
    }

    #[test]
    fn temp_names_differ_between_calls() {
        let target = temp_target("names.json");
        assert_ne!(unique_temp_path(&target), unique_temp_path(&target));
    }

    #[test]
    fn concurrent_writes_to_same_target_all_succeed() {
        let target = temp_target("concurrent.json");
        let threads: Vec<_> = (0..16)
            .map(|i| {
                let target = target.clone();
                std::thread::spawn(move || write_atomic(&target, format!("{{\"n\":{}}}", i).as_bytes()))
            })
            .collect();
        for t in threads {
            t.join().unwrap().unwrap();
        }
        let content = fs::read_to_string(&target).unwrap();
        assert!(content.starts_with("{\"n\":"), "contenu tronque : {}", content);
    }
}
