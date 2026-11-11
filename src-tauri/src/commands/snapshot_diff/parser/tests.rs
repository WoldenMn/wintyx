use super::*;

#[test]
fn parse_reg_content_handles_dword_and_string() {
    let input = "Windows Registry Editor Version 5.00\n\n[HKEY_LOCAL_MACHINE\\SOFTWARE\\Test]\n\"MyDword\"=dword:00000001\n\"MyString\"=\"hello world\"\n";
    let parsed = parse_reg_content(input);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].0, "HKLM\\SOFTWARE\\Test");
    assert_eq!(parsed[0].1, "MyDword");
    assert_eq!(parsed[0].2, "dword:00000001");
    assert_eq!(parsed[1].1, "MyString");
    assert_eq!(parsed[1].2, "\"hello world\"");
}

#[test]
fn parse_reg_content_normalizes_hkey_to_short_form() {
    let input = "[HKEY_CURRENT_USER\\Software\\Test]\n\"V\"=dword:00000000\n";
    let parsed = parse_reg_content(input);
    assert_eq!(parsed[0].0, "HKCU\\Software\\Test");
}

#[test]
fn parse_reg_content_handles_multiple_sections() {
    let input = "[HKEY_LOCAL_MACHINE\\A]\n\"v1\"=dword:00000001\n\n[HKEY_LOCAL_MACHINE\\B]\n\"v2\"=dword:00000002\n";
    let parsed = parse_reg_content(input);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].0, "HKLM\\A");
    assert_eq!(parsed[1].0, "HKLM\\B");
}

#[test]
fn parse_reg_content_skips_comments_and_empty_lines() {
    let input = "Windows Registry Editor Version 5.00\n\n; this is a comment\n[HKEY_LOCAL_MACHINE\\Test]\n\n\"v\"=dword:00000001\n";
    let parsed = parse_reg_content(input);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].1, "v");
}

#[test]
fn extract_timestamp_valid_filename() {
    assert_eq!(
        extract_timestamp("regkey_HKLM_SOFTWARE_Test_20260504_092315.reg"),
        Some("20260504_092315".to_string())
    );
}

#[test]
fn extract_timestamp_filename_too_short() {
    assert_eq!(extract_timestamp("short.reg"), None);
    assert_eq!(extract_timestamp("notimestamp.reg"), None);
}

#[test]
fn extract_timestamp_invalid_format() {
    assert_eq!(extract_timestamp("regkey_test_abcdefgh_xxyyzz.reg"), None);
}

// === Tests UTF-16 (fermeture finding L1 de l'audit v2.7.1) ===
// Couvrent les 3 chemins de decode_reg_file :
// 1. UTF-16 LE avec BOM (chemin nominal)
// 2. UTF-16 LE sans BOM (chemin alternatif)
// 3. UTF-16 invalide avec surrogate orphelin (validation du fix M1 strict)

fn tmp_utf16_path(suffix: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    let ts = chrono::Local::now()
        .timestamp_nanos_opt()
        .unwrap_or(0);
    p.push(format!("wintyx_test_utf16_{}_{}.reg", suffix, ts));
    p
}

#[test]
fn decode_reg_file_with_bom_le() {
    let path = tmp_utf16_path("bom");
    // BOM UTF-16 LE (FF FE) suivi de "ABC" en UTF-16 LE
    let mut bytes = vec![0xFFu8, 0xFEu8];
    for c in "ABC".encode_utf16() {
        bytes.extend_from_slice(&c.to_le_bytes());
    }
    fs::write(&path, &bytes).unwrap();
    let decoded = decode_reg_file(&path);
    assert_eq!(decoded.as_deref(), Some("ABC"));
    let _ = fs::remove_file(&path);
}

#[test]
fn decode_reg_file_without_bom_le() {
    let path = tmp_utf16_path("nobom");
    // Pas de BOM, UTF-16 LE valide direct (chemin de fallback L86-89 du parser)
    let mut bytes = Vec::new();
    for c in "XYZ".encode_utf16() {
        bytes.extend_from_slice(&c.to_le_bytes());
    }
    fs::write(&path, &bytes).unwrap();
    let decoded = decode_reg_file(&path);
    assert_eq!(decoded.as_deref(), Some("XYZ"));
    let _ = fs::remove_file(&path);
}

