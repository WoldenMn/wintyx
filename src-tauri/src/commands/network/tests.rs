use super::*;

#[test]
fn prefix_24_is_class_c_mask() {
    assert_eq!(prefix_to_subnet(24), "255.255.255.0");
}

#[test]
fn prefix_16_is_class_b_mask() {
    assert_eq!(prefix_to_subnet(16), "255.255.0.0");
}

#[test]
fn prefix_8_is_class_a_mask() {
    assert_eq!(prefix_to_subnet(8), "255.0.0.0");
}

#[test]
fn prefix_0_is_zero_mask() {
    assert_eq!(prefix_to_subnet(0), "0.0.0.0");
}

#[test]
fn prefix_32_is_full_mask() {
    assert_eq!(prefix_to_subnet(32), "255.255.255.255");
}

#[test]
fn prefix_above_32_returns_empty() {
    assert_eq!(prefix_to_subnet(33), "");
    assert_eq!(prefix_to_subnet(100), "");
}

#[test]
fn extract_ping_time_english_format() {
    let output = "Reply from 8.8.8.8: bytes=32 time=15ms TTL=57";
    assert_eq!(extract_ping_time(output), 15);
}

#[test]
fn extract_ping_time_french_format() {
    let output = "Reponse de 8.8.8.8 : octets=32 temps=22ms TTL=57";
    assert_eq!(extract_ping_time(output), 22);
}

#[test]
fn extract_ping_time_missing_returns_minus_one() {
    let output = "Destination host unreachable";
    assert_eq!(extract_ping_time(output), -1);
}

#[test]
fn extract_ping_time_large_values() {
    let output = "Reply from 1.1.1.1: bytes=32 time=999ms";
    assert_eq!(extract_ping_time(output), 999);
}

const TWO_ADAPTERS_TEMPLATE: &str = r#"[
  {"Name":"Ethernet","InterfaceDescription":"Intel(R) Ethernet I225-V","MacAddress":"MAC_ETHERNET","Status":"Up","LinkSpeed":"1 Gbps","MediaType":"802.3",
   "IpConfig":{"IPAddress":"192.0.2.10","PrefixLength":24,"NextHop":"192.0.2.1","DnsServers":["192.0.2.1","1.1.1.1"]}},
  {"Name":"Wi-Fi","InterfaceDescription":"Intel(R) Wireless-AC 9560","MacAddress":"MAC_WIFI","Status":"Up","LinkSpeed":"866 Mbps","MediaType":"Native 802.11",
   "IpConfig":{"IPAddress":"198.51.100.5","PrefixLength":16,"NextHop":null,"DnsServers":["198.51.100.1"]}}
]"#;

fn doc_mac(octet: u8) -> String {
    format!("00-00-5E-00-53-{octet:02X}")
}

fn two_adapters_json() -> String {
    TWO_ADAPTERS_TEMPLATE
        .replace("MAC_ETHERNET", &doc_mac(1))
        .replace("MAC_WIFI", &doc_mac(2))
}

#[test]
fn parse_adapters_json_two_adapters() {
    let (adapters, warnings) = parse_adapters_json(&two_adapters_json()).unwrap();
    assert_eq!(adapters.len(), 2);
    assert!(warnings.is_empty());

    let eth = &adapters[0];
    assert_eq!(eth.name, "Ethernet");
    assert_eq!(eth.adapter_type, "Ethernet");
    assert!(eth.is_connected);
    assert_eq!(eth.ip_address, "192.0.2.10");
    assert_eq!(eth.subnet_mask, "255.255.255.0");
    assert_eq!(eth.gateway, "192.0.2.1");
    assert_eq!(eth.dns_servers, "192.0.2.1, 1.1.1.1");
    assert_eq!(eth.speed, "1 Gbps");
    assert_eq!(eth.mac_address, doc_mac(1));
}

#[test]
fn parse_adapters_json_adapter_without_gateway() {
    let (adapters, _) = parse_adapters_json(&two_adapters_json()).unwrap();
    let wifi = &adapters[1];
    assert_eq!(wifi.adapter_type, "WiFi");
    assert_eq!(wifi.ip_address, "198.51.100.5");
    assert_eq!(wifi.subnet_mask, "255.255.0.0");
    assert_eq!(wifi.gateway, "");
    assert_eq!(wifi.dns_servers, "198.51.100.1");
    assert_eq!(wifi.mac_address, doc_mac(2));
}

