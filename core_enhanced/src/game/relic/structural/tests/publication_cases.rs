//! Literal normal-publication counterexamples for whole-context proof reuse.
//! The parent supplies authored building blocks, never expected compiler output.
use super::*;
use crate::game::{
    ability::{AbilityActivationBinding, AbilityActivationDef},
    battle::runtime_control as native,
    data::{
        equipment_data::{EquipmentMetadata, EquipmentType, WeaponArchetype, WeaponCombatProfile},
        relic_data::RelicCorruptionVariant,
        GameDataBase,
    },
    stats::TriggerType,
};

fn item(id: &str, n: u128, skills: &[&str]) -> EquipmentMetadata {
    let mut value: EquipmentMetadata = ron::de::from_str(&format!(
        "(id:{id:?},uuid:{:?},name:{id:?},equipment_type:Accessory,rarity:ZAYIN,price:0,allow_duplicate_equip:false)", Uuid::from_u128(n).to_string()
    )).unwrap();
    value.ability_activations = skills
        .iter()
        .map(|skill| AbilityActivationBinding {
            ability_id: (*skill).into(),
            activation: AbilityActivationDef::TriggerProc {
                trigger: TriggerType::OnAttack,
                proc_chance_percent: 100,
                internal_cooldown_ms: 0,
                max_triggers_per_battle: None,
            },
        })
        .collect();
    value
}
fn weapon(id: &str, n: u128, archetype: WeaponArchetype) -> EquipmentMetadata {
    let mut value = item(id, n, &[]);
    value.equipment_type = EquipmentType::Weapon;
    value.weapon_profile = Some(WeaponCombatProfile {
        weapon_archetype: archetype,
        ..Default::default()
    });
    value
}
fn replace(id: &str, amount: i64) -> RelicRuleDef {
    edit(
        id,
        tags(&["p"]),
        RelicProperty::DamageAmount,
        NumericEditDef::Replace(TypedRelicValue::Hp(amount)),
    )
}
fn relic(rules: Vec<RelicRuleDef>) -> RelicDefinition {
    let mut value = definition(rules);
    value.allow_duplicates = false;
    value.name = "Independent normal publication witness".into();
    value.description = "Literal legal/illegal coexistence".into();
    value
}
fn skills(labels: &[&str]) -> Vec<SkillDef> {
    let f = fixture(vec![], vec![damage("hit", 10, &["p"])]);
    labels
        .iter()
        .map(|label| {
            let mut skill = f.data.skill_data.skills[0].clone();
            skill.id = (*label).into();
            skill.steps[0].effects = vec![damage("hit", 10, &[*label])];
            skill
        })
        .collect()
}
fn build(
    items: Vec<EquipmentMetadata>,
    rules: Vec<RelicRuleDef>,
    labels: &[&str],
) -> Result<GameDataBase, crate::game::data::live_ron::LiveRonError> {
    GameDataBuilder::empty()
        .with_equipment(items)
        .with_skills(SkillDatabase::new(skills(labels)))
        .with_relics(RelicDatabase {
            relics: vec![relic(rules)],
        })
        .build_with_runtime_safety(crate::runtime_safety::RuntimeSafetySettings::embedded_shared())
}
fn four_items() -> Vec<EquipmentMetadata> {
    ["p", "q", "r", "s"]
        .into_iter()
        .enumerate()
        .map(|(n, id)| item(id, 0xf100 + n as u128, &[id]))
        .collect()
}
fn four_rules() -> Vec<RelicRuleDef> {
    let mut second = replace("second", 32);
    second.when = RelicConditionDef::All(
        vec!["q", "r", "s"]
            .into_iter()
            .map(|tag| RelicConditionDef::Exists(tags(&[tag])))
            .collect(),
    );
    vec![replace("first", 31), second]
}

