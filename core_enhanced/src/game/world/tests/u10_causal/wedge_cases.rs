use super::*;

pub(super) fn wedge_input(extra_rules: &str) -> String {
    relic_input(&format!(
        r#"(id:"wedge",selector:(effect:(all_tags:["u10_multi_hit"])),apply:Wedge(increment_percent:10)){extra_rules}"#
    ))
}

#[test]
fn u10_g_wedge_first_strike_counts_and_new_cast_resets() {
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[100]), (10, &[100]), (20, &[200])]),
        &wedge_input(""),
        &["u10_relic"],
        true,
    );
    let before = fixture.target_hp();
    fixture.cast();
    for (elapsed, expected_loss) in [(1, 110), (10, 230), (10, 490)] {
        fixture.advance(elapsed);
        assert_eq!(before - fixture.target_hp(), expected_loss);
    }
    let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
    existing_native_probe::refill_skill(battle, fixture.actor, 21);
    fixture.cast();
    fixture.advance(1);
    assert_eq!(
        before - fixture.target_hp(),
        600,
        "new cast begins at candidate one, regardless of previous cast count"
    );
}

#[test]
fn u10_g_wedge_zero_full_and_partial_absorption_use_actual_positive_loss() {
    for (first, shield, first_loss, total_loss) in
        [(100, 110, 0, 110), (100, 109, 1, 121), (0, 0, 0, 110)]
    {
        let mut fixture = CausalWorld::prepare(
            &skill_input(&[(0, &[first]), (10, &[100])]),
            &wedge_input(""),
            &["u10_relic"],
            true,
        );
        if shield != 0 {
            let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
            for target in &fixture.targets {
                existing_native_probe::legacy_shield(battle, *target, shield);
            }
        }
        let before = fixture.target_hp();
        fixture.cast();
        fixture.advance(1);
        assert_eq!(
            before - fixture.target_hp(),
            first_loss,
            "first={first} shield={shield}"
        );
        fixture.advance(10);
        assert_eq!(
            before - fixture.target_hp(),
            total_loss,
            "first={first} shield={shield}"
        );
    }
}

#[test]
fn u10_g_same_strike_segments_share_candidate_but_same_time_strikes_do_not() {
    for (strikes, shield, first_loss, total_loss) in [
        (vec![(0, &[100, 100][..]), (10, &[100][..])], 0, 220, 340),
        (vec![(0, &[100, 100][..]), (10, &[100][..])], 110, 110, 230),
        (
            vec![(0, &[100][..]), (0, &[100][..]), (10, &[100][..])],
            0,
            230,
            360,
        ),
    ] {
        let mut fixture = CausalWorld::prepare(
            &skill_input(&strikes),
            &wedge_input(""),
            &["u10_relic"],
            true,
        );
        if shield != 0 {
            let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
            for target in &fixture.targets {
                existing_native_probe::legacy_shield(battle, *target, shield);
            }
        }
        let before = fixture.target_hp();
        fixture.cast();
        fixture.advance(1);
        assert_eq!(before - fixture.target_hp(), first_loss);
        fixture.advance(10);
        assert_eq!(before - fixture.target_hp(), total_loss);
    }
}

#[test]
fn u10_g_wedge_percent_adds_to_existing_numeric_class_after_flat() {
    let relics = wedge_input("").replacen(
        "effects:[]",
        "effects:[(id:\"ordinary_numeric\",flat:20,percent:25)]",
        1,
    );
    let mut fixture =
        CausalWorld::prepare(&skill_input(&[(0, &[100])]), &relics, &["u10_relic"], true);
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(
        before - fixture.target_hp(),
        162,
        "independent oracle: (100 + 20) * (100 + 25 + 10) / 100"
    );
}

#[test]
fn u10_g_other_relic_child_damage_does_not_advance_original_cast_wedge() {
    let extra = format!(",{}", damage_subscriber("extra_damage", 7, 0));
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[100]), (10, &[100])]),
        &wedge_input(&extra),
        &["u10_relic"],
        true,
    );
    let before = fixture.target_hp();
    fixture.cast();
    fixture.advance(1);
    assert_eq!(before - fixture.target_hp(), 117);
    fixture.advance(10);
    assert_eq!(
        before - fixture.target_hp(),
        244,
        "second original hit is 120, and each original independent path permits one child 7"
    );
}

