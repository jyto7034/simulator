//! Relic immutable authority and real ordered damage consumers, including the
//! literal path with no numerical observer. Test values are authored probes.
use super::*;
use crate::game::{
    battle::{
        core::relic_numeric::RelicNumericOwner,
        damage::{DamageSource, DamageType},
        runtime_facts::{CommittedReinputContext, IndependentObjectOriginSnapshot},
    },
    data::relic_data::{RelicDamageEffect, RelicDatabase},
    managers::uuid_manager::UuidManager,
    relic::RelicInventory,
};

// This entire file is compiled only below value_expr_runtime::tests. These
// observers expose Weak handles and measured extents, never a strong owner.
impl BattleCore {
    pub(crate) fn u08_relic_owner_lifetime(
        &self,
    ) -> Option<(std::sync::Weak<RelicNumericOwner>, usize)> {
        self.relic_numeric_owner
            .as_ref()
            .map(|owner| (Arc::downgrade(owner), owner.retained_extent()))
    }

    pub(crate) fn u08_delayed_capture_lifetimes(
        &self,
    ) -> Vec<(std::sync::Weak<ValueExprDeliveryCapture>, usize)> {
        let mut result = Vec::new();
        let mut observe = |source: &crate::game::battle::damage::DamageSourceSnapshot| {
            if let Some(capture) = &source.value_expr_capture {
                let weak = Arc::downgrade(capture);
                if !result
                    .iter()
                    .any(|(prior, _)| std::sync::Weak::ptr_eq(prior, &weak))
                {
                    result.push((weak, capture.retained_extent()));
                }
            }
        };
        for projectile in self.active_projectiles.values() {
            observe(&projectile.source_snapshot);
        }
        for area in self.active_areas.values() {
            observe(&area.source_snapshot);
        }
        for event in self.event_queue.iter() {
            if let crate::game::battle::enums::BattleEvent::SkillProjectileImpact {
                source_snapshot,
                ..
            } = event.as_ref()
            {
                observe(source_snapshot);
            }
        }
        result
    }
}

impl ValueExprDeliveryCapture {
    pub(crate) fn u08_eval_capture_lifetimes(&self) -> Vec<(std::sync::Weak<EvalCapture>, usize)> {
        self.rows
            .iter()
            .flatten()
            .flat_map(|row| row.captures.iter())
            .map(|capture| (Arc::downgrade(capture), capture.retained_extent()))
            .collect()
    }
}

impl EvalCapture {
    pub(crate) fn u08_capture_link_lifetimes(
        &self,
    ) -> (
        std::sync::Weak<RuntimeFactSource>,
        std::sync::Weak<ContentIdentity>,
        std::sync::Weak<CompiledAmountPlan>,
        std::sync::Weak<crate::game::battle::core::relic_numeric::RelicNumericBinding>,
    ) {
        (
            Arc::downgrade(&self.source),
            Arc::downgrade(&self.generation),
            Arc::downgrade(&self.plan),
            Arc::downgrade(self.source.relic_numeric().expect("relic capture")),
        )
    }
}

fn fixture(op: &str, corrupt: bool) -> (OwnerFixture, RelicInventory) {
    let skills = ordered_skill(&format!(
        r#"(id: "damage", tags: ["one", "two"], op: {op})"#
    ));
    let mut fixture = owner_fixture(Some(skills));
    let mut relics: RelicDatabase = ron::de::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/u08/relics.ron"
    )))
    .unwrap();
    relics.relics[0].effects.push(RelicDamageEffect {
        id: "percent_second".into(),
        selector: Default::default(),
        flat: 0,
        percent: 30,
    });
    let old = ValidatedGameData::from(fixture.core.game_data.clone());
    let data = ValidatedGameData::from(
        GameDataBuilder::from_validated(&old)
            .with_relics(relics)
            .build_arc(),
    );
    fixture.core.game_data = data.database_arc();
    fixture.core.content_identity = Some(data.identity_arc());
    fixture
        .core
        .units
        .get_mut(&fixture.employee_a)
        .unwrap()
        .stats
        .attack = 100;
    fixture
        .core
        .units
        .get_mut(&fixture.employee_b)
        .unwrap()
        .stats
        .attack = 100;
    fixture
        .core
        .units
        .get_mut(&fixture.enemy)
        .unwrap()
        .stats
        .max_health = 1000;
    fixture
        .core
        .units
        .get_mut(&fixture.enemy)
        .unwrap()
        .stats
        .current_health = 1000;
    let employee = match fixture.core.units[&fixture.employee_a].source_identity {
        crate::game::battle::types::BattleUnitSourceIdentity::Employee {
            employee_uuid, ..
        } => employee_uuid,
        _ => panic!("employee source required"),
    };
    let mut inventory = RelicInventory::default();
    let copy = inventory
        .grant(
            &fixture.core.game_data.relic_data.relics[0],
            &mut UuidManager::new(8),
        )
        .unwrap();
    if corrupt {
        inventory
            .corrupt(copy, &fixture.core.game_data.relic_data)
            .unwrap();
    }
    inventory
        .equip(copy, employee, 2, &fixture.core.game_data.relic_data)
        .unwrap();
    let owner = RelicNumericOwner::capture(
        &inventory,
        fixture.core.game_data.clone(),
        fixture.core.content_identity.as_ref().unwrap().clone(),
    )
    .unwrap();
    fixture.core.install_relic_numeric_owner(owner).unwrap();
    (fixture, inventory)
}

