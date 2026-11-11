use super::tests::TempKey;
use super::*;

fn traces_mentioning(key: &str, name: &str) -> Vec<String> {
    let dir = crate::utils::paths::wintyx_local_dir().join("RegistryBackups");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
        .filter(|content| content.contains(&format!("Path: {key}\n")) && content.contains(&format!("Name: {name}\n")))
        .collect()
}

#[test]
fn delete_value_keeps_the_old_string_value_in_the_trace_like_every_other_write() {
    let key = TempKey::new("del-trace-string");
    set_string(&key.0, "Probe", "valeur d'avant").unwrap();
    delete_value(&key.0, "Probe").unwrap();
    assert_eq!(get_string(&key.0, "Probe"), None);
    let traces = traces_mentioning(&key.0, "Probe");
    assert_eq!(traces.len(), 1, "{traces:?}");
    assert!(traces[0].contains("OldValue: valeur d'avant"), "{}", traces[0]);
}

#[test]
fn delete_value_keeps_the_old_dword_value_in_the_trace() {
    let key = TempKey::new("del-trace-dword");
    set_dword(&key.0, "Flag", 42).unwrap();
    delete_value(&key.0, "Flag").unwrap();
    let traces = traces_mentioning(&key.0, "Flag");
    assert!(traces.iter().any(|t| t.contains("OldValue: 42")), "{traces:?}");
}

#[test]
fn deleting_an_absent_value_leaves_no_trace_and_still_succeeds() {
    let key = TempKey::new("del-trace-absent");
    delete_value(&key.0, "Ghost").unwrap();
    assert!(traces_mentioning(&key.0, "Ghost").is_empty());
}

#[test]
fn move_string_value_refuses_a_missing_source_key() {
    let src = TempKey::new("mv-nokey");
    let missing = format!(r"{}\Child", src.0);
    let dst = format!("{}-dst", src.0);
    let err = move_string_value(&missing, &dst, "App").unwrap_err();
    assert!(err.contains("Child"), "{err}");
    assert!(!key_exists(&dst, "App"));
}

#[test]
fn move_string_value_refuses_an_invalid_source_path() {
    let err = move_string_value("NOPE", r"HKCU\Software\Wintyx-test-never", "App").unwrap_err();
    assert!(err.contains("Invalid registry path"), "{err}");
}

#[test]
fn move_string_value_refuses_a_value_that_is_not_a_string_and_leaves_it_alone() {
    let src = TempKey::new("mv-dword");
    let dst = format!("{}-dst", src.0);
    set_dword(&src.0, "Flag", 1).unwrap();
    let err = move_string_value(&src.0, &dst, "Flag").unwrap_err();
    assert!(err.contains("non pris en charge"), "{err}");
    assert_eq!(get_dword(&src.0, "Flag"), Some(1));
    assert!(!key_exists(&dst, "Flag"));
}
