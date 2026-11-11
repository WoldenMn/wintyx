use crate::utils::errors::CommandError;
use crate::utils::wmi_helpers::with_wmi_conn;
use crate::utils::process;
use crate::utils::constants::TIMEOUT_LONG;
use crate::utils::admin::require_admin;
use crate::utils::batch::{BatchOptions, BatchResult};

mod audit;
mod phantoms;
mod pnputil_cmd;
mod pnputil_parser;
mod sources;
mod types;
mod validation;

use audit::classify_all;
use phantoms::{detect_phantoms, failure_detail, remove_phantom};
use sources::{collect_drivers, collect_problem_devices, load_pnputil_drivers};
use validation::{ensure_detected_phantom, ensure_flagged_package, validate_inf_name, validate_instance_id};

pub use types::{DriverAuditSummary, DriverInfo, DriverIssue, DriverSummary, IssueKind, ProblemDevice, Severity};

#[tauri::command]
#[specta::specta]
pub async fn get_driver_audit() -> Result<Vec<DriverIssue>, CommandError> {
    crate::utils::blocking::read_result(get_driver_audit_sync).await
}

pub fn get_driver_audit_sync() -> Result<Vec<DriverIssue>, CommandError> {
    crate::perf_timed!("get_driver_audit", {
        let drivers = with_wmi_conn(collect_drivers)?;
        let parsed_pn = load_pnputil_drivers()?;
        let phantoms = detect_phantoms()?;
        Ok(classify_all(&drivers, parsed_pn.as_deref(), phantoms))
    })
}

fn flagged_package_issues() -> Result<Vec<DriverIssue>, CommandError> {
    let drivers = with_wmi_conn(collect_drivers)?;
    let parsed_pn = load_pnputil_drivers()?;
    Ok(classify_all(&drivers, parsed_pn.as_deref(), Vec::new()))
}

#[tauri::command]
#[specta::specta]
pub async fn uninstall_driver(
    inf_name: String,
    options: Option<BatchOptions>,
) -> Result<BatchResult, CommandError> {
    crate::utils::blocking::write_result(move || uninstall_driver_sync(inf_name, options)).await
}

