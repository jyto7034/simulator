mod composition_last_will {
    use super::*;
    use crate::game::battle::{event_log::BattleLogEvent, tile_range::FacingDirection};
    fn canonical_source() -> String {
        crate::authoring::composition::canonical_funeral_tests::equipment_fixture("armor")
    }
    fn setup_will() -> (Hunt, UnitInstanceId) {
        let mut f = setup_source_with(false, false, &canonical_source(), |s| {
            s.groups[0].enemy_movement_plan = None;
            s.groups[0].spawns[0].unit_ref = ScenarioUnitRef::new("will-front");
            let mut rear = s.groups[0].spawns[0].clone();
            rear.unit_ref = ScenarioUnitRef::new("will-rear");
            rear.instance_salt += 100;
            rear.position = crate::game::resources::Position::new(1, 3);
            s.groups[0].spawns.push(rear);
        });
        f.enemy = f.core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("will-front")];
        let rear = f.core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("will-rear")];
        for (id, x, y) in [(f.a, 2, 3), (f.b, 3, 2), (f.enemy, 4, 3), (rear, 1, 3)] {
            place(&mut f.core, id, x, y);
            let u = f.core.units.get_mut(&id).unwrap();
            u.stats.defense = 0;
            u.stats.magic_resist = 0;
        }
        let u = f.core.units.get_mut(&f.a).unwrap();
        u.stats.attack = 100;
        u.stats.current_health = 100;
        u.facing_direction = Some(FacingDirection::Right);
        assert!(BattleCore::is_employee_resonance_unit(u));
        assert!(u.is_active() && !u.corroded && u.skill_id.is_none());
        assert!(f.core.equipment_defense.last_will_used.is_empty());
        (f, rear)
    }
    #[test]
    fn composition_last_will_preflight_real_armor_lethal_floor_blast_and_incapacitation() {
        let (mut f, rear) = setup_will();
        hit(&mut f.core, f.enemy, f.a, 100, 0);
        assert!(f.core.units[&f.a].lifecycle.is_dead());
        assert_eq!(f.core.units[&f.a].stats.current_health, 0);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9700);
        assert_eq!(f.core.units[&rear].stats.current_health, 10000);
        assert_eq!(f.core.units[&f.b].stats.current_health, 10000);
        let w = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_last_will
            .as_ref()
            .unwrap();
        assert_eq!(
            (
                w.hp_before,
                w.hp_protected,
                w.hp_after,
                w.attack,
                w.base_damage,
                w.targets
            ),
            (100, 1, 0, 100, 300, 1)
        );
        assert!(w.incapacitated);
        assert!(w.equipment.persistent_copy_uuid.is_some());
        assert!(w.field_path.ends_with("program.LastWill"));
        assert_eq!(f.core.equipment_defense.last_will_used.len(), 1);
        assert!(f.core.equipment_defense.last_will_resolving.is_empty());
        let events = &f.core.event_log.entries;
        let floor=events.iter().position(|e|matches!(e.event,BattleLogEvent::HpChanged { target_instance_id,hp_after:1,.. } if target_instance_id==f.a)).unwrap();
        let blast=events.iter().position(|e|matches!(e.event,BattleLogEvent::HpChanged { target_instance_id,hp_after:9700,.. } if target_instance_id==f.enemy)).unwrap();
        let exit=events.iter().position(|e|matches!(e.event,BattleLogEvent::UnitDied { unit_instance_id,.. } if unit_instance_id==f.a)).unwrap();
        assert!(floor < blast && blast < exit);
    }
    #[test]
    fn composition_last_will_nonlethal_preserves_use_and_empty_front_still_exits() {
        let (mut f, _) = setup_will();
        hit(&mut f.core, f.enemy, f.a, 99, 0);
        assert_eq!(f.core.units[&f.a].stats.current_health, 1);
        assert!(f.core.equipment_defense.last_will_used.is_empty());
        f.core.units.get_mut(&f.a).unwrap().facing_direction = Some(FacingDirection::Up);
        hit(&mut f.core, f.enemy, f.a, 1, 1);
        assert!(f.core.units[&f.a].lifecycle.is_dead());
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
        let w = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_last_will
            .as_ref()
            .unwrap();
        assert_eq!(w.targets, 0);
        assert_eq!((w.hp_before, w.hp_protected, w.hp_after), (1, 1, 0));
    }
    fn raw_damage(f: &mut Hunt, amount: u32, now: u64) -> Result<(), GameError> {
        let (attacker, target) = (f.enemy, f.a);
        f.core.runtime_transaction(|core| {
            let source = core
                .damage_source_snapshot_for_unit(
                    attacker,
                    target,
                    crate::game::battle::damage::DamageSource::Ability,
                    crate::game::battle::damage::DamageType::True,
                    amount,
                    Default::default(),
                    0,
                    now,
                    false,
                )
                .ok_or(GameError::InvalidAction)?;
            core.process_commands(
                vec![crate::game::battle::damage::BattleCommand::ApplyDamage {
                    target_id: target,
                    source_snapshot: source,
                }],
                now,
            );
            crate::game::battle::runtime_control::poll()
        })
    }
    #[test]
    fn composition_last_will_schema_roundtrip_and_native_invalid_input() {
        use crate::authoring::composition::{compile_json, compile_yaml, parse_yaml, Artifact};
        let a = compile_yaml(&canonical_source()).unwrap();
        let b = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        assert!(matches!(
            b.equipment_defenses()[0].program,
            crate::game::ability::equipment_defense::EquipmentDefenseProgram::LastWill {
                attack_percent: 300,
                ..
            }
        ));
        let doc = parse_yaml(&canonical_source()).unwrap();
        for (key, value) in [
            ("attack_percent", serde_json::json!(0)),
            ("attack_percent", serde_json::json!(10001)),
            ("range", serde_json::json!({"rows":[]})),
            ("range", serde_json::json!({"rows":["XX",".@."]})),
            ("range", serde_json::json!({"rows":["@@X"]})),
            ("range", serde_json::json!({"rows":[".@."]})),
            (
                "range",
                serde_json::json!({"rows":[format!("@{}","X".repeat(33))]}),
            ),
        ] {
            let mut changed = doc.clone();
            changed["values"][0]["value"]["DefenseProgram"]["LastWill"][key] = value;
            let error = compile_json(&changed.to_string()).unwrap_err();
            assert!(format!("{error:?}").contains("Native"), "{error:?}");
            assert!(format!("{error:?}").contains(key), "{error:?}");
        }
    }
    #[test]
    fn composition_last_will_shield_and_existing_survival_precede_armor() {
        use crate::game::ability::sustained::{SurvivalPolicy, SustainedEffectDef};
        let (mut f, _) = setup_will();
        let actor = f.a;
        for effect in [
            SustainedEffectDef::Shield {
                duration_ms: 10000,
                amount: 100,
                priority: 0,
                shared_radius_units: None,
            },
            SustainedEffectDef::Survival {
                duration_ms: 100,
                policy: SurvivalPolicy::FloorOne,
            },
            SustainedEffectDef::Survival {
                duration_ms: 10000,
                policy: SurvivalPolicy::PreventDeathOnce,
            },
        ] {
            f.core
                .runtime_transaction(|c| c.begin_sustained(actor, actor, effect, 0))
                .unwrap();
        }
        raw_damage(&mut f, 100, 1).unwrap();
        assert_eq!(f.core.units[&actor].stats.current_health, 100);
        raw_damage(&mut f, 100, 2).unwrap();
        assert_eq!(f.core.units[&actor].stats.current_health, 1);
        assert!(f.core.equipment_defense.last_will_used.is_empty());
        advance(&mut f, 100);
        raw_damage(&mut f, 100, 101).unwrap();
        assert!(f.core.units[&actor].is_active());
        assert_eq!(f.core.units[&actor].stats.current_health, 1);
        assert!(f.core.equipment_defense.last_will_used.is_empty());
        raw_damage(&mut f, 1, 102).unwrap();
        assert!(f.core.units[&actor].lifecycle.is_dead());
        assert_eq!(f.core.equipment_defense.last_will_used.len(), 1);
    }
    #[test]
    fn composition_last_will_stunned_wearer_hits_air_resistance_and_cloned_branch_agrees() {
        use crate::game::ability::sustained::{HitStatusKind, HitStatusProfile};
        let (mut f, _) = setup_will();
        let (enemy, actor) = (f.enemy, f.a);
        let u = f.core.units.get_mut(&enemy).unwrap();
        u.mobility_kind = crate::game::battle::types::MobilityKind::Airborne;
        u.stats.magic_resist = 100;
        assert!(f
            .core
            .runtime_transaction(|c| c.begin_committed_hit_status(
                enemy,
                actor,
                Side::Opponent,
                1000,
                HitStatusProfile {
                    definition: "last-will-stun".into(),
                    kind: HitStatusKind::Stun,
                    modifier_source: None,
                    vulnerability: None
                },
                0
            ))
            .unwrap());
        assert!(f.core.hit_stun_until(actor, Some(1)).is_some());
        let mut other = Hunt {
            core: f.core.clone(),
            state: f.state.clone(),
            a: f.a,
            b: f.b,
            enemy: f.enemy,
        };
        raw_damage(&mut f, 100, 1).unwrap();
        raw_damage(&mut other, 100, 1).unwrap();
        assert_eq!(f.core.units[&enemy].stats.current_health, 9850);
        assert!(f.core.units[&actor].lifecycle.is_dead());
        assert_eq!(
            f.core.transaction_debug_snapshot(),
            other.core.transaction_debug_snapshot()
        );
        assert_eq!(
            format!("{:?}", f.core.equipment_defense),
            format!("{:?}", other.core.equipment_defense)
        );
        let hp = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_nonzero_sourced_hp
            .as_ref()
            .unwrap();
        assert!(hp
            .authored
            .as_ref()
            .unwrap()
            .field_path
            .ends_with("program.LastWill"));
    }
    #[test]
    fn composition_last_will_cancel_at_reserve_hit_and_exit_restores_then_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "last_will_reserved",
            "last_will_hit",
            "last_will_after_exit",
        ] {
            let (mut f, _) = setup_will();
            let before = (
                f.core.transaction_debug_snapshot(),
                format!("{:?}", f.core.equipment_defense),
            );
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || raw_damage(&mut f, 100, 0),
                )
            });
            assert!(result.is_err() && token.is_cancelled(), "{site}");
            assert_eq!(
                before,
                (
                    f.core.transaction_debug_snapshot(),
                    format!("{:?}", f.core.equipment_defense)
                ),
                "{site}"
            );
            raw_damage(&mut f, 100, 0).unwrap();
            assert!(f.core.units[&f.a].lifecycle.is_dead());
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 9700);
        }
    }
    #[test]
    fn composition_last_will_retained_limit_rolls_back_use_and_observer_is_optional() {
        use crate::game::battle::runtime_control as native;
        let (mut f, _) = setup_will();
        let mut probe = Hunt {
            core: f.core.clone(),
            state: f.state.clone(),
            a: f.a,
            b: f.b,
            enemy: f.enemy,
        };
        let (result, admissions) =
            native::observe_retained_admissions(|| raw_damage(&mut probe, 100, 0));
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(s, _, _)| *s == "last_will_observation")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = (
            f.core.transaction_debug_snapshot(),
            format!("{:?}", f.core.equipment_defense),
        );
        let error = raw_damage(&mut f, 100, 0).unwrap_err();
        assert!(
            format!("{error:?}").contains("RetainedByteLimit"),
            "{error:?}"
        );
        assert_eq!(
            before,
            (
                f.core.transaction_debug_snapshot(),
                format!("{:?}", f.core.equipment_defense)
            )
        );
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        f.core.authoring_observation = None;
        raw_damage(&mut f, 100, 0).unwrap();
        assert!(f.core.units[&f.a].lifecycle.is_dead());
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9700);
        assert_eq!(f.core.equipment_defense.last_will_used.len(), 1);
    }
    #[test]
    fn composition_last_will_spent_copy_survives_owner_cleanup_until_real_terminal() {
        let (mut f, _) = setup_will();
        raw_damage(&mut f, 100, 0).unwrap();
        let copy = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_last_will
            .as_ref()
            .unwrap()
            .equipment
            .persistent_copy_uuid
            .unwrap();
        assert!(f.core.equipment_defense.last_will_used.contains(&copy));
        f.core.equipment_defense.remove_owner(f.a);
        assert!(f.core.equipment_defense.last_will_used.contains(&copy));
        Arc::make_mut(&mut f.core.scenario).terminal_outcomes =
            BattleScenario::fixture_survival_outcomes(100);
        advance(&mut f, 100);
        assert!(f.state.is_finished());
        f.core.finalize_battle_execution(&mut f.state).unwrap();
        assert!(f.core.equipment_defense.last_will_used.is_empty());
        assert!(f.core.equipment_defense.last_will_resolving.is_empty());
    }
    #[test]
    fn composition_last_will_actual_basic_windup_cancelled_before_blast_and_late_release() {
        let (mut f, rear) = setup_will();
        place(&mut f.core, rear, 6, 3);
        f.core
            .event_queue
            .discard_channel_periodic_starts(f.a)
            .unwrap();
        f.core.schedule_initial_attack_for_unit(f.a, 1);
        advance(&mut f, 1);
        assert_eq!(f.core.units[&f.a].current_target, Some(f.enemy));
        assert_eq!(f.core.units[&f.a].basic_attack_windup_until_ms, Some(201));
        raw_damage(&mut f, 100, 100).unwrap();
        assert!(f.core.units[&f.a].lifecycle.is_dead());
        assert!(!f.core.units[&f.a].pending_basic_attack);
        assert_eq!(f.core.units[&f.a].basic_attack_windup_until_ms, None);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9700);
        advance(&mut f, 201);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9700);
    }
    #[test]
    fn composition_last_will_actual_lethal_basic_preempts_reactive_heal() {
        use crate::game::ability::sustained::{ReactiveHealProfile, SustainedEffectDef};
        let (mut f, _) = setup_will();
        place(&mut f.core, f.b, 0, 3);
        place(&mut f.core, f.enemy, 3, 3);
        let (actor, enemy) = (f.a, f.enemy);
        let attacker = f.core.units.get_mut(&enemy).unwrap();
        attacker.stats.attack = 100;
        attacker.basic_attack.range_role = WeaponRangeRole::Ranged;
        attacker.basic_attack.windup_ms = 200;
        attacker.basic_attack.delivery = Default::default();
        attacker.facing_direction = Some(FacingDirection::Left);
        attacker.basic_attack.defense_tile_range =
            Some(crate::game::battle::tile_range::TileRangePattern {
                include_anchor_tile: true,
                rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
            });
        assert!(f.core.is_basic_attack_target_in_range(enemy, actor));
        assert_eq!(
            f.core.select_basic_attack_target(enemy, None, None),
            Some(actor)
        );
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    actor,
                    actor,
                    SustainedEffectDef::ReactiveHeal {
                        duration_ms: 10000,
                        profile: ReactiveHealProfile {
                            definition: "last-will-priority".into(),
                            charges: 2,
                            max_health_percent: 50,
                        },
                    },
                    0,
                )
            })
            .unwrap();
        f.core
            .event_queue
            .discard_channel_periodic_starts(enemy)
            .unwrap();
        f.core.schedule_initial_attack_for_unit(enemy, 1);
        advance(&mut f, 1);
        assert_eq!(f.core.units[&enemy].current_target, Some(actor));
        let release = f.core.units[&enemy].basic_attack_windup_until_ms.unwrap();
        assert!(release > 1);
        advance(&mut f, release);
        assert!(f.core.units[&actor].lifecycle.is_dead());
        assert_eq!(f.core.units[&actor].stats.current_health, 0);
        assert_eq!(f.core.units[&enemy].stats.current_health, 9700);
        assert!(!f.core.event_log.entries.iter().any(|e|matches!(e.event,
            BattleLogEvent::HpChanged{target_instance_id,delta,..} if target_instance_id==actor && delta>0)));
    }
}
