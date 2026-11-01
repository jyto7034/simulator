use super::canonical_area_tests::{holes, replace_holes};
use super::*;
use serde_json::{json, Value as Json};

pub(crate) const FRAGMENT: &str = "fragment_nothing_there_trace";
pub(crate) const SKILL: &str = "fragment_nothing_there_hello";
pub(crate) const FINISH: &str = "fragment_nothing_there_goodbye";
pub(crate) const FAMILY: &str = "o-06-20_nothing_there";
pub(crate) fn original(kind: &str) -> &'static str {
    match kind {
        "fragment" => {
            include_str!("../../../../game_resources/authoring/nothing-there/fragment.skill.yaml")
        }
        "equipment" => {
            include_str!("../../../../game_resources/authoring/nothing-there/equipment.skill.yaml")
        }
        _ => panic!("unknown source"),
    }
}
pub(crate) fn inputs() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_overlay.values.json"
    ))
    .unwrap()["values"]
        .clone()
}
pub(crate) fn resolved(kind: &str) -> Json {
    let mut d = parse_yaml(original(kind)).unwrap();
    replace_holes(&mut d, &inputs());
    d
}
pub(crate) fn overlay(d: &mut Json) -> &mut Json {
    &mut d["skills"][0]["body"]["Action"]["effects"][0]["Append"]["effects"]["Literal"]["Effects"]
        [0]["op"]["Sustained"]["definition"]["BasicAttackOverlay"]
}
pub(crate) fn fixture() -> crate::authoring::node_graph::NativeGraphContent {
    let mut d = resolved("fragment");
    d["skills"][0]["id"] = json!("r11_aura");
    d["skills"][1]["id"] = json!("r11_aura__finish");
    overlay(&mut d)["goodbye_skill"] = json!("r11_aura__finish");
    let skills = compile_json(&d.to_string()).unwrap().load_native().unwrap();
    crate::authoring::node_graph::NativeGraphContent {
        skills,
        equipment: vec![],
        mark_weapons: vec![],
    }
}
pub(crate) const FRAGMENT_CATALOG: &str =
    include_str!("../../../../game_resources/authoring/nothing-there/fragment.catalog.json");
pub(crate) const EQUIPMENT_CATALOG: &str =
    include_str!("../../../../game_resources/authoring/nothing-there/equipment.catalog.json");
pub(crate) fn fragment() -> crate::game::data::skill_fragment_data::SkillFragmentMetadata {
    let mut d: Json = serde_json::from_str(FRAGMENT_CATALOG).unwrap();
    replace_holes(&mut d, &inputs());
    serde_json::from_value(d).unwrap()
}
pub(crate) fn equipment() -> Vec<crate::game::data::equipment_data::EquipmentMetadata> {
    let mut d: Json = serde_json::from_str(EQUIPMENT_CATALOG).unwrap();
    replace_holes(&mut d, &inputs());
    serde_json::from_value(d).unwrap()
}
pub(crate) fn equipment_fixture(kind: &str) -> Json {
    let mut d = resolved("equipment");
    let (i, field, binding) = match kind {
        "weapon" => (0, "equipment_attacks", 0),
        "armor" => (1, "equipment_defenses", 0),
        "accessory" => (2, "equipment_defenses", 1),
        _ => panic!(),
    };
    let value = d["values"][i].clone();
    let mut entry = d[field][binding].clone();
    entry["equipment"] = json!(if kind == "accessory" {
        "r11_accessory"
    } else {
        "r11_weapon"
    });
    d["values"] = json!([value]);
    d.as_object_mut().unwrap().remove("equipment_attacks");
    d.as_object_mut().unwrap().remove("equipment_defenses");
    if kind != "accessory" {
        d.as_object_mut().unwrap().remove("resources");
    }
    d[field] = json!([entry]);
    d
}
pub(crate) fn relics() -> crate::game::data::relic_data::RelicDatabase {
    let mut d: Json = serde_json::from_str(include_str!(
        "../../../../game_resources/authoring/relics/overlay-modifiers.catalog.json"
    ))
    .unwrap();
    let v: Json = serde_json::from_str(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_area_strikes.values.json"
    ))
    .unwrap();
    replace_holes(&mut d, &v["relics"]);
    serde_json::from_value(d).unwrap()
}

