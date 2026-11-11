//! Execution de process externes Windows : lancement avec timeout et drainage
//! concurrent des pipes stdout/stderr.

use std::process::Command;
use std::time::Duration;

use super::decode::decode_output;
use super::system_path::resolve_system_program;
use super::text::split_args;
use super::CmdResult;

pub fn run(program: &str, args: &str, timeout_ms: u64) -> CmdResult {
    let mut parts: Vec<String> = if args.is_empty() {
        Vec::new()
    } else {
        split_args(args)
    };
    if is_powershell(program) {
        force_utf8_output(&mut parts);
    }

    let Some(executable) = resolve_system_program(program) else {
        return CmdResult {
            success: false,
            stdout: String::new(),
            stderr: format!("Programme non autorise : {program}"),
            exit_code: -1,
        };
    };
    let mut cmd = Command::new(executable);
    cmd.args(&parts);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(crate::utils::constants::CREATE_NO_WINDOW);
    }

    let result = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    match result {
        Ok(child) => {
            let output = if timeout_ms > 0 {
                wait_with_timeout(child, Duration::from_millis(timeout_ms))
            } else {
                child.wait_with_output().ok()
            };

            match output {
                Some(out) => CmdResult {
                    success: out.status.success(),
                    stdout: decode_output(&out.stdout),
                    stderr: decode_output(&out.stderr),
                    exit_code: out.status.code().unwrap_or(-1),
                },
                None => CmdResult {
                    success: false,
                    stdout: String::new(),
                    stderr: "Process timed out".to_string(),
                    exit_code: -1,
                },
            }
        }
        Err(e) => CmdResult {
            success: false,
            stdout: String::new(),
            stderr: e.to_string(),
            exit_code: -1,
        },
    }
}

fn is_powershell(program: &str) -> bool {
    let name = program.rsplit(['\\', '/']).next().unwrap_or(program);
    name.eq_ignore_ascii_case("powershell") || name.eq_ignore_ascii_case("powershell.exe")
}

// En page de codes OEM, PowerShell 5.1 remplace les caracteres hors page (guillemets
// typographiques) par un `"` droit non echappe dans son JSON ; la sortie est donc
// forcee en UTF-8 avant le script.
fn force_utf8_output(parts: &mut [String]) {
    const PREFIX: &str = "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; ";
    let command_flag = parts.iter().position(|p| p.eq_ignore_ascii_case("-Command"));
    if let Some(script) = command_flag.and_then(|i| parts.get_mut(i + 1)) {
        script.insert_str(0, PREFIX);
    }
}

pub fn run_silent(program: &str, args: &str, timeout_ms: u64) -> bool {
    run(program, args, timeout_ms).success
}

fn wait_with_timeout(
    mut child: std::process::Child,
    timeout: Duration,
) -> Option<std::process::Output> {
    use std::io::Read;

    // Drainer stdout/stderr dans des threads dedies AVANT d'attendre la fin du
    // process. Sans cela, un process qui ecrit plus que la capacite du pipe
    // (64 KB sur Windows) bloque en ecriture faute de lecteur, n'avance plus, et
    // est tue au timeout -> faux echec avec stdout vide (bug-rust-01).
    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();

    let stdout_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(pipe) = stdout_pipe.as_mut() {
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    });
    let stderr_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(pipe) = stderr_pipe.as_mut() {
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    });

    let start = std::time::Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };

    // Le process termine (ou tue) ferme ses pipes : les threads de lecture
    // atteignent EOF et se terminent, donc join() ne peut pas bloquer.
    let stdout = stdout_handle.join().unwrap_or_default();
    let stderr = stderr_handle.join().unwrap_or_default();

    status.map(|status| std::process::Output {
        status,
        stdout,
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn wait_with_timeout_drains_large_stdout() {
        // Regression bug-rust-01 : un process produisant plus que la capacite du
        // pipe Windows (64 KB) sur stdout ne doit PAS etre tue par timeout faute
        // de drainage concurrent des pipes.
        use std::process::Stdio;

        let child = Command::new("cmd")
            .args([
                "/c",
                "for /L %i in (1,1,3000) do @echo ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn cmd");

        let output = wait_with_timeout(child, Duration::from_secs(10))
            .expect("le process ne doit pas timeout : pipes draines en parallele");

        assert!(output.status.success());
        assert!(
            output.stdout.len() > 64 * 1024,
            "stdout complet attendu (>64KB), recu {} octets",
            output.stdout.len()
        );
    }

    #[cfg(windows)]
    #[test]
    fn powershell_json_keeps_typographic_quotes() {
        // Cause racine du faux echec Get-ScheduledTask : en page de codes OEM,
        // PowerShell 5.1 convertit U+201C/U+201D en `"` droit non echappe, ce
        // qui casse le JSON.
        let out = run(
            "powershell.exe",
            "-NoProfile -Command \"[PSCustomObject]@{ D = 'a \u{201C}b\u{201D} c' } | ConvertTo-Json -Compress\"",
            10_000,
        );
        assert!(out.success, "{}", out.stderr);
        let v: serde_json::Value = serde_json::from_str(out.stdout.trim()).expect("JSON valide");
        assert_eq!(v["D"], "a \u{201C}b\u{201D} c");
    }

    #[test]
    fn run_refuses_a_program_outside_the_system_list() {
        for program in ["calc.exe", r"C:\Windows\System32\sc.exe", r".\sc.exe"] {
            let out = run(program, "", 1_000);
            assert!(!out.success, "{program}");
            assert_eq!(out.exit_code, -1, "{program}");
            assert!(out.stderr.contains("non autorise"), "{}", out.stderr);
        }
    }

    #[cfg(windows)]
    #[test]
    fn run_starts_a_resolved_system_tool_by_bare_name() {
        let out = run("sc.exe", "query Spooler", 10_000);
        assert!(out.stdout.to_lowercase().contains("spooler"), "{:?} {:?}", out.stdout, out.stderr);
    }

    #[cfg(windows)]
    #[test]
    fn oem_console_output_keeps_accents() {
        let out = run("cmd.exe", "/c echo Invit\u{E9}", 10_000);
        assert!(out.stdout.contains("Invit\u{E9}"), "{:?}", out.stdout);
    }
}
