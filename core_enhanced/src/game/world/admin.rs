mod catalog;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use super::GameCore;
use crate::game::{
    behavior::GameError,
    data::{
        artifact_data::ArtifactMetadata, consumable_data::ConsumableMetadata,
        equipment_data::EquipmentMetadata,
    },
    employee::EmployeeLifeState,
    enums::RewardMode,
    map::{
        HeadquartersContactOption, MapNode, MapNodeCategory, MapNodeId, MapNodeKindId,
        MapNodePayload, MapNodeState, NodeSession, SupportNodeMode, SupportNodeType,
    },
    resources::{ActiveNodeContent, GameState, HeadquartersContactSessionState},
};

const ADMIN_NODE_NS: u64 = 0x4144_4d49_4e4e_4f44; // "ADMINNOD"

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdminCommand {
    AdminDumpState,
    AdminDumpSelectedEvent,
    AdminDumpInventory,
    AdminDumpRoster,
    AdminDumpAllowedActions,
    AdminDumpGrantCatalog,
    AdminEnterSupport {
        support_type: SupportNodeType,
        #[serde(default)]
        support_mode: Option<SupportNodeMode>,
    },
    AdminEnterMaintenance,
    AdminEnterShop {
        #[serde(default)]
        shop_id: Option<String>,
        #[serde(default)]
        shop_pool_id: Option<String>,
    },
    AdminEnterReward {
        #[serde(default)]
        reward_pool_id: Option<String>,
        #[serde(default)]
        mode: Option<RewardMode>,
        #[serde(default)]
        can_skip: Option<bool>,
    },
    AdminEnterHeadquartersContact {
        #[serde(default)]
        shop_pool_id: Option<String>,
        #[serde(default)]
        candidate_count: Option<usize>,
    },
    AdminGrantEquipment {
        definition_id: String,
        #[serde(default = "default_admin_count")]
        count: u32,
    },
    AdminGrantConsumable {
        definition_id: String,
        #[serde(default = "default_admin_count")]
        count: u32,
    },
    AdminGrantArtifact {
        definition_id: String,
    },
    AdminGrantSkillFragment {
        fragment_id: String,
        #[serde(default = "default_admin_count")]
        count: u32,
    },
    AdminGrantEquipmentMaterial {
        material_id: String,
        amount: u32,
    },
    AdminGrantFragmentDust {
        amount: u32,
    },
    AdminSetEmployeeHp {
        employee_uuid: Uuid,
        current_hp: u32,
        #[serde(default)]
        max_hp: Option<u32>,
    },
    AdminSetEmployeeTrauma {
        employee_uuid: Uuid,
        trauma: u32,
    },
    AdminSetEnkephalin {
        amount: u32,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminCommandOutput {
    pub result_type: &'static str,
    pub payload: Value,
}

fn default_admin_count() -> u32 {
    1
}

impl GameCore {
    pub fn execute_admin_command(
        &mut self,
        command: AdminCommand,
    ) -> Result<AdminCommandOutput, GameError> {
        let output = match command {
            AdminCommand::AdminDumpState => {
                self.admin_output("AdminDumpState", self.get_run_snapshot_json()?)
            }
            AdminCommand::AdminDumpSelectedEvent => self.admin_output(
                "AdminDumpSelectedEvent",
                json!({ "selected_event": self.get_selected_event_snapshot_json()? }),
            ),
            AdminCommand::AdminDumpInventory => self.admin_output(
                "AdminDumpInventory",
                json!({ "inventory": self.get_inventory_snapshot_json()? }),
            ),
            AdminCommand::AdminDumpRoster => self.admin_output(
                "AdminDumpRoster",
                json!({ "roster": self.get_employee_roster_snapshot_json()? }),
            ),
            AdminCommand::AdminDumpAllowedActions => self.admin_output(
                "AdminDumpAllowedActions",
                json!({ "allowed_actions": self.get_allowed_actions() }),
            ),
            AdminCommand::AdminDumpGrantCatalog => self.admin_dump_grant_catalog()?,
            AdminCommand::AdminEnterSupport {
                support_type,
                support_mode,
            } => self.admin_enter_support(
                support_type,
                support_mode.unwrap_or(SupportNodeMode::Known),
            )?,
            AdminCommand::AdminEnterMaintenance => self.admin_enter_maintenance()?,
            AdminCommand::AdminEnterShop {
                shop_id,
                shop_pool_id,
            } => self.admin_enter_shop(shop_id, shop_pool_id)?,
            AdminCommand::AdminEnterReward {
                reward_pool_id,
                mode,
                can_skip,
            } => self.admin_enter_reward(
                reward_pool_id,
                mode.unwrap_or(RewardMode::ClaimAll),
                can_skip.unwrap_or(false),
            )?,
            AdminCommand::AdminEnterHeadquartersContact {
                shop_pool_id,
                candidate_count,
            } => {
                self.admin_enter_headquarters_contact(shop_pool_id, candidate_count.unwrap_or(3))?
            }
            AdminCommand::AdminGrantEquipment {
                definition_id,
                count,
            } => self.admin_grant_equipment(&definition_id, count)?,
            AdminCommand::AdminGrantConsumable {
                definition_id,
                count,
            } => self.admin_grant_consumable(&definition_id, count)?,
            AdminCommand::AdminGrantArtifact { definition_id } => {
                self.admin_grant_artifact(&definition_id)?
            }
            AdminCommand::AdminGrantSkillFragment { fragment_id, count } => {
                self.admin_grant_skill_fragment(&fragment_id, count)?
            }
            AdminCommand::AdminGrantEquipmentMaterial {
                material_id,
                amount,
            } => self.admin_grant_equipment_material(&material_id, amount)?,
            AdminCommand::AdminGrantFragmentDust { amount } => {
                self.admin_grant_fragment_dust(amount)?
            }
            AdminCommand::AdminSetEmployeeHp {
                employee_uuid,
                current_hp,
                max_hp,
            } => self.admin_set_employee_hp(employee_uuid, current_hp, max_hp)?,
            AdminCommand::AdminSetEmployeeTrauma {
                employee_uuid,
                trauma,
            } => self.admin_set_employee_trauma(employee_uuid, trauma)?,
            AdminCommand::AdminSetEnkephalin { amount } => self.admin_set_enkephalin(amount),
        };

        self.sync_roster_order_with_owned_units()?;
        Ok(output)
    }

    fn admin_output(&self, result_type: &'static str, payload: Value) -> AdminCommandOutput {
        AdminCommandOutput {
            result_type,
            payload,
        }
    }

    fn admin_prepare_fixture_node(
        &mut self,
        category: MapNodeCategory,
        kind_id: impl Into<String>,
        payload: MapNodePayload,
    ) -> Result<NodeSession, GameError> {
        let node_id = self.admin_next_fixture_node_id();
        self.admin_prepare_fixture_node_with_id(node_id, category, kind_id, payload)
    }

    fn admin_next_fixture_node_id(&self) -> MapNodeId {
        MapNodeId::new(self.state.uuid_manager.peek(ADMIN_NODE_NS))
    }

    fn admin_prepare_fixture_node_with_id(
        &mut self,
        node_id: MapNodeId,
        category: MapNodeCategory,
        kind_id: impl Into<String>,
        payload: MapNodePayload,
    ) -> Result<NodeSession, GameError> {
        let outgoing = self
            .state
            .run
            .as_ref()
            .ok_or(GameError::MissingResource("RunState"))?
            .map_progression
            .available_node_ids
            .clone();
        let node = MapNode {
            id: node_id,
            depth: 0,
            lane: 0,
            kind_id: MapNodeKindId::new(kind_id),
            category,
            state: MapNodeState::Revealed,
            outgoing,
            payload: payload.clone(),
        };

        let session = NodeSession::from(&node);
        let consumed_node_id = MapNodeId::new(self.state.uuid_manager.next(ADMIN_NODE_NS));
        debug_assert_eq!(consumed_node_id, node_id);
        let run = self.run_state_mut()?;
        run.map.nodes.push(node);
        run.map_progression.current_node_id = Some(node_id);
        run.map_progression.available_node_ids = vec![node_id];
        self.state.node_session = Some(session.clone());
        Ok(session)
    }

    fn admin_enter_support(
        &mut self,
        support_type: SupportNodeType,
        support_mode: SupportNodeMode,
    ) -> Result<AdminCommandOutput, GameError> {
        let choices = match support_mode {
            SupportNodeMode::Known => Vec::new(),
            SupportNodeMode::LimitedChoice | SupportNodeMode::FullChoice => {
                Self::default_support_choices()
            }
        };
        let session = self.admin_prepare_fixture_node(
            MapNodeCategory::Support,
            format!("admin_support_{support_type:?}").to_lowercase(),
            MapNodePayload::Support {
                support_type,
                support_mode,
                choices: choices.clone(),
            },
        )?;

        let mut support = if support_mode == SupportNodeMode::Known {
            crate::game::resources::SupportSessionState::known(session.node_id, support_type)
        } else {
            crate::game::resources::SupportSessionState::choice(
                session.node_id,
                support_mode,
                choices,
            )
        };
        self.refresh_support_target_candidates(&mut support)?;
        self.state.active_node_content = Some(ActiveNodeContent::Support(support));
        self.transition_to(GameState::InNode {
            node_id: session.node_id,
            kind_id: session.kind_id.clone(),
            category: MapNodeCategory::Support,
        })?;

        Ok(self.admin_output(
            "AdminEnteredSupport",
            json!({
                "node_id": session.node_id,
                "support_type": support_type,
                "support_mode": support_mode,
            }),
        ))
    }

    fn admin_enter_maintenance(&mut self) -> Result<AdminCommandOutput, GameError> {
        let session = self.admin_prepare_fixture_node(
            MapNodeCategory::Maintenance,
            "admin_maintenance",
            MapNodePayload::Maintenance,
        )?;
        let maintenance = crate::game::resources::MaintenanceSessionState::new(session.node_id);
        self.state.active_node_content = Some(ActiveNodeContent::Maintenance(maintenance));
        self.transition_to(GameState::InNode {
            node_id: session.node_id,
            kind_id: session.kind_id.clone(),
            category: MapNodeCategory::Maintenance,
        })?;

        Ok(self.admin_output(
            "AdminEnteredMaintenance",
            json!({ "node_id": session.node_id }),
        ))
    }

    fn admin_enter_shop(
        &mut self,
        shop_id: Option<String>,
        shop_pool_id: Option<String>,
    ) -> Result<AdminCommandOutput, GameError> {
        let payload = MapNodePayload::Shop {
            shop_id,
            shop_pool_id,
        };
        let node_id = self.admin_next_fixture_node_id();
        let Some(shop) = self.resolve_map_shop(node_id, &payload)? else {
            return Err(GameError::InvalidStaticData(
                "admin shop command could not resolve a shop".to_string(),
            ));
        };
        let session = self.admin_prepare_fixture_node_with_id(
            node_id,
            MapNodeCategory::Shop,
            "admin_shop",
            payload,
        )?;
        let shop_uuid = shop.uuid;
        self.state.active_node_content = Some(ActiveNodeContent::Shop(shop));
        self.transition_to(GameState::InShop { shop_uuid })?;

        Ok(self.admin_output(
            "AdminEnteredShop",
            json!({
                "node_id": session.node_id,
                "shop_uuid": shop_uuid,
            }),
        ))
    }

    fn admin_enter_reward(
        &mut self,
        reward_pool_id: Option<String>,
        mode: RewardMode,
        can_skip: bool,
    ) -> Result<AdminCommandOutput, GameError> {
        let payload = MapNodePayload::Reward { reward_pool_id };
        let node_id = self.admin_next_fixture_node_id();
        let Some(mut reward) = self.resolve_map_reward(node_id, &payload)? else {
            return Err(GameError::InvalidStaticData(
                "admin reward command could not resolve a reward".to_string(),
            ));
        };
        let session = self.admin_prepare_fixture_node_with_id(
            node_id,
            MapNodeCategory::Reward,
            "admin_reward",
            payload,
        )?;
        reward.mode = mode;
        reward.can_skip = can_skip;
        let reward_uuid = reward.stage_uuid;
        self.state.active_node_content = Some(ActiveNodeContent::Reward(reward));
        self.transition_to(GameState::InReward { reward_uuid })?;

        Ok(self.admin_output(
            "AdminEnteredReward",
            json!({
                "node_id": session.node_id,
                "reward_uuid": reward_uuid,
                "mode": mode,
                "can_skip": can_skip,
            }),
        ))
    }

    fn admin_enter_headquarters_contact(
        &mut self,
        shop_pool_id: Option<String>,
        candidate_count: usize,
    ) -> Result<AdminCommandOutput, GameError> {
        let session = self.admin_prepare_fixture_node(
            MapNodeCategory::HeadquartersContact,
            "admin_headquarters_contact",
            MapNodePayload::HeadquartersContact {
                shop_pool_id: shop_pool_id.clone(),
                candidate_count,
            },
        )?;
        let headquarters = HeadquartersContactSessionState {
            node_id: session.node_id,
            options: vec![
                HeadquartersContactOption::RecruitEmployee,
                HeadquartersContactOption::RequestEmergencySupplies,
                HeadquartersContactOption::OpenHeadquartersShop,
            ],
            recruitment_candidates: self
                .recruitment_candidates_for_node(session.node_id, candidate_count),
            shop_pool_id,
        };
        self.state.active_node_content = Some(ActiveNodeContent::HeadquartersContact(headquarters));
        self.transition_to(GameState::InNode {
            node_id: session.node_id,
            kind_id: session.kind_id.clone(),
            category: MapNodeCategory::HeadquartersContact,
        })?;

        Ok(self.admin_output(
            "AdminEnteredHeadquartersContact",
            json!({ "node_id": session.node_id }),
        ))
    }

    fn admin_grant_equipment(
        &mut self,
        definition_id: &str,
        count: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        let metadata = self
            .game_data
            .equipment_data
            .get_by_id(definition_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "admin grant references missing equipment '{definition_id}'"
                ))
            })?
            .clone();
        let item_uuids = self.admin_add_owned_equipment(metadata, count)?;
        Ok(self.admin_output(
            "AdminGrantedEquipment",
            json!({ "definition_id": definition_id, "item_uuids": item_uuids }),
        ))
    }

    fn admin_add_owned_equipment(
        &mut self,
        metadata: EquipmentMetadata,
        count: u32,
    ) -> Result<Vec<Uuid>, GameError> {
        if count == 0 {
            return Err(GameError::InvalidAction);
        }
        let mut item_uuids = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let owned_uuid = self.state.uuid_manager.next_owned_equipment();
            self.state.inventory.add_item_owned(
                owned_uuid,
                crate::game::data::Item::Equipment(Arc::new(metadata.clone())),
            )?;
            item_uuids.push(owned_uuid);
        }
        Ok(item_uuids)
    }

    fn admin_grant_consumable(
        &mut self,
        definition_id: &str,
        count: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        let metadata = self
            .game_data
            .consumable_data
            .get_by_id(definition_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "admin grant references missing consumable '{definition_id}'"
                ))
            })?
            .clone();
        let item_uuids = self.admin_add_owned_consumable(metadata, count)?;
        Ok(self.admin_output(
            "AdminGrantedConsumable",
            json!({ "definition_id": definition_id, "item_uuids": item_uuids }),
        ))
    }

    fn admin_add_owned_consumable(
        &mut self,
        metadata: ConsumableMetadata,
        count: u32,
    ) -> Result<Vec<Uuid>, GameError> {
        if count == 0 {
            return Err(GameError::InvalidAction);
        }
        let mut item_uuids = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let owned_uuid = self.state.uuid_manager.next_owned_consumable();
            self.state.inventory.add_item_owned(
                owned_uuid,
                crate::game::data::Item::Consumable(Arc::new(metadata.clone())),
            )?;
            item_uuids.push(owned_uuid);
        }
        Ok(item_uuids)
    }

    fn admin_grant_artifact(
        &mut self,
        definition_id: &str,
    ) -> Result<AdminCommandOutput, GameError> {
        let metadata = self
            .game_data
            .artifact_data
            .get_by_id(definition_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "admin grant references missing artifact '{definition_id}'"
                ))
            })?
            .clone();
        let artifact_uuid = metadata.uuid;
        self.state.inventory.add_item_owned(
            artifact_uuid,
            crate::game::data::Item::Artifact(Arc::new(ArtifactMetadata::from(metadata))),
        )?;
        Ok(self.admin_output(
            "AdminGrantedArtifact",
            json!({ "definition_id": definition_id, "artifact_uuid": artifact_uuid }),
        ))
    }

    fn admin_grant_skill_fragment(
        &mut self,
        fragment_id: &str,
        count: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        if count == 0 {
            return Err(GameError::InvalidAction);
        }
        let fragment_id =
            crate::game::data::skill_fragment_data::SkillFragmentId::from(fragment_id);
        if self
            .game_data
            .skill_fragment_data
            .get_by_id(&fragment_id)
            .is_none()
        {
            return Err(GameError::InvalidStaticData(format!(
                "admin grant references missing skill fragment '{fragment_id}'"
            )));
        }
        for _ in 0..count {
            self.state
                .skill_fragments
                .add_id_with_policy(&fragment_id, &self.state.skill_fragment_policy)?;
        }
        Ok(self.admin_output(
            "AdminGrantedSkillFragment",
            json!({
                "fragment_id": fragment_id,
                "count": self.state.skill_fragments.count(&fragment_id),
            }),
        ))
    }

    fn admin_grant_equipment_material(
        &mut self,
        material_id: &str,
        amount: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        if amount == 0 {
            return Err(GameError::InvalidAction);
        }
        if self
            .game_data
            .equipment_data
            .get_material_by_id(material_id)
            .is_none()
        {
            return Err(GameError::InvalidStaticData(format!(
                "admin grant references missing equipment material '{material_id}'"
            )));
        }
        let amount = self
            .state
            .inventory
            .equipment_materials
            .add(material_id, amount)?;
        Ok(self.admin_output(
            "AdminGrantedEquipmentMaterial",
            json!({ "material_id": material_id, "amount": amount }),
        ))
    }

    fn admin_grant_fragment_dust(&mut self, amount: u32) -> Result<AdminCommandOutput, GameError> {
        if amount == 0 {
            return Err(GameError::InvalidAction);
        }
        let amount = self.state.skill_fragments.add_fragment_dust(amount)?;
        Ok(self.admin_output("AdminGrantedFragmentDust", json!({ "amount": amount })))
    }

    fn admin_set_employee_hp(
        &mut self,
        employee_uuid: Uuid,
        current_hp: u32,
        max_hp: Option<u32>,
    ) -> Result<AdminCommandOutput, GameError> {
        let (current_hp, max_hp) = {
            let employee = self
                .state
                .roster
                .get_mut(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            if employee.life_state != EmployeeLifeState::Alive {
                return Err(GameError::InvalidAction);
            }
            if let Some(max_hp) = max_hp {
                if max_hp == 0 {
                    return Err(GameError::InvalidAction);
                }
                employee.health.max_hp = max_hp;
            }
            employee.health.current_hp = current_hp.min(employee.health.max_hp);
            (employee.health.current_hp, employee.health.max_hp)
        };
        Ok(self.admin_output(
            "AdminSetEmployeeHp",
            json!({
                "employee_uuid": employee_uuid,
                "current_hp": current_hp,
                "max_hp": max_hp,
            }),
        ))
    }

    fn admin_set_employee_trauma(
        &mut self,
        employee_uuid: Uuid,
        trauma: u32,
    ) -> Result<AdminCommandOutput, GameError> {
        let trauma = {
            let employee = self
                .state
                .roster
                .get_mut(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            if employee.life_state != EmployeeLifeState::Alive {
                return Err(GameError::InvalidAction);
            }
            employee.trauma = trauma;
            employee.trauma
        };
        Ok(self.admin_output(
            "AdminSetEmployeeTrauma",
            json!({ "employee_uuid": employee_uuid, "trauma": trauma }),
        ))
    }

    fn admin_set_enkephalin(&mut self, amount: u32) -> AdminCommandOutput {
        self.state.enkephalin.amount = amount;
        self.admin_output("AdminSetEnkephalin", json!({ "amount": amount }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::world::RunState;
    use crate::game::{
        behavior::ActionKind,
        data::{
            artifact_data::{ArtifactDatabase, ArtifactMetadata},
            consumable_data::{
                ConsumableDatabase, ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata,
                ConsumableTargetPolicy, ConsumableTier,
            },
            equipment_data::{
                EquipmentDatabase, EquipmentMaterialMetadata, EquipmentMaterialType,
                EquipmentMetadata, EquipmentType,
            },
            reward_data::{RewardDatabase, RewardMetadata, RewardPoolMetadata},
            shop_data::{ShopDatabase, ShopMetadata, ShopPoolMetadata, ShopType},
            skill_fragment_data::{
                SkillFragmentAcquisitionSource, SkillFragmentDatabase, SkillFragmentEffectDef,
                SkillFragmentId, SkillFragmentMetadata, SkillFragmentRarity,
            },
            GameDataBase, GameDataBuilder,
        },
        enums::RiskLevel,
        map::{MapProgression, RunMap, RunProgression},
        reward::RewardEffect,
        stats::TriggeredEffects,
    };
    use std::sync::Arc;

    fn core_with_run(game_data: Arc<GameDataBase>) -> GameCore {
        let mut core = GameCore::new(game_data, 123);
        let boss_id = MapNodeId::new(Uuid::from_u128(0xB055));
        let map = RunMap {
            nodes: vec![MapNode {
                id: boss_id,
                depth: 1,
                lane: 0,
                kind_id: MapNodeKindId::new("boss"),
                category: MapNodeCategory::Boss,
                state: MapNodeState::Hidden,
                outgoing: vec![],
                payload: MapNodePayload::Encounter { encounter_id: None },
            }],
            start_node_ids: vec![],
            boss_node_id: boss_id,
        };
        let run_progression = RunProgression::new(123, 1);
        core.state.run = Some(RunState::new(
            map,
            MapProgression::default(),
            run_progression,
        ));
        core.transition_to(GameState::ViewingMap).unwrap();
        core
    }

    fn empty_core_with_run() -> GameCore {
        core_with_run(GameDataBuilder::empty().build_arc())
    }

    fn core_with_admin_map_content() -> GameCore {
        let game_data = GameDataBuilder::empty()
            .with_shops(ShopDatabase::new_with_pools(
                vec![ShopMetadata {
                    id: "admin_shop".to_string(),
                    name: "Admin Shop".to_string(),
                    uuid: Uuid::from_u128(10_001),
                    shop_type: ShopType::Shop,
                    can_reroll: false,
                    visible_items: vec![],
                    hidden_items: vec![],
                }],
                vec![ShopPoolMetadata {
                    id: "admin_shops".to_string(),
                    shop_ids: vec!["admin_shop".to_string()],
                }],
            ))
            .with_rewards(RewardDatabase::new_with_pools(
                vec![RewardMetadata {
                    id: "admin_reward".to_string(),
                    uuid: Uuid::from_u128(20_001),
                    name: "Admin Reward".to_string(),
                    description: "Admin reward".to_string(),
                    icon: "test".to_string(),
                    tags: Vec::new(),
                    effects: vec![RewardEffect::GrantEnkephalin { amount: 7 }],
                }],
                vec![RewardPoolMetadata {
                    id: "admin_rewards".to_string(),
                    reward_ids: vec!["admin_reward".to_string()],
                }],
            ))
            .build_arc();
        core_with_run(game_data)
    }

    fn core_with_admin_grant_catalog_data() -> GameCore {
        let equipment = EquipmentMetadata {
            id: "fixture_armor".to_string(),
            uuid: Uuid::from_u128(30_001),
            name: "Fixture Armor".to_string(),
            equipment_type: EquipmentType::Armor,
            rarity: RiskLevel::TETH,
            price: 10,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: TriggeredEffects::default(),
            ability_activations: Vec::new(),
            weapon_profile: None,
        };
        let material = EquipmentMaterialMetadata {
            id: "fixture_dust".to_string(),
            uuid: Uuid::from_u128(30_002),
            name: "Fixture Dust".to_string(),
            description: "fixture material".to_string(),
            material_type: EquipmentMaterialType::Generic,
            rarity: RiskLevel::ZAYIN,
            equipment_type: None,
        };
        let consumable = ConsumableMetadata {
            id: "fixture_ration".to_string(),
            uuid: Uuid::from_u128(40_001),
            name: "Fixture Ration".to_string(),
            description: "fixture consumable".to_string(),
            tier: ConsumableTier::Common,
            rarity: RiskLevel::ZAYIN,
            price: 5,
            target_policy: ConsumableTargetPolicy::SingleEmployee,
            duration_policy: ConsumableDurationPolicy::NextCombatNode,
            effect: ConsumableEffect::DeathPrevent,
            live_pool: true,
        };
        let artifact = ArtifactMetadata {
            id: "fixture_artifact".to_string(),
            uuid: Uuid::from_u128(50_001),
            name: "Fixture Artifact".to_string(),
            description: "fixture artifact".to_string(),
            rarity: RiskLevel::HE,
            price: 20,
            triggered_effects: TriggeredEffects::default(),
            ability_activations: Vec::new(),
        };
        let fragment = SkillFragmentMetadata {
            id: SkillFragmentId::from("fixture_fragment"),
            uuid: Uuid::from_u128(60_001),
            name: "Fixture Fragment".to_string(),
            description: "fixture fragment".to_string(),
            rarity: SkillFragmentRarity::Common,
            origin: None,
            sources: vec![SkillFragmentAcquisitionSource::RareReward],
            dependencies: Vec::new(),
            compatibility: Default::default(),
            effect: SkillFragmentEffectDef::BasicAttackModifier {
                attack_bonus: 1,
                attack_interval_ms_reduction: 0,
            },
        };
        let game_data = GameDataBuilder::empty()
            .with_equipment_data(Arc::new(EquipmentDatabase::with_materials_and_recipes(
                vec![equipment],
                vec![material],
                Vec::new(),
            )))
            .with_consumable_data(Arc::new(ConsumableDatabase::new(vec![consumable])))
            .with_artifact_data(Arc::new(ArtifactDatabase::new(vec![artifact])))
            .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(vec![fragment]))
            .build_arc();
        core_with_run(game_data)
    }

    fn fixture_state_fingerprint(
        core: &GameCore,
    ) -> (
        usize,
        Option<MapNodeId>,
        Vec<MapNodeId>,
        Option<NodeSession>,
        Uuid,
    ) {
        let run = core.state.run.as_ref().unwrap();
        (
            run.map.nodes.len(),
            run.map_progression.current_node_id,
            run.map_progression.available_node_ids.clone(),
            core.state.node_session.clone(),
            core.state.uuid_manager.peek(ADMIN_NODE_NS),
        )
    }

    #[test]
    fn admin_enter_support_requires_an_active_run() {
        let mut core = GameCore::new(GameDataBuilder::empty().build_arc(), 123);

        let err = core
            .execute_admin_command(AdminCommand::AdminEnterMaintenance)
            .unwrap_err();

        assert!(matches!(err, GameError::MissingResource("RunState")));
    }

    #[test]
    fn admin_enter_maintenance_creates_valid_fixture_state() {
        let mut core = empty_core_with_run();

        let output = core
            .execute_admin_command(AdminCommand::AdminEnterMaintenance)
            .unwrap();

        assert_eq!(output.result_type, "AdminEnteredMaintenance");
        assert!(matches!(core.get_state(), GameState::InNode { .. }));
        assert!(core
            .get_allowed_actions()
            .contains(&ActionKind::DismantleEquipment));
        let selected_event = core.get_selected_event_snapshot_json().unwrap().unwrap();
        assert_eq!(selected_event["type"], "maintenance");
        assert!(selected_event["maintenance_options"].is_object());
    }

    #[test]
    fn failed_admin_shop_entry_does_not_mutate_fixture_state() {
        let mut core = empty_core_with_run();
        let before = fixture_state_fingerprint(&core);

        let err = core
            .execute_admin_command(AdminCommand::AdminEnterShop {
                shop_id: Some("missing_shop".to_string()),
                shop_pool_id: None,
            })
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidStaticData(_)));
        assert_eq!(fixture_state_fingerprint(&core), before);
    }

    #[test]
    fn failed_admin_reward_entry_does_not_mutate_fixture_state() {
        let mut core = empty_core_with_run();
        let before = fixture_state_fingerprint(&core);

        let err = core
            .execute_admin_command(AdminCommand::AdminEnterReward {
                reward_pool_id: Some("missing_rewards".to_string()),
                mode: Some(RewardMode::ClaimAll),
                can_skip: None,
            })
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidStaticData(_)));
        assert_eq!(fixture_state_fingerprint(&core), before);
    }

    #[test]
    fn admin_enter_shop_creates_shop_fixture_snapshot() {
        let mut core = core_with_admin_map_content();

        let output = core
            .execute_admin_command(AdminCommand::AdminEnterShop {
                shop_id: Some("admin_shop".to_string()),
                shop_pool_id: None,
            })
            .unwrap();

        assert_eq!(output.result_type, "AdminEnteredShop");
        assert!(matches!(core.get_state(), GameState::InShop { .. }));
        let selected_event = core.get_selected_event_snapshot_json().unwrap().unwrap();
        assert_eq!(selected_event["type"], "shop");
        assert_eq!(selected_event["id"], "admin_shop");
    }

    #[test]
    fn admin_enter_reward_applies_requested_mode_to_snapshot() {
        let mut core = core_with_admin_map_content();

        let output = core
            .execute_admin_command(AdminCommand::AdminEnterReward {
                reward_pool_id: Some("admin_rewards".to_string()),
                mode: Some(RewardMode::ChooseOne),
                can_skip: Some(true),
            })
            .unwrap();

        assert_eq!(output.result_type, "AdminEnteredReward");
        assert!(matches!(core.get_state(), GameState::InReward { .. }));
        assert_eq!(output.payload["mode"], "ChooseOne");
        let selected_event = core.get_selected_event_snapshot_json().unwrap().unwrap();
        assert_eq!(selected_event["type"], "reward");
        assert_eq!(selected_event["mode"], "ChooseOne");
        assert_eq!(selected_event["can_skip"], true);
    }

    #[test]
    fn admin_enter_headquarters_contact_creates_fixture_snapshot() {
        let mut core = core_with_admin_map_content();

        let output = core
            .execute_admin_command(AdminCommand::AdminEnterHeadquartersContact {
                shop_pool_id: Some("admin_shops".to_string()),
                candidate_count: Some(2),
            })
            .unwrap();

        assert_eq!(output.result_type, "AdminEnteredHeadquartersContact");
        assert!(matches!(core.get_state(), GameState::InNode { .. }));
        let selected_event = core.get_selected_event_snapshot_json().unwrap().unwrap();
        assert_eq!(selected_event["type"], "headquarters_contact");
    }

    #[test]
    fn admin_grant_missing_equipment_is_rejected_without_inventory_change() {
        let mut core = empty_core_with_run();

        let err = core
            .execute_admin_command(AdminCommand::AdminGrantEquipment {
                definition_id: "missing_equipment".to_string(),
                count: 1,
            })
            .unwrap_err();

        assert!(matches!(err, GameError::InvalidStaticData(_)));
        assert_eq!(core.state.inventory.equipments.len(), 0);
    }

    #[test]
    fn admin_dump_grant_catalog_lists_grantable_live_data() {
        let mut core = core_with_admin_grant_catalog_data();

        let output = core
            .execute_admin_command(AdminCommand::AdminDumpGrantCatalog)
            .unwrap();

        assert_eq!(output.result_type, "AdminGrantCatalog");
        assert_eq!(output.payload["schema_version"], 1);
        assert_eq!(output.payload["equipment"][0]["id"], "fixture_armor");
        assert_eq!(output.payload["equipment"][0]["equipment_type"], "Armor");
        assert_eq!(
            output.payload["equipment"][0]["grant_command"],
            "admin_grant_equipment"
        );
        assert_eq!(output.payload["equipment"][0]["amount_mode"], "count");
        assert_eq!(output.payload["consumables"][0]["id"], "fixture_ration");
        assert_eq!(
            output.payload["consumables"][0]["grant_command"],
            "admin_grant_consumable"
        );
        assert_eq!(output.payload["artifacts"][0]["id"], "fixture_artifact");
        assert_eq!(output.payload["artifacts"][0]["amount_mode"], "single");
        assert!(output.payload["skill_fragments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["id"] == "fixture_fragment"
                && entry["grant_command"] == "admin_grant_skill_fragment"));
        assert_eq!(
            output.payload["equipment_materials"][0]["grant_command"],
            "admin_grant_equipment_material"
        );
        assert!(output.payload["resources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["id"] == "fragment_dust"
                && entry["grant_command"] == "admin_grant_fragment_dust"
                && entry["amount_mode"] == "amount"));
        assert!(output.payload["resources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["id"] == "enkephalin"
                && entry["grant_command"] == "admin_set_enkephalin"
                && entry["amount_mode"] == "set"));
    }

    #[test]
    fn admin_zero_amount_grants_are_rejected() {
        let mut core = empty_core_with_run();

        let material_err = core
            .execute_admin_command(AdminCommand::AdminGrantEquipmentMaterial {
                material_id: "equipment_dust".to_string(),
                amount: 0,
            })
            .unwrap_err();
        assert!(matches!(material_err, GameError::InvalidAction));

        let dust_err = core
            .execute_admin_command(AdminCommand::AdminGrantFragmentDust { amount: 0 })
            .unwrap_err();
        assert!(matches!(dust_err, GameError::InvalidAction));
    }

    #[test]
    fn admin_set_enkephalin_updates_resource_snapshot() {
        let mut core = empty_core_with_run();

        let output = core
            .execute_admin_command(AdminCommand::AdminSetEnkephalin { amount: 777 })
            .unwrap();

        assert_eq!(output.result_type, "AdminSetEnkephalin");
        assert_eq!(core.get_enkephalin(), 777);
        let snapshot = core.get_run_snapshot_json().unwrap();
        assert_eq!(snapshot["resources"]["enkephalin"], 777);
    }
}
