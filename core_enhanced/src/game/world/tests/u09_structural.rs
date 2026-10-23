//! Independent TB001: normal RON/builder publication and routed World execution.
//! Grants, initial charge and map placement are explicit preparation seams.
use super::*;
use crate::game::data::relic_data::RelicDatabase;
use crate::game::enums::Side;
use crate::game::relic::structural::{authoring::*, EffectiveEffectKey};

mod flow_cases;
mod publication_cases;
mod world_state_cases;

const SKILLS: &str = r#"SkillDatabase(skills: [SkillDef(
 id: "tb_strike", name: "TB original", kind: Targeted, focus_time_ms: 0,
 use_lifetime: Action,
 cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: true),
 steps: [SkillStepDef(id: "hit", delay_ms: 0, target: CastTarget, delivery: Instant,
 effects: [(id: "original", tags: ["original"], op: Damage(amount: 23, damage_type: True))])]
)])"#;
const RELICS: &str = r#"(relics: [(
 id: "tb_relic", uuid: "00000000-0000-0000-0000-00000000b001", name: "TB relic",
 description: "independent authored 30 plus 75", rarity: "probe", allow_duplicates: false,
 effects: [], dismantle_equipment_dust: 1,
 rules: [
  (id: "original_add", selector: (provider: (fragment_id: Some("tb_fragment")), effect: (all_tags: ["original"])), apply: Edit(property: DamageAmount, operation: Add(Hp(7)))),
  (id: "grant", selector: (provider: (fragment_id: Some("tb_fragment"))), apply: Grant(InlineEffects(action_id: "hit", effects: [(id: "bonus", tags: ["bonus"], op: Damage(amount: 40, damage_type: True))]))),
  (id: "bonus_add", selector: (effect: (all_tags: ["bonus"])), apply: Edit(property: DamageAmount, operation: Add(Hp(10)))),
  (id: "bonus_scale", selector: (effect: (all_tags: ["bonus"])), apply: Edit(property: DamageAmount, operation: Scale(increment_percent: 50))),
 ]
)])"#;

fn setup() -> (GameCore, Uuid, Uuid) {
    setup_with_definitions(ron::de::from_str(RELICS).unwrap(), false)
}

fn setup_with_definitions(relics: RelicDatabase, alternate: bool) -> (GameCore, Uuid, Uuid) {
    setup_with_skill_input(relics, alternate, SKILLS)
}

fn setup_with_skill_input(
    relics: RelicDatabase,
    alternate: bool,
    skills: &str,
) -> (GameCore, Uuid, Uuid) {
    let skills = SkillDatabase::from_ron("tb001/skills.ron", skills).unwrap();
    let mut fragment = active_skill_fragment("tb_fragment", 0xb002, "tb_strike");
    if alternate {
        let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &mut fragment.effect else {
            unreachable!()
        };
        let mut form = forms["base"].clone();
        form.bloom_recipe = Some(
            crate::game::data::skill_fragment_data::SkillFragmentRecipe {
                enkephalin: 1,
                materials: vec![],
            },
        );
        forms.insert("alternate".into(), form);
    }
    let base = GameCore::new(
        game_data_with_pve_equipment_and_active_skill_fragments(
            vec![],
            skills.skills.clone(),
            vec![fragment],
        ),
        0xb001,
    );
    let mut enemies = base.game_data.abnormality_data.items.clone();
    for enemy in &mut enemies {
        enemy.max_health = 2000;
        enemy.attack = 0;
    }
    let data = GameDataBuilder::from_validated(&base.game_data)
        .with_abnormalities(enemies)
        .with_relics(relics)
        .build_with_runtime_safety(crate::runtime_safety::RuntimeSafetySettings::embedded_shared())
        .expect("ordinary complete builder publication");
    let mut core = GameCore::new(Arc::new(data), 0xb001);
    let player = Uuid::from_u128(0xb001);
    start_new_game_with_default_starters(&mut core, player);
    let employee = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let serum = consumable_meta(
        0xb003,
        "tb_charge",
        ConsumableTier::Critical,
        ConsumableEffect::InitialSkillCharge { percent: 100 },
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee)
        .unwrap()
        .apply_consumable_modifier(Uuid::from_u128(0xb004), &serum);
    let fragment_copy = core
        .apply_grant_effects(&[RewardEffect::GrantSkillFragment {
            fragment_id: "tb_fragment".into(),
        }])
        .unwrap()
        .skill_fragment_diffs[0]
        .copy_uuid;
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: employee,
            copy_uuid: fragment_copy,
        },
    )
    .unwrap();
    core.execute_admin_command(super::super::AdminCommand::AdminGrantRelic {
        definition_id: "tb_relic".into(),
        count: 1,
    })
    .unwrap();
    let copy = core.state.relics.copies().next().unwrap().copy_uuid;
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            employee_uuid: employee,
            copy_uuid: copy,
            slot: 0,
        },
    )
    .unwrap();
    (core, player, employee)
}

