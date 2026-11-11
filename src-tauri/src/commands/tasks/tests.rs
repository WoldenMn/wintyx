use super::*;

fn task(author: &str, action_path: &str, action_args: &str) -> ScheduledTaskInfo {
    ScheduledTaskInfo {
        name: "Test".into(),
        path: "\\".into(),
        state: "Ready".into(),
        author: author.into(),
        description: String::new(),
        last_run_time: String::new(),
        next_run_time: String::new(),
        action_path: action_path.into(),
        action_arguments: action_args.into(),
        is_suspicious: false,
        needs_review: false,
        suspicious_reason: String::new(),
    }
}

fn analyzed(author: &str, action_path: &str, action_args: &str) -> ScheduledTaskInfo {
    let mut t = task(author, action_path, action_args);
    analyze_suspicious(&mut t);
    t
}

#[test]
fn is_trusted_author_matches_known_vendors() {
    assert!(analysis::is_trusted_author("Microsoft Corporation"));
    assert!(analysis::is_trusted_author("NVIDIA"));
    assert!(analysis::is_trusted_author("microsoft")); // case-insensitive
    assert!(analysis::is_trusted_author("SYSTEM"));
}

#[test]
fn is_trusted_author_rejects_empty_and_unknown() {
    assert!(!analysis::is_trusted_author(""));
    assert!(!analysis::is_trusted_author("   "));
    assert!(!analysis::is_trusted_author("RandomUser"));
    assert!(!analysis::is_trusted_author("Hacker LLC"));
}

#[test]
fn untrusted_powershell_is_to_review_not_suspicious() {
    let t = analyzed("Unknown", "powershell.exe", "-Command Get-Process");
    assert!(!t.is_suspicious);
    assert!(t.needs_review);
    assert!(t.suspicious_reason.contains("powershell"));
}

#[test]
fn trusted_powershell_is_not_flagged() {
    let t = analyzed("Microsoft Corporation", "powershell.exe", "-Command Update");
    assert!(!t.is_suspicious && !t.needs_review);
}

#[test]
fn user_task_without_author_running_powershell_is_to_review() {
    let t = analyzed("", "powershell.exe", "-NoProfile -File C:\\Users\\moi\\outil-perso\\run.ps1");
    assert!(!t.is_suspicious);
    assert!(t.needs_review);
}

#[test]
fn user_writable_path_by_unknown_author_is_to_review() {
    let t = analyzed("Unknown", "C:\\Users\\foo\\AppData\\Local\\tool.exe", "");
    assert!(!t.is_suspicious);
    assert!(t.needs_review);
}

#[test]
fn no_author_with_action_is_to_review() {
    let t = analyzed("", "C:\\Windows\\System32\\benign.exe", "");
    assert!(!t.is_suspicious);
    assert!(t.needs_review);
    assert!(t.suspicious_reason.contains("auteur"));
}

#[test]
fn encoded_command_is_suspicious_even_with_a_trusted_author() {
    let t = analyzed("Microsoft Corporation", "any.exe", "-enc SGVsbG8=");
    assert!(t.is_suspicious);
    assert!(!t.needs_review);
}

#[test]
fn strong_pattern_wins_over_the_powershell_review_level() {
    let t = analyzed("", "powershell.exe", "-NoProfile -EncodedCommand SGVsbG8=");
    assert!(t.is_suspicious, "un PowerShell encodé doit rester suspect : {}", t.suspicious_reason);
}

#[test]
fn download_and_proxy_binaries_are_suspicious() {
    for args in ["mshta http://x/y.hta", "certutil -urlcache -f http://x/a.exe a.exe", "bitsadmin /transfer j"] {
        assert!(analyzed("Unknown", "cmd", args).is_suspicious, "{}", args);
    }
}

#[test]
fn rundll32_from_a_trusted_author_is_ignored_and_from_unknown_is_to_review() {
    assert!(!analyzed("Microsoft Corporation", "rundll32.exe", "shell32.dll,Foo").needs_review);
    let t = analyzed("Contoso", "rundll32.exe", "foo.dll,Bar");
    assert!(!t.is_suspicious && t.needs_review);
}

#[test]
fn analyze_suspicious_ignores_encoding_flag() {
    // rust-02 : "-Encoding utf8" (parametre legitime) ne doit pas matcher le flag "-enc".
    let t = analyzed("Contoso Apps", "C:\\Program Files\\App\\app.exe", "-Encoding utf8 -Output log.txt");
    assert!(!t.is_suspicious && !t.needs_review, "-Encoding ne doit pas declencher -enc");
}

#[test]
fn analyze_suspicious_still_flags_enc_at_end() {
    // rust-02 : le flag "-enc" reste detecte meme en fin de chaine (frontiere = fin).
    let t = analyzed("Contoso Apps", "C:\\Program Files\\App\\app.exe", "-enc");
    assert!(t.is_suspicious);
}

#[test]
fn analysis_is_reset_between_runs() {
    let mut t = task("Contoso", "x.exe", "-enc AAAA");
    analyze_suspicious(&mut t);
    assert!(t.is_suspicious);
    t.action_arguments = String::new();
    analyze_suspicious(&mut t);
    assert!(!t.is_suspicious && !t.needs_review && t.suspicious_reason.is_empty());
}

#[test]
fn sanitize_task_name_accepts_paths_with_spaces() {
    assert_eq!(
        sanitize_task_name("\\Microsoft\\Windows\\Defrag\\Scheduled Defrag").unwrap(),
        "\\Microsoft\\Windows\\Defrag\\Scheduled Defrag"
    );
    assert_eq!(sanitize_task_name("  MyTask  ").unwrap(), "MyTask");
}

#[test]
fn sanitize_task_name_rejects_shell_metachars_and_unicode() {
    for bad in ["a\"b", "a;b", "a|b", "a&b", "a$b", "a`b", "a<b", "a>b", "tach\u{00e9}"] {
        assert!(sanitize_task_name(bad).is_err(), "doit rejeter: {}", bad);
    }
    assert!(sanitize_task_name("").is_err());
    assert!(sanitize_task_name("   ").is_err());
}

#[test]
fn suspicious_count_propagates_scan_failure() {
    let failure = CommandError::ProcessFailed {
        command: "Get-ScheduledTask".into(),
        detail: "acces refuse".into(),
    };
    assert!(suspicious_count(Err(failure)).is_err());
    assert_eq!(suspicious_count(Ok(Vec::new())).unwrap(), 0);
    assert_eq!(suspicious_count(Ok(vec![task("", "x.exe", ""), task("", "y.exe", "")])).unwrap(), 2);
}

#[test]
fn suspicious_severity_follows_the_pattern_in_the_reason() {
    assert_eq!(suspicious_severity(&analyzed("Vendor", "powershell.exe", "-enc AAAA")), "critical");
    assert_eq!(suspicious_severity(&analyzed("Vendor", "mshta.exe", "x")), "critical");
    assert_eq!(suspicious_severity(&analyzed("Vendor", "certutil.exe", "-urlcache")), "high");
    assert_eq!(suspicious_severity(&analyzed("Vendor", "wscript.exe", "a.vbs")), "medium");
}
