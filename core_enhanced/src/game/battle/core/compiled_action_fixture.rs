pub(in crate::game::battle::core) mod compiled_action_fixture {
    use super::*;
    use crate::{
        authoring::node_graph::{compile, PortableGraph},
        game::{
            battle::{
                core::sim::{BattleExecutionState, BattleLiveCommand},
                damage::{BattleCommand, DamageSource, DamageType},
            },
            data::{relic_data::RelicDatabase, ValidatedGameData},
            managers::uuid_manager::UuidManager,
            relic::{
                structural::{compile_effective_plan, EffectivePlanInput, ProspectiveProvider},
                RelicInventory,
            },
        },
    };
    pub(in crate::game::battle::core) fn fixture_graph(
        relic_ids: &[&str],
        equipped: &[usize],
        doc: serde_json::Value,
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        fixture_graph_with_weapon(relic_ids,equipped,doc,|_|{})
    }
    pub(in crate::game::battle::core) fn fixture_graph_with_weapon(
        relic_ids: &[&str], equipped: &[usize], doc: serde_json::Value,
        configure_weapon: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata),
    ) -> (BattleCore,BattleExecutionState,UnitInstanceId,UnitInstanceId,UnitInstanceId) {
        let graph = compile(&doc.to_string(), "r11_aura".into()).unwrap();
        let native = PortableGraph::from_ron(&graph.to_ron().unwrap())
            .unwrap()
            .load_content()
            .unwrap();
        let mut skills = SkillDatabase::from_ron("fixture", R08_SKILLS)
            .unwrap()
            .skills;
        let input_plan=native.skills.skills.iter().flat_map(|s|&s.steps).flat_map(|s|&s.effects).any(|e|matches!(e.op,crate::game::ability::SkillEffectDef::InputPlan{..}));
        skills.extend(native.skills.skills);
        let authored = ron::ser::to_string(&SkillDatabase::new(skills)).unwrap();
        let mut fragments: crate::game::data::skill_fragment_data::SkillFragmentDatabase =
            ron::de::from_str(
                &include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/r09/fragments.ron"
                ))
                .replace("r08_strike", "r11_aura"),
            )
            .unwrap();
        if input_plan {for fragment in &mut fragments.fragments {fragment.origin=Some(crate::game::data::skill_fragment_data::SkillFragmentOrigin::Concept{concept_id:"little-red".into()});}}
        if !native.mark_weapons.is_empty() {
            assert_eq!(native.mark_weapons.len(), 1);
            assert_eq!(native.mark_weapons[0].weapon, "r11_weapon");
            for f in &mut fragments.fragments {
                if let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill { forms, .. }=&mut f.effect {
                    let mut bloom=forms["base"].clone();
                    bloom.bloom_recipe=Some(crate::game::data::skill_fragment_data::SkillFragmentRecipe { enkephalin:1,materials:vec![] });
                    forms.insert("bloom".into(),bloom);
                }
            }
        }
        let rows:Vec<_>=relic_ids.iter().enumerate().map(|(i,id)|format!(r#"(id:"{id}",uuid:"{}",name:"Modifier fixture",description:"Graph owns this declaration",rarity:"fixture",allow_duplicates:true,effects:[(id:"unused",selector:(all_tags:["unused_fixture"]),percent:1)],dismantle_equipment_dust:1)"#,Uuid::from_u128(0xd100+i as u128))).collect();
        let relics: RelicDatabase =
            ron::de::from_str(&format!("(relics:[{}])", rows.join(","))).unwrap();
        let (mut core, state, actor, ally) = r12_fragment_equipped_effect_core_with_data(
            &authored,
            |scenario| {
                scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(8000);
                scenario.groups[0].enemy_movement_plan = None;
                if input_plan {
                    scenario.tactical_plan.enemy_plan=crate::game::battle::scenario::EnemyMovementPlan::PathAlongCells{cells:(0..8).map(|x|crate::game::resources::Position::new(x,1)).collect()};
                    scenario.battlefield.walkable_paths=(0..8).map(|x|crate::game::resources::Position::new(x,1)).collect();
                }
                if !native.mark_weapons.is_empty() {
                    scenario.battlefield.walkable_paths = vec![
                        crate::game::resources::Position::new(3, 1),
                        crate::game::resources::Position::new(4, 1),
                        crate::game::resources::Position::new(6, 3),
                    ];
                }
                for spawn in &mut scenario.groups[1].spawns {
                    let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                        unreachable!()
                    };
                    profile.stats.current_health = 1000;
                    profile.stats.max_health = 1000;
                    profile.fragment_source =
                        Some(crate::game::skill_fragment::FragmentSourceSnapshot {
                            copy_uuid: Uuid::from_u128(0xd200 + spawn.instance_salt as u128),
                            definition_id: "r09_fragment".into(),
                            form_id: "base".into(),
                            skill_id: "r11_aura".into(),
                        });
                }
            },
            |w| {
                configure_weapon(w);
                if let Some(binding) = native.mark_weapons.first() {
                    w.weapon_profile.as_mut().unwrap().mark_weapon =
                        Some(binding.definition.clone());
                }
            },
            Some(fragments),
            |b| b.with_relics(relics),
        );
        let enemy = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for unit in core.units.values_mut() {
            unit.stats.attack_interval_ms = 100_000;
            unit.next_basic_attack_ms = 100_000;
            unit.stats.move_speed_units_per_ms = 0;
            unit.stats.defense = 0;
        }
        place(&mut core, actor, 2, 1);
        place(&mut core, enemy, 4, 1);
        place(&mut core, ally, 7, 3);
        if !equipped.is_empty() {
            let data: ValidatedGameData = core.game_data.clone().into();
            let employee = core.units[&actor].source_owned_uuid;
            let mut inventory = RelicInventory::default();
            let mut ids = UuidManager::new(0xd300);
            for (slot, index) in equipped.iter().enumerate() {
                let copy = inventory
                    .grant(&data.relic_data.relics[*index], &mut ids)
                    .unwrap();
                inventory
                    .equip(copy, employee, slot, &data.relic_data)
                    .unwrap();
            }
            let providers = [ProspectiveProvider::Fragment(
                core.units[&actor]
                    .fragment_source
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .clone(),
            )];
            if let Some(plan) = compile_effective_plan(EffectivePlanInput {
                data: &data,
                employee_uuid: employee,
                inventory: &inventory,
                providers: &providers,
            })
            .unwrap()
            {
                let owner = crate::game::battle::core::effective_skill::EffectiveBattleOwner::new(
                    std::collections::BTreeMap::from([(employee, plan)]),
                )
                .unwrap();
                core.set_effective_owner(owner).unwrap();
            }
            let owner = crate::game::battle::core::relic_numeric::RelicNumericOwner::capture(
                &inventory,
                core.game_data.clone(),
                core.content_identity.as_ref().unwrap().clone(),
            )
            .unwrap();
            core.install_relic_numeric_owner(owner).unwrap();
        }
        (core, state, actor, enemy, ally)
    }
    pub(in crate::game::battle::core) fn place(
        core: &mut BattleCore,
        id: UnitInstanceId,
        x: i32,
        y: i32,
    ) {
        crate::game::battle::core::relic_structural_tests::flow_cases::place_stationary(
            core, id, x, y,
        );
    }
    pub(in crate::game::battle::core) fn cast(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        now: u64,
    ) {
        core.apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: actor,
            skill_id: "r11_aura".into(),
            target: None,
            time_ms: now,
        })
        .unwrap();
    }
    pub(in crate::game::battle::core) fn hit(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        amount: u32,
        now: u64,
    ) {
        core.runtime_transaction(|c| {
            let source = c
                .damage_source_snapshot_for_unit(
                    actor,
                    target,
                    DamageSource::Ability,
                    DamageType::True,
                    amount,
                    Default::default(),
                    0,
                    now,
                    false,
                )
                .ok_or(GameError::InvalidAction)?;
            c.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: target,
                    source_snapshot: source,
                }],
                now,
            );
            crate::game::battle::runtime_control::poll()
        })
        .unwrap();
    }
}
