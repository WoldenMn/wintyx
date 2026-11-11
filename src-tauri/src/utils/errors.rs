use serde::Serialize;

/// Enum semantique pour les erreurs de commandes Tauri.
///
/// Permet au frontend de presenter des messages coherents et actionnables
/// au lieu de strings opaques. Chaque variant sera serialise sous la forme
/// `{ "code": "NotAdmin", "details": "..." }` pour une discrimination facile
/// cote TypeScript.
///
/// # Migration progressive
///
/// Les commandes existantes continuent a fonctionner avec `Result<T, String>`.
/// Les nouvelles commandes devraient utiliser `Result<T, CommandError>` pour
/// beneficier de la discrimination cote frontend. `CommandError` implemente
/// `From<String>` et `From<&str>` pour faciliter l'adoption progressive.
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(tag = "code", content = "details")]
pub enum CommandError {
    /// Action requiert des privileges administrateur. Contenu : feature concernee.
    NotAdmin(String),
    /// Service WMI indisponible ou erreur de requete. Contenu : detail technique.
    WmiUnavailable(String),
    /// Commande externe a depasse son timeout.
    #[serde(rename_all = "camelCase")] // champs internes en camelCase (timeout_ms -> timeoutMs), sans toucher au tag "code"
    Timeout {
        command: String,
        timeout_ms: u64,
    },
    /// Ressource (registre, service, fichier, etc.) introuvable.
    NotFound(String),
    /// Acces refuse (en dehors du cas admin).
    PermissionDenied(String),
    /// Entree utilisateur invalide (nom, chemin, format).
    InvalidInput(String),
    /// Erreur lors d'une operation registre.
    Registry(String),
    /// Echec d'une commande externe (exit code non-0).
    ProcessFailed {
        command: String,
        detail: String,
    },
    /// Erreur generique (fallback pour erreurs legacy String).
    Generic(String),
}

impl CommandError {
    /// Retourne un message utilisateur final en francais.
    pub fn friendly_message(&self) -> String {
        match self {
            CommandError::NotAdmin(feature) => format!(
                "{} nécessite des privilèges administrateur. Relancez Wintyx en mode admin.",
                feature
            ),
            CommandError::WmiUnavailable(detail) => format!(
                "Service WMI indisponible : {}. Vérifier que le service Winmgmt est démarré.",
                detail
            ),
            CommandError::Timeout {
                command,
                timeout_ms,
            } => format!(
                "La commande '{}' a dépassé {} ms. Le système est peut-être surchargé.",
                command, timeout_ms
            ),
            CommandError::NotFound(what) => format!("Introuvable : {}", what),
            CommandError::PermissionDenied(what) => {
                format!("Accès refusé : {}", what)
            }
            CommandError::InvalidInput(detail) => format!("Entree invalide : {}", detail),
            CommandError::Registry(detail) => format!("Erreur registre : {}", detail),
            CommandError::ProcessFailed { command, detail } => {
                format!("Échec de la commande '{}' : {}", command, detail)
            }
            CommandError::Generic(msg) => msg.clone(),
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.friendly_message())
    }
}

impl From<String> for CommandError {
    fn from(s: String) -> Self {
        CommandError::Generic(s)
    }
}

impl From<&str> for CommandError {
    fn from(s: &str) -> Self {
        CommandError::Generic(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_admin_friendly_message_includes_feature() {
        let e = CommandError::NotAdmin("Arreter un service".to_string());
        let msg = e.friendly_message();
        assert!(msg.contains("Arreter un service"));
        assert!(msg.contains("administrateur"));
    }

    #[test]
    fn timeout_friendly_message_includes_command_and_ms() {
        let e = CommandError::Timeout {
            command: "powershell.exe".to_string(),
            timeout_ms: 30000,
        };
        let msg = e.friendly_message();
        assert!(msg.contains("powershell.exe"));
        assert!(msg.contains("30000"));
    }

    #[test]
    fn from_string_wraps_as_generic() {
        let e: CommandError = "legacy error".to_string().into();
        matches!(e, CommandError::Generic(_));
    }

    #[test]
    fn serializes_with_tagged_code_and_details() {
        let e = CommandError::NotAdmin("Feature".to_string());
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(json.get("code").unwrap(), &serde_json::json!("NotAdmin"));
        assert_eq!(json.get("details").unwrap(), &serde_json::json!("Feature"));
    }

    #[test]
    fn timeout_serializes_with_structured_details() {
        let e = CommandError::Timeout {
            command: "reg.exe".to_string(),
            timeout_ms: 10_000,
        };
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(json.get("code").unwrap(), &serde_json::json!("Timeout"));
        let details = json.get("details").unwrap();
        assert_eq!(details.get("command").unwrap(), &serde_json::json!("reg.exe"));
        assert_eq!(details.get("timeout_ms"), None); // rename_all camelCase sur le variant Timeout
        assert_eq!(details.get("timeoutMs").unwrap(), &serde_json::json!(10_000));
    }

    #[test]
    fn display_equals_friendly_message() {
        let e = CommandError::NotFound("service X".to_string());
        assert_eq!(format!("{}", e), e.friendly_message());
    }

    #[test]
    fn all_variants_produce_non_empty_message() {
        let variants = vec![
            CommandError::NotAdmin("f".into()),
            CommandError::WmiUnavailable("d".into()),
            CommandError::Timeout {
                command: "c".into(),
                timeout_ms: 1,
            },
            CommandError::NotFound("n".into()),
            CommandError::PermissionDenied("p".into()),
            CommandError::InvalidInput("i".into()),
            CommandError::Registry("r".into()),
            CommandError::ProcessFailed {
                command: "c".into(),
                detail: "d".into(),
            },
            CommandError::Generic("g".into()),
        ];
        for v in variants {
            assert!(!v.friendly_message().is_empty());
        }
    }
}
