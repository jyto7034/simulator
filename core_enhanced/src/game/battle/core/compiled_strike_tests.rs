mod compiled_strike {
    use super::*;
    use crate::{authoring::node_graph::tests::strike_fixture, game::battle::{core::sim::{BattleExecutionState, BattleLiveCommand}, damage::BattleCommand}};
    const RELICS: [&str; 5] = [
        "deep-wedge",
        "widened-horizon",
        "unbroken-trance",
        "broken-balance",
        "long-breath",
    ];
    fn fixture(
        equipped: &[usize],
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        fixture_graph(equipped, strike_fixture())
    }
    use super::compiled_action_fixture::{cast, hit, place};
    fn fixture_graph(equipped: &[usize], doc: serde_json::Value) -> (BattleCore, BattleExecutionState, UnitInstanceId, UnitInstanceId, UnitInstanceId) {
        super::compiled_action_fixture::fixture_graph(&RELICS, equipped, doc)
    }
    #[test]
    fn compiled_strike_current_targets_pull_stun_and_live_missing_hp() {
        let (mut core, mut state, actor, enemy, _) = fixture(&[]);
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 900);
        assert_eq!(
            core.unit_world_position_or_tile_center(enemy)
                .unwrap()
                .project_to_tile(),
            Position::new(3, 1)
        );
        core.step_battle_execution_until(&mut state, 101).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 800);
        assert!(core
            .unit_control_state(enemy, 101)
            .unwrap()
            .hard_cc_active());
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 680);
        assert!(!core.resonance_use_has_obligations(actor));
        core.step_battle_execution_until(&mut state, 401).unwrap();
        assert!(!core
            .unit_control_state(enemy, 401)
            .unwrap()
            .hard_cc_active());
    }
    #[test]
    fn compiled_strike_equipped_wedge_duration_and_independent_vulnerability() {
        let (mut core, mut state, actor, enemy, _) = fixture(&[0, 3, 4]);
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 898);
        core.step_battle_execution_until(&mut state, 101).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 794);
        let snapshot = core.sustained_effect_snapshot().unwrap();
        let source = snapshot
            .active
            .iter()
            .filter_map(|r| r.hit_status.as_ref())
            .find_map(|p| p.modifier_source.as_ref())
            .unwrap();
        assert_eq!(source.definition, "broken-balance");
        assert!(!source.copy_uuid.is_nil());
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 641);
        core.step_battle_execution_until(&mut state, 474).unwrap();
        assert!(core
            .unit_control_state(enemy, 474)
            .unwrap()
            .hard_cc_active());
        core.step_battle_execution_until(&mut state, 476).unwrap();
        assert!(!core
            .unit_control_state(enemy, 476)
            .unwrap()
            .hard_cc_active());
        hit(&mut core, actor, enemy, 100, 476);
        assert_eq!(core.units[&enemy].stats.current_health, 521);
        core.step_battle_execution_until(&mut state, 5101).unwrap();
        hit(&mut core, actor, enemy, 100, 5101);
        assert_eq!(core.units[&enemy].stats.current_health, 421);
    }
    #[test]
    fn compiled_strike_first_target_death_does_not_cancel_remaining_area_steps() {
        let (mut core, mut state, actor, enemy, other) = fixture(&[]);
        core.units.get_mut(&enemy).unwrap().stats.current_health = 100;
        core.units.get_mut(&other).unwrap().owner = Side::Opponent;
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert!(!core.units[&enemy].is_active());
        place(&mut core, other, 3, 1);
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert_eq!(core.units[&other].stats.current_health, 790);
    }
    #[test]
    fn compiled_strike_external_control_protection_keeps_states_and_still_ends_on_withdrawal() {
        for protected in [false, true] {
            let (mut core, mut state, actor, enemy, _) =
                fixture(if protected { &[2] } else { &[] });
            cast(&mut core, actor, 0);
            core.step_battle_execution_until(&mut state, 1).unwrap();
            core.process_commands(
                vec![BattleCommand::ApplyBuff {
                    caster_id: enemy,
                    target_id: actor,
                    buff_id: crate::game::battle::buffs::BuffId::from_name("r11_stun"),
                    duration_ms: 500,
                }],
                1,
            );
            core.step_battle_execution_until(&mut state, 201).unwrap();
            assert!(core
                .unit_control_state(actor, 201)
                .unwrap()
                .hard_cc_active());
            assert_eq!(
                core.units[&enemy].stats.current_health,
                if protected { 680 } else { 900 }
            );
        }
        let (mut core, mut state, actor, enemy, _) = fixture(&[2]);
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        core.apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: actor,
            time_ms: 1,
        })
        .unwrap();
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 900);
    }
    fn forward(doc: &mut serde_json::Value) {
        let ranges: Vec<_> = doc["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| {
                n["kind"] == "skill-range"
                    && !n["title"].as_str().unwrap().contains("확대")
                    && !n["title"].as_str().unwrap().contains("첫 유효")
            })
            .map(|n| n["id"].clone())
            .collect();
        for n in doc["nodes"].as_array_mut().unwrap() {
            if ranges.contains(&n["id"]) {
                n["values"]["pattern"] = serde_json::json!("X\n@");
                n["values"]["includeAnchor"] = serde_json::json!(false);
            }
        }
    }
    #[test]
    fn compiled_strike_each_release_reads_current_position_and_facing() {
        let mut doc = strike_fixture();
        forward(&mut doc);
        let (mut core, mut state, actor, enemy, _) = fixture_graph(&[], doc);
        place(&mut core, enemy, 3, 1);
        core.units.get_mut(&actor).unwrap().facing_direction =
            Some(crate::game::battle::tile_range::FacingDirection::Right);
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 900);
        core.step_battle_execution_until(&mut state, 50).unwrap();
        place(&mut core, actor, 4, 1);
        core.step_battle_execution_until(&mut state, 101).unwrap();
        assert_eq!(
            core.units[&enemy].stats.current_health, 900,
            "current right-facing area excludes the previous target behind the caster"
        );
        core.units.get_mut(&actor).unwrap().facing_direction =
            Some(crate::game::battle::tile_range::FacingDirection::Left);
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 790);
    }
    #[test]
    fn compiled_strike_equipped_range_replaces_damage_and_control_footprints() {
        for equipped in [false, true] {
            let mut doc = strike_fixture();
            forward(&mut doc);
            let (mut core, mut state, actor, enemy, _) =
                fixture_graph(if equipped { &[1] } else { &[] }, doc);
            core.units.get_mut(&actor).unwrap().facing_direction =
                Some(crate::game::battle::tile_range::FacingDirection::Right);
            if !equipped {
                assert!(
                    core.apply_live_command(BattleLiveCommand::ActivateSkill {
                        unit_id: actor,
                        skill_id: "r11_aura".into(),
                        target: None,
                        time_ms: 0
                    })
                    .is_err(),
                    "ordinary usefulness qualification rejects an entirely empty base footprint"
                );
                assert_eq!(core.units[&actor].resonance_current, 100);
                continue;
            }
            cast(&mut core, actor, 0);
            core.step_battle_execution_until(&mut state, 201).unwrap();
            assert_eq!(core.units[&enemy].stats.current_health, 680);
            assert_eq!(
                core.unit_control_state(enemy, 201)
                    .unwrap()
                    .hard_cc_active(),
                equipped
            );
            assert_eq!(
                core.units[&enemy].body.projected_tile(),
                Position::new(if equipped { 3 } else { 4 }, 1)
            );
        }
    }
    #[test]
    fn compiled_strike_rounding_keeps_fractional_hp_ratio_and_duration() {
        let mut doc = strike_fixture();
        for n in doc["nodes"].as_array_mut().unwrap() {
            if n["kind"] == "skill-weapon-power" {
                n["values"]["coefficient"] = serde_json::json!(1.005);
            }
            if n["kind"] == "skill-seconds" && n["values"]["value"] == 0.3 {
                n["values"]["value"] = serde_json::json!(0.001);
            }
        }
        let (mut core, mut state, actor, enemy, _) = fixture_graph(&[4], doc);
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 899);
        core.step_battle_execution_until(&mut state, 102).unwrap();
        assert!(core
            .unit_control_state(enemy, 102)
            .unwrap()
            .hard_cc_active());
        core.step_battle_execution_until(&mut state, 103).unwrap();
        assert!(!core
            .unit_control_state(enemy, 103)
            .unwrap()
            .hard_cc_active());
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 676);
    }
    #[test]
    fn compiled_strike_wedge_caps_and_resets_at_new_cast() {
        let mut doc = strike_fixture();
        for n in doc["nodes"].as_array_mut().unwrap() {
            if n["kind"] == "skill-modifier-multihit" {
                n["values"]["step"] = serde_json::json!(12);
            }
        }
        let (mut core, mut state, actor, enemy, _) = fixture_graph(&[0], doc);
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 620);
        core.step_battle_execution_until(&mut state, 501).unwrap();
        core.add_resonance(actor, 100, 501, false);
        cast(&mut core, actor, 501);
        core.step_battle_execution_until(&mut state, 502).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 508);
    }
    #[test]
    fn compiled_strike_caster_exit_keeps_applied_status_and_fault_rolls_back() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut core, mut state, actor, enemy, ally) = fixture(&[3]);
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 100).unwrap();
        let before = (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
            format!("{state:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("sustained_started", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || core.step_battle_execution_until(&mut state, 101),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(
            before,
            (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot(),
                serde_json::to_value(core.runtime_fact_view()).unwrap(),
                format!("{state:?}")
            )
        );
        core.step_battle_execution_until(&mut state, 101).unwrap();
        core.apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: actor,
            time_ms: 101,
        })
        .unwrap();
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 800);
        assert!(core
            .unit_control_state(enemy, 201)
            .unwrap()
            .hard_cc_active());
        hit(&mut core, ally, enemy, 100, 201);
        assert_eq!(core.units[&enemy].stats.current_health, 680);
        core.step_battle_execution_until(&mut state, 5101).unwrap();
        assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
        assert_eq!(core.runtime_fact_view().usage().outstanding_deliveries, 0);
    }
}
