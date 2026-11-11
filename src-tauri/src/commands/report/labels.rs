//! Libelles francais des valeurs techniques du backend dans le rapport HTML.
//! Meme table que `src/lib/constants.ts` (CATEGORY_LABELS_FR) cote interface.

pub(super) fn category(raw: &str) -> &str {
    match raw {
        "Entertainment" => "Divertissement",
        "Utility" => "Utilitaire",
        "System" => "Système",
        "Productivity" => "Productivité",
        "Network" | "Reseau" => "Réseau",
        "Gaming" => "Jeux",
        "Remote Access" => "Accès distant",
        "Media" => "Média",
        "Access Control" => "Contrôle d'accès",
        "Credentials" => "Identifiants",
        "Scripting" => "Scripts",
        "Auditing" => "Audit",
        "File Sharing" => "Partage de fichiers",
        "Accounts" => "Comptes",
        "Telemetrie" => "Télémétrie",
        "Publicite" => "Publicité",
        "Activite" => "Activité",
        "Securite" => "Sécurité",
        "Energie" => "Énergie",
        "Demarrage" => "Démarrage",
        other => other,
    }
}

pub(super) fn risk(raw: &str) -> &str {
    match raw.to_ascii_lowercase().as_str() {
        "low" => "Faible",
        "medium" => "Moyen",
        "high" => "Élevé",
        _ => raw,
    }
}

pub(super) fn hardening_level(raw: &str) -> &str {
    match raw {
        "Essential" => "Essentiel",
        "Recommended" => "Recommandé",
        other => other,
    }
}

pub(super) fn service_status(raw: &str) -> &str {
    match raw {
        "Running" => "Actif",
        "Stopped" => "Arrêté",
        "Absent" => "Non installé",
        "Unknown" => "Inconnu",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_are_translated_and_accented() {
        assert_eq!(category("Remote Access"), "Accès distant");
        assert_eq!(category("Telemetrie"), "Télémétrie");
        assert_eq!(category("Reseau"), "Réseau");
        assert_eq!(category("Network"), "Réseau");
        assert_eq!(category("Energie"), "Énergie");
    }

    #[test]
    fn already_french_or_unknown_values_pass_through() {
        assert_eq!(category("Affichage"), "Affichage");
        assert_eq!(category("Inconnue chez nous"), "Inconnue chez nous");
    }

    #[test]
    fn risk_levels_services_and_hardening_levels_are_translated() {
        assert_eq!(risk("Low"), "Faible");
        assert_eq!(risk("high"), "Élevé");
        assert_eq!(hardening_level("Essential"), "Essentiel");
        assert_eq!(hardening_level("Recommended"), "Recommandé");
        assert_eq!(service_status("Stopped"), "Arrêté");
        assert_eq!(service_status("Absent"), "Non installé");
    }
}
