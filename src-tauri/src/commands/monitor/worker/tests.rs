use super::*;
use std::time::Instant;

fn marker(ram_total_kb: f64) -> Sample {
    Sample {
        ram_total_kb,
        ..Sample::default()
    }
}

#[test]
fn a_worker_returns_the_samples_its_thread_takes() {
    let mut calls = 0.0;
    let mut worker = SampleWorker::spawn_with(move || {
        Ok(Box::new(move || {
            calls += 1.0;
            marker(calls)
        }) as TakeSample)
    })
    .unwrap();
    assert_eq!(worker.sample(Duration::from_secs(2)).unwrap().ram_total_kb, 1.0);
    assert_eq!(worker.sample(Duration::from_secs(2)).unwrap().ram_total_kb, 2.0);
}

#[test]
fn a_blocked_query_comes_back_as_a_timeout_instead_of_blocking_the_caller() {
    let mut worker = SampleWorker::spawn_with(|| {
        Ok(Box::new(|| {
            thread::sleep(Duration::from_secs(3));
            Sample::default()
        }) as TakeSample)
    })
    .unwrap();
    let before = Instant::now();
    let outcome = worker.sample(Duration::from_millis(100));
    assert_eq!(outcome.unwrap_err(), SampleError::TimedOut);
    assert!(before.elapsed() < Duration::from_secs(2), "{:?}", before.elapsed());
}

#[test]
fn a_failed_connection_is_reported_with_its_reason() {
    let mut worker = SampleWorker::spawn_with(|| Err("connexion WMI impossible: Winmgmt".to_string())).unwrap();
    match worker.sample(Duration::from_secs(2)) {
        Err(SampleError::Unavailable(detail)) => assert!(detail.contains("Winmgmt"), "{detail}"),
        other => panic!("attendu Unavailable, obtenu {other:?}"),
    }
}

#[test]
fn the_reason_of_a_failed_connection_survives_the_thread_having_already_left() {
    let mut worker = SampleWorker::spawn_with(|| Err("connexion WMI impossible: Winmgmt".to_string())).unwrap();
    thread::sleep(Duration::from_millis(400));
    match worker.sample(Duration::from_secs(2)) {
        Err(SampleError::Unavailable(detail)) => assert!(detail.contains("Winmgmt"), "{detail}"),
        other => panic!("attendu Unavailable avec la raison, obtenu {other:?}"),
    }
}

#[test]
fn a_panicking_sampler_is_reported_as_unavailable_not_as_a_hang() {
    let mut worker = SampleWorker::spawn_with(|| Ok(Box::new(|| -> Sample { panic!("sampler en panique (test)") }) as TakeSample)).unwrap();
    let before = Instant::now();
    let outcome = worker.sample(Duration::from_secs(5));
    assert!(matches!(outcome, Err(SampleError::Unavailable(_))), "{outcome:?}");
    assert!(before.elapsed() < Duration::from_secs(2), "{:?}", before.elapsed());
}

#[cfg(windows)]
#[test]
fn the_real_wmi_worker_takes_a_complete_sample_within_the_timeout() {
    let mut worker = SampleWorker::spawn_wmi().expect("thread de prelevement");
    let sample = worker.sample(Duration::from_secs(20)).expect("echantillon WMI");
    assert!(sample.ram_total_kb > 0.0);
    assert!(sample.cpu_total.is_some());
    assert!(!sample.processes.is_empty());
}
