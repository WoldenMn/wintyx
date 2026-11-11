use super::*;
use std::process::Command;

#[test]
fn parse_pipe_fields_basic() {
    let result = parse_pipe_fields("a|b|c", 3);
    assert_eq!(result, Some(vec!["a", "b", "c"]));
}

#[test]
fn parse_pipe_fields_trims() {
    let result = parse_pipe_fields(" a | b | c ", 3);
    assert_eq!(result, Some(vec!["a", "b", "c"]));
}

#[test]
fn parse_pipe_fields_too_few() {
    assert_eq!(parse_pipe_fields("a|b", 3), None);
}

#[test]
fn parse_pipe_fields_extra_ok() {
    let result = parse_pipe_fields("a|b|c|d", 3);
    assert!(result.is_some());
    assert_eq!(result.unwrap().len(), 4);
}

#[test]
fn parse_pipe_fields_empty() {
    assert_eq!(parse_pipe_fields("", 2), None);
}

#[test]
fn escape_powershell_basic() {
    assert_eq!(escape_powershell("hello"), "hello");
}

#[test]
fn escape_powershell_single_quotes() {
    assert_eq!(escape_powershell("it's"), "it''s");
}

#[test]
fn escape_powershell_multiple_quotes() {
    assert_eq!(escape_powershell("it's a 'test'"), "it''s a ''test''");
}

#[test]
fn escape_powershell_leaves_backtick_and_dollar_literal() {
    assert_eq!(escape_powershell("cmd`inject"), "cmd`inject");
    assert_eq!(escape_powershell("$env:PATH"), "$env:PATH");
    assert_eq!(escape_powershell("$(whoami)"), "$(whoami)");
    assert_eq!(escape_powershell(r"C:\Users\user\a$b\app.exe"), r"C:\Users\user\a$b\app.exe");
}

#[test]
fn escape_powershell_doubles_typographic_quotes() {
    for q in ['\u{2018}', '\u{2019}', '\u{201A}', '\u{201B}'] {
        let input = format!("x{}y", q);
        assert_eq!(escape_powershell(&input), format!("x{}{}y", q, q));
    }
    assert_eq!(
        escape_powershell("Profil d\u{2019}\u{e9}t\u{e9}"),
        "Profil d\u{2019}\u{2019}\u{e9}t\u{e9}"
    );
}

#[test]
fn escape_powershell_strips_control_chars() {
    assert_eq!(escape_powershell("a\nb"), "ab");
    assert_eq!(escape_powershell("a\r\nb\t"), "ab");
    assert_eq!(escape_powershell("a\u{2028}b\u{2029}c"), "abc");
}

#[test]
fn escape_powershell_keeps_double_quotes_intact() {
    assert_eq!(escape_powershell("Mon \"test\""), "Mon \"test\"");
    assert!(!escape_powershell("a\"b").contains('\\'));
}

fn parse_with_real_powershell(script: &str) -> (usize, usize, Vec<String>) {
    let file = std::env::temp_dir().join(format!(
        "wintyx-ps-parse-{}-{}.ps1",
        std::process::id(),
        script.len()
    ));
    std::fs::write(&file, script).unwrap();
    let driver = format!(
        "$t=[IO.File]::ReadAllText('{}',[Text.Encoding]::UTF8);\
         $e=$null;$ast=[System.Management.Automation.Language.Parser]::ParseInput($t,[ref]$null,[ref]$e);\
         Write-Output ('E:'+@($e).Count);\
         Write-Output ('C:'+@($ast.FindAll({{param($n) $n -is [System.Management.Automation.Language.CommandAst]}},$true)).Count);\
         foreach($s in $ast.FindAll({{param($n) $n -is [System.Management.Automation.Language.StringConstantExpressionAst] -and $n.StringConstantType -eq 'SingleQuoted'}},$true)){{Write-Output ('S:'+(($s.Value.ToCharArray()|ForEach-Object{{[int]$_}}) -join ','))}}",
        file.display()
    );
    let out = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &driver])
        .output()
        .expect("powershell.exe doit etre disponible");
    let _ = std::fs::remove_file(&file);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let mut errors = usize::MAX;
    let mut commands = usize::MAX;
    let mut strings = Vec::new();
    for line in stdout.lines() {
        if let Some(v) = line.strip_prefix("E:") {
            errors = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("C:") {
            commands = v.trim().parse().unwrap();
        } else if let Some(v) = line.strip_prefix("S:") {
            let units: Vec<u16> = v
                .trim()
                .split(',')
                .filter(|u| !u.is_empty())
                .map(|u| u.parse().unwrap())
                .collect();
            strings.push(String::from_utf16(&units).unwrap());
        }
    }
    (errors, commands, strings)
}

fn assert_stays_one_literal(payload: &str) {
    let script = format!(
        "Checkpoint-Computer -Description '{}' -RestorePointType 'MODIFY_SETTINGS'",
        escape_powershell(payload)
    );
    let (errors, commands, strings) = parse_with_real_powershell(&script);
    assert_eq!(errors, 0, "erreurs de parsing pour {:?}", script);
    assert_eq!(commands, 1, "une seule commande attendue pour {:?}", script);
    let expected: String = payload
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '\u{2028}' | '\u{2029}'))
        .collect();
    assert_eq!(strings, vec![expected, "MODIFY_SETTINGS".to_string()]);
}

#[test]
fn powershell_parser_accepts_profile_with_typographic_apostrophe() {
    assert_stays_one_literal("Profil d\u{2019}\u{e9}t\u{e9}");
}

#[test]
fn powershell_parser_keeps_typographic_quote_payload_in_one_string() {
    assert_stays_one_literal("x\u{2019}; calc #");
}

#[test]
fn powershell_parser_keeps_ascii_and_dollar_payloads_in_one_string() {
    assert_stays_one_literal("it's `calc` $(calc) $env:PATH");
    assert_stays_one_literal("\u{2018}\u{201A}\u{201B}'; calc #");
}

#[test]
fn split_args_simple() {
    assert_eq!(split_args("a b c"), vec!["a", "b", "c"]);
}

#[test]
fn split_args_quoted() {
    let result = split_args("\"hello world\" foo");
    assert_eq!(result.len(), 2);
    assert_eq!(result[1], "foo");
}

#[test]
fn split_args_empty() {
    assert!(split_args("").is_empty());
}

#[test]
fn split_args_multiple_spaces() {
    assert_eq!(split_args("a  b"), vec!["a", "b"]);
}

#[test]
fn split_args_unterminated_quote_no_panic() {
    let r = split_args("\"unterminated arg");
    assert!(!r.is_empty());
}

#[test]
fn split_args_strips_surrounding_quotes() {
    let r = split_args("-NoProfile -Command \"(Invoke-WebRequest -Uri 'https://x').Content\"");
    assert_eq!(r.len(), 3);
    assert_eq!(r[0], "-NoProfile");
    assert_eq!(r[1], "-Command");
    assert_eq!(r[2], "(Invoke-WebRequest -Uri 'https://x').Content");
}

#[test]
fn split_args_preserves_single_quotes_inside() {
    let r = split_args("-Command \"Get-NetIPConfiguration -InterfaceAlias 'Wi-Fi 2'\"");
    assert_eq!(r.len(), 2);
    assert_eq!(r[1], "Get-NetIPConfiguration -InterfaceAlias 'Wi-Fi 2'");
}

#[test]
fn split_args_unescapes_inner_double_quotes() {
    let r = split_args("-Command \"ForEach-Object { \\\"$x\\\" }\"");
    assert_eq!(r.len(), 2);
    assert_eq!(r[1], "ForEach-Object { \"$x\" }");
}
