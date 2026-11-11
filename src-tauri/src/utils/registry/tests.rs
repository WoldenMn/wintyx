use super::backup::backup_file_name;
use super::read::is_not_found_code;
use super::*;

#[test]
fn sanitize_name_valid() {
    assert_eq!(sanitize_name("DiagTrack").unwrap(), "DiagTrack");
    assert_eq!(sanitize_name("service-name_1.0").unwrap(), "service-name_1.0");
    assert_eq!(sanitize_name("WSearch").unwrap(), "WSearch");
}

#[test]
fn sanitize_name_trims() {
    assert_eq!(sanitize_name("  DiagTrack  ").unwrap(), "DiagTrack");
}

#[test]
fn sanitize_name_empty() {
    assert!(sanitize_name("").is_err());
    assert!(sanitize_name("   ").is_err());
}

#[test]
fn sanitize_name_rejects_special_chars() {
    assert!(sanitize_name("name&inject").is_err());
    assert!(sanitize_name("path\\traversal").is_err());
    assert!(sanitize_name("name;drop").is_err());
    assert!(sanitize_name("name|pipe").is_err());
    assert!(sanitize_name("name$(cmd)").is_err());
    assert!(sanitize_name("name`bt`").is_err());
    assert!(sanitize_name("name\"quote").is_err());
}

#[test]
fn sanitize_name_rejects_unicode() {
    // Defense-in-depth contre confusables et bypasses de normalisation
    assert!(sanitize_name("Сервис").is_err()); // Cyrillique lookalike "Service"
    assert!(sanitize_name("服务").is_err()); // Chinois
    assert!(sanitize_name("résumé").is_err()); // accents
}

#[test]
fn sanitize_name_rejects_spaces() {
    // Les espaces pourraient causer des problemes de splitting d'arguments
    // dans certaines invocations shell (sc.exe stop "My Service" vs stop My Service)
    assert!(sanitize_name("Xbox Live Auth").is_err());
    assert!(sanitize_name("name with space").is_err());
}

#[test]
fn sanitize_name_rejects_too_long() {
    let long_name = "a".repeat(257);
    assert!(sanitize_name(&long_name).is_err());
}

#[test]
fn allowed_key_path_accepts_4_prefixes() {
    assert!(is_allowed_key_path("HKLM\\SOFTWARE\\Foo"));
    assert!(is_allowed_key_path("HKLM\\SYSTEM\\CurrentControlSet"));
    assert!(is_allowed_key_path("HKCU\\SOFTWARE\\Bar"));
    assert!(is_allowed_key_path("HKCU\\Control Panel\\Mouse"));
}

#[test]
fn allowed_key_path_rejects_other_prefixes() {
    assert!(!is_allowed_key_path("HKCR\\Foo"));
    assert!(!is_allowed_key_path("HKLM\\Other"));
    assert!(!is_allowed_key_path("HKEY_LOCAL_MACHINE\\SOFTWARE"));
    assert!(!is_allowed_key_path(""));
    assert!(!is_allowed_key_path("/etc/passwd"));
}

#[test]
fn allowed_key_path_case_insensitive() {
    // Le registre Windows est case-insensitive : "hklm\\software\\..." doit
    // etre traite comme "HKLM\\SOFTWARE\\...". Sans normalisation, un attaquant
    // peut bypasser l'allowlist via une variation de casse.
    assert!(is_allowed_key_path("hklm\\software\\Foo"));
    assert!(is_allowed_key_path("HkLm\\SoFtWaRe\\Bar"));
    assert!(is_allowed_key_path("hkcu\\control panel\\Mouse"));
    // Et les chemins non autorises restent rejetes meme en lowercase
    assert!(!is_allowed_key_path("hkcr\\foo"));
    assert!(!is_allowed_key_path("hklm\\other"));
}

#[test]
fn is_hklm_path_case_insensitive() {
    assert!(is_hklm_path("HKLM\\SOFTWARE\\Foo"));
    assert!(is_hklm_path("hklm\\software\\Foo"));
    assert!(is_hklm_path("HkLm\\SoFtWaRe"));
    assert!(!is_hklm_path("HKCU\\SOFTWARE\\Foo"));
    assert!(!is_hklm_path("hkcu\\SOFTWARE"));
    assert!(!is_hklm_path(""));
    assert!(!is_hklm_path("HKL"));
}

#[test]
fn sanitize_value_name_accepts_empty_for_default_value() {
    assert_eq!(sanitize_value_name("").unwrap(), "");
}

