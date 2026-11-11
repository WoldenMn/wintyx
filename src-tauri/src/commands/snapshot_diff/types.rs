use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotGroup {
    pub timestamp: String,
    pub backup_files: Vec<String>,
    pub key_count: usize,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum KeyValueChange {
    /// Valeur presente en B mais pas en A.
    /// Detecte en mode vs-snapshot, et en mode vs-live pour les cles Wintyx critiques (v2.5.0+).
    #[serde(rename_all = "camelCase")]
    Added { name: String, current: String },
    #[serde(rename_all = "camelCase")]
    Removed { name: String, snapshot: String },
    #[serde(rename_all = "camelCase")]
    Modified { name: String, snapshot: String, current: String },
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RegistryDiff {
    pub key_path: String,
    pub changes: Vec<KeyValueChange>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDiff {
    pub snapshot_timestamp: String,
    /// Some(timestamp) en mode vs-snapshot, None en mode vs-live.
    pub comparison_target: Option<String>,
    pub keys: Vec<RegistryDiff>,
    pub total_changes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChangeRequestType {
    /// Valeur diffère entre snapshot et live → restaurer celle du snapshot
    RestoreModified,
    /// Valeur absente en live mais présente en snapshot → recréer
    RestoreRemoved,
    /// Valeur présente en live mais absente en snapshot → supprimer
    UndoAdded,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRequest {
    pub key_path: String,
    pub value_name: String,
    pub change_type: ChangeRequestType,
    /// Requis pour RestoreModified et RestoreRemoved (raw .reg : "dword:00000001" ou "\"hello\"").
    /// Ignoré pour UndoAdded.
    pub target_value: Option<String>,
}
