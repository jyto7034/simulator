use std::{collections::HashSet, error::Error, fmt};
use std::{sync::Arc, sync::OnceLock};

use serde::{de::DeserializeOwned, Deserialize};

use crate::game::data::visual_novel_trigger_data::{
    VisualNovelTriggerCheckpointPolicy, VisualNovelTriggerRepeat, VisualNovelTriggerSource,
};
use crate::game::{
    battle::buffs::BuffDatabase,
    combat_preview::{
        BattlefieldGenerationPolicyDatabase, BattlefieldTemplateDatabase,
        BattlefieldTemplateDefinition,
    },
    data::{
        abnormality_data::AbnormalityDatabase,
        boss_omen_data::BossOmenChainDatabase,
        consumable_data::ConsumableDatabase,
        corroded_employee_data::CorrodedEmployeeProfileDatabase,
        corroded_wave_data::CorrodedWavePresetDatabase,
        employee_data::EmployeeDefinitionDatabase,
        equipment_data::EquipmentDatabase,
        event_data::{EventDatabase, EventSceneNext, EventSuccessAction},
        pve_data::PveEncounterDatabase,
        reward_data::RewardDatabase,
        run_policy_data::RunPolicyData,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
        skill_fragment_data::SkillFragmentDatabase,
        visual_novel_data::VisualNovelDatabase,
        visual_novel_trigger_data::VisualNovelTriggerDatabase,
    },
    map::{
        FacilityTemplateDatabase, GameMode, MapNodeDefinitionDatabase, MapNodePayload,
        NodeAssignmentProfileDatabase,
    },
    reward::RewardEffect,
};

pub const LIVE_RON_SCHEMA_VERSION: u32 = 1;
pub const LIVE_GAMEPLAY_RON_SOURCE_COUNT: usize = 22;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveRonError {
    pub source_file: &'static str,
    pub domain: &'static str,
    pub object_id: Option<String>,
    pub field_path: Option<String>,
    pub cause: String,
}

impl LiveRonError {
    pub fn parse(
        source_file: &'static str,
        domain: &'static str,
        field_path: Option<String>,
        cause: impl Into<String>,
    ) -> Self {
        Self {
            source_file,
            domain,
            object_id: None,
            field_path,
            cause: cause.into(),
        }
    }

    pub fn validation(
        source_file: &'static str,
        domain: &'static str,
        object_id: Option<impl Into<String>>,
        field_path: Option<impl Into<String>>,
        cause: impl Into<String>,
    ) -> Self {
        Self {
            source_file,
            domain,
            object_id: object_id.map(Into::into),
            field_path: field_path.map(Into::into),
            cause: cause.into(),
        }
    }
}

impl fmt::Display for LiveRonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} [{}]", self.source_file, self.domain)?;
        if let Some(object_id) = &self.object_id {
            write!(formatter, " object '{object_id}'")?;
        }
        if let Some(field_path) = &self.field_path {
            write!(formatter, " at {field_path}")?;
        }
        write!(formatter, ": {}", self.cause)
    }
}

impl Error for LiveRonError {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveRonBundleManifest {
    schema_version: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GameplayRonSourceSet<'a> {
    pub manifest: &'a str,
    pub shops: &'a str,
    pub rewards: &'a str,
    pub events: &'a str,
    pub visual_novels: &'a str,
    pub visual_novel_triggers: &'a str,
    pub boss_omen_chains: &'a str,
    pub abnormalities: &'a str,
    pub corroded_employees: &'a str,
    pub corroded_wave_presets: &'a str,
    pub employee_definitions: &'a str,
    pub equipment: &'a str,
    pub consumables: &'a str,
    pub buffs: &'a str,
    pub skills: &'a str,
    pub skill_fragments: &'a str,
    pub pve_encounters: &'a str,
    pub run_policy: &'a str,
    pub map_node_definitions: &'a str,
    pub facility_templates: &'a str,
    pub node_assignment_profiles: &'a str,
    pub battlefield_archetypes: &'a str,
    pub battlefield_templates: &'a str,
}

impl GameplayRonSourceSet<'static> {
    pub(crate) fn embedded() -> Self {
        Self {
            manifest: include_str!("../../../../game_resources/data/bundle.ron"),
            shops: include_str!("../../../../game_resources/data/events/shops/base.ron"),
            rewards: include_str!("../../../../game_resources/data/events/rewards/base.ron"),
            events: include_str!("../../../../game_resources/data/events/story/base.ron"),
            visual_novels: include_str!("../../../../game_resources/data/visual_novels/base.ron"),
            visual_novel_triggers: include_str!(
                "../../../../game_resources/data/visual_novel_triggers/base.ron"
            ),
            boss_omen_chains: include_str!("../../../../game_resources/data/boss_omen/chains.ron"),
            abnormalities: include_str!("../../../../game_resources/data/abnormalities/base.ron"),
            corroded_employees: include_str!(
                "../../../../game_resources/data/enemies/corroded_employees.ron"
            ),
            corroded_wave_presets: include_str!(
                "../../../../game_resources/data/enemies/corroded_wave_presets.ron"
            ),
            employee_definitions: include_str!(
                "../../../../game_resources/data/employees/definitions.ron"
            ),
            equipment: include_str!("../../../../game_resources/data/equipments/base.ron"),
            consumables: include_str!("../../../../game_resources/data/consumables/base.ron"),
            buffs: include_str!("../../../../game_resources/data/buffs/base.ron"),
            skills: include_str!("../../../../game_resources/data/skills/base.ron"),
            skill_fragments: include_str!(
                "../../../../game_resources/data/skill_fragments/base.ron"
            ),
            pve_encounters: include_str!("../../../../game_resources/data/pve/encounters.ron"),
            run_policy: include_str!("../../../../game_resources/data/run/policy.ron"),
            map_node_definitions: include_str!(
                "../../../../game_resources/data/map/node_definitions.ron"
            ),
            facility_templates: include_str!(
                "../../../../game_resources/data/map/facility_templates.ron"
            ),
            node_assignment_profiles: include_str!(
                "../../../../game_resources/data/map/node_assignment_profiles.ron"
            ),
            battlefield_archetypes: include_str!(
                "../../../../game_resources/data/map/battlefield_archetypes.ron"
            ),
            battlefield_templates: include_str!(
                "../../../../game_resources/data/map/battlefield_templates.ron"
            ),
        }
    }
}

#[derive(Clone)]
pub(crate) struct GameplayMapCatalogs {
    pub map_node_definitions: Arc<MapNodeDefinitionDatabase>,
    pub facility_templates: Arc<FacilityTemplateDatabase>,
    pub node_assignment_profiles: Arc<NodeAssignmentProfileDatabase>,
    pub battlefield_archetypes: Arc<BattlefieldGenerationPolicyDatabase>,
    pub battlefield_templates: Arc<BattlefieldTemplateDatabase>,
}

impl GameplayMapCatalogs {
    fn validate(&self) -> Result<(), LiveRonError> {
        self.map_node_definitions
            .validate_contract()
            .map_err(|cause| {
                LiveRonError::validation(
                    "data/map/node_definitions.ron",
                    "map_node_definitions",
                    None::<String>,
                    None::<String>,
                    cause,
                )
            })?;
        self.facility_templates
            .validate_contract()
            .map_err(|cause| {
                LiveRonError::validation(
                    "data/map/facility_templates.ron",
                    "facility_templates",
                    None::<String>,
                    None::<String>,
                    cause,
                )
            })?;
        self.node_assignment_profiles
            .validate_contract(&self.facility_templates)
            .map_err(|cause| {
                LiveRonError::validation(
                    "data/map/node_assignment_profiles.ron",
                    "node_assignment_profiles",
                    None::<String>,
                    None::<String>,
                    cause,
                )
            })?;
        self.battlefield_archetypes
            .validate_contract()
            .map_err(|cause| {
                LiveRonError::validation(
                    "data/map/battlefield_archetypes.ron",
                    "battlefield_archetypes",
                    None::<String>,
                    None::<String>,
                    cause,
                )
            })?;
        self.battlefield_templates
            .validate_contract(&self.battlefield_archetypes)
            .map_err(|cause| {
                LiveRonError::validation(
                    "data/map/battlefield_templates.ron",
                    "battlefield_templates",
                    None::<String>,
                    None::<String>,
                    cause,
                )
            })?;
        Ok(())
    }
}

pub(crate) struct ParsedGameplayRon {
    pub shops: ShopDatabase,
    pub rewards: RewardDatabase,
    pub events: EventDatabase,
    pub visual_novels: VisualNovelDatabase,
    pub visual_novel_triggers: VisualNovelTriggerDatabase,
    pub boss_omen_chains: BossOmenChainDatabase,
    pub abnormalities: AbnormalityDatabase,
    pub corroded_employees: CorrodedEmployeeProfileDatabase,
    pub corroded_wave_presets: CorrodedWavePresetDatabase,
    pub employee_definitions: EmployeeDefinitionDatabase,
    pub equipment: EquipmentDatabase,
    pub consumables: ConsumableDatabase,
    pub buffs: BuffDatabase,
    pub skills: SkillDatabase,
    pub skill_fragments: SkillFragmentDatabase,
    pub pve_encounters: PveEncounterDatabase,
    pub run_policy: RunPolicyData,
    pub map_catalogs: GameplayMapCatalogs,
}

impl ParsedGameplayRon {
    pub(crate) fn validate_domain_catalogs(&self) -> Result<(), LiveRonError> {
        self.run_policy.validate_contract().map_err(|cause| {
            LiveRonError::validation(
                "data/run/policy.ron",
                "run_policy",
                None::<String>,
                None::<String>,
                cause,
            )
        })?;
        self.validate_top_level_catalog_keys()?;
        self.validate_fallible_domain_contracts()?;
        self.validate_cross_item_uuids()?;
        self.validate_unique_pool_ids()?;
        super::validation::validate_equipment_reward_pool_references(
            &self.rewards,
            &self.equipment,
        )
        .map_err(|error| live_domain_error("data/events/rewards/base.ron", "rewards", error))?;
        self.validate_reward_effect_contracts()?;
        super::validation::validate_pve_enemy_references(
            &self.pve_encounters,
            &self.abnormalities,
            &self.corroded_employees,
            &self.corroded_wave_presets,
            &self.rewards,
        )
        .map_err(|error| live_domain_error("data/pve/encounters.ron", "pve_encounters", error))?;
        self.validate_run_start_contract()?;
        self.validate_extra_wave_contracts()?;
        self.map_catalogs.validate()?;
        self.validate_map_difficulty_profile_references()?;
        self.validate_map_payload_references()
    }

