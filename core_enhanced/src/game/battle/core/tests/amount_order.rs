//! C07 runtime order and real projectile event queue, including empty completion.
use super::*;
use crate::game::ability::{ProjectileHitPolicy, SkillProjectileCollisionDef};
use crate::game::battle::core::{
    amount_observation::RecordedAmountEvaluation, authoring_observation::ObservationKey,
    commands::amount_command_observation, value_expr_runtime::RuntimeAmount,
};

fn observe(core: &mut BattleCore) {
    core.enable_authoring_observation_with_options(Default::default(), true);
    core.bind_authoring_observation(ObservationKey {
        entry_ordinal: 17,
        battle_uuid: Uuid::from_u128(0xf0307),
    })
    .unwrap();
}
fn record(core: &BattleCore) -> Arc<RecordedAmountEvaluation> {
    let state = core.authoring_observation.as_ref().unwrap();
    let record = state.latest_amount.as_ref().unwrap();
    assert_eq!(state.amount_evaluations, record.evaluation_ordinal);
    Arc::clone(record)
}

#[test]
fn d07_f03_c07_multieffect_multitarget_same_timestamp_has_execution_order_ordinals() {
    let skill = single_step_skill(
        "c07_order",
        SkillTarget::SelfUnit,
        0,
        0,
        DeliveryDef::Instant,
        vec![
            SkillEffectDef::Damage {
                amount: 2,
                damage_type: DamageType::True,
            },
            SkillEffectDef::Heal { amount: 1 },
            SkillEffectDef::Damage {
                amount: 0,
                damage_type: DamageType::True,
            },
        ],
    );
    let (mut core, _, caster, target, ally) =
        native_action_fixture(skill.clone(), Default::default());
    // Use compiled catalog sources, not the pre-publication SkillDef copy.
    let skill = core
        .game_data
        .skill_data
        .get_by_id(&skill.id)
        .unwrap()
        .clone();
    observe(&mut core);
    let original = skill.steps[0].clone();
    let mut expected_ordinal = 0;
    let mut previous = None;
    for targets in [[target, caster, ally], [ally, target, caster]] {
        for (index, amount) in [
            RuntimeAmount::Damage(2),
            RuntimeAmount::Heal(1),
            RuntimeAmount::Damage(0),
        ]
        .into_iter()
        .enumerate()
        {
            // Full prefixes preserve actual authored effect indices and expose
            // boundaries without synthesizing records or changing timestamps.
            let mut step = original.clone();
            step.effects.truncate(index + 1);
            let (result, commands) = amount_command_observation::observe(|| {
                core.runtime_transaction(|c| {
                    c.execute_ordered_skill_effects(
                        caster, &skill.id, &step, &targets, 10, 10, None, None,
                    )
                })
            });
            result.unwrap();
            expected_ordinal += 3 * (index as u64 + 1);
            let latest = record(&core);
            assert_eq!(latest.evaluation_ordinal, expected_ordinal);
            assert_eq!(latest.effect_index, index);
            assert_eq!(latest.site().effect_id, original.effects[index].id);
            assert_eq!(latest.consumption_target, Some(targets[2]));
            assert_eq!(latest.consume_time_ms, 10);
            assert_eq!(latest.amount, amount);
            let mut expected_commands = targets.map(|id| (id, RuntimeAmount::Damage(2))).to_vec();
            if index >= 1 {
                expected_commands.extend(targets.map(|id| (id, RuntimeAmount::Heal(1))));
            }
            assert_eq!(commands, expected_commands);
            if let Some(old) = previous {
                assert!(!Arc::ptr_eq(&latest, &old));
            }
            eprintln!(
                "C07_ORDER ordinal={} effect={} target={} amount={:?} time=10",
                expected_ordinal, index, targets[2], amount
            );
            previous = Some(latest);
        }
    }
    // Two passes each do Damage three times and Heal twice: net -8.
    for id in [caster, target, ally] {
        assert_eq!(core.units[&id].stats.current_health, 92);
    }
    assert_eq!(expected_ordinal, 36);
}

