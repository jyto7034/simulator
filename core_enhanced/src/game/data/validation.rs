use std::{collections::HashSet, fmt::Display};

use uuid::Uuid;

use crate::game::{
    ability::{DeliveryDef, SkillDef, SkillTarget},
    battle::{buffs::BuffDatabase, tile_range::TileRangePolicy, types::BattleUnitThreatClass},
    combat_preview::{
        required_briefing_warning_tags_for_spawn_waves, CombatMissionVariant, CombatNodeType,
        CombatPreview, ThreatWarningSource,
    },
    combat_setup::mission_policy::CombatMissionPolicy,
    data::{
        abnormality_data::AbnormalityDatabase,
        artifact_data::ArtifactDatabase,
        boss_omen_data::{BossOmenChainDatabase, BossOmenSourceKind},
        consumable_data::ConsumableDatabase,
        corroded_employee_data::CorrodedEmployeeProfileDatabase,
        corroded_wave_data::CorrodedWavePresetDatabase,
        employee_data::StarterEmployeeCandidateDatabase,
        equipment_data::EquipmentDatabase,
        event_data::{EventChoiceEffect, EventDatabase},
        pve_data::{PveEncounterClass, PveEncounterDatabase, PveWaveSource},
        reward_data::RewardDatabase,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
        skill_fragment_data::{
            self, SkillFragmentAcquisitionSource, SkillFragmentDependency, SkillFragmentEffectDef,
            SkillFragmentOrigin,
        },
    },
    enums::RewardMode,
    map::{MapNodeCategory, MapNodeId},
    resources::item_slot::{EquippedRef, ItemSlot},
    reward::{RewardEffect, RewardOption},
    reward_policy::CombatRewardPolicy,
};

use super::{GameDataBase, ItemRegistry};

pub(super) fn validate_data_references(
    abnormality_data: &AbnormalityDatabase,
    corroded_employee_data: &CorrodedEmployeeProfileDatabase,
    corroded_wave_data: &CorrodedWavePresetDatabase,
    artifact_data: &ArtifactDatabase,
    consumable_data: &ConsumableDatabase,
    equipment_data: &EquipmentDatabase,
    reward_data: &RewardDatabase,
    event_data: &EventDatabase,
    pve_data: &PveEncounterDatabase,
    boss_omen_data: &BossOmenChainDatabase,
    skill_data: &SkillDatabase,
    buff_data: &BuffDatabase,
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
) {
    skill_data.validate_buff_references(buff_data);
    validate_skill_fragment_skill_references(skill_fragment_data, abnormality_data, skill_data);
    validate_unit_skill_references(
        abnormality_data,
        corroded_employee_data,
        skill_data,
        skill_fragment_data,
    );
    validate_reward_references(
        reward_data,
        equipment_data,
        artifact_data,
        consumable_data,
        skill_fragment_data,
    );
    validate_pve_enemy_references(
        pve_data,
        abnormality_data,
        corroded_employee_data,
        corroded_wave_data,
        reward_data,
    );
    validate_event_combat_references(event_data, pve_data);
    validate_boss_omen_references(boss_omen_data, abnormality_data, event_data, pve_data);
}

pub(super) fn validate_shop_item_references(
    shop_data: &ShopDatabase,
    item_registry: &ItemRegistry,
) {
    for shop in &shop_data.shops {
        for item_uuid in shop.visible_items.iter().chain(shop.hidden_items.iter()) {
            if item_registry.get_index(item_uuid).is_none() {
                panic!(
                    "shop '{}' references missing item uuid {}",
                    shop.id, item_uuid
                );
            }
        }
    }
}

