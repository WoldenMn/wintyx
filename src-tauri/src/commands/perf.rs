use serde::{Deserialize, Serialize};
use crate::utils::errors::CommandError;

const MAX_SAMPLES_PER_BATCH: usize = 10_000;

#[derive(Deserialize, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FrontendSample {
    pub ts: i64,
    pub cmd: String,
    pub args_key: String,
    pub duration_ms: f64,
    pub cache_hit: bool,
}

#[tauri::command]
#[specta::specta]
pub fn perf_dump_frontend(samples: Vec<FrontendSample>) -> Result<(), CommandError> {
    if samples.len() > MAX_SAMPLES_PER_BATCH {
        return Err(CommandError::InvalidInput(format!(
            "perf_dump_frontend batch too large: {} samples (max {MAX_SAMPLES_PER_BATCH})",
            samples.len()
        )));
    }
    #[cfg(any(debug_assertions, feature = "perf"))]
    {
        use std::io::Write;
        if let Some(file) = crate::utils::perf::frontend_file() {
            if let Ok(mut f) = file.lock() {
                for s in &samples {
                    let line = serde_json::to_string(s).unwrap_or_default();
                    let _ = writeln!(f, "{line}");
                }
            }
        }
    }
    let _ = samples;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> FrontendSample {
        FrontendSample {
            ts: 0,
            cmd: "x".to_string(),
            args_key: String::new(),
            duration_ms: 1.0,
            cache_hit: false,
        }
    }

    #[test]
    fn rejects_batch_over_max() {
        // MAX+1 samples trigger the early-return guard BEFORE any file I/O,
        // so this test never touches the real log file.
        let samples: Vec<FrontendSample> = (0..=MAX_SAMPLES_PER_BATCH).map(|_| sample()).collect();
        let result = perf_dump_frontend(samples);
        assert!(matches!(result, Err(CommandError::InvalidInput(_))));
    }
}
