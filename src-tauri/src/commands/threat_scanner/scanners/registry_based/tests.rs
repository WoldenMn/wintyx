use super::*;

const ROOT: &str = r"C:\Windows";
const DEFAULT_USERINIT: &str = r"C:\Windows\system32\userinit.exe,";

fn shell_is_default(value: &str) -> bool {
    winlogon_value_is_default(value, "explorer.exe", ROOT)
}

fn userinit_is_default(value: &str) -> bool {
    winlogon_value_is_default(value, DEFAULT_USERINIT, ROOT)
}

#[test]
fn the_default_shell_is_recognised_whatever_its_case_or_padding() {
    for value in ["explorer.exe", "Explorer.EXE", "  explorer.exe  "] {
        assert!(shell_is_default(value), "{value:?}");
    }
}

#[test]
fn a_shell_that_merely_contains_explorer_is_not_the_default() {
    for value in [
        "explorer.exe,evil.exe",
        "explorer.exe evil.exe",
        r"C:\Users\Public\explorer.exe",
        "notexplorer.exe",
        "cmd.exe /c explorer.exe",
        "explorer.exe.bat",
        "",
    ] {
        assert!(!shell_is_default(value), "{value:?}");
    }
}

#[test]
fn the_default_userinit_is_recognised_with_or_without_the_trailing_comma() {
    for value in [
        r"C:\Windows\system32\userinit.exe,",
        r"C:\Windows\system32\userinit.exe",
        r"C:\WINDOWS\system32\userinit.exe,",
        r"c:\windows\SYSTEM32\USERINIT.EXE",
        r"%SystemRoot%\system32\userinit.exe,",
        r"%SYSTEMROOT%\System32\Userinit.exe",
        r"  C:\Windows\system32\userinit.exe,  ",
    ] {
        assert!(userinit_is_default(value), "{value:?}");
    }
}

#[test]
fn a_userinit_with_an_extra_program_or_another_location_is_not_the_default() {
    for value in [
        r"C:\Windows\system32\userinit.exe,C:\Users\Public\evil.exe",
        r"C:\Windows\system32\userinit.exe,evil.exe,",
        r"C:\Windows\system32\userinit.exe,,",
        r"C:\Users\Public\userinit.exe,",
        r"C:\Windows\userinit.exe,",
        "userinit.exe,",
        "userinit.exe",
        r"D:\Windows\system32\userinit.exe,",
        "",
    ] {
        assert!(!userinit_is_default(value), "{value:?}");
    }
}

#[test]
fn the_system_root_of_the_machine_is_what_the_default_is_built_from() {
    assert!(winlogon_value_is_default(
        r"D:\Win\system32\userinit.exe,",
        r"D:\Win\system32\userinit.exe,",
        r"D:\Win",
    ));
    assert!(winlogon_value_is_default(
        r"%SystemRoot%\system32\userinit.exe,",
        r"D:\Win\system32\userinit.exe,",
        r"D:\Win",
    ));
    assert!(!winlogon_value_is_default(
        r"C:\Windows\system32\userinit.exe,",
        r"D:\Win\system32\userinit.exe,",
        r"D:\Win",
    ));
}

#[cfg(windows)]
#[test]
fn the_scan_of_this_machine_reads_its_winlogon_values_without_failing() {
    let (scanned, threats) = scan_autorun_registry().expect("le scan doit aboutir");
    assert_eq!(scanned, 4);
    let winlogon: Vec<_> = threats.iter().filter(|t| t.name.contains("Winlogon")).collect();
    assert!(winlogon.is_empty(), "valeurs Winlogon par defaut jugees modifiees : {:?}", winlogon.iter().map(|t| &t.description).collect::<Vec<_>>());
}
