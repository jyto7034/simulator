mod composition_hp_guard {
    use super::*;
    use crate::game::{
        battle::damage::DamageType,
        resource::{ResourceId, ResourceOwner},
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/hp_guard.skill.yaml"
    ));
    fn setup_with(source: &str) -> Fixture {
        let mut document = parse_yaml(source).unwrap();
        document["equipment_states"][0]["equipment"] = json!("r11_weapon");
        let mut f = super::setup_source(&document.to_string(), |_| {}, |_| {});
        for u in f.core.units.values_mut() {
            u.stats.max_health = 1000;
            u.stats.current_health = 1000;
            u.stats.defense = 0;
            u.stats.magic_resist = 0;
            u.stats.attack_interval_ms = 100_000;
            u.next_basic_attack_ms = 100_000;
        }
        f
    }
    fn setup() -> Fixture {
        setup_with(SOURCE)
    }
    fn copy(core: &BattleCore, target: UnitInstanceId) -> Uuid {
        core.items
            .values()
            .find(|i| i.owner_unit_instance == target)
            .unwrap()
            .source_snapshot()
            .persistent_copy_uuid
            .unwrap()
    }
    fn balance(core: &BattleCore, id: Uuid) -> u32 {
        let runtime = core.runtime_facts.resources.as_ref().unwrap();
        runtime
            .store
            .view()
            .value(
                &ResourceId::new("protection_charge".into()).unwrap(),
                ResourceOwner::EquipmentCopy(id),
                runtime.attempt.ordinal(),
            )
            .unwrap()
            .balance
    }
    fn active(
        core: &BattleCore,
        target: UnitInstanceId,
    ) -> Vec<crate::game::ability::sustained::SustainedEffectView> {
        core.sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter(|s| {
                s.target == target
                    && s.target_state
                        .as_ref()
                        .is_some_and(|p| p.damage_invulnerable)
            })
            .collect()
    }
    fn hit(
        core: &mut BattleCore,
        source: UnitInstanceId,
        target: UnitInstanceId,
        amount: u32,
        kind: DamageSource,
        ty: DamageType,
        now: u64,
    ) -> Result<u64, GameError> {
        core.runtime_transaction(|c| {
            let snapshot = c
                .damage_source_snapshot_for_unit(
                    source,
                    target,
                    kind,
                    ty,
                    amount,
                    Default::default(),
                    0,
                    now,
                    false,
                )
                .ok_or(GameError::InvalidAction)?;
            let loss = c
                .execute_committed_damage(CommittedDamageInput {
                    target_id: target,
                    source_snapshot: snapshot,
                    committed_at_ms: now,
                    policy: CommittedDamagePolicy::GeneralCommand,
                })
                .ok_or(GameError::InvalidAction)?
                .actual_hp_loss;
            crate::game::battle::runtime_control::poll()?;
            Ok(loss)
        })
    }
    fn strike(f: &mut Fixture, amount: u32, now: u64) -> u64 {
        hit(
            &mut f.core,
            f.enemy,
            f.a,
            amount,
            DamageSource::Ability,
            DamageType::True,
            now,
        )
        .unwrap()
    }
    fn heal(f: &mut Fixture, amount: i32, now: u64) {
        let (actor, target) = (f.b, f.a);
        f.core
            .runtime_transaction(|c| {
                c.apply_direct_heal_and_record(Some(actor), target, amount, now)
            })
            .unwrap();
    }
    fn expire(f: &mut Fixture, now: u64) {
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(now))
            .unwrap();
    }
    fn operations(doc: &mut Value) -> &mut Vec<Value> {
        doc["values"][0]["value"]["StateProgram"]["handlers"][0]["body"][0]["If"]["then_ops"][0]
            ["If"]["then_ops"]
            .as_array_mut()
            .unwrap()
    }
    #[test]
    fn composition_hp_guard_schema_shared_roundtrip_and_precise_resource_order_errors() {
        let a = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        assert_eq!(a.equipment_states()[0].equipment, "snow_queen_armor");
        assert!(a.sources()["equipment_states[0].program"]
            .definition
            .ends_with("StateProgram"));
        for (old, new, message) in [
            ("lifetime: Attempt", "lifetime: Run", "Attempt"),
            ("owner: EquipmentCopy", "owner: Employee", "owner"),
            (
                "received_heal_percent: 50",
                "received_heal_percent: 10001",
                "profile",
            ),
            (
                "basic_attack_locked: true",
                "unsupported_lock: true",
                "unknown",
            ),
        ] {
            let e =
                crate::authoring::composition::compile_yaml(&SOURCE.replace(old, new)).unwrap_err();
            assert!(
                format!("{e:?}")
                    .to_lowercase()
                    .contains(&message.to_lowercase()),
                "{e:?}"
            );
        }
        let missing = SOURCE.replace("id: protection_charge,", "id: undeclared_charge,");
        let e = crate::authoring::composition::compile_yaml(&missing).unwrap_err();
        assert!(
            e.reason.contains("unknown resource 'protection_charge'"),
            "{e:?}"
        );
        let mut doc = parse_yaml(SOURCE).unwrap();
        let consume = operations(&mut doc)[0].clone();
        let branch = doc["values"][0]["value"]["StateProgram"]["handlers"][0]["body"][0].clone();
        doc["values"][0]["value"]["StateProgram"]["handlers"][0]["body"] = json!([consume, branch]);
        assert!(compile_json(&doc.to_string())
            .unwrap_err()
            .reason
            .contains("reads must precede"));
        let mut doc = parse_yaml(SOURCE).unwrap();
        operations(&mut doc).swap(0, 1);
        assert!(compile_json(&doc.to_string())
            .unwrap_err()
            .reason
            .contains("writes must precede"));
    }
    #[test]
    fn composition_hp_guard_exact_postdamage_threshold_one_charge_no_exit_reward() {
        let mut f = setup();
        let id = copy(&f.core, f.a);
        assert_eq!(strike(&mut f, 699, 10), 699);
        assert!(active(&f.core, f.a).is_empty());
        assert_eq!(strike(&mut f, 1, 11), 1);
        assert_eq!(balance(&f.core, id), 0);
        let protection = active(&f.core, f.a);
        assert_eq!(protection.len(), 1);
        assert_eq!(protection[0].expires_at_ms, 3011);
        assert_eq!(strike(&mut f, 200, 12), 0);
        assert_eq!(f.core.units[&f.a].stats.current_health, 300);
        expire(&mut f, 3011);
        assert!(active(&f.core, f.a).is_empty());
        assert_eq!(f.core.units[&f.a].stats.current_health, 300);
        assert_eq!(strike(&mut f, 1, 3012), 1);
        assert!(active(&f.core, f.a).is_empty());
        assert_eq!(balance(&f.core, id), 0);
    }
    #[test]
    fn composition_hp_guard_actionless_damage_types_shields_lethal_and_cost_boundaries() {
        for kind in [
            DamageSource::BasicAttack,
            DamageSource::Ability,
            DamageSource::BuffTick,
            DamageSource::Environment,
        ] {
            let mut f = setup();
            f.core.units.get_mut(&f.a).unwrap().stats.current_health = 301;
            assert_eq!(
                hit(&mut f.core, f.enemy, f.a, 1, kind, DamageType::True, 10).unwrap(),
                1
            );
            assert_eq!(active(&f.core, f.a).len(), 1, "{kind:?}");
            for ty in [DamageType::Physical, DamageType::Magic, DamageType::True] {
                assert_eq!(hit(&mut f.core, f.enemy, f.a, 10, kind, ty, 11).unwrap(), 0);
            }
        }
        let mut f = setup();
        let id = copy(&f.core, f.a);
        let a = f.a;
        f.core.units.get_mut(&a).unwrap().stats.current_health = 301;
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    a,
                    a,
                    SustainedEffectDef::Shield {
                        duration_ms: 1000,
                        amount: 100,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
            })
            .unwrap();
        assert_eq!(strike(&mut f, 0, 1), 0);
        assert_eq!(strike(&mut f, 100, 2), 0);
        assert_eq!(balance(&f.core, id), 1);
        f.core
            .runtime_transaction(|c| {
                c.apply_hp_delta_and_record(
                    Some(a),
                    a,
                    -100,
                    3,
                    crate::game::battle::event_log::HpChangeReason::HealthCost,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(balance(&f.core, id), 1);
        assert!(active(&f.core, a).is_empty());
        assert_eq!(strike(&mut f, 201, 4), 201);
        assert!(active(&f.core, a).is_empty());
        assert_eq!(balance(&f.core, id), 1);
    }
    #[test]
    fn composition_hp_guard_authored_two_charges_and_threshold_duration_variant() {
        let source = SOURCE
            .replace("initial: 1, cap: 1", "initial: 2, cap: 2")
            .replace("value: 30, unit: Scalar", "value: 40, unit: Scalar")
            .replace(
                "value: 3000, unit: Milliseconds",
                "value: 1500, unit: Milliseconds",
            );
        let mut f = setup_with(&source);
        let id = copy(&f.core, f.a);
        strike(&mut f, 600, 10);
        assert_eq!(active(&f.core, f.a)[0].expires_at_ms, 1510);
        assert_eq!(balance(&f.core, id), 1);
        expire(&mut f, 1510);
        strike(&mut f, 1, 1511);
        assert_eq!(active(&f.core, f.a)[0].expires_at_ms, 3011);
        assert_eq!(balance(&f.core, id), 0);
        expire(&mut f, 3011);
        strike(&mut f, 1, 3012);
        assert!(active(&f.core, f.a).is_empty());
    }
    #[test]
    fn composition_hp_guard_received_heal_self_cap_outgoing_order_and_raw_exclusion() {
        let mut f = setup();
        strike(&mut f, 700, 10);
        heal(&mut f, 20, 11);
        assert_eq!(f.core.units[&f.a].stats.current_health, 330);
        let a = f.a;
        f.core
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(a), a, 20, 12))
            .unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, 360);
        f.core
            .runtime_transaction(|c| {
                c.apply_hp_delta_and_record(
                    Some(a),
                    a,
                    20,
                    13,
                    crate::game::battle::event_log::HpChangeReason::Rest,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, 380);
        heal(&mut f, 1000, 14);
        assert_eq!(f.core.units[&a].stats.current_health, 1000);
        f.core.units.get_mut(&a).unwrap().stats.current_health = 100;
        heal(&mut f, 20, 3010);
        assert_eq!(f.core.units[&a].stats.current_health, 120);
        // Existing outgoing 25% precedes the recipient 50%: floor(floor(10*1.25)*1.5)=18.
        let mut doc = parse_yaml(SOURCE).unwrap();
        doc["values"].as_array_mut().unwrap().push(json!({"name":"outgoing","value":{"SupportProgram":{"heal_percent":25,"shield_percent":0}}}));
        doc["equipment_supports"] =
            json!([{"equipment":"r11_weapon","program":{"Shared":"outgoing"}}]);
        let mut f = setup_with(&doc.to_string());
        strike(&mut f, 700, 10);
        heal(&mut f, 10, 11);
        assert_eq!(f.core.units[&f.a].stats.current_health, 318);
    }
    #[test]
    fn composition_hp_guard_copy_isolation_withdraw_redeploy_and_new_attempt() {
        let mut f = setup();
        let old = f.a;
        let id = copy(&f.core, old);
        let other = copy(&f.core, f.b);
        strike(&mut f, 700, 10);
        assert_eq!(balance(&f.core, id), 0);
        assert_eq!(balance(&f.core, other), 1);
        let owned = f.core.units[&old].source_owned_uuid;
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
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: old,
                time_ms: 20,
            })
            .unwrap();
        assert!(active(&f.core, old).is_empty());
        assert_eq!(balance(&f.core, id), 0);
        let crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed {
            unit_id: fresh,
        } = f
            .core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position: crate::game::resources::Position::new(2, 1),
                tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
                facing: crate::game::battle::tile_range::FacingDirection::Right,
                instance_salt: 0xe482,
                time_ms: 1001,
                current_hp_policy: None,
            })
            .unwrap()
        else {
            unreachable!()
        };
        f.a = fresh;
        let u = f.core.units.get_mut(&fresh).unwrap();
        u.stats.max_health = 1000;
        u.stats.current_health = 301;
        u.stats.defense = 0;
        strike(&mut f, 1, 1010);
        assert!(active(&f.core, fresh).is_empty());
        assert_eq!(copy(&f.core, fresh), id);
        assert_eq!(balance(&f.core, id), 0);
        let mut next = setup();
        assert_eq!(copy(&next.core, next.a), id);
        assert_eq!(balance(&next.core, id), 1);
        strike(&mut next, 700, 10);
        assert_eq!(balance(&next.core, id), 0);
    }
    #[test]
    fn composition_hp_guard_locks_are_not_stun_or_channel_and_end_resumes_eligibility() {
        let mut f = setup();
        let a = f.a;
        let before_tile = f.core.units[&a].world_position();
        let blocks = format!("{:?}", f.core.block_state);
        strike(&mut f, 700, 10);
        assert!(f.core.channel_pauses_basic(a));
        assert!(!f.core.channel_active(a));
        assert!(!f.core.unit_control_state(a, 11).unwrap().hard_cc_active());
        assert_eq!(
            f.core.skill_start_disabled_reason(a, 11),
            Some(crate::game::behavior::ActionDisabledReasonCode::ActionLocked)
        );
        assert_eq!(f.core.units[&a].world_position(), before_tile);
        assert_eq!(format!("{:?}", f.core.block_state), blocks);
        assert!(f.core.units[&a].is_active());
        expire(&mut f, 3010);
        assert!(!f.core.channel_pauses_basic(a));
        assert!(f.core.skill_start_disabled_reason(a, 3010).is_none());
        assert!(f.core.event_queue.iter().any(|e|matches!(e.as_ref(),BattleEvent::AttackStart{time_ms:3010,attacker_instance_id,..} if *attacker_instance_id==a)));
    }
    #[test]
    fn composition_hp_guard_cancellation_restores_hp_token_state_and_retry_observation() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        f.core.enable_authoring_observation();
        let id = copy(&f.core, f.a);
        let snapshot = |f: &Fixture| {
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.core.runtime_facts.resources),
                serde_json::to_value(&f.core.authoring_observation).unwrap(),
            )
        };
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let (enemy, a) = (f.enemy, f.a);
        let result = control::with_test_cancellation("state_reaction_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    hit(
                        &mut f.core,
                        enemy,
                        a,
                        700,
                        DamageSource::Ability,
                        DamageType::True,
                        10,
                    )
                },
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(snapshot(&f), before);
        assert_eq!(balance(&f.core, id), 1);
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        hit(
            &mut plain,
            enemy,
            a,
            700,
            DamageSource::Ability,
            DamageType::True,
            10,
        )
        .unwrap();
        strike(&mut f, 700, 10);
        assert_eq!(balance(&f.core, id), 0);
        assert_eq!(
            plain.transaction_debug_snapshot(),
            f.core.transaction_debug_snapshot()
        );
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_sourced_sustained
            .as_ref()
            .unwrap();
        assert!(
            matches!(witness.change,crate::game::battle::core::authoring_observation::AppliedChange::Sustained{target_state:Some(p),..} if p.damage_invulnerable&&p.basic_attack_locked&&p.active_skill_locked&&p.received_heal_percent==50)
        );
    }
    #[test]
    fn composition_hp_guard_missing_persistent_copy_rejects_without_partial_damage() {
        let mut f = setup();
        let a = f.a;
        for item in f
            .core
            .items
            .values_mut()
            .filter(|i| i.owner_unit_instance == a)
        {
            item.persistent_copy_uuid = None;
        }
        let before = f.core.transaction_debug_snapshot();
        let result = hit(
            &mut f.core,
            f.enemy,
            a,
            700,
            DamageSource::Ability,
            DamageType::True,
            10,
        );
        assert!(result.is_err());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
    }
    #[test]
    fn composition_hp_guard_pending_basic_cancels_but_released_projectile_survives() {
        use crate::game::ability::{DeliveryDef, ProjectileHitPolicy};
        let mut f = setup();
        f.core
            .step_battle_execution_until(&mut f.state, 10)
            .unwrap();
        let (a, enemy) = (f.a, f.enemy);
        let captured = super::launch(&mut f, a, enemy, 20);
        let action = captured.fact_source.action.as_ref().unwrap().clone();
        strike(&mut f, 700, 20);
        f.core
            .step_battle_execution_until(&mut f.state, 22)
            .unwrap();
        assert_eq!(f.core.units[&enemy].stats.current_health, 1000);
        let settled = f
            .core
            .runtime_fact_view()
            .actors()
            .find(|(id, _)| **id == a)
            .unwrap()
            .1
            .latest_settled_action
            .as_ref()
            .unwrap();
        assert_eq!(settled.ordinal, action.ordinal());
        assert!(format!("{:?}", settled.terminal).contains("Interrupted"));
        let mut f = setup();
        f.core
            .step_battle_execution_until(&mut f.state, 10)
            .unwrap();
        let (a, enemy) = (f.a, f.enemy);
        f.core.units.get_mut(&a).unwrap().basic_attack.delivery = DeliveryDef::Projectile {
            speed_units_per_ms: 1000,
            hit_policy: ProjectileHitPolicy::TargetLocked,
            allow_targetless_cast: false,
            max_range_tiles: None,
            max_lifetime_ms: None,
            max_kills: None,
            max_pierces: None,
            collision: Default::default(),
        };
        super::launch(&mut f, a, enemy, 20);
        f.core
            .step_battle_execution_until(&mut f.state, 22)
            .unwrap();
        assert!(f
            .core
            .projectiles
            .values()
            .any(|p| p.source_snapshot.source_id == a));
        strike(&mut f, 700, 23);
        f.core
            .step_battle_execution_until(&mut f.state, 1200)
            .unwrap();
        assert_eq!(f.core.units[&enemy].stats.current_health, 900);
        assert_eq!(active(&f.core, a).len(), 1);
    }
    #[test]
    fn composition_hp_guard_actual_fragment_preparation_is_interrupted_without_stun() {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| *doc = parse_yaml(&SOURCE.replace("snow_queen_armor", "r11_weapon")).unwrap(),
            |text| {
                *text = r12_control_ron("Damage(amount:100,damage_type:True)")
                    .replace("focus_time_ms: 0", "focus_time_ms: 500")
            },
            |scenario| {
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                        unreachable!()
                    };
                    profile.fragment_source =
                        Some(crate::game::skill_fragment::FragmentSourceSnapshot {
                            copy_uuid: Uuid::from_u128(0xf482 + i as u128),
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
        for u in f.core.units.values_mut() {
            u.stats.max_health = 1000;
            u.stats.current_health = 1000;
            u.stats.defense = 0;
            u.stats.attack_interval_ms = 100_000;
            u.next_basic_attack_ms = 100_000;
        }
        f.core
            .step_battle_execution_until(&mut f.state, 10)
            .unwrap();
        let a = f.a;
        f.core.units.get_mut(&a).unwrap().resonance_current = 100;
        f.core
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: a,
                skill_id: "r11_aura".into(),
                target: None,
                time_ms: 20,
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 21)
            .unwrap();
        let pending = f.core.units[&a]
            .pending_skill_cast
            .as_ref()
            .expect("real fragment is preparing")
            .action
            .as_ref()
            .unwrap()
            .clone();
        strike(&mut f, 700, 22);
        assert!(f.core.units[&a].pending_skill_cast.is_none());
        let settled = f
            .core
            .runtime_fact_view()
            .actors()
            .find(|(id, _)| **id == a)
            .unwrap()
            .1
            .latest_settled_action
            .as_ref()
            .unwrap();
        assert_eq!(settled.ordinal, pending.ordinal());
        assert!(format!("{:?}", settled.terminal).contains("Interrupted"));
        assert!(!f.core.unit_control_state(a, 22).unwrap().hard_cc_active());
        f.core
            .step_battle_execution_until(&mut f.state, 600)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 1000);
        assert!(f.core.skill_start_disabled_reason(a, 600).is_some());
    }
    #[test]
    fn composition_hp_guard_received_heal_strongest_fallback_and_terminal_cleanup() {
        let mut f = setup();
        strike(&mut f, 700, 10);
        let a = f.a;
        let original = active(&f.core, a)[0].target_state.as_ref().unwrap().clone();
        let mut stronger = original.clone();
        stronger.received_heal_percent = 100;
        let mut other = original;
        other.definition = "other-healing".into();
        other.received_heal_percent = 25;
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    a,
                    a,
                    SustainedEffectDef::TargetState {
                        duration_ms: 100,
                        profile: stronger,
                    },
                    20,
                )?;
                c.begin_sustained(
                    a,
                    a,
                    SustainedEffectDef::TargetState {
                        duration_ms: 200,
                        profile: other,
                    },
                    20,
                )
            })
            .unwrap();
        heal(&mut f, 20, 21);
        assert_eq!(f.core.units[&a].stats.current_health, 345);
        heal(&mut f, 20, 120);
        assert_eq!(f.core.units[&a].stats.current_health, 380);
        heal(&mut f, 20, 220);
        assert_eq!(f.core.units[&a].stats.current_health, 410);
        f.core
            .runtime_transaction(|c| c.clear_sustained(221))
            .unwrap();
        assert!(active(&f.core, a).is_empty());
        assert!(!f.core.target_state_action_locked(a, Some(221), true));
        heal(&mut f, 20, 222);
        assert_eq!(f.core.units[&a].stats.current_health, 430);
    }
    #[test]
    fn composition_hp_guard_retained_refusal_retries_and_actual_armor_has_no_legacy_freeze() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        let (enemy, a) = (f.enemy, f.a);
        let mut probe = f.core.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            hit(
                &mut probe,
                enemy,
                a,
                700,
                DamageSource::Ability,
                DamageType::True,
                10,
            )
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "equipment_state_record")
            .unwrap()
            .2;
        let old = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let snapshot = |f: &Fixture| {
            (
                f.core.transaction_debug_snapshot(),
                format!("{:?}", f.core.runtime_facts.resources),
                f.core.event_queue_debug_snapshot(),
            )
        };
        let before = snapshot(&f);
        assert!(hit(
            &mut f.core,
            enemy,
            a,
            700,
            DamageSource::Ability,
            DamageType::True,
            10
        )
        .is_err());
        assert_eq!(snapshot(&f), before);
        f.core.runtime_facts.limits.max_retained_bytes = old;
        strike(&mut f, 700, 10);
        assert_eq!(active(&f.core, a).len(), 1);
        let db: crate::game::data::equipment_data::EquipmentDatabase =
            ron::de::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../game_resources/data/equipments/base.ron"
            )))
            .unwrap();
        let armor = db.get_by_id("snow_queen_armor").unwrap();
        let text = ron::ser::to_string(armor).unwrap();
        assert!(!text.contains("freeze"));
        assert!(text.contains("MaxHealth") && text.contains("Defense"));
    }

    #[test]
    fn composition_hp_guard_handlers_read_committed_balance_and_reject_late_resource_reads() {
        let mut doc = parse_yaml(SOURCE).unwrap();
        let handlers = doc["values"][0]["value"]["StateProgram"]["handlers"]
            .as_array_mut()
            .unwrap();
        let mut second = handlers[0].clone();
        second["id"] = json!("second-reader");
        handlers.push(second);
        let mut f = setup_with(&doc.to_string());
        let id = copy(&f.core, f.a);
        strike(&mut f, 700, 10);
        assert_eq!(balance(&f.core, id), 0);
        assert_eq!(active(&f.core, f.a).len(), 1);
        let mut doc = parse_yaml(SOURCE).unwrap();
        let grant = operations(&mut doc)[1].clone();
        let branch = doc["values"][0]["value"]["StateProgram"]["handlers"][0]["body"][0].clone();
        doc["values"][0]["value"]["StateProgram"]["handlers"][0]["body"] = json!([grant, branch]);
        assert!(compile_json(&doc.to_string())
            .unwrap_err()
            .reason
            .contains("reads must precede"));
    }
}