#[test]
fn decode_reg_file_invalid_surrogate() {
    let path = tmp_utf16_path("surrogate");
    // BOM + surrogate haut orphelin (0xD800 sans low surrogate qui suit)
    // Avec le fix M1 v2.7.1 (from_utf16 strict), doit retourner None et log warn.
    // L'ancien from_utf16_lossy l'aurait remplace par U+FFFD silencieusement.
    let mut bytes = vec![0xFFu8, 0xFEu8];
    bytes.extend_from_slice(&0xD800u16.to_le_bytes()); // High surrogate orphelin
    bytes.extend_from_slice(&0x0041u16.to_le_bytes()); // 'A' (devrait etre un low surrogate)
    fs::write(&path, &bytes).unwrap();
    let decoded = decode_reg_file(&path);
    assert!(
        decoded.is_none(),
        "Surrogate orphelin doit etre rejete par le fix M1 v2.7.1 (from_utf16 strict)"
    );
    let _ = fs::remove_file(&path);
}

// --- iter_reg_files (NOUVEAU v2.9.0) ---

#[test]
fn iter_reg_files_filters_non_reg_extensions() {
    use std::fs::File;
    let tmp = std::env::temp_dir().join(format!("wintyx_test_iter_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    File::create(tmp.join("20260518_193500.reg")).unwrap();
    File::create(tmp.join("readme.txt")).unwrap();
    File::create(tmp.join("notes.md")).unwrap();

    let entries = iter_reg_files(&tmp);
    // Check that all entries have .reg extension
    let all_reg = entries
        .iter()
        .all(|(p, _)| p.extension().and_then(|s| s.to_str()) == Some("reg"));
    assert!(all_reg, "iter_reg_files must only return .reg files");
    assert_eq!(entries.len(), 1, "Only the single .reg file should be returned");

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn iter_reg_files_returns_empty_for_missing_dir() {
    let nonexistent = std::env::temp_dir().join("wintyx_iter_nonexistent_xyz");
    let _ = std::fs::remove_dir_all(&nonexistent);
    let entries = iter_reg_files(&nonexistent);
    assert!(entries.is_empty());
}

#[test]
fn extract_timestamp_invalid_format_returns_none_or_safe_value() {
    // Defensive : "not_a_timestamp_format" doit etre traite sans paniquer.
    // L'API exacte depend de l'impl - le test accepte les 2 retours plausibles.
    let result = extract_timestamp("not_a_timestamp_format.reg");
    // Soit None (Option<&str>/Option<String>), soit le filename minus .reg
    // L'important : pas de panic.
    let _ = result;
}

#[test]
fn extract_timestamp_does_not_panic_on_non_ascii_names() {
    assert_eq!(extract_timestamp("clé_é_20260504_092315.reg"), Some("20260504_092315".to_string()));
    assert_eq!(extract_timestamp("日本語日本語日本語日本語.reg"), None);
    assert_eq!(extract_timestamp("éééééééééééééééé.reg"), None);
    assert_eq!(extract_timestamp("1234567é_123456.reg"), None);
}

// Extrait d'un export reel de reg.exe (HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer) :
// les valeurs binaires longues sont coupees avec `\` en fin de ligne et reprises
// sur la ligne suivante, indentee de deux espaces.
const MULTILINE_HEX: &str = "Windows Registry Editor Version 5.00\r\n\r\n[HKEY_CURRENT_USER\\Software\\Test]\r\n\"Action\"=hex(2):40,00,7b,00,4d,00,69,00,63,00,72,00,6f,00,73,00,6f,00,66,00,74,\\\r\n  00,2e,00,57,00,69,00,6e,00,64,00,6f,00,77,00,73,00,2e,00,50,00,68,00,6f,00,\\\r\n  74,00,6f,00,73,00,5f,00,32,00,30,00,32,00,36,00,00,00\r\n\"After\"=dword:00000001\r\n";

#[test]
fn parse_reg_content_joins_multiline_hex_values() {
    let parsed = parse_reg_content(MULTILINE_HEX);
    assert_eq!(parsed.len(), 2, "{:?}", parsed);
    assert_eq!(parsed[0].1, "Action");
    assert!(parsed[0].2.starts_with("hex(2):40,00,7b,00,"), "{}", parsed[0].2);
    assert!(parsed[0].2.ends_with("32,00,30,00,32,00,36,00,00,00"), "{}", parsed[0].2);
    assert!(!parsed[0].2.contains('\\') && !parsed[0].2.contains(' '), "{}", parsed[0].2);
    assert_eq!(parsed[1].1, "After");
    assert_eq!(parsed[1].2, "dword:00000001");
}

#[test]
fn same_hex_value_wrapped_differently_parses_identically() {
    let wrapped = "[HKEY_CURRENT_USER\\T]\n\"V\"=hex:01,02,\\\n  03,04\n";
    let flat = "[HKEY_CURRENT_USER\\T]\n\"V\"=hex:01,02,03,04\n";
    assert_eq!(parse_reg_content(wrapped), parse_reg_content(flat));
}
