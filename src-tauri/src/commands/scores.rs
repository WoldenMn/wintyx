use serde::Serialize;

use super::{bloatware, data_protection, hardening, performance, services};

#[derive(Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AllScores {
    pub data_protection: u32,
    pub services: u32,
    pub hardening: u32,
    pub performance: u32,
    pub bloatware: Option<u32>,
}

/// Agrege les 5 scores de domaine en un seul aller-retour IPC (PERF-01 v2.16.0),
/// au lieu de 5 invokes Promise.all cote ScoresContext. Les collectes sont
/// independantes (registre / sc.exe / PowerShell) -> parallelisees via
/// `std::thread::scope`, meme pattern que threat_scanner. Chaque fonction de
/// score est infaillible (degrade en 0 avec log en interne) ; un panic de
/// thread est re-propage pour conserver la semantique tout-ou-rien de
/// l'ancien Promise.all.
#[tauri::command]
#[specta::specta]
pub async fn get_all_scores() -> AllScores {
    crate::utils::blocking::read_value(get_all_scores_sync).await
}

pub fn get_all_scores_sync() -> AllScores {
    std::thread::scope(|s| {
        let data_protection = s.spawn(data_protection::get_data_protection_score);
        let services = s.spawn(services::get_services_score_sync);
        let hardening = s.spawn(hardening::get_hardening_score_sync);
        let performance = s.spawn(performance::get_performance_score_sync);
        let bloatware = s.spawn(bloatware::get_bloatware_score_sync);
        AllScores {
            data_protection: join_score(data_protection),
            services: join_score(services),
            hardening: join_score(hardening),
            performance: join_score(performance),
            bloatware: join_score(bloatware),
        }
    })
}

fn join_score<T>(handle: std::thread::ScopedJoinHandle<'_, T>) -> T {
    handle
        .join()
        .unwrap_or_else(|payload| std::panic::resume_unwind(payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_scores_serializes_camel_case() {
        let scores = AllScores {
            data_protection: 1,
            services: 2,
            hardening: 3,
            performance: 4,
            bloatware: Some(5),
        };
        let json = serde_json::to_string(&scores).expect("serialisation AllScores");
        assert!(json.contains("\"dataProtection\":1"), "json = {}", json);
        assert!(json.contains("\"services\":2"), "json = {}", json);
        assert!(json.contains("\"hardening\":3"), "json = {}", json);
        assert!(json.contains("\"performance\":4"), "json = {}", json);
        assert!(json.contains("\"bloatware\":5"), "json = {}", json);
    }

    #[test]
    fn an_unavailable_domain_serializes_as_null() {
        let scores = AllScores {
            data_protection: 1,
            services: 2,
            hardening: 3,
            performance: 4,
            bloatware: None,
        };
        let json = serde_json::to_string(&scores).expect("serialisation AllScores");
        assert!(json.contains("\"bloatware\":null"), "json = {}", json);
    }
}
