mod composition_wearer_reduction {
    use super::*;
    use crate::game::{
        battle::damage::{DamageModifiers, DamageSourceSnapshot},
        behavior::GameError,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/wearer_reduction.skill.yaml"
    ));
    fn setup(cap: u32) -> Fixture {
        setup_source(
            &SOURCE
                .replace("placeholder_schadenfreude_armor", "r11_weapon")
                .replace("value: 50,", &format!("value: {cap},")),
        )
    }
    fn setup_source(source: &str) -> Fixture {
        fixture_extra_with_defense(
            "Damage(amount:100,damage_type:True)",
            false,
            None,
            false,
            source,
            |_| {},
            |_| {},
        )
    }
    fn snap(
        c: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        source: DamageSource,
        kind: DamageType,
        now: u64,
    ) -> DamageSourceSnapshot {
        c.damage_source_snapshot_for_unit(
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
        c: &mut BattleCore,
        target: UnitInstanceId,
        source: DamageSourceSnapshot,
        now: u64,
    ) -> Result<u32, GameError> {
        let before = c.units[&target].stats.current_health;
        c.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id: target,
                source_snapshot: source,
            }],
            now,
        );
        crate::game::battle::runtime_control::poll()?;
        Ok(before - c.units[&target].stats.current_health)
    }
    fn hit(f: &mut Fixture, now: u64) -> u32 {
        f.core
            .runtime_transaction(|c| {
                let s = snap(
                    c,
                    f.target,
                    f.caster,
                    DamageSource::Ability,
                    DamageType::True,
                    now,
                );
                apply(c, f.caster, s, now)
            })
            .unwrap()
    }
    fn witness(
        c: &BattleCore,
    ) -> &crate::game::battle::core::equipment_defense::WearerReductionWitness {
        c.authoring_observation
            .as_ref()
            .unwrap()
            .latest_wearer_reduction
            .as_ref()
            .unwrap()
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
    fn wearer_reduction_schema_roundtrip_source_references_and_errors() {
        let a = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        assert!(matches!(
            a.equipment_defenses()[0].program,
            crate::game::ability::equipment_defense::EquipmentDefenseProgram::WearerReduction { .. }
        ));
        for bad in [
            SOURCE.replace("moment: Live", "moment: Snapshot"),
            SOURCE.replace("subject: Source", "subject: Target"),
            SOURCE.replace("value: 50, unit: Scalar", "value: 50, unit: Milliseconds"),
        ] {
            let error = crate::authoring::composition::compile_yaml(&bad).unwrap_err();
            assert!(
                format!("{error:?}").contains("WearerReduction"),
                "{error:?}"
            );
        }
        let mut f = setup(50);
        hit(&mut f, 1);
        assert!(witness(&f.core)
            .field_path
            .ends_with("WearerReduction.percent"));
    }
    #[test]
    fn wearer_reduction_current_missing_hp_floor_cap_and_maximum() {
        for cap in [50, 35] {
            let mut f = setup(cap);
            for (hp, missing_percent) in [
                (1000, 0),
                (999, 0),
                (750, 25),
                (500, 50),
                (100, 90),
                (900, 10),
            ] {
                f.core
                    .units
                    .get_mut(&f.caster)
                    .unwrap()
                    .stats
                    .current_health = hp;
                assert_eq!(hit(&mut f, 1), 100 - missing_percent.min(cap));
                assert_eq!(
                    (
                        witness(&f.core).current_hp,
                        witness(&f.core).max_hp,
                        witness(&f.core).percent
                    ),
                    (hp, 1000, missing_percent.min(cap))
                );
            }
            let u = f.core.units.get_mut(&f.caster).unwrap();
            u.stats.max_health = 2000;
            u.stats.current_health = 1500;
            assert_eq!(hit(&mut f, 2), 75);
        }
    }
    #[test]
    fn wearer_reduction_actual_healing_lowers_rate_and_raw_cost_is_unreduced() {
        let mut f = setup(50);
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 400;
        assert_eq!(hit(&mut f, 1), 50);
        f.core
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(f.ally), f.caster, 550, 2))
            .unwrap();
        assert_eq!(f.core.units[&f.caster].stats.current_health, 900);
        assert_eq!(hit(&mut f, 3), 90);
        f.core.process_commands(
            vec![BattleCommand::PayHealthCost {
                caster_id: f.caster,
                amount: 100,
                allow_lethal: false,
            }],
            4,
        );
        assert_eq!(f.core.units[&f.caster].stats.current_health, 710);
        assert_eq!(
            witness(&f.core).time_ms,
            3,
            "raw cost must not enter the percentage path"
        );
    }
    #[test]
    fn wearer_reduction_all_normal_types_channels_and_attacker_lifetime() {
        for source in [
            DamageSource::BasicAttack,
            DamageSource::Ability,
            DamageSource::BuffTick,
            DamageSource::Environment,
        ] {
            for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
                let mut f = setup(50);
                f.core
                    .units
                    .get_mut(&f.caster)
                    .unwrap()
                    .stats
                    .current_health = 500;
                let s = snap(&mut f.core, f.target, f.caster, source, kind, 1);
                f.core
                    .units
                    .get_mut(&f.target)
                    .unwrap()
                    .stats
                    .current_health = 0;
                assert_eq!(
                    f.core
                        .runtime_transaction(|c| apply(c, f.caster, s, 1))
                        .unwrap(),
                    50
                );
            }
        }
        for own in [true, false] {
            let mut f = setup(50);
            f.core
                .units
                .get_mut(&f.caster)
                .unwrap()
                .stats
                .current_health = 500;
            let actor = if own { f.caster } else { f.ally };
            let mut s = snap(
                &mut f.core,
                actor,
                f.caster,
                DamageSource::Environment,
                DamageType::True,
                1,
            );
            s.fact_source.reaction = true;
            assert_eq!(
                f.core
                    .runtime_transaction(|c| apply(c, f.caster, s, 1))
                    .unwrap(),
                50
            );
        }
    }
    #[test]
    fn wearer_reduction_one_strike_freezes_hp_then_next_strike_recomputes() {
        let mut f = setup(50);
        f.core
            .runtime_transaction(|c| {
                let old = c.begin_strike_scope();
                for _ in 0..2 {
                    let s = snap(
                        c,
                        f.target,
                        f.caster,
                        DamageSource::Ability,
                        DamageType::True,
                        1,
                    );
                    assert_eq!(apply(c, f.caster, s, 1)?, 100);
                    assert_eq!((witness(c).current_hp, witness(c).percent), (1000, 0));
                }
                c.end_strike_scope(old);
                Ok(())
            })
            .unwrap();
        assert_eq!(f.core.units[&f.caster].stats.current_health, 800);
        assert_eq!(hit(&mut f, 2), 80);
    }
    #[test]
    fn wearer_reduction_current_copy_removal_reequip_and_corrosion() {
        let mut f = setup(50);
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 500;
        assert_eq!(hit(&mut f, 1), 50);
        let equipped = f.core.items.clone();
        f.core.items.clear();
        assert_eq!(hit(&mut f, 2), 100);
        f.core.items = equipped;
        assert_eq!(hit(&mut f, 3), 50);
        f.core.units.get_mut(&f.caster).unwrap().corroded = true;
        assert_eq!(hit(&mut f, 4), 100);
    }
    #[test]
    fn wearer_reduction_cancel_after_evaluation_or_hp_change_restores_and_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "wearer_reduction_evaluated",
            "wearer_reduction_test_after_damage",
        ] {
            let mut f = setup(50);
            f.core
                .units
                .get_mut(&f.caster)
                .unwrap()
                .stats
                .current_health = 500;
            let before = state(&f.core);
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || {
                        f.core.runtime_transaction(|c| {
                            let s = snap(
                                c,
                                f.target,
                                f.caster,
                                DamageSource::Ability,
                                DamageType::True,
                                1,
                            );
                            let loss = apply(c, f.caster, s, 1)?;
                            control::poll("wearer_reduction_test_after_damage")?;
                            Ok(loss)
                        })
                    },
                )
            });
            assert!(result.is_err() && token.is_cancelled());
            assert_eq!(state(&f.core), before);
            assert_eq!(hit(&mut f, 1), 50);
        }
    }
    #[test]
    fn wearer_reduction_retained_refusal_restores_and_retries() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(50);
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 500;
        let (actor, target) = (f.target, f.caster);
        let action = |c: &mut BattleCore| {
            let s = snap(c, actor, target, DamageSource::Ability, DamageType::True, 1);
            apply(c, target, s, 1)
        };
        let mut probe = f.core.clone();
        let (result, admissions) =
            native::observe_retained_admissions(|| probe.runtime_transaction(action));
        assert_eq!(result.unwrap(), 50);
        let required = admissions
            .iter()
            .find(|(s, _, _)| *s == "wearer_reduction_observation")
            .unwrap()
            .2;
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = state(&f.core);
        assert!(f.core.runtime_transaction(action).is_err());
        assert_eq!(state(&f.core), before);
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        assert_eq!(hit(&mut f, 1), 50);
    }
    #[test]
    fn wearer_reduction_invalid_runtime_percentage_rolls_back() {
        let mut doc =
            parse_yaml(&SOURCE.replace("placeholder_schadenfreude_armor", "r11_weapon")).unwrap();
        for value in [-1, 100] {
            doc["values"][0]["value"]["DefenseProgram"]["WearerReduction"]["percent"]["root"] =
                json!({"Const":{"value":value,"unit":"Scalar"}});
            let mut f = setup_source(&doc.to_string());
            let before = state(&f.core);
            let result = f.core.runtime_transaction(|c| {
                let s = snap(
                    c,
                    f.target,
                    f.caster,
                    DamageSource::Ability,
                    DamageType::True,
                    1,
                );
                apply(c, f.caster, s, 1)
            });
            assert!(result.is_err());
            assert_eq!(state(&f.core), before);
        }
    }
    #[test]
    fn wearer_reduction_observation_disabled_clone_keeps_damage_and_source() {
        let mut f = setup(50);
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 500;
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        let s = snap(
            &mut plain,
            f.target,
            f.caster,
            DamageSource::Ability,
            DamageType::True,
            1,
        );
        assert_eq!(
            plain
                .runtime_transaction(|c| apply(c, f.caster, s, 1))
                .unwrap(),
            hit(&mut f, 1)
        );
        assert_eq!(
            plain.transaction_debug_snapshot(),
            f.core.transaction_debug_snapshot()
        );
        assert_eq!(
            serde_json::to_value(plain.runtime_fact_summary()).unwrap(),
            serde_json::to_value(f.core.runtime_fact_summary()).unwrap()
        );
        assert!(plain.authoring_observation.is_none());
        assert_eq!(witness(&f.core).incoming.actor, Some(f.target));
    }
    #[test]
    fn wearer_reduction_lethal_damage_does_not_create_hp_floor() {
        let mut f = setup(50);
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .stats
            .current_health = 1;
        assert_eq!(hit(&mut f, 1), 1);
        assert_eq!(f.core.units[&f.caster].stats.current_health, 0);
    }
    #[test]
    fn wearer_reduction_actual_copies_strongest_tie_and_fallback() {
        use crate::game::data::{equipment_data::EquipmentDatabase, GameDataBuilder};
        for cap in [50, 75] {
            let catalog: EquipmentDatabase = ron::de::from_str(include_str!(concat!(
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
            let mut armor = catalog
                .get_by_id("placeholder_schadenfreude_armor")
                .unwrap()
                .clone();
            armor.id = "wearer_armor".into();
            armor.triggered_effects.clear();
            let armor_id = armor.uuid;
            let mut doc =
                parse_yaml(&SOURCE.replace("placeholder_schadenfreude_armor", "r11_weapon"))
                    .unwrap();
            let mut second = doc["values"][0].clone();
            second["name"] = json!("second");
            second["value"]["DefenseProgram"]["WearerReduction"]["percent"]["root"]["Clamp"]
                ["max"]["Const"]["value"] = json!(cap);
            doc["values"].as_array_mut().unwrap().push(second);
            doc["equipment_defenses"]
                .as_array_mut()
                .unwrap()
                .push(json!({"equipment":"wearer_armor","program":{"Shared":"second"}}));
            let a = compile_json(&doc.to_string()).unwrap();
            let (mut core, state, caster, ally) = r12_fragment_equipped_effect_core_with_data(
                &r12_control_ron("Damage(amount:100,damage_type:True)"),
                |scenario| {
                    for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                        let mut item = spawn.draft.equipped_items[0];
                        item.base_uuid = armor_id;
                        item.persistent_copy_uuid = Some(Uuid::from_u128(0xe910 + i as u128));
                        spawn.draft.equipped_items.push(item);
                    }
                },
                |_| {},
                None,
                |b: GameDataBuilder| {
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
                u.stats.max_health = 10000;
                u.stats.current_health = 2500;
                u.stats.defense = 0;
                u.stats.magic_resist = 0;
            }
            core.enable_authoring_observation();
            let mut f = Fixture {
                core,
                state,
                caster,
                ally,
                target,
            };
            assert_eq!(hit(&mut f, 1), 100 - cap);
            let chosen = witness(&f.core).equipment;
            if cap == 50 {
                let least = f
                    .core
                    .items
                    .values()
                    .filter(|i| i.owner_unit_instance == caster)
                    .map(|i| i.instance_id)
                    .min()
                    .unwrap();
                assert_eq!(chosen.runtime_item_instance_id, least);
            } else {
                assert_eq!(chosen.definition_uuid, armor_id);
                f.core.items.remove(&chosen.runtime_item_instance_id);
                assert_eq!(hit(&mut f, 2), 50);
                assert_ne!(witness(&f.core).equipment.definition_uuid, armor_id);
            }
        }
    }
}