#[test]
fn relic_literal_off_expr_snapshot_live_same_skill_two_employees_and_replay() {
    use crate::game::battle::core::commands::amount_command_observation;
    for op in ["Damage(amount: 100, damage_type: True)", "DamageExpr(amount: (moment: Snapshot, root: Const((value: 100, unit: Magnitude))), damage_type: True)", "DamageExpr(amount: (moment: Live, root: Const((value: 100, unit: Magnitude))), damage_type: True)"] {
        let (fixture, _) = fixture(op, false);
        for (actor, expected) in [(fixture.employee_a, 180), (fixture.employee_b, 100)] {
            for observe in [false, true] {
                let mut core = fixture.core.clone();
                if observe {
                    core.enable_authoring_observation_with_options(Default::default(), true);
                    core.bind_authoring_observation(crate::game::battle::core::authoring_observation::ObservationKey { entry_ordinal: 8, battle_uuid: Uuid::from_u128(0x8080) }).unwrap();
                }
                let skill = core.game_data.skill_data.get_by_id("wp02_runtime_frame").unwrap().clone();
                let entry = core.game_data.value_expr_catalog().entries().next().unwrap().1.clone();
                core.runtime_fact_source = occurrence(&mut core, actor, entry.plan());
                let action = core.runtime_fact_source.action.clone().unwrap();
                let (result, inputs) = amount_command_observation::observe(|| core.runtime_transaction(|inner| inner.execute_ordered_skill_effects(actor, &skill.id, &skill.steps[0], &[fixture.enemy], 10, 10, None, None)));
                result.unwrap();
                assert_eq!(inputs, vec![(fixture.enemy, RuntimeAmount::Damage(expected))], "{op} observer={observe} actor={actor:?}");
                assert_eq!(core.units[&fixture.enemy].stats.current_health, 1000 - expected);
                let facts = &core.runtime_facts.query(&action).unwrap().facts;
                assert_eq!(facts.latest_damage.as_ref().unwrap().requested_damage, expected);
                let source = &facts.latest_damage.as_ref().unwrap().source;
                assert_eq!(source.relic_numeric().is_some(), actor == fixture.employee_a);
                if observe {
                    let record = core.authoring_observation.as_ref().unwrap().latest_amount.as_ref().unwrap();
                    assert_eq!(record.evaluated.value.value, 100, "original typed base stays base");
                    assert_eq!(record.amount, RuntimeAmount::Damage(expected));
                    let encoded = crate::authoring::encode_record_for_test(record, crate::authoring::AmountQuery::Latest { battle: crate::authoring::AmountBattle::Active }, true).unwrap();
                    let document: serde_json::Value = serde_json::from_str(&encoded).unwrap();
                    assert_eq!(document["narrowed_base_amount"]["value"], 100);
                    if actor == fixture.employee_a { assert_eq!(document["effective_relic_amount"]["value"], 180); assert_eq!(document["relic_numeric"]["contributions"].as_array().unwrap().len(), 2); }
                    else { assert!(document.get("effective_relic_amount").is_none()); }
                }
            }
        }
    }
}

#[test]
fn relic_original_catalog_ptr_guard_and_base_overflow_are_not_rescued() {
    let (mut fixture, _) = fixture("DamageExpr(amount: (moment: Snapshot, root: Mul(value: Read(Stat(subject: Source, stat: Attack)), scalar: Const((value: 42949673, unit: Scalar)))), damage_type: True)", false);
    let core = &mut fixture.core;
    let entry = core
        .game_data
        .value_expr_catalog()
        .entries()
        .next()
        .unwrap()
        .1
        .clone();
    let source = occurrence(core, fixture.employee_a, entry.plan());
    let capture = core
        .capture_value_expr_entry(&entry, &source, Some(fixture.enemy), 1, None)
        .unwrap();
    assert_eq!(
        core.evaluate_value_expr_capture(&capture, Some(fixture.enemy), 2)
            .unwrap_err()
            .code,
        ValueExprFaultCode::Overflow
    );
    let other = GameDataBuilder::from_validated(&ValidatedGameData::from(core.game_data.clone()))
        .build_arc();
    let forged = other.value_expr_catalog().entries().next().unwrap().1;
    assert!(!Arc::ptr_eq(entry.plan(), forged.plan()));
    assert_eq!(
        core.capture_value_expr_entry(forged, &source, Some(fixture.enemy), 1, None)
            .unwrap_err()
            .code,
        ValueExprFaultCode::Unavailable
    );
    let mut changed = source.clone();
    changed.actor = Some(fixture.employee_b);
    assert!(core
        .capture_value_expr_entry(&entry, &changed, Some(fixture.enemy), 1, None)
        .is_err());
}

