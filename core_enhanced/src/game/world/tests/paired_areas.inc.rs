fn paired_areas_world_fixture() -> (GameCore, Uuid, Uuid, Position) {
    let native = crate::authoring::node_graph::compile(
        &crate::authoring::node_graph::tests::paired_areas_fixture().to_string(),
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
        "pair_initial_charge",
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
fn paired_areas_world_focus_partial_cancel_atomic_pair_pause_and_projection() {
    use crate::game::behavior::{
        LiveBattleActiveSkillTargetingKind, LiveBattleTacticalFocusPurposeDto,
    };
    let (mut c, p, e, tile) = paired_areas_world_fixture();
    let skill: SkillId = "r08_strike".into();
    let active = c.state.active_battle.as_ref().unwrap();
    let u = active.live_deployment.as_ref().unwrap().deployed_units[&e].unit_instance_id;
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
        LiveBattleTacticalFocusPurposeDto::PlayerTilePairSkill
    );
    let before = battle_advance_atomic_snapshot(&c);
    for target in [
        None,
        Some(SkillCastTarget::Tile { position: tile }),
        Some(SkillCastTarget::TilePair {
            first: tile,
            second: Position::new(99, 99),
        }),
    ] {
        assert!(execute_current_battle_mutation(
            &mut c,
            p,
            PlayerBehavior::ActivateSkill {
                employee_uuid: e,
                skill_id: skill.clone(),
                target
            }
        )
        .is_err());
        assert_eq!(battle_advance_atomic_snapshot(&c), before);
    }
    execute_current_battle_mutation(&mut c, p, PlayerBehavior::CancelSkillTargeting).unwrap();
    assert!(c
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .paired_areas_snapshot()
        .unwrap()
        .areas
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
            skill_id: skill,
            target: Some(SkillCastTarget::TilePair {
                first: tile,
                second: tile,
            }),
        },
    )
    .unwrap();
    c.advance_active_battle_for_server_tick(1).unwrap();
    let areas = c
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .paired_areas_snapshot()
        .unwrap();
    assert_eq!(areas.areas.len(), 1);
    assert_eq!(areas.areas[0].first_center, tile);
    assert_eq!(areas.areas[0].second_center, tile);
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
    assert_eq!(battle_update.checkpoint.paired_areas, areas);
    let row = battle_update
        .checkpoint
        .units
        .iter()
        .find(|row| row.unit_instance_id == u)
        .unwrap();
    assert_eq!(
        row.range_previews
            .as_ref()
            .unwrap()
            .active_skill
            .targeting_kind,
        Some(LiveBattleActiveSkillTargetingKind::ManualTilePair)
    );
    execute_current_battle_mutation(&mut c, p, PlayerBehavior::ResumeBattle).unwrap();
    c.advance_active_battle_for_server_tick(3000).unwrap();
    assert!(c
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .paired_areas_snapshot()
        .unwrap()
        .areas
        .is_empty());
}
