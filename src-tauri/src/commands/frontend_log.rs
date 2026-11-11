use serde::Deserialize;

use crate::utils::logger;

const MAX_MESSAGE_CHARS: usize = 500;
const CATEGORY: &str = "frontend";

#[derive(Debug, Clone, Copy, Deserialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum FrontendLogLevel {
    Warn,
    Error,
}

fn sanitize_message(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len().min(MAX_MESSAGE_CHARS + 16));
    for (count, c) in raw.chars().enumerate() {
        if count == MAX_MESSAGE_CHARS {
            out.push_str("...(tronque)");
            break;
        }
        match c {
            '\r' => {}
            '\n' => out.push_str("\\n"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

#[tauri::command]
#[specta::specta]
pub fn log_frontend_error(level: FrontendLogLevel, message: String) {
    let clean = sanitize_message(&message);
    match level {
        FrontendLogLevel::Warn => logger::warn(CATEGORY, &clean),
        FrontendLogLevel::Error => logger::error(CATEGORY, &clean),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_message_is_kept() {
        assert_eq!(sanitize_message("boom"), "boom");
    }

    #[test]
    fn newlines_cannot_forge_a_log_line() {
        let out = sanitize_message("a\r\n[2026-01-01 00:00:00] [ERROR] [fake] b");
        assert!(!out.contains('\n') && !out.contains('\r'));
        assert_eq!(out, "a\\n[2026-01-01 00:00:00] [ERROR] [fake] b");
    }

    #[test]
    fn other_control_characters_become_spaces() {
        assert_eq!(sanitize_message("a\u{0}b\tc\u{1b}d"), "a b c d");
    }

    #[test]
    fn long_message_is_truncated_on_char_boundary() {
        let out = sanitize_message(&"é".repeat(MAX_MESSAGE_CHARS + 50));
        assert_eq!(out.chars().filter(|c| *c == 'é').count(), MAX_MESSAGE_CHARS);
        assert!(out.ends_with("...(tronque)"));
    }

    #[test]
    fn message_at_the_limit_is_not_truncated() {
        let msg = "x".repeat(MAX_MESSAGE_CHARS);
        assert_eq!(sanitize_message(&msg), msg);
    }

    #[test]
    fn command_writes_a_frontend_line_to_the_log() {
        let marker = format!("fe-{}", std::process::id());
        log_frontend_error(FrontendLogLevel::Error, format!("{}\nsecond", marker));
        let path = crate::utils::paths::logs_dir()
            .join(format!("wintyx_{}.log", chrono::Local::now().format("%Y%m%d")));
        let content = std::fs::read_to_string(path).unwrap();
        let line = content.lines().find(|l| l.contains(&marker)).expect("ligne absente");
        assert!(line.contains("[ERROR] [frontend]"), "{}", line);
        assert!(line.ends_with("second"), "{}", line);
    }
}
