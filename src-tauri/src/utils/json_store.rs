//! Lecture et ecriture de fichiers JSON de donnees utilisateur.
//!
//! Un fichier illisible n'est jamais ecrase : il est renomme en
//! `<nom>.corrompu-<horodatage>.json` avant que l'appelant ne reecrive. Une erreur
//! d'E/S remonte a l'appelant au lieu d'etre prise pour un fichier vide.

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::errors::CommandError;
use super::{fs_atomic, logger};

pub fn load_or_quarantine<T: DeserializeOwned + Default>(
    path: &Path,
    module: &str,
) -> Result<T, CommandError> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(T::default()),
        Err(e) => {
            return Err(CommandError::Generic(format!(
                "Lecture de {} impossible : {}",
                path.display(),
                e
            )))
        }
    };
    match serde_json::from_str::<T>(&content) {
        Ok(value) => Ok(value),
        Err(parse_error) => {
            let moved = quarantine(path).map_err(|e| {
                CommandError::Generic(format!(
                    "{} est illisible ({}) et n'a pas pu etre mis de cote : {}",
                    path.display(),
                    parse_error,
                    e
                ))
            })?;
            logger::warn(
                module,
                &format!(
                    "{} illisible ({}), conserve sous {}",
                    path.display(),
                    parse_error,
                    moved.display()
                ),
            );
            Ok(T::default())
        }
    }
}

pub fn write_json_atomic<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), CommandError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| CommandError::Generic(format!("Serialisation de {} : {}", path.display(), e)))?;
    fs_atomic::write_atomic(path, json.as_bytes())
        .map_err(|e| CommandError::Generic(format!("Écriture de {} : {}", path.display(), e)))
}

fn quarantine(path: &Path) -> std::io::Result<PathBuf> {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let mut target = path.with_file_name(format!("{}.corrompu-{}.json", stem, stamp));
    let mut n = 1;
    while target.exists() {
        target = path.with_file_name(format!("{}.corrompu-{}-{}.json", stem, stamp, n));
        n += 1;
    }
    fs::rename(path, &target)?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("wintyx-json-store-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn quarantined(dir: &Path) -> Vec<PathBuf> {
        fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.to_string_lossy().contains(".corrompu-"))
            .collect()
    }

    #[test]
    fn missing_file_gives_default() {
        let d = dir("missing");
        let v: Vec<u32> = load_or_quarantine(&d.join("a.json"), "t").unwrap();
        assert!(v.is_empty());
    }

    #[test]
    fn valid_file_is_loaded_and_untouched() {
        let d = dir("valid");
        let p = d.join("a.json");
        fs::write(&p, "[1,2,3]").unwrap();
        let v: Vec<u32> = load_or_quarantine(&p, "t").unwrap();
        assert_eq!(v, vec![1, 2, 3]);
        assert!(p.exists());
        assert!(quarantined(&d).is_empty());
    }

    #[test]
    fn corrupted_file_is_renamed_with_its_content_kept() {
        let d = dir("corrupt");
        let p = d.join("a.json");
        fs::write(&p, "[1,2,").unwrap();
        let v: Vec<u32> = load_or_quarantine(&p, "t").unwrap();
        assert!(v.is_empty());
        assert!(!p.exists());
        let moved = quarantined(&d);
        assert_eq!(moved.len(), 1);
        assert_eq!(fs::read_to_string(&moved[0]).unwrap(), "[1,2,");
    }

    #[test]
    fn two_corruptions_in_the_same_second_do_not_overwrite_each_other() {
        let d = dir("twice");
        let p = d.join("a.json");
        for content in ["[1,", "[2,"] {
            fs::write(&p, content).unwrap();
            let _: Vec<u32> = load_or_quarantine(&p, "t").unwrap();
        }
        assert_eq!(quarantined(&d).len(), 2);
    }

    #[test]
    fn io_error_other_than_not_found_is_reported() {
        let d = dir("ioerr");
        let p = d.join("a.json");
        fs::create_dir(&p).unwrap();
        let r: Result<Vec<u32>, _> = load_or_quarantine(&p, "t");
        assert!(r.is_err(), "un dossier a la place du fichier doit remonter");
        assert!(p.exists());
    }

    #[test]
    fn write_json_atomic_round_trips() {
        let d = dir("write");
        let p = d.join("a.json");
        write_json_atomic(&p, &vec![7u32, 8]).unwrap();
        let v: Vec<u32> = load_or_quarantine(&p, "t").unwrap();
        assert_eq!(v, vec![7, 8]);
    }
}
