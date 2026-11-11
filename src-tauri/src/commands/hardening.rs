use serde::Serialize;
use crate::utils::batch::BatchResult;
use crate::utils::{admin, constants, process, registry};

mod states;
use states::{
    account_lockout_state, credential_guard_check_state, guest_account_state, score_from_states,
    spooler_service_state, wdigest_check_state, CheckState,
};
#[cfg(test)]
use states::{
    credential_guard_state, decode_secedit_inf, guest_state, lockout_state_from_inf, spooler_state,
    wdigest_state, GUEST_DISABLE_COMMAND, GUEST_QUERY_ARGS,
};

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct HardeningCheck {
    pub id: String,
    pub category: String,
    pub name: String,
    pub description: String,
    pub risk_level: String,
    pub is_secure: bool,
    pub is_unknown: bool,
    pub is_fixable: bool,
}

struct CheckDef {
    id: &'static str,
    category: &'static str,
    name: &'static str,
    description: &'static str,
    risk_level: &'static str,
    reg_path: &'static str,
    reg_value: &'static str,
    secure_value: i32,
    fix_type: &'static str,
    fix_target: &'static str,
}

fn build_checks() -> Vec<CheckDef> {
    vec![
        CheckDef { id: "RDP", category: "Remote Access", name: "Bureau à distance (RDP)", description: "Désactive RDP pour prévenir les attaques d'accès à distance", risk_level: "Essential", reg_path: r"HKLM\SYSTEM\CurrentControlSet\Control\Terminal Server", reg_value: "fDenyTSConnections", secure_value: 1, fix_type: "Registry", fix_target: "" },
        CheckDef { id: "SMBv1", category: "Network", name: "Protocole SMBv1", description: "Désactive SMBv1 non sécurisé (vecteur d'attaque WannaCry)", risk_level: "Essential", reg_path: r"HKLM\SYSTEM\CurrentControlSet\Services\LanmanServer\Parameters", reg_value: "SMB1", secure_value: 0, fix_type: "Registry", fix_target: "" },
        CheckDef { id: "AutoRun", category: "Media", name: "Exécution automatique (AutoRun/AutoPlay)", description: "Désactive l'exécution automatique pour prévenir les malwares USB", risk_level: "Essential", reg_path: r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer", reg_value: "NoDriveTypeAutoRun", secure_value: 255, fix_type: "Registry", fix_target: "" },
        CheckDef { id: "UAC", category: "Access Control", name: "Niveau UAC", description: "Vérifie que le contrôle de compte utilisateur (UAC) est activé", risk_level: "Essential", reg_path: r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System", reg_value: "EnableLUA", secure_value: 1, fix_type: "Registry", fix_target: "" },
        CheckDef { id: "WDigest", category: "Credentials", name: "Authentification WDigest", description: "Désactive WDigest pour empêcher la mise en cache des mots de passe en clair", risk_level: "Essential", reg_path: r"HKLM\SYSTEM\CurrentControlSet\Control\SecurityProviders\WDigest", reg_value: "UseLogonCredential", secure_value: 0, fix_type: "Registry", fix_target: "" },
        CheckDef { id: "LLMNR", category: "Network", name: "LLMNR", description: "Désactive LLMNR pour prévenir l'empoisonnement de résolution de noms", risk_level: "Essential", reg_path: r"HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient", reg_value: "EnableMulticast", secure_value: 0, fix_type: "Registry", fix_target: "" },
        CheckDef { id: "NetBIOS", category: "Network", name: "NetBIOS sur TCP/IP", description: "Désactive NetBIOS pour réduire la surface d'attaque réseau", risk_level: "Essential", reg_path: r"HKLM\SYSTEM\CurrentControlSet\Services\NetBT\Parameters", reg_value: "NodeType", secure_value: 2, fix_type: "Registry", fix_target: "" },
        CheckDef { id: "CredentialGuard", category: "Credentials", name: "Credential Guard", description: "Isole les identifiants par virtualisation. Configuration manuelle : stratégie de groupe, avec UEFI, Secure Boot et virtualisation matérielle requis", risk_level: "Essential", reg_path: r"HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard", reg_value: "EnableVirtualizationBasedSecurity", secure_value: 1, fix_type: "Manual", fix_target: "" },
        CheckDef { id: "ScriptHost", category: "Scripting", name: "Windows Script Host", description: "Désactive WSH pour prévenir les attaques par script", risk_level: "Recommended", reg_path: r"HKLM\SOFTWARE\Microsoft\Windows Script Host\Settings", reg_value: "Enabled", secure_value: 0, fix_type: "Registry", fix_target: "" },
        CheckDef { id: "PSLogging", category: "Auditing", name: "Journalisation PowerShell", description: "Active la journalisation des blocs de script PowerShell", risk_level: "Recommended", reg_path: r"HKLM\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging", reg_value: "EnableScriptBlockLogging", secure_value: 1, fix_type: "Registry", fix_target: "" },
        CheckDef { id: "Spooler", category: "Services", name: "Service Spouleur d'impression", description: "Désactive le spouleur d'impression pour atténuer PrintNightmare", risk_level: "Recommended", reg_path: "", reg_value: "", secure_value: 0, fix_type: "Service", fix_target: "Spooler" },
        CheckDef { id: "IPv6", category: "Network", name: "Protocole IPv6", description: "Désactive IPv6 si inutile pour réduire la surface d'attaque", risk_level: "Recommended", reg_path: r"HKLM\SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters", reg_value: "DisabledComponents", secure_value: 255, fix_type: "Registry", fix_target: "" },
        CheckDef { id: "AdminShares", category: "File Sharing", name: "Partages administratifs", description: "Désactive les partages administratifs par défaut (C$, ADMIN$)", risk_level: "Recommended", reg_path: r"HKLM\SYSTEM\CurrentControlSet\Services\LanmanServer\Parameters", reg_value: "AutoShareWks", secure_value: 0, fix_type: "Registry", fix_target: "" },
        CheckDef { id: "GuestAccount", category: "Accounts", name: "Compte Invité", description: "Vérifie que le compte Invité est désactivé", risk_level: "Recommended", reg_path: "", reg_value: "", secure_value: 0, fix_type: "Command", fix_target: states::GUEST_DISABLE_COMMAND },
        CheckDef { id: "AccountLockout", category: "Accounts", name: "Stratégie de verrouillage de compte", description: "Définit un seuil de verrouillage contre les attaques par force brute", risk_level: "Recommended", reg_path: "", reg_value: "", secure_value: 0, fix_type: "Command", fix_target: "net.exe accounts /lockoutthreshold:5" },
        CheckDef { id: "AdvancedAudit", category: "Auditing", name: "Stratégie d'audit avancée", description: "Active la stratégie d'audit avancée pour la surveillance de sécurité", risk_level: "Recommended", reg_path: r"HKLM\SYSTEM\CurrentControlSet\Control\Lsa", reg_value: "SCENoApplyLegacyAuditPolicy", secure_value: 1, fix_type: "Registry", fix_target: "" },
    ]
}

fn is_auto_fixable(def: &CheckDef) -> bool {
    def.fix_type != "Manual"
}

fn check_status(def: &CheckDef) -> CheckState {
    match def.id {
        "GuestAccount" => return guest_account_state(),
        "AccountLockout" => return account_lockout_state(),
        "CredentialGuard" => return credential_guard_check_state(),
        "Spooler" => return spooler_service_state(),
        "WDigest" => return wdigest_check_state(def.reg_path, def.reg_value),
        _ => {}
    }
    if def.fix_type == "Registry" && !def.reg_path.is_empty() {
        let secure = registry::get_dword(def.reg_path, def.reg_value)
            .map(|v| v as i32 == def.secure_value)
            .unwrap_or(false);
        return if secure { CheckState::Secure } else { CheckState::Insecure };
    }
    CheckState::Insecure
}

fn get_checks_internal() -> Vec<(HardeningCheck, CheckDef)> {
    let defs = build_checks();
    defs.into_iter()
        .map(|def| {
            let state = check_status(&def);
            let check = HardeningCheck {
                id: def.id.to_string(),
                category: def.category.to_string(),
                name: def.name.to_string(),
                description: def.description.to_string(),
                risk_level: def.risk_level.to_string(),
                is_secure: state == CheckState::Secure,
                is_unknown: state == CheckState::Unknown,
                is_fixable: is_auto_fixable(&def),
            };
            (check, def)
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub async fn get_hardening_checks() -> Vec<HardeningCheck> {
    crate::utils::blocking::read_value(get_hardening_checks_sync).await
}

pub fn get_hardening_checks_sync() -> Vec<HardeningCheck> {
    crate::perf_timed!("get_hardening_checks", {
        get_checks_internal().into_iter().map(|(c, _)| c).collect()
    })
}

#[tauri::command]
#[specta::specta]
pub async fn apply_hardening_fix(id: String) -> Result<(), crate::utils::errors::CommandError> {
    crate::utils::blocking::write_result(move || apply_hardening_fix_sync(id)).await
}

pub fn apply_hardening_fix_sync(id: String) -> Result<(), crate::utils::errors::CommandError> {
    use crate::utils::errors::CommandError;
    admin::require_admin("Appliquer un durcissement")?;
    let checks = get_checks_internal();
    let (_, def) = checks
        .iter()
        .find(|(c, _)| c.id == id)
        .ok_or_else(|| CommandError::NotFound(format!("Check hardening : {}", id)))?;
    if !is_auto_fixable(def) {
        return Err(CommandError::InvalidInput(format!(
            "{} ne se corrige pas automatiquement : configuration manuelle requise",
            def.name
        )));
    }
    apply_fix(def)?;
    Ok(())
}

fn apply_fix(def: &CheckDef) -> Result<(), String> {
    match def.fix_type {
        "Manual" => Err(format!(
            "{} ne se corrige pas automatiquement : configuration manuelle requise",
            def.name
        )),
        "Registry" => {
            registry::ensure_key(def.reg_path)?;
            registry::set_dword(def.reg_path, def.reg_value, def.secure_value as u32)
        }
        "Service" => {
            let stop = process::run("sc.exe", &format!("stop {}", def.fix_target), constants::TIMEOUT_LONG);
            if !stop.success {
                crate::utils::logger::warn(
                    "hardening",
                    &format!(
                        "apply_fix Service {}: 'sc.exe stop' a echoue (exit {}): {} - le service sera tout de meme desactive (Start=4) au prochain demarrage",
                        def.fix_target, stop.exit_code, stop.stderr
                    ),
                );
            }
            let reg_path = format!(r"{}\{}", constants::REG_SERVICES, def.fix_target);
            registry::set_dword(&reg_path, "Start", 4)
        }
        "Command" => {
            let parts: Vec<&str> = def.fix_target.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let out = process::run(parts[0], parts[1], constants::TIMEOUT_MEDIUM);
                if !out.success {
                    crate::utils::logger::warn(
                        "hardening",
                        &format!("apply_fix Command '{}': echec (exit {}): {}", def.fix_target, out.exit_code, out.stderr),
                    );
                    return Err(format!(
                        "Echec de '{}' (code {})",
                        def.fix_target, out.exit_code
                    ));
                }
            }
            Ok(())
        }
        other => Err(format!("Type de correctif inconnu : {}", other)),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn apply_hardening_essential(options: Option<crate::utils::batch::BatchOptions>) -> BatchResult {
    crate::utils::blocking::write_value(move || apply_hardening_essential_sync(options)).await
}

pub fn apply_hardening_essential_sync(options: Option<crate::utils::batch::BatchOptions>) -> BatchResult {
    crate::perf_timed!("apply_hardening_essential", {
        let opts = options.unwrap_or_default();
        let dry = opts.dry_run;
        let mut result = if dry {
            BatchResult::new_dry_run()
        } else {
            BatchResult::new()
        };
        if !dry {
            if let Err(e) = admin::require_admin("Durcissement essentiel") {
                result.record("admin", Err::<(), String>(e.to_string()));
                return result;
            }
        }
        for (check, def) in get_checks_internal() {
            if def.risk_level == "Essential" && !check.is_secure && !check.is_unknown && check.is_fixable {
                if dry {
                    result.record_preview(def.name);
                } else {
                    result.record(def.id, apply_fix(&def));
                }
            }
        }
        result
    })
}

#[tauri::command]
#[specta::specta]
pub async fn apply_hardening_all(options: Option<crate::utils::batch::BatchOptions>) -> BatchResult {
    crate::utils::blocking::write_value(move || apply_hardening_all_sync(options)).await
}

pub fn apply_hardening_all_sync(options: Option<crate::utils::batch::BatchOptions>) -> BatchResult {
    crate::perf_timed!("apply_hardening_all", {
        let opts = options.unwrap_or_default();
        let dry = opts.dry_run;
        let mut result = if dry {
            BatchResult::new_dry_run()
        } else {
            BatchResult::new()
        };
        if !dry {
            if let Err(e) = admin::require_admin("Durcissement complet") {
                result.record("admin", Err::<(), String>(e.to_string()));
                return result;
            }
        }
        for (check, def) in get_checks_internal() {
            if !check.is_secure && !check.is_unknown && check.is_fixable {
                if dry {
                    result.record_preview(def.name);
                } else {
                    result.record(def.id, apply_fix(&def));
                }
            }
        }
        result
    })
}

pub(crate) fn ratio_score(secure: usize, total: usize) -> u32 {
    if total == 0 {
        return 0;
    }
    ((secure as f64 / total as f64) * 100.0).round() as u32
}

pub fn get_hardening_score_sync() -> u32 {
    let checks = get_checks_internal();
    let states: Vec<CheckState> = checks
        .iter()
        .map(|(c, _)| match (c.is_secure, c.is_unknown) {
            (true, _) => CheckState::Secure,
            (_, true) => CheckState::Unknown,
            _ => CheckState::Insecure,
        })
        .collect();
    score_from_states(&states)
}

#[cfg(test)]
mod tests;
