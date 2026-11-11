use super::*;
use crate::utils::process::CmdResult;

fn result(success: bool, stdout: &str, stderr: &str) -> CmdResult {
    CmdResult { success, stdout: stdout.into(), stderr: stderr.into(), exit_code: if success { 0 } else { 1 } }
}

#[test]
fn failed_process_is_an_error_not_an_empty_list() {
    let err = consumers_from(result(false, "", "Acces refuse")).unwrap_err();
    assert!(matches!(err, CommandError::ProcessFailed { .. }), "{:?}", err);
}

#[test]
fn parses_pipe_separated_lines() {
    let out = consumers_from(result(true, "4212|chrome|1073741824\r\n88|Idle|8192\r\n", "")).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!((out[0].pid, out[0].name.as_str(), out[0].memory_mb), (4212, "chrome", 1024.0));
}

#[test]
fn malformed_lines_are_skipped_not_zeroed() {
    let out = consumers_from(result(true, "abc|x|12\n77|ok|1048576\nbroken\n", "")).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].pid, 77);
}

fn row(fields: &[(&str, Variant)]) -> HashMap<String, Variant> {
    fields.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

fn os_row(total_kb: u64, free_kb: u64) -> HashMap<String, Variant> {
    row(&[
        ("TotalVisibleMemorySize", Variant::UI8(total_kb)),
        ("FreePhysicalMemory", Variant::UI8(free_kb)),
    ])
}

fn stick(speed: u32) -> MemoryStick {
    stick_from_row(&row(&[
        ("Capacity", Variant::String("8589934592".into())),
        ("ConfiguredClockSpeed", Variant::UI4(speed)),
    ]))
}

fn array_rows(devices: u16) -> Result<Vec<HashMap<String, Variant>>, CommandError> {
    Ok(vec![row(&[("MemoryDevices", Variant::UI2(devices))])])
}

#[test]
fn a_complete_read_gives_the_summary() {
    let summary = memory_summary_from(
        &[os_row(16 * 1_048_576, 4 * 1_048_576)],
        Ok(vec![stick(3200), stick(3600)]),
        array_rows(4),
    )
    .unwrap();
    assert_eq!((summary.total_gb, summary.used_gb, summary.available_gb), (16.0, 12.0, 4.0));
    assert_eq!(summary.used_percent, 75);
    assert_eq!((summary.slots_filled, summary.total_slots, summary.max_speed_mhz), (2, 4, 3600));
}

#[test]
fn a_missing_or_empty_os_row_is_an_error_not_zero_memory() {
    let sticks = || Ok(vec![stick(3200)]);
    let err = memory_summary_from(&[], sticks(), array_rows(2)).unwrap_err();
    assert!(matches!(err, CommandError::WmiUnavailable(_)), "{err:?}");
    let err = memory_summary_from(&[os_row(0, 0)], sticks(), array_rows(2)).unwrap_err();
    assert!(matches!(err, CommandError::WmiUnavailable(_)), "{err:?}");
}

#[test]
fn a_failed_sticks_query_is_an_error_not_zero_slots_filled() {
    let failure = CommandError::WmiUnavailable("Win32_PhysicalMemory".into());
    let err = memory_summary_from(&[os_row(1_048_576, 1024)], Err(failure), array_rows(2)).unwrap_err();
    assert!(err.to_string().contains("Win32_PhysicalMemory"), "{err}");
}

#[test]
fn a_failed_slot_array_query_is_an_error_not_zero_total_slots() {
    let failure = CommandError::WmiUnavailable("Win32_PhysicalMemoryArray".into());
    let err = memory_summary_from(&[os_row(1_048_576, 1024)], Ok(vec![stick(3200)]), Err(failure)).unwrap_err();
    assert!(err.to_string().contains("Win32_PhysicalMemoryArray"), "{err}");
}

#[test]
fn a_machine_that_reports_no_stick_is_a_legitimate_zero() {
    let summary = memory_summary_from(&[os_row(1_048_576, 1024)], Ok(vec![]), array_rows(0)).unwrap();
    assert_eq!((summary.slots_filled, summary.total_slots, summary.max_speed_mhz), (0, 0, 0));
}

#[cfg(windows)]
#[test]
fn the_real_machine_memory_summary_is_read_without_defaults() {
    let summary = get_memory_summary_sync().expect("lecture WMI");
    assert!(summary.total_gb > 0.0 && summary.used_gb > 0.0 && summary.available_gb > 0.0);
    assert!(summary.used_percent > 0 && summary.used_percent <= 100);
}

#[test]
fn slot_count_reads_a_16_bit_wmi_integer() {
    let rows = [row(&[("MemoryDevices", Variant::UI2(4))])];
    assert_eq!(total_slots_from(&rows), 4);
}

#[test]
fn slot_count_adds_every_memory_array() {
    let rows = [row(&[("MemoryDevices", Variant::UI2(2))]), row(&[("MemoryDevices", Variant::UI2(2))])];
    assert_eq!(total_slots_from(&rows), 4);
}

#[test]
fn stick_form_factor_is_decoded_from_a_16_bit_integer() {
    let stick = stick_from_row(&row(&[
        ("FormFactor", Variant::UI2(8)),
        ("SMBIOSMemoryType", Variant::UI4(34)),
        ("ConfiguredClockSpeed", Variant::UI4(4800)),
        ("Capacity", Variant::String("17179869184".into())),
    ]));
    assert_eq!(stick.form_factor, "DIMM");
    assert_eq!(stick.memory_type, "DDR5");
    assert_eq!(stick.speed_mhz, 4800);
    assert_eq!(stick.capacity_gb, 16.0);
}
