use super::*;

const SECOND: f64 = 10_000_000.0;

fn counter(ticks: f64, timestamp: f64) -> RawCounter {
    RawCounter { ticks, timestamp }
}

fn process(name: &str, ticks: f64, timestamp: f64, working_set: f64) -> RawProcess {
    RawProcess { name: name.into(), counter: counter(ticks, timestamp), working_set_bytes: working_set }
}

fn sample(cpu_ticks: f64, timestamp: f64, processes: Vec<(u32, RawProcess)>) -> Sample {
    Sample {
        cpu_total: Some(counter(cpu_ticks, timestamp)),
        cpu_cores: vec![(1, counter(cpu_ticks, timestamp)), (0, counter(cpu_ticks * 2.0, timestamp))],
        processes: processes.into_iter().collect(),
        ram_total_kb: 16.0 * 1_048_576.0,
        ram_free_kb: 4.0 * 1_048_576.0,
    }
}

#[test]
fn cpu_is_the_inverse_of_idle_ticks_over_elapsed_time() {
    let prev = sample(0.0, 0.0, vec![]);
    let cur = sample(0.75 * SECOND, SECOND, vec![]);
    let m = metrics_between(&prev, &cur);
    assert!((m.cpu_usage - 25.0).abs() < 1e-9, "{}", m.cpu_usage);
}

#[test]
fn cores_are_ordered_by_index() {
    let prev = sample(0.0, 0.0, vec![]);
    let cur = sample(0.5 * SECOND, SECOND, vec![]);
    let m = metrics_between(&prev, &cur);
    assert_eq!(m.cpu_per_core.len(), 2);
    assert!(m.cpu_per_core[0].abs() < 1e-9);
    assert!((m.cpu_per_core[1] - 50.0).abs() < 1e-9);
}

#[test]
fn no_elapsed_time_gives_zero_not_nan() {
    let s = sample(5.0, 100.0, vec![]);
    let m = metrics_between(&s, &s);
    assert_eq!(m.cpu_usage, 0.0);
}

#[test]
fn ram_comes_from_the_current_sample() {
    let s = sample(0.0, 1.0, vec![]);
    let m = metrics_between(&s, &s);
    assert_eq!(m.ram_total_gb, 16.0);
    assert_eq!(m.ram_used_gb, 12.0);
    assert_eq!(m.ram_percent, 75.0);
}

#[test]
fn process_cpu_is_scaled_to_the_logical_core_count_and_sorted() {
    let prev = sample(0.0, 0.0, vec![
        (10, process("busy", 0.0, 0.0, 0.0)),
        (11, process("calm", 0.0, 0.0, 0.0)),
    ]);
    let cur = sample(0.0, SECOND, vec![
        (10, process("busy", SECOND, SECOND, 104_857_600.0)),
        (11, process("calm", 0.1 * SECOND, SECOND, 0.0)),
    ]);
    let m = metrics_between(&prev, &cur);
    assert_eq!(m.top_processes.len(), 2);
    assert_eq!(m.top_processes[0].name, "busy");
    assert_eq!(m.top_processes[0].pid, 10);
    assert_eq!(m.top_processes[0].cpu_percent, 50.0);
    assert_eq!(m.top_processes[0].memory_mb, 100.0);
    assert_eq!(m.top_processes[1].cpu_percent, 5.0);
}

#[test]
fn a_process_absent_from_the_previous_sample_reports_zero_cpu() {
    let prev = sample(0.0, 0.0, vec![]);
    let cur = sample(0.0, SECOND, vec![(42, process("fresh", SECOND, SECOND, 0.0))]);
    let m = metrics_between(&prev, &cur);
    assert_eq!(m.top_processes[0].cpu_percent, 0.0);
}

#[test]
fn idle_and_total_instances_are_not_listed() {
    let cur = sample(0.0, SECOND, vec![
        (0, process("Idle", 0.0, SECOND, 0.0)),
        (7, process("_Total", 0.0, SECOND, 0.0)),
        (8, process("real", 0.0, SECOND, 0.0)),
    ]);
    let m = metrics_between(&cur, &cur);
    assert_eq!(m.top_processes.len(), 1);
    assert_eq!(m.top_processes[0].name, "real");
}

#[test]
fn only_the_top_processes_are_kept() {
    let procs: Vec<(u32, RawProcess)> = (1..=30).map(|i| (i, process(&format!("p{}", i), 0.0, SECOND, 0.0))).collect();
    let cur = sample(0.0, SECOND, procs);
    let m = metrics_between(&cur, &cur);
    assert_eq!(m.top_processes.len(), constants::TOP_PROCESSES_MONITOR);
}
