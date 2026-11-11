use super::*;

fn full() -> Reads {
    Reads {
        os_ram: Some(("Windows 11 Pro".into(), 31.8, 12.4)),
        cpu: Some(("CPU Test".into(), 8, 3600)),
        gpu: Some("GPU Test".into()),
        disk_used_percent: Some(42),
        uptime_seconds: Some(3600),
        cpu_usage: Some(7),
        antivirus: Some("Antivirus Test".into()),
    }
}

fn volatile() -> VolatileReads {
    VolatileReads {
        os_ram: Some(("Windows 11 Pro".into(), 31.8, 14.0)),
        disk_used_percent: Some(43),
        uptime_seconds: Some(3700),
        cpu_usage: Some(9),
    }
}

#[test]
fn a_complete_read_is_returned_as_read_and_cached() {
    let (info, cacheable) = assemble(full(), "PC".into()).unwrap();
    assert_eq!(info.os_name, "Windows 11 Pro");
    assert_eq!((info.cpu_name.as_str(), info.cpu_cores, info.cpu_clock_mhz), ("CPU Test", 8, 3600));
    assert_eq!(info.gpu_name, "GPU Test");
    assert_eq!(info.antivirus_name, "Antivirus Test");
    assert_eq!((info.disk_used_percent, info.uptime_seconds, info.cpu_usage), (42, 3600, 7));
    let cached = cacheable.expect("lecture complete : mise en cache attendue");
    assert_eq!(cached.gpu_name, "GPU Test");
    assert_eq!(cached.total_ram_gb, 31.8);
}

#[test]
fn an_unreadable_static_field_is_never_cached() {
    for mutate in [
        (|r: &mut Reads| r.cpu = None) as fn(&mut Reads),
        |r| r.gpu = None,
        |r| r.antivirus = None,
    ] {
        let mut reads = full();
        mutate(&mut reads);
        let (info, cacheable) = assemble(reads, "PC".into()).expect("les champs lus restent rendus");
        assert!(cacheable.is_none(), "un champ illisible ne doit pas etre fige 60 s");
        assert_eq!(info.os_name, "Windows 11 Pro");
    }
}

#[test]
fn an_unreadable_field_falls_back_to_the_unknown_marker_in_the_returned_data_only() {
    let mut reads = full();
    reads.gpu = None;
    reads.cpu = None;
    let (info, cacheable) = assemble(reads, "PC".into()).unwrap();
    assert_eq!(info.gpu_name, "Inconnu");
    assert_eq!((info.cpu_name.as_str(), info.cpu_cores, info.cpu_clock_mhz), ("Inconnu", 0, 0));
    assert!(cacheable.is_none());
}

#[test]
fn an_unreadable_volatile_field_does_not_block_the_cache_of_the_static_ones() {
    let mut reads = full();
    reads.cpu_usage = None;
    reads.disk_used_percent = None;
    reads.uptime_seconds = None;
    let (info, cacheable) = assemble(reads, "PC".into()).unwrap();
    assert_eq!((info.disk_used_percent, info.uptime_seconds, info.cpu_usage), (0, 0, 0));
    assert!(cacheable.is_some());
}

#[test]
fn when_the_os_query_failed_the_call_fails_instead_of_returning_zero_memory() {
    let mut reads = full();
    reads.os_ram = None;
    let err = assemble(reads, "PC".into()).expect_err("echec attendu");
    assert!(matches!(err, CommandError::WmiUnavailable(_)), "{err:?}");
}

#[test]
fn refreshing_a_cached_entry_takes_the_live_memory_and_keeps_the_static_fields() {
    let (_, cacheable) = assemble(full(), "PC".into()).unwrap();
    let info = refresh(cacheable.unwrap(), volatile(), "PC".into()).unwrap();
    assert_eq!(info.used_ram_gb, 14.0);
    assert_eq!((info.disk_used_percent, info.uptime_seconds, info.cpu_usage), (43, 3700, 9));
    assert_eq!(info.gpu_name, "GPU Test");
    assert_eq!(info.total_ram_gb, 31.8);
}

#[test]
fn refreshing_with_an_unreadable_os_query_fails() {
    let (_, cacheable) = assemble(full(), "PC".into()).unwrap();
    let mut reads = volatile();
    reads.os_ram = None;
    assert!(refresh(cacheable.unwrap(), reads, "PC".into()).is_err());
}
