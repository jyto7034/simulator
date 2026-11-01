//! Complete builder publication, independently chosen legal co-equip controls.
use super::*;

#[test]
fn tb002_publication_checks_legal_coequip_and_respects_inventory_exclusion() {
    let (core, _, _) = setup();
    let mut first = core.game_data.relic_data.relics[0].clone();
    first.rules = vec![RelicRuleDef {
        id: "replace".into(),
        selector: StructuralSelectorDef {
            provider: ProviderPredicateDef {
                fragment_id: Some("tb_fragment".into()),
                ..Default::default()
            },
            effect: EffectPredicateDef {
                all_tags: vec!["original".into()],
                ..Default::default()
            },
        },
        when: RelicConditionDef::Always,
        apply: RelicRuleActionDef::Edit {
            property: RelicProperty::DamageAmount,
            operation: NumericEditDef::Replace(TypedRelicValue::Hp(31)),
        },
    }];
    let mut second = first.clone();
    second.id = "tb_other_relic".into();
    second.uuid = Uuid::from_u128(0xb301);
    second.rules[0].apply = RelicRuleActionDef::Edit {
        property: RelicProperty::DamageAmount,
        operation: NumericEditDef::Replace(TypedRelicValue::Hp(32)),
    };
    let original_data = serde_json::to_vec(&core.game_data.relic_data).unwrap();
    let build = |a, b| {
        GameDataBuilder::from_validated(&core.game_data)
            .with_relics(RelicDatabase { relics: vec![a, b] })
            .build_with_runtime_safety(
                crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
            )
    };
    let error = build(first.clone(), second.clone())
        .err()
        .expect("distinct legal copies can coexist and disagree on one Replace");
    assert_eq!(error.code, "relic_structural_replace_conflict");
    first.incompatible_with = vec![second.id.clone()];
    second.incompatible_with = vec![first.id.clone()];
    let accepted =
        build(first, second).expect("inventory excludes every conflicting joint loadout");
    assert_eq!(accepted.relic_data.relics.len(), 2);
    assert_eq!(
        serde_json::to_vec(&core.game_data.relic_data).unwrap(),
        original_data
    );
    println!("TB002 publication legal_coequip=replace_conflict inventory_excluded_pair=accepted base_unchanged=true");
}

#[test]
fn tb002_publication_checks_one_active_form_not_an_impossible_union() {
    let (core, _, _) = setup();
    let mut fragments = core.game_data.skill_fragment_data.fragments.clone();
    let fragment = fragments
        .iter_mut()
        .find(|f| f.id.as_str() == "tb_fragment")
        .unwrap();
    let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &mut fragment.effect else {
        panic!("active fixture")
    };
    forms.insert("alternate".into(), forms["base"].clone());
    let mut relic = core.game_data.relic_data.relics[0].clone();
    relic.rules = [("base", 31), ("alternate", 32)]
        .into_iter()
        .map(|(form, amount)| RelicRuleDef {
            id: form.into(),
            selector: StructuralSelectorDef {
                provider: ProviderPredicateDef {
                    fragment_id: Some("tb_fragment".into()),
                    form_id: Some(form.into()),
                    ..Default::default()
                },
                effect: EffectPredicateDef {
                    all_tags: vec!["original".into()],
                    ..Default::default()
                },
            },
            when: RelicConditionDef::Always,
            apply: RelicRuleActionDef::Edit {
                property: RelicProperty::DamageAmount,
                operation: NumericEditDef::Replace(TypedRelicValue::Hp(amount)),
            },
        })
        .collect();
    let build = |relic| {
        GameDataBuilder::from_validated(&core.game_data)
            .with_skill_fragments(SkillFragmentDatabase::new(fragments.clone()))
            .with_relics(RelicDatabase {
                relics: vec![relic],
            })
            .build_with_runtime_safety(
                crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
            )
    };
    build(relic.clone()).expect("only one active form is legal at once");
    for rule in &mut relic.rules {
        rule.selector.provider.form_id = None;
    }
    assert_eq!(
        build(relic).err().unwrap().code,
        "relic_structural_replace_conflict"
    );
    println!(
        "TB002 publication mutually_exclusive_forms=accepted both_rules_same_form=replace_conflict"
    );
}

#[test]
fn tb002_legacy_only_query_is_none_and_rejected_equipping_preserves_roots() {
    let (prepared, _, _) = setup();
    let mut legacy = prepared.game_data.relic_data.relics[0].clone();
    legacy.rules.clear();
    legacy.effects = vec![crate::game::data::relic_data::RelicDamageEffect {
        id: "legacy_damage".into(),
        selector: Default::default(),
        flat: 2,
        percent: 0,
    }];
    let data = GameDataBuilder::from_validated(&prepared.game_data)
        .with_relics(RelicDatabase {
            relics: vec![legacy],
        })
        .build_arc();
    let mut core = GameCore::new(data, 0xb320);
    let player = Uuid::from_u128(0xb320);
    start_new_game_with_default_starters(&mut core, player);
    let employee = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.execute_admin_command(crate::game::world::AdminCommand::AdminGrantRelic {
        definition_id: "tb_relic".into(),
        count: 1,
    })
    .unwrap();
    let copy = core.state.relics.copies().next().unwrap().copy_uuid;
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            copy_uuid: copy,
            employee_uuid: employee,
            slot: 0,
        },
    )
    .unwrap();
    let roots = |core: &GameCore| {
        json!({"world": non_battle_state_without_profile_fingerprint(core),
        "relics": core.state.relics, "uuid": core.state.uuid_manager,
        "continue": core.export_active_run_checkpoint().unwrap().map(|v| v.core_payload_json)})
    };
    let before = roots(&core);
    for _ in 0..2 {
        assert!(matches!(
            core.execute(
                player,
                PlayerBehavior::RequestRelicEffectivePlan {
                    employee_uuid: employee
                }
            )
            .unwrap(),
            BehaviorResult::RelicEffectivePlan { result: None }
        ));
        assert!(core.capture_structural_battle_owner().unwrap().is_none());
        assert_eq!(roots(&core), before);
    }
    assert!(core
        .execute(
            player,
            PlayerBehavior::EquipRelic {
                copy_uuid: Uuid::from_u128(0xb3ffff),
                employee_uuid: employee,
                slot: 1
            }
        )
        .is_err());
    assert_eq!(roots(&core), before);
    assert!(matches!(
        core.execute(
            player,
            PlayerBehavior::RequestRelicEffectivePlan {
                employee_uuid: Uuid::from_u128(0xb3ffff)
            }
        ),
        Err(GameError::UnitNotFound)
    ));
    assert_eq!(roots(&core), before);
    println!("TB002 legacy plan=None owner=None repeat_query_readonly=true rejected_equip_query_roots_unchanged=true");
}

