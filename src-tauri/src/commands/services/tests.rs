use super::*;

#[test]
fn risk_weight_high_is_three() {
    assert_eq!(risk_weight("High"), 3);
    assert_eq!(risk_weight("high"), 3);
    assert_eq!(risk_weight("HIGH"), 3);
}

#[test]
fn risk_weight_medium_is_two() {
    assert_eq!(risk_weight("Medium"), 2);
    assert_eq!(risk_weight("medium"), 2);
}

#[test]
fn risk_weight_low_is_one() {
    assert_eq!(risk_weight("Low"), 1);
    assert_eq!(risk_weight("low"), 1);
    assert_eq!(risk_weight("Unknown"), 1);
}

#[test]
fn compute_score_empty_is_zero() {
    assert_eq!(compute_score(&[]), 0);
}

#[test]
fn compute_score_all_stopped_is_100() {
    let items = vec![
        ("High".to_string(), "Stopped".to_string()),
        ("Medium".to_string(), "Stopped".to_string()),
        ("Low".to_string(), "Stopped".to_string()),
    ];
    assert_eq!(compute_score(&items), 100);
}

#[test]
fn compute_score_all_running_is_0() {
    let items = vec![
        ("High".to_string(), "Running".to_string()),
        ("Medium".to_string(), "Running".to_string()),
        ("Low".to_string(), "Running".to_string()),
    ];
    assert_eq!(compute_score(&items), 0);
}

#[test]
fn compute_score_absent_satisfies_recommendation() {
    let items = vec![
        ("High".to_string(), "Absent".to_string()),
        ("Low".to_string(), "Running".to_string()),
    ];
    assert_eq!(compute_score(&items), 75);
}

#[test]
fn compute_score_unknown_is_not_earned() {
    let items = vec![
        ("High".to_string(), "Unknown".to_string()),
        ("Low".to_string(), "Stopped".to_string()),
    ];
    assert_eq!(compute_score(&items), 25);
}

#[test]
fn classify_absent_from_exit_code_1060() {
    assert_eq!(classify_query(false, 1060, ""), ServiceState::Absent);
}

#[test]
fn classify_absent_from_stdout_when_exit_code_lost() {
    let out = "[SC] EnumQueryServicesStatus:OpenService echec(s) 1060 :";
    assert_eq!(classify_query(false, 36, out), ServiceState::Absent);
}

#[test]
fn classify_timeout_is_unknown_not_absent() {
    assert_eq!(classify_query(false, -1, ""), ServiceState::Unknown);
}

#[test]
fn classify_unreadable_output_is_unknown() {
    assert_eq!(classify_query(true, 0, "garbage"), ServiceState::Unknown);
}

#[test]
fn classify_running_and_stopped() {
    assert_eq!(classify_query(true, 0, "STATE : 4  RUNNING"), ServiceState::Running);
    assert_eq!(classify_query(true, 0, "STATE : 1  STOPPED"), ServiceState::Stopped);
}

#[test]
fn compute_score_partial_weighted() {
    let items = vec![
        ("High".to_string(), "Stopped".to_string()),
        ("Low".to_string(), "Running".to_string()),
    ];
    assert_eq!(compute_score(&items), 75);
}

#[test]
fn classify_stop_success_is_stopped() {
    assert_eq!(classify_stop(true, 0, ""), StopOutcome::Stopped);
}

#[test]
fn classify_stop_1062_is_already_stopped_not_failure() {
    assert_eq!(classify_stop(false, 1062, ""), StopOutcome::AlreadyStopped);
    let out = "[SC] ControlService FAILED 1062:\n\nThe service has not been started.";
    assert_eq!(classify_stop(false, 1, out), StopOutcome::AlreadyStopped);
}

#[test]
fn classify_stop_1060_is_absent_not_failure() {
    assert_eq!(classify_stop(false, 1060, ""), StopOutcome::Absent);
    let out = "[SC] OpenService FAILED 1060:\n\nThe specified service does not exist as an installed service.";
    assert_eq!(classify_stop(false, 1, out), StopOutcome::Absent);
}

#[test]
fn classify_stop_other_errors_are_failures() {
    assert_eq!(classify_stop(false, 5, "[SC] OpenService FAILED 5:"), StopOutcome::Failed);
    assert_eq!(classify_stop(false, -1, ""), StopOutcome::Failed);
}

#[test]
fn original_start_is_written_when_absent() {
    let path = format!(r"HKCU\Software\Wintyx-test-{}-svc-a", std::process::id());
    backup_original_start_to(&path, Some(2));
    assert_eq!(registry::get_dword(&path, "OriginalStart"), Some(2));
    remove_test_key(&path);
}

#[test]
fn original_start_is_not_overwritten_by_a_second_backup() {
    let path = format!(r"HKCU\Software\Wintyx-test-{}-svc-b", std::process::id());
    backup_original_start_to(&path, Some(2));
    backup_original_start_to(&path, Some(4));
    assert_eq!(registry::get_dword(&path, "OriginalStart"), Some(2));
    remove_test_key(&path);
}

#[test]
fn original_start_is_not_written_without_current_value() {
    let path = format!(r"HKCU\Software\Wintyx-test-{}-svc-c", std::process::id());
    backup_original_start_to(&path, None);
    assert_eq!(registry::get_dword(&path, "OriginalStart"), None);
    remove_test_key(&path);
}

fn remove_test_key(path: &str) {
    let sub = path.strip_prefix(r"HKCU\").unwrap();
    let _ = windows_registry::CURRENT_USER.remove_tree(sub);
}

#[test]
fn reading_the_state_of_an_absent_service_leaves_no_info_line() {
    let name = format!("WintyxAbsent{}", std::process::id());
    assert_eq!(query_service_state(&name), ServiceState::Absent);
    let log = std::fs::read_to_string(crate::utils::logger::log_file()).unwrap_or_default();
    assert!(!log.lines().any(|l| l.contains(&name)), "lecture sans effet journalisée");
}
