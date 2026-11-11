//! Scanners dont la source est une commande externe via `process::run`
//! (services non signes).

use crate::utils::process::CmdResult;
use crate::utils::{constants, process};

use crate::commands::threat_scanner::{ScanOutcome, ThreatItem};

const NON_MICROSOFT_SERVICE_FILTER: &str = "$_.PathName -and $_.PathName -notlike '*\\Windows\\*' -and $_.PathName -notlike '*Microsoft*'";

pub(super) fn command_stdout<'a>(label: &str, result: &'a CmdResult) -> Result<&'a str, String> {
    if result.success {
        Ok(&result.stdout)
    } else {
        Err(format!(
            "{}: commande en échec (exit {}): {}",
            label,
            result.exit_code,
            result.stderr.trim()
        ))
    }
}

fn services_signature_args() -> String {
    format!("-NoProfile -Command \"Get-CimInstance Win32_Service | Where-Object {{ {} }} | ForEach-Object {{ $pn = $_.PathName; $q = [char]34; if ($pn.StartsWith($q)) {{ $path = $pn.Substring(1, $pn.IndexOf($q, 1) - 1) }} else {{ $i = $pn.ToLower().IndexOf('.exe'); if ($i -ge 0) {{ $path = $pn.Substring(0, $i + 4) }} else {{ $path = ($pn -split ' ')[0] }} }}; $sig = (Get-AuthenticodeSignature $path -ErrorAction SilentlyContinue); \\\"$($_.Name)|$($_.DisplayName)|$path|$($sig.Status)\\\" }}\"", NON_MICROSOFT_SERVICE_FILTER)
}

pub(crate) fn scan_unsigned_services() -> ScanOutcome {
    let mut threats = Vec::new();

    let result = process::run("powershell", &services_signature_args(), constants::TIMEOUT_EXTRA);

    let stdout = command_stdout("Services non signés", &result)?;

    let lines: Vec<&str> = stdout.lines().collect();
    let scanned = lines.len() as u32;

    for line in &lines {
        let parts = match process::parse_pipe_fields(line, 4) {
            Some(p) => p,
            None => continue,
        };
        let svc_name = parts[0];
        let display_name = parts[1];
        let svc_path = parts[2];
        let sig_status = parts[3];

        if sig_status != "Valid" && !sig_status.is_empty() {
            let severity = if sig_status == "NotSigned" {
                "high"
            } else {
                "medium"
            };
            threats.push(ThreatItem {
                name: svc_name.to_string(),
                category: "Services non signés".into(),
                severity: severity.into(),
                description: format!(
                    "Service '{}' non signe (status: {})",
                    display_name, sig_status
                ),
                path: svc_path.to_string(),
                can_fix: false,
                fix_id: format!("svc_{}", svc_name),
            });
        }
    }

    Ok((scanned, threats))
}

#[cfg(test)]
mod tests;
