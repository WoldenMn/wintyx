use windows_registry::*;

/// Préfixes registre autorisés pour les opérations Wintyx (lecture/écriture/backup).
/// Source unique de vérité — réutilisée par backup_registry_key et restore_selected_changes.
pub const ALLOWED_KEY_PREFIXES: &[&str] = &[
    "HKLM\\SOFTWARE",
    "HKLM\\SYSTEM",
    "HKCU\\SOFTWARE",
    "HKCU\\Control Panel",
];

/// Vérifie qu'un chemin registre commence par un préfixe Wintyx autorisé.
///
/// Comparaison **case-insensitive** : le registre Windows est case-insensitive,
/// donc `hklm\software\...` doit être traité comme `HKLM\SOFTWARE\...` pour
/// éviter qu'un attaquant ne contourne l'allowlist via une variation de casse.
pub fn is_allowed_key_path(path: &str) -> bool {
    ALLOWED_KEY_PREFIXES
        .iter()
        .any(|prefix| is_same_or_subkey(path, prefix))
}

/// Vérifie qu'un chemin registre cible HKLM (élévation requise).
///
/// Comparaison **case-insensitive** pour les mêmes raisons que
/// `is_allowed_key_path`.
pub fn is_hklm_path(path: &str) -> bool {
    path.len() >= 4 && path.as_bytes()[..4].eq_ignore_ascii_case(b"HKLM")
}

/// Valide un nom destine a etre utilise comme **identifiant** dans une commande
/// systeme Windows : nom de service (sc.exe), composant de chemin registre, etc.
/// (Les noms d'entrees de demarrage sont des noms de valeurs registre et passent
/// par [`sanitize_value_name`], cf. `commands::startup::sanitize_startup_name`.)
///
/// **Distinction importante avec [`sanitize_value_name`]** :
/// - Cette fonction valide les **noms d'identifiants** (services, cles, entrees)
///   qui sont passes a des commandes shell ou utilises comme path components.
///   Les espaces y sont **interdits** (risque de splitting d'arguments).
/// - [`sanitize_value_name`] valide les **noms de valeurs registre** (ex:
///   `"Display Name"`) ou les espaces sont **legitimes**.
/// - Ne **JAMAIS** utiliser [`sanitize_value_name`] sur un nom de service ou
///   de cle : la validation des espaces y est plus permissive.
///
/// Contraintes (defense-in-depth contre command injection) :
/// - non vide apres trim, <= 256 chars
/// - **ASCII uniquement** (rejette Unicode pour eviter les confusables / bypass)
/// - alphanumeriques + `[- _ .]` (les noms de services et cles registre
///   Windows ne contiennent jamais de caracteres speciaux type `"&|;$\`<>` etc.)
/// - **Espaces rejetes** : evite les problemes de splitting d'arguments dans
///   les invocations shell (sc.exe, reg.exe) meme si elles quotent l'argument
pub fn sanitize_name(input: &str) -> std::result::Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if trimmed.len() > 256 {
        return Err("Name too long".to_string());
    }
    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
    {
        Ok(trimmed.to_string())
    } else {
        Err(format!("Invalid characters in name: {}", trimmed))
    }
}

/// Validation pour les **noms de valeurs registre uniquement** (par exemple
/// la cle `HKLM\Software\Foo` peut contenir une valeur nommee `"Display Name"`).
///
/// **Distinction importante avec [`sanitize_name`]** :
/// - Cette fonction est plus permissive (autorise les espaces ASCII et la
///   chaine vide pour le cas valeur par defaut `@`).
/// - **Ne jamais l'utiliser** pour valider un nom de service, une cle
///   registre ou un identifiant passe a une commande shell : la validation
///   des espaces y est trop permissive.
/// - Pour ces cas, utiliser [`sanitize_name`] qui rejette les espaces.
///
/// Rejette toujours : Unicode (confusables), caracteres de controle,
/// caracteres shell (`\` `/` `"` `'` `;` `&` `|` `$` `` ` `` etc.).
pub fn sanitize_value_name(input: &str) -> std::result::Result<String, String> {
    if input.is_empty() {
        return Ok(String::new());
    }
    if input.len() > 256 {
        return Err("Value name too long".to_string());
    }
    if input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || " -_.".contains(c))
    {
        Ok(input.to_string())
    } else {
        Err(format!("Invalid characters in value name: {}", input))
    }
}

