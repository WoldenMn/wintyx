use std::collections::hash_map::RandomState;
use std::fs::{File, OpenOptions};
use std::hash::{BuildHasher, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn random_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let mut token = String::with_capacity(32);
    for lane in 0..2u8 {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u8(lane);
        hasher.write_u32(std::process::id());
        hasher.write_u128(nanos);
        token.push_str(&format!("{:016x}", hasher.finish()));
    }
    token
}

pub fn unpredictable_temp_path(prefix: &str, extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{prefix}-{}.{extension}", random_token()))
}

pub fn create_new_read_write(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn token_is_32_hex_characters() {
        let token = random_token();
        assert_eq!(token.len(), 32, "{token}");
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()), "{token}");
    }

    #[test]
    fn tokens_never_repeat() {
        let tokens: HashSet<String> = (0..2_000).map(|_| random_token()).collect();
        assert_eq!(tokens.len(), 2_000);
    }

    #[test]
    fn temp_path_lives_in_the_temp_dir_with_a_random_name() {
        let a = unpredictable_temp_path("wintyx-secpol", "inf");
        let b = unpredictable_temp_path("wintyx-secpol", "inf");
        assert_ne!(a, b);
        assert_eq!(a.parent(), Some(std::env::temp_dir().as_path()));
        let name = a.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.starts_with("wintyx-secpol-") && name.ends_with(".inf"), "{name}");
        assert_eq!(name.len(), "wintyx-secpol-".len() + 32 + ".inf".len(), "{name}");
    }

    #[test]
    fn create_new_fails_on_an_existing_file_and_keeps_its_content() {
        let path = unpredictable_temp_path("wintyx-temp-file-test", "tmp");
        std::fs::write(&path, b"intact").unwrap();
        let err = create_new_read_write(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&path).unwrap(), b"intact");
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn create_new_creates_a_fresh_file() {
        let path = unpredictable_temp_path("wintyx-temp-file-test", "tmp");
        assert!(!path.exists());
        create_new_read_write(&path).unwrap();
        assert!(path.is_file());
        std::fs::remove_file(&path).unwrap();
    }
}
