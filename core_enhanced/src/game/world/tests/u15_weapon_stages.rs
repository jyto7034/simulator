#[test]
fn weapon_stages_checkpoint_restore_resolves_same_copy_and_keeps_new_copies_base() {
    use crate::game::ability::weapon_marks::WeaponStage;
    let (mut core, player, fragment, weapon) = fixture_with_weapon_stages(7, 5, true, true);
    maintenance(&mut core, player);
    let owner = core.state.roster.combat_eligible_employee_ids()[0];
    core.execute(
        player,
        PlayerBehavior::EquipItem {
            item_uuid: weapon,
            target_unit: owner,
        },
    )
    .unwrap();
    let profile = |core: &GameCore| {
        let draft = crate::game::combat_setup::player_spawns::battle_unit_draft_for_employee(
            &core.state.roster,
            &core.state.inventory,
            &core.state.skill_fragments,
            &core.game_data,
            owner,
        )
        .unwrap();
        let stage = draft
            .equipped_items
            .iter()
            .find(|b| b.persistent_copy_uuid == Some(weapon))
            .unwrap()
            .weapon_stage;
        (stage, draft.combat_profile(&core.game_data).unwrap())
    };
    assert_eq!(profile(&core).0, WeaponStage::Base);
    assert_eq!(profile(&core).1.basic_attack.interval_ms, 1000);
    core.execute(
        player,
        PlayerBehavior::AwakenSkillFragmentWithWeapon {
            copy_uuid: fragment,
            form_id: "bloom".into(),
            weapon_copy_uuid: weapon,
            materials: Some(payment()),
        },
    )
    .unwrap();
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = core_with_profile_for_continue(&core, 991);
    restored
        .continue_from_active_run_checkpoint(
            player,
            &saved.core_payload_json,
            saved.loads_used,
            saved.run_peak_certified_power,
        )
        .unwrap();
    let (stage, actual) = profile(&restored);
    assert_eq!(stage, WeaponStage::Upgraded);
    assert_eq!(actual.basic_attack.interval_ms, 250);
    assert_eq!(actual.basic_attack.windup_ms, 50);
    assert_eq!(actual.block_capacity, 0);
    assert_eq!(
        actual.deployment_affinity,
        crate::game::battle::types::DeploymentAffinity::PlatformOnly
    );
    restored
        .apply_grant_effects(&[RewardEffect::GrantEquipment {
            equipment_id: "u15_weapon".into(),
        }])
        .unwrap();
    assert!(restored
        .state
        .inventory
        .equipments
        .iter()
        .filter(|c| c.meta.id == "u15_weapon" && c.instance_uuid != weapon)
        .all(|c| c.weapon_stage == WeaponStage::Base));
    assert_eq!(profile(&restored).0, WeaponStage::Upgraded);
}
