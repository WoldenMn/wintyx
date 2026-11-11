use super::*;
use crate::utils::temp_file::{create_new_read_write, unpredictable_temp_path};
use std::io::Write;

const HEADER: &str = "Windows Registry Editor Version 5.00";
const DATA_COLLECTION: &str = r"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\DataCollection";
const RUN_KEY: &str = r"HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

fn file(lines: &[&str]) -> String {
    let mut text = String::from(HEADER);
    text.push_str("\r\n");
    for line in lines {
        text.push_str(line);
        text.push_str("\r\n");
    }
    text
}

fn utf16_le(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
    bytes
}

fn verdict(lines: &[&str]) -> Result<(), String> {
    verify(&file(lines))
}

#[test]
fn a_plain_export_of_an_exported_key_is_accepted() {
    let section = format!("[{DATA_COLLECTION}]");
    let ok = verdict(&[
        &section,
        r#""AllowTelemetry"=dword:00000001"#,
        r#""Name"="C:\\Program Files\\x.exe""#,
        r#"@="default""#,
        r#""Blob"=hex:01,02,03,\"#,
        "  04,05",
    ]);
    assert_eq!(ok, Ok(()));
}

#[test]
fn a_subkey_of_an_exported_key_is_accepted() {
    let section = format!(r"[{DATA_COLLECTION}\Child]");
    assert_eq!(verdict(&[&section]), Ok(()));
    let run = format!("[{RUN_KEY}]");
    assert_eq!(verdict(&[&run]), Ok(()));
}

#[test]
fn key_comparison_ignores_case_and_accepts_short_hive_names() {
    let upper = format!("[{}]", DATA_COLLECTION.to_ascii_uppercase());
    let short = r"[HKLM\software\policies\microsoft\windows\system]";
    assert_eq!(verdict(&[&upper, short]), Ok(()));
}

#[test]
fn a_key_that_wintyx_never_exports_is_refused() {
    for key in [
        r"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\Evil",
        r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon",
        r"HKEY_LOCAL_MACHINE\SOFTWARE",
        r"HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce",
        r"HKEY_CLASSES_ROOT\exefile\shell\open\command",
        r"HKEY_USERS\.DEFAULT\Software",
    ] {
        let err = verdict(&[&format!("[{key}]")]).unwrap_err();
        assert!(err.contains("refusée") || err.contains("hors de") || err.contains("non prise"), "{key}: {err}");
    }
}

#[test]
fn a_sibling_key_sharing_the_prefix_is_refused() {
    let sibling = format!("[{DATA_COLLECTION}X]");
    assert!(verdict(&[&sibling]).is_err());
    let sibling = format!("[{RUN_KEY}Once]");
    assert!(verdict(&[&sibling]).is_err());
}

