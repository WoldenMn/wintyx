use super::{DriverIssue, IssueKind};
use crate::utils::errors::CommandError;

/// Strict validator for `oem##.inf` (used by `uninstall_driver` only).
/// Rejects: path traversal, command injection, non-oem INFs.
pub(super) fn validate_inf_name(inf: &str) -> Result<&str, CommandError> {
    if inf.len() < 5 || inf.len() > 11 {
        return Err(CommandError::InvalidInput(
            format!("Nom INF invalide ({} caracteres)", inf.len()),
        ));
    }
    if !inf.starts_with("oem") || !inf.ends_with(".inf") {
        return Err(CommandError::InvalidInput(
            "Le nom doit suivre le format oem<digits>.inf".into(),
        ));
    }
    let digits = &inf[3..inf.len() - 4];
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(CommandError::InvalidInput(
            "Le nom doit suivre le format oem<digits>.inf".into(),
        ));
    }
    Ok(inf)
}

/// Strict validator for PNP Instance IDs.
/// Allowed: A-Z, a-z, 0-9, `_`, `&`, `-`, `.`, `{`, `}`, `#`, `:`, `?`, `\` (path separator).
/// Les identifiants reels portent des GUID entre accolades (`SWD\MMDEVAPI\{...}`), des `#`
/// (`SWD\DRIVERENUM\{...}#...`), des `:` (adresses MAC) et des `?` (`STORAGE\VOLUME\_??_USBSTOR#...`).
/// L'identifiant est passe a pnputil comme argument, sans shell ni joker.
/// Must contain at least 2 backslashes (BUS\HARDWARE\INSTANCE format).
pub(super) fn validate_instance_id(id: &str) -> Result<&str, CommandError> {
    if id.is_empty() || id.len() > 200 {
        return Err(CommandError::InvalidInput(
            "Instance ID vide ou trop long".into(),
        ));
    }
    let backslash_count = id.bytes().filter(|&b| b == b'\\').count();
    if backslash_count < 2 {
        return Err(CommandError::InvalidInput(
            "Instance ID malformee (doit contenir au moins 2 backslashes)".into(),
        ));
    }
    let valid_char = |c: char| {
        c.is_ascii_alphanumeric()
            || c == '_'
            || c == '&'
            || c == '-'
            || c == '.'
            || c == '{'
            || c == '}'
            || c == '#'
            || c == ':'
            || c == '?'
            || c == '\\'
    };
    if !id.chars().all(valid_char) {
        return Err(CommandError::InvalidInput(
            "Instance ID contient des caractères non autorisés".into(),
        ));
    }
    Ok(id)
}

pub(super) fn ensure_flagged_package(inf: &str, issues: &[DriverIssue]) -> Result<(), CommandError> {
    let flagged = issues
        .iter()
        .any(|i| !matches!(i.kind, IssueKind::Phantom) && i.inf_name.eq_ignore_ascii_case(inf));
    if flagged {
        Ok(())
    } else {
        Err(CommandError::InvalidInput(format!(
            "{} ne fait pas partie des pilotes signalés par l'audit : désinstallation refusée",
            inf
        )))
    }
}

