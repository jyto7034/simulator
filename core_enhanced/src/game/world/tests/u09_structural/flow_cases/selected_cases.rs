use super::*;
use crate::game::battle::event_log::SkillCastTarget;
use personal_cases::{personal_relic, personal_skill, pools, team_fixture};

fn plain_selected() -> String {
    personal_skill("Snapshot")
        .replace(
            "selection: self, target: SelfUnit, reach: pattern",
            "selection: player_unit, target: AllySingle, reach: global",
        )
        .replace(
            "target: SelfUnit, delivery: Instant",
            "target: AllySingle, delivery: Instant",
        )
        .replace(
            "PlayerEmployeesAtRelease(radius_units: 2000000)",
            "SelectedPlayerEmployeeAtRelease(authored_reach_units: 2000000)",
        )
}

fn focus(core: &mut GameCore, player: Uuid, employee: Uuid) {
    execute_current_battle_mutation(
        core,
        player,
        PlayerBehavior::BeginSkillTargeting {
            employee_uuid: employee,
            skill_id: "tc_bite".into(),
        },
    )
    .unwrap();
}

fn activate(
    core: &mut GameCore,
    player: Uuid,
    employee: Uuid,
    target: UnitInstanceId,
) -> Result<BehaviorResult, GameError> {
    execute_current_battle_mutation(
        core,
        player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: employee,
            skill_id: "tc_bite".into(),
            target: Some(SkillCastTarget::Unit {
                unit_instance_id: target,
            }),
        },
    )
}

#[test]
fn rv002_plain_selected_personal_uses_authored_reach_qualification_and_caster() {
    for select_caster in [false, true] {
        let (mut core, player, employee, actor, allies, enemy) =
            team_fixture(&plain_selected(), &personal_relic(None));
        focus(&mut core, player, employee);
        let object = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .units
            .values()
            .find(|unit| {
                unit.owner == Side::Player
                    && !matches!(
                        unit.source_identity,
                        crate::game::battle::types::BattleUnitSourceIdentity::Employee { .. }
                    )
            })
            .unwrap()
            .instance_id;
        native_probe::place_stationary_x(
            &mut core.state.active_battle.as_mut().unwrap().battle,
            allies[1],
            4.75,
        );
        for rejected in [enemy, object, allies[1]] {
            let before =
                native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle);
            let error = activate(&mut core, player, employee, rejected).unwrap_err();
            assert!(
                matches!(
                    error,
                    GameError::ActionUnavailable {
                        reason: crate::game::behavior::ActionDisabledReasonCode::NoValidTarget,
                        ..
                    }
                ),
                "{error:?}"
            );
            assert_eq!(
                native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle),
                before
            );
            assert!(pools(&core).is_empty());
        }
        let recipient = if select_caster { actor } else { allies[0] };
        activate(&mut core, player, employee, recipient).unwrap();
        core.advance_active_battle_for_server_tick(1).unwrap();
        let rows = pools(&core);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].target, recipient);
        assert_eq!(rows[0].shield.unwrap().capacity, 450);
        println!("RV002 plain_selected caster={select_caster} amount450 pool1 enemy_object_out_of_reach_rejected=true");
    }
}

#[test]
fn rv002_mixed_selected_consumers_cannot_be_rescued_by_eligible_legacy_sibling() {
    // Same authored reach, different qualification: PersonalShieldSingle allows
    // the caster, whereas FixedRecipientSupport excludes that identity.
    let skill = support_cases::support_skill().replace(
        "effects: [(id: \"channel\"",
        r#"effects: [
        (id: "useful_legacy_sibling", op: Heal(amount: 31)),
        (id: "pool", op: PersonalShieldGrant(definition: (
            amount: (moment: Snapshot, root: Const((value: 200, unit: Magnitude))),
            duration_ms: 1000, priority: 5,
            recipients: PlayerEmployeesAtRelease(radius_units: 2000000),
            source_creation: Some((operations: [Scale(increment_percent: 50)]))))),
        (id: "channel""#,
    );
    let personal = personal_relic(Some(420));
    let personal_rules = personal.split_once("rules: [").unwrap().1;
    let support = support_cases::support_relic();
    let relic = support.replace(
        "rules: [",
        &format!(
            "rules: [ {},",
            personal_rules.strip_suffix("], )])").unwrap()
        ),
    );
    let (mut core, player, employee, actor, allies, _) = team_fixture(&skill, &relic);
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .units
        .get_mut(&allies[0])
        .unwrap()
        .stats
        .current_health = 700;
    focus(&mut core, player, employee);
    let before = native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle);
    let error = activate(&mut core, player, employee, actor).unwrap_err();
    assert!(
        matches!(
            error,
            GameError::ActionUnavailable {
                reason: crate::game::behavior::ActionDisabledReasonCode::NoValidTarget,
                ..
            }
        ),
        "{error:?}"
    );
    assert_eq!(
        native_probe::flow_roots(&core.state.active_battle.as_ref().unwrap().battle),
        before
    );
    assert!(pools(&core).is_empty());
    activate(&mut core, player, employee, allies[0]).unwrap();
    core.advance_active_battle_for_server_tick(1).unwrap();
    let rows = pools(&core);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].target, allies[0]);
    assert_eq!(rows[0].shield.unwrap().capacity, 825);
    assert_eq!(hp(&core, &[actor]), 531);
    assert_eq!(
        native_probe::support_bonus(
            &core.state.active_battle.as_ref().unwrap().battle,
            allies[0],
            true
        ),
        (40, 20)
    );
    println!("RV002 mixed_selected caster_personal_eligible_fixed_ineligible_rejected=true ally_personal825_fixed40_20=true legacy_heal31=true");
}

#[test]
fn rv002_pending_selected_personal_uses_retained_plan_after_current_owner_removal() {
    let skill = personal_skill("Snapshot").replace("focus_time_ms: 0", "focus_time_ms: 20");
    let (mut core, player, employee, actor, allies, _) =
        team_fixture(&skill, &personal_relic(Some(420)));
    focus(&mut core, player, employee);
    activate(&mut core, player, employee, allies[0]).unwrap();
    native_probe::remove_current_owner_keep_pending(
        &mut core.state.active_battle.as_mut().unwrap().battle,
        actor,
    );
    core.advance_active_battle_for_server_tick(21).unwrap();
    let rows = pools(&core);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].target, allies[0]);
    assert_eq!(rows[0].shield.unwrap().capacity, 825);
    println!("RV002 pending_selected currentSelf_retainedPlayerUnit=true released_one825_selected_ally=true");
}
