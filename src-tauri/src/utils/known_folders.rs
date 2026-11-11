//! Dossiers connus de Windows : le Bureau suit la redirection (OneDrive, profil
//! itinerant) que `USERPROFILE\Desktop` ne voit pas.

use std::path::PathBuf;

#[repr(C)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

const FOLDERID_DESKTOP: Guid = Guid {
    data1: 0xB4BF_CC3A,
    data2: 0xDB2C,
    data3: 0x424C,
    data4: [0xB0, 0x29, 0x7F, 0xE9, 0x9A, 0x87, 0xC6, 0x41],
};

#[link(name = "shell32")]
extern "system" {
    fn SHGetKnownFolderPath(
        rfid: *const Guid,
        flags: u32,
        token: *mut core::ffi::c_void,
        path: *mut *mut u16,
    ) -> i32;
}

#[link(name = "ole32")]
extern "system" {
    fn CoTaskMemFree(ptr: *mut core::ffi::c_void);
}

pub fn desktop_dir() -> PathBuf {
    known_desktop().unwrap_or_else(fallback_desktop)
}

fn known_desktop() -> Option<PathBuf> {
    let mut raw: *mut u16 = std::ptr::null_mut();
    // SAFETY : `raw` est un pointeur de sortie valide ; en cas de succes la
    // chaine UTF-16 terminee par NUL est allouee par le systeme et liberee
    // par CoTaskMemFree apres copie.
    let hr = unsafe { SHGetKnownFolderPath(&FOLDERID_DESKTOP, 0, std::ptr::null_mut(), &mut raw) };
    if raw.is_null() {
        return None;
    }
    let path = if hr >= 0 {
        // SAFETY : succes -> `raw` pointe une chaine NUL-terminee ; on la parcourt
        // jusqu'au NUL sans depasser sa fin.
        let len = (0..).take_while(|&i| unsafe { *raw.add(i) } != 0).count();
        let slice = unsafe { std::slice::from_raw_parts(raw, len) };
        Some(PathBuf::from(String::from_utf16_lossy(slice)))
    } else {
        None
    };
    // SAFETY : `raw` a ete alloue par SHGetKnownFolderPath, y compris en cas d'echec.
    unsafe { CoTaskMemFree(raw as *mut core::ffi::c_void) };
    path
}

fn fallback_desktop() -> PathBuf {
    let profile = std::env::var("USERPROFILE").unwrap_or_else(|_| {
        let drive = std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string());
        PathBuf::from(drive).join("Users").join("Default").to_string_lossy().to_string()
    });
    PathBuf::from(profile).join("Desktop")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_is_resolved_to_an_existing_directory() {
        let dir = known_desktop().expect("SHGetKnownFolderPath doit repondre");
        assert!(dir.is_dir(), "{}", dir.display());
    }

    #[test]
    fn fallback_ends_with_desktop() {
        assert!(fallback_desktop().ends_with("Desktop"));
    }
}