#[test]
fn rv004_publication_whole_context_uses_real_three_slot_witness_and_not_union() {
    build(four_items(), four_rules(), &["p", "q", "r", "s"])
        .expect("four separate accessories cannot coexist in Any3");
    let combined = vec![
        item("p", 0xf101, &["p"]),
        item("qr", 0xf102, &["q", "r"]),
        item("s", 0xf103, &["s"]),
    ];
    assert_eq!(
        build(combined, four_rules(), &["p", "q", "r", "s"])
            .err()
            .unwrap()
            .code,
        "relic_structural_replace_conflict"
    );
    let mut absent = replace("absent_q", 32);
    absent.when = RelicConditionDef::Not(Box::new(RelicConditionDef::Exists(tags(&["q"]))));
    assert_eq!(
        build(
            vec![item("p", 0xf111, &["p"]), item("q", 0xf112, &["q"])],
            vec![replace("first", 31), absent],
            &["p", "q"]
        )
        .err()
        .unwrap()
        .code,
        "relic_structural_replace_conflict",
        "P alone is legal even though Q exists elsewhere in the catalog"
    );
}

#[test]
fn rv004_publication_definition_variant_and_invocation_contexts_do_not_share_success() {
    let mut second = replace("only_b", 32);
    second.selector.provider.equipment_id = Some("b".into());
    let authored = vec![replace("first", 31), second];
    let f = fixture(vec![], vec![damage("hit", 10, &["p"])]);
    let run = |items: Vec<EquipmentMetadata>, rules: Vec<RelicRuleDef>, corrupt| {
        let mut r = relic(rules);
        if corrupt {
            r.corruption = Some(RelicCorruptionVariant {
                effects: vec![],
                penalties: vec![crate::game::data::relic_data::RelicPenalty::DefenseFlat {
                    amount: -1,
                }],
                rules: vec![replace("corrupt", 32)],
            });
        }
        let mut extra = relic(vec![replace("coalesce", 31)]);
        extra.id = "other".into();
        extra.uuid = Uuid::from_u128(0xf122);
        GameDataBuilder::from_validated(&f.data)
            .with_skill_fragments(SkillFragmentDatabase::new(vec![]))
            .with_skills(SkillDatabase::new(skills(&["p"])))
            .with_equipment(items)
            .with_relics(RelicDatabase {
                relics: vec![r, extra],
            })
            .build_with_runtime_safety(
                crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
            )
    };
    let same = || vec![item("a", 0xf120, &["p"]), item("b", 0xf121, &["p"])];
    // Separate invocations use identical definition text/UUIDs but different rules.
    run(same(), vec![replace("first", 31)], false).unwrap();
    assert_eq!(
        run(same(), authored, false).err().unwrap().code,
        "relic_structural_replace_conflict"
    );
    run(same(), vec![replace("first", 31)], false).unwrap();
    let failure = run(same(), vec![replace("first", 31)], true).err().unwrap();
    assert_eq!(
        failure.code, "relic_structural_replace_conflict",
        "corrupt32 and other31 are a legal pair: {failure:?}"
    );
    run(same(), vec![replace("first", 31)], false).unwrap();
}

fn form_fixture() -> Fixture {
    fixture(vec![replace("first", 31)], vec![damage("hit", 10, &["p"])])
}
#[test]
fn rv004_publication_form_and_weapon_compatibility_keep_distinct_contexts() {
    let f = form_fixture();
    let mut fragments = f.data.skill_fragment_data.fragments.clone();
    let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &mut fragments[0].effect else {
        panic!("active")
    };
    let mut alt = forms["base"].clone();
    alt.compatibility = Some(SkillFragmentCompatibilityRequirements {
        allowed_weapon_archetypes: vec![WeaponArchetype::Staff],
        ..Default::default()
    });
    forms.get_mut("base").unwrap().compatibility = Some(SkillFragmentCompatibilityRequirements {
        allowed_weapon_archetypes: vec![WeaponArchetype::Sword],
        ..Default::default()
    });
    forms.insert("zz_alternate".into(), alt);
    let mut bad = replace("alternate_only", 32);
    bad.selector.provider.fragment_id = Some("u09_fragment".into());
    bad.selector.provider.form_id = Some("zz_alternate".into());
    let run = |items| {
        GameDataBuilder::from_validated(&f.data)
            .with_equipment(items)
            .with_skill_fragments(SkillFragmentDatabase::new(fragments.clone()))
            .with_relics(RelicDatabase {
                relics: vec![relic(vec![replace("first", 31), bad.clone()])],
            })
            .build_with_runtime_safety(
                crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
            )
    };
    run(vec![weapon("sword", 0xf131, WeaponArchetype::Sword)]).unwrap();
    let failure = run(vec![
        weapon("sword", 0xf131, WeaponArchetype::Sword),
        weapon("staff", 0xf132, WeaponArchetype::Staff),
    ])
    .err()
    .unwrap();
    assert_eq!(
        failure.code, "relic_structural_replace_conflict",
        "{failure:?}"
    );
    run(vec![weapon("sword", 0xf131, WeaponArchetype::Sword)]).unwrap();
}