#[test]
fn tb002_duplicate_trigger_bindings_share_one_provider_but_keep_runtime_dispatches() {
    use crate::game::ability::{AbilityActivationBinding, AbilityActivationDef};
    use crate::game::relic::structural::ProspectiveProvider;
    use crate::game::stats::TriggerType;
    let (prepared, _, _) = setup();
    let mut item = equipment_meta(0xb330, "tb_trigger_item", EquipmentType::Accessory);
    item.ability_activations = [
        TriggerType::OnAttack,
        TriggerType::OnAttack,
        TriggerType::OnHit,
    ]
    .into_iter()
    .map(|trigger| AbilityActivationBinding {
        ability_id: "tb_strike".into(),
        activation: AbilityActivationDef::TriggerProc {
            trigger,
            proc_chance_percent: 100,
            internal_cooldown_ms: 0,
            max_triggers_per_battle: None,
        },
    })
    .collect();
    let mut equipment = prepared.game_data.equipment_data.items.clone();
    equipment.push(item);
    let mut relic = prepared.game_data.relic_data.relics[0].clone();
    for rule in &mut relic.rules {
        rule.selector.provider.fragment_id = None;
        rule.selector.provider.equipment_id = Some("tb_trigger_item".into());
    }
    let data = GameDataBuilder::from_validated(&prepared.game_data)
        .with_equipment_data(Arc::new(crate::game::data::equipment_data::EquipmentDatabase::with_all(
            equipment,
            prepared.game_data.equipment_data.materials.clone(),
            prepared.game_data.equipment_data.dismantle_recipes.clone(),
            prepared.game_data.equipment_data.enhancement_recipes.clone(),
        )))
        .with_relics(RelicDatabase {
            relics: vec![relic],
        })
        .build_arc();
    let mut core = GameCore::new(data, 0xb331);
    let player = Uuid::from_u128(0xb331);
    start_new_game_with_default_starters(&mut core, player);
    let employee = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.execute_admin_command(crate::game::world::AdminCommand::AdminGrantEquipment {
        definition_id: "tb_trigger_item".into(),
        count: 2,
    })
    .unwrap();
    let mut copies: Vec<_> = core
        .state
        .inventory
        .equipments
        .iter()
        .filter(|item| item.meta.id == "tb_trigger_item")
        .map(|item| item.instance_uuid)
        .collect();
    copies.sort();
    core.execute_admin_command(crate::game::world::AdminCommand::AdminGrantRelic {
        definition_id: "tb_relic".into(),
        count: 1,
    })
    .unwrap();
    let relic_copy = core.state.relics.copies().next().unwrap().copy_uuid;
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            copy_uuid: relic_copy,
            employee_uuid: employee,
            slot: 0,
        },
    )
    .unwrap();
    for copy in &copies {
        core.execute(
            player,
            PlayerBehavior::EquipItem {
                item_uuid: *copy,
                target_unit: employee,
            },
        )
        .unwrap();
        let before = non_battle_state_without_profile_fingerprint(&core);
        let plan = query(&mut core, player, employee);
        assert_eq!(
            plan.providers.len(),
            1,
            "one exact copy/definition/skill, even with three bindings"
        );
        assert!(
            matches!(&plan.providers[0], ProspectiveProvider::Equipment { copy_uuid, .. } if copy_uuid == copy)
        );
        assert_eq!(plan.effects.len(), 2);
        assert_eq!(
            plan.effects
                .iter()
                .filter(|e| matches!(e.key, EffectiveEffectKey::Generated { .. }))
                .count(),
            1
        );
        assert_eq!(non_battle_state_without_profile_fingerprint(&core), before);
    }
    assert_eq!(
        core.game_data
            .equipment_data
            .get_by_id("tb_trigger_item")
            .unwrap()
            .ability_activations
            .len(),
        3
    );
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        MapNodePayload::Encounter {
            encounter_id: Some("defense_encounter".into()),
        },
    );
    let preview = core
        .execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    let BehaviorResult::NodePreview {
        combat_preview: Some(preview),
        ..
    } = preview
    else {
        panic!("preview")
    };
    let position = preview
        .deployment_zones
        .iter()
        .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
        .unwrap()
        .cells[0];
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: employee,
            position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
    let actor = battle
        .units
        .values()
        .find(|u| u.source_owned_uuid == employee)
        .unwrap()
        .instance_id;
    assert_eq!(
        crate::game::battle::core::relic_structural_tests::dispatched_activation_counts(
            battle, actor
        ),
        [2, 1]
    );
    println!("TB002 equipment bindings=3 prospective_providers=1 generated=1 replacement_copy_distinct=true native_dispatch_on_attack=2 on_hit=1");
}
