use super::*;

fn footprint_relic() -> String {
    relic_text(Some(EFFECT_GATE)).replace("rules: [", r#"rules: [
        (id: "bite_footprint", selector: (provider: (fragment_id: Some("tc_fragment")), effect: (all_tags: ["bite_footprint"])),
            apply: Recipe(DamageFootprint(footprint: (include_anchor_tile: false, rows: ["X", "X", "@"])))),"#)
}

fn amount_skill(moment: &str) -> String {
    skill_text(100)
        .replace(
            "id: \"bite_damage\",",
            "id: \"bite_damage\", tags: [\"bite_footprint\"],",
        )
        .replace(
            "Damage(amount: 100, damage_type: Physical)",
            &format!(
                r#"DamageExpr(amount: (moment: {moment},
            root: Aggregate(source: ResolvedTargets, grouping: WholeSet, reduction: Sum,
                member: Const((value: 10, unit: Magnitude)))), damage_type: Physical)"#
            ),
        )
}

fn arranged(skills: &str) -> (GameCore, Uuid, Uuid, UnitInstanceId, Vec<UnitInstanceId>) {
    let (mut core, player, employee, actor, targets) =
        prepare_with_options(skills, &footprint_relic(), None, Some(2));
    assert_eq!(targets.len(), 2);
    let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
    // Controlled spatial fixture after normal World construction, before cast.
    // Literal positions give one inherited target and two footprint targets.
    native_probe::place_stationary(battle, actor, 2, 2);
    native_probe::place_stationary(battle, targets[0], 3, 2);
    native_probe::place_stationary(battle, targets[1], 4, 2);
    (core, player, employee, actor, targets)
}

#[test]
fn tc001_effect_footprint_instant_uses_own_amount_targets_and_keeps_sibling_single() {
    for (moment, caster_before, caster_after) in [
        ("Snapshot", 500, 537),
        ("Live", 500, 537),
        ("Live", 990, 1000),
    ] {
        let (mut core, player, employee, actor, targets) = arranged(&amount_skill(moment));
        core.state
            .active_battle
            .as_mut()
            .unwrap()
            .battle
            .units
            .get_mut(&actor)
            .unwrap()
            .stats
            .current_health = caster_before;
        let before = targets
            .iter()
            .map(|id| hp(&core, &[*id]))
            .collect::<Vec<_>>();
        cast(&mut core, player, employee);
        // Two actual bite recipients => 10+10=20 each. Only nearest A gets the
        // inherited sibling13; the settled bite triggers exactly one Self heal.
        assert_eq!(before[0] - hp(&core, &[targets[0]]), 33);
        assert_eq!(before[1] - hp(&core, &[targets[1]]), 20);
        assert_eq!(hp(&core, &[actor]), caster_after);
        assert_eq!(step_count(&core), 2);
        assert_eq!(
            core.state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .runtime_fact_summary()
                .heal_mutations,
            1
        );
        println!("TC001 instant moment={moment} bite_each=20 sibling_A=13 sibling_B=0 heal_count=1 caster={caster_before}->{caster_after}");
    }
}

