use super::*;
use crate::game::{
    ability::{
        authoring::AuthoredSkillEffect, SkillCastTargetingDef, SkillDef, SkillEffectDef,
        SkillStepDef, SkillTarget, SkillTargetReach, SkillUseLifetime,
    },
    battle::{damage::DamageType, tile_range::TileRangePattern},
    data::{
        relic_data::{RelicDatabase, RelicDefinition},
        skill_data::SkillDatabase,
        skill_fragment_data::*,
        GameDataBuilder,
    },
    managers::uuid_manager::UuidManager,
};

struct Fixture {
    data: ValidatedGameData,
    inventory: RelicInventory,
    providers: Vec<ProspectiveProvider>,
    employee: Uuid,
}
impl Fixture {
    fn explain(&self) -> Result<dto::RelicEffectivePlanDto, RelicPlanError> {
        explain_effective_plan(&self.data, self.employee, &self.inventory, &self.providers)
            .map(|p| p.expect("structural fixture has a rule"))
    }
}
fn rule(id: &str, apply: RelicRuleActionDef) -> RelicRuleDef {
    RelicRuleDef {
        id: id.into(),
        selector: Default::default(),
        when: RelicConditionDef::Always,
        apply,
    }
}
fn tags(values: &[&str]) -> StructuralSelectorDef {
    StructuralSelectorDef {
        effect: EffectPredicateDef {
            all_tags: values.iter().map(|s| (*s).into()).collect(),
            ..Default::default()
        },
        ..Default::default()
    }
}
fn grant(id: &str, tag: &str, amount: i32) -> RelicRuleDef {
    rule(
        id,
        RelicRuleActionDef::Grant(GrantBodyDef::InlineEffects {
            action_id: "anchor".into(),
            effects: vec![GrantedEffectDef {
                id: "generated".into(),
                tags: vec![tag.into()],
                op: GrantPrimitiveDef::Damage {
                    amount,
                    damage_type: DamageType::Physical,
                },
            }],
        }),
    )
}
fn edit(
    id: &str,
    selector: StructuralSelectorDef,
    property: RelicProperty,
    operation: NumericEditDef,
) -> RelicRuleDef {
    RelicRuleDef {
        selector,
        ..rule(
            id,
            RelicRuleActionDef::Edit {
                property,
                operation,
            },
        )
    }
}
fn damage(id: &str, amount: i32, labels: &[&str]) -> AuthoredSkillEffect {
    let mut e = AuthoredSkillEffect::new(
        id,
        SkillEffectDef::Damage {
            amount,
            damage_type: DamageType::Physical,
        },
    );
    e.tags = labels.iter().map(|s| (*s).into()).collect();
    e
}
fn definition(rules: Vec<RelicRuleDef>) -> RelicDefinition {
    RelicDefinition {
        id: "u09_relic".into(),
        uuid: Uuid::from_u128(9001),
        name: "U09 private compiler fixture".into(),
        description: "Not normal publication evidence".into(),
        rarity: "probe".into(),
        allow_duplicates: true,
        incompatible_with: vec![],
        effects: vec![],
        rules,
        corruption: None,
        dismantle_equipment_dust: 1,
    }
}
fn fixture(rules: Vec<RelicRuleDef>, effects: Vec<AuthoredSkillEffect>) -> Fixture {
    let skill = SkillDef {
        action_policy: None,
        resonance_admission: None,
        id: "u09_skill".into(),
        name: "U09 private compiler fixture".into(),
        kind: crate::game::ability::SkillKind::Untargeted,
        use_lifetime: SkillUseLifetime::Action,
        flow: None,
        cast_targeting: SkillCastTargetingDef::explicit(
            SkillTarget::SelfUnit,
            SkillTargetReach::Pattern,
            None,
            false,
        ),
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "anchor".into(),
            delay_ms: 0,
            target: SkillTarget::SelfUnit,
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: Default::default(),
            effects,
            presentation: Default::default(),
        }],
    };
    let fragment = SkillFragmentMetadata {
        id: "u09_fragment".into(),
        uuid: Uuid::from_u128(9002),
        name: "U09 private provider".into(),
        description: "compiler only".into(),
        rarity: SkillFragmentRarity::Common,
        equip_limit: SkillFragmentEquipLimit::OwnedCopies,
        origin: None,
        corrosion_class: Default::default(),
        sources: vec![],
        dependencies: vec![],
        compatibility: Default::default(),
        effect: SkillFragmentEffectDef::ActiveSkill {
            forms: [(
                "base".into(),
                SkillFragmentForm {
                    skill_id: skill.id.clone(),
                    resonance_max: 100,
                    compatibility: None,
                    bloom_recipe: None,
                },
            )]
            .into(),
            dismantle_recipe: None,
            passive_guard: None,
        },
    };
    // The normal builder validates an empty baseline. Test-only catalog assembly
    // plus From<Arc<_>> intentionally supplies a private immutable generation.
    // It does not bypass the public loader in any runtime or publication test.
    let mut database = GameDataBuilder::empty().build();
    database.skill_data = Arc::new(SkillDatabase::new(vec![skill]));
    database.skill_fragment_data = Arc::new(SkillFragmentDatabase::new(vec![fragment]));
    database.relic_data = Arc::new(RelicDatabase {
        relics: vec![definition(rules)],
    });
    let employee = Uuid::from_u128(9003);
    let mut inventory = RelicInventory::default();
    let mut ids = UuidManager::new(9004);
    let copy = inventory
        .grant(&database.relic_data.relics[0], &mut ids)
        .unwrap();
    inventory
        .equip(copy, employee, 0, &database.relic_data)
        .unwrap();
    let providers = vec![ProspectiveProvider::Fragment(FragmentSourceSnapshot {
        copy_uuid: Uuid::from_u128(9005),
        definition_id: "u09_fragment".into(),
        form_id: "base".into(),
        skill_id: "u09_skill".into(),
    })];
    Fixture {
        data: Arc::new(database).into(),
        inventory,
        providers,
        employee,
    }
}
fn hp(effect: &dto::EffectiveEffectDto) -> i64 {
    match effect
        .properties
        .iter()
        .find(|p| p.property == RelicProperty::DamageAmount)
        .unwrap()
        .final_value
        .unwrap()
    {
        TypedRelicValue::Hp(v) => v,
        other => panic!("wrong unit {other:?}"),
    }
}
fn generated(plan: &dto::RelicEffectivePlanDto) -> Vec<&dto::EffectiveEffectDto> {
    plan.effects
        .iter()
        .filter(|e| matches!(e.key, EffectiveEffectKey::Generated { .. }))
        .collect()
}