pub(super) fn validate_starter_employee_loadout_references(
    starter_employee_data: &StarterEmployeeCandidateDatabase,
    equipment_data: &EquipmentDatabase,
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
) {
    for candidate in &starter_employee_data.candidates {
        assert!(
            !candidate
                .starter_loadout
                .baseline_skill_fragment_ids
                .is_empty(),
            "starter employee candidate '{}' must declare baseline skill fragments",
            candidate.id
        );
        for fragment_id in &candidate.starter_loadout.baseline_skill_fragment_ids {
            assert!(
                skill_fragment_data.get_by_id(fragment_id).is_some(),
                "starter employee candidate '{}' references missing baseline skill fragment '{}'",
                candidate.id,
                fragment_id
            );
        }

        assert!(
            !candidate.starter_loadout.equipment_ids.is_empty(),
            "starter employee candidate '{}' must declare starting equipment",
            candidate.id
        );
        let mut slot = ItemSlot::default();
        for equipment_id in &candidate.starter_loadout.equipment_ids {
            let equipment = equipment_data.get_by_id(equipment_id).unwrap_or_else(|| {
                panic!(
                    "starter employee candidate '{}' references missing equipment '{}'",
                    candidate.id, equipment_id
                )
            });
            slot.equip(
                EquippedRef {
                    instance_uuid: equipment.uuid,
                    base_uuid: equipment.uuid,
                    equipment_type: equipment.equipment_type,
                },
                equipment.allow_duplicate_equip,
            )
            .unwrap_or_else(|_| {
                panic!(
                    "starter employee candidate '{}' has invalid starting equipment layout",
                    candidate.id
                )
            });
        }
    }
}

pub(super) fn validate_generated_combat_preview_contracts(game_data: &GameDataBase) {
    validate_combat_preview_threat_warning_contract(game_data);
}

fn validate_skill_fragment_skill_references(
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
    abnormality_data: &AbnormalityDatabase,
    skill_data: &SkillDatabase,
) {
    for fragment in &skill_fragment_data.fragments {
        if let Some(SkillFragmentOrigin::Abnormality { abnormality_id }) = &fragment.origin {
            assert!(
                abnormality_data.get_by_id(abnormality_id).is_some(),
                "skill fragment '{}' references unknown origin abnormality '{}'",
                fragment.id,
                abnormality_id
            );
        }

        for source in &fragment.sources {
            match source {
                SkillFragmentAcquisitionSource::AbnormalityContainment { abnormality_id } => {
                    assert!(
                        abnormality_data.get_by_id(abnormality_id).is_some(),
                        "skill fragment '{}' references unknown source abnormality '{}'",
                        fragment.id,
                        abnormality_id
                    )
                }
                SkillFragmentAcquisitionSource::RareReward
                | SkillFragmentAcquisitionSource::DependentConcept { .. } => {}
            }
        }

        for dependency in &fragment.dependencies {
            if let SkillFragmentDependency::SourceAbnormality { abnormality_id } = dependency {
                assert!(
                    abnormality_data.get_by_id(abnormality_id).is_some(),
                    "skill fragment '{}' references unknown dependency abnormality '{}'",
                    fragment.id,
                    abnormality_id
                );
            }
        }

        let (imitation_skill_id, upgrade_skill_ids, awakened_skill_id) = match &fragment.effect {
            SkillFragmentEffectDef::BasicAttackModifier { .. } => continue,
            SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id,
                upgrade_skill_ids,
                awakened_skill_id,
            } => (imitation_skill_id, upgrade_skill_ids, awakened_skill_id),
        };

        let imitation_skill = skill_data.get_by_id(imitation_skill_id);
        assert!(
            imitation_skill.is_some(),
            "skill fragment '{}' references unknown imitation skill '{}'",
            fragment.id,
            imitation_skill_id
        );
        if let Some(skill) = imitation_skill {
            validate_defense_route_skill_contract("skill fragment", &fragment.id, skill);
        }
        for (level, skill_id) in upgrade_skill_ids {
            let upgrade_skill = skill_data.get_by_id(skill_id);
            assert!(
                upgrade_skill.is_some(),
                "skill fragment '{}' references unknown upgrade skill '{}' at level {}",
                fragment.id,
                skill_id,
                level
            );
            if let Some(skill) = upgrade_skill {
                validate_defense_route_skill_contract("skill fragment", &fragment.id, skill);
            }
        }
        if let Some(awakened_skill_id) = awakened_skill_id {
            let awakened_skill = skill_data.get_by_id(awakened_skill_id);
            assert!(
                awakened_skill.is_some(),
                "skill fragment '{}' references unknown awakened skill '{}'",
                fragment.id,
                awakened_skill_id
            );
            if let Some(skill) = awakened_skill {
                validate_defense_route_skill_contract("skill fragment", &fragment.id, skill);
            }
        }
    }
}

