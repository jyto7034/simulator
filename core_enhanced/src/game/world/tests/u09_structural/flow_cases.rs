//! TC001 normal builder + World accepted-cast connection. Grants, charge, map
//! selection, HP/mitigation and optional legacy shield are preparation seams.
use super::*;
use crate::game::battle::{
    core::relic_structural_tests::flow_cases as native_probe, event_log::BattleLogEvent,
    ids::UnitInstanceId,
};

mod cancellation_cases;
mod footprint_cases;
mod isolation_cases;
mod lifetime_cases;
mod personal_cases;
mod resource_cases;
mod selected_cases;
mod support_cases;

fn skill_text(bite: u32) -> String {
    format!(
        r#"SkillDatabase(skills: [SkillDef(
        id: "tc_bite", name: "TC independent bite", kind: Targeted,
        focus_time_ms: 0, use_lifetime: Action,
        cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: true),
        flow: Some(Action(id: "original_bite", step_id: "bite", wait: EffectsSettled)),
        steps: [SkillStepDef(id: "bite", delay_ms: 0, target: CastTarget, delivery: Instant,
            effects: [(id: "bite_damage", op: Damage(amount: {bite}, damage_type: Physical)),
                (id: "sibling_damage", op: Damage(amount: 13, damage_type: True))])]
    )])"#
    )
}

fn relic_text(condition: Option<&str>) -> String {
    let action = r#"Action(id: "heal_action", step_id: "heal_once", wait: EffectsSettled)"#;
    let flow = condition.map_or_else(|| action.to_string(), |condition| format!(
        r#"Branch(id: "hp_check", condition: {condition}, then_node: {action}, else_node: Wait(id: "no_heal", duration_ms: 0))"#));
    format!(
        r#"(relics: [(
        id: "tc_relic", uuid: "00000000-0000-0000-0000-00000000c001",
        name: "TC finite child", description: "one authored self heal", rarity: "probe",
        allow_duplicates: false, effects: [], dismantle_equipment_dust: 1,
        rules: [(id: "heal_child", selector: (provider: (fragment_id: Some("tc_fragment"))),
            apply: Grant(FiniteFlow(anchor_action_id: "bite",
                steps: [(id: "heal_once", target: SelfUnit, delivery: Instant,
                    effects: [(id: "heal", op: Heal(amount: 37))])], flow: {flow})))],
    )])"#
    )
}

const EFFECT_GATE: &str = r#"EffectCompare(node_id: "bite", effect_id: "bite_damage", metric: HpLost, comparison: Greater, value: Hp(0))"#;
const ACTION_GATE: &str =
    r#"Compare(node_id: "bite", metric: HpLost, comparison: Greater, value: Hp(0))"#;

fn prepare(
    skills: &str,
    relics: &str,
) -> (GameCore, Uuid, Uuid, UnitInstanceId, Vec<UnitInstanceId>) {
    prepare_with_options(skills, relics, None, None)
}

fn prepare_with_options(
    skills: &str,
    relics: &str,
    resources: Option<crate::game::resource::ResourceCatalog>,
    enemy_count: Option<u32>,
) -> (GameCore, Uuid, Uuid, UnitInstanceId, Vec<UnitInstanceId>) {
    prepare_with_generated_count(skills, relics, resources, enemy_count, 1)
}

fn prepare_with_generated_count(
    skills: &str,
    relics: &str,
    resources: Option<crate::game::resource::ResourceCatalog>,
    enemy_count: Option<u32>,
    generated_count: usize,
) -> (GameCore, Uuid, Uuid, UnitInstanceId, Vec<UnitInstanceId>) {
    prepare_with_setup(
        skills,
        relics,
        resources,
        enemy_count,
        generated_count,
        |_, _, _| {},
    )
}

