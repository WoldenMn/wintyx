use std::collections::HashMap;
use wmi::{Variant, WMIConnection};
use crate::utils::errors::CommandError;
use crate::utils::wmi_helpers::{v_str, v_bool, v_i32};
use crate::utils::process;
use crate::utils::constants::TIMEOUT_LONG;
use super::pnputil_parser::{parse_pnputil_csv, PnputilDriver};
use super::{DriverInfo, ProblemDevice};

fn wmi_err(e: impl std::fmt::Display) -> CommandError {
    CommandError::WmiUnavailable(e.to_string())
}

pub(super) fn collect_drivers(wmi: &WMIConnection) -> Result<Vec<DriverInfo>, CommandError> {
    let results: Vec<HashMap<String, Variant>> = wmi
        .raw_query("SELECT DeviceName, DriverVersion, DriverDate, Manufacturer, DriverProviderName, DeviceClass, IsSigned, Signer, Status, InfName FROM Win32_PnPSignedDriver WHERE DeviceName IS NOT NULL")
        .map_err(wmi_err)?;

    let mut drivers: Vec<DriverInfo> = results
        .iter()
        .filter_map(|row| {
            let device_name = v_str(row.get("DeviceName"));
            if device_name.is_empty() {
                return None;
            }

            let driver_date = v_str(row.get("DriverDate"));
            let formatted_date = if driver_date.len() >= 8 && driver_date.is_ascii() {
                format!("{}-{}-{}", &driver_date[0..4], &driver_date[4..6], &driver_date[6..8])
            } else {
                driver_date
            };

            Some(DriverInfo {
                device_name,
                driver_version: v_str(row.get("DriverVersion")),
                driver_date: formatted_date,
                manufacturer: v_str(row.get("Manufacturer")),
                driver_provider: v_str(row.get("DriverProviderName")),
                device_class: v_str(row.get("DeviceClass")),
                is_signed: v_bool(row.get("IsSigned")),
                signer: v_str(row.get("Signer")),
                status: {
                    let s = v_str(row.get("Status"));
                    if s.is_empty() { "OK".to_string() } else { s }
                },
                inf_name: v_str(row.get("InfName")),
            })
        })
        .collect();

    drivers.sort_by(|a, b| {
        a.device_class.cmp(&b.device_class).then(a.device_name.cmp(&b.device_name))
    });

    Ok(drivers)
}

pub(super) fn collect_problem_devices(wmi: &WMIConnection) -> Result<Vec<ProblemDevice>, CommandError> {
    let results: Vec<HashMap<String, Variant>> = wmi
        .raw_query("SELECT Name, Manufacturer, PNPClass, ConfigManagerErrorCode FROM Win32_PnPEntity WHERE ConfigManagerErrorCode <> 0")
        .map_err(wmi_err)?;

    let problems = results
        .iter()
        .filter_map(|row| {
            let name = v_str(row.get("Name"));
            if name.is_empty() {
                return None;
            }

            let error_code = v_i32(row.get("ConfigManagerErrorCode"));

            let status = match error_code {
                1 => "Non configuré".to_string(),
                3 => "Pilote corrompu".to_string(),
                10 => "Démarrage impossible".to_string(),
                12 => "Conflit de ressources".to_string(),
                14 => "Redémarrage requis".to_string(),
                22 => "Désactivé".to_string(),
                28 => "Pilote non installé".to_string(),
                31 => "Fonctionnement incorrect".to_string(),
                _ => format!("Erreur ({})", error_code),
            };

            Some(ProblemDevice {
                device_name: name,
                manufacturer: v_str(row.get("Manufacturer")),
                device_class: v_str(row.get("PNPClass")),
                status,
            })
        })
        .collect();

    Ok(problems)
}

const PNPUTIL_ENUM_ARGS: &str = "/enum-drivers /format csv";

/// Liste les paquets du DriverStore par `pnputil /enum-drivers /format csv`.
/// `Ok(None)` si pnputil est absent ou echoue (audit degrade, trace au journal) ;
/// `Err` si sa sortie n'est pas le CSV attendu, pour ne pas decider sur du bruit.
pub(super) fn load_pnputil_drivers() -> Result<Option<Vec<PnputilDriver>>, CommandError> {
    let output = process::run("pnputil", PNPUTIL_ENUM_ARGS, TIMEOUT_LONG);
    if !output.success {
        crate::utils::logger::warn(
            "drivers",
            &format!("pnputil /enum-drivers échec (exit {}): {} - audit pilotes dégradé (doublons/OEM non détectés)", output.exit_code, output.stderr),
        );
        return Ok(None);
    }
    parse_pnputil_csv(&output.stdout)
        .map(Some)
        .map_err(|detail| {
            crate::utils::logger::warn("drivers", &format!("pnputil /enum-drivers illisible: {}", detail));
            CommandError::ProcessFailed {
                command: "pnputil /enum-drivers".into(),
                detail,
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pnputil_enum_uses_space_separated_format_flag() {
        assert_eq!(PNPUTIL_ENUM_ARGS, "/enum-drivers /format csv");
    }

    #[cfg(windows)]
    #[test]
    fn load_pnputil_drivers_parses_this_machine_driver_store() {
        let drivers = load_pnputil_drivers().expect("sortie lisible").expect("pnputil present");
        assert!(!drivers.is_empty());
        assert!(drivers.iter().all(|d| d.oem_inf.starts_with("oem") && d.oem_inf.ends_with(".inf")));
        assert!(drivers.iter().all(|d| d.version.chars().next().is_some_and(|c| c.is_ascii_digit())));
        assert!(drivers.iter().all(|d| d.date.len() == 10 && d.date.as_bytes()[4] == b'-'));
    }
}
