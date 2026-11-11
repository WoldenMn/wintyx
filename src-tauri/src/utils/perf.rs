use std::fs::{File, OpenOptions};
#[cfg(any(debug_assertions, feature = "perf"))]
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[allow(dead_code)]
static BACKEND_FILE: OnceLock<Option<Mutex<File>>> = OnceLock::new();
#[allow(dead_code)]
static FRONTEND_FILE: OnceLock<Option<Mutex<File>>> = OnceLock::new();

const PERF_FILE_MAX_BYTES: u64 = 10 * 1024 * 1024; // 10 MB

#[allow(dead_code)]
pub fn perf_log_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join("Wintyx").join("Logs")
}

fn rotate_if_needed(path: &std::path::Path) {
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > PERF_FILE_MAX_BYTES {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let bak = path.with_extension(format!("{ts}.bak"));
            let _ = std::fs::rename(path, bak);
        }
    }
}

#[allow(dead_code)]
fn open_append(name: &str) -> std::io::Result<File> {
    let dir = perf_log_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(name);
    rotate_if_needed(&path);
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
}

#[allow(dead_code)]
#[cfg(all(any(debug_assertions, feature = "perf"), not(test)))]
pub fn record_backend(cmd: &str, duration_us: u64) {
    let file = BACKEND_FILE.get_or_init(|| match open_append("perf-backend.csv") {
        Ok(f) => Some(Mutex::new(f)),
        Err(_) => None,
    });
    let Some(mutex) = file else { return };
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let line = format!("{ts},{cmd},{duration_us}\n");
    if let Ok(mut f) = mutex.lock() {
        let _ = f.write_all(line.as_bytes());
    }
}

#[allow(dead_code)]
#[cfg(any(not(any(debug_assertions, feature = "perf")), test))]
pub fn record_backend(_cmd: &str, _duration_us: u64) {}

#[cfg(any(debug_assertions, feature = "perf"))]
pub fn frontend_file() -> Option<&'static Mutex<File>> {
    let slot = FRONTEND_FILE.get_or_init(|| match open_append("perf-frontend.jsonl") {
        Ok(f) => Some(Mutex::new(f)),
        Err(_) => None,
    });
    slot.as_ref()
}

#[cfg(any(debug_assertions, feature = "perf"))]
pub struct PerfGuard {
    name: &'static str,
    start: std::time::Instant,
}

#[cfg(any(debug_assertions, feature = "perf"))]
impl PerfGuard {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: std::time::Instant::now(),
        }
    }
}

#[cfg(any(debug_assertions, feature = "perf"))]
impl Drop for PerfGuard {
    fn drop(&mut self) {
        record_backend(self.name, self.start.elapsed().as_micros() as u64);
    }
}

#[macro_export]
macro_rules! perf_timed {
    ($name:expr, $body:expr) => {{
        #[cfg(any(debug_assertions, feature = "perf"))]
        let _perf_guard = $crate::utils::perf::PerfGuard::new($name);
        $body
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn unique_temp_path() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("wintyx-perf-test-{}-{}", std::process::id(), id))
    }

    #[test]
    fn record_backend_appends_csv_line() {
        let path = unique_temp_path();
        std::fs::create_dir_all(&path).unwrap();
        let csv = path.join("perf-backend.csv");
        let mut f = OpenOptions::new().create(true).append(true).open(&csv).unwrap();
        let line = format!("{},{},{}\n", 1234567890u128, "test_cmd", 4200u64);
        f.write_all(line.as_bytes()).unwrap();
        drop(f);
        let content = std::fs::read_to_string(&csv).unwrap();
        assert!(content.contains("test_cmd"));
        assert!(content.contains("4200"));
        std::fs::remove_dir_all(&path).ok();
    }

    #[test]
    fn record_backend_format_csv_with_three_fields() {
        let line = format!("{ts},{cmd},{dur}\n", ts = 1u128, cmd = "x", dur = 2u64);
        let parts: Vec<&str> = line.trim_end().split(',').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "1");
        assert_eq!(parts[1], "x");
        assert_eq!(parts[2], "2");
    }

    #[test]
    fn perf_timed_macro_compiles_and_returns_body_value() {
        let x = crate::perf_timed!("test_macro", 42 + 1);
        assert_eq!(x, 43);
    }

    #[test]
    fn perf_timed_macro_can_wrap_result_type() {
        let r: Result<u32, &str> = crate::perf_timed!("test_macro_result", Ok(7));
        assert_eq!(r.unwrap(), 7);
    }

    #[test]
    fn perf_timed_macro_captures_early_return_via_question_mark() {
        fn inner() -> Result<i32, &'static str> {
            let _x: i32 = crate::perf_timed!("test_early_return", {
                let r: Result<i32, &'static str> = Err("early");
                r?
            });
            Ok(99)
        }
        assert_eq!(inner(), Err("early"));
    }

    #[test]
    fn rotate_if_needed_renames_large_file() {
        let dir = unique_temp_path();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test-rotate.log");
        let big = vec![b'x'; (PERF_FILE_MAX_BYTES + 1) as usize];
        std::fs::write(&path, &big).unwrap();
        super::rotate_if_needed(&path);
        let backups: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".bak"))
            .collect();
        assert!(!backups.is_empty(), "expected at least one .bak file");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rotate_if_needed_keeps_small_file() {
        let dir = unique_temp_path();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test-small.log");
        std::fs::write(&path, b"small content").unwrap();
        super::rotate_if_needed(&path);
        assert!(path.exists(), "small file should not be rotated");
        std::fs::remove_dir_all(&dir).ok();
    }
}
