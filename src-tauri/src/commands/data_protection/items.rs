//! Table declarative des items de confidentialite (donnees pures), separee de la
//! logique d'application et de scoring (voir `data_protection/mod.rs`).

pub(super) struct ItemDef {
    pub(super) category: &'static str,
    pub(super) name: &'static str,
    pub(super) description: &'static str,
    pub(super) reg_path: &'static str,
    pub(super) reg_value: &'static str,
    pub(super) disabled_value: u32,
}

const EXCLUDED_FROM_BATCH: &[(&str, &str)] = &[(
    r"HKLM\SOFTWARE\Policies\Microsoft\Windows\System",
    "EnableSmartScreen",
)];

impl ItemDef {
    pub(super) fn in_batch(&self) -> bool {
        !EXCLUDED_FROM_BATCH.iter().any(|(path, value)| {
            self.reg_path.eq_ignore_ascii_case(path) && self.reg_value.eq_ignore_ascii_case(value)
        })
    }
}

pub(super) fn build_items() -> Vec<ItemDef> {
    vec![
        ItemDef {
            category: "Telemetrie",
            name: "Collecte de données",
            description: "Désactiver la collecte de données de diagnostic Windows",
            reg_path: r"HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection",
            reg_value: "AllowTelemetry",
            disabled_value: 0,
        },
        ItemDef {
            category: "Telemetrie",
            name: "Télémétrie secondaire",
            description: "Désactiver la collecte supplémentaire de données",
            reg_path: r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection",
            reg_value: "AllowTelemetry",
            disabled_value: 0,
        },
        ItemDef {
            category: "Telemetrie",
            name: "Service DiagTrack",
            description: "Désactiver le service de suivi diagnostique",
            reg_path: r"HKLM\SYSTEM\CurrentControlSet\Services\DiagTrack",
            reg_value: "Start",
            disabled_value: 4,
        },
        ItemDef {
            category: "Telemetrie",
            name: "Programme CEIP",
            description: "Désactiver le programme d'amélioration de l'expérience client",
            reg_path: r"HKLM\SOFTWARE\Microsoft\SQMClient\Windows",
            reg_value: "CEIPEnable",
            disabled_value: 0,
        },
        ItemDef {
            category: "Telemetrie",
            name: "Fréquence de feedback",
            description: "Désactiver les demandes de feedback Windows",
            reg_path: r"HKCU\SOFTWARE\Microsoft\Siuf\Rules",
            reg_value: "NumberOfSIUFInPeriod",
            disabled_value: 0,
        },
        ItemDef {
            category: "Telemetrie",
            name: "Télémétrie App-V",
            description: "Désactiver la télémétrie des applications virtuelles",
            reg_path: r"HKLM\SOFTWARE\Policies\Microsoft\AppV\CEIP",
            reg_value: "CEIPEnable",
            disabled_value: 0,
        },
        ItemDef {
            category: "Recherche",
            name: "Recherche Bing",
            description: "Désactiver les résultats Bing dans la recherche Windows",
            reg_path: r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Search",
            reg_value: "BingSearchEnabled",
            disabled_value: 0,
        },
        ItemDef {
            category: "Recherche",
            name: "Suggestions cloud",
            description: "Désactiver les suggestions de recherche MSA/cloud",
            reg_path: r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Search",
            reg_value: "IsMSACloudSearchEnabled",
            disabled_value: 0,
        },
        ItemDef {
            category: "Publicite",
            name: "Identifiant publicitaire",
            description: "Désactiver le suivi publicitaire des applications",
            reg_path: r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\AdvertisingInfo",
            reg_value: "Enabled",
            disabled_value: 0,
        },
        ItemDef {
            category: "Publicite",
            name: "Expériences personnalisées",
            description: "Désactiver les suggestions basées sur les données de diagnostic",
            reg_path: r"HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent",
            reg_value: "DisableTailoredExperiencesWithDiagnosticData",
            disabled_value: 1,
        },
        ItemDef {
            category: "Publicite",
            name: "Politique de confidentialité",
            description: "Refuser la politique de personnalisation acceptée",
            reg_path: r"HKCU\SOFTWARE\Microsoft\Personalization\Settings",
            reg_value: "AcceptedPrivacyPolicy",
            disabled_value: 0,
        },
        ItemDef {
            category: "Localisation",
            name: "Suivi de position",
            description: "Désactiver le suivi de localisation de l'appareil",
            reg_path: r"HKLM\SOFTWARE\Policies\Microsoft\Windows\LocationAndSensors",
            reg_value: "DisableLocation",
            disabled_value: 1,
        },
        ItemDef {
            category: "Activite",
            name: "Historique d'activité",
            description: "Désactiver la collecte de l'historique d'activité",
            reg_path: r"HKLM\SOFTWARE\Policies\Microsoft\Windows\System",
            reg_value: "EnableActivityFeed",
            disabled_value: 0,
        },
        ItemDef {
            category: "Activite",
            name: "Timeline",
            description: "Désactiver la publication d'activité entre appareils",
            reg_path: r"HKLM\SOFTWARE\Policies\Microsoft\Windows\System",
            reg_value: "PublishUserActivities",
            disabled_value: 0,
        },
        ItemDef {
            category: "Activite",
            name: "Expériences partagées",
            description: "Désactiver le partage d'expériences entre appareils",
            reg_path: r"HKLM\SOFTWARE\Policies\Microsoft\Windows\System",
            reg_value: "EnableCdp",
            disabled_value: 0,
        },
        ItemDef {
            category: "Saisie",
            name: "Écriture manuscrite",
            description: "Désactiver le partage des données d'écriture manuscrite",
            reg_path: r"HKLM\SOFTWARE\Policies\Microsoft\Windows\TabletPC",
            reg_value: "PreventHandwritingDataSharing",
            disabled_value: 1,
        },
        ItemDef {
            category: "Saisie",
            name: "Données de saisie",
            description: "Désactiver la collecte de données de frappe et d'encrage",
            reg_path: r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\TextInput",
            reg_value: "AllowLinguisticDataCollection",
            disabled_value: 0,
        },
        ItemDef {
            category: "Reseau",
            name: "Wi-Fi Sense",
            description: "Désactiver la connexion automatique Wi-Fi Sense",
            reg_path: r"HKLM\SOFTWARE\Microsoft\WcmSvc\wifinetworkmanager\config",
            reg_value: "AutoConnectAllowedOEM",
            disabled_value: 0,
        },
        ItemDef {
            category: "Reseau",
            name: "Presse-papiers cloud",
            description: "Désactiver la synchronisation cloud du presse-papiers",
            reg_path: r"HKLM\SOFTWARE\Policies\Microsoft\Windows\System",
            reg_value: "AllowCrossDeviceClipboard",
            disabled_value: 0,
        },
        ItemDef {
            category: "Telemetrie",
            name: "SmartScreen (télémétrie)",
            description: "Désactiver l'envoi des URLs de navigation à Microsoft via SmartScreen. Facultatif : SmartScreen protège aussi des sites et fichiers malveillants, ce réglage n'entre ni dans le score ni dans les lots",
            reg_path: r"HKLM\SOFTWARE\Policies\Microsoft\Windows\System",
            reg_value: "EnableSmartScreen",
            disabled_value: 0,
        },
        ItemDef {
            category: "Securite",
            name: "Rapport d'erreurs",
            description: "Désactiver l'envoi automatique de rapports d'erreurs",
            reg_path: r"HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting",
            reg_value: "Disabled",
            disabled_value: 1,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_items_non_empty_and_unique_ids() {
        let items = build_items();
        assert!(!items.is_empty(), "liste data_protection vide");
    }

    #[test]
    fn only_smartscreen_is_kept_out_of_the_batches() {
        let excluded: Vec<&str> = build_items().iter().filter(|d| !d.in_batch()).map(|d| d.reg_value).collect();
        assert_eq!(excluded, ["EnableSmartScreen"]);
    }

    #[test]
    fn build_items_categories_known() {
        let items = build_items();
        let known_categories = [
            "Telemetrie",
            "Recherche",
            "Publicite",
            "Activite",
            "Localisation",
            "Reseau",
            "Saisie",
            "Securite",
        ];
        for item in &items {
            assert!(
                known_categories.contains(&item.category),
                "Categorie inconnue: {}",
                item.category
            );
        }
    }
}
