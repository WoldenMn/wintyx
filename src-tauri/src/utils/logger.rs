use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use chrono::Local;
use super::{constants, paths};

const ROTATE_CHECK_EVERY: u32 = 100;
static WRITE_COUNT: AtomicU32 = AtomicU32::new(0);

static WRITE_LOCK: Mutex<()> = Mutex::new(());

fn log_dir() -> PathBuf {
    paths::logs_dir()
}

pub(crate) fn log_file() -> PathBuf {
    log_dir().join(format!("wintyx_{}.log", Local::now().format("%Y%m%d")))
}

pub fn init() {
    let _ = fs::create_dir_all(log_dir());
    purge_old_logs();
}

pub fn info(category: &str, message: &str) {
    append(&format_entry("INFO ", category, message));
}

pub fn warn(category: &str, message: &str) {
    append(&format_entry("WARN ", category, message));
}

pub fn error(category: &str, message: &str) {
    append(&format_entry("ERROR", category, message));
}

fn format_entry(level: &str, category: &str, message: &str) -> String {
    format!(
        "[{}] [{}] [{}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        level,
        single_line(category),
        single_line(message)
    )
}

/// Une entrée du journal tient sur une ligne : les retours à la ligne deviennent la
/// suite littérale `\n`, comme pour `log_frontend_error`.
fn single_line(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n").replace('\n', "\\n")
}

fn append(line: &str) {
    let path = log_file();
    let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = fs::create_dir_all(log_dir());
    let count = WRITE_COUNT.fetch_add(1, Ordering::Relaxed);
    if count.is_multiple_of(ROTATE_CHECK_EVERY) {
        rotate_if_too_big(&path);
    }
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(format!("{}\n", line).as_bytes());
    }
}

fn rotate_if_too_big(path: &Path) {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return,
    };
    if meta.len() <= constants::MAX_LOG_FILE_BYTES {
        return;
    }
    let parent = match path.parent() {
        Some(p) => p,
        None => return,
    };
    let stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return,
    };
    let ts = Local::now().format("%H%M%S");
    let rotated = parent.join(format!("{}_{}.log", stem, ts));
    let _ = fs::rename(path, rotated);
}

fn purge_old_logs() {
    if let Ok(entries) = fs::read_dir(log_dir()) {
        let cutoff = Local::now() - chrono::Duration::days(constants::LOG_RETENTION_DAYS);
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    let modified_time: chrono::DateTime<Local> = modified.into();
                    if modified_time < cutoff {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_dir_is_redirected_in_tests() {
        assert!(log_dir().starts_with(std::env::temp_dir()));
        assert!(log_dir().to_string_lossy().contains("wintyx-test-"));
    }

    #[test]
    fn multi_line_messages_are_written_on_one_line() {
        let entry = format_entry("WARN ", "backup", "echec\r\nAu caractere Ligne:1\n+ CategoryInfo");
        assert!(!entry.contains('\n') && !entry.contains('\r'));
        assert!(entry.ends_with("echec\\nAu caractere Ligne:1\\n+ CategoryInfo"));
    }

    #[test]
    fn single_line_messages_are_untouched() {
        assert!(format_entry("INFO ", "app", "simple").ends_with("[INFO ] [app] simple"));
    }

    #[test]
    fn concurrent_writes_stay_one_line_each() {
        let marker = format!("conc-{}", std::process::id());
        let handles: Vec<_> = (0..8)
            .map(|t| {
                let marker = marker.clone();
                std::thread::spawn(move || {
                    for i in 0..50 {
                        info("test", &format!("{} t{} i{} {}", marker, t, i, "x".repeat(200)));
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let content = fs::read_to_string(log_file()).unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| l.contains(&marker)).collect();
        assert_eq!(lines.len(), 400);
        for l in lines {
            assert!(l.starts_with('[') && l.ends_with(&"x".repeat(200)), "{}", l);
        }
    }
}
