use crate::commands::bloatware::BloatwareItem;
use crate::commands::tasks::ScheduledTaskInfo;
use crate::utils::errors::CommandError;
use crate::utils::logger;

use super::html::{escape, section};
use super::labels;

pub(super) fn unavailable_section(title: &str, error: &CommandError) -> String {
    logger::warn("report", &format!("{} indisponible dans le rapport: {}", title, error));
    section(
        title,
        &format!(
            "<p class=\"warn\">Collecte indisponible : {}. Cette section n'a pas été vérifiée.</p>",
            escape(&error.friendly_message())
        ),
    )
}

pub(super) fn render_or_unavailable<T>(
    title: &str,
    collected: Result<T, CommandError>,
    render: impl FnOnce(T) -> String,
) -> String {
    match collected {
        Ok(value) => render(value),
        Err(e) => unavailable_section(title, &e),
    }
}

pub(super) fn suspicious_tasks_section(tasks: &Result<Vec<ScheduledTaskInfo>, CommandError>) -> String {
    const TITLE: &str = "Tâches planifiées suspectes";
    let all = match tasks {
        Ok(all) => all,
        Err(e) => return unavailable_section(TITLE, e),
    };
    let suspicious: Vec<_> = all.iter().filter(|t| t.is_suspicious).collect();
    if suspicious.is_empty() {
        return section(TITLE, "<p class=\"good\">Aucune tâche planifiée suspecte détectée.</p>");
    }
    let mut html = String::from("<table><tr><th>Nom</th><th>Action</th><th>Raison</th></tr>");
    for t in &suspicious {
        html.push_str(&format!(
            "<tr><td>{}</td><td><code>{}</code></td><td class=\"bad\">{}</td></tr>",
            escape(&t.name),
            escape(&t.action_path),
            escape(&t.suspicious_reason)
        ));
    }
    html.push_str("</table>");
    section(TITLE, &html)
}

pub(super) fn bloatware_section(items: &Result<Vec<BloatwareItem>, CommandError>) -> String {
    const TITLE: &str = "Bloatware détecté";
    let all = match items {
        Ok(all) => all,
        Err(e) => return unavailable_section(TITLE, e),
    };
    let installed: Vec<_> = all.iter().filter(|b| b.is_installed).collect();
    if installed.is_empty() {
        return section(TITLE, "<p class=\"good\">Aucun bloatware détecté.</p>");
    }
    let mut html = String::from("<table><tr><th>Application</th><th>Éditeur</th><th>Catégorie</th></tr>");
    for b in &installed {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape(&b.display_name),
            escape(&b.publisher),
            escape(labels::category(&b.category))
        ));
    }
    html.push_str("</table>");
    html.push_str(&format!("<p>{} application(s) bloatware installée(s).</p>", installed.len()));
    section(TITLE, &html)
}

#[cfg(test)]
mod tests;
