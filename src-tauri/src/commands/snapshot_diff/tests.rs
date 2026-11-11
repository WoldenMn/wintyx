use super::*;

fn req(key: &str, value_name: &str, ct: ChangeRequestType, target: Option<&str>) -> ChangeRequest {
    ChangeRequest {
        key_path: key.to_string(),
        value_name: value_name.to_string(),
        change_type: ct,
        target_value: target.map(String::from),
    }
}

#[test]
fn list_snapshot_groups_empty_when_folder_missing() {
    let _ = list_snapshot_groups_sync();
}

#[test]
fn restore_rejects_empty_changes() {
    let result = restore_selected_changes_sync(vec![], None);
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

#[test]
fn restore_rejects_too_many_changes() {
    let many: Vec<ChangeRequest> = (0..51)
        .map(|i| {
            req(
                "HKCU\\SOFTWARE\\Foo",
                &format!("v{}", i),
                ChangeRequestType::UndoAdded,
                None,
            )
        })
        .collect();
    let result = restore_selected_changes_sync(many, None);
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

#[test]
fn restore_rejects_disallowed_key_path() {
    let changes = vec![req(
        "HKCR\\Foo",
        "V",
        ChangeRequestType::UndoAdded,
        None,
    )];
    let result = restore_selected_changes_sync(changes, None);
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

#[test]
fn restore_rejects_invalid_value_name() {
    let changes = vec![req(
        "HKCU\\SOFTWARE\\Foo",
        "name;inject",
        ChangeRequestType::UndoAdded,
        None,
    )];
    let result = restore_selected_changes_sync(changes, None);
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

#[test]
fn restore_rejects_modified_without_target_value() {
    let changes = vec![req(
        "HKCU\\SOFTWARE\\Foo",
        "V",
        ChangeRequestType::RestoreModified,
        None,
    )];
    let result = restore_selected_changes_sync(changes, None);
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

fn recorded_foo() -> Vec<provenance::RecordedValues> {
    let mut snapshot = provenance::RecordedValues::new();
    snapshot
        .entry("hkcu\\software\\foo".to_string())
        .or_default()
        .insert("v1".to_string(), "dword:00000001".to_string());
    vec![snapshot]
}

#[test]
fn restore_refuses_a_value_that_no_snapshot_recorded() {
    let changes = vec![req(
        "HKCU\\SOFTWARE\\Foo",
        "Unknown",
        ChangeRequestType::RestoreRemoved,
        Some("dword:00000001"),
    )];
    let result = restore_verified(changes, None, &recorded_foo());
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

#[test]
fn restore_refuses_a_value_that_differs_from_the_snapshot_even_in_dry_run() {
    let changes = vec![req(
        "HKCU\\SOFTWARE\\Foo",
        "V1",
        ChangeRequestType::RestoreModified,
        Some("dword:00000002"),
    )];
    let opts = Some(BatchOptions { dry_run: true });
    let result = restore_verified(changes, opts, &recorded_foo());
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

#[test]
fn restore_without_any_snapshot_on_disk_refuses_every_write() {
    let changes = vec![req(
        "HKCU\\SOFTWARE\\Foo",
        "V1",
        ChangeRequestType::RestoreModified,
        Some("dword:00000001"),
    )];
    assert!(matches!(restore_verified(changes, None, &[]), Err(CommandError::InvalidInput(_))));
}

#[test]
fn restore_dry_run_returns_preview_items() {
    let changes = vec![
        req(
            "HKCU\\SOFTWARE\\Foo",
            "V1",
            ChangeRequestType::RestoreModified,
            Some("dword:00000001"),
        ),
        req(
            "HKCU\\SOFTWARE\\Foo",
            "V2",
            ChangeRequestType::UndoAdded,
            None,
        ),
    ];
    let opts = Some(BatchOptions { dry_run: true });
    let result = restore_verified(changes, opts, &recorded_foo()).unwrap();
    assert!(result.dry_run);
    assert_eq!(result.preview_items.len(), 2);
    assert_eq!(result.total, 2);
    assert_eq!(result.success, 0);
    assert_eq!(result.failed, 0);
    assert!(result.preview_items[0].contains("V1"));
    assert!(result.preview_items[0].contains("dword:00000001"));
    assert!(result.preview_items[1].contains("V2"));
    assert!(result.preview_items[1].contains("supprimer"));
}