#[test]
fn rv004_publication_lowering_still_rejects_conflicting_selected_reaches() {
    use crate::game::relic::structural::authoring::PersonalShieldRecipientsDef;
    let mut a = personal();
    a.id = "a".into();
    let mut b = a.clone();
    b.id = "b".into();
    let set_reach = |e: &mut AuthoredSkillEffect, radius| {
        let SkillEffectDef::PersonalShieldGrant { definition } = &mut e.op else {
            panic!("personal")
        };
        definition.recipients = PersonalShieldRecipientsDef::PlayerEmployeesAtRelease {
            radius_units: radius,
        };
    };
    set_reach(&mut a, 1_000_000);
    set_reach(&mut b, 2_000_000);
    let run = |effects| {
        let f = fixture(
            vec![single(420, crate::game::value_expr::EvalMoment::Snapshot)],
            effects,
        );
        GameDataBuilder::from_validated(&f.data)
            .with_equipment(vec![weapon("sword", 0xf140, WeaponArchetype::Sword)])
            .build_with_runtime_safety(
                crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
            )
    };
    let mut same = a.clone();
    same.id = "b".into();
    run(vec![a.clone(), same]).unwrap();
    let failure = run(vec![a, b])
        .err()
        .expect("each effect is legal; conflicting effective selected reach must fail lowering");
    assert_eq!(failure.code, "relic_structural_invalid_reference");
    assert!(failure
        .to_string()
        .contains("selected support reach conflict"));
}

fn f01_personal_effect(id: &str, reach: u32) -> AuthoredSkillEffect {
    let mut effect = personal();
    effect.id = id.into();
    let SkillEffectDef::PersonalShieldGrant { definition } = &mut effect.op else {
        unreachable!()
    };
    definition.recipients = PersonalShieldRecipientsDef::PlayerEmployeesAtRelease {
        radius_units: reach,
    };
    effect
}

fn f01_generated(id: &str, reach: u32) -> GrantedEffectDef {
    let SkillEffectDef::PersonalShieldGrant { definition } = f01_personal_effect(id, reach).op
    else {
        unreachable!()
    };
    GrantedEffectDef {
        id: id.into(),
        tags: vec![],
        op: GrantPrimitiveDef::PersonalShieldGrant { definition },
    }
}

fn f01_grant(id: &str, finite: bool, effects: Vec<GrantedEffectDef>) -> RelicRuleDef {
    let body = if finite {
        GrantBodyDef::FiniteFlow {
            anchor_action_id: "anchor".into(),
            steps: vec![GrantedFlowStepDef {
                id: "child".into(),
                delay_ms: 0,
                target: SkillTarget::SelfUnit,
                delivery: Default::default(),
                effects,
            }],
            flow: ron::de::from_str(
                r#"Action(id: "child_release", step_id: "child", wait: EffectsSettled)"#,
            )
            .unwrap(),
        }
    } else {
        GrantBodyDef::InlineEffects {
            action_id: "anchor".into(),
            effects,
        }
    };
    rule(id, RelicRuleActionDef::Grant(body))
}

