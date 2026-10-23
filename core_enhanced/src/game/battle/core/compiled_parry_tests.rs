mod compiled_parry {
    use super::*;
    use crate::{
        authoring::node_graph::{compile, tests::parry_fixture, PortableGraph},
        game::{
            ability::sustained::SustainedEffectDef,
            battle::{
                core::sim::{BattleExecutionState, BattleLiveCommand},
                damage::{BattleCommand, DamageSource, DamageType},
                runtime_facts::ReflectionOutcome,
            },
        },
    };
    fn fixture() -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        fixture_with_accessory(false)
    }
    fn fixture_with_accessory(
        accessory: bool,
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let graph = compile(&parry_fixture().to_string(), "r11_aura".into()).unwrap();
        let compiled = PortableGraph::from_ron(&graph.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap();
        let mut skills = SkillDatabase::from_ron("fixture", R08_SKILLS)
            .unwrap()
            .skills;
        skills.extend(compiled.skills);
        if accessory {
            skills.extend(SkillDatabase::from_ron("fixture:kill-resource", r#"(skills:[(id:"kill_resource",name:"",kind:Untargeted,focus_time_ms:0,use_lifetime:Action,cast_targeting:Explicit(selection:self,target:SelfUnit,reach:pattern,air_capable:false),steps:[(id:"gain",delay_ms:0,target:SelfUnit,delivery:Instant,effects:[(id:"gain",op:ModifyResonance(amount:25))])])])"#).unwrap().skills);
        }
        let authored = ron::ser::to_string(&SkillDatabase::new(skills)).unwrap();
        let fragments = ron::de::from_str(
            &include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/r09/fragments.ron"
            ))
            .replace("r08_strike", "r11_aura"),
        )
        .unwrap();
        let (core, state, actor, _) = r12_fragment_equipped_effect_core(
            &authored,
            |scenario| {
                if accessory {
                    let mut second = scenario.groups[0].spawns[0].clone();
                    second.unit_ref = ScenarioUnitRef::new("surviving_enemy");
                    second.instance_salt = 90;
                    second.position = Position::new(7, 3);
                    scenario.groups[0].spawns.push(second);
                }
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                        unreachable!()
                    };
                    profile.fragment_source =
                        Some(crate::game::skill_fragment::FragmentSourceSnapshot {
                            copy_uuid: Uuid::from_u128(0x1800 + i as u128),
                            definition_id: "r09_fragment".into(),
                            form_id: "base".into(),
                            skill_id: "r11_aura".into(),
                        });
                    profile.stats.current_health = 1000;
                    profile.stats.max_health = 1000;
                }
            },
            |weapon| {
                if accessory {
                    weapon.ability_activations =
                        vec![crate::game::ability::AbilityActivationBinding {
                            ability_id: "kill_resource".into(),
                            activation: crate::game::ability::AbilityActivationDef::TriggerProc {
                                trigger: crate::game::stats::TriggerType::OnKill,
                                proc_chance_percent: 100,
                                internal_cooldown_ms: 0,
                                max_triggers_per_battle: None,
                            },
                        }];
                }
            },
            Some(fragments),
        );
        let enemy = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent && u.body.projected_tile() == Position::new(6, 1))
            .unwrap()
            .instance_id;
        (core, state, actor, enemy)
    }
    fn cast(core: &mut BattleCore, actor: UnitInstanceId) {
        core.apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: actor,
            skill_id: "r11_aura".into(),
            target: None,
            time_ms: 0,
        })
        .unwrap();
    }
    fn hit(
        core: &mut BattleCore,
        enemy: UnitInstanceId,
        actor: UnitInstanceId,
        source: DamageSource,
        segments: &[u32],
        now: u64,
    ) {
        try_hit(core, enemy, actor, source, segments, now).unwrap();
    }
    fn try_hit(
        core: &mut BattleCore,
        enemy: UnitInstanceId,
        actor: UnitInstanceId,
        source: DamageSource,
        segments: &[u32],
        now: u64,
    ) -> Result<(), GameError> {
        core.runtime_transaction(|c| {
            let prior = c.begin_strike_scope();
            for amount in segments {
                let snapshot = c
                    .damage_source_snapshot_for_unit(
                        enemy,
                        actor,
                        source,
                        DamageType::True,
                        *amount,
                        Default::default(),
                        0,
                        now,
                        false,
                    )
                    .unwrap();
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: actor,
                        source_snapshot: snapshot,
                    }],
                    now,
                );
            }
            c.end_strike_scope(prior);
            crate::game::battle::runtime_control::poll()
        })
    }
    #[test]
    fn compiled_parry_passive_requires_equipped_form_and_reduces_special_damage_without_reflection()
    {
        let (mut core, _, actor, enemy) = fixture();
        core.units.get_mut(&actor).unwrap().resonance_current = 0;
        assert_eq!(
            core.sustained_effect_snapshot()
                .unwrap()
                .passive_guards
                .len(),
            2
        );
        let enemy_hp = core.units[&enemy].stats.current_health;
        hit(
            &mut core,
            enemy,
            actor,
            DamageSource::Environment,
            &[100],
            1,
        );
        assert_eq!(core.units[&actor].stats.current_health, 930);
        assert_eq!(core.units[&enemy].stats.current_health, enemy_hp);
        assert_eq!(
            core.runtime_fact_summary()
                .latest_damage
                .as_ref()
                .unwrap()
                .protection
                .passive_guard
                .as_ref()
                .unwrap()
                .reflection,
            ReflectionOutcome::NotEligible
        );
        core.units.get_mut(&actor).unwrap().fragment_source = None;
        hit(&mut core, enemy, actor, DamageSource::Ability, &[100], 2);
        assert_eq!(core.units[&actor].stats.current_health, 830);
    }
    #[test]
    fn compiled_parry_reflection_uses_first_actual_hp_segment_and_skips_fully_absorbed_strike() {
        for shield in [70, 140] {
            let (mut core, _, actor, enemy) = fixture();
            core.begin_sustained(
                actor,
                actor,
                SustainedEffectDef::Shield {
                    duration_ms: 1000,
                    amount: shield,
                    priority: 0,
                    shared_radius_units: None,
                },
                0,
            )
            .unwrap();
            let enemy_hp = core.units[&enemy].stats.current_health;
            hit(
                &mut core,
                enemy,
                actor,
                DamageSource::Ability,
                &[100, 100],
                1,
            );
            assert_eq!(
                core.units[&actor].stats.current_health,
                1000 - (140 - shield)
            );
            assert_eq!(
                core.units[&enemy].stats.current_health,
                enemy_hp - if shield == 70 { 35 } else { 0 }
            );
            if shield == 140 {
                assert_eq!(
                    core.runtime_fact_summary()
                        .latest_damage
                        .as_ref()
                        .unwrap()
                        .protection
                        .passive_guard
                        .as_ref()
                        .unwrap()
                        .roll_percent,
                    None
                );
            }
        }
    }
    #[test]
    fn compiled_parry_success_counters_once_and_ends_at_actual_immunity_boundary() {
        let (mut core, mut state, actor, enemy) = fixture();
        cast(&mut core, actor);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert!(core.resonance_use_has_obligations(actor));
        let enemy_hp = core.units[&enemy].stats.current_health;
        hit(
            &mut core,
            enemy,
            actor,
            DamageSource::Ability,
            &[100, 100],
            10,
        );
        assert_eq!(core.units[&actor].stats.current_health, 1000);
        assert_eq!(core.units[&enemy].stats.current_health, enemy_hp - 250);
        assert_eq!(
            core.sustained_effect_snapshot().unwrap().active[0].expires_at_ms,
            510
        );
        hit(
            &mut core,
            enemy,
            actor,
            DamageSource::Environment,
            &[100],
            509,
        );
        assert_eq!(core.units[&actor].stats.current_health, 1000);
        core.step_battle_execution_until(&mut state, 510).unwrap();
        assert!(!core.resonance_use_has_obligations(actor));
        hit(
            &mut core,
            enemy,
            actor,
            DamageSource::Environment,
            &[100],
            510,
        );
        assert_eq!(core.units[&actor].stats.current_health, 930);
        assert_eq!(core.units[&enemy].stats.current_health, enemy_hp - 250);
    }
    #[test]
    fn compiled_parry_miss_removes_only_own_guard_then_restores_and_unlocks() {
        let (mut core, mut state, actor, enemy) = fixture();
        core.units
            .get_mut(&actor)
            .unwrap()
            .incoming_damage_modifiers
            .damage_reduction_percent = 20;
        cast(&mut core, actor);
        core.step_battle_execution_until(&mut state, 31).unwrap();
        let enemy_hp = core.units[&enemy].stats.current_health;
        hit(&mut core, enemy, actor, DamageSource::Ability, &[100], 31);
        assert_eq!(core.units[&actor].stats.current_health, 920);
        assert_eq!(core.units[&enemy].stats.current_health, enemy_hp - 40);
        core.step_battle_execution_until(&mut state, 61).unwrap();
        assert!(!core.resonance_use_has_obligations(actor));
        hit(&mut core, enemy, actor, DamageSource::Ability, &[100], 61);
        assert_eq!(core.units[&actor].stats.current_health, 870);
        assert_eq!(core.units[&enemy].stats.current_health, enemy_hp - 65);
    }
    #[test]
    fn compiled_parry_control_after_release_does_not_cancel_window_and_immunity_blocks_control() {
        let (mut core, mut state, actor, enemy) = fixture();
        cast(&mut core, actor);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        core.process_event_for_test(
            BattleEvent::ApplyBuff {
                time_ms: 2,
                caster_instance_id: enemy,
                target_instance_id: actor,
                buff_id: crate::game::battle::buffs::BuffId::from_name("r11_stun"),
                duration_ms: 3,
                cause: Default::default(),
            },
            2,
        )
        .unwrap();
        core.step_battle_execution_until(&mut state, 10).unwrap();
        hit(&mut core, enemy, actor, DamageSource::Ability, &[100], 10);
        assert_eq!(core.units[&actor].stats.current_health, 1000);
        let before = core.sustained_effect_snapshot().unwrap();
        core.process_event_for_test(
            BattleEvent::ApplyBuff {
                time_ms: 11,
                caster_instance_id: enemy,
                target_instance_id: actor,
                buff_id: crate::game::battle::buffs::BuffId::from_name("r11_stun"),
                duration_ms: 100,
                cause: Default::default(),
            },
            11,
        )
        .unwrap();
        core.process_event_for_test(
            BattleEvent::ApplyBuff {
                time_ms: 11,
                caster_instance_id: enemy,
                target_instance_id: actor,
                buff_id: crate::game::battle::buffs::BuffId::from_name("r11_silence"),
                duration_ms: 100,
                cause: Default::default(),
            },
            11,
        )
        .unwrap();
        core.runtime_transaction(|c| {
            c.process_commands(
                vec![BattleCommand::BeginSustained {
                    caster_id: enemy,
                    target_id: actor,
                    definition: SustainedEffectDef::Control {
                        duration_ms: 100,
                        kind: crate::game::ability::sustained::ControlKind::Root,
                    },
                }],
                11,
            );
            crate::game::battle::runtime_control::poll()
        })
        .unwrap();
        assert_eq!(core.sustained_effect_snapshot().unwrap(), before);
        assert!(
            core.buffs.is_empty(),
            "new stun and silence are both blocked"
        );
        assert!(core.active_hard_cc_release_time_ms(actor, 11).is_none());
    }

    #[test]
    fn compiled_parry_dot_attachment_and_hp_cost_survive_immunity_but_damage_waits() {
        use crate::game::battle::buffs::{BuffDatabase, BuffId};
        let (mut core, mut state, actor, enemy) = fixture();
        let buffs:BuffDatabase=ron::de::from_str(r#"(buffs:[(name:"parry_dot",kind:PeriodicDamage(damage_per_tick:100,damage_type:True),tick_interval_ms:100,max_stacks:1,reapply_policy:RefreshDuration)])"#).unwrap();
        core.game_data = GameDataBuilder::empty()
            .with_skills((*core.game_data.skill_data).clone())
            .with_skill_fragments((*core.game_data.skill_fragment_data).clone())
            .with_equipment(core.game_data.equipment_data.items.clone())
            .with_buffs(buffs)
            .build_arc();
        cast(&mut core, actor);
        core.step_battle_execution_until(&mut state, 10).unwrap();
        // A tick never consumes a pending parry window.
        hit(&mut core, enemy, actor, DamageSource::BuffTick, &[100], 10);
        assert_eq!(core.units[&actor].stats.current_health, 930);
        hit(&mut core, enemy, actor, DamageSource::Ability, &[100], 11);
        core.process_event_for_test(
            BattleEvent::ApplyBuff {
                time_ms: 12,
                caster_instance_id: enemy,
                target_instance_id: actor,
                buff_id: BuffId::from_name("parry_dot"),
                duration_ms: 700,
                cause: Default::default(),
            },
            12,
        )
        .unwrap();
        core.runtime_transaction(|c| {
            c.process_commands(
                vec![BattleCommand::PayHealthCost {
                    caster_id: actor,
                    amount: 100,
                    allow_lethal: false,
                }],
                12,
            );
            crate::game::battle::runtime_control::poll()
        })
        .unwrap();
        assert_eq!(core.units[&actor].stats.current_health, 830);
        core.step_battle_execution_until(&mut state, 500).unwrap();
        assert_eq!(core.units[&actor].stats.current_health, 830);
        assert_eq!(
            core.buffs.len(),
            1,
            "periodic debuff remains attached during immunity"
        );
        core.step_battle_execution_until(&mut state, 512).unwrap();
        assert_eq!(
            core.units[&actor].stats.current_health, 760,
            "first tick after immunity loses HP with passive reduction"
        );
    }
    #[test]
    fn compiled_parry_active_window_keeps_basic_attack_and_block_capacity() {
        let (mut core, mut state, actor, enemy) = fixture();
        let block = core.units[&actor].block_capacity;
        cast(&mut core, actor);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        let enemy_hp = core.units[&enemy].stats.current_health;
        core.units
            .get_mut(&actor)
            .unwrap()
            .basic_attack
            .range_policy = crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        core.units.get_mut(&actor).unwrap().next_basic_attack_ms = 2;
        core.process_event_for_test(
            BattleEvent::AttackStart {
                time_ms: 2,
                attacker_instance_id: actor,
                target_instance_id: Some(enemy),
                schedule_next: true,
                cause: Default::default(),
            },
            2,
        )
        .unwrap();
        core.step_battle_execution_until(&mut state, 3).unwrap();
        assert_eq!(core.units[&actor].block_capacity, block);
        assert_eq!(core.units[&enemy].stats.current_health, enemy_hp - 100);
        assert!(core.resonance_use_has_obligations(actor));
    }
    #[test]
    fn compiled_parry_withdraw_death_and_terminal_clear_window_and_passive() {
        for mode in 0..3 {
            let (mut core, mut state, actor, enemy) = fixture();
            if mode == 2 {
                std::sync::Arc::make_mut(&mut core.scenario).terminal_outcomes =
                    BattleScenario::fixture_survival_outcomes(20);
            }
            cast(&mut core, actor);
            core.step_battle_execution_until(&mut state, 1).unwrap();
            match mode {
                0 => {
                    core.apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: actor,
                        time_ms: 1,
                    })
                    .unwrap();
                }
                1 => {
                    hit(&mut core, enemy, actor, DamageSource::BuffTick, &[2000], 2);
                }
                _ => {
                    core.step_battle_execution_until(&mut state, 21).unwrap();
                    core.finalize_battle_execution(&mut state).unwrap();
                }
            }
            let view = core.sustained_effect_snapshot().unwrap();
            assert!(view.active.is_empty());
            assert!(!view.passive_guards.iter().any(|g| g.unit == actor));
        }
    }
    #[test]
    fn compiled_parry_deadline_counter_and_expiry_faults_restore_core_queue_and_retry() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "parry_after_deflection",
            "parry_after_counter_invoke",
            "parry_after_counter_execution",
            "sustained_ended",
        ] {
            let (mut core, mut state, actor, enemy) = fixture();
            cast(&mut core, actor);
            core.step_battle_execution_until(&mut state, 1).unwrap();
            if site == "sustained_ended" {
                hit(&mut core, enemy, actor, DamageSource::Ability, &[100], 10);
                core.step_battle_execution_until(&mut state, 509).unwrap();
            }
            let before = (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot(),
                serde_json::to_string(&core.runtime_fact_snapshot().unwrap()).unwrap(),
                format!("{state:?}"),
            );
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || {
                        if site == "sustained_ended" {
                            core.step_battle_execution_until(&mut state, 510)
                                .map(|_| ())
                        } else {
                            try_hit(&mut core, enemy, actor, DamageSource::Ability, &[100], 10)
                        }
                    },
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
                before,
                "{site}"
            );
            if site == "sustained_ended" {
                core.step_battle_execution_until(&mut state, 510).unwrap();
                assert!(!core.resonance_use_has_obligations(actor));
            } else {
                hit(&mut core, enemy, actor, DamageSource::Ability, &[100], 10);
                assert_eq!(core.units[&actor].stats.current_health, 1000);
            }
        }
    }

    #[test]
    fn compiled_parry_equipment_on_kill_keeps_reflection_counter_credit_and_global_gain_lock() {
        for counter in [false, true] {
            let (mut core, mut state, actor, enemy) = fixture_with_accessory(true);
            core.units.get_mut(&enemy).unwrap().stats.current_health =
                if counter { 200 } else { 20 };
            if counter {
                cast(&mut core, actor);
                core.step_battle_execution_until(&mut state, 1).unwrap();
            } else {
                core.units.get_mut(&actor).unwrap().resonance_current = 0;
            }
            hit(&mut core, enemy, actor, DamageSource::Ability, &[100], 10);
            core.step_battle_execution_until(&mut state, 11).unwrap();
            assert!(!core.units.get(&enemy).is_some_and(|u| u.is_active()));
            assert_eq!(
                core.units[&actor].resonance_current,
                if counter { 0 } else { 25 }
            );
            let fact = core
                .runtime_fact_summary()
                .latest_resource_change
                .as_ref()
                .unwrap();
            assert_eq!(fact.requested_delta, 25);
            assert_eq!(fact.after - fact.before, if counter { 0 } else { 25 });
            assert_eq!(core.runtime_fact_summary().deaths, 1);
        }
    }
}
