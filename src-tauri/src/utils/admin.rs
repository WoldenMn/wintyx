use std::sync::OnceLock;

use serde::Serialize;
use tauri::AppHandle;

use super::constants;
use super::errors::CommandError;
use super::logger;
use super::process;

static IS_ADMIN_CACHE: OnceLock<bool> = OnceLock::new();

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AdminStatus {
    pub is_admin: bool,
    pub can_elevate: bool,
}

/// Detecte si le processus courant tourne avec un jeton eleve (UAC accepte).
///
/// Strategie : lit `TokenElevation` du jeton du processus, qui distingue un admin
/// eleve d'un admin filtre par UAC, sans lancer de processus ni dependre de la
/// langue ou du service Serveur. Le resultat est mis en cache car l'elevation ne
/// change pas pendant la vie du processus.
pub fn is_admin() -> bool {
    *IS_ADMIN_CACHE.get_or_init(detect_admin)
}

#[cfg(windows)]
fn detect_admin() -> bool {
    match token_is_elevated() {
        Ok(elevated) => elevated,
        Err(e) => {
            logger::warn("admin", &format!("Elevation du jeton illisible, traite comme non admin: {}", e));
            false
        }
    }
}

#[cfg(windows)]
const TOKEN_QUERY: u32 = 0x0008;
#[cfg(windows)]
const TOKEN_ELEVATION_CLASS: u32 = 20;

#[cfg(windows)]
#[repr(C)]
struct TokenElevation {
    token_is_elevated: u32,
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut core::ffi::c_void;
    fn CloseHandle(handle: *mut core::ffi::c_void) -> i32;
}

#[cfg(windows)]
#[link(name = "advapi32")]
extern "system" {
    fn OpenProcessToken(
        process: *mut core::ffi::c_void,
        desired_access: u32,
        token: *mut *mut core::ffi::c_void,
    ) -> i32;
    fn GetTokenInformation(
        token: *mut core::ffi::c_void,
        class: u32,
        info: *mut core::ffi::c_void,
        info_len: u32,
        return_len: *mut u32,
    ) -> i32;
}

#[cfg(windows)]
fn token_is_elevated() -> Result<bool, String> {
    let mut token: *mut core::ffi::c_void = std::ptr::null_mut();
    // SAFETY: GetCurrentProcess renvoie un pseudo-handle valide sans besoin de le
    // fermer ; `token` est une sortie valide. En cas de succes le handle obtenu
    // nous appartient et est ferme ci-dessous par CloseHandle.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(format!("OpenProcessToken: {}", std::io::Error::last_os_error()));
    }
    let mut elevation = TokenElevation { token_is_elevated: 0 };
    let mut returned = 0u32;
    // SAFETY: `token` est un handle TOKEN_QUERY valide ; `elevation` est un
    // TOKEN_ELEVATION (un u32, repr(C)) dont la taille est passee en longueur ;
    // `returned` est une sortie valide.
    let ok = unsafe {
        GetTokenInformation(
            token,
            TOKEN_ELEVATION_CLASS,
            &mut elevation as *mut TokenElevation as *mut core::ffi::c_void,
            std::mem::size_of::<TokenElevation>() as u32,
            &mut returned,
        )
    };
    let last_error = std::io::Error::last_os_error();
    // SAFETY: `token` provient d'un OpenProcessToken reussi et n'est ferme qu'ici.
    unsafe { CloseHandle(token) };
    if ok == 0 {
        return Err(format!("GetTokenInformation: {}", last_error));
    }
    Ok(elevation.token_is_elevated != 0)
}

#[cfg(not(windows))]
fn detect_admin() -> bool {
    false
}

/// Retourne une erreur standard si le processus n'a pas les privileges admin.
///
/// A utiliser en debut des commandes Tauri necessitant une elevation pour echouer
/// tot avec un message utilisable par le frontend, plutot que d'attendre un echec
/// PowerShell/registre cryptique.
pub fn require_admin(feature: &str) -> Result<(), CommandError> {
    if is_admin() {
        Ok(())
    } else {
        logger::warn(
            "admin",
            &format!("Tentative d'accès à '{}' sans privilèges admin", feature),
        );
        Err(CommandError::NotAdmin(feature.to_string()))
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_admin_status() -> AdminStatus {
    AdminStatus {
        is_admin: is_admin(),
        can_elevate: cfg!(windows),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn relaunch_as_admin(app: AppHandle) -> Result<(), CommandError> {
    crate::utils::blocking::read_result(move || relaunch_as_admin_sync(app)).await
}

pub fn relaunch_as_admin_sync(app: AppHandle) -> Result<(), CommandError> {
    if is_admin() {
        return Err(CommandError::InvalidInput(
            "Wintyx tourne déjà en mode administrateur".into(),
        ));
    }

    let exe = std::env::current_exe().map_err(|e| {
        CommandError::NotFound(format!("Executable Wintyx : {}", e))
    })?;
    let exe_str = exe.to_string_lossy().to_string();
    let escaped = process::escape_powershell(&exe_str);

    let result = process::run(
        "powershell.exe",
        &format!(
            "-NoProfile -Command \"Start-Process -FilePath '{}' -Verb RunAs\"",
            escaped
        ),
        constants::TIMEOUT_MEDIUM,
    );

    if !result.success {
        let detail = if !result.stderr.trim().is_empty() {
            result.stderr.trim().to_string()
        } else {
            "UAC refusé ou commande annulée".to_string()
        };
        logger::warn("admin", &format!("Relance en admin echouee: {}", detail));
        return Err(CommandError::ProcessFailed {
            command: "Start-Process -Verb RunAs".to_string(),
            detail,
        });
    }

    logger::info("admin", "Relance en admin demandee, fermeture de l'instance");
    app.exit(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn token_elevation_matches_powershell_admin_role() {
        let reference = process::run(
            "powershell.exe",
            "-NoProfile -Command \"([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)\"",
            constants::TIMEOUT_MEDIUM,
        );
        assert!(reference.success, "reference PowerShell en echec: {}", reference.stderr);
        let expected = reference.stdout.trim().eq_ignore_ascii_case("true");
        assert_eq!(token_is_elevated().expect("jeton lisible"), expected);
    }

    #[test]
    fn is_admin_is_deterministic() {
        let a = is_admin();
        let b = is_admin();
        assert_eq!(a, b);
    }

    #[test]
    fn require_admin_returns_error_when_not_admin() {
        if !is_admin() {
            let result = require_admin("feature test");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, CommandError::NotAdmin(_)));
            let msg = err.to_string();
            assert!(msg.contains("administrateur"));
            assert!(msg.contains("feature test"));
        }
    }

    #[test]
    fn require_admin_returns_ok_when_admin() {
        if is_admin() {
            assert!(require_admin("feature test").is_ok());
        }
    }

    #[test]
    fn admin_status_reflects_is_admin() {
        let status = get_admin_status();
        assert_eq!(status.is_admin, is_admin());
    }

    #[test]
    fn admin_status_can_elevate_on_windows() {
        let status = get_admin_status();
        assert_eq!(status.can_elevate, cfg!(windows));
    }

    #[test]
    fn admin_status_serializes_camel_case() {
        let status = AdminStatus {
            is_admin: false,
            can_elevate: true,
        };
        let json = serde_json::to_value(&status).unwrap();
        assert!(json.get("isAdmin").is_some());
        assert!(json.get("canElevate").is_some());
        assert_eq!(json.get("isAdmin").unwrap(), &serde_json::json!(false));
        assert_eq!(json.get("canElevate").unwrap(), &serde_json::json!(true));
    }
}
