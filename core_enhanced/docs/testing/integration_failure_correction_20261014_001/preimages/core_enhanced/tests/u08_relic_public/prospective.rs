//! Prospective query expectations bind to the agreed public DTO after its
//! producer is published. Setup and assertions use public ports exclusively.
use super::support::*;
use serde_json::{json, Value};

fn skill<'a>(plan: &'a Value, skill_id: &str, effect: &str) -> &'a Value {
    plan["targets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|target| {
            target["kind"] == "skill_damage"
                && target["site"]["action"]["definition_id"] == skill_id
                && target["site"]["effect_id"] == effect
        })
        .unwrap_or_else(|| panic!("missing {skill_id}/{effect}: {plan}"))
}

fn basic(plan: &Value) -> &Value {
    plan["targets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["kind"] == "basic_attack_damage")
        .unwrap()
}

fn effect<'a>(target: &'a Value, id: &str) -> &'a Value {
    target["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["effect_id"] == id)
        .unwrap()
}

fn available(target: &Value, base: u64, effective: u64) {
    assert_eq!(
        target["evaluation"],
        json!({"status":"available", "base_amount":base, "effective_amount":effective})
    );
}

#[test]
fn u08_public_external_fixture_registers_new_definitions_before_app_setup() {
    let fixture = content();
    assert!(fixture.root.join("u08_public/relics.ron").is_file());
    assert_eq!(fixture.selection.packs.len(), 2);
    let (mut app, employees) = app();
    let copy = grant_relic(&mut app, "u08_public_blend");
    equip_relic(&mut app, copy, employees[0], 0);
    let state = relic_state(&mut app);
    assert_eq!(state.copies.len(), 1);
    assert_eq!(state.loadouts[0].employee_uuid, employees[0]);
    assert_eq!(state.loadouts[0].slots, [Some(copy), None, None]);
}

#[test]
fn u08_public_plan_two_employees_same_skill_and_queries_preserve_all_public_roots() {
    let (mut app, employees) = app();
    for employee in &employees[..2] {
        equip_fragment(&mut app, *employee, "u08_public_fragment");
        equip_weapon(&mut app, *employee, "u08_public_weapon");
    }
    let copy = grant_relic(&mut app, "u08_public_blend");
    equip_relic(&mut app, copy, employees[0], 2);
    let before = public_roots(&app);
    let first = numeric_plan(&mut app, employees[0]);
    let second = numeric_plan(&mut app, employees[1]);
    assert_eq!(first["employee_uuid"], json!(employees[0]));
    assert_eq!(first["scope"], "prospective_loadout");
    assert_eq!(first["generation"], json!(content().data.identity()));
    let hit = skill(&first, "u08_public_literal", "both");
    available(hit, 100, 180);
    available(skill(&second, "u08_public_literal", "both"), 100, 100);
    assert_eq!(hit["effects"].as_array().unwrap().len(), 2);
    assert!(hit["effects"]
        .as_array()
        .unwrap()
        .iter()
        .all(|row| row["copy_uuid"] == json!(copy) && row["state"]["status"] == "active"));
    assert_eq!(numeric_plan(&mut app, employees[0]), first);
    assert_eq!(
        public_roots(&app),
        before,
        "queries must not mint actions, consume IDs or change persistence"
    );
    assert!(!first.to_string().contains("runtime_item_instance_id"));
    assert!(!first.to_string().contains("action_ordinal"));
    assert!(!first.to_string().contains("attack_epoch"));
}

#[test]
fn u08_public_plan_uses_real_fragment_form_and_weapon_provider_without_cross_borrowing() {
    let (mut app, employees) = app();
    let employee = employees[0];
    let fragment = equip_fragment(&mut app, employee, "u08_public_fragment");
    let weapon = equip_weapon(&mut app, employee, "u08_public_weapon");
    let copy = grant_relic(&mut app, "u08_public_exact");
    equip_relic(&mut app, copy, employee, 0);
    let plan = numeric_plan(&mut app, employee);
    let hit = skill(&plan, "u08_public_literal", "both");
    available(hit, 100, 133); // (100 + 7) * 125 / 100, TowardZero.
    assert!(hit["provider"]["equipment"].is_null());
    assert_eq!(hit["provider"]["fragment"]["copy_uuid"], json!(fragment));
    assert_eq!(
        effect(hit, "weapon_is_not_every_skill_provider")["state"],
        json!({"status":"dormant", "reason":"equipment"})
    );
    let attack = basic(&plan);
    assert_eq!(attack["weapon"]["persistent_copy_uuid"], json!(weapon));
    assert_eq!(
        attack["evaluation"]["effective_amount"].as_u64().unwrap(),
        attack["evaluation"]["base_amount"].as_u64().unwrap() + 9
    );
    assert_eq!(
        effect(attack, "fragment_base")["state"]["status"],
        "dormant"
    );
    select_form(&mut app, fragment, "same_skill");
    let changed = numeric_plan(&mut app, employee);
    let hit = skill(&changed, "u08_public_literal", "both");
    available(hit, 100, 100);
    assert_eq!(
        effect(hit, "fragment_base")["state"],
        json!({"status":"dormant", "reason":"form"})
    );
    equip_fragment(&mut app, employee, "u08_public_other_fragment");
    let changed = numeric_plan(&mut app, employee);
    let hit = skill(&changed, "u08_public_literal", "both");
    available(hit, 100, 100);
    assert_eq!(
        effect(hit, "fragment_base")["state"],
        json!({"status":"dormant", "reason":"fragment"})
    );
}

