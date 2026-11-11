const ERROR_SUCCESS_REBOOT_REQUIRED: i32 = 3010;

/// Sans `/deviceid` ni `/subtree`, pnputil vise exactement cette instance, sans joker ;
/// sans `/force`, il refuse de retirer un peripherique systeme critique.
pub(super) fn remove_device_args(instance_id: &str) -> String {
    format!("/remove-device {}", instance_id)
}

/// Ni `/force` ni `/uninstall` : pnputil refuse de supprimer un paquet encore utilise par un
/// peripherique present (`/uninstall` le retirerait d'abord de ces peripheriques), si bien
/// qu'un doublon mal classe ne peut pas emporter le pilote actif.
pub(super) fn delete_driver_args(inf: &str) -> String {
    format!("/delete-driver {}", inf)
}

pub(super) fn reboot_required(exit_code: i32) -> bool {
    exit_code == ERROR_SUCCESS_REBOOT_REQUIRED
}

pub(super) fn succeeded(exit_code: i32) -> bool {
    exit_code == 0 || reboot_required(exit_code)
}

#[cfg(test)]
mod tests {
    use super::super::validation::{validate_inf_name, validate_instance_id};
    use super::*;

    #[test]
    fn remove_device_args_targets_one_exact_instance_without_force() {
        let id = "STORAGE\\VOLUME\\_??_USBSTOR#Disk&Ven_X&Prod_Y#0001#{53f56307-b6bf-11d0-94f2-00a0c91efb8b}";
        assert!(validate_instance_id(id).is_ok());
        assert_eq!(remove_device_args(id), format!("/remove-device {}", id));
    }

    #[test]
    fn delete_driver_args_neither_forces_nor_uninstalls_from_devices() {
        let inf = validate_inf_name("oem24.inf").unwrap();
        let args = delete_driver_args(inf);
        assert_eq!(args, "/delete-driver oem24.inf");
        assert!(!args.contains("/force") && !args.contains("/uninstall"), "{args}");
    }

    #[test]
    fn reboot_required_counts_as_success() {
        assert!(succeeded(0));
        assert!(succeeded(3010));
        assert!(reboot_required(3010));
        assert!(!succeeded(1));
        assert!(!succeeded(-1));
    }
}
