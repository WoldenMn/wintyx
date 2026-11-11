use super::*;
use std::env::temp_dir;
use std::fs;

fn temp_path(name: &str) -> PathBuf {
    let mut p = temp_dir();
    p.push(format!(
        "wintyx_test_{}_{}",
        chrono::Local::now().timestamp_millis(),
        name
    ));
    p
}

fn remove_quarantined(path: &Path) -> usize {
    let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
    let mut removed = 0;
    for e in fs::read_dir(path.parent().unwrap()).unwrap().filter_map(|e| e.ok()) {
        if e.file_name().to_string_lossy().starts_with(&format!("{}.corrompu-", stem)) {
            let _ = fs::remove_file(e.path());
            removed += 1;
        }
    }
    removed
}

fn dummy_custom(id: &str) -> Profile {
    Profile {
        id: id.to_string(),
        name: id.to_string(),
        description: "".to_string(),
        data_protection_ids: vec!["dp_0".to_string()],
        hardening_ids: vec![],
        performance_ids: vec![],
        is_custom: true,
        services_ids: vec![],
        bloatware_ids: vec![],
        startup_ids: vec![],
    }
}

#[test]
fn save_and_load_roundtrip() {
    let path = temp_path("roundtrip.json");
    let profiles = vec![dummy_custom("custom_1")];
    save_custom_profiles_to(&path, &profiles).unwrap();
    let loaded = load_custom_profiles_from(&path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, "custom_1");
    assert!(loaded[0].is_custom);
    let _ = fs::remove_file(&path);
}

#[test]
fn save_overwrites_existing_file_atomically() {
    // Non-regression : ancien pattern remove_file + rename introduisait
    // une fenetre TOCTOU. Le nouveau pattern utilise fs::rename direct
    // (atomique sur NTFS et ecrase la cible). Verifie qu'un re-save
    // ecrase bien le fichier existant et que le tempfile ne reste pas.
    let path = temp_path("overwrite.json");
    let v1 = vec![dummy_custom("custom_v1")];
    let v2 = vec![dummy_custom("custom_v2")];
    save_custom_profiles_to(&path, &v1).unwrap();
    save_custom_profiles_to(&path, &v2).unwrap();
    let loaded = load_custom_profiles_from(&path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, "custom_v2");
    let prefix = path.file_name().unwrap().to_string_lossy().into_owned();
    let orphans = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.starts_with(&prefix) && name.ends_with(".tmp")
        })
        .count();
    assert_eq!(orphans, 0, "tempfile orphelin");
    let _ = fs::remove_file(&path);
}

#[test]
fn concurrent_read_modify_write_loses_no_update() {
    let path = temp_path("concurrent.json");
    let threads: Vec<_> = (0..12)
        .map(|i| {
            let path = path.clone();
            std::thread::spawn(move || {
                update_custom_profiles_at(&path, |profiles| {
                    profiles.push(dummy_custom(&format!("custom_c{}", i)));
                    Ok(())
                })
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap().unwrap();
    }
    assert_eq!(load_custom_profiles_from(&path).unwrap().len(), 12);
    let _ = fs::remove_file(&path);
}

#[test]
fn load_returns_empty_when_missing() {
    let path = temp_path("nonexistent.json");
    let loaded = load_custom_profiles_from(&path).unwrap();
    assert!(loaded.is_empty());
}

#[test]
fn load_returns_empty_when_invalid_json() {
    let path = temp_path("invalid.json");
    fs::write(&path, "not valid json {").unwrap();
    let loaded = load_custom_profiles_from(&path).unwrap();
    assert!(loaded.is_empty());
    remove_quarantined(&path);
}

#[test]
fn load_truncates_above_max_profiles() {
    // Non-regression : un fichier JSON edite a la main ne doit pas pouvoir
    // depasser MAX_CUSTOM_PROFILES au chargement.
    let path = temp_path("overflow.json");
    let mut profiles: Vec<Profile> = Vec::new();
    for i in 0..(MAX_CUSTOM_PROFILES + 5) {
        profiles.push(dummy_custom(&format!("custom_overflow_{}", i)));
    }
    let json = serde_json::to_string_pretty(&profiles).unwrap();
    fs::write(&path, json).unwrap();
    let loaded = load_custom_profiles_from(&path).unwrap();
    assert_eq!(loaded.len(), MAX_CUSTOM_PROFILES);
    let _ = fs::remove_file(&path);
}

#[test]
fn validate_profile_fields_accepts_valid() {
    assert!(validate_profile_fields("Mon profil", "Description", 5).is_ok());
}

#[test]
fn validate_profile_fields_rejects_empty_name() {
    assert!(validate_profile_fields("", "Desc", 5).is_err());
    assert!(validate_profile_fields("   ", "Desc", 5).is_err());
}

#[test]
fn validate_profile_fields_rejects_long_name() {
    let long = "a".repeat(MAX_NAME_LEN + 1);
    assert!(validate_profile_fields(&long, "Desc", 5).is_err());
}

#[test]
fn validate_profile_fields_rejects_long_description() {
    let long = "a".repeat(MAX_DESCRIPTION_LEN + 1);
    assert!(validate_profile_fields("Name", &long, 5).is_err());
}

#[test]
fn validate_profile_fields_accepts_accents_and_apostrophes() {
    assert!(validate_profile_fields("Profil d\u{2019}\u{e9}t\u{e9}", "Desc", 1).is_ok());
    assert!(validate_profile_fields("L'\u{e9}t\u{e9} de Jos\u{e9}", "Desc", 1).is_ok());
    assert!(validate_profile_fields("x\u{2019}; calc #", "Desc", 1).is_ok());
}

#[test]
fn validate_profile_fields_rejects_control_chars() {
    assert!(validate_profile_fields("a\nb", "Desc", 1).is_err());
    assert!(validate_profile_fields("a\tb", "Desc", 1).is_err());
    assert!(validate_profile_fields("a\u{0}b", "Desc", 1).is_err());
    assert!(validate_profile_fields("Name", "de\rsc", 1).is_err());
}

#[test]
fn load_skips_profiles_with_control_chars_in_name() {
    let path = temp_path("bad_name.json");
    let mut bad = dummy_custom("custom_bad");
    bad.name = "a\nb".to_string();
    save_custom_profiles_to(&path, &[bad, dummy_custom("custom_ok")]).unwrap();
    let loaded = load_custom_profiles_from(&path).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, "custom_ok");
    let _ = fs::remove_file(&path);
}

#[test]
fn validate_profile_fields_rejects_zero_items() {
    assert!(validate_profile_fields("Name", "Desc", 0).is_err());
}

#[test]
fn corrupted_file_is_quarantined_before_any_rewrite() {
    let path = temp_path("quarantine.json");
    fs::write(&path, "not valid json {").unwrap();
    let _ = load_custom_profiles_from(&path).unwrap();
    assert!(!path.exists(), "le fichier illisible doit etre renomme");
    let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
    let moved: Vec<_> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(&format!("{}.corrompu-", stem)))
        .collect();
    assert_eq!(moved.len(), 1);
    for e in moved {
        let _ = fs::remove_file(e.path());
    }
}
