mod composition_equipment_forms {
    use super::*;
    use crate::game::{battle::{types::BattleVerdict, damage::{BattleCommand, DamageSource, DamageType}, core::sim::BattleLiveCommand}, resource::ResourceId, stats::{StatId, StatModifier, StatModifierKind}};
    const FORMS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/skill_composition/equipment_forms.skill.yaml"));

    fn copy(f: &Hunt, actor: UnitInstanceId) -> Uuid {
        f.core.items.values().find(|i| i.owner_unit_instance == actor && i.base_uuid == ACCESSORY).unwrap().persistent_copy_uuid.unwrap()
    }
    fn mount(f: &mut Hunt, run: &RunResourceState) {
        let owners = f.core.items.values().filter_map(|i| i.persistent_copy_uuid).map(ResourceOwner::EquipmentCopy).collect();
        f.core.mount_run_resources(run, &owners).unwrap();
    }
    fn setup_forms(initial: u32) -> (Hunt, RunResourceState) {
        let mut f = setup_source(false, true, FORMS);
        for actor in [f.a, f.b] { f.core.units.get_mut(&actor).unwrap().stats.attack = 100; }
        let run = RunResourceState { entries: if initial == 0 { vec![] } else { vec![RunResourceEntry {
            resource: ResourceId::new("mask_wins".into()).unwrap(), owner: ResourceOwner::EquipmentCopy(copy(&f, f.a)), balance: initial, revision: 0,
        }] }, ..Default::default() };
        mount(&mut f, &run);
        (f, run)
    }
    fn balance(f: &Hunt, actor: UnitInstanceId, confirmed: bool) -> u32 {
        balance_copy(f, copy(f, actor), confirmed)
    }
    fn balance_copy(f: &Hunt, copy: Uuid, confirmed: bool) -> u32 {
        let runtime = f.core.runtime_facts.resources.as_ref().unwrap();
        let (id, scope) = if confirmed { ("mask_wins", &runtime.run.as_ref().unwrap().scope) } else { ("mask_pending", &runtime.attempt) };
        let lease = runtime.store.bind(&ResourceId::new(id.into()).unwrap(), ResourceOwner::EquipmentCopy(copy), scope).unwrap();
        runtime.store.value(&lease).unwrap().balance
    }
    fn finish(f: &mut Hunt, won: bool) {
        let mut outcomes = BattleScenario::fixture_survival_outcomes(100);
        if !won { outcomes.iter_mut().find(|o| o.verdict == BattleVerdict::Victory).unwrap().verdict = BattleVerdict::Defeat; }
        Arc::make_mut(&mut f.core.scenario).terminal_outcomes = outcomes;
        advance(f, 100);
        assert!(f.state.is_finished());
        f.core.finalize_battle_execution(&mut f.state).unwrap();
    }

    #[test]
    fn composition_equipment_forms_incoming_only_initial_observation_survives_idle_refresh() {
        let mut f = setup_source(false, true, FORMS);
        f.core.units.get_mut(&f.b).unwrap().corroded = true;
        let initial = f.core.units[&f.a].stats;
        let owned = copy(&f, f.a);
        let run = RunResourceState { entries: vec![RunResourceEntry {
            resource: ResourceId::new("mask_wins".into()).unwrap(),
            owner: ResourceOwner::EquipmentCopy(owned), balance: 3, revision: 0,
        }], ..Default::default() };
        assert!(f.core.authoring_observation.as_ref().unwrap().latest_equipment_form.is_none());
        // Before the Run is mounted, the form has no admitted progression scope.
        f.core.runtime_transaction(|core| core.reconcile_sustained(0)).unwrap();
        assert!(f.core.authoring_observation.as_ref().unwrap().latest_equipment_form.is_none());
        mount(&mut f, &run);
        assert_eq!((f.core.units[&f.a].stats.max_health, f.core.units[&f.a].stats.attack),
            (initial.max_health, initial.attack));
        let observed = f.core.authoring_observation.as_ref().unwrap().latest_equipment_form.as_ref().unwrap();
        assert_eq!((observed.equipment_copy, observed.confirmed, observed.form.index), (owned, 3, 2));
        assert_eq!(observed.form.name, "금이 간 탈");
        assert_eq!(observed.form.effect, crate::game::ability::equipment_forms::EquipmentFormEffect::IncomingDamage { percent: 25 });
        assert!(matches!(observed.operation, crate::game::battle::core::equipment_forms::FormOperation::Equipped));
        let first = serde_json::to_value(observed).unwrap();
        f.core.runtime_transaction(|core| core.reconcile_sustained(10)).unwrap();
        assert_eq!(serde_json::to_value(f.core.authoring_observation.as_ref().unwrap().latest_equipment_form.as_ref().unwrap()).unwrap(), first);
        hit(&mut f.core, f.a, f.enemy, 20000, 10);
        let earned = f.core.authoring_observation.as_ref().unwrap().latest_equipment_form.as_ref().unwrap();
        assert_eq!(earned.pending, 1);
        assert!(matches!(earned.operation, crate::game::battle::core::equipment_forms::FormOperation::Earned));
        let earned = serde_json::to_value(earned).unwrap();
        f.core.runtime_transaction(|core| core.reconcile_sustained(20)).unwrap();
        assert_eq!(serde_json::to_value(f.core.authoring_observation.as_ref().unwrap().latest_equipment_form.as_ref().unwrap()).unwrap(), earned);
    }