#[test]
fn tc001_effect_footprint_area_has_separate_history_and_snapshot_live_amounts() {
    for (policy, moment, second_bite) in [
        ("OncePerArea", "Snapshot", 0),
        ("OncePerArea", "Live", 0),
        ("OnEnter", "Snapshot", 20),
        ("OnEnter", "Live", 10),
    ] {
        let area = format!(
            r#"target: SelfUnit, delivery: TileArea(
            footprint: (include_anchor_tile: false, rows: ["X", "@"]),
            area: (anchor: Caster, tile_origin: Caster, tracking: GroundFixed,
                hit_targets: Enemies, include_caster: false, tick_policy: {policy},
                duration_ms: 30, tick_interval_ms: Some(10)))"#
        );
        // TileArea only resolves recipients at consumption. Snapshot therefore
        // uses a legal release-time literal; Live counts current hit candidates.
        let skill = if moment == "Snapshot" {
            amount_skill(moment).replace(
                "Aggregate(source: ResolvedTargets, grouping: WholeSet, reduction: Sum,\n                member: Const((value: 10, unit: Magnitude)))",
                "Const((value: 20, unit: Magnitude))",
            )
        } else {
            amount_skill(moment)
        };
        let skills = skill.replace("target: CastTarget, delivery: Instant", &area);
        let (mut core, player, employee, actor, targets) = arranged(&skills);
        let before = targets
            .iter()
            .map(|id| hp(&core, &[*id]))
            .collect::<Vec<_>>();
        cast(&mut core, player, employee);
        assert_eq!(before[0] - hp(&core, &[targets[0]]), 33);
        assert_eq!(before[1] - hp(&core, &[targets[1]]), 20);
        assert_eq!(
            hp(&core, &[actor]),
            500,
            "anchor effects are not settled while area is alive"
        );
        let expected_history = if policy == "OncePerArea" {
            vec![(0, 2, 0)]
        } else {
            vec![(0, 0, 2)]
        };
        assert_eq!(
            native_probe::effect_history_counts(&core.state.active_battle.as_ref().unwrap().battle),
            expected_history
        );
        native_probe::place_stationary(
            &mut core.state.active_battle.as_mut().unwrap().battle,
            targets[1],
            2,
            2,
        );
        core.advance_active_battle_for_server_tick(10).unwrap();
        assert_eq!(before[1] - hp(&core, &[targets[1]]), 20);
        native_probe::place_stationary(
            &mut core.state.active_battle.as_mut().unwrap().battle,
            targets[1],
            4,
            2,
        );
        core.advance_active_battle_for_server_tick(10).unwrap();
        assert_eq!(before[0] - hp(&core, &[targets[0]]), 33);
        assert_eq!(before[1] - hp(&core, &[targets[1]]), 20 + second_bite);
        core.advance_active_battle_for_server_tick(10).unwrap();
        assert_eq!(hp(&core, &[actor]), 537);
        assert_eq!(step_count(&core), 2);
        assert!(native_probe::effect_history_counts(
            &core.state.active_battle.as_ref().unwrap().battle
        )
        .is_empty());
        assert_eq!(
            native_probe::active_cast_count(&core.state.active_battle.as_ref().unwrap().battle),
            0
        );
        println!("TC001 area policy={policy} moment={moment} A_loss=33 B_loss={} reentry_bite={second_bite} sibling_B=0 heal_once_at31=true history_released=true", 20+second_bite);
    }
}

