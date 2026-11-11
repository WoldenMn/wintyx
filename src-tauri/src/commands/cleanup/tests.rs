use super::*;

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("wintyx-cleanup-{}-{}", tag, std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn folder_size_missing_dir_is_accessible_zero() {
    let missing = temp_dir("missing").join("nope");
    assert_eq!(get_folder_size(&missing.to_string_lossy(), "*"), Some(0));
}

#[test]
fn folder_size_unreadable_dir_is_none() {
    let d = temp_dir("file");
    let f = d.join("a.txt");
    fs::write(&f, b"x").unwrap();
    assert_eq!(get_folder_size(&f.to_string_lossy(), "*"), None);
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn folder_size_counts_files() {
    let d = temp_dir("size");
    fs::write(d.join("a.bin"), vec![0u8; 100]).unwrap();
    assert_eq!(get_folder_size(&d.to_string_lossy(), "*"), Some(100));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn clean_directory_reports_unreadable_dir() {
    let d = temp_dir("clean-file");
    let f = d.join("a.txt");
    fs::write(&f, b"x").unwrap();
    assert!(clean_directory(&f, "*").is_err());
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn clean_directory_removes_matching_entries_only() {
    let d = temp_dir("clean-pattern");
    fs::write(d.join("a.pf"), b"x").unwrap();
    fs::write(d.join("keep.txt"), b"x").unwrap();
    let errors = clean_directory(&d, "*.pf").unwrap();
    assert!(errors.is_empty());
    assert!(!d.join("a.pf").exists());
    assert!(d.join("keep.txt").exists());
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn read_dir_error_maps_permission_denied() {
    let e = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    assert!(matches!(read_dir_error("X", &e), CommandError::PermissionDenied(_)));
    let e = std::io::Error::from(std::io::ErrorKind::NotFound);
    assert!(matches!(read_dir_error("X", &e), CommandError::Generic(_)));
}

#[test]
fn unreadable_folder_is_logged_as_a_read_not_an_action() {
    let d = temp_dir("read-category");
    let f = d.join("lecture-marker.txt");
    fs::write(&f, b"x").unwrap();
    assert_eq!(get_folder_size(&f.to_string_lossy(), "*"), None);
    let log = fs::read_to_string(crate::utils::logger::log_file()).unwrap();
    let line = log.lines().find(|l| l.contains("lecture-marker.txt")).expect("ligne de journal");
    assert!(line.contains("[cleanup.lecture]"), "{}", line);
    let _ = fs::remove_dir_all(&d);
}

fn target(id: &str, accessible: bool, size_bytes: u64) -> CleanupTarget {
    CleanupTarget {
        id: id.into(),
        name: id.into(),
        description: String::new(),
        path: String::new(),
        size_bytes,
        size_display: String::new(),
        is_system: false,
        accessible,
    }
}

#[test]
fn only_accessible_non_empty_targets_are_cleanable() {
    assert!(is_cleanable(&target("a", true, 10)));
    assert!(!is_cleanable(&target("b", true, 0)));
    assert!(!is_cleanable(&target("c", false, 0)));
    assert!(!is_cleanable(&target("d", false, 10)));
}

fn system_target(id: &str) -> CleanupTarget {
    CleanupTarget {
        is_system: true,
        ..target(id, true, 10)
    }
}

fn pair(t: CleanupTarget) -> (CleanupTarget, String) {
    (t, "*".to_string())
}

#[test]
fn without_admin_rights_system_targets_are_skipped_with_one_clear_line_each() {
    let targets = vec![
        pair(target("user", true, 10)),
        pair(system_target("sys")),
        pair(target("empty", true, 0)),
    ];
    let (selected, skipped) = select_cleanable(&targets, false);
    assert_eq!(selected.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), ["user"]);
    assert_eq!(skipped.len(), 1);
    assert!(skipped[0].contains("droits administrateur"), "{}", skipped[0]);
}

#[test]
fn with_admin_rights_every_cleanable_target_is_selected_and_nothing_is_reported() {
    let targets = vec![pair(target("user", true, 10)), pair(system_target("sys"))];
    let (selected, skipped) = select_cleanable(&targets, true);
    assert_eq!(selected.len(), 2);
    assert!(skipped.is_empty());
}

#[test]
fn an_inaccessible_system_target_is_not_reported_as_a_rights_problem_twice() {
    let mut inaccessible = system_target("sys");
    inaccessible.accessible = false;
    let targets = [pair(inaccessible)];
    let (selected, skipped) = select_cleanable(&targets, false);
    assert!(selected.is_empty() && skipped.is_empty());
}

#[test]
fn cleaning_a_system_target_without_admin_rights_is_refused_up_front() {
    let err = ensure_rights(&system_target("sys"), false).unwrap_err();
    assert!(matches!(err, CommandError::NotAdmin(_)), "{err:?}");
    assert!(ensure_rights(&system_target("sys"), true).is_ok());
}

#[test]
fn a_user_target_never_needs_admin_rights() {
    assert!(ensure_rights(&target("user", true, 10), false).is_ok());
}