fn query(
    core: &mut GameCore,
    player: Uuid,
    employee: Uuid,
) -> crate::game::relic::structural::dto::RelicEffectivePlanDto {
    match core
        .execute(
            player,
            PlayerBehavior::RequestRelicEffectivePlan {
                employee_uuid: employee,
            },
        )
        .unwrap()
    {
        BehaviorResult::RelicEffectivePlan { result: Some(plan) } => plan,
        other => panic!("unexpected effective query: {other:?}"),
    }
}

#[test]
fn tb001_normal_publication_query_and_first_world_cast() {
    let (mut core, player, employee) = setup();
    let before = non_battle_state_without_profile_fingerprint(&core);
    let plan = query(&mut core, player, employee);
    assert_eq!(plan.execution_status, "runtime_supported_prospective_plan");
    assert!(plan.capabilities.iter().all(|c| c.runtime_connected));
    assert_eq!(plan.effects.len(), 2);
    for effect in &plan.effects {
        let value = effect
            .properties
            .iter()
            .find(|p| p.property == RelicProperty::DamageAmount)
            .unwrap()
            .final_value;
        let expected = if matches!(effect.key, EffectiveEffectKey::Original { .. }) {
            30
        } else {
            75
        };
        assert_eq!(value, Some(TypedRelicValue::Hp(expected)));
    }
    assert_eq!(
        serde_json::to_value(&query(&mut core, player, employee)).unwrap(),
        serde_json::to_value(&plan).unwrap()
    );
    assert_eq!(non_battle_state_without_profile_fingerprint(&core), before);
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        MapNodePayload::Encounter {
            encounter_id: Some("defense_encounter".into()),
        },
    );
    let preview = core
        .execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    let BehaviorResult::NodePreview {
        combat_preview: Some(preview),
        ..
    } = preview
    else {
        panic!("expected combat preview");
    };
    let position = preview
        .deployment_zones
        .iter()
        .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
        .and_then(|zone| zone.cells.first().copied())
        .expect("ground deployment");
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: employee,
            position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let hp = |core: &GameCore| -> u32 {
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .units
            .values()
            .filter(|u| u.owner == Side::Opponent)
            .map(|u| u.stats.current_health)
            .sum()
    };
    let before_hp = hp(&core);
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: employee,
            skill_id: "tb_strike".into(),
            target: None,
        },
    )
    .expect("first accepted manual cast through World");
    core.advance_active_battle_for_server_tick(1).unwrap();
    // Authored independent arithmetic: original 23+7=30, generated
    // (40+10)*150/100=75. True damage avoids mitigation ambiguity.
    assert_eq!(before_hp - hp(&core), 105);
    println!("TB001 NORMAL publication=builder query_original=30 query_generated=75 first_world_cast_hp_lost=105");
}
