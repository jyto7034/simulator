mod composition_recipient_support {
    use super::*;
    use crate::game::{
        ability::{
            equipment_support::ControlDurationKind as K,
            sustained::{
                ControlKind, HitStatusKind as H, HitStatusProfile, SustainedEffectDef as D,
            },
        },
        battle::{buffs::BuffId, enums::BattleEvent},
        value_expr as expr,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/recipient_support.skill.yaml"
    ));
    fn source() -> String {
        SOURCE.replace("black_swan_memory_brooch", "r11_weapon")
    }
    fn setup_with(s: &str) -> Fixture {
        fixture_extra_with_defense(
            "Damage(amount:1,damage_type:True)",
            false,
            None,
            false,
            s,
            |_| {},
            |_| {},
        )
    }
    fn setup() -> Fixture {
        setup_with(&source())
    }
    fn block(f: &mut Fixture, count: usize) {
        f.core.block_state.enemy_to_blocker.clear();
        f.core.block_state.blocker_to_enemies.clear();
        for index in 0..count {
            let id = if index == 0 {
                f.target
            } else {
                UnitInstanceId(Uuid::from_u128(0x8600 + index as u128))
            };
            if !f.core.units.contains_key(&id) {
                let mut unit = f.core.units[&f.target].clone();
                unit.instance_id = id;
                f.core.units.insert(id, unit);
            }
            f.core.block_state.enemy_to_blocker.insert(id, f.caster);
            f.core
                .block_state
                .blocker_to_enemies
                .entry(f.caster)
                .or_default()
                .push(id);
        }
    }
    fn heal(f: &mut Fixture, actor: Option<UnitInstanceId>, amount: i32, now: u64) -> u32 {
        let target = f.caster;
        let before = f.core.units[&target].stats.current_health;
        f.core
            .runtime_transaction(|c| c.apply_direct_heal_and_record(actor, target, amount, now))
            .unwrap();
        f.core.units[&target].stats.current_health - before
    }
    fn wound(f: &mut Fixture) {
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 100;
    }
    fn hw(f: &Fixture) -> &crate::game::battle::core::equipment_support::ReceivedHealWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_received_heal
            .as_ref()
            .unwrap()
    }
    fn cw(f: &Fixture) -> &crate::game::battle::core::equipment_support::ControlDurationWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_control_duration
            .as_ref()
            .unwrap()
    }
    fn hit(kind: H, duration_ms: u32) -> D {
        D::HitStatus {
            duration_ms,
            profile: HitStatusProfile {
                definition: "recipient-control".into(),
                kind,
                modifier_source: None,
                vulnerability: None,
            },
        }
    }
    fn attach(f: &mut Fixture, definition: D, now: u64) {
        let (a, t) = (f.target, f.caster);
        f.core
            .runtime_transaction(|c| c.begin_sustained(a, t, definition, now))
            .unwrap();
    }
    #[test]
    fn recipient_support_schema_roundtrip_context_units_bounds_and_shared_origin() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let restored = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(restored.equipment_supports()[0]
            .program
            .received_heal_percent
            .is_some());
        assert!(restored
            .sources()
            .values()
            .any(|s| s.definition.ends_with("SupportProgram") && !s.calls.is_empty()));
        for (from, to) in [
            ("moment: Live", "moment: Snapshot"),
            ("subject: Source", "subject: Target"),
            ("subject: Source", "subject: AggregateMember"),
            ("value: 1, unit: Count", "value: 0, unit: Count"),
            ("reduction_percent: 50", "reduction_percent: 100"),
            ("reduction_percent: 50", "reduction_percent: 0"),
            (
                "kinds: [Stun, Freeze, Root, Slow, Silence, Taunt, BasicLock]",
                "kinds: []",
            ),
            (
                "kinds: [Stun, Freeze, Root, Slow, Silence, Taunt, BasicLock]",
                "kinds: [Stun, Stun]",
            ),
        ] {
            let error =
                crate::authoring::composition::compile_yaml(&SOURCE.replace(from, to)).unwrap_err();
            assert!(
                error.source.definition.contains("SupportProgram"),
                "{error:?}"
            );
        }
    }
    #[test]
    fn recipient_support_heal_reads_current_unique_blocked_hostiles_and_authored_cap() {
        for (rate, cap) in [(20, 60), (35, 90)] {
            let mut f = setup_with(
                &source()
                    .replace("value: 20,", &format!("value: {rate},"))
                    .replace("value: 60,", &format!("value: {cap},")),
            );
            let a = f.ally;
            for count in [0, 1, 2, 4, 1, 0] {
                block(&mut f, count);
                wound(&mut f);
                let expected = 100 + (count as u32 * rate).min(cap);
                assert_eq!(heal(&mut f, Some(a), 100, 1), expected);
                assert_eq!(hw(&f).blocked_hostiles, count as u32);
            }
            block(&mut f, 2);
            f.core
                .units
                .get_mut(&f.target)
                .unwrap()
                .stats
                .current_health = 0;
            wound(&mut f);
            assert_eq!(heal(&mut f, Some(a), 100, 2), 100 + rate);
            let other = *f
                .core
                .block_state
                .enemy_to_blocker
                .keys()
                .find(|&&id| id != f.target)
                .unwrap();
            f.core.block_state.enemy_to_blocker.insert(other, f.ally);
            wound(&mut f);
            assert_eq!(heal(&mut f, Some(a), 100, 3), 100);
            // An enemy target still attacking the wearer does not imply a block relation.
            f.core.block_state.enemy_to_blocker.clear();
            wound(&mut f);
            assert_eq!(heal(&mut f, Some(a), 100, 4), 100);
        }
    }
    #[test]
    fn recipient_support_self_source_less_heal_cap_raw_restore_and_live_equipment_removal() {
        let mut f = setup();
        block(&mut f, 1);
        let t = f.caster;
        for actor in [Some(t), None, Some(f.ally)] {
            wound(&mut f);
            assert_eq!(heal(&mut f, actor, 100, 1), 120);
        }
        f.core.units.get_mut(&t).unwrap().stats.current_health = 995;
        assert_eq!(heal(&mut f, Some(t), 100, 2), 5);
        assert_eq!(hw(&f).after, 120);
        wound(&mut f);
        let observed = serde_json::to_value(hw(&f)).unwrap();
        f.core.apply_hp_delta_and_record(
            Some(t),
            t,
            100,
            3,
            crate::game::battle::event_log::HpChangeReason::Command,
        );
        assert_eq!(f.core.units[&t].stats.current_health, 200);
        assert_eq!(serde_json::to_value(hw(&f)).unwrap(), observed);
        f.core.items.retain(|_, i| i.owner_unit_instance != t);
        wound(&mut f);
        assert_eq!(heal(&mut f, Some(t), 100, 4), 100);
    }
    #[test]
    fn recipient_support_finite_hit_control_kinds_rounding_expiry_and_reapplication() {
        for (kind, expected) in [
            (H::Stun, K::Stun),
            (H::Freeze, K::Freeze),
            (H::Root, K::Root),
            (H::Slow { percent: 40 }, K::Slow),
            (H::BasicLock, K::BasicLock),
        ] {
            let mut f = setup();
            attach(&mut f, hit(kind.clone(), 1001), 10);
            assert_eq!(
                (cw(&f).kind, cw(&f).original_ms, cw(&f).effective_ms),
                (expected, 1001, 501)
            );
            assert!(f
                .core
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .iter()
                .any(|r| r.target == f.caster && r.expires_at_ms == 511));
            f.core
                .runtime_transaction(|c| c.reconcile_sustained(511))
                .unwrap();
            assert!(f
                .core
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .iter()
                .all(|r| r.target != f.caster));
            attach(&mut f, hit(kind, 1), 512);
            assert_eq!(cw(&f).effective_ms, 1);
        }
        for kind in [
            ControlKind::Root,
            ControlKind::Slow { percent: 30 },
            ControlKind::Taunt { priority: 1 },
        ] {
            let mut f = setup();
            attach(
                &mut f,
                D::Control {
                    duration_ms: 2000,
                    kind,
                },
                1,
            );
            assert_eq!(cw(&f).effective_ms, 1000);
        }
    }
    #[test]
    fn recipient_support_native_buff_duration_refresh_and_silence() {
        for (name, kind) in [
            ("r11_stun", K::Stun),
            ("r11_silence", K::Silence),
            ("r11_freeze", K::Freeze),
        ] {
            let mut f = setup();
            let (a, t) = (f.target, f.caster);
            let buff_id = BuffId::from_name(name);
            if name == "r11_freeze" {
                let mut buffs = f.core.game_data.buff_data.buffs.clone();
                buffs.push(crate::game::battle::buffs::BuffMetadata {
                    name: name.into(),
                    kind: crate::game::battle::buffs::BuffKind::Freeze,
                    tick_interval_ms: 0,
                    max_stacks: 1,
                    reapply_policy: crate::game::battle::buffs::BuffReapplyPolicy::RefreshDuration,
                });
                let artifact = crate::authoring::composition::compile_yaml(&source()).unwrap();
                f.core.game_data = GameDataBuilder::empty()
                    .with_skills((*f.core.game_data.skill_data).clone())
                    .with_skill_fragments((*f.core.game_data.skill_fragment_data).clone())
                    .with_equipment(f.core.game_data.equipment_data.items.clone())
                    .with_equipment_supports(artifact.equipment_supports().to_vec())
                    .with_buffs(crate::game::battle::buffs::BuffDatabase::new(buffs))
                    .build_arc();
            }

            for (now, duration) in [(1, 2000), (100, 3000)] {
                f.core
                    .process_event_for_test(
                        BattleEvent::ApplyBuff {
                            time_ms: now,
                            caster_instance_id: a,
                            target_instance_id: t,
                            buff_id,
                            duration_ms: duration,
                            cause: Default::default(),
                        },
                        now,
                    )
                    .unwrap();
                assert_eq!((cw(&f).kind, cw(&f).effective_ms), (kind, duration / 2));
                assert_eq!(
                    f.core.buffs.values().map(|b| b.expires_at_ms).max(),
                    Some(now + duration / 2)
                );
            }
        }
    }
    #[test]
    fn recipient_support_noncontrol_self_ally_immunity_and_released_source() {
        let mut f = setup();
        let (a, t) = (f.target, f.caster);
        for actor in [t, f.ally] {
            assert_eq!(
                f.core
                    .runtime_transaction(|c| c.received_control_duration(
                        actor,
                        t,
                        None,
                        K::Stun,
                        2000,
                        1
                    ))
                    .unwrap(),
                2000
            );
        }
        attach(&mut f, hit(H::BasicAttackSlow { percent: 30 }, 2000), 1);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_control_duration
            .is_none());
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    t,
                    t,
                    D::TargetState {
                        duration_ms: 2000,
                        profile: crate::game::ability::sustained::TargetStateProfile {
                            definition: "self-burden".into(),
                            attack_percent: 0,
                            defense_percent: 0,
                            attack_speed_points: 0,
                            hp_floor_one: false,
                            damage_invulnerable: false,
                            basic_attack_locked: true,
                            active_skill_locked: true,
                            received_heal_percent: 0,
                            end_on_source_exit: false,
                        },
                    },
                    1,
                )
            })
            .unwrap();
        assert!(f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .any(|r| r
                .target_state
                .as_ref()
                .is_some_and(|p| p.definition == "self-burden")
                && r.expires_at_ms == 2001));
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_control_duration
            .is_none());
        f.core
            .units
            .get_mut(&t)
            .unwrap()
            .target_traits
            .push(crate::game::battle::types::UnitTargetTrait::StunImmune);
        attach(&mut f, hit(H::Stun, 2000), 2);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_control_duration
            .is_none());
        f.core.units.get_mut(&t).unwrap().target_traits.clear();
        f.core.units.get_mut(&a).unwrap().lifecycle =
            crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn;
        let D::HitStatus { profile, .. } = hit(H::Freeze, 2000) else {
            unreachable!()
        };
        f.core
            .runtime_transaction(|c| {
                c.begin_committed_hit_status(a, t, Side::Opponent, 2000, profile, 3)
            })
            .unwrap();
        assert_eq!(cw(&f).effective_ms, 1000);
    }
    #[test]
    fn recipient_support_cancel_heal_and_control_restore_complete_state_then_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "received_heal_evaluated",
            "direct_heal_applied",
            "control_duration_evaluated",
            "sustained_started",
        ] {
            let mut f = setup();
            block(&mut f, 1);
            wound(&mut f);
            let (a, t) = (f.target, f.caster);
            let before = f.core.transaction_debug_snapshot();
            let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
            let queue = f.core.event_queue_debug_snapshot();
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || {
                        f.core.runtime_transaction(|c| {
                            if site == "control_duration_evaluated" || site == "sustained_started" {
                                c.begin_sustained(a, t, hit(H::Stun, 2000), 1)
                            } else {
                                c.apply_direct_heal_and_record(Some(t), t, 100, 1)
                            }
                        })
                    },
                )
            });
            assert!(result.is_err() && token.is_cancelled(), "{site}");
            assert_eq!(f.core.transaction_debug_snapshot(), before);
            assert_eq!(
                serde_json::to_value(&f.core.authoring_observation).unwrap(),
                obs
            );
            assert_eq!(f.core.event_queue_debug_snapshot(), queue);
            if site == "control_duration_evaluated" || site == "sustained_started" {
                attach(&mut f, hit(H::Stun, 2000), 1);
                assert_eq!(cw(&f).effective_ms, 1000);
            } else {
                assert_eq!(heal(&mut f, Some(t), 100, 1), 120);
            }
        }
    }
    #[test]
    fn recipient_support_observer_disabled_clone_and_retained_refusal_preserve_authority() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        block(&mut f, 1);
        wound(&mut f);
        let t = f.caster;
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        plain
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(t), t, 100, 1))
            .unwrap();
        let mut probe = f.core.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.runtime_transaction(|c| c.apply_direct_heal_and_record(Some(t), t, 100, 1))
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(s, _, _)| *s == "received_heal_observation")
            .map(|(_, _, b)| *b)
            .max()
            .unwrap();
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = f.core.transaction_debug_snapshot();
        assert!(f
            .core
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(t), t, 100, 1))
            .is_err());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        f.core.runtime_facts.limits = Default::default();
        assert_eq!(heal(&mut f, Some(t), 100, 1), 120);
        assert_eq!(
            f.core.transaction_debug_snapshot(),
            plain.transaction_debug_snapshot()
        );
        let mut copy = f.core.clone();
        copy.block_state.enemy_to_blocker.clear();
        assert_eq!(
            copy.runtime_transaction(|c| c.received_heal_amount(Some(t), t, 100, 2))
                .unwrap(),
            100
        );
        assert_eq!(
            f.core
                .runtime_transaction(|c| c.received_heal_amount(Some(t), t, 100, 2))
                .unwrap(),
            120
        );
    }
    #[test]
    fn recipient_support_invalid_live_result_rolls_back_only_when_healing_is_consumed() {
        let mut doc = parse_yaml(&source()).unwrap();
        doc["values"][0]["value"]["SupportProgram"]["received_heal_percent"]["root"] =
            json!({"Const":{"value":10001,"unit":"Scalar"}});
        let mut f = setup_with(&doc.to_string());
        attach(&mut f, hit(H::Stun, 2000), 1);
        assert_eq!(cw(&f).effective_ms, 1000);
        wound(&mut f);
        let t = f.caster;
        let before = f.core.transaction_debug_snapshot();
        assert!(f
            .core
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(t), t, 100, 2))
            .is_err());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
    }
    #[test]
    fn recipient_support_relation_count_source_target_snapshot_live_and_general_skill() {
        use expr::{
            AmountExpr, DeliveryCapabilities, EvalMoment, ExprMeter, RelationCountKind, ValueExpr,
            ValueRef, ValueSubject,
        };
        let mut f = setup();
        block(&mut f, 1);
        let catalog = f.core.game_data.equipment_support_catalog();
        let original = catalog
            .by_equipment
            .values()
            .next()
            .unwrap()
            .received_heal
            .as_ref()
            .unwrap();
        let expression = |moment, subject| {
            AmountExpr::authored(
                moment,
                ValueExpr::Read(ValueRef::RelationCount {
                    subject,
                    relation: RelationCountKind::BlockingHostiles,
                }),
            )
        };
        let snapshot = expr::compile(
            &expression(EvalMoment::Snapshot, ValueSubject::Source),
            original.source(),
            DeliveryCapabilities::all(),
            &Default::default(),
        )
        .unwrap();
        let live = expr::compile(
            &expression(EvalMoment::Live, ValueSubject::Target),
            original.source(),
            DeliveryCapabilities::all(),
            &Default::default(),
        )
        .unwrap();
        let source = crate::game::battle::runtime_facts::RuntimeFactSource {
            actor: Some(f.caster),
            ..Default::default()
        };
        let frame = f
            .core
            .build_value_expr_frame(&snapshot, &source, Some(f.caster), 1, None)
            .unwrap();
        block(&mut f, 2);
        assert_eq!(
            expr::evaluate(
                &snapshot,
                &frame,
                &mut ExprMeter::new(snapshot.evaluation_limits()).unwrap()
            )
            .unwrap()
            .value
            .value,
            1
        );
        let frame = f
            .core
            .build_value_expr_frame(&live, &source, Some(f.caster), 2, None)
            .unwrap();
        assert_eq!(
            expr::evaluate(
                &live,
                &frame,
                &mut ExprMeter::new(live.evaluation_limits()).unwrap()
            )
            .unwrap()
            .value
            .value,
            2
        );
        let mut f=fixture_extra_with_defense("DamageExpr(amount:(moment:Live,root:Mul(value:Const((value:10,unit:Magnitude)),scalar:Ratio(numerator:Read(RelationCount(subject:Source,relation:BlockingHostiles)),denominator:Const((value:1,unit:Count)),rounding:Floor))),damage_type:True)",false,None,false,&source_string(),|_|{},|s| {
            s.tactical_plan.player_plan = crate::game::battle::scenario::PlayerMovementPlan::FixedDefense;
        });
        f.core.units.get_mut(&f.caster).unwrap().block_capacity = 4;
        f.core.units.get_mut(&f.caster).unwrap().block_radius_units = 2.0;
        f.core.units.get_mut(&f.ally).unwrap().block_capacity = 0;
        let position = f.core.units[&f.caster].body.position;
        f.core.units.get_mut(&f.target).unwrap().body.position = position;
        for count in [1, 2] {
            let now = (count as u64 - 1) * 10;
            // The public activation creates the action occurrence required by
            // expression delivery; install the relation before its live read.
            cast(&mut f, now);
            block(&mut f, count);
            f.core.refresh_block_state();
            assert_eq!(
                f.core
                    .relation_count(f.caster, RelationCountKind::BlockingHostiles)
                    .unwrap(),
                count as u32
            );
            let health = |c: &BattleCore| {
                c.units
                    .values()
                    .filter(|u| u.owner == Side::Opponent)
                    .map(|u| u.stats.current_health)
                    .sum::<u32>()
            };
            let before = health(&f.core);
            f.core
                .step_battle_execution_until(&mut f.state, now + 1)
                .unwrap();
            assert_eq!(before - health(&f.core), count as u32 * 10);
        }
    }
    fn source_string() -> String {
        SOURCE.replace("black_swan_memory_brooch", "r11_weapon")
    }
    #[test]
    fn recipient_support_lifesteal_uses_the_same_incoming_heal_boundary_once() {
        let mut f = fixture_extra_with_defense(
            "Damage(amount:1,damage_type:True)",
            false,
            None,
            false,
            &source(),
            |w| r12_lifesteal_weapon(w, 100),
            |_| {},
        );
        block(&mut f, 1);
        wound(&mut f);
        let (a, t) = (f.caster, f.target);
        f.core.runtime_transaction(|c|{
            let snapshot=c.damage_source_snapshot_for_unit(a,t,DamageSource::BasicAttack,DamageType::True,40,Default::default(),0,1,false).ok_or(crate::game::behavior::GameError::InvalidAction)?;
            let loss=c.execute_committed_damage(crate::game::battle::core::damage_runtime::CommittedDamageInput{target_id:t,source_snapshot:snapshot.clone(),committed_at_ms:1,policy:crate::game::battle::core::damage_runtime::CommittedDamagePolicy::GeneralCommand}).ok_or(crate::game::behavior::GameError::InvalidAction)?.actual_hp_loss;
            assert_eq!(loss,40);c.finish_basic_lifesteal(&snapshot,loss,1)
        }).unwrap();
        assert_eq!(f.core.units[&a].stats.current_health, 112);
        assert_eq!(
            (hw(&f).before, hw(&f).after, hw(&f).actor),
            (10, 12, Some(a))
        );
    }
    #[test]
    fn recipient_support_outgoing_then_combined_target_percent_floors_once() {
        let mut f = setup_with(&source().replace("heal_percent: 0", "heal_percent: 25"));
        let (a, t) = (f.target, f.caster);
        // Fixture's other equipped actor uses the same catalog on its own side.
        f.core.units.get_mut(&a).unwrap().owner = Side::Player;
        for item in f
            .core
            .items
            .values_mut()
            .filter(|i| i.owner_unit_instance == a)
        {
            item.owner = Side::Player;
        }
        let mut hostile = f.core.units[&a].clone();
        let e = UnitInstanceId(Uuid::from_u128(0x86ff));
        hostile.instance_id = e;
        hostile.owner = Side::Opponent;
        f.core.units.insert(e, hostile);
        f.core.block_state.enemy_to_blocker.insert(e, t);
        f.core.block_state.blocker_to_enemies.insert(t, vec![e]);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    t,
                    t,
                    D::TargetState {
                        duration_ms: 200,
                        profile: crate::game::ability::sustained::TargetStateProfile {
                            definition: "other-received".into(),
                            attack_percent: 0,
                            defense_percent: 0,
                            attack_speed_points: 0,
                            hp_floor_one: false,
                            damage_invulnerable: false,
                            basic_attack_locked: false,
                            active_skill_locked: false,
                            received_heal_percent: 10,
                            end_on_source_exit: false,
                        },
                    },
                    1,
                )
            })
            .unwrap();
        wound(&mut f);
        assert_eq!(heal(&mut f, Some(a), 19, 2), 29);
        assert_eq!(
            (
                hw(&f).before,
                hw(&f).percent,
                hw(&f).other_percent,
                hw(&f).after
            ),
            (23, 20, 10, 29)
        );
    }
    #[test]
    fn recipient_support_equipped_copies_strongest_per_channel_and_owner_isolation() {
        let mut f = setup();
        let mut doc = parse_yaml(&source()).unwrap();
        let mut stronger = doc["values"][0].clone();
        stronger["name"] = json!("stronger");
        stronger["value"]["SupportProgram"]["received_heal_percent"]["root"]["Clamp"]["value"]
            ["Mul"]["scalar"]["Const"]["value"] = json!(35);
        stronger["value"]["SupportProgram"]["control_duration"]["reduction_percent"] = json!(75);
        doc["values"].as_array_mut().unwrap().push(stronger);
        doc["equipment_supports"]
            .as_array_mut()
            .unwrap()
            .push(json!({"equipment":"second_brooch","program":{"Shared":"stronger"}}));
        let artifact = compile_json(&doc.to_string()).unwrap();
        let mut items = f.core.game_data.equipment_data.items.clone();
        let mut second = items.iter().find(|i| i.id == "r11_weapon").unwrap().clone();
        second.id = "second_brooch".into();
        second.uuid = Uuid::from_u128(0xe863);
        second.equipment_type = crate::game::data::equipment_data::EquipmentType::Accessory;
        second.weapon_profile = None;
        second.triggered_effects.clear();
        items.push(second);
        f.core.game_data = GameDataBuilder::empty()
            .with_skills((*f.core.game_data.skill_data).clone())
            .with_skill_fragments((*f.core.game_data.skill_fragment_data).clone())
            .with_equipment(items)
            .with_equipment_supports(artifact.equipment_supports().to_vec())
            .with_buffs((*f.core.game_data.buff_data).clone())
            .build_arc();
        let mut copy = f
            .core
            .items
            .values()
            .find(|i| i.owner_unit_instance == f.caster)
            .unwrap()
            .clone();
        copy.instance_id = Uuid::from_u128(0xe864);
        copy.base_uuid = Uuid::from_u128(0xe863);
        copy.persistent_copy_uuid = Some(Uuid::from_u128(0xe865));
        let chosen = copy.instance_id;
        f.core.items.insert(copy.instance_id, copy);
        block(&mut f, 1);
        wound(&mut f);
        let t = f.caster;
        assert_eq!(heal(&mut f, Some(t), 100, 1), 135);
        assert_eq!(hw(&f).equipment.runtime_item_instance_id, chosen);
        attach(&mut f, hit(H::Slow { percent: 20 }, 2000), 2);
        assert_eq!(cw(&f).effective_ms, 500);
        f.core.items.get_mut(&chosen).unwrap().owner_unit_instance = f.ally;
        block(&mut f, 1);
        wound(&mut f);
        assert_eq!(heal(&mut f, Some(t), 100, 3), 120);
        attach(&mut f, hit(H::Slow { percent: 20 }, 2000), 4);
        assert_eq!(cw(&f).effective_ms, 1000);
        let ally = f.ally;
        assert_eq!(
            f.core
                .runtime_transaction(|c| c.received_heal_amount(Some(t), ally, 100, 5))
                .unwrap(),
            100
        );
    }
    #[test]
    fn recipient_support_ground_occupancy_controls_keep_area_and_exit_lifetimes() {
        use super::super::compiled_action_fixture::{
            cast as cast_ground, fixture_content_with_data, place,
        };
        for lifetime in ["UntilAreaEnd", "UntilExit"] {
            let mut doc = parse_yaml(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/skill_composition/channel_ground.skill.yaml"
            )))
            .unwrap();
            doc["version"] = json!(2);
            doc["skills"][0]["id"] = json!("r11_aura");
            doc["definitions"][0]["body"]["Action"]["modifiers"] = json!([]);
            doc["values"][0]["value"]["Delivery"]["TileArea"]["area"]["ground_status"]
                ["statuses"][1]["lifetime"] = json!(lifetime);
            let support = parse_yaml(&source()).unwrap();
            doc["values"]
                .as_array_mut()
                .unwrap()
                .push(support["values"][0].clone());
            doc["equipment_supports"] = support["equipment_supports"].clone();
            let artifact = compile_json(&doc.to_string()).unwrap();
            let native = crate::authoring::node_graph::NativeGraphContent {
                skills: artifact.load_native().unwrap(),
                equipment: vec![],
                mark_weapons: vec![],
            };
            let (mut core, mut execution, a, e, _) = fixture_content_with_data(
                &[],
                &[],
                native,
                |_| {},
                crate::game::resource::ResourceCatalog { resources: vec![] },
                |b| b.with_equipment_supports(artifact.equipment_supports().to_vec()),
                |scenario| {
                    scenario.groups[0].spawns[0].draft.equipped_items =
                        vec![crate::game::battle::types::BattleEquipmentBinding {
                            weapon_stage: Default::default(),
                            base_uuid: Uuid::from_u128(0x1101),
                            persistent_copy_uuid: Some(Uuid::from_u128(0xe869)),
                            slot: None,
                        }];
                },
            );
            place(&mut core, e, 3, 1);
            let mut probe = core.clone();
            assert_eq!(
                probe
                    .runtime_transaction(|c| c.received_control_duration(
                        a,
                        e,
                        None,
                        K::Stun,
                        1000,
                        0
                    ))
                    .unwrap(),
                500
            );
            cast_ground(&mut core, a, 0);
            core.step_battle_execution_until(&mut execution, 1501)
                .unwrap();
            assert!(core.unit_control_state(e, 1501).unwrap().hard_cc_active());
            if lifetime == "UntilExit" {
                place(&mut core, e, 7, 2);
                core.runtime_transaction(|c| c.reconcile_sustained(1502))
                    .unwrap();
                assert!(!core.unit_control_state(e, 1502).unwrap().hard_cc_active());
            } else {
                core.step_battle_execution_until(&mut execution, 2002)
                    .unwrap();
                assert!(!core.unit_control_state(e, 2002).unwrap().hard_cc_active());
            }
        }
    }
    #[test]
    fn recipient_support_native_control_cancellation_and_observation_refusal_are_atomic() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        let (a, t) = (f.target, f.caster);
        let apply = |c: &mut BattleCore| {
            c.process_event_for_test(
                BattleEvent::ApplyBuff {
                    time_ms: 1,
                    caster_instance_id: a,
                    target_instance_id: t,
                    buff_id: BuffId::from_name("r11_stun"),
                    duration_ms: 2000,
                    cause: Default::default(),
                },
                1,
            )
        };
        let before = f.core.transaction_debug_snapshot();
        let observation = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let queue = f.core.event_queue_debug_snapshot();
        let token = CancellationToken::default();
        let rejected = control::with_test_cancellation("battle_event_inner_complete", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.runtime_transaction(apply),
            )
        });
        assert!(rejected.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            observation
        );
        assert_eq!(f.core.event_queue_debug_snapshot(), queue);

        let mut accepted = f.core.clone();
        let mut unobserved = f.core.clone();
        unobserved.authoring_observation = None;
        let (result, admissions) =
            native::observe_retained_admissions(|| accepted.runtime_transaction(apply));
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "control_duration_observation")
            .map(|(_, _, bytes)| *bytes)
            .max()
            .unwrap();
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = f.core.transaction_debug_snapshot();
        assert!(f.core.runtime_transaction(apply).is_err());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            observation
        );
        assert_eq!(f.core.event_queue_debug_snapshot(), queue);
        f.core.runtime_facts.limits = Default::default();
        f.core.runtime_transaction(apply).unwrap();
        assert_eq!(cw(&f).effective_ms, 1000);
        assert_eq!(
            f.core.buffs.values().map(|b| b.expires_at_ms).max(),
            Some(1001)
        );
        assert_eq!(
            f.core.transaction_debug_snapshot(),
            accepted.transaction_debug_snapshot()
        );
        unobserved.runtime_transaction(apply).unwrap();
        assert_eq!(
            unobserved.transaction_debug_snapshot(),
            accepted.transaction_debug_snapshot()
        );
    }
}
