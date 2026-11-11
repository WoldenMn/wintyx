use super::*;

#[test]
fn create_custom_profile_rejects_unknown_preset() {
    let result = create_custom_profile("nonexistent_preset".to_string(), "Test".to_string());
    assert!(matches!(result, Err(CommandError::NotFound(_))));
}

#[test]
fn create_custom_profile_rejects_empty_name() {
    let result = create_custom_profile("bureau".to_string(), "".to_string());
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

#[test]
fn update_rejects_preset_id() {
    let preset_clone = Profile {
        id: "bureau".to_string(),
        name: "Hijack".to_string(),
        description: "".to_string(),
        data_protection_ids: vec!["dp_0".to_string()],
        hardening_ids: vec![],
        performance_ids: vec![],
        is_custom: true,
        services_ids: vec![],
        bloatware_ids: vec![],
        startup_ids: vec![],
    };
    let result = update_custom_profile(preset_clone);
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

#[test]
fn update_rejects_empty_name() {
    let custom = Profile {
        id: "custom_999".to_string(),
        name: "".to_string(),
        description: "".to_string(),
        data_protection_ids: vec!["dp_0".to_string()],
        hardening_ids: vec![],
        performance_ids: vec![],
        is_custom: true,
        services_ids: vec![],
        bloatware_ids: vec![],
        startup_ids: vec![],
    };
    let result = update_custom_profile(custom);
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

#[test]
fn update_rejects_zero_items() {
    let custom = Profile {
        id: "custom_999".to_string(),
        name: "Empty".to_string(),
        description: "".to_string(),
        data_protection_ids: vec![],
        hardening_ids: vec![],
        performance_ids: vec![],
        is_custom: true,
        services_ids: vec![],
        bloatware_ids: vec![],
        startup_ids: vec![],
    };
    let result = update_custom_profile(custom);
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

#[test]
fn delete_rejects_preset_id() {
    let result = delete_custom_profile("gaming".to_string());
    assert!(matches!(result, Err(CommandError::InvalidInput(_))));
}

#[test]
fn get_profiles_includes_customs_with_flag() {
    let profiles = get_profiles().unwrap();
    assert!(profiles.len() >= 4);
    for p in &profiles {
        if !p.is_custom {
            assert!(["bureau", "discretion_max", "gaming", "performance"].contains(&p.id.as_str()));
        } else {
            assert!(p.id.starts_with("custom_"));
        }
    }
}