#[test]
fn relic_basic_once_actual_damage_preserves_attack_and_excludes_other_producers() {
    use crate::game::battle::core::damage_runtime::{CommittedDamageInput, CommittedDamagePolicy};
    let (mut fixture, _) = fixture("Damage(amount: 100, damage_type: True)", false);
    let core = &mut fixture.core;
    for source in [
        DamageSource::Ability,
        DamageSource::BuffTick,
        DamageSource::Environment,
    ] {
        let snapshot = core
            .damage_source_snapshot_template_for_unit(
                fixture.employee_a,
                source,
                DamageType::True,
                100,
                Default::default(),
                0,
                10,
                false,
            )
            .unwrap();
        assert_eq!(snapshot.base_damage, 100);
        assert!(snapshot.fact_source.relic_numeric().is_none());
    }
    let snapshot = core
        .damage_source_snapshot_template_for_unit(
            fixture.employee_a,
            DamageSource::BasicAttack,
            DamageType::True,
            100,
            Default::default(),
            0,
            10,
            false,
        )
        .unwrap();
    assert_eq!(snapshot.base_damage, 180);
    assert_eq!(snapshot.source_attack, 100);
    assert!(snapshot
        .fact_source
        .relic_numeric()
        .unwrap()
        .basic_profile_fingerprint
        .is_some());
    let retained = snapshot.try_clone_fact_source().unwrap();
    assert!(Arc::ptr_eq(
        snapshot.fact_source.relic_numeric().unwrap(),
        retained.fact_source.relic_numeric().unwrap()
    ));
    core.execute_committed_damage(CommittedDamageInput {
        target_id: fixture.enemy,
        source_snapshot: retained,
        committed_at_ms: 10,
        policy: CommittedDamagePolicy::GeneralCommand,
    })
    .unwrap();
    assert_eq!(
        core.units[&fixture.enemy].stats.current_health, 820,
        "immutable effective base is consumed once"
    );
    core.units
        .get_mut(&fixture.employee_a)
        .unwrap()
        .basic_attack
        .action = crate::game::data::abnormality_data::BasicAttackAction::Heal {
        flat: 10,
        spell_power_percent: 0,
        footprint: None,
    };
    let healing_producer = core
        .damage_source_snapshot_template_for_unit(
            fixture.employee_a,
            DamageSource::BasicAttack,
            DamageType::True,
            100,
            Default::default(),
            0,
            10,
            false,
        )
        .unwrap();
    assert_eq!(healing_producer.base_damage, 100);
    assert!(healing_producer.fact_source.relic_numeric().is_none());
}

#[test]
fn relic_source_origin_reinput_removal_preserves_binding_and_common_layout() {
    let (mut fixture, _) = fixture("Damage(amount: 100, damage_type: True)", false);
    let core = &mut fixture.core;
    let entry = core
        .game_data
        .value_expr_catalog()
        .entries()
        .next()
        .unwrap()
        .1
        .clone();
    let source = occurrence(core, fixture.employee_a, entry.plan());
    let mut source = core
        .capture_value_expr_entry(&entry, &source, Some(fixture.enemy), 1, None)
        .unwrap()
        .source()
        .clone();
    let binding = source.relic_numeric().unwrap().clone();
    let origin = Arc::new(IndependentObjectOriginSnapshot {
        object_id: fixture.employee_a,
        definition_id: crate::game::data::independent_object_data::IndependentObjectId::new(
            "probe",
        ),
        creator: fixture.employee_b,
        creation_action: None,
        creation_cast: None,
        creation_fragment: None,
        creation_equipment: None,
        creation_authored: None,
    });
    source.extended_observation = RuntimeFactSource::extended_observation(
        Some(origin.clone()),
        Some(Arc::new(CommittedReinputContext {
            activation: 1,
            committed_at_ms: 1,
            remaining_ms: 5,
        })),
    )
    .unwrap();
    source = source.with_relic_numeric(binding.clone()).unwrap();
    let original = serde_json::to_value(&source).unwrap();
    let stripped = source.without_reinput().unwrap();
    assert!(stripped.reinput().is_none());
    assert!(Arc::ptr_eq(stripped.independent_object().unwrap(), &origin));
    assert!(Arc::ptr_eq(stripped.relic_numeric().unwrap(), &binding));
    let child = source.independent_origin().unwrap();
    assert!(child.action.is_none());
    assert!(child.reinput().is_none());
    assert!(Arc::ptr_eq(child.relic_numeric().unwrap(), &binding));
    assert_eq!(
        serde_json::to_value(&source).unwrap(),
        original,
        "original immutable graph unchanged"
    );
    assert_eq!(size_of::<RuntimeFactSource>(), 136);
    println!(
        "relic source layout source={} capture={} cold={}",
        size_of::<RuntimeFactSource>(),
        size_of::<EvalCapture>(),
        size_of::<crate::game::battle::runtime_facts::ExtendedSourceObservation>()
    );
}