#[test]
fn d07_f03_c07_piercing_hits_replace_zero_record_but_empty_expiry_preserves_it() {
    for (amount, stop_after_two) in [(0, false), (2, false), (0, true), (2, true)] {
        let skill = single_step_skill(
            "c07_pierce",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            5,
            0,
            DeliveryDef::Projectile {
                speed_units_per_ms: 10_000,
                hit_policy: ProjectileHitPolicy::DirectionalCollision,
                allow_targetless_cast: false,
                max_range_tiles: Some(5),
                max_lifetime_ms: Some(500),
                max_kills: None,
                max_pierces: stop_after_two.then_some(1),
                collision: SkillProjectileCollisionDef {
                    radius_units: 100_000,
                    piercing: true,
                    ..Default::default()
                },
            },
            vec![SkillEffectDef::Damage {
                amount,
                damage_type: DamageType::True,
            }],
        );
        let (mut core, _, caster, target, ally) =
            native_action_fixture(skill.clone(), Default::default());
        core.units.get_mut(&ally).unwrap().owner = Side::Opponent;
        place_unit(&mut core, ally, Position::new(3, 1));
        sync_unit_to_battlefield_tile_center(&mut core, ally);
        // Fixture units remain tile-fixed while only projectile queue advances.
        observe(&mut core);
        core.invoke_ability(
            0,
            caster,
            &skill.id,
            Some(SkillCastTarget::Unit {
                unit_instance_id: target,
            }),
            Default::default(),
            false,
        );
        let mut hits = Vec::new();
        let mut empty = 0;
        let mut last = None;
        let mut action: Option<crate::game::battle::runtime_facts::ActionHandle> = None;
        let mut delivery = None;
        // At most one reevaluation per millisecond, plus release/hit/completion events.
        for _ in 0..(500 + 16) {
            let Some(event) = core.event_queue.pop() else {
                break;
            };
            let impact = match event.event.view().as_ref() {
                BattleEvent::SkillProjectileImpact {
                    first_hit_unit_id,
                    terminal,
                    source_snapshot,
                    ..
                } => {
                    action = Some(source_snapshot.fact_source.action.clone().unwrap());
                    delivery = Some(
                        source_snapshot
                            .fact_source
                            .delivery
                            .as_ref()
                            .unwrap()
                            .ordinal,
                    );
                    Some((*first_hit_unit_id, *terminal))
                }
                _ => None,
            };
            let time = event.time_ms();
            assert!(time <= 500, "unexpected event beyond projectile lifetime");
            let (result, commands) =
                amount_command_observation::observe(|| core.process_event(event, time));
            result.unwrap();
            if let Some((Some(id), _)) = impact {
                hits.push(id);
                let r = record(&core);
                assert_eq!(r.evaluation_ordinal, hits.len() as u64);
                assert_eq!(r.amount, RuntimeAmount::Damage(amount as u32));
                assert_eq!(r.consumption_target, Some(id));
                assert_eq!(r.consume_time_ms, time);
                assert_eq!(
                    r.source.delivery.as_ref().unwrap().ordinal,
                    delivery.unwrap()
                );
                assert_eq!(
                    commands,
                    if amount == 0 {
                        vec![]
                    } else {
                        vec![(id, RuntimeAmount::Damage(amount as u32))]
                    }
                );
                if let Some(old) = last {
                    assert!(!Arc::ptr_eq(&r, &old));
                }
                last = Some(r);
            } else if let Some((None, terminal)) = impact {
                assert!(terminal);
                empty += 1;
                assert!(commands.is_empty());
                assert!(Arc::ptr_eq(&record(&core), last.as_ref().unwrap()));
                assert_eq!(record(&core).evaluation_ordinal, 2);
            }
        }
        assert_eq!(hits, vec![target, ally]);
        assert_eq!(
            empty, 0,
            "a pierced carrier expires without an additional empty impact"
        );
        assert!(Arc::ptr_eq(&record(&core), last.as_ref().unwrap()));
        assert_eq!(record(&core).evaluation_ordinal, 2);
        assert!(core.event_queue.is_empty());
        assert!(core.active_projectiles.is_empty());
        let snapshot = core.runtime_fact_snapshot().unwrap();
        let settled = snapshot.latest_settled_action.as_ref().unwrap();
        assert_eq!(settled.ordinal, action.as_ref().unwrap().ordinal());
        let action = settled;
        assert_eq!(action.outstanding_deliveries, 0);
        assert_eq!(action.pending_control_steps, 0);
        assert_eq!(
            action.facts.damage_mutations,
            if amount == 0 { 0 } else { 2 }
        );
        assert_eq!(action.facts.damage_hp_lost, 2 * amount as u64);
        // C09 closes C07-OBS01 without another impact or amount evaluation.
        assert!(core.active_skill_casts.is_empty());
        assert!(action.effects_settled_at_ms.is_some());
        for id in [target, ally] {
            assert_eq!(core.units[&id].stats.current_health, 100 - amount as u32);
        }
        eprintln!(
            "C07_PROJECTILE {}",
            serde_json::json!({
                "amount": amount, "stop": if stop_after_two { "hit_limit" } else { "expiry" },
                "hits": 2, "empty_impacts": 0, "ordinal": 2,
                "open_deliveries": action.outstanding_deliveries,
                "pending_casts": core.active_skill_casts.len(),
                "effects_settled_at_ms": action.effects_settled_at_ms,
            })
        );
    }
}

