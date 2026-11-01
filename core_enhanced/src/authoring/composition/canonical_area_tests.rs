use super::*;
use serde_json::{json, Value as Json};

pub(crate) fn original(family: &str) -> &'static str {
    match family {
        "woodsman" => {
            include_str!("../../../../game_resources/authoring/woodsman/fragment.skill.yaml")
        }
        "clouded-monk" => {
            include_str!("../../../../game_resources/authoring/clouded-monk/fragment.skill.yaml")
        }
        _ => panic!("unknown test family"),
    }
}

pub(crate) fn catalog_original(family: &str) -> &'static str {
    match family {
        "woodsman" => {
            include_str!("../../../../game_resources/authoring/woodsman/fragment.catalog.json")
        }
        "clouded-monk" => {
            include_str!("../../../../game_resources/authoring/clouded-monk/fragment.catalog.json")
        }
        _ => panic!("unknown test family"),
    }
}

pub(crate) const RELICS_ORIGINAL: &str =
    include_str!("../../../../game_resources/authoring/relics/area-modifiers.catalog.json");

pub(crate) fn relics() -> crate::game::data::relic_data::RelicDatabase {
    let mut doc: Json = serde_json::from_str(RELICS_ORIGINAL).unwrap();
    let inputs: Json = serde_json::from_str(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_area_strikes.values.json"
    ))
    .unwrap();
    replace_holes(&mut doc, &inputs["relics"]);
    serde_json::from_value(doc).unwrap()
}

pub(crate) fn replace_holes(value: &mut Json, inputs: &Json) {
    if let Some(key) = value.get("Unresolved").and_then(Json::as_str) {
        assert_eq!(value.as_object().unwrap().len(), 1);
        *value = inputs
            .get(key)
            .unwrap_or_else(|| panic!("test input missing {key}"))
            .clone();
    } else {
        match value {
            Json::Object(map) => map.values_mut().for_each(|v| replace_holes(v, inputs)),
            Json::Array(values) => values.iter_mut().for_each(|v| replace_holes(v, inputs)),
            _ => (),
        }
    }
}

/// Acceptance data only. Product code never replaces unresolved values.
pub(crate) fn resolved(family: &str) -> Json {
    let mut doc = parse_yaml(original(family)).unwrap();
    let inputs: Json = serde_json::from_str(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_area_strikes.values.json"
    ))
    .unwrap();
    replace_holes(&mut doc, &inputs[family]);
    doc
}

pub(crate) fn catalog(
    family: &str,
) -> crate::game::data::skill_fragment_data::SkillFragmentMetadata {
    let mut doc: Json = serde_json::from_str(catalog_original(family)).unwrap();
    let inputs: Json = serde_json::from_str(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_area_strikes.values.json"
    ))
    .unwrap();
    replace_holes(&mut doc, &inputs[family]);
    serde_json::from_value(doc).unwrap()
}

pub(crate) fn holes(value: &Json, path: &str, out: &mut Vec<(String, Json)>) {
    if value.get("Unresolved").is_some() {
        out.push((path.to_owned(), value.clone()));
    } else {
        match value {
            Json::Object(map) => {
                for (key, value) in map {
                    holes(value, &format!("{path}/{key}"), out);
                }
            }
            Json::Array(values) => {
                for (i, value) in values.iter().enumerate() {
                    holes(value, &format!("{path}/{i}"), out);
                }
            }
            _ => (),
        }
    }
}

