use super::*;
use crate::game::data::skill_data::SkillDatabase;

const RON: &str = include_str!("../../../../tests/fixtures/content_authoring/literal_effects.ron");
const SOURCE: &str = "fixture/literal_effects.ron";

fn site(skill: &str, action: &str, effect: &str) -> EffectSite {
    EffectSite {
        action: ActionSite {
            definition_id: skill.into(),
            action_id: action.into(),
        },
        effect_id: effect.into(),
    }
}

#[test]
fn source_addresses_and_queries_distinguish_effect_and_definition_identity() {
    let database = SkillDatabase::from_ron(SOURCE, RON).unwrap();
    let predicate = |tags: &[&str]| EffectPredicate {
        all_tags: tags.iter().map(|tag| (*tag).into()).collect(),
        op: Some(LiteralEffectOp::Damage),
        property: Some(EffectProperty::Amount),
    };
    assert!(database.matches_effects(
        &"authoring_alpha".into(),
        &EffectSelection::SameEffect(predicate(&["impact", "red"]))
    ));
    assert!(!database.matches_effects(
        &"authoring_alpha".into(),
        &EffectSelection::SameEffect(predicate(&["red", "blue"]))
    ));
    assert!(database.matches_effects(
        &"authoring_alpha".into(),
        &EffectSelection::EachExists(vec![predicate(&["red"]), predicate(&["blue"])])
    ));
    assert!(!database.matches_effects(
        &"absent_form".into(),
        &EffectSelection::SameEffect(predicate(&["impact"]))
    ));
    let alpha = database
        .effect(&site("authoring_alpha", "recover", "heal"))
        .unwrap()
        .source()
        .unwrap();
    let beta = database
        .effect(&site("authoring_beta", "recover", "heal"))
        .unwrap()
        .source()
        .unwrap();
    assert_eq!(alpha.origin, EffectOrigin::Original);
    assert_eq!(
        beta.origin,
        EffectOrigin::Added {
            source_definition_id: "authoring_alpha".into()
        }
    );
    assert_eq!(beta.origin_definition_id, alpha.origin_definition_id);
    assert_ne!(beta.site, alpha.site);
    assert_eq!(beta.source_file, SOURCE);
    assert_eq!(beta.field_path, "skills[1].steps[1].effects[0]");
    let recovery = EffectPredicate {
        all_tags: vec!["recovery".into()],
        ..Default::default()
    };
    assert!(database.matches_effects(
        &"authoring_beta".into(),
        &EffectSelection::SameEffect(recovery)
    ));
    assert!(database.matches_effects(
        &"authoring_alpha".into(),
        &EffectSelection::SameEffect(EffectPredicate::default())
    ));
    assert!(database.matches_effects(
        &"authoring_alpha".into(),
        &EffectSelection::EachExists(vec![])
    ));
    assert!(!database.matches_effects(&"missing".into(), &EffectSelection::EachExists(vec![])));
    let empty = AuthoredSkillEffect::new("empty_tags", SkillEffectDef::Heal { amount: 0 });
    assert!(EffectPredicate::default().matches(&empty));
    assert!(!EffectPredicate {
        all_tags: vec!["recovery".into()],
        ..Default::default()
    }
    .matches(&empty));
}

#[test]
fn scalar_edit_preserves_other_effects_and_rejects_shared_carrier_without_mutation() {
    let mut database = SkillDatabase::from_ron(SOURCE, RON).unwrap();
    let damage = site("authoring_alpha", "strike", "damage");
    let heal = site("authoring_alpha", "recover", "heal");
    let before = serde_json::to_value(&database).unwrap();
    let error = database
        .set_effect_amount(&damage, EffectProperty::Target, 3)
        .unwrap_err();
    assert_eq!(error.affected_effects.len(), 2);
    assert_eq!(error.site, damage);
    assert_eq!(error.source.unwrap().site, damage);
    assert_eq!(serde_json::to_value(&database).unwrap(), before);
    database
        .set_effect_amount(&damage, EffectProperty::Amount, 0)
        .unwrap();
    assert_eq!(database.effect(&damage).unwrap().literal_amount(), Some(0));
    assert_eq!(database.effect(&heal).unwrap().literal_amount(), Some(9));
    database
        .set_effect_amount(&damage, EffectProperty::Amount, i32::MAX)
        .unwrap();
    assert!(database
        .set_effect_amount(&damage, EffectProperty::Amount, -1)
        .is_err());
    assert_eq!(
        database.effect(&damage).unwrap().literal_amount(),
        Some(i32::MAX)
    );
    database
        .set_effect_amount(&heal, EffectProperty::Amount, i32::MIN)
        .unwrap();
    assert_eq!(
        database.effect(&heal).unwrap().literal_amount(),
        Some(i32::MIN)
    );
    for missing in [
        site("missing", "strike", "damage"),
        site("authoring_alpha", "missing", "damage"),
        site("authoring_alpha", "strike", "missing"),
    ] {
        let before = serde_json::to_value(&database).unwrap();
        assert!(database
            .set_effect_amount(&missing, EffectProperty::Amount, 1)
            .is_err());
        assert_eq!(serde_json::to_value(&database).unwrap(), before);
    }
}

