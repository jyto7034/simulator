//! Expedition-owned eggs and already hatched encounters. Shop purchases and
//! combat attempts have separate owners; neither can reset this progression.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::game::{behavior::GameError, map::MapNodeId, reward::RewardEffect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RevivalStage {
    Stage2,
    Stage3,
}

impl RevivalStage {
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Stage2 => 2,
            Self::Stage3 => 3,
        }
    }
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Stage2 => Some(Self::Stage3),
            Self::Stage3 => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevivalStagePolicy {
    /// May reuse the original encounter or name a separately authored revival.
    pub encounter_id: String,
    pub stat_multiplier_percent: u32,
    pub rewards: Vec<RewardEffect>,
    pub equipment_choices: Vec<String>,
}

impl RevivalStagePolicy {
    /// One constructor for the result producer and retained-content reader.
    pub(crate) fn equipment_reward_options(
        &self,
        equipment: &crate::game::data::equipment_data::EquipmentDatabase,
    ) -> Result<Vec<crate::game::reward::RewardOption>, GameError> {
        self.equipment_choices
            .iter()
            .map(|id| {
                let item = equipment.get_by_id(id).ok_or(GameError::InvalidAction)?;
                Ok(crate::game::reward::RewardOption {
                    id: format!("revival_equipment:{id}"),
                    uuid: item.uuid,
                    name: item.name.clone(),
                    description: format!("{} 전용 장비 선택", item.name),
                    icon: String::new(),
                    effects: vec![RewardEffect::GrantEquipment {
                        equipment_id: id.clone(),
                    }],
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossRevivalPolicy {
    pub catalyst_material_id: String,
    pub catalyst_amount: u32,
    pub part_material_id: String,
    pub stage_2: RevivalStagePolicy,
    pub stage_3: RevivalStagePolicy,
}

impl BossRevivalPolicy {
    pub fn stage(&self, stage: RevivalStage) -> &RevivalStagePolicy {
        match stage {
            RevivalStage::Stage2 => &self.stage_2,
            RevivalStage::Stage3 => &self.stage_3,
        }
    }
}

/// Concrete authored tuning fixed together with the node replacement. Its source
/// is checked against the hatched encounter when entering or restoring the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevivalNodeTuning {
    pub stage: RevivalStage,
    pub stat_multiplier_percent: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HatchedEncounter {
    pub reward_source_id: String,
    pub stage: RevivalStage,
    pub resolved: bool,
    pub payment: crate::game::material_economy::MaterialInvestment,
}

impl HatchedEncounter {
    /// The retained source and stage select the same policy during result
    /// production and restoration. A reward UUID is not a source identifier.
    pub(crate) fn stage_policy<'a>(
        &self,
        data: &'a crate::game::data::pve_data::PveEncounterDatabase,
    ) -> Result<&'a RevivalStagePolicy, GameError> {
        data.reward_sources.iter()
            .find(|source| source.reward_source_id == self.reward_source_id)
            .and_then(|source| source.revival.as_ref())
            .map(|policy| policy.stage(self.stage))
            .ok_or(GameError::InvalidAction)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevivalRunState {
    /// One pending opportunity for a source. An egg is not a tradable material.
    pub eggs: BTreeMap<String, RevivalStage>,
    /// Includes resolved nodes so their identity and stage remain queryable.
    pub nodes: BTreeMap<MapNodeId, HatchedEncounter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatchOption {
    pub reward_source_id: String,
    pub abnormality_id: String,
    pub display_name: String,
    pub stage: RevivalStage,
    pub encounter_id: String,
    pub stat_multiplier_percent: u32,
    pub catalyst_material_id: String,
    pub catalyst_required: u32,
    pub catalyst_carried: u32,
    pub catalyst_warehouse: u32,
    pub rewards: Vec<RewardEffect>,
    pub equipment_choices: Vec<String>,
    pub next_egg: Option<RevivalStage>,
    pub can_hatch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevivalRunSnapshot {
    pub eggs: BTreeMap<String, RevivalStage>,
    pub hatched_nodes: BTreeMap<MapNodeId, HatchedEncounterView>,
    pub selected_node_id: Option<MapNodeId>,
    pub hatch_options: Vec<HatchOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatchedEncounterView {
    pub encounter: HatchedEncounter,
    pub encounter_id: String,
    pub stat_multiplier_percent: u32,
    pub rewards: Vec<RewardEffect>,
    pub equipment_choices: Vec<String>,
    pub next_egg_on_victory: Option<RevivalStage>,
}

impl RevivalRunState {
    pub fn grant_first_egg(&mut self, source: &str) -> Result<(), GameError> {
        if self.eggs.contains_key(source)
            || self.nodes.values().any(|n| n.reward_source_id == source)
        {
            return Err(GameError::InvalidAction);
        }
        self.eggs.insert(source.to_owned(), RevivalStage::Stage2);
        Ok(())
    }

    pub fn hatch(
        &mut self,
        node: MapNodeId,
        source: &str,
        stage: RevivalStage,
        payment: crate::game::material_economy::MaterialInvestment,
    ) -> Result<(), GameError> {
        if self.nodes.contains_key(&node) || self.eggs.get(source) != Some(&stage) {
            return Err(GameError::InvalidAction);
        }
        self.eggs.remove(source);
        self.nodes.insert(
            node,
            HatchedEncounter {
                reward_source_id: source.to_owned(),
                stage,
                resolved: false,
                payment,
            },
        );
        Ok(())
    }

    pub fn resolve(
        &mut self,
        node: MapNodeId,
        victory: bool,
    ) -> Result<HatchedEncounter, GameError> {
        let encounter = self
            .nodes
            .get(&node)
            .filter(|n| !n.resolved)
            .ok_or(GameError::InvalidAction)?;
        let next_egg = victory.then(|| encounter.stage.next()).flatten();
        if next_egg.is_some() && self.eggs.contains_key(&encounter.reward_source_id) {
            return Err(GameError::InvalidAction);
        }
        let mut result = encounter.clone();
        result.resolved = true;
        if let Some(stage) = next_egg {
            self.eggs.insert(result.reward_source_id.clone(), stage);
        }
        self.nodes.insert(node, result.clone());
        Ok(result)
    }

    pub fn leave_floor(&mut self) {
        // Eggs travel with the expedition. A hatched encounter stays on its floor.
        self.nodes.clear();
    }
}

crate::authoring::clone_shape::struct_shape!(RevivalStagePolicy {
    encounter_id,
    stat_multiplier_percent,
    rewards,
    equipment_choices
});
crate::authoring::clone_shape::struct_shape!(BossRevivalPolicy {
    catalyst_material_id,
    catalyst_amount,
    part_material_id,
    stage_2,
    stage_3
});
crate::authoring::clone_shape::copy_shape!(RevivalStage);
crate::authoring::clone_shape::copy_shape!(RevivalNodeTuning);
crate::authoring::clone_shape::struct_shape!(HatchedEncounter {
    reward_source_id,
    stage,
    resolved,
    payment
});
crate::authoring::clone_shape::struct_shape!(RevivalRunState { eggs, nodes });

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn u17_eggs_follow_one_two_three_sequence_and_failure_leaves_none() {
        for victory in [false, true] {
            let mut state = RevivalRunState::default();
            let first = MapNodeId(uuid::Uuid::from_u128(1));
            state.grant_first_egg("boss").unwrap();
            assert_eq!(state.eggs.get("boss"), Some(&RevivalStage::Stage2));
            assert!(state
                .hatch(first, "boss", RevivalStage::Stage3, Default::default())
                .is_err());
            state
                .hatch(first, "boss", RevivalStage::Stage2, Default::default())
                .unwrap();
            assert!(state.eggs.is_empty());
            state.resolve(first, victory).unwrap();
            assert!(state.resolve(first, victory).is_err());
            if victory {
                assert_eq!(state.eggs.get("boss"), Some(&RevivalStage::Stage3));
                state.leave_floor();
                assert_eq!(state.eggs.get("boss"), Some(&RevivalStage::Stage3));
                let third = MapNodeId(uuid::Uuid::from_u128(3));
                state
                    .hatch(third, "boss", RevivalStage::Stage3, Default::default())
                    .unwrap();
                state.resolve(third, true).unwrap();
            }
            assert!(state.eggs.is_empty());
        }
    }
}
