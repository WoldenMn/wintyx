//! Scanner de menaces : orchestration des 6 phases de scan.
//!
//! Module decompose :
//! - `scanners` : 6 fonctions `scan_*` (taches planifiees, demarrage, ports,
//!   services, autorun, pare-feu) + helper `extract_exe_path`
//!
//! Ce fichier expose la commande Tauri `run_threat_scan` qui orchestre les
//! 6 phases en sequence, emet des events `threat-scan-progress` au frontend,
//! et calcule le `risk_score` global a partir des poids des ThreatItem.

mod scanners;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tauri::{AppHandle, Emitter};
use tauri_specta::Event;

use crate::utils::errors::CommandError;

// tauri-specta rc.20 derive le nom du canal d'event du nom de struct en kebab-case
// (ThreatScanProgress -> "threat-scan-progress") et ignore #[tauri_specta(event_name)].
// On s'appuie donc sur le nom de struct ; le canal reste "threat-scan-progress",
// coherent avec app.emit/listen et le wrapper genere events.threatScanProgress.
#[derive(Serialize, Deserialize, Clone, specta::Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct ThreatScanProgress {
    pub phase_name: String,
    pub phase_index: u32,
    pub phase_count: u32,
    pub scanned_so_far: u32,
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ThreatScanResult {
    pub risk_score: u32,
    pub total_items_scanned: u32,
    pub threats: Vec<ThreatItem>,
    pub categories: Vec<ThreatCategory>,
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ThreatCategory {
    pub name: String,
    pub scanned: u32,
    pub threats_found: u32,
    pub risk_level: String,
    pub failed: bool,
}

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ThreatItem {
    pub name: String,
    pub category: String,
    pub severity: String,
    pub description: String,
    pub path: String,
    pub can_fix: bool,
    pub fix_id: String,
}

pub(crate) type ScanOutcome = Result<(u32, Vec<ThreatItem>), String>;

static SCANNING: AtomicBool = AtomicBool::new(false);

struct ScanGuard(&'static AtomicBool);

impl ScanGuard {
    fn acquire(flag: &'static AtomicBool) -> Result<Self, CommandError> {
        if flag
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(CommandError::Generic(
                "Un scan de menaces est déjà en cours".into(),
            ));
        }
        Ok(Self(flag))
    }
}

impl Drop for ScanGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

struct ScanPhase {
    name: &'static str,
    scan: fn() -> ScanOutcome,
}

fn failed_category(name: &str) -> ThreatCategory {
    ThreatCategory {
        name: name.into(),
        scanned: 0,
        threats_found: 0,
        risk_level: "unknown".into(),
        failed: true,
    }
}

fn collect_phase(phase: &ScanPhase) -> (u32, Vec<ThreatItem>, ThreatCategory) {
    let (scanned, threats) = match (phase.scan)() {
        Ok(outcome) => outcome,
        Err(detail) => {
            crate::utils::logger::warn(
                "threat_scanner",
                &format!("phase '{}' en échec, résultat non fiable: {}", phase.name, detail),
            );
            return (0, Vec::new(), failed_category(phase.name));
        }
    };
    let count = threats.len() as u32;
    // Niveau de la categorie = plus haute severite presente, PAS le simple nombre
    // d'items : un grand nombre d'items "low" (ex: ports ouverts a verifier) ne
    // doit pas afficher "Eleve" (run-07, anti fear-marketing).
    let risk_level = if threats
        .iter()
        .any(|t| t.severity == "critical" || t.severity == "high")
    {
        "high"
    } else if threats.iter().any(|t| t.severity == "medium") {
        "medium"
    } else {
        "low"
    };
    let category = ThreatCategory {
        name: phase.name.into(),
        scanned,
        threats_found: count,
        risk_level: risk_level.into(),
        failed: false,
    };
    (scanned, threats, category)
}

fn calculate_risk_score(threats: &[ThreatItem]) -> u32 {
    let count = |sev: &str| threats.iter().filter(|t| t.severity == sev).count() as u32;
    let critical = count("critical");
    let high = count("high");
    let medium = count("medium");
    // "low" = informationnel (ex: ports ouverts a verifier). Contribution plafonnee
    // a 5 items pour qu'un grand nombre ne sature pas le score (run-07).
    let low = count("low").min(5);
    critical
        .saturating_mul(25)
        .saturating_add(high.saturating_mul(15))
        .saturating_add(medium.saturating_mul(5))
        .saturating_add(low.saturating_mul(2))
        .min(100)
}

fn emit_progress(app: &AppHandle, progress: ThreatScanProgress) {
    if let Err(e) = app.emit("threat-scan-progress", progress) {
        crate::utils::logger::warn(
            "threat_scanner",
            &format!("emission de la progression echouee: {}", e),
        );
    }
}

// Parallelisation des 6 scanners independants (perf-rust-04) : chacun dans
// son thread (process + lectures registre, thread-safe). L'achevement n'est
// plus sequentiel -> l'event de progression devient "N/phase_count terminees"
// via un compteur atomique. Les resultats sont reordonnes par index d'origine
// pour un affichage de categories stable. Le panic d'un scanner est isole : sa
// categorie revient en `failed`, sans faire echouer tout le scan.
fn run_phases(
    phases: &[ScanPhase],
    report: &(dyn Fn(ThreatScanProgress) + Sync),
) -> (Vec<ThreatItem>, Vec<ThreatCategory>, u32) {
    let phase_count = phases.len() as u32;
    let completed = AtomicU32::new(0);
    let scanned_total = AtomicU32::new(0);

    let mut results: Vec<(usize, Vec<ThreatItem>, ThreatCategory)> = std::thread::scope(|s| {
        let handles: Vec<_> = phases
            .iter()
            .enumerate()
            .map(|(index, phase)| {
                let completed = &completed;
                let scanned_total = &scanned_total;
                s.spawn(move || {
                    let (scanned, threats, category) = collect_phase(phase);
                    let done = completed.fetch_add(1, Ordering::SeqCst) + 1;
                    let total = scanned_total.fetch_add(scanned, Ordering::SeqCst) + scanned;
                    report(ThreatScanProgress {
                        phase_name: phase.name.into(),
                        phase_index: done.saturating_sub(1),
                        phase_count,
                        scanned_so_far: total,
                    });
                    (index, threats, category)
                })
            })
            .collect();
        handles
            .into_iter()
            .enumerate()
            .map(|(index, handle)| match handle.join() {
                Ok(outcome) => outcome,
                Err(_) => {
                    let name = phases[index].name;
                    crate::utils::logger::warn(
                        "threat_scanner",
                        &format!("phase '{}' interrompue par une panique, résultat non fiable", name),
                    );
                    (index, Vec::new(), failed_category(name))
                }
            })
            .collect()
    });

    results.sort_by_key(|(index, _, _)| *index);

    let mut all_threats = Vec::new();
    let mut categories = Vec::new();
    for (_, threats, category) in results {
        all_threats.extend(threats);
        categories.push(category);
    }
    (all_threats, categories, scanned_total.load(Ordering::SeqCst))
}

#[tauri::command]
#[specta::specta]
pub async fn run_threat_scan(app: AppHandle) -> Result<ThreatScanResult, CommandError> {
    crate::utils::blocking::read_result(move || run_threat_scan_sync(app)).await
}

pub fn run_threat_scan_sync(app: AppHandle) -> Result<ThreatScanResult, CommandError> {
    let _guard = ScanGuard::acquire(&SCANNING)?;
    crate::perf_timed!("run_threat_scan", {
        let phases: Vec<ScanPhase> = vec![
            ScanPhase { name: "Tâches planifiées", scan: scanners::scan_scheduled_tasks },
            ScanPhase { name: "Programmes au démarrage", scan: scanners::scan_startup_entries },
            ScanPhase { name: "Ports ouverts", scan: scanners::scan_open_ports },
            ScanPhase { name: "Services non signés", scan: scanners::scan_unsigned_services },
            ScanPhase { name: "Clés de registre (démarrage auto)", scan: scanners::scan_autorun_registry },
            ScanPhase { name: "Pare-feu", scan: scanners::scan_firewall },
        ];

        let phase_count = phases.len() as u32;
        let (all_threats, categories, total_scanned) =
            run_phases(&phases, &|progress| emit_progress(&app, progress));

        emit_progress(
            &app,
            ThreatScanProgress {
                phase_name: "Finalisation".into(),
                phase_index: phase_count,
                phase_count,
                scanned_so_far: total_scanned,
            },
        );

        let risk_score = calculate_risk_score(&all_threats);

        Ok(ThreatScanResult {
            risk_score,
            total_items_scanned: total_scanned,
            threats: all_threats,
            categories,
        })
    })
}

#[cfg(test)]
mod tests;