#[test]
fn d07_f03_c07_d04_f01_empty_dynamic_impact_keeps_record_and_runs_following_resource() {
    // This predecessor fixture releases real DamageExpr/HealExpr, removes its
    // target and queues the empty impact. Enable observation before consuming it.
    let (mut core, caster, resource, source) = d04_f01_resource_projectile(false);
    observe(&mut core);
    let action = source.fact_source.action.as_ref().unwrap().clone();
    let before_count = core
        .authoring_observation
        .as_ref()
        .unwrap()
        .amount_evaluations;
    for _ in 0..16 {
        let Some(event) = core.event_queue.pop() else {
            break;
        };
        let time = event.time_ms();
        let (result, commands) =
            amount_command_observation::observe(|| core.process_event(event, time));
        result.unwrap();
        assert!(commands.is_empty());
    }
    let observer = core.authoring_observation.as_ref().unwrap();
    assert_eq!(observer.amount_evaluations, before_count);
    assert!(observer.latest_amount.is_none());
    assert_eq!(d04_f01_source_balance(&core, caster, &resource), 3);
    assert!(core.active_projectiles.is_empty());
    assert!(core.active_skill_casts.is_empty());
    assert!(core.event_queue.is_empty());
    let snapshot = core.runtime_fact_snapshot().unwrap();
    let settled = snapshot.latest_settled_action.as_ref().unwrap();
    assert_eq!(settled.ordinal, action.ordinal());
    assert_eq!(settled.outstanding_deliveries, 0);
    assert_eq!(settled.pending_control_steps, 0);
    assert!(settled.effects_settled_at_ms.is_some());
    eprintln!("C07_EMPTY_DYNAMIC ordinal={before_count} record=none resource=3 settled=true");
}

