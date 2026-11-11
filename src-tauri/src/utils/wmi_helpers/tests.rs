use super::*;

#[test]
fn kb_to_gb_rounded_zero() {
    assert_eq!(kb_to_gb_rounded(0.0), 0.0);
}

#[test]
fn kb_to_gb_rounded_8gb() {
    assert_eq!(kb_to_gb_rounded(8388608.0), 8.0);
}

#[test]
fn kb_to_gb_rounded_precision() {
    let result = kb_to_gb_rounded(17163018.24);
    assert!((result - 16.37).abs() < 0.01);
}

#[test]
fn v_str_string_variant() {
    let v = Variant::String("hello".into());
    assert_eq!(v_str(Some(&v)), "hello");
}

#[test]
fn v_str_none() {
    assert_eq!(v_str(None), "");
}

#[test]
fn v_str_null_variant() {
    let v = Variant::Null;
    assert_eq!(v_str(Some(&v)), "");
}

#[test]
fn v_str_i4_variant() {
    let v = Variant::I4(42);
    assert_eq!(v_str(Some(&v)), "42");
}

#[test]
fn v_f64_r8_variant() {
    let v = Variant::R8(2.5);
    assert!((v_f64(Some(&v)) - 2.5).abs() < f64::EPSILON);
}

#[test]
fn v_f64_none() {
    assert_eq!(v_f64(None), 0.0);
}

#[test]
fn v_f64_ui4_variant() {
    let v = Variant::UI4(100);
    assert_eq!(v_f64(Some(&v)), 100.0);
}

#[test]
fn v_f64_string_variant() {
    let v = Variant::String("99.5".into());
    assert!((v_f64(Some(&v)) - 99.5).abs() < f64::EPSILON);
}

#[test]
fn v_bool_true() {
    let v = Variant::Bool(true);
    assert!(v_bool(Some(&v)));
}

#[test]
fn v_bool_false() {
    let v = Variant::Bool(false);
    assert!(!v_bool(Some(&v)));
}

#[test]
fn v_bool_none() {
    assert!(!v_bool(None));
}

#[test]
fn v_bool_string_true() {
    let v = Variant::String("True".into());
    assert!(v_bool(Some(&v)));
}

#[test]
fn v_u32_ui4_variant() {
    let v = Variant::UI4(42);
    assert_eq!(v_u32(Some(&v)), 42);
}

#[test]
fn com_thread_returns_the_closure_result() {
    let out = run_on_com_thread_within(|| Ok(42), Duration::from_secs(5));
    assert_eq!(out.unwrap(), 42);
}

#[test]
fn com_thread_times_out_instead_of_waiting_forever() {
    let started = std::time::Instant::now();
    let out: Result<(), CommandError> = run_on_com_thread_within(
        || {
            std::thread::sleep(Duration::from_millis(600));
            Ok(())
        },
        Duration::from_millis(50),
    );
    assert!(matches!(out, Err(CommandError::Timeout { timeout_ms: 50, .. })));
    assert!(started.elapsed() < Duration::from_millis(400));
}

#[test]
fn com_thread_panic_becomes_wmi_unavailable() {
    let out: Result<(), CommandError> =
        run_on_com_thread_within(|| panic!("boom"), Duration::from_secs(5));
    assert!(matches!(out, Err(CommandError::WmiUnavailable(m)) if m.contains("boom")));
}

#[test]
fn v_u32_none() {
    assert_eq!(v_u32(None), 0);
}

#[test]
fn v_i32_i4_variant() {
    let v = Variant::I4(-5);
    assert_eq!(v_i32(Some(&v)), -5);
}

#[test]
fn v_i32_none() {
    assert_eq!(v_i32(None), 0);
}

// Regression v2.12.0 (crash-02/03) : une valeur WMI String non numerique ne
// doit pas paniquer mais retomber sur 0 (le WARNING est journalise via logger).
#[test]
fn v_f64_malformed_string_returns_zero() {
    let v = Variant::String("not-a-number".into());
    assert_eq!(v_f64(Some(&v)), 0.0);
}

#[test]
fn v_i32_malformed_string_returns_zero() {
    let v = Variant::String("xyz".into());
    assert_eq!(v_i32(Some(&v)), 0);
}

#[test]
fn v_u32_malformed_string_returns_zero() {
    let v = Variant::String("12.5abc".into());
    assert_eq!(v_u32(Some(&v)), 0);
}

fn wmi_err(code: &str) -> CommandError {
    CommandError::WmiUnavailable(format!("HRESULT Call failed with: {}", code))
}