    fn validate_unique_pool_ids(&self) -> Result<(), LiveRonError> {
        validate_pool_ids(
            "data/events/shops/base.ron",
            "shops",
            "pools",
            self.shops.pools.iter().map(|pool| pool.id.as_str()),
        )?;
        validate_pool_ids(
            "data/events/rewards/base.ron",
            "rewards",
            "pools",
            self.rewards.pools.iter().map(|pool| pool.id.as_str()),
        )?;
        validate_pool_ids(
            "data/events/rewards/base.ron",
            "rewards",
            "equipment_pools",
            self.rewards
                .equipment_pools
                .iter()
                .map(|pool| pool.id.as_str()),
        )
    }

    fn validate_fallible_domain_contracts(&self) -> Result<(), LiveRonError> {
        self.consumables.validate_contract().map_err(|error| {
            live_domain_error("data/consumables/base.ron", "consumables", error)
        })?;
        self.buffs
            .validate_contract()
            .map_err(|error| live_domain_error("data/buffs/base.ron", "buffs", error))?;
        self.skills
            .validate_contract()
            .map_err(|error| live_domain_error("data/skills/base.ron", "skills", error))?;
        self.skills
            .validate_buff_references(&self.buffs)
            .map_err(|error| live_domain_error("data/skills/base.ron", "skills", error))?;
        self.employee_definitions
            .validate_contract()
            .map_err(|error| {
                live_domain_error(
                    "data/employees/definitions.ron",
                    "employee_definitions",
                    error,
                )
            })?;
        self.corroded_employees
            .validate_contract()
            .map_err(|error| {
                live_domain_error(
                    "data/enemies/corroded_employees.ron",
                    "corroded_employees",
                    error,
                )
            })?;
        self.corroded_wave_presets
            .validate_contract()
            .map_err(|error| {
                live_domain_error(
                    "data/enemies/corroded_wave_presets.ron",
                    "corroded_wave_presets",
                    error,
                )
            })?;
        self.shops
            .validate_contract()
            .map_err(|error| live_domain_error("data/events/shops/base.ron", "shops", error))?;
        self.rewards
            .validate_contract()
            .map_err(|error| live_domain_error("data/events/rewards/base.ron", "rewards", error))?;
        self.abnormalities.validate_contract().map_err(|error| {
            live_domain_error("data/abnormalities/base.ron", "abnormalities", error)
        })?;
        self.boss_omen_chains.validate_contract().map_err(|error| {
            live_domain_error("data/boss_omen/chains.ron", "boss_omen_chains", error)
        })?;
        super::validation::validate_abnormality_omen_references(
            &self.abnormalities,
            &self.boss_omen_chains,
        )
        .map_err(|error| {
            live_domain_error("data/abnormalities/base.ron", "abnormalities", error)
        })?;
        super::validation::validate_boss_omen_references(
            &self.boss_omen_chains,
            &self.abnormalities,
            &self.events,
            &self.pve_encounters,
        )
        .map_err(|error| {
            live_domain_error("data/boss_omen/chains.ron", "boss_omen_chains", error)
        })?;
        self.events
            .validate_contract()
            .map_err(|error| live_domain_error("data/events/story/base.ron", "events", error))?;
        super::validation::validate_event_combat_references(&self.events, &self.pve_encounters)
            .map_err(|error| live_domain_error("data/events/story/base.ron", "events", error))?;
        self.pve_encounters.validate_contract().map_err(|error| {
            live_domain_error("data/pve/encounters.ron", "pve_encounters", error)
        })?;
        self.visual_novels.validate_contract().map_err(|error| {
            live_domain_error("data/visual_novels/base.ron", "visual_novels", error)
        })?;
        self.visual_novel_triggers
            .validate_contract()
            .map_err(|error| {
                live_domain_error(
                    "data/visual_novel_triggers/base.ron",
                    "visual_novel_triggers",
                    error,
                )
            })?;
        crate::game::data::visual_novel_trigger_data::validate_visual_novel_trigger_references(
            &self.visual_novel_triggers,
            &self.visual_novels,
        )
        .map_err(|error| {
            live_domain_error(
                "data/visual_novel_triggers/base.ron",
                "visual_novel_triggers",
                error,
            )
        })?;
        self.skill_fragments.validate_contract().map_err(|error| {
            live_domain_error("data/skill_fragments/base.ron", "skill_fragments", error)
        })?;
        super::validation::validate_skill_fragment_skill_references(
            &self.skill_fragments,
            &self.abnormalities,
            &self.skills,
        )
        .map_err(|error| {
            live_domain_error("data/skill_fragments/base.ron", "skill_fragments", error)
        })?;
        super::validation::validate_abnormality_unit_skill_references(
            &self.abnormalities,
            &self.skills,
            &self.skill_fragments,
        )
        .map_err(|error| {
            live_domain_error("data/abnormalities/base.ron", "abnormalities", error)
        })?;
        super::validation::validate_corroded_employee_skill_references(
            &self.corroded_employees,
            &self.skills,
        )
        .map_err(|error| {
            live_domain_error(
                "data/enemies/corroded_employees.ron",
                "corroded_employees",
                error,
            )
        })?;
        self.equipment
            .validate_contract()
            .map_err(|error| live_domain_error("data/equipments/base.ron", "equipment", error))?;
        super::validation::validate_event_choice_key_references(&self.events, &self.equipment)
            .map_err(|error| live_domain_error("data/events/story/base.ron", "events", error))?;
        super::validation::validate_shop_item_references(
            &self.shops,
            &self.consumables,
            &self.equipment,
        )
        .map_err(|error| live_domain_error("data/events/shops/base.ron", "shops", error))?;
        super::validation::validate_monolith_references(
            &self.run_policy,
            &self.shops,
            &self.visual_novels,
            &self.pve_encounters,
            &self.abnormalities,
        )
        .map_err(|error| live_domain_error("data/run/policy.ron", "monolith", error))?;
        super::validation::validate_starter_employee_loadout_references(
            &self.employee_definitions,
            &self.equipment,
            &self.skill_fragments,
        )
        .map_err(|error| {
            live_domain_error(
                "data/employees/definitions.ron",
                "employee_definitions",
                error,
            )
        })?;
        Ok(())
    }

    fn validate_top_level_catalog_keys(&self) -> Result<(), LiveRonError> {
        validate_catalog_ids(
            "data/events/shops/base.ron",
            "shops",
            "shops",
            self.shops.shops.iter().map(|item| item.id.as_str()),
        )?;
        validate_catalog_uuids(
            "data/events/shops/base.ron",
            "shops",
            "shops",
            self.shops.shops.iter().map(|item| item.uuid),
        )?;
        validate_catalog_ids(
            "data/events/rewards/base.ron",
            "rewards",
            "rewards",
            self.rewards.rewards.iter().map(|item| item.id.as_str()),
        )?;
        validate_catalog_uuids(
            "data/events/rewards/base.ron",
            "rewards",
            "rewards",
            self.rewards.rewards.iter().map(|item| item.uuid),
        )?;
        validate_catalog_ids(
            "data/events/story/base.ron",
            "events",
            "events",
            self.events.events.iter().map(|item| item.id.as_str()),
        )?;
        validate_catalog_ids(
            "data/visual_novels/base.ron",
            "visual_novels",
            "visual_novels",
            self.visual_novels
                .visual_novels
                .iter()
                .map(|item| item.id.as_str()),
        )?;
        validate_catalog_ids(
            "data/visual_novel_triggers/base.ron",
            "visual_novel_triggers",
            "triggers",
            self.visual_novel_triggers
                .triggers
                .iter()
                .map(|item| item.id.as_str()),
        )?;
        validate_catalog_ids(
            "data/boss_omen/chains.ron",
            "boss_omen_chains",
            "chains",
            self.boss_omen_chains
                .chains
                .iter()
                .map(|item| item.id.as_str()),
        )?;
        validate_catalog_ids(
            "data/abnormalities/base.ron",
            "abnormalities",
            "items",
            self.abnormalities.items.iter().map(|item| item.id.as_str()),
        )?;
        validate_catalog_uuids(
            "data/abnormalities/base.ron",
            "abnormalities",
            "items",
            self.abnormalities.items.iter().map(|item| item.uuid),
        )?;
        validate_catalog_ids(
            "data/enemies/corroded_employees.ron",
            "corroded_employees",
            "range_presets",
            self.corroded_employees
                .range_presets
                .iter()
                .map(|item| item.id.as_str()),
        )?;
        validate_catalog_ids(
            "data/enemies/corroded_employees.ron",
            "corroded_employees",
            "profiles",
            self.corroded_employees
                .profiles
                .iter()
                .map(|item| item.id.as_str()),
        )?;
        validate_catalog_uuids(
            "data/enemies/corroded_employees.ron",
            "corroded_employees",
            "profiles",
            self.corroded_employees
                .profiles
                .iter()
                .map(|item| item.uuid),
        )?;
        validate_catalog_ids(
            "data/enemies/corroded_wave_presets.ron",
            "corroded_wave_presets",
            "presets",
            self.corroded_wave_presets
                .presets
                .iter()
                .map(|item| item.id.as_str()),
        )?;
        validate_catalog_ids(
            "data/employees/definitions.ron",
            "employee_definitions",
            "definitions",
            self.employee_definitions
                .definitions
                .iter()
                .map(|item| item.employee_definition_id.as_str()),
        )?;
        validate_catalog_ids(
            "data/equipments/base.ron",
            "equipment",
            "items",
            self.equipment.items.iter().map(|item| item.id.as_str()),
        )?;
        validate_catalog_uuids(
            "data/equipments/base.ron",
            "equipment",
            "items",
            self.equipment.items.iter().map(|item| item.uuid),
        )?;
        validate_catalog_ids(
            "data/equipments/base.ron",
            "equipment",
            "materials",
            self.equipment.materials.iter().map(|item| item.id.as_str()),
        )?;
        validate_catalog_uuids(
            "data/equipments/base.ron",
            "equipment",
            "materials",
            self.equipment.materials.iter().map(|item| item.uuid),
        )?;
        validate_catalog_ids(
            "data/consumables/base.ron",
            "consumables",
            "items",
            self.consumables.items.iter().map(|item| item.id.as_str()),
        )?;
        validate_catalog_uuids(
            "data/consumables/base.ron",
            "consumables",
            "items",
            self.consumables.items.iter().map(|item| item.uuid),
        )?;
        validate_catalog_ids(
            "data/buffs/base.ron",
            "buffs",
            "buffs",
            self.buffs.buffs.iter().map(|item| item.name.as_str()),
        )?;
        validate_catalog_ids(
            "data/skills/base.ron",
            "skills",
            "range_presets",
            self.skills
                .range_presets
                .iter()
                .map(|item| item.id.as_str()),
        )?;
        validate_catalog_ids(
            "data/skills/base.ron",
            "skills",
            "skills",
            self.skills.skills.iter().map(|item| item.id.as_str()),
        )?;
        validate_catalog_ids(
            "data/skill_fragments/base.ron",
            "skill_fragments",
            "fragments",
            self.skill_fragments
                .fragments
                .iter()
                .map(|item| item.id.as_str()),
        )?;
        validate_catalog_uuids(
            "data/skill_fragments/base.ron",
            "skill_fragments",
            "fragments",
            self.skill_fragments.fragments.iter().map(|item| item.uuid),
        )?;
        validate_catalog_ids(
            "data/pve/encounters.ron",
            "pve_encounters",
            "encounters",
            self.pve_encounters
                .encounters
                .iter()
                .map(|item| item.id.as_str()),
        )
    }