#[test]
fn relic_corrupt_penalty_is_final_battle_only_after_scale_and_exact_override() {
    let (fixture, _) = fixture("Damage(amount: 100, damage_type: True)", true);
    let core = &fixture.core;
    let mut draft = core.scenario.groups[0].spawns[0].draft.clone();
    let certified_before =
        crate::game::battle::stat_pipeline::persistent_stats_for_draft(&draft, &core.game_data)
            .unwrap();
    let base = draft.effective_stats(&core.game_data).unwrap();
    let result = core.relic_penalized_stats(&draft).unwrap();
    assert_eq!(result.defense, base.defense - 15);
    assert_eq!(result.magic_resist, base.magic_resist - 10);
    draft.stat_scale.defense_percent = 200;
    let scaled = draft.effective_stats(&core.game_data).unwrap();
    assert_eq!(
        core.relic_penalized_stats(&draft).unwrap().defense,
        scaled.defense - 15
    );
    draft.stat_scale.exact_override =
        Some(crate::game::battle::types::BattleUnitExactStatOverride {
            max_health: 100,
            attack: 100,
            defense: 70,
            magic_resistance: 60,
        });
    let final_stats = core.relic_penalized_stats(&draft).unwrap();
    assert_eq!((final_stats.defense, final_stats.magic_resist), (55, 50));
    assert_eq!(
        serde_json::to_value(
            crate::game::battle::stat_pipeline::persistent_stats_for_draft(&draft, &core.game_data)
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(certified_before).unwrap()
    );
}

#[test]
fn relic_released_literal_and_expr_delivery_keep_contributions_after_source_withdrawal() {
    for op in ["Damage(amount: 100, damage_type: True)", "DamageExpr(amount: (moment: Snapshot, root: Read(Stat(subject: Source, stat: Attack))), damage_type: True)"] {
        let (mut fixture, _) = fixture(op, false);
        let core = &mut fixture.core;
        let skill = core.game_data.skill_data.get_by_id("wp02_runtime_frame").unwrap().clone();
        let entry = core.game_data.value_expr_catalog().entries().next().unwrap().1.clone();
        let mut source = occurrence(core, fixture.employee_a, entry.plan());
        let action = source.action.clone().unwrap();
        source.delivery = core.runtime_facts.delivery(&action).unwrap();
        core.runtime_facts.release(&action, 0, 9).unwrap();
        let carrier = core.build_value_expr_delivery_capture(&skill.steps[0], &source, Some(&[fixture.enemy]), 10, None).unwrap().unwrap();
        let released_binding = carrier.capture_for(0, Some(fixture.enemy)).unwrap().source().relic_numeric().unwrap().clone();
        let mut template = core.damage_source_snapshot_template_for_unit(fixture.employee_a, DamageSource::Ability, DamageType::True, 0, Default::default(), 0, 10, false).unwrap();
        template.fact_source = source.clone();
        template.value_expr_capture = Some(carrier);
        core.withdraw_unit(fixture.employee_a, 11).unwrap();
        assert!(!core.units[&fixture.employee_a].lifecycle.is_active());
        // A release binding has its own immutable allocation; its producer
        // snapshot is no longer needed to consume or explain this delivery.
        core.relic_numeric_owner = None;
        core.runtime_fact_source = source;
        core.runtime_transaction(|candidate| candidate.execute_ordered_skill_effects(fixture.employee_a, &skill.id, &skill.steps[0], &[fixture.enemy], 12, 10, Some(&template), None)).unwrap();
        assert_eq!(core.units[&fixture.enemy].stats.current_health, 820);
        let facts = core.runtime_facts.query(&action).unwrap();
        assert!(Arc::ptr_eq(facts.facts.latest_damage.as_ref().unwrap().source.relic_numeric().unwrap(), &released_binding));
    }
}

// Added by the separate Astra/high test author after the Sol-authored tests
// above. These are focused TD03/04 probes, not full public-path coverage.
fn astra_selector_fixture(
    effects: &str,
    selector: crate::game::data::relic_data::RelicDamageSelector,
) -> (OwnerFixture, RelicInventory) {
    use crate::game::data::skill_fragment_data::*;
    let (mut fixture, inventory) = fixture("Damage(amount: 100, damage_type: True)", false);
    let mut relics = fixture.core.game_data.relic_data.as_ref().clone();
    for effect in &mut relics.relics[0].effects {
        effect.selector = selector.clone();
    }
    let fragments = ["astra_fragment", "astra_other_fragment"]
        .into_iter()
        .enumerate()
        .map(|(index, id)| SkillFragmentMetadata {
            id: id.into(),
            uuid: Uuid::from_u128(0xa580 + index as u128),
            name: id.into(),
            description: "Exact provider/form test input".into(),
            rarity: SkillFragmentRarity::Rare,
            equip_limit: SkillFragmentEquipLimit::OwnedCopies,
            origin: None,
            corrosion_class: Default::default(),
            sources: vec![SkillFragmentAcquisitionSource::RareReward],
            dependencies: vec![],
            compatibility: Default::default(),
            effect: SkillFragmentEffectDef::ActiveSkill {
                forms: ["base", "alternate"]
                    .into_iter()
                    .map(|form| {
                        (
                            form.into(),
                            SkillFragmentForm {
                                skill_id: "wp02_runtime_frame".into(),
                                resonance_max: 100,
                                compatibility: None,
                                bloom_recipe: None,
                            },
                        )
                    })
                    .collect(),
                dismantle_recipe: None,
                passive_guard: None,
            },
        })
        .collect();
    let old = ValidatedGameData::from(fixture.core.game_data.clone());
    let data = ValidatedGameData::from(
        GameDataBuilder::from_validated(&old)
            .with_skills(ordered_skill(effects))
            .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(fragments))
            .with_relics(relics)
            .build_arc(),
    );
    fixture.core.game_data = data.database_arc();
    fixture.core.content_identity = Some(data.identity_arc());
    let owner = RelicNumericOwner::capture(
        &inventory,
        fixture.core.game_data.clone(),
        data.identity_arc(),
    )
    .unwrap();
    fixture.core.install_relic_numeric_owner(owner).unwrap();
    (fixture, inventory)
}

