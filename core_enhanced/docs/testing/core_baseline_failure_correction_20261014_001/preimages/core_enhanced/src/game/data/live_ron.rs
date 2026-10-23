use std::sync::Arc;
use std::{collections::HashSet, error::Error, fmt};

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
use crate::game::combat_preview::BattlefieldTemplateDefinition;
#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
use std::sync::OnceLock;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::game::data::visual_novel_trigger_data::{
    VisualNovelTriggerCheckpointPolicy, VisualNovelTriggerRepeat, VisualNovelTriggerSource,
};
use crate::game::{
    battle::buffs::BuffDatabase,
    combat_preview::{BattlefieldGenerationPolicyDatabase, BattlefieldTemplateDatabase},
    data::{
        abnormality_data::AbnormalityDatabase,
        boss_omen_data::BossOmenChainDatabase,
        boss_runtime_data::{BossRuntimeCatalogData, BossRuntimeDatabase},
        consumable_data::ConsumableDatabase,
        corroded_employee_data::CorrodedEmployeeProfileDatabase,
        corroded_wave_data::CorrodedWavePresetDatabase,
        employee_data::EmployeeDefinitionDatabase,
        equipment_data::EquipmentDatabase,
        event_data::{EventDatabase, EventSceneNext, EventSuccessAction},
        pve_data::{PveEncounterDatabase, PveWaveEnemyData},
        reward_data::RewardDatabase,
        run_policy_data::RunPolicyData,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
        skill_fragment_data::SkillFragmentDatabase,
        summoned_unit_data::SummonedUnitCatalogData,
        visual_novel_data::VisualNovelDatabase,
        visual_novel_trigger_data::VisualNovelTriggerDatabase,
    },
    map::{
        FacilityTemplateDatabase, GameMode, MapNodeDefinitionDatabase, MapNodePayload,
        NodeAssignmentProfileDatabase,
    },
    reward::RewardEffect,
};

pub const LIVE_RON_SCHEMA_VERSION: u32 = 6;

/// The actual content producer that rejected an input, independent of its
/// human-readable message. Wrappers preserve a producer's phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveRonPhase {
    Acquisition,
    Parse,
    Structure,
    Resolution,
    FinalValidation,
    Compile,
    GeneratedSetup,
}

impl LiveRonPhase {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Acquisition => "content_acquisition",
            Self::Parse => "ron_parse",
            Self::Structure => "content_structure",
            Self::Resolution => "content_resolution",
            Self::FinalValidation => "content_validation",
            Self::Compile => "content_compile",
            Self::GeneratedSetup => "generated_setup",
        }
    }
}

/// One-based position supplied by RON's typed Parser error. Serde Message
/// errors do not supply this precision and leave the position absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RonPosition {
    pub line: usize,
    pub column: usize,
}

/// An authored input that contributes to a cross-domain validation failure.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LiveRonSource {
    pub source_file: String,
    pub object_id: Option<String>,
    pub field_path: Option<String>,
    pub pack_id: Option<String>,
    pub catalog_kind: Option<super::content_pack::CatalogKind>,
    pub definition_kind: Option<super::content_pack::DefinitionKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LiveRonError {
    pub code: &'static str,
    pub phase: LiveRonPhase,
    pub source_file: String,
    pub domain: &'static str,
    pub object_id: Option<String>,
    pub action_id: Option<String>,
    pub effect_id: Option<String>,
    pub field_path: Option<String>,
    pub cause: String,
    pub related_sources: Vec<LiveRonSource>,
    pub pack_id: Option<String>,
    pub catalog_kind: Option<super::content_pack::CatalogKind>,
    pub definition_kind: Option<super::content_pack::DefinitionKind>,
    pub position: Option<RonPosition>,
    pub parser_reason: Option<String>,
    pub expected: Option<serde_json::Value>,
    pub actual: Option<serde_json::Value>,
}

impl LiveRonError {
    pub fn parse(
        source_file: impl Into<String>,
        domain: &'static str,
        field_path: Option<String>,
        cause: impl Into<String>,
    ) -> Self {
        Self {
            code: LiveRonPhase::Parse.code(),
            phase: LiveRonPhase::Parse,
            source_file: source_file.into(),
            domain,
            object_id: None,
            action_id: None,
            effect_id: None,
            field_path,
            cause: cause.into(),
            related_sources: Vec::new(),
            pack_id: None,
            catalog_kind: None,
            definition_kind: None,
            position: None,
            parser_reason: None,
            expected: None,
            actual: None,
        }
    }

    pub fn validation(
        source_file: impl Into<String>,
        domain: &'static str,
        object_id: Option<impl Into<String>>,
        field_path: Option<impl Into<String>>,
        cause: impl Into<String>,
    ) -> Self {
        let mut error = Self::parse(source_file, domain, field_path.map(Into::into), cause)
            .with_phase(LiveRonPhase::FinalValidation);
        error.object_id = object_id.map(Into::into);
        error
    }

    pub(crate) fn with_phase(mut self, phase: LiveRonPhase) -> Self {
        self.phase = phase;
        self.code = phase.code();
        self
    }

    pub(crate) fn with_code(mut self, code: &'static str) -> Self {
        self.code = code;
        self
    }

    pub(crate) fn with_pack(mut self, pack_id: &str) -> Self {
        self.pack_id = Some(pack_id.to_owned());
        self
    }

    pub(crate) fn with_catalog(mut self, kind: super::content_pack::CatalogKind) -> Self {
        self.catalog_kind = Some(kind);
        self
    }

    pub(crate) fn with_definition(mut self, kind: super::content_pack::DefinitionKind) -> Self {
        self.definition_kind = Some(kind);
        self.catalog_kind = Some(kind.catalog());
        self
    }

    fn from_ron(
        source_file: &str,
        domain: &'static str,
        field_path: Option<String>,
        error: &ron::de::Error,
    ) -> Self {
        let mut diagnostic = Self::parse(source_file, domain, field_path, error.to_string());
        if let ron::de::Error::Parser(reason, position) = error {
            // Error::from(Utf8Error) has a sentinel (0, 0), not a source span.
            if position.line != 0 && position.col != 0 {
                diagnostic.position = Some(RonPosition {
                    line: position.line,
                    column: position.col,
                });
            }
            diagnostic.parser_reason = Some(format!("{reason:?}"));
        }
        diagnostic
    }
}

impl fmt::Display for LiveRonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} [{}]", self.source_file, self.domain)?;
        if let Some(object_id) = &self.object_id {
            write!(formatter, " object '{object_id}'")?;
        }
        if let Some(action_id) = &self.action_id {
            write!(formatter, " action '{action_id}'")?;
        }
        if let Some(effect_id) = &self.effect_id {
            write!(formatter, " effect '{effect_id}'")?;
        }
        if let Some(field_path) = &self.field_path {
            write!(formatter, " at {field_path}")?;
        }
        write!(formatter, ": {}", self.cause)?;
        for source in &self.related_sources {
            write!(formatter, "; contributing source {}", source.source_file)?;
            if let Some(object_id) = &source.object_id {
                write!(formatter, " object '{object_id}'")?;
            }
            if let Some(field_path) = &source.field_path {
                write!(formatter, " at {field_path}")?;
            }
        }
        Ok(())
    }
}

impl Error for LiveRonError {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveRonBundleManifest {
    schema_version: u32,
}

macro_rules! gameplay_ron_sources {
    ($( $field:ident => ($logical_path:literal, $embedded_path:literal) ),+ $(,)?) => {
        pub const LIVE_GAMEPLAY_RON_SOURCE_PATHS: &[&str] = &[$($logical_path),+];
        pub const LIVE_GAMEPLAY_RON_SOURCE_COUNT: usize = LIVE_GAMEPLAY_RON_SOURCE_PATHS.len();

        #[derive(Debug, Clone, Copy)]
        #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
        pub(crate) struct GameplayRonSourceSet<'a> {
            pub manifest: &'a str,
            $(pub $field: &'a str),+
        }

        #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
        impl GameplayRonSourceSet<'static> {
            pub(crate) fn embedded() -> Self {
                Self {
                    manifest: include_str!("../../../../game_resources/data/bundle.ron"),
                    $($field: include_str!($embedded_path)),+
                }
            }
        }

        #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
        impl<'a> GameplayRonSourceSet<'a> {
            pub(crate) fn source_inventory(
                &self,
            ) -> [(&'static str, &'a str); LIVE_GAMEPLAY_RON_SOURCE_COUNT] {
                [$(($logical_path, self.$field)),+]
            }
        }
    };
}

gameplay_ron_sources! {
    shops => ("data/events/shops/base.ron", "../../../../game_resources/data/events/shops/base.ron"),
    rewards => ("data/events/rewards/base.ron", "../../../../game_resources/data/events/rewards/base.ron"),
    events => ("data/events/story/base.ron", "../../../../game_resources/data/events/story/base.ron"),
    visual_novels => ("data/visual_novels/base.ron", "../../../../game_resources/data/visual_novels/base.ron"),
    visual_novel_triggers => ("data/visual_novel_triggers/base.ron", "../../../../game_resources/data/visual_novel_triggers/base.ron"),
    boss_omen_chains => ("data/boss_omen/chains.ron", "../../../../game_resources/data/boss_omen/chains.ron"),
    boss_runtime => ("data/boss_runtime/base.ron", "../../../../game_resources/data/boss_runtime/base.ron"),
    summoned_units => ("data/summoned_units/base.ron", "../../../../game_resources/data/summoned_units/base.ron"),
    resources => ("data/resources/base.ron", "../../../../game_resources/data/resources/base.ron"),
    abnormalities => ("data/abnormalities/base.ron", "../../../../game_resources/data/abnormalities/base.ron"),
    corroded_employees => ("data/enemies/corroded_employees.ron", "../../../../game_resources/data/enemies/corroded_employees.ron"),
    corroded_wave_presets => ("data/enemies/corroded_wave_presets.ron", "../../../../game_resources/data/enemies/corroded_wave_presets.ron"),
    employee_definitions => ("data/employees/definitions.ron", "../../../../game_resources/data/employees/definitions.ron"),
    equipment => ("data/equipments/base.ron", "../../../../game_resources/data/equipments/base.ron"),
    consumables => ("data/consumables/base.ron", "../../../../game_resources/data/consumables/base.ron"),
    buffs => ("data/buffs/base.ron", "../../../../game_resources/data/buffs/base.ron"),
    skills => ("data/skills/base.ron", "../../../../game_resources/data/skills/base.ron"),
    skill_fragments => ("data/skill_fragments/base.ron", "../../../../game_resources/data/skill_fragments/base.ron"),
    relics => ("data/relics/base.ron", "../../../../game_resources/data/relics/base.ron"),
    pve_encounters => ("data/pve/encounters.ron", "../../../../game_resources/data/pve/encounters.ron"),
    run_policy => ("data/run/policy.ron", "../../../../game_resources/data/run/policy.ron"),
    map_node_definitions => ("data/map/node_definitions.ron", "../../../../game_resources/data/map/node_definitions.ron"),
    facility_templates => ("data/map/facility_templates.ron", "../../../../game_resources/data/map/facility_templates.ron"),
    node_assignment_profiles => ("data/map/node_assignment_profiles.ron", "../../../../game_resources/data/map/node_assignment_profiles.ron"),
    battlefield_archetypes => ("data/map/battlefield_archetypes.ron", "../../../../game_resources/data/map/battlefield_archetypes.ron"),
    battlefield_templates => ("data/map/battlefield_templates.ron", "../../../../game_resources/data/map/battlefield_templates.ron"),
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
    pub boss_runtime: BossRuntimeCatalogData,
    pub resources: crate::game::resource::ResourceCatalog,
    pub summoned_units: SummonedUnitCatalogData,
    pub abnormalities: AbnormalityDatabase,
    pub corroded_employees: CorrodedEmployeeProfileDatabase,
    pub corroded_wave_presets: CorrodedWavePresetDatabase,
    pub employee_definitions: EmployeeDefinitionDatabase,
    pub equipment: EquipmentDatabase,
    pub consumables: ConsumableDatabase,
    pub buffs: BuffDatabase,
    pub skills: SkillDatabase,
    pub skill_fragments: SkillFragmentDatabase,
    pub relics: super::relic_data::RelicDatabase,
    pub pve_encounters: PveEncounterDatabase,
    pub run_policy: RunPolicyData,
    pub map_catalogs: GameplayMapCatalogs,
}

pub(crate) struct ValidatedGameplayRon {
    pub parsed: ParsedGameplayRon,
    pub boss_runtime: BossRuntimeDatabase,
}

impl ParsedGameplayRon {
    #[cfg(test)]
    pub(crate) fn validate_domain_catalogs(&self) -> Result<(), LiveRonError> {
        self.validate_domain_catalogs_with_compiled_boss().map(drop)
    }

    pub(crate) fn into_validated(self) -> Result<ValidatedGameplayRon, LiveRonError> {
        let boss_runtime = self.validate_domain_catalogs_with_compiled_boss()?;
        Ok(ValidatedGameplayRon {
            parsed: self,
            boss_runtime,
        })
    }