#[test]
fn sanitize_value_name_accepts_alphanumeric() {
    assert_eq!(sanitize_value_name("DisableSmartScreen").unwrap(), "DisableSmartScreen");
    assert_eq!(sanitize_value_name("1024MB").unwrap(), "1024MB");
}

#[test]
fn sanitize_value_name_accepts_spaces() {
    // Les noms de valeurs registre comme "Display Name" sont légitimes
    assert_eq!(sanitize_value_name("Display Name").unwrap(), "Display Name");
}

#[test]
fn sanitize_value_name_accepts_dash_underscore_dot() {
    assert_eq!(sanitize_value_name("my-value_1.0").unwrap(), "my-value_1.0");
}

#[test]
fn sanitize_value_name_rejects_special_chars() {
    assert!(sanitize_value_name("name&inject").is_err());
    assert!(sanitize_value_name("name|pipe").is_err());
    assert!(sanitize_value_name("name;drop").is_err());
    assert!(sanitize_value_name("name\\path").is_err());
    assert!(sanitize_value_name("name/path").is_err());
    assert!(sanitize_value_name("name\"quote").is_err());
    assert!(sanitize_value_name("name`bt`").is_err());
}

#[test]
fn sanitize_value_name_rejects_unicode() {
    assert!(sanitize_value_name("résumé").is_err());
    assert!(sanitize_value_name("Сервис").is_err());
}

#[test]
fn sanitize_value_name_rejects_too_long() {
    let long = "a".repeat(257);
    assert!(sanitize_value_name(&long).is_err());
}

#[test]
fn sanitize_key_path_accepts_backslash_and_spaces() {
    assert!(sanitize_key_path("HKLM\\SYSTEM\\CurrentControlSet\\Control\\Terminal Server").is_ok());
    assert!(sanitize_key_path("HKCU\\SOFTWARE\\Microsoft\\Windows").is_ok());
}

#[test]
fn sanitize_key_path_rejects_metachars_and_empty() {
    assert!(sanitize_key_path("").is_err());
    assert!(sanitize_key_path("   ").is_err());
    for bad in ["HKLM\\a&b", "HKLM\\a|b", "HKLM\\a\"b", "HKLM\\a`b", "HKLM\\a;b", "HKLM\\r\u{00e9}s"] {
        assert!(sanitize_key_path(bad).is_err(), "doit rejeter: {}", bad);
    }
}

#[test]
fn parse_root_invalid_prefix_or_no_separator_returns_none() {
    // tests-be-07 : parse_root_and_subpath (prive) teste via get_dword, qui
    // renvoie None sans lecture registre quand le prefixe est inconnu ou qu'il
    // n'y a pas de backslash.
    assert_eq!(get_dword("FOO\\Bar\\Baz", "X"), None);
    assert_eq!(get_dword("HKLM", "X"), None);
}

pub(super) struct TempKey(pub(super) String);

impl TempKey {
    pub(super) fn new(tag: &str) -> Self {
        let path = format!(r"HKCU\Software\Wintyx-test-{}-{}", std::process::id(), tag);
        ensure_key(&path).unwrap();
        TempKey(path)
    }
}