#[test]
fn u09_generated_75_finite_identity_and_dto_are_read_only() {
    let rules = vec![
        grant("grant", "bonus", 40),
        edit(
            "add",
            tags(&["bonus"]),
            RelicProperty::DamageAmount,
            NumericEditDef::Add(TypedRelicValue::Hp(10)),
        ),
        edit(
            "scale",
            tags(&["bonus"]),
            RelicProperty::DamageAmount,
            NumericEditDef::Scale {
                increment_percent: 50,
            },
        ),
    ];
    let mut f = fixture(rules, vec![damage("original", 11, &["original"])]);
    let before_inventory = serde_json::to_vec(&f.inventory).unwrap();
    let before_relics = serde_json::to_vec(&f.data.relic_data).unwrap();
    let before_skills = serde_json::to_vec(&f.data.skill_data).unwrap();
    let first = f.explain().unwrap();
    assert_eq!(first.effects.len(), 2);
    assert_eq!(first.compile_usage.candidate_count, 2);
    assert_eq!(generated(&first).len(), 1);
    assert_eq!(hp(generated(&first)[0]), 75); // (40 + 10) * 150 / 100
    assert_eq!(
        hp(first
            .effects
            .iter()
            .find(|e| matches!(e.key, EffectiveEffectKey::Original { .. }))
            .unwrap()),
        11
    );
    assert_eq!(first.schema_version, 1);
    assert_eq!(first.scope, "prospective_loadout");
    assert_eq!(first.execution_status, "runtime_supported_prospective_plan");
    assert!(first.capabilities.iter().all(|c| c.runtime_connected));
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&f.explain().unwrap()).unwrap()
    );
    assert_eq!(serde_json::to_vec(&f.inventory).unwrap(), before_inventory);
    assert_eq!(
        serde_json::to_vec(&f.data.relic_data).unwrap(),
        before_relics
    );
    assert_eq!(
        serde_json::to_vec(&f.data.skill_data).unwrap(),
        before_skills
    );
    // Same skill/form, different actual provider copy: two distinct addresses,
    // exactly one finite grant each. Enumeration order cannot change the DTO.
    let mut second = f.providers[0].clone();
    if let ProspectiveProvider::Fragment(ref mut p) = second {
        p.copy_uuid = Uuid::from_u128(9006);
    }
    f.providers.push(second);
    let two = f.explain().unwrap();
    assert_eq!(generated(&two).len(), 2);
    assert_eq!(two.compile_usage.candidate_count, 4);
    assert_ne!(
        generated(&two)[0].provider_index,
        generated(&two)[1].provider_index
    );
    assert!(generated(&two).iter().all(|e| hp(e) == 75));
    f.providers.reverse();
    assert_eq!(
        serde_json::to_vec(&two).unwrap(),
        serde_json::to_vec(&f.explain().unwrap()).unwrap()
    );
}