#[test]
fn parse_adapters_json_disconnected_has_empty_ip_and_no_warning() {
    let json = r#"{"Name":"Bluetooth","InterfaceDescription":"Bluetooth PAN","MacAddress":"MAC_BLUETOOTH","Status":"Disconnected","LinkSpeed":"3 Mbps","MediaType":"802.3","IpConfig":null}"#.replace("MAC_BLUETOOTH", &doc_mac(3));
    let (adapters, warnings) = parse_adapters_json(&json).unwrap();
    assert_eq!(adapters.len(), 1);
    assert!(warnings.is_empty());
    assert_eq!(adapters[0].mac_address, doc_mac(3));
    assert!(!adapters[0].is_connected);
    assert_eq!(adapters[0].ip_address, "");
    assert_eq!(adapters[0].subnet_mask, "");
    assert_eq!(adapters[0].gateway, "");
    assert_eq!(adapters[0].dns_servers, "");
}

#[test]
fn parse_adapters_json_connected_without_ipconfig_warns() {
    let json = r#"{"Name":"VPN","InterfaceDescription":"WireGuard Tunnel","MacAddress":"","Status":"Up","LinkSpeed":"100 Mbps","MediaType":"","IpConfig":null}"#;
    let (adapters, warnings) = parse_adapters_json(json).unwrap();
    assert_eq!(adapters.len(), 1);
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("VPN"));
    assert_eq!(adapters[0].ip_address, "");
}

#[test]
fn parse_adapters_json_invalid_json_is_error() {
    assert!(parse_adapters_json("pas du json").is_err());
}

#[test]
fn ping_host_starting_with_dash_is_rejected_as_option_injection() {
    for host in ["-4", "-n", "-t", "-w", "-1.1.1.1"] {
        let result = ping_host_sync(host.to_string());
        assert!(
            matches!(result, Err(CommandError::InvalidInput(_))),
            "hote {:?} accepte alors qu'il se lit comme une option de ping",
            host
        );
    }
}

#[test]
fn brackets_cannot_hide_an_option_from_the_dash_guard() {
    for host in ["[-t]", "[-n]", "[-4]", "[[-t]]", "[-1.1.1.1]", "[", "]", "[]", "[::1", "::1]", "[a]b]", "[[::1]]"] {
        let result = ping_host_sync(host.to_string());
        assert!(
            matches!(result, Err(CommandError::InvalidInput(_))),
            "hote {:?} accepte",
            host
        );
    }
}

#[test]
fn the_target_handed_to_ping_is_the_validated_inner_text() {
    assert_eq!(ping_target("[::1]").as_deref(), Some("::1"));
    assert_eq!(ping_target("[fe80::1]").as_deref(), Some("fe80::1"));
    assert_eq!(ping_target("8.8.8.8").as_deref(), Some("8.8.8.8"));
    assert_eq!(ping_target("my-host.local").as_deref(), Some("my-host.local"));
    assert_eq!(ping_target("[-t]"), None);
}

#[test]
fn ping_host_validation_accepts_hostnames_with_inner_dashes_and_ipv6() {
    for host in ["my-host.local", "8.8.8.8", "google.com", "[::1]", "fe80::1"] {
        assert!(ping_target(host).is_some(), "{:?} doit etre accepte", host);
    }
    for host in ["", "-", "-4", "a b", "a;b", "a&b"] {
        assert!(ping_target(host).is_none(), "{:?} doit etre refuse", host);
    }
}

#[test]
fn public_ip_accepts_v4_and_v6_with_whitespace() {
    assert_eq!(parse_public_ip("203.0.113.5\r\n").as_deref(), Some("203.0.113.5"));
    assert_eq!(parse_public_ip(" 2001:db8::1 ").as_deref(), Some("2001:db8::1"));
}

#[test]
fn public_ip_rejects_empty_and_non_ip_bodies() {
    assert_eq!(parse_public_ip(""), None);
    assert_eq!(parse_public_ip("   \n"), None);
    assert_eq!(parse_public_ip("<html>rate limited</html>"), None);
}

#[test]
fn firewall_state_reads_one_as_enabled_and_zero_as_disabled() {
    assert_eq!(firewall_state("Public", Ok(Some(1))), Some(true));
    assert_eq!(firewall_state("Public", Ok(Some(0))), Some(false));
}

#[test]
fn firewall_state_is_unknown_when_value_missing_or_unreadable() {
    assert_eq!(firewall_state("Public", Ok(None)), None);
    assert_eq!(firewall_state("Public", Err("acces refuse".into())), None);
}

#[cfg(windows)]
#[test]
fn firewall_status_on_this_machine_matches_get_netfirewallprofile() {
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", "(Get-NetFirewallProfile | ForEach-Object { \"$($_.Name)=$($_.Enabled)\" }) -join ';'"])
        .output()
        .expect("powershell doit etre disponible");
    let reference = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let profiles = get_firewall_status();
    assert_eq!(profiles.len(), 3);
    for p in profiles {
        let windows_name = match p.name.as_str() {
            "Domaine" => "Domain",
            "Privé" => "Private",
            other => other,
        };
        let expected = reference.contains(&format!("{}=True", windows_name));
        assert_eq!(p.is_enabled, Some(expected), "{} : reference {}", p.name, reference);
    }
}