#[test]
fn relic_astra_dynamic_moments_keep_release_contribution_and_immutable_replay() {
    for (moment, base, expected) in [("Snapshot", 100, 180), ("Live", 200, 330)] {
        let op = format!("DamageExpr(amount: (moment: {moment}, root: Read(Stat(subject: Source, stat: Attack))), damage_type: True)");
        let (mut fixture, _) = fixture(&op, false);
        let core = &mut fixture.core;
        core.enable_authoring_observation_with_options(Default::default(), true);
        core.bind_authoring_observation(
            crate::game::battle::core::authoring_observation::ObservationKey {
                entry_ordinal: 808,
                battle_uuid: Uuid::from_u128(0xa588),
            },
        )
        .unwrap();
        let skill = core
            .game_data
            .skill_data
            .get_by_id("wp02_runtime_frame")
            .unwrap()
            .clone();
        let entry = core
            .game_data
            .value_expr_catalog()
            .entries()
            .next()
            .unwrap()
            .1
            .clone();
        let mut source = occurrence(core, fixture.employee_a, entry.plan());
        let action = source.action.clone().unwrap();
        source.delivery = core.runtime_facts.delivery(&action).unwrap();
        core.runtime_facts.release(&action, 0, 9).unwrap();
        let carrier = core
            .build_value_expr_delivery_capture(
                &skill.steps[0],
                &source,
                Some(&[fixture.enemy]),
                10,
                None,
            )
            .unwrap()
            .unwrap();
        let binding = carrier
            .capture_for(0, Some(fixture.enemy))
            .unwrap()
            .source()
            .relic_numeric()
            .unwrap()
            .clone();
        let mut template = core
            .damage_source_snapshot_template_for_unit(
                fixture.employee_a,
                DamageSource::Ability,
                DamageType::True,
                0,
                Default::default(),
                0,
                10,
                false,
            )
            .unwrap();
        template.fact_source = source.clone();
        template.value_expr_capture = Some(carrier);
        core.units
            .get_mut(&fixture.employee_a)
            .unwrap()
            .stats
            .attack = 200;
        core.relic_numeric_owner = None;
        core.runtime_fact_source = source;
        core.runtime_transaction(|candidate| {
            candidate.execute_ordered_skill_effects(
                fixture.employee_a,
                &skill.id,
                &skill.steps[0],
                &[fixture.enemy],
                20,
                10,
                Some(&template),
                None,
            )
        })
        .unwrap();
        assert_eq!(
            core.units[&fixture.enemy].stats.current_health,
            1000 - expected,
            "{moment}"
        );
        let record = core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_amount
            .as_ref()
            .unwrap()
            .clone();
        assert_eq!(record.evaluated.value.value, base);
        assert_eq!(record.amount, RuntimeAmount::Damage(expected));
        assert!(Arc::ptr_eq(
            record.source.relic_numeric().unwrap(),
            &binding
        ));
        let query = crate::authoring::AmountQuery::Exact {
            battle_key: record.battle_key.into(),
            evaluation_ordinal: record.evaluation_ordinal,
        };
        let encoded = crate::authoring::encode_record_for_test(&record, query, true).unwrap();
        core.units.remove(&fixture.employee_a);
        core.items.clear();
        assert_eq!(
            crate::authoring::encode_record_for_test(&record, query, true).unwrap(),
            encoded,
            "replay never reads the removed producer"
        );
    }
}

#[test]
fn relic_astra_selector_requires_same_effect_and_every_exact_provider_condition() {
    use crate::game::data::relic_data::{RelicDamageSelector, RelicDamageTarget};
    for mismatch in [
        "none",
        "equipment",
        "fragment",
        "form",
        "damage_type",
        "tag",
    ] {
        let mut selector = RelicDamageSelector {
            target: RelicDamageTarget::SkillDamage,
            all_tags: vec!["one".into(), "two".into()],
            damage_type: Some(DamageType::True),
            equipment_id: Some("owner_weapon".into()),
            fragment_id: Some("astra_fragment".into()),
            form_id: Some("base".into()),
        };
        match mismatch {
            "equipment" => selector.equipment_id = Some("owner_armor".into()),
            "fragment" => selector.fragment_id = Some("astra_other_fragment".into()),
            "form" => selector.form_id = Some("alternate".into()),
            "damage_type" => selector.damage_type = Some(DamageType::Magic),
            "tag" => selector.all_tags.push("absent".into()),
            _ => {}
        }
        let (mut fixture, _) = astra_selector_fixture(
            r#"(id: "damage", tags: ["one", "two"], op: Damage(amount: 100, damage_type: True))"#,
            selector,
        );
        let core = &mut fixture.core;
        let entry = core
            .game_data
            .value_expr_catalog()
            .entries()
            .next()
            .unwrap()
            .1
            .clone();
        let provider = core
            .items
            .values()
            .find(|item| item.persistent_copy_uuid == Some(fixture.weapon_x))
            .unwrap()
            .source_snapshot();
        let fragment = Arc::new(crate::game::skill_fragment::FragmentSourceSnapshot {
            copy_uuid: Uuid::from_u128(0xa589),
            definition_id: "astra_fragment".into(),
            form_id: "base".into(),
            skill_id: "wp02_runtime_frame".into(),
        });
        core.units
            .get_mut(&fixture.employee_a)
            .unwrap()
            .fragment_source = Some(fragment.clone());
        let basis = RuntimeFactSource {
            actor: Some(fixture.employee_a),
            equipment: Some(provider),
            fragment: Some(fragment),
            authored: Some(Arc::new(entry.plan().source().clone())),
            ..Default::default()
        };
        let action = core
            .runtime_facts
            .accept(fixture.employee_a, None, basis.clone(), 1, false)
            .unwrap();
        let source = RuntimeFactSource {
            action: Some(action),
            ..basis
        };
        let capture = core
            .capture_value_expr_entry(&entry, &source, Some(fixture.enemy), 10, None)
            .unwrap();
        assert_eq!(
            core.evaluate_value_expr_capture(&capture, Some(fixture.enemy), 20)
                .unwrap()
                .amount,
            RuntimeAmount::Damage(if mismatch == "none" { 180 } else { 100 }),
            "{mismatch}"
        );
        assert_eq!(
            capture.source().relic_numeric().is_some(),
            mismatch == "none",
            "{mismatch}"
        );
        let mut forged = source.clone();
        forged.equipment.as_mut().unwrap().persistent_copy_uuid = Some(fixture.weapon_z);
        assert_eq!(
            core.capture_value_expr_entry(&entry, &forged, Some(fixture.enemy), 10, None)
                .unwrap_err()
                .code,
            ValueExprFaultCode::Unavailable,
            "provider guard precedes selector dormancy: {mismatch}"
        );
        let mut forged = source.clone();
        forged.fragment = source
            .fragment
            .as_ref()
            .map(|value| Arc::new((**value).clone()));
        assert_eq!(
            core.capture_value_expr_entry(&entry, &forged, Some(fixture.enemy), 10, None)
                .unwrap_err()
                .code,
            ValueExprFaultCode::Unavailable,
            "equal fragment metadata cannot replace accepted Arc: {mismatch}"
        );
    }
}

