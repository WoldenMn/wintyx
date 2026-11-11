//! Antivirus actif, lu dans `root\SecurityCenter2` et choisi par `productState`.
//!
//! `productState` est un entier dont l'hexadecimal a 6 chiffres se lit par masques
//! (convention communautaire, la seule documentee pour ce champ WMI : Microsoft ne
//! publie que l'enumeration `WSC_SECURITY_PRODUCT_STATE` de l'API `IWscProduct`) :
//! `0xF000` etat (0x0000 inactif, 0x1000 actif, 0x2000 en pause, 0x3000 expire),
//! `0x0F00` editeur (0x0100 Microsoft). Exemple releve : antivirus tiers 0x041000
//! (actif), Windows Defender 0x060100 (inactif, passif derriere un tiers).

use std::collections::HashMap;

use wmi::{COMLibrary, Variant, WMIConnection};

use crate::utils::errors::CommandError;
use crate::utils::wmi_helpers::{run_on_com_thread, v_f64, v_str};

const UNKNOWN: &str = "Inconnu";
const STATE_MASK: u32 = 0xF000;
const STATE_ON: u32 = 0x1000;
const OWNER_MASK: u32 = 0x0F00;
const OWNER_MICROSOFT: u32 = 0x0100;

pub(super) fn active_antivirus() -> Result<String, CommandError> {
    // Namespace distinct de `root\cimv2`, donc init COM separee, sur thread dedie :
    // `COMLibrary::new()` echoue sur le thread principal STA (RPC_E_CHANGED_MODE).
    let products = run_on_com_thread(|| {
        let com = COMLibrary::new().map_err(|e| CommandError::WmiUnavailable(e.to_string()))?;
        let wmi = WMIConnection::with_namespace_path("root\\SecurityCenter2", com)
            .map_err(|e| CommandError::WmiUnavailable(e.to_string()))?;
        let rows: Vec<HashMap<String, Variant>> = wmi
            .raw_query("SELECT displayName, productState FROM AntiVirusProduct")
            .map_err(|e| CommandError::WmiUnavailable(e.to_string()))?;
        Ok(rows
            .iter()
            .map(|row| (v_str(row.get("displayName")), v_f64(row.get("productState")) as u32))
            .collect::<Vec<_>>())
    });
    products
        .map(|products| select_active(&products))
        .inspect_err(|e| {
            crate::utils::logger::warn(
                "system_info",
                &format!("antivirus : lecture SecurityCenter2 impossible: {}", e.friendly_message()),
            );
        })
}

/// Produit dont la protection temps reel est active ; un tiers passe avant
/// Microsoft quand les deux le sont. Aucun actif : `Unknown`, pour ne pas
/// annoncer une protection absente.
fn select_active(products: &[(String, u32)]) -> String {
    let active = |p: &&(String, u32)| p.1 & STATE_MASK == STATE_ON && !p.0.trim().is_empty();
    let third_party = products
        .iter()
        .filter(active)
        .find(|p| p.1 & OWNER_MASK != OWNER_MICROSOFT);
    third_party
        .or_else(|| products.iter().find(active))
        .map(|p| p.0.trim().to_string())
        .unwrap_or_else(|| UNKNOWN.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(name: &str, state: u32) -> (String, u32) {
        (name.to_string(), state)
    }

    #[test]
    fn real_capture_picks_the_active_third_party_not_the_passive_defender() {
        let products = [p("Windows Defender", 393472), p("Antivirus tiers", 266240)];
        assert_eq!(select_active(&products), "Antivirus tiers");
        let reversed = [p("Antivirus tiers", 266240), p("Windows Defender", 393472)];
        assert_eq!(select_active(&reversed), "Antivirus tiers");
    }

    #[test]
    fn defender_alone_and_active_is_chosen() {
        assert_eq!(select_active(&[p("Windows Defender", 0x061100)]), "Windows Defender");
    }

    #[test]
    fn third_party_wins_when_both_are_active() {
        let products = [p("Windows Defender", 0x061100), p("Autre AV", 0x041000)];
        assert_eq!(select_active(&products), "Autre AV");
    }

    #[test]
    fn nothing_active_gives_unknown() {
        let products = [p("Windows Defender", 393472), p("Snoozed AV", 0x042000), p("Expired AV", 0x043000)];
        assert_eq!(select_active(&products), "Inconnu");
        assert_eq!(select_active(&[]), "Inconnu");
    }
}
