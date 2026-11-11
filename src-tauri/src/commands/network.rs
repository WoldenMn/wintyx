use serde::Serialize;
use crate::utils::{constants, process, registry};
use crate::utils::errors::CommandError;

mod parse;
use parse::{extract_ping_time, parse_adapters_json};
#[cfg(test)]
use parse::prefix_to_subnet;

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAdapterInfo {
    pub name: String,
    pub description: String,
    pub mac_address: String,
    pub ip_address: String,
    pub subnet_mask: String,
    pub gateway: String,
    pub dns_servers: String,
    pub speed: String,
    pub status: String,
    pub adapter_type: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FirewallProfile {
    pub name: String,
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PingResult {
    pub host: String,
    pub success: bool,
    pub roundtrip_ms: i64,
    pub status: String,
}

// PERF-03 v2.16.0 : un SEUL process PowerShell renvoie les adaptateurs ET leur
// configuration IP (avant : 1 + N processes sequentiels, ~200-300ms de demarrage
// chacun). Pas d'interpolation d'input utilisateur, donc pas d'escape_powershell.
// Les scalaires IP sont projetes en PSCustomObject car ConvertTo-Json (PS 5.1)
// serialise les CimInstance en metadonnees CimClass sans les valeurs adaptees
// ("IPAddress = \"x\"" dans une chaine, jamais "IPAddress": "x") : l'ancien
// parse par adaptateur ne trouvait jamais IP/passerelle/DNS sur PS 5.1.
const ADAPTERS_PS_ARGS: &str = "-NoProfile -Command \"Get-NetAdapter | ForEach-Object { $cfg = $null; if ($_.Status -eq 'Up') { $c = Get-NetIPConfiguration -InterfaceIndex $_.ifIndex -ErrorAction SilentlyContinue; if ($c) { $v4 = $c.IPv4Address | Select-Object -First 1; $gw = $c.IPv4DefaultGateway | Select-Object -First 1; $cfg = [PSCustomObject]@{ IPAddress = $v4.IPAddress; PrefixLength = $v4.PrefixLength; NextHop = $gw.NextHop; DnsServers = @($c.DNSServer.ServerAddresses) } } }; [PSCustomObject]@{ Name = $_.Name; InterfaceDescription = $_.InterfaceDescription; MacAddress = $_.MacAddress; Status = $_.Status; LinkSpeed = $_.LinkSpeed; MediaType = $_.MediaType; IpConfig = $cfg } } | ConvertTo-Json -Depth 4\"";

#[tauri::command]
#[specta::specta]
pub async fn get_network_adapters() -> Result<Vec<NetworkAdapterInfo>, CommandError> {
    crate::utils::blocking::read_result(get_network_adapters_sync).await
}

pub fn get_network_adapters_sync() -> Result<Vec<NetworkAdapterInfo>, CommandError> {
    let result = process::run("powershell.exe", ADAPTERS_PS_ARGS, constants::TIMEOUT_MEDIUM);

    if !result.success {
        crate::utils::logger::warn(
            "network",
            &format!("get_network_adapters: echec Get-NetAdapter (exit {}): {}", result.exit_code, result.stderr),
        );
        return Err(CommandError::ProcessFailed {
            command: "Get-NetAdapter".into(),
            detail: if result.stderr.trim().is_empty() {
                format!("code de sortie {}", result.exit_code)
            } else {
                result.stderr.trim().to_string()
            },
        });
    }

    let stdout = result.stdout.trim();
    if stdout.is_empty() {
        return Ok(Vec::new());
    }

    let (adapters, warnings) = parse_adapters_json(stdout).map_err(|e| {
        crate::utils::logger::warn("network", &format!("get_network_adapters: JSON Get-NetAdapter invalide: {}", e));
        CommandError::Generic(format!("Reponse Get-NetAdapter illisible: {}", e))
    })?;
    for warning in &warnings {
        crate::utils::logger::warn("network", warning);
    }

    Ok(adapters)
}

#[tauri::command]
#[specta::specta]
pub fn get_firewall_status() -> Vec<FirewallProfile> {
    let profiles = vec![
        ("Domaine", r"HKLM\SYSTEM\CurrentControlSet\Services\SharedAccess\Parameters\FirewallPolicy\DomainProfile"),
        ("Privé", r"HKLM\SYSTEM\CurrentControlSet\Services\SharedAccess\Parameters\FirewallPolicy\StandardProfile"),
        ("Public", r"HKLM\SYSTEM\CurrentControlSet\Services\SharedAccess\Parameters\FirewallPolicy\PublicProfile"),
    ];

    profiles
        .into_iter()
        .map(|(name, path)| FirewallProfile {
            name: name.to_string(),
            is_enabled: firewall_state(name, registry::read_dword(path, "EnableFirewall")),
        })
        .collect()
}

/// Une valeur absente ou illisible donne `None` (« Inconnu ») : jamais « desactive » par defaut.
fn firewall_state(profile: &str, read: Result<Option<u32>, String>) -> Option<bool> {
    match read {
        Ok(value) => value.map(|v| v == 1),
        Err(e) => {
            crate::utils::logger::warn("network", &format!("pare-feu {} illisible : {}", profile, e));
            None
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn ping_host(host: String) -> Result<PingResult, CommandError> {
    crate::utils::blocking::read_result(move || ping_host_sync(host)).await
}

fn ping_target(host: &str) -> Option<String> {
    let inner = match host.strip_prefix('[') {
        Some(rest) => rest.strip_suffix(']')?,
        None => host,
    };
    let valid = !inner.is_empty()
        && inner.len() <= 253
        && !inner.starts_with('-')
        && inner.chars().all(|c| c.is_alphanumeric() || ".-:".contains(c));
    valid.then(|| inner.to_string())
}

pub fn ping_host_sync(host: String) -> Result<PingResult, CommandError> {
    let Some(target) = ping_target(&host) else {
        return Err(CommandError::InvalidInput(format!(
            "Adresse a pinguer invalide : {}",
            host
        )));
    };
    let use_ipv6 = target.contains(':');
    let args = if use_ipv6 {
        format!("-6 -n 1 -w 3000 {}", target)
    } else {
        format!("-n 1 -w 3000 {}", target)
    };

    let result = process::run("ping", &args, constants::TIMEOUT_SHORT);
    let has_reply = result.success
        && (result.stdout.contains("TTL=") || result.stdout.contains("time=") || result.stdout.contains("temps="));

    if has_reply {
        let ms = extract_ping_time(&result.stdout);
        Ok(PingResult {
            host,
            success: true,
            roundtrip_ms: ms,
            status: "Réussi".to_string(),
        })
    } else {
        Ok(PingResult {
            host,
            success: false,
            roundtrip_ms: -1,
            status: "Échec".to_string(),
        })
    }
}

#[tauri::command]
#[specta::specta]
pub async fn test_connectivity() -> Vec<PingResult> {
    crate::utils::blocking::read_value(test_connectivity_sync).await
}

pub fn test_connectivity_sync() -> Vec<PingResult> {
    let hosts = vec!["8.8.8.8", "1.1.1.1", "google.com"];
    hosts
        .into_iter()
        .filter_map(|h| ping_host_sync(h.to_string()).ok())
        .collect()
}

#[tauri::command]
#[specta::specta]
pub async fn get_public_ip() -> Result<String, CommandError> {
    crate::utils::blocking::read_result(get_public_ip_sync).await
}

fn parse_public_ip(stdout: &str) -> Option<String> {
    stdout
        .trim()
        .parse::<std::net::IpAddr>()
        .ok()
        .map(|ip| ip.to_string())
}

pub fn get_public_ip_sync() -> Result<String, CommandError> {
    let result = process::run(
        "powershell.exe",
        "-NoProfile -Command \"(Invoke-WebRequest -Uri 'https://api.ipify.org' -UseBasicParsing -TimeoutSec 5).Content\"",
        constants::TIMEOUT_MEDIUM,
    );
    if !result.success {
        crate::utils::logger::warn(
            "network",
            &format!("get_public_ip: requete echouee (exit {}): {}", result.exit_code, result.stderr),
        );
        return Err(CommandError::ProcessFailed {
            command: "Invoke-WebRequest api.ipify.org".into(),
            detail: if result.stderr.trim().is_empty() {
                format!("code de sortie {}", result.exit_code)
            } else {
                result.stderr.trim().to_string()
            },
        });
    }
    parse_public_ip(&result.stdout).ok_or_else(|| {
        crate::utils::logger::warn("network", "get_public_ip: reponse vide ou qui n'est pas une adresse IP");
        CommandError::Generic("Reponse inattendue du service d'adresse publique".into())
    })
}

#[cfg(test)]
mod tests;