#[test]
fn invalid_authoring_has_source_object_action_effect_and_field() {
    for (from, to, expected_field) in [
        (
            "Damage(amount: 17, damage_type: True)",
            "Damage(amount: -1, damage_type: True)",
            "op.Damage.amount",
        ),
        ("id: \"other_damage\"", "id: \"damage\"", ".id"),
        ("id: \"damage\"", "id: \"\"", ".id"),
        (
            "Damage(amount: 17, damage_type: True)",
            "ExecuteFile(path: \"inert-never-executed\")",
            ".op",
        ),
        (
            "Damage(amount: 17, damage_type: True)",
            "Damage(amount: 17, damage_type: True, target: SelfUnit)",
            ".op",
        ),
        (
            "source_definition_id: \"authoring_alpha\"",
            "source_definition_id: \"missing\"",
            ".origin",
        ),
    ] {
        let error = SkillDatabase::from_ron(SOURCE, &RON.replacen(from, to, 1)).unwrap_err();
        assert_eq!(error.source_file, SOURCE);
        assert!(error.object_id.is_some(), "{error:?}");
        assert!(error.action_id.is_some(), "{error:?}");
        assert!(error.effect_id.is_some(), "{error:?}");
        assert!(
            error.field_path.as_ref().unwrap().contains(expected_field),
            "{error:?}"
        );
        assert!(!error.cause.is_empty());
    }
    let legacy = RON.replacen(
        "(id: \"damage\", tags: [\"impact\", \"red\"], op: Damage(amount: 17, damage_type: True))",
        "Damage(amount: 17, damage_type: True)",
        1,
    );
    assert!(SkillDatabase::from_ron(SOURCE, &legacy).is_err());
    let overflow = RON.replacen("amount: 17", "amount: 2147483648", 1);
    let error = SkillDatabase::from_ron(SOURCE, &overflow).unwrap_err();
    assert_eq!(error.object_id.as_deref(), Some("authoring_alpha"));
    assert_eq!(error.action_id.as_deref(), Some("strike"));
    assert_eq!(error.effect_id.as_deref(), Some("damage"));
    assert!(error.field_path.unwrap().contains("amount"));
    let missing_id = RON.replacen("id: \"damage\", ", "", 1);
    let error = SkillDatabase::from_ron(SOURCE, &missing_id).unwrap_err();
    assert_eq!(error.object_id.as_deref(), Some("authoring_alpha"));
    assert_eq!(error.action_id.as_deref(), Some("strike"));
    assert!(error.effect_id.is_none());
    assert!(error.cause.contains("id"));
}

#[test]
fn reorder_and_failed_retry_preserve_stable_addresses_and_supported_operations() {
    let database = SkillDatabase::from_ron(SOURCE, RON).unwrap();
    let mut reordered = database.skills.clone();
    reordered.reverse();
    for skill in &mut reordered {
        skill.steps[0].effects.reverse();
    }
    let reordered = SkillDatabase::new(reordered);
    for definition in ["authoring_alpha", "authoring_beta"] {
        for effect in ["damage", "other_damage"] {
            let address = site(definition, "strike", effect);
            assert_eq!(
                database.effect(&address).unwrap().literal_amount(),
                reordered.effect(&address).unwrap().literal_amount()
            );
            assert_eq!(
                reordered.effect(&address).unwrap().source().unwrap().site,
                address
            );
        }
    }
    assert!(SkillDatabase::from_ron(SOURCE, &RON.replace("amount: 17", "amount: -1")).is_err());
    let retry = SkillDatabase::from_ron(SOURCE, RON).unwrap();
    assert_eq!(
        serde_json::to_value(&database).unwrap(),
        serde_json::to_value(&retry).unwrap()
    );
    for support in EFFECT_SUPPORT {
        let effect = database
            .skills
            .iter()
            .flat_map(|skill| &skill.steps)
            .flat_map(|step| &step.effects)
            .find(|effect| LiteralEffectOp::of(&effect.op) == Some(support.op))
            .unwrap();
        assert_eq!(
            effect_support(&effect.op, &DeliveryDef::Instant)
                .unwrap()
                .op,
            support.op
        );
    }
}
