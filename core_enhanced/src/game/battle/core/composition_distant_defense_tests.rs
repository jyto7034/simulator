mod composition_distant_defense {
    use super::*;
    use crate::game::{
        battle::{
            core::movement::types::{ActiveMovementSegment, WorldVec2},
            damage::{DamageModifiers, DamageSourceSnapshot},
        },
        behavior::GameError,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/distant_defense.skill.yaml"
    ));
    fn setup(percent: u32) -> Fixture {
        let source = SOURCE
            .replace("porccubus_ecstasy_coat", "r11_weapon")
            .replace("value: 20,", &format!("value: {percent},"));
        setup_source(&source)
    }
    fn setup_source(source: &str) -> Fixture {
        let mut f = fixture_extra_with_defense(
            "Damage(amount:100,damage_type:True)",
            false,
            None,
            false,
            source,
            |_| {},
            |_| {},
        );
        for u in f.core.units.values_mut() {
            u.stats.current_health = 10000;
            u.stats.max_health = 10000;
        }
        place(&mut f.core, f.caster, 0.5, 0.5);
        place(&mut f.core, f.target, 3.5, 0.5);
        f
    }
    fn place(core: &mut BattleCore, id: UnitInstanceId, x: f32, y: f32) {
        core.active_movement_segments.remove(&id);
        core.units
            .get_mut(&id)
            .unwrap()
            .set_world_position(WorldVec2::new(x, y));
    }
    fn snapshot(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        source: DamageSource,
        kind: DamageType,
        now: u64,
    ) -> DamageSourceSnapshot {
        core.damage_source_snapshot_for_unit(
            actor,
            target,
            source,
            kind,
            100,
            DamageModifiers::default(),
            0,
            now,
            false,
        )
        .unwrap()
    }
    fn apply(
        core: &mut BattleCore,
        target: UnitInstanceId,
        snap: DamageSourceSnapshot,
        now: u64,
    ) -> Result<u32, GameError> {
        let before = core.units[&target].stats.current_health;
        core.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id: target,
                source_snapshot: snap,
            }],
            now,
        );
        crate::game::battle::runtime_control::poll()?;
        Ok(before - core.units[&target].stats.current_health)
    }
    fn hit(f: &mut Fixture, source: DamageSource, kind: DamageType, now: u64) -> u32 {
        f.core
            .runtime_transaction(|core| {
                let snap = snapshot(core, f.target, f.caster, source, kind, now);
                apply(core, f.caster, snap, now)
            })
            .unwrap()
    }
    fn observed(
        core: &BattleCore,
    ) -> &crate::game::battle::core::equipment_defense::IncomingConditionalWitness {
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_conditional_reduction
            .as_ref()
            .unwrap()
    }
    #[test]
    fn distant_defense_strict_continuous_boundary_and_authored_rate() {
        for (percent, reduced) in [(20, 80), (35, 65)] {
            let mut f = setup(percent);
            for (x, expected) in [(2.4999, 100), (2.5, 100), (2.5001, reduced), (3.5, reduced)] {
                place(&mut f.core, f.target, x, 0.5);
                assert_eq!(
                    hit(&mut f, DamageSource::BasicAttack, DamageType::True, 1),
                    expected
                );
                assert_eq!(
                    observed(&f.core).evaluation.percent,
                    if expected == 100 { 0 } else { percent }
                );
            }
            let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
            assert!(Artifact::from_ron(&artifact.to_ron().unwrap()).is_ok());
        }
    }
    #[test]
    fn distant_defense_samples_current_motion_not_stored_endpoint() {
        let mut f = setup(20);
        place(&mut f.core, f.target, 4.5, 0.5);
        f.core.active_movement_segments.insert(
            f.target,
            ActiveMovementSegment {
                start: WorldVec2::new(1.5, 0.5),
                target: WorldVec2::new(4.5, 0.5),
                started_at_ms: 10,
                ends_at_ms: 310,
            },
        );
        assert_eq!(
            hit(&mut f, DamageSource::Ability, DamageType::Magic, 10),
            100
        );
        assert_eq!(
            hit(&mut f, DamageSource::Ability, DamageType::Magic, 110),
            100
        );
        assert_eq!(
            hit(&mut f, DamageSource::Ability, DamageType::Magic, 111),
            80
        );
        assert_eq!(
            hit(&mut f, DamageSource::Ability, DamageType::Magic, 310),
            80
        );
    }
    #[test]
    fn distant_defense_freezes_same_actor_strike_then_rechecks_next_strike() {
        let mut f = setup(20);
        f.core
            .runtime_transaction(|core| {
                let prior = core.begin_strike_scope();
                let snap = snapshot(
                    core,
                    f.target,
                    f.caster,
                    DamageSource::BasicAttack,
                    DamageType::True,
                    1,
                );
                assert_eq!(apply(core, f.caster, snap.clone(), 1)?, 80);
                place(core, f.target, 1.5, 0.5);
                assert_eq!(apply(core, f.caster, snap, 1)?, 80);
                core.end_strike_scope(prior);
                Ok(())
            })
            .unwrap();
        assert_eq!(
            hit(&mut f, DamageSource::BasicAttack, DamageType::True, 2),
            100
        );
    }

    #[test]
    fn distant_defense_other_actors_do_not_reuse_or_replace_primary_strike_value() {
        let mut f = setup(20);
        let other_id = Uuid::from_u128(0xd188).into();
        let mut other = f.core.units[&f.target].clone();
        other.instance_id = other_id;
        other.spawn_order += 1;
        f.core.units.insert(other_id, other);
        place(&mut f.core, other_id, 1.5, 0.5);
        f.core
            .runtime_transaction(|core| {
                let old = core.begin_strike_scope();
                for (actor, expected) in [
                    (f.target, 80),
                    (other_id, 100),
                    (f.ally, 100),
                    (f.target, 80),
                ] {
                    let snap = snapshot(
                        core,
                        actor,
                        f.caster,
                        DamageSource::Ability,
                        DamageType::True,
                        1,
                    );
                    assert_eq!(apply(core, f.caster, snap, 1)?, expected);
                    if actor == f.target {
                        place(core, f.target, 1.5, 0.5);
                    }
                }
                core.end_strike_scope(old);
                Ok(())
            })
            .unwrap();
        assert_eq!(hit(&mut f, DamageSource::Ability, DamageType::True, 2), 100);
    }

    #[test]
    fn distant_defense_normal_sources_types_and_environment_boundary() {
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            for source in [
                DamageSource::BasicAttack,
                DamageSource::Ability,
                DamageSource::BuffTick,
            ] {
                let mut f = setup(20);
                let wearer = f.core.units.get_mut(&f.caster).unwrap();
                wearer.stats.defense = 0;
                wearer.stats.magic_resist = 0;
                assert_eq!(hit(&mut f, source, kind, 1), 80);
                assert_eq!(observed(&f.core).evaluation.actor, f.target);
                assert_eq!(observed(&f.core).evaluation.wearer, f.caster);
            }
        }
        for reaction in [false, true] {
            let mut f = setup(20);
            let mut snap = snapshot(
                &mut f.core,
                f.target,
                f.caster,
                DamageSource::Environment,
                DamageType::True,
                1,
            );
            snap.fact_source.reaction = reaction;
            assert_eq!(
                f.core
                    .runtime_transaction(|c| apply(c, f.caster, snap, 1))
                    .unwrap(),
                if reaction { 80 } else { 100 }
            );
        }
    }

    #[test]
    fn distant_defense_rejects_self_ally_and_no_longer_living_source() {
        for actor_kind in 0..4 {
            let mut f = setup(20);
            let actor = match actor_kind {
                0 => f.caster,
                1 => f.ally,
                _ => f.target,
            };
            place(&mut f.core, actor, 3.5, 0.5);
            let snap = snapshot(
                &mut f.core,
                actor,
                f.caster,
                DamageSource::Ability,
                DamageType::True,
                1,
            );
            if actor_kind == 2 {
                f.core.units.get_mut(&actor).unwrap().stats.current_health = 0;
            } else if actor_kind == 3 {
                f.core.units.remove(&actor);
            }
            assert_eq!(
                f.core
                    .runtime_transaction(|c| apply(c, f.caster, snap, 1))
                    .unwrap(),
                100
            );
            assert!(f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_conditional_reduction
                .is_none());
        }
    }

    fn state(c: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
            serde_json::to_value(&c.authoring_observation).unwrap(),
        )
    }

    #[test]
    fn distant_defense_cancel_after_evaluation_and_damage_restores_then_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "conditional_reduction_evaluated",
            "distant_defense_test_after_damage",
        ] {
            let mut f = setup(20);
            let before = state(&f.core);
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || {
                        f.core.runtime_transaction(|c| {
                            let old = c.begin_strike_scope();
                            let snap = snapshot(
                                c,
                                f.target,
                                f.caster,
                                DamageSource::BasicAttack,
                                DamageType::True,
                                1,
                            );
                            let damage = apply(c, f.caster, snap, 1)?;
                            if !token.is_cancelled() {
                                assert_eq!(damage, 80);
                            }
                            control::poll("distant_defense_test_after_damage")?;
                            c.end_strike_scope(old);
                            Ok(damage)
                        })
                    },
                )
            });
            assert!(
                result.is_err() && token.is_cancelled(),
                "{site}: {result:?}"
            );
            assert_eq!(state(&f.core), before);
            assert_eq!(
                hit(&mut f, DamageSource::BasicAttack, DamageType::True, 1),
                80
            );
        }
    }

    #[test]
    fn distant_defense_retained_refusal_restores_observation_damage_and_strike() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(20);
        let mut probe = f.core.clone();
        let action = |c: &mut BattleCore| {
            let old = c.begin_strike_scope();
            let snap = snapshot(
                c,
                f.target,
                f.caster,
                DamageSource::BasicAttack,
                DamageType::True,
                1,
            );
            let damage = apply(c, f.caster, snap, 1)?;
            c.end_strike_scope(old);
            Ok(damage)
        };
        let (result, admissions) =
            native::observe_retained_admissions(|| probe.runtime_transaction(action));
        assert_eq!(result.unwrap(), 80);
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "conditional_reduction_observation")
            .unwrap()
            .2;
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = state(&f.core);
        assert!(f.core.runtime_transaction(action).is_err());
        assert_eq!(state(&f.core), before);
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        assert_eq!(f.core.runtime_transaction(action).unwrap(), 80);
    }

    #[test]
    fn distant_defense_observation_disabled_clone_preserves_authoritative_results() {
        let mut f = setup(35);
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        for (now, x, expected) in [(1, 3.5, 65), (2, 1.5, 100), (3, 4.5, 65)] {
            place(&mut f.core, f.target, x, 0.5);
            place(&mut plain, f.target, x, 0.5);
            assert_eq!(
                hit(&mut f, DamageSource::Ability, DamageType::True, now),
                expected
            );
            let snap = snapshot(
                &mut plain,
                f.target,
                f.caster,
                DamageSource::Ability,
                DamageType::True,
                now,
            );
            assert_eq!(
                plain
                    .runtime_transaction(|c| apply(c, f.caster, snap, now))
                    .unwrap(),
                expected
            );
            assert_eq!(
                f.core.transaction_debug_snapshot(),
                plain.transaction_debug_snapshot()
            );
            assert_eq!(
                f.core.event_queue_debug_snapshot(),
                plain.event_queue_debug_snapshot()
            );
            assert_eq!(
                serde_json::to_value(f.core.runtime_fact_summary()).unwrap(),
                serde_json::to_value(plain.runtime_fact_summary()).unwrap()
            );
        }
    }

    #[test]
    fn distant_defense_unmatched_condition_skips_faulting_amount_and_matched_rolls_back() {
        let source = SOURCE.replace("porccubus_ecstasy_coat", "r11_weapon")
            .replace("{ Const: { value: 20, unit: Scalar } }",
                "{ Ratio: { numerator: { Const: { value: 20, unit: Scalar } }, denominator: { Ratio: { numerator: { Read: { HitPoints: { subject: Source, field: Missing } } }, denominator: { Const: { value: 1, unit: Magnitude } }, rounding: Floor } }, rounding: Floor } }");
        let mut f = setup_source(&source);
        place(&mut f.core, f.target, 1.5, 0.5);
        assert_eq!(hit(&mut f, DamageSource::Ability, DamageType::True, 1), 100);
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 10000;
        place(&mut f.core, f.target, 3.5, 0.5);
        let before = state(&f.core);
        let result = f.core.runtime_transaction(|c| {
            let snap = snapshot(
                c,
                f.target,
                f.caster,
                DamageSource::Ability,
                DamageType::True,
                2,
            );
            apply(c, f.caster, snap, 2)
        });
        assert!(result.is_err());
        assert_eq!(state(&f.core), before);
    }

    #[test]
    fn distant_defense_mixed_actor_components_and_guard_credit_stay_separate() {
        use crate::game::{
            battle::{
                cooldown::{CooldownSource, SourcedEffect},
                damage::{
                    calculate_damage_with_defenses, reduction_credit_with_defenses, DamageContext,
                    DamageRequest,
                },
            },
            stats::{Effect, TriggerEffectTarget},
        };
        let request = DamageRequest {
            source: DamageSource::BasicAttack,
            damage_type: DamageType::Physical,
            modifiers: Default::default(),
            crit_roll_percent: None,
            attacker_id: Uuid::from_u128(10).into(),
            target_id: Uuid::from_u128(20).into(),
            base_damage: 100,
            minimum_damage: 1,
            time_ms: 0,
        };
        let bonus = |actor, damage_type| SourcedEffect {
            source: CooldownSource::Unit {
                unit_instance_id: actor,
            },
            equipment_source: None,
            target: TriggerEffectTarget::CounterpartUnit,
            effect: Effect::BonusDamage {
                flat: 100,
                percent: 0,
                damage_type,
            },
        };
        let attack = [bonus(request.attacker_id, DamageType::Magic)];
        let on_hit = [
            bonus(request.target_id, DamageType::True),
            bonus(Uuid::from_u128(30).into(), DamageType::Physical),
        ];
        let ctx = DamageContext {
            attacker_side: Side::Opponent,
            target_side: Side::Player,
            attacker_attack: 100,
            target_armor: 0,
            target_magic_resist: 0,
            target_incoming_modifiers: DamageModifiers {
                damage_reduction_percent: 30,
                ..Default::default()
            },
            target_current_hp: 1000,
            target_max_hp: 1000,
            on_attack_effects: &attack,
            on_hit_effects: &on_hit,
        };
        let mut flat = 10;
        let result = calculate_damage_with_defenses(&request, &ctx, &mut flat, 20);
        assert_eq!(flat, 0);
        assert_eq!(
            result
                .breakdown
                .iter()
                .map(|b| b.final_damage)
                .collect::<Vec<_>>(),
            [45, 50, 70, 70]
        );
        assert_eq!(result.final_damage, 235);
        // Removing the guard's30% leaves distance20% on only the first two components:
        // 72 + 80 + 100 + 100 = 352; the guard alone prevented117.
        assert_eq!(
            reduction_credit_with_defenses(
                &request,
                &ctx,
                &result,
                DamageModifiers::default(),
                10,
                20
            ),
            117
        );
    }
    #[test]
    fn distant_defense_two_actual_copies_choose_strongest_and_new_strike_falls_back() {
        check_copies(35);
    }
    #[test]
    fn distant_defense_equal_copies_choose_lowest_runtime_uuid() {
        check_copies(20);
    }
    fn check_copies(second_percent: u32) {
        let catalog: crate::game::data::equipment_data::EquipmentDatabase =
            ron::de::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../game_resources/data/equipments/base.ron"
            )))
            .unwrap();
        let mut weapon = catalog
            .get_by_id("standard_training_sword")
            .unwrap()
            .clone();
        weapon.id = "r11_weapon".into();
        weapon.uuid = Uuid::from_u128(0x1101);
        weapon.triggered_effects.clear();
        weapon.ability_activations.clear();
        let mut armor = catalog.get_by_id("porccubus_ecstasy_coat").unwrap().clone();
        armor.id = "distance_armor".into();
        armor.triggered_effects.clear();
        let armor_id = armor.uuid;
        let mut doc = parse_yaml(&SOURCE.replace("porccubus_ecstasy_coat", "r11_weapon")).unwrap();
        let mut stronger = doc["values"][0].clone();
        stronger["name"] = json!("stronger");
        stronger["value"]["DefenseProgram"]["ConditionalReduction"]["percent"]["root"]["Const"]
            ["value"] = json!(second_percent);
        doc["values"].as_array_mut().unwrap().push(stronger);
        doc["equipment_defenses"]
            .as_array_mut()
            .unwrap()
            .push(json!({"equipment":"distance_armor","program":{"Shared":"stronger"}}));
        let a = compile_json(&doc.to_string()).unwrap();
        let (mut core, state, caster, ally) = r12_fragment_equipped_effect_core_with_data(
            &r12_control_ron("Damage(amount:100,damage_type:True)"),
            |scenario| {
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    let mut second = spawn.draft.equipped_items[0];
                    second.base_uuid = armor_id;
                    second.persistent_copy_uuid = Some(Uuid::from_u128(0xe880 + i as u128));
                    spawn.draft.equipped_items.push(second);
                }
            },
            |_| {},
            None,
            |b| {
                b.with_equipment(vec![weapon, armor])
                    .with_equipment_defenses(a.equipment_defenses().to_vec())
            },
        );
        let target = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for u in core.units.values_mut() {
            u.stats.defense = 20;
            u.stats.max_health = 10000;
            u.stats.current_health = 10000;
        }
        core.enable_authoring_observation();
        let mut f = Fixture {
            core,
            state,
            caster,
            target,
            ally,
        };
        place(&mut f.core, f.caster, 0.5, 0.5);
        place(&mut f.core, f.ally, 0.5, 1.5);
        place(&mut f.core, f.target, 3.5, 0.5);
        assert_eq!(
            hit(&mut f, DamageSource::BasicAttack, DamageType::True, 1),
            100 - second_percent
        );
        let chosen = observed(&f.core).equipment;
        if second_percent == 20 {
            let least = observed(&f.core)
                .evaluation
                .contributors
                .iter()
                .map(|entry| entry.equipment.runtime_item_instance_id)
                .min()
                .unwrap();
            assert_eq!(chosen.runtime_item_instance_id, least);
            return;
        }
        assert_eq!(chosen.definition_uuid, armor_id);
        let s = snapshot(
            &mut f.core,
            f.target,
            f.ally,
            DamageSource::BasicAttack,
            DamageType::True,
            2,
        );
        assert_eq!(
            f.core
                .runtime_transaction(|c| apply(c, f.ally, s, 2))
                .unwrap(),
            65
        );
        assert_ne!(
            chosen.persistent_copy_uuid,
            observed(&f.core).equipment.persistent_copy_uuid
        );
        f.core.items.remove(&chosen.runtime_item_instance_id);
        assert_eq!(
            hit(&mut f, DamageSource::BasicAttack, DamageType::True, 3),
            80
        );
        assert_ne!(observed(&f.core).equipment.definition_uuid, armor_id);
    }

    #[test]
    fn distant_defense_common_hp_condition_reads_wearer_and_current_attacker() {
        let mut doc = parse_yaml(&SOURCE.replace("porccubus_ecstasy_coat", "r11_weapon")).unwrap();
        let amount = |subject: &str| {
            json!({"moment":"Live","root":{
                "Read":{"HitPoints":{"subject":subject,"field":"Current"}}
            }})
        };
        doc["values"][0]["value"]["DefenseProgram"]["ConditionalReduction"]["condition"] = json!({"Compare":{"left":amount("Source"),"comparison":"Less","right":amount("Target")}});
        let mut f = setup_source(&doc.to_string());
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 1000;
        f.core
            .units
            .get_mut(&f.target)
            .unwrap()
            .stats
            .current_health = 2000;
        assert_eq!(hit(&mut f, DamageSource::Ability, DamageType::True, 1), 80);
        assert_eq!(
            (
                observed(&f.core).evaluation.wearer_hp,
                observed(&f.core).evaluation.target_hp
            ),
            (1000, 2000)
        );
        assert!(observed(&f.core).evaluation.distance_squared.is_none());
        f.core
            .units
            .get_mut(&f.target)
            .unwrap()
            .stats
            .current_health = 900;
        assert_eq!(hit(&mut f, DamageSource::Ability, DamageType::True, 2), 100);
    }
}