    #[test]
    fn composition_equipment_forms_qualified_victories_follow_one_two_one_after_save_restore() {
        let (mut f, mut run) = setup_forms(0);
        let original_copy = copy(&f, f.a);
        for (wins, expected_form, progress) in [(1, 1, 0), (2, 1, 1), (3, 2, 0), (4, 3, 0)] {
            assert!(f.core.units[&f.a].is_active());
            assert_eq!(copy(&f, f.a), original_copy);
            assert!(f.core.scenario_runtime.spawned.unit_refs.values().any(|id| *id == f.enemy));
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
            hit(&mut f.core, f.a, f.enemy, 20000, 0);
            assert!(f.core.units[&f.enemy].lifecycle.is_dead());
            assert_eq!((balance(&f, f.a, false), balance(&f, f.a, true)), (1, wins - 1));
            assert_eq!(balance(&f, f.b, false), 0);
            finish(&mut f, true);
            assert_eq!((balance(&f, f.a, false), balance(&f, f.a, true)), (0, wins));
            assert!(f.core.finalize_battle_execution(&mut f.state).is_err());
            f.core.capture_run_resources().unwrap().unwrap().apply(&mut run).unwrap();
            run = ron::de::from_str(&ron::ser::to_string(&run).unwrap()).unwrap();
            f = setup_source(false, true, FORMS);
            for actor in [f.a, f.b] { f.core.units.get_mut(&actor).unwrap().stats.attack = 100; }
            mount(&mut f, &run);
            assert_eq!(balance(&f, f.a, false), 0);
            let program = f.core.game_data.equipment_attack_catalog().by_equipment.get(&ACCESSORY).unwrap();
            let view = program.victory_forms.as_ref().unwrap().view(balance(&f, f.a, true));
            assert_eq!((view.index, view.progress), (expected_form, progress));
            let expected_attack = [100, 110, 110, 100, 140][wins as usize];
            assert_eq!((f.core.units[&f.a].stats.max_health, f.core.units[&f.a].stats.attack), (10000, expected_attack));
        }
        let (fresh, _) = setup_forms(0);
        assert_eq!(copy(&fresh, fresh.a), original_copy);
        assert_eq!((balance(&fresh, fresh.a, true), fresh.core.units[&fresh.a].stats.max_health), (0, 10100));
    }

    #[test]
    fn composition_equipment_forms_maximum_hp_is_not_healing_and_effects_are_replaced() {
        for (wins, max_hp, attack, damage) in [(0, 10100, 100, 100), (1, 10000, 110, 100), (2, 10000, 110, 100), (3, 10000, 100, 125), (4, 10000, 140, 100)] {
            let (mut f, _) = setup_forms(wins);
            for _ in 0..3 { f.core.runtime_transaction(|c| c.reconcile_sustained(0)).unwrap(); }
            assert_eq!((f.core.units[&f.a].stats.max_health, f.core.units[&f.a].stats.current_health, f.core.units[&f.a].stats.attack), (max_hp, 10000, attack));
            hit(&mut f.core, f.enemy, f.a, 100, 0);
            assert_eq!(f.core.units[&f.a].stats.current_health, 10000 - damage);
            f.core.units.get_mut(&f.a).unwrap().corroded = true;
            f.core.runtime_transaction(|c| c.reconcile_sustained(0)).unwrap();
            assert_eq!((f.core.units[&f.a].stats.max_health, f.core.units[&f.a].stats.attack), (10000, 100));
        }
    }

