use std::path::{Path, PathBuf};

const SYSTEM32_TOOLS: &[&str] = &[
    "cmd", "ipconfig", "net", "netstat", "ping", "pnputil", "powercfg", "reg", "sc", "schtasks",
    "secedit",
];

pub fn system_root() -> PathBuf {
    let from_env = ["SystemRoot", "windir"]
        .iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .find(|path| path.is_absolute());
    from_env.unwrap_or_else(|| {
        let drive = std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string());
        PathBuf::from(format!("{drive}\\")).join("Windows")
    })
}

pub fn resolve_system_program(program: &str) -> Option<PathBuf> {
    resolve_in(&system_root(), program)
}

pub(crate) fn resolve_in(root: &Path, program: &str) -> Option<PathBuf> {
    let name = program.trim();
    if name.is_empty() || name.contains(['\\', '/', ':']) {
        return None;
    }
    let lowered = name.to_ascii_lowercase();
    let stem = lowered.strip_suffix(".exe").unwrap_or(&lowered);
    let system32 = root.join("System32");
    match stem {
        "powershell" => Some(
            system32
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe"),
        ),
        "explorer" => Some(root.join("explorer.exe")),
        tool if SYSTEM32_TOOLS.contains(&tool) => Some(system32.join(format!("{tool}.exe"))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_root() -> PathBuf {
        PathBuf::from(r"D:\Win")
    }

    #[test]
    fn powershell_resolves_to_its_versioned_folder() {
        for name in ["powershell", "powershell.exe", "PowerShell.EXE"] {
            assert_eq!(
                resolve_in(&fake_root(), name),
                Some(PathBuf::from(
                    r"D:\Win\System32\WindowsPowerShell\v1.0\powershell.exe"
                )),
                "{name}"
            );
        }
    }

    #[test]
    fn explorer_lives_at_the_root_of_systemroot() {
        assert_eq!(
            resolve_in(&fake_root(), "explorer.exe"),
            Some(PathBuf::from(r"D:\Win\explorer.exe"))
        );
    }

    #[test]
    fn console_tools_resolve_to_system32_with_or_without_extension() {
        for tool in SYSTEM32_TOOLS {
            let expected = PathBuf::from(format!(r"D:\Win\System32\{tool}.exe"));
            assert_eq!(resolve_in(&fake_root(), tool), Some(expected.clone()), "{tool}");
            assert_eq!(
                resolve_in(&fake_root(), &format!("{tool}.exe")),
                Some(expected),
                "{tool}.exe"
            );
        }
    }

    #[test]
    fn names_outside_the_list_are_refused() {
        for name in ["", "  ", "calc.exe", "evil", "sc.exe.bat", "powershell.com"] {
            assert_eq!(resolve_in(&fake_root(), name), None, "{name:?}");
        }
    }

    #[test]
    fn explicit_paths_are_refused() {
        for name in [
            r"C:\Users\user\AppData\Local\Wintyx\sc.exe",
            r".\sc.exe",
            r"..\sc.exe",
            "System32/sc.exe",
            "C:sc.exe",
        ] {
            assert_eq!(resolve_in(&fake_root(), name), None, "{name:?}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn every_allowed_program_exists_on_this_machine() {
        let mut names: Vec<&str> = SYSTEM32_TOOLS.to_vec();
        names.extend(["powershell.exe", "explorer.exe"]);
        for name in names {
            let path = resolve_system_program(name).expect(name);
            assert!(path.is_absolute(), "{}", path.display());
            assert!(path.is_file(), "{}", path.display());
        }
    }
}