#[test]
fn r06_finite_many_cast_repeated_area_observation_cost_and_cleanup() {
    use crate::authoring::{
        clone_shape::CloneShape,
        control::{self, CancellationToken, CloneMeter, ExecutionLimits, ExecutionPhase},
    };
    use crate::game::value_expr::{
        AmountExpr, EvalMoment, StatSelector, ValueExpr, ValueRef, ValueSubject,
    };
    for casts in [1u64, 8, 32] {
        let mut oracle = None;
        for enabled in [false, true] {
            let mut skill = enemy_tile_area_skill(
                "r06_finite_area",
                vec![SkillEffectDef::DamageExpr {
                    amount: AmountExpr::authored(
                        EvalMoment::Live,
                        ValueExpr::Read(ValueRef::Stat {
                            subject: ValueSubject::Source,
                            stat: StatSelector::Attack,
                        }),
                    ),
                    damage_type: DamageType::True,
                }],
            );
            let DeliveryDef::TileArea { area, .. } = &mut skill.steps[0].delivery else {
                unreachable!()
            };
            area.duration_ms = 2;
            area.tick_interval_ms = Some(1);
            let (mut core, mut execution, a, target, b) =
                native_action_fixture(skill.clone(), Default::default());
            place_unit(&mut core, b, Position::new(1, 2));
            sync_unit_to_battlefield_tile_center(&mut core, b);
            core.units.get_mut(&a).unwrap().stats.attack = 1;
            core.units.get_mut(&b).unwrap().stats.attack = 3;
            core.units.get_mut(&target).unwrap().stats.max_health = 10_000;
            core.units.get_mut(&target).unwrap().stats.current_health = 10_000;
            if enabled {
                observe(&mut core);
            }
            let mut extents = Vec::new();
            let (result, usage) = control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || {
                    for index in 0..casts {
                        let actor = if index % 2 == 0 { a } else { b };
                        let start = index * 4;
                        core.units.get_mut(&actor).unwrap().resonance_current = 100;
                        let handle =
                            core.request_skill_action(start, actor, skill.id.clone(), None)?;
                        core.step_battle_execution_until(&mut execution, start + 3)?;
                        let settled = core.take_settled_action_result(&handle).unwrap();
                        assert_eq!(settled.action.facts.damage_strikes, 2);
                        assert!(core.active_areas.is_empty());
                        assert!(core.active_skill_casts.is_empty());
                        if enabled {
                            let latest = record(&core);
                            assert_eq!(latest.evaluation_ordinal, (index + 1) * 2);
                            assert_eq!(
                                latest.amount,
                                RuntimeAmount::Damage(if actor == a { 1 } else { 3 })
                            );
                            extents.push(latest.retained_extent());
                        }
                    }
                    Ok::<_, GameError>(())
                },
            )
            .unwrap();
            result.unwrap();
            let hp = core.units[&target].stats.current_health;
            let expected_damage = (casts.div_ceil(2) * 2 + (casts / 2) * 6) as u32;
            assert_eq!(hp, 10_000 - expected_damage);
            let facts = serde_json::to_string(&core.runtime_fact_view()).unwrap();
            if let Some((old_hp, old_facts)) = oracle.as_ref() {
                assert_eq!((*old_hp, old_facts), (hp, &facts));
            } else {
                oracle = Some((hp, facts));
            }
            let (_, cloned) = control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::Inspection,
                || {
                    let mut meter = CloneMeter::default();
                    core.visit_clone_shape(&mut meter).unwrap();
                    meter
                        .finish("r06_many_cast_retained", "r06_many_cast_clone")
                        .unwrap();
                    core.clone()
                },
            )
            .unwrap();
            if enabled {
                assert!(extents.iter().all(|extent| *extent == extents[0]));
            }
            eprintln!("R06_FINITE casts={casts} area_ticks={} observation={enabled} hp={hp} work={} retained_record={:?} clone_slots={} metadata={}", casts*2, usage.work, extents.last(), cloned.clone_slots_peak, cloned.metadata_bytes_peak);
        }
    }
}

#[cfg(feature = "runtime-observation")]
#[test]
fn runtime_observation_captures_real_projectile_peak_after_same_attempt_cleanup() {
    use crate::{application::ApplicationError, runtime_observation as observation};
    let (sender, receiver) = observation::channel();
    let (mut observer, reader) = sender.session(91);
    let guard = observer.begin(observation::Operation::HostTick, 0).unwrap();
    // Existing native fixture releases real expression projectiles, removes the
    // target and queues empty impacts. All insertion/cleanup occurs in capture.
    let (mut core, caster, resource, _) = d04_f01_resource_projectile(false);
    for _ in 0..16 {
        let Some(event) = core.event_queue.pop() else {
            break;
        };
        let time = event.time_ms();
        core.process_event(event, time).unwrap();
    }
    assert_eq!(d04_f01_source_balance(&core, caster, &resource), 3);
    assert!(core.active_projectiles.is_empty());
    assert!(core.active_skill_casts.is_empty());
    assert!(core.event_queue.is_empty());
    guard.finish(&Ok::<_, ApplicationError>(()));
    let summary = receiver.try_recv().unwrap();
    assert!(summary.peak.0[observation::PROJECTILES] > 0);
    assert!(summary.peak.0[observation::EVENTS] > 0);
    assert!(summary.peak.0[observation::ACTIONS] > 0);
    assert_eq!(
        summary.committed, None,
        "private fixture is not a public root commit"
    );
    assert_eq!(reader.snapshot(false).unwrap().commits, 0);
    assert!(receiver.try_recv().is_err());
}
