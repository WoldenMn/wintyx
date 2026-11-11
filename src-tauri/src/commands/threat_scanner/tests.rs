use super::*;

#[test]
fn scan_guard_rejects_a_second_scan_and_releases_on_drop() {
    static FLAG: AtomicBool = AtomicBool::new(false);
    let first = ScanGuard::acquire(&FLAG).expect("premier scan");
    assert!(matches!(
        ScanGuard::acquire(&FLAG),
        Err(CommandError::Generic(msg)) if msg.contains("déjà en cours")
    ));
    drop(first);
    assert!(ScanGuard::acquire(&FLAG).is_ok());
}

#[test]
fn scan_progress_serializes_expected_camelcase_keys() {
    // ThreatScanProgress est desormais genere dans bindings.ts via tauri-specta
    // (collect_events!, types-event). Ce test garantit que le rename camelCase
    // reste applique au RUNTIME : l'emission serde doit produire les memes cles
    // que le type genere consomme par le frontend.
    let p = ThreatScanProgress {
        phase_name: "x".into(),
        phase_index: 1,
        phase_count: 6,
        scanned_so_far: 0,
    };
    let json = serde_json::to_value(&p).unwrap();
    let keys: std::collections::BTreeSet<String> =
        json.as_object().unwrap().keys().cloned().collect();
    let expected: std::collections::BTreeSet<String> =
        ["phaseName", "phaseIndex", "phaseCount", "scannedSoFar"]
            .iter()
            .map(|s| s.to_string())
            .collect();
    assert_eq!(keys, expected, "ThreatScanProgress: cles modifiees -> sync src/lib/types.ts");
}

#[test]
fn risk_score_empty() {
    assert_eq!(calculate_risk_score(&[]), 0);
}

#[test]
fn collect_phase_risk_levels() {
    // run-07 : le niveau de categorie suit la severite MAX presente, pas le nombre
    // d'items (un grand nombre de "low" reste "low", anti fear-marketing).
    fn mk(sev: &str) -> ThreatItem {
        ThreatItem {
            name: "t".into(),
            category: "c".into(),
            severity: sev.into(),
            description: String::new(),
            path: String::new(),
            can_fix: false,
            fix_id: String::new(),
        }
    }
    fn none() -> ScanOutcome { Ok((5, vec![])) }
    fn many_low() -> ScanOutcome { Ok((5, vec![mk("low"), mk("low"), mk("low")])) }
    fn has_medium() -> ScanOutcome { Ok((5, vec![mk("low"), mk("medium")])) }
    fn has_high() -> ScanOutcome { Ok((5, vec![mk("low"), mk("high")])) }

    assert_eq!(collect_phase(&ScanPhase { name: "X", scan: none }).2.risk_level, "low");
    assert_eq!(collect_phase(&ScanPhase { name: "X", scan: many_low }).2.risk_level, "low");
    assert_eq!(collect_phase(&ScanPhase { name: "X", scan: has_medium }).2.risk_level, "medium");
    assert_eq!(collect_phase(&ScanPhase { name: "X", scan: has_high }).2.risk_level, "high");
}

#[test]
fn risk_score_low_items_capped() {
    // run-07 : 59 ports "low" ne saturent pas le score (max 5 comptes -> 10 points).
    let threats: Vec<ThreatItem> = (0..59)
        .map(|_| ThreatItem {
            name: "p".into(),
            category: "Ports ouverts".into(),
            severity: "low".into(),
            description: String::new(),
            path: String::new(),
            can_fix: false,
            fix_id: String::new(),
        })
        .collect();
    assert_eq!(calculate_risk_score(&threats), 10);
}

#[test]
fn risk_score_single_critical() {
    let threats = vec![ThreatItem {
        name: "test".into(),
        description: "test".into(),
        severity: "critical".into(),
        category: "test".into(),
        path: "test".into(),
        can_fix: false,
        fix_id: "test".into(),
    }];
    assert_eq!(calculate_risk_score(&threats), 25);
}

#[test]
fn risk_score_capped_at_100() {
    let threats: Vec<ThreatItem> = (0..10)
        .map(|i| ThreatItem {
            name: format!("t{}", i),
            description: String::new(),
            severity: "critical".into(),
            category: String::new(),
            path: String::new(),
            can_fix: false,
            fix_id: String::new(),
        })
        .collect();
    assert_eq!(calculate_risk_score(&threats), 100);
}

