use super::canonical_area_tests::{holes, replace_holes};
use super::*;
use serde_json::{json, Value as Json};

pub(crate) const FRAGMENT: &str = "fragment_king_of_greed_trace";
pub(crate) const SKILL: &str = "fragment_king_of_greed_horizon";
pub(crate) const FAMILY: &str = "o-01-64_king_of_greed";
pub(crate) const RELICS: [&str; 7] = [
    "long-breath",
    "widened-horizon",
    "double-harvest",
    "unbroken-trance",
    "deep-wedge",
    "king-of-greed-stunning-treasures",
    "king-of-greed-endless-treasury",
];
pub(crate) fn original(kind: &str) -> &'static str {
    match kind {
        "fragment" => {
            include_str!("../../../../game_resources/authoring/king-of-greed/fragment.skill.yaml")
        }
        "equipment" => {
            include_str!("../../../../game_resources/authoring/king-of-greed/equipment.skill.yaml")
        }
        _ => panic!("unknown source"),
    }
}
pub(crate) fn inputs() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_greed.values.json"
    ))
    .unwrap()["values"]
        .clone()
}
pub(crate) fn resolved(kind: &str) -> Json {
    let mut d = parse_yaml(original(kind)).unwrap();
    replace_holes(&mut d, &inputs());
    d
}
pub(crate) fn program(d: &mut Json) -> &mut Json {
    &mut d["values"][0]["value"]["Effects"][0]["op"]["PreparedVolley"]["program"]
}
pub(crate) fn fixture() -> crate::authoring::node_graph::NativeGraphContent {
    let mut d = resolved("fragment");
    d["skills"][0]["id"] = json!("r11_aura");
    crate::authoring::node_graph::NativeGraphContent {
        skills: compile_json(&d.to_string()).unwrap().load_native().unwrap(),
        equipment: vec![],
        mark_weapons: vec![],
    }
}
pub(crate) const FRAGMENT_CATALOG: &str =
    include_str!("../../../../game_resources/authoring/king-of-greed/fragment.catalog.json");
pub(crate) const EQUIPMENT_CATALOG: &str =
    include_str!("../../../../game_resources/authoring/king-of-greed/equipment.catalog.json");
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
        "../../../../game_resources/authoring/relics/volley-modifiers.catalog.json"
    ))
    .unwrap();
    let v: Json = serde_json::from_str(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_area_strikes.values.json"
    ))
    .unwrap();
    replace_holes(&mut d, &v["relics"]);
    let mut r: crate::game::data::relic_data::RelicDatabase = serde_json::from_value(d).unwrap();
    r.relics.extend(
        super::canonical_knight_tests::relics()
            .relics
            .into_iter()
            .filter(|r| RELICS.contains(&r.id.as_str())),
    );
    r
}