/// Every candidate is submitted to the complete normal builder. Reversing
/// distinct definitions also reverses their generated publication copy witness:
/// publication_input grants in definition order, then compiler orders by UUID.
/// No invalid generation is supplied to a running World.
fn f01_candidate(
    layout: &str,
    finite: bool,
    reach: u32,
    reversed: bool,
) -> (SkillDef, Vec<RelicDefinition>) {
    let inert = AuthoredSkillEffect::new("inert", SkillEffectDef::Heal { amount: 1 });
    let mut originals = vec![inert];
    let mut recipes = vec![single(420, crate::game::value_expr::EvalMoment::Snapshot)];
    if layout == "original_personal" {
        originals.push(f01_personal_effect("original", 2_000_000));
    } else if layout == "mixed_fixed" {
        originals.push(
            ron::de::from_str(
                r#"(id: "channel", op: Sustained(definition: ChannelAura(
            duration_ms: 30000, aura: (family: "f01", priority: 0, radius_units: 2000000,
                basic_damage_percent: 20, basic_speed_points: 15))))"#,
            )
            .unwrap(),
        );
        let mut fixed = rule(
            "fixed",
            RelicRuleActionDef::Recipe(StructuralRecipeDef::FixedRecipientSupport),
        );
        fixed.selector.effect.op = Some(EffectOperation::ChannelAura);
        recipes.push(fixed);
    }
    if reversed {
        originals.reverse();
    }
    // Set the channel's lifetime before SkillDatabase::new validates it.
    let mut skill = fixture(vec![], vec![]).data.skill_data.skills[0].clone();
    skill.steps[0].effects = originals;
    if layout == "mixed_fixed" {
        skill.kind = crate::game::ability::SkillKind::Targeted;
        skill.use_lifetime = ron::de::from_str(
            r#"ActionAndEffects(effects: [(step: "anchor", effect: "channel")])"#,
        )
        .unwrap();
    }
    let mut grants = if layout == "generated_effects" {
        let mut effects = vec![
            f01_generated("first", 2_000_000),
            f01_generated("second", reach),
        ];
        if reversed {
            effects.reverse();
        }
        vec![f01_grant("grant", finite, effects)]
    } else if layout.starts_with("generated_") {
        vec![
            f01_grant("grant_a", finite, vec![f01_generated("first", 2_000_000)]),
            f01_grant("grant_b", finite, vec![f01_generated("second", reach)]),
        ]
    } else {
        let mut effects = vec![
            f01_generated("generated", reach),
            GrantedEffectDef {
                id: "inert_generated".into(),
                tags: vec![],
                op: GrantPrimitiveDef::Heal { amount: 1 },
            },
        ];
        if reversed {
            effects.reverse();
        }
        vec![f01_grant("grant", finite, effects)]
    };
    let mut definitions = if layout == "generated_copies" {
        let other = grants.pop().unwrap();
        recipes.extend(grants);
        let first = relic(recipes);
        let mut second = relic(vec![other]);
        second.id = "f01_second".into();
        second.uuid = Uuid::from_u128(0xf152);
        vec![first, second]
    } else {
        recipes.extend(grants);
        vec![relic(recipes)]
    };
    if reversed {
        definitions.reverse();
        for definition in &mut definitions {
            definition.rules.reverse();
        }
    }
    (skill, definitions)
}

fn f01_publish(
    skill: SkillDef,
    definitions: Vec<RelicDefinition>,
) -> Result<GameDataBase, crate::game::data::live_ron::LiveRonError> {
    let fragments = fixture(vec![], vec![])
        .data
        .skill_fragment_data
        .fragments
        .clone();
    GameDataBuilder::empty()
        .with_skills(SkillDatabase::new(vec![skill]))
        .with_skill_fragments(SkillFragmentDatabase::new(fragments))
        .with_equipment(vec![weapon("f01_sword", 0xf150, WeaponArchetype::Sword)])
        .with_relics(RelicDatabase {
            relics: definitions,
        })
        .build_with_runtime_safety(crate::runtime_safety::RuntimeSafetySettings::embedded_shared())
}

