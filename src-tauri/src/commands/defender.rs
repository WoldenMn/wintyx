use serde::{Deserialize, Serialize};
use crate::utils::constants;
use crate::utils::errors::CommandError;
use crate::utils::process;

#[derive(Serialize, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DefenderStatus {
    pub real_time_protection: bool,
    pub antivirus_enabled: bool,
    pub am_service_enabled: bool,
    pub tamper_protected: bool,
    pub signature_version: Option<String>,
    pub signature_last_updated: Option<String>,
    pub signature_age_days: Option<i64>,
    pub last_scan: Option<String>,
}

#[derive(Deserialize)]
struct RawMpStatus {
    #[serde(rename = "RealTimeProtectionEnabled")]
    real_time_protection_enabled: Option<bool>,
    #[serde(rename = "AntivirusEnabled")]
    antivirus_enabled: Option<bool>,
    #[serde(rename = "AMServiceEnabled")]
    am_service_enabled: Option<bool>,
    #[serde(rename = "IsTamperProtected")]
    is_tamper_protected: Option<bool>,
    #[serde(rename = "AntivirusSignatureVersion")]
    antivirus_signature_version: Option<String>,
    #[serde(rename = "SignatureLastUpdated")]
    signature_last_updated: Option<String>,
    #[serde(rename = "QuickScanEnd")]
    quick_scan_end: Option<String>,
    #[serde(rename = "FullScanEnd")]
    full_scan_end: Option<String>,
}

fn none_if_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}

/// Parse la sortie JSON de `Get-MpComputerStatus`. `today` est injecte pour
/// rendre le calcul de l'age des signatures testable sans horloge.
pub(crate) fn parse_defender_status(
    json: &str,
    today: chrono::NaiveDate,
) -> Result<DefenderStatus, String> {
    let raw: RawMpStatus =
        serde_json::from_str(json.trim()).map_err(|e| format!("JSON Defender invalide: {}", e))?;

    let signature_last_updated = none_if_empty(raw.signature_last_updated);
    let signature_age_days = signature_last_updated.as_deref().and_then(|s| {
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")
            .ok()
            .map(|dt| (today - dt.date()).num_days())
    });

    let quick = none_if_empty(raw.quick_scan_end);
    let full = none_if_empty(raw.full_scan_end);
    // Format "YYYY-MM-DD HH:MM" : la comparaison lexicographique == chronologique.
    let last_scan = match (quick, full) {
        (Some(q), Some(f)) => Some(if q >= f { q } else { f }),
        (Some(q), None) => Some(q),
        (None, Some(f)) => Some(f),
        (None, None) => None,
    };

    Ok(DefenderStatus {
        real_time_protection: raw.real_time_protection_enabled.unwrap_or(false),
        antivirus_enabled: raw.antivirus_enabled.unwrap_or(false),
        am_service_enabled: raw.am_service_enabled.unwrap_or(false),
        tamper_protected: raw.is_tamper_protected.unwrap_or(false),
        signature_version: none_if_empty(raw.antivirus_signature_version),
        signature_last_updated,
        signature_age_days,
        last_scan,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_defender_status() -> Result<DefenderStatus, CommandError> {
    crate::utils::blocking::read_result(get_defender_status_sync).await
}

pub fn get_defender_status_sync() -> Result<DefenderStatus, CommandError> {
    let cmd = "-NoProfile -Command \"Get-MpComputerStatus | Select-Object RealTimeProtectionEnabled, AntivirusEnabled, AMServiceEnabled, IsTamperProtected, AntivirusSignatureVersion, @{N='SignatureLastUpdated';E={if($_.AntivirusSignatureLastUpdated){$_.AntivirusSignatureLastUpdated.ToString('yyyy-MM-dd HH:mm')}else{''}}}, @{N='QuickScanEnd';E={if($_.QuickScanEndTime){$_.QuickScanEndTime.ToString('yyyy-MM-dd HH:mm')}else{''}}}, @{N='FullScanEnd';E={if($_.FullScanEndTime){$_.FullScanEndTime.ToString('yyyy-MM-dd HH:mm')}else{''}}} | ConvertTo-Json -Compress\"";
    let result = process::run("powershell", cmd, constants::TIMEOUT_MEDIUM);
    if !result.success || result.stdout.trim().is_empty() {
        return Err(CommandError::WmiUnavailable(
            "Get-MpComputerStatus indisponible (Defender désactivé ou absent)".to_string(),
        ));
    }
    parse_defender_status(&result.stdout, chrono::Local::now().date_naive())
        .map_err(CommandError::Generic)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> chrono::NaiveDate {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn parses_active_defender() {
        let json = r#"{"RealTimeProtectionEnabled":true,"AntivirusEnabled":true,"AMServiceEnabled":true,"IsTamperProtected":true,"AntivirusSignatureVersion":"1.405.1","SignatureLastUpdated":"2026-06-20 08:00","QuickScanEnd":"2026-06-22 09:00","FullScanEnd":"2026-06-10 03:00"}"#;
        let s = parse_defender_status(json, day("2026-06-24")).unwrap();
        assert!(s.real_time_protection);
        assert!(s.tamper_protected);
        assert_eq!(s.signature_version.as_deref(), Some("1.405.1"));
        assert_eq!(s.signature_age_days, Some(4));
        assert_eq!(s.last_scan.as_deref(), Some("2026-06-22 09:00"));
    }

    #[test]
    fn parses_passive_defender_all_false() {
        let json = r#"{"RealTimeProtectionEnabled":false,"AntivirusEnabled":false,"AMServiceEnabled":false,"IsTamperProtected":false,"AntivirusSignatureVersion":null,"SignatureLastUpdated":"","QuickScanEnd":"","FullScanEnd":""}"#;
        let s = parse_defender_status(json, day("2026-06-24")).unwrap();
        assert!(!s.real_time_protection);
        assert_eq!(s.signature_version, None);
        assert_eq!(s.signature_age_days, None);
        assert_eq!(s.last_scan, None);
    }

    #[test]
    fn last_scan_picks_full_when_more_recent() {
        let json = r#"{"RealTimeProtectionEnabled":true,"AntivirusEnabled":true,"AMServiceEnabled":true,"IsTamperProtected":true,"AntivirusSignatureVersion":"1.0","SignatureLastUpdated":"2026-06-20 08:00","QuickScanEnd":"2026-06-01 09:00","FullScanEnd":"2026-06-23 03:00"}"#;
        let s = parse_defender_status(json, day("2026-06-24")).unwrap();
        assert_eq!(s.last_scan.as_deref(), Some("2026-06-23 03:00"));
    }

    #[test]
    fn invalid_json_errors() {
        assert!(parse_defender_status("not json", day("2026-06-24")).is_err());
    }
}
