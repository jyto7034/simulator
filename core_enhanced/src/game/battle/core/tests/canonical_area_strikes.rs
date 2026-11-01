use super::*;
use crate::authoring::composition::canonical_area_tests;
use crate::game::battle::core::boss_runtime::tests::compiled_action_fixture::{
    cast, fixture_content_with_data, hit, place,
};

const RELICS: [&str; 5] = [
    "deep-wedge",
    "widened-horizon",
    "unbroken-trance",
    "broken-balance",
    "long-breath",
];

fn canonical(
    family: &str,
    equipped: &[usize],
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let mut document = canonical_area_tests::resolved(family);
    // This low-level fixture supplies its own fragment identity/capacity. The
    // ordinary published-World test checks the unmodified canonical identities.
    document["skills"][0]["id"] = json!("r11_aura");
    document["skills"][0]["admission"]["capacity"] = json!(100);
    let artifact = compile_json(&document.to_string()).unwrap();
    let native = crate::authoring::node_graph::NativeGraphContent {
        skills: Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap(),
        equipment: vec![],
        mark_weapons: vec![],
    };
    fixture_content_with_data(
        &RELICS,
        equipped,
        native,
        |_| {},
        crate::game::resource::ResourceCatalog { resources: vec![] },
        |builder| builder.with_relics(canonical_area_tests::relics()),
        |_| {},
    )
}

#[test]
fn canonical_woodsman_equipped_shared_multihit_and_independent_status_deadlines() {
    for equipped in [false, true] {
        let (mut core, mut state, actor, enemy, _) =
            canonical("woodsman", if equipped { &[0, 3, 4] } else { &[] });
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if equipped { 898 } else { 900 }
        );
        assert_eq!(
            core.unit_world_position_or_tile_center(enemy)
                .unwrap()
                .project_to_tile(),
            Position::new(3, 1)
        );
        core.step_battle_execution_until(&mut state, 101).unwrap();
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if equipped { 794 } else { 800 }
        );
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if equipped { 487 } else { 560 }
        );
        assert!(!core.resonance_use_has_obligations(actor));
        core.step_battle_execution_until(&mut state, 401).unwrap();
        assert_eq!(
            core.unit_control_state(enemy, 401)
                .unwrap()
                .hard_cc_active(),
            equipped
        );
        core.step_battle_execution_until(&mut state, 476).unwrap();
        assert!(!core
            .unit_control_state(enemy, 476)
            .unwrap()
            .hard_cc_active());
        let before = core.units[&enemy].stats.current_health;
        hit(&mut core, actor, enemy, 100, 476);
        assert_eq!(
            before - core.units[&enemy].stats.current_health,
            if equipped { 120 } else { 100 }
        );
        core.step_battle_execution_until(&mut state, 5101).unwrap();
        let before = core.units[&enemy].stats.current_health;
        hit(&mut core, actor, enemy, 100, 5101);
        assert_eq!(before - core.units[&enemy].stats.current_health, 100);
    }
}

#[test]
fn canonical_woodsman_shared_multihit_is_per_target_and_resets_after_cast() {
    let (mut core, mut state, actor, enemy, other) = canonical("woodsman", &[0]);
    core.units.get_mut(&other).unwrap().owner = Side::Opponent;
    core.units.get_mut(&other).unwrap().skill_id = None;
    core.units.get_mut(&other).unwrap().fragment_source = None;
    cast(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 794);
    place(&mut core, other, 2, 2);
    core.step_battle_execution_until(&mut state, 201).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 538);
    assert_eq!(
        core.units[&other].stats.current_health, 796,
        "first hit receives only 2%, even at third action"
    );
    core.units.get_mut(&actor).unwrap().resonance_current = 100;
    cast(&mut core, actor, 202);
    core.step_battle_execution_until(&mut state, 203).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 436);
    assert_eq!(core.units[&other].stats.current_health, 694);
}

#[test]
fn canonical_woodsman_range_modifier_reaches_new_enemy_without_extending_pull() {
    for equipped in [false, true] {
        let (mut core, mut state, actor, enemy, other) =
            canonical("woodsman", if equipped { &[1] } else { &[] });
        core.units.get_mut(&other).unwrap().owner = Side::Opponent;
        core.units.get_mut(&other).unwrap().skill_id = None;
        core.units.get_mut(&other).unwrap().fragment_source = None;
        // Keep the pull destination (4, 1) unoccupied regardless of hit order.
        place(&mut core, enemy, 3, 2);
        place(&mut core, other, 5, 1); // actor at (2, 1): outside base radius 2
        cast(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(
            core.units[&other].stats.current_health,
            if equipped { 900 } else { 1000 }
        );
        assert_eq!(
            core.units[&other].body.projected_tile(),
            Position::new(if equipped { 4 } else { 5 }, 1)
        );
        core.step_battle_execution_until(&mut state, 201).unwrap();
        assert_eq!(
            core.units[&other].stats.current_health,
            if equipped { 560 } else { 1000 }
        );
        assert_eq!(
            core.unit_control_state(other, 201)
                .unwrap()
                .hard_cc_active(),
            equipped
        );
    }
}

#[test]
fn canonical_woodsman_protection_keeps_cc_but_withdrawal_ends_remaining_actions() {
    for protected in [false, true] {
        let (mut core, mut state, actor, enemy, _) =
            canonical("woodsman", if protected { &[2] } else { &[] });
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
            if protected { 560 } else { 900 }
        );
    }
    let (mut core, mut state, actor, enemy, _) = canonical("woodsman", &[2]);
    cast(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    core.apply_live_command(BattleLiveCommand::WithdrawUnit {
        unit_id: actor,
        time_ms: 1,
    })
    .unwrap();
    core.step_battle_execution_until(&mut state, 201).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 900);
    assert!(!core.resonance_use_has_obligations(actor));
}

#[test]
fn canonical_monk_fixed_admission_tile_release_and_one_actual_hit_recovery() {
    let (mut core, mut state, actor, enemy, other) = canonical("clouded-monk", &[]);
    core.units.get_mut(&actor).unwrap().stats.current_health = 500;
    core.units.get_mut(&other).unwrap().owner = Side::Opponent;
    core.units.get_mut(&other).unwrap().skill_id = None;
    core.units.get_mut(&other).unwrap().fragment_source = None;
    place(&mut core, other, 4, 2);
    cast(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 1000);
    core.step_battle_execution_until(&mut state, 301).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert_eq!(core.units[&other].stats.current_health, 850);
    assert_eq!(
        core.units[&actor].stats.current_health, 700,
        "two recipients still produce one 20% max-HP recovery"
    );
}
