//! Native ownership/admission probes. The prepared fixture generation and
//! direct owner installation are separate from normal World publication proof.
use super::*;
use crate::game::{
    battle::runtime_control as native,
    behavior::GameError,
    data::{
        equipment_data::{EquipmentMetadata, EquipmentType},
        relic_data::RelicDatabase,
        skill_data::SkillDatabase,
        skill_fragment_data::SkillFragmentDatabase,
        GameDataBuilder, ValidatedGameData,
    },
    managers::uuid_manager::UuidManager,
    relic::{
        structural::{
            compile_effective_plan, EffectivePlanInput, EmployeeEffectivePlan, ProspectiveProvider,
        },
        RelicInventory,
    },
    skill_fragment::FragmentSourceSnapshot,
};
use std::collections::BTreeMap;

pub(crate) mod flow_cases;

pub(crate) fn prepared_plan() -> (ValidatedGameData, Arc<EmployeeEffectivePlan>) {
    let skills = SkillDatabase::from_ron("tb002/native_skill.ron", r#"SkillDatabase(skills: [SkillDef(
        id: "tb_native", name: "native retained fixture", kind: Untargeted,
        use_lifetime: Action, focus_time_ms: 50,
        cast_targeting: Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: false),
        flow: Some(Action(id: "original_action", step_id: "restore", wait: EffectsSettled)),
        steps: [SkillStepDef(id: "restore", delay_ms: 0, target: SelfUnit, delivery: Instant,
            effects: [(id: "original", op: Heal(amount: 11))])]
    )])"#).unwrap();
    let fragments: SkillFragmentDatabase = ron::de::from_str(
        r#"SkillFragmentDatabase(fragments: [SkillFragmentMetadata(
        id: "tb_native_fragment", uuid: "00000000-0000-0000-0000-00000000b201",
        name: "native fragment", description: "native fixture", rarity: Common,
        equip_limit: OwnedCopies, sources: [RareReward], corrosion_class: Ordinary,
        effect: ActiveSkill(forms: {"base": (skill_id: "tb_native", resonance_max: 100)})
    )])"#,
    )
    .unwrap();
    let relics: RelicDatabase = ron::de::from_str(r#"(relics: [(
        id: "tb_native_relic", uuid: "00000000-0000-0000-0000-00000000b202",
        name: "native relic", description: "one finite restore", rarity: "probe",
        allow_duplicates: false, effects: [], dismantle_equipment_dust: 1,
        rules: [(id: "restore_once", selector: (provider: (fragment_id: Some("tb_native_fragment"))),
            apply: Grant(InlineEffects(action_id: "restore", effects: [(id: "bonus", op: Heal(amount: 37))])))]
    )])"#).unwrap();
    let weapon = EquipmentMetadata {
        content_family: None,
        id: "tb_native_weapon".into(),
        uuid: Uuid::from_u128(0xb203),
        name: "native weapon".into(),
        equipment_type: EquipmentType::Weapon,
        rarity: crate::game::enums::RiskLevel::ZAYIN,
        price: 0,
        allow_duplicate_equip: false,
        bound: false,
        cannot_unequip_reason: "bound".into(),
        triggered_effects: Default::default(),
        ability_activations: vec![],
        weapon_profile: Some(Default::default()),
        event_check_modifiers: vec![],
        event_choice_keys: vec![],
    };
    let database = GameDataBuilder::empty()
        .with_skills(skills)
        .with_skill_fragments(fragments)
        .with_equipment(vec![weapon])
        .with_relics(relics)
        .build_with_runtime_safety(crate::runtime_safety::RuntimeSafetySettings::embedded_shared())
        .unwrap();
    // Explicit native fixture generation. This does not claim external loader
    // or World acquisition/Continue acceptance.
    let data: ValidatedGameData = Arc::new(database).into();
    let employee = Uuid::from_u128(0xb204);
    let mut inventory = RelicInventory::default();
    let copy = inventory
        .grant(&data.relic_data.relics[0], &mut UuidManager::new(0xb205))
        .unwrap();
    inventory
        .equip(copy, employee, 0, &data.relic_data)
        .unwrap();
    let providers = [ProspectiveProvider::Fragment(FragmentSourceSnapshot {
        copy_uuid: Uuid::from_u128(0xb206),
        definition_id: "tb_native_fragment".into(),
        form_id: "base".into(),
        skill_id: "tb_native".into(),
    })];
    let plan = compile_effective_plan(EffectivePlanInput {
        data: &data,
        employee_uuid: employee,
        inventory: &inventory,
        providers: &providers,
    })
    .unwrap()
    .unwrap();
    (data, plan)
}