#[test]
fn risk_score_mixed_severities_capped() {
    let mut threats = Vec::new();
    for _ in 0..2 {
        threats.push(ThreatItem {
            name: "crit".into(),
            description: String::new(),
            severity: "critical".into(),
            category: String::new(),
            path: String::new(),
            can_fix: false,
            fix_id: String::new(),
        });
    }
    for _ in 0..2 {
        threats.push(ThreatItem {
            name: "high".into(),
            description: String::new(),
            severity: "high".into(),
            category: String::new(),
            path: String::new(),
            can_fix: false,
            fix_id: String::new(),
        });
    }
    for _ in 0..3 {
        threats.push(ThreatItem {
            name: "med".into(),
            description: String::new(),
            severity: "medium".into(),
            category: String::new(),
            path: String::new(),
            can_fix: false,
            fix_id: String::new(),
        });
    }
    assert_eq!(calculate_risk_score(&threats), 95);
}

#[test]
fn risk_score_does_not_overflow() {
    let count = u32::MAX / 10;
    let threats: Vec<ThreatItem> = (0..count.min(50) as usize)
        .map(|_| ThreatItem {
            name: "x".into(),
            description: String::new(),
            severity: "critical".into(),
            category: String::new(),
            path: String::new(),
            can_fix: false,
            fix_id: String::new(),
        })
        .collect();
    assert_eq!(calculate_risk_score(&threats), 100);
}

#[test]
fn collect_phase_marks_a_failed_scan_instead_of_counting_zero_threats() {
    fn broken() -> ScanOutcome { Err("Services non signés: commande en echec (exit -1): Process timed out".into()) }
    fn fine() -> ScanOutcome { Ok((5, vec![])) }

    let (scanned, threats, category) = collect_phase(&ScanPhase { name: "Services non signés", scan: broken });
    assert!(category.failed);
    assert_eq!(category.risk_level, "unknown");
    assert_eq!((scanned, threats.len(), category.threats_found), (0, 0, 0));

    let (_, _, ok_category) = collect_phase(&ScanPhase { name: "X", scan: fine });
    assert!(!ok_category.failed);
    assert_eq!(ok_category.risk_level, "low");
}

fn one_threat() -> ThreatItem {
    ThreatItem {
        name: "t".into(),
        category: "c".into(),
        severity: "high".into(),
        description: String::new(),
        path: String::new(),
        can_fix: false,
        fix_id: String::new(),
    }
}

#[test]
fn a_phase_whose_thread_panics_comes_back_failed_not_missing() {
    fn first() -> ScanOutcome { Ok((4, vec![one_threat()])) }
    fn explodes() -> ScanOutcome { panic!("scanner en panique (test)") }
    fn last() -> ScanOutcome { Ok((2, vec![])) }
    let phases = [
        ScanPhase { name: "Premiere", scan: first },
        ScanPhase { name: "Instable", scan: explodes },
        ScanPhase { name: "Derniere", scan: last },
    ];

    let (threats, categories, scanned) = run_phases(&phases, &|_| {});

    assert_eq!(categories.len(), 3, "une categorie ne doit pas disparaitre");
    let names: Vec<&str> = categories.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Premiere", "Instable", "Derniere"]);
    assert!(!categories[0].failed && !categories[2].failed);
    assert!(categories[1].failed);
    assert_eq!(categories[1].risk_level, "unknown");
    assert_eq!(categories[1].scanned, 0);
    assert_eq!(threats.len(), 1);
    assert_eq!(scanned, 6);
}

#[test]
fn every_phase_reports_progress_exactly_once() {
    fn one() -> ScanOutcome { Ok((1, vec![])) }
    let phases = [
        ScanPhase { name: "A", scan: one },
        ScanPhase { name: "B", scan: one },
        ScanPhase { name: "C", scan: one },
    ];
    let seen = std::sync::Mutex::new(Vec::new());
    let (_, categories, scanned) = run_phases(&phases, &|p| {
        seen.lock().unwrap().push((p.phase_name.clone(), p.phase_count, p.scanned_so_far));
    });
    let mut seen = seen.into_inner().unwrap();
    seen.sort();
    assert_eq!(categories.len(), 3);
    assert_eq!(scanned, 3);
    assert_eq!(seen.len(), 3);
    assert!(seen.iter().all(|(_, count, _)| *count == 3));
    assert_eq!(seen.iter().map(|(n, _, _)| n.as_str()).collect::<Vec<_>>(), ["A", "B", "C"]);
}
