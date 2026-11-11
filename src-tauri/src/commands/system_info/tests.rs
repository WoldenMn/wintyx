use super::*;

#[test]
fn parse_valid_datetime() {
    let result = parse_wmi_datetime_to_uptime("20260412140000.000000+060");
    assert!(result.is_some());
}

#[test]
fn parse_too_short() {
    assert_eq!(parse_wmi_datetime_to_uptime("2026"), None);
}

#[test]
fn parse_non_ascii() {
    assert_eq!(parse_wmi_datetime_to_uptime("日本語テスト文字列"), None);
}

#[test]
fn is_fresh_within_ttl() {
    let cached_at = Instant::now();
    let now = cached_at + Duration::from_secs(30);
    assert!(is_fresh(cached_at, now, 60));
}

#[test]
fn is_fresh_expired_at_exact_ttl() {
    let cached_at = Instant::now();
    let now = cached_at + Duration::from_secs(60);
    assert!(!is_fresh(cached_at, now, 60));
}

#[test]
fn is_fresh_expired_beyond_ttl() {
    let cached_at = Instant::now();
    let now = cached_at + Duration::from_secs(120);
    assert!(!is_fresh(cached_at, now, 60));
}

#[test]
fn is_fresh_now_before_cached_at_saturates_to_fresh() {
    let now = Instant::now();
    let cached_at = now + Duration::from_secs(5);
    assert!(is_fresh(cached_at, now, 60));
}

#[cfg(windows)]
#[test]
fn the_real_machine_is_read_field_by_field_and_its_statics_are_cached() {
    let info = get_system_info_sync().expect("lecture WMI");
    assert_ne!(info.os_name, "Inconnu");
    assert_ne!(info.cpu_name, "Inconnu");
    assert_ne!(info.gpu_name, "Inconnu");
    assert!(info.total_ram_gb > 0.0 && info.used_ram_gb > 0.0 && info.used_ram_gb <= info.total_ram_gb);
    assert!(info.cpu_cores > 0 && info.cpu_clock_mhz > 0);
    assert!(info.disk_used_percent > 0 && info.uptime_seconds > 0);
    let cached = system_info_cache().lock().unwrap().clone();
    assert!(cached.is_some(), "lecture complete : les champs statiques doivent etre en cache");
}