fn prepare_with_setup(
    skills: &str,
    relics: &str,
    resources: Option<crate::game::resource::ResourceCatalog>,
    enemy_count: Option<u32>,
    generated_count: usize,
    extra_prepare: impl FnOnce(&mut GameCore, Uuid, Uuid),
) -> (GameCore, Uuid, Uuid, UnitInstanceId, Vec<UnitInstanceId>) {
    let control_buffs = skills.contains("rv_control_");
    let skills = SkillDatabase::from_ron("tc001/independent_skills.ron", skills).unwrap();
    let fragment = active_skill_fragment("tc_fragment", 0xc002, "tc_bite");
    let mut fragments = vec![fragment];
    if skills
        .skills
        .iter()
        .any(|skill| skill.id.as_str() == "rv_aux")
    {
        fragments.push(active_skill_fragment("rv_aux_fragment", 0xc020, "rv_aux"));
    }
    // The shared PvE constructor publishes its seed immediately. Assemble
    // control-only buff references with their catalog in the final builder.
    let mut seed_skills = skills.skills.clone();
    if control_buffs {
        for skill in &mut seed_skills {
            for step in &mut skill.steps {
                step.effects.retain(|effect| {
                    !matches!(
                        effect.op,
                        crate::game::ability::SkillEffectDef::ApplyBuff { .. }
                    )
                });
            }
        }
    }
    let base = GameCore::new(
        game_data_with_pve_equipment_and_active_skill_fragments(vec![], seed_skills, fragments),
        0xc001,
    );
    let mut enemies = base.game_data.abnormality_data.items.clone();
    for enemy in &mut enemies {
        enemy.max_health = 2000;
        enemy.attack = 0;
        enemy.defense = 0;
        enemy.magic_resist = 0;
    }
    let mut builder = GameDataBuilder::from_validated(&base.game_data)
        .with_skills(skills)
        .with_abnormalities(enemies)
        .with_relics(ron::de::from_str::<RelicDatabase>(relics).unwrap());
    if control_buffs {
        builder = builder.with_buffs(ron::de::from_str(r#"BuffDatabase(buffs: [
            (name: "rv_control_silence", kind: Silence, tick_interval_ms: 0, max_stacks: 1, reapply_policy: RefreshDuration),
            (name: "rv_control_stun", kind: Stun, tick_interval_ms: 0, max_stacks: 1, reapply_policy: RefreshDuration)])"#).unwrap());
    }
    if let Some(resources) = resources {
        builder = builder.with_resources(resources);
    }
    if let Some(enemy_count) = enemy_count {
        let mut pve = (*base.game_data.pve_data).clone();
        let encounter = pve
            .encounters
            .iter_mut()
            .find(|entry| entry.id == "defense_encounter")
            .unwrap();
        let PveWaveSource::Manual(entries) = &mut encounter.waves[0].source else {
            panic!("manual fixture wave")
        };
        let PveWaveEnemyData::Abnormality { count, .. } = &mut entries[0] else {
            panic!("abnormality fixture")
        };
        *count = enemy_count;
        builder = builder.with_pve(pve);
    }
    let data = builder
        .build_with_runtime_safety(crate::runtime_safety::RuntimeSafetySettings::embedded_shared())
        .expect("normal complete builder publishes finite child");
    let mut core = GameCore::new(Arc::new(data), 0xc001);
    let player = Uuid::from_u128(0xc001);
    start_new_game_with_default_starters(&mut core, player);
    let employee = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let serum = consumable_meta(
        0xc003,
        "tc_charge",
        ConsumableTier::Critical,
        ConsumableEffect::InitialSkillCharge { percent: 100 },
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee)
        .unwrap()
        .apply_consumable_modifier(Uuid::from_u128(0xc004), &serum);
    let fragment_copy = core
        .apply_grant_effects(&[RewardEffect::GrantSkillFragment {
            fragment_id: "tc_fragment".into(),
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
    core.execute_admin_command(super::super::super::AdminCommand::AdminGrantRelic {
        definition_id: "tc_relic".into(),
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
    let plan = query(&mut core, player, employee);
    assert_eq!(
        plan.effects
            .iter()
            .filter(|effect| matches!(effect.key, EffectiveEffectKey::Generated { .. }))
            .count(),
        generated_count
    );
    extra_prepare(&mut core, player, employee);
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
        panic!("combat preview")
    };
    let position = preview
        .deployment_zones
        .iter()
        .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
        .and_then(|zone| zone.cells.first().copied())
        .unwrap();
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
    let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
    let actor = battle
        .units
        .values()
        .find(|unit| unit.source_owned_uuid == employee)
        .unwrap()
        .instance_id;
    let targets = battle
        .units
        .values()
        .filter(|unit| unit.owner == Side::Opponent)
        .map(|unit| unit.instance_id)
        .collect::<Vec<_>>();
    assert!(!targets.is_empty());
    let unit = battle.units.get_mut(&actor).unwrap();
    unit.stats.max_health = 1000;
    unit.stats.current_health = 500;
    unit.stats.attack = 0;
    for target in &targets {
        let unit = battle.units.get_mut(target).unwrap();
        unit.stats.defense = 0;
        unit.stats.magic_resist = 0;
    }
    (core, player, employee, actor, targets)
}

fn cast(core: &mut GameCore, player: Uuid, employee: Uuid) {
    execute_current_battle_mutation(
        core,
        player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: employee,
            skill_id: "tc_bite".into(),
            target: None,
        },
    )
    .expect("normal World accepted cast");
    core.advance_active_battle_for_server_tick(1).unwrap();
}

fn hp(core: &GameCore, units: &[UnitInstanceId]) -> u32 {
    let battle = &core.state.active_battle.as_ref().unwrap().battle;
    units
        .iter()
        .map(|id| battle.units[id].stats.current_health)
        .sum()
}

fn step_count(core: &GameCore) -> usize {
    core.state.active_battle.as_ref().unwrap().battle.event_log.entries.iter()
        .filter(|entry| matches!(&entry.event, BattleLogEvent::AbilityStepTriggered { skill_id, .. } if skill_id.as_str() == "tc_bite"))
        .count()
}

#[test]
fn tc001_finite_child_settles_once_after_normal_world_cast() {
    let (mut core, player, employee, actor, targets) = prepare(&skill_text(100), &relic_text(None));
    let before = hp(&core, &targets);
    cast(&mut core, player, employee);
    assert_eq!(before - hp(&core, &targets), 113);
    assert_eq!(hp(&core, &[actor]), 537);
    assert_eq!(step_count(&core), 2);
    assert_eq!(
        native_probe::active_cast_count(&core.state.active_battle.as_ref().unwrap().battle),
        0
    );
    core.advance_active_battle_for_server_tick(2).unwrap();
    assert_eq!(hp(&core, &[actor]), 537);
    assert_eq!(step_count(&core), 2);
    println!("TC001 finite normal_builder/world_cast=true damage=113 self_heal=37 steps=2 active_casts=0 no_recursive_child=true");
}

#[test]
fn tc001_effect_gate_excludes_sibling_loss_and_legacy_compare_keeps_aggregate() {
    for (bite, condition, expected_hp, steps) in [
        (0, EFFECT_GATE, 500, 1),
        (100, EFFECT_GATE, 537, 2),
        (0, ACTION_GATE, 537, 2),
    ] {
        let (mut core, player, employee, actor, targets) =
            prepare(&skill_text(bite), &relic_text(Some(condition)));
        let before = hp(&core, &targets);
        cast(&mut core, player, employee);
        assert_eq!(before - hp(&core, &targets), bite + 13);
        assert_eq!(hp(&core, &[actor]), expected_hp);
        assert_eq!(step_count(&core), steps);
        println!("TC001 gate bite={bite} sibling=13 effect_gate={} caster_hp={expected_hp} steps={steps}", condition == EFFECT_GATE);
    }
}

#[test]
fn tc001_effect_gate_uses_actual_hp_loss_after_legacy_shield_absorption() {
    for (shield, expected_loss, expected_hp, steps) in [(100, 13, 500, 1), (99, 14, 537, 2)] {
        let (mut core, player, employee, actor, targets) =
            prepare(&skill_text(100), &relic_text(Some(EFFECT_GATE)));
        let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
        for target in &targets {
            native_probe::legacy_shield(battle, *target, shield);
        }
        let before = hp(&core, &targets);
        cast(&mut core, player, employee);
        assert_eq!(before - hp(&core, &targets), expected_loss);
        assert_eq!(hp(&core, &[actor]), expected_hp);
        assert_eq!(step_count(&core), steps);
        println!("TC001 shield={shield} bite_hp_loss={} sibling_hp_loss=13 caster_hp={expected_hp} steps={steps}", 100-shield);
    }
}

#[test]
fn tc001_predicates_boolean_effect_gate_survives_structural_namespace_lowering() {
    let condition = format!("Logic(All([{EFFECT_GATE}, Logic(Not(Always(false)))]))");
    for (bite, expected_hp, steps) in [(0, 500, 1), (100, 537, 2)] {
        let (mut core, player, employee, actor, targets) =
            prepare(&skill_text(bite), &relic_text(Some(&condition)));
        let before = hp(&core, &targets);
        cast(&mut core, player, employee);
        assert_eq!(before - hp(&core, &targets), bite + 13);
        assert_eq!(hp(&core, &[actor]), expected_hp);
        assert_eq!(step_count(&core), steps);
    }
}
