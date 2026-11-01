mod composition_mountain {
    use super::*;
    use crate::game::{
        ability::sustained::ControlKind,
        battle::{
            damage::{BattleCommand, DamageSource, DamageType},
            tile_range::TileRangePattern,
        },
        data::{equipment_data::WeaponArchetype, ValidatedGameData},
        relic::{
            structural::{compile_effective_plan, EffectivePlanInput, ProspectiveProvider},
            RelicInventory,
        },
    };
    const RELICS: [&str; 5] = [
        "widened-horizon",
        "long-breath",
        "taunted-enemy-damage-reduction",
        "taunted-enemy-basic-attack-boost",
        "double-harvest",
    ];
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/mountain.skill.yaml"
    ));

    fn fixture(selected: &[usize], owner_is_actor: bool, initial: u32) -> Fixture {
        fixture_mask(
            selected,
            owner_is_actor,
            initial,
            TileRangePattern {
                include_anchor_tile: true,
                rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
            },
        )
    }

    fn fixture_mask(
        selected: &[usize],
        owner_is_actor: bool,
        initial: u32,
        mask: TileRangePattern,
    ) -> Fixture {
        let source = SOURCE.replace("authored-mountain", "r11_aura");
        let artifact = crate::authoring::composition::compile_yaml(&source).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let native = artifact.load_native().unwrap();
        // This in-memory fixture bypasses the pack loader. Supply the authored
        // file identity explicitly instead of using the typed-program fallback.
        let mut equipment_attacks = artifact.equipment_attacks().to_vec();
        for (index, binding) in equipment_attacks.iter_mut().enumerate() {
            binding.source = Some((
                format!("composition:mountain-combination-fixture:{}", artifact.source_digest()),
                format!("equipment_attacks[{index}]"),
            ));
        }
        assert_eq!(
            native.skills[0]
                .action_policy
                .as_ref()
                .unwrap()
                .modifiers
                .len(),
            4
        );
        let mut relics: RelicDatabase = ron::de::from_str(r#"(relics:[(id:"placeholder",uuid:"00000000-0000-0000-0000-00000000d290",name:"Combination fixture",description:"Explicit test values",rarity:"fixture",allow_duplicates:true,effects:[(id:"unused",selector:(all_tags:["unused_fixture"]),percent:1)],dismantle_equipment_dust:1)])"#).unwrap();
        let template = relics.relics.pop().unwrap();
        for (index, id) in RELICS.iter().enumerate() {
            let mut relic = template.clone();
            relic.id = (*id).into();
            relic.uuid = Uuid::from_u128(0xd290 + index as u128);
            relics.relics.push(relic);
        }
        let fragments = ron::de::from_str(
            &include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/r09/fragments.ron"
            ))
            .replace("r08_strike", "r11_aura"),
        )
        .unwrap();
        let (mut core, state, a, b) = r12_fragment_equipped_effect_core_with_data(
            &ron::ser::to_string(&native).unwrap(),
            |scenario| {
                scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(8000);
                scenario.groups[0].enemy_movement_plan = None;
                let mut secondary = scenario.groups[0].spawns[0].clone();
                secondary.unit_ref =
                    crate::game::battle::scenario::ScenarioUnitRef::new("mountain-secondary");
                secondary.instance_salt += 100;
                secondary.position = crate::game::resources::Position::new(3, 2);
                scenario.groups[0].spawns.push(secondary);
                for (index, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    spawn.draft.equipped_items[0].persistent_copy_uuid =
                        Some(Uuid::from_u128(0xe290 + index as u128));
                    let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                        unreachable!()
                    };
                    profile.fragment_source =
                        Some(crate::game::skill_fragment::FragmentSourceSnapshot {
                            copy_uuid: Uuid::from_u128(0xf290 + index as u128),
                            definition_id: "r09_fragment".into(),
                            form_id: "base".into(),
                            skill_id: "r11_aura".into(),
                        });
                }
            },
            |weapon| {
                let profile = weapon.weapon_profile.as_mut().unwrap();
                profile.weapon_archetype = WeaponArchetype::Shield;
                profile.block_capacity = 1;
                profile.defense_tile_range = TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                };
                profile.action =
                    crate::game::data::abnormality_data::BasicAttackAction::DamageArea {
                        footprint: mask,
                        secondary_percent: 40,
                    };
            },
            Some(fragments),
            |builder| {
                builder
                    .with_resources(ResourceCatalog {
                        resources: artifact.resources().to_vec(),
                    })
                    .with_equipment_attacks(equipment_attacks)
                    .with_relics(relics)
            },
        );
        let mut enemies: Vec<_> = core
            .units
            .values()
            .filter(|u| u.owner == Side::Opponent)
            .map(|u| (u.spawn_order, u.instance_id))
            .collect();
        enemies.sort();
        let enemy = enemies[0].1;
        let secondary = enemies[1].1;
        for unit in core.units.values_mut() {
            unit.stats.attack_interval_ms = 100_000;
            unit.next_basic_attack_ms = 100_000;
            unit.stats.move_speed_units_per_ms = 0;
            unit.body.move_speed = 0.0;
            unit.stats.defense = 0;
            unit.stats.current_health = 1000;
            unit.stats.max_health = 1000;
            unit.threat_class = BattleUnitThreatClass::Normal;
        }
        core.units.get_mut(&a).unwrap().stats.current_health = 500;
        for (id, x, y) in [(a, 2, 1), (b, 7, 3), (enemy, 3, 1), (secondary, 3, 2)] {
            super::super::compiled_action_fixture::place(&mut core, id, x, y);
        }
        let owners = core
            .items
            .values()
            .filter_map(|i| i.persistent_copy_uuid)
            .map(ResourceOwner::EquipmentCopy)
            .collect();
        let run = RunResourceState {
            revision: 0,
            closed: false,
            entries: vec![RunResourceEntry {
                resource: artifact.resources()[0].id.clone(),
                owner: owner(&core, a),
                balance: initial,
                revision: 0,
            }],
        };
        core.mount_run_resources(&run, &owners).unwrap();
        let data: ValidatedGameData = core.game_data.clone().into();
        let mut inventory = RelicInventory::default();
        let mut ids = crate::game::managers::uuid_manager::UuidManager::new(0xa290);
        for (slot, index) in selected.iter().enumerate() {
            let copy = inventory
                .grant(data.relic_data.get_by_id(RELICS[*index]).unwrap(), &mut ids)
                .unwrap();
            inventory
                .equip(
                    copy,
                    core.units[&if owner_is_actor { a } else { b }].source_owned_uuid,
                    slot,
                    &data.relic_data,
                )
                .unwrap();
        }
        let numeric = crate::game::battle::core::relic_numeric::RelicNumericOwner::capture(
            &inventory,
            core.game_data.clone(),
            core.content_identity.as_ref().unwrap().clone(),
        )
        .unwrap();
        core.install_relic_numeric_owner(numeric).unwrap();
        let mut plans = std::collections::BTreeMap::new();
        for actor in [a, b] {
            let employee = core.units[&actor].source_owned_uuid;
            let providers = [ProspectiveProvider::Fragment(
                core.units[&actor]
                    .fragment_source
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .clone(),
            )];
            if let Some(plan) = compile_effective_plan(EffectivePlanInput {
                data: &data,
                employee_uuid: employee,
                inventory: &inventory,
                providers: &providers,
            })
            .unwrap()
            {
                plans.insert(employee, plan);
            }
        }
        core.set_effective_owner(
            crate::game::battle::core::effective_skill::EffectiveBattleOwner::new(plans).unwrap(),
        )
        .unwrap();
        Fixture {
            core,
            state,
            a,
            b,
            enemy,
            secondary,
            run,
        }
    }
    fn cast(f: &mut Fixture) {
        super::super::compiled_action_fixture::cast(&mut f.core, f.a, 0);
        f.core.step_battle_execution_until(&mut f.state, 1).unwrap();
        assert_eq!(balance(f, f.a), 3, "taunt is not a growth event");
        assert_eq!(
            f.core.units[&f.a].stats.current_health, 500,
            "no removed armor healing"
        );
        for enemy in [f.enemy, f.secondary] {
            assert_eq!(
                f.core.units[&enemy].stats.current_health, 1000,
                "taunt does no damage"
            );
        }
        assert!(!f.core.resonance_use_has_obligations(f.a));
    }
    #[test]
    fn composition_mountain_shield_primary_secondary_growth_and_owner() {
        for (selected, owner_is_actor, boost, grant) in [
            (&[][..], true, false, 2),
            (&[2, 3, 4][..], false, false, 2),
            (&[2, 3, 4][..], true, true, 4),
            (&[0, 1, 4][..], true, false, 4),
        ] {
            for (primary_hp, secondary_hp) in [(1000u32, 1000u32), (1, 1000), (1000, 1), (1, 1)] {
                let mut f = fixture(selected, owner_is_actor, 3);
                cast(&mut f);
                f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = primary_hp;
                f.core
                    .units
                    .get_mut(&f.secondary)
                    .unwrap()
                    .stats
                    .current_health = secondary_hp;
                attack(&mut f, 10, true);
                f.core
                    .step_battle_execution_until(&mut f.state, 12)
                    .unwrap();
                // 100 base + 3*10 primary, floor(100*40%) + floor(3*10*40%) secondary.
                // Taunt boost applies to the entire eligible direct hit after growth.
                let primary_damage = if boost { 169 } else { 130 };
                let secondary_damage = if boost { 67 } else { 52 };
                assert_eq!(
                    f.core.units[&f.enemy].stats.current_health,
                    primary_hp.saturating_sub(primary_damage)
                );
                assert_eq!(
                    f.core.units[&f.secondary].stats.current_health,
                    secondary_hp.saturating_sub(secondary_damage)
                );
                assert_eq!(
                    balance(&f, f.a),
                    3 + if primary_hp == 1 { grant } else { 0 }
                );
                assert_eq!(balance(&f, f.b), 0);
                assert_eq!(f.core.units[&f.a].stats.current_health, 500);
            }
        }
    }
    #[test]
    fn composition_mountain_damage_area_direction_hole_and_ally_filter() {
        use crate::game::battle::tile_range::FacingDirection;
        for (facing, x, y, hit) in [
            (FacingDirection::Up, 4, 2, true),
            (FacingDirection::Right, 3, 3, true),
            (FacingDirection::Down, 2, 2, true),
            (FacingDirection::Left, 3, 1, true),
            (FacingDirection::Up, 4, 3, false),
        ] {
            let mut f = fixture_mask(
                &[],
                true,
                3,
                TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec!["@X".into(), "..".into()],
                },
            );
            for (id, x, y) in [
                (f.a, 4, 1),
                (f.enemy, 3, 2),
                (f.secondary, x, y),
                (f.b, 3, 2),
            ] {
                super::super::compiled_action_fixture::place(&mut f.core, id, x, y);
            }
            f.core.units.get_mut(&f.a).unwrap().facing_direction = Some(facing);
            attack(&mut f, 10, true);
            f.core
                .step_battle_execution_until(&mut f.state, 12)
                .unwrap();
            assert_eq!(
                f.core.units[&f.enemy].stats.current_health, 870,
                "primary {facing:?}"
            );
            assert_eq!(
                f.core.units[&f.secondary].stats.current_health,
                if hit { 948 } else { 1000 },
                "secondary {facing:?}"
            );
            assert_eq!(
                f.core.units[&f.b].stats.current_health, 1000,
                "ally shares the hit tile"
            );
            assert_eq!(balance(&f, f.a), 3);
        }
    }

    #[test]
    fn composition_mountain_area_cancellation_rolls_back_and_observation_is_inert() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut outcomes = Vec::new();
        for observe in [false, true] {
            for cancel_site in [
                None,
                Some(("equipment_attack_damage", 1)),
                Some(("equipment_attack_damage", 2)),
                Some(("equipment_attack_reacted", 1)),
            ] {
                let mut f = fixture(&[2, 3, 4], true, 3);
                if observe {
                    f.core
                        .enable_authoring_observation_with_options(Default::default(), true);
                    f.core
                        .bind_authoring_observation(
                            crate::game::battle::core::authoring_observation::ObservationKey {
                                entry_ordinal: 1,
                                battle_uuid: Uuid::from_u128(0xb29b),
                            },
                        )
                        .unwrap();
                }
                cast(&mut f);
                f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
                attack(&mut f, 10, true);
                f.core
                    .step_battle_execution_until(&mut f.state, 10)
                    .unwrap();
                if let Some((site, occurrence)) = cancel_site {
                    let snapshot = |f: &Fixture| {
                        (
                            f.core.transaction_debug_snapshot(),
                            f.core.event_queue_debug_snapshot(),
                            serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
                            format!("{:?}", f.state),
                            balance(f, f.a),
                        )
                    };
                    let before = snapshot(&f);
                    let token = CancellationToken::default();
                    let result = control::with_test_cancellation(site, occurrence, || {
                        control::run(
                            &ExecutionLimits::default(),
                            &token,
                            ExecutionPhase::TrialExecution,
                            || f.core.step_battle_execution_until(&mut f.state, 12),
                        )
                    });
                    assert!(result.is_err() && token.is_cancelled(), "{site}");
                    assert_eq!(snapshot(&f), before, "{site}");
                }
                f.core
                    .step_battle_execution_until(&mut f.state, 12)
                    .unwrap();
                assert_eq!(balance(&f, f.a), 7);
                assert_eq!(f.core.units[&f.secondary].stats.current_health, 933);
                assert_eq!(f.core.units[&f.a].stats.current_health, 500);
                outcomes.push((
                    serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
                    balance(&f, f.a),
                    f.core.sustained_effect_snapshot().unwrap(),
                ));
            }
        }
        for result in &outcomes[1..] {
            assert_eq!(result, &outcomes[0]);
        }
    }

    #[test]
    fn composition_mountain_area_preserves_weapon_air_filter() {
        let mut f = fixture(&[], true, 3);
        f.core.units.get_mut(&f.secondary).unwrap().mobility_kind =
            crate::game::battle::types::MobilityKind::Airborne;
        assert!(!f.core.units[&f.a].basic_attack.air_capable);
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 12)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 870);
        assert_eq!(f.core.units[&f.secondary].stats.current_health, 1000);
    }

    #[test]
    fn composition_mountain_duration_range_and_damage_expire_together() {
        for (selected, duration, taken) in [([0, 1, 4], 2502, 100), ([2, 3, 4], 2001, 80)] {
            let mut f = fixture(&selected, true, 3);
            cast(&mut f);
            let snapshot = f.core.sustained_effect_snapshot().unwrap();
            let row = snapshot
                .active
                .iter()
                .find(|r| r.target == f.enemy)
                .unwrap();
            assert_eq!(row.expires_at_ms, duration + 1);
            assert!(matches!(
                row.control,
                Some(ControlKind::ApproachTaunt { .. })
            ));
            let hit = |f: &mut Fixture, now| {
                let before = f.core.units[&f.a].stats.current_health;
                f.core
                    .runtime_transaction(|core| {
                        let source = core
                            .damage_source_snapshot_for_unit(
                                f.enemy,
                                f.a,
                                DamageSource::Ability,
                                DamageType::Physical,
                                100,
                                Default::default(),
                                0,
                                now,
                                false,
                            )
                            .unwrap();
                        core.process_commands(
                            vec![BattleCommand::ApplyDamage {
                                target_id: f.a,
                                source_snapshot: source,
                            }],
                            now,
                        );
                        crate::game::battle::runtime_control::poll()
                    })
                    .unwrap();
                before - f.core.units[&f.a].stats.current_health
            };
            assert_eq!(hit(&mut f, 2), taken);
            f.core
                .step_battle_execution_until(&mut f.state, duration + 1)
                .unwrap();
            assert!(f
                .core
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .iter()
                .all(|r| r.control.is_none()));
            assert_eq!(hit(&mut f, duration + 2), 100);
            assert_eq!(balance(&f, f.a), 3);
        }
    }
    include!("basic_weapon_source_tests.inc.rs");
}