fn validate_defense_route_skill_contract(
    source_kind: &str,
    source_id: &impl Display,
    skill: &SkillDef,
) {
    for step in &skill.steps {
        let requires_tile_range = matches!(
            step.target,
            SkillTarget::EnemySingle { .. } | SkillTarget::CastTarget
        ) || matches!(step.delivery, DeliveryDef::TileArea { .. });
        assert!(
            !requires_tile_range
                || step.range_policy == TileRangePolicy::WholeFieldValidTiles
                || step.defense_tile_range.is_some(),
            "{} '{}' references DefenseRoute skill '{}' step '{}' without defense_tile_range or WholeFieldValidTiles range_policy",
            source_kind,
            source_id,
            skill.id,
            step.id
        );
    }
}

fn validate_unit_skill_references(
    abnormality_data: &AbnormalityDatabase,
    corroded_employee_data: &CorrodedEmployeeProfileDatabase,
    skill_data: &SkillDatabase,
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
) {
    for abnormality in &abnormality_data.items {
        assert!(
            !matches!(abnormality.threat_class, BattleUnitThreatClass::Normal),
            "abnormality '{}' must use Elite or Boss threat_class",
            abnormality.id
        );
        if !skill_fragment_data.fragments.is_empty() {
            let response_fragment_id = abnormality
                .response_complete_skill_fragment_id
                .as_ref()
                .unwrap_or_else(|| {
                    panic!(
                        "abnormality '{}' must declare response_complete_skill_fragment_id",
                        abnormality.id
                    )
                });
            assert!(
                skill_fragment_data
                    .get_by_id(response_fragment_id)
                    .is_some(),
                "abnormality '{}' references unknown response_complete_skill_fragment_id '{}'",
                abnormality.id,
                response_fragment_id
            );
        }
        if let Some(skill_id) = &abnormality.skill_id {
            let skill = skill_data.get_by_id(skill_id);
            assert!(
                skill.is_some(),
                "abnormality '{}' references unknown skill '{}'",
                abnormality.id,
                skill_id
            );
            if let Some(skill) = skill {
                validate_defense_route_skill_contract("abnormality", &abnormality.id, skill);
            }
        }
    }

    for profile in &corroded_employee_data.profiles {
        if let Some(skill_id) = &profile.skill_id {
            let skill = skill_data.get_by_id(skill_id);
            assert!(
                skill.is_some(),
                "corroded employee profile '{}' references unknown skill '{}'",
                profile.id,
                skill_id
            );
            if let Some(skill) = skill {
                validate_defense_route_skill_contract(
                    "corroded employee profile",
                    &profile.id,
                    skill,
                );
            }
        }
    }
}

