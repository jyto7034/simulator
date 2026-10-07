use game_core::game::ability::authoring::{ActionSite, EffectOrigin, EffectPredicate, EffectProperty, EffectSelection, EffectSite, LiteralEffectOp, EFFECT_SUPPORT};
use game_core::game::ability::{DeliveryDef, SkillEffectDef};
use game_core::game::data::{content_pack::{load, ContentSelection, InputLimits}, ValidatedGameData};
use std::path::PathBuf;

fn root(name: &str) -> PathBuf { PathBuf::from("/tmp/r01-successor-probe").join(name) }
fn load_case(name: &str) -> Result<ValidatedGameData, game_core::game::data::live_ron::LiveRonError> {
    load(&root(name), &ContentSelection { packs: vec!["author".into(), "official".into()], winners: vec![] }, &InputLimits::default())
}
fn site(def: &str, action: &str, effect: &str) -> EffectSite {
    EffectSite { action: ActionSite { definition_id: def.into(), action_id: action.into() }, effect_id: effect.into() }
}

#[test]
fn external_source_query_and_scalar_support_are_the_same_public_contract() {
    let token = load_case("support").unwrap();
    let beta = site("authoring_beta", "recover", "heal");
    let source = token.skill_data.effect(&beta).unwrap().source().unwrap().clone();
    assert_eq!(source.source_file, "author/effects.ron");
    assert_eq!(source.field_path, "skills[1].steps[1].effects[0]");
    assert_eq!(source.origin, EffectOrigin::Added { source_definition_id: "authoring_alpha".into() });
    let pred = |tags: &[&str]| EffectPredicate { all_tags: tags.iter().map(|s|s.to_string()).collect(), op: Some(LiteralEffectOp::Damage), property: Some(EffectProperty::Amount) };
    assert!(!token.skill_data.matches_effects(&"authoring_alpha".into(), &EffectSelection::SameEffect(pred(&["red", "blue"]))));
    assert!(token.skill_data.matches_effects(&"authoring_alpha".into(), &EffectSelection::EachExists(vec![pred(&["red"]), pred(&["blue"])])));
    assert!(!token.skill_data.matches_effects(&"unwritten_form".into(), &EffectSelection::SameEffect(pred(&["red"]))));
    let mut candidate = (*token.skill_data).clone();
    let damage = site("authoring_alpha", "strike", "damage");
    let original = candidate.effect(&damage).unwrap().literal_amount();
    for property in [EffectProperty::Target, EffectProperty::Delivery, EffectProperty::DamageType] {
        let error = candidate.set_effect_amount(&damage, property, 999).unwrap_err();
        assert_eq!(error.site, damage); assert_eq!(error.affected_effects.len(), 2);
        assert_eq!(candidate.effect(&damage).unwrap().literal_amount(), original);
    }
    candidate.set_effect_amount(&damage, EffectProperty::Amount, i32::MAX).unwrap();
    assert!(candidate.set_effect_amount(&damage, EffectProperty::Amount, -1).is_err());
    assert_eq!(candidate.effect(&damage).unwrap().literal_amount(), Some(i32::MAX));
    assert_eq!(candidate.effect(&beta).unwrap().literal_amount(), Some(9));
    assert_eq!(token.skill_data.effect(&damage).unwrap().literal_amount(), original);
    assert_eq!(EFFECT_SUPPORT.len(), 2);
    let step = &mut candidate.skills.iter_mut().find(|s|s.id.as_ref()=="authoring_alpha").unwrap().steps[0];
    step.effects[0].op = SkillEffectDef::ModifyTacticalResource { amount: 1 };
    step.effects[0].tags.push("Damage".into());
    assert!(candidate.set_effect_amount(&damage, EffectProperty::Amount, 100).is_err());
    assert!(matches!(candidate.skills.iter().find(|s|s.id.as_ref()=="authoring_alpha").unwrap().steps[0].delivery, DeliveryDef::Instant));
    drop(token); assert_eq!(source.site, beta); assert_eq!(source.origin_definition_id.as_ref(), "authoring_alpha");
}

#[test]
fn external_semantic_rejection_has_all_addresses_and_does_not_replace_retained_token() {
    let token = load_case("semantic").unwrap(); let identity = token.identity().clone();
    let path = root("semantic").join("author/effects.ron"); let original = std::fs::read_to_string(&path).unwrap();
    let bad = original.replacen("Damage(amount: 17, damage_type: True)", "Damage(amount: -1, damage_type: True)", 1);
    assert_ne!(bad, original); std::fs::write(&path, bad).unwrap();
    for _ in 0..2 {
        let error = load_case("semantic").err().expect("negative damage cannot publish");
        assert_eq!(error.source_file, "author/effects.ron");
        assert_eq!(error.object_id.as_deref(), Some("authoring_alpha"));
        assert_eq!(error.action_id.as_deref(), Some("strike")); assert_eq!(error.effect_id.as_deref(), Some("damage"));
        assert_eq!(error.field_path.as_deref(), Some("skills[0].steps[0].effects[0].op.Damage.amount"));
        assert_eq!(token.identity(), &identity);
    }
    std::fs::write(&path, original).unwrap(); assert_eq!(load_case("semantic").unwrap().identity(), &identity);
}

#[test]
fn external_authored_source_injection_and_opcode_are_strictly_rejected() {
    let path = root("parse").join("author/effects.ron"); let original = std::fs::read_to_string(&path).unwrap();
    for bad in [original.replacen("id: \"damage\",", "id: \"damage\", source: None,", 1), original.replacen("Damage(amount: 17, damage_type: True)", "ExecuteFile(path: \"never-executed\")", 1)] {
        assert_ne!(bad, original); std::fs::write(&path, bad).unwrap();
        let error = load_case("parse").err().expect("invalid authoring cannot publish");
        assert_eq!(error.source_file, "author/effects.ron");
        assert_eq!(error.object_id.as_deref(), Some("authoring_alpha"));
        assert_eq!(error.action_id.as_deref(), Some("strike")); assert_eq!(error.effect_id.as_deref(), Some("damage"));
        assert!(error.field_path.is_some());
    }
    std::fs::write(&path, original).unwrap(); assert!(load_case("parse").is_ok());
}