#[test]
fn a_good_section_followed_by_a_bad_one_is_refused() {
    let good = format!("[{DATA_COLLECTION}]");
    assert!(verdict(&[&good, r#""a"=dword:00000001"#, r"[HKEY_LOCAL_MACHINE\SYSTEM\Setup]"]).is_err());
}

#[test]
fn key_deletions_are_refused_even_inside_an_exported_key() {
    for section in [
        format!("[-{DATA_COLLECTION}]"),
        format!("[- {DATA_COLLECTION}]"),
        format!("[-{DATA_COLLECTION}\\Child]"),
    ] {
        let err = verdict(&[&section]).unwrap_err();
        assert!(err.contains("Suppression de clé"), "{section}: {err}");
    }
}

#[test]
fn value_deletions_are_refused() {
    let section = format!("[{DATA_COLLECTION}]");
    for line in [
        r#""AllowTelemetry"=-"#,
        r#""AllowTelemetry" = -"#,
        r#"@=-"#,
        r#""a=b"=-"#,
        r#""quote\"inside"=-"#,
        r#""x"=-;trailing"#,
    ] {
        let err = verdict(&[&section, line]).unwrap_err();
        assert!(err.contains("Suppression de valeur"), "{line}: {err}");
    }
}

#[test]
fn a_value_containing_an_equal_sign_and_a_dash_is_not_a_deletion() {
    let section = format!("[{DATA_COLLECTION}]");
    assert_eq!(verdict(&[&section, r#""a"="x=-y""#]), Ok(()));
    assert_eq!(verdict(&[&section, r#""a-b"=dword:00000000"#]), Ok(()));
}

#[test]
fn lone_carriage_returns_cannot_hide_a_line() {
    let good = format!("[{DATA_COLLECTION}]");
    let sneaky = format!("{HEADER}\r\n{good}\r[HKEY_LOCAL_MACHINE\\SYSTEM\\Setup]\r\n");
    assert!(verify(&sneaky).is_err());
    let deletion = format!("{HEADER}\n{good}\r\"a\"=-\n");
    assert!(verify(&deletion).is_err());
}

#[test]
fn a_hex_continuation_cannot_smuggle_a_header() {
    let good = format!("[{DATA_COLLECTION}]");
    assert!(verdict(&[&good, r#""a"=hex:01,\"#, r"[HKEY_LOCAL_MACHINE\SYSTEM\Setup]"]).is_err());
}

#[test]
fn malformed_headers_and_paths_are_refused() {
    for line in [
        format!("[{DATA_COLLECTION}"),
        format!("[{DATA_COLLECTION}] ; note"),
        format!(r"[{DATA_COLLECTION}\\Child]"),
        format!(r"[{DATA_COLLECTION}\]"),
        "[]".to_string(),
        "[HKEY_LOCAL_MACHINE]".to_string(),
    ] {
        assert!(verdict(&[&line]).is_err(), "{line}");
    }
}

#[test]
fn the_file_must_start_with_a_registry_script_header() {
    assert!(verify("").is_err());
    assert!(verify("\r\n\r\n").is_err());
    assert!(verify(&format!("[{DATA_COLLECTION}]\r\n")).is_err());
    assert!(verify("not a registry file\r\n").is_err());
    assert_eq!(verify("REGEDIT4\r\n"), Ok(()));
}

#[test]
fn a_nul_character_is_refused() {
    assert!(verify(&format!("{HEADER}\r\n\0\r\n")).is_err());
}

#[test]
fn utf16_le_with_bom_is_decoded_and_ascii_is_accepted() {
    let text = file(&[&format!("[{DATA_COLLECTION}]")]);
    assert_eq!(decode(&utf16_le(&text)).unwrap(), text);
    assert_eq!(decode(text.as_bytes()).unwrap(), text);
}

#[test]
fn other_encodings_are_refused() {
    let text = file(&[]);
    let mut utf16_be = vec![0xFE, 0xFF];
    utf16_be.extend(text.encode_utf16().flat_map(u16::to_be_bytes));
    assert!(decode(&utf16_be).is_err());

    let without_bom: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
    assert!(decode(&without_bom).is_err());

    let mut utf8_with_bom = vec![0xEF, 0xBB, 0xBF];
    utf8_with_bom.extend(text.as_bytes());
    assert!(decode(&utf8_with_bom).is_err());

    assert!(decode("Windows Registry Editor Version 5.00\r\n\"é\"=\"é\"".as_bytes()).is_err());
}

#[test]
fn truncated_or_broken_utf16_is_refused() {
    let mut odd = utf16_le(&file(&[]));
    odd.pop();
    assert!(decode(&odd).is_err());
    assert!(decode(&[0xFF, 0xFE, 0x00, 0xD8]).is_err());
}

#[test]
fn utf16_deletion_hidden_after_a_valid_section_is_refused() {
    let text = file(&[&format!("[{DATA_COLLECTION}]"), r#""a"=-"#]);
    let decoded = decode(&utf16_le(&text)).unwrap();
    assert!(verify(&decoded).is_err());
}

fn temp_reg(bytes: &[u8]) -> std::path::PathBuf {
    let path = unpredictable_temp_path("wintyx-regguard-test", "reg");
    let mut f = create_new_read_write(&path).unwrap();
    f.write_all(bytes).unwrap();
    path
}

#[test]
fn open_verified_accepts_a_sound_file_and_locks_it_against_writes_and_deletion() {
    let path = temp_reg(&utf16_le(&file(&[&format!("[{DATA_COLLECTION}]")])));
    let guard = open_verified(&path).expect("fichier sain");
    assert!(OpenOptions::new().write(true).open(&path).is_err(), "ecriture possible pendant le verrou");
    assert!(std::fs::remove_file(&path).is_err(), "suppression possible pendant le verrou");
    assert!(File::open(&path).is_ok(), "lecture (reg.exe) bloquee");
    drop(guard);
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn open_verified_refuses_a_hostile_file_and_releases_it() {
    let path = temp_reg(&utf16_le(&file(&[r"[HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\Evil]"])));
    let err = open_verified(&path).err().expect("refus attendu");
    assert!(matches!(err, CommandError::InvalidInput(_)), "{err:?}");
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn open_verified_refuses_an_oversized_file() {
    let mut bytes = utf16_le(&file(&[]));
    bytes.resize(MAX_BACKUP_BYTES as usize + 2, b' ');
    let path = temp_reg(&bytes);
    let err = open_verified(&path).err().expect("refus attendu");
    assert!(err.to_string().contains("volumineuse"), "{err}");
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn open_verified_reports_a_missing_file() {
    let path = unpredictable_temp_path("wintyx-regguard-test", "reg");
    assert!(open_verified(&path).is_err());
}

#[cfg(windows)]
#[test]
fn reg_exe_still_imports_a_file_held_under_the_read_lock() {
    let pid = std::process::id();
    let key = format!(r"HKCU\Software\Wintyx-test-{pid}-regguard");
    let text = file(&[
        &format!(r"[HKEY_CURRENT_USER\Software\Wintyx-test-{pid}-regguard]"),
        r#""V"=dword:00000007"#,
    ]);
    let path = temp_reg(&utf16_le(&text));
    let lock = lock_read_shared(&path).unwrap();
    let out = crate::utils::process::run("reg.exe", &format!("import \"{}\"", path.display()), 10_000);
    drop(lock);
    let imported = crate::utils::registry::get_dword(&key, "V");
    let _ = windows_registry::CURRENT_USER.remove_tree(key.strip_prefix(r"HKCU\").unwrap());
    std::fs::remove_file(&path).unwrap();
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(imported, Some(7));
}

#[cfg(windows)]
#[test]
fn a_real_reg_exe_export_of_an_exported_key_passes_the_guard() {
    let key = r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    let exported = super::super::registry_export::export_registry_key(key, "20261023_120000").unwrap();
    let bytes = std::fs::read(&exported).unwrap();
    assert_eq!(&bytes[..2], &[0xFF, 0xFE], "reg.exe doit ecrire de l'UTF-16 LE avec BOM");
    let text = decode(&bytes).unwrap();
    assert_eq!(verify(&text), Ok(()), "{text}");
    open_verified(std::path::Path::new(&exported)).unwrap();
    std::fs::remove_file(&exported).unwrap();
}
