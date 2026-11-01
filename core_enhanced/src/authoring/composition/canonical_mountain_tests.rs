use super::canonical_area_tests::{holes, replace_holes};
use super::*;
use serde_json::{json, Value as Json};

pub(crate) const FRAGMENT: &str = "fragment_mountain_smiling_bodies_trace";
pub(crate) const SKILL: &str = "fragment_mountain_smiling_bodies_chorus";
pub(crate) const FAMILY: &str = "t-01-75_mountain";
pub(crate) fn original(kind: &str) -> &'static str {
    match kind {
        "fragment" => include_str!(
            "../../../../game_resources/authoring/mountain-of-smiling-bodies/fragment.skill.yaml"
        ),
        "equipment" => include_str!(
            "../../../../game_resources/authoring/mountain-of-smiling-bodies/equipment.skill.yaml"
        ),
        _ => panic!("unknown source"),
    }
}
pub(crate) fn inputs() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_mountain.values.json"
    ))
    .unwrap()["values"]
        .clone()
}
pub(crate) fn resolved(kind: &str) -> Json {
    let mut d = parse_yaml(original(kind)).unwrap();
    replace_holes(&mut d, &inputs());
    d
}
pub(crate) const FRAGMENT_CATALOG: &str = include_str!(
    "../../../../game_resources/authoring/mountain-of-smiling-bodies/fragment.catalog.json"
);
pub(crate) const EQUIPMENT_CATALOG: &str = include_str!(
    "../../../../game_resources/authoring/mountain-of-smiling-bodies/equipment.catalog.json"
);
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
        "../../../../game_resources/authoring/relics/taunt-modifiers.catalog.json"
    ))
    .unwrap();
    let values: Json = serde_json::from_str(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_area_strikes.values.json"
    ))
    .unwrap();
    replace_holes(&mut d, &values["relics"]);
    let mut r: crate::game::data::relic_data::RelicDatabase = serde_json::from_value(d).unwrap();
    r.relics.extend(
        super::canonical_knight_tests::relics()
            .relics
            .into_iter()
            .filter(|r| {
                ["widened-horizon", "long-breath", "double-harvest"].contains(&r.id.as_str())
            }),
    );
    r
}

#[test]
fn canonical_mountain_sources_preserve_unknowns_and_existing_shared_programs() {
    for kind in ["fragment", "equipment"] {
        let raw = parse_yaml(original(kind)).unwrap();
        let d = resolved(kind);
        let mut pending = vec![];
        holes(&raw, "", &mut pending);
        assert!(!pending.is_empty());
        for (path, marker) in pending {
            let mut one = d.clone();
            *one.pointer_mut(&path).unwrap() = marker;
            let e = compile_json(&one.to_string()).unwrap_err();
            assert_eq!(e.code, DiagnosticCode::Unresolved, "{kind}/{path}: {e:?}");
            assert!(!e.source.definition.is_empty());
        }
        let a = compile_json(&d.to_string()).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        let old = compile_yaml(include_str!(
            "../../../tests/fixtures/skill_composition/mountain.skill.yaml"
        ))
        .unwrap();
        if kind == "fragment" {
            let mut actual = serde_json::to_value(&a.load_native().unwrap().skills[0]).unwrap();
            let mut expected = serde_json::to_value(&old.load_native().unwrap().skills[0]).unwrap();
            assert_eq!(actual["id"], SKILL);
            for v in [&mut actual, &mut expected] {
                v.as_object_mut().unwrap().remove("id");
                v.as_object_mut().unwrap().remove("name");
            }
            assert_eq!(actual, expected);
            assert!(a.equipment_attacks().is_empty());
            // The one-tile cap and long-breath's 25% are rules, not unknown balance.
            assert_eq!(
                raw["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["Control"]
                    ["kind"]["ApproachTaunt"]["profile"]["attack_range_units"],
                1_000_000
            );
            assert_eq!(raw["values"][4]["value"]["Count"], 25);
            assert!(!original(kind).contains("broken-balance"));
        } else {
            assert_eq!(
                serde_json::to_value(&a.equipment_attacks()[0].program).unwrap(),
                serde_json::to_value(&old.equipment_attacks()[0].program).unwrap()
            );
            for (fixture, actual) in [(
                include_str!("../../../tests/fixtures/skill_composition/primary_hp.skill.yaml"),
                serde_json::to_value(&a.equipment_attacks()[1].program).unwrap(),
            )] {
                let e = compile_yaml(fixture).unwrap();
                assert_eq!(
                    actual,
                    serde_json::to_value(&e.equipment_attacks()[0].program).unwrap()
                );
            }
            let armor = compile_yaml(include_str!(
                "../../../tests/fixtures/skill_composition/nearby_defense.skill.yaml"
            ))
            .unwrap();
            assert_eq!(
                serde_json::to_value(&a.equipment_defenses()[0].program).unwrap(),
                serde_json::to_value(&armor.equipment_defenses()[0].program).unwrap()
            );
            assert_eq!(
                d["resources"],
                json!([{"id":"weapon_mass","owner":"EquipmentCopy","lifetime":"Run","initial":0,"cap":4294967295u64}])
            );
            assert!(a.load_native().unwrap().skills.is_empty());
        }
    }
}

#[test]
fn canonical_mountain_catalog_preserves_reward_identities_and_current_shield_roles() {
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
    use crate::game::data::equipment_data::WeaponArchetype;
    let profile = gear[0].weapon_profile.as_ref().unwrap();
    assert_eq!(profile.weapon_archetype, WeaponArchetype::Shield);
    assert!(matches!(
        profile.action,
        crate::game::data::abnormality_data::BasicAttackAction::DamageArea {
            secondary_percent: 40,
            ..
        }
    ));
    assert!(
        profile.kill_growth.is_none(),
        "authored program is the only growth owner"
    );
    let f = fragment();
    assert_eq!(f.id.as_str(), FRAGMENT);
    assert_eq!(f.uuid.to_string(), "730e8400-e29b-41d4-a716-446655440011");
    assert_eq!(
        f.compatibility.allowed_weapon_archetypes,
        vec![WeaponArchetype::Shield]
    );
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = f.effect
    else {
        panic!()
    };
    assert_eq!(forms["base"].skill_id.as_str(), SKILL);
    let r = relics();
    r.validate_contract().unwrap();
    assert_eq!(r.relics.len(), 5);
    assert!(!r.relics.iter().any(|r| r.id.as_str() == "broken-balance"));
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