/// Valide un **chemin de cle registre** destine a `reg.exe export`.
///
/// Distinct de [`sanitize_name`] : les chemins de cles (ex
/// `HKLM\SYSTEM\...\Terminal Server`) contiennent des `\` et des **espaces
/// legitimes**. Autorise alphanumerique + `\ _ - . espace` ; rejette l'Unicode
/// et tout metacaractere shell (`" & | ; $ \` < >` etc.). A combiner avec
/// [`is_allowed_key_path`] qui restreint le prefixe autorise.
pub fn sanitize_key_path(path: &str) -> std::result::Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Registry key path cannot be empty".to_string());
    }
    if trimmed.len() > 512 {
        return Err("Registry key path too long".to_string());
    }
    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "\\_-. ".contains(c))
    {
        Ok(())
    } else {
        Err(format!("Invalid characters in registry key path: {}", trimmed))
    }
}

pub(super) fn parse_root_and_subpath(path: &str) -> Option<(&Key, &str)> {
    let pos = path.find('\\')?;
    let (prefix, rest) = (&path[..pos], &path[pos + 1..]);

    let root: &Key = match prefix.to_uppercase().as_str() {
        "HKLM" | "HKEY_LOCAL_MACHINE" => LOCAL_MACHINE,
        "HKCU" | "HKEY_CURRENT_USER" => CURRENT_USER,
        "HKCR" | "HKEY_CLASSES_ROOT" => CLASSES_ROOT,
        _ => return None,
    };

    Some((root, rest))
}

pub fn get_dword(path: &str, name: &str) -> Option<u32> {
    let Some((root, sub)) = parse_root_and_subpath(path) else {
        crate::utils::logger::warn("registry", &format!("get_dword: chemin registre invalide '{}'", path));
        return None;
    };
    let key = root.open(sub).ok()?;
    key.get_u32(name).ok()
}

pub fn get_string(path: &str, name: &str) -> Option<String> {
    let Some((root, sub)) = parse_root_and_subpath(path) else {
        crate::utils::logger::warn("registry", &format!("get_string: chemin registre invalide '{}'", path));
        return None;
    };
    let key = root.open(sub).ok()?;
    key.get_string(name).ok()
}

pub fn set_dword(path: &str, name: &str, value: u32) -> std::result::Result<(), String> {
    backup::backup_before_write(path, name);
    let (root, sub) = parse_root_and_subpath(path)
        .ok_or_else(|| format!("Invalid registry path: {}", path))?;
    let key = root.create(sub).map_err(|e| e.to_string())?;
    key.set_u32(name, value).map_err(|e| e.to_string())
}

pub fn set_string(path: &str, name: &str, value: &str) -> std::result::Result<(), String> {
    backup::backup_before_write(path, name);
    let (root, sub) = parse_root_and_subpath(path)
        .ok_or_else(|| format!("Invalid registry path: {}", path))?;
    let key = root.create(sub).map_err(|e| e.to_string())?;
    key.set_string(name, value).map_err(|e| e.to_string())
}

const ERROR_FILE_NOT_FOUND: u32 = 2;
const HRESULT_FILE_NOT_FOUND: u32 = 0x8007_0002;
const KEY_SET_VALUE: u32 = 0x0002;

#[link(name = "advapi32")]
extern "system" {
    fn RegOpenKeyExW(
        hkey: *mut core::ffi::c_void,
        sub_key: *const u16,
        options: u32,
        sam_desired: u32,
        result: *mut *mut core::ffi::c_void,
    ) -> u32;
}

