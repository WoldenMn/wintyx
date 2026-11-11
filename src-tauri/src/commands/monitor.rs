use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tauri_specta::Event;
use crate::utils::constants;
use crate::utils::errors::CommandError;

mod collect;
mod controller;
mod sample;
mod supervisor;
mod worker;
use controller::{MonitorController, StopToken};
use supervisor::supervise;
use worker::SampleWorker;

const DEFAULT_MONITOR_INTERVAL_MS: u64 = 1500;
/// Une collecte dure ~400 ms : sous 1 s l'intervalle demande ne serait pas tenu.
const MIN_MONITOR_INTERVAL_MS: u64 = 1000;
const MAX_MONITOR_INTERVAL_MS: u64 = 10000;
const MONITOR_SAMPLE_TIMEOUT: Duration = Duration::from_millis(constants::TIMEOUT_MEDIUM);
const MONITOR_RESTART_BACKOFF: Duration = Duration::from_secs(30);

// Nom du canal derive du nom de struct par tauri-specta (SystemMetrics ->
// "system-metrics"), coherent avec app.emit/listen (event_name est ignore en rc.20).
#[derive(Serialize, Deserialize, Clone, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub cpu_per_core: Vec<f64>,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub ram_percent: f64,
    pub top_processes: Vec<ProcessMetric>,
}

#[derive(Serialize, Deserialize, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetric {
    pub name: String,
    pub pid: u32,
    pub cpu_percent: f64,
    pub memory_mb: f64,
}

static MONITOR_INTERVAL_MS: AtomicU64 = AtomicU64::new(DEFAULT_MONITOR_INTERVAL_MS);
static CONTROLLER: MonitorController = MonitorController::new();

fn clamp_interval(requested: u64) -> u64 {
    requested.clamp(MIN_MONITOR_INTERVAL_MS, MAX_MONITOR_INTERVAL_MS)
}

fn monitor_loop(app: AppHandle, token: Arc<StopToken>) {
    supervise(
        &token,
        SampleWorker::spawn_wmi,
        MONITOR_SAMPLE_TIMEOUT,
        MONITOR_RESTART_BACKOFF,
        || Duration::from_millis(MONITOR_INTERVAL_MS.load(Ordering::SeqCst)),
        |metrics| {
            if let Err(e) = app.emit("system-metrics", metrics) {
                crate::utils::logger::warn("monitor", &format!("emission des metriques echouee: {}", e));
            }
        },
    );
}

// start et stop restent des commandes synchrones : executees sur le thread
// principal, elles sont traitees dans l'ordre d'arrivee des messages IPC. Le
// frontend enchaine stop puis start sans attendre (double montage StrictMode) ;
// en async, ce stop pourrait passer apres le start et couper le nouveau thread.
#[tauri::command]
#[specta::specta]
pub fn start_monitoring(app: AppHandle, interval_ms: Option<u64>) -> Result<(), CommandError> {
    let interval = clamp_interval(interval_ms.unwrap_or(DEFAULT_MONITOR_INTERVAL_MS));
    MONITOR_INTERVAL_MS.store(interval, Ordering::SeqCst);
    CONTROLLER.start(move |token| monitor_loop(app, token));
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_monitor_interval(interval_ms: u64) -> u64 {
    let clamped = clamp_interval(interval_ms);
    MONITOR_INTERVAL_MS.store(clamped, Ordering::SeqCst);
    clamped
}

#[tauri::command]
#[specta::specta]
pub fn stop_monitoring() {
    CONTROLLER.stop();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_metrics_and_process_metric_serialize_expected_camelcase_keys() {
        // SystemMetrics/ProcessMetric sont desormais generes dans bindings.ts via
        // tauri-specta (collect_events!, types-event). Ce test garantit que le rename
        // camelCase reste applique au RUNTIME : l'emission serde doit produire les
        // memes cles que le type genere consomme par le frontend.
        let m = SystemMetrics {
            cpu_usage: 1.0,
            cpu_per_core: vec![1.0],
            ram_used_gb: 1.0,
            ram_total_gb: 1.0,
            ram_percent: 1.0,
            top_processes: vec![ProcessMetric {
                name: "x".into(),
                pid: 1,
                cpu_percent: 1.0,
                memory_mb: 1.0,
            }],
        };
        let json = serde_json::to_value(&m).unwrap();
        let keys: std::collections::BTreeSet<String> =
            json.as_object().unwrap().keys().cloned().collect();
        let expected: std::collections::BTreeSet<String> = [
            "cpuUsage", "cpuPerCore", "ramUsedGb", "ramTotalGb", "ramPercent", "topProcesses",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(keys, expected, "SystemMetrics: cles modifiees -> sync src/lib/types.ts");

        let proc_keys: std::collections::BTreeSet<String> = json["topProcesses"][0]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        let proc_expected: std::collections::BTreeSet<String> =
            ["name", "pid", "cpuPercent", "memoryMb"]
                .iter()
                .map(|s| s.to_string())
                .collect();
        assert_eq!(
            proc_keys, proc_expected,
            "ProcessMetric: cles modifiees -> sync src/lib/types.ts"
        );
    }

    #[test]
    fn clamps_below_minimum() {
        assert_eq!(clamp_interval(100), MIN_MONITOR_INTERVAL_MS);
    }

    #[test]
    fn clamps_above_maximum() {
        assert_eq!(clamp_interval(60_000), MAX_MONITOR_INTERVAL_MS);
    }

    #[test]
    fn keeps_values_inside_range() {
        assert_eq!(clamp_interval(1_250), 1_250);
        assert_eq!(clamp_interval(5_000), 5_000);
    }

    #[test]
    fn default_interval_within_bounds() {
        let clamped = clamp_interval(DEFAULT_MONITOR_INTERVAL_MS);
        assert!(clamped >= MIN_MONITOR_INTERVAL_MS);
        assert!(clamped <= MAX_MONITOR_INTERVAL_MS);
        assert_eq!(clamped, DEFAULT_MONITOR_INTERVAL_MS);
    }
}
