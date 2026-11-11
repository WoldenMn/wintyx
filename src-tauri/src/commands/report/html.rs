//! Helpers de presentation HTML + feuille de style du rapport systeme.
//! Separe de l'orchestration (`report/mod.rs`).

pub(super) fn section(title: &str, content: &str) -> String {
    format!("<div class=\"section\">\n<h2>{}</h2>\n{}\n</div>\n", title, content)
}

pub(super) fn row(label: &str, value: &str) -> String {
    format!("<tr><td class=\"label\">{}</td><td>{}</td></tr>", label, value)
}

pub(super) fn score_row(category: &str, score: u32) -> String {
    let (css, status) = if score >= 75 {
        ("good", "Bon")
    } else if score >= 40 {
        ("warn", "À surveiller")
    } else {
        ("bad", "À risque")
    };
    format!("<tr><td>{}</td><td class=\"{}\">{:.0}%</td><td>{}</td></tr>", category, css, score, status)
}

pub(super) fn firewall_table(profiles: &[crate::commands::network::FirewallProfile]) -> String {
    let mut html = String::from("<table><tr><th>Profil</th><th>Statut</th></tr>");
    for p in profiles {
        let (css, status) = match p.is_enabled {
            Some(true) => ("good", "Activé"),
            Some(false) => ("bad", "DÉSACTIVÉ"),
            None => ("warn", "Inconnu"),
        };
        html.push_str(&format!("<tr><td>{}</td><td class=\"{}\">{}</td></tr>", escape(&p.name), css, status));
    }
    html.push_str("</table>");
    html
}

/// Echappe le texte pour insertion HTML. A appliquer a TOUT champ provenant de
/// WMI / PowerShell / env (defense-in-depth : le rapport est rendu hors CSP app).
pub(super) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub(super) const REPORT_CSS: &str = r#"
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: 'Segoe UI', Tahoma, sans-serif; background: #0a0a14; color: #e0e0e0; padding: 20px; }
.header { text-align: center; padding: 30px; background: linear-gradient(135deg, #1a1a2e, #16213e); border-radius: 12px; margin-bottom: 20px; }
.header h1 { color: #7c5cfc; font-size: 28px; margin-bottom: 8px; }
.header p { color: #8888aa; font-size: 14px; }
.section { background: #12121e; border-radius: 10px; padding: 20px; margin-bottom: 16px; border: 1px solid #1e1e30; }
.section h2 { color: #7c5cfc; font-size: 18px; margin-bottom: 12px; border-bottom: 1px solid #1e1e30; padding-bottom: 8px; }
table { width: 100%; border-collapse: collapse; }
th { background: #1a1a2e; color: #7c5cfc; text-align: left; padding: 8px 12px; font-weight: 600; }
td { padding: 6px 12px; border-bottom: 1px solid #1a1a2e; }
td.label { color: #8888aa; width: 200px; }
code { background: #1a1a2e; padding: 2px 6px; border-radius: 4px; font-size: 12px; }
.good { color: #22c55e; font-weight: 600; }
.warn { color: #f59e0b; font-weight: 600; }
.bad { color: #ef4444; font-weight: 600; }
.footer { text-align: center; color: #555; padding: 20px; font-size: 12px; }
p.good { color: #22c55e; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_ampersand() {
        assert_eq!(escape("A & B"), "A &amp; B");
    }

    #[test]
    fn escape_lt_gt() {
        assert_eq!(escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn escape_quotes() {
        assert_eq!(escape("\"hello\""), "&quot;hello&quot;");
        assert_eq!(escape("it's"), "it&#39;s");
    }

    #[test]
    fn escape_combined() {
        assert_eq!(
            escape("<a href=\"x\">A & 'B'</a>"),
            "&lt;a href=&quot;x&quot;&gt;A &amp; &#39;B&#39;&lt;/a&gt;"
        );
    }

    #[test]
    fn escape_empty_and_plain_text() {
        assert_eq!(escape(""), "");
        assert_eq!(escape("hello world"), "hello world");
    }

    #[test]
    fn score_row_good_range() {
        let html = score_row("Cat", 80);
        assert!(html.contains("good"));
        assert!(html.contains("Bon"));
        assert!(html.contains("80%"));
    }

    #[test]
    fn score_row_warn_range() {
        let html = score_row("Cat", 50);
        assert!(html.contains("warn"));
        assert!(html.contains("À surveiller"));
    }

    #[test]
    fn score_row_bad_range() {
        let html = score_row("Cat", 20);
        assert!(html.contains("bad"));
        assert!(html.contains("À risque"));
    }

    #[test]
    fn score_row_boundary_75() {
        assert!(score_row("Cat", 75).contains("good"));
        assert!(score_row("Cat", 74).contains("warn"));
    }

    #[test]
    fn score_row_boundary_40() {
        assert!(score_row("Cat", 40).contains("warn"));
        assert!(score_row("Cat", 39).contains("bad"));
    }

    #[test]
    fn section_wraps_content() {
        let html = section("Title", "<p>body</p>");
        assert!(html.contains("<h2>Title</h2>"));
        assert!(html.contains("<p>body</p>"));
    }

    #[test]
    fn firewall_table_never_reports_unknown_as_disabled() {
        use crate::commands::network::FirewallProfile;
        let profile = |name: &str, is_enabled| FirewallProfile { name: name.into(), is_enabled };
        let html = firewall_table(&[profile("Domaine", Some(true)), profile("Privé", Some(false)), profile("Public", None)]);
        assert!(html.contains("<td>Domaine</td><td class=\"good\">Activé</td>"));
        assert!(html.contains("<td>Privé</td><td class=\"bad\">DÉSACTIVÉ</td>"));
        assert!(html.contains("<td>Public</td><td class=\"warn\">Inconnu</td>"));
    }

    #[test]
    fn row_formats_label_value() {
        let html = row("Label", "Value");
        assert!(html.contains("<td class=\"label\">Label</td>"));
        assert!(html.contains("<td>Value</td>"));
    }
}
