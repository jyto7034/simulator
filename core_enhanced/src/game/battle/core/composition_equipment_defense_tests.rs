mod composition_equipment_defense {
    use super::*;
    use crate::{
        authoring::composition::{compile_json, parse_yaml, Artifact},
        game::battle::{
            core::sim::{BattleExecutionState, BattleLiveCommand},
            damage::{BattleCommand, DamageSource, DamageType},
        },
    };
    use serde_json::json;
    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/equipment_defense.skill.yaml"
    ));
    struct Fixture {
        core: BattleCore,
        state: BattleExecutionState,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        ally: UnitInstanceId,
    }
    fn fixture(op: &str, attached_buff: bool) -> Fixture {
        fixture_extra(op, attached_buff, None, false)
    }
    fn fixture_extra(
        op: &str,
        attached_buff: bool,
        extra: Option<&str>,
        enemy_auto: bool,
    ) -> Fixture {
        fixture_extra_with_defense(op, attached_buff, extra, enemy_auto, YAML, |_| {}, |_| {})
    }
    fn fixture_extra_with_defense(
        op: &str,
        attached_buff: bool,
        extra: Option<&str>,
        enemy_auto: bool,
        defense: &str,
        configure_weapon: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata),
        configure_scenario: impl FnOnce(&mut BattleScenario),
    ) -> Fixture {
        let mut authored = r12_control_ron(op);
        if attached_buff {
            authored = authored.replace(&format!("op: {op})"), &format!(
                "op: {op}), (id: \"attached\", attached_to: Some(\"control\"), op: ApplyBuff(buff_id: \"r11_stun\", duration_ms: 100))"));
        }
        if let Some(extra) = extra {
            let mut skills = SkillDatabase::from_ron("barrier-extra.ron", &authored).unwrap();
            skills.skills[0].steps[0]
                .effects
                .push(ron::de::from_str(extra).unwrap());
            authored = ron::ser::to_string(&skills).unwrap();
        }
        let artifact = crate::authoring::composition::compile_yaml(defense).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let (mut core, state, caster, ally) = r12_fragment_equipped_effect_core_with_data(
            &authored,
            |scenario| {
                scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(50000);
                scenario.groups[0].enemy_movement_plan = None;
                if enemy_auto {
                    let BattleUnitSource::TestFixture { profile, .. } =
                        &mut scenario.groups[0].spawns[0].draft.source
                    else {
                        unreachable!()
                    };
                    profile.skill_id = Some("r11_aura".into());
                    profile.skill_activation_mode = SkillActivationMode::Auto;
                    profile.resonance.start = 90;
                    profile.resonance.max = 100;
                }
                scenario.groups[0].spawns[0].draft.equipped_items =
                    vec![crate::game::battle::types::BattleEquipmentBinding {
                        weapon_stage: Default::default(),
                        base_uuid: Uuid::from_u128(0x1101),
                        persistent_copy_uuid: Some(Uuid::from_u128(0xe370)),
                        slot: None,
                    }];
                configure_scenario(scenario);
            },
            configure_weapon,
            None,
            |builder| builder.with_equipment_defenses(artifact.equipment_defenses().to_vec())
                .with_equipment_supports(artifact.equipment_supports().to_vec()),
        );
        let target = core
            .units
            .values()
            .filter(|u| u.owner == Side::Opponent)
            .min_by_key(|u| u.spawn_order)
            .unwrap()
            .instance_id;
        for unit in core.units.values_mut() {
            unit.stats.current_health = 1000;
            unit.stats.max_health = 1000;
            unit.stats.defense = 0;
            unit.stats.magic_resist = 0;
            unit.stats.attack_interval_ms = 100000;
            unit.next_basic_attack_ms = 100000;
            unit.stats.move_speed_units_per_ms = 0;
        }
        core.enable_authoring_observation();
        core.bind_authoring_observation(
            crate::game::battle::core::authoring_observation::ObservationKey {
                entry_ordinal: 1,
                battle_uuid: Uuid::from_u128(0xb370),
            },
        )
        .unwrap();
        Fixture {
            core,
            state,
            caster,
            target,
            ally,
        }
    }
    fn cast(f: &mut Fixture, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
        let unit = f.core.units.get_mut(&f.caster).unwrap();
        unit.resonance_current = unit.resonance_max;
        f.core
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: f.caster,
                skill_id: "r11_aura".into(),
                target: None,
                time_ms: now,
            })
            .unwrap();
    }
    fn resolve(f: &mut Fixture, now: u64) {
        cast(f, now);
        f.core
            .step_battle_execution_until(&mut f.state, now + 1)
            .unwrap();
    }
    fn witness(f: &Fixture) -> &crate::game::battle::core::equipment_defense::SkillBarrierWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_skill_barrier
            .as_ref()
            .unwrap()
    }
    fn raw_damage(f: &mut Fixture, source: DamageSource, now: u64) {
        f.core
            .runtime_transaction(|c| {
                let snapshot = c
                    .damage_source_snapshot_for_unit(
                        f.caster,
                        f.target,
                        source,
                        DamageType::True,
                        10,
                        Default::default(),
                        0,
                        now,
                        false,
                    )
                    .unwrap();
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: f.target,
                        source_snapshot: snapshot,
                    }],
                    now,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
    }

    #[test]
    fn composition_equipment_defense_roundtrip_shared_source_and_rejections() {
        let doc = parse_yaml(YAML).unwrap();
        let artifact = compile_json(&doc.to_string()).unwrap();
        assert!(artifact
            .sources()
            .values()
            .any(|s| s.definition.contains("DefenseProgram")));
        let loaded = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert_eq!(loaded.equipment_defenses().len(), 1);
        for unused in [false, true] {
            let mut invalid = doc.clone();
            invalid["values"][0]["value"]["DefenseProgram"]["RechargingSkillBarrier"]
                ["recharge_ms"] = json!(0);
            if unused {
                invalid["equipment_defenses"][0]["program"] = json!({"Literal":{
                "DefenseProgram":{"RechargingSkillBarrier":{"recharge_ms":100}}}});
            }
            let error = compile_json(&invalid.to_string()).unwrap_err();
            assert!(
                error
                    .source
                    .definition
                    .ends_with("DefenseProgram.RechargingSkillBarrier.recharge_ms"),
                "{error:?}"
            );
        }
        let mut duplicate = doc.clone();
        duplicate["equipment_defenses"]
            .as_array_mut()
            .unwrap()
            .push(doc["equipment_defenses"][0].clone());
        assert!(compile_json(&duplicate.to_string())
            .unwrap_err()
            .to_string()
            .contains("duplicate"));
        let mut missing = doc;
        missing["equipment_defenses"][0]["program"] =
            json!({"Unresolved":"choose recharge policy"});
        assert!(compile_json(&missing.to_string()).is_err());
    }

    #[test]
    fn composition_equipment_defense_blocks_one_hit_and_attached_buff_all_damage_types() {
        for kind in ["Physical", "Magic", "True"] {
            let mut f = fixture(&format!("Damage(amount:100, damage_type:{kind})"), true);
            resolve(&mut f, 0);
            assert_eq!(f.core.units[&f.target].stats.current_health, 1000, "{kind}");
            assert!(f.core.buffs.is_empty());
            assert_eq!((witness(&f).time_ms, witness(&f).ready_at_ms), (1, 20001));
            assert_eq!(
                witness(&f).equipment.persistent_copy_uuid,
                Some(Uuid::from_u128(0xe370))
            );
            assert_eq!(
                witness(&f)
                    .incoming
                    .cast
                    .as_ref()
                    .unwrap()
                    .skill_id
                    .as_str(),
                "r11_aura"
            );
            resolve(&mut f, 10);
            assert_eq!(f.core.units[&f.target].stats.current_health, 900);
            assert!(
                !f.core.buffs.is_empty(),
                "unblocked hit applies its attached status"
            );
            assert_eq!(witness(&f).ready_at_ms, 20001);
        }
    }

    #[test]
    fn composition_equipment_defense_exact_recharge_ignores_basic_tick_and_environment() {
        let mut f = fixture("Damage(amount:100, damage_type:True)", false);
        resolve(&mut f, 0);
        for (index, source) in [
            DamageSource::BasicAttack,
            DamageSource::BuffTick,
            DamageSource::Environment,
        ]
        .into_iter()
        .enumerate()
        {
            raw_damage(&mut f, source, 2 + index as u64);
        }
        assert_eq!(f.core.units[&f.target].stats.current_health, 970);
        assert_eq!(witness(&f).ready_at_ms, 20001);
        resolve(&mut f, 19999); // hit at 20000 is one millisecond too early
        assert_eq!(f.core.units[&f.target].stats.current_health, 870);
        resolve(&mut f, 20000); // hit at exactly 20001 consumes the regenerated charge
        assert_eq!(f.core.units[&f.target].stats.current_health, 870);
        assert_eq!(
            (witness(&f).time_ms, witness(&f).ready_at_ms),
            (20001, 40001)
        );
    }

    #[test]
    fn composition_equipment_defense_pure_control_and_immunity_before_consumption() {
        let op = "Sustained(definition:HitStatus(duration_ms:100, profile:(definition:\"barrier-stun\", kind:Stun)))";
        let mut f = fixture(op, false);
        f.core
            .units
            .get_mut(&f.target)
            .unwrap()
            .target_traits
            .push(crate::game::battle::types::UnitTargetTrait::StunImmune);
        resolve(&mut f, 0);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_skill_barrier
            .is_none());
        f.core
            .units
            .get_mut(&f.target)
            .unwrap()
            .target_traits
            .clear();
        resolve(&mut f, 10);
        assert!(f.core.hit_stun_until(f.target, Some(11)).is_none());
        assert_eq!(witness(&f).ready_at_ms, 20011);
        resolve(&mut f, 20);
        assert!(f.core.hit_stun_until(f.target, Some(21)).is_some());
        assert_eq!(f.core.units[&f.target].stats.current_health, 1000);
    }

    #[test]
    fn composition_equipment_defense_precedes_hp_shield_and_cleans_withdrawal() {
        let mut f = fixture("Damage(amount:100, damage_type:True)", false);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.target,
                    f.target,
                    crate::game::ability::sustained::SustainedEffectDef::Shield {
                        duration_ms: 30000,
                        amount: 250,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
            })
            .unwrap();
        resolve(&mut f, 0);
        let target = f.target;
        let shields = |c: &BattleCore| {
            c.sustained_effect_snapshot()
                .unwrap()
                .active
                .iter()
                .filter(|v| v.target == target)
                .filter_map(|v| v.shield.as_ref().map(|s| s.remaining))
                .sum::<u32>()
        };
        assert_eq!(shields(&f.core), 250);
        resolve(&mut f, 10);
        assert_eq!(shields(&f.core), 150);
        assert_eq!(f.core.units[&f.target].stats.current_health, 1000);
        // The real player copy uses the same generic program and independent state.
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.ally,
                time_ms: 11,
            })
            .unwrap();
        assert_eq!(witness(&f).target, f.target);
        f.core
            .runtime_transaction(|c| c.clear_sustained(12))
            .unwrap();
        assert!(f.core.equipment_defense.is_empty());
    }

    #[test]
    fn composition_equipment_defense_same_hit_segments_and_pure_debuffs() {
        let mut f = fixture_extra(
            "Damage(amount:100, damage_type:True)",
            false,
            Some("(id:\"second-component\",op:Damage(amount:50,damage_type:True))"),
            false,
        );
        resolve(&mut f, 0);
        assert_eq!(f.core.units[&f.target].stats.current_health, 1000);
        resolve(&mut f, 10);
        assert_eq!(f.core.units[&f.target].stats.current_health, 850);
        for op in [
            "ModifyStats(modifier:(stat:Defense,kind:Flat,value:-10))",
            "Sustained(definition:Modifiers(duration_ms:100,modifiers:[(stat:Defense,kind:Flat,value:-10)]))",
            "ModifyResonance(amount:-10)",
        ] {
            let mut f = fixture(op, false);
            f.core.units.get_mut(&f.target).unwrap().resonance_current = 50;
            resolve(&mut f, 0);
            assert_eq!(f.core.units[&f.target].stats.defense, 0, "{op}");
            assert_eq!(f.core.units[&f.target].resonance_current, 50, "{op}");
            assert_eq!(witness(&f).time_ms, 1, "{op}");
            resolve(&mut f, 10);
            if op.starts_with("ModifyResonance") {
                assert_eq!(f.core.units[&f.target].resonance_current, 40);
            } else { assert_eq!(f.core.units[&f.target].stats.defense, -10); }
        }
    }

    #[test]
    fn composition_equipment_defense_actual_hostile_auto_skill_and_withdrawal() {
        let mut f = fixture_extra("Damage(amount:100, damage_type:True)", false, None, true);
        // Enemy automatic admission requires a real gain crossing the cap.
        f.core
            .runtime_transaction(|core| {
                core.process_commands(
                    vec![BattleCommand::ModifyResonance {
                        target_id: f.target,
                        amount: 10,
                        allow_autocast_when_full: true,
                    }],
                    0,
                );
                core.schedule_pending_autocasts(0);
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 1001)
            .unwrap();
        let target = witness(&f).target;
        let old_copy = witness(&f).equipment.runtime_item_instance_id;
        let owned = f.core.units[&target].source_owned_uuid;
        let position = f.core.units[&target].body.projected_tile();
        let draft = f
            .core
            .scenario
            .groups
            .iter()
            .flat_map(|g| &g.spawns)
            .find(|s| s.draft.owned_uuid == owned)
            .unwrap()
            .draft
            .clone();
        assert!(target == f.caster || target == f.ally);
        assert_eq!(f.core.units[&target].stats.current_health, 1000);
        assert!(!f.core.equipment_defense.is_empty());
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: target,
                time_ms: 1001,
            })
            .unwrap();
        assert!(f.core.equipment_defense.is_empty());
        assert_eq!(
            witness(&f).target,
            target,
            "past receipt remains immutable after exit"
        );
        let other = if target == f.caster { f.ally } else { f.caster };
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: other,
                time_ms: 1001,
            })
            .unwrap();
        let super::super::super::sim::BattleLiveCommandOutcome::UnitDeployed { unit_id: fresh } = f
            .core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position,
                tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
                facing: crate::game::battle::tile_range::FacingDirection::Right,
                instance_salt: 0xe371,
                time_ms: 1001,
                current_hp_policy: None,
            })
            .unwrap()
        else {
            unreachable!()
        };
        assert_ne!(fresh, target);
        let unit = f.core.units.get_mut(&fresh).unwrap();
        unit.stats.max_health = 1000;
        unit.stats.current_health = 1000;
        unit.next_basic_attack_ms = 100000;
        f.core
            .runtime_transaction(|core| {
                core.process_commands(
                    vec![BattleCommand::ModifyResonance {
                        target_id: f.target,
                        amount: 100,
                        allow_autocast_when_full: true,
                    }],
                    1001,
                );
                core.schedule_pending_autocasts(1001);
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 1002)
            .unwrap();
        assert_eq!(witness(&f).target, fresh);
        assert_ne!(witness(&f).equipment.runtime_item_instance_id, old_copy);
        assert_eq!(witness(&f).time_ms, 1002);
        assert_eq!(f.core.units[&fresh].stats.current_health, 1000);
    }

    #[test]
    fn composition_equipment_defense_pull_requires_actual_displacement() {
        let mut f = fixture("Pull(max_distance_tiles:2)", false);
        let from = f.core.units[&f.target].body.position;
        f.core
            .battlefield
            .add_static_obstacle(Position::new(5, 1))
            .unwrap();
        resolve(&mut f, 0);
        assert_eq!(f.core.units[&f.target].body.position, from);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_skill_barrier
            .is_none());
        assert!(f
            .core
            .battlefield
            .remove_static_obstacle(Position::new(5, 1)));
        resolve(&mut f, 10);
        assert_eq!(f.core.units[&f.target].body.position, from);
        assert_eq!(witness(&f).time_ms, 11);
        resolve(&mut f, 20);
        assert!(f.core.units[&f.target].body.position.x < from.x);
    }

    #[test]
    fn composition_equipment_defense_parry_invulnerability_and_reaction_do_not_consume() {
        use crate::game::ability::sustained::SustainedEffectDef;
        let mut f = fixture("Damage(amount:100, damage_type:True)", false);
        f.core
            .runtime_transaction(|core| {
                core.begin_sustained(
                    f.target,
                    f.target,
                    SustainedEffectDef::Parry {
                        duration_ms: 200,
                        window_ms: 30,
                        invulnerable_ms: 40,
                        miss_recovery_ms: 20,
                        reduction_percent: 0,
                        counter_skill: "r11_aura".into(),
                        end_on_resolution: false,
                        fragment_guard: None,
                        guard_charge: None,
                    },
                    0,
                )
            })
            .unwrap();
        raw_damage(&mut f, DamageSource::BasicAttack, 0);
        resolve(&mut f, 0);
        assert_eq!(f.core.units[&f.target].stats.current_health, 1000);
        assert_eq!(
            f.core.units[&f.caster].stats.current_health, 900,
            "the real parry reaction bypasses the attacker's ready barrier"
        );
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_skill_barrier
            .is_none());
        resolve(&mut f, 41);
        assert_eq!(witness(&f).time_ms, 42);
        assert_eq!(f.core.units[&f.target].stats.current_health, 1000);
    }

    #[test]
    fn composition_equipment_defense_slow_immunity_corrosion_and_death_cleanup() {
        use crate::game::battle::types::UnitTargetTrait;
        let mut f = fixture(
            "Sustained(definition:Control(duration_ms:100,kind:Slow(percent:50)))",
            false,
        );
        f.core
            .units
            .get_mut(&f.target)
            .unwrap()
            .target_traits
            .push(UnitTargetTrait::SlowImmune);
        resolve(&mut f, 0);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_skill_barrier
            .is_none());
        f.core
            .units
            .get_mut(&f.target)
            .unwrap()
            .target_traits
            .clear();
        resolve(&mut f, 10);
        assert_eq!(witness(&f).time_ms, 11);
        // This branch tests reconciliation of an already committed corrosion flag.
        f.core.units.get_mut(&f.target).unwrap().corroded = true;
        f.core
            .runtime_transaction(|core| core.reconcile_equipment_defenses())
            .unwrap();
        assert!(format!("{:?}", f.core.equipment_defense).contains("cooldowns: {}"));
        let mut f = fixture("Damage(amount:100, damage_type:True)", false);
        resolve(&mut f, 0);
        f.core
            .units
            .get_mut(&f.target)
            .unwrap()
            .stats
            .current_health = 5;
        raw_damage(&mut f, DamageSource::BasicAttack, 2);
        assert!(!f.core.units[&f.target].is_active());
        assert!(format!("{:?}", f.core.equipment_defense).contains("cooldowns: {}"));
        assert_eq!(witness(&f).time_ms, 1, "historical receipt survives death");
    }

    #[test]
    fn composition_equipment_defense_observation_parity_and_retained_refusal_restore() {
        use crate::game::battle::runtime_control as native;
        let mut observed = fixture("Damage(amount:100, damage_type:True)", false);
        let mut plain = Fixture {
            core: observed.core.clone(),
            state: observed.state.clone(),
            caster: observed.caster,
            target: observed.target,
            ally: observed.ally,
        };
        plain.core.authoring_observation = None;
        for now in [0, 10, 20001] {
            resolve(&mut observed, now);
            resolve(&mut plain, now);
            assert_eq!(
                observed.core.transaction_debug_snapshot(),
                plain.core.transaction_debug_snapshot()
            );
            assert_eq!(
                observed.core.event_queue_debug_snapshot(),
                plain.core.event_queue_debug_snapshot()
            );
        }
        let mut f = fixture("Damage(amount:100, damage_type:True)", false);
        cast(&mut f, 0);
        f.core.step_battle_execution_until(&mut f.state, 0).unwrap();
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 1)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "skill_barrier_observation")
            .unwrap()
            .2;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            format!("{:?}", f.state),
        );
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 1)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "skill_barrier_observation",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(
            before,
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                serde_json::to_value(&f.core.authoring_observation).unwrap(),
                format!("{:?}", f.state)
            )
        );
        f.core.runtime_facts.limits = Default::default();
        f.core.step_battle_execution_until(&mut f.state, 1).unwrap();
        assert_eq!(witness(&f).time_ms, 1);
    }

    #[test]
    fn composition_equipment_defense_cancel_restores_cooldown_receipt_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = fixture("Damage(amount:100, damage_type:True)", true);
        cast(&mut f, 0);
        f.core.step_battle_execution_until(&mut f.state, 0).unwrap();
        let snapshot = |f: &Fixture| {
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.core.equipment_defense),
                format!("{:?}", f.state),
                serde_json::to_value(&f.core.authoring_observation).unwrap(),
            )
        };
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_defense_consumed", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 1),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(snapshot(&f), before);
        f.core.step_battle_execution_until(&mut f.state, 1).unwrap();
        assert_eq!(f.core.units[&f.target].stats.current_health, 1000);
        assert_eq!(witness(&f).ready_at_ms, 20001);
        resolve(&mut f, 10);
        assert_eq!(f.core.units[&f.target].stats.current_health, 900);
    }
    include!("composition_equipment_evasion_tests.rs");
    include!("composition_equipment_support_tests.rs");
    include!("composition_shield_beneficiary_tests.rs");
    include!("composition_resonance_gain_tests.rs");
    include!("composition_heal_barrier_tests.rs");
    include!("composition_actual_heal_reaction_tests.rs");
    include!("composition_basic_heal_echo_tests.rs");
    include!("composition_overheal_reserve_tests.rs");
    include!("composition_deferred_damage_tests.rs");
    include!("composition_channel_opening_tests.rs");
    include!("composition_idle_hp_guard_tests.rs");
    include!("composition_incoming_flat_tests.rs");
    include!("composition_wearer_reduction_tests.rs");
    include!("composition_distant_defense_tests.rs");
    include!("composition_recipient_support_tests.rs");
}
