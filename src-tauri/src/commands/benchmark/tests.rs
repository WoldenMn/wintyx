use super::*;

#[test]
fn compute_overall_zero_inputs_returns_zero() {
    let score = compute_overall_score(0.0, 0.0, 0.0, 0.0, 999.0);
    assert_eq!(score, 0);
}

#[test]
fn compute_overall_perfect_inputs_returns_100() {
    let score = compute_overall_score(100.0, 20000.0, 1000.0, 1000.0, 0.0);
    assert_eq!(score, 100);
}

#[test]
fn compute_overall_cpu_weight_is_30_percent() {
    let only_cpu = compute_overall_score(50.0, 0.0, 0.0, 0.0, 999.0);
    assert!((30..=31).contains(&only_cpu), "expected ~30, got {}", only_cpu);
}

#[test]
fn compute_overall_memory_weight_is_20_percent() {
    let only_mem = compute_overall_score(0.0, 10000.0, 0.0, 0.0, 999.0);
    assert!((20..=21).contains(&only_mem), "expected ~20, got {}", only_mem);
}

#[test]
fn compute_overall_disk_weight_is_30_percent() {
    let only_disk = compute_overall_score(0.0, 0.0, 500.0, 500.0, 999.0);
    assert!((30..=31).contains(&only_disk), "expected ~30, got {}", only_disk);
}

#[test]
fn compute_overall_network_weight_is_20_percent() {
    let only_net = compute_overall_score(0.0, 0.0, 0.0, 0.0, 0.0);
    assert!((20..=21).contains(&only_net), "expected ~20, got {}", only_net);
}

#[test]
fn compute_overall_clamped_above_100() {
    let exaggerated = compute_overall_score(500.0, 100000.0, 10000.0, 10000.0, 0.0);
    assert!(exaggerated <= 100);
}

#[test]
fn compute_overall_high_latency_floors_network_to_zero() {
    let high_latency = compute_overall_score(50.0, 0.0, 0.0, 0.0, 200.0);
    let no_latency = compute_overall_score(50.0, 0.0, 0.0, 0.0, 50.0);
    assert_eq!(high_latency, no_latency);
}

#[test]
fn limit_history_caps_at_history_max() {
    // Verifie que get_benchmark_history ne retourne jamais plus de
    // BENCHMARK_HISTORY_MAX entrees, meme si le fichier en contient davantage.
    let default_result = BenchmarkResult {
        cpu_score: 0.0,
        memory_score: 0.0,
        disk_read_score: 0.0,
        disk_write_score: 0.0,
        network_latency: 0.0,
        overall_score: 0,
        duration_ms: 0,
    };
    let entries: Vec<BenchmarkEntry> = (0..100)
        .map(|i| BenchmarkEntry {
            timestamp: format!("2026-01-{:02}T00:00:00Z", (i % 28) + 1),
            result: default_result.clone(),
            plausible: true,
            version: None,
        })
        .collect();
    let limited = limit_history(entries);
    assert_eq!(limited.len(), BENCHMARK_HISTORY_MAX);
}

fn sample_result(score: u32) -> BenchmarkResult {
    BenchmarkResult {
        cpu_score: 0.0,
        memory_score: 0.0,
        disk_read_score: 0.0,
        disk_write_score: 0.0,
        network_latency: 0.0,
        overall_score: score,
        duration_ms: 0,
    }
}

fn history_file(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wintyx-bench-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    dir.join("benchmarks.json")
}

#[test]
fn history_is_newest_first_and_keeps_the_latest_entries() {
    let path = history_file("order");
    for i in 0..(BENCHMARK_HISTORY_MAX + 5) {
        let ts = format!("2026-01-01 00:00:{:02}", i);
        persist_benchmark_at(&path, &sample_result(i as u32), ts).unwrap();
    }
    let history = limit_history(load_history_at(&path).unwrap());
    assert_eq!(history.len(), BENCHMARK_HISTORY_MAX);
    assert_eq!(history[0].result.overall_score, (BENCHMARK_HISTORY_MAX + 4) as u32);
    assert!(history.windows(2).all(|w| w[0].timestamp >= w[1].timestamp));
}

#[test]
fn legacy_ascending_file_is_read_newest_first() {
    let entries = vec![
        BenchmarkEntry { timestamp: "2026-01-01 10:00:00".into(), result: sample_result(1), plausible: true, version: None },
        BenchmarkEntry { timestamp: "2026-01-02 10:00:00".into(), result: sample_result(2), plausible: true, version: None },
    ];
    let ordered = limit_history(entries);
    assert_eq!(ordered[0].result.overall_score, 2);
}

#[test]
fn corrupted_benchmark_file_is_kept_aside_not_overwritten() {
    let path = history_file("corrupt");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "[{oops").unwrap();
    persist_benchmark_at(&path, &sample_result(5), "2026-01-01 00:00:00".into()).unwrap();
    assert_eq!(load_history_at(&path).unwrap().len(), 1);
    let kept = std::fs::read_dir(path.parent().unwrap())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".corrompu-"))
        .count();
    assert_eq!(kept, 1);
}

use std::time::Duration;