#[test]
fn u09_positive_cycle_needs_current_seed_and_does_not_recurse() {
    let mut a = grant("a", "a", 40);
    a.when = RelicConditionDef::Exists(tags(&["b"]));
    let mut b = grant("b", "b", 7);
    b.when = RelicConditionDef::Exists(tags(&["a"]));
    for reverse in [false, true] {
        let mut rules = vec![a.clone(), b.clone()];
        if reverse {
            rules.reverse();
        }
        for seed in [false, true, false] {
            let f = fixture(
                rules.clone(),
                vec![damage("original", 1, if seed { &["a"] } else { &["none"] })],
            );
            let plan = f.explain().unwrap();
            assert_eq!(plan.compile_usage.candidate_count, 3);
            assert_eq!(
                generated(&plan).len(),
                if seed { 2 } else { 0 },
                "seed={seed} reverse={reverse}"
            );
            assert!(plan
                .rules
                .iter()
                .all(|r| matches!(r.state, dto::RelicRuleState::Active) == seed));
        }
    }
}

#[test]
fn u09_same_effect_witness_cannot_be_split_across_siblings() {
    let a = edit(
        "all_tags",
        tags(&["red", "blue"]),
        RelicProperty::DamageAmount,
        NumericEditDef::Add(TypedRelicValue::Hp(10)),
    );
    let mut b = edit(
        "same",
        tags(&["red"]),
        RelicProperty::DamageAmount,
        NumericEditDef::Add(TypedRelicValue::Hp(20)),
    );
    b.when = RelicConditionDef::SameEffect(EffectPredicateDef {
        all_tags: vec!["blue".into()],
        ..Default::default()
    });
    let f = fixture(
        vec![a, b],
        vec![
            damage("red", 10, &["red"]),
            damage("blue", 10, &["blue"]),
            damage("both", 10, &["red", "blue"]),
        ],
    );
    let p = f.explain().unwrap();
    for e in &p.effects {
        assert_eq!(hp(e), if e.tags.len() == 2 { 40 } else { 10 });
    }
    assert!(p.rules.iter().all(|r| r.targets.len() == 1));
}

#[test]
fn u09_negative_cycle_and_provider_mismatch_are_typed_rejections() {
    let mut a = grant("a", "a", 40);
    a.when = RelicConditionDef::Not(Box::new(RelicConditionDef::Exists(tags(&["b"]))));
    let mut b = grant("b", "b", 7);
    b.when = RelicConditionDef::Exists(tags(&["a"]));
    let f = fixture(vec![a, b], vec![damage("original", 1, &[])]);
    assert_eq!(
        f.explain().unwrap_err().code,
        FaultCode::NegativeDependencyCycle
    );
    let mut f = fixture(
        vec![grant("grant", "bonus", 40)],
        vec![damage("original", 1, &[])],
    );
    if let ProspectiveProvider::Fragment(ref mut p) = f.providers[0] {
        p.form_id = "absent".into();
    }
    assert_eq!(
        f.explain().unwrap_err().code,
        FaultCode::ProviderUnavailable
    );
    f.providers.clear();
    let p = f.explain().unwrap();
    assert!(p.effects.is_empty());
    assert!(matches!(
        p.rules[0].state,
        dto::RelicRuleState::Dormant {
            reason: dto::RelicDormantReason::ProviderUnavailable
        }
    ));
}

