use super::*;

fn failure() -> CommandError {
    CommandError::ProcessFailed {
        command: "Get-ScheduledTask".into(),
        detail: "accès <refusé> & délai".into(),
    }
}

fn task(name: &str, suspicious: bool) -> ScheduledTaskInfo {
    ScheduledTaskInfo {
        name: name.into(),
        path: "\\".into(),
        state: "Ready".into(),
        author: String::new(),
        description: String::new(),
        last_run_time: String::new(),
        next_run_time: String::new(),
        action_path: "mshta.exe".into(),
        action_arguments: String::new(),
        is_suspicious: suspicious,
        needs_review: false,
        suspicious_reason: "mshta".into(),
    }
}

fn bloat(name: &str, installed: bool) -> BloatwareItem {
    BloatwareItem {
        package_name: format!("Pkg.{name}"),
        display_name: name.into(),
        publisher: "Editeur".into(),
        category: "Games".into(),
        is_installed: installed,
        version: "1.0".into(),
    }
}

#[test]
fn a_failed_task_collection_is_never_reported_as_no_suspicious_task() {
    let html = suspicious_tasks_section(&Err(failure()));
    assert!(html.contains("Collecte indisponible"), "{html}");
    assert!(html.contains("class=\"warn\""), "{html}");
    assert!(!html.contains("Aucune tâche"), "{html}");
    assert!(!html.contains("class=\"good\""), "{html}");
}

#[test]
fn the_failure_detail_is_escaped_in_the_report() {
    let html = suspicious_tasks_section(&Err(failure()));
    assert!(html.contains("&lt;refusé&gt; &amp; délai"), "{html}");
    assert!(!html.contains("<refusé>"), "{html}");
}

#[test]
fn an_empty_but_successful_task_collection_is_the_only_green_case() {
    let html = suspicious_tasks_section(&Ok(vec![task("Sain", false)]));
    assert!(html.contains("Aucune tâche planifiée suspecte détectée."), "{html}");
    assert!(html.contains("class=\"good\""), "{html}");
}

#[test]
fn suspicious_tasks_are_listed_and_the_others_are_not() {
    let html = suspicious_tasks_section(&Ok(vec![task("Sain", false), task("Louche", true)]));
    assert!(html.contains("Louche") && !html.contains("Sain"), "{html}");
    assert!(!html.contains("Aucune tâche"), "{html}");
}

#[test]
fn a_failed_application_list_is_never_reported_as_no_bloatware() {
    let html = bloatware_section(&Err(failure()));
    assert!(html.contains("Collecte indisponible"), "{html}");
    assert!(!html.contains("Aucun bloatware"), "{html}");
    assert!(!html.contains("class=\"good\""), "{html}");
}

#[test]
fn bloatware_reports_none_only_when_the_list_was_read_and_empty_of_installed_items() {
    let none_installed = bloatware_section(&Ok(vec![bloat("Jeu", false)]));
    assert!(none_installed.contains("Aucun bloatware détecté."), "{none_installed}");
    let some = bloatware_section(&Ok(vec![bloat("Jeu", true), bloat("Autre", false)]));
    assert!(some.contains("Jeu") && !some.contains("Autre") && !some.contains("Aucun bloatware"), "{some}");
    assert!(some.contains("1 application(s)"), "{some}");
}

#[test]
fn render_or_unavailable_renders_on_success_and_warns_on_failure() {
    let ok = render_or_unavailable("Disques", Ok(3), |n| format!("<p>{n} disques</p>"));
    assert_eq!(ok, "<p>3 disques</p>");
    let ko = render_or_unavailable("Disques", Err::<u32, _>(failure()), |_| unreachable!());
    assert!(ko.contains("<h2>Disques</h2>") && ko.contains("Collecte indisponible"), "{ko}");
}
