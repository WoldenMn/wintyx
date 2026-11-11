//! Echantillons bruts et calcul des taux entre deux echantillons.
//!
//! Les classes `Win32_PerfFormattedData_*` recalculent les pourcentages a chaque requete :
//! mesure sur ce projet, ~265 ms pour le CPU et ~800 ms pour les processus, donc ~1,1 s par
//! cycle et un intervalle de 0,5 s intenable. Les classes `Win32_PerfRawData_*` rendent les
//! compteurs cumules (CPU ~11 ms, processus ~390 ms) ; le taux se calcule entre deux cycles
//! avec la meme formule que le systeme, sans repasser par WMI.

use std::collections::HashMap;

use super::{ProcessMetric, SystemMetrics};
use crate::utils::constants;
use crate::utils::wmi_helpers::kb_to_gb_rounded;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct RawCounter {
    pub ticks: f64,
    pub timestamp: f64,
}

#[derive(Debug, Clone)]
pub(super) struct RawProcess {
    pub name: String,
    pub counter: RawCounter,
    pub working_set_bytes: f64,
}

#[derive(Debug, Clone, Default)]
pub(super) struct Sample {
    pub cpu_total: Option<RawCounter>,
    pub cpu_cores: Vec<(u32, RawCounter)>,
    pub processes: HashMap<u32, RawProcess>,
    pub ram_total_kb: f64,
    pub ram_free_kb: f64,
}

/// CPU inverse (`PERF_100NSEC_TIMER_INV`) : 100 * (1 - delta ticks / delta temps).
fn busy_percent(prev: RawCounter, cur: RawCounter) -> f64 {
    let elapsed = cur.timestamp - prev.timestamp;
    if elapsed <= 0.0 {
        return 0.0;
    }
    (100.0 * (1.0 - (cur.ticks - prev.ticks) / elapsed)).clamp(0.0, 100.0)
}

/// Temps processeur d'un processus (`PERF_100NSEC_TIMER`) en pourcentage d'un coeur.
fn process_core_percent(prev: RawCounter, cur: RawCounter) -> f64 {
    let elapsed = cur.timestamp - prev.timestamp;
    if elapsed <= 0.0 {
        return 0.0;
    }
    (100.0 * (cur.ticks - prev.ticks) / elapsed).max(0.0)
}

pub(super) fn metrics_between(prev: &Sample, cur: &Sample) -> SystemMetrics {
    let cpu_usage = match (prev.cpu_total, cur.cpu_total) {
        (Some(p), Some(c)) => busy_percent(p, c),
        _ => 0.0,
    };

    let mut cores: Vec<(u32, f64)> = cur
        .cpu_cores
        .iter()
        .map(|(idx, c)| {
            let usage = prev
                .cpu_cores
                .iter()
                .find(|(i, _)| i == idx)
                .map_or(0.0, |(_, p)| busy_percent(*p, *c));
            (*idx, usage)
        })
        .collect();
    cores.sort_by_key(|(idx, _)| *idx);
    let cpu_per_core: Vec<f64> = cores.into_iter().map(|(_, usage)| usage).collect();

    let used_kb = (cur.ram_total_kb - cur.ram_free_kb).max(0.0);
    let ram_percent = if cur.ram_total_kb > 0.0 {
        (used_kb / cur.ram_total_kb * 100.0).round().clamp(0.0, 100.0)
    } else {
        0.0
    };

    let logical_cores = cpu_per_core.len().max(1) as f64;
    let mut top_processes: Vec<ProcessMetric> = cur
        .processes
        .iter()
        .filter(|(pid, p)| **pid != 0 && !p.name.is_empty() && p.name != "_Total" && !p.name.eq_ignore_ascii_case("Idle"))
        .map(|(pid, p)| {
            let core_percent = prev
                .processes
                .get(pid)
                .map_or(0.0, |before| process_core_percent(before.counter, p.counter));
            let cpu_percent = (core_percent / logical_cores).clamp(0.0, 100.0);
            ProcessMetric {
                name: p.name.clone(),
                pid: *pid,
                cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                memory_mb: (p.working_set_bytes / constants::MB * 10.0).round() / 10.0,
            }
        })
        .collect();
    top_processes.sort_by(|a, b| {
        b.cpu_percent
            .partial_cmp(&a.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.pid.cmp(&b.pid))
    });
    top_processes.truncate(constants::TOP_PROCESSES_MONITOR);

    SystemMetrics {
        cpu_usage,
        cpu_per_core,
        ram_used_gb: kb_to_gb_rounded(used_kb),
        ram_total_gb: kb_to_gb_rounded(cur.ram_total_kb),
        ram_percent,
        top_processes,
    }
}

#[cfg(test)]
mod tests;