#[test]
fn u10_g_two_copy_percent_contributions_add_before_damage_composition() {
    let first = wedge_input("");
    let second = first
        .replacen("id:\"u10_relic\"", "id:\"u10_second\"", 1)
        .replacen("00000000d101", "00000000d102", 1)
        .replacen("increment_percent:10", "increment_percent:20", 1);
    let body = |input: &str| {
        input
            .strip_prefix("(relics:[")
            .unwrap()
            .strip_suffix("])")
            .unwrap()
            .to_owned()
    };
    let relics = format!("(relics:[{},{}])", body(&first), body(&second));
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[100]), (10, &[100]), (20, &[200])]),
        &relics,
        &["u10_relic", "u10_second"],
        true,
    );
    let before = fixture.target_hp();
    fixture.cast();
    for (elapsed, expected_loss) in [(1, 130), (10, 290), (10, 670)] {
        fixture.advance(elapsed);
        assert_eq!(
            before - fixture.target_hp(),
            expected_loss,
            "r10 + r20 yields independent strike amounts 130, 160, 380"
        );
    }
}

#[test]
fn u10_g_spatial_deliveries_within_one_cast_count_each_actual_target_separately() {
    let skills = skill_input(&[(0, &[100]), (10, &[100]), (20, &[100])]).replace(
        "target:CastTarget,delivery:Instant",
        r#"target:SelfUnit,delivery:TileArea(
            footprint:(include_anchor_tile:false,rows:["X","@"]),
            area:(anchor:Caster,tile_origin:Caster,tracking:GroundFixed,
                hit_targets:Enemies,include_caster:false,tick_policy:OncePerArea,
                duration_ms:0,tick_interval_ms:None))"#,
    );
    let mut fixture = CausalWorld::prepare_full(
        &skills,
        &wedge_input(""),
        &["u10_relic"],
        true,
        None,
        Some(2),
    );
    assert_eq!(fixture.targets.len(), 2);
    let first = fixture.targets[0];
    let second = fixture.targets[1];
    let before_first = fixture.hp(first);
    let before_second = fixture.hp(second);
    fixture.cast();
    fixture.advance(1);
    assert_eq!(before_first - fixture.hp(first), 110);
    assert_eq!(before_second - fixture.hp(second), 0);
    let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
    existing_native_probe::place_stationary(battle, first, 3, 0);
    existing_native_probe::place_stationary(battle, second, 1, 0);
    fixture.advance(10);
    assert_eq!(before_first - fixture.hp(first), 110);
    assert_eq!(
        before_second - fixture.hp(second),
        110,
        "another recipient starts at one even inside the same accepted cast"
    );
    let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
    existing_native_probe::place_stationary(battle, first, 1, 0);
    existing_native_probe::place_stationary(battle, second, 3, 0);
    fixture.advance(10);
    assert_eq!(
        before_first - fixture.hp(first),
        230,
        "returning to the first recipient advances its own second strike"
    );
    assert_eq!(before_second - fixture.hp(second), 110);
}

#[test]
fn u10_g_withdrawal_cancels_unreleased_steps_but_released_projectiles_keep_wedge() {
    for released in [false, true] {
        let skills = if released {
            skill_input(&[(0, &[100]), (0, &[100])]).replace(
                "delivery:Instant",
                "delivery:Projectile(speed_units_per_ms:100000)",
            )
        } else {
            skill_input(&[(20, &[100]), (30, &[100])])
        };
        let mut fixture = CausalWorld::prepare(&skills, &wedge_input(""), &["u10_relic"], true);
        let before = fixture.target_hp();
        fixture.cast();
        fixture.advance(1);
        assert_eq!(fixture.target_hp(), before, "pre-impact checkpoint");
        execute_current_battle_mutation(
            &mut fixture.core,
            fixture.player,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: fixture.employee,
            },
        )
        .unwrap();
        fixture.advance(100);
        assert_eq!(
            before - fixture.target_hp(),
            if released { 230 } else { 0 },
            "released={released}: accepted source and shared cast count follow carrier lifetime"
        );
        fixture.advance(100);
        assert_eq!(before - fixture.target_hp(), if released { 230 } else { 0 });
        assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
    }
}