#[test]
fn tb002_native_owner_install_exact_admission_refuses_one_short_and_retries() {
    let (data, plan) = prepared_plan();
    let owner = effective_skill::EffectiveBattleOwner::new(BTreeMap::from([(
        plan.employee_uuid,
        plan.clone(),
    )]))
    .unwrap()
    .unwrap();
    let mut core =
        BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), data.clone(), 0xb207);
    let before = core.retained_runtime_fact_extent().unwrap();
    let state = core.transaction_debug_snapshot();
    let (result, observations) = native::observe_retained_admissions(|| {
        native::run(Default::default(), || {
            native::seed_retained_extent(before)?;
            core.set_effective_owner(Some(owner.clone()))
        })
    });
    result.unwrap();
    let (_, previous, boundary) = observations
        .iter()
        .copied()
        .find(|(site, _, _)| *site == "effective_battle_binding")
        .unwrap();
    assert_eq!(previous, before);
    assert!(boundary > before + std::mem::size_of::<EmployeeEffectivePlan>());
    assert!(boundary < native::RuntimeFactLimits::default().max_retained_bytes);
    assert!(core.retained_runtime_fact_extent().unwrap() >= before + plan.retained_extent());
    core.set_effective_owner(None).unwrap();
    assert_eq!(core.retained_runtime_fact_extent().unwrap(), before);
    for (cap, accepted) in [(boundary - 1, false), (boundary, true)] {
        let outcome = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: cap,
                ..Default::default()
            },
            || {
                native::seed_retained_extent(before)?;
                core.set_effective_owner(Some(owner.clone()))
            },
        );
        if accepted {
            outcome.unwrap();
            assert!(core.effective_owner.is_some());
        } else {
            assert!(matches!(
                outcome,
                Err(GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "effective_battle_binding",
                    ..
                }))
            ));
            assert!(core.effective_owner.is_none());
            assert_eq!(core.retained_runtime_fact_extent().unwrap(), before);
            assert_eq!(core.transaction_debug_snapshot(), state);
        }
    }
    core.set_effective_owner(None).unwrap();
    assert_eq!(core.retained_runtime_fact_extent().unwrap(), before);
    println!("TB002 native owner before={before} install_exact={boundary} one_short={} rollback=true retry=true", boundary - 1);
}

