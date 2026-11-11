const ERROR_FILE_NOT_FOUND: i32 = 0x8007_0002u32 as i32;
const ERROR_PATH_NOT_FOUND: i32 = 0x8007_0003u32 as i32;

pub(super) fn is_not_found_code(code: i32) -> bool {
    code == ERROR_FILE_NOT_FOUND || code == ERROR_PATH_NOT_FOUND
}

/// Lecture qui distingue une valeur absente (`Ok(None)`) d'un echec de lecture (`Err`).
pub fn read_dword(path: &str, name: &str) -> std::result::Result<Option<u32>, String> {
    let (root, sub) = super::parse_root_and_subpath(path)
        .ok_or_else(|| format!("chemin registre invalide '{}'", path))?;
    let key = match root.open(sub) {
        Ok(key) => key,
        Err(e) if is_not_found_code(e.code().0) => return Ok(None),
        Err(e) => return Err(format!("{} : {}", path, e.message())),
    };
    match key.get_u32(name) {
        Ok(value) => Ok(Some(value)),
        Err(e) if is_not_found_code(e.code().0) => Ok(None),
        Err(e) => Err(format!("{} ({}) : {}", path, name, e.message())),
    }
}
