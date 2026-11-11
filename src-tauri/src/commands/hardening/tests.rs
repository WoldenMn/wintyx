use super::*;

#[test]
fn ratio_score_zero_total_is_zero() {
    assert_eq!(ratio_score(0, 0), 0);
    assert_eq!(ratio_score(5, 0), 0);
}

#[test]
fn ratio_score_all_secure_is_100() {
    assert_eq!(ratio_score(10, 10), 100);
}

#[test]
fn ratio_score_none_secure_is_0() {
    assert_eq!(ratio_score(0, 10), 0);
}

#[test]
fn ratio_score_half_is_50() {
    assert_eq!(ratio_score(5, 10), 50);
}

#[test]
fn ratio_score_rounds_correctly() {
    assert_eq!(ratio_score(2, 3), 67);
    assert_eq!(ratio_score(1, 3), 33);
}

#[test]
fn ratio_score_single_check() {
    assert_eq!(ratio_score(1, 1), 100);
    assert_eq!(ratio_score(0, 1), 0);
}

#[test]
fn build_checks_contains_essential_and_recommended() {
    let checks = build_checks();
    assert!(checks.iter().any(|c| c.risk_level == "Essential"));
    assert!(checks.iter().any(|c| c.risk_level == "Recommended"));
}

#[test]
fn an_unknown_fix_type_is_an_error_not_a_silent_success() {
    let def = CheckDef {
        id: "X",
        category: "c",
        name: "Inconnu",
        description: "d",
        risk_level: "Essential",
        reg_path: "",
        reg_value: "",
        secure_value: 0,
        fix_type: "Bogus",
        fix_target: "",
    };
    let err = apply_fix(&def).unwrap_err();
    assert!(err.contains("Bogus"), "{err}");
}

#[test]
fn every_shipped_check_uses_a_fix_type_that_apply_fix_handles() {
    for def in build_checks() {
        assert!(
            ["Registry", "Service", "Command", "Manual"].contains(&def.fix_type),
            "{} : fix_type {} non gere",
            def.id,
            def.fix_type
        );
    }
}

#[test]
fn build_checks_all_have_non_empty_id_name() {
    let checks = build_checks();
    assert!(!checks.is_empty());
    for c in &checks {
        assert!(!c.id.is_empty(), "id vide detecte");
        assert!(!c.name.is_empty(), "name vide detecte");
    }
}

#[test]
fn wdigest_absent_value_is_secure_because_windows_8_1_defaults_to_no_clear_text_cache() {
    assert_eq!(wdigest_state(Ok(None)), CheckState::Secure);
    assert_eq!(wdigest_state(Ok(Some(0))), CheckState::Secure);
}

#[test]
fn wdigest_enabled_value_is_insecure_and_an_unreadable_one_is_unknown() {
    assert_eq!(wdigest_state(Ok(Some(1))), CheckState::Insecure);
    assert_eq!(wdigest_state(Ok(Some(7))), CheckState::Insecure);
    assert_eq!(wdigest_state(Err("acces refuse".into())), CheckState::Unknown);
}

#[test]
fn wdigest_check_goes_through_the_dedicated_reader_not_the_generic_one() {
    let wdigest = build_checks().into_iter().find(|c| c.id == "WDigest").expect("check WDigest");
    assert_eq!(wdigest.reg_value, "UseLogonCredential");
    assert_eq!(wdigest.secure_value, 0);
    let live = wdigest_check_state(wdigest.reg_path, wdigest.reg_value);
    assert_eq!(check_status(&wdigest), live);
}

#[test]
fn guest_state_reads_enabled_flag_of_the_sid_query() {
    assert_eq!(guest_state(true, "False\r\n"), CheckState::Secure);
    assert_eq!(guest_state(true, "True\r\n"), CheckState::Insecure);
}

#[test]
fn guest_state_is_unknown_when_account_not_found_or_command_failed() {
    assert_eq!(guest_state(true, ""), CheckState::Unknown);
    assert_eq!(guest_state(false, "False"), CheckState::Unknown);
    assert_eq!(guest_state(true, "Get-LocalUser : introuvable"), CheckState::Unknown);
}

#[test]
fn guest_query_targets_the_sid_not_the_account_name() {
    assert!(GUEST_QUERY_ARGS.contains("-501"));
    assert!(!GUEST_QUERY_ARGS.contains("-Name Guest"));
    assert!(GUEST_DISABLE_COMMAND.contains("-501"));
}

const SC_RUNNING: &str = "SERVICE_NAME: Spooler \r\n        TYPE               : 110  WIN32_OWN_PROCESS  (interactive)\r\n        STATE              : 4  RUNNING \r\n                                (STOPPABLE, NOT_PAUSABLE, IGNORES_SHUTDOWN)\r\n";
const SC_STOPPED: &str = "SERVICE_NAME: Spooler \r\n        TYPE               : 110  WIN32_OWN_PROCESS  (interactive)\r\n        STATE              : 1  STOPPED \r\n";