fn open_for_value_write(root: &Key, sub: &str) -> std::result::Result<Key, u32> {
    let wide: Vec<u16> = sub.encode_utf16().chain(std::iter::once(0)).collect();
    let mut handle: *mut core::ffi::c_void = std::ptr::null_mut();
    // SAFETY: `root.as_raw()` est un HKEY valide tant que `root` vit ; `wide` est
    // terminee par nul et reste vivante pendant l'appel ; `handle` est une sortie
    // valide. En cas de succes (0) le handle nous appartient : `Key::from_raw`
    // en prend la propriete et le fermera par `RegCloseKey` au drop. `Key::open`
    // de windows-registry 0.4 ouvre en KEY_READ seul, d'ou ce passage par FFI.
    let code = unsafe { RegOpenKeyExW(root.as_raw(), wide.as_ptr(), 0, KEY_SET_VALUE, &mut handle) };
    if code != 0 {
        return Err(code);
    }
    // SAFETY: handle non nul fraichement ouvert par RegOpenKeyExW, jamais partage.
    Ok(unsafe { Key::from_raw(handle) })
}

/// Supprime une valeur registre. Une cle ou une valeur deja absente
/// (`ERROR_FILE_NOT_FOUND`) vaut succes : l'operation est idempotente.
pub fn delete_value(path: &str, name: &str) -> std::result::Result<(), String> {
    let (root, sub) = parse_root_and_subpath(path)
        .ok_or_else(|| format!("Invalid registry path: {}", path))?;
    backup::backup_before_write(path, name);
    let key = match open_for_value_write(root, sub) {
        Ok(key) => key,
        Err(ERROR_FILE_NOT_FOUND) => return Ok(()),
        Err(code) => return Err(format!("RegOpenKeyExW({}) a échoué : code {}", path, code)),
    };
    match key.remove_value(name) {
        Ok(()) => Ok(()),
        Err(e) if e.code().0 as u32 == HRESULT_FILE_NOT_FOUND => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Deplace une valeur chaine d'une cle vers une autre : ecrit d'abord la
/// destination, ne supprime la source qu'apres cette ecriture reussie (un echec
/// laisse la valeur d'origine en place). Une valeur source absente ou d'un type
/// non pris en charge est une erreur : rien n'a ete deplace.
/// Le type est conserve : un `REG_EXPAND_SZ` reste `REG_EXPAND_SZ`, sinon les
/// variables d'environnement (`%ProgramFiles%`) ne seraient plus developpees.
pub fn move_string_value(src: &str, dst: &str, name: &str) -> std::result::Result<(), String> {
    let (src_root, src_sub) = parse_root_and_subpath(src)
        .ok_or_else(|| format!("Invalid registry path: {}", src))?;
    let src_key = src_root
        .open(src_sub)
        .map_err(|e| format!("Clé source illisible ou introuvable ({}) : {}", src, e.message()))?;
    let value = src_key
        .get_value(name)
        .map_err(|e| format!("Valeur '{}' introuvable ou illisible dans {} : {}", name, src, e.message()))?;
    if !matches!(value.ty(), Type::String | Type::ExpandString) {
        return Err(format!(
            "Valeur '{}' de {} : type {:?} non pris en charge, rien n'a été déplacé",
            name,
            src,
            value.ty()
        ));
    }
    let (dst_root, dst_sub) = parse_root_and_subpath(dst)
        .ok_or_else(|| format!("Invalid registry path: {}", dst))?;
    backup::backup_before_write(dst, name);
    let dst_key = dst_root.create(dst_sub).map_err(|e| e.to_string())?;
    dst_key.set_value(name, &value).map_err(|e| e.to_string())?;
    delete_value(src, name)
}

pub fn key_exists(path: &str, name: &str) -> bool {
    let Some((root, sub)) = parse_root_and_subpath(path) else {
        return false;
    };
    let Ok(key) = root.open(sub) else {
        return false;
    };
    key.get_string(name).is_ok() || key.get_u32(name).is_ok()
}

pub fn ensure_key(path: &str) -> std::result::Result<(), String> {
    let (root, sub) = parse_root_and_subpath(path)
        .ok_or_else(|| format!("Invalid registry path: {}", path))?;
    root.create(sub).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_value_names(path: &str) -> Vec<String> {
    let Some((root, sub)) = parse_root_and_subpath(path) else {
        return Vec::new();
    };
    let Ok(key) = root.open(sub) else {
        return Vec::new();
    };
    match key.values() {
        Ok(iter) => iter.map(|(name, _)| name).collect(),
        Err(_) => Vec::new(),
    }
}

mod backup;
mod read;
mod within;
pub use read::read_dword;
pub use within::is_same_or_subkey;

pub use backup::path_hash;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod write_tests;
