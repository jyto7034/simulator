use super::*;

fn definitions() -> RelicDatabase {
    let mut relics: RelicDatabase = ron::de::from_str(RELICS).unwrap();
    for rule in &mut relics.relics[0].rules {
        if rule.selector.provider.fragment_id.is_some() {
            rule.selector.provider.form_id = Some("base".into());
        }
    }
    relics.relics[0].corruption = Some(crate::game::data::relic_data::RelicCorruptionVariant {
        effects: vec![],
        rules: vec![RelicRuleDef {
            id: "corrupt_add".into(),
            selector: StructuralSelectorDef {
                provider: ProviderPredicateDef {
                    fragment_id: Some("tb_fragment".into()),
                    ..Default::default()
                },
                effect: EffectPredicateDef {
                    op: Some(EffectOperation::Damage),
                    ..Default::default()
                },
            },
            when: RelicConditionDef::Always,
            apply: RelicRuleActionDef::Edit {
                property: RelicProperty::DamageAmount,
                operation: NumericEditDef::Add(TypedRelicValue::Hp(17)),
            },
        }],
        penalties: vec![crate::game::data::relic_data::RelicPenalty::DefenseFlat { amount: -1 }],
    });
    relics
}

fn roots(core: &GameCore) -> serde_json::Value {
    serde_json::json!({
        "world": non_battle_state_without_profile_fingerprint(core),
        "relics": core.state.relics, "fragments": core.state.skill_fragments,
        "uuid": core.state.uuid_manager,
        "continue": core.export_active_run_checkpoint().unwrap().map(|saved| saved.core_payload_json),
    })
}

fn amounts(plan: &crate::game::relic::structural::dto::RelicEffectivePlanDto) -> Vec<i64> {
    let mut values: Vec<_> = plan
        .effects
        .iter()
        .filter_map(|effect| {
            effect
                .properties
                .iter()
                .find(|property| property.property == RelicProperty::DamageAmount)
                .and_then(|property| property.final_value)
                .map(|value| value.value())
        })
        .collect();
    values.sort();
    values
}

#[test]
fn rv002_world_forms_corruption_unequip_and_queries_publish_current_prospective_plan() {
    let (mut core, player, employee) = setup_with_definitions(definitions(), true);
    let fragment = core
        .state
        .skill_fragments
        .copies()
        .find(|copy| copy.definition_id.as_str() == "tb_fragment")
        .unwrap()
        .copy_uuid;
    let relic = core.state.relics.copies().next().unwrap().copy_uuid;
    let before = roots(&core);
    let base = query(&mut core, player, employee);
    assert_eq!(amounts(&base), vec![30, 75]);
    assert_eq!(roots(&core), before);
    for action in [
        PlayerBehavior::SelectSkillFragmentForm {
            copy_uuid: fragment,
            form_id: "missing".into(),
        },
        PlayerBehavior::UnequipRelic {
            copy_uuid: Uuid::from_u128(0xdeadbeef),
        },
    ] {
        assert!(core.execute(player, action).is_err());
        assert_eq!(roots(&core), before);
    }
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute_admin_command(crate::game::world::AdminCommand::AdminSetEnkephalin { amount: 10 })
        .unwrap();
    core.execute(
        player,
        PlayerBehavior::AwakenSkillFragment {
            copy_uuid: fragment,
            form_id: "alternate".into(),
        },
    )
    .unwrap();
    core.execute(
        player,
        PlayerBehavior::SelectSkillFragmentForm {
            copy_uuid: fragment,
            form_id: "alternate".into(),
        },
    )
    .unwrap();
    assert_eq!(core.state.enkephalin.amount, 9);
    let before = roots(&core);
    let alternate = query(&mut core, player, employee);
    assert_eq!(amounts(&alternate), vec![23]);
    assert_ne!(alternate.fingerprint, base.fingerprint);
    assert_eq!(roots(&core), before);
    core.execute_admin_command(
        crate::game::world::AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 },
    )
    .unwrap();
    core.execute(player, PlayerBehavior::CorruptRelic { copy_uuid: relic })
        .unwrap();
    assert_eq!(amounts(&query(&mut core, player, employee)), vec![40]);
    assert_eq!(core.state.crafting_resources.relic_corruption_catalysts, 0);
    core.execute(player, PlayerBehavior::UnequipRelic { copy_uuid: relic })
        .unwrap();
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
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            employee_uuid: employee,
            copy_uuid: relic,
            slot: 0,
        },
    )
    .unwrap();
    let expected = query(&mut core, player, employee);
    assert_eq!(amounts(&expected), vec![40]);
    let before = roots(&core);
    for _ in 0..2 {
        assert_eq!(
            serde_json::to_value(query(&mut core, player, employee)).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
    }
    assert_eq!(roots(&core), before);
    println!("RV002 world base30_75 alternate23 corrupt40 unequipNone_reequip40 queries_readonly invalid_identity_unchanged=true");
}

