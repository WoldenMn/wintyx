use super::*;

#[test]
fn restore_type_known_codes() {
    assert_eq!(restore_type_label(0), "Installation d'application");
    assert_eq!(restore_type_label(1), "Désinstallation d'application");
    assert_eq!(restore_type_label(6), "Restauration");
    assert_eq!(restore_type_label(7), "Point de contrôle");
    assert_eq!(restore_type_label(10), "Installation de pilote");
    assert_eq!(restore_type_label(12), "Modification de paramètres");
}

#[test]
fn restore_type_unknown_code() {
    assert_eq!(restore_type_label(42), "Autre");
    assert_eq!(restore_type_label(999), "Autre");
}

#[test]
fn parse_restore_points_single_object_and_array() {
    let one = r#"{"Description":"A","CreationTime":"20260901","SequenceNumber":4,"RestorePointType":12}"#;
    let list = format!("[{},{}]", one, one.replace("\"A\"", "\"B\""));
    let single = parse_restore_points(one).unwrap();
    assert_eq!(single.len(), 1);
    assert_eq!(single[0].restore_type, "Modification de paramètres");
    assert_eq!(single[0].sequence_number, 4);
    let many = parse_restore_points(&list).unwrap();
    assert_eq!(many.iter().map(|p| p.description.as_str()).collect::<Vec<_>>(), ["A", "B"]);
}

#[test]
fn parse_restore_points_empty_and_invalid() {
    assert!(parse_restore_points("  \r\n").unwrap().is_empty());
    assert!(parse_restore_points("not json").is_err());
}

#[test]
fn sequence_increase_means_created() {
    assert_eq!(outcome_from_sequences(12, 13), RestorePointOutcome::Created);
    assert_eq!(outcome_from_sequences(0, 1), RestorePointOutcome::Created);
}

#[test]
fn unchanged_sequence_means_skipped_by_windows_limit() {
    assert_eq!(outcome_from_sequences(12, 12), RestorePointOutcome::SkippedWindowsLimit);
    assert_eq!(outcome_from_sequences(0, 0), RestorePointOutcome::SkippedWindowsLimit);
}

#[test]
fn parse_sequence_pair_reads_the_marker_line() {
    assert_eq!(parse_sequence_pair("SEQ:12:13\r\n"), Some((12, 13)));
    assert_eq!(parse_sequence_pair("SEQ:0:1"), Some((0, 1)));
}

#[test]
fn parse_sequence_pair_ignores_the_windows_24h_warning_text() {
    let out = "WARNING: A new system restore point cannot be created because one has already been created within the past 24 hours. Please try again later.\r\nSEQ:12:12\r\n";
    let (before, after) = parse_sequence_pair(out).unwrap();
    assert_eq!(outcome_from_sequences(before, after), RestorePointOutcome::SkippedWindowsLimit);
}

#[test]
fn parse_sequence_pair_rejects_unreadable_output() {
    assert_eq!(parse_sequence_pair(""), None);
    assert_eq!(parse_sequence_pair("garbage"), None);
    assert_eq!(parse_sequence_pair("SEQ:a:b"), None);
    assert_eq!(parse_sequence_pair("SEQ:5"), None);
}

#[test]
fn create_script_escapes_description_and_counts_before_after() {
    let script = build_create_script("Wintyx - avant d\u{2019}\u{e9}t\u{e9}; calc");
    assert!(script.contains("-Description 'Wintyx - avant d\u{2019}\u{2019}\u{e9}t\u{e9}; calc'"));
    assert_eq!(script.matches("Get-ComputerRestorePoint").count(), 2);
    assert!(script.contains("Write-Output ('SEQ:'+$before+':'+$after)"));
    assert!(script.starts_with("$ErrorActionPreference='Stop'"));
}

#[test]
fn outcome_serializes_camel_case() {
    assert_eq!(serde_json::to_value(RestorePointOutcome::Created).unwrap(), "created");
    assert_eq!(
        serde_json::to_value(RestorePointOutcome::SkippedWindowsLimit).unwrap(),
        "skippedWindowsLimit"
    );
}
