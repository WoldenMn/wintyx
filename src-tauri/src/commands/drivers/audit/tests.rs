use super::*;
use super::super::pnputil_parser::parse_pnputil_csv;
use chrono::NaiveDate;

fn make_driver(date: &str, provider: &str) -> DriverInfo {
    DriverInfo {
        device_name: "Test Device".into(),
        driver_version: "1.0".into(),
        driver_date: date.into(),
        manufacturer: "Test".into(),
        driver_provider: provider.into(),
        device_class: "Test".into(),
        is_signed: true,
        signer: "Microsoft Windows".into(),
        status: "OK".into(),
        inf_name: "oem1.inf".into(),
    }
}

fn pnp(oem: &str, orig: &str, provider: &str, class: &str, version: &str) -> PnputilDriver {
    PnputilDriver {
        oem_inf: oem.into(),
        original_inf: orig.into(),
        provider: provider.into(),
        class: class.into(),
        version: version.into(),
        date: "2024-01-01".into(),
    }
}

#[test]
fn is_obsolete_within_2_years_returns_false() {
    let now = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap();
    let d = make_driver("2025-01-01", "Realtek");
    assert!(!is_obsolete(&d, now));
}

#[test]
fn is_obsolete_old_third_party_returns_true() {
    let now = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap();
    let d = make_driver("2023-01-01", "Realtek");
    assert!(is_obsolete(&d, now));
}

#[test]
fn is_obsolete_old_microsoft_returns_false() {
    let now = NaiveDate::from_ymd_opt(2026, 5, 17).unwrap();
    let d = make_driver("2020-01-01", "Microsoft");
    assert!(!is_obsolete(&d, now));
}

#[test]
fn is_third_party_microsoft_provider() {
    let d = make_driver("2024-01-01", "Microsoft");
    assert!(!is_third_party(&d));
}

#[test]
fn is_third_party_microsoft_signer() {
    let mut d = make_driver("2024-01-01", "Generic");
    d.signer = "Microsoft Windows Hardware Compatibility Publisher".into();
    assert!(!is_third_party(&d));
}

#[test]
fn is_third_party_realtek() {
    let mut d = make_driver("2024-01-01", "Realtek Semiconductor Corp.");
    d.signer = "Realtek Semiconductor Corp.".into();
    assert!(is_third_party(&d));
}

#[test]
fn detect_duplicates_two_versions_returns_oldest() {
    let pnputil = vec![
        pnp("oem1.inf", "realtek.inf", "Realtek", "Net", "12.0.1.0"),
        pnp("oem5.inf", "realtek.inf", "Realtek", "Net", "12.0.2.0"),
    ];
    let issues = detect_duplicates(&pnputil);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].inf_name, "oem1.inf");
    assert!(matches!(issues[0].kind, IssueKind::Duplicate));
}

#[test]
fn detect_duplicates_keeps_the_numerically_newest_version() {
    let pnputil = vec![
        pnp("oem2.inf", "nv.inf", "NVIDIA", "Display", "9.1.0"),
        pnp("oem7.inf", "nv.inf", "NVIDIA", "Display", "10.0.0"),
    ];
    let issues = detect_duplicates(&pnputil);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].inf_name, "oem2.inf", "9.1.0 est l'ancienne, 10.0.0 doit rester");
}

#[test]
fn detect_duplicates_compares_each_segment_as_a_number() {
    let pnputil = vec![
        pnp("oem3.inf", "nv.inf", "NVIDIA", "Display", "31.0.101.5186"),
        pnp("oem4.inf", "nv.inf", "NVIDIA", "Display", "31.0.15.4601"),
    ];
    let issues = detect_duplicates(&pnputil);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].inf_name, "oem4.inf", "le segment 101 est plus recent que 15");
}

#[test]
fn detect_duplicates_single_version_returns_empty() {
    let pnputil = vec![pnp("oem1.inf", "realtek.inf", "Realtek", "Net", "12.0.1.0")];
    let issues = detect_duplicates(&pnputil);
    assert_eq!(issues.len(), 0);
}

#[test]
fn detect_duplicates_three_versions_returns_two() {
    let pnputil = vec![
        pnp("oem1.inf", "realtek.inf", "Realtek", "Net", "12.0.1.0"),
        pnp("oem5.inf", "realtek.inf", "Realtek", "Net", "12.0.2.0"),
        pnp("oem9.inf", "realtek.inf", "Realtek", "Net", "12.0.3.0"),
    ];
    let issues = detect_duplicates(&pnputil);
    assert_eq!(issues.len(), 2);
}

#[test]
fn classify_all_unsigned_obsolete_returns_two_issues() {
    let mut d = make_driver("2020-01-01", "Realtek");
    d.is_signed = false;
    d.signer = "Unknown".into();
    let issues = classify_all(&[d], None, Vec::new());
    assert_eq!(issues.len(), 2);
    assert!(issues.iter().any(|i| matches!(i.kind, IssueKind::Unsigned)));
    assert!(issues.iter().any(|i| matches!(i.kind, IssueKind::Obsolete)));
}