#[test]
fn rv002_continue_reconstructs_structural_plan_from_current_generation_without_saved_execution() {
    let (mut core, player, employee) = setup_with_definitions(definitions(), true);
    core.save_run_checkpoint_for_test().unwrap();
    let old_plan = query(&mut core, player, employee);
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let document = String::from_utf8(saved.core_payload_json.to_vec()).unwrap();
    assert!(!document.contains(&old_plan.fingerprint));
    for field in [
        "effective_owner",
        "relic_effective",
        "execution_templates",
        "runtime_supported_prospective_plan",
    ] {
        assert!(
            !document.contains(field),
            "derived execution owner cannot be saved: {field}"
        );
    }
    let updated_skills = SkillDatabase::from_ron(
        "rv002/current_skills.ron",
        &SKILLS.replace("amount: 23", "amount: 33"),
    )
    .unwrap();
    let incompatible: crate::game::data::ValidatedGameData =
        GameDataBuilder::from_validated(&core.game_data)
            .with_skills(updated_skills)
            .build_arc()
            .into();
    assert_ne!(incompatible.identity(), core.game_data.identity());
    let mut refused = GameCore::new(incompatible, 0xdead);
    let before_refusal = roots(&refused);
    let error = refused
        .continue_from_active_run_checkpoint(
            player,
            &saved.core_payload_json,
            saved.loads_used,
            saved.run_peak_certified_power,
        )
        .unwrap_err();
    assert!(
        matches!(error, GameError::InvalidStaticData(ref message) if message == "run checkpoint content identity does not match the retained generation")
    );
    assert_eq!(roots(&refused), before_refusal);
    // Continue requires the same content identity. A fresh compatible
    // generation must reconstruct executable plans from the restored loadout.
    let current: crate::game::data::ValidatedGameData =
        GameDataBuilder::from_validated(&core.game_data)
            .build_arc()
            .into();
    assert_eq!(current.identity(), core.game_data.identity());
    assert!(!std::ptr::eq(current.database(), core.game_data.database()));
    let mut restored = GameCore::new(current.clone(), 0xdead);
    restored
        .install_authoritative_player_profile(core.state.player_profile.clone())
        .unwrap();
    restored
        .continue_from_active_run_checkpoint(
            player,
            &saved.core_payload_json,
            saved.loads_used,
            saved.run_peak_certified_power,
        )
        .unwrap();
    let rebuilt = query(&mut restored, player, employee);
    assert_eq!(amounts(&rebuilt), vec![30, 75]);
    assert_eq!(&rebuilt.generation, current.identity());
    assert_eq!(rebuilt.fingerprint, old_plan.fingerprint);
    let old_owner = core.capture_structural_battle_owner().unwrap().unwrap();
    let new_owner = restored.capture_structural_battle_owner().unwrap().unwrap();
    assert!(!Arc::ptr_eq(
        &old_owner.plans[&employee],
        &new_owner.plans[&employee]
    ));
    assert!(std::ptr::eq(
        new_owner.plans[&employee].generation.database(),
        current.database()
    ));
    assert_eq!(
        serde_json::to_value(&restored.state.relics).unwrap(),
        serde_json::to_value(&core.state.relics).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&restored.state.skill_fragments).unwrap(),
        serde_json::to_value(&core.state.skill_fragments).unwrap()
    );
    assert_eq!(
        String::from_utf8(saved.core_payload_json.to_vec()).unwrap(),
        document
    );
    let before = roots(&restored);
    query(&mut restored, player, employee);
    assert_eq!(roots(&restored), before);
    println!("RV002 continue saved_no_effective_execution=true same_identity_fresh_generation30_75=true incompatible_generation_rejected_prestate=true inventory_copy_identity_preserved=true query_readonly=true");
}

