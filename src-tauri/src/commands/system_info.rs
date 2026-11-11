use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use wmi::{Variant, WMIConnection};
use crate::utils::constants;
use crate::utils::errors::CommandError;
use crate::utils::wmi_helpers::{v_str, v_f64, v_u32, kb_to_gb_rounded, with_wmi_conn};

mod antivirus;
mod assemble;

use assemble::{assemble as assemble_reads, refresh, Reads, VolatileReads};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfoData {
    pub os_name: String,
    pub cpu_name: String,
    pub total_ram_gb: f64,
    pub used_ram_gb: f64,
    pub disk_used_percent: u32,
    pub uptime_seconds: u64,
    pub computer_name: String,
    pub antivirus_name: String,
    pub cpu_usage: u32,
    pub gpu_name: String,
    pub cpu_cores: u32,
    pub cpu_clock_mhz: u32,
}

// PERF-02 v2.16.0 : seuls les champs STATIQUES sont caches (TTL 60s). Les champs
// volatils (used_ram_gb, cpu_usage, uptime_seconds, disk_used_percent) sont
// recalcules a chaque appel : le Dashboard les affiche et son bouton Rafraichir
// serait un no-op visuel s'ils etaient geles. Le gain vient du saut de la
// connexion SecurityCenter2 (antivirus, COM separe) et des requetes CPU/GPU.
#[derive(Debug, Clone)]
struct StaticSystemInfo {
    os_name: String,
    cpu_name: String,
    total_ram_gb: f64,
    antivirus_name: String,
    gpu_name: String,
    cpu_cores: u32,
    cpu_clock_mhz: u32,
}

static SYSTEM_INFO_CACHE: OnceLock<Mutex<Option<(StaticSystemInfo, Instant)>>> = OnceLock::new();

fn system_info_cache() -> &'static Mutex<Option<(StaticSystemInfo, Instant)>> {
    SYSTEM_INFO_CACHE.get_or_init(|| Mutex::new(None))
}

fn is_fresh(cached_at: Instant, now: Instant, ttl_secs: u64) -> bool {
    now.saturating_duration_since(cached_at) < Duration::from_secs(ttl_secs)
}

type Row = HashMap<String, Variant>;

/// Une requete WMI en echec ne doit pas se confondre avec une absence de ligne :
/// l'echec est journalise et rendu en `None`, l'appelant traite le champ comme non lu.
fn query_rows(con: &WMIConnection, query: &str) -> Option<Vec<Row>> {
    match con.raw_query(query) {
        Ok(rows) => Some(rows),
        Err(e) => {
            crate::utils::logger::warn("system_info", &format!("requete WMI en echec ({}): {}", query, e));
            None
        }
    }
}

fn first_row(con: &WMIConnection, query: &str) -> Option<Row> {
    let row = query_rows(con, query)?.into_iter().next();
    if row.is_none() {
        crate::utils::logger::warn("system_info", &format!("requete WMI sans ligne ({})", query));
    }
    row
}

fn present<'a>(row: &'a Row, key: &str) -> Option<&'a Variant> {
    row.get(key).filter(|v| !matches!(v, Variant::Null | Variant::Empty))
}

fn non_empty_text(row: &Row, key: &str) -> Option<String> {
    let text = v_str(present(row, key)).trim().to_string();
    (!text.is_empty()).then_some(text)
}

fn get_os_and_ram(con: &WMIConnection) -> Option<(String, f64, f64)> {
    let row = first_row(con, "SELECT Caption, TotalVisibleMemorySize, FreePhysicalMemory FROM Win32_OperatingSystem")?;
    let os_name = non_empty_text(&row, "Caption")?;
    let total_kb = v_f64(Some(present(&row, "TotalVisibleMemorySize")?));
    let free_kb = v_f64(Some(present(&row, "FreePhysicalMemory")?));
    if total_kb <= 0.0 {
        return None;
    }
    let used_kb = (total_kb - free_kb).max(0.0);
    Some((os_name, kb_to_gb_rounded(total_kb), kb_to_gb_rounded(used_kb)))
}

fn get_cpu_info(con: &WMIConnection) -> Option<(String, u32, u32)> {
    let row = first_row(con, "SELECT Name, NumberOfCores, MaxClockSpeed FROM Win32_Processor")?;
    let name = non_empty_text(&row, "Name")?;
    let cores = v_u32(Some(present(&row, "NumberOfCores")?));
    let clock = v_u32(Some(present(&row, "MaxClockSpeed")?));
    Some((name, cores, clock))
}

fn get_gpu_info(con: &WMIConnection) -> Option<String> {
    let row = first_row(con, "SELECT Name FROM Win32_VideoController")?;
    non_empty_text(&row, "Name")
}

fn get_disk_used_percent(con: &WMIConnection) -> Option<u32> {
    let row = first_row(con, "SELECT Size, FreeSpace FROM Win32_LogicalDisk WHERE DeviceID='C:'")?;
    let size = v_f64(Some(present(&row, "Size")?));
    let free = v_f64(Some(present(&row, "FreeSpace")?));
    (size > 0.0).then(|| ((size - free) / size * 100.0).round() as u32)
}

