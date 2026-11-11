use crate::utils::{admin, constants, process, registry, temp_file};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CheckState {
    Secure,
    Insecure,
    Unknown,
}

pub(super) const GUEST_QUERY_ARGS: &str = "-NoProfile -Command \"(Get-LocalUser | Where-Object { $_.SID.Value -like '*-501' }).Enabled\"";
pub(super) const GUEST_DISABLE_COMMAND: &str = "powershell.exe -NoProfile -Command \"Get-LocalUser | Where-Object { $_.SID.Value -like '*-501' } | Disable-LocalUser\"";

const ERROR_SERVICE_DOES_NOT_EXIST: i32 = 1060;
const LOCKOUT_MAX_SECURE_THRESHOLD: u32 = 10;
const LSA_PATH: &str = r"HKLM\SYSTEM\CurrentControlSet\Control\Lsa";
const DEVICE_GUARD_PATH: &str = r"HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard";

pub(super) fn guest_state(success: bool, stdout: &str) -> CheckState {
    if !success {
        return CheckState::Unknown;
    }
    match stdout.trim().to_ascii_lowercase().as_str() {
        "false" => CheckState::Secure,
        "true" => CheckState::Insecure,
        _ => CheckState::Unknown,
    }
}

pub(super) fn spooler_state(success: bool, exit_code: i32, stdout: &str) -> CheckState {
    if exit_code == ERROR_SERVICE_DOES_NOT_EXIST {
        return CheckState::Secure;
    }
    if !success {
        return CheckState::Unknown;
    }
    let upper = stdout.to_uppercase();
    if upper.contains("RUNNING") {
        CheckState::Insecure
    } else if upper.contains("STOPPED") {
        CheckState::Secure
    } else {
        CheckState::Unknown
    }
}

pub(super) fn credential_guard_state(vbs: Option<u32>, lsa_cfg_flags: Option<u32>) -> CheckState {
    if vbs == Some(1) && matches!(lsa_cfg_flags, Some(1) | Some(2)) {
        CheckState::Secure
    } else {
        CheckState::Insecure
    }
}

pub(super) fn decode_secedit_inf(bytes: &[u8]) -> String {
    match bytes {
        [0xFF, 0xFE, rest @ ..] => {
            let units: Vec<u16> = rest.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect();
            String::from_utf16_lossy(&units)
        }
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

pub(super) fn lockout_state_from_inf(inf: &str) -> CheckState {
    let value = inf.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == "LockoutBadCount").then(|| value.trim().parse::<u32>().ok())
    });
    match value {
        Some(Some(n)) if (1..=LOCKOUT_MAX_SECURE_THRESHOLD).contains(&n) => CheckState::Secure,
        Some(Some(_)) => CheckState::Insecure,
        _ => CheckState::Unknown,
    }
}

pub(super) fn wdigest_state(read: Result<Option<u32>, String>) -> CheckState {
    match read {
        Ok(None) | Ok(Some(0)) => CheckState::Secure,
        Ok(Some(_)) => CheckState::Insecure,
        Err(_) => CheckState::Unknown,
    }
}

pub(super) fn wdigest_check_state(path: &str, value: &str) -> CheckState {
    let read = registry::read_dword(path, value);
    if let Err(e) = &read {
        crate::utils::logger::warn("hardening.lecture", &format!("WDigest illisible: {}", e));
    }
    wdigest_state(read)
}

pub(super) fn score_from_states(states: &[CheckState]) -> u32 {
    let known = states.iter().filter(|s| **s != CheckState::Unknown).count();
    let secure = states.iter().filter(|s| **s == CheckState::Secure).count();
    super::ratio_score(secure, known)
}

pub(super) fn guest_account_state() -> CheckState {
    let result = process::run("powershell.exe", GUEST_QUERY_ARGS, constants::TIMEOUT_SHORT);
    let state = guest_state(result.success, &result.stdout);
    if state == CheckState::Unknown {
        crate::utils::logger::warn(
            "hardening.lecture",
            &format!("compte Invite (SID -501) illisible (exit {}): {}", result.exit_code, result.stderr.trim()),
        );
    }
    state
}

pub(super) fn spooler_service_state() -> CheckState {
    let result = process::run("sc.exe", "query Spooler", constants::TIMEOUT_SHORT);
    let state = spooler_state(result.success, result.exit_code, &result.stdout);
    if state == CheckState::Unknown {
        crate::utils::logger::warn(
            "hardening.lecture",
            &format!("état du Spooler illisible (exit {}): {}", result.exit_code, result.stderr.trim()),
        );
    }
    state
}

pub(super) fn credential_guard_check_state() -> CheckState {
    credential_guard_state(
        registry::get_dword(DEVICE_GUARD_PATH, "EnableVirtualizationBasedSecurity"),
        registry::get_dword(LSA_PATH, "LsaCfgFlags"),
    )
}

pub(super) fn account_lockout_state() -> CheckState {
    if !admin::is_admin() {
        return CheckState::Unknown;
    }
    let path = temp_file::unpredictable_temp_path("wintyx-secpol", "inf");
    let args = format!("/export /cfg \"{}\" /areas SECURITYPOLICY", path.display());
    let result = process::run("secedit.exe", &args, constants::TIMEOUT_MEDIUM);
    let state = if result.success {
        match std::fs::read(&path) {
            Ok(bytes) => lockout_state_from_inf(&decode_secedit_inf(&bytes)),
            Err(e) => {
                crate::utils::logger::warn("hardening.lecture", &format!("export secedit illisible: {}", e));
                CheckState::Unknown
            }
        }
    } else {
        crate::utils::logger::warn(
            "hardening.lecture",
            &format!("secedit /export a échoué (exit {}): {}", result.exit_code, result.stdout.trim()),
        );
        CheckState::Unknown
    };
    let _ = std::fs::remove_file(&path);
    state
}
