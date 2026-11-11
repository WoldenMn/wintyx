use serde_json::Value;

const REQUIRED_WINDOW_PERMISSIONS: [&str; 5] = [
    "core:window:allow-minimize",
    "core:window:allow-toggle-maximize",
    "core:window:allow-is-maximized",
    "core:window:allow-close",
    "core:window:allow-start-dragging",
];

#[test]
fn default_capability_grants_titlebar_window_controls() {
    let raw = include_str!("../capabilities/default.json");
    let caps: Value = serde_json::from_str(raw).expect("default.json must be valid JSON");
    let granted: Vec<&str> = caps["permissions"]
        .as_array()
        .expect("default.json must declare a permissions array")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    for required in REQUIRED_WINDOW_PERMISSIONS {
        assert!(
            granted.contains(&required),
            "default.json must grant {required} — used by the custom TitleBar (decorations:false)"
        );
    }
}
