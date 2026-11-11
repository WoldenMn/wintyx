//! Ports en ecoute : un port ouvert n'est pas une menace en soi, seuls comptent
//! l'executable qui le tient et quelques numeros historiquement lies a des portes derobees.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::process_based::command_stdout;
use crate::commands::threat_scanner::{ScanOutcome, ThreatItem};
use crate::utils::{constants, process};

/// Ports par defaut de chevaux de Troie anciens et documentes : Back Orifice (31337),
/// NetBus (12345, 12346, 20034), SubSeven (1243, 6711, 6712, 6713, 6776, 27374).
/// Les numeros partages avec des outils de developpement courants (4444, 5554) sont exclus.
const BACKDOOR_PORTS: &[u16] = &[1243, 6711, 6712, 6713, 6776, 12345, 12346, 20034, 27374, 31337];

const MAX_PORTS_LISTED: usize = 8;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Listener {
    pub pid: u32,
    pub port: u16,
    pub proto: &'static str,
}

pub(super) struct Owner {
    pub path: Option<String>,
    pub signature: String,
}

fn port_of(address: &str) -> Option<u16> {
    address.rsplit(':').next()?.parse().ok()
}

/// `netstat -ano` : TCP en ecoute (adresse distante au port 0, independant de la langue)
/// et sockets UDP liees, avec le PID du proprietaire.
pub(super) fn parse_listeners(netstat: &str) -> Vec<Listener> {
    let mut found = Vec::new();
    for line in netstat.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let (proto, local, pid) = match (parts.first().copied(), parts.len()) {
            (Some("TCP"), 5) if port_of(parts[2]) == Some(0) => ("TCP", parts[1], parts[4]),
            (Some("UDP"), 4) => ("UDP", parts[1], parts[3]),
            _ => continue,
        };
        if let (Some(port), Ok(pid)) = (port_of(local), pid.parse::<u32>()) {
            found.push(Listener { pid, port, proto });
        }
    }
    found
}

/// Sortie du script de resolution : `pid|chemin|statut de signature`.
pub(super) fn parse_owners(output: &str) -> HashMap<u32, Owner> {
    let mut owners = HashMap::new();
    for line in output.lines() {
        let Some(parts) = process::parse_pipe_fields(line, 3) else { continue };
        let Ok(pid) = parts[0].parse::<u32>() else { continue };
        let path = (!parts[1].is_empty()).then(|| parts[1].to_string());
        owners.insert(pid, Owner { path, signature: parts[2].to_string() });
    }
    owners
}

pub(super) fn is_trusted_dir(path: &str, roots: &[String]) -> bool {
    let lower = path.to_lowercase();
    roots.iter().any(|root| {
        let prefix = format!("{}\\", root.trim_end_matches('\\').to_lowercase());
        lower.starts_with(&prefix)
    })
}

fn file_name(path: &str) -> &str {
    path.rsplit('\\').next().unwrap_or(path)
}

fn ports_text(ports: &BTreeSet<u16>) -> String {
    let listed: Vec<String> = ports.iter().take(MAX_PORTS_LISTED).map(|p| p.to_string()).collect();
    let extra = ports.len().saturating_sub(MAX_PORTS_LISTED);
    if extra > 0 {
        format!("{} et {} autres", listed.join(", "), extra)
    } else {
        listed.join(", ")
    }
}

fn threat(name: String, severity: &str, description: String, path: String, fix_id: String) -> ThreatItem {
    ThreatItem {
        name,
        category: "Ports ouverts".into(),
        severity: severity.into(),
        description,
        path,
        can_fix: false,
        fix_id,
    }
}

/// Un port est signale pour l'un de ces deux indices seulement : l'executable qui le tient
/// est identifie et non signe (gravite selon son dossier), ou le numero est un port de
/// porte derobee connu. Proprietaire introuvable (processus protege hors administrateur)
/// ou signature illisible : inconnu, jamais coupable.
pub(super) fn classify_listeners(
    listeners: &[Listener],
    owners: &HashMap<u32, Owner>,
    roots: &[String],
) -> Vec<ThreatItem> {
    let mut by_pid: BTreeMap<u32, BTreeSet<u16>> = BTreeMap::new();
    for l in listeners {
        by_pid.entry(l.pid).or_default().insert(l.port);
    }

    let mut threats = Vec::new();
    for (pid, ports) in by_pid {
        let owner = owners.get(&pid);
        let path = owner.and_then(|o| o.path.as_deref());
        let signature = owner.map_or("", |o| o.signature.as_str());
        let backdoor: Vec<u16> = ports.iter().copied().filter(|p| BACKDOOR_PORTS.contains(p)).collect();

        let unsigned = path.is_some() && !signature.is_empty() && signature != "Valid";
        if let (true, Some(path)) = (unsigned, path) {
            let severity = if !backdoor.is_empty() {
                "high"
            } else if is_trusted_dir(path, roots) {
                "low"
            } else {
                "medium"
            };
            let mut description =
                format!("Exécutable non signé ({}) en écoute sur : {}", signature, ports_text(&ports));
            if !backdoor.is_empty() {
                description.push_str(" ; port associé à des portes dérobées connues");
            }
            threats.push(threat(file_name(path).to_string(), severity, description, path.to_string(), format!("port_pid_{}", pid)));
            continue;
        }

        let severity = if signature == "Valid" { "medium" } else { "high" };
        for port in backdoor {
            let holder = path.map_or("un processus non identifié".to_string(), |p| file_name(p).to_string());
            threats.push(threat(
                format!("Port {}", port),
                severity,
                format!("Port {} en écoute, associé à des portes dérobées connues (ouvert par {})", port, holder),
                path.unwrap_or_default().to_string(),
                format!("port_{}", port),
            ));
        }
    }
    threats
}

fn trusted_roots() -> Vec<String> {
    ["SystemRoot", "ProgramFiles", "ProgramFiles(x86)"]
        .iter()
        .filter_map(|var| std::env::var(var).ok())
        .filter(|v| !v.is_empty())
        .collect()
}

fn owners_script(pids: &BTreeSet<u32>) -> String {
    let ids: Vec<String> = pids.iter().map(|p| p.to_string()).collect();
    format!(
        "-NoProfile -Command \"$cache = @{{}}; foreach ($i in @({})) {{ $p = Get-Process -Id $i -ErrorAction SilentlyContinue; $path = if ($p) {{ $p.Path }} else {{ '' }}; $sig = ''; if ($path) {{ if (-not $cache.ContainsKey($path)) {{ $cache[$path] = [string](Get-AuthenticodeSignature -LiteralPath $path -ErrorAction SilentlyContinue).Status }}; $sig = $cache[$path] }}; \\\"$i|$path|$sig\\\" }}\"",
        ids.join(",")
    )
}

pub(crate) fn scan_open_ports() -> ScanOutcome {
    let netstat = process::run("netstat", "-ano", constants::TIMEOUT_MEDIUM);
    let stdout = command_stdout("Ports ouverts", &netstat)?;
    let listeners = parse_listeners(stdout);
    let scanned = listeners.len() as u32;
    if listeners.is_empty() {
        return Ok((0, Vec::new()));
    }

    let pids: BTreeSet<u32> = listeners.iter().map(|l| l.pid).collect();
    let resolved = process::run("powershell", &owners_script(&pids), constants::TIMEOUT_VERY_LONG);
    let owners = parse_owners(command_stdout("Propriétaires des ports", &resolved)?);

    Ok((scanned, classify_listeners(&listeners, &owners, &trusted_roots())))
}

#[cfg(test)]
mod tests;
