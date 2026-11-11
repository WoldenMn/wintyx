use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use crate::utils::constants;
mod measure;
use measure::{bench_cpu, bench_memory, is_plausible, throughput_mbs, CPU_MOPS_RANGE, DISK_MBS_RANGE, MEMORY_MBS_RANGE};
use crate::utils::errors::CommandError;
use crate::utils::{json_store, logger, paths, process, temp_file};

const BENCHMARK_HISTORY_MAX: usize = 20;
pub(crate) const LIGNEE_BANC: &str = "2.18.0";

const CPU_SCORE_REF_MOPS: f64 = 50.0;
const MEMORY_SCORE_REF_MBS: f64 = 10_000.0;
const DISK_SCORE_REF_MBS: f64 = 500.0;
const NETWORK_LATENCY_CAP_MS: f64 = 50.0;
const WEIGHT_CPU: f64 = 0.3;
const WEIGHT_MEMORY: f64 = 0.2;
const WEIGHT_DISK: f64 = 0.3;
const WEIGHT_NETWORK: f64 = 0.2;

static BENCHMARKING: AtomicBool = AtomicBool::new(false);

struct BenchmarkGuard;

impl BenchmarkGuard {
    fn acquire() -> Result<Self, String> {
        if BENCHMARKING
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            Err("Un benchmark est déjà en cours".into())
        } else {
            Ok(Self)
        }
    }
}

impl Drop for BenchmarkGuard {
    fn drop(&mut self) {
        BENCHMARKING.store(false, Ordering::SeqCst);
    }
}

#[derive(Serialize, Deserialize, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkResult {
    pub cpu_score: f64,
    pub memory_score: f64,
    pub disk_read_score: f64,
    pub disk_write_score: f64,
    pub network_latency: f64,
    pub overall_score: u32,
    pub duration_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkEntry {
    pub timestamp: String,
    pub result: BenchmarkResult,
    /// Recalcule a chaque lecture : faux pour une entree dont un score sort des bornes
    /// plausibles (anciennes versions du banc), sans reecrire le fichier de l'utilisateur.
    #[serde(default = "default_plausible")]
    pub plausible: bool,
    /// Lignee du banc qui a mesure (`LIGNEE_BANC`), jamais la version affichee. Absente des
    /// entrees ecrites avant 2.18.0, dont la metrique CPU n'est pas comparable a celle d'aujourd'hui.
    #[serde(default)]
    pub version: Option<String>,
}

fn default_plausible() -> bool {
    true
}

pub(crate) fn result_is_plausible(r: &BenchmarkResult) -> bool {
    is_plausible(r.cpu_score, CPU_MOPS_RANGE)
        && is_plausible(r.memory_score, MEMORY_MBS_RANGE)
        && is_plausible(r.disk_read_score, DISK_MBS_RANGE)
        && is_plausible(r.disk_write_score, DISK_MBS_RANGE)
}

#[tauri::command]
#[specta::specta]
pub async fn run_benchmark() -> Result<BenchmarkResult, CommandError> {
    crate::utils::blocking::read_result(run_benchmark_sync).await
}

pub fn run_benchmark_sync() -> Result<BenchmarkResult, CommandError> {
    let _guard = BenchmarkGuard::acquire()?;

    if cfg!(debug_assertions) {
        logger::warn("benchmark", "build de developpement : scores CPU et memoire non comparables a une version release");
    }

    let start = Instant::now();

    let cpu_score = bench_cpu();
    let memory_score = bench_memory();
    let (disk_read, disk_write) = bench_disk()?;
    let network_latency = bench_network();

    let duration = start.elapsed().as_millis() as u64;

    let overall = compute_overall_score(cpu_score, memory_score, disk_read, disk_write, network_latency);

    let result = BenchmarkResult {
        cpu_score,
        memory_score,
        disk_read_score: disk_read,
        disk_write_score: disk_write,
        network_latency,
        overall_score: overall,
        duration_ms: duration,
    };

    if let Err(e) = persist_benchmark(&result) {
        logger::warn("benchmark", &format!("Persistence echouee: {}", e.friendly_message()));
    }

    Ok(result)
}

fn load_history_at(path: &Path) -> Result<Vec<BenchmarkEntry>, CommandError> {
    json_store::load_or_quarantine(path, "benchmark")
}

/// Ordre canonique de l'historique : plus recent d'abord, borne a BENCHMARK_HISTORY_MAX.
/// Le tri redresse aussi les fichiers ecrits avant v2.18 (plus ancien d'abord).
pub(crate) fn limit_history(mut entries: Vec<BenchmarkEntry>) -> Vec<BenchmarkEntry> {
    for entry in &mut entries {
        entry.plausible = result_is_plausible(&entry.result);
    }
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries.truncate(BENCHMARK_HISTORY_MAX);
    entries
}