impl Drop for TempKey {
    fn drop(&mut self) {
        let sub = self.0.strip_prefix(r"HKCU\").unwrap();
        let _ = CURRENT_USER.remove_tree(sub);
    }
}

#[test]
fn delete_value_removes_existing_value_on_real_key() {
    let key = TempKey::new("delete");
    set_string(&key.0, "Probe", "valeur").unwrap();
    set_dword(&key.0, "ProbeDword", 7).unwrap();
    assert_eq!(get_string(&key.0, "Probe").as_deref(), Some("valeur"));

    delete_value(&key.0, "Probe").unwrap();
    assert_eq!(get_string(&key.0, "Probe"), None);
    assert_eq!(get_dword(&key.0, "ProbeDword"), Some(7));
}

#[test]
fn delete_value_is_ok_when_value_or_key_is_absent() {
    let key = TempKey::new("absent");
    assert!(delete_value(&key.0, "NeverThere").is_ok());
    let missing = format!(r"{}\Child", key.0);
    assert!(delete_value(&missing, "NeverThere").is_ok());
}

#[test]
fn delete_value_rejects_invalid_path() {
    assert!(delete_value("NOPE", "x").is_err());
}

#[test]
fn backup_file_name_differs_between_keys_with_same_value_name() {
    let now = chrono::Local::now();
    let a = backup_file_name(r"HKLM\SOFTWARE\A", "Start", now);
    let b = backup_file_name(r"HKLM\SOFTWARE\B", "Start", now);
    assert_ne!(a, b);
}

#[test]
fn backup_file_name_differs_by_millisecond() {
    let t0 = chrono::Local::now();
    let t1 = t0 + chrono::Duration::milliseconds(1);
    assert_ne!(
        backup_file_name(r"HKLM\SOFTWARE\A", "Start", t0),
        backup_file_name(r"HKLM\SOFTWARE\A", "Start", t1)
    );
}

#[test]
fn backup_file_name_is_stable_and_safe() {
    let now = chrono::Local::now();
    let a = backup_file_name(r"HKLM\SOFTWARE\A", "Start", now);
    assert_eq!(a, backup_file_name(r"HKLM\SOFTWARE\A", "Start", now));
    assert!(a.ends_with(".txt"));
    assert!(!a.contains(['\\', '/']));
}

#[test]
fn move_string_value_copies_then_removes_source() {
    let src = TempKey::new("mv-src");
    let dst = format!("{}-dst", src.0);
    set_string(&src.0, "App", r"C:\app.exe").unwrap();

    move_string_value(&src.0, &dst, "App").unwrap();

    assert_eq!(get_string(&src.0, "App"), None);
    assert_eq!(get_string(&dst, "App").as_deref(), Some(r"C:\app.exe"));
    let _ = CURRENT_USER.remove_tree(dst.strip_prefix(r"HKCU\").unwrap());
}

#[test]
fn move_string_value_keeps_the_expand_sz_type() {
    let src = TempKey::new("mv-expand");
    let dst = format!("{}-dst", src.0);
    let sub = src.0.strip_prefix(r"HKCU\").unwrap();
    CURRENT_USER
        .create(sub)
        .unwrap()
        .set_expand_string("App", r"%ProgramFiles%\app.exe")
        .unwrap();

    move_string_value(&src.0, &dst, "App").unwrap();

    let key = CURRENT_USER.open(dst.strip_prefix(r"HKCU\").unwrap()).unwrap();
    assert_eq!(key.get_type("App").unwrap(), Type::ExpandString);
    assert_eq!(key.get_string("App").unwrap(), r"%ProgramFiles%\app.exe");
    let _ = CURRENT_USER.remove_tree(dst.strip_prefix(r"HKCU\").unwrap());
}

#[test]
fn move_string_value_keeps_source_when_destination_write_fails() {
    let src = TempKey::new("mv-keep");
    set_string(&src.0, "App", r"C:\app.exe").unwrap();

    assert!(move_string_value(&src.0, r"NOPE\Dst", "App").is_err());

    assert_eq!(get_string(&src.0, "App").as_deref(), Some(r"C:\app.exe"));
}

#[test]
fn move_string_value_fails_when_source_absent() {
    let src = TempKey::new("mv-none");
    let dst = format!("{}-dst", src.0);
    let err = move_string_value(&src.0, &dst, "Ghost").unwrap_err();
    assert!(err.contains("Ghost"), "{err}");
    assert!(!key_exists(&dst, "Ghost"));
}

#[test]
fn backup_before_write_keeps_both_originals_for_same_value_name() {
    let a = TempKey::new("bk-a");
    let b = TempKey::new("bk-b");
    set_string(&a.0, "Same", "old-a").unwrap();
    set_string(&b.0, "Same", "old-b").unwrap();
    set_string(&a.0, "Same", "new-a").unwrap();
    set_string(&b.0, "Same", "new-b").unwrap();

    let dir = crate::utils::paths::wintyx_local_dir().join("RegistryBackups");
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        if content.contains(&a.0) || content.contains(&b.0) {
            found.push(content);
        }
    }
    assert!(found.iter().any(|c| c.contains("OldValue: old-a")), "{:?}", found);
    assert!(found.iter().any(|c| c.contains("OldValue: old-b")), "{:?}", found);
}

#[test]
fn not_found_codes_are_absent_values_not_failures() {
    assert!(is_not_found_code(0x8007_0002u32 as i32));
    assert!(is_not_found_code(0x8007_0003u32 as i32));
    assert!(!is_not_found_code(0x8007_0005u32 as i32));
}

#[test]
fn read_dword_distinguishes_absent_from_present() {
    assert_eq!(
        read_dword(r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion", "WintyxAbsentValue").unwrap(),
        None
    );
    assert_eq!(read_dword(r"HKLM\SOFTWARE\WintyxAbsentKey", "X").unwrap(), None);
    assert!(read_dword(r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion", "CurrentMajorVersionNumber").unwrap().is_some());
}

#[test]
fn read_dword_rejects_an_invalid_root() {
    assert!(read_dword(r"NOPE\x", "y").is_err());
}
