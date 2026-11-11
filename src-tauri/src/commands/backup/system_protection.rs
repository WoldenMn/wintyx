//! Etat de la Protection du systeme lu dans le registre.
//!
//! La valeur `DisableSR` qui fait foi est celle de la strategie
//! (`HKLM\SOFTWARE\Policies\Microsoft\Windows NT\SystemRestore`), absente par defaut ;
//! l'ancien emplacement sous `CurrentVersion` est lu aussi. L'etat par volume exige
//! les droits administrateur (WMI), il n'est pas lu ici.

use crate::utils::errors::CommandError;
use crate::utils::registry;

const POLICY_KEY: &str = r"HKLM\SOFTWARE\Policies\Microsoft\Windows NT\SystemRestore";
const LEGACY_KEY: &str = r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyState {
    NotDisabled,
    DisabledByPolicy,
}

impl PolicyState {
    pub fn label(self) -> &'static str {
        match self {
            PolicyState::NotDisabled => "Non désactivée par stratégie (état par volume non lu)",
            PolicyState::DisabledByPolicy => "Désactivée par stratégie",
        }
    }
}

pub(super) fn policy_state() -> Result<PolicyState, CommandError> {
    enabled_from_reads(
        registry::read_dword(POLICY_KEY, "DisableSR"),
        registry::read_dword(LEGACY_KEY, "DisableSR"),
    )
    .map(|enabled| if enabled { PolicyState::NotDisabled } else { PolicyState::DisabledByPolicy })
}

pub(super) fn is_enabled() -> Result<bool, CommandError> {
    policy_state().map(|state| state == PolicyState::NotDisabled)
}

fn enabled_from_reads(
    policy: Result<Option<u32>, String>,
    legacy: Result<Option<u32>, String>,
) -> Result<bool, CommandError> {
    let policy = policy.map_err(CommandError::Registry)?;
    let legacy = legacy.map_err(CommandError::Registry)?;
    Ok(enabled_from(policy, legacy))
}

fn enabled_from(policy: Option<u32>, legacy: Option<u32>) -> bool {
    policy.unwrap_or(0) == 0 && legacy.unwrap_or(0) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_when_no_value_is_set() {
        assert!(enabled_from(None, None));
    }

    #[test]
    fn policy_disable_wins_over_an_absent_legacy_value() {
        assert!(!enabled_from(Some(1), None));
    }

    #[test]
    fn legacy_disable_is_still_honoured() {
        assert!(!enabled_from(None, Some(1)));
    }

    #[test]
    fn explicit_zero_means_enabled() {
        assert!(enabled_from(Some(0), Some(0)));
    }

    #[test]
    fn unreadable_policy_is_an_error_not_enabled() {
        assert!(enabled_from_reads(Err("acces refuse".into()), Ok(None)).is_err());
        assert!(enabled_from_reads(Ok(None), Err("acces refuse".into())).is_err());
    }

    #[test]
    fn readable_values_decide() {
        assert!(enabled_from_reads(Ok(None), Ok(None)).unwrap());
        assert!(!enabled_from_reads(Ok(Some(1)), Ok(None)).unwrap());
    }

    #[test]
    fn the_label_states_what_was_read_and_never_claims_the_volume_state() {
        let not_disabled = PolicyState::NotDisabled.label();
        assert!(not_disabled.contains("Non désactivée par stratégie"), "{not_disabled}");
        assert!(not_disabled.contains("par volume non lu"), "{not_disabled}");
        assert!(!not_disabled.starts_with("Activée"), "{not_disabled}");
        assert_eq!(PolicyState::DisabledByPolicy.label(), "Désactivée par stratégie");
    }

    #[cfg(windows)]
    #[test]
    fn the_command_bool_and_the_policy_state_agree_on_this_machine() {
        let state = policy_state().expect("registre lisible");
        assert_eq!(is_enabled().unwrap(), state == PolicyState::NotDisabled);
    }
}
