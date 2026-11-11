use super::*;

const NETSTAT: &str = "\
Connexions actives

  Proto  Adresse locale         Adresse distante       Etat           PID
  TCP    0.0.0.0:135            0.0.0.0:0              LISTENING       1100
  TCP    127.0.0.1:8080         0.0.0.0:0              LISTENING       2200
  TCP    [::]:445               [::]:0                 LISTENING       4
  TCP    192.0.2.10:50123       198.51.100.7:443       ESTABLISHED     3300
  UDP    0.0.0.0:5353           *:*                                    2200
  UDP    [::]:31337             *:*                                    4400
  TCP    0.0.0.0:31337          0.0.0.0:0              LISTENING       4400
";

fn roots() -> Vec<String> {
    vec![r"C:\Windows".into(), r"C:\Program Files".into(), r"C:\Program Files (x86)".into()]
}

fn owner(path: &str, signature: &str) -> Owner {
    Owner { path: Some(path.into()), signature: signature.into() }
}

fn unresolved() -> Owner {
    Owner { path: None, signature: String::new() }
}

fn owners(entries: Vec<(u32, Owner)>) -> HashMap<u32, Owner> {
    entries.into_iter().collect()
}

fn names(threats: &[ThreatItem]) -> Vec<String> {
    threats.iter().map(|t| t.name.clone()).collect()
}

#[test]
fn netstat_keeps_listening_tcp_and_bound_udp_with_their_pid() {
    let found = parse_listeners(NETSTAT);
    let pairs: Vec<(u32, u16, &str)> = found.iter().map(|l| (l.pid, l.port, l.proto)).collect();
    assert_eq!(
        pairs,
        vec![
            (1100, 135, "TCP"),
            (2200, 8080, "TCP"),
            (4, 445, "TCP"),
            (2200, 5353, "UDP"),
            (4400, 31337, "UDP"),
            (4400, 31337, "TCP"),
        ]
    );
}

#[test]
fn a_signed_listener_in_a_trusted_folder_is_not_a_threat() {
    let l = parse_listeners("  TCP    0.0.0.0:135   0.0.0.0:0   LISTENING   1100\n  TCP    127.0.0.1:8080   0.0.0.0:0   LISTENING   2200\n");
    let o = owners(vec![
        (1100, owner(r"C:\Windows\System32\svchost.exe", "Valid")),
        (2200, owner(r"C:\Program Files\Vendor\agent.exe", "Valid")),
    ]);
    let threats = classify_listeners(&l, &o, &roots());
    assert!(threats.is_empty(), "{:?}", names(&threats));
}

#[test]
fn an_unresolved_owner_is_unknown_not_guilty() {
    let l = parse_listeners("  TCP    0.0.0.0:8081   0.0.0.0:0   LISTENING   77\n");
    let threats = classify_listeners(&l, &owners(vec![(77, unresolved())]), &roots());
    assert!(threats.is_empty());
}

#[test]
fn an_unsigned_listener_outside_trusted_folders_is_medium_and_grouped_per_process() {
    let l = parse_listeners(
        "  TCP    0.0.0.0:8081   0.0.0.0:0   LISTENING   77\n  TCP    0.0.0.0:8082   0.0.0.0:0   LISTENING   77\n",
    );
    let o = owners(vec![(77, owner(r"C:\Users\user\AppData\Local\tool\tool.exe", "NotSigned"))]);
    let threats = classify_listeners(&l, &o, &roots());
    assert_eq!(threats.len(), 1);
    assert_eq!(threats[0].severity, "medium");
    assert_eq!(threats[0].name, "tool.exe");
    assert!(threats[0].description.contains("8081") && threats[0].description.contains("8082"));
    assert_eq!(threats[0].path, r"C:\Users\user\AppData\Local\tool\tool.exe");
}

#[test]
fn a_signed_listener_outside_trusted_folders_is_not_a_threat() {
    let l = parse_listeners("  TCP    127.0.0.1:9229   0.0.0.0:0   LISTENING   88\n");
    let o = owners(vec![(88, owner(r"C:\Users\user\AppData\Local\Programs\editor\editor.exe", "Valid"))]);
    assert!(classify_listeners(&l, &o, &roots()).is_empty());
}

#[test]
fn an_unsigned_listener_inside_program_files_is_low() {
    let l = parse_listeners("  TCP    0.0.0.0:7000   0.0.0.0:0   LISTENING   99\n");
    let o = owners(vec![(99, owner(r"C:\Program Files\Vendor\old.exe", "NotSigned"))]);
    let threats = classify_listeners(&l, &o, &roots());
    assert_eq!(threats.len(), 1);
    assert_eq!(threats[0].severity, "low");
}

#[test]
fn a_known_backdoor_port_is_flagged_even_for_a_signed_owner() {
    let l = parse_listeners("  TCP    0.0.0.0:31337   0.0.0.0:0   LISTENING   55\n");
    let o = owners(vec![(55, owner(r"C:\Program Files\Vendor\agent.exe", "Valid"))]);
    let threats = classify_listeners(&l, &o, &roots());
    assert_eq!(threats.len(), 1);
    assert_eq!(threats[0].name, "Port 31337");
    assert_eq!(threats[0].severity, "medium");
}

#[test]
fn a_backdoor_port_held_by_an_unresolved_owner_is_high() {
    let l = parse_listeners("  TCP    0.0.0.0:12345   0.0.0.0:0   LISTENING   56\n");
    let threats = classify_listeners(&l, &owners(vec![(56, unresolved())]), &roots());
    assert_eq!(threats.len(), 1);
    assert_eq!(threats[0].severity, "high");
}

#[test]
fn an_unsigned_process_on_a_backdoor_port_is_one_high_item() {
    let l = parse_listeners("  TCP    0.0.0.0:27374   0.0.0.0:0   LISTENING   57\n");
    let o = owners(vec![(57, owner(r"C:\Users\user\AppData\Local\Temp\x.exe", "NotSigned"))]);
    let threats = classify_listeners(&l, &o, &roots());
    assert_eq!(threats.len(), 1);
    assert_eq!(threats[0].severity, "high");
}

#[test]
fn trusted_folder_test_ignores_case_and_requires_a_path_boundary() {
    assert!(is_trusted_dir(r"c:\WINDOWS\system32\a.exe", &roots()));
    assert!(!is_trusted_dir(r"C:\Windows.old\a.exe", &roots()));
    assert!(!is_trusted_dir(r"C:\Program Files Extra\a.exe", &roots()));
}

#[test]
fn owner_lines_are_parsed_and_malformed_ones_ignored() {
    let parsed = parse_owners("12|C:\\Tools\\a.exe|Valid\nbad line\n13||\n");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[&12].path.as_deref(), Some(r"C:\Tools\a.exe"));
    assert_eq!(parsed[&12].signature, "Valid");
    assert_eq!(parsed[&13].path, None);
}
