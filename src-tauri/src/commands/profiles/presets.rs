//! Profils predefinis (presets) hardcodes : Bureau, Discrétion max, Gaming, Performance.
//!
//! Chaque preset cumule des items sur les 6 domaines (data_protection, hardening,
//! performance, services, bloatware, startup). La fonction `presets()` est appelee
//! par la commande Tauri `get_profiles` qui les combine avec les profils customs
//! charges depuis le JSON local.

use super::Profile;

/// Decode l'id composite `${location}|${name}` utilise pour les entrees startup.
///
/// `location` est la branche registre/dossier (ex: `HKLM_Run`, `StartupFolder`),
/// `name` est le nom de l'entree. Le split est fait sur le **premier** `|` pour
/// supporter les noms contenant des pipes.
pub(super) fn parse_startup_id(id: &str) -> (String, String) {
    match id.split_once('|') {
        Some((loc, name)) => (name.to_string(), loc.to_string()),
        None => (id.to_string(), String::new()),
    }
}

/// Retourne les 4 profils predefinis Wintyx.
pub(super) fn presets() -> Vec<Profile> {
    vec![
        Profile {
            id: "bureau".to_string(),
            name: "Bureau".to_string(),
            description: "Équilibré standard : toutes les protections de données, durcissements essentiels, optimisations conservatrices.".to_string(),
            data_protection_ids: crate::commands::data_protection::batch_item_ids(),
            hardening_ids: vec![
                "RDP".to_string(),
                "SMBv1".to_string(),
                "AutoRun".to_string(),
                "UAC".to_string(),
                "WDigest".to_string(),
                "LLMNR".to_string(),
                "NetBIOS".to_string(),
            ],
            performance_ids: vec!["visual_effects".to_string(), "prefetch".to_string()],
            is_custom: false,
            services_ids: vec![
                "DiagTrack".to_string(),
                "dmwappushservice".to_string(),
                "RemoteRegistry".to_string(),
                "Fax".to_string(),
                "RetailDemo".to_string(),
            ],
            bloatware_ids: vec![
                "Microsoft.MicrosoftSolitaireCollection".to_string(),
                "Microsoft.WindowsMaps".to_string(),
                "Microsoft.MixedReality.Portal".to_string(),
                "Microsoft.SkypeApp".to_string(),
                "Microsoft.WindowsFeedbackHub".to_string(),
            ],
            startup_ids: vec![],
        },
        Profile {
            id: "discretion_max".to_string(),
            name: "Discrétion max".to_string(),
            description: "Confidentialité maximale : tous les domaines au maximum. Toutes les protections, tous les durcissements, toutes les optimisations.".to_string(),
            data_protection_ids: crate::commands::data_protection::batch_item_ids(),
            hardening_ids: vec![
                "RDP".to_string(), "SMBv1".to_string(), "AutoRun".to_string(),
                "UAC".to_string(), "WDigest".to_string(), "LLMNR".to_string(),
                "NetBIOS".to_string(),
                "ScriptHost".to_string(), "PSLogging".to_string(), "Spooler".to_string(),
                "IPv6".to_string(), "AdminShares".to_string(), "GuestAccount".to_string(),
                "AccountLockout".to_string(), "AdvancedAudit".to_string(),
            ],
            performance_ids: vec![
                "visual_effects".to_string(), "animations".to_string(), "transparency".to_string(),
                "game_mode".to_string(), "prefetch".to_string(), "power_throttling".to_string(),
                "nagle".to_string(), "network_throttling".to_string(),
                "high_perf_power".to_string(), "onedrive_startup".to_string(),
            ],
            is_custom: false,
            services_ids: vec![
                "DiagTrack", "dmwappushservice", "SysMain", "WSearch", "RemoteRegistry",
                "Fax", "lfsvc", "MapsBroker", "RetailDemo", "WMPNetworkSvc",
                "XblAuthManager", "XblGameSave", "XboxNetApiSvc", "WbioSrvc",
            ].iter().map(|s| s.to_string()).collect(),
            bloatware_ids: vec![
                "Microsoft.3DBuilder", "Microsoft.BingWeather", "Microsoft.BingNews",
                "Microsoft.GetHelp", "Microsoft.Getstarted", "Microsoft.MicrosoftOfficeHub",
                "Microsoft.MicrosoftSolitaireCollection", "Microsoft.MixedReality.Portal",
                "Microsoft.OneConnect", "Microsoft.People", "Microsoft.SkypeApp",
                "Microsoft.WindowsFeedbackHub", "Microsoft.WindowsMaps",
                "Microsoft.Xbox.TCUI", "Microsoft.XboxApp", "Microsoft.XboxGameOverlay",
                "Microsoft.XboxGamingOverlay", "Microsoft.YourPhone", "Microsoft.ZuneMusic",
                "Microsoft.ZuneVideo", "Microsoft.Teams", "MicrosoftTeams",
                "Microsoft.Windows.Cortana", "Microsoft.WindowsAlarms",
                "Microsoft.WindowsCamera", "microsoft.windowscommunicationsapps",
                "Microsoft.Todos", "Microsoft.PowerAutomateDesktop", "Microsoft.GamingApp",
            ].iter().map(|s| s.to_string()).collect(),
            startup_ids: vec![
                "HKCU_Run|OneDrive".to_string(),
            ],
        },
        Profile {
            id: "gaming".to_string(),
            name: "Gaming".to_string(),
            description: "Optimisé pour les jeux : télémétrie + publicité désactivées, aucun durcissement (pour éviter de casser des protocoles), toutes les optimisations performances.".to_string(),
            data_protection_ids: vec![
                "dp_0".to_string(), "dp_1".to_string(), "dp_2".to_string(),
                "dp_3".to_string(), "dp_4".to_string(), "dp_5".to_string(),
                "dp_8".to_string(),
            ],
            hardening_ids: vec![],
            performance_ids: vec![
                "visual_effects".to_string(), "animations".to_string(), "transparency".to_string(),
                "game_mode".to_string(), "prefetch".to_string(), "power_throttling".to_string(),
                "nagle".to_string(), "network_throttling".to_string(),
                "high_perf_power".to_string(), "onedrive_startup".to_string(),
            ],
            is_custom: false,
            services_ids: vec![],
            bloatware_ids: vec![
                "Microsoft.3DBuilder", "Microsoft.BingWeather", "Microsoft.BingNews",
                "Microsoft.GetHelp", "Microsoft.Getstarted", "Microsoft.MicrosoftOfficeHub",
                "Microsoft.MixedReality.Portal", "Microsoft.OneConnect", "Microsoft.People",
                "Microsoft.SkypeApp", "Microsoft.WindowsFeedbackHub", "Microsoft.WindowsMaps",
                "Microsoft.YourPhone", "Microsoft.ZuneMusic", "Microsoft.ZuneVideo",
                "Microsoft.Teams", "MicrosoftTeams", "Microsoft.Windows.Cortana",
                "Microsoft.WindowsAlarms", "microsoft.windowscommunicationsapps",
                "Microsoft.Todos", "Microsoft.PowerAutomateDesktop",
            ].iter().map(|s| s.to_string()).collect(),
            startup_ids: vec![
                "HKCU_Run|OneDrive".to_string(),
            ],
        },
        Profile {
            id: "performance".to_string(),
            name: "Performance".to_string(),
            description: "Maximise les performances système : télémétrie minimum (réduit l'activité en arrière-plan), aucun durcissement, toutes les optimisations performances. Désactive la recherche Windows (WSearch) et Superfetch (SysMain) : le menu Démarrer ne cherchera plus les fichiers.".to_string(),
            data_protection_ids: vec![
                "dp_0".to_string(), "dp_1".to_string(), "dp_2".to_string(), "dp_3".to_string(),
            ],
            hardening_ids: vec![],
            performance_ids: vec![
                "visual_effects".to_string(), "animations".to_string(), "transparency".to_string(),
                "game_mode".to_string(), "prefetch".to_string(), "power_throttling".to_string(),
                "nagle".to_string(), "network_throttling".to_string(),
                "high_perf_power".to_string(), "onedrive_startup".to_string(),
            ],
            is_custom: false,
            services_ids: vec![
                "SysMain".to_string(),
                "WSearch".to_string(),
                "WMPNetworkSvc".to_string(),
                "MapsBroker".to_string(),
                "lfsvc".to_string(),
            ],
            bloatware_ids: vec![
                "Microsoft.WindowsMaps".to_string(),
                "Microsoft.MixedReality.Portal".to_string(),
                "Microsoft.MicrosoftOfficeHub".to_string(),
                "Microsoft.SkypeApp".to_string(),
                "Microsoft.OneConnect".to_string(),
            ],
            startup_ids: vec![
                "HKCU_Run|OneDrive".to_string(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_returns_4_profiles_with_expected_ids() {
        let profiles = presets();
        assert_eq!(profiles.len(), 4);
        let ids: Vec<&str> = profiles.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"bureau"));
        assert!(ids.contains(&"discretion_max"));
        assert!(ids.contains(&"gaming"));
        assert!(ids.contains(&"performance"));
    }

    #[test]
    fn each_preset_has_at_least_one_item() {
        for profile in presets() {
            let total = profile.data_protection_ids.len()
                + profile.hardening_ids.len()
                + profile.performance_ids.len()
                + profile.services_ids.len()
                + profile.bloatware_ids.len()
                + profile.startup_ids.len();
            assert!(total > 0, "Profile {} is empty", profile.id);
        }
    }

    #[test]
    fn parse_startup_id_splits_correctly() {
        let (name, location) = parse_startup_id("HKCU|OneDrive");
        assert_eq!(name, "OneDrive");
        assert_eq!(location, "HKCU");
    }

    #[test]
    fn parse_startup_id_handles_pipe_in_name() {
        let (name, location) = parse_startup_id("HKCU|name|with|pipes");
        assert_eq!(name, "name|with|pipes");
        assert_eq!(location, "HKCU");
    }

    #[test]
    fn parse_startup_id_no_pipe_returns_empty_location() {
        let (name, location) = parse_startup_id("justname");
        assert_eq!(name, "justname");
        assert_eq!(location, "");
    }

    #[test]
    fn no_preset_lowers_a_security_setting_through_the_data_protection_domain() {
        let smartscreen = crate::commands::data_protection::get_data_protection_items()
            .into_iter()
            .find(|item| item.name.contains("SmartScreen"))
            .expect("SmartScreen reste un item protegeable seul");
        for profile in presets() {
            assert!(
                !profile.data_protection_ids.contains(&smartscreen.id),
                "le preset {} desactive SmartScreen",
                profile.id
            );
        }
    }

    #[test]
    fn bureau_and_discretion_max_still_cover_every_other_protection() {
        let expected = crate::commands::data_protection::batch_item_ids();
        for id in ["bureau", "discretion_max"] {
            let profile = presets().into_iter().find(|p| p.id == id).unwrap();
            assert_eq!(profile.data_protection_ids, expected, "{id}");
        }
    }

    #[test]
    fn bureau_preset_includes_telemetry_services() {
        let bureau = presets().into_iter().find(|p| p.id == "bureau").unwrap();
        assert!(bureau.services_ids.contains(&"DiagTrack".to_string()));
        assert!(bureau.services_ids.contains(&"dmwappushservice".to_string()));
    }

    #[test]
    fn discretion_max_preset_includes_all_29_bloatwares() {
        let discr = presets().into_iter().find(|p| p.id == "discretion_max").unwrap();
        assert_eq!(discr.bloatware_ids.len(), 29);
    }

    #[test]
    fn gaming_preset_excludes_xbox_bloatware() {
        let gaming = presets().into_iter().find(|p| p.id == "gaming").unwrap();
        assert!(!gaming.bloatware_ids.iter().any(|id| id.contains("Xbox")));
        assert!(!gaming.bloatware_ids.contains(&"Microsoft.GamingApp".to_string()));
        assert!(!gaming.bloatware_ids.contains(&"Microsoft.MicrosoftSolitaireCollection".to_string()));
    }

    #[test]
    fn performance_preset_targets_heavy_services() {
        let perf = presets().into_iter().find(|p| p.id == "performance").unwrap();
        assert!(perf.services_ids.contains(&"SysMain".to_string()));
        assert!(perf.services_ids.contains(&"WSearch".to_string()));
    }

    #[test]
    fn all_non_bureau_presets_have_onedrive_startup() {
        let one_drive_id = "HKCU_Run|OneDrive";
        for p in presets() {
            if p.id != "bureau" {
                assert!(
                    p.startup_ids.iter().any(|id| id == one_drive_id),
                    "Preset {} should include OneDrive startup_ids",
                    p.id
                );
            }
        }
    }

    #[test]
    fn presets_startup_ids_use_short_location_codes() {
        let valid_locations = ["HKCU_Run", "HKLM_Run", "HKCU_RunOnce", "HKLM_RunOnce", "StartupFolder"];
        for profile in presets() {
            for sup_id in &profile.startup_ids {
                let (_, location) = parse_startup_id(sup_id);
                assert!(
                    valid_locations.contains(&location.as_str()),
                    "Profile {} has invalid startup location code: {:?}",
                    profile.id,
                    location
                );
            }
        }
    }
}
