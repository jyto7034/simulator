use game_core::game::{ability::{authoring::{ActionSite, EffectProperty, EffectSite, effect_support}, DeliveryDef, SkillEffectDef}, data::{skill_data::SkillDatabase, ValidatedGameData}};
const RON: &str = include_str!("/mnt/f/work/simulator/core_enhanced/tests/fixtures/content_authoring/literal_effects.ron");
fn address() -> EffectSite { EffectSite { action: ActionSite { definition_id: "authoring_alpha".into(), action_id: "strike".into() }, effect_id: "damage".into() } }
#[test]
fn compiler_source_cannot_be_injected_by_authoring() {
    let input = RON.replacen("id: \"damage\",", "id: \"damage\", source: None,", 1);
    let error = SkillDatabase::from_ron("audit/probe.ron", &input).unwrap_err();
    assert_eq!(error.object_id.as_deref(), Some("authoring_alpha"));
    assert_eq!(error.action_id.as_deref(), Some("strike"));
    assert_eq!(error.effect_id.as_deref(), Some("damage"));
    assert!(error.cause.contains("unknown field"));
}
#[test]
fn unsupported_existing_operation_remains_uneditable_despite_tags() {
    let mut db = SkillDatabase::from_ron("audit/probe.ron", RON).unwrap();
    db.skills[0].steps[0].effects[0].op = SkillEffectDef::InterruptCast;
    db.skills[0].steps[0].effects[0].tags = vec!["Damage".into(), "amount".into()];
    assert!(effect_support(&db.skills[0].steps[0].effects[0].op, &DeliveryDef::Instant).is_none());
    let diagnostic = db.set_effect_amount(&address(), EffectProperty::Amount, 70).unwrap_err();
    assert_eq!(diagnostic.source.unwrap().site, address());
    assert!(matches!(db.skills[0].steps[0].effects[0].op, SkillEffectDef::InterruptCast));
}
#[test]
fn supported_literal_on_spatial_delivery_rejects_edit_without_mutation() {
    let live = ValidatedGameData::load_live_embedded().unwrap();
    let delivery = live.skill_data.skills.iter().flat_map(|skill| &skill.steps)
        .find_map(|step| matches!(step.delivery, DeliveryDef::Projectile { .. }).then(|| step.delivery.clone())).unwrap();
    let mut db = SkillDatabase::from_ron("audit/probe.ron", RON).unwrap();
    db.skills[0].steps[0].delivery = delivery;
    assert!(effect_support(&db.skills[0].steps[0].effects[0].op, &db.skills[0].steps[0].delivery).is_none());
    let diagnostic = db.set_effect_amount(&address(), EffectProperty::Amount, 70).unwrap_err();
    assert_eq!(diagnostic.site, address());
    assert_eq!(db.effect(&address()).unwrap().literal_amount(), Some(17));
}
#[test]
fn both_action_carriers_and_readonly_damage_type_are_rejected() {
    let mut db = SkillDatabase::from_ron("audit/probe.ron", RON).unwrap();
    for property in [EffectProperty::Target, EffectProperty::Delivery, EffectProperty::DamageType] {
        let diagnostic = db.set_effect_amount(&address(), property, 70).unwrap_err();
        assert_eq!(diagnostic.property, property);
        assert_eq!(diagnostic.affected_effects.len(), 2);
        assert_eq!(diagnostic.source.unwrap().site, address());
        assert_eq!(db.effect(&address()).unwrap().literal_amount(), Some(17));
    }
}