#[test]
fn classify_all_microsoft_driver_no_issues() {
    let d = make_driver("2020-01-01", "Microsoft");
    let issues = classify_all(&[d], None, Vec::new());
    assert_eq!(issues.len(), 0);
}

#[test]
fn classify_all_recent_third_party_returns_oem() {
    let mut d = make_driver("2025-06-01", "Realtek");
    d.signer = "Realtek Semiconductor Corp.".into();
    let issues = classify_all(&[d], None, Vec::new());
    assert_eq!(issues.len(), 1);
    assert!(matches!(issues[0].kind, IssueKind::ThirdPartyOem));
}

fn pnp_dated(oem: &str, version: &str, date: &str) -> PnputilDriver {
    PnputilDriver {
        date: date.into(),
        ..pnp(oem, "nv.inf", "NVIDIA", "Display", version)
    }
}

#[test]
fn detect_duplicates_same_version_newest_date_wins() {
    let pnputil = vec![
        pnp_dated("oem8.inf", "1.0.0.0", "2025-06-01"),
        pnp_dated("oem3.inf", "1.0.0.0", "2024-01-01"),
    ];
    let issues = detect_duplicates(&pnputil);
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].inf_name, "oem3.inf");
}

#[test]
fn detect_duplicates_full_tie_is_deterministic() {
    let a = vec![
        pnp_dated("oem8.inf", "1.0.0.0", "2025-06-01"),
        pnp_dated("oem3.inf", "1.0.0.0", "2025-06-01"),
    ];
    let b: Vec<PnputilDriver> = a.iter().rev().cloned().collect();
    let flagged_a = detect_duplicates(&a)[0].inf_name.clone();
    let flagged_b = detect_duplicates(&b)[0].inf_name.clone();
    assert_eq!(flagged_a, flagged_b);
    assert_eq!(flagged_a, "oem3.inf");
}

#[test]
fn detect_duplicates_on_real_pnputil_output_keeps_the_newest() {
    let drivers = parse_pnputil_csv(include_str!("../fixtures/enum_drivers.csv")).unwrap();
    let issues = detect_duplicates(&drivers);
    assert!(!issues.is_empty());
    let flagged: Vec<&str> = issues.iter().map(|i| i.inf_name.as_str()).collect();
    assert!(flagged.contains(&"oem24.inf"), "paquet085 1.4.5.7 (2025) est plus ancien que oem118");
    assert!(!flagged.contains(&"oem118.inf"), "paquet085 1.4.6.3 est le plus recent");
}

fn instance(name: &str, inf: &str, version: &str) -> DriverInfo {
    DriverInfo {
        device_name: name.into(),
        inf_name: inf.into(),
        driver_version: version.into(),
        ..make_driver("2019-03-01", "Fabricant A")
    }
}

#[test]
fn one_package_shared_by_two_device_instances_is_listed_once() {
    let drivers = vec![
        instance("Interface audio USB", "oem13.inf", "1.1.2.0"),
        instance("Interface audio USB", "oem13.inf", "1.1.2.0"),
        instance("Périphérique de contrôle", "oem71.inf", "10.0.15063.0"),
        instance("Périphérique de contrôle", "oem71.inf", "10.0.15063.0"),
    ];
    let issues = classify_all(&drivers, None, Vec::new());
    assert_eq!(issues.iter().filter(|i| i.inf_name == "oem13.inf").count(), 1);
    assert_eq!(issues.iter().filter(|i| i.inf_name == "oem71.inf").count(), 1);
}

#[test]
fn issue_ids_are_unique_even_for_distinct_devices_with_long_identical_prefixes() {
    let long = "Intel(R) Wireless Bluetooth(R) Extensible Host Controller";
    let drivers = vec![
        instance(&format!("{} A", long), "oem9.inf", "1.0"),
        instance(&format!("{} B", long), "oem9.inf", "1.0"),
    ];
    let issues = classify_all(&drivers, None, Vec::new());
    assert_eq!(issues.len(), 2);
    let ids: std::collections::HashSet<_> = issues.iter().map(|i| i.id.clone()).collect();
    assert_eq!(ids.len(), issues.len());
}

#[test]
fn same_device_with_two_versions_keeps_both_with_distinct_ids() {
    let drivers = vec![
        instance("Interface audio USB", "oem13.inf", "1.1.2.0"),
        instance("Interface audio USB", "oem13.inf", "1.0.0.0"),
    ];
    let issues = classify_all(&drivers, None, Vec::new());
    let ids: std::collections::HashSet<_> = issues.iter().map(|i| i.id.clone()).collect();
    assert_eq!(issues.len(), 2);
    assert_eq!(ids.len(), 2);
}