pub(super) fn ensure_detected_phantom(id: &str, phantoms: &[DriverIssue]) -> Result<(), CommandError> {
    let detected = phantoms.iter().any(|i| {
        matches!(i.kind, IssueKind::Phantom)
            && i.instance_id.as_deref().is_some_and(|known| known.eq_ignore_ascii_case(id))
    });
    if detected {
        Ok(())
    } else {
        Err(CommandError::InvalidInput(format!(
            "{} n'est pas un périphérique fantôme détecté : suppression refusée",
            id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::super::Severity;
    use super::*;

    fn issue(kind: IssueKind, inf_name: &str, instance_id: Option<&str>) -> DriverIssue {
        DriverIssue {
            id: "x".into(),
            kind,
            severity: Severity::Low,
            device_name: "Peripherique".into(),
            manufacturer: String::new(),
            driver_version: String::new(),
            driver_date: String::new(),
            provider: String::new(),
            inf_name: inf_name.into(),
            instance_id: instance_id.map(str::to_string),
            reason: String::new(),
        }
    }

    #[test]
    fn flagged_package_accepts_every_non_phantom_kind_listed_by_the_audit() {
        for kind in [IssueKind::Duplicate, IssueKind::Obsolete, IssueKind::ThirdPartyOem, IssueKind::Unsigned] {
            let issues = [issue(kind, "oem12.inf", None)];
            assert!(ensure_flagged_package("oem12.inf", &issues).is_ok());
        }
    }

    #[test]
    fn flagged_package_refuses_a_valid_oem_name_the_audit_never_listed() {
        let issues = [issue(IssueKind::Duplicate, "oem12.inf", None)];
        let err = ensure_flagged_package("oem13.inf", &issues).unwrap_err();
        assert!(matches!(err, CommandError::InvalidInput(_)), "{err:?}");
        assert!(ensure_flagged_package("oem13.inf", &[]).is_err());
    }

    #[test]
    fn flagged_package_ignores_a_phantom_carrying_that_inf_name() {
        let issues = [issue(IssueKind::Phantom, "oem12.inf", Some("USB\\A\\B"))];
        assert!(ensure_flagged_package("oem12.inf", &issues).is_err());
    }

    #[test]
    fn detected_phantom_matches_the_exact_instance_regardless_of_case() {
        let id = "USB\\VID_1234&PID_0001\\0123456789ABCDEF";
        let phantoms = [issue(IssueKind::Phantom, "", Some(id))];
        assert!(ensure_detected_phantom(id, &phantoms).is_ok());
        assert!(ensure_detected_phantom(&id.to_lowercase(), &phantoms).is_ok());
    }

    #[test]
    fn detected_phantom_refuses_a_well_formed_id_that_was_not_detected() {
        let phantoms = [issue(IssueKind::Phantom, "", Some("USB\\A\\B"))];
        let err = ensure_detected_phantom("PCI\\VEN_8086&DEV_1234\\3&11583659&0&E0", &phantoms).unwrap_err();
        assert!(matches!(err, CommandError::InvalidInput(_)), "{err:?}");
        assert!(ensure_detected_phantom("USB\\A\\B", &[]).is_err());
    }

    #[test]
    fn detected_phantom_does_not_accept_a_present_device_of_another_kind() {
        let issues = [issue(IssueKind::Unsigned, "oem1.inf", Some("USB\\A\\B"))];
        assert!(ensure_detected_phantom("USB\\A\\B", &issues).is_err());
    }

    #[test]
    fn validate_inf_name_oem_valid() {
        assert!(validate_inf_name("oem42.inf").is_ok());
        assert!(validate_inf_name("oem1.inf").is_ok());
        assert!(validate_inf_name("oem9999.inf").is_ok());
    }

    #[test]
    fn validate_inf_name_path_rejected() {
        assert!(validate_inf_name("../oem1.inf").is_err());
        assert!(validate_inf_name("C:\\oem1.inf").is_err());
        assert!(validate_inf_name("subdir/oem1.inf").is_err());
    }

    #[test]
    fn validate_inf_name_injection_rejected() {
        assert!(validate_inf_name("oem1.inf; rm -rf /").is_err());
        assert!(validate_inf_name("oem1.inf && calc.exe").is_err());
        assert!(validate_inf_name("oem1.inf`whoami`").is_err());
        assert!(validate_inf_name("oem1.inf|cat").is_err());
    }

    #[test]
    fn validate_inf_name_non_oem_rejected() {
        assert!(validate_inf_name("usb.inf").is_err());
        assert!(validate_inf_name("microsoft.inf").is_err());
        assert!(validate_inf_name("oem.inf").is_err());
        assert!(validate_inf_name("oem10000.inf").is_err());
    }

    #[test]
    fn validate_instance_id_pnp_valid() {
        assert!(validate_instance_id("USB\\VID_046D&PID_C52B\\6&abc&0&2").is_ok());
        assert!(validate_instance_id("PCI\\VEN_8086&DEV_1234\\3&11583659&0&E0").is_ok());
        assert!(validate_instance_id("HID\\VID_045E&PID_07A5\\7&234&0").is_ok());
    }

    #[test]
    fn validate_instance_id_accepts_braced_guid() {
        assert!(validate_instance_id("SWD\\MMDEVAPI\\{0.0.0.00000000}.{8c8b0a1e-6b1d-4a3e-9f0e-2d8c8f1a7b11}").is_ok());
        assert!(validate_instance_id("ROOT\\SYSTEM\\{4d36e97d-e325-11ce-bfc1-08002be10318}").is_ok());
    }

    #[test]
    fn validate_instance_id_accepts_storage_volume_question_marks() {
        assert!(validate_instance_id("STORAGE\\VOLUME\\_??_USBSTOR#Disk&Ven_X#0001#{53f56307-b6bf-11d0-94f2-00a0c91efb8b}").is_ok());
        assert!(validate_instance_id("SWD\\WPDBUSENUM\\_??_USBSTOR#Disk&Ven_X#0001#{53f56307-b6bf-11d0-94f2-00a0c91efb8b}").is_ok());
    }

    #[test]
    fn validate_instance_id_braces_do_not_open_injection() {
        assert!(validate_instance_id("SWD\\X\\{a}; Remove-PnpDevice Z").is_err());
        assert!(validate_instance_id("SWD\\X\\{a}$(calc)").is_err());
        assert!(validate_instance_id("SWD\\X\\{a}`whoami`").is_err());
        assert!(validate_instance_id("SWD\\X\\{a}|cat").is_err());
        assert!(validate_instance_id("SWD\\X\\{a}' ; calc ; '").is_err());
        assert!(validate_instance_id("SWD\\X\\{a}\"").is_err());
        assert!(validate_instance_id("SWD\\X\\{a} b").is_err());
    }

    #[test]
    fn validate_instance_id_injection_rejected() {
        assert!(validate_instance_id("USB\\X\\Y; Remove-PnpDevice Z").is_err());
        assert!(validate_instance_id("USB\\X`whoami`").is_err());
        assert!(validate_instance_id("USB\\X|cat /etc/passwd").is_err());
        assert!(validate_instance_id("USB\\X$(calc)").is_err());
        assert!(validate_instance_id("USB\\X 'quote'").is_err());
    }
}
