use super::canonical_area_tests::{holes, replace_holes};
use super::*;
use serde_json::{json, Value as Json};

pub(crate) const FRAGMENT: &str = "fragment_burrowing_heaven_trace";
pub(crate) const SKILL: &str = "fragment_burrowing_heaven_red_eye";
pub(crate) const FAMILY: &str = "o-04-72_burrowing_heaven";
pub(crate) fn original(kind: &str) -> &'static str {
    match kind {
        "fragment" => include_str!(
            "../../../../game_resources/authoring/burrowing-heaven/fragment.skill.yaml"
        ),
        "equipment" => include_str!(
            "../../../../game_resources/authoring/burrowing-heaven/equipment.skill.yaml"
        ),
        _ => panic!("unknown source"),
    }
}
pub(crate) fn inputs() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_channel.values.json"
    ))
    .unwrap()["values"]
        .clone()
}
pub(crate) fn resolved(kind: &str) -> Json {
    let mut d = parse_yaml(original(kind)).unwrap();
    replace_holes(&mut d, &inputs());
    d
}
pub(crate) const FRAGMENT_CATALOG: &str =
    include_str!("../../../../game_resources/authoring/burrowing-heaven/fragment.catalog.json");
pub(crate) const EQUIPMENT_CATALOG: &str =
    include_str!("../../../../game_resources/authoring/burrowing-heaven/equipment.catalog.json");
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
pub(crate) fn relics() -> crate::game::data::relic_data::RelicDatabase {
    let mut d: Json = serde_json::from_str(include_str!(
        "../../../../game_resources/authoring/relics/channel-modifiers.catalog.json"
    ))
    .unwrap();
    let values: Json = serde_json::from_str(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_area_strikes.values.json"
    ))
    .unwrap();
    replace_holes(&mut d, &values["relics"]);
    let mut r: crate::game::data::relic_data::RelicDatabase = serde_json::from_value(d).unwrap();
    r.relics.extend(
        super::canonical_area_tests::relics()
            .relics
            .into_iter()
            .filter(|r| {
                [
                    "widened-horizon",
                    "long-breath",
                    "unbroken-trance",
                    "broken-balance",
                ]
                .contains(&r.id.as_str())
            }),
    );
    r
}
pub(crate) fn equipment_fixture(kind: &str) -> Json {
    let mut d = resolved("equipment");
    let (i, field) = match kind {
        "armor" => (0, "equipment_defenses"),
        "accessory" => (1, "equipment_states"),
        _ => panic!(),
    };
    let value = d["values"][i].clone();
    let mut binding = d[field][0].clone();
    binding["equipment"] = json!("r11_weapon");
    d["values"] = json!([value]);
    d.as_object_mut().unwrap().remove("equipment_defenses");
    d.as_object_mut().unwrap().remove("equipment_states");
    d[field] = json!([binding]);
    d
}

#[test]
fn canonical_channel_sources_preserve_unknowns_and_exact_existing_native_programs() {
    for (kind, count) in [("fragment", 11), ("equipment", 3)] {
        let raw = parse_yaml(original(kind)).unwrap();
        let d = resolved(kind);
        let mut pending = vec![];
        holes(&raw, "", &mut pending);
        assert_eq!(pending.len(), count);
        for (path, marker) in pending {
            let mut one = d.clone();
            *one.pointer_mut(&path).unwrap() = marker;
            let e = compile_json(&one.to_string()).unwrap_err();
            assert_eq!(e.code, DiagnosticCode::Unresolved, "{kind}/{path}: {e:?}");
            assert!(!e.source.definition.is_empty());
        }
        let a = compile_json(&d.to_string()).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        if kind == "fragment" {
            let mut actual = serde_json::to_value(&a.load_native().unwrap().skills[0]).unwrap();
            let old = compile_yaml(include_str!(
                "../../../tests/fixtures/skill_composition/control_vulnerability.skill.yaml"
            ))
            .unwrap();
            let mut expected = serde_json::to_value(&old.load_native().unwrap().skills[0]).unwrap();
            assert_eq!(actual["id"], SKILL);
            for v in [&mut actual, &mut expected] {
                v.as_object_mut().unwrap().remove("id");
                v.as_object_mut().unwrap().remove("name");
            }
            assert_eq!(actual, expected, "only canonical identity/name changes");
            assert!(
                a.equipment_attacks().is_empty()
                    && a.equipment_defenses().is_empty()
                    && a.equipment_states().is_empty()
            );
            assert_eq!(raw["values"][1]["value"]["Count"], 25);
            assert_eq!(raw["values"][6]["value"]["Milliseconds"], 5000);
            assert_eq!(
                raw["values"][0]["value"]["Delivery"]["TileArea"]["area"]["ground_status"]
                    ["statuses"][2]["kind"]["DamageTaken"]["percent"],
                20
            );
        } else {
            let armor = compile_yaml(include_str!(
                "../../../tests/fixtures/skill_composition/channel_opening_guard.skill.yaml"
            ))
            .unwrap();
            let accessory = compile_yaml(include_str!(
                "../../../tests/fixtures/skill_composition/equipment_stun_end.skill.yaml"
            ))
            .unwrap();
            assert_eq!(
                serde_json::to_value(&a.equipment_defenses()[0].program).unwrap(),
                serde_json::to_value(&armor.equipment_defenses()[0].program).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&a.equipment_states()[0].program).unwrap(),
                serde_json::to_value(&accessory.equipment_states()[0].program).unwrap()
            );
            assert!(a.equipment_attacks().is_empty() && a.load_native().unwrap().skills.is_empty());
        }
    }
}

#[test]
fn canonical_channel_catalog_preserves_staff_tile_attack_ids_and_shared_relics() {
    let gear = equipment();
    let old: crate::game::data::equipment_data::EquipmentDatabase = ron::de::from_str(
        include_str!("../../../../game_resources/data/equipments/base.ron"),
    )
    .unwrap();
    for item in &gear {
        let previous = old.get_by_id(&item.id).unwrap();
        assert_eq!(
            (item.uuid, item.equipment_type),
            (previous.uuid, previous.equipment_type)
        );
        assert_eq!(item.content_family.as_deref(), Some(FAMILY));
    }
    assert_eq!(
        serde_json::to_value(&gear[0].weapon_profile).unwrap(),
        serde_json::to_value(
            &old.get_by_id("placeholder_burrowing_heaven_weapon")
                .unwrap()
                .weapon_profile
        )
        .unwrap()
    );
    assert!(gear[0].triggered_effects.is_empty() && gear[0].ability_activations.is_empty());
    let f = fragment();
    assert_eq!(f.id.as_str(), FRAGMENT);
    assert_eq!(f.uuid.to_string(), "730e8400-e29b-41d4-a716-446655440025");
    assert_eq!(
        f.compatibility.allowed_weapon_archetypes,
        vec![crate::game::data::equipment_data::WeaponArchetype::Staff]
    );
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = f.effect
    else {
        panic!()
    };
    assert_eq!(forms["base"].skill_id.as_str(), SKILL);
    assert_eq!(forms["base"].resonance_max, 100);
    let relics = relics();
    relics.validate_contract().unwrap();
    assert_eq!(relics.relics.len(), 5);
    assert_eq!(
        relics
            .relics
            .iter()
            .map(|r| &r.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        5
    );
    for (raw, count) in [(FRAGMENT_CATALOG, 3), (EQUIPMENT_CATALOG, 23)] {
        let mut h = vec![];
        holes(&serde_json::from_str(raw).unwrap(), "", &mut h);
        assert_eq!(h.len(), count);
    }
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
}
