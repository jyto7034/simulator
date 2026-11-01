mod composition_incoming_flat {
    use super::*;
    use crate::game::{
        ability::sustained::SustainedEffectDef,
        battle::damage::{
            calculate_damage_with_flat, reduction_credit_with_flat, DamageContext, DamageModifiers,
            DamageRequest, DamageSourceSnapshot,
        },
        behavior::GameError,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/incoming_flat.skill.yaml"
    ));
    fn setup(percent: u32) -> Fixture {
        let source = SOURCE
            .replace("black_swan_nettle_coat", "r11_weapon")
            .replace("value: 50,", &format!("value: {percent},"));
        let mut f = fixture_extra_with_defense(
            "Damage(amount:100,damage_type:True)",
            false,
            None,
            false,
            &source,
            |_| {},
            |_| {},
        );
        for u in f.core.units.values_mut() {
            u.stats.max_health = 10000;
            u.stats.current_health = 10000;
            u.stats.defense = 20;
        }
        f
    }
    fn snapshot(
        c: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        source: DamageSource,
        kind: DamageType,
        amount: u32,
        minimum: u32,
        now: u64,
    ) -> DamageSourceSnapshot {
        c.damage_source_snapshot_for_unit(
            actor,
            target,
            source,
            kind,
            amount,
            DamageModifiers::default(),
            minimum,
            now,
            false,
        )
        .unwrap()
    }
    fn apply(
        c: &mut BattleCore,
        target: UnitInstanceId,
        snap: DamageSourceSnapshot,
        now: u64,
    ) -> Result<u32, GameError> {
        let before = c.units[&target].stats.current_health;
        c.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id: target,
                source_snapshot: snap,
            }],
            now,
        );
        crate::game::battle::runtime_control::poll()?;
        Ok(before - c.units[&target].stats.current_health)
    }
    fn hit(f: &mut Fixture, amount: u32, now: u64) -> u32 {
        f.core
            .runtime_transaction(|c| {
                let snap = snapshot(
                    c,
                    f.target,
                    f.caster,
                    DamageSource::BasicAttack,
                    DamageType::Physical,
                    amount,
                    0,
                    now,
                );
                apply(c, f.caster, snap, now)
            })
            .unwrap()
    }
    fn evidence(f: &Fixture) -> &crate::game::battle::core::equipment_defense::IncomingFlatWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_incoming_flat
            .as_ref()
            .unwrap()
    }
    fn state(c: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
        )
    }
    #[test]
    fn incoming_flat_schema_shared_origins_roundtrip_and_context_rejections() {
        let a = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let b = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        assert!(matches!(b.equipment_defenses()[0].program,
            crate::game::ability::equipment_defense::EquipmentDefenseProgram::PhysicalBasicFlatReduction{..}));
        assert!(b
            .sources()
            .values()
            .any(|s| s.definition.contains("DefenseProgram") && !s.calls.is_empty()));
        for (from, to) in [
            ("moment: Live", "moment: Snapshot"),
            ("subject: Source", "subject: Target"),
            ("stat: Defense", "stat: Attack"),
            ("unit: Magnitude", "unit: Scalar"),
            ("value: 100, unit: Scalar", "value: 0, unit: Scalar"),
        ] {
            let error =
                crate::authoring::composition::compile_yaml(&SOURCE.replace(from, to)).unwrap_err();
            assert!(
                error
                    .source
                    .definition
                    .contains("PhysicalBasicFlatReduction"),
                "{error:?}"
            );
        }
        let mut f = setup(50);
        assert_eq!(hit(&mut f, 100, 1), 73);
        assert!(evidence(&f)
            .field_path
            .ends_with("PhysicalBasicFlatReduction.amount"));
        assert_eq!(evidence(&f).definition_id, "r11_weapon");
    }
    #[test]
    fn incoming_flat_authored_coefficients_cap_live_defense_and_negative_defense() {
        for (percent, expected) in [(50, 73), (75, 68)] {
            let mut f = setup(percent);
            assert_eq!(hit(&mut f, 100, 1), expected);
            assert_eq!(
                (
                    evidence(&f).defense,
                    evidence(&f).allowance,
                    evidence(&f).remaining
                ),
                (20, 100 - expected - 17, 0)
            );
            f.core.units.get_mut(&f.caster).unwrap().stats.defense = 100;
            assert_eq!(hit(&mut f, 100, 2), 20);
            assert_eq!(evidence(&f).allowance, 30);
            f.core.units.get_mut(&f.caster).unwrap().stats.defense = -20;
            assert!(hit(&mut f, 100, 3) > 100);
            assert_eq!(evidence(&f).allowance, 0);
        }
    }
    #[test]
    fn incoming_flat_source_type_self_ally_reaction_and_withdrawn_attacker() {
        let mut f = setup(50);
        for source in [
            DamageSource::Ability,
            DamageSource::BuffTick,
            DamageSource::Environment,
        ] {
            let loss = f
                .core
                .runtime_transaction(|c| {
                    let s = snapshot(
                        c,
                        f.target,
                        f.caster,
                        source,
                        DamageType::Physical,
                        100,
                        0,
                        1,
                    );
                    apply(c, f.caster, s, 1)
                })
                .unwrap();
            assert_eq!(loss, 83, "{source:?}");
        }
        for kind in [DamageType::Magic, DamageType::True] {
            let loss = f
                .core
                .runtime_transaction(|c| {
                    let s = snapshot(
                        c,
                        f.target,
                        f.caster,
                        DamageSource::BasicAttack,
                        kind,
                        100,
                        0,
                        2,
                    );
                    apply(c, f.caster, s, 2)
                })
                .unwrap();
            assert_eq!(loss, 100);
            assert!(f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_incoming_flat
                .is_none());
        }
        for actor in [f.caster, f.ally] {
            let s = snapshot(
                &mut f.core,
                actor,
                f.caster,
                DamageSource::BasicAttack,
                DamageType::Physical,
                100,
                0,
                3,
            );
            assert!(!f.core.incoming_flat_eligible(f.caster, &s));
        }
        let mut s = snapshot(
            &mut f.core,
            f.target,
            f.caster,
            DamageSource::BasicAttack,
            DamageType::Physical,
            100,
            0,
            4,
        );
        s.fact_source.reaction = true;
        assert_eq!(
            f.core
                .runtime_transaction(|c| apply(c, f.caster, s, 4))
                .unwrap(),
            83
        );
        let s = snapshot(
            &mut f.core,
            f.target,
            f.caster,
            DamageSource::BasicAttack,
            DamageType::Physical,
            100,
            0,
            5,
        );
        f.core
            .units
            .get_mut(&f.target)
            .unwrap()
            .stats
            .current_health = 0;
        assert_eq!(
            f.core
                .runtime_transaction(|c| apply(c, f.caster, s, 5))
                .unwrap(),
            73,
            "a previously released hostile basic retains eligibility"
        );
    }
    #[test]
    fn incoming_flat_one_target_strike_shares_allowance_then_next_strike_reads_current_defense() {
        let mut f = setup(50);
        let first = snapshot(
            &mut f.core,
            f.target,
            f.caster,
            DamageSource::BasicAttack,
            DamageType::Physical,
            6,
            0,
            1,
        );
        let mut second = first.clone();
        second.base_damage = 120;
        f.core
            .runtime_transaction(|c| {
                let old = c.begin_strike_scope();
                assert_eq!(apply(c, f.caster, first, 1)?, 0);
                let w = c
                    .authoring_observation
                    .as_ref()
                    .unwrap()
                    .latest_incoming_flat
                    .as_ref()
                    .unwrap();
                assert_eq!((w.allowance, w.used, w.remaining), (10, 5, 5));
                let strike = w.strike;
                c.units.get_mut(&f.caster).unwrap().stats.defense = 100;
                assert_eq!(apply(c, f.caster, second, 1)?, 55);
                let w = c
                    .authoring_observation
                    .as_ref()
                    .unwrap()
                    .latest_incoming_flat
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    (
                        w.strike,
                        w.defense,
                        w.allowance,
                        w.before,
                        w.used,
                        w.remaining
                    ),
                    (strike, 20, 10, 5, 5, 0)
                );
                c.end_strike_scope(old);
                Ok(())
            })
            .unwrap();
        assert_eq!(hit(&mut f, 120, 2), 30);
        assert_eq!(evidence(&f).allowance, 30);
    }
    fn formula_context<'a>(
        attack: &'a [crate::game::battle::cooldown::SourcedEffect],
        hit: &'a [crate::game::battle::cooldown::SourcedEffect],
    ) -> DamageContext<'a> {
        DamageContext {
            attacker_side: Side::Opponent,
            target_side: Side::Player,
            attacker_attack: 100,
            target_armor: 0,
            target_magic_resist: 0,
            target_incoming_modifiers: DamageModifiers::default(),
            target_current_hp: 1000,
            target_max_hp: 1000,
            on_attack_effects: attack,
            on_hit_effects: hit,
        }
    }
    fn request(base: u32, minimum: u32) -> DamageRequest {
        DamageRequest {
            source: DamageSource::BasicAttack,
            damage_type: DamageType::Physical,
            modifiers: DamageModifiers::default(),
            crit_roll_percent: None,
            attacker_id: Uuid::from_u128(10).into(),
            target_id: Uuid::from_u128(20).into(),
            base_damage: base,
            minimum_damage: minimum,
            time_ms: 0,
        }
    }
    #[test]
    fn incoming_flat_mixed_components_share_cap_and_defender_bonus_does_not_consume() {
        use crate::game::{
            battle::cooldown::{CooldownSource, SourcedEffect},
            stats::{Effect, TriggerEffectTarget},
        };
        let mut req = request(20, 0);
        let bonus = |actor, kind, flat| SourcedEffect {
            source: CooldownSource::Unit {
                unit_instance_id: actor,
            },
            equipment_source: None,
            target: TriggerEffectTarget::CounterpartUnit,
            effect: Effect::BonusDamage {
                flat,
                percent: 0,
                damage_type: kind,
            },
        };
        let attack = [
            bonus(req.attacker_id, DamageType::Physical, 10),
            bonus(req.attacker_id, DamageType::Magic, 40),
        ];
        let hit = [bonus(req.target_id, DamageType::Physical, 7)];
        let ctx = formula_context(&attack, &hit);
        let mut remaining = 25;
        let result = calculate_damage_with_flat(&req, &ctx, &mut remaining);
        assert_eq!(remaining, 0);
        assert_eq!(result.final_damage, 52);
        assert_eq!(
            result
                .breakdown
                .iter()
                .map(|b| b.final_damage)
                .collect::<Vec<_>>(),
            [0, 5, 40, 7]
        );
        let mut remaining = 100;
        let result = calculate_damage_with_flat(&req, &ctx, &mut remaining);
        assert_eq!((remaining, result.final_damage), (70, 47));
        req.damage_type = DamageType::Magic;
        assert!(crate::game::battle::damage::has_attacker_physical_component(&req, &ctx));
        let mut remaining = 25;
        let result = calculate_damage_with_flat(&req, &ctx, &mut remaining);
        assert_eq!(
            (remaining, result.final_damage),
            (15, 67),
            "Magic base keeps only attacker Physical bonus eligible"
        );
    }
    #[test]
    fn incoming_flat_guard_credit_uses_identical_prepercentage_budget_across_segments() {
        let mut ctx = formula_context(&[], &[]);
        ctx.target_incoming_modifiers.damage_reduction_percent = 50;
        for minimum in [0, 1] {
            let mut remaining = 50;
            let first = request(40, minimum);
            let before = remaining;
            let a = calculate_damage_with_flat(&first, &ctx, &mut remaining);
            assert_eq!((a.final_damage, remaining), (minimum, 10));
            assert_eq!(
                reduction_credit_with_flat(&first, &ctx, &a, DamageModifiers::default(), before),
                0
            );
            let second = request(100, minimum);
            let before = remaining;
            let b = calculate_damage_with_flat(&second, &ctx, &mut remaining);
            assert_eq!((b.final_damage, remaining), (45, 0));
            assert_eq!(
                reduction_credit_with_flat(&second, &ctx, &b, DamageModifiers::default(), before),
                45
            );
        }
        let mut ctx = formula_context(&[], &[]);
        ctx.target_armor = 100;
        ctx.target_incoming_modifiers.damage_amp_percent = 100;
        ctx.target_incoming_modifiers.damage_reduction_percent = 25;
        let mut remaining = 20;
        let r = calculate_damage_with_flat(&request(100, 0), &ctx, &mut remaining);
        assert_eq!(r.final_damage, 45, "(100/2-20)*2*0.75");
    }
    #[test]
    fn incoming_flat_shield_consumes_post_formula_amount_and_minimum_remains() {
        let mut f = setup(50);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.caster,
                    f.caster,
                    SustainedEffectDef::Shield {
                        duration_ms: 1000,
                        amount: 70,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
            })
            .unwrap();
        assert_eq!(hit(&mut f, 100, 1), 3);
        assert_eq!(evidence(&f).used, 10);
        let fact = f
            .core
            .runtime_fact_summary()
            .latest_damage
            .as_ref()
            .unwrap();
        assert_eq!(fact.protection.absorbed, 70);
        let s = snapshot(
            &mut f.core,
            f.target,
            f.caster,
            DamageSource::BasicAttack,
            DamageType::Physical,
            1,
            1,
            2,
        );
        assert_eq!(
            f.core
                .runtime_transaction(|c| apply(c, f.caster, s, 2))
                .unwrap(),
            1
        );
        let s = snapshot(
            &mut f.core,
            f.target,
            f.caster,
            DamageSource::BasicAttack,
            DamageType::Physical,
            0,
            1,
            3,
        );
        assert_eq!(
            f.core
                .runtime_transaction(|c| apply(c, f.caster, s, 3))
                .unwrap(),
            0
        );
    }
    #[test]
    fn incoming_flat_observation_disabled_clone_and_owner_isolation() {
        let mut f = setup(50);
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        for (now, defense, expected) in [(1, 20, 73), (2, 100, 20), (3, 0, 100)] {
            f.core.units.get_mut(&f.caster).unwrap().stats.defense = defense;
            plain.units.get_mut(&f.caster).unwrap().stats.defense = defense;
            assert_eq!(hit(&mut f, 100, now), expected);
            let s = snapshot(
                &mut plain,
                f.target,
                f.caster,
                DamageSource::BasicAttack,
                DamageType::Physical,
                100,
                0,
                now,
            );
            assert_eq!(
                plain
                    .runtime_transaction(|c| apply(c, f.caster, s, now))
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
        let s = snapshot(
            &mut f.core,
            f.target,
            f.ally,
            DamageSource::BasicAttack,
            DamageType::Physical,
            100,
            0,
            4,
        );
        assert_eq!(
            f.core
                .runtime_transaction(|c| apply(c, f.ally, s, 4))
                .unwrap(),
            73
        );
        assert_eq!(evidence(&f).target, f.ally);
        assert!(plain.authoring_observation.is_none());
    }
    #[test]
    fn incoming_flat_cancelled_evaluation_restores_exact_state_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in ["incoming_flat_evaluated", "incoming_flat_applied"] {
            let mut f = setup(50);
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
                            let s = snapshot(
                                c,
                                f.target,
                                f.caster,
                                DamageSource::BasicAttack,
                                DamageType::Physical,
                                100,
                                0,
                                1,
                            );
                            let loss = apply(c, f.caster, s, 1)?;
                            c.end_strike_scope(old);
                            Ok(loss)
                        })
                    },
                )
            });
            assert!(result.is_err() && token.is_cancelled());
            assert_eq!(state(&f.core), before);
            assert_eq!(hit(&mut f, 100, 1), 73);
        }
    }
    #[test]
    fn incoming_flat_retained_refusal_restores_damage_allowance_and_observation() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(50);
        let mut probe = f.core.clone();
        let action = |c: &mut BattleCore| {
            let old = c.begin_strike_scope();
            let s = snapshot(
                c,
                f.target,
                f.caster,
                DamageSource::BasicAttack,
                DamageType::Physical,
                100,
                0,
                1,
            );
            let loss = apply(c, f.caster, s, 1)?;
            c.end_strike_scope(old);
            Ok(loss)
        };
        let (result, admissions) =
            native::observe_retained_admissions(|| probe.runtime_transaction(action));
        assert_eq!(result.unwrap(), 73);
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "incoming_flat_observation")
            .unwrap()
            .2;
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = state(&f.core);
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        assert!(f.core.runtime_transaction(action).is_err());
        assert_eq!(state(&f.core), before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        assert_eq!(hit(&mut f, 100, 1), 73);
    }
    #[test]
    fn incoming_flat_parry_invulnerability_and_later_attack() {
        let mut f = setup(50);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.caster,
                    f.caster,
                    SustainedEffectDef::Parry {
                        duration_ms: 500,
                        window_ms: 200,
                        invulnerable_ms: 200,
                        miss_recovery_ms: 20,
                        reduction_percent: 0,
                        counter_skill: "r11_aura".into(),
                        end_on_resolution: false,
                        fragment_guard: None,
                        guard_charge: None,
                    },
                    0,
                )
            })
            .unwrap();
        assert_eq!(hit(&mut f, 100, 10), 0);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_incoming_flat
            .is_none());
        assert_eq!(hit(&mut f, 100, 110), 0);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_incoming_flat
            .is_none());
        assert_eq!(hit(&mut f, 100, 600), 73);
    }

    #[test]
    fn incoming_flat_actual_passive_guard_and_miss_recovery_keep_armor_reduction() {
        let authored =
            r12_parry_ron(5, false).replace("reduction_percent: 30", "reduction_percent: 0");
        let fragments:crate::game::data::skill_fragment_data::SkillFragmentDatabase=ron::de::from_str(
            &include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/r09/fragments.ron"))
                .replace("r08_strike","r11_aura").replace("ActiveSkill(forms:",
                    "ActiveSkill(passive_guard: Some((reduction_percent: 30, reflection_chance_percent: 0, reflection_percent: 50)), forms:")).unwrap();
        let a = crate::authoring::composition::compile_yaml(
            &SOURCE.replace("black_swan_nettle_coat", "r11_weapon"),
        )
        .unwrap();
        let (mut core, state, caster, ally) = r12_fragment_equipped_effect_core_with_data(
            &authored,
            |scenario| {
                scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(50000);
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                        unreachable!()
                    };
                    profile.fragment_source =
                        Some(crate::game::skill_fragment::FragmentSourceSnapshot {
                            copy_uuid: Uuid::from_u128(0x1850 + i as u128),
                            definition_id: "r09_fragment".into(),
                            form_id: "base".into(),
                            skill_id: "r11_aura".into(),
                        });
                }
            },
            |_| {},
            Some(fragments),
            |b| b.with_equipment_defenses(a.equipment_defenses().to_vec()),
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
            u.stats.attack_interval_ms = 100000;
            u.next_basic_attack_ms = 100000;
            u.stats.move_speed_units_per_ms = 0;
        }
        core.enable_authoring_observation();
        let mut f = Fixture {
            core,
            state,
            caster,
            target,
            ally,
        };
        assert!(f.core.passive_parry_guard(f.caster, 1).unwrap().is_some());
        assert_eq!(hit(&mut f, 100, 1), 51, "(83-10)*70%=51");
        f.core
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: f.caster,
                skill_id: "r11_aura".into(),
                target: None,
                time_ms: 10,
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 20)
            .unwrap();
        assert!(
            f.core
                .passive_parry_guard(f.caster, 20)
                .unwrap()
                .unwrap()
                .reduction_suppressed
        );
        assert_eq!(hit(&mut f, 100, 20), 73);
        assert_eq!(evidence(&f).used, 10);
        f.core
            .step_battle_execution_until(&mut f.state, 40)
            .unwrap();
        assert!(
            !f.core
                .passive_parry_guard(f.caster, 40)
                .unwrap()
                .unwrap()
                .reduction_suppressed
        );
        assert_eq!(hit(&mut f, 100, 40), 51);
    }
    #[test]
    fn incoming_flat_two_actual_copies_choose_strongest_and_new_strike_falls_back() {
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
        let mut armor = catalog.get_by_id("black_swan_nettle_coat").unwrap().clone();
        armor.id = "flat_armor".into();
        armor.triggered_effects.clear();
        let armor_id = armor.uuid;
        let mut doc = parse_yaml(&SOURCE.replace("black_swan_nettle_coat", "r11_weapon")).unwrap();
        let mut stronger = doc["values"][0].clone();
        stronger["name"] = json!("stronger");
        stronger["value"]["DefenseProgram"]["PhysicalBasicFlatReduction"]["amount"]["root"]
            ["Clamp"]["value"]["Ratio"]["numerator"]["Mul"]["scalar"]["Const"]["value"] = json!(75);
        doc["values"].as_array_mut().unwrap().push(stronger);
        doc["equipment_defenses"]
            .as_array_mut()
            .unwrap()
            .push(json!({"equipment":"flat_armor","program":{"Shared":"stronger"}}));
        let a = compile_json(&doc.to_string()).unwrap();
        let (mut core, state, caster, ally) = r12_fragment_equipped_effect_core_with_data(
            &r12_control_ron("Damage(amount:100,damage_type:True)"),
            |scenario| {
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    let mut second = spawn.draft.equipped_items[0];
                    second.base_uuid = armor_id;
                    second.persistent_copy_uuid = Some(Uuid::from_u128(0xe850 + i as u128));
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
        assert_eq!(hit(&mut f, 100, 1), 68);
        let chosen = evidence(&f).equipment;
        assert_eq!(chosen.definition_uuid, armor_id);
        let s = snapshot(
            &mut f.core,
            f.target,
            f.ally,
            DamageSource::BasicAttack,
            DamageType::Physical,
            100,
            0,
            2,
        );
        assert_eq!(
            f.core
                .runtime_transaction(|c| apply(c, f.ally, s, 2))
                .unwrap(),
            68
        );
        assert_ne!(
            chosen.persistent_copy_uuid,
            evidence(&f).equipment.persistent_copy_uuid
        );
        f.core.items.remove(&chosen.runtime_item_instance_id);
        assert_eq!(hit(&mut f, 100, 3), 73);
        assert_ne!(evidence(&f).equipment.definition_uuid, armor_id);
    }

    #[test]
    fn incoming_flat_defense_reference_also_drives_ordinary_authored_skill_damage() {
        let source = SOURCE.replace("black_swan_nettle_coat", "r11_weapon");
        let mut f=fixture_extra_with_defense(
            "DamageExpr(amount:(moment:Live,root:Read(Stat(subject:Source,stat:Defense))),damage_type:True)",
            false,None,false,&source,|_|{},|_|{});
        f.core.units.get_mut(&f.caster).unwrap().stats.defense = 20;
        let before = f.core.units[&f.target].stats.current_health;
        resolve(&mut f, 0);
        assert_eq!(before - f.core.units[&f.target].stats.current_health, 20);
        f.core.units.get_mut(&f.caster).unwrap().stats.defense = 35;
        let before = f.core.units[&f.target].stats.current_health;
        resolve(&mut f, 10);
        assert_eq!(before - f.core.units[&f.target].stats.current_health, 35);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_incoming_flat
            .is_none());
    }

    #[test]
    fn incoming_flat_unused_invalid_expression_cannot_fail_magic_but_physical_fault_rolls_back() {
        let mut doc = parse_yaml(&SOURCE.replace("black_swan_nettle_coat", "r11_weapon")).unwrap();
        doc["values"][0]["value"]["DefenseProgram"]["PhysicalBasicFlatReduction"]["amount"]
            ["root"] = json!({"Const":{"value":-1,"unit":"Magnitude"}});
        let mut f = fixture_extra_with_defense(
            "Damage(amount:100,damage_type:True)",
            false,
            None,
            false,
            &doc.to_string(),
            |_| {},
            |_| {},
        );
        let s = snapshot(
            &mut f.core,
            f.target,
            f.caster,
            DamageSource::BasicAttack,
            DamageType::Magic,
            100,
            0,
            1,
        );
        assert_eq!(
            f.core
                .runtime_transaction(|c| apply(c, f.caster, s, 1))
                .unwrap(),
            100
        );
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_incoming_flat
            .is_none());
        let before = state(&f.core);
        let result = f.core.runtime_transaction(|c| {
            let s = snapshot(
                c,
                f.target,
                f.caster,
                DamageSource::BasicAttack,
                DamageType::Physical,
                100,
                0,
                2,
            );
            apply(c, f.caster, s, 2)
        });
        assert!(matches!(result, Err(GameError::ValueExpressionFault(_))));
        assert_eq!(state(&f.core), before);
        let s = snapshot(
            &mut f.core,
            f.target,
            f.caster,
            DamageSource::BasicAttack,
            DamageType::True,
            100,
            0,
            3,
        );
        assert_eq!(
            f.core
                .runtime_transaction(|c| apply(c, f.caster, s, 3))
                .unwrap(),
            100
        );
    }
}
