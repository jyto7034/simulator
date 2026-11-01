use super::canonical_area_tests::{holes, replace_holes};
use super::*;
use serde_json::{json, Value as Json};

pub(crate) const FRAGMENT: &str = "fragment_melting_love_trace";
pub(crate) const SKILL: &str = "fragment_melting_love_spore_burst";
pub(crate) const FAMILY: &str = "d-03-109_melting_love";
pub(crate) fn original(kind: &str) -> &'static str {
    match kind {
        "fragment" => {
            include_str!("../../../../game_resources/authoring/melting-love/fragment.skill.yaml")
        }
        "equipment" => {
            include_str!("../../../../game_resources/authoring/melting-love/equipment.skill.yaml")
        }
        _ => panic!("unknown canonical source"),
    }
}
pub(crate) fn inputs() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_presence.values.json"
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
    include_str!("../../../../game_resources/authoring/melting-love/fragment.catalog.json");
pub(crate) const EQUIPMENT_CATALOG: &str =
    include_str!("../../../../game_resources/authoring/melting-love/equipment.catalog.json");
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
        "../../../../game_resources/authoring/relics/presence-modifiers.catalog.json"
    ))
    .unwrap();
    replace_holes(&mut d, &inputs());
    let mut result: crate::game::data::relic_data::RelicDatabase =
        serde_json::from_value(d).unwrap();
    result.relics.extend(
        super::canonical_funeral_tests::relics()
            .relics
            .into_iter()
            .filter(|r| {
                [
                    "widened-horizon",
                    "overlapping-explosions",
                    "explosion-vulnerability",
                ]
                .contains(&r.id.as_str())
            }),
    );
    result
}
pub(crate) fn program(d: &mut Json) -> &mut Json {
    &mut d["values"][0]["value"]["Effects"][0]["op"]["PresenceBurst"]["program"]
}
pub(crate) fn battle_content() -> crate::authoring::node_graph::NativeGraphContent {
    let mut d = resolved("fragment");
    d["skills"][0]["id"] = json!("r11_aura");
    crate::authoring::node_graph::NativeGraphContent {
        skills: compile_json(&d.to_string()).unwrap().load_native().unwrap(),
        equipment: vec![],
        mark_weapons: vec![],
    }
}
pub(crate) fn equipment_fixture(kind: &str) -> String {
    let mut d = resolved("equipment");
    let index = match kind {
        "armor" => 0,
        "accessory" => 1,
        _ => panic!(),
    };
    let value = d["values"][index].clone();
    let mut binding = d["equipment_states"][index].clone();
    binding["equipment"] = json!("r11_weapon");
    d["values"] = json!([value]);
    d["equipment_states"] = json!([binding]);
    d.to_string()
}
#[test]
fn canonical_presence_sources_keep_unknowns_and_existing_fragment_equipment_meaning() {
    for (kind, count) in [("fragment", 14), ("equipment", 4)] {
        let raw = parse_yaml(original(kind)).unwrap();
        let d = resolved(kind);
        let mut pending = vec![];
        holes(&raw, "", &mut pending);
        assert_eq!(pending.len(), count, "{kind}");
        for (path, marker) in pending {
            let mut one = d.clone();
            *one.pointer_mut(&path).unwrap() = marker;
            let error = compile_json(&one.to_string()).unwrap_err();
            assert_eq!(
                error.code,
                DiagnosticCode::Unresolved,
                "{kind}/{path}: {error:?}"
            );
            assert!(!error.source.definition.is_empty());
        }
        let a = compile_json(&d.to_string()).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        if kind == "fragment" {
            let loaded = a.load_native().unwrap();
            let old = crate::authoring::node_graph::compile(
                &crate::authoring::node_graph::tests::presence_fixture().to_string(),
                "legacy".into(),
            )
            .unwrap()
            .load_native()
            .unwrap();
            let crate::game::ability::SkillEffectDef::PresenceBurst { program: actual } =
                &loaded.skills[0].steps[0].effects[0].op
            else {
                panic!()
            };
            let crate::game::ability::SkillEffectDef::PresenceBurst { program: expected } =
                &old.skills[0].steps[0].effects[0].op
            else {
                panic!()
            };
            let mut expected = expected.as_ref().clone();
            expected.marks.definition = "melting-love-spores".into();
            for modifier in &mut expected.modifiers {
                modifier.id = format!("spore-{}", modifier.relic_definition);
            }
            assert_eq!(actual.as_ref(), &expected);
            assert_eq!(loaded.skills[0].id.as_str(), SKILL);
            assert_eq!(
                loaded.skills[0].resonance_admission,
                old.skills[0].resonance_admission
            );
            assert_eq!(
                loaded.skills[0].use_lifetime,
                crate::game::ability::SkillUseLifetime::Action
            );
            assert!(a.equipment_attacks().is_empty() && a.equipment_states().is_empty());
        } else {
            assert!(a.load_native().unwrap().skills.is_empty());
            for (i, source) in [
                include_str!(
                    "../../../tests/fixtures/skill_composition/equipment_dot_recovery.skill.yaml"
                ),
                include_str!(
                    "../../../tests/fixtures/skill_composition/equipment_dot_kill.skill.yaml"
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let previous = compile_yaml(source).unwrap();
                assert_eq!(
                    serde_json::to_value(&a.equipment_states()[i].program).unwrap(),
                    serde_json::to_value(&previous.equipment_states()[0].program).unwrap()
                );
            }
            assert!(
                a.equipment_attacks().is_empty(),
                "plain Shield has no special program"
            );
        }
    }
}
#[test]
fn canonical_presence_catalog_retains_real_ids_plain_shield_and_six_relics() {
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
    let gun = gear[0].weapon_profile.as_ref().unwrap();
    let old_gun = old
        .get_by_id("placeholder_melting_love_weapon")
        .unwrap()
        .weapon_profile
        .as_ref()
        .unwrap();
    assert_eq!(
        serde_json::to_value(gun).unwrap(),
        serde_json::to_value(old_gun).unwrap()
    );
    assert!(gear[0].triggered_effects.is_empty() && gear[0].ability_activations.is_empty());
    let f = fragment();
    assert_eq!(f.id.as_str(), FRAGMENT);
    assert_eq!(f.uuid.to_string(), "730e8400-e29b-41d4-a716-446655440015");
    assert!(
        matches!(&f.origin,Some(crate::game::data::skill_fragment_data::SkillFragmentOrigin::Abnormality{abnormality_id}) if abnormality_id==FAMILY)
    );
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = f.effect
    else {
        panic!()
    };
    assert_eq!(forms.len(), 1);
    assert_eq!(forms["base"].skill_id.as_str(), SKILL);
    let relics = relics();
    assert_eq!(relics.relics.len(), 6);
    assert_eq!(
        relics
            .relics
            .iter()
            .map(|r| &r.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        6
    );
    for raw in [FRAGMENT_CATALOG, EQUIPMENT_CATALOG] {
        let mut h = vec![];
        holes(&serde_json::from_str(raw).unwrap(), "", &mut h);
        assert_eq!(h.len(), if raw == FRAGMENT_CATALOG { 4 } else { 18 });
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
