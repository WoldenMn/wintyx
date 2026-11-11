use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use crate::utils::errors::CommandError;
use crate::utils::{json_store, paths};

const SCORES_HISTORY_MAX: usize = 90;

#[derive(Serialize, Deserialize, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DomainScores {
    pub data_protection: u8,
    pub services: u8,
    pub hardening: u8,
    pub performance: u8,
    pub bloatware: u8,
}

#[derive(Serialize, Deserialize, Clone, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ScoreSnapshot {
    /// Cle d'upsert journalier, format "YYYY-MM-DD".
    pub date: String,
    /// Horodatage complet "YYYY-MM-DD HH:MM:SS".
    pub timestamp: String,
    pub global_score: u8,
    pub scores: DomainScores,
}

/// Moyenne equipondere des 5 domaines, arrondie, plafonnee a 100.
pub(crate) fn compute_global(s: &DomainScores) -> u8 {
    let sum = s.data_protection as u32
        + s.services as u32
        + s.hardening as u32
        + s.performance as u32
        + s.bloatware as u32;
    ((sum as f64 / 5.0).round() as u32).min(100) as u8
}

/// Upsert par jour : remplace l'entree dont `date == today` si elle existe,
/// sinon l'ajoute en fin ; conserve ensuite les SCORES_HISTORY_MAX plus recentes.
/// `today` est injecte (pas d'acces horloge) pour rendre la fonction testable.
pub(crate) fn upsert_today(
    mut history: Vec<ScoreSnapshot>,
    snap: ScoreSnapshot,
    today: &str,
) -> Vec<ScoreSnapshot> {
    if let Some(existing) = history.iter_mut().find(|e| e.date == today) {
        *existing = snap;
    } else {
        history.push(snap);
    }
    if history.len() > SCORES_HISTORY_MAX {
        let drop = history.len() - SCORES_HISTORY_MAX;
        history.drain(..drop);
    }
    history
}

fn load_history_from(path: &Path) -> Result<Vec<ScoreSnapshot>, CommandError> {
    json_store::load_or_quarantine(path, "scores_history")
}

static HISTORY_LOCK: Mutex<()> = Mutex::new(());

/// Section critique chargement -> upsert -> ecriture : sans le verrou, deux
/// enregistrements concurrents partent du meme historique et l'un efface l'autre.
fn record_snapshot_at(path: &Path, snap: ScoreSnapshot) -> Result<(), CommandError> {
    let _guard = HISTORY_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let today = snap.date.clone();
    let history = upsert_today(load_history_from(path)?, snap, &today);
    json_store::write_json_atomic(path, &history)
}

#[tauri::command]
#[specta::specta]
pub fn record_score_snapshot(scores: DomainScores) -> Result<(), CommandError> {
    let now = chrono::Local::now();
    let snap = ScoreSnapshot {
        date: now.format("%Y-%m-%d").to_string(),
        timestamp: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        global_score: compute_global(&scores),
        scores,
    };
    record_snapshot_at(&paths::scores_history_path(), snap)
}

#[tauri::command]
#[specta::specta]
pub fn get_score_history() -> Result<Vec<ScoreSnapshot>, CommandError> {
    load_history_from(&paths::scores_history_path())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(date: &str, g: u8) -> ScoreSnapshot {
        ScoreSnapshot {
            date: date.to_string(),
            timestamp: format!("{} 12:00:00", date),
            global_score: g,
            scores: DomainScores { data_protection: g, services: g, hardening: g, performance: g, bloatware: g },
        }
    }

    #[test]
    fn compute_global_rounds_average() {
        let s = DomainScores { data_protection: 80, services: 80, hardening: 80, performance: 80, bloatware: 82 };
        assert_eq!(compute_global(&s), 80);
        let s2 = DomainScores { data_protection: 90, services: 90, hardening: 90, performance: 90, bloatware: 93 };
        assert_eq!(compute_global(&s2), 91);
    }

    #[test]
    fn upsert_appends_new_day() {
        let h = upsert_today(vec![snap("2026-06-23", 50)], snap("2026-06-24", 60), "2026-06-24");
        assert_eq!(h.len(), 2);
        assert_eq!(h[1].date, "2026-06-24");
        assert_eq!(h[1].global_score, 60);
    }

    #[test]
    fn upsert_overwrites_same_day() {
        let h = upsert_today(vec![snap("2026-06-24", 40)], snap("2026-06-24", 70), "2026-06-24");
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].global_score, 70);
    }

    #[test]
    fn upsert_caps_at_90_keeping_recent() {
        let mut h: Vec<ScoreSnapshot> = (1..=90).map(|i| snap(&format!("2026-01-{:02}", (i % 28) + 1), i as u8)).collect();
        h = upsert_today(h, snap("2026-12-31", 99), "2026-12-31");
        assert_eq!(h.len(), 90);
        assert_eq!(h.last().unwrap().date, "2026-12-31");
    }

    #[test]
    fn snapshot_round_trips_through_json() {
        let history = vec![snap("2026-06-24", 75)];
        let json = serde_json::to_string(&history).unwrap();
        let back: Vec<ScoreSnapshot> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].global_score, 75);
        assert_eq!(back[0].scores.data_protection, 75);
        assert!(json.contains("globalScore"));
        assert!(json.contains("dataProtection"));
    }

    #[test]
    fn concurrent_records_lose_no_snapshot() {
        let path = std::env::temp_dir()
            .join(format!("wintyx-scores-concurrent-{}", std::process::id()))
            .join("scores_history.json");
        let _ = std::fs::remove_file(&path);
        let threads: Vec<_> = (1..=12)
            .map(|day| {
                let path = path.clone();
                std::thread::spawn(move || {
                    record_snapshot_at(&path, snap(&format!("2026-01-{:02}", day), day as u8))
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap().unwrap();
        }
        assert_eq!(load_history_from(&path).unwrap().len(), 12);
    }

    #[test]
    fn corrupted_history_is_quarantined_not_overwritten() {
        let dir = std::env::temp_dir().join(format!("wintyx-scores-corrupt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("scores_history.json");
        std::fs::write(&path, "[{\"date\":").unwrap();
        record_snapshot_at(&path, snap("2026-06-24", 60)).unwrap();
        assert_eq!(load_history_from(&path).unwrap().len(), 1);
        let kept: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".corrompu-"))
            .collect();
        assert_eq!(kept.len(), 1);
        assert_eq!(std::fs::read_to_string(kept[0].path()).unwrap(), "[{\"date\":");
    }

    #[test]
    fn unreadable_history_is_an_error_not_an_empty_list() {
        let dir = std::env::temp_dir().join(format!("wintyx-scores-ioerr-{}", std::process::id()));
        let path = dir.join("scores_history.json");
        std::fs::create_dir_all(&path).unwrap();
        assert!(load_history_from(&path).is_err());
        assert!(record_snapshot_at(&path, snap("2026-06-24", 60)).is_err());
    }
}