fn get_uptime_seconds(con: &WMIConnection) -> Option<u64> {
    let row = first_row(con, "SELECT LastBootUpTime FROM Win32_OperatingSystem")?;
    parse_wmi_datetime_to_uptime(&non_empty_text(&row, "LastBootUpTime")?)
}

fn get_cpu_usage(con: &WMIConnection) -> Option<u32> {
    // _Total de Win32_PerfFormattedData_PerfOS_Processor (cf. monitor.rs run-09) :
    // Win32_Processor.LoadPercentage lisait souvent 0.
    let row = first_row(con, "SELECT Name, PercentProcessorTime FROM Win32_PerfFormattedData_PerfOS_Processor WHERE Name='_Total'")?;
    Some(v_f64(Some(present(&row, "PercentProcessorTime")?)).round() as u32)
}

#[tauri::command]
#[specta::specta]
pub async fn get_system_info() -> Result<SystemInfoData, CommandError> {
    crate::utils::blocking::read_result(get_system_info_sync).await
}

pub fn get_system_info_sync() -> Result<SystemInfoData, CommandError> {
    crate::perf_timed!("get_system_info", {
        let computer_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Inconnu".to_string());

        let cached = system_info_cache()
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
            .filter(|(_, at)| is_fresh(*at, Instant::now(), constants::SYSTEM_INFO_CACHE_TTL_SECS))
            .map(|(info, _)| info);

        if let Some(cached) = cached {
            let (os_ram, disk_used_percent, uptime_seconds, cpu_usage) =
                with_wmi_conn(|wmi| {
                    Ok((
                        get_os_and_ram(wmi),
                        get_disk_used_percent(wmi),
                        get_uptime_seconds(wmi),
                        get_cpu_usage(wmi),
                    ))
                })?;
            return refresh(
                cached,
                VolatileReads { os_ram, disk_used_percent, uptime_seconds, cpu_usage },
                computer_name,
            );
        }

        // Fix v2.7.9 H3 : `with_wmi_conn` centralise l'init COM + WMI (gain ~200-400ms
        // mesurable sur les commandes en cascade WMI, cf. v2.1.4 H1 sur memory/drivers).
        // L'ancien code creait COM + WMI a chaque appel sans cache.
        // Antivirus reste en init separe car namespace different (root\SecurityCenter2).
        // Les deux inits COM/WMI etant independantes, elles s'executent en parallele
        // (BUG-05 v2.16.0, meme pattern thread::scope que threat_scanner).
        let (antivirus, wmi_result) = std::thread::scope(|s| {
            let av_handle = s.spawn(antivirus::active_antivirus);
            let wmi_handle = s.spawn(|| {
                with_wmi_conn(|wmi| {
                    Ok((
                        get_os_and_ram(wmi),
                        get_cpu_info(wmi),
                        get_gpu_info(wmi),
                        get_disk_used_percent(wmi),
                        get_uptime_seconds(wmi),
                        get_cpu_usage(wmi),
                    ))
                })
            });
            let antivirus = av_handle.join().unwrap_or_else(|_| {
                Err(CommandError::WmiUnavailable("thread antivirus: panic".to_string()))
            });
            let wmi_result = wmi_handle.join().unwrap_or_else(|_| {
                Err(CommandError::WmiUnavailable("thread WMI: panic".to_string()))
            });
            (antivirus, wmi_result)
        });

        let (os_ram, cpu, gpu, disk_used_percent, uptime_seconds, cpu_usage) = wmi_result?;
        let (info, cacheable) = assemble_reads(
            Reads {
                os_ram,
                cpu,
                gpu,
                disk_used_percent,
                uptime_seconds,
                cpu_usage,
                antivirus: antivirus.ok(),
            },
            computer_name,
        )?;

        if let (Some(statics), Ok(mut guard)) = (cacheable, system_info_cache().lock()) {
            *guard = Some((statics, Instant::now()));
        }

        Ok(info)
    })
}

fn parse_wmi_datetime_to_uptime(wmi_dt: &str) -> Option<u64> {
    if wmi_dt.len() < 14 || !wmi_dt.is_ascii() {
        return None;
    }
    let year: i32 = wmi_dt[0..4].parse().ok()?;
    let month: u32 = wmi_dt[4..6].parse().ok()?;
    let day: u32 = wmi_dt[6..8].parse().ok()?;
    let hour: u32 = wmi_dt[8..10].parse().ok()?;
    let min: u32 = wmi_dt[10..12].parse().ok()?;
    let sec: u32 = wmi_dt[12..14].parse().ok()?;

    let boot = chrono::NaiveDate::from_ymd_opt(year, month, day)?
        .and_hms_opt(hour, min, sec)?;
    let now = chrono::Local::now().naive_local();
    let duration = now.signed_duration_since(boot);
    Some(duration.num_seconds().max(0) as u64)
}

#[cfg(test)]
mod tests;