#[test]
fn tb002_native_pending_retains_plan_clone_boundary_and_strong_data_is_not_authority() {
    use crate::game::{
        ability::SkillActivationMode,
        battle::{
            tile_range::FacingDirection,
            types::{
                BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass,
                EmployeeBattleStartCondition, TacticalTileKind, UnitCombatProfile,
            },
        },
        enums::Tier,
    };
    let (data, plan) = prepared_plan();
    let weak = Arc::downgrade(&plan);
    let owner = effective_skill::EffectiveBattleOwner::new(BTreeMap::from([(
        plan.employee_uuid,
        plan.clone(),
    )]))
    .unwrap()
    .unwrap();
    let mut scenario = BattleScenario::empty((4, 4));
    scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(1000);
    let mut core = BattleCore::new_from_scenario(scenario, data.clone(), 0xb210);
    core.set_effective_owner(Some(owner.clone())).unwrap();
    let mut execution = core.start_battle_execution().unwrap();
    let mut profile = UnitCombatProfile::for_employee(1000, 0, 0, 0, 100);
    profile.weapon_profile = Some(Default::default());
    profile.skill_id = Some("tb_native".into());
    profile.skill_activation_mode = SkillActivationMode::Manual;
    profile.resonance.start = 100;
    profile.resonance.max = 100;
    let ProspectiveProvider::Fragment(fragment) = &plan.providers[0] else {
        panic!("fragment fixture")
    };
    profile.fragment_source = Some(fragment.clone());
    let draft = BattleUnitDraft {
        owned_uuid: plan.employee_uuid,
        source: BattleUnitSource::Employee(profile),
        threat_class: BattleUnitThreatClass::Normal,
        level: Tier::I,
        stat_scale: Default::default(),
        growth_stacks: Default::default(),
        equipment_layout: Some(crate::game::resources::item_slot::SlotLayoutKind::ByType),
        equipped_items: vec![crate::game::battle::types::BattleEquipmentBinding {
            weapon_stage: Default::default(),
            base_uuid: Uuid::from_u128(0xb203),
            persistent_copy_uuid: Some(Uuid::from_u128(0xb213)),
            slot: Some(crate::game::resources::item_slot::EquipmentSlotAddress::Weapon),
        }],
        equipped_item_enhancements: vec![],
        employee_start_condition: Some(EmployeeBattleStartCondition {
            trauma: 0,
            battle_hp_bonus_percent: 0,
        }),
    };
    let outcome = core
        .apply_live_command(sim::BattleLiveCommand::DeployPlayerUnit {
            draft,
            position: Position::new(1, 1),
            tile_kind: TacticalTileKind::Ground,
            facing: FacingDirection::Right,
            instance_salt: 1,
            time_ms: 0,
            current_hp_policy: None,
        })
        .unwrap();
    let sim::BattleLiveCommandOutcome::UnitDeployed { unit_id } = outcome else {
        panic!("deployment")
    };
    core.units.get_mut(&unit_id).unwrap().stats.current_health = 10;
    let skill = crate::game::ability::SkillId::from("tb_native");
    let prepared = core.clone();
    let command = || sim::BattleLiveCommand::ActivateSkill {
        unit_id,
        skill_id: skill.clone(),
        target: None,
        time_ms: 0,
    };
    let (accepted, cast_admissions) =
        native::observe_retained_admissions(|| core.apply_live_command(command()));
    accepted.unwrap();
    let cast_boundary = cast_admissions
        .iter()
        .map(|(_, _, total)| *total)
        .max()
        .unwrap();
    assert!(cast_boundary < native::RuntimeFactLimits::default().max_retained_bytes);
    // Each probe starts immediately before this cast; a refused candidate must
    // not consume resonance, RNG/ID counters, event queues or fact ownership.
    let roots = |battle: &BattleCore| {
        (
            battle.transaction_debug_snapshot(),
            battle.event_queue_debug_snapshot(),
            serde_json::to_vec(&battle.runtime_fact_usage()).unwrap(),
            battle.event_log.to_json_string().unwrap(),
            battle.seed,
        )
    };
    let mut retry = prepared.clone();
    let before_cast = roots(&retry);
    let rejected = native::run(
        native::RuntimeFactLimits {
            max_retained_bytes: cast_boundary - 1,
            ..Default::default()
        },
        || retry.apply_live_command(command()),
    );
    assert!(matches!(
        rejected,
        Err(GameError::BattleRuntimeFault(native::BattleRuntimeFault {
            code: native::RuntimeFaultCode::RetainedByteLimit,
            ..
        }))
    ));
    assert_eq!(roots(&retry), before_cast);
    native::run(
        native::RuntimeFactLimits {
            max_retained_bytes: cast_boundary,
            ..Default::default()
        },
        || retry.apply_live_command(command()),
    )
    .unwrap();
    assert_eq!(roots(&retry), roots(&core));
    drop(retry);
    drop(prepared);
    let pending = core.units[&unit_id]
        .pending_skill_cast
        .clone()
        .expect("focus keeps accepted action pending");
    let action = pending.action.clone().unwrap();
    let retained_source = core.runtime_facts.accepted_source(&action).unwrap().clone();
    assert!(Arc::ptr_eq(
        &retained_source.relic_effective().unwrap().plan,
        &plan
    ));
    assert_eq!(
        core.current_skill_view(&skill, unit_id, &Default::default())
            .unwrap()
            .steps[0]
            .effects
            .len(),
        2
    );
    core.set_effective_owner(None).unwrap();
    assert_eq!(
        core.current_skill_view(&skill, unit_id, &Default::default())
            .unwrap()
            .steps[0]
            .effects
            .len(),
        1
    );
    assert_eq!(
        core.pending_skill_view(&pending, unit_id).unwrap().steps[0]
            .effects
            .len(),
        2
    );
    assert_eq!(
        core.skill_for_source(&skill, &retained_source)
            .unwrap()
            .steps[0]
            .effects
            .len(),
        2
    );
    let state = core.transaction_debug_snapshot();
    let mut wrong = retained_source.clone();
    Arc::make_mut(wrong.fragment.as_mut().unwrap()).copy_uuid = Uuid::from_u128(0xb2ffff);
    assert!(core
        .bind_effective_skill_source(&skill, unit_id, &wrong)
        .is_err());
    assert!(core
        .runtime_facts
        .validate_action_source(&action, &wrong)
        .is_err());
    assert_eq!(core.transaction_debug_snapshot(), state);

    // Clone preflight seeds a live retained owner and prices copy slots, rather
    // than admitting another complete immutable payload. Find its byte boundary
    // from actual refusal/success, independently of the retained-size getter.
    let (result, admissions) = native::observe_retained_admissions(|| {
        native::run(Default::default(), || core.preflight_runtime_fact_clone())
    });
    result.unwrap();
    assert!(
        admissions.is_empty(),
        "shared immutable payload is not deep-copied"
    );
    let mut lower = 1;
    let mut upper = native::RuntimeFactLimits::default().max_retained_bytes;
    let mut probes = 0;
    while lower < upper {
        let candidate = lower + (upper - lower) / 2;
        probes += 1;
        match native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: candidate,
                ..Default::default()
            },
            || core.preflight_runtime_fact_clone(),
        ) {
            Ok(()) => upper = candidate,
            Err(GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                code: native::RuntimeFaultCode::RetainedByteLimit,
                site: "retained_operation_start",
                ..
            })) => lower = candidate + 1,
            Err(other) => panic!("unexpected clone probe failure: {other:?}"),
        }
        assert_eq!(core.transaction_debug_snapshot(), state);
    }
    let boundary = upper;
    assert!(boundary < native::RuntimeFactLimits::default().max_retained_bytes);
    for (cap, accepted) in [(boundary - 1, false), (boundary, true)] {
        let mut copied = false;
        let result = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: cap,
                ..Default::default()
            },
            || {
                core.preflight_runtime_fact_clone()?;
                let copy = core.clone();
                copied = true;
                assert_eq!(copy.transaction_debug_snapshot(), state);
                let source = copy.runtime_facts.accepted_source(&action).unwrap();
                assert!(Arc::ptr_eq(&source.relic_effective().unwrap().plan, &plan));
                Ok(())
            },
        );
        if accepted {
            result.unwrap();
        } else {
            assert!(matches!(
                result,
                Err(GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "retained_operation_start",
                    ..
                }))
            ));
        }
        assert_eq!(copied, accepted);
        assert_eq!(core.transaction_debug_snapshot(), state);
    }
    core.step_battle_execution_until(&mut execution, 51)
        .unwrap();
    assert_eq!(
        core.units[&unit_id].stats.current_health, 58,
        "retained cast heals 11+37 after current owner removal"
    );
    assert!(
        core.runtime_facts
            .validate_action_source(&action, &retained_source)
            .is_err(),
        "strong data cannot revive the settled action"
    );
    assert!(weak.upgrade().is_some());
    let mut other_skills = data.skill_data.skills.clone();
    other_skills[0].name.push_str(" distinct generation");
    let other: ValidatedGameData = GameDataBuilder::from_validated(&data)
        .with_skills(SkillDatabase::new(other_skills))
        .build_arc()
        .into();
    assert_ne!(data.identity(), other.identity());
    let mut other_core =
        BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), other, 0xb211);
    let before = other_core.transaction_debug_snapshot();
    let extent = other_core.retained_runtime_fact_extent().unwrap();
    assert!(matches!(
        other_core.set_effective_owner(Some(owner.clone())),
        Err(GameError::InvalidStaticData(_))
    ));
    assert!(other_core.effective_owner.is_none());
    assert_eq!(other_core.retained_runtime_fact_extent().unwrap(), extent);
    assert_eq!(other_core.transaction_debug_snapshot(), before);
    drop(other_core);
    drop(wrong);
    drop(retained_source);
    drop(pending);
    drop(core);
    drop(owner);
    drop(plan);
    assert!(
        weak.upgrade().is_none(),
        "all plan owners released; Weak itself confers no ownership"
    );
    println!("TB002 pending cast_exact={cast_boundary} cast_one_short={} cast_rollback_retry=true current_effects=1 retained_effects=2 heal=48 clone_exact={boundary} one_short={} boundary_probes={probes} provider_generation_refused=true settled_source_refused=true plan_released=true", cast_boundary-1, boundary-1);
}

/// Inspect real native trigger dispatch from an actual World-created battle.
/// Counts are scheduled activation commands, not a claim of executed hits.
pub(crate) fn dispatched_activation_counts(
    core: &mut BattleCore,
    actor: UnitInstanceId,
) -> [usize; 2] {
    use crate::game::stats::TriggerType;
    [TriggerType::OnAttack, TriggerType::OnHit].map(|trigger_type| {
        core.dispatch_item_trigger_occurrence(triggers::TriggerOccurrenceContext {
            trigger_type,
            trigger_unit_id: actor,
            counterpart_unit_id: None,
            target_id: None,
            occurrence_id: Uuid::from_u128(0xb212),
            occurrence_index: 0,
            allow_dead_caster: false,
            committed_hit: None,
            time_ms: 0,
        })
        .activation_commands
        .len()
    })
}