    #[test]
    fn composition_equipment_forms_no_kill_and_defeat_do_not_progress() {
        for won in [false, true] {
            let (mut f, _) = setup_forms(2);
            if !won { hit(&mut f.core, f.a, f.enemy, 20000, 0); assert_eq!(balance(&f, f.a, false), 1); }
            finish(&mut f, won);
            assert_eq!((balance(&f, f.a, false), balance(&f, f.a, true)), (0, 2));
        }
    }
    fn raw_hit(f: &mut Hunt, kind: DamageSource, reaction: bool) -> Result<(), GameError> {
        f.core.runtime_transaction(|c| {
            let mut source = c.damage_source_snapshot_for_unit(f.a, f.enemy, kind, DamageType::True, 20000, Default::default(), 0, 0, false).ok_or(GameError::InvalidAction)?;
            source.fact_source.reaction = reaction;
            c.process_commands(vec![BattleCommand::ApplyDamage { target_id: f.enemy, source_snapshot: source }], 0);
            crate::game::battle::runtime_control::poll()
        })
    }
    #[test]
    fn composition_equipment_forms_excluded_damage_still_kills_without_progress() {
        for (kind, reaction, corroded) in [
            (DamageSource::BuffTick, false, false), (DamageSource::Environment, false, false),
            (DamageSource::Ability, true, false), (DamageSource::BasicAttack, true, false),
            (DamageSource::Ability, false, true),
        ] {
            let (mut f, _) = setup_forms(0);
            f.core.units.get_mut(&f.a).unwrap().corroded = corroded;
            raw_hit(&mut f, kind, reaction).unwrap();
            assert!(f.core.units[&f.enemy].lifecycle.is_dead(), "excluded input must reach the actual kill");
            assert_eq!(balance(&f, f.a, false), 0, "{kind:?} reaction={reaction} corrosion={corroded}");
        }
        let (mut f, _) = setup_forms(0);
        raw_hit(&mut f, DamageSource::BasicAttack, false).unwrap();
        assert_eq!(balance(&f, f.a, false), 1);
    }
    #[test]
    fn composition_equipment_forms_many_kills_then_withdrawal_are_one_qualified_victory() {
        let mut f = setup_source_with(false, true, FORMS, |scenario| {
            scenario.groups[0].enemy_movement_plan = None;
            let mut second = scenario.groups[0].spawns[0].clone();
            second.unit_ref = ScenarioUnitRef::new("mask-second-enemy");
            second.instance_salt += 100;
            second.position = crate::game::resources::Position::new(6, 3);
            scenario.groups[0].spawns.push(second);
        });
        mount(&mut f, &RunResourceState::default());
        let owned = copy(&f, f.a);
        let enemies: Vec<_> = f.core.units.values().filter(|u| u.owner == Side::Opponent).map(|u| u.instance_id).collect();
        assert_eq!(enemies.len(), 2);
        for enemy in enemies {
            hit(&mut f.core, f.a, enemy, 20000, 0);
            assert!(f.core.units[&enemy].lifecycle.is_dead());
            assert_eq!(balance(&f, f.a, false), 1);
        }
        f.core.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id: f.a, time_ms: 0 }).unwrap();
        assert!(!f.core.units[&f.a].is_active());
        finish(&mut f, true);
        assert_eq!((balance_copy(&f, owned, false), balance_copy(&f, owned, true)), (0, 1));
    }
    #[test]
    fn composition_equipment_forms_hp_baseline_survives_permanent_changes_and_clamps_only_effective_maximum() {
        let (mut f, _) = setup_forms(0);
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = 10080;
        let current = f.core.units[&f.a].stats;
        let stats = f.core.apply_permanent_with_sustained(f.a, current, StatModifier { stat: StatId::MaxHealth, kind: StatModifierKind::Flat, value: 50 });
        f.core.units.get_mut(&f.a).unwrap().stats = stats;
        assert_eq!((stats.current_health, stats.max_health), (10080, 10150));
        f.core.runtime_transaction(|c| c.reconcile_equipment_forms()).unwrap();
        assert_eq!((f.core.units[&f.a].stats.current_health, f.core.units[&f.a].stats.max_health), (10080, 10150));
        f.core.units.get_mut(&f.a).unwrap().corroded = true;
        f.core.runtime_transaction(|c| c.reconcile_equipment_forms()).unwrap();
        assert_eq!((f.core.units[&f.a].stats.current_health, f.core.units[&f.a].stats.max_health), (10050, 10050));
        f.core.units.get_mut(&f.a).unwrap().corroded = false;
        f.core.runtime_transaction(|c| c.reconcile_equipment_forms()).unwrap();
        assert_eq!((f.core.units[&f.a].stats.current_health, f.core.units[&f.a].stats.max_health), (10050, 10150));
    }
    #[test]
    fn composition_equipment_forms_kill_and_finalizer_cancellation_restore_registration_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut f, _) = setup_forms(0);
        let snapshot = |f: &Hunt| (f.core.transaction_debug_snapshot(), format!("{:?}", f.core.typed_resources()), format!("{:?}", f.state));
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_form_earned", 1, || control::run(
            &ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution,
            || raw_hit(&mut f, DamageSource::Ability, false)));
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, snapshot(&f));
        raw_hit(&mut f, DamageSource::Ability, false).unwrap();
        assert_eq!(balance(&f, f.a, false), 1);
        Arc::make_mut(&mut f.core.scenario).terminal_outcomes = BattleScenario::fixture_survival_outcomes(100);
        advance(&mut f, 100);
        assert!(f.state.is_finished());
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_forms_settled", 1, || control::run(
            &ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution,
            || f.core.finalize_battle_execution(&mut f.state)));
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, snapshot(&f));
        assert_eq!((balance(&f, f.a, false), balance(&f, f.a, true)), (1, 0));
        f.core.finalize_battle_execution(&mut f.state).unwrap();
        assert_eq!((balance(&f, f.a, false), balance(&f, f.a, true)), (0, 1));
    }

    #[test]
    fn composition_equipment_forms_supplemental_skill_consumption_and_temporary_composition() {
        use crate::game::{
            ability::sustained::SustainedEffectDef,
            stats::{StatId, StatModifier, StatModifierKind},
        };
        let mut doc = crate::authoring::composition::parse_yaml(FORMS).unwrap();
        let template = crate::authoring::composition::parse_yaml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/manual_state.skill.yaml"
        )))
        .unwrap();
        let mut probe = template["skills"][1].clone();
        probe["id"] = json!("form-stat-probe");
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
                profile.skill_id = Some("form-stat-probe".into());
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
                    resource: ResourceId::new("mask_wins".into()).unwrap(),
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
            .runtime_transaction(|core| core.reconcile_equipment_forms())
            .unwrap();
        assert_eq!(
            f.core.units[&a].stats.attack, 137,
            "recomposition uses the single baseline"
        );
        let skill = f
            .core
            .game_data
            .skill_data
            .get_by_id("form-stat-probe")
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
            .runtime_transaction(|core| core.reconcile_equipment_forms())
            .unwrap();
        assert_eq!(f.core.units[&a].stats.attack, 100);
        f.core.units.get_mut(&a).unwrap().corroded = false;
        f.core
            .runtime_transaction(|core| core.reconcile_equipment_forms())
            .unwrap();
        assert_eq!(f.core.units[&a].stats.attack, 110);
    }

    #[test]
    fn composition_equipment_forms_supplemental_actual_summon_is_not_the_wearer() {
        use crate::game::battle::{
            core::spawn_kernel::{
                SpawnOccupancyClass, SpawnOccupancyPolicy, SpawnPlacement, SpawnRequest,
                SpawnRequestId, SpawnRequestSettlement, SpawnRequestSource,
            },
            types::{BattleUnitSourceIdentity, UnitCombatProfile},
        };
        let (mut f, _) = setup_forms(0);
        let target = f.enemy;
        let mut group = f.core.scenario.groups[1].clone();
        group.id = ScenarioGroupId::new("forms-friendly-summon");
        group.required_for_victory = false;
        group.enemy_movement_plan = None;
        group.spawns.truncate(1);
        let spawn = &mut group.spawns[0];
        spawn.unit_ref = ScenarioUnitRef::new("forms-friendly-summon");
        spawn.instance_salt = 900;
        spawn.draft.owned_uuid = Uuid::from_u128(0xe600);
        spawn.draft.source = BattleUnitSource::SummonedUnit {
            definition_id: "forms-friendly-summon".into(),
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
            id: SpawnRequestId("forms-friendly-summon".into()),
            source: SpawnRequestSource::Summon {
                summoner: f.a,
                definition_id: "forms-friendly-summon".into(),
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
        f.core.runtime_transaction(|core| {
            let source = core.damage_source_snapshot_for_unit(summoned, target, DamageSource::BasicAttack,
                DamageType::True, 20000, Default::default(), 0, 0, false).ok_or(GameError::InvalidAction)?;
            core.process_commands(vec![BattleCommand::ApplyDamage { target_id: target, source_snapshot: source }], 0);
            crate::game::battle::runtime_control::poll()
        }).unwrap();
        assert!(
            !f.core.units[&target].is_active(),
            "actual summon damage must kill the encounter enemy"
        );
        assert_eq!((balance(&f, f.a, false), balance(&f, f.b, false)), (0, 0));
    }

    #[test]
    fn composition_equipment_forms_supplemental_actual_basic_attack_earns_one_victory() {
        let (mut f, _) = setup_forms(1);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 110;
        assert_eq!(f.core.units[&f.a].stats.attack, 110);
        f.core.event_queue.discard_channel_periodic_starts(f.a).unwrap();
        f.core.schedule_initial_attack_for_unit(f.a, 1);
        advance(&mut f, 1);
        assert_eq!(f.core.units[&f.a].current_target, Some(f.enemy));
        assert_eq!(f.core.units[&f.a].basic_attack_windup_until_ms, Some(201));
        advance(&mut f, 201);
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        assert_eq!((balance(&f, f.a, false), balance(&f, f.a, true)), (1, 1));
        Arc::make_mut(&mut f.core.scenario).terminal_outcomes = BattleScenario::fixture_survival_outcomes(300);
        advance(&mut f, 300);
        f.core.finalize_battle_execution(&mut f.state).unwrap();
        assert_eq!((balance(&f, f.a, false), balance(&f, f.a, true)), (0, 2));
        let witness = f.core.authoring_observation.as_ref().unwrap().latest_equipment_form.as_ref().unwrap();
        assert_eq!((witness.form.index, witness.form.progress), (1, 1));
    }
    #[test]
    fn composition_equipment_forms_supplemental_retained_failure_restores_and_observer_is_optional() {
        use crate::game::battle::runtime_control as native;
        let (mut f, _) = setup_forms(0);
        let mut probe = Hunt { core: f.core.clone(), state: f.state.clone(), a: f.a, b: f.b, enemy: f.enemy };
        let (result, admissions) = native::observe_retained_admissions(|| raw_hit(&mut probe, DamageSource::Ability, false));
        result.unwrap();
        let required = admissions.iter().filter(|(site, _, _)| *site == "form_observation").map(|(_, _, n)| *n).max().unwrap();
        let original = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let snapshot = |f: &Hunt| (f.core.transaction_debug_snapshot(), format!("{:?}", f.core.typed_resources()), format!("{:?}", f.core.sustained.forms));
        let before = snapshot(&f);
        let error = raw_hit(&mut f, DamageSource::Ability, false).unwrap_err();
        assert!(format!("{error:?}").contains("RetainedByteLimit"), "{error:?}");
        assert_eq!(before, snapshot(&f));
        f.core.runtime_facts.limits.max_retained_bytes = original;
        f.core.authoring_observation = None;
        raw_hit(&mut f, DamageSource::Ability, false).unwrap();
        assert_eq!(balance(&f, f.a, false), balance(&probe, probe.a, false));
        assert_eq!(f.core.units[&f.enemy].stats.current_health, probe.core.units[&probe.enemy].stats.current_health);
        assert_eq!(format!("{:?}", f.core.sustained.forms), format!("{:?}", probe.core.sustained.forms));
    }
    #[test]
    fn composition_equipment_forms_supplemental_dead_wearer_credit_and_late_withdrawn_carrier() {
        let (mut f, _) = setup_forms(0);
        let owned = copy(&f, f.a);
        let released = f.core.damage_source_snapshot_for_unit(f.enemy, f.a, DamageSource::Ability,
            DamageType::True, 20000, Default::default(), 0, 0, false).unwrap();
        raw_hit(&mut f, DamageSource::Ability, false).unwrap();
        // A carrier released before the enemy died kills the wearer afterward;
        // another employee keeps the team eligible for a survival victory.
        f.core.runtime_transaction(|core| {
            core.process_commands(vec![BattleCommand::ApplyDamage { target_id: f.a, source_snapshot: released }], 0);
            crate::game::battle::runtime_control::poll()
        }).unwrap();
        assert!(f.core.units[&f.a].lifecycle.is_dead());
        assert!(f.core.units[&f.b].is_active());
        finish(&mut f, true);
        assert_eq!(balance_copy(&f, owned, true), 1);

        let (mut f, _) = setup_forms(0);
        let owned = copy(&f, f.a);
        let source = f.core.damage_source_snapshot_for_unit(f.a, f.enemy, DamageSource::Ability, DamageType::True, 20000, Default::default(), 0, 0, false).unwrap();
        f.core.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id: f.a, time_ms: 0 }).unwrap();
        f.core.runtime_transaction(|c| {
            c.process_commands(vec![BattleCommand::ApplyDamage { target_id: f.enemy, source_snapshot: source }], 0);
            crate::game::battle::runtime_control::poll()
        }).unwrap();
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        assert_eq!(balance_copy(&f, owned, false), 0);
    }

}
