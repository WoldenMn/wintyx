use crate::utils::errors::CommandError;
use crate::utils::logger;

use super::{StaticSystemInfo, SystemInfoData};

const UNKNOWN: &str = "Inconnu";

pub(super) struct Reads {
    pub os_ram: Option<(String, f64, f64)>,
    pub cpu: Option<(String, u32, u32)>,
    pub gpu: Option<String>,
    pub disk_used_percent: Option<u32>,
    pub uptime_seconds: Option<u64>,
    pub cpu_usage: Option<u32>,
    pub antivirus: Option<String>,
}

fn log_unread(fields: &[&str]) {
    if !fields.is_empty() {
        logger::warn(
            "system_info",
            &format!(
                "lecture impossible, valeur par defaut renvoyee et rien en cache : {}",
                fields.join(", ")
            ),
        );
    }
}

fn os_ram_unreadable() -> CommandError {
    CommandError::WmiUnavailable(
        "Win32_OperatingSystem illisible : systeme et memoire inconnus".into(),
    )
}

pub(super) fn assemble(
    reads: Reads,
    computer_name: String,
) -> Result<(SystemInfoData, Option<StaticSystemInfo>), CommandError> {
    let (os_name, total_ram_gb, used_ram_gb) = reads.os_ram.ok_or_else(os_ram_unreadable)?;

    let mut unread: Vec<&str> = Vec::new();
    let mut take = |present: bool, name: &'static str| {
        if !present {
            unread.push(name);
        }
    };
    take(reads.cpu.is_some(), "processeur");
    take(reads.gpu.is_some(), "carte graphique");
    take(reads.antivirus.is_some(), "antivirus");
    take(reads.disk_used_percent.is_some(), "disque C:");
    take(reads.uptime_seconds.is_some(), "uptime");
    take(reads.cpu_usage.is_some(), "charge CPU");

    let statics_complete = reads.cpu.is_some() && reads.gpu.is_some() && reads.antivirus.is_some();
    log_unread(&unread);

    let (cpu_name, cpu_cores, cpu_clock_mhz) = reads.cpu.unwrap_or_else(|| (UNKNOWN.into(), 0, 0));
    let gpu_name = reads.gpu.unwrap_or_else(|| UNKNOWN.into());
    let antivirus_name = reads.antivirus.unwrap_or_else(|| UNKNOWN.into());

    let cacheable = statics_complete.then(|| StaticSystemInfo {
        os_name: os_name.clone(),
        cpu_name: cpu_name.clone(),
        total_ram_gb,
        antivirus_name: antivirus_name.clone(),
        gpu_name: gpu_name.clone(),
        cpu_cores,
        cpu_clock_mhz,
    });

    let info = SystemInfoData {
        os_name,
        cpu_name,
        total_ram_gb,
        used_ram_gb,
        disk_used_percent: reads.disk_used_percent.unwrap_or(0),
        uptime_seconds: reads.uptime_seconds.unwrap_or(0),
        computer_name,
        antivirus_name,
        cpu_usage: reads.cpu_usage.unwrap_or(0),
        gpu_name,
        cpu_cores,
        cpu_clock_mhz,
    };
    Ok((info, cacheable))
}

pub(super) struct VolatileReads {
    pub os_ram: Option<(String, f64, f64)>,
    pub disk_used_percent: Option<u32>,
    pub uptime_seconds: Option<u64>,
    pub cpu_usage: Option<u32>,
}

pub(super) fn refresh(
    cached: StaticSystemInfo,
    reads: VolatileReads,
    computer_name: String,
) -> Result<SystemInfoData, CommandError> {
    let (_, _, used_ram_gb) = reads.os_ram.ok_or_else(os_ram_unreadable)?;
    let mut unread: Vec<&str> = Vec::new();
    if reads.disk_used_percent.is_none() {
        unread.push("disque C:");
    }
    if reads.uptime_seconds.is_none() {
        unread.push("uptime");
    }
    if reads.cpu_usage.is_none() {
        unread.push("charge CPU");
    }
    log_unread(&unread);

    Ok(SystemInfoData {
        os_name: cached.os_name,
        cpu_name: cached.cpu_name,
        total_ram_gb: cached.total_ram_gb,
        used_ram_gb,
        disk_used_percent: reads.disk_used_percent.unwrap_or(0),
        uptime_seconds: reads.uptime_seconds.unwrap_or(0),
        computer_name,
        antivirus_name: cached.antivirus_name,
        cpu_usage: reads.cpu_usage.unwrap_or(0),
        gpu_name: cached.gpu_name,
        cpu_cores: cached.cpu_cores,
        cpu_clock_mhz: cached.cpu_clock_mhz,
    })
}

#[cfg(test)]
mod tests;
