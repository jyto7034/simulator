mod composition_revival {
    include!("composition_metamorphosis_tests.rs");
    use super::*;
    use crate::game::{
        ability::sustained::{HitStatusKind, HitStatusProfile, SurvivalPolicy, SustainedEffectDef},
        battle::{
            damage::{BattleCommand, DamageSource, DamageType},
            event_log::BattleLogEvent,
        },
        resource::{ResourceId, ResourceLifetime},
    };
    const REVIVAL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/equipment_revival.skill.yaml"
    ));
    fn setup_revive(initial: u32) -> Hunt {
        setup_revive_source(initial, REVIVAL, |b| b)
    }
    fn setup_revive_source(initial: u32, source: &str,
        configure_data: impl FnOnce(crate::game::data::GameDataBuilder) -> crate::game::data::GameDataBuilder) -> Hunt {
        let mut f = setup_source_with_graph_and_data(false, true, source,
            &crate::authoring::node_graph::tests::input_plan_fixture(), SHARED_COHORT_INPUT_RELICS, configure_data, |_| {});
        let owners = f
            .core
            .units
            .values()
            .filter(|u| BattleCore::is_employee_resonance_unit(u))
            .map(|u| ResourceOwner::Employee(u.source_owned_uuid))
            .chain(
                f.core
                    .items
                    .values()
                    .filter_map(|i| i.persistent_copy_uuid)
                    .map(ResourceOwner::EquipmentCopy),
            )
            .collect();
        let run = RunResourceState {
            revision: 0,
            closed: false,
            entries: if initial == 0 {
                vec![]
            } else {
                vec![RunResourceEntry {
                    resource: ResourceId::new("revival_total".into()).unwrap(),
                    owner: ResourceOwner::Employee(f.core.units[&f.a].source_owned_uuid),
                    balance: initial,
                    revision: 0,
                }]
            },
        };
        f.core.mount_run_resources(&run, &owners).unwrap();
        for id in [f.a, f.b] {
            let u = f.core.units.get_mut(&id).unwrap();
            u.stats.current_health = 100;
            u.stats.defense = 0;
            u.stats.magic_resist = 0;
        }
        f
    }
    fn raw(f: &mut Hunt, target: UnitInstanceId, amount: u32, now: u64) -> Result<(), GameError> {
        let attacker = f.enemy;
        f.core.runtime_transaction(|c| {
            let source = c
                .damage_source_snapshot_for_unit(
                    attacker,
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
    }
    fn counts(f: &Hunt, actor: UnitInstanceId) -> (u32, u32) {
        let r = f.core.runtime_facts.resources.as_ref().unwrap();
        let read = |name: &str, lifetime| {
            let scope = if lifetime == ResourceLifetime::Run {
                &r.run.as_ref().unwrap().scope
            } else {
                &r.attempt
            };
            let lease = r
                .store
                .bind(
                    &ResourceId::new(name.into()).unwrap(),
                    ResourceOwner::Employee(f.core.units[&actor].source_owned_uuid),
                    scope,
                )
                .unwrap();
            r.store.value(&lease).unwrap().balance
        };
        (
            read("revival_total", ResourceLifetime::Run),
            read("revival_used", ResourceLifetime::Attempt),
        )
    }
    fn snap(f: &Hunt) -> impl PartialEq + std::fmt::Debug {
        (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
        )
    }
    #[test]
    fn composition_revival_first_second_restore_same_runtime_and_third_limit_is_not_a_transform() {
        for initial in [0, 1, 2] {
            let mut f = setup_revive(initial);
            let actor = f.a;
            let position = f
                .core
                .sample_unit_body_at(actor, 10)
                .unwrap()
                .projected_tile();
            let epoch = f.core.units[&actor].basic_attack_epoch;
            raw(&mut f, actor, 100, 10).unwrap();
            assert_eq!(
                counts(&f, actor),
                if initial < 2 {
                    (initial + 1, 1)
                } else {
                    (2, 0)
                }
            );
            if initial < 2 {
                let u = &f.core.units[&actor];
                assert!(u.is_active());
                assert_eq!(u.stats.current_health, 5000);
                assert_eq!(
                    f.core
                        .sample_unit_body_at(actor, 10)
                        .unwrap()
                        .projected_tile(),
                    position
                );
                assert_eq!(u.basic_attack_epoch, epoch);
                assert!(!f.core.event_log.entries.iter().any(|e|matches!(e.event,BattleLogEvent::UnitDied{unit_instance_id,..} if unit_instance_id==actor)));
                let heal = f.core.runtime_fact_summary().latest_heal.as_ref().unwrap();
                assert_eq!(heal.actual_delta(), 4999);
                assert_eq!(heal.source.equipment.unwrap().definition_uuid, ACCESSORY);
                assert!(heal
                    .source
                    .authored
                    .as_ref()
                    .unwrap()
                    .field_path
                    .ends_with("program.EmployeeRevival"));
            } else {
                assert!(f.core.units[&actor].lifecycle.is_dead());
            }
        }
    }
    #[test]
    fn composition_revival_exact_immunity_boundary_once_per_employee_and_other_employee_independent(
    ) {
        let mut f = setup_revive(0);
        let (a, b) = (f.a, f.b);
        raw(&mut f, a, 100, 10).unwrap();
        raw(&mut f, a, 99999, 1009).unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, 5000);
        assert_eq!(counts(&f, a), (1, 1));
        raw(&mut f, b, 100, 1009).unwrap();
        assert_eq!(counts(&f, b), (1, 1));
        raw(&mut f, a, 5000, 1010).unwrap();
        assert!(f.core.units[&a].lifecycle.is_dead());
        assert_eq!(counts(&f, a), (1, 1));
        assert!(f.core.units[&b].is_active());
    }
    #[test]
    fn composition_revival_shield_survival_precedence_and_control_preservation() {
        let mut f = setup_revive(0);
        let a = f.a;
        let enemy = f.enemy;
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
                .runtime_transaction(|c| c.begin_sustained(a, a, effect, 0))
                .unwrap();
        }
        raw(&mut f, a, 100, 1).unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, 100);
        raw(&mut f, a, 100, 2).unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, 1);
        raw(&mut f, a, 100, 100).unwrap();
        assert_eq!(counts(&f, a), (0, 0));
        f.core
            .runtime_transaction(|c| {
                c.begin_committed_hit_status(
                    enemy,
                    a,
                    Side::Opponent,
                    3000,
                    HitStatusProfile {
                        definition: "revival-stun".into(),
                        kind: HitStatusKind::Stun,
                        modifier_source: None,
                        vulnerability: None,
                    },
                    100,
                )
            })
            .unwrap();
        raw(&mut f, a, 100, 101).unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, 5000);
        assert!(f.core.hit_stun_until(a, Some(102)).is_some());
        assert!(
            !f.core.sustained_invulnerable(a, 102).unwrap(),
            "damage immunity does not grant control immunity"
        );
        raw(&mut f, a, 50000, 102).unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, 5000);
    }
    #[test]
    fn composition_revival_cancel_each_commit_boundary_restores_and_retry_is_single_use() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in ["revival_reserved", "revival_restored", "revival_protected"] {
            let mut f = setup_revive(0);
            let a = f.a;
            let before = snap(&f);
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || raw(&mut f, a, 100, 0),
                )
            });
            assert!(result.is_err() && token.is_cancelled(), "{site}");
            assert_eq!(before, snap(&f), "{site}");
            assert_eq!(counts(&f, a), (0, 0));
            raw(&mut f, a, 100, 0).unwrap();
            assert_eq!(counts(&f, a), (1, 1));
            assert_eq!(f.core.units[&a].stats.current_health, 5000);
        }
    }
    #[test]
    fn composition_revival_observation_optional_and_resource_run_export_preserves_employee_owner() {
        let mut f = setup_revive(1);
        let a = f.a;
        let mut off = Hunt {
            core: f.core.clone(),
            state: f.state.clone(),
            a: f.a,
            b: f.b,
            enemy: f.enemy,
        };
        off.core.authoring_observation = None;
        raw(&mut f, a, 100, 0).unwrap();
        raw(&mut off, a, 100, 0).unwrap();
        assert_eq!(snap(&f), snap(&off));
        let mut run = RunResourceState {
            revision: 0,
            closed: false,
            entries: vec![RunResourceEntry {
                resource: ResourceId::new("revival_total".into()).unwrap(),
                owner: ResourceOwner::Employee(f.core.units[&a].source_owned_uuid),
                balance: 1,
                revision: 0,
            }],
        };
        // Apply to the actual mounted base, including its existing owner entry.
        f.core
            .capture_run_resources()
            .unwrap()
            .unwrap()
            .apply(&mut run)
            .unwrap();
        assert!(run.entries.iter().any(|e| e.owner
            == ResourceOwner::Employee(f.core.units[&a].source_owned_uuid)
            && e.balance == 2));
        assert!(run
            .entries
            .iter()
            .all(|e| e.resource.as_str() == "revival_total"));
    }
    #[test]
    fn composition_revival_authored_roundtrip_and_invalid_percent_duration_limit() {
        let a = crate::authoring::composition::compile_yaml(REVIVAL).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        assert_eq!(a.resources().len(), 2);
        for (old, new) in [
            ("restore_percent: 50", "restore_percent: 0"),
            ("restore_percent: 50", "restore_percent: 101"),
            ("invulnerable_ms: 1000", "invulnerable_ms: 0"),
            ("max_revivals: 2", "max_revivals: 0"),
            ("battle_used: revival_used", "battle_used: revival_total"),
        ] {
            let e = crate::authoring::composition::compile_yaml(&REVIVAL.replace(old, new))
                .unwrap_err();
            assert!(format!("{e:?}").contains("Native"), "{e:?}");
        }
        let f = setup_revive(0);
        for which in 0..5 {
            let mut equipment = (*f.core.game_data.equipment_data).clone();
            let mut resources = (*f.core.game_data.resource_data).clone();
            match which {
                0 => {
                    equipment
                        .items
                        .iter_mut()
                        .find(|i| i.id == "r11_accessory")
                        .unwrap()
                        .equipment_type = EquipmentType::Armor
                }
                1 => {
                    resources.resources[0].owner =
                        crate::game::resource::ResourceOwnerKind::EquipmentCopy
                }
                2 => resources.resources[1].lifetime = ResourceLifetime::Deployment,
                3 => resources.resources[0].cap = 1,
                _ => resources.resources[1].initial = 1,
            }
            let e = crate::game::ability::equipment_defense::EquipmentDefenseCatalog::compile(
                &equipment, &resources, &f.core.game_data.abnormality_data, &crate::game::value_expr::ExprLimits::default(),
            )
            .unwrap_err();
            assert!(
                e.to_string().contains(if which == 0 {
                    "requires accessory"
                } else {
                    "resource"
                }),
                "{e:?}"
            );
        }
    }
    #[test]
    fn composition_revival_withdraw_redeploy_keeps_employee_attempt_use_and_clears_immunity() {
        use crate::game::battle::{
            core::sim::{BattleLiveCommand, BattleLiveCommandOutcome},
            tile_range::FacingDirection,
            types::TacticalTileKind,
        };
        let mut f = setup_revive(0);
        let a = f.a;
        let owned = f.core.units[&a].source_owned_uuid;
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
        let position = f.core.units[&a].body.projected_tile();
        raw(&mut f, a, 100, 0).unwrap();
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 1,
            })
            .unwrap();
        let BattleLiveCommandOutcome::UnitDeployed { unit_id: fresh } = f
            .core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position,
                tile_kind: TacticalTileKind::Ground,
                facing: FacingDirection::Right,
                instance_salt: 0xe680,
                time_ms: 1,
                current_hp_policy: None,
            })
            .unwrap()
        else {
            unreachable!()
        };
        assert_ne!(a, fresh);
        assert_eq!(f.core.units[&fresh].source_owned_uuid, owned);
        assert_eq!(counts(&f, fresh), (1, 1));
        raw(&mut f, fresh, 100000, 2).unwrap();
        assert!(f.core.units[&fresh].lifecycle.is_dead());
        assert_eq!(counts(&f, fresh), (1, 1));
    }
    #[test]
    fn composition_revival_preserves_actual_unreleased_basic_and_shared_immunity_is_independently_authored(
    ) {
        use crate::game::ability::sustained::TargetStateProfile;
        let mut f = setup_revive(0);
        let a = f.a;
        let enemy = f.enemy;
        place(&mut f.core, a, 2, 1);
        place(&mut f.core, enemy, 3, 1);
        f.core
            .event_queue
            .discard_channel_periodic_starts(a)
            .unwrap();
        f.core.schedule_initial_attack_for_unit(a, 10);
        f.core
            .step_battle_execution_until(&mut f.state, 10)
            .unwrap();
        assert_eq!(f.core.units[&a].basic_attack_windup_until_ms, Some(210));
        assert_eq!(f.core.units[&a].current_target, Some(enemy));
        let epoch = f.core.units[&a].basic_attack_epoch;
        raw(&mut f, a, 100, 11).unwrap();
        assert_eq!(f.core.units[&a].basic_attack_windup_until_ms, Some(210));
        assert_eq!(f.core.units[&a].current_target, Some(enemy));
        assert_eq!(f.core.units[&a].basic_attack_epoch, epoch);
        f.core
            .step_battle_execution_until(&mut f.state, 211)
            .unwrap();
        assert!(f.core.units[&enemy].stats.current_health < 10000);
        let b = f.b;
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    b,
                    b,
                    SustainedEffectDef::TargetState {
                        duration_ms: 200,
                        profile: TargetStateProfile {
                            definition: "authored-damage-only-immunity".into(),
                            attack_percent: 0,
                            defense_percent: 0,
                            attack_speed_points: 0,
                            hp_floor_one: false,
                            damage_invulnerable: true,
                            basic_attack_locked: false,
                            active_skill_locked: false,
                            received_heal_percent: 0,
                            end_on_source_exit: false,
                        },
                    },
                    211,
                )
            })
            .unwrap();
        raw(&mut f, b, 100, 410).unwrap();
        assert_eq!(f.core.units[&b].stats.current_health, 100);
        assert_eq!(counts(&f, b), (0, 0));
        raw(&mut f, b, 100, 411).unwrap();
        assert_eq!(counts(&f, b), (1, 1));
    }
    #[test]
    fn composition_revival_retained_refusal_restores_both_counters_and_hp() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup_revive(0);
        let a = f.a;
        let mut probe = Hunt {
            core: f.core.clone(),
            state: f.state.clone(),
            a: f.a,
            b: f.b,
            enemy: f.enemy,
        };
        let (result, admissions) =
            native::observe_retained_admissions(|| raw(&mut probe, a, 100, 0));
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "revival_source")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = snap(&f);
        let error = raw(&mut f, a, 100, 0).unwrap_err();
        assert!(format!("{error:?}").contains("RetainedByteLimit"));
        assert_eq!(before, snap(&f));
        assert_eq!(counts(&f, a), (0, 0));
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        raw(&mut f, a, 100, 0).unwrap();
        assert_eq!(counts(&f, a), (1, 1));
    }

    #[test]
    fn composition_revival_lethal_health_payment_is_fully_paid_and_damage_immunity_does_not_discount_it(
    ) {
        use crate::game::battle::runtime_facts::{CostReason, FactResource};
        let mut f = setup_revive(0);
        let a = f.a;
        f.core
            .runtime_transaction(|c| c.pay_health_cost(a, 100, true, 0))
            .unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, 5000);
        assert_eq!(counts(&f, a), (1, 1));
        let cost = f.core.runtime_fact_summary().latest_cost.as_ref().unwrap();
        assert_eq!(cost.resource, FactResource::Health);
        assert_eq!(cost.reason, CostReason::AuthoredHealth);
        assert_eq!(cost.actual_spent(), 100);
        assert_eq!(f.core.runtime_fact_summary().damage_hp_lost, 0);
        f.core
            .runtime_transaction(|c| c.pay_health_cost(a, 5000, true, 1))
            .unwrap();
        assert!(f.core.units[&a].lifecycle.is_dead());
        assert_eq!(counts(&f, a), (1, 1));
        let mut direct = setup_revive(0);
        let b = direct.b;
        direct
            .core
            .runtime_transaction(|c| {
                c.apply_hp_delta_and_record(
                    None,
                    b,
                    -100,
                    0,
                    crate::game::battle::event_log::HpChangeReason::Command,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(direct.core.units[&b].stats.current_health, 5000);
        assert_eq!(counts(&direct, b), (1, 1));
    }
}