#[test]
fn rv006_same_skill_two_structural_owners_keep_paused_immediate_and_delayed_results_independent() {
    for (delay, second_corrupt) in [(0, true), (20, true), (20, false)] {
        let skills = SKILLS.replace("delay_ms: 0", &format!("delay_ms: {delay}"));
        let (mut core, player, first) = setup_with_skill_input(definitions(), false, &skills);
        let prototype_owner = core.game_data.skill_data.clone();
        let prototype = serde_json::to_vec(prototype_owner.as_ref()).unwrap();
        let second = core
            .roster()
            .unwrap()
            .combat_eligible_employee_ids()
            .into_iter()
            .find(|id| *id != first)
            .unwrap();
        let first_relic = core.state.relics.copies().next().unwrap().copy_uuid;
        let first_fragment = core
            .state
            .skill_fragments
            .copies()
            .find(|copy| copy.definition_id.as_str() == "tb_fragment")
            .unwrap()
            .copy_uuid;
        let serum = consumable_meta(
            0xb103,
            "tb_second_charge",
            ConsumableTier::Critical,
            ConsumableEffect::InitialSkillCharge { percent: 100 },
        );
        core.roster_mut()
            .unwrap()
            .get_mut(&second)
            .unwrap()
            .apply_consumable_modifier(Uuid::from_u128(0xb104), &serum);
        let second_fragment = core
            .apply_grant_effects(&[RewardEffect::GrantSkillFragment {
                fragment_id: "tb_fragment".into(),
            }])
            .unwrap()
            .skill_fragment_diffs[0]
            .copy_uuid;
        assert_ne!(first_fragment, second_fragment);
        core.execute(
            player,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid: second,
                copy_uuid: second_fragment,
            },
        )
        .unwrap();
        core.execute_admin_command(crate::game::world::AdminCommand::AdminGrantRelic {
            definition_id: "tb_relic".into(),
            count: 1,
        })
        .unwrap();
        let second_relic = core
            .state
            .relics
            .copies()
            .find(|copy| copy.copy_uuid != first_relic)
            .unwrap()
            .copy_uuid;
        core.execute(
            player,
            PlayerBehavior::EquipRelic {
                employee_uuid: second,
                copy_uuid: second_relic,
                slot: 0,
            },
        )
        .unwrap();
        core.execute_admin_command(
            crate::game::world::AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 },
        )
        .unwrap();
        if second_corrupt {
            core.execute(
                player,
                PlayerBehavior::CorruptRelic {
                    copy_uuid: second_relic,
                },
            )
            .unwrap();
        }
        assert_eq!(
            core.state.crafting_resources.relic_corruption_catalysts,
            if second_corrupt { 0 } else { 1 }
        );
        assert_eq!(
            core.state.relics.get(first_relic).unwrap().variant,
            crate::game::relic::RelicVariant::Normal
        );
        assert_eq!(
            core.state.relics.get(second_relic).unwrap().variant,
            if second_corrupt {
                crate::game::relic::RelicVariant::Corrupt
            } else {
                crate::game::relic::RelicVariant::Normal
            }
        );
        let plans = [
            query(&mut core, player, first),
            query(&mut core, player, second),
        ];
        assert_eq!(amounts(&plans[0]), vec![30, 75]);
        assert_eq!(
            amounts(&plans[1]),
            if second_corrupt {
                vec![40]
            } else {
                vec![30, 75]
            }
        );
        assert_ne!(plans[0].fingerprint, plans[1].fingerprint);
        for (index, employee) in [first, second].into_iter().enumerate() {
            assert_eq!(plans[index].employee_uuid, employee);
            assert!(plans[index]
                .rules
                .iter()
                .all(|rule| rule.contribution.copy_uuid == [first_relic, second_relic][index]));
        }
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
            panic!("combat preview")
        };
        let cells = &preview
            .deployment_zones
            .iter()
            .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
            .unwrap()
            .cells;
        core.execute(player, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        core.state
            .active_battle
            .as_mut()
            .unwrap()
            .tactical_resource
            .set_current_for_test(100);
        for (index, employee) in [first, second].into_iter().enumerate() {
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::DeployUnit {
                    employee_uuid: employee,
                    position: cells[index],
                    facing: FacingDirection::Right,
                },
            )
            .unwrap();
        }
        let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
        let actors = [first, second].map(|employee| {
            battle
                .units
                .values()
                .find(|unit| unit.source_owned_uuid == employee)
                .unwrap()
                .instance_id
        });
        for actor in actors {
            let unit = battle.units.get_mut(&actor).unwrap();
            assert_eq!(unit.skill_id.as_ref().unwrap().as_str(), "tb_strike");
            unit.stats.attack = 0;
        }
        let enemy_hp = |core: &GameCore| -> u32 {
            core.state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .units
                .values()
                .filter(|unit| unit.owner == Side::Opponent)
                .map(|unit| unit.stats.current_health)
                .sum()
        };
        let before_hp = enemy_hp(&core);
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::PauseBattle).unwrap();
        for employee in [first, second] {
            execute_current_battle_mutation(
                &mut core,
                player,
                PlayerBehavior::ActivateSkill {
                    employee_uuid: employee,
                    skill_id: "tb_strike".into(),
                    target: None,
                },
            )
            .unwrap();
        }
        assert!(core
            .advance_active_battle_for_server_tick(100)
            .unwrap()
            .is_none());
        assert_eq!(enemy_hp(&core), before_hp);
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::ResumeBattle).unwrap();
        core.advance_active_battle_for_server_tick(1).unwrap();
        if delay != 0 {
            assert_eq!(
                enemy_hp(&core),
                before_hp,
                "both delayed accepted owners remain pending"
            );
            core.advance_active_battle_for_server_tick(20).unwrap();
        }
        let expected_damage = [105, if second_corrupt { 40 } else { 105 }];
        assert_eq!(
            u64::from(before_hp - enemy_hp(&core)),
            expected_damage.iter().sum::<u64>()
        );
        let battle = &core.state.active_battle.as_ref().unwrap().battle;
        for (index, actor) in actors.into_iter().enumerate() {
            let caused = &battle.actor_fact_summary(actor).unwrap().caused;
            assert_eq!(
                caused.damage_hp_lost, expected_damage[index],
                "per-actor result cannot hide a swapped or shared graph"
            );
            let source = &caused.latest_nonzero_damage.as_ref().unwrap().source;
            assert_eq!(source.actor, Some(actor));
            assert_eq!(
                source.fragment.as_ref().unwrap().copy_uuid,
                [first_fragment, second_fragment][index]
            );
            assert_eq!(source.cast.as_ref().unwrap().skill_id.as_str(), "tb_strike");
            let binding = source.relic_effective().unwrap();
            assert_eq!(binding.plan.employee_uuid, [first, second][index]);
            assert_eq!(binding.plan.fingerprint, plans[index].fingerprint);

            // The oracle comes from the granted copies and literal authored
            // rules above, not from a prospective plan or the other actor.
            // Two normal copies intentionally have identical amounts/aliases.
            let copy = [first_relic, second_relic][index];
            let corrupt = index == 1 && second_corrupt;
            let authored = source.authored.as_ref().unwrap();
            let effect = binding.effect(&authored.site).unwrap();
            let key = if corrupt {
                EffectiveEffectKey::Original {
                    site: crate::game::ability::authoring::EffectSite {
                        action: crate::game::ability::authoring::ActionSite {
                            definition_id: "tb_strike".into(),
                            action_id: "hit".into(),
                        },
                        effect_id: "original".into(),
                    },
                }
            } else {
                assert_eq!(authored.source_file, "effective:relic_structural");
                EffectiveEffectKey::Generated {
                    relic_copy_uuid: copy,
                    grant_rule_id: "grant".into(),
                    anchor: crate::game::ability::authoring::ActionSite {
                        definition_id: "tb_strike".into(),
                        action_id: "hit".into(),
                    },
                    local_action_id: "hit".into(),
                    local_effect_id: "bonus".into(),
                }
            };
            assert_eq!(effect.key, key);
            let property = effect
                .properties
                .iter()
                .find(|property| property.property == RelicProperty::DamageAmount)
                .unwrap();
            let rules: &[&str] = if corrupt {
                &["corrupt_add"]
            } else {
                &["bonus_add", "bonus_scale"]
            };
            let variant = if corrupt {
                crate::game::relic::RelicVariant::Corrupt
            } else {
                crate::game::relic::RelicVariant::Normal
            };
            let matches_contributors =
                |rows: &[crate::game::relic::structural::PlanContribution]| {
                    rows.len() == rules.len()
                        && rules.iter().all(|rule| {
                            rows.iter()
                                .filter(|row| {
                                    row.copy_uuid == copy
                                        && row.definition_id == "tb_relic"
                                        && row.variant == variant
                                        && row.rule_id == *rule
                                })
                                .count()
                                == 1
                        })
                };
            assert!(matches_contributors(&property.contributions));
            let mut wrong = property.contributions.clone();
            wrong[0].copy_uuid = [first_relic, second_relic][1 - index];
            assert!(
                !matches_contributors(&wrong),
                "same-value other-copy contribution must fail"
            );
            wrong = property.contributions.clone();
            wrong.pop();
            assert!(
                !matches_contributors(&wrong),
                "missing contribution must fail"
            );
            wrong = property.contributions.clone();
            wrong.push(wrong[0].clone());
            assert!(
                !matches_contributors(&wrong),
                "duplicate contribution must fail"
            );
            wrong = property.contributions.clone();
            wrong[0].rule_id = "original_add".into();
            assert!(
                !matches_contributors(&wrong),
                "different authored rule must fail"
            );

            binding.validate(core.game_data.identity(), source).unwrap();
            let other = &battle
                .actor_fact_summary(actors[1 - index])
                .unwrap()
                .caused
                .latest_nonzero_damage
                .as_ref()
                .unwrap()
                .source;
            let mut mixed = source.as_ref().clone();
            mixed.fragment = other.fragment.clone();
            assert!(binding.validate(core.game_data.identity(), &mixed).is_err());
            mixed = source.as_ref().clone();
            mixed.actor = other.actor;
            assert!(binding.validate(core.game_data.identity(), &mixed).is_err());
            let mut absent_site = authored.site.clone();
            absent_site.effect_id = "missing_effect".into();
            assert!(binding.effect(&absent_site).is_none());
            assert_eq!(binding.effect(&authored.site).unwrap().key, key);
        }
        for (index, employee) in [first, second].into_iter().enumerate() {
            assert_eq!(
                serde_json::to_value(query(&mut core, player, employee)).unwrap(),
                serde_json::to_value(&plans[index]).unwrap()
            );
        }
        assert!(Arc::ptr_eq(&prototype_owner, &core.game_data.skill_data));
        assert_eq!(
            serde_json::to_vec(core.game_data.skill_data.as_ref()).unwrap(),
            prototype
        );
        println!("RV006 same_skill=tb_strike delay={delay} second_corrupt={second_corrupt} actor_damage={expected_damage:?} both_accepted_paused=true exact_generated_key_and_contributions=true mixed_missing_duplicate_sources_rejected=true");
    }
}