#[test]
fn transient_codes_are_the_documented_memory_and_quota_errors() {
    assert!(is_transient_wmi_error(&wmi_err("0x80041006")));
    assert!(is_transient_wmi_error(&wmi_err("0x8004106C")));
    assert!(is_transient_wmi_error(&wmi_err("0x8004106c")));
    assert!(!is_transient_wmi_error(&wmi_err("0x80041003")));
    assert!(!is_transient_wmi_error(&wmi_err("0x80041017")));
    assert!(!is_transient_wmi_error(&CommandError::Timeout { command: "WMI".into(), timeout_ms: 1 }));
}

#[test]
fn retry_succeeds_after_one_transient_failure() {
    let mut calls = 0;
    let out = retry_transient(2, Duration::ZERO, || {
        calls += 1;
        if calls == 1 { Err(wmi_err("0x80041006")) } else { Ok(calls) }
    });
    assert_eq!(out.unwrap(), 2);
}

#[test]
fn retry_is_bounded_and_returns_the_last_error() {
    let mut calls = 0;
    let out: Result<u32, _> = retry_transient(2, Duration::ZERO, || {
        calls += 1;
        Err(wmi_err("0x80041006"))
    });
    assert!(out.is_err());
    assert_eq!(calls, 3);
}

#[test]
fn retry_never_repeats_a_permanent_error() {
    let mut calls = 0;
    let out: Result<u32, _> = retry_transient(2, Duration::ZERO, || {
        calls += 1;
        Err(wmi_err("0x80041017"))
    });
    assert!(out.is_err());
    assert_eq!(calls, 1);
}

#[test]
fn v_u32_reads_every_integer_width() {
    assert_eq!(v_u32(Some(&Variant::UI1(200))), 200);
    assert_eq!(v_u32(Some(&Variant::UI2(4))), 4);
    assert_eq!(v_u32(Some(&Variant::UI4(70_000))), 70_000);
    assert_eq!(v_u32(Some(&Variant::UI8(5_000))), 5_000);
    assert_eq!(v_u32(Some(&Variant::I1(7))), 7);
    assert_eq!(v_u32(Some(&Variant::I2(300))), 300);
    assert_eq!(v_u32(Some(&Variant::I4(9))), 9);
    assert_eq!(v_u32(Some(&Variant::I8(11))), 11);
}

#[test]
fn v_u32_out_of_range_is_zero_not_wrapped() {
    assert_eq!(v_u32(Some(&Variant::I2(-5))), 0);
    assert_eq!(v_u32(Some(&Variant::UI8(u64::MAX))), 0);
}

#[test]
fn v_u32_unexpected_type_is_zero() {
    assert_eq!(v_u32(Some(&Variant::Bool(true))), 0);
    assert_eq!(v_u32(Some(&Variant::Null)), 0);
}

#[test]
fn v_i32_reads_every_integer_width() {
    assert_eq!(v_i32(Some(&Variant::UI1(200))), 200);
    assert_eq!(v_i32(Some(&Variant::UI2(65_000))), 65_000);
    assert_eq!(v_i32(Some(&Variant::UI4(70_000))), 70_000);
    assert_eq!(v_i32(Some(&Variant::UI8(5))), 5);
    assert_eq!(v_i32(Some(&Variant::I1(-3))), -3);
    assert_eq!(v_i32(Some(&Variant::I2(-300))), -300);
    assert_eq!(v_i32(Some(&Variant::I8(-9))), -9);
}

#[test]
fn v_i32_out_of_range_is_zero_not_wrapped() {
    assert_eq!(v_i32(Some(&Variant::UI4(u32::MAX))), 0);
    assert_eq!(v_i32(Some(&Variant::I8(i64::MAX))), 0);
}

#[test]
fn v_f64_reads_every_integer_width() {
    assert_eq!(v_f64(Some(&Variant::UI1(200))), 200.0);
    assert_eq!(v_f64(Some(&Variant::UI2(4))), 4.0);
    assert_eq!(v_f64(Some(&Variant::I1(-3))), -3.0);
    assert_eq!(v_f64(Some(&Variant::I2(-300))), -300.0);
}

#[test]
fn v_str_reads_every_integer_width() {
    assert_eq!(v_str(Some(&Variant::UI1(1))), "1");
    assert_eq!(v_str(Some(&Variant::UI2(2))), "2");
    assert_eq!(v_str(Some(&Variant::I1(-3))), "-3");
    assert_eq!(v_str(Some(&Variant::I2(-4))), "-4");
}

#[test]
fn v_bool_reads_every_integer_width() {
    assert!(v_bool(Some(&Variant::UI1(1))));
    assert!(v_bool(Some(&Variant::UI2(1))));
    assert!(v_bool(Some(&Variant::UI4(1))));
    assert!(v_bool(Some(&Variant::I2(-1))));
    assert!(!v_bool(Some(&Variant::UI2(0))));
}