#[test]
fn canonical_overlay_sources_preserve_each_unknown_and_existing_effect_programs() {
    for (kind, count) in [("fragment", 9), ("equipment", 2)] {
        let raw = parse_yaml(original(kind)).unwrap();
        let d = resolved(kind);
        let mut pending = vec![];
        holes(&raw, "", &mut pending);
        assert_eq!(pending.len(), count);
        for (path, marker) in pending {
            let mut one = d.clone();
            *one.pointer_mut(&path).unwrap() = marker;
            let error = compile_json(&one.to_string()).unwrap_err();
            assert_eq!(error.code, DiagnosticCode::Unresolved, "{path}: {error:?}");
            assert!(!error.source.definition.is_empty());
        }
        let a = compile_json(&d.to_string()).unwrap();
        Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
    }
    let graph = crate::authoring::node_graph::compile(
        &crate::authoring::node_graph::tests::attack_overlay_fixture().to_string(),
        "r11_aura".into(),
    )
    .unwrap()
    .load_native()
    .unwrap();
    let actual = fixture();
    for (a, b) in actual.skills.skills.iter().zip(&graph.skills) {
        assert_eq!(
            serde_json::to_value(&a.steps[0].effects).unwrap(),
            serde_json::to_value(&b.steps[0].effects).unwrap()
        );
        assert_eq!(a.resonance_admission, b.resonance_admission);
        assert_eq!(a.focus_permissions, b.focus_permissions);
        assert_eq!(a.cast_targeting, b.cast_targeting);
    }
    let a = compile_json(&resolved("equipment").to_string()).unwrap();
    let weapon = compile_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/equipment_lifesteal.skill.yaml"
    ))
    .unwrap();
    let armor = compile_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/adaptive_shield.skill.yaml"
    ))
    .unwrap();
    let accessory = compile_yaml(include_str!(
        "../../../tests/fixtures/skill_composition/equipment_metamorphosis.skill.yaml"
    ))
    .unwrap();
    assert_eq!(
        serde_json::to_value(&a.equipment_attacks()[0].program).unwrap(),
        serde_json::to_value(&weapon.equipment_attacks()[0].program).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&a.equipment_defenses()[0].program).unwrap(),
        serde_json::to_value(&armor.equipment_defenses()[0].program).unwrap()
    );
    let actual = serde_json::to_value(&a.equipment_defenses()[1].program).unwrap();
    let mut expected = serde_json::to_value(&accessory.equipment_defenses()[0].program).unwrap();
    expected["EmployeeRevival"]["transform_into"] = json!(FAMILY);
    assert_eq!(actual, expected);
    assert_eq!(
        serde_json::to_value(a.resources()).unwrap(),
        serde_json::to_value(accessory.resources()).unwrap()
    );
}

#[test]
fn canonical_overlay_catalog_preserves_ids_known_values_and_unresolved_release_choices() {
    let old: crate::game::data::equipment_data::EquipmentDatabase = ron::de::from_str(
        include_str!("../../../../game_resources/data/equipments/base.ron"),
    )
    .unwrap();
    let gear = equipment();
    for item in &gear {
        let previous = old.get_by_id(&item.id).unwrap();
        assert_eq!(
            (item.uuid, item.equipment_type),
            (previous.uuid, previous.equipment_type)
        );
        assert_eq!(item.content_family.as_deref(), Some(FAMILY));
    }
    let profile = gear[0].weapon_profile.as_ref().unwrap();
    assert_eq!(
        profile.weapon_archetype,
        crate::game::data::equipment_data::WeaponArchetype::Sword
    );
    assert_eq!(profile.block_capacity, 1);
    assert!(!profile.air_capable);
    assert_eq!(
        profile.damage_type,
        crate::game::battle::damage::DamageType::Physical
    );
    let f = fragment();
    assert_eq!(f.id.as_str(), FRAGMENT);
    assert_eq!(f.uuid.to_string(), "730e8400-e29b-41d4-a716-446655440003");
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = f.effect
    else {
        panic!()
    };
    assert_eq!(forms.len(), 1);
    assert_eq!(forms["base"].skill_id.as_str(), SKILL);
    assert_eq!(resolved("fragment")["skills"][1]["id"], FINISH);
    assert!(
        serde_json::from_str::<crate::game::data::skill_fragment_data::SkillFragmentMetadata>(
            FRAGMENT_CATALOG
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<Vec<crate::game::data::equipment_data::EquipmentMetadata>>(
            EQUIPMENT_CATALOG
        )
        .is_err()
    );
    let raw = parse_yaml(original("equipment")).unwrap();
    assert_eq!(
        raw["values"][0]["value"]["AttackProgram"]["basic_lifesteal"]["percent"],
        25
    );
    assert_eq!(
        raw["values"][1]["value"]["DefenseProgram"]["AdaptiveBasicShield"],
        json!({"max_hp_percent":20,"duration_ms":5000,"cooldown_ms":15000,"priority":0})
    );
    let mut revive = raw["values"][2]["value"]["DefenseProgram"]["EmployeeRevival"].clone();
    revive.as_object_mut().unwrap().remove("transform_into");
    assert_eq!(
        revive,
        json!({"cumulative":"revival_total","battle_used":"revival_used","max_revivals":2,"restore_percent":50,"invulnerable_ms":1000})
    );
    relics().validate_contract().unwrap();
    assert_eq!(relics().relics[0].id, "piercing-damage-increase");
}
