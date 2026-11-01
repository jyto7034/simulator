mod priority_heal_tests {
    use super::*;

    fn setup(count: usize) -> (BattleCore, UnitInstanceId, Vec<UnitInstanceId>) {
        let mut core = test_core();
        let actor = healer(&mut core);
        core.units.get_mut(&actor).unwrap().basic_attack.action =
            BasicAttackAction::HealLowestHealth {
                flat: 10,
                spell_power_percent: 50,
                max_targets: 2,
            };
        let targets = (0..count)
            .map(|i| {
                let id = add_player_target(
                    &mut core,
                    10 + i as u128,
                    WorldVec2::new(1.0 + i as f32, 1.0),
                );
                core.units.get_mut(&id).unwrap().stats.current_health = 50;
                id
            })
            .collect();
        (core, actor, targets)
    }
    fn release(core: &mut BattleCore) {
        // Death/control can enqueue earlier movement or wake events. Execute
        // those too instead of assuming the heal is the next heap entry.
        for _ in 0..1000 {
            let event = core.event_queue.pop().unwrap();
            let time = event.time_ms();
            assert!(time <= 50);
            let release = matches!(
                event.event.view().as_ref(),
                BattleEvent::BasicHealResolve { .. }
            );
            core.process_event(event, time).unwrap();
            if release {
                assert_eq!(time, 50);
                return;
            }
        }
        panic!("bounded fixture must reach its heal release");
    }
    fn healed(core: &BattleCore) -> Vec<(UnitInstanceId, i32)> {
        core.event_log
            .entries_after_seq(0)
            .iter()
            .filter_map(|entry| match entry.event {
                BattleLogEvent::HpChanged {
                    target_instance_id,
                    delta,
                    damage_source,
                    damage_type,
                    critical,
                    ..
                } if delta > 0 => {
                    assert_eq!(entry.time_ms, 50);
                    assert!(damage_source.is_none() && damage_type.is_none() && critical.is_none());
                    Some((target_instance_id, delta))
                }
                _ => None,
            })
            .collect()
    }
    #[test]
    fn priority_heal_zero_one_two_three_targets_full_amount_once() {
        for count in 0..=3 {
            let (mut core, actor, targets) = setup(count);
            start(&mut core, actor);
            if count == 0 {
                assert!(core.event_queue.is_empty());
                assert_eq!(core.runtime_fact_view().active_actions().count(), 0);
                continue;
            }
            release(&mut core);
            assert_eq!(
                healed(&core),
                targets
                    .iter()
                    .take(2)
                    .map(|id| (*id, 20))
                    .collect::<Vec<_>>()
            );
            for (i, id) in targets.iter().enumerate() {
                assert_eq!(
                    core.units[id].stats.current_health,
                    if i < 2 { 70 } else { 50 }
                );
            }
            assert_eq!(core.runtime_fact_summary().damage_hp_lost, 0);
            assert_eq!(core.runtime_fact_view().active_actions().count(), 0);
        }
    }
    #[test]
    fn priority_heal_ratio_then_deficit_then_identity_ignores_distance_and_insertion() {
        for reverse in [false, true] {
            let (mut core, actor, targets) = setup(3);
            core.units
                .get_mut(&targets[0])
                .unwrap()
                .stats
                .current_health = 25;
            core.units.get_mut(&targets[1]).unwrap().stats.max_health = 200;
            core.units
                .get_mut(&targets[1])
                .unwrap()
                .stats
                .current_health = 50;
            core.units.get_mut(&targets[2]).unwrap().stats.max_health = 200;
            core.units
                .get_mut(&targets[2])
                .unwrap()
                .stats
                .current_health = 50;
            if reverse {
                let units: Vec<_> = targets
                    .iter()
                    .map(|id| core.units.remove(id).unwrap())
                    .collect();
                for unit in units.into_iter().rev() {
                    core.units.insert(unit.instance_id, unit);
                }
            }
            assert_eq!(core.choose_basic_heal_target(actor), Some(targets[1]));
            start(&mut core, actor);
            release(&mut core);
            assert_eq!(healed(&core), vec![(targets[1], 20), (targets[2], 20)]);
        }
    }
    #[test]
    fn priority_heal_release_reselects_current_candidates_and_caps_each() {
        let (mut core, actor, targets) = setup(3);
        start(&mut core, actor);
        core.units
            .get_mut(&targets[0])
            .unwrap()
            .stats
            .current_health = 100;
        core.units
            .get_mut(&targets[1])
            .unwrap()
            .stats
            .current_health = 95;
        core.units
            .get_mut(&targets[2])
            .unwrap()
            .stats
            .current_health = 99;
        core.units.get_mut(&actor).unwrap().stats.spell_power = 40;
        release(&mut core);
        assert_eq!(healed(&core), vec![(targets[1], 5), (targets[2], 1)]);
        assert_eq!(core.units[&targets[0]].stats.current_health, 100);
    }
    #[test]
    fn priority_heal_self_included_enemy_corrosion_and_noncombatants_excluded() {
        let (mut core, actor, targets) = setup(3);
        core.units.get_mut(&actor).unwrap().stats.current_health = 10;
        core.units.get_mut(&targets[0]).unwrap().role = BattleUnitRole::IndependentWall;
        core.units.get_mut(&targets[1]).unwrap().corroded = true;
        let enemy = add_enemy(&mut core, 99, WorldVec2::new(1.0, 1.0), 1.0);
        core.units.get_mut(&enemy).unwrap().stats.current_health = 1;
        start(&mut core, actor);
        release(&mut core);
        assert_eq!(healed(&core), vec![(actor, 20), (targets[2], 20)]);
    }
    #[test]
    fn priority_heal_epoch_cancel_and_no_remaining_candidates_close_action() {
        for cancel in [false, true] {
            let (mut core, actor, targets) = setup(2);
            start(&mut core, actor);
            if cancel {
                core.units.get_mut(&actor).unwrap().basic_attack_epoch += 1;
            } else {
                for id in targets {
                    core.units.get_mut(&id).unwrap().stats.current_health = 100;
                }
            }
            release(&mut core);
            assert!(healed(&core).is_empty());
            assert_eq!(core.runtime_fact_view().active_actions().count(), 0);
        }
    }
    #[test]
    fn priority_heal_dead_original_target_is_replaced_at_release() {
        let (mut core, actor, targets) = setup(3);
        start(&mut core, actor);
        core.apply_hp_delta_and_record(None, targets[0], -100, 1, HpChangeReason::Command);
        release(&mut core);
        assert_eq!(core.units[&targets[0]].stats.current_health, 0);
        assert_eq!(healed(&core), vec![(targets[1], 20), (targets[2], 20)]);
    }
    #[test]
    fn priority_heal_enemy_approach_taunt_does_not_redirect_healing() {
        let (mut core, actor, targets) = setup(2);
        let enemy = add_enemy(&mut core, 99, WorldVec2::new(7.0, 1.0), 1.0);
        let effect=ron::de::from_str("Control(duration_ms:1500,kind:ApproachTaunt(profile:(speed_percent:100,attack_range_units:1000,damage_reduction_percent:20,basic_damage_bonus_percent:30)))").unwrap();
        core.begin_sustained(enemy, actor, effect, 0).unwrap();
        assert!(core.approach_taunt(actor).is_some());
        assert!(!core.approach_taunt_attack_allowed(actor, targets[0]));
        assert_eq!(core.choose_basic_heal_target(actor), Some(targets[0]));
        start(&mut core, actor);
        release(&mut core);
        assert_eq!(healed(&core), vec![(targets[0], 20), (targets[1], 20)]);
    }
    #[test]
    fn priority_heal_authoring_rejects_invalid_count_and_delivery() {
        use crate::game::ability::DeliveryDef;
        for count in [0, 129, u32::MAX] {
            let action = BasicAttackAction::HealLowestHealth {
                flat: 10,
                spell_power_percent: 50,
                max_targets: count,
            };
            assert!(action.validate_delivery(&DeliveryDef::Instant).is_err());
        }
        let action: BasicAttackAction =
            ron::de::from_str("HealLowestHealth(flat:10,spell_power_percent:50,max_targets:2)")
                .unwrap();
        assert!(action.validate_delivery(&DeliveryDef::Instant).is_ok());
        assert!(action
            .validate_delivery(
                &ron::de::from_str::<DeliveryDef>("Projectile(speed_units_per_ms:1000)").unwrap()
            )
            .is_err());
        let encoded = ron::ser::to_string(&action).unwrap();
        assert_eq!(
            encoded,
            ron::ser::to_string(&ron::de::from_str::<BasicAttackAction>(&encoded).unwrap())
                .unwrap()
        );
    }
    #[test]
    fn priority_heal_range_rechecked_and_summoned_combatant_eligible() {
        use crate::game::battle::types::BattleUnitSourceIdentity;
        let (mut core, actor, targets) = setup(3);
        let unit = core.units.get_mut(&actor).unwrap();
        unit.basic_attack.range_policy = TileRangePolicy::Pattern;
        unit.basic_attack.defense_tile_range = Some(TileRangePattern {
            include_anchor_tile: true,
            rows: vec!["XXXXXXX".into(), "XXX@XXX".into(), "XXXXXXX".into()],
        });
        core.units.get_mut(&targets[2]).unwrap().source_identity =
            BattleUnitSourceIdentity::SummonedUnit {
                definition_id: "friendly_summon_fixture".into(),
                base_uuid: Uuid::nil(),
                summoner_instance_id: actor,
                summon_sequence: 1,
                batch_uuid: Uuid::from_u128(42),
            };
        start(&mut core, actor);
        core.units.get_mut(&targets[0]).unwrap().body.position = WorldVec2::new(7.0, 1.0);
        release(&mut core);
        assert_eq!(healed(&core), vec![(targets[1], 20), (targets[2], 20)]);
    }
    #[test]
    fn priority_heal_observation_does_not_change_gameplay_or_action_identity() {
        let (mut core, actor, _) = setup(3);
        let mut observed = core.clone();
        observed.enable_authoring_observation();
        for candidate in [&mut core, &mut observed] {
            start(candidate, actor);
            release(candidate);
        }
        assert_eq!(
            core.transaction_debug_snapshot(),
            observed.transaction_debug_snapshot()
        );
        assert_eq!(
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
            serde_json::to_value(observed.runtime_fact_view()).unwrap()
        );
        let heals: Vec<_> = core
            .runtime_fact_view()
            .actors()
            .filter_map(|(_, fact)| fact.received.latest_heal.as_ref())
            .collect();
        assert_eq!(heals.len(), 2);
        assert!(heals[0].source.action.is_some());
        assert_eq!(heals[0].source.action, heals[1].source.action);
        assert_eq!(heals[0].time_ms, heals[1].time_ms);
    }
    #[test]
    fn priority_heal_retained_budget_rejection_preserves_state_and_retries() {
        use crate::game::battle::runtime_control as native;
        let (mut core, actor, _) = setup(3);
        start(&mut core, actor);
        let event = core.event_queue.pop().unwrap();
        let mut probe = core.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.runtime_transaction(|candidate| candidate.process_event(event.clone(), 50))
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "basic_heal_targets")
            .unwrap()
            .2;
        core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = core.transaction_debug_snapshot();
        let error = core
            .runtime_transaction(|candidate| candidate.process_event(event.clone(), 50))
            .unwrap_err();
        assert!(
            matches!(
                error,
                crate::game::behavior::GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(core.transaction_debug_snapshot(), before);
        core.runtime_facts.limits = Default::default();
        core.runtime_transaction(|candidate| candidate.process_event(event, 50))
            .unwrap();
        assert_eq!(healed(&core), healed(&probe));
    }
    #[test]
    fn priority_heal_cancellation_after_first_target_rolls_back_and_retry_matches_clone() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut core, actor, _) = setup(3);
        core.enable_authoring_observation();
        start(&mut core, actor);
        let event = core.event_queue.pop().unwrap();
        let mut expected = core.clone();
        expected.process_event(event.clone(), 50).unwrap();
        let before = core.transaction_debug_snapshot();
        let before_obs = serde_json::to_value(&core.authoring_observation).unwrap();
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("basic_heal_target_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || core.runtime_transaction(|candidate| candidate.process_event(event.clone(), 50)),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(core.transaction_debug_snapshot(), before);
        assert_eq!(
            serde_json::to_value(&core.authoring_observation).unwrap(),
            before_obs
        );
        core.process_event(event, 50).unwrap();
        assert_eq!(
            core.transaction_debug_snapshot(),
            expected.transaction_debug_snapshot()
        );
        assert_eq!(healed(&core), healed(&expected));
    }
    #[test]
    fn queen_bee_basic_native_catalog_three_targets_and_new_profile_input() {
        let data = crate::game::data::ValidatedGameData::load_live_embedded().unwrap();
        let profile = data
            .equipment_data
            .items
            .iter()
            .find(|item| item.id == "placeholder_queen_bee_weapon")
            .unwrap()
            .weapon_profile
            .as_ref()
            .unwrap();
        for (action, maximum, amount) in [
            (profile.action.clone(), 3, 14),
            (
                BasicAttackAction::HealLowestHealth {
                    flat: 7,
                    spell_power_percent: 0,
                    max_targets: 4,
                },
                4,
                7,
            ),
        ] {
            for count in 0..=4 {
                let (mut core, actor, targets) = setup(count);
                core.units.get_mut(&actor).unwrap().basic_attack.action = action.clone();
                start(&mut core, actor);
                if count == 0 {
                    assert!(core.event_queue.is_empty());
                    continue;
                }
                release(&mut core);
                assert_eq!(
                    healed(&core),
                    targets
                        .iter()
                        .take(maximum)
                        .map(|id| (*id, amount))
                        .collect::<Vec<_>>()
                );
                assert_eq!(core.runtime_fact_summary().damage_hp_lost, 0);
                assert_eq!(core.runtime_fact_view().active_actions().count(), 0);
            }
        }
    }
}
