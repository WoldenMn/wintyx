use super::NetworkAdapterInfo;

pub(super) fn parse_adapters_json(stdout: &str) -> Result<(Vec<NetworkAdapterInfo>, Vec<String>), String> {
    let json = serde_json::from_str::<serde_json::Value>(stdout).map_err(|e| e.to_string())?;
    let items = match &json {
        serde_json::Value::Array(arr) => arr.clone(),
        obj @ serde_json::Value::Object(_) => vec![obj.clone()],
        _ => vec![],
    };

    let mut adapters = Vec::new();
    let mut warnings = Vec::new();
    for item in &items {
        let name = item.get("Name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = item.get("InterfaceDescription").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mac = item.get("MacAddress").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let status = item.get("Status").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let speed = item.get("LinkSpeed").and_then(|v| v.as_str()).unwrap_or("N/A").to_string();
        let media_type = item.get("MediaType").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let is_connected = status == "Up";

        let adapter_type = if media_type.contains("802.3") {
            "Ethernet"
        } else if media_type.contains("802.11") || media_type.contains("Wireless") {
            "WiFi"
        } else {
            "Other"
        };

        let (ip_address, subnet_mask, gateway, dns_servers) = if is_connected {
            match item.get("IpConfig").filter(|v| !v.is_null()) {
                Some(cfg) => extract_ip_fields(cfg),
                None => {
                    warnings.push(format!(
                        "get_network_adapters: configuration IP indisponible pour '{}' (Get-NetIPConfiguration en échec)",
                        name
                    ));
                    (String::new(), String::new(), String::new(), String::new())
                }
            }
        } else {
            (String::new(), String::new(), String::new(), String::new())
        };

        adapters.push(NetworkAdapterInfo {
            name,
            description,
            mac_address: mac,
            ip_address,
            subnet_mask,
            gateway,
            dns_servers,
            speed,
            status,
            adapter_type: adapter_type.to_string(),
            is_connected,
        });
    }

    Ok((adapters, warnings))
}

fn extract_ip_fields(json: &serde_json::Value) -> (String, String, String, String) {
    let ip = json.get("IPAddress").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let subnet_mask = json.get("PrefixLength")
        .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .map(|prefix| prefix_to_subnet(prefix as u8))
        .unwrap_or_default();

    let gw = json.get("NextHop").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let dns = match json.get("DnsServers") {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<&str>>()
            .join(", "),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => String::new(),
    };

    (ip, subnet_mask, gw, dns)
}

pub(super) fn prefix_to_subnet(prefix: u8) -> String {
    if prefix > 32 {
        return String::new();
    }
    let mask: u32 = if prefix == 0 { 0 } else { !0u32 << (32 - prefix) };
    format!(
        "{}.{}.{}.{}",
        (mask >> 24) & 0xFF,
        (mask >> 16) & 0xFF,
        (mask >> 8) & 0xFF,
        mask & 0xFF
    )
}

pub(super) fn extract_ping_time(output: &str) -> i64 {
    for line in output.lines() {
        if let Some(pos) = line.find("time=") {
            let after = &line[pos + 5..];
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            return num_str.parse().unwrap_or(-1);
        }
        if let Some(pos) = line.find("temps=") {
            let after = &line[pos + 6..];
            let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            return num_str.parse().unwrap_or(-1);
        }
    }
    -1
}
