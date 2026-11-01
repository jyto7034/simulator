mod composition_nearby_ally_damage {
    use super::*;
    use crate::game::{
        ability::DeliveryDef,
        battle::{
            damage::{BattleCommand, DamageSource, DamageType},
            tile_range::TileRangePolicy,
            types::BattleUnitSourceIdentity,
        },
        resources::Position,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/nearby_ally_damage.skill.yaml"
    ));
    fn setup(edit: impl FnOnce(&mut Value)) -> Fixture {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| {
                *doc = parse_yaml(SOURCE).unwrap();
                edit(doc);
            },
            |_| {},
            |scenario| {
                for i in 0..3 {
                    let mut extra = scenario.groups[1].spawns[1].clone();
                    extra.unit_ref = crate::game::battle::scenario::ScenarioUnitRef::new(format!(
                        "nearby_extra_{i}"
                    ));
                    extra.instance_salt += 770 + i;
                    extra.draft.owned_uuid = Uuid::from_u128(0x7700 + i as u128);
                    extra.draft.equipped_items[0].persistent_copy_uuid =
                        Some(Uuid::from_u128(0x77c0 + i as u128));
                    extra.draft.equipped_item_enhancements.clear();
                    extra.position = Position::new(7, i as i32);
                    scenario.groups[1].spawns.push(extra);
                }
            },
            None,
        );
        for u in f.core.units.values_mut() {
            u.stats.magic_resist = 0;
        }
        let u = f.core.units.get_mut(&f.a).unwrap();
        u.stats.attack = 100;
        u.basic_attack.delivery = DeliveryDef::Instant;
        u.basic_attack.windup_ms = 80;
        u.basic_attack.damage_type = DamageType::Physical;
        u.basic_attack.range_policy = TileRangePolicy::WholeFieldValidTiles;
        f.core.enable_authoring_observation();
        f
    }
    fn place(core: &mut BattleCore, actor: UnitInstanceId, x: i32, y: i32) {
        super::super::compiled_action_fixture::place(core, actor, x, y);
    }
    fn allies(f: &Fixture) -> Vec<UnitInstanceId> {
        let mut result: Vec<_> = f
            .core
            .units
            .iter()
            .filter_map(|(id, u)| {
                (*id != f.a
                    && u.owner == Side::Player
                    && matches!(u.source_identity, BattleUnitSourceIdentity::Employee { .. }))
                .then_some(*id)
            })
            .collect();
        result.sort();
        result
    }
    fn set_count(f: &mut Fixture, count: usize) {
        let allies = allies(f);
        assert_eq!(allies.len(), 4);
        for (i, id) in allies.into_iter().enumerate() {
            place(&mut f.core, id, if i < count { 2 } else { 7 }, 2);
        }
    }
    fn witness(
        core: &BattleCore,
    ) -> &crate::game::battle::core::nearby_ally_damage::NearbyAllyDamageWitness {
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_nearby_ally_damage
            .as_ref()
            .unwrap()
    }
    fn damage(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        kind: DamageType,
        source: DamageSource,
        reaction: bool,
        now: u64,
    ) {
        core.runtime_transaction(|c| {
            let mut snapshot = c
                .damage_source_snapshot_for_unit(
                    actor,
                    target,
                    source,
                    kind,
                    100,
                    Default::default(),
                    0,
                    now,
                    false,
                )
                .unwrap();
            snapshot.fact_source.reaction = reaction;
            c.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: target,
                    source_snapshot: snapshot,
                }],
                now,
            );
            crate::game::battle::runtime_control::poll()
        })
        .unwrap();
    }
    fn state(core: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
        )
    }
    #[test]
    fn composition_nearby_ally_damage_roundtrip_and_precise_expression_range_diagnostics() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(artifact.equipment_attacks()[0]
            .program
            .nearby_ally_damage
            .is_some());
        for bad in [
            SOURCE.replace("moment: Live", "moment: Snapshot"),
            SOURCE.replace("grouping: WholeSet", "grouping: ByProjectedTile"),
            SOURCE.replace("value: 10, unit: Scalar", "value: 10, unit: Milliseconds"),
            SOURCE.replace("\"XX@XX\"", "\"XXXXX\""),
        ] {
            let error = crate::authoring::composition::compile_yaml(&bad).unwrap_err();
            assert!(
                error.source.definition.contains("nearby_ally_damage"),
                "{error:?}"
            );
        }
        let f = setup(|_| {});
        let mut equipment = (*f.core.game_data.equipment_data).clone();
        let item = equipment
            .items
            .iter_mut()
            .find(|i| i.id == "r11_weapon")
            .unwrap();
        item.equipment_type = crate::game::data::equipment_data::EquipmentType::Armor;
        item.weapon_profile = None;
        crate::game::ability::equipment_attack::EquipmentAttackCatalog::compile(
            &equipment,
            &ResourceCatalog { resources: vec![] },
            &RelicDatabase::default(),
            &crate::game::value_expr::ExprLimits::default(),
        )
        .unwrap();
    }
    #[test]
    fn composition_nearby_ally_damage_current_zero_to_four_unique_employees_and_authored_combination(
    ) {
        for (rate, cap) in [(10, 3), (17, 2)] {
            let mut f = setup(|doc| {
                let r = &mut program(doc)["nearby_ally_damage"]["percent"]["root"]["Ratio"]
                    ["numerator"]["Mul"];
                r["scalar"]["Const"]["value"] = json!(rate);
                r["value"]["Clamp"]["max"]["Const"]["value"] = json!(cap);
            });
            for n in 0..=4 {
                set_count(&mut f, n);
                let before = f.core.units[&f.enemy].stats.current_health;
                damage(
                    &mut f.core,
                    f.a,
                    f.enemy,
                    DamageType::True,
                    DamageSource::Ability,
                    false,
                    n as u64 + 1,
                );
                let percent = n.min(cap) as u32 * rate;
                assert_eq!(
                    before - f.core.units[&f.enemy].stats.current_health,
                    100 + percent
                );
                let w = witness(&f.core);
                assert_eq!(w.percent, percent);
                assert_eq!(w.contributors[0].count, n as u32);
                assert_eq!(w.wearer, f.a);
                assert_eq!(w.actor, f.a);
                assert!(w.contributors[0]
                    .field_path
                    .ends_with("nearby_ally_damage.percent"));
                assert_eq!(
                    Some(w.contributors[0].equipment.persistent_copy_uuid.unwrap()),
                    match owner(&f.core, f.a) {
                        ResourceOwner::EquipmentCopy(id) => Some(id),
                        _ => None,
                    }
                );
                assert_eq!(f.core.units[&f.a].stats.attack, 100);
            }
        }
    }
    #[test]
    fn composition_nearby_ally_damage_all_types_and_owned_damage_channels_are_additive() {
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            for source in [
                DamageSource::BasicAttack,
                DamageSource::Ability,
                DamageSource::BuffTick,
                DamageSource::Environment,
            ] {
                for reaction in [false, true] {
                    let mut f = setup(|_| {});
                    set_count(&mut f, 2);
                    damage(&mut f.core, f.a, f.enemy, kind, source, reaction, 10);
                    assert_eq!(
                        f.core.units[&f.enemy].stats.current_health,
                        if source == DamageSource::Environment && !reaction {
                            900
                        } else {
                            880
                        }
                    );
                }
            }
        }
    }
    #[test]
    fn composition_nearby_ally_damage_excludes_self_hostile_dead_corroded_objects_and_summons() {
        let mut f = setup(|_| {});
        set_count(&mut f, 1);
        let id = allies(&f)[0];
        let original = f.core.units[&id].clone();
        let variants = [
            BattleUnitSourceIdentity::TestFixture {
                base_uuid: Uuid::nil(),
            },
            BattleUnitSourceIdentity::DefenseObject {
                base_uuid: Uuid::nil(),
            },
            BattleUnitSourceIdentity::SummonedUnit {
                definition_id: "fixture".into(),
                base_uuid: Uuid::nil(),
                summoner_instance_id: f.a,
                summon_sequence: 1,
                batch_uuid: Uuid::nil(),
            },
        ];
        for identity in variants {
            f.core.units.get_mut(&id).unwrap().source_identity = identity;
            damage(
                &mut f.core,
                f.a,
                f.enemy,
                DamageType::True,
                DamageSource::Ability,
                false,
                10,
            );
            assert_eq!(witness(&f.core).percent, 0);
        }
        f.core.units.insert(id, original.clone());
        f.core.units.get_mut(&id).unwrap().corroded = true;
        damage(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::True,
            DamageSource::Ability,
            false,
            11,
        );
        assert_eq!(witness(&f.core).percent, 0);
        f.core.units.insert(id, original.clone());
        f.core.units.get_mut(&id).unwrap().owner = Side::Opponent;
        damage(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::True,
            DamageSource::Ability,
            false,
            12,
        );
        assert_eq!(witness(&f.core).percent, 0);
        f.core.units.insert(id, original);
        f.core.units.get_mut(&id).unwrap().stats.current_health = 0;
        damage(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::True,
            DamageSource::Ability,
            false,
            13,
        );
        assert_eq!(witness(&f.core).percent, 0);
    }
    #[test]
    fn composition_nearby_ally_damage_samples_moving_ally_and_wearer_current_positions() {
        use crate::game::battle::core::movement::types::{ActiveMovementSegment, WorldVec2};
        let mut f = setup(|_| {});
        set_count(&mut f, 0);
        let id = allies(&f)[0];
        place(&mut f.core, id, 7, 2);
        f.core.active_movement_segments.insert(
            id,
            ActiveMovementSegment {
                start: WorldVec2::from_tile_center(Position::new(2, 2)),
                target: WorldVec2::from_tile_center(Position::new(7, 2)),
                started_at_ms: 10,
                ends_at_ms: 110,
            },
        );
        damage(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::True,
            DamageSource::Ability,
            false,
            10,
        );
        assert_eq!(witness(&f.core).contributors[0].count, 1);
        damage(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::True,
            DamageSource::Ability,
            false,
            110,
        );
        assert_eq!(witness(&f.core).contributors[0].count, 0);
        place(&mut f.core, f.a, 7, 1);
        f.core.active_movement_segments.insert(
            f.a,
            ActiveMovementSegment {
                start: WorldVec2::from_tile_center(Position::new(2, 1)),
                target: WorldVec2::from_tile_center(Position::new(7, 1)),
                started_at_ms: 110,
                ends_at_ms: 210,
            },
        );
        damage(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::True,
            DamageSource::Ability,
            false,
            110,
        );
        assert_eq!(witness(&f.core).contributors[0].count, 0);
        damage(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::True,
            DamageSource::Ability,
            false,
            210,
        );
        assert_eq!(witness(&f.core).contributors[0].count, 4);
    }
    #[test]
    fn composition_nearby_ally_damage_public_cancel_preserves_action_then_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(|_| {});
        set_count(&mut f, 2);
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 89)
            .unwrap();
        let before = (state(&f.core), format!("{:?}", f.state));
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("nearby_ally_damage_evaluated", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 90),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, (state(&f.core), format!("{:?}", f.state)));
        f.core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 880);
    }
    #[test]
    fn composition_nearby_ally_damage_clone_and_observation_preserve_damage_and_queue() {
        let mut f = setup(|_| {});
        set_count(&mut f, 2);
        attack(&mut f, 10, true);
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        let mut execution = f.state.clone();
        f.core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap();
        plain
            .step_battle_execution_until(&mut execution, 90)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 880);
        assert_eq!(plain.units[&f.enemy].stats.current_health, 880);
        assert_eq!(
            serde_json::to_value(&f.core.event_log).unwrap(),
            serde_json::to_value(&plain.event_log).unwrap()
        );
        assert_eq!(
            serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
            serde_json::to_value(plain.runtime_fact_view()).unwrap()
        );
        assert_eq!(
            f.core.event_queue_debug_snapshot(),
            plain.event_queue_debug_snapshot()
        );
    }
    #[test]
    fn composition_nearby_ally_damage_withdraw_after_release_keeps_projectile_without_live_bonus() {
        let mut f = setup(|_| {});
        set_count(&mut f, 2);
        f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery =
            ron::de::from_str("Projectile(speed_units_per_ms:6000)").unwrap();
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap();
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.a,
                time_ms: 91,
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 800)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 900);
    }

    #[test]
    fn composition_nearby_ally_damage_retained_refusal_restores_and_retries_with_finite_limit() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(|_| {});
        set_count(&mut f, 2);
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 89)
            .unwrap();
        let mut probe = f.core.clone();
        let mut execution = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut execution, 90)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "nearby_ally_damage_frame")
            .unwrap()
            .2;
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = (state(&f.core), format!("{:?}", f.state));
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "nearby_ally_damage_frame",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(before, (state(&f.core), format!("{:?}", f.state)));
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        f.core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 880);
    }
    #[test]
    fn composition_nearby_ally_damage_shared_definition_keeps_wearers_and_copies_distinct() {
        let mut f = setup(|_| {});
        set_count(&mut f, 0);
        // Three extras are near B, but outside A's range.
        damage(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::True,
            DamageSource::Ability,
            false,
            10,
        );
        assert_eq!(witness(&f.core).percent, 0);
        damage(
            &mut f.core,
            f.b,
            f.secondary,
            DamageType::True,
            DamageSource::Ability,
            false,
            11,
        );
        let w = witness(&f.core);
        assert_eq!(w.percent, 30);
        assert_eq!(w.wearer, f.b);
        assert_eq!(
            Some(w.contributors[0].equipment.persistent_copy_uuid.unwrap()),
            match owner(&f.core, f.b) {
                ResourceOwner::EquipmentCopy(id) => Some(id),
                _ => None,
            }
        );
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 900);
        assert_eq!(f.core.units[&f.secondary].stats.current_health, 870);
    }
    #[test]
    fn composition_nearby_ally_damage_actual_friend_explosion_uses_birth_attack_and_current_wearer_count(
    ) {
        for (enter, withdraw, expected) in
            [(true, false, 110), (false, false, 100), (true, true, 100)]
        {
            let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
            let doc = crate::authoring::node_graph::tests::fusion_placement_fixture(false);
            let graph =
                crate::authoring::node_graph::compile(&doc.to_string(), "r11_aura".into()).unwrap();
            let native =
                crate::authoring::node_graph::PortableGraph::from_ron(&graph.to_ron().unwrap())
                    .unwrap()
                    .load_content()
                    .unwrap();
            let (mut c, mut s, a, e, b) =
                super::super::compiled_action_fixture::fixture_content_with_data(
                    &[
                        "explosion-vulnerability", "widened-horizon", "summon-vulnerability",
                        "long-breath", "red-thread",
                    ],
                    &[],
                    native,
                    |_| {},
                    ResourceCatalog { resources: vec![] },
                    |builder| builder.with_equipment_attacks(artifact.equipment_attacks().to_vec()),
                    |scenario| {
                        scenario.terminal_outcomes =
                            BattleScenario::fixture_survival_outcomes(120000);
                        scenario.groups[0].enemy_movement_plan = None;
                        scenario.battlefield.object_placement =
                            Some(crate::game::battle::scenario::IndependentPlacementDomain {
                                ground: (0..8)
                                    .flat_map(|x| (0..4).map(move |y| Position::new(x, y)))
                                    .collect(),
                                reserved: vec![],
                            });
                        for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                            spawn.draft.equipped_items[0].persistent_copy_uuid =
                                Some(Uuid::from_u128(0x77e0 + i as u128));
                        }
                    },
                );
            for u in c.units.values_mut() {
                u.stats.attack = 100;
                u.stats.current_health = 1000;
                u.stats.max_health = 1000;
                u.stats.defense = 0;
                u.stats.magic_resist = 0;
                u.stats.move_speed_units_per_ms = 0;
                u.next_basic_attack_ms = 100000;
                u.stats.attack_interval_ms = 100000;
            }
            place(&mut c, a, 1, 1);
            place(&mut c, b, if enter { 7 } else { 1 }, 3);
            place(&mut c, e, 4, 2);
            c.enable_authoring_observation();
            c.apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: a,
                skill_id: "r11_aura".into(),
                target: Some(SkillCastTarget::Tile {
                    position: Position::new(3, 2),
                }),
                time_ms: 0,
            })
            .unwrap();
            c.step_battle_execution_until(&mut s, 0).unwrap();
            assert!(c
                .mobile_effects_snapshot()
                .unwrap()
                .effects
                .iter()
                .any(|effect| effect.owner == a));
            c.units.get_mut(&a).unwrap().stats.attack = 500;
            place(&mut c, b, if enter { 1 } else { 7 }, 3);
            if withdraw {
                c.apply_live_command(BattleLiveCommand::WithdrawUnit {
                    unit_id: a,
                    time_ms: 50,
                })
                .unwrap();
            }
            c.step_battle_execution_until(&mut s, 200).unwrap();
            assert_eq!(
                c.units[&e].stats.current_health,
                1000 - expected,
                "enter={enter} withdraw={withdraw}"
            );
            if !withdraw {
                assert_eq!(witness(&c).wearer, a);
                assert_eq!(witness(&c).percent, if enter { 10 } else { 0 });
            }
        }
    }

    #[test]
    fn composition_nearby_ally_damage_checked_object_creator_and_direct_employee_take_correct_copy()
    {
        use crate::game::battle::runtime_facts::{
            IndependentObjectOriginSnapshot, RuntimeFactSource,
        };
        let mut f = setup(|_| {});
        for id in allies(&f) {
            place(&mut f.core, id, 7, 3);
        }
        let selected: Vec<_> = allies(&f)
            .into_iter()
            .filter(|id| *id != f.b)
            .take(2)
            .collect();
        for id in selected {
            place(&mut f.core, id, 2, 2);
        }
        let object = UnitInstanceId(Uuid::from_u128(0x770b));
        for (direct, matching, expected) in
            [(true, false, 145), (false, true, 145), (false, false, 125)]
        {
            let before = f.core.units[&f.enemy].stats.current_health;
            f.core
                .runtime_transaction(|c| {
                    let mut snapshot = c
                        .damage_source_snapshot_for_unit(
                            f.a,
                            f.enemy,
                            DamageSource::Ability,
                            DamageType::True,
                            100,
                            crate::game::battle::damage::DamageModifiers {
                                damage_amp_percent: 25,
                                ..Default::default()
                            },
                            0,
                            10,
                            false,
                        )
                        .unwrap();
                    if !direct {
                        snapshot.source_id = object;
                        snapshot.fact_source.actor = Some(object);
                    }
                    snapshot.fact_source.extended_observation =
                        RuntimeFactSource::extended_observation(
                            Some(std::sync::Arc::new(IndependentObjectOriginSnapshot {
                                object_id: if matching {
                                    object
                                } else {
                                    UnitInstanceId(Uuid::from_u128(0x770c))
                                },
                                definition_id:
                                    crate::game::data::independent_object_data::IndependentObjectId(
                                        "fixture-owned".into(),
                                    ),
                                creator: if direct { f.b } else { f.a },
                                creation_action: None,
                                creation_cast: None,
                                creation_fragment: None,
                                creation_equipment: None,
                                creation_authored: None,
                            })),
                            None,
                        )?;
                    c.process_commands(
                        vec![BattleCommand::ApplyDamage {
                            target_id: f.enemy,
                            source_snapshot: snapshot,
                        }],
                        10,
                    );
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
            assert_eq!(
                before - f.core.units[&f.enemy].stats.current_health,
                expected
            );
            if expected == 145 {
                assert_eq!(witness(&f.core).wearer, f.a);
                assert_eq!(witness(&f.core).percent, 20);
            }
        }
    }
}