fn validate_reward_references(
    reward_data: &RewardDatabase,
    equipment_data: &EquipmentDatabase,
    artifact_data: &ArtifactDatabase,
    consumable_data: &ConsumableDatabase,
    skill_fragment_data: &skill_fragment_data::SkillFragmentDatabase,
) {
    for reward in &reward_data.rewards {
        for effect in &reward.effects {
            match effect {
                RewardEffect::GrantEquipment { equipment_id } => assert!(
                    equipment_data.get_by_id(equipment_id).is_some(),
                    "reward '{}' references unknown equipment '{}'",
                    reward.id,
                    equipment_id
                ),
                RewardEffect::GrantEquipmentFromPool { pool_id } => {
                    let pool = reward_data
                        .equipment_pool_by_id(pool_id)
                        .unwrap_or_else(|| {
                            panic!(
                                "reward '{}' references unknown equipment reward pool '{}'",
                                reward.id, pool_id
                            )
                        });
                    assert!(
                        !pool.candidates(equipment_data).is_empty(),
                        "reward '{}' equipment reward pool '{}' has no candidates",
                        reward.id,
                        pool_id
                    );
                }
                RewardEffect::GrantEquipmentMaterial {
                    material_id,
                    amount,
                } => {
                    assert!(
                        *amount > 0,
                        "reward '{}' grants zero equipment material",
                        reward.id
                    );
                    assert!(
                        equipment_data.get_material_by_id(material_id).is_some(),
                        "reward '{}' references unknown equipment material '{}'",
                        reward.id,
                        material_id
                    );
                }
                RewardEffect::GrantArtifact { artifact_id } => assert!(
                    artifact_data.get_by_id(artifact_id).is_some(),
                    "reward '{}' references unknown artifact '{}'",
                    reward.id,
                    artifact_id
                ),
                RewardEffect::GrantConsumable { consumable_id } => assert!(
                    consumable_data.get_by_id(consumable_id).is_some(),
                    "reward '{}' references unknown consumable '{}'",
                    reward.id,
                    consumable_id
                ),
                RewardEffect::GrantSkillFragment { fragment_id } => assert!(
                    skill_fragment_data.get_by_id(fragment_id).is_some(),
                    "reward '{}' references unknown skill fragment '{}'",
                    reward.id,
                    fragment_id
                ),
                RewardEffect::GrantSkillFragmentResearch {
                    fragment_id,
                    amount,
                } => {
                    assert!(
                        *amount > 0,
                        "reward '{}' grants zero skill fragment research progress",
                        reward.id
                    );
                    assert!(
                        skill_fragment_data.get_by_id(fragment_id).is_some(),
                        "reward '{}' references unknown skill fragment research target '{}'",
                        reward.id,
                        fragment_id
                    );
                }
                RewardEffect::GrantEnkephalin { .. } => {}
                RewardEffect::GrantExperience { amount, .. } => {
                    assert!(*amount > 0, "reward '{}' grants zero experience", reward.id)
                }
                RewardEffect::GrantFragmentDust { amount } => assert!(
                    *amount > 0,
                    "reward '{}' grants zero skill fragment dust",
                    reward.id
                ),
            }
        }
    }

    let mut seen_equipment_pools = HashSet::new();
    for pool in &reward_data.equipment_pools {
        assert!(
            !pool.id.is_empty(),
            "equipment reward pool id must not be empty"
        );
        assert!(
            seen_equipment_pools.insert(pool.id.as_str()),
            "duplicate equipment reward pool id '{}'",
            pool.id
        );
        for entry in &pool.entries {
            assert!(
                entry.weight > 0,
                "equipment reward pool '{}' entry '{}' must have positive weight",
                pool.id,
                entry.equipment_id
            );
            assert!(
                equipment_data.get_by_id(&entry.equipment_id).is_some(),
                "equipment reward pool '{}' references unknown equipment '{}'",
                pool.id,
                entry.equipment_id
            );
        }
        assert!(
            !pool.candidates(equipment_data).is_empty(),
            "equipment reward pool '{}' must produce at least one candidate",
            pool.id
        );
    }
}

