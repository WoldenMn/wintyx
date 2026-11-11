//! Application des changements selectionnes au registre Windows.
//!
//! Parse les valeurs cibles (DWORD ou String escape-aware), puis applique
//! au registre via `crate::utils::registry`. Reutilise les helpers
//! d'allowlist + require_admin du module parent (gates effectues
//! en amont par `restore_selected_changes`).

use crate::utils::errors::CommandError;
use crate::utils::registry;

use super::{ChangeRequest, ChangeRequestType};

#[derive(Debug)]
pub(super) enum TargetValue {
    Dword(u32),
    String(String),
}

pub(super) fn parse_target_value(raw: &str) -> Result<TargetValue, CommandError> {
    if let Some(hex) = raw.strip_prefix("dword:") {
        u32::from_str_radix(hex, 16)
            .map(TargetValue::Dword)
            .map_err(|e| CommandError::InvalidInput(format!("dword invalide: {}", e)))
    } else if let Some(inner) = raw.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        // Décodage : \\ → \ et \" → "
        let mut decoded = String::with_capacity(inner.len());
        let mut chars = inner.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('\\') => decoded.push('\\'),
                    Some('"') => decoded.push('"'),
                    Some(other) => {
                        decoded.push('\\');
                        decoded.push(other);
                    }
                    None => decoded.push('\\'),
                }
            } else {
                decoded.push(c);
            }
        }
        Ok(TargetValue::String(decoded))
    } else {
        Err(CommandError::InvalidInput(format!(
            "Format de valeur non supporté: {}",
            raw
        )))
    }
}

/// Applique un seul changement au registre. Idempotent pour `UndoAdded`
/// (delete d'une valeur inexistante = succes).
pub(super) fn apply_single_change(change: &ChangeRequest) -> Result<(), String> {
    match change.change_type {
        ChangeRequestType::RestoreModified | ChangeRequestType::RestoreRemoved => {
            let raw = change
                .target_value
                .as_ref()
                .ok_or_else(|| "target_value requis".to_string())?;
            let target = parse_target_value(raw).map_err(|e| e.to_string())?;
            match target {
                TargetValue::Dword(v) => {
                    registry::set_dword(&change.key_path, &change.value_name, v)
                }
                TargetValue::String(s) => {
                    registry::set_string(&change.key_path, &change.value_name, &s)
                }
            }
        }
        ChangeRequestType::UndoAdded => {
            registry::delete_value(&change.key_path, &change.value_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_target_value_dword_lowercase_hex() {
        let result = parse_target_value("dword:00000001").unwrap();
        assert!(matches!(result, TargetValue::Dword(1)));
    }

    #[test]
    fn parse_target_value_dword_uppercase_hex() {
        let result = parse_target_value("dword:000000FF").unwrap();
        assert!(matches!(result, TargetValue::Dword(255)));
    }

    #[test]
    fn parse_target_value_dword_invalid_hex() {
        assert!(parse_target_value("dword:zzzz").is_err());
    }

    #[test]
    fn parse_target_value_string_simple() {
        let result = parse_target_value("\"hello\"").unwrap();
        match result {
            TargetValue::String(s) => assert_eq!(s, "hello"),
            _ => panic!("expected String variant"),
        }
    }

    #[test]
    fn parse_target_value_string_with_escaped_quote() {
        let result = parse_target_value("\"say \\\"hi\\\"\"").unwrap();
        match result {
            TargetValue::String(s) => assert_eq!(s, "say \"hi\""),
            _ => panic!("expected String variant"),
        }
    }

    #[test]
    fn parse_target_value_string_with_escaped_backslash() {
        let result = parse_target_value("\"C:\\\\path\"").unwrap();
        match result {
            TargetValue::String(s) => assert_eq!(s, "C:\\path"),
            _ => panic!("expected String variant"),
        }
    }

    #[test]
    fn undo_added_removes_value_then_is_idempotent_on_real_key() {
        let path = format!(r"HKCU\Software\Wintyx-test-{}-undo", std::process::id());
        registry::set_string(&path, "Added", "x").unwrap();
        let undo = || {
            apply_single_change(&ChangeRequest {
                key_path: path.clone(),
                value_name: "Added".to_string(),
                change_type: ChangeRequestType::UndoAdded,
                target_value: None,
            })
        };
        assert!(undo().is_ok());
        assert_eq!(registry::get_string(&path, "Added"), None);
        assert!(undo().is_ok());
        let sub = path.strip_prefix(r"HKCU\").unwrap();
        let _ = windows_registry::CURRENT_USER.remove_tree(sub);
    }

    #[test]
    fn parse_target_value_unsupported_format_returns_err() {
        assert!(parse_target_value("hex:01,02").is_err());
        assert!(parse_target_value("garbage").is_err());
        assert!(parse_target_value("").is_err());
    }
}
