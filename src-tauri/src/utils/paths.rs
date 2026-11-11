//! Chemins systeme partages entre modules de commandes.

use std::path::PathBuf;

/// Dossier de donnees Wintyx : `%LOCALAPPDATA%\Wintyx`.
/// Resout `LOCALAPPDATA`, avec repli robuste via `USERPROFILE` puis `SYSTEMDRIVE`.
pub fn wintyx_local_dir() -> PathBuf {
    #[cfg(test)]
    {
        test_root().join("Local").join("Wintyx")
    }
    #[cfg(not(test))]
    real_local_dir()
}

/// Dossier de donnees roaming : `%APPDATA%\Wintyx`.
pub fn wintyx_roaming_dir() -> PathBuf {
    #[cfg(test)]
    {
        test_root().join("Roaming").join("Wintyx")
    }
    #[cfg(not(test))]
    real_roaming_dir()
}

/// Dossier des journaux : `%LOCALAPPDATA%\Wintyx\Logs`.
pub fn logs_dir() -> PathBuf {
    wintyx_local_dir().join("Logs")
}

#[cfg(test)]
fn test_root() -> PathBuf {
    std::env::temp_dir().join(format!("wintyx-test-{}", std::process::id()))
}

#[cfg(not(test))]
fn real_roaming_dir() -> PathBuf {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| {
        let profile = std::env::var("USERPROFILE").unwrap_or_else(|_| {
            let drive = std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string());
            PathBuf::from(drive)
                .join("Users")
                .join("Default")
                .to_string_lossy()
                .to_string()
        });
        PathBuf::from(profile)
            .join("AppData")
            .join("Roaming")
            .to_string_lossy()
            .to_string()
    });
    PathBuf::from(app_data).join("Wintyx")
}

#[cfg(not(test))]
fn real_local_dir() -> PathBuf {
    let local_app = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
        let profile = std::env::var("USERPROFILE").unwrap_or_else(|_| {
            let drive = std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string());
            PathBuf::from(drive)
                .join("Users")
                .join("Default")
                .to_string_lossy()
                .to_string()
        });
        PathBuf::from(profile)
            .join("AppData")
            .join("Local")
            .to_string_lossy()
            .to_string()
    });
    PathBuf::from(local_app).join("Wintyx")
}

/// Dossier des sauvegardes Wintyx : `%LOCALAPPDATA%\Wintyx\Backups`.
pub fn backup_folder() -> PathBuf {
    wintyx_local_dir().join("Backups")
}

/// Fichier d'historique des scores : `%LOCALAPPDATA%\Wintyx\scores_history.json`.
pub fn scores_history_path() -> PathBuf {
    wintyx_local_dir().join("scores_history.json")
}

/// Historique des benchmarks : fichier `benchmarks.json` du dossier de donnees local.
pub fn benchmark_history_path() -> PathBuf {
    wintyx_local_dir().join("benchmarks.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builds_never_resolve_under_real_appdata() {
        for dir in [logs_dir(), backup_folder(), wintyx_roaming_dir(), scores_history_path(), benchmark_history_path()] {
            assert!(dir.starts_with(std::env::temp_dir()), "{}", dir.display());
            assert!(dir.to_string_lossy().contains("wintyx-test-"), "{}", dir.display());
        }
    }

    #[test]
    fn backup_folder_targets_wintyx_backups() {
        // Verrouille le comportement apres la deduplication arch-03 : quel que soit
        // l'environnement (LOCALAPPDATA defini ou repli), le dossier vise toujours
        // ...\Wintyx\Backups.
        let folder = backup_folder();
        assert!(folder.ends_with("Backups"), "dernier composant attendu : Backups");
        assert!(
            folder.to_string_lossy().contains("Wintyx"),
            "chemin attendu sous Wintyx : {}",
            folder.display()
        );
    }
}