#[test]
fn u09_footprint_recipe_produces_an_effect_local_typed_property() {
    let mut recipe = rule(
        "footprint",
        RelicRuleActionDef::Recipe(StructuralRecipeDef::DamageFootprint {
            footprint: Arc::new(TileRangePattern {
                include_anchor_tile: true,
                rows: vec!["@X".into()],
            }),
        }),
    );
    recipe.selector = tags(&["selected"]);
    let mut conditional = edit(
        "after",
        tags(&["selected"]),
        RelicProperty::DamageAmount,
        NumericEditDef::Add(TypedRelicValue::Hp(9)),
    );
    conditional.selector.effect.has_property = Some(RelicProperty::DamageFootprint);
    for rules in [
        vec![recipe.clone(), conditional.clone()],
        vec![conditional, recipe],
    ] {
        let f = fixture(
            rules,
            vec![
                damage("selected", 10, &["selected"]),
                damage("sibling", 10, &["sibling"]),
            ],
        );
        let p = f.explain().unwrap();
        for e in &p.effects {
            let selected = e.tags == ["selected"];
            assert_eq!(hp(e), if selected { 19 } else { 10 });
            assert_eq!(
                e.properties
                    .iter()
                    .any(|p| p.property == RelicProperty::DamageFootprint),
                selected
            );
            assert_eq!(e.recipes.len(), usize::from(selected));
        }
        assert!(matches!(
            f.data.skill_data.skills[0].steps[0].delivery,
            DeliveryDef::Instant
        ));
    }
}

#[test]
fn u09_normal_builder_accepts_dormant_structural_publication_and_legacy_serde_stays_stable() {
    let authored = definition(vec![grant("grant", "bonus", 40)]);
    let result = GameDataBuilder::empty()
        .with_relics(RelicDatabase {
            relics: vec![authored],
        })
        .build_with_runtime_safety(crate::runtime_safety::RuntimeSafetySettings::embedded_shared());
    // No provider exists: the finite grant is dormant, not a publication error.
    // An actual provider/cast is covered independently by the World TB tests.
    let published = result.expect("connected structural schema must publish dormant rules");
    assert_eq!(published.relic_data.relics[0].rules.len(), 1);
    let legacy = r#"(id:"legacy",uuid:"00000000-0000-0000-0000-000000009001",name:"legacy",description:"legacy",rarity:"probe",allow_duplicates:false,effects:[(id:"damage",flat:2)],dismantle_equipment_dust:1)"#;
    let d: RelicDefinition = ron::de::from_str(legacy).unwrap();
    assert!(d.rules.is_empty());
    let serialized = serde_json::to_value(&d).unwrap();
    assert!(serialized.get("rules").is_none());
    let db = GameDataBuilder::empty()
        .with_relics(RelicDatabase { relics: vec![d] })
        .build();
    let data: ValidatedGameData = Arc::new(db).into();
    assert!(
        explain_effective_plan(&data, Uuid::from_u128(9), &RelicInventory::default(), &[])
            .unwrap()
            .is_none()
    );
}

fn amount(
    value: i128,
    moment: crate::game::value_expr::EvalMoment,
) -> Arc<crate::game::value_expr::AmountExpr> {
    use crate::game::value_expr::{AmountExpr, TypedInteger, ValueExpr, ValueUnit};
    Arc::new(AmountExpr::authored(
        moment,
        ValueExpr::Const(TypedInteger::new(value, ValueUnit::Magnitude)),
    ))
}
fn personal() -> AuthoredSkillEffect {
    AuthoredSkillEffect::new(
        "personal",
        SkillEffectDef::PersonalShieldGrant {
            definition: PersonalShieldGrantDef {
                support: None,
                replace_only_recipients: false,
                amount: amount(200, crate::game::value_expr::EvalMoment::Snapshot),
                duration_ms: 1000,
                priority: 5,
                recipients: PersonalShieldRecipientsDef::PlayerEmployeesAtRelease {
                    radius_units: 2000,
                },
                source_creation: Some(ShieldCreationModifierDef {
                    operations: vec![NumericEditDef::Scale {
                        increment_percent: 50,
                    }],
                }),
            },
        },
    )
}
fn shield_selector() -> StructuralSelectorDef {
    StructuralSelectorDef {
        effect: EffectPredicateDef {
            op: Some(EffectOperation::PersonalShieldGrant),
            ..Default::default()
        },
        ..Default::default()
    }
}
fn single(value: i128, moment: crate::game::value_expr::EvalMoment) -> RelicRuleDef {
    RelicRuleDef {
        selector: shield_selector(),
        ..rule(
            "single",
            RelicRuleActionDef::Recipe(StructuralRecipeDef::PersonalShieldSingle {
                initial_amount: amount(value, moment),
            }),
        )
    }
}