#[test]
fn relic_astra_split_tags_do_not_union_and_basic_has_no_skill_tags() {
    use crate::game::battle::core::commands::amount_command_observation;
    let (mut fixture, _) = astra_selector_fixture(
        r#"(id: "first", tags: ["one"], op: Damage(amount: 100, damage_type: True)),
            (id: "second", tags: ["two"], op: Damage(amount: 100, damage_type: True)),
            (id: "both", tags: ["one", "two"], op: Damage(amount: 100, damage_type: True))"#,
        crate::game::data::relic_data::RelicDamageSelector {
            all_tags: vec!["one".into(), "two".into()],
            ..Default::default()
        },
    );
    let core = &mut fixture.core;
    let skill = core
        .game_data
        .skill_data
        .get_by_id("wp02_runtime_frame")
        .unwrap()
        .clone();
    let entry = core
        .game_data
        .value_expr_catalog()
        .entries()
        .find(|(_, entry)| entry.plan().source().site.effect_id == "first")
        .unwrap()
        .1
        .clone();
    core.runtime_fact_source = occurrence(core, fixture.employee_a, entry.plan());
    let (result, inputs) = amount_command_observation::observe(|| {
        core.runtime_transaction(|candidate| {
            candidate.execute_ordered_skill_effects(
                fixture.employee_a,
                &skill.id,
                &skill.steps[0],
                &[fixture.enemy],
                10,
                10,
                None,
                None,
            )
        })
    });
    result.unwrap();
    assert_eq!(
        inputs,
        vec![
            (fixture.enemy, RuntimeAmount::Damage(100)),
            (fixture.enemy, RuntimeAmount::Damage(100)),
            (fixture.enemy, RuntimeAmount::Damage(180))
        ]
    );
    assert_eq!(core.units[&fixture.enemy].stats.current_health, 620);
    core.runtime_fact_source = Default::default();
    let basic = core
        .damage_source_snapshot_template_for_unit(
            fixture.employee_a,
            DamageSource::BasicAttack,
            DamageType::True,
            100,
            Default::default(),
            0,
            20,
            false,
        )
        .unwrap();
    assert_eq!(basic.base_damage, 100);
    assert!(basic.fact_source.relic_numeric().is_none());
}

#[test]
fn relic_astra_valid_base_final_narrow_failure_rolls_back_earlier_damage() {
    // Legacy Damage.amount is i32. The typed expression admits u32::MAX as a
    // valid Damage base; only the subsequent relic composition must overflow.
    let (mut fixture, _) = astra_selector_fixture(
        r#"(id: "first", op: Damage(amount: 1, damage_type: True)),
            (id: "overflow", op: DamageExpr(amount: (moment: Snapshot, root: Const((value: 4294967295, unit: Magnitude))), damage_type: True))"#,
        Default::default(),
    );
    let core = &mut fixture.core;
    let skill = core
        .game_data
        .skill_data
        .get_by_id("wp02_runtime_frame")
        .unwrap()
        .clone();
    let first = core
        .game_data
        .value_expr_catalog()
        .entries()
        .find(|(_, entry)| entry.plan().source().site.effect_id == "first")
        .unwrap()
        .1
        .clone();
    core.runtime_fact_source = occurrence(core, fixture.employee_a, first.plan());
    let before = (
        core.transaction_debug_snapshot(),
        core.event_queue_debug_snapshot(),
        format!("{:?}", core.runtime_facts),
    );
    assert!(core
        .runtime_transaction(|candidate| candidate.execute_ordered_skill_effects(
            fixture.employee_a,
            &skill.id,
            &skill.steps[0],
            &[fixture.enemy],
            10,
            10,
            None,
            None
        ))
        .is_err());
    assert_eq!(
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            format!("{:?}", core.runtime_facts)
        ),
        before
    );
    assert_eq!(core.units[&fixture.enemy].stats.current_health, 1000);
    let overflow = core
        .game_data
        .value_expr_catalog()
        .entries()
        .find(|(_, entry)| entry.plan().source().site.effect_id == "overflow")
        .unwrap()
        .1
        .clone();
    let source = occurrence(core, fixture.employee_a, overflow.plan());
    let capture = core
        .capture_value_expr_entry(&overflow, &source, Some(fixture.enemy), 10, None)
        .unwrap();
    assert_eq!(
        core.evaluate_value_expr_capture(&capture, Some(fixture.enemy), 20)
            .unwrap_err()
            .code,
        ValueExprFaultCode::Overflow
    );
    core.relic_numeric_owner = None;
    let base_only = core
        .capture_value_expr_entry(&overflow, &source, Some(fixture.enemy), 10, None)
        .unwrap();
    assert_eq!(
        core.evaluate_value_expr_capture(&base_only, Some(fixture.enemy), 20)
            .unwrap()
            .amount,
        RuntimeAmount::Damage(u32::MAX)
    );
}

