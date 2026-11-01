mod compiled_support {
    use super::*;
    use crate::{
        authoring::node_graph::{compile, tests::support_fixture, PortableGraph},
        game::{
            battle::{
                core::sim::{BattleExecutionState, BattleLiveCommand},
                event_log::SkillCastTarget,
            },
            data::{relic_data::RelicDatabase, ValidatedGameData},
            managers::uuid_manager::UuidManager,
            relic::{
                structural::{compile_effective_plan, EffectivePlanInput, ProspectiveProvider},
                RelicInventory,
            },
        },
    };
    fn fixture(
        channel: bool,
        equipped: bool,
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        fixture_extra(channel, equipped, false)
    }
    fn fixture_extra(
        channel: bool,
        equipped: bool,
        third: bool,
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let graph = compile(&support_fixture(channel).to_string(), "r11_aura".into()).unwrap();
        let native = PortableGraph::from_ron(&graph.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap();
        fixture_from_native(native, equipped, third)
    }
    fn fixture_from_native(
        native: SkillDatabase,
        equipped: bool,
        third: bool,
    ) -> (BattleCore, BattleExecutionState, UnitInstanceId, UnitInstanceId) {
        fixture_from_native_with_weapon(native, equipped, third, |_| {})
    }
    fn fixture_from_native_with_weapon(
        native: SkillDatabase,
        equipped: bool,
        third: bool,
        configure_weapon: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata),
    ) -> (BattleCore, BattleExecutionState, UnitInstanceId, UnitInstanceId) {
        fixture_from_native_with_catalog(native, equipped, third, configure_weapon, |_| {})
    }
    fn fixture_from_native_with_catalog(
        native: SkillDatabase,
        equipped: bool,
        third: bool,
        configure_weapon: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata),
        configure_relics: impl FnOnce(&mut RelicDatabase),
    ) -> (BattleCore, BattleExecutionState, UnitInstanceId, UnitInstanceId) {
        let mut skills = SkillDatabase::from_ron("fixture", R08_SKILLS)
            .unwrap()
            .skills;
        skills.extend(native.skills);
        let authored = ron::ser::to_string(&SkillDatabase::new(skills)).unwrap();
        let fragments = ron::de::from_str(
            &include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/r09/fragments.ron"
            ))
            .replace("r08_strike", "r11_aura"),
        )
        .unwrap();
        let mut relics: RelicDatabase = ron::de::from_str(r#"(relics:[(id:"single-target-conversion",uuid:"00000000-0000-0000-0000-00000000c501",name:"Conversion fixture",description:"Only the graph owns this support recipe",rarity:"fixture",allow_duplicates:true,effects:[(id:"unrelated_damage",selector:(all_tags:["unrelated_fixture_damage"]),percent:1)],dismantle_equipment_dust:1)])"#).unwrap();
        for (id, uuid) in [("long-breath", 0xc502), ("resonator", 0xc503), ("taunted-enemy-damage-reduction", 0xc504), ("taunted-enemy-basic-attack-boost", 0xc505)] {
            let mut relic = relics.relics[0].clone();
            relic.id = id.into(); relic.uuid = Uuid::from_u128(uuid);
            relics.relics.push(relic);
        }
        configure_relics(&mut relics);
        let (mut core, state, actor, ally) = r12_fragment_equipped_effect_core_with_data(
            &authored,
            |scenario| {
                if third {
                    let mut extra = scenario.groups[1].spawns[0].clone();
                    extra.unit_ref = ScenarioUnitRef::new("support_third");
                    extra.instance_salt = 99;
                    extra.position = Position::new(4, 3);
                    scenario.groups[1].spawns.push(extra);
                }
                for (index, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                        unreachable!()
                    };
                    profile.stats.current_health = 1000;
                    profile.stats.max_health = 1000;
                    profile.fragment_source =
                        Some(crate::game::skill_fragment::FragmentSourceSnapshot {
                            copy_uuid: Uuid::from_u128(0xc510 + index as u128),
                            definition_id: "r09_fragment".into(),
                            form_id: "base".into(),
                            skill_id: "r11_aura".into(),
                        });
                }
            },
            configure_weapon,
            Some(fragments),
            |builder| builder.with_relics(relics),
        );
        place(&mut core, actor, 2, 1);
        place(&mut core, ally, 3, 1);
        if equipped {
            let data: ValidatedGameData = core.game_data.clone().into();
            let employee = core.units[&actor].source_owned_uuid;
            let mut inventory = RelicInventory::default();
            let mut ids = UuidManager::new(0xc520);
            // Two real equipped copies still apply the authored once policy.
            for slot in [0, 1] {
                let copy = inventory
                    .grant(&data.relic_data.relics[0], &mut ids)
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
            let plan = compile_effective_plan(EffectivePlanInput {
                data: &data,
                employee_uuid: employee,
                inventory: &inventory,
                providers: &providers,
            })
            .unwrap()
            .unwrap();
            let owner = crate::game::battle::core::effective_skill::EffectiveBattleOwner::new(
                std::collections::BTreeMap::from([(employee, plan)]),
            )
            .unwrap();
            core.set_effective_owner(owner).unwrap();
        }
        (core, state, actor, ally)
    }
    fn hit(core: &mut BattleCore, target: UnitInstanceId, amount: u32, now: u64) {
        use crate::game::battle::damage::{BattleCommand, DamageSource, DamageType};
        let enemy = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        core.runtime_transaction(|c| {
            let snapshot = c
                .damage_source_snapshot_for_unit(
                    enemy,
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
                    source_snapshot: snapshot,
                }],
                now,
            );
            crate::game::battle::runtime_control::poll()
        })
        .unwrap();
    }
    fn place(core: &mut BattleCore, actor: UnitInstanceId, x: i32, y: i32) {
        crate::game::battle::core::relic_structural_tests::flow_cases::place_stationary(
            core, actor, x, y,
        );
    }
    fn cast(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        target: Option<UnitInstanceId>,
        now: u64,
    ) {
        core.apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: actor,
            skill_id: "r11_aura".into(),
            target: target.map(|unit_instance_id| SkillCastTarget::Unit { unit_instance_id }),
            time_ms: now,
        })
        .unwrap();
    }
    fn shields(core: &BattleCore, target: UnitInstanceId) -> Vec<u32> {
        core.sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter(|r| r.target == target && r.personal_grant.is_some())
            .filter_map(|r| r.shield.map(|s| s.remaining))
            .collect()
    }
    #[test]
    fn personal_shield_composition_uses_release_snapshot_and_independent_native_pools() {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/personal_shield.skill.yaml"))
            .replace("authored-personal-shield", "r11_aura")
            .replace("Milliseconds: 6000", "Milliseconds: 200");
        let artifact = crate::authoring::composition::compile_yaml(&source).unwrap();
        let native = artifact.load_native().unwrap();
        let (mut core, mut state, actor, ally) = fixture_from_native(native, false, false);
        cast(&mut core, actor, None, 0);
        core.units.get_mut(&actor).unwrap().stats.max_health = 1500;
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(shields(&core, actor), vec![300]);
        assert_eq!(shields(&core, ally), vec![300]);
        assert!(!core.resonance_use_has_obligations(actor));
        hit(&mut core, ally, 100, 2);
        assert_eq!(shields(&core, actor), vec![300]);
        assert_eq!(shields(&core, ally), vec![200]);
        assert_eq!(core.units[&ally].stats.current_health, 1000);
        place(&mut core, ally, 7, 3);
        core.units.get_mut(&actor).unwrap().stats.max_health = 2000;
        core.units.get_mut(&actor).unwrap().resonance_current = 100;
        cast(&mut core, actor, None, 3);
        core.step_battle_execution_until(&mut state, 4).unwrap();
        assert_eq!(shields(&core, actor), vec![400]);
        assert_eq!(shields(&core, ally), vec![200]);
        place(&mut core, ally, 3, 1);
        core.reconcile_sustained(5).unwrap();
        assert_eq!(shields(&core, ally), vec![200]);
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert!(shields(&core, ally).is_empty());
        assert_eq!(shields(&core, actor), vec![400]);
        core.step_battle_execution_until(&mut state, 204).unwrap();
        assert!(shields(&core, actor).is_empty());
    }
    #[test]
    fn compiled_support_personal_shields_are_independent_and_release_unlocks() {
        let (mut core, mut state, actor, ally) = fixture(false, false);
        cast(&mut core, actor, None, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(shields(&core, actor), vec![200]);
        assert_eq!(shields(&core, ally), vec![200]);
        assert!(!core.resonance_use_has_obligations(actor));
        assert_eq!(core.units[&actor].resonance_current, 0);
    }
    #[test]
    fn compiled_support_owned_or_other_employee_relic_does_not_enable_conversion() {
        let (core, _, actor, ally) = fixture(false, false);
        let data: ValidatedGameData = core.game_data.clone().into();
        let employee = core.units[&actor].source_owned_uuid;
        let provider = ProspectiveProvider::Fragment(
            core.units[&actor]
                .fragment_source
                .as_ref()
                .unwrap()
                .as_ref()
                .clone(),
        );
        for equip_to in [None, Some(core.units[&ally].source_owned_uuid)] {
            let mut inventory = RelicInventory::default();
            let copy = inventory
                .grant(&data.relic_data.relics[0], &mut UuidManager::new(0xc550))
                .unwrap();
            if let Some(owner) = equip_to {
                inventory.equip(copy, owner, 0, &data.relic_data).unwrap();
            }
            let plan = compile_effective_plan(EffectivePlanInput {
                data: &data,
                employee_uuid: employee,
                inventory: &inventory,
                providers: std::slice::from_ref(&provider),
            })
            .unwrap();
            assert!(plan.is_none());
        }
    }
    #[test]
    fn compiled_support_single_shield_uses_actual_equipped_copy_once_and_separate_range() {
        let (mut core, mut state, actor, ally) = fixture(false, true);
        place(&mut core, ally, 4, 1);
        cast(&mut core, actor, Some(ally), 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(shields(&core, ally), vec![500]);
        assert!(shields(&core, actor).is_empty());
        place(&mut core, ally, 7, 3);
        core.reconcile_sustained(2).unwrap();
        assert_eq!(shields(&core, ally), vec![500]);
    }
    #[test]
    fn compiled_support_channel_regular_coefficients_and_stop() {
        let (mut core, mut state, actor, ally) = fixture(true, false);
        cast(&mut core, actor, None, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert!(core.channel_active(actor));
        assert!(core.resonance_use_has_obligations(actor));
        assert_eq!(
            core.aura_basic_bonus_for_occurrence(ally, true).unwrap(),
            (20, 15)
        );
        assert_eq!(
            core.aura_basic_bonus_for_occurrence(actor, true).unwrap(),
            (0, 0)
        );
        assert_eq!(core.units[&ally].stats.attack, 100);
        cast(&mut core, actor, None, 2);
        assert!(!core.channel_active(actor));
        assert_eq!(
            core.aura_basic_bonus_for_occurrence(ally, true).unwrap(),
            (0, 0)
        );
    }
    #[test]
    fn compiled_support_single_channel_ends_on_range_exit() {
        let (mut core, mut state, actor, ally) = fixture(true, true);
        place(&mut core, ally, 4, 1);
        cast(&mut core, actor, Some(ally), 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(
            core.aura_basic_bonus_for_occurrence(ally, true).unwrap(),
            (40, 20)
        );
        place(&mut core, ally, 7, 3);
        core.reconcile_sustained(2).unwrap();
        assert!(!core.channel_active(actor));
        assert_eq!(
            core.aura_basic_bonus_for_occurrence(ally, true).unwrap(),
            (0, 0)
        );
        assert!(!core.resonance_use_has_obligations(actor));
    }
    #[test]
    fn compiled_support_shield_snapshots_release_hp_and_replaces_only_new_recipients() {
        let (mut core, mut state, actor, ally) = fixture(false, false);
        cast(&mut core, actor, None, 0);
        core.units.get_mut(&actor).unwrap().stats.max_health = 1500;
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(shields(&core, actor), vec![300]);
        assert_eq!(shields(&core, ally), vec![300]);
        hit(&mut core, ally, 100, 2);
        assert_eq!(shields(&core, ally), vec![200]);
        assert_eq!(shields(&core, actor), vec![300]);
        assert_eq!(core.units[&ally].stats.current_health, 1000);
        place(&mut core, ally, 7, 3);
        core.reconcile_sustained(3).unwrap();
        core.units.get_mut(&actor).unwrap().stats.max_health = 2000;
        core.units.get_mut(&actor).unwrap().resonance_current = 100;
        cast(&mut core, actor, None, 3);
        core.step_battle_execution_until(&mut state, 4).unwrap();
        assert_eq!(shields(&core, actor), vec![400]);
        assert_eq!(shields(&core, ally), vec![200]);
        place(&mut core, ally, 3, 1);
        core.reconcile_sustained(5).unwrap();
        assert_eq!(
            shields(&core, ally),
            vec![200],
            "entry does not grant again"
        );
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert!(shields(&core, ally).is_empty());
        assert_eq!(shields(&core, actor), vec![400]);
        core.step_battle_execution_until(&mut state, 204).unwrap();
        assert!(shields(&core, actor).is_empty());
    }
    #[test]
    fn compiled_support_selection_rejection_is_atomic_and_shield_allows_self() {
        for channel in [false, true] {
            let (mut core, mut state, actor, ally) = fixture(channel, true);
            place(&mut core, ally, 7, 3);
            for target in [None, Some(ally)]
                .into_iter()
                .chain(channel.then_some(Some(actor)))
            {
                let before = (
                    core.transaction_debug_snapshot(),
                    core.event_queue_debug_snapshot(),
                );
                assert!(core
                    .apply_live_command(BattleLiveCommand::ActivateSkill {
                        unit_id: actor,
                        skill_id: "r11_aura".into(),
                        target: target
                            .map(|unit_instance_id| SkillCastTarget::Unit { unit_instance_id }),
                        time_ms: 0
                    })
                    .is_err());
                assert_eq!(
                    (
                        core.transaction_debug_snapshot(),
                        core.event_queue_debug_snapshot()
                    ),
                    before
                );
                assert_eq!(core.units[&actor].resonance_current, 100);
            }
            if !channel {
                cast(&mut core, actor, Some(actor), 0);
                core.step_battle_execution_until(&mut state, 1).unwrap();
                assert_eq!(shields(&core, actor), vec![500]);
            }
        }
    }
    #[test]
    fn compiled_support_base_aura_rejoins_and_released_basic_keeps_its_snapshot() {
        let (mut core, mut state, actor, ally) = fixture(true, false);
        cast(&mut core, actor, None, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        place(&mut core, ally, 7, 3);
        core.reconcile_sustained(2).unwrap();
        assert!(core.channel_active(actor));
        assert_eq!(
            core.aura_basic_bonus_for_occurrence(ally, true).unwrap(),
            (0, 0)
        );
        place(&mut core, ally, 3, 1);
        core.reconcile_sustained(3).unwrap();
        assert_eq!(
            core.aura_basic_bonus_for_occurrence(ally, true).unwrap(),
            (20, 15)
        );
        assert_eq!(
            core.aura_basic_bonus_for_occurrence(ally, false).unwrap(),
            (0, 0)
        );
        let enemy = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        let hp = core.units[&enemy].stats.current_health;
        let (_, snapshot) =
            crate::game::battle::core::relic_structural_tests::flow_cases::start_basic_for_support(
                &mut core, ally, enemy, true, 3,
            );
        assert_eq!(snapshot.base_damage, 120);
        assert_eq!(
            crate::game::battle::core::relic_structural_tests::flow_cases::support_contributors(
                &snapshot
            ),
            vec![(20, 15)]
        );
        cast(&mut core, actor, None, 4);
        core.step_battle_execution_until(&mut state, 23).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, hp - 120);
    }
    #[test]
    fn compiled_support_same_family_prefers_stronger_source_then_restores_survivor() {
        for strong_first in [false, true] {
            let (mut core, mut state, actor, ally) = fixture_extra(true, true, true);
            let recipient = core
                .units
                .values()
                .find(|u| {
                    u.owner == Side::Player && u.instance_id != actor && u.instance_id != ally
                })
                .unwrap()
                .instance_id;
            place(&mut core, recipient, 2, 2);
            if strong_first {
                cast(&mut core, actor, Some(recipient), 0);
            } else {
                cast(&mut core, ally, None, 0);
            }
            core.step_battle_execution_until(&mut state, 1).unwrap();
            if strong_first {
                cast(&mut core, ally, None, 2);
            } else {
                cast(&mut core, actor, Some(recipient), 2);
            }
            core.step_battle_execution_until(&mut state, 3).unwrap();
            // The base range contains both other employees. Check every member,
            // including the one also receiving the stronger selected support.
            assert_eq!(
                core.aura_basic_bonus_for_occurrence(actor, true).unwrap(),
                (20, 15)
            );
            for record in core.sustained_effect_snapshot().unwrap().active {
                assert!(record.members.windows(2).all(|pair| pair[0] < pair[1]));
            }
            assert_eq!(
                core.aura_basic_bonus_for_occurrence(recipient, true)
                    .unwrap(),
                (40, 20)
            );
            cast(&mut core, actor, None, 4);
            assert_eq!(
                core.aura_basic_bonus_for_occurrence(recipient, true)
                    .unwrap(),
                (20, 15)
            );
            assert!(core.channel_active(ally));
        }
    }
    #[test]
    fn compiled_support_source_exit_and_terminal_remove_shields_and_channels() {
        for channel in [false, true] {
            for mode in 0..3 {
                let (mut core, mut state, actor, ally) = fixture(channel, false);
                if mode == 2 {
                    Arc::make_mut(&mut core.scenario).terminal_outcomes =
                        BattleScenario::fixture_survival_outcomes(20);
                }
                cast(&mut core, actor, None, 0);
                core.step_battle_execution_until(&mut state, 1).unwrap();
                match mode {
                    0 => {
                        core.apply_live_command(BattleLiveCommand::WithdrawUnit {
                            unit_id: actor,
                            time_ms: 2,
                        })
                        .unwrap();
                    }
                    1 => hit(&mut core, actor, 2000, 2),
                    _ => {
                        core.step_battle_execution_until(&mut state, 21).unwrap();
                        core.finalize_battle_execution(&mut state).unwrap();
                    }
                }
                assert!(shields(&core, ally).is_empty());
                assert!(!core.channel_active(actor));
                assert_eq!(
                    core.aura_basic_bonus_for_occurrence(ally, true).unwrap(),
                    (0, 0)
                );
            }
        }
    }
    #[test]
    fn compiled_support_hp_cost_bypasses_shields_and_only_hard_cc_stops_channels() {
        use crate::game::battle::damage::BattleCommand;
        for channel in [false, true] {
            let (mut core, mut state, actor, ally) = fixture(channel, false);
            cast(&mut core, actor, None, 0);
            core.step_battle_execution_until(&mut state, 1).unwrap();
            core.runtime_transaction(|c| {
                c.process_commands(
                    vec![BattleCommand::PayHealthCost {
                        caster_id: actor,
                        amount: 100,
                        allow_lethal: false,
                    }],
                    2,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
            assert_eq!(core.units[&actor].stats.current_health, 900);
            if !channel {
                assert_eq!(shields(&core, actor), vec![200]);
            }
            let enemy = core
                .units
                .values()
                .find(|u| u.owner == Side::Opponent)
                .unwrap()
                .instance_id;
            for (name, now) in [("r11_silence", 3), ("r11_stun", 4)] {
                core.process_event_for_test(
                    BattleEvent::ApplyBuff {
                        time_ms: now,
                        caster_instance_id: enemy,
                        target_instance_id: actor,
                        buff_id: crate::game::battle::buffs::BuffId::from_name(name),
                        duration_ms: 50,
                        cause: Default::default(),
                    },
                    now,
                )
                .unwrap();
                if channel {
                    assert_eq!(core.channel_active(actor), name == "r11_silence");
                } else {
                    assert_eq!(shields(&core, ally), vec![200]);
                }
            }
        }
    }
    #[test]
    fn compiled_support_batch_and_expiry_cancellation_restore_state_and_retry() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        for channel in [false, true] {
            for ending in [false, true] {
                let (mut core, mut state, actor, _) = fixture(channel, false);
                cast(&mut core, actor, None, 0);
                core.step_battle_execution_until(&mut state, 0).unwrap();
                if ending {
                    core.step_battle_execution_until(&mut state, 200).unwrap();
                }
                let before = (
                    core.transaction_debug_snapshot(),
                    core.event_queue_debug_snapshot(),
                    serde_json::to_string(&core.runtime_fact_snapshot().unwrap()).unwrap(),
                    format!("{state:?}"),
                );
                let token = CancellationToken::default();
                let site = if ending {
                    "sustained_ended"
                } else {
                    "sustained_started"
                };
                let time = if ending { 201 } else { 1 };
                let result = control::with_test_cancellation(site, 1, || {
                    control::run(
                        &ExecutionLimits::default(),
                        &token,
                        ExecutionPhase::TrialExecution,
                        || core.step_battle_execution_until(&mut state, time),
                    )
                });
                assert!(
                    token.is_cancelled() && result.is_err(),
                    "{site}: {result:?}"
                );
                assert_eq!(
                    (
                        core.transaction_debug_snapshot(),
                        core.event_queue_debug_snapshot(),
                        serde_json::to_string(&core.runtime_fact_snapshot().unwrap()).unwrap(),
                        format!("{state:?}")
                    ),
                    before
                );
                core.step_battle_execution_until(&mut state, time).unwrap();
                assert_eq!(
                    core.sustained_effect_snapshot().unwrap().active.is_empty(),
                    ending
                );
            }
        }
    }
    fn equip_support_modifiers(core: &mut BattleCore, actor: UnitInstanceId, single: bool) {
        let data: ValidatedGameData = core.game_data.clone().into();
        let employee = core.units[&actor].source_owned_uuid;
        let mut inventory = RelicInventory::default();
        let mut ids = UuidManager::new(0xc580);
        let names = if single { vec!["single-target-conversion", "long-breath", "resonator"] } else { vec!["long-breath", "resonator"] };
        for (slot, name) in names.iter().enumerate() {
            let copy=inventory.grant(data.relic_data.get_by_id(name).unwrap(), &mut ids).unwrap();
            inventory.equip(copy,employee,slot,&data.relic_data).unwrap();
        }
        let providers=[ProspectiveProvider::Fragment(core.units[&actor].fragment_source.as_ref().unwrap().as_ref().clone())];
        let plan=compile_effective_plan(EffectivePlanInput {data:&data,employee_uuid:employee,inventory:&inventory,providers:&providers}).unwrap().unwrap();
        let receipt=serde_json::to_value(plan.explain().unwrap()).unwrap();
        // The range rule is explicitly dormant after conversion; its copy is not applied.
        let text=receipt.to_string();
        assert!(text.contains("long-breath"));
        if single { assert!(text.contains("Dormant")); }
        let owner=crate::game::battle::core::effective_skill::EffectiveBattleOwner::new(std::collections::BTreeMap::from([(employee,plan)])).unwrap();
        core.set_effective_owner(owner).unwrap();
    }
    #[test]
    fn compiled_support_duration_and_base_footprint_modifiers_use_real_equipped_copies() {
        let (mut core,mut state,actor,ally)=fixture(true,false);
        equip_support_modifiers(&mut core,actor,false);
        place(&mut core,ally,5,1);
        cast(&mut core,actor,None,0);
        core.step_battle_execution_until(&mut state,1).unwrap();
        assert_eq!(core.aura_basic_bonus_for_occurrence(ally,true).unwrap(),(20,15));
        core.step_battle_execution_until(&mut state,201).unwrap();
        assert!(core.channel_active(actor));
        core.step_battle_execution_until(&mut state,251).unwrap();
        assert!(!core.channel_active(actor));
        assert_eq!(core.aura_basic_bonus_for_occurrence(ally,true).unwrap(),(0,0));
    }
    #[test]
    fn compiled_support_conversion_keeps_duration_but_never_expands_selection_reach() {
        let (mut core,mut state,actor,ally)=fixture(true,false);
        equip_support_modifiers(&mut core,actor,true);
        place(&mut core,ally,5,1);
        assert!(core.apply_live_command(BattleLiveCommand::ActivateSkill {unit_id:actor,skill_id:"r11_aura".into(),target:Some(SkillCastTarget::Unit {unit_instance_id:ally}),time_ms:0}).is_err());
        place(&mut core,ally,4,1);
        cast(&mut core,actor,Some(ally),0);
        core.step_battle_execution_until(&mut state,1).unwrap();
        assert_eq!(core.aura_basic_bonus_for_occurrence(ally,true).unwrap(),(40,20));
        core.step_battle_execution_until(&mut state,201).unwrap();
        assert!(core.channel_active(actor));
        core.step_battle_execution_until(&mut state,251).unwrap();
        assert!(!core.channel_active(actor));
    }

    include!("composition_channel_support_tests.rs");
    include!("composition_channel_upkeep_tests.rs");
    include!("composition_duration_modifier_tests.rs");
    include!("composition_support_radius_tests.rs");
    include!("composition_control_tests.rs");
    include!("composition_taunt_damage_tests.rs");
    include!("composition_range_tests.rs");

}
