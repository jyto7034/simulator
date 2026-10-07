//! Closed typed catalog adapters. Parsing does not lower cross-definition
//! references or build indexes. Only the resolved winners reach `assemble`.
use super::super::abnormality_data::{RawAbnormalityDatabase, RawAbnormalityMetadata};
use super::super::boss_omen_data::{BossOmenChain, BossOmenChainDatabase};
use super::super::boss_runtime_data::{
    BossActorBehaviorDefinitionData, BossEncounterDefinitionData, BossPatternDefinitionData,
    BossRuntimeCatalogData, BossSignalDefinitionData,
};
use super::super::consumable_data::{ConsumableDatabase, ConsumableMetadata};
use super::super::corroded_employee_data::{
    CorrodedEmployeeBasicAttackRangePresetDef, RawCorrodedEmployeeProfileDatabase,
    RawCorrodedEmployeeProfileMetadata,
};
use super::super::corroded_wave_data::{CorrodedWavePreset, CorrodedWavePresetDatabase};
use super::super::employee_data::EmployeeDefinitionDatabase;
use super::super::equipment_data::{
    EquipmentDatabase, EquipmentDismantleRecipeMetadata, EquipmentEnhancementRecipeMetadata,
    EquipmentMaterialMetadata, EquipmentMetadata,
};
use super::super::event_data::{EventDatabase, EventDefinition};
use super::super::independent_object_data::IndependentObjectDefinition;
use super::super::live_ron::{
    parse_live_ron, GameplayMapCatalogs, LiveRonError, ParsedGameplayRon,
};
use super::super::pve_data::{PveEncounter, PveEncounterDatabase, PveEncounterRewardSource};
use super::super::relic_data::{RelicDatabase, RelicDefinition};
use super::super::reward_data::{
    EquipmentRewardPoolMetadata, RewardDatabase, RewardMetadata, RewardPoolMetadata,
};
use super::super::run_policy_data::RunPolicyData;
use super::super::shop_data::{
    ShopDatabase, ShopMerchandisePreset, ShopMetadata, ShopPoolMetadata,
};
use super::super::skill_data::{RawSkillDatabase, RawSkillDef, SkillRangePresetDef};
use super::super::skill_fragment_data::{SkillFragmentDatabase, SkillFragmentMetadata};
use super::super::summoned_unit_data::{SummonedUnitCatalogData, SummonedUnitDefinitionData};
use super::super::visual_novel_data::{VisualNovelDatabase, VisualNovelDefinition};
use super::super::visual_novel_trigger_data::{VisualNovelTrigger, VisualNovelTriggerDatabase};
use crate::game::battle::buffs::{BuffDatabase, BuffMetadata};
use crate::game::combat_preview::{
    BattlefieldArchetypeGenerationDefinition, BattlefieldGenerationPolicyDatabase,
    BattlefieldSizeClassDefinition, BattlefieldTemplateDefinition,
};
use crate::game::employee::EmployeeDefinition;
use crate::game::map::{
    EndlessFacilityScheduleStage, FacilityTemplate, FacilityTemplateDatabase, FacilityTemplatePool,
    MapNodeDefinition, MapNodeDefinitionDatabase, NodeAssignmentProfile,
    NodeAssignmentProfileDatabase, StandardFacilityScheduleEntry,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CatalogKind {
    Shops,
    Rewards,
    Events,
    VisualNovels,
    VisualNovelTriggers,
    BossOmenChains,
    BossRuntime,
    SummonedUnits,
    Resources,
    Abnormalities,
    CorrodedEmployees,
    CorrodedWavePresets,
    Employees,
    Equipment,
    Consumables,
    Buffs,
    Skills,
    SkillFragments,
    Relics,
    PveEncounters,
    RunPolicy,
    MapNodes,
    FacilityTemplates,
    NodeAssignmentProfiles,
    BattlefieldArchetypes,
    BattlefieldTemplates,
}

impl CatalogKind {
    pub const ALL: &'static [Self] = &[
        Self::Shops,
        Self::Rewards,
        Self::Events,
        Self::VisualNovels,
        Self::VisualNovelTriggers,
        Self::BossOmenChains,
        Self::BossRuntime,
        Self::SummonedUnits,
        Self::Resources,
        Self::Abnormalities,
        Self::CorrodedEmployees,
        Self::CorrodedWavePresets,
        Self::Employees,
        Self::Equipment,
        Self::Consumables,
        Self::Buffs,
        Self::Skills,
        Self::SkillFragments,
        Self::Relics,
        Self::PveEncounters,
        Self::RunPolicy,
        Self::MapNodes,
        Self::FacilityTemplates,
        Self::NodeAssignmentProfiles,
        Self::BattlefieldArchetypes,
        Self::BattlefieldTemplates,
    ];
    pub fn official_path(self) -> &'static str {
        match self {
            Self::Shops => "data/events/shops/base.ron",
            Self::Rewards => "data/events/rewards/base.ron",
            Self::Events => "data/events/story/base.ron",
            Self::VisualNovels => "data/visual_novels/base.ron",
            Self::VisualNovelTriggers => "data/visual_novel_triggers/base.ron",
            Self::BossOmenChains => "data/boss_omen/chains.ron",
            Self::BossRuntime => "data/boss_runtime/base.ron",
            Self::SummonedUnits => "data/summoned_units/base.ron",
            Self::Resources => "data/resources/base.ron",
            Self::Abnormalities => "data/abnormalities/base.ron",
            Self::CorrodedEmployees => "data/enemies/corroded_employees.ron",
            Self::CorrodedWavePresets => "data/enemies/corroded_wave_presets.ron",
            Self::Employees => "data/employees/definitions.ron",
            Self::Equipment => "data/equipments/base.ron",
            Self::Consumables => "data/consumables/base.ron",
            Self::Buffs => "data/buffs/base.ron",
            Self::Skills => "data/skills/base.ron",
            Self::SkillFragments => "data/skill_fragments/base.ron",
            Self::Relics => "data/relics/base.ron",
            Self::PveEncounters => "data/pve/encounters.ron",
            Self::RunPolicy => "data/run/policy.ron",
            Self::MapNodes => "data/map/node_definitions.ron",
            Self::FacilityTemplates => "data/map/facility_templates.ron",
            Self::NodeAssignmentProfiles => "data/map/node_assignment_profiles.ron",
            Self::BattlefieldArchetypes => "data/map/battlefield_archetypes.ron",
            Self::BattlefieldTemplates => "data/map/battlefield_templates.ron",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyScope {
    IndependentString,
    ReferenceKey,
    FixedKey,
}

macro_rules! definitions {
    ($( $variant:ident($ty:ty) => ($catalog:ident, $collection:literal, $scope:ident, $key:expr) ),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        pub enum DefinitionKind { $($variant),+ }

        #[derive(Debug, Clone, Serialize)]
        pub(super) enum RawDefinition { $($variant($ty)),+ }

        impl DefinitionKind {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
            pub fn catalog(self) -> CatalogKind { match self { $(Self::$variant => CatalogKind::$catalog),+ } }
            pub(super) fn collection(self) -> &'static str { match self { $(Self::$variant => $collection),+ } }
            pub(super) fn scope(self) -> KeyScope { match self { $(Self::$variant => KeyScope::$scope),+ } }
        }
        impl RawDefinition {
            pub(super) fn key(&self) -> DefinitionKey {
                match self { $(Self::$variant(value) => DefinitionKey { kind: DefinitionKind::$variant, id: ($key)(value) }),+ }
            }
        }
    };
}

definitions! {
    ShopPreset(ShopMerchandisePreset) => (Shops, "merchandise_presets", IndependentString, |value: &ShopMerchandisePreset| value.id.as_str().to_owned()),
    Shop(ShopMetadata) => (Shops, "shops", IndependentString, |value: &ShopMetadata| value.id.as_str().to_owned()),
    ShopPool(ShopPoolMetadata) => (Shops, "pools", IndependentString, |value: &ShopPoolMetadata| value.id.as_str().to_owned()),
    Reward(RewardMetadata) => (Rewards, "rewards", IndependentString, |value: &RewardMetadata| value.id.as_str().to_owned()),
    RewardPool(RewardPoolMetadata) => (Rewards, "pools", IndependentString, |value: &RewardPoolMetadata| value.id.as_str().to_owned()),
    EquipmentRewardPool(EquipmentRewardPoolMetadata) => (Rewards, "equipment_pools", IndependentString, |value: &EquipmentRewardPoolMetadata| value.id.as_str().to_owned()),
    Event(EventDefinition) => (Events, "events", IndependentString, |value: &EventDefinition| value.id.as_str().to_owned()),
    VisualNovel(VisualNovelDefinition) => (VisualNovels, "visual_novels", IndependentString, |value: &VisualNovelDefinition| value.id.as_str().to_owned()),
    VisualNovelTrigger(VisualNovelTrigger) => (VisualNovelTriggers, "triggers", IndependentString, |value: &VisualNovelTrigger| value.id.as_str().to_owned()),
    BossOmenChain(BossOmenChain) => (BossOmenChains, "chains", IndependentString, |value: &BossOmenChain| value.id.as_str().to_owned()),
    BossPattern(BossPatternDefinitionData) => (BossRuntime, "patterns", IndependentString, |value: &BossPatternDefinitionData| value.id.as_str().to_owned()),
    BossActorBehavior(BossActorBehaviorDefinitionData) => (BossRuntime, "actor_behaviors", IndependentString, |value: &BossActorBehaviorDefinitionData| value.id.as_str().to_owned()),
    BossEncounter(BossEncounterDefinitionData) => (BossRuntime, "encounters", IndependentString, |value: &BossEncounterDefinitionData| value.id.as_str().to_owned()),
    BossSignal(BossSignalDefinitionData) => (BossRuntime, "signals", IndependentString, |value: &BossSignalDefinitionData| value.id.as_str().to_owned()),
    Resource(crate::game::resource::ResourceDefinition) => (Resources, "resources", IndependentString, |value: &crate::game::resource::ResourceDefinition| value.id.as_str().to_owned()),
    SummonedUnit(SummonedUnitDefinitionData) => (SummonedUnits, "definitions", IndependentString, |value: &SummonedUnitDefinitionData| value.id.as_str().to_owned()),
    IndependentObject(IndependentObjectDefinition) => (SummonedUnits, "objects", IndependentString, |value: &IndependentObjectDefinition| value.id.as_str().to_owned()),
    Abnormality(RawAbnormalityMetadata) => (Abnormalities, "items", IndependentString, |value: &RawAbnormalityMetadata| value.id.as_str().to_owned()),
    CorrodedRangePreset(CorrodedEmployeeBasicAttackRangePresetDef) => (CorrodedEmployees, "range_presets", IndependentString, |value: &CorrodedEmployeeBasicAttackRangePresetDef| value.id.as_str().to_owned()),
    CorrodedProfile(RawCorrodedEmployeeProfileMetadata) => (CorrodedEmployees, "profiles", IndependentString, |value: &RawCorrodedEmployeeProfileMetadata| value.id.as_str().to_owned()),
    CorrodedWavePreset(CorrodedWavePreset) => (CorrodedWavePresets, "presets", IndependentString, |value: &CorrodedWavePreset| value.id.as_str().to_owned()),
    Employee(EmployeeDefinition) => (Employees, "definitions", IndependentString, |value: &EmployeeDefinition| value.employee_definition_id.as_str().to_owned()),
    Equipment(EquipmentMetadata) => (Equipment, "items", IndependentString, |value: &EquipmentMetadata| value.id.as_str().to_owned()),
    EquipmentMaterial(EquipmentMaterialMetadata) => (Equipment, "materials", IndependentString, |value: &EquipmentMaterialMetadata| value.id.as_str().to_owned()),
    DismantleRecipe(EquipmentDismantleRecipeMetadata) => (Equipment, "dismantle_recipes", ReferenceKey, |value: &EquipmentDismantleRecipeMetadata| value.equipment_id.as_str().to_owned()),
    EnhancementRecipe(EquipmentEnhancementRecipeMetadata) => (Equipment, "enhancement_recipes", ReferenceKey, |value: &EquipmentEnhancementRecipeMetadata| value.equipment_id.as_str().to_owned()),
    Consumable(ConsumableMetadata) => (Consumables, "items", IndependentString, |value: &ConsumableMetadata| value.id.as_str().to_owned()),
    Buff(BuffMetadata) => (Buffs, "buffs", IndependentString, |value: &BuffMetadata| value.name.as_str().to_owned()),
    SkillRangePreset(SkillRangePresetDef) => (Skills, "range_presets", IndependentString, |value: &SkillRangePresetDef| value.id.as_str().to_owned()),
    Skill(RawSkillDef) => (Skills, "skills", IndependentString, |value: &RawSkillDef| value.id.as_str().to_owned()),
    SkillFragment(SkillFragmentMetadata) => (SkillFragments, "fragments", IndependentString, |value: &SkillFragmentMetadata| value.id.as_str().to_owned()),
    Relic(RelicDefinition) => (Relics, "relics", IndependentString, |value: &RelicDefinition| value.id.to_owned()),
    PveEncounter(PveEncounter) => (PveEncounters, "encounters", IndependentString, |value: &PveEncounter| value.id.as_str().to_owned()),
    PveRewardSource(PveEncounterRewardSource) => (PveEncounters, "reward_sources", IndependentString, |value: &PveEncounterRewardSource| value.reward_source_id.as_str().to_owned()),
    RunPolicy(RunPolicyData) => (RunPolicy, "", FixedKey, |_value: &RunPolicyData| "run_policy".to_owned()),
    MapNode(MapNodeDefinition) => (MapNodes, "nodes", IndependentString, |value: &MapNodeDefinition| value.kind_id.as_str().to_owned()),
    FacilityTags(Vec<String>) => (FacilityTemplates, "tag_catalog", FixedKey, |_value: &Vec<String>| "tag_catalog".to_owned()),
    FacilityTemplate(FacilityTemplate) => (FacilityTemplates, "templates", IndependentString, |value: &FacilityTemplate| value.id.as_str().to_owned()),
    FacilityPool(FacilityTemplatePool) => (FacilityTemplates, "pools", IndependentString, |value: &FacilityTemplatePool| value.id.as_str().to_owned()),
    NodeAssignmentProfile(NodeAssignmentProfile) => (NodeAssignmentProfiles, "profiles", IndependentString, |value: &NodeAssignmentProfile| value.id.as_str().to_owned()),
    StandardSchedule(StandardFacilityScheduleEntry) => (NodeAssignmentProfiles, "standard_schedule", FixedKey, |value: &StandardFacilityScheduleEntry| value.floor.to_string()),
    EndlessSchedule(Vec<EndlessFacilityScheduleStage>) => (NodeAssignmentProfiles, "endless_stages", FixedKey, |_value: &Vec<EndlessFacilityScheduleStage>| "endless_stages".to_owned()),
    BattlefieldSize(BattlefieldSizeClassDefinition) => (BattlefieldArchetypes, "size_classes", FixedKey, |value: &BattlefieldSizeClassDefinition| format!("{:?}", value.size_class)),
    BattlefieldArchetype(BattlefieldArchetypeGenerationDefinition) => (BattlefieldArchetypes, "archetypes", FixedKey, |value: &BattlefieldArchetypeGenerationDefinition| format!("{:?}", value.archetype)),
    BattlefieldTemplate(BattlefieldTemplateDefinition) => (BattlefieldTemplates, "", IndependentString, |value: &BattlefieldTemplateDefinition| value.id.as_str().to_owned()),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefinitionKey {
    pub kind: DefinitionKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefinitionSource {
    pub pack_id: String,
    pub source_file: String,
    pub field_path: String,
}

#[derive(Debug, Clone)]
pub(super) struct LocatedDefinition {
    pub definition: RawDefinition,
    pub source: DefinitionSource,
    pub source_order: usize,
}

impl CatalogKind {
    pub(super) fn parse(
        self,
        pack_id: &str,
        source_file: &str,
        input: &str,
        limits: &super::InputLimits,
    ) -> Result<Vec<LocatedDefinition>, LiveRonError> {
        let mut definitions = Vec::new();
        let mut push = |definition, field_path| -> Result<(), LiveRonError> {
            use crate::authoring::control::{self, Resource};
            control::admit(
                Resource::Definitions,
                definitions.len(),
                1,
                "raw_definition",
            )
            .map_err(super::control_error)?;
            control::charge(1, "raw_definition").map_err(super::control_error)?;
            let source_order = definitions.len();
            definitions.push(LocatedDefinition {
                definition,
                source_order,
                source: DefinitionSource {
                    pack_id: pack_id.into(),
                    source_file: source_file.into(),
                    field_path,
                },
            });
            Ok(())
        };
        match self {
            Self::Shops => {
                let catalog: ShopDatabase = parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.merchandise_presets.into_iter().enumerate() {
                    push(
                        RawDefinition::ShopPreset(value),
                        format!("merchandise_presets[{index}]"),
                    )?;
                }
                for (index, value) in catalog.shops.into_iter().enumerate() {
                    push(RawDefinition::Shop(value), format!("shops[{index}]"))?;
                }
                for (index, value) in catalog.pools.into_iter().enumerate() {
                    push(RawDefinition::ShopPool(value), format!("pools[{index}]"))?;
                }
            }
            Self::Rewards => {
                let catalog: RewardDatabase = parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.rewards.into_iter().enumerate() {
                    push(RawDefinition::Reward(value), format!("rewards[{index}]"))?;
                }
                for (index, value) in catalog.pools.into_iter().enumerate() {
                    push(RawDefinition::RewardPool(value), format!("pools[{index}]"))?;
                }
                for (index, value) in catalog.equipment_pools.into_iter().enumerate() {
                    push(
                        RawDefinition::EquipmentRewardPool(value),
                        format!("equipment_pools[{index}]"),
                    )?;
                }
            }
            Self::Events => {
                let catalog: EventDatabase = parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.events.into_iter().enumerate() {
                    push(RawDefinition::Event(value), format!("events[{index}]"))?;
                }
            }
            Self::VisualNovels => {
                let catalog: VisualNovelDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.visual_novels.into_iter().enumerate() {
                    push(
                        RawDefinition::VisualNovel(value),
                        format!("visual_novels[{index}]"),
                    )?;
                }
            }
            Self::VisualNovelTriggers => {
                let catalog: VisualNovelTriggerDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.triggers.into_iter().enumerate() {
                    push(
                        RawDefinition::VisualNovelTrigger(value),
                        format!("triggers[{index}]"),
                    )?;
                }
            }
            Self::BossOmenChains => {
                let catalog: BossOmenChainDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.chains.into_iter().enumerate() {
                    push(
                        RawDefinition::BossOmenChain(value),
                        format!("chains[{index}]"),
                    )?;
                }
            }
            Self::BossRuntime => {
                let catalog: BossRuntimeCatalogData =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.patterns.into_iter().enumerate() {
                    push(
                        RawDefinition::BossPattern(value),
                        format!("patterns[{index}]"),
                    )?;
                }
                for (index, value) in catalog.actor_behaviors.into_iter().enumerate() {
                    push(
                        RawDefinition::BossActorBehavior(value),
                        format!("actor_behaviors[{index}]"),
                    )?;
                }
                for (index, value) in catalog.encounters.into_iter().enumerate() {
                    push(
                        RawDefinition::BossEncounter(value),
                        format!("encounters[{index}]"),
                    )?;
                }
                for (index, value) in catalog.signals.into_iter().enumerate() {
                    push(
                        RawDefinition::BossSignal(value),
                        format!("signals[{index}]"),
                    )?;
                }
            }
            Self::Resources => {
                let catalog: crate::game::resource::ResourceCatalog =
                    parse_live_ron(source_file, "resources", input)?;
                for (index, value) in catalog.resources.into_iter().enumerate() {
                    push(
                        RawDefinition::Resource(value),
                        format!("resources[{index}]"),
                    )?;
                }
            }
            Self::SummonedUnits => {
                let catalog: SummonedUnitCatalogData =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.definitions.into_iter().enumerate() {
                    push(
                        RawDefinition::SummonedUnit(value),
                        format!("definitions[{index}]"),
                    )?;
                }
                for (index, value) in catalog.objects.into_iter().enumerate() {
                    push(
                        RawDefinition::IndependentObject(value),
                        format!("objects[{index}]"),
                    )?;
                }
            }
            Self::Abnormalities => {
                let catalog: RawAbnormalityDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.items.into_iter().enumerate() {
                    push(RawDefinition::Abnormality(value), format!("items[{index}]"))?;
                }
            }
            Self::CorrodedEmployees => {
                let catalog: RawCorrodedEmployeeProfileDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.range_presets.into_iter().enumerate() {
                    push(
                        RawDefinition::CorrodedRangePreset(value),
                        format!("range_presets[{index}]"),
                    )?;
                }
                for (index, value) in catalog.profiles.into_iter().enumerate() {
                    push(
                        RawDefinition::CorrodedProfile(value),
                        format!("profiles[{index}]"),
                    )?;
                }
            }
            Self::CorrodedWavePresets => {
                let catalog: CorrodedWavePresetDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.presets.into_iter().enumerate() {
                    push(
                        RawDefinition::CorrodedWavePreset(value),
                        format!("presets[{index}]"),
                    )?;
                }
            }
            Self::Employees => {
                let catalog: EmployeeDefinitionDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.definitions.into_iter().enumerate() {
                    push(
                        RawDefinition::Employee(value),
                        format!("definitions[{index}]"),
                    )?;
                }
            }
            Self::Equipment => {
                let catalog: EquipmentDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.items.into_iter().enumerate() {
                    push(RawDefinition::Equipment(value), format!("items[{index}]"))?;
                }
                for (index, value) in catalog.materials.into_iter().enumerate() {
                    push(
                        RawDefinition::EquipmentMaterial(value),
                        format!("materials[{index}]"),
                    )?;
                }
                for (index, value) in catalog.dismantle_recipes.into_iter().enumerate() {
                    push(
                        RawDefinition::DismantleRecipe(value),
                        format!("dismantle_recipes[{index}]"),
                    )?;
                }
                for (index, value) in catalog.enhancement_recipes.into_iter().enumerate() {
                    push(
                        RawDefinition::EnhancementRecipe(value),
                        format!("enhancement_recipes[{index}]"),
                    )?;
                }
            }
            Self::Consumables => {
                let catalog: ConsumableDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.items.into_iter().enumerate() {
                    push(RawDefinition::Consumable(value), format!("items[{index}]"))?;
                }
            }
            Self::Buffs => {
                let catalog: BuffDatabase = parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.buffs.into_iter().enumerate() {
                    push(RawDefinition::Buff(value), format!("buffs[{index}]"))?;
                }
            }
            Self::Skills => {
                let catalog: RawSkillDatabase =
                    super::super::skill_data::RawSkillDatabase::from_ron_with_limits(
                        source_file,
                        input,
                        &limits.value_expr,
                        limits.max_nesting,
                    )?;
                for (index, value) in catalog.range_presets.into_iter().enumerate() {
                    push(
                        RawDefinition::SkillRangePreset(value),
                        format!("range_presets[{index}]"),
                    )?;
                }
                for (index, value) in catalog.skills.into_iter().enumerate() {
                    push(RawDefinition::Skill(value), format!("skills[{index}]"))?;
                }
            }
            Self::Relics => {
                let catalog: RelicDatabase = parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.relics.into_iter().enumerate() {
                    push(RawDefinition::Relic(value), format!("relics[{index}]"))?;
                }
            }
            Self::SkillFragments => {
                let catalog: SkillFragmentDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.fragments.into_iter().enumerate() {
                    push(
                        RawDefinition::SkillFragment(value),
                        format!("fragments[{index}]"),
                    )?;
                }
            }
            Self::PveEncounters => {
                let catalog: PveEncounterDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.encounters.into_iter().enumerate() {
                    push(
                        RawDefinition::PveEncounter(value),
                        format!("encounters[{index}]"),
                    )?;
                }
                for (index, value) in catalog.reward_sources.into_iter().enumerate() {
                    push(
                        RawDefinition::PveRewardSource(value),
                        format!("reward_sources[{index}]"),
                    )?;
                }
            }
            Self::RunPolicy => {
                let catalog: RunPolicyData = parse_live_ron(source_file, "content_pack", input)?;
                push(RawDefinition::RunPolicy(catalog), "".to_owned())?;
            }
            Self::MapNodes => {
                let catalog: MapNodeDefinitionDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.nodes.into_iter().enumerate() {
                    push(RawDefinition::MapNode(value), format!("nodes[{index}]"))?;
                }
            }
            Self::FacilityTemplates => {
                let catalog: FacilityTemplateDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                push(
                    RawDefinition::FacilityTags(catalog.tag_catalog),
                    "tag_catalog".to_owned(),
                )?;
                for (index, value) in catalog.templates.into_iter().enumerate() {
                    push(
                        RawDefinition::FacilityTemplate(value),
                        format!("templates[{index}]"),
                    )?;
                }
                for (index, value) in catalog.pools.into_iter().enumerate() {
                    push(
                        RawDefinition::FacilityPool(value),
                        format!("pools[{index}]"),
                    )?;
                }
            }
            Self::NodeAssignmentProfiles => {
                let catalog: NodeAssignmentProfileDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.profiles.into_iter().enumerate() {
                    push(
                        RawDefinition::NodeAssignmentProfile(value),
                        format!("profiles[{index}]"),
                    )?;
                }
                for (index, value) in catalog.standard_schedule.into_iter().enumerate() {
                    push(
                        RawDefinition::StandardSchedule(value),
                        format!("standard_schedule[{index}]"),
                    )?;
                }
                push(
                    RawDefinition::EndlessSchedule(catalog.endless_stages),
                    "endless_stages".to_owned(),
                )?;
            }
            Self::BattlefieldArchetypes => {
                let catalog: BattlefieldGenerationPolicyDatabase =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.size_classes.into_iter().enumerate() {
                    push(
                        RawDefinition::BattlefieldSize(value),
                        format!("size_classes[{index}]"),
                    )?;
                }
                for (index, value) in catalog.archetypes.into_iter().enumerate() {
                    push(
                        RawDefinition::BattlefieldArchetype(value),
                        format!("archetypes[{index}]"),
                    )?;
                }
            }
            Self::BattlefieldTemplates => {
                let catalog: Vec<BattlefieldTemplateDefinition> =
                    parse_live_ron(source_file, "content_pack", input)?;
                for (index, value) in catalog.into_iter().enumerate() {
                    push(
                        RawDefinition::BattlefieldTemplate(value),
                        format!("[{index}]"),
                    )?;
                }
            }
        }
        Ok(definitions)
    }
}

/// Field relocation retained separately from semantic identity. Records follow
/// the actual final collection order, including a replacement's inherited slot.
#[derive(Debug, Clone)]
pub(super) struct SourceTable(pub Vec<(DefinitionKey, DefinitionSource)>);

impl SourceTable {
    pub(super) fn from_definitions(definitions: &[LocatedDefinition]) -> Self {
        Self(
            definitions
                .iter()
                .map(|d| (d.definition.key(), d.source.clone()))
                .collect(),
        )
    }
    pub(super) fn relocate(&self, mut error: LiveRonError) -> LiveRonError {
        for source in &mut error.related_sources {
            let relocated = self.relocate(LiveRonError::validation(
                &source.source_file,
                error.domain,
                source.object_id.clone(),
                source.field_path.clone(),
                "",
            ));
            source.source_file = relocated.source_file;
            source.object_id = relocated.object_id;
            source.field_path = relocated.field_path;
            source.pack_id = relocated.pack_id.or_else(|| source.pack_id.clone());
            source.catalog_kind = relocated.catalog_kind.or(source.catalog_kind);
            source.definition_kind = relocated.definition_kind.or(source.definition_kind);
        }
        // EffectSource is already bound to the selected writer's actual file.
        // Attach its pack/kind without prefixing the field path a second time.
        let mut exact = self.0.iter().filter(|(key, source)| {
            source.source_file == error.source_file
                && error.object_id.as_deref() == Some(key.id.as_str())
        });
        if let Some((key, source)) = exact.next() {
            if exact.next().is_none() {
                return error.with_pack(&source.pack_id).with_definition(key.kind);
            }
        }
        let path = error.field_path.as_deref().unwrap_or("");
        let mut counts = std::collections::BTreeMap::<DefinitionKind, usize>::new();
        for (key, source) in &self.0 {
            if key.kind.catalog().official_path() != error.source_file {
                continue;
            }
            if !path.is_empty()
                && path == key.kind.collection()
                && error.object_id.as_deref() == Some(key.id.as_str())
            {
                error.source_file = source.source_file.clone();
                error.field_path = Some(source.field_path.clone());
                return error.with_pack(&source.pack_id).with_definition(key.kind);
            }
            let index = counts.entry(key.kind).or_default();
            let prefix = match key.kind {
                DefinitionKind::RunPolicy
                | DefinitionKind::FacilityTags
                | DefinitionKind::EndlessSchedule => key.kind.collection().to_owned(),
                _ => format!("{}[{}]", key.kind.collection(), index),
            };
            *index += 1;
            let suffix = path
                .strip_prefix(&prefix)
                .filter(|tail| tail.is_empty() || tail.starts_with('.') || tail.starts_with('['));
            if let Some(suffix) = suffix {
                error.source_file = source.source_file.clone();
                error.field_path = Some(format!("{}{suffix}", source.field_path));
                return error.with_pack(&source.pack_id).with_definition(key.kind);
            }
        }
        // Some owner diagnostics identify only the object. Relocate only when
        // that key is unambiguous in the source domain.
        let candidates = self
            .0
            .iter()
            .filter(|(key, _)| {
                key.kind.catalog().official_path() == error.source_file
                    && error.object_id.as_deref() == Some(key.id.as_str())
            })
            .collect::<Vec<_>>();
        if let [(key, source)] = candidates.as_slice() {
            error.source_file = source.source_file.clone();
            if !source.field_path.is_empty() {
                error.field_path = Some(match error.field_path {
                    Some(path) if !path.is_empty() => format!("{}.{}", source.field_path, path),
                    _ => source.field_path.clone(),
                });
            }
            return error.with_pack(&source.pack_id).with_definition(key.kind);
        }
        let files: std::collections::BTreeSet<_> = self
            .0
            .iter()
            .filter(|(key, _)| key.kind.catalog().official_path() == error.source_file)
            .map(|(_, source)| source.source_file.as_str())
            .collect();
        if files.len() == 1 {
            let file = files.into_iter().next().unwrap();
            if let Some((key, source)) =
                self.0.iter().find(|(_, source)| source.source_file == file)
            {
                error.pack_id = Some(source.pack_id.clone());
                error.catalog_kind = Some(key.kind.catalog());
            }
            error.source_file = file.to_owned();
        } else if !files.is_empty() {
            for file in &files {
                if let Some((key, source)) = self
                    .0
                    .iter()
                    .find(|(_, source)| source.source_file == *file)
                {
                    let related = super::super::live_ron::LiveRonSource {
                        source_file: source.source_file.clone(),
                        pack_id: Some(source.pack_id.clone()),
                        catalog_kind: Some(key.kind.catalog()),
                        ..Default::default()
                    };
                    if !error.related_sources.contains(&related) {
                        error.related_sources.push(related);
                    }
                }
            }
            error.cause = format!(
                "{}; contributing sources: {}",
                error.cause,
                files.into_iter().collect::<Vec<_>>().join(", ")
            );
            error.source_file = format!("resolved:{}", error.source_file);
        }
        error
    }
}

pub(super) fn assemble(
    definitions: Vec<LocatedDefinition>,
) -> Result<ParsedGameplayRon, LiveRonError> {
    let sources = SourceTable::from_definitions(&definitions);
    let mut raw_shop_preset: Vec<ShopMerchandisePreset> = Default::default();
    let mut raw_shop: Vec<ShopMetadata> = Default::default();
    let mut raw_shop_pool: Vec<ShopPoolMetadata> = Default::default();
    let mut raw_reward: Vec<RewardMetadata> = Default::default();
    let mut raw_reward_pool: Vec<RewardPoolMetadata> = Default::default();
    let mut raw_equipment_reward_pool: Vec<EquipmentRewardPoolMetadata> = Default::default();
    let mut raw_event: Vec<EventDefinition> = Default::default();
    let mut raw_visual_novel: Vec<VisualNovelDefinition> = Default::default();
    let mut raw_visual_novel_trigger: Vec<VisualNovelTrigger> = Default::default();
    let mut raw_boss_omen_chain: Vec<BossOmenChain> = Default::default();
    let mut raw_boss_pattern: Vec<BossPatternDefinitionData> = Default::default();
    let mut raw_boss_actor_behavior: Vec<BossActorBehaviorDefinitionData> = Default::default();
    let mut raw_boss_encounter: Vec<BossEncounterDefinitionData> = Default::default();
    let mut raw_boss_signal: Vec<BossSignalDefinitionData> = Default::default();
    let mut raw_resource = Vec::new();
    let mut raw_summoned_unit: Vec<SummonedUnitDefinitionData> = Default::default();
    let mut raw_independent_object: Vec<IndependentObjectDefinition> = Default::default();
    let mut raw_abnormality: Vec<RawAbnormalityMetadata> = Default::default();
    let mut raw_corroded_range_preset: Vec<CorrodedEmployeeBasicAttackRangePresetDef> =
        Default::default();
    let mut raw_corroded_profile: Vec<RawCorrodedEmployeeProfileMetadata> = Default::default();
    let mut raw_corroded_wave_preset: Vec<CorrodedWavePreset> = Default::default();
    let mut raw_employee: Vec<EmployeeDefinition> = Default::default();
    let mut raw_equipment: Vec<EquipmentMetadata> = Default::default();
    let mut raw_equipment_material: Vec<EquipmentMaterialMetadata> = Default::default();
    let mut raw_dismantle_recipe: Vec<EquipmentDismantleRecipeMetadata> = Default::default();
    let mut raw_enhancement_recipe: Vec<EquipmentEnhancementRecipeMetadata> = Default::default();
    let mut raw_consumable: Vec<ConsumableMetadata> = Default::default();
    let mut raw_buff: Vec<BuffMetadata> = Default::default();
    let mut raw_skill_range_preset: Vec<SkillRangePresetDef> = Default::default();
    let mut raw_skill: Vec<RawSkillDef> = Default::default();
    let mut raw_skill_fragment: Vec<SkillFragmentMetadata> = Default::default();
    let mut raw_relic: Vec<RelicDefinition> = Default::default();
    let mut raw_pve_encounter: Vec<PveEncounter> = Default::default();
    let mut raw_pve_reward_source: Vec<PveEncounterRewardSource> = Default::default();
    let mut raw_run_policy: Option<RunPolicyData> = Default::default();
    let mut raw_map_node: Vec<MapNodeDefinition> = Default::default();
    let mut raw_facility_tags: Option<Vec<String>> = Default::default();
    let mut raw_facility_template: Vec<FacilityTemplate> = Default::default();
    let mut raw_facility_pool: Vec<FacilityTemplatePool> = Default::default();
    let mut raw_node_assignment_profile: Vec<NodeAssignmentProfile> = Default::default();
    let mut raw_standard_schedule: Vec<StandardFacilityScheduleEntry> = Default::default();
    let mut raw_endless_schedule: Option<Vec<EndlessFacilityScheduleStage>> = Default::default();
    let mut raw_battlefield_size: Vec<BattlefieldSizeClassDefinition> = Default::default();
    let mut raw_battlefield_archetype: Vec<BattlefieldArchetypeGenerationDefinition> =
        Default::default();
    let mut raw_battlefield_template: Vec<BattlefieldTemplateDefinition> = Default::default();
    for located in definitions {
        match located.definition {
            RawDefinition::ShopPreset(value) => {
                raw_shop_preset.push(value);
            }
            RawDefinition::Shop(value) => {
                raw_shop.push(value);
            }
            RawDefinition::ShopPool(value) => {
                raw_shop_pool.push(value);
            }
            RawDefinition::Reward(value) => {
                raw_reward.push(value);
            }
            RawDefinition::RewardPool(value) => {
                raw_reward_pool.push(value);
            }
            RawDefinition::EquipmentRewardPool(value) => {
                raw_equipment_reward_pool.push(value);
            }
            RawDefinition::Event(value) => {
                raw_event.push(value);
            }
            RawDefinition::VisualNovel(value) => {
                raw_visual_novel.push(value);
            }
            RawDefinition::VisualNovelTrigger(value) => {
                raw_visual_novel_trigger.push(value);
            }
            RawDefinition::BossOmenChain(value) => {
                raw_boss_omen_chain.push(value);
            }
            RawDefinition::BossPattern(value) => {
                raw_boss_pattern.push(value);
            }
            RawDefinition::BossActorBehavior(value) => {
                raw_boss_actor_behavior.push(value);
            }
            RawDefinition::BossEncounter(value) => {
                raw_boss_encounter.push(value);
            }
            RawDefinition::BossSignal(value) => {
                raw_boss_signal.push(value);
            }
            RawDefinition::IndependentObject(value) => {
                raw_independent_object.push(value);
            }
            RawDefinition::SummonedUnit(value) => {
                raw_summoned_unit.push(value);
            }
            RawDefinition::Abnormality(value) => {
                raw_abnormality.push(value);
            }
            RawDefinition::CorrodedRangePreset(value) => {
                raw_corroded_range_preset.push(value);
            }
            RawDefinition::CorrodedProfile(value) => {
                raw_corroded_profile.push(value);
            }
            RawDefinition::CorrodedWavePreset(value) => {
                raw_corroded_wave_preset.push(value);
            }
            RawDefinition::Employee(value) => {
                raw_employee.push(value);
            }
            RawDefinition::Equipment(value) => {
                raw_equipment.push(value);
            }
            RawDefinition::EquipmentMaterial(value) => {
                raw_equipment_material.push(value);
            }
            RawDefinition::DismantleRecipe(value) => {
                raw_dismantle_recipe.push(value);
            }
            RawDefinition::EnhancementRecipe(value) => {
                raw_enhancement_recipe.push(value);
            }
            RawDefinition::Consumable(value) => {
                raw_consumable.push(value);
            }
            RawDefinition::Buff(value) => {
                raw_buff.push(value);
            }
            RawDefinition::SkillRangePreset(value) => {
                raw_skill_range_preset.push(value);
            }
            RawDefinition::Resource(value) => {
                raw_resource.push(value);
            }
            RawDefinition::Skill(value) => {
                raw_skill.push(value);
            }
            RawDefinition::Relic(value) => {
                raw_relic.push(value);
            }
            RawDefinition::SkillFragment(value) => {
                raw_skill_fragment.push(value);
            }
            RawDefinition::PveEncounter(value) => {
                raw_pve_encounter.push(value);
            }
            RawDefinition::PveRewardSource(value) => {
                raw_pve_reward_source.push(value);
            }
            RawDefinition::RunPolicy(value) => {
                if raw_run_policy.replace(value).is_some() {
                    return Err(LiveRonError::validation(
                        located.source.source_file,
                        "content_pack",
                        Some("run_policy"),
                        Some(located.source.field_path),
                        "duplicate resolved singleton",
                    ));
                };
            }
            RawDefinition::MapNode(value) => {
                raw_map_node.push(value);
            }
            RawDefinition::FacilityTags(value) => {
                if raw_facility_tags.replace(value).is_some() {
                    return Err(LiveRonError::validation(
                        located.source.source_file,
                        "content_pack",
                        Some("tag_catalog"),
                        Some(located.source.field_path),
                        "duplicate resolved singleton",
                    ));
                };
            }
            RawDefinition::FacilityTemplate(value) => {
                raw_facility_template.push(value);
            }
            RawDefinition::FacilityPool(value) => {
                raw_facility_pool.push(value);
            }
            RawDefinition::NodeAssignmentProfile(value) => {
                raw_node_assignment_profile.push(value);
            }
            RawDefinition::StandardSchedule(value) => {
                raw_standard_schedule.push(value);
            }
            RawDefinition::EndlessSchedule(value) => {
                if raw_endless_schedule.replace(value).is_some() {
                    return Err(LiveRonError::validation(
                        located.source.source_file,
                        "content_pack",
                        Some("endless_stages"),
                        Some(located.source.field_path),
                        "duplicate resolved singleton",
                    ));
                };
            }
            RawDefinition::BattlefieldSize(value) => {
                raw_battlefield_size.push(value);
            }
            RawDefinition::BattlefieldArchetype(value) => {
                raw_battlefield_archetype.push(value);
            }
            RawDefinition::BattlefieldTemplate(value) => {
                raw_battlefield_template.push(value);
            }
        }
    }
    let raw_run_policy = raw_run_policy.ok_or_else(|| {
        LiveRonError::validation(
            "data/run/policy.ron",
            "run_policy",
            Some("run_policy"),
            None::<String>,
            "final catalog is missing run policy",
        )
    })?;
    let raw_facility_tags = raw_facility_tags.unwrap_or_default();
    let raw_endless_schedule = raw_endless_schedule.unwrap_or_default();
    let mut skills = RawSkillDatabase {
        range_presets: raw_skill_range_preset,
        skills: raw_skill,
    }
    .try_into_unvalidated_database()
    .map_err(|e| {
        sources.relocate(LiveRonError::validation(
            "data/skills/base.ron",
            "skills",
            e.object_id,
            e.field_path,
            e.cause,
        ))
    })?;
    let skill_sources: Vec<_> = sources
        .0
        .iter()
        .filter(|(k, _)| k.kind == DefinitionKind::Skill)
        .map(|(_, s)| s)
        .collect();
    skills.bind_definition_sources(|index, _| {
        (
            skill_sources[index].source_file.clone(),
            skill_sources[index].field_path.clone(),
        )
    });
    let corroded_employees = RawCorrodedEmployeeProfileDatabase {
        range_presets: raw_corroded_range_preset,
        profiles: raw_corroded_profile,
    }
    .try_into_database()
    .map_err(|e| {
        sources.relocate(LiveRonError::validation(
            "data/enemies/corroded_employees.ron",
            "corroded_employees",
            e.object_id,
            e.field_path,
            e.cause,
        ))
    })?;
    Ok(ParsedGameplayRon {
        shops: ShopDatabase::from_raw_parts(raw_shop_preset, raw_shop, raw_shop_pool),
        rewards: RewardDatabase::from_raw_parts(
            raw_reward,
            raw_reward_pool,
            raw_equipment_reward_pool,
        ),
        events: EventDatabase::from_raw_parts(raw_event),
        visual_novels: VisualNovelDatabase::from_raw_parts(raw_visual_novel),
        visual_novel_triggers: VisualNovelTriggerDatabase::from_raw_parts(raw_visual_novel_trigger),
        boss_omen_chains: BossOmenChainDatabase::from_raw_parts(raw_boss_omen_chain),
        boss_runtime: BossRuntimeCatalogData {
            patterns: raw_boss_pattern,
            actor_behaviors: raw_boss_actor_behavior,
            encounters: raw_boss_encounter,
            signals: raw_boss_signal,
        },
        resources: crate::game::resource::ResourceCatalog {
            resources: raw_resource,
        },
        summoned_units: SummonedUnitCatalogData {
            objects: raw_independent_object,
            definitions: raw_summoned_unit,
        },
        abnormalities: RawAbnormalityDatabase {
            items: raw_abnormality,
        }
        .into_database(),
        corroded_employees: corroded_employees,
        corroded_wave_presets: CorrodedWavePresetDatabase::from_raw_parts(raw_corroded_wave_preset),
        employee_definitions: EmployeeDefinitionDatabase::from_raw_parts(raw_employee),
        equipment: EquipmentDatabase::with_all(
            raw_equipment,
            raw_equipment_material,
            raw_dismantle_recipe,
            raw_enhancement_recipe,
        ),
        consumables: ConsumableDatabase::from_raw_parts(raw_consumable),
        buffs: BuffDatabase::from_raw_parts(raw_buff),
        skills: skills,
        skill_fragments: SkillFragmentDatabase::from_raw_parts(raw_skill_fragment),
        relics: RelicDatabase { relics: raw_relic },
        pve_encounters: PveEncounterDatabase::from_raw_parts(
            raw_pve_encounter,
            raw_pve_reward_source,
        ),
        run_policy: raw_run_policy,
        map_catalogs: GameplayMapCatalogs {
            map_node_definitions: Arc::new(MapNodeDefinitionDatabase {
                nodes: raw_map_node,
            }),
            facility_templates: Arc::new(FacilityTemplateDatabase {
                tag_catalog: raw_facility_tags,
                templates: raw_facility_template,
                pools: raw_facility_pool,
            }),
            node_assignment_profiles: Arc::new(NodeAssignmentProfileDatabase {
                profiles: raw_node_assignment_profile,
                standard_schedule: raw_standard_schedule,
                endless_stages: raw_endless_schedule,
            }),
            battlefield_archetypes: Arc::new(BattlefieldGenerationPolicyDatabase {
                size_classes: raw_battlefield_size,
                archetypes: raw_battlefield_archetype,
            }),
            battlefield_templates: Arc::new(
                crate::game::combat_preview::BattlefieldTemplateDatabase {
                    templates: raw_battlefield_template,
                },
            ),
        },
    })
}

#[cfg(test)]
mod tests;
