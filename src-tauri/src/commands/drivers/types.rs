use serde::Serialize;

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DriverInfo {
    pub device_name: String,
    pub driver_version: String,
    pub driver_date: String,
    pub manufacturer: String,
    pub driver_provider: String,
    pub device_class: String,
    pub is_signed: bool,
    pub signer: String,
    pub status: String,
    pub inf_name: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ProblemDevice {
    pub device_name: String,
    pub manufacturer: String,
    pub device_class: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DriverSummary {
    pub total: u32,
    pub unsigned: u32,
    pub problems: u32,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IssueKind {
    Duplicate,
    Phantom,
    Obsolete,
    ThirdPartyOem,
    Unsigned,
}

#[derive(Debug, Clone, Copy, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DriverIssue {
    pub id: String,
    pub kind: IssueKind,
    pub severity: Severity,
    pub device_name: String,
    pub manufacturer: String,
    pub driver_version: String,
    pub driver_date: String,
    pub provider: String,
    pub inf_name: String,
    pub instance_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DriverAuditSummary {
    pub total: u32,
    pub unsigned: u32,
    pub duplicates: u32,
    pub phantoms: u32,
    pub obsolete: u32,
    pub third_party: u32,
    pub problems: u32,
}
