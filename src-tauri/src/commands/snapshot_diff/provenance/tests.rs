use super::*;
use crate::utils::temp_file::random_token;

const KEY: &str = r"HKLM\SOFTWARE\Policies\Microsoft\Windows\System";

fn snapshot(sections: &str) -> RecordedValues {
    let mut recorded = RecordedValues::new();
    for (section, name, raw) in parser::parse_reg_content(sections) {
        recorded
            .entry(section.to_lowercase())
            .or_default()
            .insert(name.to_lowercase(), raw);
    }
    recorded
}

fn older() -> RecordedValues {
    snapshot(
        r#"[HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\System]
"EnableSmartScreen"=dword:00000001
"Label"="C:\\Tools\\x"
@="default text"
"#,
    )
}

fn newer() -> RecordedValues {
    snapshot(
        r#"[HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\System]
"EnableSmartScreen"=dword:00000000
"Extra"=dword:00000005
"#,
    )
}

fn change(key: &str, name: &str, kind: ChangeRequestType, target: Option<&str>) -> ChangeRequest {
    ChangeRequest {
        key_path: key.to_string(),
        value_name: name.to_string(),
        change_type: kind,
        target_value: target.map(String::from),
    }
}

fn refusal(result: Result<(), CommandError>) -> String {
    match result {
        Err(CommandError::InvalidInput(msg)) => msg,
        other => panic!("refus attendu, obtenu {other:?}"),
    }
}