    fn validate_cross_item_uuids(&self) -> Result<(), LiveRonError> {
        let groups = [
            (
                "data/equipments/base.ron",
                "equipment",
                "items",
                self.equipment
                    .items
                    .iter()
                    .map(|item| (item.uuid, item.id.as_str()))
                    .collect::<Vec<_>>(),
            ),
            (
                "data/consumables/base.ron",
                "consumables",
                "items",
                self.consumables
                    .items
                    .iter()
                    .map(|item| (item.uuid, item.id.as_str()))
                    .collect::<Vec<_>>(),
            ),
        ];
        let mut seen = std::collections::HashMap::new();
        for (source_file, domain, collection, items) in groups {
            for (index, (uuid, id)) in items.into_iter().enumerate() {
                if let Some(previous) = seen.insert(uuid, format!("{domain} '{id}'")) {
                    return Err(LiveRonError::validation(
                        source_file,
                        domain,
                        Some(id),
                        Some(format!("{collection}[{index}].uuid")),
                        format!("item uuid '{uuid}' duplicates previously declared {previous}"),
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_reward_effect_contracts(&self) -> Result<(), LiveRonError> {
        for (reward_index, reward) in self.rewards.rewards.iter().enumerate() {
            for (effect_index, effect) in reward.effects.iter().enumerate() {
                self.validate_reward_effect_at(
                    effect,
                    "data/events/rewards/base.ron",
                    "rewards",
                    reward.id.as_str(),
                    format!("rewards[{reward_index}].effects[{effect_index}]"),
                )?;
            }
        }

        for (event_index, event) in self.events.events.iter().enumerate() {
            for (scene_index, scene) in event.scenes.iter().enumerate() {
                let scene_path = format!("events[{event_index}].scenes[{scene_index}]");
                let EventSceneNext::EmployeeCheckChoices { choices, .. } = &scene.next else {
                    continue;
                };
                for (choice_index, choice) in choices.iter().enumerate() {
                    if let Some(EventSuccessAction::Grant { effects }) = &choice.success_action {
                        for (grant_index, reward_effect) in effects.iter().enumerate() {
                            self.validate_reward_effect_at(
                                reward_effect,
                                "data/events/story/base.ron",
                                "events",
                                event.id.as_str(),
                                format!(
                                    "{scene_path}.next.EmployeeCheckChoices.choices[{choice_index}].success_action.Grant.effects[{grant_index}]"
                                ),
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_reward_effect_at(
        &self,
        effect: &RewardEffect,
        source_file: &'static str,
        domain: &'static str,
        object_id: &str,
        field_path: String,
    ) -> Result<(), LiveRonError> {
        super::validation::validate_reward_effect(
            effect,
            &self.rewards,
            &self.equipment,
            &self.consumables,
            &self.skill_fragments,
        )
        .map_err(|cause| {
            LiveRonError::validation(
                source_file,
                domain,
                Some(object_id),
                Some(field_path),
                cause,
            )
        })
    }

    fn validate_run_start_contract(&self) -> Result<(), LiveRonError> {
        for game_mode in [GameMode::Standard, GameMode::Endless] {
            let matching = self
                .visual_novel_triggers
                .triggers
                .iter()
                .filter(|trigger| {
                    matches!(
                        trigger.source,
                        VisualNovelTriggerSource::RunStart {
                            game_mode: trigger_mode
                        } if trigger_mode == game_mode
                    )
                })
                .collect::<Vec<_>>();
            if matching.len() != 1 {
                return Err(LiveRonError::validation(
                    "data/visual_novel_triggers/base.ron",
                    "visual_novel_triggers",
                    None::<String>,
                    Some(format!("triggers.RunStart.{game_mode:?}")),
                    format!(
                        "supported game mode {game_mode:?} requires exactly one RunStart trigger; found {}",
                        matching.len()
                    ),
                ));
            }
            let trigger = matching[0];
            if trigger.repeat != VisualNovelTriggerRepeat::OncePerRun
                || trigger.checkpoint_policy
                    != VisualNovelTriggerCheckpointPolicy::PersistAcrossCheckpointLoad
                || !trigger.blocking
            {
                return Err(LiveRonError::validation(
                    "data/visual_novel_triggers/base.ron",
                    "visual_novel_triggers",
                    Some(trigger.id.as_str()),
                    Some("source.RunStart.policy"),
                    "RunStart trigger must be blocking, OncePerRun, and PersistAcrossCheckpointLoad",
                ));
            }
        }
        Ok(())
    }

    fn validate_map_payload_references(&self) -> Result<(), LiveRonError> {
        for (node_index, definition) in self
            .map_catalogs
            .map_node_definitions
            .nodes
            .iter()
            .enumerate()
        {
            let result = match &definition.payload {
                MapNodePayload::HeadquartersContact {
                    shop_pool_id: Some(pool_id),
                    ..
                } => self
                    .shops
                    .pool_by_id(pool_id)
                    .map(|_| ())
                    .ok_or_else(|| format!("references unknown Shop pool '{pool_id}'")),
                MapNodePayload::Event {
                    event_id: Some(event_id),
                } => self
                    .events
                    .get_by_id(event_id.as_str())
                    .map(|_| ())
                    .ok_or_else(|| format!("references unknown Event '{}'", event_id.as_str())),
                MapNodePayload::Encounter {
                    encounter_id: Some(encounter_id),
                } => self
                    .pve_encounters
                    .get_by_id(encounter_id)
                    .map(|_| ())
                    .ok_or_else(|| format!("references unknown PVE encounter '{encounter_id}'")),
                MapNodePayload::Shop {
                    shop_id: Some(shop_id),
                    ..
                } if self.shops.get_by_id(shop_id).is_none() => {
                    Err(format!("references unknown Shop '{shop_id}'"))
                }
                MapNodePayload::Shop {
                    shop_pool_id: Some(pool_id),
                    ..
                } => self
                    .shops
                    .pool_by_id(pool_id)
                    .map(|_| ())
                    .ok_or_else(|| format!("references unknown Shop pool '{pool_id}'")),
                MapNodePayload::Reward {
                    reward_pool_id: Some(pool_id),
                } => self
                    .rewards
                    .pool_by_id(pool_id)
                    .map(|_| ())
                    .ok_or_else(|| format!("references unknown Reward pool '{pool_id}'")),
                _ => Ok(()),
            };
            result.map_err(|cause| {
                LiveRonError::validation(
                    "data/map/node_definitions.ron",
                    "map_node_definitions",
                    Some(definition.kind_id.as_str()),
                    Some(format!("nodes[{node_index}].payload")),
                    cause,
                )
            })?;
        }
        Ok(())
    }

    fn validate_extra_wave_contracts(&self) -> Result<(), LiveRonError> {
        let defense_route_required = self.pve_encounters.encounters.iter().any(|encounter| {
            encounter.node_type == Some(crate::game::combat_preview::CombatNodeType::Defense)
        });
        for (profile_index, profile) in self
            .run_policy
            .combat_difficulty
            .profiles
            .iter()
            .enumerate()
        {
            let mut profile_wave_ids = HashSet::new();
            let tunings = profile
                .ordinary_bands
                .iter()
                .enumerate()
                .map(|(index, entry)| (format!("ordinary_bands[{index}].tuning"), &entry.tuning))
                .chain(
                    profile
                        .role_tuning
                        .iter()
                        .enumerate()
                        .map(|(index, entry)| {
                            (format!("role_tuning[{index}].tuning"), &entry.tuning)
                        }),
                );
            for (tuning_path, tuning) in tunings {
                for (wave_index, wave) in tuning.extra_waves.iter().enumerate() {
                    let path = format!(
                        "combat_difficulty.profiles[{profile_index}].{tuning_path}.extra_waves[{wave_index}]"
                    );
                    if !profile_wave_ids.insert(wave.id.as_str()) {
                        return Err(LiveRonError::validation(
                            "data/run/policy.ron",
                            "run_policy",
                            Some(wave.id.as_str()),
                            Some(format!("{path}.id")),
                            format!("duplicate difficulty profile extra wave id '{}'", wave.id),
                        ));
                    }
                    if let Some(encounter) =
                        self.pve_encounters.encounters.iter().find(|encounter| {
                            encounter
                                .waves
                                .iter()
                                .any(|encounter_wave| encounter_wave.id == wave.id)
                        })
                    {
                        return Err(LiveRonError::validation(
                            "data/run/policy.ron",
                            "run_policy",
                            Some(wave.id.as_str()),
                            Some(format!("{path}.id")),
                            format!(
                                "extra wave id '{}' collides with pve encounter '{}' wave id",
                                wave.id, encounter.id
                            ),
                        ));
                    }
                    crate::game::data::pve_data::validate_pve_wave_authoring_contract(
                        wave,
                        defense_route_required,
                    )
                    .and_then(|()| {
                        super::validation::validate_pve_wave_references(
                            wave,
                            &self.abnormalities,
                            &self.corroded_employees,
                            &self.corroded_wave_presets,
                        )
                    })
                    .map_err(|cause| {
                        LiveRonError::validation(
                            "data/run/policy.ron",
                            "run_policy",
                            Some(wave.id.as_str()),
                            Some(path),
                            cause,
                        )
                    })?;
                }
            }
        }
        Ok(())
    }

    fn validate_map_difficulty_profile_references(&self) -> Result<(), LiveRonError> {
        for (profile_index, profile) in self
            .map_catalogs
            .node_assignment_profiles
            .profiles
            .iter()
            .enumerate()
        {
            if self
                .run_policy
                .combat_difficulty_profile(&profile.difficulty_profile_id)
                .is_none()
            {
                return Err(LiveRonError::validation(
                    "data/map/node_assignment_profiles.ron",
                    "node_assignment_profiles",
                    Some(profile.id.as_str()),
                    Some(format!("profiles[{profile_index}].difficulty_profile_id")),
                    format!(
                        "references missing combat difficulty profile '{}'",
                        profile.difficulty_profile_id
                    ),
                ));
            }
        }
        Ok(())
    }
}

fn validate_pool_ids<'a>(
    source_file: &'static str,
    domain: &'static str,
    collection: &'static str,
    ids: impl Iterator<Item = &'a str>,
) -> Result<(), LiveRonError> {
    let mut seen = HashSet::new();
    for (index, id) in ids.enumerate() {
        if id.trim().is_empty() {
            return Err(LiveRonError::validation(
                source_file,
                domain,
                None::<String>,
                Some(format!("{collection}[{index}].id")),
                "pool id must not be empty",
            ));
        }
        if !seen.insert(id) {
            return Err(LiveRonError::validation(
                source_file,
                domain,
                Some(id),
                Some(format!("{collection}[{index}].id")),
                format!("duplicate pool id '{id}'"),
            ));
        }
    }
    Ok(())
}

fn validate_catalog_ids<'a>(
    source_file: &'static str,
    domain: &'static str,
    collection: &'static str,
    ids: impl Iterator<Item = &'a str>,
) -> Result<(), LiveRonError> {
    let mut seen = HashSet::new();
    for (index, id) in ids.enumerate() {
        if id.trim().is_empty() {
            return Err(LiveRonError::validation(
                source_file,
                domain,
                None::<String>,
                Some(format!("{collection}[{index}].id")),
                "catalog id must not be empty",
            ));
        }
        if !seen.insert(id) {
            return Err(LiveRonError::validation(
                source_file,
                domain,
                Some(id),
                Some(format!("{collection}[{index}].id")),
                format!("duplicate catalog id '{id}'"),
            ));
        }
    }
    Ok(())
}

fn validate_catalog_uuids(
    source_file: &'static str,
    domain: &'static str,
    collection: &'static str,
    uuids: impl Iterator<Item = uuid::Uuid>,
) -> Result<(), LiveRonError> {
    let mut seen = HashSet::new();
    for (index, uuid) in uuids.enumerate() {
        if !seen.insert(uuid) {
            return Err(LiveRonError::validation(
                source_file,
                domain,
                Some(uuid.to_string()),
                Some(format!("{collection}[{index}].uuid")),
                format!("duplicate catalog uuid '{uuid}'"),
            ));
        }
    }
    Ok(())
}

fn live_domain_error(
    source_file: &'static str,
    domain: &'static str,
    error: super::DataValidationError,
) -> LiveRonError {
    LiveRonError {
        source_file,
        domain,
        object_id: error.object_id,
        field_path: error.field_path,
        cause: error.cause,
    }
}

pub(crate) fn parse_live_ron<T: DeserializeOwned>(
    source_file: &'static str,
    domain: &'static str,
    input: &str,
) -> Result<T, LiveRonError> {
    let mut deserializer = ron::de::Deserializer::from_str(input)
        .map_err(|error| LiveRonError::parse(source_file, domain, None, error.to_string()))?;
    let value = serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
        let path = error.path().to_string();
        LiveRonError::parse(
            source_file,
            domain,
            (!path.is_empty() && path != ".").then_some(path),
            error.inner().to_string(),
        )
    })?;
    deserializer
        .end()
        .map_err(|error| LiveRonError::parse(source_file, domain, None, error.to_string()))?;
    Ok(value)
}

pub(crate) fn validate_live_ron_manifest(input: &str) -> Result<(), LiveRonError> {
    const SOURCE_FILE: &str = "data/bundle.ron";
    const DOMAIN: &str = "bundle_manifest";

    let manifest: LiveRonBundleManifest = parse_live_ron(SOURCE_FILE, DOMAIN, input)?;
    if manifest.schema_version != LIVE_RON_SCHEMA_VERSION {
        return Err(LiveRonError::validation(
            SOURCE_FILE,
            DOMAIN,
            None::<String>,
            Some("schema_version"),
            format!(
                "unsupported schema version {}; expected exactly {}",
                manifest.schema_version, LIVE_RON_SCHEMA_VERSION
            ),
        ));
    }
    Ok(())
}

pub(crate) fn parse_gameplay_ron_sources(
    sources: GameplayRonSourceSet<'_>,
) -> Result<ParsedGameplayRon, LiveRonError> {
    validate_live_ron_manifest(sources.manifest)?;
    let map_catalogs = parse_map_catalogs(sources)?;

    Ok(ParsedGameplayRon {
        shops: parse_live_ron("data/events/shops/base.ron", "shops", sources.shops)?,
        rewards: parse_live_ron("data/events/rewards/base.ron", "rewards", sources.rewards)?,
        events: parse_live_ron("data/events/story/base.ron", "events", sources.events)?,
        visual_novels: parse_live_ron(
            "data/visual_novels/base.ron",
            "visual_novels",
            sources.visual_novels,
        )?,
        visual_novel_triggers: parse_live_ron(
            "data/visual_novel_triggers/base.ron",
            "visual_novel_triggers",
            sources.visual_novel_triggers,
        )?,
        boss_omen_chains: parse_live_ron(
            "data/boss_omen/chains.ron",
            "boss_omen_chains",
            sources.boss_omen_chains,
        )?,
        abnormalities: parse_live_ron(
            "data/abnormalities/base.ron",
            "abnormalities",
            sources.abnormalities,
        )?,
        corroded_employees: parse_live_ron(
            "data/enemies/corroded_employees.ron",
            "corroded_employees",
            sources.corroded_employees,
        )?,
        corroded_wave_presets: parse_live_ron(
            "data/enemies/corroded_wave_presets.ron",
            "corroded_wave_presets",
            sources.corroded_wave_presets,
        )?,
        employee_definitions: parse_live_ron(
            "data/employees/definitions.ron",
            "employee_definitions",
            sources.employee_definitions,
        )?,
        equipment: parse_live_ron("data/equipments/base.ron", "equipment", sources.equipment)?,
        consumables: parse_live_ron(
            "data/consumables/base.ron",
            "consumables",
            sources.consumables,
        )?,
        buffs: parse_live_ron("data/buffs/base.ron", "buffs", sources.buffs)?,
        skills: parse_live_ron("data/skills/base.ron", "skills", sources.skills)?,
        skill_fragments: parse_live_ron(
            "data/skill_fragments/base.ron",
            "skill_fragments",
            sources.skill_fragments,
        )?,
        pve_encounters: parse_live_ron(
            "data/pve/encounters.ron",
            "pve_encounters",
            sources.pve_encounters,
        )?,
        run_policy: parse_live_ron("data/run/policy.ron", "run_policy", sources.run_policy)?,
        map_catalogs,
    })
}

fn parse_map_catalogs(
    sources: GameplayRonSourceSet<'_>,
) -> Result<GameplayMapCatalogs, LiveRonError> {
    Ok(GameplayMapCatalogs {
        map_node_definitions: Arc::new(parse_live_ron(
            "data/map/node_definitions.ron",
            "map_node_definitions",
            sources.map_node_definitions,
        )?),
        facility_templates: Arc::new(parse_live_ron(
            "data/map/facility_templates.ron",
            "facility_templates",
            sources.facility_templates,
        )?),
        node_assignment_profiles: Arc::new(parse_live_ron(
            "data/map/node_assignment_profiles.ron",
            "node_assignment_profiles",
            sources.node_assignment_profiles,
        )?),
        battlefield_archetypes: Arc::new(parse_live_ron(
            "data/map/battlefield_archetypes.ron",
            "battlefield_archetypes",
            sources.battlefield_archetypes,
        )?),
        battlefield_templates: Arc::new(BattlefieldTemplateDatabase {
            templates: parse_live_ron::<Vec<BattlefieldTemplateDefinition>>(
                "data/map/battlefield_templates.ron",
                "battlefield_templates",
                sources.battlefield_templates,
            )?,
        }),
    })
}

pub(crate) fn embedded_map_catalogs_for_fixtures() -> GameplayMapCatalogs {
    static CATALOGS: OnceLock<GameplayMapCatalogs> = OnceLock::new();
    CATALOGS
        .get_or_init(|| {
            let catalogs = parse_map_catalogs(GameplayRonSourceSet::embedded())
                .unwrap_or_else(|error| panic!("invalid embedded map RON fixture: {error}"));
            catalogs
                .validate()
                .unwrap_or_else(|error| panic!("invalid embedded map RON fixture: {error}"));
            catalogs
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use crate::game::battle::damage::DamageType;
    use crate::game::data::event_data::{EventChoiceDefinition, EventChoiceId, EventSuccessAction};

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[allow(dead_code)]
    struct NestedFixture {
        entry: NestedEntry,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[allow(dead_code)]
    struct NestedEntry {
        count: u32,
    }

    fn install_event_check_fixture(parsed: &mut ParsedGameplayRon) -> (usize, usize) {
        let event_index = 0;
        let scene_index = 0;
        let success_scene_id = parsed.events.events[event_index].scenes[1].id.clone();
        let failure_scene_id = parsed.events.events[event_index].scenes[2].id.clone();
        parsed.events.events[event_index].scenes[scene_index].next =
            EventSceneNext::EmployeeCheckChoices {
                choices: vec![EventChoiceDefinition {
                    id: EventChoiceId::new("fixture_choice"),
                    label_id: "fixture_choice_label".to_string(),
                    check: crate::game::data::event_data::EventChoiceCheckDefinition {
                        aptitude: crate::game::employee::EventAptitude::Analysis,
                        dc: 12,
                        required_choice_key: None,
                    },
                    success_action: None,
                    success_scene_id,
                    failure_scene_id,
                }],
            };
        (event_index, scene_index)
    }

    #[test]
    fn bundle_validation_rejects_event_choice_key_without_equipment_provider() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let (event_index, scene_index) = install_event_check_fixture(&mut parsed);
        let EventSceneNext::EmployeeCheckChoices { choices } =
            &mut parsed.events.events[event_index].scenes[scene_index].next
        else {
            unreachable!()
        };
        choices[0].check.required_choice_key = Some(
            crate::game::data::event_data::EventChoiceKeyId::new("missing_archive_key"),
        );

        let error = parsed
            .validate_domain_catalogs()
            .expect_err("an unprovided Event choice key must fail closed");
        assert_eq!(error.source_file, "data/events/story/base.ron");
        assert!(error.cause.contains("no equipment provides"));
    }

    #[test]
    fn bundle_validation_accepts_event_choice_key_with_equipment_provider() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let (event_index, scene_index) = install_event_check_fixture(&mut parsed);
        let key = crate::game::data::event_data::EventChoiceKeyId::new("sealed_archive_key");
        let EventSceneNext::EmployeeCheckChoices { choices } =
            &mut parsed.events.events[event_index].scenes[scene_index].next
        else {
            unreachable!()
        };
        choices[0].check.required_choice_key = Some(key.clone());
        parsed.equipment.items[0].event_choice_keys.push(key);

        parsed
            .validate_domain_catalogs()
            .expect("a provided Event choice key should validate");
    }

    #[test]
    fn manifest_requires_exact_supported_schema_version() {
        validate_live_ron_manifest("(schema_version: 1)")
            .expect("current schema version should load");

        let error = validate_live_ron_manifest("(schema_version: 2)")
            .expect_err("unsupported schema version must reject");
        assert_eq!(error.source_file, "data/bundle.ron");
        assert_eq!(error.domain, "bundle_manifest");
        assert_eq!(error.field_path.as_deref(), Some("schema_version"));
        assert!(error.cause.contains("expected exactly 1"));
    }

    #[test]
    fn manifest_rejects_missing_and_unknown_fields() {
        let missing =
            validate_live_ron_manifest("()").expect_err("missing schema version must reject");
        assert!(missing.cause.contains("missing field `schema_version`"));

        let unknown = validate_live_ron_manifest("(schema_version: 1, legacy_version: 1)")
            .expect_err("unknown manifest field must reject");
        assert!(unknown.cause.contains("unknown field `legacy_version`"));
    }

    #[test]
    fn parser_reports_source_domain_and_nested_field_path() {
        let error = parse_live_ron::<NestedFixture>(
            "data/test.ron",
            "test_domain",
            "(entry: (count: \"not-a-number\"))",
        )
        .expect_err("wrong nested field type must reject");

        assert_eq!(error.source_file, "data/test.ron");
        assert_eq!(error.domain, "test_domain");
        assert_eq!(error.field_path.as_deref(), Some("entry.count"));
        assert!(error.cause.contains("Expected integer"));
    }

    #[test]
    fn embedded_source_set_parses_all_versioned_gameplay_ron() {
        let parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("all embedded gameplay RON sources should parse under schema version 1");

        assert!(!parsed.abnormalities.items.is_empty());
        assert!(!parsed.pve_encounters.encounters.is_empty());
        assert!(!parsed.map_catalogs.map_node_definitions.nodes.is_empty());
        let _ = (
            parsed.shops,
            parsed.rewards,
            parsed.events,
            parsed.visual_novels,
            parsed.visual_novel_triggers,
            parsed.boss_omen_chains,
            parsed.corroded_employees,
            parsed.corroded_wave_presets,
            parsed.employee_definitions,
            parsed.equipment,
            parsed.consumables,
            parsed.buffs,
            parsed.skills,
            parsed.skill_fragments,
            parsed.run_policy,
            parsed.map_catalogs,
        );
        assert_eq!(LIVE_GAMEPLAY_RON_SOURCE_COUNT, 22);
    }

    #[test]
    fn canonical_damage_type_rejects_legacy_aliases() {
        for legacy in ["physical", "ad", "Ad", "magic", "ap", "Ap", "true"] {
            parse_live_ron::<DamageType>("data/test.ron", "damage_type", legacy)
                .expect_err("legacy DamageType spelling must reject");
        }

        assert_eq!(
            parse_live_ron::<DamageType>("data/test.ron", "damage_type", "Physical")
                .expect("canonical DamageType spelling should load"),
            DamageType::Physical
        );
    }

    #[test]
    fn removed_boss_omen_event_effect_variant_rejects_during_parse() {
        parse_live_ron::<EventSuccessAction>(
            "data/events/story/test.ron",
            "events",
            "ApplyBossOmenStepResult",
        )
        .expect_err("removed BossOmen Event action must reject during parse");
    }

    #[test]
    fn source_set_rejects_unknown_nested_field_with_source_context() {
        let mut sources = GameplayRonSourceSet::embedded();
        let corrupted = sources.consumables.replacen(
            "id: \"stabilizing_ampoule\"",
            "id: \"stabilizing_ampoule\", legacy_power: 1",
            1,
        );
        sources.consumables = &corrupted;

        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("unknown nested consumable field must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/consumables/base.ron");
        assert_eq!(error.domain, "consumables");
        assert!(error.cause.contains("unknown field `legacy_power`"));
        assert_eq!(error.field_path.as_deref(), Some("items[0].legacy_power"));
    }

    #[test]
    fn source_set_rejects_unknown_abnormality_movement_and_resonance_fields() {
        for (needle, replacement, unknown_field) in [
            (
                "movement: (speed_units_per_ms: 1450)",
                "movement: (speed_units_per_ms: 1450, legacy_speed: 9999)",
                "legacy_speed",
            ),
            (
                "resonance: (start: 0, max: 100, gain_lock_ms: 1000)",
                "resonance: (start: 0, max: 100, gain_lock_ms: 1000, legacy_gain: 1)",
                "legacy_gain",
            ),
        ] {
            let mut sources = GameplayRonSourceSet::embedded();
            let corrupted = sources.abnormalities.replacen(needle, replacement, 1);
            assert_ne!(corrupted, sources.abnormalities);
            sources.abnormalities = &corrupted;

            let error = match parse_gameplay_ron_sources(sources) {
                Ok(_) => panic!("unknown nested abnormality field must reject: {unknown_field}"),
                Err(error) => error,
            };
            assert_eq!(error.source_file, "data/abnormalities/base.ron");
            assert_eq!(error.domain, "abnormalities");
            assert!(error
                .cause
                .contains(&format!("unknown field `{unknown_field}`")));
        }
    }

    #[test]
    fn source_set_rejects_unknown_skill_range_delivery_and_effect_fields() {
        for (needle, replacement, unknown_field) in [
            (
                "range: (include_anchor_tile: false, rows:",
                "range: (include_anchor_tile: false, legacy_ring: true, rows:",
                "legacy_ring",
            ),
            (
                "delivery: Projectile(speed_units_per_ms: 7500)",
                "delivery: Projectile(speed_units_per_ms: 7500, legacy_guidance: true)",
                "legacy_guidance",
            ),
            (
                "Damage(amount: 60, damage_type: Magic)",
                "Damage(amount: 60, damage_type: Magic, legacy_scale: 2)",
                "legacy_scale",
            ),
        ] {
            let mut sources = GameplayRonSourceSet::embedded();
            let corrupted = sources.skills.replacen(needle, replacement, 1);
            assert_ne!(corrupted, sources.skills);
            sources.skills = &corrupted;

            let error = match parse_gameplay_ron_sources(sources) {
                Ok(_) => panic!("unknown nested skill field must reject: {unknown_field}"),
                Err(error) => error,
            };
            assert_eq!(error.source_file, "data/skills/base.ron");
            assert_eq!(error.domain, "skills");
            assert!(error
                .cause
                .contains(&format!("unknown field `{unknown_field}`")));
        }
    }

    #[test]
    fn source_set_rejects_unknown_equipment_stat_modifier_field() {
        let mut sources = GameplayRonSourceSet::embedded();
        let corrupted = sources.equipment.replacen(
            "StatModifier(\n                        stat: Attack,",
            "StatModifier(\n                        stat: Attack,\n                        legacy_stack: 1,",
            1,
        );
        assert_ne!(corrupted, sources.equipment);
        sources.equipment = &corrupted;

        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("unknown nested equipment stat modifier field must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/equipments/base.ron");
        assert_eq!(error.domain, "equipment");
        assert!(error.cause.contains("unknown field `legacy_stack`"));
    }

    #[test]
    fn source_set_rejects_weapon_profile_missing_block_capacity() {
        let mut sources = GameplayRonSourceSet::embedded();
        let corrupted = sources
            .equipment
            .replacen("                block_capacity: 1,\n", "", 1);
        assert_ne!(corrupted, sources.equipment);
        sources.equipment = &corrupted;

        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("weapon profile missing block_capacity must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/equipments/base.ron");
        assert_eq!(error.domain, "equipment");
        assert!(error.cause.contains("missing field `block_capacity`"));
    }

    #[test]
    fn source_set_rejects_unknown_map_node_payload_field() {
        let mut sources = GameplayRonSourceSet::embedded();
        let corrupted = sources.map_node_definitions.replacen(
            "payload: HeadquartersContact(\n                shop_pool_id: Some(\"headquarters_basic_supplies\"),",
            "payload: HeadquartersContact(\n                legacy_mode: true,\n                shop_pool_id: Some(\"headquarters_basic_supplies\"),",
            1,
        );
        assert_ne!(corrupted, sources.map_node_definitions);
        sources.map_node_definitions = &corrupted;

        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("unknown nested map node payload field must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/map/node_definitions.ron");
        assert_eq!(error.domain, "map_node_definitions");
        assert!(error.cause.contains("unknown field `legacy_mode`"));
    }

    #[test]
    fn source_set_rejects_removed_floor_combat_scaling_authority() {
        let mut sources = GameplayRonSourceSet::embedded();
        let corrupted = sources.run_policy.replacen(
            "    combat_difficulty: (",
            "    floor_combat_scaling: (",
            1,
        );
        assert_ne!(corrupted, sources.run_policy);
        sources.run_policy = &corrupted;

        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("removed floor scaling authority must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/run/policy.ron");
        assert_eq!(error.domain, "run_policy");
        assert!(
            error.cause.contains("floor_combat_scaling")
                || error.cause.contains("combat_difficulty")
        );
    }

    #[test]
    fn source_set_rejects_missing_generated_consumable_field() {
        let mut sources = GameplayRonSourceSet::embedded();
        let corrupted = sources
            .consumables
            .replacen("            live_pool: true,\n", "", 1);
        assert_ne!(corrupted, sources.consumables);
        sources.consumables = &corrupted;

        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("missing generated Consumable live_pool must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/consumables/base.ron");
        assert_eq!(error.domain, "consumables");
        assert_eq!(error.field_path.as_deref(), Some("items[0]"));
        assert!(error.cause.contains("missing field `live_pool`"));
    }

    #[test]
    fn source_set_rejects_missing_required_generated_skill_field() {
        let mut sources = GameplayRonSourceSet::embedded();
        let corrupted =
            sources
                .skills
                .replacen("            name: \"Unknown Distortion Strike\",\n", "", 1);
        assert_ne!(corrupted, sources.skills);
        sources.skills = &corrupted;

        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("missing generated Skill name must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/skills/base.ron");
        assert_eq!(error.domain, "skills");
        assert_eq!(error.field_path.as_deref(), Some("skills[0]"));
        assert!(error.cause.contains("missing field `name`"));
    }

    #[test]
    fn source_set_rejects_missing_required_abnormality_combat_field() {
        let mut sources = GameplayRonSourceSet::embedded();
        let corrupted = sources
            .abnormalities
            .replacen("            magic_resist: 45,\n", "", 1);
        assert_ne!(corrupted, sources.abnormalities);
        sources.abnormalities = &corrupted;

        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("missing generated Abnormality magic_resist must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/abnormalities/base.ron");
        assert_eq!(error.domain, "abnormalities");
        assert_eq!(error.field_path.as_deref(), Some("items[0]"));
        assert!(error.cause.contains("missing field `magic_resist`"));
    }

    #[test]
    fn source_set_rejects_missing_required_corroded_wave_role_field() {
        let mut sources = GameplayRonSourceSet::embedded();
        let corrupted =
            sources
                .corroded_wave_presets
                .replacen("                    weight: 3,\n", "", 1);
        assert_ne!(corrupted, sources.corroded_wave_presets);
        sources.corroded_wave_presets = &corrupted;

        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("missing generated corroded-wave weight must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/enemies/corroded_wave_presets.ron");
        assert_eq!(error.domain, "corroded_wave_presets");
        assert_eq!(error.field_path.as_deref(), Some("presets[0].role_mix[0]"));
        assert!(error.cause.contains("missing field `weight`"));
    }

    #[test]
    fn source_set_rejects_missing_required_pve_enemy_tier() {
        let sources = GameplayRonSourceSet::embedded();
        let pve_encounters = sources.pve_encounters.replacen("tier: I,", "", 1);
        let sources = GameplayRonSourceSet {
            pve_encounters: &pve_encounters,
            ..sources
        };

        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("missing generated PVE enemy tier must reject"),
            Err(error) => error,
        };

        assert_eq!(error.source_file, "data/pve/encounters.ron");
        assert_eq!(error.domain, "pve_encounters");
        assert!(error.cause.contains("missing field `tier`"));
    }

    #[test]
    fn core_only_generated_collections_and_options_are_required() {
        let shop = parse_live_ron::<ShopDatabase>(
            "data/events/shops/test.ron",
            "shops",
            "(merchandise_presets: [], shops: [])",
        )
        .expect_err("generated Shop database must explicitly author pools");
        assert!(shop.cause.contains("missing field `pools`"));

        let reward = parse_live_ron::<RewardDatabase>(
            "data/events/rewards/test.ron",
            "rewards",
            "(rewards: [], pools: [])",
        )
        .expect_err("generated Reward database must explicitly author equipment_pools");
        assert!(reward.cause.contains("missing field `equipment_pools`"));

        let tuning = parse_live_ron::<crate::game::data::run_policy_data::CombatDifficultyTuning>(
            "data/run/test.ron",
            "run_policy",
            "(target_power_percent: 100, max_health_multiplier_percent: 100, attack_multiplier_percent: 100, defense_multiplier_percent: 100, generated_wave_budget_multiplier_percent: 100, reward_multiplier_percent: 100)",
        )
        .expect_err("generated difficulty tuning must explicitly author extra_waves");
        assert!(tuning.cause.contains("missing field `extra_waves`"));

        let omen = parse_live_ron::<crate::game::data::boss_omen_data::BossOmenStep>(
            "data/boss_omen/test.ron",
            "boss_omen_chains",
            "(id: \"step\", source_kind: Event, hint_id: \"hint\", title_id: \"title\", description_id: \"description\", event_id: None)",
        )
        .expect_err("generated BossOmen step must explicitly author encounter_id");
        assert!(omen.cause.contains("missing field `encounter_id`"));
    }

    #[test]
    fn bundle_validation_requires_one_policy_correct_run_start_per_supported_mode() {
        let mut missing = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        missing.visual_novel_triggers.triggers.retain(|trigger| {
            !matches!(
                trigger.source,
                VisualNovelTriggerSource::RunStart {
                    game_mode: GameMode::Endless
                }
            )
        });
        let error = missing
            .validate_domain_catalogs()
            .expect_err("missing Endless RunStart must reject before publication");
        assert_eq!(
            error.field_path.as_deref(),
            Some("triggers.RunStart.Endless")
        );
        assert!(error.cause.contains("found 0"));

        let mut wrong_policy = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let standard = wrong_policy
            .visual_novel_triggers
            .triggers
            .iter_mut()
            .find(|trigger| {
                matches!(
                    trigger.source,
                    VisualNovelTriggerSource::RunStart {
                        game_mode: GameMode::Standard
                    }
                )
            })
            .expect("live Standard RunStart trigger");
        standard.blocking = false;
        let standard_id = standard.id.as_str().to_string();
        let error = wrong_policy
            .validate_domain_catalogs()
            .expect_err("nonblocking RunStart must reject before publication");
        assert_eq!(error.object_id.as_deref(), Some(standard_id.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("source.RunStart.policy"));
    }

    #[test]
    fn bundle_validation_rejects_first_match_pool_duplicates() {
        let mut duplicate_shop = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let repeated = duplicate_shop
            .shops
            .pools
            .first()
            .expect("live Shop data should contain a pool")
            .clone();
        duplicate_shop.shops.pools.push(repeated.clone());
        let error = duplicate_shop
            .validate_domain_catalogs()
            .expect_err("duplicate Shop pool must reject before first-match lookup");
        assert_eq!(error.source_file, "data/events/shops/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(repeated.id.as_str()));
        assert!(error.cause.contains("pool id must be nonempty and unique"));

        let mut duplicate_reward = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let repeated = duplicate_reward
            .rewards
            .pools
            .first()
            .expect("live Reward data should contain a pool")
            .clone();
        duplicate_reward.rewards.pools.push(repeated.clone());
        let error = duplicate_reward
            .validate_domain_catalogs()
            .expect_err("duplicate Reward pool must reject before first-match lookup");
        assert_eq!(error.source_file, "data/events/rewards/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(repeated.id.as_str()));
    }

    #[test]
    fn bundle_validation_rejects_duplicate_catalog_ids_and_cross_item_uuids() {
        let mut duplicate_id = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let repeated_id = duplicate_id.consumables.items[0].id.clone();
        duplicate_id.consumables.items[1].id = repeated_id.clone();
        let error = duplicate_id
            .validate_domain_catalogs()
            .expect_err("duplicate Consumable ID must reject before index construction");
        assert_eq!(error.source_file, "data/consumables/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(repeated_id.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("items[1].id"));
        assert!(error.cause.contains("duplicate catalog id"));

        let mut duplicate_uuid = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let equipment_uuid = duplicate_uuid.equipment.items[0].uuid;
        let consumable_id = duplicate_uuid.consumables.items[0].id.clone();
        duplicate_uuid.consumables.items[0].uuid = equipment_uuid;
        let error = duplicate_uuid
            .validate_domain_catalogs()
            .expect_err("cross-item UUID collision must reject before ItemRegistry construction");
        assert_eq!(error.source_file, "data/consumables/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(consumable_id.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("items[0].uuid"));
        assert!(error
            .cause
            .contains("duplicates previously declared equipment"));
    }

    #[test]
    fn bundle_validation_returns_typed_consumable_and_buff_domain_errors() {
        let mut consumable = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let consumable_id = consumable.consumables.items[0].id.clone();
        consumable.consumables.items[0].effect =
            crate::game::data::consumable_data::ConsumableEffect::TraumaMitigation { percent: 101 };
        let error = consumable
            .validate_domain_catalogs()
            .expect_err("out-of-range Consumable effect must return a typed bundle error");
        assert_eq!(error.source_file, "data/consumables/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(consumable_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("items[0].effect.TraumaMitigation.percent")
        );

        let mut buff = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let buff_name = buff.buffs.buffs[0].name.clone();
        buff.buffs.buffs[0].max_stacks = 0;
        let error = buff
            .validate_domain_catalogs()
            .expect_err("zero-stack Buff must return a typed bundle error");
        assert_eq!(error.source_file, "data/buffs/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(buff_name.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("buffs[0].max_stacks"));
    }

    #[test]
    fn bundle_validation_returns_typed_abnormality_contract_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let abnormality_id = parsed.abnormalities.items[0].id.clone();
        parsed.abnormalities.items[0].basic_attack.interval_ms = 0;
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("zero abnormality attack interval must reject before publication");
        assert_eq!(error.source_file, "data/abnormalities/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(abnormality_id.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("items[0].basic_attack"));
        assert!(error.cause.contains("interval_ms"));
    }

    #[test]
    fn bundle_validation_returns_typed_corroded_employee_contract_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let profile_id = parsed.corroded_employees.profiles[0].id.clone();
        parsed.corroded_employees.profiles[0].max_health = 0;
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("zero corroded employee health must reject before publication");
        assert_eq!(error.source_file, "data/enemies/corroded_employees.ron");
        assert_eq!(error.object_id.as_deref(), Some(profile_id.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("profiles[0].max_health"));
        assert!(error.cause.contains("greater than zero"));
    }

    #[test]
    fn bundle_validation_returns_typed_skill_contract_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let skill_id = parsed.skills.skills[0].id.to_string();
        parsed.skills.skills[0].steps.clear();
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("empty Skill steps must reject before publication");
        assert_eq!(error.source_file, "data/skills/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(skill_id.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("skills[0].steps"));
        assert!(error.cause.contains("at least one step"));
    }

    #[test]
    fn bundle_validation_returns_typed_employee_definition_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let definition_id = parsed.employee_definitions.definitions[0]
            .employee_definition_id
            .clone();
        parsed.employee_definitions.definitions[0]
            .profile
            .role
            .clear();
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("empty definition role must return a typed bundle error");
        assert_eq!(error.source_file, "data/employees/definitions.ron");
        assert_eq!(error.object_id.as_deref(), Some(definition_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("definitions[0].profile.role")
        );
    }

    #[test]
    fn bundle_validation_returns_typed_starter_loadout_reference_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let definition_id = parsed.employee_definitions.definitions[0]
            .employee_definition_id
            .clone();
        parsed.employee_definitions.definitions[0]
            .initial_loadout
            .baseline_skill_fragment_ids[0] =
            crate::game::data::skill_fragment_data::SkillFragmentId::new(
                "__missing_g12_fragment__",
            );
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("missing starter Skill Fragment must reject before publication");
        assert_eq!(error.source_file, "data/employees/definitions.ron");
        assert_eq!(error.object_id.as_deref(), Some(definition_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("definitions[0].initial_loadout.baseline_skill_fragment_ids[0]")
        );
        assert!(error.cause.contains("__missing_g12_fragment__"));
    }

    #[test]
    fn bundle_validation_returns_typed_corroded_wave_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let preset_id = parsed.corroded_wave_presets.presets[0].id.clone();
        parsed.corroded_wave_presets.presets[0].role_mix[0].weight = 0;
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("zero corroded-wave role weight must return a typed bundle error");
        assert_eq!(error.source_file, "data/enemies/corroded_wave_presets.ron");
        assert_eq!(error.object_id.as_deref(), Some(preset_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("presets[0].role_mix[0].weight")
        );
    }

    #[test]
    fn bundle_validation_returns_typed_shop_and_reward_pool_member_errors() {
        let mut shop = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let pool_id = shop.shops.pools[0].id.clone();
        shop.shops.pools[0].shop_ids[0] = "__missing_g12_shop__".to_string();
        let error = shop
            .validate_domain_catalogs()
            .expect_err("missing Shop pool member must return a typed bundle error");
        assert_eq!(error.object_id.as_deref(), Some(pool_id.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("pools[0].shop_ids[0]"));

        let mut reward = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let pool_id = reward.rewards.pools[0].id.clone();
        reward.rewards.pools[0].reward_ids[0] = "__missing_g12_reward__".to_string();
        let error = reward
            .validate_domain_catalogs()
            .expect_err("missing Reward pool member must return a typed bundle error");
        assert_eq!(error.object_id.as_deref(), Some(pool_id.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("pools[0].reward_ids[0]"));
    }

    #[test]
    fn bundle_validation_returns_typed_equipment_reward_pool_reference_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let pool_id = parsed.rewards.equipment_pools[0].id.clone();
        parsed.rewards.equipment_pools[0].entries.push(
            crate::game::data::reward_data::EquipmentRewardPoolEntry {
                equipment_id: "__missing_g12_equipment__".to_string(),
                weight: 1,
            },
        );
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("missing equipment reward-pool target must reject before publication");
        assert_eq!(error.source_file, "data/events/rewards/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(pool_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("equipment_pools[0].entries[0].equipment_id")
        );
        assert!(error.cause.contains("__missing_g12_equipment__"));
    }

    #[test]
    fn bundle_validation_returns_typed_boss_omen_structure_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let chain_id = parsed.boss_omen_chains.chains[0].id.as_str().to_string();
        parsed.boss_omen_chains.chains[0].steps[0].encounter_id =
            Some("__invalid_for_event_step__".to_string());
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("Event BossOmen step with encounter_id must reject");
        assert_eq!(error.source_file, "data/boss_omen/chains.ron");
        assert_eq!(error.object_id.as_deref(), Some(chain_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("chains[0].steps[0].encounter_id")
        );
    }

    #[test]
    fn bundle_validation_returns_typed_boss_omen_reference_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let chain_index = parsed
            .boss_omen_chains
            .chains
            .iter()
            .position(|chain| {
                chain.steps.iter().any(|step| {
                    step.source_kind == crate::game::data::boss_omen_data::BossOmenSourceKind::Event
                })
            })
            .expect("live BossOmen data must contain an Event step");
        let step_index = parsed.boss_omen_chains.chains[chain_index]
            .steps
            .iter()
            .position(|step| {
                step.source_kind == crate::game::data::boss_omen_data::BossOmenSourceKind::Event
            })
            .expect("selected chain must contain an Event step");
        let chain_id = parsed.boss_omen_chains.chains[chain_index]
            .id
            .as_str()
            .to_string();
        parsed.boss_omen_chains.chains[chain_index].steps[step_index].event_id = Some(
            crate::game::data::event_data::EventId::new("__missing_g12_omen_event__"),
        );
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("missing BossOmen Event must reject before publication");
        assert_eq!(error.source_file, "data/boss_omen/chains.ron");
        assert_eq!(error.object_id.as_deref(), Some(chain_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(format!("chains[{chain_index}].steps[{step_index}].event_id").as_str())
        );
        assert!(error.cause.contains("__missing_g12_omen_event__"));
    }

    #[test]
    fn bundle_validation_returns_typed_event_graph_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let event_id = parsed.events.events[0].id.as_str().to_string();
        parsed.events.events[0].scenes[0].presentation.panels[0]
            .image_id
            .clear();
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("empty Event panel image must reject before publication");
        assert_eq!(error.source_file, "data/events/story/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(event_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("events[0].scenes[0].presentation.panels[0].image_id")
        );
    }

    #[test]
    fn bundle_validation_returns_typed_event_combat_reference_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let (event_index, scene_index) = install_event_check_fixture(&mut parsed);
        let event_id = parsed.events.events[event_index].id.as_str().to_string();
        let choices = match &mut parsed.events.events[event_index].scenes[scene_index].next {
            EventSceneNext::EmployeeCheckChoices { choices, .. } => choices,
            _ => unreachable!("choice location was selected above"),
        };
        choices[0].success_action = Some(EventSuccessAction::StartCombat {
            encounter_id: "__missing_g12_encounter__".to_string(),
            primary_abnormality_id: None,
        });
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("missing Event combat encounter must reject before publication");
        assert_eq!(error.source_file, "data/events/story/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(event_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(
                format!(
                    "events[{event_index}].scenes[{scene_index}].next.EmployeeCheckChoices.choices[0].success_action.StartCombat.encounter_id"
                )
                .as_str()
            )
        );
        assert!(error.cause.contains("__missing_g12_encounter__"));
    }

    #[test]
    fn bundle_validation_returns_typed_pve_authoring_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let encounter_id = parsed.pve_encounters.encounters[0].id.clone();
        parsed.pve_encounters.encounters[0].waves.clear();
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("empty PVE wave list must reject before publication");
        assert_eq!(error.source_file, "data/pve/encounters.ron");
        assert_eq!(error.object_id.as_deref(), Some(encounter_id.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("encounters[0].waves"));
        assert!(error.cause.contains("at least one wave"));
    }

    #[test]
    fn bundle_validation_returns_typed_pve_cross_contract_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let encounter_id = parsed.pve_encounters.encounters[0].id.clone();
        parsed.pve_encounters.encounters[0].survive_timer_ms = Some(0);
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("zero PVE survive timer must reject before publication");
        assert_eq!(error.source_file, "data/pve/encounters.ron");
        assert_eq!(error.object_id.as_deref(), Some(encounter_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("encounters[0].survive_timer_ms")
        );
        assert!(error.cause.contains("greater than zero"));
    }

    #[test]
    fn bundle_validation_returns_typed_visual_novel_trigger_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let trigger_id = parsed.visual_novel_triggers.triggers[0]
            .id
            .as_str()
            .to_string();
        parsed.visual_novel_triggers.triggers[0].vn_id =
            crate::game::data::visual_novel_data::VisualNovelId::new("");
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("empty trigger vn_id must reject before publication");
        assert_eq!(error.source_file, "data/visual_novel_triggers/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(trigger_id.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("triggers[0].vn_id"));
        assert!(error.cause.contains("must not be empty"));
    }

    #[test]
    fn bundle_validation_returns_typed_visual_novel_trigger_reference_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let trigger_id = parsed.visual_novel_triggers.triggers[0]
            .id
            .as_str()
            .to_string();
        parsed.visual_novel_triggers.triggers[0].vn_id =
            crate::game::data::visual_novel_data::VisualNovelId::new("__missing_g12_vn__");
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("missing trigger VN must reject before publication");
        assert_eq!(error.source_file, "data/visual_novel_triggers/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(trigger_id.as_str()));
        assert_eq!(error.field_path.as_deref(), Some("triggers[0].vn_id"));
        assert!(error.cause.contains("__missing_g12_vn__"));
    }

    #[test]
    fn bundle_validation_returns_typed_visual_novel_graph_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let visual_novel_id = parsed.visual_novels.visual_novels[0]
            .id
            .as_str()
            .to_string();
        parsed.visual_novels.visual_novels[0].scenes[0]
            .script_id
            .clear();
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("empty Visual Novel script_id must reject before publication");
        assert_eq!(error.source_file, "data/visual_novels/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(visual_novel_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("visual_novels[0].scenes[0].script_id")
        );
        assert!(error.cause.contains("must not be empty"));
    }

    #[test]
    fn bundle_validation_returns_typed_skill_fragment_compatibility_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let fragment_id = parsed.skill_fragments.fragments[0].id.as_str().to_string();
        parsed.skill_fragments.fragments[0]
            .compatibility
            .allowed_damage_types = vec![
            crate::game::battle::damage::DamageType::Physical,
            crate::game::battle::damage::DamageType::Physical,
        ];
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("duplicate compatibility axis must reject before publication");
        assert_eq!(error.source_file, "data/skill_fragments/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(fragment_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("fragments[0].compatibility.allowed_damage_types")
        );
        assert!(error.cause.contains("duplicate"));
    }

    #[test]
    fn bundle_validation_returns_typed_skill_fragment_skill_reference_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let fragment_index =
            parsed
                .skill_fragments
                .fragments
                .iter()
                .position(|fragment| {
                    matches!(
                    fragment.effect,
                    crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
                        ..
                    }
                )
                })
                .expect("live fragments must contain an ActiveSkill effect");
        let fragment_id = parsed.skill_fragments.fragments[fragment_index]
            .id
            .as_str()
            .to_string();
        let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
            imitation_skill_id,
            ..
        } = &mut parsed.skill_fragments.fragments[fragment_index].effect
        else {
            unreachable!("selected fragment must contain ActiveSkill")
        };
        *imitation_skill_id = crate::game::ability::SkillId::from("__missing_g12_skill__");
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("missing Fragment Skill must reject before publication");
        assert_eq!(error.source_file, "data/skill_fragments/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(fragment_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(
                format!("fragments[{fragment_index}].effect.ActiveSkill.imitation_skill_id")
                    .as_str()
            )
        );
        assert!(error.cause.contains("__missing_g12_skill__"));
    }

    #[test]
    fn bundle_validation_returns_typed_equipment_contract_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let item_index = parsed
            .equipment
            .items
            .iter()
            .position(|item| {
                item.equipment_type == crate::game::data::equipment_data::EquipmentType::Weapon
            })
            .expect("live equipment must contain a weapon");
        let item_id = parsed.equipment.items[item_index].id.clone();
        parsed.equipment.items[item_index].weapon_profile = None;
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("weapon without profile must reject before publication");
        assert_eq!(error.source_file, "data/equipments/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(item_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(format!("items[{item_index}].weapon_profile").as_str())
        );
        assert!(error.cause.contains("missing weapon_profile"));
    }

    #[test]
    fn bundle_validation_returns_typed_shop_item_reference_errors() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let shop_id = parsed.shops.shops[0].id.clone();
        let missing_id = "__missing_g28_shop_equipment__";
        parsed.shops.shops[0].local_entries.push(
            crate::game::data::shop_data::ShopMerchandiseEntry {
                merchandise: crate::game::data::shop_data::ShopMerchandise::Equipment {
                    equipment_id: missing_id.to_string(),
                },
                quantity: 1,
                weight: 1,
                repeatable: false,
                buy_percent: None,
                tags: vec![],
            },
        );
        let entries = parsed
            .shops
            .flattened_entries(&parsed.shops.shops[0])
            .expect("mutation remains structurally valid");
        let item_index = entries
            .iter()
            .position(|entry| entry.merchandise.definition_id() == missing_id)
            .unwrap();
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("missing typed Shop merchandise must reject before publication");
        assert_eq!(error.source_file, "data/events/shops/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(shop_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(format!("shops[0].merchandise[{item_index}]").as_str())
        );
        assert!(error.cause.contains(missing_id));
    }

    #[test]
    fn bundle_validation_applies_one_reward_contract_to_reward_and_event_success_action() {
        const MISSING_CONSUMABLE: &str = "__missing_g12_consumable__";
        let invalid_effect = || RewardEffect::GrantConsumable {
            consumable_id: MISSING_CONSUMABLE.to_string(),
            quantity: 1,
        };

        let mut reward = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let reward_id = reward.rewards.rewards[0].id.clone();
        let reward_effect_index = reward.rewards.rewards[0].effects.len();
        reward.rewards.rewards[0].effects.push(invalid_effect());
        let error = reward
            .validate_domain_catalogs()
            .expect_err("invalid Reward record effect must reject before publication");
        assert_eq!(error.source_file, "data/events/rewards/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(reward_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(format!("rewards[0].effects[{reward_effect_index}]").as_str())
        );
        assert!(error.cause.contains(MISSING_CONSUMABLE));

        let mut event = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let (event_index, scene_index) = install_event_check_fixture(&mut event);
        let event_id = event.events.events[event_index].id.as_str().to_string();
        let choices = match &mut event.events.events[event_index].scenes[scene_index].next {
            EventSceneNext::EmployeeCheckChoices { choices, .. } => choices,
            _ => unreachable!("choice location was selected above"),
        };
        choices[0].success_action = Some(EventSuccessAction::Grant {
            effects: vec![invalid_effect()],
        });
        let error = event
            .validate_domain_catalogs()
            .expect_err("invalid Event success grant must reject before publication");
        assert_eq!(error.source_file, "data/events/story/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(event_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(
                format!(
                    "events[{event_index}].scenes[{scene_index}].next.EmployeeCheckChoices.choices[0].success_action.Grant.effects[0]"
                )
                .as_str()
            )
        );
    }

    #[test]
    fn bundle_validation_rejects_zero_consumable_grant_quantity() {
        let mut bundle = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let consumable_id = bundle.consumables.items[0].id.clone();
        let reward_id = bundle.rewards.rewards[0].id.clone();
        let effect_index = bundle.rewards.rewards[0].effects.len();
        bundle.rewards.rewards[0]
            .effects
            .push(RewardEffect::GrantConsumable {
                consumable_id,
                quantity: 0,
            });

        let error = bundle
            .validate_domain_catalogs()
            .expect_err("zero consumable quantity must reject before publication");
        assert_eq!(error.source_file, "data/events/rewards/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(reward_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(format!("rewards[0].effects[{effect_index}]").as_str())
        );
        assert!(error.cause.contains("zero consumable quantity"));
    }

    #[test]
    fn bundle_validation_rejects_broken_explicit_map_payload_references() {
        let mut event = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let (node_index, definition) = Arc::make_mut(&mut event.map_catalogs.map_node_definitions)
            .nodes
            .iter_mut()
            .enumerate()
            .find(|(_, definition)| matches!(definition.payload, MapNodePayload::Event { .. }))
            .expect("live map definitions should contain an Event node");
        let kind_id = definition.kind_id.as_str().to_string();
        definition.payload = MapNodePayload::Event {
            event_id: Some(crate::game::data::event_data::EventId::new(
                "__missing_g12_event__",
            )),
        };
        let error = event
            .validate_domain_catalogs()
            .expect_err("missing explicit map Event must reject before publication");
        assert_eq!(error.source_file, "data/map/node_definitions.ron");
        assert_eq!(error.object_id.as_deref(), Some(kind_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(format!("nodes[{node_index}].payload").as_str())
        );
        assert!(error.cause.contains("__missing_g12_event__"));

        let mut shop_pool = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let definition = Arc::make_mut(&mut shop_pool.map_catalogs.map_node_definitions)
            .nodes
            .iter_mut()
            .find(|definition| matches!(definition.payload, MapNodePayload::Shop { .. }))
            .expect("live map definitions should contain a Shop node");
        definition.payload = MapNodePayload::Shop {
            shop_id: None,
            shop_pool_id: Some("__missing_g12_shop_pool__".to_string()),
        };
        let error = shop_pool
            .validate_domain_catalogs()
            .expect_err("missing explicit map Shop pool must reject before publication");
        assert!(error.cause.contains("__missing_g12_shop_pool__"));
    }

    #[test]
    fn bundle_validation_rejects_missing_map_difficulty_profile_reference() {
        let mut bundle = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let profiles =
            &mut Arc::make_mut(&mut bundle.map_catalogs.node_assignment_profiles).profiles;
        let profile_id = profiles[0].id.clone();
        profiles[0].difficulty_profile_id = "__missing_difficulty_profile__".to_string();

        let error = bundle
            .validate_domain_catalogs()
            .expect_err("missing difficulty profile must reject before publication");
        assert_eq!(error.source_file, "data/map/node_assignment_profiles.ron");
        assert_eq!(error.object_id.as_deref(), Some(profile_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("profiles[0].difficulty_profile_id")
        );
        assert!(error.cause.contains("__missing_difficulty_profile__"));
    }

    #[test]
    fn bundle_validation_applies_encounter_wave_contract_to_every_difficulty_extra_wave() {
        use crate::game::data::pve_data::{PveWaveData, PveWaveEnemyData, PveWaveSource};

        let invalid_wave = |count| PveWaveData {
            id: "g12_invalid_extra_wave".to_string(),
            time_ms: 0,
            spawn_zone_ids: Vec::new(),
            route_id: Some("defense_main".to_string()),
            required_for_victory: true,
            source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                abnormality_id: "__missing_g12_abnormality__".to_string(),
                tier: crate::game::enums::Tier::I,
                count,
            }]),
        };

        let mut broken_reference = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        broken_reference.run_policy.combat_difficulty.profiles[0].ordinary_bands[0]
            .tuning
            .extra_waves
            .push(invalid_wave(1));
        let error = broken_reference
            .validate_domain_catalogs()
            .expect_err("missing extra-wave enemy reference must reject before publication");
        assert_eq!(error.source_file, "data/run/policy.ron");
        assert_eq!(
            error.field_path.as_deref(),
            Some("combat_difficulty.profiles[0].ordinary_bands[0].tuning.extra_waves[0]")
        );
        assert!(error.cause.contains("__missing_g12_abnormality__"));

        let mut invalid_count = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        invalid_count.run_policy.combat_difficulty.profiles[0].ordinary_bands[0]
            .tuning
            .extra_waves
            .push(invalid_wave(0));
        let error = invalid_count
            .validate_domain_catalogs()
            .expect_err("zero-count extra-wave enemy must reject before publication");
        assert!(error.cause.contains("count must be greater than zero"));
    }
}