#[test]
fn canonical_area_sources_reject_each_unresolved_leaf_and_preserve_known_contracts() {
    for family in ["woodsman", "clouded-monk"] {
        let source = original(family);
        assert_eq!(
            compile_yaml(source).unwrap_err().code,
            DiagnosticCode::Unresolved
        );
        let original = parse_yaml(source).unwrap();
        let resolved = resolved(family);
        let mut pending = Vec::new();
        holes(&original, "", &mut pending);
        assert_eq!(pending.len(), if family == "woodsman" { 19 } else { 7 });
        for (path, hole) in pending {
            let mut one = resolved.clone();
            *one.pointer_mut(&path).unwrap() = hole;
            let diagnostic = compile_json(&one.to_string()).unwrap_err();
            assert_eq!(
                diagnostic.code,
                DiagnosticCode::Unresolved,
                "{family} {path}: {diagnostic:?}"
            );
            assert!(!diagnostic.source.definition.is_empty());
        }
        let compiled = compile_json(&resolved.to_string()).unwrap();
        let restored = Artifact::from_ron(&compiled.to_ron().unwrap()).unwrap();
        assert_eq!(restored.source_digest, compiled.source_digest);
        let native = restored.load_native().unwrap();
        let skill = &native.skills[0];
        assert_eq!(skill.steps.len(), if family == "woodsman" { 3 } else { 1 });
        assert!(compiled.equipment_attacks().is_empty());
        assert!(compiled.equipment_states().is_empty());
        let metadata = catalog(family);
        let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
            forms,
            ..
        } = metadata.effect
        else {
            panic!()
        };
        assert_eq!(forms["base"].skill_id, skill.id);
        assert_eq!(forms["base"].resonance_max, 5);
        assert!(serde_json::from_str::<
            crate::game::data::skill_fragment_data::SkillFragmentMetadata,
        >(catalog_original(family))
        .is_err());
        assert_eq!(parse_yaml(source).unwrap(), original);
        assert_eq!(original["skills"][0]["admission"]["requirement"], "Full");
        assert_eq!(original["skills"][0]["admission"]["cost"], "All");
        if family == "woodsman" {
            assert_eq!(metadata.compatibility.allowed_weapon_archetypes.len(), 1);
            let policy = skill.action_policy.as_ref().unwrap();
            assert_eq!(policy.modifiers.len(), 7); // one shared wedge, protection, three ranges, stun duration/vulnerability
            let crate::game::ability::action::ActionChange::Multihit {
                steps,
                increment_percent,
                cap_percent,
            } = &policy.modifiers[0].change
            else {
                panic!()
            };
            assert_eq!(steps.len(), 3);
            assert_eq!((*increment_percent, *cap_percent), (2, 20));
            assert!(skill.steps.iter().all(|s| matches!(
                s.delivery,
                crate::game::ability::DeliveryDef::TileArea { .. }
            )));
        } else {
            assert!(skill.action_policy.is_none());
            let crate::game::ability::DeliveryDef::TileArea { area, .. } = &skill.steps[0].delivery
            else {
                panic!()
            };
            assert_eq!(
                serde_json::to_value(area.anchor).unwrap(),
                json!("AcceptedCastTarget")
            );
            assert_eq!(area.on_hit_caster_heal_percent, Some(20));
            assert_eq!(area.first_tick_delay_ms, 200);
        }
    }
}

#[test]
fn canonical_area_provider_authored_relics_need_explicit_declaration_and_keep_legacy_wire() {
    use crate::game::data::relic_data::{RelicCorruptionVariant, RelicDamageEffect, RelicPenalty};
    assert!(
        serde_json::from_str::<crate::game::data::relic_data::RelicDatabase>(RELICS_ORIGINAL)
            .is_err()
    );
    let original = relics();
    original.validate_contract().unwrap();
    assert_eq!(original.relics.len(), 5);
    assert!(original
        .relics
        .iter()
        .all(|r| r.provider_authored && r.effects.is_empty() && r.rules.is_empty()));
    let roundtrip: crate::game::data::relic_data::RelicDatabase =
        ron::de::from_str(&ron::ser::to_string(&original).unwrap()).unwrap();
    assert_eq!(roundtrip.relics, original.relics);
    let mut missing = original.clone();
    missing.relics[0].provider_authored = false;
    assert!(missing
        .validate_contract()
        .unwrap_err()
        .contains("effects/rules"));
    missing.relics[0].effects.push(RelicDamageEffect {
        id: "existing-damage".into(),
        selector: Default::default(),
        flat: 1,
        percent: 0,
    });
    missing.validate_contract().unwrap();
    let legacy = serde_json::to_value(&missing.relics[0]).unwrap();
    assert!(
        legacy.get("provider_authored").is_none(),
        "false preserves legacy wire"
    );
    let decoded: crate::game::data::relic_data::RelicDefinition =
        serde_json::from_value(legacy).unwrap();
    assert!(!decoded.provider_authored);
    missing.relics[0].provider_authored = true;
    missing.relics[0].effects[0].percent = -1;
    assert!(
        missing.validate_contract().is_err(),
        "provider declaration does not waive numeric validation"
    );
    let mut invalid_variant = original;
    invalid_variant.relics[0].corruption = Some(RelicCorruptionVariant {
        effects: vec![],
        rules: vec![],
        penalties: vec![RelicPenalty::DefenseFlat { amount: -1 }],
    });
    assert!(
        invalid_variant
            .validate_contract()
            .unwrap_err()
            .contains("corruption.effects/rules"),
        "normal provider declaration does not invent corruption behavior"
    );
}