#[test]
fn a_value_found_verbatim_in_a_snapshot_is_accepted() {
    let snapshots = [older(), newer()];
    for (name, kind, target) in [
        ("EnableSmartScreen", ChangeRequestType::RestoreModified, "dword:00000001"),
        ("EnableSmartScreen", ChangeRequestType::RestoreRemoved, "dword:00000000"),
        ("Label", ChangeRequestType::RestoreModified, r#""C:\\Tools\\x""#),
        ("", ChangeRequestType::RestoreModified, r#""default text""#),
    ] {
        let request = change(KEY, name, kind, Some(target));
        assert!(verify_changes(&[request], &snapshots).is_ok(), "{name} {target}");
    }
}

#[test]
fn a_value_that_no_snapshot_holds_is_refused() {
    let request = change(KEY, "Injected", ChangeRequestType::RestoreRemoved, Some("dword:00000001"));
    let msg = refusal(verify_changes(&[request], &[older(), newer()]));
    assert!(msg.contains("absente"), "{msg}");
    let request = change(KEY, "Label", ChangeRequestType::RestoreRemoved, Some("dword:00000001"));
    assert!(verify_changes(&[request], &[]).is_err());
}

#[test]
fn a_value_that_differs_from_every_recorded_one_is_refused() {
    let request = change(KEY, "EnableSmartScreen", ChangeRequestType::RestoreModified, Some("dword:00000002"));
    let msg = refusal(verify_changes(&[request], &[older(), newer()]));
    assert!(msg.contains("différente"), "{msg}");
    let request = change(KEY, "Label", ChangeRequestType::RestoreModified, Some(r#""C:\\Tools\\y""#));
    assert!(verify_changes(&[request], &[older()]).is_err());
}

#[test]
fn the_comparison_of_the_value_text_is_exact() {
    let request = change(KEY, "EnableSmartScreen", ChangeRequestType::RestoreModified, Some("DWORD:00000001"));
    assert!(verify_changes(&[request], &[older()]).is_err());
    let request = change(KEY, "EnableSmartScreen", ChangeRequestType::RestoreModified, Some("dword:1"));
    assert!(verify_changes(&[request], &[older()]).is_err());
}

#[test]
fn key_and_value_names_are_matched_the_way_the_registry_matches_them() {
    let request = change(
        r"hklm\software\policies\microsoft\windows\SYSTEM",
        "enablesmartscreen",
        ChangeRequestType::RestoreModified,
        Some("dword:00000001"),
    );
    assert!(verify_changes(&[request], &[older()]).is_ok());
}

#[test]
fn a_key_that_no_snapshot_recorded_is_refused() {
    let request = change(
        r"HKLM\SYSTEM\CurrentControlSet\Services\Evil",
        "ImagePath",
        ChangeRequestType::RestoreModified,
        Some(r#""C:\\evil.exe""#),
    );
    assert!(verify_changes(&[request], &[older(), newer()]).is_err());
    let sibling = change(
        r"HKLM\SOFTWARE\Policies\Microsoft\Windows\SystemX",
        "EnableSmartScreen",
        ChangeRequestType::RestoreModified,
        Some("dword:00000001"),
    );
    assert!(verify_changes(&[sibling], &[older()]).is_err());
}

#[test]
fn the_first_bad_change_of_a_batch_refuses_the_whole_batch() {
    let good = change(KEY, "EnableSmartScreen", ChangeRequestType::RestoreModified, Some("dword:00000001"));
    let bad = change(KEY, "Injected", ChangeRequestType::RestoreRemoved, Some("dword:00000001"));
    assert!(verify_changes(&[good.clone(), bad], &[older()]).is_err());
    assert!(verify_changes(&[good], &[older()]).is_ok());
}

#[test]
fn undoing_a_value_added_after_a_snapshot_of_that_key_is_accepted() {
    let request = change(KEY, "Extra", ChangeRequestType::UndoAdded, None);
    assert!(verify_changes(std::slice::from_ref(&request), &[older()]).is_ok());
    assert!(verify_changes(&[request], &[older(), newer()]).is_ok());
}

#[test]
fn undoing_a_value_that_every_snapshot_holds_is_refused() {
    let request = change(KEY, "EnableSmartScreen", ChangeRequestType::UndoAdded, None);
    let msg = refusal(verify_changes(&[request], &[older(), newer()]));
    assert!(msg.contains("tous les snapshots"), "{msg}");
}

#[test]
fn undoing_a_value_in_a_key_that_no_snapshot_covers_is_refused() {
    let request = change(r"HKLM\SYSTEM\CurrentControlSet\Services\Spooler", "Start", ChangeRequestType::UndoAdded, None);
    let msg = refusal(verify_changes(std::slice::from_ref(&request), &[older(), newer()]));
    assert!(msg.contains("Aucun snapshot"), "{msg}");
    assert!(verify_changes(&[request], &[]).is_err());
}

fn temp_folder() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wintyx-provenance-{}", random_token()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_reg(dir: &std::path::Path, name: &str, text: &str) {
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
    std::fs::write(dir.join(name), bytes).unwrap();
}

#[test]
fn load_snapshots_merges_the_files_of_one_batch_and_separates_batches() {
    let dir = temp_folder();
    let header = "Windows Registry Editor Version 5.00\r\n\r\n";
    write_reg(
        &dir,
        "regkey_a_20261001_100000.reg",
        &format!("{header}[HKEY_LOCAL_MACHINE\\SOFTWARE\\A]\r\n\"v\"=dword:00000001\r\n"),
    );
    write_reg(
        &dir,
        "regkey_b_20261001_100000.reg",
        &format!("{header}[HKEY_LOCAL_MACHINE\\SOFTWARE\\B]\r\n\"w\"=dword:00000002\r\n"),
    );
    write_reg(
        &dir,
        "regkey_a_20261002_100000.reg",
        &format!("{header}[HKEY_LOCAL_MACHINE\\SOFTWARE\\A]\r\n\"v\"=dword:00000009\r\n"),
    );
    std::fs::write(dir.join("notes.txt"), "ignore").unwrap();

    let snapshots = load_snapshots(&dir);
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0]["hklm\\software\\a"]["v"], "dword:00000001");
    assert_eq!(snapshots[0]["hklm\\software\\b"]["w"], "dword:00000002");
    assert_eq!(snapshots[1]["hklm\\software\\a"]["v"], "dword:00000009");
    assert!(!snapshots[1].contains_key("hklm\\software\\b"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn load_snapshots_of_a_missing_folder_is_empty() {
    let dir = std::env::temp_dir().join(format!("wintyx-provenance-{}", random_token()));
    assert!(load_snapshots(&dir).is_empty());
}