#[test]
fn rv005_effect_gate_empty_invulnerable_and_lethal_footprints_use_actual_hp_loss() {
    for (mode, losses, caster_after, heals) in [
        ("empty", [13, 0], 500, 0),
        ("invulnerable", [0, 0], 500, 0),
        ("lethal", [10, 0], 537, 1),
        ("nonlethal", [33, 20], 537, 1),
    ] {
        // Snapshot literal20 keeps this a target/gate test. Existing cases
        // independently cover amount evaluation against resolved target sets.
        let skills = amount_skill("Snapshot").replace(
            "Aggregate(source: ResolvedTargets, grouping: WholeSet, reduction: Sum,\n                member: Const((value: 10, unit: Magnitude)))",
            "Const((value: 20, unit: Magnitude))",
        );
        let (mut core, player, employee, actor, targets) = arranged(&skills);
        let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
        match mode {
            "empty" => {
                // Valid right-facing two-tile footprint has no recipients;
                // original targets remain behind caster on known fixture cells.
                native_probe::place_stationary(battle, actor, 4, 2);
                native_probe::place_stationary(battle, targets[0], 3, 2);
                native_probe::place_stationary(battle, targets[1], 2, 2);
            }
            "invulnerable" => {
                for target in &targets {
                    native_probe::prepare_existing_invulnerability(battle, actor, *target);
                }
            }
            "lethal" => {
                battle
                    .units
                    .get_mut(&targets[0])
                    .unwrap()
                    .stats
                    .current_health = 10;
                // Keep a second live enemy unharmed so ordinary lethal
                // settlement is distinct from terminal-battle cleanup.
                native_probe::prepare_existing_invulnerability(battle, actor, targets[1]);
            }
            "nonlethal" => {}
            _ => unreachable!(),
        }
        assert_eq!(
            step_count(&core),
            0,
            "preparation never invokes a skill/counter"
        );
        let prepared = &core.state.active_battle.as_ref().unwrap().battle;
        assert_eq!(native_probe::active_cast_count(prepared), 0);
        assert_eq!(prepared.runtime_fact_summary().damage_hp_lost, 0);
        assert_eq!(prepared.runtime_fact_summary().heal_mutations, 0);
        let target_hp = |core: &GameCore, id: UnitInstanceId| {
            core.state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .units
                .get(&id)
                .map_or(0, |unit| unit.stats.current_health)
        };
        let before = [target_hp(&core, targets[0]), target_hp(&core, targets[1])];
        if mode == "lethal" {
            use crate::game::battle::runtime_control as native;
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::ActivateSkill {
                    employee_uuid: employee,
                    skill_id: "tc_bite".into(),
                    target: None,
                },
            )
            .unwrap();
            // Meter the actual accepted payload's consumption tick, including
            // the finite-child occurrence lookup and lethal-to-Self dispatch.
            // The earlier withdrawal boundary meters a different operation.
            let roots = |core: &GameCore| {
                let active = core.state.active_battle.as_ref().unwrap();
                serde_json::json!({
                    "battle": native_probe::flow_roots(&active.battle),
                    "execution": format!("{:?}", active.execution),
                    "revision": active.battle_state_revision,
                    "resources": format!("{:?}", core.battle_typed_resources()),
                    "save": core.get_run_snapshot_json().unwrap(),
                })
            };
            let accepted = core.clone();
            let before_tick = roots(&accepted);
            core.advance_active_battle_for_server_tick(1).unwrap();
            let clean = roots(&core);
            assert_eq!(hp(&core, &[actor]), 537);
            let defaults = native::RuntimeFactLimits::default();
            let (mut low, mut high) = (1, defaults.max_work);
            while low < high {
                let middle = low + (high - low) / 2;
                let mut trial = accepted.clone();
                let result = native::run(
                    native::RuntimeFactLimits {
                        max_work: middle,
                        ..defaults
                    },
                    || trial.advance_active_battle_for_server_tick(1),
                );
                match result {
                    Ok(_) => {
                        assert_eq!(roots(&trial), clean);
                        high = middle;
                    }
                    Err(GameError::BattleRuntimeFault(fault))
                        if fault.code == native::RuntimeFaultCode::WorkLimit =>
                    {
                        assert_eq!(roots(&trial), before_tick);
                        low = middle + 1;
                    }
                    other => panic!("unexpected lethal tick work boundary: {other:?}"),
                }
            }
            assert!(high > 1 && high < defaults.max_work);
            let mut retry = accepted.clone();
            let error = native::run(
                native::RuntimeFactLimits {
                    max_work: high - 1,
                    ..defaults
                },
                || retry.advance_active_battle_for_server_tick(1),
            )
            .unwrap_err();
            assert!(matches!(error, GameError::BattleRuntimeFault(ref fault)
                if fault.code == native::RuntimeFaultCode::WorkLimit));
            assert_eq!(roots(&retry), before_tick);
            native::run(
                native::RuntimeFactLimits {
                    max_work: high,
                    ..defaults
                },
                || retry.advance_active_battle_for_server_tick(1),
            )
            .unwrap();
            assert_eq!(roots(&retry), clean);
            println!("RV005 lethal_tick native_work_minimum={high} one_short_fault={error:?} accepted_state_rollback=true exact_retry_clean=true actual_hp_lost10_self_heal37=true");
        } else {
            cast(&mut core, player, employee);
        }
        assert_eq!(before[0] - target_hp(&core, targets[0]), losses[0]);
        assert_eq!(before[1] - target_hp(&core, targets[1]), losses[1]);
        assert_eq!(hp(&core, &[actor]), caster_after);
        assert_eq!(step_count(&core), 1 + heals);
        let battle = &core.state.active_battle.as_ref().unwrap().battle;
        let facts = battle.runtime_fact_summary();
        assert_eq!(facts.heal_mutations, heals as u64);
        if mode == "invulnerable" {
            assert_eq!(
                facts
                    .latest_damage
                    .as_ref()
                    .unwrap()
                    .protection
                    .deflection
                    .as_ref()
                    .unwrap()
                    .kind,
                crate::game::battle::runtime_facts::DeflectionKind::Invulnerability
            );
        }
        if mode == "lethal" {
            assert!(battle
                .units
                .get(&targets[0])
                .is_none_or(|unit| !unit.is_active()));
            assert_eq!(
                facts
                    .latest_death
                    .as_ref()
                    .unwrap()
                    .lethal_damage
                    .as_ref()
                    .unwrap()
                    .actual_hp_loss(),
                10
            );
        }
        assert_eq!(native_probe::active_cast_count(battle), 0);
        println!("RV005 footprint mode={mode} target_losses={losses:?} caster={caster_after} self_heals={heals} actual_hp_loss_gate=true");
    }
}