#[test]
fn canonical_greed_sources_preserve_unknowns_and_shared_programs() {
    for kind in ["fragment", "equipment"] {
        let raw = parse_yaml(original(kind)).unwrap();
        let d = resolved(kind);
        let mut pending = vec![];
        holes(&raw, "", &mut pending);
        assert!(!pending.is_empty());
        for (path, marker) in pending {
            let mut one = d.clone();
            *one.pointer_mut(&path).unwrap() = marker;
            let err = compile_json(&one.to_string()).unwrap_err();
            assert_eq!(
                err.code,
                DiagnosticCode::Unresolved,
                "{kind}/{path}: {err:?}"
            );
            assert!(!err.source.definition.is_empty());
        }
        let a = compile_json(&d.to_string()).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        if kind == "fragment" {
            let native = crate::authoring::node_graph::compile(
                &crate::authoring::node_graph::tests::prepared_volley_fixture().to_string(),
                SKILL.into(),
            )
            .unwrap()
            .load_native()
            .unwrap();
            let mut old = serde_json::to_value(&native.skills[0].steps[0].effects).unwrap();
            // The technical graph fixture used @ for both areas. The current
            // original's explicit test completion exercises the agreed 1->3x3.
            for m in old[0]["op"]["PreparedVolley"]["program"]["modifiers"]
                .as_array_mut()
                .unwrap()
            {
                if m["relic_definition"] == "widened-horizon" {
                    m["change"]["Area"]["pattern"] =
                        json!({"include_anchor_tile":true,"rows":["XXX","X@X","XXX"]});
                }
            }
            let loaded = a.load_native().unwrap();
            assert_eq!(
                serde_json::to_value(&loaded.skills[0].steps[0].effects).unwrap(),
                old
            );
            let mut raw = raw;
            let p = program(&mut raw);
            assert_eq!(p["ledger"]["grant"], 10);
            assert_eq!(p["ledger"]["credit_limit"], 20);
            assert_eq!(p["recovery"], 1);
            assert_eq!(p["projectile"]["damage_type"], "Physical");
            let continuous = p["modifiers"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["relic_definition"] == "king-of-greed-endless-treasury")
                .unwrap();
            assert_eq!(
                continuous["change"]["Continuous"]["percent"],
                json!({"numerator":5,"denominator":1})
            );
            assert_eq!(continuous["change"]["Continuous"]["cost_interval_ms"], 1000);
            assert_eq!(p["modifiers"].as_array().unwrap().len(), 7);
            assert_eq!(loaded.skills[0].id.as_str(), SKILL);
            assert!(!original(kind).contains("broken-balance"));
        } else {
            let armor = compile_yaml(include_str!(
                "../../../tests/fixtures/skill_composition/ready_wait.skill.yaml"
            ))
            .unwrap();
            let accessory = compile_yaml(include_str!(
                "../../../tests/fixtures/skill_composition/area_batch.skill.yaml"
            ))
            .unwrap();
            assert_eq!(
                serde_json::to_value(&a.equipment_defenses()[0].program).unwrap(),
                serde_json::to_value(&armor.equipment_defenses()[0].program).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&a.equipment_attacks()[0].program).unwrap(),
                serde_json::to_value(&accessory.equipment_attacks()[0].program).unwrap()
            );
            assert!(a.load_native().unwrap().skills.is_empty());
            assert_eq!(
                raw["values"][0]["value"]["DefenseProgram"]["ReadyWaitDamage"]["duration_ms"],
                15000
            );
        }
    }
}

#[test]
fn canonical_greed_catalog_keeps_reward_identities_and_bow_without_stock_owner() {
    use crate::game::data::equipment_data::{EquipmentDatabase, WeaponArchetype};
    let old: EquipmentDatabase = ron::de::from_str(include_str!(
        "../../../../game_resources/data/equipments/base.ron"
    ))
    .unwrap();
    let gear = equipment();
    for item in &gear {
        let previous = old.get_by_id(&item.id).unwrap();
        assert_eq!(
            (item.uuid, item.equipment_type),
            (previous.uuid, previous.equipment_type)
        );
        assert_eq!(item.content_family.as_deref(), Some(FAMILY));
        assert!(item.ability_activations.is_empty());
    }
    let profile = gear[0].weapon_profile.as_ref().unwrap();
    assert_eq!(profile.weapon_archetype, WeaponArchetype::Bow);
    assert_eq!(profile.block_capacity, 0);
    assert!(profile.air_capable);
    assert!(matches!(
        profile.action,
        crate::game::data::abnormality_data::BasicAttackAction::DamageTileAfterDelay {
            delay_ms: 400
        }
    ));
    assert!(profile.kill_growth.is_none());
    let f = fragment();
    assert_eq!(f.id.as_str(), FRAGMENT);
    assert_eq!(f.uuid.to_string(), "730e8400-e29b-41d4-a716-446655440009");
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = f.effect
    else {
        panic!()
    };
    assert_eq!(forms["base"].skill_id.as_str(), SKILL);
    let r = relics();
    r.validate_contract().unwrap();
    assert_eq!(r.relics.len(), 7);
    for id in RELICS {
        assert!(r.get_by_id(id).is_some(), "{id}");
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
