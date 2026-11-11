use serde::Serialize;
use std::collections::HashMap;
use wmi::{Variant, WMIConnection};
use crate::utils::constants;
use crate::utils::errors::CommandError;
use crate::utils::wmi_helpers::{v_str, v_f64, v_u32, kb_to_gb_rounded, with_wmi_conn};
use crate::utils::process;

fn wmi_err(e: impl std::fmt::Display) -> CommandError {
    CommandError::WmiUnavailable(e.to_string())
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStick {
    pub bank_label: String,
    pub device_locator: String,
    pub capacity_gb: f64,
    pub speed_mhz: u32,
    pub memory_type: String,
    pub form_factor: String,
    pub manufacturer: String,
    pub part_number: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MemorySummary {
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub used_percent: u32,
    pub slots_filled: u32,
    pub total_slots: u32,
    pub max_speed_mhz: u32,
}

fn stick_from_row(row: &HashMap<String, Variant>) -> MemoryStick {
    let capacity_bytes = v_f64(row.get("Capacity"));
    let speed = {
        let s = v_u32(row.get("ConfiguredClockSpeed"));
        if s > 0 { s } else { v_u32(row.get("Speed")) }
    };
    let memory_type = match v_u32(row.get("SMBIOSMemoryType")) {
        20 => "DDR",
        21 => "DDR2",
        24 => "DDR3",
        26 => "DDR4",
        34 => "DDR5",
        _ => "Inconnu",
    };
    let form_factor = match v_u32(row.get("FormFactor")) {
        8 => "DIMM",
        12 => "SO-DIMM",
        _ => "Autre",
    };

    MemoryStick {
        bank_label: v_str(row.get("BankLabel")),
        device_locator: v_str(row.get("DeviceLocator")),
        capacity_gb: (capacity_bytes / constants::GB * 100.0).round() / 100.0,
        speed_mhz: speed,
        memory_type: memory_type.to_string(),
        form_factor: form_factor.to_string(),
        manufacturer: v_str(row.get("Manufacturer")),
        part_number: v_str(row.get("PartNumber")),
    }
}

fn total_slots_from(rows: &[HashMap<String, Variant>]) -> u32 {
    rows.iter().map(|row| v_u32(row.get("MemoryDevices"))).sum()
}

fn collect_memory_sticks(wmi: &WMIConnection) -> Result<Vec<MemoryStick>, CommandError> {
    let results: Vec<HashMap<String, Variant>> = wmi
        .raw_query("SELECT BankLabel, DeviceLocator, Capacity, ConfiguredClockSpeed, Speed, SMBIOSMemoryType, FormFactor, Manufacturer, PartNumber FROM Win32_PhysicalMemory")
        .map_err(wmi_err)?;

    let sticks = results.iter().map(stick_from_row).collect();

    Ok(sticks)
}

#[tauri::command]
#[specta::specta]
pub async fn get_memory_sticks() -> Result<Vec<MemoryStick>, CommandError> {
    crate::utils::blocking::read_result(get_memory_sticks_sync).await
}

pub fn get_memory_sticks_sync() -> Result<Vec<MemoryStick>, CommandError> {
    with_wmi_conn(collect_memory_sticks)
}

#[tauri::command]
#[specta::specta]
pub async fn get_memory_summary() -> Result<MemorySummary, CommandError> {
    crate::utils::blocking::read_result(get_memory_summary_sync).await
}

fn memory_summary_from(
    os_rows: &[HashMap<String, Variant>],
    sticks: Result<Vec<MemoryStick>, CommandError>,
    slot_rows: Result<Vec<HashMap<String, Variant>>, CommandError>,
) -> Result<MemorySummary, CommandError> {
    let row = os_rows
        .first()
        .ok_or_else(|| wmi_err("Win32_OperatingSystem sans ligne : memoire inconnue"))?;
    let total_kb = v_f64(row.get("TotalVisibleMemorySize"));
    let free_kb = v_f64(row.get("FreePhysicalMemory"));
    if total_kb <= 0.0 {
        return Err(wmi_err("Win32_OperatingSystem : memoire totale illisible"));
    }
    let used_kb = (total_kb - free_kb).max(0.0);
    let total_gb = kb_to_gb_rounded(total_kb);
    let available_gb = kb_to_gb_rounded(free_kb);
    let used_gb = kb_to_gb_rounded(used_kb);
    let used_percent = if total_gb > 0.0 {
        ((used_gb / total_gb * 100.0).round() as u32).min(100)
    } else {
        0
    };

    let sticks = sticks?;
    let slots_filled = sticks.len() as u32;
    let max_speed = sticks.iter().map(|s| s.speed_mhz).max().unwrap_or(0);

    let total_slots = total_slots_from(&slot_rows?);
    if total_slots < slots_filled {
        crate::utils::logger::warn(
            "memory",
            &format!("{} barrettes pour {} emplacements déclarés: données WMI incohérentes", slots_filled, total_slots),
        );
    }

    Ok(MemorySummary {
        total_gb,
        used_gb,
        available_gb,
        used_percent,
        slots_filled,
        total_slots,
        max_speed_mhz: max_speed,
    })
}

pub fn get_memory_summary_sync() -> Result<MemorySummary, CommandError> {
    crate::perf_timed!("get_memory_summary", {
        with_wmi_conn(|wmi| {
            let os_results: Vec<HashMap<String, Variant>> = wmi
                .raw_query("SELECT TotalVisibleMemorySize, FreePhysicalMemory FROM Win32_OperatingSystem")
                .map_err(wmi_err)?;
            let sticks = collect_memory_sticks(wmi).inspect_err(|e| {
                crate::utils::logger::warn("memory", &format!("collect_memory_sticks échec: {}", e));
            });
            let slot_results = wmi
                .raw_query("SELECT MemoryDevices FROM Win32_PhysicalMemoryArray")
                .map_err(wmi_err)
                .inspect_err(|e| {
                    crate::utils::logger::warn("memory", &format!("requête PhysicalMemoryArray échec: {}", e));
                });
            memory_summary_from(&os_results, sticks, slot_results)
        })
    })
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MemoryConsumer {
    pub name: String,
    pub pid: u32,
    pub memory_mb: f64,
}

#[tauri::command]
#[specta::specta]
pub async fn get_memory_consumers() -> Result<Vec<MemoryConsumer>, CommandError> {
    crate::utils::blocking::read_result(get_memory_consumers_sync).await
}

pub fn get_memory_consumers_sync() -> Result<Vec<MemoryConsumer>, CommandError> {
    let result = process::run(
        "powershell",
        &format!("-NoProfile -Command \"Get-Process | Sort-Object WorkingSet64 -Descending | Select-Object -First {} | ForEach-Object {{ \\\"$($_.Id)|$($_.ProcessName)|$($_.WorkingSet64)\\\" }}\"", constants::TOP_PROCESSES_CONSUMERS),
        constants::TIMEOUT_LONG,
    );
    consumers_from(result)
}

fn consumers_from(result: process::CmdResult) -> Result<Vec<MemoryConsumer>, CommandError> {
    if !result.success {
        crate::utils::logger::warn(
            "memory",
            &format!("get_memory_consumers: échec Get-Process (exit {}): {}", result.exit_code, result.stderr),
        );
        return Err(CommandError::ProcessFailed {
            command: "Get-Process".into(),
            detail: if result.stderr.trim().is_empty() {
                format!("code de sortie {}", result.exit_code)
            } else {
                result.stderr.trim().to_string()
            },
        });
    }
    let mut consumers = Vec::new();
    for line in result.stdout.lines().filter(|l| !l.trim().is_empty()) {
        let Some(parts) = process::parse_pipe_fields(line, 3) else {
            crate::utils::logger::warn("memory", &format!("ligne Get-Process ignoree: {}", line));
            continue;
        };
        let (Ok(pid), Ok(mem_bytes)) = (parts[0].parse::<u32>(), parts[2].parse::<f64>()) else {
            crate::utils::logger::warn("memory", &format!("ligne Get-Process illisible: {}", line));
            continue;
        };
        consumers.push(MemoryConsumer {
            name: parts[1].to_string(),
            pid,
            memory_mb: (mem_bytes / constants::MB * 10.0).round() / 10.0,
        });
    }
    Ok(consumers)
}

#[cfg(test)]
mod tests;
