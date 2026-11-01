use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::game::data::boss_omen_data::{BossOmenRevealLevel, BossOmenSourceKind};
use crate::game::map::facility::FacilityTerminalResolution;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MapNodeId(pub Uuid);

impl MapNodeId {
    pub const fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct MapNodeKindId(pub String);

impl MapNodeKindId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapNodeCategory {
    Start,
    Combat,
    SavePoint,
    Maintenance,
    HeadquartersContact,
    Event,
    Shop,
    Boss,
    Gate,
    Reward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeadquartersContactOption {
    RecruitEmployee,
    RequestEmergencySupplies,
    OpenHeadquartersShop,
}

fn default_headquarters_candidate_count() -> usize {
    3
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MapNodePayload {
    None,
    SavePoint,
    Maintenance,
    HeadquartersContact {
        shop_pool_id: Option<String>,
        #[serde(default = "default_headquarters_candidate_count")]
        candidate_count: usize,
    },
    Event {
        event_id: Option<crate::game::data::event_data::EventId>,
    },
    Encounter {
        encounter_id: Option<String>,
    },
    Shop {
        shop_id: Option<String>,
        shop_pool_id: Option<String>,
    },
    Reward {
        reward_pool_id: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredEncounterActorStatOverride {
    pub abnormality_id: String,
    pub max_health: u32,
    pub attack: u32,
    pub defense: i32,
    pub magic_resistance: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredCombatNodePolicy {
    pub outcome_policies: Vec<AuthoredCombatOutcomePolicy>,
    #[serde(default)]
    pub actor_stat_overrides: Vec<AuthoredEncounterActorStatOverride>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredCombatOutcomePolicy {
    pub outcome_id: String,
    pub reward_ids: Vec<String>,
    pub grant_node_bonus: bool,
    pub advance_encounter_progression: bool,
}

impl Default for MapNodePayload {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapNodeState {
    Available,
    Completed,
    Locked,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapNodeVisibility {
    Concealed,
    Obscured,
    Revealed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapEdgeDirection {
    Bidirectional,
    ForwardOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct MapTemplateId(pub String);

impl MapTemplateId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapSlotId(pub String);

impl MapSlotId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapNode {
    pub id: MapNodeId,
    pub slot_id: MapSlotId,
    pub kind_id: MapNodeKindId,
    pub category: MapNodeCategory,
    pub state: MapNodeState,
    pub visibility: MapNodeVisibility,
    #[serde(default)]
    pub payload: MapNodePayload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub omen: Option<MapNodeOmenOverlayDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat_difficulty: Option<MapNodeCombatDifficulty>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapNodeCombatDifficulty {
    pub power_model_version: u32,
    pub difficulty_profile_id: String,
    pub class: MapNodeCombatDifficultyClass,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub revival: Option<crate::game::revival::RevivalNodeTuning>,
    pub floor_power_anchor: u64,
    pub target_power: u64,
}

fn deserialize_required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

pub use crate::game::revival::RevivalStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapNodeCombatDifficultyClass {
    Ordinary(crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand),
    Role(crate::game::data::run_policy_data::CombatNodeDifficultyRole),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapNodeDefinition {
    pub kind_id: MapNodeKindId,
    /// Omission inherits the run's authored default; explicit zero is a free activity.
    #[serde(default)]
    pub first_entry_cost: Option<u32>,
    pub category: MapNodeCategory,
    pub weight: u32,
    #[serde(default)]
    pub payload: MapNodePayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapNodeDefinitionDatabase {
    pub nodes: Vec<MapNodeDefinition>,
}

impl MapNodeDefinitionDatabase {
    pub fn from_ron_str(input: &str) -> Result<Self, String> {
        ron::de::from_str(input).map_err(|err| err.to_string())
    }

    pub fn validate_contract(&self) -> Result<(), String> {
        if self.nodes.is_empty() {
            return Err("map node definitions must not be empty".to_string());
        }

        let mut seen = HashSet::new();
        for definition in &self.nodes {
            if definition.kind_id.as_str().is_empty() {
                return Err("map node definition kind_id must not be empty".to_string());
            }
            if !seen.insert(definition.kind_id.as_str()) {
                return Err(format!(
                    "duplicate map node definition kind_id '{}'",
                    definition.kind_id.as_str()
                ));
            }
            if definition.category == MapNodeCategory::Event {
                match &definition.payload {
                    MapNodePayload::Event { event_id: Some(id) }
                        if !id.as_str().trim().is_empty() => {}
                    MapNodePayload::Event { event_id: Some(_) } => {
                        return Err(format!(
                            "event map node definition '{}' event_id must not be empty",
                            definition.kind_id.as_str()
                        ));
                    }
                    MapNodePayload::Event { event_id: None } => {
                        return Err(format!(
                            "event map node definition '{}' must define event_id",
                            definition.kind_id.as_str()
                        ));
                    }
                    _ => {
                        return Err(format!(
                            "event map node definition '{}' must use Event payload",
                            definition.kind_id.as_str()
                        ));
                    }
                }
            }
        }

        if !self
            .nodes
            .iter()
            .any(|definition| definition.category == MapNodeCategory::Boss)
        {
            return Err("map node definitions must include at least one Boss entry".to_string());
        }
        if !self
            .nodes
            .iter()
            .any(|definition| definition.category == MapNodeCategory::Event)
        {
            return Err("map node definitions must include at least one Event entry".to_string());
        }
        if !self
            .nodes
            .iter()
            .any(|definition| definition.category == MapNodeCategory::Gate)
        {
            return Err("map node definitions must include at least one Gate entry".to_string());
        }
        if !self.nodes.iter().any(|definition| {
            !matches!(
                definition.category,
                MapNodeCategory::Boss | MapNodeCategory::Gate
            ) && definition.weight > 0
        }) {
            return Err(
                "map node definitions must include at least one weighted non-terminal entry"
                    .to_string(),
            );
        }

        Ok(())
    }

    pub fn boss_candidates(&self) -> Vec<&MapNodeDefinition> {
        self.nodes
            .iter()
            .filter(|definition| definition.category == MapNodeCategory::Boss)
            .collect()
    }

    pub fn category_candidates(&self, category: MapNodeCategory) -> Vec<&MapNodeDefinition> {
        self.nodes
            .iter()
            .filter(|definition| definition.category == category)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunMap {
    pub map_template_id: MapTemplateId,
    pub facility_template_version: u32,
    pub nodes: Vec<MapNode>,
    pub start_node_ids: Vec<MapNodeId>,
    pub terminals: Vec<MapTerminalNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapTerminalNode {
    pub node_id: MapNodeId,
    pub resolution: FacilityTerminalResolution,
}

impl RunMap {
    pub fn node(&self, node_id: MapNodeId) -> Option<&MapNode> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    pub fn node_mut(&mut self, node_id: MapNodeId) -> Option<&mut MapNode> {
        self.nodes.iter_mut().find(|node| node.id == node_id)
    }

    pub fn terminal_resolution(&self, node_id: MapNodeId) -> Option<FacilityTerminalResolution> {
        self.terminals
            .iter()
            .find(|terminal| terminal.node_id == node_id)
            .map(|terminal| terminal.resolution)
    }

    pub fn terminal_node_ids(&self) -> Vec<MapNodeId> {
        let mut ids = self
            .terminals
            .iter()
            .map(|terminal| terminal.node_id)
            .collect::<Vec<_>>();
        ids.sort_by_key(|id| id.0.as_u128());
        ids.dedup();
        ids
    }

    pub fn neighbor_node_ids(
        &self,
        template: &crate::game::map::FacilityTemplate,
        node_id: MapNodeId,
    ) -> Vec<MapNodeId> {
        let Some(node) = self.node(node_id) else {
            return Vec::new();
        };
        let neighbor_slots = template.neighbor_slot_ids(&node.slot_id);
        let mut neighbors = self
            .nodes
            .iter()
            .filter(|candidate| neighbor_slots.contains(&candidate.slot_id))
            .map(|candidate| candidate.id)
            .collect::<Vec<_>>();
        neighbors.sort_by_key(|id| id.0.as_u128());
        neighbors.dedup();
        neighbors
    }

    pub fn manifest_satisfied_rules(
        &mut self,
        template: &crate::game::map::FacilityTemplate,
    ) -> Result<Vec<String>, String> {
        if self.map_template_id != template.id || self.facility_template_version != template.version
        {
            return Err(format!(
                "run map template '{}@{}' does not match manifestation template '{}@{}'",
                self.map_template_id.as_str(),
                self.facility_template_version,
                template.id.as_str(),
                template.version
            ));
        }

        let mut manifested_rule_ids = Vec::new();
        for rule in &template.manifestation_rules {
            let prerequisites_satisfied = rule
                .prerequisite_slot_ids
                .iter()
                .map(|slot_id| {
                    self.nodes
                        .iter()
                        .find(|node| node.slot_id == *slot_id)
                        .ok_or_else(|| {
                            format!(
                                "manifestation rule '{}' references missing prerequisite slot '{}'",
                                rule.id,
                                slot_id.as_str()
                            )
                        })
                })
                .collect::<Result<Vec<_>, String>>()?
                .iter()
                .all(|node| node.state == MapNodeState::Completed);
            if !prerequisites_satisfied {
                continue;
            }
            let target = self
                .nodes
                .iter_mut()
                .find(|node| node.slot_id == rule.manifest_slot_id)
                .ok_or_else(|| {
                    format!(
                        "manifestation rule '{}' references missing target slot '{}'",
                        rule.id,
                        rule.manifest_slot_id.as_str()
                    )
                })?;
            if target.visibility != MapNodeVisibility::Concealed {
                continue;
            }
            target.visibility = MapNodeVisibility::Revealed;
            target.state = MapNodeState::Unavailable;
            manifested_rule_ids.push(rule.id.clone());
        }
        manifested_rule_ids.sort();
        Ok(manifested_rule_ids)
    }

    pub fn validate_facility_template_contract(
        &self,
        template: &crate::game::map::FacilityTemplate,
    ) -> Result<(), String> {
        if self.map_template_id != template.id || self.facility_template_version != template.version
        {
            return Err(format!(
                "run map template '{}@{}' does not match facility template '{}@{}'",
                self.map_template_id.as_str(),
                self.facility_template_version,
                template.id.as_str(),
                template.version
            ));
        }

        let mut node_ids = HashSet::new();
        let mut slot_ids = HashSet::new();
        for node in &self.nodes {
            if !node_ids.insert(node.id) {
                return Err(format!("duplicate map node id {:?}", node.id));
            }
            if node.slot_id.as_str().is_empty() {
                return Err(format!("map node {:?} has empty slot_id", node.id));
            }
            if !slot_ids.insert(node.slot_id.as_str()) {
                return Err(format!("duplicate map slot_id '{}'", node.slot_id.as_str()));
            }
            let slot = template
                .slots
                .iter()
                .find(|slot| slot.id == node.slot_id)
                .ok_or_else(|| {
                    format!(
                        "map node {:?} references slot '{}' outside facility template '{}'",
                        node.id,
                        node.slot_id.as_str(),
                        template.id.as_str()
                    )
                })?;
            if !slot.allowed_categories.contains(&node.category) {
                return Err(format!(
                    "map node {:?} category {:?} is not allowed by slot '{}'",
                    node.id,
                    node.category,
                    node.slot_id.as_str()
                ));
            }
            if node.category == MapNodeCategory::Gate && node.payload != MapNodePayload::None {
                return Err(format!("gate node {:?} must not carry an activity payload", node.id));
            }
        }
        for slot in &template.slots {
            if !slot_ids.contains(slot.id.as_str()) {
                return Err(format!(
                    "run map is missing facility template slot '{}'",
                    slot.id.as_str()
                ));
            }
        }

        let mut expected_start_ids = template
            .navigation_start_slot_ids()
            .iter()
            .map(|slot_id| {
                self.nodes
                    .iter()
                    .find(|node| node.slot_id == *slot_id)
                    .map(|node| node.id)
                    .ok_or_else(|| {
                        format!("navigation start slot '{}' has no node", slot_id.as_str())
                    })
            })
            .collect::<Result<Vec<_>, String>>()?;
        expected_start_ids.sort_by_key(|id| id.0.as_u128());
        let mut actual_start_ids = self.start_node_ids.clone();
        actual_start_ids.sort_by_key(|id| id.0.as_u128());
        if actual_start_ids != expected_start_ids {
            return Err("run map start_node_ids do not match facility template starts".to_string());
        }

        let expected_terminals = template
            .slots
            .iter()
            .filter_map(|slot| match slot.role {
                crate::game::map::FacilitySlotRole::Terminal(resolution) => {
                    Some((slot, resolution))
                }
                _ => None,
            })
            .map(|(slot, resolution)| {
                self.nodes
                    .iter()
                    .find(|node| node.slot_id == slot.id)
                    .map(|node| (node.id, resolution))
                    .ok_or_else(|| format!("terminal slot '{}' has no node", slot.id.as_str()))
            })
            .collect::<Result<HashSet<_>, String>>()?;
        let mut terminal_ids = HashSet::new();
        let mut actual_terminals = HashSet::new();
        for terminal in &self.terminals {
            if !terminal_ids.insert(terminal.node_id) {
                return Err(format!(
                    "duplicate map terminal node {:?}",
                    terminal.node_id
                ));
            }
            self.node(terminal.node_id).ok_or_else(|| {
                format!(
                    "map terminal references missing node {:?}",
                    terminal.node_id
                )
            })?;
            actual_terminals.insert((terminal.node_id, terminal.resolution));
        }
        if actual_terminals != expected_terminals {
            return Err(
                "run map typed terminals do not match facility template terminals".to_string(),
            );
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreakdownEntryPreviewDto {
    pub employee_uuid: Uuid,
    pub remaining_cost_before: u32,
    pub remaining_cost_after: u32,
    pub dies_on_entry: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeEntryPreviewDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stress_combat: Option<crate::game::stress_combat::StressCombatEntryView>,
    pub combat_deterrence: Option<crate::game::combat_preview::BattleDeterrenceSnapshot>,
    pub sealed_golden_bough: bool,
    pub regulated_cost: u32,
    pub charged_cost: u32,
    pub deterrence_after: u32,
    pub unrestrained_after_entry: bool,
    pub breakdown: Vec<BreakdownEntryPreviewDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapNodeDto {
    pub id: MapNodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_preview: Option<NodeEntryPreviewDto>,
    pub slot_id: MapSlotId,
    pub kind_id: MapNodeKindId,
    pub category: MapNodeCategory,
    pub state: MapNodeState,
    pub visibility: MapNodeVisibility,
    pub payload: MapNodePayload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub omen: Option<MapNodeOmenOverlayDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty_band: Option<MapNodeDifficultyBandDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_threat: Option<RelativeThreatBandDto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub readiness_warnings: Vec<CombatReadinessWarningDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapNodeDifficultyBandDto {
    Easy,
    Standard,
    Hard,
    Elite,
    NormalBoss,
    FinalBoss,
    BossOmen,
    Revival,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelativeThreatBandDto {
    Low,
    Equal,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatReadinessWarningDto {
    NoDeployableEmployees,
    TraumaBreakdown,
    CriticalPhysicalDamage,
}

impl From<&MapNode> for MapNodeDto {
    fn from(value: &MapNode) -> Self {
        Self {
            id: value.id,
            entry_preview: None,
            slot_id: value.slot_id.clone(),
            kind_id: value.kind_id.clone(),
            category: value.category,
            state: value.state,
            visibility: value.visibility,
            payload: value.payload.clone(),
            omen: value.omen.clone(),
            difficulty_band: value.combat_difficulty.as_ref().map(|difficulty| {
                match difficulty.class {
                    MapNodeCombatDifficultyClass::Ordinary(band) => match band {
                        crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand::Easy => {
                            MapNodeDifficultyBandDto::Easy
                        }
                        crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand::Standard => {
                            MapNodeDifficultyBandDto::Standard
                        }
                        crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand::Hard => {
                            MapNodeDifficultyBandDto::Hard
                        }
                    },
                    MapNodeCombatDifficultyClass::Role(role) => match role {
                        crate::game::data::run_policy_data::CombatNodeDifficultyRole::Elite => {
                            MapNodeDifficultyBandDto::Elite
                        }
                        crate::game::data::run_policy_data::CombatNodeDifficultyRole::NormalBoss => {
                            MapNodeDifficultyBandDto::NormalBoss
                        }
                        crate::game::data::run_policy_data::CombatNodeDifficultyRole::FinalBoss => {
                            MapNodeDifficultyBandDto::FinalBoss
                        }
                        crate::game::data::run_policy_data::CombatNodeDifficultyRole::BossOmen => {
                            MapNodeDifficultyBandDto::BossOmen
                        }
                        crate::game::data::run_policy_data::CombatNodeDifficultyRole::Revival => {
                            MapNodeDifficultyBandDto::Revival
                        }
                    },
                }
            }),
            relative_threat: None,
            readiness_warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapNodeOmenOverlayDto {
    pub present: bool,
    pub source_kind: BossOmenSourceKind,
    pub hint_id: String,
    pub reveal_level: BossOmenRevealLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapViewDto {
    pub map_template_id: MapTemplateId,
    pub facility_template_version: u32,
    pub nodes: Vec<MapNodeDto>,
}

#[cfg(test)]
mod combat_difficulty_schema_tests {
    use super::*;

    #[test]
    fn revival_field_is_required_even_when_none() {
        let legacy = "(power_model_version:2,difficulty_profile_id:\"default_facility\",class:Ordinary(Standard),floor_power_anchor:100,target_power:100)";
        let error = ron::de::from_str::<MapNodeCombatDifficulty>(legacy)
            .expect_err("legacy difficulty without typed monolith stage must reject");

        assert!(error.to_string().contains("revival"));
    }

    #[test]
    fn typed_revival_round_trips_exactly() {
        let difficulty = MapNodeCombatDifficulty {
            power_model_version: 2,
            difficulty_profile_id: "default_facility".to_string(),
            class: MapNodeCombatDifficultyClass::Role(
                crate::game::data::run_policy_data::CombatNodeDifficultyRole::Revival,
            ),
            revival: Some(crate::game::revival::RevivalNodeTuning { stage: RevivalStage::Stage3, stat_multiplier_percent: 200 }),
            floor_power_anchor: 100,
            target_power: 275,
        };

        let encoded = serde_json::to_vec(&difficulty).unwrap();
        let decoded: MapNodeCombatDifficulty = serde_json::from_slice(&encoded).unwrap();

        assert_eq!(decoded, difficulty);
    }
}
