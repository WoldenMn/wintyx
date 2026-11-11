use super::*;
use crate::utils::process::CmdResult;

fn failed(exit_code: i32, stderr: &str) -> CmdResult {
    CmdResult { success: false, stdout: String::new(), stderr: stderr.into(), exit_code }
}

#[test]
fn command_stdout_returns_output_on_success() {
    let ok = CmdResult { success: true, stdout: "a|b".into(), stderr: String::new(), exit_code: 0 };
    assert_eq!(command_stdout("Tâches planifiées", &ok).unwrap(), "a|b");
}

#[test]
fn command_stdout_turns_a_failed_command_into_an_error_instead_of_zero_threats() {
    let timeout = failed(-1, "Process timed out");
    let err = command_stdout("Services non signés", &timeout).unwrap_err();
    assert!(err.contains("Services non signés"), "{}", err);
    assert!(err.contains("Process timed out"), "{}", err);
    assert!(command_stdout("netstat", &failed(1, "")).is_err());
}

fn powershell_lines(script: &str) -> Vec<String> {
    let result = process::run("powershell", &format!("-NoProfile -Command \"{}\"", script), constants::TIMEOUT_VERY_LONG);
    assert!(result.success, "PowerShell en echec: {}", result.stderr);
    result.stdout.lines().map(|l| l.trim().to_lowercase()).filter(|l| !l.is_empty()).collect()
}

#[test]
fn service_filter_excludes_windows_and_microsoft_paths_on_real_services() {
    let all = powershell_lines("Get-CimInstance Win32_Service | Where-Object { $_.PathName } | ForEach-Object { $_.PathName }");
    assert!(
        all.iter().any(|p| p.contains("\\windows\\")),
        "instrument invalide : aucun service sous Windows sur ce poste"
    );

    let filtered = powershell_lines(&format!(
        "Get-CimInstance Win32_Service | Where-Object {{ {} }} | ForEach-Object {{ $_.PathName }}",
        NON_MICROSOFT_SERVICE_FILTER
    ));
    assert!(!filtered.is_empty(), "le filtre ne doit pas tout exclure");
    for path in &filtered {
        assert!(!path.contains("\\windows\\"), "service Windows non exclu : {}", path);
        assert!(!path.contains("microsoft"), "service Microsoft non exclu : {}", path);
    }
    assert!(filtered.len() < all.len());
}

#[test]
fn the_service_script_has_no_sampling_cap() {
    let args = services_signature_args();
    assert!(!args.contains("Select-Object -First"), "{args}");
    assert!(args.contains(NON_MICROSOFT_SERVICE_FILTER));
}

#[test]
fn every_non_microsoft_service_of_this_machine_is_examined() {
    let expected = powershell_lines(&format!(
        "Get-CimInstance Win32_Service | Where-Object {{ {} }} | ForEach-Object {{ $_.Name }}",
        NON_MICROSOFT_SERVICE_FILTER
    ))
    .len();
    let (scanned, _) = scan_unsigned_services().expect("le scan doit aboutir");
    assert_eq!(scanned as usize, expected);
}
