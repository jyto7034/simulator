mod composition_metamorphosis {
    use super::*;
    use crate::game::battle::{
        core::types::RuntimeUnitLifecycle,
        runtime_facts::DeathCause,
        types::{BattleUnitSourceIdentity, BattleVerdict, TacticalTileKind},
    };
    const TRANSFORM: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/equipment_metamorphosis.skill.yaml"
    ));
    fn form() -> crate::game::data::abnormality_data::AbnormalityMetadata {
        let mut meta: crate::game::data::abnormality_data::AbnormalityMetadata = ron::de::from_str(r#"(
            id: "revival_form", uuid: "00000000-0000-0000-0000-00000000e690", name: "변신 검증 형태",
            risk_level: ZAYIN, price: 0, max_health: 333, attack: 17, defense: 0,
            response_complete_skill_fragment_id: Some("r09_fragment"),
            movement: (speed_units_per_ms: 0)
        )"#).unwrap();
        // A stationary ranged form can attack adjacent employees without the
        // fixed-defense melee rule requiring a blocker or route-end objective.
        meta.basic_attack.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
        meta.basic_attack.defense_tile_range =
            Some(crate::game::battle::tile_range::TileRangePattern {
                include_anchor_tile: true,
                rows: vec![
                    "XXXXX".into(),
                    "XXXXX".into(),
                    "XX@XX".into(),
                    "XXXXX".into(),
                    "XXXXX".into(),
                ],
            });
        meta
    }
    fn setup_transform(initial: u32) -> Hunt {
        let mut source =
            crate::authoring::composition::canonical_overlay_tests::equipment_fixture("accessory");
        source["values"][0]["value"]["DefenseProgram"]["EmployeeRevival"]["transform_into"] =
            json!("revival_form");
        setup_revive_source(initial, &source.to_string(), |b| {
            b.with_abnormalities(vec![form()])
        })
    }
    fn successor(f: &Hunt) -> UnitInstanceId {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_employee_transformation
            .as_ref()
            .unwrap()
            .successor
    }
    #[test]
    fn composition_metamorphosis_roundtrip_reference_cap_and_empty_reference_diagnostics() {
        let a = crate::authoring::composition::compile_yaml(TRANSFORM).unwrap();
        assert!(a.to_ron().unwrap().contains("revival_form"));
        assert!(crate::authoring::composition::compile_yaml(
            &TRANSFORM.replace("transform_into: revival_form", "transform_into: ''")
        )
        .is_err());
        let f = setup_transform(0);
        for missing in [false, true] {
            let equipment = (*f.core.game_data.equipment_data).clone();
            let mut resources = (*f.core.game_data.resource_data).clone();
            if !missing {
                resources.resources[0].cap = 2;
            }
            let empty = crate::game::data::abnormality_data::AbnormalityDatabase::new(vec![]);
            let error = crate::game::ability::equipment_defense::EquipmentDefenseCatalog::compile(
                &equipment,
                &resources,
                if missing {
                    &empty
                } else {
                    &f.core.game_data.abnormality_data
                },
                &crate::game::value_expr::ExprLimits::default(),
            )
            .unwrap_err();
            assert!(
                error.to_string().contains(if missing {
                    "transform_into"
                } else {
                    "cumulative"
                }),
                "{error}"
            );
        }
    }
    #[test]
    fn composition_metamorphosis_third_trigger_new_identity_same_position_no_transfer_or_heal() {
        for initial in [0, 1, 2] {
            let mut f = setup_transform(initial);
            let a = f.a;
            let position = f.core.units[&a].world_position();
            let employee = f.core.units[&a].source_owned_uuid;
            raw(&mut f, a, 100, 10).unwrap();
            assert_eq!(counts(&f, a), (initial + 1, 1));
            if initial < 2 {
                assert!(f.core.units[&a].is_active());
                assert_eq!(f.core.units[&a].stats.current_health, 5000);
                assert!(f
                    .core
                    .authoring_observation
                    .as_ref()
                    .unwrap()
                    .latest_employee_transformation
                    .is_none());
                continue;
            }
            let new = successor(&f);
            assert_ne!(new, a);
            let old = &f.core.units[&a];
            assert_eq!(old.lifecycle, RuntimeUnitLifecycle::PermanentlyDead);
            assert_eq!(old.stats.current_health, 0);
            assert_eq!(old.source_owned_uuid, employee);
            let new = &f.core.units[&new];
            assert_eq!(new.world_position(), position);
            assert_eq!(new.owner, Side::Opponent);
            assert_eq!(new.stats.current_health, 333);
            assert_eq!(new.stats.attack, 17);
            assert!(new.equipment_slots.is_none() && new.fragment_source.is_none());
            assert!(
                matches!(&new.source_identity, BattleUnitSourceIdentity::SummonedUnit { definition_id, summoner_instance_id, .. } if definition_id == "revival_form" && *summoner_instance_id == a)
            );
            assert!(f.core.required_transformation_alive());
            assert!(f.core.runtime_fact_summary().latest_heal.is_none());
            let death = f.core.runtime_fact_summary().latest_death.as_ref().unwrap();
            assert_eq!(death.cause, DeathCause::Transformation);
            assert_eq!(death.source.equipment.unwrap().definition_uuid, ACCESSORY);
            assert_eq!(
                death
                    .source
                    .authored
                    .as_ref()
                    .unwrap()
                    .site
                    .effect_id
                    .as_str(),
                "transform"
            );
            assert_eq!(
                f.core
                    .event_log
                    .entries
                    .iter()
                    .filter(|e| matches!(e.event, BattleLogEvent::EmployeeTransformed { .. }))
                    .count(),
                1
            );
        }
    }
    #[test]
    fn composition_metamorphosis_blocks_timed_victory_until_actual_enemy_death_and_keeps_defeat_priority(
    ) {
        let mut f = setup_transform(2);
        f.core.scenario_for_deployment().unwrap().terminal_outcomes =
            BattleScenario::fixture_survival_outcomes(10);
        for g in &mut f.core.scenario_for_deployment().unwrap().groups {
            g.required_for_victory = false;
        }
        assert_eq!(
            f.core.compute_terminal_outcome(10).unwrap().verdict,
            BattleVerdict::Victory
        );
        let a = f.a;
        raw(&mut f, a, 100, 10).unwrap();
        assert!(f.core.compute_terminal_outcome(10).is_none());
        let mut grouped = f.core.clone();
        for outcome in &mut grouped.scenario_for_deployment().unwrap().terminal_outcomes {
            if outcome.verdict == BattleVerdict::Victory {
                outcome.condition = crate::game::battle::scenario::TerminalCondition::AllRequiredEnemyGroupsDefeated;
            }
        }
        assert!(grouped.compute_terminal_outcome(10).is_none());
        let new = successor(&f);
        let mut defeat = f.core.clone();
        for outcome in &mut defeat.scenario_for_deployment().unwrap().terminal_outcomes {
            if outcome.verdict == BattleVerdict::Victory {
                outcome.verdict = BattleVerdict::Defeat;
            }
        }
        assert_eq!(
            defeat.compute_terminal_outcome(10).unwrap().verdict,
            BattleVerdict::Defeat
        );
        raw(&mut f, new, 333, 11).unwrap();
        assert!(!f.core.required_transformation_alive());
        assert_eq!(
            f.core.compute_terminal_outcome(11).unwrap().verdict,
            BattleVerdict::Victory
        );
        assert_eq!(
            f.core.units[&a].lifecycle,
            RuntimeUnitLifecycle::PermanentlyDead
        );
    }
    #[test]
    fn composition_metamorphosis_cancellation_restores_employee_counter_spawn_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "revival_reserved",
            "metamorphosis_employee_exit",
            "metamorphosis_spawned",
            "metamorphosis_complete",
        ] {
            let mut f = setup_transform(2);
            let a = f.a;
            let before = snap(&f);
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || raw(&mut f, a, 100, 10),
                )
            });
            assert!(result.is_err() && token.is_cancelled(), "{site}");
            assert_eq!(snap(&f), before, "{site}");
            assert_eq!(counts(&f, a), (2, 0));
            raw(&mut f, a, 100, 10).unwrap();
            assert_eq!(counts(&f, a), (3, 1));
            assert_eq!(
                f.core.units[&a].lifecycle,
                RuntimeUnitLifecycle::PermanentlyDead
            );
        }
    }
    #[test]
    fn composition_metamorphosis_clone_observation_independent_and_real_enemy_attack() {
        let mut f = setup_transform(2);
        let a = f.a;
        let mut off = Hunt {
            core: f.core.clone(),
            state: f.state.clone(),
            a,
            b: f.b,
            enemy: f.enemy,
        };
        off.core.authoring_observation = None;
        raw(&mut f, a, 100, 10).unwrap();
        raw(&mut off, a, 100, 10).unwrap();
        assert_eq!(snap(&f), snap(&off));
        let new = successor(&f);
        let b = f.b;
        let tile = f.core.units[&new].body.projected_tile();
        place(&mut f.core, b, tile.x + 1, tile.y);
        f.core.units.get_mut(&b).unwrap().stats.current_health = 100;
        assert_eq!(
            f.core.units[&new].facing_direction,
            f.core.units[&a].facing_direction
        );
        assert_eq!(f.core.select_basic_attack_target(new, None, None), Some(b));
        // A real queued basic attack uses the new form's own power and source.
        advance(&mut f, 1500);
        assert!(f
            .core
            .runtime_fact_summary()
            .latest_damage
            .as_ref()
            .is_some_and(|d| d.source.actor == Some(new)));
        assert_eq!(f.core.units[&b].stats.current_health, 83);
        assert_eq!(counts(&f, a), (3, 1));
    }
    #[test]
    fn composition_metamorphosis_suppresses_last_will_while_ordinary_death_still_uses_it() {
        let mut doc = crate::authoring::composition::parse_yaml(TRANSFORM).unwrap();
        doc["values"].as_array_mut().unwrap().push(json!({
            "name":"farewell", "value":{"DefenseProgram":{"LastWill":{
                "range":{"include_anchor_tile":true,"rows":["XXXXX","XXXXX","XX@XX","XXXXX","XXXXX"]},
                "attack_percent":300,"damage_type":"Magic"
            }}}
        }));
        doc["equipment_defenses"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "equipment":"last_will_fixture_armor","program":{"Shared":"farewell"}
            }));
        let mut f =
            setup_revive_source(2, &doc.to_string(), |b| b.with_abnormalities(vec![form()]));
        let a = f.a;
        let b = f.b;
        let enemy_hp = f.core.units[&f.enemy].stats.current_health;
        raw(&mut f, a, 100, 10).unwrap();
        assert_eq!(
            f.core.units[&a].lifecycle,
            RuntimeUnitLifecycle::PermanentlyDead
        );
        assert!(f.core.equipment_defense.last_will_used.is_empty());
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_last_will
            .is_none());
        assert_eq!(f.core.units[&f.enemy].stats.current_health, enemy_hp);
        // A different employee still follows the ordinary revival -> exhausted
        // battle use -> last-will path; suppression is specific to transformation.
        raw(&mut f, b, 100, 20).unwrap();
        assert_eq!(f.core.units[&b].stats.current_health, 5000);
        raw(&mut f, b, 5000, 1021).unwrap();
        assert!(f.core.units[&b].is_dead());
        assert_eq!(f.core.equipment_defense.last_will_used.len(), 1);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_last_will
            .is_some());
    }
    #[test]
    fn composition_metamorphosis_terminal_identity_cannot_be_redeployed() {
        let mut f = setup_transform(2);
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
        raw(&mut f, a, 100, 10).unwrap();
        let before = snap(&f);
        assert!(f
            .core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position,
                tile_kind: TacticalTileKind::Ground,
                facing: FacingDirection::Right,
                instance_salt: 0xe690,
                time_ms: 11,
                current_hp_policy: None,
            })
            .is_err());
        assert_eq!(snap(&f), before);
    }
    #[test]
    fn composition_metamorphosis_once_in_battle_before_limit_and_lethal_health_cost() {
        let mut f = setup_transform(1);
        let a = f.a;
        raw(&mut f, a, 100, 10).unwrap();
        assert_eq!(counts(&f, a), (2, 1));
        raw(&mut f, a, 5000, 1010).unwrap();
        assert_eq!(f.core.units[&a].lifecycle, RuntimeUnitLifecycle::Dead);
        assert_eq!(counts(&f, a), (2, 1));
        assert!(!f.core.required_transformation_alive());
        for cost in [false, true] {
            let mut f = setup_transform(2);
            let a = f.a;
            f.core
                .runtime_transaction(|c| {
                    if cost {
                        c.pay_health_cost(a, 100, true, 10)?;
                    } else {
                        c.apply_hp_delta_and_record(
                            None,
                            a,
                            -100,
                            10,
                            crate::game::battle::event_log::HpChangeReason::Command,
                        );
                    }
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
            assert_eq!(
                f.core.units[&a].lifecycle,
                RuntimeUnitLifecycle::PermanentlyDead
            );
            assert_eq!(counts(&f, a), (3, 1));
            assert_eq!(f.core.runtime_fact_summary().damage_hp_lost, 0);
            if cost {
                assert_eq!(
                    f.core
                        .runtime_fact_summary()
                        .latest_cost
                        .as_ref()
                        .unwrap()
                        .actual_spent(),
                    100
                );
            }
        }
    }
    #[test]
    fn composition_metamorphosis_retained_refusal_is_atomic_before_spawn_and_retries() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup_transform(2);
        let a = f.a;
        let mut probe = Hunt {
            core: f.core.clone(),
            state: f.state.clone(),
            a,
            b: f.b,
            enemy: f.enemy,
        };
        let (result, admissions) =
            native::observe_retained_admissions(|| raw(&mut probe, a, 100, 10));
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "metamorphosis_required_enemy")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = snap(&f);
        let error = raw(&mut f, a, 100, 10).unwrap_err();
        assert!(format!("{error:?}").contains("RetainedByteLimit"));
        assert_eq!(snap(&f), before);
        assert_eq!(counts(&f, a), (2, 0));
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        raw(&mut f, a, 100, 10).unwrap();
        assert_eq!(counts(&f, a), (3, 1));
    }
}