fn validate_pve_enemy_references(
    pve_data: &PveEncounterDatabase,
    abnormality_data: &AbnormalityDatabase,
    corroded_employee_data: &CorrodedEmployeeProfileDatabase,
    corroded_wave_data: &CorrodedWavePresetDatabase,
    reward_data: &RewardDatabase,
) {
    for encounter in &pve_data.encounters {
        match encounter.encounter_class {
            PveEncounterClass::Normal => {
                assert!(
                    encounter.primary_abnormality_id().is_none(),
                    "normal pve encounter '{}' must not define primary_abnormality_id",
                    encounter.id
                );
            }
            PveEncounterClass::Elite
            | PveEncounterClass::NormalBoss
            | PveEncounterClass::FinalBoss => {
                let primary_abnormality_id =
                    encounter.primary_abnormality_id().unwrap_or_else(|| {
                        panic!(
                            "{:?} pve encounter '{}' must define primary_abnormality_id",
                            encounter.encounter_class, encounter.id
                        )
                    });
                let abnormality = abnormality_data
                    .get_by_id(primary_abnormality_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "pve encounter '{}' references missing primary abnormality '{}'",
                            encounter.id, primary_abnormality_id
                        )
                    });
                match encounter.encounter_class {
                    PveEncounterClass::Elite => assert_eq!(
                        abnormality.threat_class,
                        BattleUnitThreatClass::Elite,
                        "elite pve encounter '{}' primary abnormality '{}' must be Elite",
                        encounter.id,
                        primary_abnormality_id
                    ),
                    PveEncounterClass::NormalBoss | PveEncounterClass::FinalBoss => assert_eq!(
                        abnormality.threat_class,
                        BattleUnitThreatClass::Boss,
                        "{:?} pve encounter '{}' primary abnormality '{}' must be Boss",
                        encounter.encounter_class,
                        encounter.id,
                        primary_abnormality_id
                    ),
                    PveEncounterClass::Normal => unreachable!(),
                }
            }
        }

        if encounter.has_bonus_objectives() {
            assert!(
                encounter.primary_abnormality_id().is_some(),
                "pve encounter '{}' defines bonus objectives but has no primary abnormality",
                encounter.id
            );
        }

        for wave in &encounter.waves {
            if let PveWaveSource::GeneratedCorroded { preset_id, .. } = &wave.source {
                let preset = corroded_wave_data.get_by_id(preset_id).unwrap_or_else(|| {
                    panic!(
                        "pve encounter '{}' wave '{}' references unknown corroded wave preset '{}'",
                        encounter.id, wave.id, preset_id
                    )
                });
                for role in &preset.role_mix {
                    assert!(
                        corroded_employee_data.get_by_id(&role.profile_id).is_some(),
                        "corroded wave preset '{}' references unknown corroded employee profile '{}'",
                        preset.id,
                        role.profile_id
                    );
                }
            }

            for enemy in wave.manual_enemies() {
                match enemy {
                    crate::game::data::pve_data::PveWaveEnemyData::Abnormality {
                        abnormality_id,
                        ..
                    } => assert!(
                        abnormality_data.get_by_id(abnormality_id).is_some(),
                        "pve encounter '{}' wave '{}' references unknown abnormality enemy '{}'",
                        encounter.id,
                        wave.id,
                        abnormality_id
                    ),
                    crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
                        profile_id,
                        ..
                    } => assert!(
                        corroded_employee_data.get_by_id(profile_id).is_some(),
                        "pve encounter '{}' wave '{}' references unknown corroded employee profile '{}'",
                        encounter.id,
                        wave.id,
                        profile_id
                    ),
                    crate::game::data::pve_data::PveWaveEnemyData::FacilityEntity { .. } => {
                        panic!(
                            "pve encounter '{}' wave '{}' uses FacilityEntity before facility entity profiles are implemented",
                            encounter.id, wave.id
                        );
                    }
                }
            }
        }
        for reward_uuid in &encounter.reward_uuids {
            assert!(
                reward_data.get_by_uuid(reward_uuid).is_some(),
                "pve encounter '{}' references missing reward uuid {}",
                encounter.id,
                reward_uuid
            );
        }
        if let Some(survive_timer_ms) = encounter.survive_timer_ms {
            assert!(
                survive_timer_ms > 0,
                "pve encounter '{}' survive_timer_ms must be greater than zero",
                encounter.id
            );
            assert!(
                encounter.node_type == Some(CombatNodeType::Defense),
                "pve encounter '{}' defines survive_timer_ms but is not an explicit Defense encounter",
                encounter.id
            );
        }
        if let Some(node_type) = encounter.node_type {
            assert_eq!(
                encounter.reward_mode,
                RewardMode::ClaimAll,
                "pve encounter '{}' uses {:?} reward_mode, but live combat result rewards are automatically granted and must use ClaimAll",
                encounter.id,
                encounter.reward_mode
            );
            let mission_variant = encounter
                .mission_variant
                .unwrap_or_else(|| CombatMissionVariant::default_for_node_type(node_type));
            assert!(
                mission_variant.is_compatible_with(node_type),
                "pve encounter '{}' mission_variant {:?} is incompatible with node_type {:?}",
                encounter.id,
                mission_variant,
                node_type
            );
            assert!(
                CombatMissionPolicy::is_supported_encounter_node_type(node_type),
                "pve encounter '{}' uses deferred combat node type {:?}",
                encounter.id,
                node_type
            );
            let rewards = encounter
                .reward_uuids
                .iter()
                .filter_map(|reward_uuid| reward_data.get_by_uuid(reward_uuid))
                .map(RewardOption::from_metadata)
                .collect::<Vec<_>>();
            CombatRewardPolicy::for_mission(node_type, mission_variant)
                .validate_rewards(&rewards)
                .unwrap_or_else(|message| {
                    panic!(
                        "pve encounter '{}' has invalid {:?} reward policy: {}",
                        encounter.id, node_type, message
                    )
                });
        }
    }
}