#[test]
fn f01_generated_selected_reach_publication_refuses_conflicts_in_all_orders() {
    // Literal oracle: one provider has one reach. 2m combined with 1m or 4m
    // refuses; 2m plus 2m remains legal. A successful pre-overlay publication
    // distinguishes the reported defect from an already-invalid carrier.
    let mut unexpected = Vec::new();
    let mut cases = 0;
    for finite in [false, true] {
        for layout in [
            "original_personal",
            "mixed_fixed",
            "generated_effects",
            "generated_grants",
            "generated_copies",
        ] {
            for reach in [1_000_000, 2_000_000, 4_000_000] {
                for reversed in [false, true] {
                    let label = format!(
                        "finite={finite} layout={layout} reach={reach} reversed={reversed}"
                    );
                    let (skill, definitions) = f01_candidate(layout, finite, reach, reversed);
                    let mut preoverlay = definitions.clone();
                    for definition in &mut preoverlay {
                        definition
                            .rules
                            .retain(|rule| !matches!(rule.apply, RelicRuleActionDef::Recipe(_)));
                    }
                    f01_publish(skill.clone(), preoverlay).unwrap_or_else(|error| {
                        panic!("valid pre-overlay carrier {label}: {error}")
                    });
                    let result = f01_publish(skill, definitions);
                    let actual = match result {
                        Ok(_) => "published".to_string(),
                        Err(error) => format!("{}: {error}", error.code),
                    };
                    let expected = if reach == 2_000_000 {
                        "published"
                    } else {
                        "relic_structural_invalid_reference"
                    };
                    let correct = if reach == 2_000_000 {
                        actual == "published"
                    } else {
                        actual.starts_with("relic_structural_invalid_reference:")
                            && actual.contains("selected support reach conflict")
                    };
                    println!("F01 PUBLICATION {label} preoverlay=published expected={expected} actual={actual}");
                    if !correct {
                        unexpected.push(format!("{label}: {actual}"));
                    }
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 60);
    assert!(
        unexpected.is_empty(),
        "F01 publication mismatches: {unexpected:#?}"
    );
    println!(
        "F01 PUBLICATION cases60 preoverlay60 conflicting40_typed_refusals equal20_accepted=true"
    );
}

#[test]
fn rv004_publication_work_retained_basis_and_cancellation_fail_closed_then_retry() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let make = || build(four_items(), four_rules(), &["p", "q", "r", "s"]);
    let limits = ExecutionLimits::default();
    let (accepted, usage) = control::run(
        &limits,
        &CancellationToken::default(),
        ExecutionPhase::FinalValidation,
        make,
    )
    .unwrap();
    accepted.unwrap();
    for (cap, good) in [(usage.work - 1, false), (usage.work, true)] {
        let result = control::run(
            &ExecutionLimits {
                max_work: cap,
                ..limits.clone()
            },
            &CancellationToken::default(),
            ExecutionPhase::FinalValidation,
            make,
        );
        if good {
            result.unwrap().0.unwrap();
        } else {
            assert_eq!(result.err().unwrap().code, "authoring_work_limit");
        }
    }
    let (accepted, admissions) =
        native::observe_retained_admissions(|| native::run(Default::default(), || Ok(make())));
    accepted.unwrap().unwrap();
    let peak = admissions
        .iter()
        .map(|row| row.2)
        .max()
        .expect("actual native admissions");
    assert!(peak > 0);
    for (cap, good) in [(peak - 1, false), (peak, true)] {
        let result = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: cap,
                ..Default::default()
            },
            || Ok(make()),
        );
        if good {
            result.unwrap().unwrap();
        } else {
            assert!(matches!(
                result,
                Err(crate::game::behavior::GameError::BattleRuntimeFault(
                    native::BattleRuntimeFault {
                        code: native::RuntimeFaultCode::RetainedByteLimit,
                        ..
                    }
                ))
            ));
        }
    }
    for site in ["publication_context_lookup", "publication_context_expand", "publication_equipment_prefix", "publication_weapon_witness_lookup", "publication_storage_proof", "publication_storage_identity"] {
        let make_cancel = || {
            let mut items = four_items();
            if site == "publication_weapon_witness_lookup" {
                items.push(weapon("cancellation_sword", 0xf200, WeaponArchetype::Sword));
            }
            build(items, four_rules(), &["p", "q", "r", "s"])
        };
        let result = control::with_test_cancellation(site, 2, || {
            control::run(
                &limits,
                &CancellationToken::default(),
                ExecutionPhase::FinalValidation,
                make_cancel,
            )
        });
        assert_eq!(result.err().unwrap_or_else(|| panic!("cancellation site was not visited: {site}")).code, "authoring_cancelled", "{site}");
        control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::FinalValidation,
            make_cancel,
        )
        .unwrap()
        .0
        .unwrap();
    }
    println!("RV004 publication work_exact={} cumulative_conservative_retained_basis={} one_short_rejects=true retries=true (not_live_heap_or_RSS)",usage.work,peak);
}
