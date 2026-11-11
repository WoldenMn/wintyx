use super::*;

#[test]
fn describe_change_restore_modified() {
    let c = ChangeRequest {
        key_path: "HKLM\\SOFTWARE\\Foo".to_string(),
        value_name: "Setting".to_string(),
        change_type: ChangeRequestType::RestoreModified,
        target_value: Some("dword:00000001".to_string()),
    };
    let desc = describe_change(&c);
    assert!(desc.contains("HKLM\\SOFTWARE\\Foo"));
    assert!(desc.contains("Setting"));
    assert!(desc.contains("dword:00000001"));
    assert!(desc.contains("restaurer") || desc.contains("Restaurer"));
}

#[test]
fn describe_change_restore_removed() {
    let c = ChangeRequest {
        key_path: "HKCU\\SOFTWARE\\Bar".to_string(),
        value_name: "DeletedValue".to_string(),
        change_type: ChangeRequestType::RestoreRemoved,
        target_value: Some("\"hello\"".to_string()),
    };
    let desc = describe_change(&c);
    assert!(desc.contains("HKCU\\SOFTWARE\\Bar"));
    assert!(desc.contains("DeletedValue"));
    assert!(desc.contains("recréer") || desc.contains("Recréer"));
}

#[test]
fn describe_change_undo_added() {
    let c = ChangeRequest {
        key_path: "HKLM\\SOFTWARE\\Foo".to_string(),
        value_name: "Spurious".to_string(),
        change_type: ChangeRequestType::UndoAdded,
        target_value: None,
    };
    let desc = describe_change(&c);
    assert!(desc.contains("HKLM\\SOFTWARE\\Foo"));
    assert!(desc.contains("Spurious"));
    assert!(desc.contains("supprimer") || desc.contains("Supprimer"));
}

#[test]
fn diff_snapshot_vs_snapshot_detects_modified() {
    let mut a: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut b: HashMap<String, HashMap<String, String>> = HashMap::new();
    a.entry("HKLM\\SOFTWARE\\Foo".to_string())
        .or_default()
        .insert("V".to_string(), "dword:00000001".to_string());
    b.entry("HKLM\\SOFTWARE\\Foo".to_string())
        .or_default()
        .insert("V".to_string(), "dword:00000002".to_string());

    let diff = diff_two_maps(&a, &b);
    assert_eq!(diff.total_changes, 1);
    assert_eq!(diff.keys.len(), 1);
    match &diff.keys[0].changes[0] {
        KeyValueChange::Modified { name, snapshot, current } => {
            assert_eq!(name, "V");
            assert_eq!(snapshot, "dword:00000001");
            assert_eq!(current, "dword:00000002");
        }
        _ => panic!("expected Modified"),
    }
}

#[test]
fn diff_snapshot_vs_snapshot_detects_removed() {
    let mut a: HashMap<String, HashMap<String, String>> = HashMap::new();
    let b: HashMap<String, HashMap<String, String>> = HashMap::new();
    a.entry("HKLM\\SOFTWARE\\Foo".to_string())
        .or_default()
        .insert("Gone".to_string(), "dword:00000001".to_string());

    let diff = diff_two_maps(&a, &b);
    assert_eq!(diff.total_changes, 1);
    match &diff.keys[0].changes[0] {
        KeyValueChange::Removed { name, snapshot } => {
            assert_eq!(name, "Gone");
            assert_eq!(snapshot, "dword:00000001");
        }
        _ => panic!("expected Removed"),
    }
}

#[test]
fn diff_snapshot_vs_snapshot_detects_added() {
    let a: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut b: HashMap<String, HashMap<String, String>> = HashMap::new();
    b.entry("HKLM\\SOFTWARE\\Foo".to_string())
        .or_default()
        .insert("New".to_string(), "dword:00000003".to_string());

    let diff = diff_two_maps(&a, &b);
    assert_eq!(diff.total_changes, 1);
    match &diff.keys[0].changes[0] {
        KeyValueChange::Added { name, current } => {
            assert_eq!(name, "New");
            assert_eq!(current, "dword:00000003");
        }
        _ => panic!("expected Added"),
    }
}

#[test]
fn diff_snapshot_vs_snapshot_identical_returns_empty() {
    let mut a: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut b: HashMap<String, HashMap<String, String>> = HashMap::new();
    a.entry("HKLM\\SOFTWARE\\Foo".to_string())
        .or_default()
        .insert("V".to_string(), "dword:00000001".to_string());
    b.entry("HKLM\\SOFTWARE\\Foo".to_string())
        .or_default()
        .insert("V".to_string(), "dword:00000001".to_string());

    let diff = diff_two_maps(&a, &b);
    assert_eq!(diff.total_changes, 0);
    assert!(diff.keys.is_empty());
}

// --- format_live_value_dword (NOUVEAU v2.9.0) ---

#[test]
fn format_live_value_dword_zero() {
    assert_eq!(format_live_value_dword(0), "dword:00000000");
}

#[test]
fn format_live_value_dword_small() {
    assert_eq!(format_live_value_dword(42), "dword:0000002a");
}

#[test]
fn format_live_value_dword_large() {
    assert_eq!(format_live_value_dword(0xFFFFFFFE), "dword:fffffffe");
}

// --- format_live_value_string (NOUVEAU v2.9.0) ---

#[test]
fn format_live_value_string_empty_and_simple() {
    assert_eq!(format_live_value_string(""), "\"\"");
    assert_eq!(format_live_value_string("hello"), "\"hello\"");
}

#[test]
fn format_live_value_string_escapes_quotes_and_backslashes() {
    assert_eq!(
        format_live_value_string("path\\to\\file"),
        "\"path\\\\to\\\\file\""
    );
    assert_eq!(
        format_live_value_string("a \"quoted\" value"),
        "\"a \\\"quoted\\\" value\""
    );
}