#[test]
fn u10_g_invulnerable_first_strike_does_not_consume_wedge_count() {
    let mut fixture = CausalWorld::prepare(
        &skill_input(&[(0, &[100]), (110, &[100])]),
        &wedge_input(""),
        &["u10_relic"],
        true,
    );
    let before = fixture.target_hp();
    let battle = &mut fixture.core.state.active_battle.as_mut().unwrap().battle;
    crate::game::battle::core::relic_causal_tests::prepare_existing_invulnerability(
        battle,
        fixture.actor,
        fixture.targets[0],
    );
    fixture.cast();
    fixture.advance(1);
    assert_eq!(before - fixture.target_hp(), 0);
    fixture.advance(110);
    assert_eq!(
        before - fixture.target_hp(),
        110,
        "the first post-invulnerability positive strike still uses candidate one"
    );
}

#[test]
fn u10_g_structural_guards_and_rounding_precede_numeric_and_wedge_composition() {
    let edits = r#"
        ,(id:"replace",selector:(effect:(all_tags:["u10_direct"])),
            apply:Edit(property:DamageAmount,operation:Replace(Hp(80))))
        ,(id:"subtract",selector:(effect:(all_tags:["u10_direct"])),
            apply:Edit(property:DamageAmount,operation:Add(Hp(-20))))
        ,(id:"halve",selector:(effect:(all_tags:["u10_direct"])),
            apply:Edit(property:DamageAmount,operation:Scale(increment_percent:-50)))
    "#;
    // This one-copy control must also be valid under publication's complete
    // legal coequip check: three duplicate -50% copies would be a negative scale.
    let pure = relic_input(edits.trim().trim_start_matches(',')).replacen(
        "allow_duplicates:true",
        "allow_duplicates:false",
        1,
    );
    let combined = wedge_input(edits).replacen(
        "effects:[]",
        r#"effects:[(id:"ordinary_numeric",flat:20,percent:25)]"#,
        1,
    );
    for (relics, dto_amount, first, total) in [(&pure, 30, 30, 60), (&combined, 60, 68, 144)] {
        let mut fixture = CausalWorld::prepare(
            &skill_input(&[(0, &[100]), (10, &[100])]),
            relics,
            &["u10_relic"],
            true,
        );
        let query = fixture
            .core
            .execute(
                fixture.player,
                PlayerBehavior::RequestRelicEffectivePlan {
                    employee_uuid: fixture.employee,
                },
            )
            .unwrap();
        let BehaviorResult::RelicEffectivePlan { result: Some(plan) } = query else {
            panic!("ordinary structural plan query");
        };
        let amounts: Vec<_> = plan
            .effects
            .iter()
            .flat_map(|effect| &effect.properties)
            .filter(|property| {
                property.property
                    == crate::game::relic::structural::authoring::RelicProperty::DamageAmount
            })
            .map(|property| property.final_value)
            .collect();
        assert_eq!(amounts, vec![
            Some(crate::game::relic::structural::authoring::TypedRelicValue::Hp(dto_amount)),
            Some(crate::game::relic::structural::authoring::TypedRelicValue::Hp(dto_amount)),
        ], "pure U09 is (80-20)*50%=30; legacy contributions normalize into the same program: (80-20+20)*75%=60");
        let before = fixture.target_hp();
        fixture.cast();
        fixture.advance(1);
        assert_eq!(before - fixture.target_hp(), first,
            "Wedge adds to the common percentage class before one rounding: (80-20+20)*(100-50+25+10)/100=68");
        fixture.advance(10);
        assert_eq!(before - fixture.target_hp(), total,
            "second Wedge strike is (80-20+20)*(100-50+25+20)/100=76; cumulative144; pure control stays30 each");
        assert_causal_reclaimed(&fixture.core.state.active_battle.as_ref().unwrap().battle);
    }
}