#[test]
fn u09_personal_amount_and_source_creation_remain_separate_typed_stages() {
    use crate::game::value_expr::EvalMoment;
    // Representative private values from the independent P02 oracle. Runtime
    // recipient grants are not executed here: the DTO must expose both stages.
    for (single_base, add, typed, after_creation) in [
        (None, 40, 300, 450),
        (Some(420), 20, 550, 825),
        (Some(620), 20, 800, 1200),
    ] {
        let mut rules = vec![
            edit(
                "amount_add",
                shield_selector(),
                RelicProperty::PersonalShieldInitialAmount,
                NumericEditDef::Add(TypedRelicValue::Hp(add)),
            ),
            edit(
                "amount_scale",
                shield_selector(),
                RelicProperty::PersonalShieldInitialAmount,
                NumericEditDef::Scale {
                    increment_percent: 25,
                },
            ),
        ];
        if let Some(base) = single_base {
            rules.push(single(base, EvalMoment::Snapshot));
        }
        let f = fixture(rules, vec![personal()]);
        let before = serde_json::to_vec(&f.data.skill_data).unwrap();
        let p = f.explain().unwrap();
        let e = &p.effects[0];
        let value = e
            .properties
            .iter()
            .find(|p| p.property == RelicProperty::PersonalShieldInitialAmount)
            .unwrap()
            .final_value
            .unwrap();
        assert_eq!(value, TypedRelicValue::Hp(typed));
        assert_eq!(
            e.properties
                .iter()
                .any(|p| p.property == RelicProperty::SupportAreaRadius),
            single_base.is_none()
        );
        assert_eq!(e.amount.as_ref().unwrap().moment, EvalMoment::Snapshot);
        let creation = e.source_creation.as_ref().unwrap();
        assert_eq!(
            creation.operations,
            [NumericEditDef::Scale {
                increment_percent: 50
            }]
        );
        let mut program = NumericProgram::default();
        let desc = descriptor(
            EffectOperation::PersonalShieldGrant,
            RelicProperty::PersonalShieldInitialAmount,
        )
        .unwrap();
        for operation in &creation.operations {
            program.include(*operation, desc).unwrap();
        }
        assert_eq!(
            program.apply(value, desc).unwrap(),
            TypedRelicValue::Hp(after_creation)
        );
        assert_eq!(serde_json::to_vec(&f.data.skill_data).unwrap(), before);
    }
}

#[test]
fn u09_removed_radius_is_dormant_only_with_explicit_property_condition() {
    use crate::game::value_expr::EvalMoment;
    let transformation = single(420, EvalMoment::Snapshot);
    let unconditional = edit(
        "radius",
        shield_selector(),
        RelicProperty::SupportAreaRadius,
        NumericEditDef::Add(TypedRelicValue::DataUnits(100)),
    );
    let f = fixture(
        vec![transformation.clone(), unconditional.clone()],
        vec![personal()],
    );
    assert_eq!(f.explain().unwrap_err().code, FaultCode::MissingProperty);
    let mut conditional = unconditional;
    conditional.selector.effect.has_property = Some(RelicProperty::SupportAreaRadius);
    for rules in [
        vec![transformation.clone(), conditional.clone()],
        vec![conditional, transformation],
    ] {
        let p = fixture(rules, vec![personal()]).explain().unwrap();
        assert!(!p.effects[0]
            .properties
            .iter()
            .any(|p| p.property == RelicProperty::SupportAreaRadius));
        let state = &p
            .rules
            .iter()
            .find(|r| r.contribution.rule_id == "radius")
            .unwrap()
            .state;
        assert!(matches!(
            state,
            dto::RelicRuleState::Dormant {
                reason: dto::RelicDormantReason::ConditionUnmet
            }
        ));
    }
}

#[test]
fn u09_personal_single_preserves_eval_moment_and_conflicting_recipes_reject() {
    use crate::game::value_expr::EvalMoment;
    let f = fixture(vec![single(420, EvalMoment::Live)], vec![personal()]);
    assert_eq!(
        f.explain().unwrap_err().code,
        FaultCode::InvalidPreserveContract
    );
    let mut other = single(421, EvalMoment::Snapshot);
    other.id = "different_single".into();
    let f = fixture(
        vec![single(420, EvalMoment::Snapshot), other],
        vec![personal()],
    );
    assert_eq!(
        f.explain().unwrap_err().code,
        FaultCode::StructuralWriteConflict
    );
    let mut same = single(420, EvalMoment::Snapshot);
    same.id = "same_single".into();
    let p = fixture(
        vec![single(420, EvalMoment::Snapshot), same],
        vec![personal()],
    )
    .explain()
    .unwrap();
    assert_eq!(p.effects[0].recipes.len(), 1);
    assert_eq!(p.effects[0].recipe_contributions.len(), 2);
}

#[path = "publication_cases.rs"]
mod publication_cases;
