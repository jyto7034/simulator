mod composition_source_direct {
    use super::*;
    use crate::game::{
        ability::sustained::{HitStatusKind, SustainedEffectDef},
        battle::{
            core::damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
            damage::{DamageSource, DamageSourceSnapshot, DamageType},
        },
    };
    const INSIGHT: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/source_direct.skill.yaml"
    ));

    fn setup() -> Fixture {
        fixture(0, false, |doc| *doc = parse_yaml(INSIGHT).unwrap())
    }
    fn states(f: &Fixture) -> Vec<(UnitInstanceId, UnitInstanceId, u32, u64)> {
        f.core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter_map(|v| match v.hit_status?.kind {
                HitStatusKind::SourceDirectDamageTaken { stacks, .. } => {
                    Some((v.caster, v.target, stacks, v.expires_at_ms))
                }
                _ => None,
            })
            .collect()
    }
    fn basic(f: &mut Fixture, now: u64) -> u32 {
        let before = f.core.units[&f.enemy].stats.current_health;
        attack(f, now, true);
        f.core
            .step_battle_execution_until(&mut f.state, now + 2)
            .unwrap();
        before - f.core.units[&f.enemy].stats.current_health
    }
    fn snapshot(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        source: DamageSource,
        now: u64,
    ) -> DamageSourceSnapshot {
        core.damage_source_snapshot_for_unit(
            actor,
            target,
            source,
            DamageType::True,
            100,
            Default::default(),
            0,
            now,
            false,
        )
        .unwrap()
    }
    fn damage(
        core: &mut BattleCore,
        source: DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> u64 {
        core.execute_committed_damage(CommittedDamageInput {
            target_id: target,
            source_snapshot: source,
            committed_at_ms: now,
            policy: CommittedDamagePolicy::GeneralCommand,
        })
        .unwrap()
        .actual_hp_loss
    }
    #[test]
    fn composition_source_direct_schema_roundtrip_shared_origin_and_bounds() {
        let artifact = crate::authoring::composition::compile_yaml(INSIGHT).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert_eq!(
            artifact.equipment_attacks()[0]
                .program
                .on_hit_status
                .as_ref()
                .unwrap()
                .duration_ms,
            5000
        );
        for (old, new) in [
            ("duration_ms: 5000", "duration_ms: 0"),
            ("max_stacks: 5", "max_stacks: 0"),
            ("stacks: 1,", "stacks: 6,"),
            ("per_stack_percent: 3", "per_stack_percent: 0"),
            ("per_stack_percent: 3", "per_stack_percent: 4294967295"),
        ] {
            let error = crate::authoring::composition::compile_yaml(&INSIGHT.replace(old, new))
                .unwrap_err();
            assert!(
                error
                    .source
                    .definition
                    .contains("AttackProgram.on_hit_status"),
                "{error:?}"
            );
        }
    }
    #[test]
    fn composition_source_direct_actual_basics_cap_refresh_and_expiry() {
        let mut f = setup();
        let mut losses = Vec::new();
        for now in [10, 20, 30, 40, 50, 60] {
            losses.push(basic(&mut f, now));
        }
        assert_eq!(losses, vec![100, 103, 106, 109, 112, 115]);
        let active = states(&f);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].2, 5);
        assert_eq!(active[0].3, 5061);
        f.core
            .step_battle_execution_until(&mut f.state, 5011)
            .unwrap();
        assert_eq!(
            states(&f)[0].2,
            5,
            "obsolete first expiry must not end refreshed state"
        );
        f.core
            .step_battle_execution_until(&mut f.state, 5060)
            .unwrap();
        assert_eq!(states(&f).len(), 1);
        f.core
            .step_battle_execution_until(&mut f.state, 5061)
            .unwrap();
        assert!(states(&f).is_empty());
        assert_eq!(basic(&mut f, 5070), 100);
    }
    #[test]
    fn composition_source_direct_owner_target_and_non_direct_sources_are_isolated() {
        let mut f = setup();
        assert_eq!(basic(&mut f, 10), 100);
        let a = f.a;
        let b = f.b;
        let target = f.enemy;
        let other = f.secondary;
        for (actor, victim, kind, expected) in [
            (a, target, DamageSource::BasicAttack, 103),
            (b, target, DamageSource::BasicAttack, 100),
            (a, other, DamageSource::BasicAttack, 100),
            (a, target, DamageSource::Ability, 100),
            (a, target, DamageSource::BuffTick, 100),
            (a, target, DamageSource::Environment, 100),
        ] {
            f.core
                .runtime_transaction(|c| {
                    let s = snapshot(c, actor, victim, kind, 20);
                    assert_eq!(damage(c, s, victim, 20), expected);
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
        }
        assert_eq!(states(&f).len(), 1);
    }
    #[test]
    fn composition_source_direct_shielded_hit_stacks_without_hp_loss() {
        let mut f = setup();
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
                    SustainedEffectDef::Shield {
                        duration_ms: 1000,
                        amount: 500,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
            })
            .unwrap();
        assert_eq!(basic(&mut f, 10), 0);
        assert_eq!(states(&f)[0].2, 1);
        assert_eq!(basic(&mut f, 20), 0);
        assert_eq!(states(&f)[0].2, 2);
        let remaining = f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter_map(|v| v.shield.map(|p| p.remaining))
            .sum::<u32>();
        assert_eq!(remaining, 297);
    }
    fn captured(f: &mut Fixture, now: u64) -> DamageSourceSnapshot {
        attack(f, now, true);
        f.core
            .event_queue
            .iter()
            .find_map(|event| match event.as_ref() {
                BattleEvent::AttackResolve {
                    attacker_instance_id,
                    source_snapshot,
                    ..
                } if *attacker_instance_id == f.a => Some(source_snapshot.clone()),
                _ => None,
            })
            .expect("real regular attack must capture equipment")
    }
    #[test]
    fn composition_source_direct_segments_freeze_before_stack_and_new_strike_advances() {
        let mut f = setup();
        let source = captured(&mut f, 10);
        let target = f.enemy;
        let hits = source.equipment_hits.as_ref().expect("admitted on-hit providers");
        assert_eq!(hits.providers.len(), 1);
        assert_eq!(hits.providers[0].equipment.definition_uuid, Uuid::from_u128(0x1101));
        let employee = f.core.units[&f.a].source_owned_uuid;
        let expected_copy = f.core.scenario.groups[1].spawns.iter()
            .find(|s| s.draft.owned_uuid == employee).unwrap().draft.equipped_items[0].persistent_copy_uuid;
        assert_eq!(hits.providers[0].equipment.persistent_copy_uuid, expected_copy);
        assert_eq!(hits.providers[0].equipment, source.fact_source.equipment.unwrap());
        assert!(hits.providers[0].program.on_hit_status.is_some());
        for expected in [100, 103] {
            f.core
                .runtime_transaction(|c| {
                    let scope = c.begin_strike_scope();
                    assert_eq!(damage(c, source.clone(), target, 11), expected);
                    assert_eq!(damage(c, source.clone(), target, 11), expected);
                    c.end_strike_scope(scope);
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
        }
        assert_eq!(states(&f)[0].2, 2);
    }
    #[test]
    fn composition_source_direct_does_not_amplify_equipment_bonus_or_reactions() {
        use crate::game::battle::{cooldown::CooldownSource, damage::DamageBonusSnapshot};
        let mut f = setup();
        assert_eq!(basic(&mut f, 10), 100);
        let target = f.enemy;
        let actor = f.a;
        let item = f
            .core
            .items
            .values()
            .find(|i| i.owner_unit_instance == actor)
            .unwrap()
            .source_snapshot();
        f.core
            .runtime_transaction(|c| {
                let mut source = snapshot(c, actor, target, DamageSource::BasicAttack, 20);
                source.on_attack_bonus_damage.push(DamageBonusSnapshot {
                    source: CooldownSource::Item {
                        item_instance_id: item.runtime_item_instance_id,
                    },
                    equipment_source: Some(item),
                    flat: 50,
                    percent: 50,
                    damage_type: DamageType::True,
                });
                assert_eq!(
                    damage(c, source, target, 20),
                    228,
                    "103 base + unchanged 125 bonus: 50 + (100 + 50) * 50%"
                );
                let mut source = snapshot(c, actor, target, DamageSource::BasicAttack, 21);
                source.fact_source.reaction = true;
                assert_eq!(damage(c, source, target, 21), 100);
                c.units.get_mut(&actor).unwrap().corroded = true;
                let source = snapshot(c, actor, target, DamageSource::BasicAttack, 22);
                assert_eq!(damage(c, source, target, 22), 100);
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
    }
    #[test]
    fn composition_source_direct_real_fragment_damage_uses_own_stack() {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| *doc = parse_yaml(INSIGHT).unwrap(),
            |ron| *ron = r12_control_ron("Damage(amount:100,damage_type:True)"),
            |scenario| {
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                        unreachable!()
                    };
                    profile.fragment_source =
                        Some(crate::game::skill_fragment::FragmentSourceSnapshot {
                            copy_uuid: Uuid::from_u128(0xf400 + i as u128),
                            definition_id: "r09_fragment".into(),
                            form_id: "base".into(),
                            skill_id: "r11_aura".into(),
                        });
                }
            },
            Some(
                ron::de::from_str(
                    &include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/tests/fixtures/r09/fragments.ron"
                    ))
                    .replace("r08_strike", "r11_aura"),
                )
                .unwrap(),
            ),
        );
        assert_eq!(basic(&mut f, 10), 100);
        for (actor, expected, now) in [(f.a, 103, 20), (f.b, 100, 30)] {
            if actor == f.b {
                super::super::compiled_action_fixture::place(&mut f.core, f.a, 7, 3);
                super::super::compiled_action_fixture::place(&mut f.core, f.b, 2, 1);
            }
            let before = f.core.units[&f.enemy].stats.current_health;
            f.core.units.get_mut(&actor).unwrap().resonance_current = 100;
            f.core
                .apply_live_command(BattleLiveCommand::ActivateSkill {
                    unit_id: actor,
                    skill_id: "r11_aura".into(),
                    target: None,
                    time_ms: now,
                })
                .unwrap();
            f.core
                .step_battle_execution_until(&mut f.state, now + 1)
                .unwrap();
            assert_eq!(
                before - f.core.units[&f.enemy].stats.current_health,
                expected
            );
        }
        assert_eq!(
            states(&f)[0].2,
            1,
            "fragment direct hits read but never grow weapon stacks"
        );
    }
    #[test]
    fn composition_source_direct_projectile_and_landed_state_survive_withdrawal() {
        use crate::game::ability::{DeliveryDef, ProjectileHitPolicy};
        let mut f = setup();
        let owned = f.core.units[&f.a].source_owned_uuid;
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
        let original = f.a;
        assert_eq!(basic(&mut f, 10), 100);
        f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery = DeliveryDef::Projectile {
            speed_units_per_ms: 1000,
            hit_policy: ProjectileHitPolicy::TargetLocked,
            allow_targetless_cast: false,
            max_range_tiles: None,
            max_lifetime_ms: None,
            max_kills: None,
            max_pierces: None,
            collision: Default::default(),
        };
        attack(&mut f, 20, true);
        f.core
            .step_battle_execution_until(&mut f.state, 22)
            .unwrap();
        assert!(f
            .core
            .projectiles
            .values()
            .any(|p| p.source_snapshot.source_id == f.a));
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.a,
                time_ms: 22,
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 1200)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 797);
        assert_eq!(states(&f)[0].2, 2);
        let crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed {
            unit_id: fresh,
        } = f
            .core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position: crate::game::resources::Position::new(2, 1),
                tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
                facing: crate::game::battle::tile_range::FacingDirection::Right,
                instance_salt: 0xe410,
                time_ms: 1201,
                current_hp_policy: None,
            })
            .unwrap()
        else {
            unreachable!()
        };
        assert_ne!(fresh, original);
        let unit = f.core.units.get_mut(&fresh).unwrap();
        unit.stats.attack_interval_ms = 100_000;
        unit.next_basic_attack_ms = 100_000;
        unit.stats.move_speed_units_per_ms = 0;
        f.a = fresh;
        assert_eq!(
            basic(&mut f, 1210),
            100,
            "same owned copy on a new battle unit cannot inherit old stacks"
        );
        let active = states(&f);
        assert_eq!(active.len(), 2);
        assert_eq!(active.iter().find(|s| s.0 == original).unwrap().2, 2);
        assert_eq!(active.iter().find(|s| s.0 == fresh).unwrap().2, 1);
    }
    #[test]
    fn composition_source_direct_two_owners_and_area_targets_keep_separate_stacks() {
        use crate::game::{
            battle::tile_range::TileRangePattern, data::abnormality_data::BasicAttackAction,
        };
        let mut f = setup();
        super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 3, 2);
        f.core.units.get_mut(&f.a).unwrap().basic_attack.action = BasicAttackAction::DamageArea {
            footprint: TileRangePattern {
                include_anchor_tile: true,
                rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
            },
            secondary_percent: 100,
        };
        assert_eq!(basic(&mut f, 10), 100);
        assert_eq!(f.core.units[&f.secondary].stats.current_health, 900);
        assert_eq!(states(&f).len(), 2);
        assert_eq!(basic(&mut f, 20), 103);
        assert_eq!(f.core.units[&f.secondary].stats.current_health, 797);
        let old = f.a;
        super::super::compiled_action_fixture::place(&mut f.core, old, 7, 3);
        super::super::compiled_action_fixture::place(&mut f.core, f.b, 2, 1);
        f.a = f.b;
        assert_eq!(basic(&mut f, 30), 100);
        assert_eq!(basic(&mut f, 40), 103);
        let active = states(&f);
        assert_eq!(active.len(), 3);
        assert_eq!(
            active
                .iter()
                .find(|s| s.0 == old && s.1 == f.enemy)
                .unwrap()
                .2,
            2
        );
        assert_eq!(
            active
                .iter()
                .find(|s| s.0 == f.b && s.1 == f.enemy)
                .unwrap()
                .2,
            2
        );
    }
    #[test]
    fn composition_source_direct_zero_hit_parry_invulnerability_and_death() {
        // Zero attack power is not a useful target-selection input. Check an
        // already admitted regular strike whose committed component is zero.
        let mut zero = setup();
        let mut source = captured(&mut zero, 10);
        source.base_damage = 0;
        zero.core
            .runtime_transaction(|c| {
                assert_eq!(damage(c, source, zero.enemy, 11), 0);
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(states(&zero)[0].2, 1);
        let mut f = setup();
        assert_eq!(basic(&mut f, 10), 100);
        assert_eq!(states(&f)[0].2, 1);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
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
                    15,
                )
            })
            .unwrap();
        f.core.units.get_mut(&f.a).unwrap().stats.attack = 100;
        assert_eq!(basic(&mut f, 20), 0);
        assert!(f.core.sustained_invulnerable(f.enemy, 22).unwrap());
        assert_eq!(states(&f)[0].2, 1, "parry must not stack");
        assert_eq!(basic(&mut f, 30), 0);
        assert_eq!(states(&f)[0].2, 1, "invulnerability must not stack");
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
        assert_eq!(basic(&mut f, 220), 1);
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        assert!(states(&f).is_empty());
    }
    #[test]
    fn composition_source_direct_observation_does_not_change_execution() {
        let mut plain = setup();
        let mut observed = Fixture {
            core: plain.core.clone(),
            state: plain.state.clone(),
            a: plain.a,
            b: plain.b,
            enemy: plain.enemy,
            secondary: plain.secondary,
            run: plain.run.clone(),
        };
        observed.core.enable_authoring_observation();
        for now in [10, 20, 30] {
            assert_eq!(basic(&mut plain, now), basic(&mut observed, now));
        }
        assert_eq!(states(&plain), states(&observed));
        assert_eq!(
            plain.core.event_queue_debug_snapshot(),
            observed.core.event_queue_debug_snapshot()
        );
        assert_eq!(
            serde_json::to_value(plain.core.runtime_fact_summary()).unwrap(),
            serde_json::to_value(observed.core.runtime_fact_summary()).unwrap()
        );
        assert!(observed
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_sourced_sustained
            .is_some());
    }
    #[test]
    fn composition_source_direct_noncombatant_receives_damage_without_status_or_failed_attack() {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| *doc = parse_yaml(INSIGHT).unwrap(),
            |_| {},
            |scenario| {
                let draft = &mut scenario.groups[0].spawns[0].draft;
                let BattleUnitSource::TestFixture { base_uuid, profile } = draft.source.clone()
                else {
                    unreachable!()
                };
                draft.source = BattleUnitSource::DefenseObject { base_uuid, profile };
            },
            None,
        );
        assert!(!f.core.units[&f.enemy].is_combatant());
        assert_eq!(basic(&mut f, 10), 100);
        assert!(states(&f).is_empty());
        assert!(!format!("{:?}", f.core.event_log.entries).contains("AttackMiss"));
    }
    #[test]
    fn composition_source_direct_evaded_regular_hit_never_stacks() {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| {
                *doc = parse_yaml(INSIGHT).unwrap();
                let defense = parse_yaml(include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/skill_composition/equipment_evasion.skill.yaml"
                )))
                .unwrap();
                doc["values"]
                    .as_array_mut()
                    .unwrap()
                    .push(defense["values"][0].clone());
                doc["values"][1]["value"]["DefenseProgram"]["BasicAttackEvasion"]
                    ["chance_percent"] = json!(100);
                doc["equipment_defenses"] = defense["equipment_defenses"].clone();
            },
            |_| {},
            |scenario| {
                let mut weapon = scenario.groups[1].spawns[0].draft.equipped_items[0].clone();
                weapon.persistent_copy_uuid = Some(Uuid::from_u128(0xe409));
                scenario.groups[0].spawns[0].draft.equipped_items = vec![weapon];
            },
            None,
        );
        f.core.enable_authoring_observation();
        assert_eq!(basic(&mut f, 10), 0);
        assert!(
            f.core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_basic_evasion
                .as_ref()
                .unwrap()
                .evaded
        );
        assert!(states(&f).is_empty());
    }
    #[test]
    fn composition_source_direct_cancel_and_retained_rejection_restore_damage_and_state() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        f.core.enable_authoring_observation();
        assert_eq!(basic(&mut f, 10), 100);
        attack(&mut f, 20, true);
        let snapshot = |f: &Fixture| {
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.state),
                serde_json::to_value(&f.core.authoring_observation).unwrap(),
            )
        };
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_attack_hit_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 22),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(snapshot(&f), before);
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 22)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "source_direct_hit_replacements")
            .unwrap()
            .2;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = snapshot(&f);
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 22)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "source_direct_hit_replacements",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(snapshot(&f), before);
        f.core.runtime_facts.limits = Default::default();
        f.core
            .step_battle_execution_until(&mut f.state, 22)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 797);
        assert_eq!(states(&f)[0].2, 2);
        assert_eq!(
            f.core.transaction_debug_snapshot(),
            probe.transaction_debug_snapshot()
        );
    }
}