fn persist_benchmark_at(
    path: &Path,
    result: &BenchmarkResult,
    timestamp: String,
) -> Result<(), CommandError> {
    let mut history = load_history_at(path)?;
    history.push(BenchmarkEntry {
        timestamp,
        result: result.clone(),
        plausible: result_is_plausible(result),
        version: Some(LIGNEE_BANC.to_string()),
    });
    json_store::write_json_atomic(path, &limit_history(history))
}

fn persist_benchmark(result: &BenchmarkResult) -> Result<(), CommandError> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    persist_benchmark_at(&paths::benchmark_history_path(), result, timestamp)
}

#[tauri::command]
#[specta::specta]
pub fn get_benchmark_history() -> Result<Vec<BenchmarkEntry>, CommandError> {
    Ok(limit_history(load_history_at(&paths::benchmark_history_path())?))
}

pub(crate) fn compute_overall_score(
    cpu_score: f64,
    memory_score: f64,
    disk_read: f64,
    disk_write: f64,
    network_latency: f64,
) -> u32 {
    let cpu_norm = (cpu_score / CPU_SCORE_REF_MOPS).min(1.0) * 100.0;
    let mem_norm = (memory_score / MEMORY_SCORE_REF_MBS).min(1.0) * 100.0;
    let disk_norm = ((disk_read + disk_write) / 2.0 / DISK_SCORE_REF_MBS).min(1.0) * 100.0;
    let net_norm = ((NETWORK_LATENCY_CAP_MS - network_latency.min(NETWORK_LATENCY_CAP_MS)) / NETWORK_LATENCY_CAP_MS) * 100.0;
    let overall = cpu_norm * WEIGHT_CPU + mem_norm * WEIGHT_MEMORY + disk_norm * WEIGHT_DISK + net_norm * WEIGHT_NETWORK;
    (overall as u32).min(100)
}

fn disk_speed(bytes: u64, elapsed: std::time::Duration) -> f64 {
    throughput_mbs(bytes, elapsed, DISK_MBS_RANGE).unwrap_or(DISK_MBS_RANGE.0)
}

const BENCH_DISK_BYTES: usize = 50 * 1024 * 1024;

struct RemoveOnDrop<'a>(&'a Path);

impl Drop for RemoveOnDrop<'_> {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(self.0) {
            logger::warn(
                "benchmark",
                &format!("fichier de mesure non supprime ({}): {}", self.0.display(), e),
            );
        }
    }
}

fn bench_disk() -> Result<(f64, f64), String> {
    let path = temp_file::unpredictable_temp_path("wintyx-bench", "tmp");
    bench_disk_at(&path, BENCH_DISK_BYTES)
}

fn bench_disk_at(path: &Path, size: usize) -> Result<(f64, f64), String> {
    use std::io::{Read, Seek, SeekFrom, Write};

    let data = vec![0xABu8; size];

    let start = Instant::now();
    let mut file = temp_file::create_new_read_write(path)
        .map_err(|e| format!("Écriture disque impossible ({}): {}", path.display(), e))?;
    let _cleanup = RemoveOnDrop(path);
    file.write_all(std::hint::black_box(&data))
        .map_err(|e| format!("Écriture disque impossible ({}): {}", path.display(), e))?;
    let write_elapsed = start.elapsed();
    let write_speed = disk_speed(size as u64, write_elapsed);

    let start = Instant::now();
    let mut read_back = Vec::with_capacity(size);
    let read_result = file
        .seek(SeekFrom::Start(0))
        .and_then(|_| file.read_to_end(&mut read_back));
    let read_elapsed = start.elapsed();
    read_result.map_err(|e| format!("Lecture disque impossible: {}", e))?;
    std::hint::black_box(read_back);
    let read_speed = disk_speed(size as u64, read_elapsed);

    Ok((read_speed, write_speed))
}

fn bench_network() -> f64 {
    let result = process::run("ping", "-n 5 -w 1000 8.8.8.8", constants::TIMEOUT_MEDIUM);
    if !result.success {
        logger::warn(
            "benchmark",
            &format!("ping 8.8.8.8 en échec (exit {}), latence réseau fixée à 999 ms", result.exit_code),
        );
        return 999.0;
    }

    for line in result.stdout.lines() {
        if line.contains("Average") || line.contains("Moyenne") {
            if let Some(ms_str) = line.split('=').next_back() {
                let cleaned = ms_str.trim().replace("ms", "").trim().to_string();
                return cleaned.parse().unwrap_or_else(|_| {
                    logger::warn(
                        "benchmark",
                        &format!("latence ping illisible ({:?}), fixee a 999 ms", cleaned),
                    );
                    999.0
                });
            }
        }
    }
    logger::warn("benchmark", "aucune ligne de moyenne dans la sortie ping, latence fixee a 999 ms");
    999.0
}

#[cfg(test)]
mod tests;