fn rebind_fixture() -> (
    OwnerFixture,
    Arc<ValueExprDeliveryCapture>,
    RuntimeFactSource,
) {
    use crate::game::data::relic_data::{RelicDamageSelector, RelicDamageTarget};
    let (mut fixture, _) = astra_selector_fixture(
        r#"(id: "selected", tags: ["selected"], op: DamageExpr(amount: (moment: Snapshot, root: Const((value: 100, unit: Magnitude))), damage_type: True)),
            (id: "unselected", tags: ["other"], op: DamageExpr(amount: (moment: Snapshot, root: Const((value: 100, unit: Magnitude))), damage_type: True))"#,
        RelicDamageSelector {
            target: RelicDamageTarget::SkillDamage,
            all_tags: vec!["selected".into()],
            ..Default::default()
        },
    );
    let core = &mut fixture.core;
    let skill = core
        .game_data
        .skill_data
        .get_by_id("wp02_runtime_frame")
        .unwrap()
        .clone();
    let entry = core
        .game_data
        .value_expr_catalog()
        .entries()
        .find(|(_, entry)| entry.plan().source().site.effect_id == "selected")
        .unwrap()
        .1
        .clone();
    let mut release = RuntimeFactSource {
        actor: Some(fixture.employee_a),
        authored: Some(Arc::new(entry.plan().source().clone())),
        ..Default::default()
    };
    let origin = |name: &str, actor| {
        Arc::new(IndependentObjectOriginSnapshot {
            object_id: actor,
            definition_id: crate::game::data::independent_object_data::IndependentObjectId::new(
                name,
            ),
            creator: actor,
            creation_action: None,
            creation_cast: None,
            creation_fragment: None,
            creation_equipment: None,
            creation_authored: None,
        })
    };
    release.extended_observation = RuntimeFactSource::extended_observation(
        Some(origin("release_origin", fixture.employee_a)),
        Some(Arc::new(CommittedReinputContext {
            activation: 1,
            committed_at_ms: 1,
            remaining_ms: 7,
        })),
    )
    .unwrap();
    core.runtime_facts.resources = Some(
        crate::game::resource::runtime::RuntimeResources::new(Arc::new(
            crate::game::resource::ResourceCatalog { resources: vec![] },
        ))
        .map(Box::new)
        .unwrap(),
    );
    let action = core
        .runtime_facts
        .accept(fixture.employee_a, None, release.clone(), 1, false)
        .unwrap();
    release.action = Some(action.clone());
    release.delivery = core.runtime_facts.delivery(&action).unwrap();
    core.runtime_facts.release(&action, 0, 9).unwrap();
    let carrier = core
        .build_value_expr_delivery_capture(
            &skill.steps[0],
            &release,
            Some(&[fixture.enemy]),
            10,
            None,
        )
        .unwrap()
        .unwrap();
    assert!(carrier
        .capture_for(0, Some(fixture.enemy))
        .unwrap()
        .source
        .relic_numeric()
        .is_some());
    assert!(carrier
        .capture_for(1, Some(fixture.enemy))
        .unwrap()
        .source
        .relic_numeric()
        .is_none());
    let mut incoming = release.clone();
    incoming.delivery = core.runtime_facts.delivery(&action).unwrap();
    assert_ne!(incoming.delivery, release.delivery);
    incoming.extended_observation = RuntimeFactSource::extended_observation(
        Some(origin("delivery_origin", fixture.employee_b)),
        Some(Arc::new(CommittedReinputContext {
            activation: 2,
            committed_at_ms: 20,
            remaining_ms: 99,
        })),
    )
    .unwrap();
    incoming.authored = None;
    incoming.reaction = true;
    incoming.use_control = true;
    incoming.allows_finite_children = true;
    incoming.flow = Some(Arc::new(
        crate::game::battle::runtime_facts::FlowOccurrence {
            node_index: 8,
            occurrence: 9,
        },
    ));
    incoming.equipment = Some(
        core.items
            .values()
            .find(|item| item.persistent_copy_uuid == Some(fixture.weapon_x))
            .unwrap()
            .source_snapshot(),
    );
    incoming.fragment = Some(Arc::new(
        crate::game::skill_fragment::FragmentSourceSnapshot {
            copy_uuid: Uuid::from_u128(0x8e208),
            definition_id: "astra_fragment".into(),
            form_id: "base".into(),
            skill_id: skill.id.clone(),
        },
    ));
    let (context, delivery) = core
        .runtime_facts
        .resources
        .as_mut()
        .unwrap()
        .finite(None)
        .unwrap();
    incoming.resource_capabilities =
        RuntimeFactSource::resource_capabilities(Some(context), Some(delivery)).unwrap();
    (fixture, carrier, incoming)
}

