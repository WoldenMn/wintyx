use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct BatchOptions {
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, specta::Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub errors: Vec<String>,
    pub dry_run: bool,
    pub preview_items: Vec<String>,
}

impl BatchResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_dry_run() -> Self {
        Self {
            dry_run: true,
            ..Self::default()
        }
    }

    pub fn record_success(&mut self) {
        self.total += 1;
        self.success += 1;
    }

    pub fn record_failure(&mut self, context: impl Into<String>) {
        self.total += 1;
        self.failed += 1;
        self.errors.push(context.into());
    }

    pub fn record<T, E: std::fmt::Display>(
        &mut self,
        context: impl Into<String>,
        outcome: Result<T, E>,
    ) {
        match outcome {
            Ok(_) => self.record_success(),
            Err(e) => self.record_failure(format!("{}: {}", context.into(), e)),
        }
    }

    pub fn record_preview(&mut self, item_name: impl Into<String>) {
        self.total += 1;
        self.preview_items.push(item_name.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_empty() {
        let r = BatchResult::new();
        assert_eq!(r.total, 0);
        assert_eq!(r.success, 0);
        assert_eq!(r.failed, 0);
        assert!(r.errors.is_empty());
        assert!(!r.dry_run);
        assert!(r.preview_items.is_empty());
    }

    #[test]
    fn record_success_increments_totals() {
        let mut r = BatchResult::new();
        r.record_success();
        r.record_success();
        assert_eq!(r.total, 2);
        assert_eq!(r.success, 2);
        assert_eq!(r.failed, 0);
    }

    #[test]
    fn record_failure_stores_context() {
        let mut r = BatchResult::new();
        r.record_failure("item-A: boom");
        assert_eq!(r.total, 1);
        assert_eq!(r.failed, 1);
        assert_eq!(r.errors, vec!["item-A: boom".to_string()]);
    }

    #[test]
    fn record_helper_dispatches() {
        let mut r = BatchResult::new();
        r.record::<(), &str>("ok-item", Ok(()));
        r.record::<(), &str>("bad-item", Err("nope"));
        assert_eq!(r.total, 2);
        assert_eq!(r.success, 1);
        assert_eq!(r.failed, 1);
        assert_eq!(r.errors, vec!["bad-item: nope".to_string()]);
    }

    #[test]
    fn new_dry_run_sets_flag() {
        let r = BatchResult::new_dry_run();
        assert!(r.dry_run);
        assert_eq!(r.total, 0);
        assert!(r.preview_items.is_empty());
    }

    #[test]
    fn record_preview_adds_item_name() {
        let mut r = BatchResult::new_dry_run();
        r.record_preview("SMBv1");
        r.record_preview("RDP");
        assert_eq!(r.total, 2);
        assert_eq!(r.success, 0);
        assert_eq!(r.failed, 0);
        assert_eq!(
            r.preview_items,
            vec!["SMBv1".to_string(), "RDP".to_string()]
        );
    }

    #[test]
    fn batch_result_serializes_camel_case() {
        let r = BatchResult::new_dry_run();
        let json = serde_json::to_value(&r).unwrap();
        assert!(json.get("dryRun").is_some());
        assert!(json.get("previewItems").is_some());
        assert_eq!(json.get("dryRun").unwrap(), &serde_json::json!(true));
    }

    #[test]
    fn batch_options_default_is_not_dry_run() {
        let opts = BatchOptions::default();
        assert!(!opts.dry_run);
    }

    #[test]
    fn batch_options_deserializes_from_json() {
        let json = serde_json::json!({"dryRun": true});
        let opts: BatchOptions = serde_json::from_value(json).unwrap();
        assert!(opts.dry_run);
    }

    #[test]
    fn batch_options_ignores_the_retired_filter_key_sent_by_an_old_client() {
        let json = serde_json::json!({"dryRun": true, "filter": ["a", "b"]});
        let opts: BatchOptions = serde_json::from_value(json).unwrap();
        assert!(opts.dry_run);
    }

    #[test]
    fn batch_options_handles_missing_fields() {
        let json = serde_json::json!({});
        let opts: BatchOptions = serde_json::from_value(json).unwrap();
        assert!(!opts.dry_run);
    }
}
