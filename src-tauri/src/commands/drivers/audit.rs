use super::pnputil_parser::PnputilDriver;
use super::{DriverInfo, DriverIssue, IssueKind, Severity};
use crate::utils::constants::DRIVER_OBSOLETE_THRESHOLD_DAYS;

/// True if driver date is > 2 years old AND provider is not Microsoft.
/// Microsoft drivers are excluded because Windows Update manages them.
fn is_obsolete(driver: &DriverInfo, now: chrono::NaiveDate) -> bool {
    let provider_lc = driver.driver_provider.to_lowercase();
    if provider_lc.contains("microsoft") {
        return false;
    }
    let Ok(d) = chrono::NaiveDate::parse_from_str(&driver.driver_date, "%Y-%m-%d") else {
        return false;
    };
    (now - d).num_days() > DRIVER_OBSOLETE_THRESHOLD_DAYS
}

fn is_third_party(driver: &DriverInfo) -> bool {
    let p = driver.driver_provider.to_lowercase();
    let s = driver.signer.to_lowercase();
    !p.contains("microsoft") && !s.contains("microsoft")
}

/// Version as numeric segments: "31.0.101.5186" -> [31, 0, 101, 5186].
/// A non-numeric segment counts as 0. Compared as strings, "9.1.0" would beat "10.0.0".
fn version_key(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|s| s.trim().parse().unwrap_or(0))
        .collect()
}

/// "oem136.inf" -> 136. Dernier critere de depart entre deux paquets de meme version et meme date.
fn oem_number(oem_inf: &str) -> u64 {
    oem_inf
        .trim_start_matches("oem")
        .trim_end_matches(".inf")
        .parse()
        .unwrap_or(0)
}

/// Detect duplicate drivers in the DriverStore.
/// Groups by (provider, class, original_inf). Multi-entry groups flag all but newest.
/// Newest = highest version compared segment by segment as numbers, then latest date, then highest oem number.
pub(super) fn detect_duplicates(pnputil: &[PnputilDriver]) -> Vec<DriverIssue> {
    use std::collections::HashMap;
    let mut groups: HashMap<(String, String, String), Vec<&PnputilDriver>> = HashMap::new();
    for d in pnputil {
        let key = (
            d.provider.to_lowercase(),
            d.class.to_lowercase(),
            d.original_inf.to_lowercase(),
        );
        groups.entry(key).or_default().push(d);
    }
    let mut issues = Vec::new();
    for entries in groups.values() {
        if entries.len() < 2 {
            continue;
        }
        let newest_idx = entries
            .iter()
            .enumerate()
            .max_by_key(|(_, e)| (version_key(&e.version), e.date.clone(), oem_number(&e.oem_inf)))
            .map(|(i, _)| i)
            .unwrap_or(0);
        for (i, d) in entries.iter().enumerate() {
            if i == newest_idx {
                continue;
            }
            issues.push(DriverIssue {
                id: format!("dup-{}", d.oem_inf),
                kind: IssueKind::Duplicate,
                severity: Severity::Low,
                device_name: format!("{} {}", d.provider, d.original_inf),
                manufacturer: d.provider.clone(),
                driver_version: d.version.clone(),
                driver_date: d.date.clone(),
                provider: d.provider.clone(),
                inf_name: d.oem_inf.clone(),
                instance_id: None,
                reason: format!(
                    "Version obsolète dans DriverStore (v{} alors qu'une plus récente existe)",
                    d.version
                ),
            });
        }
    }
    issues
}

pub(super) fn classify_all(
    drivers: &[DriverInfo],
    pnputil: Option<&[PnputilDriver]>,
    phantoms: Vec<DriverIssue>,
) -> Vec<DriverIssue> {
    let now = chrono::Local::now().date_naive();
    let mut issues = Vec::new();
    let mut seen_packages = std::collections::HashSet::new();

    for d in drivers {
        // Win32_PnPSignedDriver liste une ligne par peripherique : un meme paquet
        // partage par plusieurs instances (par exemple deux interfaces audio USB identiques)
        // ne doit produire qu'un constat.
        if !seen_packages.insert((d.device_name.clone(), d.inf_name.clone(), d.driver_version.clone())) {
            continue;
        }
        let base_id_suffix = format!(
            "{}-{}",
            d.device_name.chars().take(40).collect::<String>(),
            d.inf_name
        );

        if !d.is_signed {
            issues.push(DriverIssue {
                id: format!("unsigned-{}", base_id_suffix),
                kind: IssueKind::Unsigned,
                severity: Severity::High,
                device_name: d.device_name.clone(),
                manufacturer: d.manufacturer.clone(),
                driver_version: d.driver_version.clone(),
                driver_date: d.driver_date.clone(),
                provider: d.driver_provider.clone(),
                inf_name: d.inf_name.clone(),
                instance_id: None,
                reason: format!("Pilote non signe (signataire: {})", d.signer),
            });
        }

        if is_obsolete(d, now) {
            issues.push(DriverIssue {
                id: format!("obsolete-{}", base_id_suffix),
                kind: IssueKind::Obsolete,
                severity: Severity::Medium,
                device_name: d.device_name.clone(),
                manufacturer: d.manufacturer.clone(),
                driver_version: d.driver_version.clone(),
                driver_date: d.driver_date.clone(),
                provider: d.driver_provider.clone(),
                inf_name: d.inf_name.clone(),
                instance_id: None,
                reason: format!("Pilote de {} (>2 ans), fournisseur {}", d.driver_date, d.driver_provider),
            });
        } else if is_third_party(d) {
            issues.push(DriverIssue {
                id: format!("oem-{}", base_id_suffix),
                kind: IssueKind::ThirdPartyOem,
                severity: Severity::Low,
                device_name: d.device_name.clone(),
                manufacturer: d.manufacturer.clone(),
                driver_version: d.driver_version.clone(),
                driver_date: d.driver_date.clone(),
                provider: d.driver_provider.clone(),
                inf_name: d.inf_name.clone(),
                instance_id: None,
                reason: format!("Pilote tiers (fournisseur: {})", d.driver_provider),
            });
        }
    }

    if let Some(pn) = pnputil {
        issues.extend(detect_duplicates(pn));
    }

    issues.extend(phantoms);

    ensure_unique_ids(&mut issues);
    issues
}

/// Les identifiants servent de cle React : une collision restante (noms tronques a 40
/// caracteres, deux versions d'un meme paquet) recoit un suffixe d'ordre.
fn ensure_unique_ids(issues: &mut [DriverIssue]) {
    let mut seen: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for issue in issues.iter_mut() {
        let count = seen.entry(issue.id.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            issue.id = format!("{}-{}", issue.id, count);
        }
    }
}

#[cfg(test)]
mod tests;
