//! Prelevement des compteurs bruts via WMI : CPU global et par coeur logique, RAM,
//! processus. Les taux sont calcules par `sample::metrics_between`.

use std::collections::HashMap;

use wmi::{Variant, WMIConnection};

use crate::utils::wmi_helpers::{v_f64, v_str};

use super::sample::{RawCounter, RawProcess, Sample};

type Row = HashMap<String, Variant>;

fn counter(row: &Row) -> RawCounter {
    RawCounter { ticks: v_f64(row.get("PercentProcessorTime")), timestamp: v_f64(row.get("Timestamp_Sys100NS")) }
}

pub(super) fn take_sample(wmi: &WMIConnection) -> Sample {
    let mut sample = Sample::default();

    // Une instance par coeur logique (Name = "0".."N") plus "_Total". Win32_Processor.LoadPercentage
    // ne donnait qu'une instance par socket et lisait souvent 0 (run-09).
    match wmi.raw_query::<Row>(
        "SELECT Name, PercentProcessorTime, Timestamp_Sys100NS FROM Win32_PerfRawData_PerfOS_Processor",
    ) {
        Ok(rows) => {
            if rows.is_empty() {
                crate::utils::logger::warn("monitor", "requete CPU WMI vide, metriques CPU a 0");
            }
            for row in &rows {
                let name = v_str(row.get("Name"));
                if name == "_Total" {
                    sample.cpu_total = Some(counter(row));
                } else if let Ok(idx) = name.parse::<u32>() {
                    sample.cpu_cores.push((idx, counter(row)));
                }
            }
        }
        Err(e) => {
            crate::utils::logger::warn("monitor", &format!("requete CPU WMI echouee: {}", e));
        }
    }

    match wmi.raw_query::<Row>("SELECT TotalVisibleMemorySize, FreePhysicalMemory FROM Win32_OperatingSystem") {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                sample.ram_total_kb = v_f64(row.get("TotalVisibleMemorySize"));
                sample.ram_free_kb = v_f64(row.get("FreePhysicalMemory"));
            }
        }
        Err(e) => {
            crate::utils::logger::warn("monitor", &format!("requete RAM WMI echouee: {}", e));
        }
    }

    match wmi.raw_query::<Row>(
        "SELECT IDProcess, Name, PercentProcessorTime, WorkingSetPrivate, Timestamp_Sys100NS FROM Win32_PerfRawData_PerfProc_Process",
    ) {
        Ok(rows) => {
            for row in &rows {
                let pid = v_f64(row.get("IDProcess")) as u32;
                sample.processes.insert(
                    pid,
                    RawProcess {
                        name: v_str(row.get("Name")),
                        counter: counter(row),
                        working_set_bytes: v_f64(row.get("WorkingSetPrivate")),
                    },
                );
            }
        }
        Err(e) => {
            crate::utils::logger::warn("monitor", &format!("requete processus WMI echouee: {}", e));
        }
    }

    sample
}