fn validate_boss_omen_references(
    boss_omen_data: &BossOmenChainDatabase,
    abnormality_data: &AbnormalityDatabase,
    event_data: &EventDatabase,
    pve_data: &PveEncounterDatabase,
) {
    let chain_ids = boss_omen_data
        .chains
        .iter()
        .map(|chain| chain.id.as_str())
        .collect::<HashSet<_>>();

    for abnormality in &abnormality_data.items {
        let Some(omen_chain_id) = &abnormality.omen_chain_id else {
            continue;
        };
        assert!(
            chain_ids.contains(omen_chain_id.as_str()),
            "abnormality '{}' references unknown omen_chain_id '{}'",
            abnormality.id,
            omen_chain_id.as_str()
        );
    }

    for chain in &boss_omen_data.chains {
        let abnormality = abnormality_data
            .get_by_id(&chain.boss_abnormality_id)
            .unwrap_or_else(|| {
                panic!(
                    "boss omen chain '{}' references missing boss abnormality '{}'",
                    chain.id.as_str(),
                    chain.boss_abnormality_id
                )
            });
        assert!(
            abnormality
                .omen_chain_id
                .as_ref()
                .is_some_and(|id| id == &chain.id),
            "boss omen chain '{}' boss abnormality '{}' must reference it through omen_chain_id",
            chain.id.as_str(),
            abnormality.id
        );

        let boss_encounter = pve_data
            .get_by_id(&chain.boss_encounter_id)
            .unwrap_or_else(|| {
                panic!(
                    "boss omen chain '{}' references missing boss encounter '{}'",
                    chain.id.as_str(),
                    chain.boss_encounter_id
                )
            });
        assert_eq!(
            boss_encounter.primary_abnormality_id(),
            Some(chain.boss_abnormality_id.as_str()),
            "boss omen chain '{}' boss encounter '{}' must target abnormality '{}'",
            chain.id.as_str(),
            boss_encounter.id,
            chain.boss_abnormality_id
        );
        assert!(
            matches!(boss_encounter.encounter_class, PveEncounterClass::FinalBoss),
            "boss omen chain '{}' boss encounter '{}' must be FinalBoss",
            chain.id.as_str(),
            boss_encounter.id
        );

        for step in &chain.steps {
            match step.source_kind {
                BossOmenSourceKind::Event => {
                    let event_id = step
                        .event_id
                        .as_ref()
                        .expect("event step must have event_id after local validation");
                    assert!(
                        event_data.get_by_id(event_id.as_str()).is_some(),
                        "boss omen chain '{}' step '{}' references missing event '{}'",
                        chain.id.as_str(),
                        step.id.as_str(),
                        event_id.as_str()
                    );
                }
                BossOmenSourceKind::Combat => {
                    let encounter_id = step
                        .encounter_id
                        .as_ref()
                        .expect("combat step must have encounter_id after local validation");
                    let encounter = pve_data.get_by_id(encounter_id).unwrap_or_else(|| {
                        panic!(
                            "boss omen chain '{}' step '{}' references missing encounter '{}'",
                            chain.id.as_str(),
                            step.id.as_str(),
                            encounter_id
                        )
                    });
                    assert!(
                        matches!(
                            encounter.encounter_class,
                            PveEncounterClass::Elite
                                | PveEncounterClass::NormalBoss
                                | PveEncounterClass::FinalBoss
                        ),
                        "boss omen chain '{}' step '{}' combat encounter '{}' must be abnormality-bearing",
                        chain.id.as_str(),
                        step.id.as_str(),
                        encounter.id
                    );
                }
            }
        }
    }
}

