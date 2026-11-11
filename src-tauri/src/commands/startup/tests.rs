use super::*;

#[test]
fn dirs_user_startup_uses_appdata_env() {
    let path = dirs_user_startup();
    // Sur un systeme Windows reel, APPDATA est toujours defini.
    // Si absent (CI Linux), on accepte None ; sinon on verifie la structure.
    if let Some(p) = path {
        let s = p.to_string_lossy();
        assert!(
            s.contains("Microsoft") && s.contains("Startup"),
            "Expected user startup path to contain 'Microsoft\\...\\Startup', got: {}",
            s
        );
    }
}

#[test]
fn dirs_common_startup_uses_programdata_env() {
    let path = dirs_common_startup();
    if let Some(p) = path {
        let s = p.to_string_lossy();
        assert!(
            s.contains("Microsoft") && s.contains("Startup"),
            "Expected common startup path to contain 'Microsoft\\...\\Startup', got: {}",
            s
        );
    }
}

#[test]
fn find_disabled_in_dir_matches_lnk_by_stem() {
    // Regression bug-rust-02 : un raccourci desactive "MyApp.lnk.disabled" doit
    // etre retrouve a partir du stem "MyApp" (l'ancien enable cherchait
    // "MyApp.disabled" et ne le trouvait jamais).
    use std::fs::File;
    let dir = std::env::temp_dir().join(format!("wintyx_startup_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create test dir");
    File::create(dir.join("MyApp.lnk.disabled")).expect("create test file");

    assert_eq!(
        find_disabled_in_dir(&dir, "MyApp"),
        Some(dir.join("MyApp.lnk.disabled"))
    );
    assert_eq!(find_disabled_in_dir(&dir, "Autre"), None);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sanitize_startup_name_accepts_spaces() {
    // Regression BUG-01 v2.16.0 : "NVIDIA Share" / "Adobe Updater" etaient
    // rejetes par sanitize_name (espaces interdits) -> impossible de les
    // desactiver/reactiver depuis l'UI.
    assert_eq!(sanitize_startup_name("NVIDIA Share").unwrap(), "NVIDIA Share");
    assert_eq!(sanitize_startup_name("Adobe Updater").unwrap(), "Adobe Updater");
}

#[test]
fn sanitize_startup_name_rejects_dangerous_as_invalid_input() {
    for bad in ["evil\"quote", "a|pipe", "a;drop", "a$(cmd)", "a`bt`", "a\\path", "Сервис"] {
        let err = sanitize_startup_name(bad).unwrap_err();
        assert!(
            matches!(err, CommandError::InvalidInput(_)),
            "doit rejeter '{}' en InvalidInput, obtenu: {:?}",
            bad,
            err
        );
    }
}

#[test]
fn file_stem_or_unnamed_falls_back_without_stem() {
    // Regression v2.12.0 (env-005) : un chemin sans nom exploitable retourne
    // "(sans nom)" au lieu d'une chaine vide (element fantome dans l'UI).
    assert_eq!(file_stem_or_unnamed(std::path::Path::new("..")), "(sans nom)");
    assert_eq!(file_stem_or_unnamed(std::path::Path::new("MyApp.lnk")), "MyApp");
}

#[test]
fn startup_dir_lists_disabled_shortcut_under_its_real_name() {
    let dir = std::env::temp_dir().join(format!("wintyx_startup_list_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::File::create(dir.join("Actif.lnk")).unwrap();
    fs::File::create(dir.join("Eteint.lnk.disabled")).unwrap();

    let mut items = items_in_startup_dir(&dir, "test");
    items.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(items.len(), 2);
    assert_eq!((items[0].name.as_str(), items[0].is_enabled), ("Actif", true));
    assert_eq!((items[1].name.as_str(), items[1].is_enabled), ("Eteint", false));
    assert!(items[1].command.ends_with("Eteint.lnk.disabled"));
    let _ = fs::remove_dir_all(&dir);
}

fn set_attributes(file: &Path, flags: &str) {
    let attrib = crate::utils::process::system_root().join("System32").join("attrib.exe");
    let status = std::process::Command::new(attrib)
        .args(flags.split(' '))
        .arg(file)
        .status()
        .expect("lancement d'attrib");
    assert!(status.success(), "attrib {flags} {}", file.display());
}

#[test]
fn hidden_and_system_files_are_not_startup_programs() {
    let dir = std::env::temp_dir().join(format!("wintyx_startup_hidden_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::File::create(dir.join("Visible.lnk")).unwrap();
    fs::File::create(dir.join("Eteint.lnk.disabled")).unwrap();
    for (name, flags) in [("desktop.ini", "+h +s"), ("Cache.lnk", "+h"), ("Systeme.lnk", "+s")] {
        let path = dir.join(name);
        fs::File::create(&path).unwrap();
        set_attributes(&path, flags);
    }

    let mut names: Vec<String> = items_in_startup_dir(&dir, "test").into_iter().map(|i| i.name).collect();
    names.sort();

    for name in ["desktop.ini", "Cache.lnk", "Systeme.lnk"] {
        let path = dir.join(name);
        set_attributes(&path, "-h -s");
    }
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(names, ["Eteint", "Visible"]);
}

#[test]
fn registry_items_carry_the_enabled_flag() {
    let path = format!(r"HKCU\Software\Wintyx-test-{}-startup", std::process::id());
    registry::set_string(&path, "Vieux Truc", r"C:\x.exe").unwrap();

    let off = get_registry_startup_items(&path, "HKCU_Run", "d", false);
    let on = get_registry_startup_items(&path, "HKCU_Run", "d", true);

    assert_eq!(off.len(), 1);
    assert!(!off[0].is_enabled && on[0].is_enabled);
    assert_eq!(off[0].name, "Vieux Truc");
    let _ = windows_registry::CURRENT_USER.remove_tree(path.strip_prefix(r"HKCU\").unwrap());
}

fn unknown_entry(test: &str) -> String {
    format!("WintyxNoSuchEntry{}{}", test, std::process::id())
}

#[test]
fn disabling_or_enabling_an_entry_that_does_not_exist_is_an_error_for_the_run_keys() {
    let name = unknown_entry("RunKeys");
    assert!(disable_startup_item_sync(name.clone(), "HKCU_Run".into()).is_err());
    assert!(enable_startup_item_sync(name, "HKCU_Run".into()).is_err());
}

#[test]
fn disabling_a_missing_runonce_entry_is_not_found_instead_of_ok() {
    let err = disable_startup_item_sync(unknown_entry("MissingRunOnce"), "HKCU_RunOnce".into()).unwrap_err();
    assert!(matches!(err, CommandError::NotFound(_)), "{err:?}");
}

#[test]
fn a_missing_startup_folder_entry_is_not_found_in_both_directions() {
    let err = disable_startup_item_sync(unknown_entry("MissingFolder"), "StartupFolder".into()).unwrap_err();
    assert!(matches!(err, CommandError::NotFound(_)), "{err:?}");
    let err = enable_startup_item_sync(unknown_entry("MissingFolder"), "StartupFolder".into()).unwrap_err();
    assert!(matches!(err, CommandError::NotFound(_)), "{err:?}");
}

#[test]
fn a_present_runonce_entry_is_still_removed() {
    let name = unknown_entry("PresentRunOnce");
    registry::set_string(HKCU_RUNONCE, &name, r"cmd.exe /c exit").unwrap();
    disable_startup_item_sync(name.clone(), "HKCU_RunOnce".into()).unwrap();
    assert!(!registry::key_exists(HKCU_RUNONCE, &name));
}