#[test]
fn u08_public_plan_split_tags_unresolved_target_and_heal_basic_are_distinct() {
    let (mut app, employees) = app();
    let employee = employees[0];
    let fragment = equip_fragment(&mut app, employee, "u08_public_fragment");
    equip_weapon(&mut app, employee, "u08_public_weapon");
    let copy = grant_relic(&mut app, "u08_public_tags");
    equip_relic(&mut app, copy, employee, 1);
    select_form(&mut app, fragment, "split");
    let plan = numeric_plan(&mut app, employee);
    available(skill(&plan, "u08_public_split", "one"), 3, 3);
    available(skill(&plan, "u08_public_split", "two"), 3, 3);
    available(skill(&plan, "u08_public_split", "both"), 3, 34);
    assert_eq!(
        effect(basic(&plan), "both_tags")["state"],
        json!({"status":"dormant", "reason":"tags"})
    );
    select_form(&mut app, fragment, "live");
    let live = numeric_plan(&mut app, employee);
    assert_eq!(
        skill(&live, "u08_public_target", "target_attack")["evaluation"],
        json!({"status":"unresolved", "reason":"runtime_context_required"})
    );
    equip_weapon(&mut app, employee, "u08_public_heal_weapon");
    let healing = numeric_plan(&mut app, employee);
    assert!(
        healing["targets"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["kind"] != "basic_attack_damage"),
        "Heal is not a prospective Damage producer"
    );
}

#[test]
fn u08_public_plan_corrupt_penalty_matches_roster_and_preserves_certified_power() {
    let (mut app, employees) = app();
    let employee = employees[0];
    equip_fragment(&mut app, employee, "u08_public_fragment");
    equip_weapon(&mut app, employee, "u08_public_weapon");
    let copy = grant_relic(&mut app, "u08_public_blend");
    equip_relic(&mut app, copy, employee, 0);
    let normal = numeric_plan(&mut app, employee);
    let certified = app
        .export_active_run_checkpoint()
        .unwrap()
        .unwrap()
        .run_peak_certified_power();
    admin(
        &mut app,
        json!({"type":"admin_grant_relic_corruption_catalyst", "amount":1}),
    );
    command(&mut app, json!({"type":"corrupt_relic", "copy_uuid":copy}));
    let corrupt = numeric_plan(&mut app, employee);
    assert_eq!(corrupt["base_battle_stats"], normal["base_battle_stats"]);
    assert_eq!(
        corrupt["effective_battle_stats"]["defense"]
            .as_i64()
            .unwrap(),
        normal["base_battle_stats"]["defense"].as_i64().unwrap() - 15
    );
    assert_eq!(
        corrupt["effective_battle_stats"]["magic_resist"]
            .as_i64()
            .unwrap(),
        normal["base_battle_stats"]["magic_resist"]
            .as_i64()
            .unwrap()
            - 10
    );
    let employee_view = roster(&app)
        .employees
        .into_iter()
        .find(|row| row.uuid == employee)
        .unwrap();
    assert_eq!(
        json!(employee_view.combat_profile.effective_stats.unwrap()),
        corrupt["effective_battle_stats"]
    );
    assert_eq!(
        app.export_active_run_checkpoint()
            .unwrap()
            .unwrap()
            .run_peak_certified_power(),
        certified
    );
    available(skill(&corrupt, "u08_public_literal", "both"), 100, 210);
    let before = public_roots(&app);
    assert_eq!(numeric_plan(&mut app, employee), corrupt);
    assert_eq!(public_roots(&app), before);
}

