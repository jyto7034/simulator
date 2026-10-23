fn fusion_placement_world_fixture() -> (GameCore, Uuid, Uuid, Position) {
    let native = crate::authoring::node_graph::compile(
        &crate::authoring::node_graph::tests::fusion_placement_fixture(false).to_string(),
        "r08_strike".into(),
    )
    .unwrap()
    .load_native()
    .unwrap();
    let fragments: SkillFragmentDatabase = ron::de::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/r09/fragments.ron"
    )))
    .unwrap();
    let fragment = fragments.fragments[0].clone();
    let data = game_data_with_pve_equipment_and_active_skill_fragments(
        vec![],
        native.skills,
        vec![fragment.clone()],
    );
    let mut c = GameCore::new(data, 0xabc99);
    let p = Uuid::from_u128(0xabc99);
    start_new_game_with_default_starters(&mut c, p);
    let employee = c.roster().unwrap().combat_eligible_employee_ids()[0];
    let serum = consumable_meta(
        0xabc98,
        "fusion_initial_charge",
        ConsumableTier::Critical,
        ConsumableEffect::InitialSkillCharge { percent: 100 },
    );
    c.roster_mut()
        .unwrap()
        .get_mut(&employee)
        .unwrap()
        .apply_consumable_modifier(Uuid::from_u128(0xabc97), &serum);
    let copy = c
        .apply_grant_effects(&[RewardEffect::GrantSkillFragment {
            fragment_id: fragment.id,
        }])
        .unwrap()
        .skill_fragment_diffs[0]
        .copy_uuid;
    c.execute(
        p,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: employee,
            copy_uuid: copy,
        },
    )
    .unwrap();
    let node = force_map_combat_node(
        &mut c,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );
    let preview = c
        .execute(p, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    let position = first_ground_deployment_cell(&preview);
    c.execute(p, PlayerBehavior::ConfirmEnterNode).unwrap();
    execute_current_battle_mutation(
        &mut c,
        p,
        PlayerBehavior::DeployUnit {
            employee_uuid: employee,
            position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    (c, p, employee, position)
}
#[test]
fn fusion_placement_world_tile_focus_cancel_pause_and_public_mobile_projection() {
    use crate::game::behavior::{
        LiveBattleActiveSkillTargetingKind, LiveBattleTacticalFocusPurposeDto,
    };
    let (mut c, p, e, occupied) = fusion_placement_world_fixture();
    let skill: SkillId = "r08_strike".into();
    let active = c.state.active_battle.as_ref().unwrap();
    let u = active.live_deployment.as_ref().unwrap().deployed_units[&e].unit_instance_id;
    let tile = *active
        .battle
        .scenario()
        .battlefield
        .object_placement
        .as_ref()
        .unwrap()
        .ground
        .iter()
        .find(|tile| {
            active
                .battle
                .validate_manual_skill_activation(
                    active.execution.last_event_time_ms(),
                    u,
                    &skill,
                    Some(SkillCastTarget::Tile { position: **tile }),
                )
                .is_ok()
        })
        .expect("fixture has a free eligible ground tile");
    execute_current_battle_mutation(
        &mut c,
        p,
        PlayerBehavior::BeginSkillTargeting {
            employee_uuid: e,
            skill_id: skill.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        c.state
            .active_battle
            .as_ref()
            .unwrap()
            .tactical_focus_dto()
            .unwrap()
            .purpose,
        LiveBattleTacticalFocusPurposeDto::PlayerTileSkill
    );
    let before = battle_advance_atomic_snapshot(&c);
    assert!(execute_current_battle_mutation(
        &mut c,
        p,
        PlayerBehavior::ActivateSkill {
            employee_uuid: e,
            skill_id: skill.clone(),
            target: Some(SkillCastTarget::Tile { position: occupied })
        }
    )
    .is_err());
    assert_eq!(battle_advance_atomic_snapshot(&c), before);
    execute_current_battle_mutation(&mut c, p, PlayerBehavior::CancelSkillTargeting).unwrap();
    assert!(c
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .mobile_effects_snapshot()
        .unwrap()
        .effects
        .is_empty());
    execute_current_battle_mutation(
        &mut c,
        p,
        PlayerBehavior::BeginSkillTargeting {
            employee_uuid: e,
            skill_id: skill.clone(),
        },
    )
    .unwrap();
    execute_current_battle_mutation(
        &mut c,
        p,
        PlayerBehavior::ActivateSkill {
            employee_uuid: e,
            skill_id: skill.clone(),
            target: Some(SkillCastTarget::Tile { position: tile }),
        },
    )
    .unwrap();
    c.advance_active_battle_for_server_tick(1).unwrap();
    let mobile = c
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .mobile_effects_snapshot()
        .unwrap();
    assert!(!mobile.effects.is_empty());
    assert_eq!(mobile.effects.iter().map(|e| e.units).sum::<u32>(), 2);
    execute_current_battle_mutation(&mut c, p, PlayerBehavior::PauseBattle).unwrap();
    let before = battle_advance_atomic_snapshot(&c);
    assert!(c
        .advance_active_battle_for_server_tick(10000)
        .unwrap()
        .is_none());
    assert_eq!(battle_advance_atomic_snapshot(&c), before);
    let BehaviorResult::BattleState { battle_update, .. } = c
        .execute(p, PlayerBehavior::RequestBattleState { since_seq: None })
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(battle_update.checkpoint.mobile_effects, mobile);
    let row = battle_update
        .checkpoint
        .units
        .iter()
        .find(|r| r.unit_instance_id == u)
        .unwrap();
    assert_eq!(
        row.range_previews
            .as_ref()
            .unwrap()
            .active_skill
            .targeting_kind,
        Some(LiveBattleActiveSkillTargetingKind::ManualTile)
    );
    execute_current_battle_mutation(&mut c, p, PlayerBehavior::ResumeBattle).unwrap();
    c.advance_active_battle_for_server_tick(1).unwrap();
    assert_eq!(
        c.state
            .active_battle
            .as_ref()
            .unwrap()
            .execution
            .last_event_time_ms(),
        2
    );
}