#[test]
fn relic_rebind_exact_numeric_slot_preserves_incoming_authority_and_cold_provenance() {
    let (fixture, carrier, incoming) = rebind_fixture();
    let target = Some(fixture.enemy);
    let binding = carrier
        .capture_for(0, target)
        .unwrap()
        .source
        .relic_numeric()
        .unwrap()
        .clone();
    let original_carrier = format!("{carrier:?}");
    for incoming_numeric in [false, true] {
        let incoming = if incoming_numeric {
            incoming.with_relic_numeric(binding.clone()).unwrap()
        } else {
            incoming.clone()
        };
        let original_incoming = incoming.clone();
        let rebound = carrier.rebind_consumption_source(&incoming).unwrap();
        assert_eq!(rebound.action, carrier.action);
        assert_eq!(rebound.delivery, incoming.delivery);
        assert!(Arc::ptr_eq(&rebound.effect_ids, &carrier.effect_ids));
        assert!(Arc::ptr_eq(&rebound.resource_ids, &carrier.resource_ids));
        for index in 0..2 {
            let old = carrier.capture_for(index, target).unwrap();
            let new = rebound.capture_for(index, target).unwrap();
            let mut incoming_authority = incoming.clone();
            incoming_authority.extended_observation = None;
            incoming_authority.authored = None;
            let mut rebound_authority = new.source.as_ref().clone();
            rebound_authority.extended_observation = None;
            rebound_authority.authored = None;
            assert_eq!(
                rebound_authority, incoming_authority,
                "all execution/provider/resource slots stay incoming"
            );
            assert!(Arc::ptr_eq(
                new.source.independent_object().unwrap(),
                incoming.independent_object().unwrap()
            ));
            assert!(Arc::ptr_eq(
                new.source.reinput().unwrap(),
                incoming.reinput().unwrap()
            ));
            assert!(!Arc::ptr_eq(
                new.source.independent_object().unwrap(),
                old.source.independent_object().unwrap()
            ));
            assert!(Arc::ptr_eq(
                new.source.authored.as_ref().unwrap(),
                old.source.authored.as_ref().unwrap()
            ));
            match (old.source.relic_numeric(), new.source.relic_numeric()) {
                (Some(old), Some(new)) => assert!(Arc::ptr_eq(old, new)),
                (None, None) => {}
                _ => panic!("captured numeric slot must survive exactly, including None"),
            }
            assert!(Arc::ptr_eq(&new.generation, &old.generation));
            assert!(Arc::ptr_eq(&new.plan, &old.plan));
            assert_eq!(new.site, old.site);
            assert_eq!(new.fingerprint, old.fingerprint);
            assert_eq!(new.kind, old.kind);
            assert_eq!(new.target_at_release, old.target_at_release);
            assert_eq!(new.release_time_ms, old.release_time_ms);
            assert_eq!(new.deadline_ms, old.deadline_ms);
            assert_eq!(
                format!("{:?}", new.snapshot_frame),
                format!("{:?}", old.snapshot_frame)
            );
            assert_eq!(
                new.retained_extent(),
                capture_extent(&new.plan, &new.source, true).unwrap()
            );
        }
        assert_eq!(incoming, original_incoming);
        assert_eq!(format!("{carrier:?}"), original_carrier);
    }
}

#[test]
fn relic_rebind_native_retained_outer_and_wrapper_boundaries_are_atomic() {
    use crate::game::battle::runtime_control::{
        self as native, RuntimeFactLimits, RuntimeFaultCode,
    };
    let (fixture, carrier, incoming) = rebind_fixture();
    let binding = carrier
        .capture_for(0, Some(fixture.enemy))
        .unwrap()
        .source
        .relic_numeric()
        .unwrap()
        .clone();
    for incoming_numeric in [false, true] {
        let incoming = if incoming_numeric {
            incoming.with_relic_numeric(binding.clone()).unwrap()
        } else {
            incoming.clone()
        };
        let original_incoming = incoming.clone();
        let original_carrier = format!("{carrier:?}");
        let (result, admissions) = native::observe_retained_admissions(|| {
            native::run(Default::default(), || {
                carrier.rebind_consumption_source(&incoming)
            })
        });
        result.unwrap();
        assert_eq!(admissions[0].0, "value_expr_carrier_rebind");
        assert_eq!(admissions[1].0, "relic_source_observation");
        assert_eq!(admissions.len(), if incoming_numeric { 3 } else { 2 });
        if incoming_numeric {
            assert_eq!(admissions[2].0, "extended_source_observation");
        }
        // The observed native ceiling proves enforcement/order, not an
        // independent allocation-size formula or a new accepted budget.
        let required = admissions.last().unwrap().2;
        let (result, exact) = native::observe_retained_admissions(|| {
            native::run(
                RuntimeFactLimits {
                    max_retained_bytes: required,
                    ..Default::default()
                },
                || carrier.rebind_consumption_source(&incoming),
            )
        });
        result.unwrap();
        assert_eq!(exact, admissions);
        for refused_index in 0..admissions.len() {
            let (site, _, threshold) = admissions[refused_index];
            let cap = threshold - 1;
            let (result, prefix) = native::observe_retained_admissions(|| {
                native::run(
                    RuntimeFactLimits {
                        max_retained_bytes: cap,
                        ..Default::default()
                    },
                    || carrier.rebind_consumption_source(&incoming),
                )
            });
            let GameError::BattleRuntimeFault(fault) = result.unwrap_err() else {
                panic!("typed retained refusal required")
            };
            assert_eq!(fault.code, RuntimeFaultCode::RetainedByteLimit);
            assert_eq!(fault.site, site);
            assert_eq!(fault.limit, Some(cap as u64));
            assert_eq!(fault.requested, Some(threshold as u64));
            assert_eq!(prefix, admissions[..refused_index]);
            assert_eq!(incoming, original_incoming);
            assert_eq!(format!("{carrier:?}"), original_carrier);
        }
        println!("relic rebind incoming_numeric={incoming_numeric} admissions={admissions:?} exact={required}");
    }
}