#[test]
fn u08_public_plan_canonical_copy_order_survives_slot_permutation_and_rejects_final_overflow() {
    let (mut app, employees) = app();
    let employee = employees[0];
    let fragment = equip_fragment(&mut app, employee, "u08_public_fragment");
    equip_weapon(&mut app, employee, "u08_public_weapon");
    let first = grant_relic(&mut app, "u08_public_blend");
    let second = grant_relic(&mut app, "u08_public_blend");
    equip_relic(&mut app, first, employee, 0);
    equip_relic(&mut app, second, employee, 2);
    let original = numeric_plan(&mut app, employee);
    available(skill(&original, "u08_public_literal", "both"), 100, 280);
    for copy in [first, second] {
        command(&mut app, json!({"type":"unequip_relic", "copy_uuid":copy}));
    }
    equip_relic(&mut app, second, employee, 0);
    equip_relic(&mut app, first, employee, 1);
    assert_eq!(
        numeric_plan(&mut app, employee),
        original,
        "slot order cannot change canonical contribution order or one final rounding"
    );
    select_form(&mut app, fragment, "maximum");
    let before = public_roots(&app);
    let rejected = numeric_plan(&mut app, employee);
    assert_eq!(
        skill(&rejected, "u08_public_max", "maximum")["evaluation"],
        json!({"status":"rejected", "fault":{"stage":"final", "code":"overflow"}})
    );
    assert_eq!(
        public_roots(&app),
        before,
        "a rejected prospective amount is also read-only"
    );
    for copy in [first, second] {
        command(&mut app, json!({"type":"unequip_relic", "copy_uuid":copy}));
    }
    available(
        skill(
            &numeric_plan(&mut app, employee),
            "u08_public_max",
            "maximum",
        ),
        u64::from(u32::MAX),
        u64::from(u32::MAX),
    );
}

#[test]
fn u08_public_plan_missing_weapon_has_no_fabricated_stats_or_damage_provider() {
    for corrupt in [false, true] {
        let (mut app, employees) = app();
        let employee = employees[0];
        equip_fragment(&mut app, employee, "u08_public_fragment");
        let weapon = equip_weapon(&mut app, employee, "u08_public_weapon");
        let copy = grant_relic(&mut app, "u08_public_blend");
        equip_relic(&mut app, copy, employee, 0);
        admin(
            &mut app,
            json!({"type":"admin_grant_relic_corruption_catalyst", "amount":1}),
        );
        if corrupt {
            command(&mut app, json!({"type":"corrupt_relic", "copy_uuid":copy}));
        }
        let equipped = numeric_plan(&mut app, employee);
        assert!(equipped["base_battle_stats"].is_object());
        assert!(equipped["effective_battle_stats"].is_object());
        assert!(equipped["stats_unavailable_reason"].is_null());
        assert_eq!(
            equipped["penalties"].as_array().unwrap().len(),
            if corrupt { 2 } else { 0 }
        );
        let relics_before = json!(relic_state(&mut app));
        assert_eq!(relics_before["copies"][0]["copy_uuid"], json!(copy));
        assert_eq!(
            relics_before["copies"][0]["variant"],
            if corrupt { "Corrupt" } else { "Normal" }
        );
        assert_eq!(
            relics_before["loadouts"][0]["employee_uuid"],
            json!(employee)
        );
        assert_eq!(
            relics_before["loadouts"][0]["slots"],
            json!([copy, null, null])
        );
        assert_eq!(
            relics_before["resources"]["relic_corruption_catalysts"],
            if corrupt { 0 } else { 1 }
        );
        command(
            &mut app,
            json!({"type":"un_equip_item", "item_uuid":weapon, "target_unit":employee}),
        );
        let relics_after = json!(relic_state(&mut app));
        assert_eq!(
            relics_after, relics_before,
            "weapon removal preserves every relic copy, slot, variant and resource"
        );
        let employee_view = roster(&app)
            .employees
            .into_iter()
            .find(|row| row.uuid == employee)
            .unwrap();
        assert!(
            employee_view.combat_profile.effective_stats.is_none(),
            "normal employee projection requires an actual equipped weapon"
        );
        let before = public_roots(&app);
        let unavailable = numeric_plan(&mut app, employee);
        assert!(unavailable["base_battle_stats"].is_null());
        assert!(unavailable["effective_battle_stats"].is_null());
        assert_eq!(
            unavailable["stats_unavailable_reason"],
            "provider_unavailable"
        );
        assert_eq!(unavailable["targets"], json!([]));
        assert_eq!(unavailable["penalties"], equipped["penalties"]);
        assert_eq!(numeric_plan(&mut app, employee), unavailable);
        assert_eq!(public_roots(&app), before);
    }
}