    fn validate_domain_catalogs_with_compiled_boss(
        &self,
    ) -> Result<BossRuntimeDatabase, LiveRonError> {
        self.run_policy.validate_contract().map_err(|cause| {
            LiveRonError::validation(
                "data/run/policy.ron",
                "run_policy",
                None::<String>,
                None::<String>,
                cause,
            )
        })?;
        self.resources.validate().map_err(|error| {
            let object_id = match &error {
                crate::game::resource::ResourceDefinitionError::InitialAboveCap(id)
                | crate::game::resource::ResourceDefinitionError::DuplicateId(id)
                | crate::game::resource::ResourceDefinitionError::OwnerKindMismatch(id) => {
                    Some(id.as_str())
                }
                _ => None,
            };
            LiveRonError::validation(
                "data/resources/base.ron",
                "resources",
                object_id,
                Some("resources"),
                error.to_string(),
            )
            .with_definition(super::content_pack::DefinitionKind::Resource)
        })?;
        self.validate_top_level_catalog_keys()?;
        let compiled_boss_runtime = self.validate_fallible_domain_contracts()?;
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
        self.validate_map_payload_references()?;
        self.validate_fixed_standard_authoring()?;
        self.validate_facility_layout_authoring()?;
        Ok(compiled_boss_runtime)
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

    fn validate_fallible_domain_contracts(&self) -> Result<BossRuntimeDatabase, LiveRonError> {
        self.consumables.validate_contract().map_err(|error| {
            live_domain_error("data/consumables/base.ron", "consumables", error)
        })?;
        self.buffs
            .validate_contract()
            .map_err(|error| live_domain_error("data/buffs/base.ron", "buffs", error))?;
        self.skills
            .validate_contract()
            .map_err(|error| self.skills.live_error("data/skills/base.ron", error))?;
        self.skills
            .validate_buff_references(&self.buffs)
            .map_err(|error| self.skills.live_error("data/skills/base.ron", error))?;
        self.skills
            .validate_resource_references(&self.resources)
            .map_err(|error| self.skills.live_error("data/skills/base.ron", error))?;
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
        let compiled_summoned_units = self.summoned_units.compile().map_err(|error| {
            live_domain_error("data/summoned_units/base.ron", "summoned_units", error)
                .with_phase(LiveRonPhase::Compile)
        })?;
        let compiled_boss_runtime = self
            .boss_runtime
            .compile_with_summoned_unit_database(compiled_summoned_units)
            .map_err(|error| {
                live_domain_error("data/boss_runtime/base.ron", "boss_runtime", error)
                    .with_phase(LiveRonPhase::Compile)
            })?;
        compiled_boss_runtime
            .get_summoned_unit_skill_validation(&self.skills)
            .map_err(|error| {
                live_domain_error("data/summoned_units/base.ron", "summoned_units", error)
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
            &self.skill_fragments,
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
        self.pve_encounters
            .validate_boss_runtime_references(&compiled_boss_runtime)
            .map_err(|error| {
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
        self.relics
            .validate_contract()
            .and_then(|()| {
                self.relics
                    .validate_references(&self.equipment, &self.skill_fragments)
            })
            .map_err(|cause| {
                LiveRonError::validation(
                    "data/relics/base.ron",
                    "relics",
                    None::<String>,
                    Some("relics"),
                    cause,
                )
                .with_definition(super::content_pack::DefinitionKind::Relic)
            })?;
        self.skill_fragments.validate_contract().map_err(|error| {
            live_domain_error("data/skill_fragments/base.ron", "skill_fragments", error)
        })?;
        super::validation::validate_pve_reward_source_references(
            &self.pve_encounters,
            &self.abnormalities,
            &self.skill_fragments,
        )
        .map_err(|error| live_domain_error("data/pve/encounters.ron", "pve_encounters", error))?;
        super::validation::validate_ordinary_reward_source_exclusion(
            &self.rewards,
            &self.pve_encounters,
        )
        .map_err(|error| live_domain_error("data/events/rewards/base.ron", "rewards", error))?;
        super::validation::validate_skill_fragment_skill_references(
            &self.skill_fragments,
            &self.abnormalities,
            &self.skills,
            &self.equipment,
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
        self.equipment
            .validate_effect_and_activation_contract(&self.skills, &self.buffs)
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
        for id in &self.run_policy.tutorial_employee_definition_ids {
            if !self.employee_definitions.get_by_id(id).is_some_and(|definition| definition.starter_eligible) {
                return Err(LiveRonError::validation("data/run/policy.ron", "run_policy", Some(id),
                    Some("tutorial_employee_definition_ids"), "tutorial preset requires a current starter definition"));
            }
        }
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
        Ok(compiled_boss_runtime)
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
        for (encounter_index, encounter) in self.pve_encounters.encounters.iter().enumerate() {
            for (objective_index, objective) in encounter.bonus_objectives.iter().enumerate() {
                for (effect_index, effect) in objective.effects.iter().enumerate() {
                    self.validate_reward_effect_at(effect, "data/pve/encounters.ron", "pve_encounters",
                        &encounter.id, format!("encounters[{encounter_index}].bonus_objectives[{objective_index}].effects[{effect_index}]"))?;
                }
            }
        }
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
            &self.relics,
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
        for game_mode in [GameMode::Tutorial, GameMode::Expedition] {
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

    fn validate_facility_layout_authoring(&self) -> Result<(), LiveRonError> {
        for layout in &self.map_catalogs.facility_templates.layouts {
            let error = |cause: String| LiveRonError::validation("data/map/facility_templates.ron", "facility_layouts", Some(&layout.id), None::<String>, cause);
            if self.run_policy.combat_difficulty_profile(&layout.difficulty_profile_id).is_none() {
                return Err(error("unknown layout difficulty profile".into()));
            }
            for activity in &layout.activities {
                let node = self.map_catalogs.map_node_definitions.nodes.iter().find(|n| n.kind_id == activity.kind_id).ok_or_else(|| error(format!("activity '{}' has unknown node kind",activity.id)))?;
                validate_authored_map_payload(self,&activity.payload).map_err(&error)?;
                if activity.golden_bough_candidate {
                    use crate::game::{map::{MapNodeCategory,MapNodePayload},data::pve_data::PveEncounterClass};
                    let valid = node.category == MapNodeCategory::Boss && activity.condition.is_none()
                        && match &activity.payload {
                            MapNodePayload::Encounter { encounter_id: Some(id) } => self.pve_encounters.get_by_id(id)
                                .is_some_and(|e| e.encounter_class == PveEncounterClass::NormalBoss),
                            MapNodePayload::Encounter { encounter_id: None } => !activity.support.is_supported(),
                            _ => false,
                        };
                    if !valid {return Err(error(format!("activity '{}' is not an immediate normal-boss candidate",activity.id)));}
                }
                if std::mem::discriminant(&node.payload) != std::mem::discriminant(&activity.payload) {
                    return Err(error(format!("activity '{}' payload does not match node kind",activity.id)));
                }
            }
            // Role IDs are authoring choices, but their semantics must match
            // the allocator and the existing difficulty/encounter consumers.
            use crate::game::map::{MapNodeCategory as C, MapNodePayload as P};
            let roles = [
                ("normal", &layout.allocation.normal_activity_id),
                ("elite", &layout.allocation.elite_activity_id),
                ("event", &layout.allocation.event_activity_id),
                ("reward", &layout.allocation.reward_activity_id),
                ("gate", &layout.allocation.gate_activity_id),
            ];
            for (role, id) in roles.into_iter().chain(layout.allocation.support_activities.keys().map(|id| ("support", id))) {
                let a = layout.activity(id).expect("allocation references validated");
                let node = self.map_catalogs.map_node_definitions.nodes.iter().find(|n| n.kind_id == a.kind_id).expect("kind validated");
                let valid = match (role, node.category, &a.payload) {
                    ("normal", C::Combat, P::Encounter { .. }) => !a.kind_id.as_str().contains("elite"),
                    ("elite", C::Combat, P::Encounter { .. }) => a.kind_id.as_str() == "combat_elite",
                    ("event", C::Event, P::Event { .. }) | ("reward", C::Reward, P::Reward { .. }) |
                    ("gate", C::Gate, P::None) | ("support", C::Maintenance, P::Maintenance) |
                    ("support", C::Shop, P::Shop { .. }) | ("support", C::HeadquartersContact, P::HeadquartersContact { .. }) => true,
                    _ => false,
                };
                let encounter_class_matches = match (&a.payload, role) {
                    (P::Encounter { encounter_id: Some(id) }, "normal" | "elite") => {
                        use crate::game::data::pve_data::PveEncounterClass;
                        self.pve_encounters.encounters.iter().find(|e| &e.id == id).is_some_and(|e|
                            e.encounter_class == if role == "normal" { PveEncounterClass::Normal } else { PveEncounterClass::Elite })
                    }
                    _ => true,
                };
                if !valid || !encounter_class_matches || (role != "reward" && a.reward_activity.is_some()) {
                    return Err(error(format!("allocation role '{}' activity '{}' has incompatible semantics", role, id)));
                }
            }
            for room in &layout.rooms {
                if let Some(template) = room.template_id.as_ref().and_then(|id| self.map_catalogs.facility_templates.template(id)) {
                    // Uniform allocation uses every free position. Reject a authored
                    // template/category mismatch before it can reach generation.
                    for slot in &template.slots {
                        let reservation=room.reservations.iter().find(|r|r.slot_id==slot.id);
                        let mut possible=Vec::new();
                        match slot.role {
                            crate::game::map::FacilitySlotRole::Start => {},
                            crate::game::map::FacilitySlotRole::Terminal(_) => possible.push(&layout.allocation.gate_activity_id),
                            _ if reservation.is_some() => {
                                let id=&reservation.unwrap().activity_id;
                                if room.final_room && layout.allocation.support_activities.contains_key(id) {possible.extend(layout.allocation.support_activities.keys());}
                            },
                            _ => {
                                possible.push(&layout.allocation.normal_activity_id);
                                if room.omen_slot.as_ref()!=Some(&slot.id) {
                                    possible.push(&layout.allocation.event_activity_id);
                                    possible.push(&layout.allocation.reward_activity_id);
                                    possible.extend(layout.allocation.support_activities.keys());
                                    if room.elite_slots.contains(&slot.id) {possible.push(&layout.allocation.elite_activity_id);}
                                }
                            },
                        }
                        for id in possible {
                            let activity=layout.activity(id).expect("allocation references validated");
                            let node=self.map_catalogs.map_node_definitions.nodes.iter().find(|n|n.kind_id==activity.kind_id).expect("kind validated");
                            if !slot.allowed_categories.contains(&node.category) {return Err(error(format!("room '{}' slot '{}' rejects a possible allocated category",room.id,slot.id.as_str())));}
                        }
                    }
                    for reservation in &room.reservations {
                        let activity = layout.activity(&reservation.activity_id).expect("layout references validated");
                        let node = self.map_catalogs.map_node_definitions.nodes.iter().find(|n| n.kind_id == activity.kind_id).expect("kind validated");
                        if template.slots.iter().find(|s| s.id == reservation.slot_id).is_none_or(|s| !s.allowed_categories.contains(&node.category)) {
                            return Err(error(format!("room '{}' reserved slot rejects activity category",room.id)));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_fixed_standard_authoring(&self) -> Result<(), LiveRonError> {
        for (profile_index, profile) in self
            .map_catalogs
            .node_assignment_profiles
            .profiles
            .iter()
            .enumerate()
        {
            for (assignment_index, assignment) in profile.fixed_slot_assignments.iter().enumerate()
            {
                let definition = self
                    .map_catalogs
                    .map_node_definitions
                    .nodes
                    .iter()
                    .find(|definition| definition.kind_id == assignment.kind_id)
                    .ok_or_else(|| {
                        LiveRonError::validation(
                            "data/map/node_assignment_profiles.ron",
                            "node_assignment_profiles",
                            Some(profile.id.as_str()),
                            Some(format!(
                                "profiles[{profile_index}].fixed_slot_assignments[{assignment_index}].kind_id"
                            )),
                            format!(
                                "references unknown node kind '{}'",
                                assignment.kind_id.as_str()
                            ),
                        )
                    })?;
                validate_authored_map_payload(self, &assignment.payload).map_err(|cause| {
                    LiveRonError::validation(
                        "data/map/node_assignment_profiles.ron",
                        "node_assignment_profiles",
                        Some(profile.id.as_str()),
                        Some(format!(
                            "profiles[{profile_index}].fixed_slot_assignments[{assignment_index}].payload"
                        )),
                        cause,
                    )
                })?;
                if std::mem::discriminant(&definition.payload)
                    != std::mem::discriminant(&assignment.payload)
                    && !matches!(
                        (&definition.payload, &assignment.payload),
                        (MapNodePayload::None, MapNodePayload::None)
                    )
                {
                    return Err(LiveRonError::validation(
                        "data/map/node_assignment_profiles.ron",
                        "node_assignment_profiles",
                        Some(profile.id.as_str()),
                        Some(format!(
                            "profiles[{profile_index}].fixed_slot_assignments[{assignment_index}].payload"
                        )),
                        format!(
                            "payload variant does not match node kind '{}'",
                            assignment.kind_id.as_str()
                        ),
                    ));
                }
                if let Some(combat) = &assignment.authored_combat {
                    if !matches!(assignment.payload, MapNodePayload::Encounter { .. }) {
                        return Err(LiveRonError::validation(
                            "data/map/node_assignment_profiles.ron",
                            "node_assignment_profiles",
                            Some(profile.id.as_str()),
                            Some(format!(
                                "profiles[{profile_index}].fixed_slot_assignments[{assignment_index}].authored_combat"
                            )),
                            "authored combat policy requires an Encounter payload",
                        ));
                    }
                    let encounter_id = match &assignment.payload {
                        MapNodePayload::Encounter {
                            encounter_id: Some(encounter_id),
                        } => encounter_id,
                        _ => continue,
                    };
                    let encounter = self
                        .pve_encounters
                        .get_by_id(encounter_id)
                        .expect("payload reference was validated above");
                    let declared_outcomes = encounter
                        .terminal_outcomes
                        .iter()
                        .map(|outcome| outcome.id.as_str())
                        .collect::<std::collections::HashSet<_>>();
                    let authored_outcomes = combat
                        .outcome_policies
                        .iter()
                        .map(|policy| policy.outcome_id.as_str())
                        .collect::<std::collections::HashSet<_>>();
                    if authored_outcomes != declared_outcomes
                        || authored_outcomes.len() != combat.outcome_policies.len()
                    {
                        return Err(LiveRonError::validation(
                            "data/map/node_assignment_profiles.ron",
                            "node_assignment_profiles",
                            Some(profile.id.as_str()),
                            Some(format!(
                                "profiles[{profile_index}].fixed_slot_assignments[{assignment_index}].authored_combat.outcome_policies"
                            )),
                            format!(
                                "authored outcome policies must cover encounter '{}' outcomes exactly once",
                                encounter_id
                            ),
                        ));
                    }
                    for (policy_index, policy) in combat.outcome_policies.iter().enumerate() {
                        for reward_id in &policy.reward_ids {
                            if self.rewards.get_by_id(reward_id).is_none() {
                                return Err(LiveRonError::validation(
                                    "data/map/node_assignment_profiles.ron",
                                    "node_assignment_profiles",
                                    Some(profile.id.as_str()),
                                    Some(format!(
                                        "profiles[{profile_index}].fixed_slot_assignments[{assignment_index}].authored_combat.outcome_policies[{policy_index}].reward_ids"
                                    )),
                                    format!("references unknown Reward '{reward_id}'"),
                                ));
                            }
                        }
                        let verdict = encounter
                            .terminal_outcomes
                            .iter()
                            .find(|outcome| outcome.id == policy.outcome_id)
                            .map(|outcome| outcome.verdict)
                            .expect("exhaustive outcome identity was checked");
                        if verdict == crate::game::battle::types::BattleVerdict::Defeat
                            && (!policy.reward_ids.is_empty()
                                || policy.grant_node_bonus
                                || policy.advance_encounter_progression)
                        {
                            return Err(LiveRonError::validation(
                                "data/map/node_assignment_profiles.ron",
                                "node_assignment_profiles",
                                Some(profile.id.as_str()),
                                Some(format!(
                                    "profiles[{profile_index}].fixed_slot_assignments[{assignment_index}].authored_combat.outcome_policies[{policy_index}]"
                                )),
                                "Defeat outcome policy cannot grant rewards, node bonus, or progression",
                            ));
                        }
                    }
                    for stat_override in &combat.actor_stat_overrides {
                        if self
                            .abnormalities
                            .get_by_id(&stat_override.abnormality_id)
                            .is_none()
                            || !encounter.waves.iter().any(|wave| {
                                wave.manual_enemies().iter().any(|entry| {
                                    matches!(
                                        entry,
                                        PveWaveEnemyData::Abnormality { abnormality_id, .. }
                                            if abnormality_id == &stat_override.abnormality_id
                                    )
                                })
                            })
                        {
                            return Err(LiveRonError::validation(
                                "data/map/node_assignment_profiles.ron",
                                "node_assignment_profiles",
                                Some(profile.id.as_str()),
                                Some(format!(
                                    "profiles[{profile_index}].fixed_slot_assignments[{assignment_index}].authored_combat.actor_stat_overrides"
                                )),
                                format!(
                                    "actor override '{}' is not an abnormality actor in encounter '{}'",
                                    stat_override.abnormality_id, encounter_id
                                ),
                            ));
                        }
                    }
                }
            }
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

fn validate_authored_map_payload(
    catalogs: &ParsedGameplayRon,
    payload: &MapNodePayload,
) -> Result<(), String> {
    match payload {
        MapNodePayload::HeadquartersContact {
            shop_pool_id: Some(pool_id),
            ..
        } => catalogs
            .shops
            .pool_by_id(pool_id)
            .map(|_| ())
            .ok_or_else(|| format!("references unknown Shop pool '{pool_id}'")),
        MapNodePayload::Event {
            event_id: Some(event_id),
        } => catalogs
            .events
            .get_by_id(event_id.as_str())
            .map(|_| ())
            .ok_or_else(|| format!("references unknown Event '{}'", event_id.as_str())),
        MapNodePayload::Encounter {
            encounter_id: Some(encounter_id),
        } => catalogs
            .pve_encounters
            .get_by_id(encounter_id)
            .map(|_| ())
            .ok_or_else(|| format!("references unknown PVE encounter '{encounter_id}'")),
        MapNodePayload::Shop {
            shop_id: Some(shop_id),
            ..
        } if catalogs.shops.get_by_id(shop_id).is_none() => {
            Err(format!("references unknown Shop '{shop_id}'"))
        }
        MapNodePayload::Shop {
            shop_pool_id: Some(pool_id),
            ..
        } => catalogs
            .shops
            .pool_by_id(pool_id)
            .map(|_| ())
            .ok_or_else(|| format!("references unknown Shop pool '{pool_id}'")),
        MapNodePayload::Reward {
            reward_pool_id: Some(pool_id),
        } => catalogs
            .rewards
            .pool_by_id(pool_id)
            .map(|_| ())
            .ok_or_else(|| format!("references unknown Reward pool '{pool_id}'")),
        _ => Ok(()),
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
    LiveRonError::validation(
        source_file,
        domain,
        error.object_id,
        error.field_path,
        error.cause,
    )
}

pub(crate) fn parse_live_ron<T: DeserializeOwned>(
    source_file: &str,
    domain: &'static str,
    input: &str,
) -> Result<T, LiveRonError> {
    let mut deserializer = ron::de::Deserializer::from_str(input)
        .map_err(|error| LiveRonError::from_ron(source_file, domain, None, &error))?;
    let value = serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
        let path = error.path().to_string();
        LiveRonError::from_ron(
            source_file,
            domain,
            (!path.is_empty() && path != ".").then_some(path),
            error.inner(),
        )
    })?;
    deserializer
        .end()
        .map_err(|error| LiveRonError::from_ron(source_file, domain, None, &error))?;
    Ok(value)
}

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
fn parse_live_skill_database(input: &str) -> Result<SkillDatabase, LiveRonError> {
    const SOURCE_FILE: &str = "data/skills/base.ron";
    SkillDatabase::from_ron(SOURCE_FILE, input)
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

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
pub(crate) fn parse_gameplay_ron_sources(
    sources: GameplayRonSourceSet<'_>,
) -> Result<ParsedGameplayRon, LiveRonError> {
    validate_gameplay_source_inventory(sources)?;
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
        boss_runtime: parse_live_ron(
            "data/boss_runtime/base.ron",
            "boss_runtime",
            sources.boss_runtime,
        )?,
        resources: parse_live_ron("data/resources/base.ron", "resources", sources.resources)?,
        summoned_units: parse_live_ron(
            "data/summoned_units/base.ron",
            "summoned_units",
            sources.summoned_units,
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
        skills: parse_live_skill_database(sources.skills)?,
        skill_fragments: parse_live_ron(
            "data/skill_fragments/base.ron",
            "skill_fragments",
            sources.skill_fragments,
        )?,
        relics: parse_live_ron("data/relics/base.ron", "relics", sources.relics)?,
        pve_encounters: parse_live_ron(
            "data/pve/encounters.ron",
            "pve_encounters",
            sources.pve_encounters,
        )?,
        run_policy: parse_live_ron("data/run/policy.ron", "run_policy", sources.run_policy)?,
        map_catalogs,
    })
}

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
fn validate_gameplay_source_inventory(
    sources: GameplayRonSourceSet<'_>,
) -> Result<(), LiveRonError> {
    let mut paths = HashSet::new();
    for (path, source) in sources.source_inventory() {
        if !paths.insert(path) {
            return Err(LiveRonError::validation(
                "data/bundle.ron",
                "source_inventory",
                None::<String>,
                Some(path),
                "duplicate gameplay RON source path",
            ));
        }
        if source.trim().is_empty() {
            return Err(LiveRonError::validation(
                path,
                "source_inventory",
                None::<String>,
                None::<String>,
                "embedded gameplay RON source is empty",
            ));
        }
    }
    Ok(())
}

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
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

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
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
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::game::ability::{
        DeliveryDef, ProjectileHitPolicy, SkillHitTargetFilter, SkillProjectileCollisionDef,
        SkillTileAreaDeliveryDef,
    };
    use crate::game::battle::damage::DamageType;
    use crate::game::battle::tile_range::TileRangePattern;
    use crate::game::data::event_data::{EventChoiceDefinition, EventChoiceId, EventSuccessAction};

    fn assert_ron_equal<T: Serialize>(label: &str, parsed: &T, published: &T) {
        assert_eq!(
            serde_json::to_value(parsed).expect("parsed catalog should serialize"),
            serde_json::to_value(published).expect("published catalog should serialize"),
            "parsed-to-published mismatch for {label}"
        );
    }

    fn assert_publications_semantically_equal(
        left: &crate::game::data::ValidatedGameData,
        right: &crate::game::data::ValidatedGameData,
    ) {
        assert_ron_equal("shops", left.shop_data.as_ref(), right.shop_data.as_ref());
        assert_ron_equal(
            "rewards",
            left.reward_data.as_ref(),
            right.reward_data.as_ref(),
        );
        assert_ron_equal(
            "events",
            left.event_data.as_ref(),
            right.event_data.as_ref(),
        );
        assert_ron_equal(
            "visual_novels",
            left.visual_novel_data.as_ref(),
            right.visual_novel_data.as_ref(),
        );
        assert_ron_equal(
            "visual_novel_triggers",
            left.visual_novel_trigger_data.as_ref(),
            right.visual_novel_trigger_data.as_ref(),
        );
        assert_ron_equal(
            "boss_omen_chains",
            left.boss_omen_data.as_ref(),
            right.boss_omen_data.as_ref(),
        );
        assert_ron_equal(
            "abnormalities",
            left.abnormality_data.as_ref(),
            right.abnormality_data.as_ref(),
        );
        assert_ron_equal(
            "corroded_employees",
            left.corroded_employee_data.as_ref(),
            right.corroded_employee_data.as_ref(),
        );
        assert_ron_equal(
            "corroded_wave_presets",
            left.corroded_wave_data.as_ref(),
            right.corroded_wave_data.as_ref(),
        );
        assert_ron_equal(
            "employee_definitions",
            left.employee_definition_data.as_ref(),
            right.employee_definition_data.as_ref(),
        );
        assert_ron_equal(
            "equipment",
            left.equipment_data.as_ref(),
            right.equipment_data.as_ref(),
        );
        assert_ron_equal(
            "consumables",
            left.consumable_data.as_ref(),
            right.consumable_data.as_ref(),
        );
        assert_ron_equal("buffs", left.buff_data.as_ref(), right.buff_data.as_ref());
        assert_ron_equal(
            "skills",
            left.skill_data.as_ref(),
            right.skill_data.as_ref(),
        );
        assert_ron_equal(
            "skill_fragments",
            left.skill_fragment_data.as_ref(),
            right.skill_fragment_data.as_ref(),
        );
        assert_ron_equal("pve", left.pve_data.as_ref(), right.pve_data.as_ref());
        assert_ron_equal(
            "run_policy",
            left.run_policy.as_ref(),
            right.run_policy.as_ref(),
        );
        assert_ron_equal(
            "map_node_definitions",
            left.map_catalogs.map_node_definitions.as_ref(),
            right.map_catalogs.map_node_definitions.as_ref(),
        );
        assert_ron_equal(
            "facility_templates",
            left.map_catalogs.facility_templates.as_ref(),
            right.map_catalogs.facility_templates.as_ref(),
        );
        assert_ron_equal(
            "node_assignment_profiles",
            left.map_catalogs.node_assignment_profiles.as_ref(),
            right.map_catalogs.node_assignment_profiles.as_ref(),
        );
        assert_ron_equal(
            "battlefield_archetypes",
            left.map_catalogs.battlefield_archetypes.as_ref(),
            right.map_catalogs.battlefield_archetypes.as_ref(),
        );
        assert_ron_equal(
            "battlefield_templates",
            left.map_catalogs.battlefield_templates.as_ref(),
            right.map_catalogs.battlefield_templates.as_ref(),
        );
        assert_eq!(
            left.boss_runtime_data.pattern_count(),
            right.boss_runtime_data.pattern_count()
        );
        assert_eq!(
            left.boss_runtime_data.actor_behavior_count(),
            right.boss_runtime_data.actor_behavior_count()
        );
        assert_eq!(
            left.boss_runtime_data.encounter_count(),
            right.boss_runtime_data.encounter_count()
        );
    }

    #[test]
    fn live_publication_preserves_every_parsed_catalog_and_derived_item_registry() {
        let parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("embedded source set should parse");
        parsed
            .validate_domain_catalogs()
            .expect("embedded parsed catalogs should validate");
        let published = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("embedded catalogs should publish");

        assert_ron_equal("shops", &parsed.shops, published.shop_data.as_ref());
        assert_ron_equal("rewards", &parsed.rewards, published.reward_data.as_ref());
        assert_ron_equal("events", &parsed.events, published.event_data.as_ref());
        assert_ron_equal(
            "visual_novels",
            &parsed.visual_novels,
            published.visual_novel_data.as_ref(),
        );
        assert_ron_equal(
            "visual_novel_triggers",
            &parsed.visual_novel_triggers,
            published.visual_novel_trigger_data.as_ref(),
        );
        assert_ron_equal(
            "boss_omen_chains",
            &parsed.boss_omen_chains,
            published.boss_omen_data.as_ref(),
        );
        assert_ron_equal(
            "abnormalities",
            &parsed.abnormalities,
            published.abnormality_data.as_ref(),
        );
        assert_ron_equal(
            "corroded_employees",
            &parsed.corroded_employees,
            published.corroded_employee_data.as_ref(),
        );
        assert_ron_equal(
            "corroded_wave_presets",
            &parsed.corroded_wave_presets,
            published.corroded_wave_data.as_ref(),
        );
        assert_ron_equal(
            "employee_definitions",
            &parsed.employee_definitions,
            published.employee_definition_data.as_ref(),
        );
        assert_ron_equal(
            "equipment",
            &parsed.equipment,
            published.equipment_data.as_ref(),
        );
        assert_ron_equal(
            "consumables",
            &parsed.consumables,
            published.consumable_data.as_ref(),
        );
        assert_ron_equal("buffs", &parsed.buffs, published.buff_data.as_ref());
        assert_ron_equal("skills", &parsed.skills, published.skill_data.as_ref());
        assert_ron_equal(
            "skill_fragments",
            &parsed.skill_fragments,
            published.skill_fragment_data.as_ref(),
        );
        assert_ron_equal(
            "pve_encounters",
            &parsed.pve_encounters,
            published.pve_data.as_ref(),
        );
        assert_ron_equal(
            "run_policy",
            &parsed.run_policy,
            published.run_policy.as_ref(),
        );
        assert_ron_equal(
            "map_node_definitions",
            parsed.map_catalogs.map_node_definitions.as_ref(),
            published.map_catalogs.map_node_definitions.as_ref(),
        );
        assert_ron_equal(
            "facility_templates",
            parsed.map_catalogs.facility_templates.as_ref(),
            published.map_catalogs.facility_templates.as_ref(),
        );
        assert_ron_equal(
            "node_assignment_profiles",
            parsed.map_catalogs.node_assignment_profiles.as_ref(),
            published.map_catalogs.node_assignment_profiles.as_ref(),
        );
        assert_ron_equal(
            "battlefield_archetypes",
            parsed.map_catalogs.battlefield_archetypes.as_ref(),
            published.map_catalogs.battlefield_archetypes.as_ref(),
        );
        assert_ron_equal(
            "battlefield_templates",
            parsed.map_catalogs.battlefield_templates.as_ref(),
            published.map_catalogs.battlefield_templates.as_ref(),
        );

        assert_eq!(
            published.boss_runtime_data.pattern_count(),
            parsed.boss_runtime.patterns.len()
        );
        assert_eq!(
            published.boss_runtime_data.actor_behavior_count(),
            parsed.boss_runtime.actor_behaviors.len()
        );
        assert_eq!(
            published.boss_runtime_data.encounter_count(),
            parsed.boss_runtime.encounters.len()
        );
        for pattern in &parsed.boss_runtime.patterns {
            assert_eq!(
                published
                    .boss_runtime_data
                    .get_pattern(&pattern.id)
                    .map(|compiled| &compiled.id),
                Some(&pattern.id)
            );
        }
        for behavior in &parsed.boss_runtime.actor_behaviors {
            assert_eq!(
                published
                    .boss_runtime_data
                    .get_actor_behavior(&behavior.id)
                    .map(|compiled| &compiled.id),
                Some(&behavior.id)
            );
        }
        for encounter in &parsed.boss_runtime.encounters {
            assert_eq!(
                published
                    .boss_runtime_data
                    .get_encounter(&encounter.id)
                    .map(|compiled| &compiled.id),
                Some(&encounter.id)
            );
        }

        for expected in &parsed.consumables.items {
            let Some(super::super::ItemRef::Consumable(actual)) = published.item(&expected.uuid)
            else {
                panic!("published item registry lost consumable {}", expected.id);
            };
            assert_ron_equal("item_registry.consumable", expected, actual);
        }
        for expected in &parsed.equipment.items {
            let Some(super::super::ItemRef::Equipment(actual)) = published.item(&expected.uuid)
            else {
                panic!("published item registry lost equipment {}", expected.id);
            };
            assert_ron_equal("item_registry.equipment", expected, actual);
        }
        for abnormality in &parsed.abnormalities.items {
            assert!(published.item(&abnormality.uuid).is_none());
        }
    }

    #[test]
    fn successful_live_publication_compiles_boss_runtime_exactly_once() {
        let before = crate::game::data::boss_runtime_data::BossRuntimeCatalogData::
            compile_count_for_current_thread();
        crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("embedded catalogs should publish");
        let after = crate::game::data::boss_runtime_data::BossRuntimeCatalogData::
            compile_count_for_current_thread();

        assert_eq!(
            after - before,
            1,
            "production must compile Boss Runtime once"
        );
    }

    #[test]
    fn repeated_valid_and_invalid_publication_attempts_are_idempotent() {
        let first = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("first embedded publication must succeed");
        let second = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("second embedded publication must succeed");
        assert_publications_semantically_equal(&first, &second);

        let mut invalid_sources = GameplayRonSourceSet::embedded();
        invalid_sources.manifest = "(schema_version: 1)";
        let first_error = match crate::game::data::GameDataBase::load_live_sources(invalid_sources)
        {
            Ok(_) => panic!("unsupported manifest must fail before publication"),
            Err(error) => error,
        };
        let second_error = match crate::game::data::GameDataBase::load_live_sources(invalid_sources)
        {
            Ok(_) => panic!("identical unsupported manifest must fail identically"),
            Err(error) => error,
        };
        assert_eq!(first_error, second_error);

        let after_failure = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("failed candidates must not alter later official publication");
        assert_publications_semantically_equal(&first, &after_failure);
    }

    #[test]
    fn effect_authoring_failures_never_publish_or_change_an_existing_token() {
        let before = crate::game::data::ValidatedGameData::load_live_embedded().unwrap();
        let sources = GameplayRonSourceSet::embedded();
        for replacement in [
            "Damage(amount: -1, damage_type: Magic)",
            "ExecuteFile(path: \"inert-never-executed\")",
            "ApplyBuff(buff_id: \"missing_authoring_buff\", duration_ms: 10)",
        ] {
            let invalid =
                sources
                    .skills
                    .replacen("Damage(amount: 60, damage_type: Magic)", replacement, 1);
            assert_ne!(invalid, sources.skills);
            let candidate = GameplayRonSourceSet {
                skills: &invalid,
                ..sources
            };
            let error = match crate::game::data::GameDataBase::load_live_sources(candidate) {
                Ok(_) => panic!("invalid effect must not produce a validated token"),
                Err(error) => error,
            };
            assert_eq!(
                error.object_id.as_deref(),
                Some("unknown_distortion_strike")
            );
            assert_eq!(error.action_id.as_deref(), Some("strike"));
            assert_eq!(error.effect_id.as_deref(), Some("effect_01"));
            assert_eq!(error.source_file, "data/skills/base.ron");
            let retry = crate::game::data::ValidatedGameData::load_live_embedded().unwrap();
            assert_publications_semantically_equal(&before, &retry);
            assert_eq!(
                before.skill_data.skills[0].steps[0].effects[0].literal_amount(),
                Some(60)
            );
        }
    }

    fn target_locked_projectile() -> DeliveryDef {
        DeliveryDef::Projectile {
            speed_units_per_ms: 1,
            hit_policy: ProjectileHitPolicy::TargetLocked,
            allow_targetless_cast: false,
            max_range_tiles: None,
            max_lifetime_ms: None,
            max_kills: None,
            max_pierces: None,
            collision: SkillProjectileCollisionDef::default(),
        }
    }

    fn basic_delivery_incompatibility_cases(
    ) -> Vec<(&'static str, DeliveryDef, &'static str, &'static str)> {
        let projectile = |hit_policy,
                          allow_targetless_cast,
                          max_range_tiles,
                          max_lifetime_ms,
                          max_kills,
                          max_pierces,
                          collision| DeliveryDef::Projectile {
            speed_units_per_ms: 1,
            hit_policy,
            allow_targetless_cast,
            max_range_tiles,
            max_lifetime_ms,
            max_kills,
            max_pierces,
            collision,
        };
        vec![
            (
                "directional hit policy",
                projectile(
                    ProjectileHitPolicy::DirectionalCollision,
                    false,
                    Some(1),
                    None,
                    None,
                    None,
                    Default::default(),
                ),
                "delivery.Projectile.hit_policy",
                "DirectionalCollision is skill-only for a basic-attack consumer",
            ),
            (
                "targetless cast",
                projectile(
                    ProjectileHitPolicy::TargetLocked,
                    true,
                    None,
                    None,
                    None,
                    None,
                    Default::default(),
                ),
                "delivery.Projectile.allow_targetless_cast",
                "allow_targetless_cast is skill-only for a basic-attack consumer",
            ),
            (
                "range stop",
                projectile(
                    ProjectileHitPolicy::TargetLocked,
                    false,
                    Some(1),
                    None,
                    None,
                    None,
                    Default::default(),
                ),
                "delivery.Projectile.max_range_tiles",
                "max_range_tiles is skill-only for a basic-attack consumer",
            ),
            (
                "lifetime stop",
                projectile(
                    ProjectileHitPolicy::TargetLocked,
                    false,
                    None,
                    Some(1),
                    None,
                    None,
                    Default::default(),
                ),
                "delivery.Projectile.max_lifetime_ms",
                "max_lifetime_ms is skill-only for a basic-attack consumer",
            ),
            (
                "kill stop",
                projectile(
                    ProjectileHitPolicy::TargetLocked,
                    false,
                    None,
                    None,
                    Some(1),
                    None,
                    Default::default(),
                ),
                "delivery.Projectile.max_kills",
                "max_kills is skill-only for a basic-attack consumer",
            ),
            (
                "pierce stop",
                projectile(
                    ProjectileHitPolicy::TargetLocked,
                    false,
                    None,
                    None,
                    None,
                    Some(1),
                    Default::default(),
                ),
                "delivery.Projectile.max_pierces",
                "max_pierces is skill-only for a basic-attack consumer",
            ),
            (
                "collision radius",
                projectile(
                    ProjectileHitPolicy::TargetLocked,
                    false,
                    None,
                    None,
                    None,
                    None,
                    SkillProjectileCollisionDef {
                        radius_units: 1,
                        ..Default::default()
                    },
                ),
                "delivery.Projectile.collision.radius_units",
                "collision.radius_units is skill-only for a basic-attack consumer",
            ),
            (
                "collision target filter",
                projectile(
                    ProjectileHitPolicy::TargetLocked,
                    false,
                    None,
                    None,
                    None,
                    None,
                    SkillProjectileCollisionDef {
                        hit_targets: SkillHitTargetFilter::Allies,
                        ..Default::default()
                    },
                ),
                "delivery.Projectile.collision.hit_targets",
                "collision.hit_targets is skill-only for a basic-attack consumer",
            ),
            (
                "collision piercing",
                projectile(
                    ProjectileHitPolicy::TargetLocked,
                    false,
                    None,
                    None,
                    None,
                    None,
                    SkillProjectileCollisionDef {
                        piercing: true,
                        ..Default::default()
                    },
                ),
                "delivery.Projectile.collision.piercing",
                "collision.piercing is skill-only for a basic-attack consumer",
            ),
            (
                "collision despawn policy",
                projectile(
                    ProjectileHitPolicy::TargetLocked,
                    false,
                    None,
                    None,
                    None,
                    None,
                    SkillProjectileCollisionDef {
                        despawn_on_hit: Some(false),
                        ..Default::default()
                    },
                ),
                "delivery.Projectile.collision.despawn_on_hit",
                "collision.despawn_on_hit is skill-only for a basic-attack consumer",
            ),
            (
                "collision hit stop",
                projectile(
                    ProjectileHitPolicy::TargetLocked,
                    false,
                    None,
                    None,
                    None,
                    None,
                    SkillProjectileCollisionDef {
                        max_hits: Some(1),
                        ..Default::default()
                    },
                ),
                "delivery.Projectile.collision.max_hits",
                "collision.max_hits is skill-only for a basic-attack consumer",
            ),
            (
                "tile area",
                DeliveryDef::TileArea {
                    footprint: TileRangePattern {
                        include_anchor_tile: true,
                        rows: vec!["@".to_string()],
                    },
                    area: SkillTileAreaDeliveryDef::default(),
                },
                "delivery.TileArea",
                "TileArea delivery is skill-only for a basic-attack consumer",
            ),
        ]
    }

    fn assert_live_delivery_error(
        error: LiveRonError,
        source_file: &'static str,
        domain: &'static str,
        object_id: &str,
        field_path: &str,
        cause: &str,
    ) {
        assert_eq!(error.source_file, source_file);
        assert_eq!(error.domain, domain);
        assert_eq!(error.object_id.as_deref(), Some(object_id));
        assert_eq!(error.field_path.as_deref(), Some(field_path));
        assert_eq!(error.cause, cause);
    }

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
        validate_live_ron_manifest("(schema_version: 5)")
            .expect("current schema version should load");

        let error = validate_live_ron_manifest("(schema_version: 4)")
            .expect_err("unsupported schema version must reject");
        assert_eq!(error.source_file, "data/bundle.ron");
        assert_eq!(error.domain, "bundle_manifest");
        assert_eq!(error.field_path.as_deref(), Some("schema_version"));
        assert!(error.cause.contains("expected exactly 5"));
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
            parsed.boss_runtime,
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
        assert_eq!(LIVE_GAMEPLAY_RON_SOURCE_COUNT, 26);
        assert!(
            parsed.relics.relics.is_empty(),
            "official relic balance remains unauthored"
        );
        assert!(GameplayRonSourceSet::embedded()
            .source_inventory()
            .iter()
            .any(|(path, _)| *path == "data/relics/base.ron"));
    }

    #[test]
    fn embedded_cwbs_reference_is_connected_but_excluded_from_every_production_registry() {
        const ENCOUNTER_ID: &str = "reference_cwbs_final_boss";
        const ABNORMALITY_ID: &str = "x-00-00_cwbs_reference_boss";
        const FRAGMENT_ID: &str = "fragment_cwbs_reference_dormant";

        let sources = GameplayRonSourceSet::embedded();
        let parsed =
            parse_gameplay_ron_sources(sources).expect("embedded gameplay RON should parse");
        parsed
            .validate_domain_catalogs()
            .expect("CWBS reference catalogs should validate together");

        let encounter = parsed
            .pve_encounters
            .get_by_id(ENCOUNTER_ID)
            .expect("CWBS reference encounter should publish");
        assert!(encounter.is_reference_only());
        assert_eq!(encounter.primary_abnormality_id(), Some(ABNORMALITY_ID));
        assert!(encounter.boss_runtime.is_some());
        assert!(encounter.outcome_policies.iter().all(|policy| {
            policy.reward_uuids.is_empty()
                && !policy.grant_node_bonus
                && !policy.advance_encounter_progression
        }));
        assert!(parsed.abnormalities.get_by_id(ABNORMALITY_ID).is_some());
        assert!(parsed
            .skill_fragments
            .get_by_id(&crate::game::data::skill_fragment_data::SkillFragmentId::new(FRAGMENT_ID,))
            .is_some());
        assert!(parsed
            .summoned_units
            .definitions
            .iter()
            .any(|definition| definition.id.as_str() == "cwbs_reference_minion"));
        assert!(parsed
            .boss_runtime
            .encounters
            .iter()
            .any(|runtime| runtime.id.as_str() == "cwbs_reference_final_boss_runtime"));

        assert!(parsed
            .pve_encounters
            .reward_sources
            .iter()
            .all(|source| source.encounter_id != ENCOUNTER_ID
                && source.fragment_id.as_str() != FRAGMENT_ID));
        assert!(parsed.boss_omen_chains.chains.iter().all(|chain| {
            chain.boss_encounter_id != ENCOUNTER_ID && chain.boss_abnormality_id != ABNORMALITY_ID
        }));
        assert_eq!(
            parsed
                .boss_omen_chains
                .chains
                .iter()
                .map(|chain| chain.id.as_str())
                .collect::<std::collections::BTreeSet<_>>(),
            std::collections::BTreeSet::from(["white_night"]),
            "the all-live-FinalBoss record is the closed BossOmen chain registry"
        );
        for (path, source) in [
            (
                "data/map/node_definitions.ron",
                sources.map_node_definitions,
            ),
            (
                "data/map/facility_templates.ron",
                sources.facility_templates,
            ),
            (
                "data/map/node_assignment_profiles.ron",
                sources.node_assignment_profiles,
            ),
        ] {
            assert!(
                !source.contains(ENCOUNTER_ID) && !source.contains(ABNORMALITY_ID),
                "reference content leaked into production map source {path}"
            );
        }
    }

    #[test]
    fn cwbs_reference_outcome_policy_cannot_be_promoted_by_data_drift() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("embedded gameplay RON should parse");
        parsed
            .pve_encounters
            .encounters
            .iter_mut()
            .find(|encounter| encounter.id == "reference_cwbs_final_boss")
            .expect("CWBS reference encounter")
            .outcome_policies[0]
            .advance_encounter_progression = true;

        let error = parsed
            .validate_domain_catalogs()
            .expect_err("reference-only progression authority must be rejected");
        assert_eq!(error.source_file, "data/pve/encounters.ron");
        assert!(error.cause.contains("reference-only outcomes"));
    }

    #[test]
    fn embedded_pve_authors_exact_bootstrap_reward_sources() {
        let parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("embedded gameplay RON should parse");
        parsed
            .validate_domain_catalogs()
            .expect("embedded reward sources should pass cross-catalog validation");

        let actual = parsed
            .pve_encounters
            .reward_sources
            .iter()
            .map(|source| {
                let encounter = parsed
                    .pve_encounters
                    .get_by_id(&source.encounter_id)
                    .expect("validated reward source encounter");
                let class = match encounter.encounter_class {
                    crate::game::data::pve_data::PveEncounterClass::Elite => "Elite",
                    crate::game::data::pve_data::PveEncounterClass::NormalBoss => "NormalBoss",
                    crate::game::data::pve_data::PveEncounterClass::FinalBoss => "FinalBoss",
                    crate::game::data::pve_data::PveEncounterClass::ReferenceFinalBoss => {
                        "ReferenceFinalBoss"
                    }
                    crate::game::data::pve_data::PveEncounterClass::Normal => "Normal",
                };
                let dust = match source.repeat_reward {
                    crate::game::reward::RewardEffect::GrantFragmentDust { amount } => amount,
                    _ => panic!("validated repeat reward must grant fragment dust"),
                };
                (
                    class,
                    source.reward_source_id.as_str(),
                    source.encounter_id.as_str(),
                    source.fragment_id.as_str(),
                    dust,
                )
            })
            .collect::<std::collections::BTreeSet<_>>();
        let expected = [
            (
                "Elite",
                "elite_black_swan",
                "suppress_black_swan",
                "fragment_black_swan_trace",
                5,
            ),
            (
                "Elite",
                "elite_censored",
                "suppress_censored",
                "fragment_censored_trace",
                5,
            ),
            (
                "Elite",
                "elite_clouded_monk",
                "suppress_clouded_monk",
                "fragment_clouded_monk_trace",
                5,
            ),
            (
                "Elite",
                "elite_laetitia",
                "suppress_laetitia",
                "fragment_laetitia_trace",
                5,
            ),
            (
                "Elite",
                "elite_nameless_fetus",
                "suppress_nameless_fetus",
                "fragment_nameless_fetus_trace",
                5,
            ),
            (
                "Elite",
                "elite_parasite_tree",
                "suppress_parasite_tree",
                "fragment_parasite_tree_trace",
                5,
            ),
            (
                "Elite",
                "elite_porcubbus",
                "suppress_porcubbus",
                "fragment_porcubbus_trace",
                5,
            ),
            (
                "Elite",
                "elite_queen_bee",
                "suppress_queen_bee",
                "fragment_queen_bee_trace",
                5,
            ),
            (
                "Elite",
                "elite_rudolta",
                "suppress_rudolta",
                "fragment_rudolta_trace",
                5,
            ),
            (
                "Elite",
                "elite_scarecrow",
                "suppress_scarecrow",
                "fragment_scarecrow_trace",
                5,
            ),
            (
                "Elite",
                "elite_schadenfreude",
                "suppress_schadenfreude",
                "fragment_schadenfreude_trace",
                5,
            ),
            (
                "Elite",
                "elite_snow_queen",
                "suppress_snow_queen",
                "fragment_snow_queen_trace",
                5,
            ),
            (
                "Elite",
                "elite_warm_hearted_woodsman",
                "suppress_warm_hearted_woodsman",
                "fragment_warm_hearted_woodsman_trace",
                5,
            ),
            (
                "NormalBoss",
                "normal_boss_big_bad_wolf",
                "suppress_big_bad_wolf",
                "fragment_big_bad_wolf_trace",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_burrowing_heaven",
                "suppress_burrowing_heaven",
                "fragment_burrowing_heaven_trace",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_freischutz",
                "suppress_freischutz",
                "fragment_freischutz_black_round",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_funeral_butterfly",
                "suppress_funeral_butterfly",
                "fragment_funeral_butterfly_eulogy",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_king_of_greed",
                "suppress_king_of_greed",
                "fragment_king_of_greed_trace",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_knight_of_despair",
                "suppress_knight_of_despair",
                "fragment_knight_of_despair_trace",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_little_red",
                "suppress_little_red",
                "fragment_little_red_trace",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_melting_love",
                "suppress_melting_love",
                "fragment_melting_love_trace",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_mountain",
                "suppress_mountain",
                "fragment_mountain_smiling_bodies_trace",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_nothing_there",
                "suppress_nothing_there",
                "fragment_nothing_there_trace",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_queen_of_hatred",
                "suppress_queen_of_hatred",
                "fragment_queen_of_hatred_trace",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_yang",
                "suppress_yang",
                "fragment_yang_trace",
                10,
            ),
            (
                "NormalBoss",
                "normal_boss_yin",
                "suppress_yin",
                "fragment_yin_trace",
                10,
            ),
            (
                "FinalBoss",
                "final_boss_apocalypse_bird",
                "suppress_apocalypse_bird",
                "fragment_apocalypse_bird_trace",
                20,
            ),
            (
                "FinalBoss",
                "final_boss_silent_orchestra",
                "suppress_silent_orchestra",
                "fragment_silent_orchestra_trace",
                20,
            ),
            (
                "FinalBoss",
                "final_boss_white_night",
                "suppress_white_night",
                "fragment_white_night_trace",
                20,
            ),
        ]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(actual, expected);
        assert!(parsed
            .pve_encounters
            .get_reward_source_by_encounter_id("suppress_moonlit_wail")
            .is_none());
    }

    #[test]
    fn reward_source_contract_rejects_duplicate_wrong_reward_and_cross_reference() {
        for (case, mutate) in [
            ("duplicate source", 0_u8),
            ("wrong repeat reward", 1_u8),
            ("fragment mismatch", 2_u8),
        ] {
            let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
                .expect("embedded gameplay RON should parse");
            match mutate {
                0 => {
                    parsed.pve_encounters.reward_sources[1].reward_source_id =
                        parsed.pve_encounters.reward_sources[0]
                            .reward_source_id
                            .clone();
                }
                1 => {
                    parsed.pve_encounters.reward_sources[0].repeat_reward =
                        crate::game::reward::RewardEffect::GrantFragmentDust { amount: 6 };
                }
                2 => {
                    parsed.pve_encounters.reward_sources[0].fragment_id =
                        crate::game::data::skill_fragment_data::SkillFragmentId::from(
                            "fragment_moonlit_wail_trace",
                        );
                }
                _ => unreachable!(),
            }

            let error = parsed
                .validate_domain_catalogs()
                .expect_err("invalid reward-source authoring must fail closed");
            assert_eq!(error.source_file, "data/pve/encounters.ron", "{case}");
            assert_eq!(error.domain, "pve_encounters", "{case}");
            assert!(
                error
                    .field_path
                    .as_deref()
                    .is_some_and(|path| path.starts_with("reward_sources[")),
                "{case}: {error}"
            );
        }
    }

    #[test]
    fn embedded_source_inventory_matches_every_physical_gameplay_ron_exactly() {
        let data_root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../game_resources/data");
        let mut pending = vec![data_root.clone()];
        let mut physical_paths = Vec::new();
        while let Some(directory) = pending.pop() {
            for entry in
                std::fs::read_dir(&directory).expect("gameplay data directory should exist")
            {
                let entry = entry.expect("gameplay data entry should be readable");
                let path = entry.path();
                if path.is_dir() {
                    pending.push(path);
                } else if path.extension().is_some_and(|extension| extension == "ron") {
                    let relative = path
                        .strip_prefix(data_root.parent().unwrap())
                        .expect("RON source should be under game_resources")
                        .to_string_lossy()
                        .replace('\\', "/");
                    physical_paths.push(relative);
                }
            }
        }
        physical_paths.sort();
        let mut expected = LIVE_GAMEPLAY_RON_SOURCE_PATHS
            .iter()
            .map(|path| (*path).to_string())
            .collect::<Vec<_>>();
        expected.push("data/bundle.ron".to_string());
        expected.sort();

        assert_eq!(physical_paths, expected);
        assert_eq!(
            GameplayRonSourceSet::embedded()
                .source_inventory()
                .map(|(path, _)| path),
            LIVE_GAMEPLAY_RON_SOURCE_PATHS
        );
    }

    #[test]
    fn empty_embedded_source_is_rejected_by_inventory_before_domain_parse() {
        let mut sources = GameplayRonSourceSet::embedded();
        sources.boss_runtime = "   ";
        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("empty source must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/boss_runtime/base.ron");
        assert_eq!(error.domain, "source_inventory");
    }

    #[test]
    fn boss_source_unknown_field_rejects_before_candidate_validation() {
        let mut sources = GameplayRonSourceSet::embedded();
        sources.boss_runtime =
            "(patterns: [], actor_behaviors: [], encounters: [], unexpected: true)";
        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("unknown Boss source field must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/boss_runtime/base.ron");
        assert_eq!(error.domain, "boss_runtime");
    }

    #[test]
    fn invalid_boss_compiler_input_rejects_the_whole_candidate() {
        let mut catalog = crate::game::data::boss_runtime_data::tests::valid_catalog();
        catalog.patterns.push(catalog.patterns[0].clone());
        let boss_source = ron::ser::to_string(&catalog).expect("Boss fixture should serialize");
        let mut pve: PveEncounterDatabase = parse_live_ron(
            "data/pve/encounters.ron",
            "pve_encounters",
            GameplayRonSourceSet::embedded().pve_encounters,
        )
        .expect("embedded PvE data should parse");
        pve.encounters
            .retain(|encounter| encounter.id != "reference_cwbs_final_boss");
        for encounter in &mut pve.encounters {
            encounter.boss_runtime = None;
        }
        let pve_source = ron::ser::to_string(&pve).expect("isolated PvE fixture should serialize");
        let mut sources = GameplayRonSourceSet::embedded();
        sources.boss_runtime = &boss_source;
        sources.summoned_units = "(definitions: [])";
        sources.pve_encounters = &pve_source;

        let parsed = parse_gameplay_ron_sources(sources).expect("raw Boss fixture should parse");
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("duplicate Boss definition must reject the candidate");
        assert_eq!(error.source_file, "data/boss_runtime/base.ron");
        assert_eq!(error.domain, "boss_runtime");
        assert!(error.cause.contains("duplicate id"), "{}", error.cause);
    }

    #[test]
    fn invalid_summoned_unit_input_reports_the_summoned_unit_source() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let mut definition = crate::game::data::summoned_unit_data::tests::definition();
        definition.max_alive = 0;
        parsed.summoned_units.definitions.push(definition);

        let error = parsed
            .validate_domain_catalogs()
            .expect_err("invalid summoned-unit definition must reject the candidate");
        assert_eq!(error.source_file, "data/summoned_units/base.ron");
        assert_eq!(error.domain, "summoned_units");
        assert!(error.cause.contains("greater than zero"), "{}", error.cause);
    }

    #[test]
    fn dangling_pve_boss_reference_rejects_the_whole_candidate() {
        let mut pve: PveEncounterDatabase = parse_live_ron(
            "data/pve/encounters.ron",
            "pve_encounters",
            GameplayRonSourceSet::embedded().pve_encounters,
        )
        .expect("embedded PvE data should parse");
        pve.encounters[0].boss_runtime =
            Some(crate::game::data::pve_data::PveBossRuntimeBindingData {
                encounter_id: crate::game::data::boss_runtime_data::BossEncounterRuntimeId::new(
                    "missing_boss_encounter",
                ),
                actors: vec![],
            });
        let pve_source = ron::ser::to_string(&pve).expect("PvE fixture should serialize");
        let mut sources = GameplayRonSourceSet::embedded();
        sources.pve_encounters = &pve_source;

        let parsed = parse_gameplay_ron_sources(sources).expect("raw PvE fixture should parse");
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("dangling PvE Boss reference must reject the candidate");
        assert_eq!(error.source_file, "data/pve/encounters.ron");
        assert_eq!(error.domain, "pve_encounters");
        assert!(error.cause.contains("missing Boss encounter definition"));
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
    fn source_set_rejects_missing_and_unknown_weapon_tactical_profile_fields() {
        for field in [
            "                deployment_affinity: ground_only,\n",
            "                ground_defense_focus: Some((\n                    block_capacity_bonus: 1,\n                    incoming_damage_reduction_percent: 10,\n                )),\n",
            "                platform_rest: None,\n",
        ] {
            let mut sources = GameplayRonSourceSet::embedded();
            let corrupted = sources.equipment.replacen(field, "", 1);
            assert_ne!(corrupted, sources.equipment);
            sources.equipment = &corrupted;
            let error = match parse_gameplay_ron_sources(sources) {
                Ok(_) => panic!("missing required weapon tactical field must reject"),
                Err(error) => error,
            };
            assert_eq!(error.source_file, "data/equipments/base.ron");
            assert!(error.cause.contains("missing field"));
        }

        let mut sources = GameplayRonSourceSet::embedded();
        let corrupted = sources.equipment.replacen(
            "                deployment_affinity: ground_only,\n",
            "                deployment_affinity: ground_only,\n                legacy_tactical_mode: true,\n",
            1,
        );
        assert_ne!(corrupted, sources.equipment);
        sources.equipment = &corrupted;
        let error = match parse_gameplay_ron_sources(sources) {
            Ok(_) => panic!("unknown weapon tactical field must reject"),
            Err(error) => error,
        };
        assert!(error.cause.contains("unknown field `legacy_tactical_mode`"));
    }

    #[test]
    fn bundle_validation_rejects_mismatched_weapon_tactical_profile() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let item_id = parsed.equipment.items[0].id.clone();
        parsed.equipment.items[0]
            .weapon_profile
            .as_mut()
            .unwrap()
            .deployment_affinity = crate::game::battle::types::DeploymentAffinity::PlatformOnly;
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("tile/profile mismatch must reject before publication");
        assert_eq!(error.source_file, "data/equipments/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(item_id.as_str()));
        assert!(error.cause.contains("PlatformOnly"));
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
                    game_mode: GameMode::Expedition
                }
            )
        });
        let error = missing
            .validate_domain_catalogs()
            .expect_err("missing Endless RunStart must reject before publication");
        assert_eq!(
            error.field_path.as_deref(),
            Some("triggers.RunStart.Expedition")
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
                        game_mode: GameMode::Tutorial
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
        assert_eq!(
            error.field_path.as_deref(),
            Some("items[0].basic_attack.interval_ms")
        );
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
    fn programmatic_skill_publication_rejects_every_zero_projectile_scalar_with_exact_provenance() {
        #[derive(Clone, Copy)]
        enum ZeroField {
            Speed,
            Range,
            Lifetime,
            Kills,
            Pierces,
            Hits,
        }

        for (field, suffix, cause) in [
            (
                ZeroField::Speed,
                "speed_units_per_ms",
                "projectile speed_units_per_ms must be greater than zero",
            ),
            (
                ZeroField::Range,
                "max_range_tiles",
                "projectile max_range_tiles must be greater than zero",
            ),
            (
                ZeroField::Lifetime,
                "max_lifetime_ms",
                "projectile max_lifetime_ms must be greater than zero",
            ),
            (
                ZeroField::Kills,
                "max_kills",
                "projectile max_kills must be greater than zero",
            ),
            (
                ZeroField::Pierces,
                "max_pierces",
                "projectile max_pierces must be greater than zero",
            ),
            (
                ZeroField::Hits,
                "collision.max_hits",
                "projectile collision max_hits must be greater than zero",
            ),
        ] {
            let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
                .expect("live source set should parse");
            let (skill_index, step_index) = parsed
                .skills
                .skills
                .iter()
                .enumerate()
                .find_map(|(skill_index, skill)| {
                    skill
                        .steps
                        .iter()
                        .enumerate()
                        .find_map(|(step_index, step)| {
                            matches!(step.delivery, DeliveryDef::Projectile { .. })
                                .then_some((skill_index, step_index))
                        })
                })
                .expect("live skills must contain a projectile step");
            let skill_id = parsed.skills.skills[skill_index].id.to_string();
            let mut delivery = target_locked_projectile();
            let DeliveryDef::Projectile {
                speed_units_per_ms,
                max_range_tiles,
                max_lifetime_ms,
                max_kills,
                max_pierces,
                collision,
                ..
            } = &mut delivery
            else {
                unreachable!();
            };
            match field {
                ZeroField::Speed => *speed_units_per_ms = 0,
                ZeroField::Range => *max_range_tiles = Some(0),
                ZeroField::Lifetime => *max_lifetime_ms = Some(0),
                ZeroField::Kills => *max_kills = Some(0),
                ZeroField::Pierces => *max_pierces = Some(0),
                ZeroField::Hits => collision.max_hits = Some(0),
            }
            parsed.skills.skills[skill_index].steps[step_index].delivery = delivery;

            let error = parsed
                .validate_domain_catalogs()
                .expect_err("zero projectile scalar must reject the whole candidate");
            assert_live_delivery_error(
                error,
                "data/skills/base.ron",
                "skills",
                &skill_id,
                &format!("skills[{skill_index}].steps[{step_index}].delivery.Projectile.{suffix}"),
                cause,
            );
        }
    }

    #[test]
    fn ron_skill_publication_rejects_every_zero_projectile_scalar_with_exact_provenance() {
        for (authored_field, suffix, cause) in [
            (
                "speed_units_per_ms: 0",
                "speed_units_per_ms",
                "projectile speed_units_per_ms must be greater than zero",
            ),
            (
                "speed_units_per_ms: 7500, max_range_tiles: Some(0)",
                "max_range_tiles",
                "projectile max_range_tiles must be greater than zero",
            ),
            (
                "speed_units_per_ms: 7500, max_lifetime_ms: Some(0)",
                "max_lifetime_ms",
                "projectile max_lifetime_ms must be greater than zero",
            ),
            (
                "speed_units_per_ms: 7500, max_kills: Some(0)",
                "max_kills",
                "projectile max_kills must be greater than zero",
            ),
            (
                "speed_units_per_ms: 7500, max_pierces: Some(0)",
                "max_pierces",
                "projectile max_pierces must be greater than zero",
            ),
            (
                "speed_units_per_ms: 7500, collision: (max_hits: Some(0))",
                "collision.max_hits",
                "projectile collision max_hits must be greater than zero",
            ),
        ] {
            let mut sources = GameplayRonSourceSet::embedded();
            let corrupted = sources
                .skills
                .replacen("speed_units_per_ms: 7500", authored_field, 1);
            assert_ne!(corrupted, sources.skills);
            sources.skills = &corrupted;
            let error = match crate::game::data::GameDataBase::load_live_sources(sources) {
                Ok(_) => panic!("zero RON projectile scalar must reject the whole candidate"),
                Err(error) => error,
            };
            assert_live_delivery_error(
                error,
                "data/skills/base.ron",
                "skills",
                "little_red_hunt_the_prey",
                &format!("skills[2].steps[0].delivery.Projectile.{suffix}"),
                cause,
            );
        }
    }

    #[test]
    fn programmatic_basic_and_weapon_publication_reject_every_ignored_delivery_field() {
        for (case, delivery, suffix, cause) in basic_delivery_incompatibility_cases() {
            let mut abnormality = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
                .expect("live source set should parse");
            let abnormality_id = abnormality.abnormalities.items[0].id.clone();
            abnormality.abnormalities.items[0].basic_attack.delivery = delivery.clone();
            let error = match abnormality.validate_domain_catalogs() {
                Err(error) => error,
                Ok(()) => panic!("ignored basic delivery field must reject: {case}"),
            };
            assert_live_delivery_error(
                error,
                "data/abnormalities/base.ron",
                "abnormalities",
                &abnormality_id,
                &format!("items[0].basic_attack.{suffix}"),
                cause,
            );

            let mut equipment = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
                .expect("live source set should parse");
            let item_index = equipment
                .equipment
                .items
                .iter()
                .position(|item| {
                    item.weapon_profile.as_ref().is_some_and(|profile| {
                        matches!(profile.delivery, DeliveryDef::Projectile { .. })
                    })
                })
                .expect("live equipment must contain a projectile weapon");
            let item_id = equipment.equipment.items[item_index].id.clone();
            equipment.equipment.items[item_index]
                .weapon_profile
                .as_mut()
                .expect("selected item must have a weapon profile")
                .delivery = delivery;
            let error = match equipment.validate_domain_catalogs() {
                Err(error) => error,
                Ok(()) => panic!("ignored weapon delivery field must reject: {case}"),
            };
            assert_live_delivery_error(
                error,
                "data/equipments/base.ron",
                "equipment",
                &item_id,
                &format!("items[{item_index}].weapon_profile.{suffix}"),
                cause,
            );
        }
    }

    #[test]
    fn ron_basic_and_weapon_publication_share_exact_delivery_compatibility_errors() {
        let mut abnormality_sources = GameplayRonSourceSet::embedded();
        let abnormality_ron = abnormality_sources.abnormalities.replacen(
            "delivery: Projectile(speed_units_per_ms: 6000)",
            "delivery: Projectile(speed_units_per_ms: 6000, max_kills: Some(1))",
            1,
        );
        abnormality_sources.abnormalities = &abnormality_ron;
        let abnormality_error =
            match crate::game::data::GameDataBase::load_live_sources(abnormality_sources) {
                Ok(_) => panic!("skill-only basic delivery policy must reject"),
                Err(error) => error,
            };
        assert_live_delivery_error(
            abnormality_error,
            "data/abnormalities/base.ron",
            "abnormalities",
            "o-01-04_queen_of_hatred",
            "items[0].basic_attack.delivery.Projectile.max_kills",
            "max_kills is skill-only for a basic-attack consumer",
        );

        let mut equipment_sources = GameplayRonSourceSet::embedded();
        let equipment_ron = equipment_sources.equipment.replacen(
            "delivery: Projectile(speed_units_per_ms: 6000)",
            "delivery: Projectile(speed_units_per_ms: 6000, max_kills: Some(1))",
            1,
        );
        equipment_sources.equipment = &equipment_ron;
        let equipment_error =
            match crate::game::data::GameDataBase::load_live_sources(equipment_sources) {
                Ok(_) => panic!("skill-only weapon delivery policy must reject"),
                Err(error) => error,
            };
        assert_live_delivery_error(
            equipment_error,
            "data/equipments/base.ron",
            "equipment",
            "paradise_lost",
            "items[1].weapon_profile.delivery.Projectile.max_kills",
            "max_kills is skill-only for a basic-attack consumer",
        );
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
            .baseline_skill_fragment_ids = vec![
            crate::game::data::skill_fragment_data::SkillFragmentId::new(
                "__missing_g12_fragment__",
            ),
        ];
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
    fn u06_live_loadout_rejects_missing_wrong_kind_and_incompatible_active() {
        for (case, active) in [
            ("missing", "__missing_training_active__"),
            ("baseline", "starter_basic_attack_enhancement"),
            ("weapon", "training_adrenaline"),
        ] {
            let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded()).unwrap();
            let definition = parsed
                .employee_definitions
                .definitions
                .iter_mut()
                .find(|d| d.employee_definition_id == "field_medic")
                .unwrap();
            definition.initial_loadout.active_skill_fragment_id = Some(active.into());
            let error = parsed.validate_domain_catalogs().expect_err(case);
            assert_eq!(error.source_file, "data/employees/definitions.ron");
            assert_eq!(error.object_id.as_deref(), Some("field_medic"));
            assert!(error
                .field_path
                .as_deref()
                .unwrap()
                .ends_with("initial_loadout.active_skill_fragment_id"));
        }
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded()).unwrap();
        let definition = &mut parsed.employee_definitions.definitions[0];
        definition
            .initial_loadout
            .baseline_skill_fragment_ids
            .push("training_single_heal".into());
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("active cannot be hidden in intrinsic baseline grants");
        assert!(error
            .field_path
            .as_deref()
            .unwrap()
            .ends_with("baseline_skill_fragment_ids[0]"));
    }

    #[test]
    fn bundle_validation_requires_a_starter_weapon() {
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let definition = &mut parsed.employee_definitions.definitions[0];
        let definition_id = definition.employee_definition_id.clone();
        definition
            .initial_loadout
            .equipment_ids
            .retain(|id| id.as_str() == "standard_armor");
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("starter without weapon must reject before publication");
        assert_eq!(error.source_file, "data/employees/definitions.ron");
        assert_eq!(error.object_id.as_deref(), Some(definition_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("definitions[0].initial_loadout.equipment_ids")
        );
        assert!(error.cause.contains("initial weapon"));
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
        for wave in &mut parsed.pve_encounters.encounters[0].waves {
            wave.required_for_victory = false;
        }
        parsed.pve_encounters.encounters[0].terminal_outcomes[0].condition =
            crate::game::data::pve_data::PveTerminalConditionData::TimeReached { time_ms: 0 };
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("zero PVE survive timer must reject before publication");
        assert_eq!(error.source_file, "data/pve/encounters.ron");
        assert_eq!(error.object_id.as_deref(), Some(encounter_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some("encounters[0].terminal_outcomes[0].condition.time_ms")
        );
        assert_eq!(
            error.cause,
            "terminal time_ms must be within 1..=4294967295"
        );
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
    fn bundle_validation_rejects_every_runtime_invalid_trigger_pair_with_exact_provenance() {
        let cases = vec![
            (
                "bundle_run_start_once_per_battle",
                VisualNovelTriggerSource::RunStart {
                    game_mode: GameMode::Tutorial,
                },
                VisualNovelTriggerRepeat::OncePerBattle,
                "OncePerBattle requires BattleStarted or BattleTimeReached source",
            ),
            (
                "bundle_run_start_once_per_node",
                VisualNovelTriggerSource::RunStart {
                    game_mode: GameMode::Tutorial,
                },
                VisualNovelTriggerRepeat::OncePerNode,
                "OncePerNode requires MapNodeEntered, BattleStarted, or BattleTimeReached source",
            ),
            (
                "bundle_map_node_entered_once_per_battle",
                VisualNovelTriggerSource::MapNodeEntered {
                    category: crate::game::map::MapNodeCategory::SavePoint,
                    kind_id: crate::game::map::MapNodeKindId::new("save_point"),
                },
                VisualNovelTriggerRepeat::OncePerBattle,
                "OncePerBattle requires BattleStarted or BattleTimeReached source",
            ),
        ];

        let actual = cases
            .iter()
            .map(|(id, source, repeat, _)| {
                let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
                    .expect("live source set should parse");
                let mut candidate = parsed.visual_novel_triggers.triggers[0].clone();
                candidate.id =
                    crate::game::data::visual_novel_trigger_data::VisualNovelTriggerId::new(*id);
                candidate.source = source.clone();
                candidate.repeat = *repeat;
                let trigger_index = if matches!(source, VisualNovelTriggerSource::RunStart { .. }) {
                    parsed.visual_novel_triggers.triggers[0] = candidate;
                    0
                } else {
                    parsed.visual_novel_triggers.triggers.push(candidate);
                    parsed.visual_novel_triggers.triggers.len() - 1
                };
                let expected_path = format!("triggers[{trigger_index}].repeat");
                let error = parsed.validate_domain_catalogs().err();
                (
                    expected_path,
                    error.map(|error| {
                        (
                            error.source_file.to_string(),
                            error.domain.to_string(),
                            error.object_id,
                            error.field_path,
                            error.cause,
                        )
                    }),
                )
            })
            .collect::<Vec<_>>();
        let expected = cases
            .iter()
            .enumerate()
            .map(|(case_index, (id, _, _, cause))| {
                let trigger_index = if case_index < 2 { 0 } else { 2 };
                (
                    format!("triggers[{trigger_index}].repeat"),
                    Some((
                        "data/visual_novel_triggers/base.ron".to_string(),
                        "visual_novel_triggers".to_string(),
                        Some((*id).to_string()),
                        Some(format!("triggers[{trigger_index}].repeat")),
                        (*cause).to_string(),
                    )),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(actual, expected);
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
            forms,
            ..
        } = &mut parsed.skill_fragments.fragments[fragment_index].effect
        else {
            unreachable!("selected fragment must contain ActiveSkill")
        };
        forms.get_mut("base").unwrap().skill_id =
            crate::game::ability::SkillId::from("__missing_g12_skill__");
        let error = parsed
            .validate_domain_catalogs()
            .expect_err("missing Fragment Skill must reject before publication");
        assert_eq!(error.source_file, "data/skill_fragments/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(fragment_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(
                format!("fragments[{fragment_index}].effect.ActiveSkill.forms[base].skill_id")
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

    fn equipment_effect_item(
        parsed: &mut ParsedGameplayRon,
    ) -> (
        usize,
        String,
        &mut crate::game::data::equipment_data::EquipmentMetadata,
    ) {
        let item_index = 0;
        let item_id = parsed.equipment.items[item_index].id.clone();
        let item = &mut parsed.equipment.items[item_index];
        item.triggered_effects.clear();
        item.ability_activations.clear();
        (item_index, item_id, item)
    }

    #[test]
    fn bundle_validation_rejects_equipment_missing_buff_reference_with_exact_provenance() {
        const MISSING_BUFF: &str = "__missing_equipment_buff__";
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let (item_index, item_id, item) = equipment_effect_item(&mut parsed);
        item.triggered_effects.insert(
            crate::game::stats::TriggerType::OnBattleStart,
            vec![crate::game::stats::TriggeredEffect::targeted(
                crate::game::stats::TriggerEffectTarget::SelfUnit,
                crate::game::stats::Effect::ApplyBuff {
                    buff_id: MISSING_BUFF.to_string(),
                    duration_ms: 100,
                },
            )],
        );

        let error = parsed
            .validate_domain_catalogs()
            .expect_err("missing equipment buff must reject before publication");
        assert_eq!(error.source_file, "data/equipments/base.ron");
        assert_eq!(error.domain, "equipment");
        assert_eq!(error.object_id.as_deref(), Some(item_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(
                format!(
                    "items[{item_index}].triggered_effects.OnBattleStart[0].effect.ApplyBuff.buff_id"
                )
                .as_str()
            )
        );
        assert_eq!(
            error.cause,
            "references unknown equipment effect buff '__missing_equipment_buff__'"
        );
    }

    #[test]
    fn bundle_validation_rejects_equipment_missing_activation_skill_with_exact_provenance() {
        const MISSING_SKILL: &str = "__missing_equipment_skill__";
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let (item_index, item_id, item) = equipment_effect_item(&mut parsed);
        item.ability_activations
            .push(crate::game::ability::AbilityActivationBinding {
                ability_id: crate::game::ability::SkillId::from(MISSING_SKILL),
                activation: crate::game::ability::AbilityActivationDef::TriggerProc {
                    trigger: crate::game::stats::TriggerType::OnBattleStart,
                    proc_chance_percent: 100,
                    internal_cooldown_ms: 0,
                    max_triggers_per_battle: None,
                },
            });

        let error = parsed
            .validate_domain_catalogs()
            .expect_err("missing equipment activation skill must reject before publication");
        assert_eq!(error.source_file, "data/equipments/base.ron");
        assert_eq!(error.domain, "equipment");
        assert_eq!(error.object_id.as_deref(), Some(item_id.as_str()));
        assert_eq!(
            error.field_path.as_deref(),
            Some(format!("items[{item_index}].ability_activations[0].ability_id").as_str())
        );
        assert_eq!(
            error.cause,
            "references unknown equipment activation skill '__missing_equipment_skill__'"
        );
    }

    #[derive(Debug, Clone, Copy)]
    enum EquipmentEffectFixtureKind {
        Modifier,
        BonusDamage,
        ModifyDamage,
        Heal,
        ApplyBuff,
    }

    impl EquipmentEffectFixtureKind {
        fn name(self) -> &'static str {
            match self {
                Self::Modifier => "Modifier",
                Self::BonusDamage => "BonusDamage",
                Self::ModifyDamage => "ModifyDamage",
                Self::Heal => "Heal",
                Self::ApplyBuff => "ApplyBuff",
            }
        }

        fn effect(self) -> crate::game::stats::Effect {
            match self {
                Self::Modifier => {
                    crate::game::stats::Effect::Modifier(crate::game::stats::StatModifier {
                        stat: crate::game::stats::StatId::Attack,
                        kind: crate::game::stats::StatModifierKind::Flat,
                        value: 1,
                    })
                }
                Self::BonusDamage => crate::game::stats::Effect::BonusDamage {
                    flat: 1,
                    percent: 0,
                    damage_type: DamageType::Physical,
                },
                Self::ModifyDamage => crate::game::stats::Effect::ModifyDamage(Default::default()),
                Self::Heal => crate::game::stats::Effect::Heal {
                    flat: 1,
                    percent: 0,
                },
                Self::ApplyBuff => crate::game::stats::Effect::ApplyBuff {
                    buff_id: "poison".to_string(),
                    duration_ms: 100,
                },
            }
        }

        fn is_ordinary(self) -> bool {
            matches!(self, Self::Modifier | Self::Heal | Self::ApplyBuff)
        }

        fn is_damage(self) -> bool {
            matches!(self, Self::BonusDamage | Self::ModifyDamage)
        }
    }

    fn equipment_effect_case_is_supported(
        trigger: crate::game::stats::TriggerType,
        target: crate::game::stats::TriggerEffectTarget,
        effect: EquipmentEffectFixtureKind,
    ) -> bool {
        use crate::game::stats::{TriggerEffectTarget as Target, TriggerType as Trigger};

        match trigger {
            Trigger::Permanent => {
                matches!(effect, EquipmentEffectFixtureKind::Modifier)
                    && matches!(target, Target::SelfUnit | Target::AllAllies)
            }
            Trigger::OnBattleStart => effect.is_ordinary() && target != Target::CounterpartUnit,
            Trigger::OnAttack => {
                effect.is_ordinary() || (effect.is_damage() && target == Target::CounterpartUnit)
            }
            Trigger::OnHit => {
                effect.is_ordinary()
                    || (effect.is_damage() && target == Target::SelfUnit)
                    || (matches!(effect, EquipmentEffectFixtureKind::BonusDamage)
                        && target == Target::CounterpartUnit)
            }
            Trigger::OnDeath => effect.is_ordinary() && target != Target::SelfUnit,
            Trigger::OnKill | Trigger::OnAllyDeath => {
                effect.is_ordinary() && target != Target::CounterpartUnit
            }
        }
    }

    #[test]
    fn equipment_trigger_effect_and_activation_matrix_matches_runtime_occurrences() {
        use crate::game::stats::{TriggerEffectTarget as Target, TriggerType as Trigger};

        let triggers = [
            Trigger::Permanent,
            Trigger::OnBattleStart,
            Trigger::OnAttack,
            Trigger::OnHit,
            Trigger::OnDeath,
            Trigger::OnKill,
            Trigger::OnAllyDeath,
        ];
        let targets = [
            Target::SelfUnit,
            Target::CounterpartUnit,
            Target::AllAllies,
            Target::AllEnemies,
        ];
        let effects = [
            EquipmentEffectFixtureKind::Modifier,
            EquipmentEffectFixtureKind::BonusDamage,
            EquipmentEffectFixtureKind::ModifyDamage,
            EquipmentEffectFixtureKind::Heal,
            EquipmentEffectFixtureKind::ApplyBuff,
        ];
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let item_index = 0;
        let item_id = parsed.equipment.items[item_index].id.clone();
        parsed.equipment.items[item_index]
            .ability_activations
            .clear();

        for trigger in triggers {
            for target in targets {
                for effect in effects {
                    parsed.equipment.items[item_index].triggered_effects.clear();
                    parsed.equipment.items[item_index].triggered_effects.insert(
                        trigger,
                        vec![crate::game::stats::TriggeredEffect::targeted(
                            target,
                            effect.effect(),
                        )],
                    );
                    let result = parsed.validate_domain_catalogs();
                    if equipment_effect_case_is_supported(trigger, target, effect) {
                        result.unwrap_or_else(|error| {
                            panic!(
                                "supported case rejected: trigger={trigger:?} target={target:?} effect={effect:?}: {error}"
                            )
                        });
                    } else {
                        let error = result.expect_err(&format!(
                            "unsupported case must reject: trigger={trigger:?} target={target:?} effect={effect:?}"
                        ));
                        assert_eq!(error.source_file, "data/equipments/base.ron");
                        assert_eq!(error.domain, "equipment");
                        assert_eq!(error.object_id.as_deref(), Some(item_id.as_str()));
                        assert_eq!(
                            error.field_path.as_deref(),
                            Some(
                                format!("items[{item_index}].triggered_effects.{trigger:?}[0]")
                                    .as_str()
                            )
                        );
                        assert_eq!(
                            error.cause,
                            format!(
                                "unsupported equipment trigger effect: trigger={trigger:?}, target={target:?}, effect={}",
                                effect.name()
                            )
                        );
                    }
                }
            }
        }

        parsed.equipment.items[item_index].triggered_effects.clear();
        let valid_skill_id = parsed.skills.skills[0].id.clone();
        for trigger in triggers {
            parsed.equipment.items[item_index].ability_activations =
                vec![crate::game::ability::AbilityActivationBinding {
                    ability_id: valid_skill_id.clone(),
                    activation: crate::game::ability::AbilityActivationDef::TriggerProc {
                        trigger,
                        proc_chance_percent: 100,
                        internal_cooldown_ms: 0,
                        max_triggers_per_battle: None,
                    },
                }];
            let result = parsed.validate_domain_catalogs();
            if trigger == Trigger::Permanent {
                let error = result.expect_err("Permanent activation must reject");
                assert_eq!(error.object_id.as_deref(), Some(item_id.as_str()));
                assert_eq!(
                    error.field_path.as_deref(),
                    Some(
                        format!(
                            "items[{item_index}].ability_activations[0].activation.TriggerProc.trigger"
                        )
                        .as_str()
                    )
                );
                assert_eq!(
                    error.cause,
                    "unsupported equipment ability activation trigger Permanent"
                );
            } else {
                result.unwrap_or_else(|error| {
                    panic!("supported activation trigger {trigger:?} rejected: {error}")
                });
            }
        }
    }

    #[test]
    fn invalid_equipment_source_is_rejected_before_database_publication() {
        const MISSING_BUFF: &str = "__atomic_missing_equipment_buff__";
        let installed = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("current official bundle must publish");
        let installed_item_count = installed.equipment_data.items.len();
        let mut parsed = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        let (_, item_id, item) = equipment_effect_item(&mut parsed);
        item.triggered_effects.insert(
            crate::game::stats::TriggerType::OnBattleStart,
            vec![crate::game::stats::TriggeredEffect::targeted(
                crate::game::stats::TriggerEffectTarget::SelfUnit,
                crate::game::stats::Effect::ApplyBuff {
                    buff_id: MISSING_BUFF.to_string(),
                    duration_ms: 100,
                },
            )],
        );
        let invalid_equipment_ron =
            ron::ser::to_string(&parsed.equipment).expect("typed equipment fixture must serialize");
        let mut sources = GameplayRonSourceSet::embedded();
        sources.equipment = &invalid_equipment_ron;

        let error = match crate::game::data::GameDataBase::load_live_sources(sources) {
            Ok(_) => panic!("invalid equipment source must not construct a GameDataBase"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/equipments/base.ron");
        assert_eq!(error.object_id.as_deref(), Some(item_id.as_str()));
        assert!(error.cause.contains(MISSING_BUFF));
        assert_eq!(installed.equipment_data.items.len(), installed_item_count);
        assert_eq!(
            crate::game::data::ValidatedGameData::load_live_embedded()
                .expect("failed candidate must not affect the official bundle")
                .equipment_data
                .items
                .len(),
            installed_item_count
        );
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
            activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                "g12_invalid_extra_wave",
                0,
            ),
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
        let default_profile_index = broken_reference
            .run_policy
            .combat_difficulty
            .profiles
            .iter()
            .position(|profile| profile.id == "default_facility")
            .unwrap();
        broken_reference.run_policy.combat_difficulty.profiles[default_profile_index]
            .ordinary_bands[0]
            .tuning
            .extra_waves
            .push(invalid_wave(1));
        let error = broken_reference
            .validate_domain_catalogs()
            .expect_err("missing extra-wave enemy reference must reject before publication");
        assert_eq!(error.source_file, "data/run/policy.ron");
        assert_eq!(
            error.field_path.as_deref(),
            Some(
                format!(
                    "combat_difficulty.profiles[{default_profile_index}].ordinary_bands[0].tuning.extra_waves[0]"
                )
                .as_str()
            )
        );
        assert!(error.cause.contains("__missing_g12_abnormality__"));

        let mut invalid_count = parse_gameplay_ron_sources(GameplayRonSourceSet::embedded())
            .expect("live source set should parse");
        invalid_count.run_policy.combat_difficulty.profiles[default_profile_index].ordinary_bands
            [0]
        .tuning
        .extra_waves
        .push(invalid_wave(0));
        let error = invalid_count
            .validate_domain_catalogs()
            .expect_err("zero-count extra-wave enemy must reject before publication");
        assert!(error.cause.contains("count must be greater than zero"));
    }

    #[test]
    fn live_candidate_rejects_preview_failure_unique_to_manifested_difficulty() {
        use crate::game::data::pve_data::{PveWaveData, PveWaveEnemyData, PveWaveSource};

        let mut run_policy: crate::game::data::run_policy_data::RunPolicyData = parse_live_ron(
            "data/run/policy.ron",
            "run_policy",
            GameplayRonSourceSet::embedded().run_policy,
        )
        .expect("embedded run policy should parse");
        let default_profile_index = run_policy
            .combat_difficulty
            .profiles
            .iter()
            .position(|profile| profile.id == "default_facility")
            .unwrap();
        run_policy.combat_difficulty.profiles[default_profile_index].ordinary_bands[0]
            .tuning
            .extra_waves
            .push(PveWaveData {
                id: "p05_invalid_manifested_spawn_zone".to_string(),
                activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                    "p05_invalid_manifested_spawn_zone",
                    5_000,
                ),
                spawn_zone_ids: vec!["__missing_manifested_spawn_zone__".to_string()],
                route_id: Some("defense_main".to_string()),
                required_for_victory: true,
                source: PveWaveSource::Manual(vec![PveWaveEnemyData::CorrodedEmployee {
                    profile_id: "corroded_guard".to_string(),
                    tier: crate::game::enums::Tier::I,
                    count: 1,
                }]),
            });
        let run_policy_source =
            ron::ser::to_string(&run_policy).expect("run policy fixture should serialize");
        let mut sources = GameplayRonSourceSet::embedded();
        sources.run_policy = &run_policy_source;

        let error = match super::super::GameDataBase::load_live_sources(sources) {
            Ok(_) => panic!(
                "a preview failure unique to one manifested difficulty must reject publication"
            ),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/run/policy.ron");
        assert_eq!(error.field_path.as_deref(), Some(format!(
            "combat_difficulty.profiles[{default_profile_index}].ordinary_bands[0].tuning.extra_waves[0].spawn_zone_ids[0]"
        ).as_str()));
        assert!(error
            .related_sources
            .iter()
            .any(|source| source.source_file == "data/pve/encounters.ron"));
        assert_eq!(error.domain, "generated_combat_preview");
        assert!(error.cause.contains("manifested difficulty"));
        assert!(error
            .cause
            .contains("spawn wave references missing spawn zone"));
    }

    #[test]
    fn live_candidate_rejects_boss_binding_that_cannot_build_runtime_setup() {
        let catalog = crate::game::data::boss_runtime_data::tests::valid_catalog();
        let boss_source = ron::ser::to_string(&catalog).expect("Boss fixture should serialize");
        let mut pve: PveEncounterDatabase = parse_live_ron(
            "data/pve/encounters.ron",
            "pve_encounters",
            GameplayRonSourceSet::embedded().pve_encounters,
        )
        .expect("embedded PvE data should parse");
        pve.encounters
            .retain(|encounter| encounter.id != "reference_cwbs_final_boss");
        for live_encounter in &mut pve.encounters {
            live_encounter.boss_runtime = None;
        }
        let encounter = pve
            .encounters
            .iter_mut()
            .find(|encounter| encounter.id == "suppress_big_bad_wolf")
            .expect("route-less live Boss encounter");
        assert!(encounter
            .waves
            .iter()
            .find(|wave| wave.id == "primary_abnormality")
            .is_some_and(|wave| wave.route_id.is_none()));
        encounter.boss_runtime = Some(crate::game::data::pve_data::PveBossRuntimeBindingData {
            encounter_id: crate::game::data::boss_runtime_data::BossEncounterRuntimeId::new(
                "tracking_encounter",
            ),
            actors: vec![crate::game::data::pve_data::PveBossActorBindingData {
                actor_slot_id: crate::game::data::boss_runtime_data::BossActorSlotId::new(
                    "primary",
                ),
                wave_id: "primary_abnormality".to_string(),
                spawn_index: 0,
            }],
        });
        let pve_source = ron::ser::to_string(&pve).expect("PvE fixture should serialize");
        let mut sources = GameplayRonSourceSet::embedded();
        sources.boss_runtime = &boss_source;
        // Replace boss summons only; retain the official independent consumers.
        let mut summoned: crate::game::data::summoned_unit_data::SummonedUnitCatalogData =
            ron::de::from_str(sources.summoned_units).unwrap();
        summoned.definitions.clear();
        let summoned_source = ron::ser::to_string(&summoned).unwrap();
        sources.summoned_units = &summoned_source;
        sources.pve_encounters = &pve_source;

        let error = match super::super::GameDataBase::load_live_sources(sources) {
            Ok(_) => panic!("a Boss binding without its required authored route must reject"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/pve/encounters.ron", "{error:?}");
        assert_eq!(error.domain, "generated_combat_preview");
        assert!(error
            .related_sources
            .iter()
            .any(|source| source.source_file == "data/boss_runtime/base.ron"
                && source.object_id.as_deref() == Some("tracking_encounter")
                && source.field_path.as_deref() == Some("encounters")));
        assert!(error
            .related_sources
            .iter()
            .any(|source| source.source_file == "data/run/policy.ron"));
        assert!(error
            .related_sources
            .iter()
            .any(|source| source.source_file == "data/map/battlefield_templates.ron"));
        assert!(
            error.cause.contains("resolved battle setup"),
            "{}",
            error.cause
        );
        assert!(
            error.cause.contains("requires a non-empty authored route"),
            "{}",
            error.cause
        );
    }

    #[test]
    fn fixed_standard_rejects_unknown_authored_reward_before_publication() {
        let mut profiles: crate::game::map::NodeAssignmentProfileDatabase = parse_live_ron(
            "data/map/node_assignment_profiles.ron",
            "node_assignment_profiles",
            GameplayRonSourceSet::embedded().node_assignment_profiles,
        )
        .expect("embedded assignment profiles should parse");
        profiles.profile("standard_floor_3").expect("F3 profile");
        profiles
            .profiles
            .iter_mut()
            .find(|profile| profile.id == "standard_floor_3")
            .unwrap()
            .fixed_slot_assignments
            .iter_mut()
            .find(|assignment| assignment.slot_id.as_str() == "final_chamber")
            .unwrap()
            .authored_combat
            .as_mut()
            .unwrap()
            .outcome_policies
            .iter_mut()
            .find(|policy| policy.outcome_id == "victory")
            .expect("Victory node policy")
            .reward_ids[0] = "__missing_authored_reward__".to_string();
        let source = ron::ser::to_string(&profiles).unwrap();
        let mut sources = GameplayRonSourceSet::embedded();
        sources.node_assignment_profiles = &source;

        let error = match super::super::GameDataBase::load_live_sources(sources) {
            Ok(_) => panic!("unknown fixed Standard reward must reject publication"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/map/node_assignment_profiles.ron");
        assert_eq!(error.domain, "node_assignment_profiles");
        assert!(error.cause.contains("__missing_authored_reward__"));
    }

    #[test]
    fn fixed_standard_rejects_stat_override_for_non_actor_before_publication() {
        let mut profiles: crate::game::map::NodeAssignmentProfileDatabase = parse_live_ron(
            "data/map/node_assignment_profiles.ron",
            "node_assignment_profiles",
            GameplayRonSourceSet::embedded().node_assignment_profiles,
        )
        .expect("embedded assignment profiles should parse");
        profiles
            .profiles
            .iter_mut()
            .find(|profile| profile.id == "standard_floor_3")
            .unwrap()
            .fixed_slot_assignments
            .iter_mut()
            .find(|assignment| assignment.slot_id.as_str() == "final_chamber")
            .unwrap()
            .authored_combat
            .as_mut()
            .unwrap()
            .actor_stat_overrides[0]
            .abnormality_id = "o-01-45_white_night".to_string();
        let source = ron::ser::to_string(&profiles).unwrap();
        let mut sources = GameplayRonSourceSet::embedded();
        sources.node_assignment_profiles = &source;

        let error = match super::super::GameDataBase::load_live_sources(sources) {
            Ok(_) => panic!("non-actor fixed Standard override must reject publication"),
            Err(error) => error,
        };
        assert_eq!(error.source_file, "data/map/node_assignment_profiles.ron");
        assert_eq!(error.domain, "node_assignment_profiles");
        assert!(error.cause.contains("not an abnormality actor"));
        assert!(error.cause.contains("suppress_nothing_there"));
    }
}
