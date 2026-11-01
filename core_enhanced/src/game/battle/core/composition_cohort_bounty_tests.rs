mod composition_cohort_bounty {
    use super::*;
    use crate::game::{
        battle::{
            damage::{BattleCommand, DamageSource, DamageType},
            types::BattleVerdict,
        },
        resource::ResourceId,
    };
    const BOUNTY: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/cohort_bounty.skill.yaml"
    ));

    fn copy(f: &Hunt, actor: UnitInstanceId) -> Uuid {
        f.core
            .items
            .values()
            .find(|i| i.owner_unit_instance == actor && i.base_uuid == ACCESSORY)
            .unwrap()
            .persistent_copy_uuid
            .unwrap()
    }
    fn mount(f: &mut Hunt, run: &RunResourceState) {
        let owners = f
            .core
            .items
            .values()
            .filter_map(|i| i.persistent_copy_uuid)
            .map(ResourceOwner::EquipmentCopy)
            .collect();
        f.core.mount_run_resources(run, &owners).unwrap();
    }
    fn setup_bounty(initial: u32) -> Hunt {
        let mut f = setup_source(false, true, BOUNTY);
        f.core.units.get_mut(&f.a).unwrap().stats.attack = 100;
        let run = RunResourceState {
            revision: 0,
            closed: false,
            entries: if initial == 0 {
                vec![]
            } else {
                vec![RunResourceEntry {
                    resource: ResourceId::new("bounty_confirmed".into()).unwrap(),
                    owner: ResourceOwner::EquipmentCopy(copy(&f, f.a)),
                    balance: initial,
                    revision: 0,
                }]
            },
        };
        mount(&mut f, &run);
        f
    }
    fn balance(f: &Hunt, actor: UnitInstanceId, confirmed: bool) -> u32 {
        balance_copy(f, copy(f, actor), confirmed)
    }
    fn balance_copy(f: &Hunt, copy: Uuid, confirmed: bool) -> u32 {
        let runtime = f.core.runtime_facts.resources.as_ref().unwrap();
        let (id, scope) = if confirmed {
            ("bounty_confirmed", &runtime.run.as_ref().unwrap().scope)
        } else {
            ("bounty_pending", &runtime.attempt)
        };
        let lease = runtime
            .store
            .bind(
                &ResourceId::new(id.into()).unwrap(),
                ResourceOwner::EquipmentCopy(copy),
                scope,
            )
            .unwrap();
        runtime.store.value(&lease).unwrap().balance
    }
    fn finish(f: &mut Hunt, victory: bool, now: u64) {
        let outcomes = &mut Arc::make_mut(&mut f.core.scenario).terminal_outcomes;
        *outcomes = BattleScenario::fixture_survival_outcomes(now);
        if !victory {
            outcomes
                .iter_mut()
                .find(|o| o.verdict == BattleVerdict::Victory)
                .unwrap()
                .verdict = BattleVerdict::Defeat;
        }
        advance(f, now);
        assert!(f.state.is_finished());
    }
    fn raw_hit(
        f: &mut Hunt,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        kind: DamageSource,
        reaction: bool,
        now: u64,
    ) -> Result<(), GameError> {
        f.core.runtime_transaction(|core| {
            let mut source = core
                .damage_source_snapshot_for_unit(
                    actor,
                    target,
                    kind,
                    DamageType::True,
                    20000,
                    Default::default(),
                    0,
                    now,
                    false,
                )
                .ok_or(GameError::InvalidAction)?;
            source.fact_source.reaction = reaction;
            core.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: target,
                    source_snapshot: source,
                }],
                now,
            );
            crate::game::battle::runtime_control::poll()
        })
    }
    #[test]
    fn composition_cohort_bounty_direct_kills_are_pending_and_victory_confirms_once() {
        let mut f = setup_bounty(0);
        assert_eq!(f.core.units[&f.a].stats.attack, 100);
        advance(&mut f, 5000);
        let target = wolf(&f, f.a);
        assert_eq!(f.core.units[&target].stats.current_health, 300);
        hit(&mut f.core, f.a, target, 1000, 5000);
        assert!(!f.core.units[&target].is_active());
        assert_eq!((balance(&f, f.a, false), balance(&f, f.a, true)), (1, 0));
        let other = wolf(&f, f.b);
        hit(&mut f.core, f.a, other, 1000, 5000);
        assert_eq!((balance(&f, f.a, false), balance(&f, f.b, false)), (2, 0));
        assert_eq!(f.core.units[&f.a].stats.attack, 100);
        Arc::make_mut(&mut f.core.scenario).terminal_outcomes =
            BattleScenario::fixture_survival_outcomes(5100);
        advance(&mut f, 5100);
        assert!(f.state.is_finished());
        let result = f.core.finalize_battle_execution(&mut f.state).unwrap();
        assert_eq!(result.verdict, BattleVerdict::Victory);
        assert_eq!((balance(&f, f.a, false), balance(&f, f.a, true)), (0, 2));
        assert!(f.core.finalize_battle_execution(&mut f.state).is_err());
        assert_eq!(balance(&f, f.a, true), 2);
        let mut saved = RunResourceState {
            revision: 0,
            closed: false,
            entries: vec![],
        };
        f.core
            .capture_run_resources()
            .unwrap()
            .unwrap()
            .apply(&mut saved)
            .unwrap();
        let restored: RunResourceState =
            ron::de::from_str(&ron::ser::to_string(&saved).unwrap()).unwrap();
        let mut next = setup_source(false, true, BOUNTY);
        next.core.units.get_mut(&next.a).unwrap().stats.attack = 100;
        mount(&mut next, &restored);
        assert_eq!(balance(&next, next.a, true), 2);
        assert_eq!(balance(&next, next.a, false), 0);
        assert_eq!(next.core.units[&next.a].stats.attack, 110);
        let mut transferred = setup_source_with(false, true, BOUNTY, |scenario| {
            let copies: Vec<_> = scenario.groups[1]
                .spawns
                .iter()
                .map(|s| {
                    s.draft
                        .equipped_items
                        .iter()
                        .find(|i| i.base_uuid == ACCESSORY)
                        .unwrap()
                        .persistent_copy_uuid
                })
                .collect();
            assert_eq!(copies.len(), 2);
            for (index, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                spawn
                    .draft
                    .equipped_items
                    .iter_mut()
                    .find(|i| i.base_uuid == ACCESSORY)
                    .unwrap()
                    .persistent_copy_uuid = copies[1 - index];
            }
        });
        for actor in [transferred.a, transferred.b] {
            transferred.core.units.get_mut(&actor).unwrap().stats.attack = 100;
        }
        assert_eq!(copy(&transferred, transferred.b), copy(&f, f.a));
        mount(&mut transferred, &restored);
        assert_eq!(
            (
                balance(&transferred, transferred.a, true),
                balance(&transferred, transferred.b, true)
            ),
            (0, 2)
        );
        assert_eq!(
            (
                transferred.core.units[&transferred.a].stats.attack,
                transferred.core.units[&transferred.b].stats.attack
            ),
            (100, 110)
        );
        let fresh = setup_bounty(0);
        assert_eq!(copy(&fresh, fresh.a), copy(&f, f.a));
        assert_eq!(
            (
                balance(&fresh, fresh.a, true),
                fresh.core.units[&fresh.a].stats.attack
            ),
            (0, 100),
            "a new run has no inherited balance"
        );
    }
    #[test]
    fn composition_cohort_bounty_actual_summon_kill_does_not_credit_its_wearer() {
        use crate::game::battle::{
            core::spawn_kernel::{
                SpawnOccupancyClass, SpawnOccupancyPolicy, SpawnPlacement, SpawnRequest,
                SpawnRequestId, SpawnRequestSettlement, SpawnRequestSource,
            },
            types::{BattleUnitSourceIdentity, UnitCombatProfile},
        };
        let mut f = setup_bounty(0);
        advance(&mut f, 5000);
        let target = wolf(&f, f.a);
        let mut group = f.core.scenario.groups[1].clone();
        group.id = ScenarioGroupId::new("bounty-friendly-summon");
        group.required_for_victory = false;
        group.enemy_movement_plan = None;
        group.spawns.truncate(1);
        let spawn = &mut group.spawns[0];
        spawn.unit_ref = ScenarioUnitRef::new("bounty-friendly-summon");
        spawn.instance_salt = 900;
        spawn.draft.owned_uuid = Uuid::from_u128(0xe600);
        spawn.draft.source = BattleUnitSource::SummonedUnit {
            definition_id: "bounty-friendly-summon".into(),
            base_uuid: Uuid::from_u128(0xe600),
            profile: UnitCombatProfile::for_employee(100, 100, 0, 0, 100),
            summoner_instance_id: f.a,
            summon_sequence: 0,
            batch_uuid: Uuid::from_u128(0xe601),
        };
        spawn.draft.equipment_layout = None;
        spawn.draft.equipped_items.clear();
        spawn.draft.equipped_item_enhancements.clear();
        spawn.draft.employee_start_condition = None;
        let request = SpawnRequest {
            id: SpawnRequestId("bounty-friendly-summon".into()),
            source: SpawnRequestSource::Summon {
                summoner: f.a,
                definition_id: "bounty-friendly-summon".into(),
                count: 1,
            },
            owner: Side::Player,
            occupancy: SpawnOccupancyPolicy::AllowOccupied(vec![SpawnOccupancyClass::Any]),
            placements: vec![SpawnPlacement {
                order: 0,
                position: spawn.position,
                terrain_walkable: true,
                occupants: vec![],
            }],
        };
        let SpawnRequestSettlement::Spawned { unit_ids } = f
            .core
            .execute_spawn_request(request, |core, _| {
                core.spawn_scenario_group_member(&group, 0)
            })
            .unwrap()
        else {
            panic!("summon must be admitted")
        };
        assert_eq!(unit_ids.len(), 1);
        let summoned = unit_ids[0];
        assert!(f.core.units[&summoned].is_active());
        assert_eq!(f.core.units[&summoned].owner, Side::Player);
        assert!(matches!(f.core.units[&summoned].source_identity,
            BattleUnitSourceIdentity::SummonedUnit { summoner_instance_id, .. } if summoner_instance_id == f.a));
        raw_hit(
            &mut f,
            summoned,
            target,
            DamageSource::BasicAttack,
            false,
            5000,
        )
        .unwrap();
        assert!(
            !f.core.units[&target].is_active(),
            "actual summon damage must kill the cohort"
        );
        assert_eq!((balance(&f, f.a, false), balance(&f, f.b, false)), (0, 0));
    }
    #[test]
    fn composition_cohort_bounty_tiers_change_actual_attack_only_at_thresholds() {
        for (initial, attack) in [(0, 100), (1, 100), (2, 110), (3, 110), (4, 125)] {
            let f = setup_bounty(initial);
            assert_eq!(f.core.units[&f.a].stats.attack, attack, "balance {initial}");
            assert_eq!(balance(&f, f.a, false), 0);
        }
    }
    #[test]
    fn composition_cohort_bounty_cap_and_defeat_preserve_existing_confirmed_balance() {
        let mut f = setup_bounty(3);
        advance(&mut f, 5000);
        for target in [wolf(&f, f.a), wolf(&f, f.b)] {
            hit(&mut f.core, f.a, target, 1000, 5000);
        }
        assert_eq!(balance(&f, f.a, false), 2);
        advance(&mut f, 10000);
        let target = wolf(&f, f.a);
        hit(&mut f.core, f.a, target, 1000, 10000);
        assert_eq!(balance(&f, f.a, false), 2);
        assert_eq!(balance(&f, f.a, true), 3);
        assert_eq!(f.core.units[&f.a].stats.attack, 110);
        let mut incomplete = RunResourceState {
            revision: 0,
            closed: false,
            entries: vec![RunResourceEntry {
                resource: ResourceId::new("bounty_confirmed".into()).unwrap(),
                owner: ResourceOwner::EquipmentCopy(copy(&f, f.a)),
                balance: 3,
                revision: 0,
            }],
        };
        f.core
            .capture_run_resources()
            .unwrap()
            .unwrap()
            .apply(&mut incomplete)
            .unwrap();
        assert_eq!(
            incomplete.entries[0].balance, 3,
            "incomplete capture cannot promote pending"
        );
        let mut won = Hunt {
            core: f.core.clone(),
            state: f.state.clone(),
            a: f.a,
            b: f.b,
            enemy: f.enemy,
        };
        finish(&mut won, true, 10100);
        won.core.finalize_battle_execution(&mut won.state).unwrap();
        assert_eq!(
            (balance(&won, won.a, false), balance(&won, won.a, true)),
            (0, 4)
        );
        finish(&mut f, false, 10100);
        assert_eq!(
            f.core
                .finalize_battle_execution(&mut f.state)
                .unwrap()
                .verdict,
            BattleVerdict::Defeat
        );
        assert_eq!((balance(&f, f.a, false), balance(&f, f.a, true)), (0, 3));
    }
    #[test]
    fn composition_cohort_bounty_rejects_non_direct_and_inactive_wearer_kills() {
        for (kind, reaction, inactive) in [
            (DamageSource::BuffTick, false, 0),
            (DamageSource::Environment, false, 0),
            (DamageSource::Ability, true, 0),
            (DamageSource::BasicAttack, true, 0),
            (DamageSource::Ability, false, 1),
            (DamageSource::Ability, false, 2),
        ] {
            let mut f = setup_bounty(0);
            advance(&mut f, 5000);
            let actor = f.a;
            let key = copy(&f, actor);
            let target = wolf(&f, actor);
            // Capture the released carrier before the wearer exits.
            let mut source = f
                .core
                .damage_source_snapshot_for_unit(
                    actor,
                    target,
                    kind,
                    DamageType::True,
                    20000,
                    Default::default(),
                    0,
                    5000,
                    false,
                )
                .unwrap();
            source.fact_source.reaction = reaction;
            if inactive == 1 {
                f.core.units.get_mut(&actor).unwrap().corroded = true;
            }
            if inactive == 2 {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: actor,
                        time_ms: 5000,
                    })
                    .unwrap();
            }
            assert_eq!(f.core.units[&target].stats.current_health, 300);
            f.core
                .runtime_transaction(|core| {
                    core.process_commands(
                        vec![BattleCommand::ApplyDamage {
                            target_id: target,
                            source_snapshot: source,
                        }],
                        5000,
                    );
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
            assert!(
                !f.core.units[&target].is_active(),
                "the excluded cause must still finalize the wolf death"
            );
            assert_eq!(
                balance_copy(&f, key, false),
                0,
                "{kind:?} reaction={reaction} inactive={inactive}"
            );
        }
        let mut f = setup_bounty(0);
        advance(&mut f, 5000);
        let target = wolf(&f, f.a);
        hit(&mut f.core, f.b, target, 1000, 5000);
        assert_eq!((balance(&f, f.a, false), balance(&f, f.b, false)), (0, 1));
        let a = f.a;
        let normal = f.enemy;
        raw_hit(&mut f, a, normal, DamageSource::Ability, false, 5000).unwrap();
        assert!(!f.core.units[&normal].is_active());
        assert_eq!(
            balance(&f, a, false),
            0,
            "ordinary enemy never becomes a bounty target"
        );
    }
    #[test]
    fn composition_cohort_bounty_earned_balance_survives_withdrawal_until_team_victory() {
        for killed in [false, true] {
            let mut f = setup_bounty(2);
            advance(&mut f, 5000);
            let a = f.a;
            let key = copy(&f, a);
            let target = wolf(&f, a);
            hit(&mut f.core, a, target, 1000, 5000);
            if killed {
                let enemy = f.enemy;
                raw_hit(&mut f, enemy, a, DamageSource::Ability, false, 5000).unwrap();
                assert!(!f.core.units[&a].is_active());
            } else {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: a,
                        time_ms: 5000,
                    })
                    .unwrap();
            }
            assert_eq!(balance_copy(&f, key, false), 1);
            finish(&mut f, true, 5100);
            f.core.finalize_battle_execution(&mut f.state).unwrap();
            assert_eq!(
                (balance_copy(&f, key, false), balance_copy(&f, key, true)),
                (0, 3)
            );
        }
    }
    #[test]
    fn composition_cohort_bounty_kill_and_finalizer_cancellation_restore_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup_bounty(0);
        advance(&mut f, 5000);
        let target = wolf(&f, f.a);
        let actor = f.a;
        let snapshot = |f: &Hunt| {
            (
                f.core.transaction_debug_snapshot(),
                format!("{:?}", f.core.typed_resources()),
                format!("{:?}", f.state),
                f.core.shared_cohorts_snapshot().unwrap(),
            )
        };
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("cohort_bounty_earned", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || raw_hit(&mut f, actor, target, DamageSource::Ability, false, 5000),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, snapshot(&f));
        raw_hit(&mut f, actor, target, DamageSource::Ability, false, 5000).unwrap();
        assert_eq!(balance(&f, actor, false), 1);
        finish(&mut f, true, 5100);
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("battle_finalize_commit", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.finalize_battle_execution(&mut f.state),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, snapshot(&f));
        assert_eq!(
            (balance(&f, actor, false), balance(&f, actor, true)),
            (1, 0)
        );
        f.core.finalize_battle_execution(&mut f.state).unwrap();
        assert_eq!(
            (balance(&f, actor, false), balance(&f, actor, true)),
            (0, 1)
        );
    }
    #[test]
    fn composition_cohort_bounty_real_basic_attack_consumes_composed_attack() {
        let mut f = setup_bounty(2);
        assert_eq!(f.core.units[&f.a].stats.attack, 110);
        f.core
            .event_queue
            .discard_channel_periodic_starts(f.a)
            .unwrap();
        f.core.schedule_initial_attack_for_unit(f.a, 1);
        advance(&mut f, 1);
        assert_eq!(f.core.units[&f.a].current_target, Some(f.enemy));
        assert_eq!(f.core.units[&f.a].basic_attack_windup_until_ms, Some(201));
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
        advance(&mut f, 201);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9890);
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_bounty
            .as_ref()
            .unwrap();
        assert_eq!(
            (
                witness.confirmed,
                witness.tier_attack,
                witness.actual_attack
            ),
            (2, 10, Some(110))
        );
    }
    #[test]
    fn composition_cohort_bounty_skill_reads_same_attack_and_temporary_stats_do_not_compound() {
        use crate::game::{
            ability::sustained::SustainedEffectDef,
            stats::{StatId, StatModifier, StatModifierKind},
        };
        let mut doc = crate::authoring::composition::parse_yaml(BOUNTY).unwrap();
        let template = crate::authoring::composition::parse_yaml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/manual_state.skill.yaml"
        )))
        .unwrap();
        let mut probe = template["skills"][1].clone();
        probe["id"] = json!("bounty-stat-probe");
        probe["body"]["Action"]["effects"][0]["Damage"]["amount"] = json!({"Literal":{"Amount":{
            "moment":"Snapshot","root":{"Read":{"Stat":{"subject":"Source","stat":"Attack"}}}
        }}});
        probe["body"]["Action"]["effects"][0]["Damage"]["damage_type"] = json!("True");
        doc["skills"].as_array_mut().unwrap().push(probe);
        let mut f = setup_source_with(false, true, &doc.to_string(), |scenario| {
            for spawn in &mut scenario.groups[1].spawns {
                let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                    panic!()
                };
                profile.skill_id = Some("bounty-stat-probe".into());
            }
        });
        assert_eq!(f.core.units[&f.a].stats.attack, 100);
        let equipment_copy = copy(&f, f.a);
        mount(
            &mut f,
            &RunResourceState {
                revision: 0,
                closed: false,
                entries: vec![RunResourceEntry {
                    resource: ResourceId::new("bounty_confirmed".into()).unwrap(),
                    owner: ResourceOwner::EquipmentCopy(equipment_copy),
                    balance: 2,
                    revision: 0,
                }],
            },
        );
        let a = f.a;
        f.core
            .runtime_transaction(|core| {
                core.begin_sustained(
                    a,
                    a,
                    SustainedEffectDef::Modifiers {
                        duration_ms: 200,
                        modifiers: vec![StatModifier {
                            stat: StatId::Attack,
                            kind: StatModifierKind::Percent,
                            value: 25,
                        }],
                    },
                    0,
                )
            })
            .unwrap();
        assert_eq!(
            f.core.units[&a].stats.attack, 137,
            "(100+10)*125%, integer floor"
        );
        f.core
            .runtime_transaction(|core| core.reconcile_bounty_stats())
            .unwrap();
        assert_eq!(
            f.core.units[&a].stats.attack, 137,
            "recomposition uses the single baseline"
        );
        let skill = f
            .core
            .game_data
            .skill_data
            .get_by_id("bounty-stat-probe")
            .unwrap()
            .clone();
        let accepted_target = Some(crate::game::battle::event_log::SkillCastTarget::Unit {
            unit_instance_id: a,
        });
        assert!(f
            .core
            .admitted_skill_target_is_valid(a, &skill, accepted_target));
        f.core
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: a,
                skill_id: skill.id.clone(),
                target: None,
                time_ms: 1,
            })
            .unwrap();
        assert_eq!(
            f.core.units[&a]
                .pending_skill_cast
                .as_ref()
                .unwrap()
                .cast_target,
            accepted_target
        );
        assert!(
            f.core.units[&a].resonance_use.is_some(),
            "the actual command must admit its zero-cost resonance use"
        );
        advance(&mut f, 2);
        assert_eq!(
            f.core.units[&f.enemy].stats.current_health, 9863,
            "actual authored Attack expression damages the enemy; pending={:?}; active={:?}; events={:?}",
            f.core.units[&a].pending_skill_cast,
            f.core.active_skill_casts,
            f.core.event_log_entries_after_seq(0),
        );
        advance(&mut f, 200);
        assert_eq!(f.core.units[&a].stats.attack, 110);
        f.core.units.get_mut(&a).unwrap().corroded = true;
        f.core
            .runtime_transaction(|core| core.reconcile_bounty_stats())
            .unwrap();
        assert_eq!(f.core.units[&a].stats.attack, 100);
        f.core.units.get_mut(&a).unwrap().corroded = false;
        f.core
            .runtime_transaction(|core| core.reconcile_bounty_stats())
            .unwrap();
        assert_eq!(f.core.units[&a].stats.attack, 110);
    }
    #[test]
    fn composition_cohort_bounty_observation_limit_rollback_and_disabled_observer_agree() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup_bounty(0);
        advance(&mut f, 5000);
        let target = wolf(&f, f.a);
        let actor = f.a;
        let mut probe = Hunt {
            core: f.core.clone(),
            state: f.state.clone(),
            a: f.a,
            b: f.b,
            enemy: f.enemy,
        };
        let (result, admissions) = native::observe_retained_admissions(|| {
            raw_hit(
                &mut probe,
                actor,
                target,
                DamageSource::Ability,
                false,
                5000,
            )
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "bounty_observation")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        let original = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = (
            f.core.transaction_debug_snapshot(),
            format!("{:?}", f.core.typed_resources()),
            f.core.shared_cohorts_snapshot().unwrap(),
        );
        let error = raw_hit(&mut f, actor, target, DamageSource::Ability, false, 5000).unwrap_err();
        assert!(
            format!("{error:?}").contains("RetainedByteLimit"),
            "{error:?}"
        );
        assert_eq!(
            before,
            (
                f.core.transaction_debug_snapshot(),
                format!("{:?}", f.core.typed_resources()),
                f.core.shared_cohorts_snapshot().unwrap()
            )
        );
        f.core.runtime_facts.limits.max_retained_bytes = original;
        f.core.authoring_observation = None;
        raw_hit(&mut f, actor, target, DamageSource::Ability, false, 5000).unwrap();
        assert_eq!(balance(&f, actor, false), balance(&probe, actor, false));
        assert_eq!(
            f.core.units[&target].stats.current_health,
            probe.core.units[&target].stats.current_health
        );
        assert_eq!(
            f.core.shared_cohorts_snapshot().unwrap(),
            probe.core.shared_cohorts_snapshot().unwrap()
        );
    }
    #[test]
    fn composition_cohort_bounty_authored_resource_and_tier_contracts() {
        let doc = crate::authoring::composition::parse_yaml(BOUNTY).unwrap();
        crate::authoring::composition::compile_json(&doc.to_string()).unwrap();
        for (pointer, value, needle) in [
            ("/resources/0/lifetime", json!("Run"), "pending"),
            ("/resources/1/owner", json!("Employee"), "confirmed"),
            ("/resources/0/initial", json!(1), "pending"),
            ("/resources/0/cap", json!(0), "pending"),
            ("/resources/1/cap", json!(5), "confirmed"),
            (
                "/values/0/value/AttackProgram/cohort/bounty/tiers/1/threshold",
                json!(2),
                "tiers[1]",
            ),
            (
                "/values/0/value/AttackProgram/cohort/bounty/tiers/1/attack",
                json!(9),
                "tiers[1]",
            ),
            (
                "/values/0/value/AttackProgram/cohort/bounty/confirmed",
                json!("absent"),
                "confirmed",
            ),
        ] {
            let mut bad = doc.clone();
            *bad.pointer_mut(pointer).unwrap() = value;
            let error = crate::authoring::composition::compile_json(&bad.to_string()).unwrap_err();
            assert!(
                format!("{error:?}").contains(needle),
                "{pointer}: {error:?}"
            );
        }
    }
}