pub fn uninstall_driver_sync(
    inf_name: String,
    options: Option<BatchOptions>,
) -> Result<BatchResult, CommandError> {
    let dry_run = options.as_ref().map(|o| o.dry_run).unwrap_or(false);
    let inf = validate_inf_name(&inf_name)?;

    if !dry_run {
        require_admin("Désinstallation de pilote")?;
    }
    ensure_flagged_package(inf, &flagged_package_issues()?)?;

    if dry_run {
        let mut result = BatchResult::new_dry_run();
        result.record_preview(format!("Désinstaller le pilote {}", inf));
        return Ok(result);
    }

    let output = process::run("pnputil", &pnputil_cmd::delete_driver_args(inf), TIMEOUT_LONG * 3);

    let mut result = BatchResult::new();
    if pnputil_cmd::succeeded(output.exit_code) {
        result.record_success();
    } else {
        let detail = if !output.stderr.is_empty() {
            output.stderr.clone()
        } else {
            format!("Code de sortie {}", output.exit_code)
        };
        result.record_failure(format!("Échec désinstallation {}: {}", inf, detail));
    }
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn remove_phantom_device(
    instance_id: String,
    options: Option<BatchOptions>,
) -> Result<BatchResult, CommandError> {
    crate::utils::blocking::write_result(move || remove_phantom_device_sync(instance_id, options)).await
}

pub fn remove_phantom_device_sync(
    instance_id: String,
    options: Option<BatchOptions>,
) -> Result<BatchResult, CommandError> {
    let dry_run = options.as_ref().map(|o| o.dry_run).unwrap_or(false);
    let id = validate_instance_id(&instance_id)?;

    if !dry_run {
        require_admin("Suppression de périphérique fantome")?;
    }
    ensure_detected_phantom(id, &detect_phantoms()?)?;

    if dry_run {
        let mut result = BatchResult::new_dry_run();
        result.record_preview(format!("Supprimer le périphérique fantome {}", id));
        return Ok(result);
    }

    let output = remove_phantom(id);

    let mut result = BatchResult::new();
    if output.success {
        result.record_success();
    } else {
        result.record_failure(format!("Échec suppression {}: {}", id, failure_detail(&output)));
    }
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn remove_all_phantom_devices(
    options: Option<BatchOptions>,
) -> Result<BatchResult, CommandError> {
    crate::utils::blocking::write_result(move || remove_all_phantom_devices_sync(options)).await
}

pub fn remove_all_phantom_devices_sync(
    options: Option<BatchOptions>,
) -> Result<BatchResult, CommandError> {
    let dry_run = options.as_ref().map(|o| o.dry_run).unwrap_or(false);
    let phantoms = detect_phantoms()?;

    if dry_run {
        let mut result = BatchResult::new_dry_run();
        for issue in &phantoms {
            result.record_preview(format!("Supprimer {}", issue.device_name));
        }
        return Ok(result);
    }

    if phantoms.is_empty() {
        return Ok(BatchResult::new());
    }

    require_admin("Suppression des périphériques fantômes")?;

    let mut result = BatchResult::new();
    for issue in phantoms {
        let Some(instance_id) = issue.instance_id.as_deref() else {
            continue;
        };
        let Ok(id) = validate_instance_id(instance_id) else {
            result.record_failure(format!("Instance ID invalide ignoree pour {}", issue.device_name));
            continue;
        };
        let output = remove_phantom(id);
        if output.success {
            result.record_success();
        } else {
            result.record_failure(format!("Échec suppression {}: {}", issue.device_name, failure_detail(&output)));
        }
    }
    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn get_driver_audit_summary() -> Result<DriverAuditSummary, CommandError> {
    crate::utils::blocking::read_result(get_driver_audit_summary_sync).await
}

pub fn get_driver_audit_summary_sync() -> Result<DriverAuditSummary, CommandError> {
    crate::perf_timed!("get_driver_audit_summary", {
        // Appels non-WMI hors de la connexion partagee.
        let parsed_pn = load_pnputil_drivers()?;
        let phantoms = detect_phantoms()?;

        // Une seule init COM/WMI partagee pour les deux requetes (perf-rust-01),
        // comme get_driver_summary, au lieu de deux with_wmi_conn separes.
        let (drivers, problems) = with_wmi_conn(|wmi| {
            let drivers = collect_drivers(wmi)?;
            let problems = collect_problem_devices(wmi).inspect_err(|e| {
                crate::utils::logger::warn(
                    "drivers",
                    &format!("collect_problem_devices échec (audit summary): {}", e),
                );
            })?;
            Ok((drivers, problems))
        })?;

        let issues = classify_all(&drivers, parsed_pn.as_deref(), phantoms);

        let unsigned = issues.iter().filter(|i| matches!(i.kind, IssueKind::Unsigned)).count() as u32;
        let duplicates = issues.iter().filter(|i| matches!(i.kind, IssueKind::Duplicate)).count() as u32;
        let phantoms_count = issues.iter().filter(|i| matches!(i.kind, IssueKind::Phantom)).count() as u32;
        let obsolete = issues.iter().filter(|i| matches!(i.kind, IssueKind::Obsolete)).count() as u32;
        let third_party = issues.iter().filter(|i| matches!(i.kind, IssueKind::ThirdPartyOem)).count() as u32;

        Ok(DriverAuditSummary {
            total: drivers.len() as u32,
            unsigned,
            duplicates,
            phantoms: phantoms_count,
            obsolete,
            third_party,
            problems: problems.len() as u32,
        })
    })
}

pub fn get_driver_summary_sync() -> Result<DriverSummary, CommandError> {
    with_wmi_conn(|wmi| {
        let drivers = collect_drivers(wmi)?;
        let unsigned = drivers.iter().filter(|d| !d.is_signed).count();
        let problems = collect_problem_devices(wmi).inspect_err(|e| {
            crate::utils::logger::warn(
                "drivers",
                &format!("collect_problem_devices échec (driver summary): {}", e),
            );
        })?;

        Ok(DriverSummary {
            total: drivers.len() as u32,
            unsigned: unsigned as u32,
            problems: problems.len() as u32,
        })
    })
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    fn dry_run() -> Option<BatchOptions> {
        Some(BatchOptions { dry_run: true })
    }

    #[test]
    fn removing_a_phantom_that_the_detection_does_not_list_is_refused() {
        let err = remove_phantom_device_sync("ROOT\\NOSUCHBUS\\0000".into(), dry_run()).unwrap_err();
        assert!(matches!(err, CommandError::InvalidInput(_)), "{err:?}");
    }

    #[test]
    fn both_summaries_read_this_machine_including_its_problem_devices() {
        let summary = get_driver_summary_sync().expect("resume lisible");
        assert!(summary.total > 0);
        let audit = get_driver_audit_summary_sync().expect("audit lisible");
        assert!(audit.total > 0);
        assert_eq!(audit.problems, summary.problems);
    }

    #[test]
    fn uninstalling_an_oem_package_the_audit_does_not_list_is_refused() {
        let err = uninstall_driver_sync("oem9999.inf".into(), dry_run()).unwrap_err();
        assert!(matches!(err, CommandError::InvalidInput(_)), "{err:?}");
    }
}