#[test]
fn spooler_state_running_is_insecure_stopped_is_secure() {
    assert_eq!(spooler_state(true, 0, SC_RUNNING), CheckState::Insecure);
    assert_eq!(spooler_state(true, 0, SC_STOPPED), CheckState::Secure);
}

#[test]
fn spooler_state_absent_service_is_secure() {
    assert_eq!(spooler_state(false, 1060, "[SC] EnumQueryServicesStatus:OpenService echec(s) 1060"), CheckState::Secure);
}

#[test]
fn spooler_state_failure_is_unknown_not_secure() {
    assert_eq!(spooler_state(false, -1, ""), CheckState::Unknown);
    assert_eq!(spooler_state(false, 5, "Acces refuse"), CheckState::Unknown);
    assert_eq!(spooler_state(true, 0, "sortie sans etat"), CheckState::Unknown);
}

fn utf16le_with_bom(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xFE];
    for unit in text.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes
}

#[test]
fn secedit_export_is_decoded_from_utf16le() {
    let inf = "[System Access]\r\nLockoutBadCount = 5\r\n";
    assert_eq!(decode_secedit_inf(&utf16le_with_bom(inf)), inf);
    assert_eq!(decode_secedit_inf(inf.as_bytes()), inf);
}

#[test]
fn lockout_state_requires_a_threshold_between_1_and_10() {
    let inf = |n: u32| format!("[System Access]\r\nMinimumPasswordAge = 0\r\nLockoutBadCount = {}\r\nResetLockoutCount = 30\r\n", n);
    assert_eq!(lockout_state_from_inf(&inf(0)), CheckState::Insecure);
    assert_eq!(lockout_state_from_inf(&inf(5)), CheckState::Secure);
    assert_eq!(lockout_state_from_inf(&inf(10)), CheckState::Secure);
    assert_eq!(lockout_state_from_inf(&inf(11)), CheckState::Insecure);
}

#[test]
fn lockout_state_unknown_when_key_missing_or_unparsable() {
    assert_eq!(lockout_state_from_inf(""), CheckState::Unknown);
    assert_eq!(lockout_state_from_inf("[System Access]\r\nMinimumPasswordAge = 0\r\n"), CheckState::Unknown);
    assert_eq!(lockout_state_from_inf("LockoutBadCount = abc"), CheckState::Unknown);
}

#[test]
fn credential_guard_needs_vbs_and_lsa_cfg_flags() {
    assert_eq!(credential_guard_state(Some(1), Some(1)), CheckState::Secure);
    assert_eq!(credential_guard_state(Some(1), Some(2)), CheckState::Secure);
    assert_eq!(credential_guard_state(Some(1), Some(0)), CheckState::Insecure);
    assert_eq!(credential_guard_state(Some(1), None), CheckState::Insecure);
    assert_eq!(credential_guard_state(Some(0), Some(1)), CheckState::Insecure);
    assert_eq!(credential_guard_state(None, None), CheckState::Insecure);
}

#[test]
fn score_ignores_unknown_checks() {
    assert_eq!(score_from_states(&[CheckState::Secure, CheckState::Unknown, CheckState::Insecure]), 50);
    assert_eq!(score_from_states(&[CheckState::Unknown, CheckState::Unknown]), 0);
}

#[test]
fn credential_guard_is_not_auto_fixable() {
    let defs = build_checks();
    let cg = defs.iter().find(|d| d.id == "CredentialGuard").unwrap();
    assert!(!is_auto_fixable(cg));
    assert!(apply_fix(cg).is_err());
}

#[test]
fn every_other_check_is_auto_fixable() {
    for def in build_checks().iter().filter(|d| d.id != "CredentialGuard") {
        assert!(is_auto_fixable(def), "{} doit rester corrigeable", def.id);
    }
}

#[test]
fn batches_never_preview_manual_checks() {
    let dry = crate::utils::batch::BatchOptions { dry_run: true };
    let all = apply_hardening_all_sync(Some(dry));
    assert!(!all.preview_items.iter().any(|n| n == "Credential Guard"));
    let dry = crate::utils::batch::BatchOptions { dry_run: true };
    let essential = apply_hardening_essential_sync(Some(dry));
    assert!(!essential.preview_items.iter().any(|n| n == "Credential Guard"));
}

#[test]
fn checks_expose_fixability_to_the_frontend() {
    let checks = get_hardening_checks_sync();
    let cg = checks.iter().find(|c| c.id == "CredentialGuard").unwrap();
    assert!(!cg.is_fixable);
    assert!(checks.iter().filter(|c| c.id != "CredentialGuard").all(|c| c.is_fixable));
}

#[test]
fn manual_fix_error_explains_manual_configuration() {
    let defs = build_checks();
    let cg = defs.iter().find(|d| d.id == "CredentialGuard").unwrap();
    let err = apply_fix(cg).unwrap_err();
    assert!(err.contains("manuelle"), "{err}");
}
