pub fn is_same_or_subkey(path: &str, root: &str) -> bool {
    if root.is_empty() {
        return false;
    }
    let (path, root) = (path.as_bytes(), root.as_bytes());
    path.len() >= root.len()
        && path[..root.len()].eq_ignore_ascii_case(root)
        && (path.len() == root.len() || path[root.len()] == b'\\')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_root_itself_and_its_subkeys_are_within() {
        assert!(is_same_or_subkey(r"HKLM\SOFTWARE", r"HKLM\SOFTWARE"));
        assert!(is_same_or_subkey(r"HKLM\SOFTWARE\Foo", r"HKLM\SOFTWARE"));
        assert!(is_same_or_subkey(r"HKLM\SOFTWARE\Foo\Bar", r"HKLM\SOFTWARE"));
    }

    #[test]
    fn a_sibling_sharing_the_prefix_is_not_within() {
        assert!(!is_same_or_subkey(r"HKLM\SOFTWAREX", r"HKLM\SOFTWARE"));
        assert!(!is_same_or_subkey(r"HKLM\SOFTWARE2\Foo", r"HKLM\SOFTWARE"));
        assert!(!is_same_or_subkey(r"HKCU\Control Panelx", r"HKCU\Control Panel"));
    }

    #[test]
    fn comparison_ignores_case_like_the_registry_does() {
        assert!(is_same_or_subkey(r"hklm\software\foo", r"HKLM\SOFTWARE"));
        assert!(!is_same_or_subkey(r"hklm\softwarex", r"HKLM\SOFTWARE"));
    }

    #[test]
    fn a_shorter_path_a_parent_or_an_empty_root_is_not_within() {
        assert!(!is_same_or_subkey(r"HKLM", r"HKLM\SOFTWARE"));
        assert!(!is_same_or_subkey(r"HKLM\SOFT", r"HKLM\SOFTWARE"));
        assert!(!is_same_or_subkey(r"HKLM\SOFTWARE", ""));
        assert!(!is_same_or_subkey("", r"HKLM\SOFTWARE"));
    }

    #[test]
    fn the_allowlist_respects_the_backslash_boundary() {
        use crate::utils::registry::is_allowed_key_path;
        assert!(is_allowed_key_path(r"HKLM\SOFTWARE"));
        assert!(is_allowed_key_path(r"hklm\software\Foo"));
        assert!(!is_allowed_key_path(r"HKLM\SOFTWAREX"));
        assert!(!is_allowed_key_path(r"HKLM\SOFTWAREX\Foo"));
        assert!(!is_allowed_key_path(r"HKLM\SYSTEMS\CurrentControlSet"));
        assert!(!is_allowed_key_path(r"HKCU\SOFTWARE2"));
        assert!(!is_allowed_key_path(r"HKCU\Control Panels\Mouse"));
        assert!(!is_allowed_key_path(r"hklm\softwarex"));
    }

    #[test]
    fn multibyte_text_never_panics() {
        assert!(!is_same_or_subkey("HKLM\\SOFTWARÉ\\é", r"HKLM\SOFTWARE"));
        assert!(!is_same_or_subkey("é", r"HKLM\SOFTWARE"));
    }
}