fn validate_event_combat_references(event_data: &EventDatabase, pve_data: &PveEncounterDatabase) {
    for event in &event_data.events {
        for scene in &event.scenes {
            let crate::game::data::event_data::EventSceneNext::Choices { choices } = &scene.next
            else {
                continue;
            };
            for choice in choices {
                for effect in &choice.effects {
                    let EventChoiceEffect::StartCombat {
                        encounter_id,
                        primary_abnormality_id,
                    } = effect
                    else {
                        continue;
                    };
                    let encounter = pve_data.get_by_id(encounter_id).unwrap_or_else(|| {
                        panic!(
                            "event '{}' choice '{}' references missing combat encounter '{}'",
                            event.id.as_str(),
                            choice.id.as_str(),
                            encounter_id
                        )
                    });
                    if let Some(primary_abnormality_id) = primary_abnormality_id {
                        assert_eq!(
                            encounter.primary_abnormality_id(),
                            Some(primary_abnormality_id.as_str()),
                            "event '{}' choice '{}' primary_abnormality_id '{}' does not match encounter '{}'",
                            event.id.as_str(),
                            choice.id.as_str(),
                            primary_abnormality_id,
                            encounter_id
                        );
                    }
                }
            }
        }
    }
}

fn validate_combat_preview_threat_warning_contract(game_data: &GameDataBase) {
    const PREVIEW_VALIDATION_SEEDS: [u64; 5] = [0, 1, 17, 41, 99];

    for encounter in &game_data.pve_data.encounters {
        for seed in PREVIEW_VALIDATION_SEEDS {
            let node_id = MapNodeId::new(Uuid::from_u128(
                0xADAD_0000_0000_0000_0000_0000_0000_0000_u128 + u128::from(seed),
            ));
            let preview = CombatPreview::try_generate_for_node(
                node_id,
                MapNodeCategory::Combat,
                Some(encounter.id.as_str()),
                game_data,
                seed,
            )
            .unwrap_or_else(|error| {
                panic!(
                    "combat preview generation failed for pve encounter '{}' seed {}: {:?}",
                    encounter.id, seed, error
                )
            });
            let required =
                required_briefing_warning_tags_for_spawn_waves(&preview.spawn_waves, game_data);
            let briefing_tags = preview
                .threat_warnings
                .iter()
                .filter(|warning| warning.source == ThreatWarningSource::Briefing)
                .map(|warning| warning.tag)
                .collect::<HashSet<_>>();

            for required_tag in required {
                assert!(
                    briefing_tags.contains(&required_tag),
                    "combat preview for pve encounter '{}' seed {} omits required briefing warning {:?}",
                    encounter.id,
                    seed,
                    required_tag
                );
            }
        }
    }
}