#[test]
fn throughput_refuses_a_zero_or_tick_sized_duration() {
    assert_eq!(throughput_mbs(10 * 1024 * 1024, Duration::ZERO, MEMORY_MBS_RANGE), None);
    assert_eq!(throughput_mbs(10 * 1024 * 1024, Duration::from_nanos(100), MEMORY_MBS_RANGE), None);
}

#[test]
fn throughput_is_clamped_to_the_range_ceiling() {
    let v = throughput_mbs(u64::MAX / 2, Duration::from_millis(1), MEMORY_MBS_RANGE).unwrap();
    assert_eq!(v, MEMORY_MBS_RANGE.1);
}

#[test]
fn throughput_matches_a_known_rate() {
    let v = throughput_mbs(100 * 1024 * 1024, Duration::from_secs(1), MEMORY_MBS_RANGE).unwrap();
    assert!((v - 100.0).abs() < 1e-6, "got {}", v);
}

#[test]
fn cpu_and_memory_scores_are_measured_and_inside_bounds() {
    let cpu = measure::bench_cpu();
    let mem = measure::bench_memory();
    assert!(is_plausible(cpu, CPU_MOPS_RANGE) && cpu < CPU_MOPS_RANGE.1, "cpu {}", cpu);
    assert!(is_plausible(mem, MEMORY_MBS_RANGE) && mem < MEMORY_MBS_RANGE.1, "mem {}", mem);
}

#[test]
fn cpu_score_is_stable_between_two_runs() {
    let a = measure::bench_cpu();
    let b = measure::bench_cpu();
    let ratio = a.max(b) / a.min(b);
    assert!(ratio < 2.0, "{} vs {}", a, b);
}

fn result_with(cpu: f64, memory: f64) -> BenchmarkResult {
    BenchmarkResult { cpu_score: cpu, memory_score: memory, disk_read_score: 4000.0, disk_write_score: 1500.0, ..sample_result(69) }
}

#[test]
fn legacy_sentinel_memory_score_is_flagged_invalid() {
    assert!(!result_is_plausible(&result_with(319.47, 100_000_000.0)));
    assert!(result_is_plausible(&result_with(319.47, 9_072.8)));
    assert!(!result_is_plausible(&result_with(f64::NAN, 9_072.8)));
    assert!(!result_is_plausible(&result_with(0.0, 9_072.8)));
}

#[test]
fn history_recomputes_plausibility_without_touching_the_file() {
    let path = history_file("plausible");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let raw = r#"[{"timestamp":"2026-05-08 17:53:26","result":{"cpuScore":319.47,"memoryScore":100000000.0,"diskReadScore":4581.9,"diskWriteScore":1589.2,"networkLatency":21.0,"overallScore":91,"durationMs":4206}}]"#;
    std::fs::write(&path, raw).unwrap();
    let history = limit_history(load_history_at(&path).unwrap());
    assert!(!history[0].plausible);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), raw);
}

#[test]
fn a_new_entry_records_the_bench_lineage_not_the_displayed_version() {
    let path = history_file("version-recorded");
    persist_benchmark_at(&path, &sample_result(50), "2026-10-03 09:00:00".into()).unwrap();
    let history = load_history_at(&path).unwrap();
    assert_eq!(history[0].version.as_deref(), Some(LIGNEE_BANC));
}

#[test]
fn the_bench_lineage_matches_the_frontend_comparability_threshold() {
    let frontend = include_str!("../../../../src/lib/benchmark.ts");
    let seuil = frontend
        .lines()
        .find_map(|ligne| ligne.strip_prefix("export const MIN_COMPARABLE_VERSION = \""))
        .and_then(|reste| reste.split('"').next())
        .expect("MIN_COMPARABLE_VERSION introuvable dans src/lib/benchmark.ts");
    assert_eq!(LIGNEE_BANC, seuil);
}

#[test]
fn an_entry_written_before_the_version_field_reads_as_unversioned() {
    let path = history_file("version-legacy");
    let legacy = serde_json::json!([{
        "timestamp": "2026-06-19 10:00:00",
        "result": serde_json::to_value(sample_result(69)).unwrap(),
        "plausible": true
    }]);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, legacy.to_string()).unwrap();
    let history = load_history_at(&path).unwrap();
    assert_eq!(history[0].version, None);
}

#[test]
fn disk_bench_measures_both_directions_and_leaves_no_file() {
    let path = temp_file::unpredictable_temp_path("wintyx-bench-test", "tmp");
    let (read, write) = bench_disk_at(&path, 1024 * 1024).unwrap();
    assert!(read >= DISK_MBS_RANGE.0 && write >= DISK_MBS_RANGE.0, "{read} {write}");
    assert!(!path.exists(), "fichier de mesure resté : {}", path.display());
}

#[test]
fn disk_bench_never_touches_a_path_that_already_exists() {
    let path = temp_file::unpredictable_temp_path("wintyx-bench-test", "tmp");
    std::fs::write(&path, b"intact").unwrap();
    let err = bench_disk_at(&path, 1024 * 1024).unwrap_err();
    assert!(err.contains("Écriture disque impossible"), "{err}");
    assert_eq!(std::fs::read(&path).unwrap(), b"intact");
    std::fs::remove_file(&path).unwrap();
}
