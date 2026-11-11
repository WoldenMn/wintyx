use crate::utils::constants::TIMEOUT_LONG;
use crate::utils::errors::CommandError;
use crate::utils::process::{self, CmdResult};
use super::{pnputil_cmd, DriverIssue, IssueKind, Severity};

const DETECTION_SCRIPT: &str = "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
    Get-PnpDevice -PresentOnly:$false | \
    Where-Object { $_.Status -ne 'OK' -and $_.Present -eq $false } | \
    ForEach-Object { \\\"$($_.InstanceId)|$($_.FriendlyName)|$($_.Class)|$($_.Manufacturer)|$($_.Status)\\\" }";

fn detection_args() -> String {
    format!("-NoProfile -NonInteractive -Command \"{}\"", DETECTION_SCRIPT)
}

fn absent_reason(status: &str) -> String {
    match status.trim() {
        "" | "Unknown" => "Périphérique non présent".to_string(),
        "Error" => "Périphérique non présent (en erreur)".to_string(),
        "Degraded" => "Périphérique non présent (dégradé)".to_string(),
        other => format!("Périphérique non présent ({})", other),
    }
}

fn parse_phantoms(stdout: &str) -> Vec<DriverIssue> {
    stdout
        .lines()
        .filter_map(|line| {
            let fields = process::parse_pipe_fields(line, 5)?;
            let instance_id = fields[0].to_string();
            let name = fields[1].to_string();
            if instance_id.is_empty() || name.is_empty() {
                return None;
            }
            Some(DriverIssue {
                id: format!("phantom-{}", instance_id),
                kind: IssueKind::Phantom,
                severity: Severity::Medium,
                device_name: name,
                manufacturer: fields[3].to_string(),
                driver_version: String::new(),
                driver_date: String::new(),
                provider: fields[3].to_string(),
                inf_name: String::new(),
                instance_id: Some(instance_id),
                reason: absent_reason(fields[4]),
            })
        })
        .collect()
}

/// Peripheriques non presents dont le statut n'est pas OK. Un echec de la
/// detection est une erreur : « aucun fantome » ne doit jamais en etre le repli.
pub(super) fn detect_phantoms() -> Result<Vec<DriverIssue>, CommandError> {
    let output = process::run("powershell", &detection_args(), TIMEOUT_LONG * 3);
    if !output.success {
        let detail = failure_detail(&output);
        crate::utils::logger::warn(
            "drivers",
            &format!("Get-PnpDevice (fantômes) échec: {}", detail),
        );
        return Err(CommandError::ProcessFailed {
            command: "Get-PnpDevice".into(),
            detail,
        });
    }
    Ok(parse_phantoms(&output.stdout))
}

pub(super) fn failure_detail(output: &CmdResult) -> String {
    if output.stderr.trim().is_empty() {
        format!("Code de sortie {}", output.exit_code)
    } else {
        output.stderr.trim().to_string()
    }
}

/// Supprime un peripherique par son Instance ID, deja valide par l'appelant.
/// `Remove-PnpDevice` n'existe pas : le module PnpDevice n'offre que Get, Enable et Disable.
pub(super) fn remove_phantom(instance_id: &str) -> CmdResult {
    let mut output = process::run("pnputil", &pnputil_cmd::remove_device_args(instance_id), TIMEOUT_LONG * 2);
    if pnputil_cmd::reboot_required(output.exit_code) {
        crate::utils::logger::info(
            "drivers",
            &format!("{} supprimé, redémarrage requis pour finaliser", instance_id),
        );
    }
    output.success = pnputil_cmd::succeeded(output.exit_code);
    output
}

#[cfg(test)]
mod tests {
    use super::super::validation::validate_instance_id;
    use super::*;

    const SAMPLE: &str = include_str!("fixtures/get_pnpdevice_phantoms.txt");

    #[test]
    fn detection_script_has_no_unescaped_double_quote() {
        let mut prev = ' ';
        for c in DETECTION_SCRIPT.chars() {
            assert!(c != '"' || prev == '\\', "guillemet non echappe dans le script");
            prev = c;
        }
    }

    #[test]
    fn parse_phantoms_reads_real_sample() {
        let issues = parse_phantoms(SAMPLE);
        assert_eq!(issues.len(), 9);
        assert_eq!(issues[0].device_name, "Téléphone USB");
        assert_eq!(issues[0].provider, "Fabricant A");
        assert_eq!(issues[0].instance_id.as_deref(), Some("USB\\VID_1234&PID_0001\\0123456789ABCDEF"));
        assert!(issues[4].provider.is_empty());
        assert_eq!(issues[1].reason, "Périphérique non présent");
        assert_eq!(absent_reason("Error"), "Périphérique non présent (en erreur)");
    }

    #[test]
    fn parse_phantoms_sample_ids_pass_validation() {
        for issue in parse_phantoms(SAMPLE) {
            let id = issue.instance_id.unwrap();
            assert!(validate_instance_id(&id).is_ok(), "refuse: {}", id);
        }
    }

    #[test]
    fn parse_phantoms_skips_malformed_lines() {
        assert!(parse_phantoms("pas de separateur\n\n|sans id|a|b|c").is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn detect_phantoms_succeeds_on_this_machine_without_mojibake() {
        let issues = detect_phantoms().expect("la detection doit aboutir");
        for issue in &issues {
            assert!(!issue.device_name.contains('\u{FFFD}'), "encodage: {}", issue.device_name);
            assert!(issue.instance_id.as_deref().is_some_and(|i| i.contains('\\')));
        }
    }
}
