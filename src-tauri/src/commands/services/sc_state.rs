use crate::utils::{constants, logger, process, registry};

pub(super) const SC_ERROR_SERVICE_DOES_NOT_EXIST: i32 = 1060;
pub(super) const SC_ERROR_SERVICE_NOT_ACTIVE: i32 = 1062;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum StopOutcome {
    Stopped,
    AlreadyStopped,
    Absent,
    Failed,
}

pub(super) fn sc_reported(exit_code: i32, stdout: &str, code: i32) -> bool {
    exit_code == code || stdout.contains(&code.to_string())
}

pub(super) fn classify_stop(success: bool, exit_code: i32, stdout: &str) -> StopOutcome {
    if success {
        StopOutcome::Stopped
    } else if sc_reported(exit_code, stdout, SC_ERROR_SERVICE_NOT_ACTIVE) {
        StopOutcome::AlreadyStopped
    } else if sc_reported(exit_code, stdout, SC_ERROR_SERVICE_DOES_NOT_EXIST) {
        StopOutcome::Absent
    } else {
        StopOutcome::Failed
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ServiceState {
    Running,
    Stopped,
    Absent,
    Unknown,
}

impl ServiceState {
    pub(super) fn label(&self) -> &'static str {
        match self {
            ServiceState::Running => "Running",
            ServiceState::Stopped => "Stopped",
            ServiceState::Absent => "Absent",
            ServiceState::Unknown => "Unknown",
        }
    }
}

pub(super) fn classify_query(success: bool, exit_code: i32, stdout: &str) -> ServiceState {
    if !success {
        let absent = sc_reported(exit_code, stdout, SC_ERROR_SERVICE_DOES_NOT_EXIST);
        return if absent { ServiceState::Absent } else { ServiceState::Unknown };
    }
    let upper = stdout.to_uppercase();
    if upper.contains("RUNNING") {
        ServiceState::Running
    } else if upper.contains("STOPPED") {
        ServiceState::Stopped
    } else {
        ServiceState::Unknown
    }
}

pub(super) fn query_service_state(name: &str) -> ServiceState {
    let result = process::run("sc.exe", &format!("query {}", name), constants::TIMEOUT_SHORT);
    let state = classify_query(result.success, result.exit_code, &result.stdout);
    if state == ServiceState::Unknown {
        logger::warn(
            "services.lecture",
            &format!("query_service_state {}: sc.exe query inexploitable (exit {}): {}", name, result.exit_code, result.stderr),
        );
    }
    state
}

pub(super) fn query_service_status(name: &str) -> (String, String) {
    let state = query_service_state(name);
    if matches!(state, ServiceState::Unknown | ServiceState::Absent) {
        return (state.label().to_string(), "--".to_string());
    }

    let reg_path = format!(r"{}\{}", constants::REG_SERVICES, name);
    let start_type = match registry::get_dword(&reg_path, "Start") {
        Some(2) => "Automatique",
        Some(3) => "Manuel",
        Some(4) => "Désactivé",
        _ => "--",
    };

    (state.label().to_string(), start_type.to_string())
}
