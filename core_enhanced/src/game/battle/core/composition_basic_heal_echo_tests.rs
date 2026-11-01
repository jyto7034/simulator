mod composition_basic_heal_echo {
    use super::*;
    use crate::game::{
        battle::{event_log::BattleLogEvent, tile_range::TileRangePolicy},
        data::abnormality_data::BasicAttackAction,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/basic_heal_echo.skill.yaml"
    ));
    fn setup(source: &str, flat: u32, count: u32) -> Fixture {
        let mut f = fixture_extra_with_defense(
            "Heal(amount:20)",
            false,
            None,
            false,
            source,
            |w| {
                w.weapon_profile.as_mut().unwrap().action = BasicAttackAction::HealLowestHealth {
                    flat,
                    spell_power_percent: 0,
                    max_targets: count,
                }
            },
            |s| {
                let mut ally = s.groups[1].spawns[0].clone();
                ally.unit_ref =
                    crate::game::battle::scenario::ScenarioUnitRef::new("echo-third-ally");
                ally.instance_salt += 100;
                ally.position = crate::game::resources::Position::new(4, 3);
                s.groups[1].spawns.push(ally);
            },
        );
        let u = f.core.units.get_mut(&f.caster).unwrap();
        u.basic_attack.range_policy = TileRangePolicy::WholeFieldValidTiles;
        u.basic_attack.defense_tile_range = None;
        u.stats.current_health = 900;
        u.basic_attack.windup_ms = 1;
        f
    }
    fn others(f: &Fixture) -> Vec<UnitInstanceId> {
        f.core
            .units
            .values()
            .filter(|u| u.owner == Side::Player && u.instance_id != f.caster)
            .map(|u| u.instance_id)
            .collect()
    }
    fn start(f: &mut Fixture, normal: bool) {
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .next_basic_attack_ms = 10;
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: 10,
                    attacker_instance_id: f.caster,
                    target_instance_id: None,
                    schedule_next: normal,
                    cause: Default::default(),
                },
                10,
            )
            .unwrap();
    }
    fn release(f: &mut Fixture) -> Result<(), crate::game::behavior::GameError> {
        for _ in 0..1000 {
            let event = f.core.event_queue.pop().expect("heal event");
            let time = event.time_ms();
            assert!(time <= 11, "missing basic heal: {event:?}");
            let heal = matches!(
                event.event.view().as_ref(),
                BattleEvent::BasicHealResolve { .. }
            );
            if heal {
                return f.core.process_event(event, time);
            }
            f.core.process_event(event, time)?;
        }
        panic!("bounded basic heal release");
    }
    fn witness(f: &Fixture) -> &crate::game::battle::core::equipment_support::BasicHealEchoWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_basic_heal_echo
            .as_ref()
            .expect("echo witness")
    }
    #[test]
    fn composition_basic_heal_echo_roundtrip_bounds_and_old_support() {
        let a = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        assert_eq!(
            a.equipment_supports()[0]
                .program
                .basic_heal_echo
                .unwrap()
                .max_hp_cap_percent,
            10
        );
        assert!(a.sources().get("equipment_supports[0].program").is_some());
        for (old, new) in [
            ("actual_heal_percent: 25", "actual_heal_percent: 0"),
            ("actual_heal_percent: 25", "actual_heal_percent: 10001"),
            ("max_hp_cap_percent: 10", "max_hp_cap_percent: 0"),
            ("max_hp_cap_percent: 10", "max_hp_cap_percent: 101"),
        ] {
            assert!(
                crate::authoring::composition::compile_yaml(&SOURCE.replace(old, new)).is_err()
            );
        }
        let old = crate::authoring::composition::compile_yaml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/equipment_support.skill.yaml"
        )))
        .unwrap();
        assert!(old.equipment_supports()[0]
            .program
            .basic_heal_echo
            .is_none());
    }
    #[test]
    fn composition_basic_heal_echo_sum_once_after_boost_and_target_hp_cap() {
        let mut f = setup(
            &SOURCE.replace("heal_percent: 0", "heal_percent: 25"),
            100,
            3,
        );
        let other = others(&f);
        f.core
            .units
            .get_mut(&other[0])
            .unwrap()
            .stats
            .current_health = 800;
        f.core
            .units
            .get_mut(&other[1])
            .unwrap()
            .stats
            .current_health = 990;
        start(&mut f, true);
        release(&mut f).unwrap();
        assert_eq!(f.core.units[&other[0]].stats.current_health, 925);
        assert_eq!(f.core.units[&other[1]].stats.current_health, 1000);
        let w = witness(&f);
        assert_eq!(
            (
                w.actual_other_healed,
                w.requested_self_heal,
                w.actual_self_healed
            ),
            (135, 33, 0)
        );
        assert_eq!(
            f.core.units[&f.caster].stats.current_health, 1000,
            "own basic heal excluded from sum; echo is overheal"
        );
        assert_eq!(w.healing_source.actor, Some(f.caster));
        assert!(w.healing_source.action.is_some());
        assert!(w.equipment.runtime_item_instance_id != Uuid::nil());
    }
    #[test]
    fn composition_basic_heal_echo_single_focus_and_cap_at_current_max_hp() {
        let mut f = setup(SOURCE, 600, 1);
        let other = others(&f);
        f.core
            .units
            .get_mut(&other[0])
            .unwrap()
            .stats
            .current_health = 100;
        f.core
            .units
            .get_mut(&other[1])
            .unwrap()
            .stats
            .current_health = 200;
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 800;
        start(&mut f, true);
        release(&mut f).unwrap();
        assert_eq!(f.core.units[&other[0]].stats.current_health, 700);
        assert_eq!(f.core.units[&other[1]].stats.current_health, 200);
        assert_eq!(f.core.units[&f.caster].stats.current_health, 900);
        let w = witness(&f);
        assert_eq!(
            (
                w.actual_other_healed,
                w.requested_self_heal,
                w.actual_self_healed
            ),
            (600, 100, 100)
        );
    }
    #[test]
    fn composition_basic_heal_echo_rounds_total_once_and_ignores_self_only() {
        let mut f = setup(SOURCE, 3, 2);
        for id in others(&f) {
            f.core.units.get_mut(&id).unwrap().stats.current_health = 100;
        }
        start(&mut f, true);
        release(&mut f).unwrap();
        assert_eq!(
            (
                witness(&f).actual_other_healed,
                witness(&f).requested_self_heal
            ),
            (6, 1)
        );
        assert_eq!(f.core.units[&f.caster].stats.current_health, 901);
        let mut f = setup(SOURCE, 100, 1);
        start(&mut f, true);
        release(&mut f).unwrap();
        assert_eq!(f.core.units[&f.caster].stats.current_health, 1000);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_basic_heal_echo
            .is_none());
    }
    #[test]
    fn composition_basic_heal_echo_triggered_and_fragment_heals_do_not_echo() {
        let mut f = setup(SOURCE, 100, 1);
        let id = others(&f)[0];
        f.core.units.get_mut(&id).unwrap().stats.current_health = 100;
        start(&mut f, false);
        release(&mut f).unwrap();
        assert_eq!(f.core.units[&id].stats.current_health, 200);
        assert_eq!(f.core.units[&f.caster].stats.current_health, 900);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_basic_heal_echo
            .is_none());
        resolve(&mut f, 20);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_basic_heal_echo
            .is_none());
    }
    #[test]
    fn composition_basic_heal_echo_zero_rounding_and_provider_removed_before_release() {
        for remove in [false, true] {
            let mut f = setup(SOURCE, if remove { 100 } else { 3 }, 1);
            let id = others(&f)[0];
            f.core.units.get_mut(&id).unwrap().stats.current_health = 100;
            start(&mut f, true);
            if remove {
                f.core
                    .items
                    .retain(|_, i| i.owner_unit_instance != f.caster);
            }
            release(&mut f).unwrap();
            assert_eq!(f.core.units[&f.caster].stats.current_health, 900);
            assert!(f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_basic_heal_echo
                .is_none());
        }
    }
    #[test]
    fn composition_basic_heal_echo_cancel_rolls_back_base_and_self_heal_then_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(SOURCE, 100, 1);
        let id = others(&f)[0];
        f.core.units.get_mut(&id).unwrap().stats.current_health = 100;
        start(&mut f, true);
        let before = f.core.transaction_debug_snapshot();
        let state_before = format!("{:?}", f.state);
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("basic_heal_echo_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 11),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(format!("{:?}", f.state), state_before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        f.core
            .step_battle_execution_until(&mut f.state, 11)
            .unwrap();
        assert_eq!(f.core.units[&id].stats.current_health, 200);
        assert_eq!(f.core.units[&f.caster].stats.current_health, 925);
    }
    #[test]
    fn composition_basic_heal_echo_old_single_heal_and_observer_off_match() {
        let mut f = setup(SOURCE, 100, 1);
        f.core.units.get_mut(&f.caster).unwrap().basic_attack.action = BasicAttackAction::Heal {
            flat: 100,
            spell_power_percent: 0,
            footprint: None,
        };
        let id = others(&f)[0];
        f.core.units.get_mut(&id).unwrap().stats.current_health = 100;
        let mut clone = Fixture {
            core: f.core.clone(),
            state: f.state.clone(),
            caster: f.caster,
            target: f.target,
            ally: f.ally,
        };
        clone.core.authoring_observation = None;
        start(&mut f, true);
        release(&mut f).unwrap();
        start(&mut clone, true);
        release(&mut clone).unwrap();
        assert_eq!(f.core.units[&f.caster].stats.current_health, 925);
        assert_eq!(clone.core.units[&f.caster].stats.current_health, 925);
        assert_eq!(
            serde_json::to_value(f.core.runtime_fact_summary()).unwrap(),
            serde_json::to_value(clone.core.runtime_fact_summary()).unwrap()
        );
        let heals = f
            .core
            .event_log
            .entries_after_seq(0)
            .iter()
            .filter(|e| matches!(e.event,BattleLogEvent::HpChanged {delta,..} if delta>0))
            .count();
        assert_eq!(heals, 2);
        let hp = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_nonzero_sourced_hp
            .as_ref()
            .unwrap();
        assert_eq!(hp.target_unit, f.caster);
        assert_eq!(hp.change.hp_delta(), 25);
        assert_eq!(hp.equipment, Some(witness(&f).equipment));
        assert!(hp.source.is_none() && hp.fragment.is_none());
        assert!(hp
            .authored
            .as_ref()
            .unwrap()
            .field_path
            .ends_with("basic_heal_echo"));
    }
    #[test]
    fn composition_basic_heal_echo_tile_heal_sums_other_allies_only() {
        let mut f = setup(SOURCE, 3, 3);
        let mut rows = vec!["XXXXXXXXXXXXXXXXX".to_string(); 17];
        rows[8] = "XXXXXXXX@XXXXXXXX".into();
        f.core.units.get_mut(&f.caster).unwrap().basic_attack.action = BasicAttackAction::Heal {
            flat: 3,
            spell_power_percent: 0,
            footprint: Some(crate::game::battle::tile_range::TileRangePattern {
                include_anchor_tile: true,
                rows,
            }),
        };
        for id in others(&f) {
            f.core.units.get_mut(&id).unwrap().stats.current_health = 100;
        }
        start(&mut f, true);
        release(&mut f).unwrap();
        assert_eq!(witness(&f).actual_other_healed, 6);
        assert_eq!(f.core.units[&f.caster].stats.current_health, 904);
    }
    #[test]
    fn composition_basic_heal_echo_release_max_hp_and_actor_exit() {
        let mut f = setup(SOURCE, 600, 1);
        let id = others(&f)[0];
        f.core.units.get_mut(&id).unwrap().stats.current_health = 100;
        start(&mut f, true);
        f.core.units.get_mut(&f.caster).unwrap().stats.max_health = 2000;
        release(&mut f).unwrap();
        assert_eq!(
            (witness(&f).max_hp, witness(&f).requested_self_heal),
            (2000, 150)
        );
        for withdraw in [false, true] {
            let mut f = setup(SOURCE, 100, 1);
            let id = others(&f)[0];
            f.core.units.get_mut(&id).unwrap().stats.current_health = 100;
            start(&mut f, true);
            if withdraw {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: f.caster,
                        time_ms: 10,
                    })
                    .unwrap();
            } else {
                f.core.units.get_mut(&f.caster).unwrap().corroded = true;
            }
            f.core
                .step_battle_execution_until(&mut f.state, 11)
                .unwrap();
            assert!(f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_basic_heal_echo
                .is_none());
        }
    }
    #[test]
    fn composition_basic_heal_echo_retained_failure_restores_public_step_and_retries() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(SOURCE, 100, 1);
        let id = others(&f)[0];
        f.core.units.get_mut(&id).unwrap().stats.current_health = 100;
        start(&mut f, true);
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 11)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "basic_heal_echo_observation")
            .map(|(_, _, bytes)| *bytes)
            .max()
            .unwrap();
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = f.core.transaction_debug_snapshot();
        let state_before = format!("{:?}", f.state);
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 11)
            .unwrap_err();
        assert!(
            matches!(
                error,
                crate::game::behavior::GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "basic_heal_echo_observation",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(format!("{:?}", f.state), state_before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        f.core.runtime_facts.limits = Default::default();
        f.core
            .step_battle_execution_until(&mut f.state, 11)
            .unwrap();
        assert_eq!(f.core.units[&id].stats.current_health, 200);
        assert_eq!(f.core.units[&f.caster].stats.current_health, 925);
    }
    #[test]
    fn composition_overheal_reserve_pendant_basic_heal_echo_stores_actual_overflow() {
        let mut doc = parse_yaml(SOURCE).unwrap();
        let armor = parse_yaml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/overheal_reserve.skill.yaml"
        )))
        .unwrap();
        doc["values"]
            .as_array_mut()
            .unwrap()
            .extend(armor["values"].as_array().unwrap().iter().cloned());
        doc["equipment_defenses"] = armor["equipment_defenses"].clone();
        let mut f = setup(&doc.to_string(), 100, 1);
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 990;
        let ally = others(&f)[0];
        f.core.units.get_mut(&ally).unwrap().stats.current_health = 100;
        start(&mut f, true);
        release(&mut f).unwrap();
        assert_eq!(f.core.units[&ally].stats.current_health, 200);
        assert_eq!(f.core.units[&f.caster].stats.current_health, 1000);
        assert_eq!(
            (
                witness(&f).requested_self_heal,
                witness(&f).actual_self_healed
            ),
            (25, 10)
        );
        let reserve = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_overheal_reserve
            .as_ref()
            .unwrap();
        assert_eq!((reserve.incoming_overheal, reserve.stored_after), (15, 15));
        assert_eq!(reserve.target, f.caster);
        assert!(reserve.incoming.reaction);
        assert!(reserve
            .incoming
            .authored
            .as_ref()
            .unwrap()
            .field_path
            .ends_with("basic_heal_echo"));
        assert!(reserve.incoming.action.is_some());
    }
}
