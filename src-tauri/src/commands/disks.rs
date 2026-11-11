use serde::Serialize;
use std::collections::HashMap;
use wmi::Variant;
use crate::utils::constants;
use crate::utils::errors::CommandError;
use crate::utils::wmi_helpers::{v_str, v_f64, v_i32, with_wmi_conn};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub name: String,
    pub volume_name: String,
    pub used_gb: f64,
    pub total_gb: f64,
    pub used_percent: u32,
    pub disk_type: String,
    pub free_gb: f64,
    pub filesystem: String,
}

#[tauri::command]
#[specta::specta]
pub async fn get_disks() -> Result<Vec<DiskInfo>, CommandError> {
    crate::utils::blocking::read_result(get_disks_sync).await
}

pub fn get_disks_sync() -> Result<Vec<DiskInfo>, CommandError> {
    crate::perf_timed!("get_disks", {
        // Fix v2.7.9 H4 : `with_wmi_conn` centralise l'init COM + WMI (~200ms gain
        // vs init direct, cf. v2.1.4 H1 + v2.7.9 H3 sur memory/drivers/system_info).
        with_wmi_conn(|wmi| {
            let results: Vec<HashMap<String, Variant>> = wmi
                .raw_query("SELECT DeviceID, VolumeName, Size, FreeSpace, DriveType, FileSystem FROM Win32_LogicalDisk WHERE DriveType=3 OR DriveType=2 OR DriveType=5")
                .map_err(|e| CommandError::WmiUnavailable(e.to_string()))?;

            let disks = results
                .iter()
                .map(|row| {
                let size_bytes = v_f64(row.get("Size"));
                let free_bytes = v_f64(row.get("FreeSpace"));
                let total_gb = (size_bytes / constants::GB * 100.0).round() / 100.0;
                let free_gb = (free_bytes / constants::GB * 100.0).round() / 100.0;
                let used_gb = ((size_bytes - free_bytes) / constants::GB * 100.0).round() / 100.0;
                let drive_type = v_i32(row.get("DriveType"));

                let type_str = match drive_type {
                    2 => "Amovible",
                    3 => "Disque local",
                    5 => "CD-ROM",
                    _ => "Inconnu",
                };

                let used_percent = if total_gb > 0.0 {
                    (used_gb / total_gb * 100.0).round() as u32
                } else {
                    0
                };

                DiskInfo {
                    name: v_str(row.get("DeviceID")),
                    volume_name: v_str(row.get("VolumeName")),
                    used_gb,
                    total_gb,
                    used_percent,
                    disk_type: type_str.to_string(),
                    free_gb,
                    filesystem: v_str(row.get("FileSystem")),
                }
            })
            .collect();

            Ok(disks)
        })
    })
}
